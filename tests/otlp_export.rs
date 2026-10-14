//! End-to-end OTLP test: spins up an in-process gRPC "collector" on localhost,
//! points the crate at it and checks that spans and metrics actually arrive.
//!
//! Regression coverage for 0.1.3 bugs:
//! - spans were dropped unless `RUST_LOG` was set (default `EnvFilter` was ERROR-only),
//! - spans were all named "operation" instead of the name passed to `with_span`,
//! - `shutdown_telemetry` never flushed metrics, so short-lived processes lost them.

use std::sync::{Arc, Mutex};
use std::time::Duration;

use opentelemetry_proto::tonic::collector::metrics::v1::metrics_service_server::{
    MetricsService, MetricsServiceServer,
};
use opentelemetry_proto::tonic::collector::metrics::v1::{
    ExportMetricsServiceRequest, ExportMetricsServiceResponse,
};
use opentelemetry_proto::tonic::collector::trace::v1::trace_service_server::{
    TraceService, TraceServiceServer,
};
use opentelemetry_proto::tonic::collector::trace::v1::{
    ExportTraceServiceRequest, ExportTraceServiceResponse,
};
use opentelemetry_proto::tonic::common::v1::any_value::Value;
use opentelemetry_proto::tonic::common::v1::KeyValue;
use rust_mobile_telemetry::instrumentation::{get_counter, get_histogram, with_span_async};
use rust_mobile_telemetry::{
    init_telemetry, shutdown_telemetry, with_span, ConfigBuilder, ExporterType, MobileMetadata,
};
use tokio::net::TcpListener;
use tokio_stream::wrappers::TcpListenerStream;
use tonic::{Request, Response, Status};

#[derive(Clone, Default)]
struct Collected {
    span_names: Arc<Mutex<Vec<String>>>,
    metric_names: Arc<Mutex<Vec<String>>>,
    resource_attributes: Arc<Mutex<Vec<(String, String)>>>,
}

impl Collected {
    fn record_resource(&self, attributes: &[KeyValue]) {
        let mut stored = self.resource_attributes.lock().unwrap();
        for attribute in attributes {
            if let Some(Value::StringValue(value)) =
                attribute.value.as_ref().and_then(|v| v.value.clone())
            {
                stored.push((attribute.key.clone(), value));
            }
        }
    }
}

#[tonic::async_trait]
impl TraceService for Collected {
    async fn export(
        &self,
        request: Request<ExportTraceServiceRequest>,
    ) -> Result<Response<ExportTraceServiceResponse>, Status> {
        for resource_spans in request.into_inner().resource_spans {
            if let Some(resource) = &resource_spans.resource {
                self.record_resource(&resource.attributes);
            }
            for scope_spans in resource_spans.scope_spans {
                for span in scope_spans.spans {
                    self.span_names.lock().unwrap().push(span.name);
                }
            }
        }
        Ok(Response::new(ExportTraceServiceResponse::default()))
    }
}

#[tonic::async_trait]
impl MetricsService for Collected {
    async fn export(
        &self,
        request: Request<ExportMetricsServiceRequest>,
    ) -> Result<Response<ExportMetricsServiceResponse>, Status> {
        for resource_metrics in request.into_inner().resource_metrics {
            for scope_metrics in resource_metrics.scope_metrics {
                for metric in scope_metrics.metrics {
                    self.metric_names.lock().unwrap().push(metric.name);
                }
            }
        }
        Ok(Response::new(ExportMetricsServiceResponse::default()))
    }
}

async fn start_collector(collected: Collected) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    tokio::spawn(
        tonic::transport::Server::builder()
            .add_service(TraceServiceServer::new(collected.clone()))
            .add_service(MetricsServiceServer::new(collected))
            .serve_with_incoming(TcpListenerStream::new(listener)),
    );
    format!("http://{address}")
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn spans_and_metrics_reach_an_otlp_collector_without_rust_log() {
    // The default must work out of the box; no RUST_LOG, no endpoint overrides.
    for variable in [
        "RUST_LOG",
        "OTEL_EXPORTER_OTLP_ENDPOINT",
        "OTEL_EXPORTER_OTLP_TRACES_ENDPOINT",
        "OTEL_EXPORTER_OTLP_METRICS_ENDPOINT",
    ] {
        std::env::remove_var(variable);
    }

    let collected = Collected::default();
    let endpoint = start_collector(collected.clone()).await;

    let config = ConfigBuilder::new()
        .service_name("otlp-e2e-test")
        .exporter_type(ExporterType::Otlp)
        .otlp_endpoint(endpoint)
        .mobile_metadata(MobileMetadata {
            app_version: Some("2.4.1".to_string()),
            device_os: Some("Android 14".to_string()),
            device_id: None,
        })
        .build();
    init_telemetry(config).expect("init_telemetry should succeed");

    with_span("checkout", || {
        get_counter("orders_total", "Orders placed").add(3, &[]);
    });
    with_span_async("sync_profile", || async {
        get_histogram("sync_duration_seconds", "Profile sync duration").record(0.25, &[]);
    })
    .await;
    // Plain `tracing` spans from application code are exported too.
    tracing::info_span!("native_tracing_span").in_scope(|| {});

    // Only shutdown can flush the metrics: the OTLP reader exports every 3 s
    // and this test is done long before that.
    shutdown_telemetry();

    // Give the server task a moment to finish handling the final request.
    tokio::time::sleep(Duration::from_millis(200)).await;

    let span_names = collected.span_names.lock().unwrap().clone();
    assert!(
        span_names.contains(&"checkout".to_string()),
        "spans: {span_names:?}"
    );
    assert!(
        span_names.contains(&"sync_profile".to_string()),
        "spans: {span_names:?}"
    );
    assert!(
        span_names.contains(&"native_tracing_span".to_string()),
        "spans: {span_names:?}"
    );

    let metric_names = collected.metric_names.lock().unwrap().clone();
    assert!(
        metric_names.contains(&"orders_total".to_string()),
        "metrics: {metric_names:?}"
    );
    assert!(
        metric_names.contains(&"sync_duration_seconds".to_string()),
        "metrics: {metric_names:?}"
    );

    let resource = collected.resource_attributes.lock().unwrap().clone();
    assert!(resource.contains(&("service.name".to_string(), "otlp-e2e-test".to_string())));
    assert!(resource.contains(&("app.version".to_string(), "2.4.1".to_string())));
    assert!(resource.contains(&("device.os".to_string(), "Android 14".to_string())));
    assert!(
        !resource.iter().any(|(key, _)| key == "device.id"),
        "device.id must not be sent when it was not provided: {resource:?}"
    );
}
