//! # rust_mobile_telemetry
//!
//! One-call [OpenTelemetry](https://opentelemetry.io/) setup for Rust mobile
//! backends and edge services: `tracing` spans and metrics, exported to stdout
//! (`Console`) or to any OTLP/gRPC collector (`Otlp`), tagged with the service
//! name and optional app/device metadata, and flushed on shutdown.
//!
//! ## Quick start
//!
//! ```
//! use rust_mobile_telemetry::instrumentation::get_counter;
//! use rust_mobile_telemetry::{
//!     init_telemetry, shutdown_telemetry, with_span, ConfigBuilder, ExporterType, MobileMetadata,
//! };
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
//!     let config = ConfigBuilder::new()
//!         .service_name("checkout-api")
//!         .exporter_type(ExporterType::Console) // `Otlp` in production
//!         .mobile_metadata(MobileMetadata {
//!             app_version: Some("2.4.1".into()),
//!             device_os: Some("Android 14".into()),
//!             device_id: None, // only sent if you set it
//!         })
//!         .build();
//!     init_telemetry(config)?;
//!
//!     let orders = get_counter("orders_total", "Orders placed");
//!     with_span("place_order", || orders.add(1, &[]));
//!
//!     shutdown_telemetry(); // flush spans and metrics before exit
//!     Ok(())
//! }
//! ```
//!
//! ## What gets installed
//!
//! [`init_telemetry`] sets a global `tracing` subscriber (filtered by `RUST_LOG`,
//! default `info`) bridged to OpenTelemetry, plus the global OpenTelemetry
//! tracer and meter providers. Any `tracing` span in your application at or
//! above the filter level is exported, not just the ones created through
//! [`with_span`].
//!
//! Nothing is collected automatically: the exported resource carries
//! `service.name` plus whichever [`MobileMetadata`] fields you set.
//!
//! See the [README](https://github.com/emorilebo/rust_mobile_telemetry#readme)
//! for configuration, limitations and FAQ.

#![warn(missing_docs)]

pub mod config;
pub mod exporters;
pub mod init;
pub mod instrumentation;

pub use config::Config;
pub use config::ConfigBuilder;
pub use config::ExporterType;
pub use config::MobileMetadata;
pub use init::init_telemetry;
pub use init::shutdown_telemetry;
pub use instrumentation::with_span;
pub use instrumentation::with_span_async;

/// Compiles and runs the README's Rust code blocks as doctests.
#[cfg(doctest)]
#[doc = include_str!("../README.md")]
struct ReadmeDoctests;
