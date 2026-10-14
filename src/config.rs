//! Telemetry configuration and its builder.

use std::env;

/// Settings consumed by [`init_telemetry`](crate::init_telemetry).
///
/// Build it with [`ConfigBuilder`]; [`Config::default`] gives the same values as
/// `ConfigBuilder::new().build()`.
#[derive(Debug, Clone)]
pub struct Config {
    /// Exported as the `service.name` resource attribute. Default: `"unknown_service"`.
    pub service_name: String,
    /// OTLP/gRPC collector URL. `None` means `http://localhost:4317`.
    pub otlp_endpoint: Option<String>,
    /// Whether to install the metrics pipeline. Default: `true`.
    pub enable_metrics: bool,
    /// Whether to install the tracing pipeline and global subscriber. Default: `true`.
    pub enable_tracing: bool,
    /// Optional app/device attributes added to every exported span and metric.
    pub mobile_metadata: MobileMetadata,
    /// Where telemetry is sent. Default: [`ExporterType::Console`].
    pub exporter_type: ExporterType,
}

/// Optional client metadata, exported as resource attributes on every span and
/// metric of this process.
///
/// Fields left as `None` are not exported. `device_id` is a persistent
/// identifier and may count as personal data; prefer a pseudonymous value.
#[derive(Debug, Clone, Default)]
pub struct MobileMetadata {
    /// Exported as `app.version`.
    pub app_version: Option<String>,
    /// Exported as `device.os`.
    pub device_os: Option<String>,
    /// Exported as `device.id`.
    pub device_id: Option<String>,
}

/// Telemetry destination.
#[derive(Debug, Clone, PartialEq)]
pub enum ExporterType {
    /// OTLP over gRPC to a collector (Jaeger, Grafana Tempo, the OpenTelemetry
    /// Collector, or any vendor endpoint that accepts OTLP/gRPC).
    Otlp,
    /// JSON lines on stdout. Handy for local development.
    Console,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            service_name: "unknown_service".to_string(),
            otlp_endpoint: None,
            enable_metrics: true,
            enable_tracing: true,
            mobile_metadata: MobileMetadata::default(),
            exporter_type: ExporterType::Console,
        }
    }
}

/// Builder for [`Config`].
///
/// ```
/// use rust_mobile_telemetry::{ConfigBuilder, ExporterType};
///
/// let config = ConfigBuilder::new()
///     .service_name("auth-api")
///     .exporter_type(ExporterType::Otlp)
///     .otlp_endpoint("http://otel-collector:4317")
///     .build();
/// assert_eq!(config.service_name, "auth-api");
/// ```
pub struct ConfigBuilder {
    config: Config,
}

impl ConfigBuilder {
    /// Starts from [`Config::default`].
    pub fn new() -> Self {
        Self {
            config: Config::default(),
        }
    }

    /// Sets the `service.name` resource attribute.
    pub fn service_name(mut self, name: impl Into<String>) -> Self {
        self.config.service_name = name.into();
        self
    }

    /// Sets the OTLP/gRPC collector URL (only used with [`ExporterType::Otlp`]).
    pub fn otlp_endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.config.otlp_endpoint = Some(endpoint.into());
        self
    }

    /// Enables or disables the metrics pipeline.
    pub fn enable_metrics(mut self, enable: bool) -> Self {
        self.config.enable_metrics = enable;
        self
    }

    /// Enables or disables the tracing pipeline and global subscriber.
    pub fn enable_tracing(mut self, enable: bool) -> Self {
        self.config.enable_tracing = enable;
        self
    }

    /// Sets the app/device metadata attached to all telemetry.
    pub fn mobile_metadata(mut self, metadata: MobileMetadata) -> Self {
        self.config.mobile_metadata = metadata;
        self
    }

    /// Chooses where telemetry is sent.
    pub fn exporter_type(mut self, exporter_type: ExporterType) -> Self {
        self.config.exporter_type = exporter_type;
        self
    }

    /// Overrides configuration from environment variables if present.
    ///
    /// - `OTEL_SERVICE_NAME`: Sets service name.
    /// - `OTEL_EXPORTER_OTLP_ENDPOINT`: Sets OTLP endpoint.
    pub fn with_env(mut self) -> Self {
        if let Ok(name) = env::var("OTEL_SERVICE_NAME") {
            self.config.service_name = name;
        }
        if let Ok(endpoint) = env::var("OTEL_EXPORTER_OTLP_ENDPOINT") {
            self.config.otlp_endpoint = Some(endpoint);
        }
        self
    }

    /// Returns the finished [`Config`].
    pub fn build(self) -> Config {
        self.config
    }
}

impl Default for ConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}
