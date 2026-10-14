//! Installing and tearing down the global telemetry pipeline.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, PoisonError};

use opentelemetry::global;
use opentelemetry::KeyValue;
use opentelemetry_sdk::metrics::MeterProvider;
use opentelemetry_sdk::Resource;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Registry};

use crate::config::{Config, ExporterType};
use crate::exporters;

type BoxError = Box<dyn std::error::Error + Send + Sync + 'static>;

/// Filter used when `RUST_LOG` is unset or unparsable. [`with_span`](crate::with_span)
/// creates `INFO` spans, so anything stricter would silently drop them.
const DEFAULT_FILTER: &str = "info";

/// `init_telemetry` installs process-wide state (a global `tracing` subscriber,
/// global OpenTelemetry providers), so it may only succeed once per process.
static INITIALIZED: AtomicBool = AtomicBool::new(false);

/// The global meter-provider API cannot flush or shut down, so a handle is kept
/// here for [`shutdown_telemetry`].
static METER_PROVIDER: Mutex<Option<MeterProvider>> = Mutex::new(None);

/// Installs tracing and metrics export according to `config`.
///
/// This sets a global `tracing` subscriber (spans are filtered by `RUST_LOG`,
/// defaulting to `info`) and the global OpenTelemetry tracer and meter providers.
/// It works from synchronous code and from any Tokio runtime: background export
/// runs on a small runtime owned by this crate.
///
/// # Errors
///
/// Returns an error instead of panicking when:
/// - it has already succeeded once in this process,
/// - another global `tracing` subscriber or `log` logger is already installed,
/// - the OTLP endpoint is not a valid URI,
/// - the background export runtime cannot be started.
pub fn init_telemetry(config: Config) -> Result<(), BoxError> {
    if INITIALIZED
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_err()
    {
        return Err(
            "rust_mobile_telemetry: init_telemetry has already been called in this process".into(),
        );
    }

    let result = install(&config);
    if result.is_err() {
        INITIALIZED.store(false, Ordering::SeqCst);
    }
    result
}

/// Flushes buffered spans and metrics and shuts the exporters down.
///
/// Call it once, right before the process exits. It blocks until the final
/// export has finished (or the exporter's timeout fires). It is safe to call
/// from synchronous code and from inside any Tokio runtime, including
/// `current_thread` runtimes such as the default `#[tokio::test]`.
pub fn shutdown_telemetry() {
    global::shutdown_tracer_provider();

    let meter_provider = METER_PROVIDER
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .take();
    if let Some(meter_provider) = meter_provider {
        if let Err(error) = meter_provider.force_flush() {
            global::handle_error(error);
        }
        // opentelemetry_sdk 0.21's PeriodicReader marks itself shut down *before*
        // its final collection, so `shutdown()` exports nothing and always reports
        // "reader is shut down". The flush above already delivered the data; this
        // call only stops the background task and exporter, so its error is noise.
        let _ = meter_provider.shutdown();
    }
}

fn install(config: &Config) -> Result<(), BoxError> {
    // Start the export runtime up front so a failure surfaces as an error here
    // rather than as a panic inside the exporter constructors.
    exporters::export_runtime()?;
    let resource = build_resource(config);

    if config.enable_tracing {
        let tracer = match config.exporter_type {
            ExporterType::Console => exporters::init_console_tracer(resource.clone()),
            ExporterType::Otlp => exporters::init_otlp_tracer(config, resource.clone())?,
        };

        Registry::default()
            .with(env_filter())
            .with(tracing_opentelemetry::layer().with_tracer(tracer))
            .try_init()
            .map_err(|error| {
                format!(
                    "rust_mobile_telemetry: could not install the tracing subscriber \
                     (is another global subscriber or `log` logger already set?): {error}"
                )
            })?;
    }

    if config.enable_metrics {
        let meter_provider = match config.exporter_type {
            ExporterType::Console => exporters::init_console_meter_provider(resource),
            ExporterType::Otlp => exporters::init_otlp_meter_provider(config, resource)?,
        };
        global::set_meter_provider(meter_provider.clone());
        *METER_PROVIDER
            .lock()
            .unwrap_or_else(PoisonError::into_inner) = Some(meter_provider);
    }

    Ok(())
}

/// Builds the OpenTelemetry resource. Only metadata the caller supplied is added.
fn build_resource(config: &Config) -> Resource {
    let metadata = &config.mobile_metadata;
    let mut attributes = vec![KeyValue::new("service.name", config.service_name.clone())];

    let optional_attributes = [
        ("app.version", &metadata.app_version),
        ("device.os", &metadata.device_os),
        ("device.id", &metadata.device_id),
    ];
    for (key, value) in optional_attributes {
        if let Some(value) = value {
            attributes.push(KeyValue::new(key, value.clone()));
        }
    }

    Resource::new(attributes)
}

fn env_filter() -> EnvFilter {
    EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(DEFAULT_FILTER))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{ConfigBuilder, MobileMetadata};
    use opentelemetry::Key;
    use tracing_subscriber::filter::LevelFilter;
    use tracing_subscriber::Layer;

    #[test]
    fn default_filter_keeps_info_spans() {
        let filter = EnvFilter::new(DEFAULT_FILTER);
        assert_eq!(
            Layer::<Registry>::max_level_hint(&filter),
            Some(LevelFilter::INFO)
        );
    }

    #[test]
    fn resource_contains_only_supplied_metadata() {
        let config = ConfigBuilder::new()
            .service_name("svc")
            .mobile_metadata(MobileMetadata {
                app_version: Some("1.2.3".to_string()),
                device_os: None,
                device_id: None,
            })
            .build();

        let resource = build_resource(&config);

        assert_eq!(
            resource
                .get(Key::new("service.name"))
                .map(|v| v.to_string()),
            Some("svc".to_string())
        );
        assert_eq!(
            resource.get(Key::new("app.version")).map(|v| v.to_string()),
            Some("1.2.3".to_string())
        );
        assert!(resource.get(Key::new("device.os")).is_none());
        assert!(resource.get(Key::new("device.id")).is_none());
    }
}
