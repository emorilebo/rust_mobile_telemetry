//! # Rust Mobile Telemetry
//!
//! `rust_mobile_telemetry` provides a production-quality observability instrumentation
//! (tracing + metrics) layer for mobile-backend applications or embedded / edge services.
//!
//! It is built on top of the OpenTelemetry ecosystem and simplifies the setup process
//! by providing sensible defaults and easy-to-use builders.
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use rust_mobile_telemetry::{ConfigBuilder, init_telemetry};
//!
//! fn main() {
//!     let config = ConfigBuilder::new()
//!         .service_name("my-service")
//!         .build();
//!
//!     init_telemetry(config).unwrap();
//! }
//! ```

pub mod config;
pub mod init;
pub mod instrumentation;
pub mod exporters;

pub use config::Config;
pub use config::ConfigBuilder;
pub use config::MobileMetadata;
pub use config::ExporterType;
pub use init::init_telemetry;
pub use instrumentation::with_span;
