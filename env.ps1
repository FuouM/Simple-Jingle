# Local Rust Environment Setup for Simple-Jingle
$RootPath = $PSScriptRoot

$LocalRust = "$RootPath\.rust"
if (Test-Path $LocalRust) {
    $env:RUSTUP_HOME = "$LocalRust\.rustup"
    $env:CARGO_HOME = "$LocalRust\.cargo"

    $BinPath = "$LocalRust\.cargo\bin"
    if ($env:PATH -notlike "*$BinPath*") {
        $env:PATH = "$BinPath;" + $env:PATH
    }
}

Write-Host "Simple-Jingle local Rust environment loaded." -ForegroundColor Green
Write-Host "RUSTUP_HOME: $env:RUSTUP_HOME"
Write-Host "CARGO_HOME:  $env:CARGO_HOME"
Write-Host "rustc version:"
rustc --version
