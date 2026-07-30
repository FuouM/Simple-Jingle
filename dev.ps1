# Build and run Simple-Jingle
. "$PSScriptRoot\env.ps1"

# Stop any running instance
$procs = Get-Process simple-jingle -ErrorAction SilentlyContinue
if ($procs) {
    Write-Host "Stopping running Simple-Jingle..." -ForegroundColor Yellow
    $procs | Stop-Process -Force
    Start-Sleep -Seconds 1
}

# Build
Write-Host "Building Simple-Jingle..." -ForegroundColor Cyan
cargo build --release --manifest-path "$PSScriptRoot\Cargo.toml"
if ($LASTEXITCODE -ne 0) {
    Write-Host "Build failed!" -ForegroundColor Red
    exit 1
}
Write-Host "Build OK." -ForegroundColor Green

# Copy assets next to the exe
$releaseDir = "$PSScriptRoot\target\release"
Copy-Item -Path "$PSScriptRoot\sounds" -Destination "$releaseDir" -Recurse -Force
Copy-Item -Path "$PSScriptRoot\settings.ini" -Destination "$releaseDir\settings.ini" -Force

# Run directly (not detached) so debug output is visible
$exePath = "$releaseDir\simple-jingle.exe"
if (Test-Path $exePath) {
    Write-Host "Starting Simple-Jingle..." -ForegroundColor Cyan
    & $exePath
} else {
    Write-Host "Executable not found: $exePath" -ForegroundColor Red
    exit 1
}
