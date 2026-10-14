# Changelog

All notable changes to this project are documented here.
The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and the project adheres to [Semantic Versioning](https://semver.org/).

## [0.2.0] - 2026-10-14

Minor bump because exported span names, the default span filter and some error
behaviour changed (all of them were bugs, but dashboards may notice).

### Fixed
- Spans created by `with_span` / `with_span_async` were silently dropped unless
  `RUST_LOG` was set: the `EnvFilter` defaulted to `ERROR`. The default is now `info`.
- Exported spans were all named `operation` / `async_operation`; they now carry the
  name passed to `with_span` / `with_span_async` (via `otel.name`). The `name`
  attribute is still set for compatibility.
- `shutdown_telemetry()` did not flush metrics, so short-lived processes lost their
  final values. It now force-flushes the meter provider and shuts it down
  (opentelemetry_sdk 0.21's `MeterProvider::shutdown()` alone exports nothing).
- `init_telemetry` panicked outside a Tokio runtime ("there is no reactor running").
  Background export now runs on a small crate-owned Tokio runtime, so it works from
  synchronous code too.
- `shutdown_telemetry()` deadlocked on a `current_thread` Tokio runtime (e.g. a plain
  `#[tokio::test]`) with the OTLP exporter; it now returns normally on every runtime flavour.
- `init_telemetry` panicked when called twice or when another global `tracing`
  subscriber was installed; it now returns an `Err`.
- The crate-level Quick Start example panicked when copied into a real program (see
  above); the new example runs as a doctest.
- `cargo fmt --check` failed, so the CI workflow could not pass.

### Added
- Re-exports `shutdown_telemetry` and `with_span_async` at the crate root.
- `exporters::DEFAULT_OTLP_ENDPOINT` constant.
- End-to-end OTLP test against an in-process gRPC collector, plus regression tests
  for the fixes above; README code blocks now run as doctests.
- Rustdoc for every public item.

### Changed
- Package metadata: author email, description, keywords, categories, homepage and
  documentation links; the published package now contains only sources, examples,
  tests, README, CHANGELOG and LICENSE.
- CI also lints tests/examples, runs all-feature tests and builds docs with `-D warnings`.
- Refreshed `Cargo.lock` within existing semver ranges.

### Deprecated
- The `instrumentation` cargo feature is now a no-op and will be removed.

### Removed
- Unused dependencies `serde`, `thiserror` and the optional `reqwest` (which also drops
  the implicit `reqwest` cargo feature). If your crate relied on them being enabled
  transitively, add them to your own `Cargo.toml`.

## [0.1.3] - 2025-11-23
### Fixed
- Repository URL in package metadata.

## [0.1.1] - 2025-11-23
### Added
- LICENSE file and documentation updates.

## [0.1.0] - 2025-11-23
- Initial release.

[0.2.0]: https://github.com/emorilebo/rust_mobile_telemetry/compare/e8c895a...main
[0.1.3]: https://crates.io/crates/rust_mobile_telemetry/0.1.3
[0.1.1]: https://crates.io/crates/rust_mobile_telemetry/0.1.1
[0.1.0]: https://crates.io/crates/rust_mobile_telemetry/0.1.0
