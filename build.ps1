# Build script for Simple-Jingle distribution
. "$PSScriptRoot\env.ps1"

$BuildRoot = $PSScriptRoot
$ReleaseDest = "$BuildRoot\release"

# Build release profile
Write-Host "Compiling Simple-Jingle in release mode..." -ForegroundColor Cyan
cargo build --release --manifest-path "$BuildRoot\Cargo.toml"
if ($LASTEXITCODE -ne 0) {
    Write-Host "Build failed!" -ForegroundColor Red
    exit 1
}

# Create distribution directory
if (Test-Path $ReleaseDest) {
    Remove-Item -Path $ReleaseDest -Recurse -Force
}
New-Item -ItemType Directory -Path $ReleaseDest -Force | Out-Null

# Copy assets
Write-Host "Packaging assets..." -ForegroundColor Cyan
Copy-Item -Path "$BuildRoot\target\release\simple-jingle.exe" -Destination $ReleaseDest -Force
Copy-Item -Path "$BuildRoot\settings.ini" -Destination $ReleaseDest -Force
Copy-Item -Path "$BuildRoot\sounds" -Destination $ReleaseDest -Recurse -Force

Write-Host "Packaging complete! Portable release files are in $ReleaseDest" -ForegroundColor Green
