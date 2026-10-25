<#
.SYNOPSIS
    Builds the Inno Setup installer executable (.exe) for Nixo Media Player on Windows.
.PARAMETER OutDir
    Output directory for the compiled installer (defaults to 'dist').
#>
param (
    [string]$OutDir = "dist"
)

$ErrorActionPreference = "Stop"
$rootDir = (Get-Item $PSScriptRoot).Parent.FullName

# 1. Ensure libmpv-2.dll is available in vendor
$mpvDll = Join-Path $rootDir "vendor\mpv\libmpv-2.dll"
if (-not (Test-Path $mpvDll)) {
    Write-Host "libmpv-2.dll nao encontrada. Baixando via scripts/setup-mpv.ps1..."
    & (Join-Path $rootDir "scripts\setup-mpv.ps1")
}

# 2. Extract version from Cargo.toml
$cargoToml = Get-Content (Join-Path $rootDir "Cargo.toml") -Raw
if ($cargoToml -match 'version\s*=\s*"([^"]+)"') {
    $version = $matches[1]
} else {
    $version = "0.1.0"
}

# 3. Compile release binary if needed
$releaseExe = Join-Path $rootDir "target\release\nixo.exe"
if (-not (Test-Path $releaseExe)) {
    Write-Host "Compilando Nixo v$version em modo Release..."
    Push-Location $rootDir
    try {
        cargo build --release -p nixo-app
        if ($LASTEXITCODE -ne 0) {
            throw "Falha ao compilar nixo-app em modo release."
        }
    } finally {
        Pop-Location
    }
}

# 4. Locate ISCC.exe (Inno Setup Compiler)
$isccCandidates = @(
    "ISCC.exe",
    "$env:LOCALAPPDATA\Programs\Inno Setup 7\ISCC.exe",
    "$env:LOCALAPPDATA\Programs\Inno Setup 6\ISCC.exe",
    "$env:LOCALAPPDATA\Programs\InnoSetup\ISCC.exe",
    "C:\Program Files\Inno Setup 7\ISCC.exe",
    "C:\Program Files (x86)\Inno Setup 7\ISCC.exe",
    "C:\Program Files\Inno Setup 6\ISCC.exe",
    "C:\Program Files (x86)\Inno Setup 6\ISCC.exe"
)

$isccPath = $null
foreach ($candidate in $isccCandidates) {
    if (Get-Command $candidate -ErrorAction SilentlyContinue) {
        $isccPath = $candidate
        break
    }
    if (Test-Path $candidate) {
        $isccPath = $candidate
        break
    }
}

if (-not $isccPath) {
    Write-Host "Inno Setup Compiler (ISCC.exe) nao encontrado. Instalando..."
    & (Join-Path $rootDir "scripts\setup-inno.ps1")
    $defaultUserPath = "$env:LOCALAPPDATA\Programs\InnoSetup\ISCC.exe"
    if (Test-Path $defaultUserPath) {
        $isccPath = $defaultUserPath
    } else {
        throw "Nao foi possivel encontrar ou instalar o Inno Setup Compiler (ISCC.exe)."
    }
}

# 5. Compile Installer
$distDir = Join-Path $rootDir $OutDir
if (-not (Test-Path $distDir)) {
    New-Item -ItemType Directory -Path $distDir -Force | Out-Null
}

$issScript = Join-Path $rootDir "scripts\installer.iss"
Write-Host "Compilando instalador com Inno Setup ($isccPath)..."
& $isccPath "/DMyAppVersion=$version" "/O$distDir" $issScript

if ($LASTEXITCODE -ne 0) {
    throw "Falha ao compilar o instalador Inno Setup."
}

$installerExe = Join-Path $distDir "nixo-v$version-windows-x86_64-setup.exe"
if (Test-Path $installerExe) {
    Write-Host "Calculando SHA256 do instalador..."
    $hash = (Get-FileHash -Path $installerExe -Algorithm SHA256).Hash.ToLower()
    $hashFile = "$installerExe.sha256"
    Set-Content -Path $hashFile -Value "$hash  $(Split-Path -Leaf $installerExe)"
    Write-Host "`n==============================================="
    Write-Host "Instalador gerado com sucesso!"
    Write-Host "Arquivo: $installerExe"
    Write-Host "SHA256:  $hash"
    Write-Host "===============================================`n"
}
