<#
.SYNOPSIS
    Packages Nixo Media Player into a standalone redistributable zip archive for Windows.
.PARAMETER OutDir
    Output directory for the packaged release (defaults to 'dist').
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

# 3. Build release binary
Write-Host "Compilando Nixo v$version em modo Release (LTO + opt-level=3 + strip)..."
Push-Location $rootDir
try {
    cargo build --release -p nixo-app
    if ($LASTEXITCODE -ne 0) {
        throw "Falha ao compilar nixo-app em modo release."
    }
} finally {
    Pop-Location
}

$releaseExe = Join-Path $rootDir "target\release\nixo.exe"
if (-not (Test-Path $releaseExe)) {
    throw "Executavel nao encontrado em $releaseExe"
}

# 4. Prepare staging folder
$distRoot = Join-Path $rootDir $OutDir
$bundleName = "nixo-v$version-windows-x86_64"
$stageDir = Join-Path $distRoot $bundleName

if (Test-Path $stageDir) {
    Remove-Item $stageDir -Recurse -Force
}
New-Item -ItemType Directory -Force -Path $stageDir | Out-Null

Write-Host "Copiando arquivos para $stageDir..."
Copy-Item $releaseExe -Destination $stageDir
Copy-Item $mpvDll -Destination $stageDir
Copy-Item (Join-Path $rootDir "assets\fonts") -Destination (Join-Path $stageDir "fonts") -Recurse
Copy-Item (Join-Path $rootDir "README.md") -Destination $stageDir
Copy-Item (Join-Path $rootDir "LICENSE") -Destination $stageDir

# 5. Create ZIP archive
$zipPath = Join-Path $distRoot "$bundleName.zip"
if (Test-Path $zipPath) {
    Remove-Item $zipPath -Force
}

Write-Host "Compactando em $zipPath..."
Compress-Archive -Path "$stageDir\*" -DestinationPath $zipPath -CompressionLevel Optimal

# 6. Generate SHA256 checksum
$hash = (Get-FileHash -Path $zipPath -Algorithm SHA256).Hash
$hashFile = "$zipPath.sha256"
"$hash  $bundleName.zip" | Out-File -FilePath $hashFile -Encoding utf8

$zipItem = Get-Item $zipPath
$zipMb = [math]::Round($zipItem.Length / 1MB, 2)
$exeItem = Get-Item (Join-Path $stageDir "nixo.exe")
$exeMb = [math]::Round($exeItem.Length / 1MB, 2)

Write-Host "=================================================" -ForegroundColor Green
Write-Host "Pacote Nixo v$version gerado com sucesso!" -ForegroundColor Green
Write-Host " - Executavel : $exeMb MB"
Write-Host " - Arquivo ZIP: $zipPath ($zipMb MB)"
Write-Host " - SHA256     : $hash"
Write-Host "=================================================" -ForegroundColor Green
