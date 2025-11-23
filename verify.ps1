$ErrorActionPreference = "Stop"

Write-Host "1. Running Unit and Integration Tests..." -ForegroundColor Cyan
cargo test
if ($LASTEXITCODE -ne 0) { 
    Write-Error "Tests failed!" 
    exit 1 
}
Write-Host "Tests passed!" -ForegroundColor Green

Write-Host "`n2. Running Example Application (basic_usage)..." -ForegroundColor Cyan
cargo run --example basic_usage
if ($LASTEXITCODE -ne 0) { 
    Write-Error "Example failed to run!" 
    exit 1 
}
Write-Host "Example finished successfully!" -ForegroundColor Green
