# ==============================================================================
# Flame & Blaze Toolchain Installer (PowerShell for Windows)
# ==============================================================================

$ErrorActionPreference = "Stop"

Write-Host "  _    _      _ _             _____                 _                         " -ForegroundColor Cyan
Write-Host " | |  | |    | | |           |  __ \               | |                        " -ForegroundColor Cyan
Write-Host " | |__| | ___| | | ___       | |  | | _____   _____| | ___  _ __   ___ _ __   " -ForegroundColor Cyan
Write-Host " |  __  |/ _ \ | |/ _ \      | |  | |/ _ \ \ / / _ \ |/ _ \| '_ \ / _ \ '__|  " -ForegroundColor Cyan
Write-Host " | |  | |  __/ | | (_) |     | |__| |  __/\ V /  __/ | (_) | |_) |  __/ |     " -ForegroundColor Cyan
Write-Host " |_|  |_|\___|_|_|\___/      |_____/ \___| \_/ \___|_|\___/| .__/ \___|_|     " -ForegroundColor Cyan
Write-Host "                                                           | |                " -ForegroundColor Cyan
Write-Host "                                                           |_|                " -ForegroundColor Cyan
Write-Host ""
Write-Host "Installing Flame Language & Blaze Toolchain..." -ForegroundColor Yellow
Write-Host ""

# 1. Verify Cargo is available
$cargoCmd = Get-Command cargo -ErrorAction SilentlyContinue
if (-not $cargoCmd) {
    Write-Host "Error: Cargo is not installed or not in PATH." -ForegroundColor Red
    Write-Host "Please install Rust and Cargo from https://rustup.rs/ before continuing."
    exit 1
}

$cargoVer = & cargo --version
Write-Host "Rust/Cargo detected: $cargoVer" -ForegroundColor Green

# 2. Build and install via Cargo
$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
if (-not $scriptDir) { $scriptDir = (Get-Location).Path }
Set-Location $scriptDir

Write-Host ""
Write-Host "[1/3] Building and installing Flame binaries (fmp & flamelang)..." -ForegroundColor Cyan
& cargo install --path . --force

# 3. Locate Cargo bin directory
$cargoBin = Join-Path $env:USERPROFILE ".cargo\bin"
if ($env:CARGO_HOME) {
    $cargoBin = if ($env:CARGO_HOME.EndsWith("bin")) { $env:CARGO_HOME } else { Join-Path $env:CARGO_HOME "bin" }
}

# Ensure fmp.exe exists
$fmpExe = Join-Path $cargoBin "fmp.exe"
$flamelangExe = Join-Path $cargoBin "flamelang.exe"

if (Test-Path $flamelangExe) {
    Copy-Item $flamelangExe $fmpExe -Force
}
elseif (Test-Path $fmpExe) {
    Copy-Item $fmpExe $flamelangExe -Force
}

# Create command shims
$fmpCmd = Join-Path $cargoBin "fmp.cmd"
$fmpBat = Join-Path $cargoBin "fmp.bat"
Set-Content -Path $fmpCmd -Value '@"%~dp0fmp.exe" %*' -Encoding ASCII
Set-Content -Path $fmpBat -Value '@"%~dp0fmp.exe" %*' -Encoding ASCII

# 4. Install Blaze standard library definitions
Write-Host ""
Write-Host "[2/3] Setting up Blaze standard library definition directory..." -ForegroundColor Cyan

$sourceBlaze = Join-Path $scriptDir "Blaze\std"
if (-not (Test-Path $sourceBlaze)) {
    $sourceBlaze = Join-Path $scriptDir "std"
}

if (-not (Test-Path $sourceBlaze)) {
    Write-Host "Error: Could not locate source standard library definitions at $sourceBlaze" -ForegroundColor Red
    exit 1
}

$targetBlazeDirs = @(
    (Join-Path $env:LOCALAPPDATA "Blaze\std"),
    (Join-Path $env:USERPROFILE ".blaze\std")
)

if ($env:ProgramFiles) {
    $progBlaze = Join-Path $env:ProgramFiles "Blaze\std"
    $targetBlazeDirs = @($progBlaze) + $targetBlazeDirs
}

$primaryBlazeDir = ""

foreach ($dest in $targetBlazeDirs) {
    try {
        if (-not (Test-Path $dest)) {
            New-Item -ItemType Directory -Path $dest -Force | Out-Null
        }
        Copy-Item -Path "$sourceBlaze\*" -Destination $dest -Recurse -Force | Out-Null
        Write-Host "  Installed definitions to: $dest" -ForegroundColor Green
        if (-not $primaryBlazeDir) {
            $primaryBlazeDir = (Split-Path -Parent $dest)
        }
    }
    catch {
        # continue to next candidate directory if permission denied
    }
}

# 5. Environment & Permanent PATH persistence
Write-Host ""
Write-Host "[3/3] Setting up environment and permanently persisting PATH..." -ForegroundColor Cyan

$userPath = [Environment]::GetEnvironmentVariable("Path", "User")
if (-not ($userPath -split ";" -contains $cargoBin)) {
    $newPath = ($userPath.TrimEnd(";") + ";" + $cargoBin).TrimStart(";")
    [Environment]::SetEnvironmentVariable("Path", $newPath, "User")
    Write-Host "  Permanently added $cargoBin to User PATH." -ForegroundColor Green
}

if ($primaryBlazeDir) {
    [Environment]::SetEnvironmentVariable("BLAZE_HOME", $primaryBlazeDir, "User")
    Write-Host "  Set BLAZE_HOME to $primaryBlazeDir in User Environment." -ForegroundColor Green
}

Write-Host ""
Write-Host "[OK] Flame and Blaze toolchain successfully installed!" -ForegroundColor Green
Write-Host ""
Write-Host "  Primary Command:  fmp (also available as flamelang)" -ForegroundColor Cyan
Write-Host "  Binary Location:  $fmpExe"
Write-Host "  Blaze Definitions:$primaryBlazeDir\std"

Write-Host ""
Write-Host "Quick Start:" -ForegroundColor Yellow
Write-Host "  Check version:    fmp --version"
Write-Host "  CLI Help menu:    fmp help"
Write-Host "  Update release:   fmp update"
Write-Host "  Uninstall:        fmp uninstall"
Write-Host ""
