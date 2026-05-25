# Desktop App Icons

This directory contains icons for the desktop application.

## Required Files

Tauri requires the following icon files for cross-platform builds:

- `icon.ico` - Windows icon (256x256)
- `icon.icns` - macOS icon (512x512@2x)
- `32x32.png` - Small icon
- `128x128.png` - Medium icon
- `128x128@2x.png` - Medium icon (retina)
- `icon.png` - Source icon (1024x1024 recommended)

## Generating Icons

### Option 1: Using Tauri CLI

If you have a high-resolution PNG icon (1024x1024), use:

```bash
cd desktop
npm run icons
```

This will generate all required formats from `src-tauri/icons/icon.png`.

### Option 2: Manual Generation

1. Create a 1024x1024 PNG icon
2. Use ImageMagick to generate other sizes:

```bash
# Install ImageMagick
# macOS: brew install imagemagick
# Ubuntu: sudo apt install imagemagick
# Windows: choco install imagemagick

# Generate PNGs
convert icon.png -resize 32x32 32x32.png
convert icon.png -resize 128x128 128x128.png
convert icon.png -resize 256x256 128x128@2x.png

# Generate ICO (Windows)
convert icon.png -resize 256x256 icon.ico

# Generate ICNS (macOS)
# Requires iconutil on macOS
mkdir icon.iconset
sips -z 16 16     icon.png --out icon.iconset/icon_16x16.png
sips -z 32 32     icon.png --out icon.iconset/icon_16x16@2x.png
sips -z 32 32     icon.png --out icon.iconset/icon_32x32.png
sips -z 64 64     icon.png --out icon.iconset/icon_32x32@2x.png
sips -z 128 128   icon.png --out icon.iconset/icon_128x128.png
sips -z 256 256   icon.png --out icon.iconset/icon_128x128@2x.png
sips -z 256 256   icon.png --out icon.iconset/icon_256x256.png
sips -z 512 512   icon.png --out icon.iconset/icon_256x256@2x.png
sips -z 512 512   icon.png --out icon.iconset/icon_512x512.png
sips -z 1024 1024 icon.png --out icon.iconset/icon_512x512@2x.png
iconutil -c icns icon.iconset
rm -rf icon.iconset
```

### Option 3: Online Tools

Use online icon generators:
- [CloudConvert](https://cloudconvert.com/svg-to-ico)
- [Favicon.io](https://favicon.io/)
- [RealFaviconGenerator](https://realfavicongenerator.net/)

## Current Icons

The `icon.svg` file is the source vector icon from the Agent Zero webui.
Before building for production, convert it to the required formats above.
