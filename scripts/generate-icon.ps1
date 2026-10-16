# Generates high-res PNG and Windows ICO assets for Nixo Media Player
Add-Type -AssemblyName System.Drawing
$assetsDir = Join-Path $PSScriptRoot "..\assets"
New-Item -ItemType Directory -Force -Path $assetsDir | Out-Null

$width = 256
$height = 256
$bmp = New-Object System.Drawing.Bitmap($width, $height, [System.Drawing.Imaging.PixelFormat]::Format32bppArgb)
$g = [System.Drawing.Graphics]::FromImage($bmp)
$g.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::AntiAlias
$g.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
$g.PixelOffsetMode = [System.Drawing.Drawing2D.PixelOffsetMode]::HighQuality

$g.Clear([System.Drawing.Color]::Transparent)

# Outer squircle / rounded rect
$rect = New-Object System.Drawing.Rectangle(12, 12, 232, 232)
$path = New-Object System.Drawing.Drawing2D.GraphicsPath
$radius = 54
$d = $radius * 2
$path.AddArc($rect.X, $rect.Y, $d, $d, 180, 90)
$path.AddArc($rect.Right - $d, $rect.Y, $d, $d, 270, 90)
$path.AddArc($rect.Right - $d, $rect.Bottom - $d, $d, $d, 0, 90)
$path.AddArc($rect.X, $rect.Bottom - $d, $d, $d, 90, 90)
$path.CloseFigure()

# Background fill with subtle dark gradient
$bgBrush = New-Object System.Drawing.Drawing2D.LinearGradientBrush(
    (New-Object System.Drawing.PointF(0, 12)),
    (New-Object System.Drawing.PointF(0, 244)),
    [System.Drawing.Color]::FromArgb(255, 28, 32, 44),
    [System.Drawing.Color]::FromArgb(255, 12, 14, 20)
)
$g.FillPath($bgBrush, $path)

# Subtle border
$borderPen = New-Object System.Drawing.Pen([System.Drawing.Color]::FromArgb(180, 255, 107, 53), 4)
$g.DrawPath($borderPen, $path)

# Play triangle icon with vibrant gradient
$trianglePath = New-Object System.Drawing.Drawing2D.GraphicsPath
$p1 = New-Object System.Drawing.PointF(92, 70)
$p2 = New-Object System.Drawing.PointF(92, 186)
$p3 = New-Object System.Drawing.PointF(190, 128)
$trianglePath.AddLines([System.Drawing.PointF[]]@($p1, $p2, $p3))
$trianglePath.CloseFigure()

$playBrush = New-Object System.Drawing.Drawing2D.LinearGradientBrush(
    $p1, $p3,
    [System.Drawing.Color]::FromArgb(255, 255, 140, 60),
    [System.Drawing.Color]::FromArgb(255, 255, 75, 30)
)
$g.FillPath($playBrush, $trianglePath)

# Play icon inner subtle highlight
$highlightPen = New-Object System.Drawing.Pen([System.Drawing.Color]::FromArgb(120, 255, 255, 255), 3)
$g.DrawPath($highlightPen, $trianglePath)

$pngPath = Join-Path $assetsDir "icon.png"
$bmp.Save($pngPath, [System.Drawing.Imaging.ImageFormat]::Png)

# Build .ico with PNG payload
$pngBytes = [System.IO.File]::ReadAllBytes($pngPath)
$icoStream = New-Object System.IO.MemoryStream
$writer = New-Object System.IO.BinaryWriter($icoStream)

# ICONDIR
$writer.Write([uint16]0) # Reserved
$writer.Write([uint16]1) # Type (1=icon)
$writer.Write([uint16]1) # Count (1 image)

# ICONDIRENTRY
$writer.Write([byte]0)   # Width (0 = 256px)
$writer.Write([byte]0)   # Height (0 = 256px)
$writer.Write([byte]0)   # Color count
$writer.Write([byte]0)   # Reserved
$writer.Write([uint16]1) # Planes
$writer.Write([uint16]32)# Bit count
$writer.Write([uint32]$pngBytes.Length) # Bytes in resource
$writer.Write([uint32]22) # Image offset (6 + 16 = 22)

# Write PNG bytes
$writer.Write($pngBytes)
$writer.Flush()

$icoPath = Join-Path $assetsDir "icon.ico"
[System.IO.File]::WriteAllBytes($icoPath, $icoStream.ToArray())

$writer.Dispose()
$icoStream.Dispose()
$g.Dispose()
$bmp.Dispose()

Write-Host "Icons generated in $assetsDir"
