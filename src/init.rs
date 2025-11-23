use opentelemetry::sdk::Resource;
use opentelemetry::KeyValue;
use opentelemetry::global;
use tracing_subscriber::{layer::SubscriberExt, Registry};
use tracing_subscriber::util::SubscriberInitExt;

use crate::config::{Config, ExporterType};
use crate::exporters;

pub fn init_telemetry(config: Config) -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    // 1. Create Resource
    let mut resource_attributes = vec![
        KeyValue::new("service.name", config.service_name.clone()),
    ];

    if let Some(app_version) = &config.mobile_metadata.app_version {
        resource_attributes.push(KeyValue::new("app.version", app_version.clone()));
    }
    if let Some(device_os) = &config.mobile_metadata.device_os {
        resource_attributes.push(KeyValue::new("device.os", device_os.clone()));
    }
    if let Some(device_id) = &config.mobile_metadata.device_id {
        resource_attributes.push(KeyValue::new("device.id", device_id.clone()));
    }

    let resource = Resource::new(resource_attributes);

    // 2. Initialize Tracer
    if config.enable_tracing {
        let tracer = match config.exporter_type {
            ExporterType::Console => exporters::init_console_tracer(resource.clone()),
            ExporterType::Otlp => exporters::init_otlp_tracer(&config, resource.clone())?,
        };

        // 3. Configure tracing-subscriber
        let telemetry_layer = tracing_opentelemetry::layer().with_tracer(tracer);
        let subscriber = Registry::default()
            .with(tracing_subscriber::EnvFilter::from_default_env())
            .with(telemetry_layer);
        
        // If console exporter is chosen, we might also want to see logs in stdout? 
        // For now, we just set the global subscriber.
        // Note: tracing_subscriber::fmt::layer() could be added if users want standard logs too.
        
        subscriber.init();
    }

    // 4. Initialize MeterProvider
    if config.enable_metrics {
        let meter_provider = match config.exporter_type {
            ExporterType::Console => exporters::init_console_meter_provider(resource),
            ExporterType::Otlp => exporters::init_otlp_meter_provider(&config, resource)?,
        };
        
        global::set_meter_provider(meter_provider);
    }

    Ok(())
}

pub fn shutdown_telemetry() {
    global::shutdown_tracer_provider();
    // MeterProvider shutdown is not yet fully standardized in the same global way in all versions, 
    // but dropping the provider usually flushes. 
    // For OTLP, global::shutdown_tracer_provider() handles the tracer.
}
