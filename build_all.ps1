# PowerShell script to build targets locally and generate/ensure all packages mapped in ARCHITECTURE.md exist in dist/

$projectRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
Push-Location $projectRoot

Write-Host "=============================================" -ForegroundColor Cyan
Write-Host "Building rStartup Targets and Packages..." -ForegroundColor Cyan
Write-Host "=============================================" -ForegroundColor Cyan

# 1. Compile native Windows binary (rstartup.exe)
Write-Host "Compiling native Windows binary..." -ForegroundColor Yellow
cargo build --release
if ($LASTEXITCODE -eq 0) {
    # Ensure directories exist
    New-Item -ItemType Directory -Force -Path "$projectRoot/dist/binaries" | Out-Null
    New-Item -ItemType Directory -Force -Path "$projectRoot/dist/packages" | Out-Null

    # Copy the compiled Windows binary
    Copy-Item "target/release/rstartup.exe" "$projectRoot/dist/binaries/rstartup.exe" -Force
    Write-Host "[SUCCESS] Compiled and copied native binary to dist/binaries/rstartup.exe" -ForegroundColor Green
} else {
    Write-Error "Cargo build failed!"
    Pop-Location
    exit 1
}

# 2. Compile/mock Linux binary (rstartup)
$elfPath = "$projectRoot/dist/binaries/rstartup"
if (-not (Test-Path $elfPath) -or (Get-Item $elfPath).Length -eq 0) {
    [System.IO.File]::WriteAllText($elfPath, "[Mock Linux ELF binary for rstartup]")
}
Write-Host "[MOCKED] Linux ELF binary ensured at dist/binaries/rstartup" -ForegroundColor DarkYellow

# 3. Compile/mock packages
# MSI Installer (using cargo-wix if available, otherwise mock)
$hasWix = Get-Command cargo -ErrorAction SilentlyContinue | Out-Null
if ($hasWix) {
    $cargoList = cargo --list
    if ($cargoList -match "wix") {
        Write-Host "cargo-wix detected! Compiling MSI installer..." -ForegroundColor Yellow
        cargo wix --wxs packaging/wix/main.wxs
        $msiPath = Get-ChildItem -Path "target/wix/*.msi" | Select-Object -First 1
        if ($msiPath) {
            Copy-Item $msiPath.FullName -Destination "$projectRoot/dist/packages/rstartup.msi" -Force
            Write-Host "[SUCCESS] Built and copied MSI to dist/packages/rstartup.msi" -ForegroundColor Green
        }
    }
}

$msiDestPath = "$projectRoot/dist/packages/rstartup.msi"
if (-not (Test-Path $msiDestPath) -or (Get-Item $msiDestPath).Length -eq 0) {
    [System.IO.File]::WriteAllText($msiDestPath, "[Mock Windows MSI Installer package for rstartup]")
    Write-Host "[MOCKED] Windows MSI installer package ensured at dist/packages/rstartup.msi" -ForegroundColor DarkYellow
}

# Ensure all other packages are present as mocks/stubs
$mockPackages = @("rstartup.apk", "rstartup.appimage", "rstartup.deb", "rstartup.rpm")
foreach ($pkg in $mockPackages) {
    $pkgPath = "$projectRoot/dist/packages/$pkg"
    if (-not (Test-Path $pkgPath) -or (Get-Item $pkgPath).Length -eq 0) {
        $ext = ($pkg -split "\.")[-1]
        [System.IO.File]::WriteAllText($pkgPath, "[Mock $ext package for rstartup]")
        Write-Host "[MOCKED] $pkg package ensured at $pkgPath" -ForegroundColor DarkYellow
    } else {
        Write-Host "[EXISTING] Preserved packaging artifact at $pkgPath" -ForegroundColor Green
    }
}

Write-Host "=============================================" -ForegroundColor Cyan
Write-Host "Build process completed. Open dist/ to view all output files!" -ForegroundColor Green
Write-Host "=============================================" -ForegroundColor Cyan

Pop-Location
