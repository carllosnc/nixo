# Generates high-res PNG and Windows ICO assets for Nixo Media Player
$assetsDir = Join-Path $PSScriptRoot "..\assets"
New-Item -ItemType Directory -Force -Path $assetsDir | Out-Null

$masterIcon = Join-Path $assetsDir "icon-1024.png"
$pngPath = Join-Path $assetsDir "icon.png"
$icoPath = Join-Path $assetsDir "icon.ico"
$masterLogo = Join-Path $assetsDir "logo-1024.png"
$logoPath = Join-Path $assetsDir "logo.png"

# Se o ícone mestre e o ImageMagick estiverem disponíveis, usa ImageMagick para gerar ícones multirresolução
$hasMagick = (Get-Command magick -ErrorAction SilentlyContinue) -ne $null
if ((Test-Path $masterIcon) -and $hasMagick) {
    Write-Host "Gerando icon.png e icon.ico a partir de icon-1024.png usando ImageMagick..."
    & magick $masterIcon -resize 256x256 $pngPath
    & magick $masterIcon -define icon:auto-resize=256,128,64,48,32,16 $icoPath
    if (Test-Path $masterLogo) {
        Write-Host "Gerando logo.png otimizado a partir de logo-1024.png..."
        & magick $masterLogo -filter Lanczos -resize x128 $logoPath
    }
    Write-Host "Ativos gerados com sucesso em $assetsDir"
    exit 0
}

# Fallback: Geração procedural via System.Drawing
Add-Type -AssemblyName System.Drawing

$width = 256
$height = 256
$bmp = New-Object System.Drawing.Bitmap($width, $height, [System.Drawing.Imaging.PixelFormat]::Format32bppArgb)
$g = [System.Drawing.Graphics]::FromImage($bmp)
$g.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::AntiAlias
$g.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
$g.PixelOffsetMode = [System.Drawing.Drawing2D.PixelOffsetMode]::HighQuality

$g.Clear([System.Drawing.Color]::Transparent)

# Outer squircle verde da marca Nixo (#2FE50B)
$rect = New-Object System.Drawing.Rectangle(12, 12, 232, 232)
$path = New-Object System.Drawing.Drawing2D.GraphicsPath
$radius = 54
$d = $radius * 2
$path.AddArc($rect.X, $rect.Y, $d, $d, 180, 90)
$path.AddArc($rect.Right - $d, $rect.Y, $d, $d, 270, 90)
$path.AddArc($rect.Right - $d, $rect.Bottom - $d, $d, $d, 0, 90)
$path.AddArc($rect.X, $rect.Bottom - $d, $d, $d, 90, 90)
$path.CloseFigure()

$bgBrush = New-Object System.Drawing.SolidBrush([System.Drawing.Color]::FromArgb(255, 47, 229, 11))
$g.FillPath($bgBrush, $path)

# Letra N em preto puro centralizada
$nBrush = New-Object System.Drawing.SolidBrush([System.Drawing.Color]::FromArgb(255, 0, 0, 0))
$font = New-Object System.Drawing.Font("Arial Black", 120, [System.Drawing.FontStyle]::Bold)
$sf = New-Object System.Drawing.StringFormat
$sf.Alignment = [System.Drawing.StringAlignment]::Center
$sf.LineAlignment = [System.Drawing.StringAlignment]::Center
$g.DrawString("N", $font, $nBrush, (New-Object System.Drawing.RectangleF(0, 0, 256, 256)), $sf)

$bmp.Save($pngPath, [System.Drawing.Imaging.ImageFormat]::Png)

# Build .ico with PNG payload
$pngBytes = [System.IO.File]::ReadAllBytes($pngPath)
$icoStream = New-Object System.IO.MemoryStream
$writer = New-Object System.IO.BinaryWriter($icoStream)

$writer.Write([uint16]0) # Reserved
$writer.Write([uint16]1) # Type (1=icon)
$writer.Write([uint16]1) # Count (1 image)

$writer.Write([byte]0)   # Width (0 = 256px)
$writer.Write([byte]0)   # Height (0 = 256px)
$writer.Write([byte]0)   # Color count
$writer.Write([byte]0)   # Reserved
$writer.Write([uint16]1) # Planes
$writer.Write([uint16]32)# Bit count
$writer.Write([uint32]$pngBytes.Length) # Bytes in resource
$writer.Write([uint32]22) # Image offset (6 + 16 = 22)

$writer.Write($pngBytes)
$writer.Flush()

[System.IO.File]::WriteAllBytes($icoPath, $icoStream.ToArray())

$writer.Dispose()
$icoStream.Dispose()
$g.Dispose()
$bmp.Dispose()

Write-Host "Icons generated in $assetsDir"
