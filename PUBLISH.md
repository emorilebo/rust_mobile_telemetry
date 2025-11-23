# Publishing Instructions

Follow these steps to publish `rust_mobile_telemetry` to crates.io.

## Prerequisites

- A [crates.io](https://crates.io/) account.
- `cargo` installed and authenticated.

## Steps

1.  **Login to crates.io**
    ```bash
    cargo login <your-api-token>
    ```

2.  **Verify Metadata**
    Check `Cargo.toml` to ensure version, description, and repository are correct.
    ```toml
    [package]
    name = "rust_mobile_telemetry"
    version = "0.1.0"
    ...
    ```

3.  **Dry Run**
    Run a dry run to verify the package can be published.
    ```bash
    cargo publish --dry-run
    ```

4.  **Publish**
    If the dry run succeeds, publish the crate.
    ```bash
    cargo publish
    ```

5.  **Tag Release**
    After successful publish, tag the release in git.
    ```bash
    git tag v0.1.0
    git push origin v0.1.0
    ```
