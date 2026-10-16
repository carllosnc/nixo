$ErrorActionPreference = "Stop"

$vendorDir = Join-Path $PSScriptRoot "..\vendor\mpv"
New-Item -ItemType Directory -Force -Path $vendorDir | Out-Null

Write-Host "Consultando ultima versao do libmpv (shinchiro/mpv-winbuild-cmake)..."
$rel = Invoke-RestMethod -Uri "https://api.github.com/repos/shinchiro/mpv-winbuild-cmake/releases/latest"
$asset = $rel.assets | Where-Object { $_.name -match "^mpv-dev-x86_64-20.*\.7z$" } | Select-Object -First 1

if (-not $asset) {
    throw "Pacote mpv-dev-x86_64 nao encontrado na release mais recente."
}

$archivePath = Join-Path $vendorDir "mpv-dev.7z"
Write-Host "Baixando $($asset.name) ($([math]::Round($asset.size / 1MB, 1)) MB)..."
Invoke-WebRequest -Uri $asset.browser_download_url -OutFile $archivePath

Write-Host "Extraindo libmpv-2.dll para $vendorDir..."
7z x $archivePath -o"$vendorDir" -y | Out-Null
Remove-Item $archivePath -Force

Write-Host "Concluido! libmpv-2.dll instalado em $vendorDir"
