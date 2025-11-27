#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ICONS_DIR="$SCRIPT_DIR/../src-tauri/icons"
SOURCE_SVG="$SCRIPT_DIR/../public/sena-icon.svg"

mkdir -p "$ICONS_DIR"

if command -v rsvg-convert &> /dev/null; then
    rsvg-convert -w 32 -h 32 "$SOURCE_SVG" -o "$ICONS_DIR/32x32.png"
    rsvg-convert -w 128 -h 128 "$SOURCE_SVG" -o "$ICONS_DIR/128x128.png"
    rsvg-convert -w 256 -h 256 "$SOURCE_SVG" -o "$ICONS_DIR/128x128@2x.png"
    rsvg-convert -w 512 -h 512 "$SOURCE_SVG" -o "$ICONS_DIR/icon.png"
    echo "Icons generated with rsvg-convert"
elif command -v convert &> /dev/null; then
    convert -background none "$SOURCE_SVG" -resize 32x32 "$ICONS_DIR/32x32.png"
    convert -background none "$SOURCE_SVG" -resize 128x128 "$ICONS_DIR/128x128.png"
    convert -background none "$SOURCE_SVG" -resize 256x256 "$ICONS_DIR/128x128@2x.png"
    convert -background none "$SOURCE_SVG" -resize 512x512 "$ICONS_DIR/icon.png"
    echo "Icons generated with ImageMagick"
else
    echo "No SVG converter found. Using placeholder icon."
    echo "Install librsvg or ImageMagick for proper icon generation."
fi

if [ -f "$ICONS_DIR/icon.png" ]; then
    if command -v iconutil &> /dev/null; then
        ICONSET_DIR="$ICONS_DIR/icon.iconset"
        mkdir -p "$ICONSET_DIR"

        if command -v sips &> /dev/null; then
            sips -z 16 16 "$ICONS_DIR/icon.png" --out "$ICONSET_DIR/icon_16x16.png" 2>/dev/null || true
            sips -z 32 32 "$ICONS_DIR/icon.png" --out "$ICONSET_DIR/icon_16x16@2x.png" 2>/dev/null || true
            sips -z 32 32 "$ICONS_DIR/icon.png" --out "$ICONSET_DIR/icon_32x32.png" 2>/dev/null || true
            sips -z 64 64 "$ICONS_DIR/icon.png" --out "$ICONSET_DIR/icon_32x32@2x.png" 2>/dev/null || true
            sips -z 128 128 "$ICONS_DIR/icon.png" --out "$ICONSET_DIR/icon_128x128.png" 2>/dev/null || true
            sips -z 256 256 "$ICONS_DIR/icon.png" --out "$ICONSET_DIR/icon_128x128@2x.png" 2>/dev/null || true
            sips -z 256 256 "$ICONS_DIR/icon.png" --out "$ICONSET_DIR/icon_256x256.png" 2>/dev/null || true
            sips -z 512 512 "$ICONS_DIR/icon.png" --out "$ICONSET_DIR/icon_256x256@2x.png" 2>/dev/null || true
            sips -z 512 512 "$ICONS_DIR/icon.png" --out "$ICONSET_DIR/icon_512x512.png" 2>/dev/null || true
            cp "$ICONS_DIR/icon.png" "$ICONSET_DIR/icon_512x512@2x.png"

            iconutil -c icns "$ICONSET_DIR" -o "$ICONS_DIR/icon.icns"
            rm -rf "$ICONSET_DIR"
            echo "macOS .icns icon generated"
        fi
    fi
fi

echo "Icon generation complete"
ls -la "$ICONS_DIR/"
