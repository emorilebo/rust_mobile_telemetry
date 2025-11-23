//! # Rust Mobile Telemetry
//!
//! `rust_mobile_telemetry` provides a production-quality observability instrumentation
//! (tracing + metrics) layer for mobile-backend applications or embedded / edge services.
//!
//! It is built on top of the OpenTelemetry ecosystem.

pub mod config;
pub mod init;
pub mod instrumentation;
pub mod exporters;

pub use config::Config;
pub use init::init_telemetry;
pub use instrumentation::with_span;
