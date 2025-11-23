use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub service_name: String,
    pub otlp_endpoint: Option<String>,
    pub enable_metrics: bool,
    pub enable_tracing: bool,
    pub mobile_metadata: MobileMetadata,
    pub exporter_type: ExporterType,
}

#[derive(Debug, Clone, Default)]
pub struct MobileMetadata {
    pub app_version: Option<String>,
    pub device_os: Option<String>,
    pub device_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExporterType {
    Otlp,
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

pub struct ConfigBuilder {
    config: Config,
}

impl ConfigBuilder {
    pub fn new() -> Self {
        Self {
            config: Config::default(),
        }
    }

    pub fn service_name(mut self, name: impl Into<String>) -> Self {
        self.config.service_name = name.into();
        self
    }

    pub fn otlp_endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.config.otlp_endpoint = Some(endpoint.into());
        self
    }

    pub fn enable_metrics(mut self, enable: bool) -> Self {
        self.config.enable_metrics = enable;
        self
    }

    pub fn enable_tracing(mut self, enable: bool) -> Self {
        self.config.enable_tracing = enable;
        self
    }

    pub fn mobile_metadata(mut self, metadata: MobileMetadata) -> Self {
        self.config.mobile_metadata = metadata;
        self
    }

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

    pub fn build(self) -> Config {
        self.config
    }
}

impl Default for ConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}
