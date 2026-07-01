#!/usr/bin/env bash
# Download Noto Sans SC (Simplified Chinese) font for PDF generation
set -euo pipefail

FONTS_DIR="$(dirname "$0")/../fonts"
FONT_NAME="NotoSansCJKsc-Regular.otf"
FONT_PATH="$FONTS_DIR/$FONT_NAME"

# Check if font already exists
if [ -f "$FONT_PATH" ]; then
  echo "✅ Font already exists: $FONT_PATH ($(du -h "$FONT_PATH" | cut -f1))"
  exit 0
fi

# Option 1: Try jsDelivr CDN (fast in China, from GitHub mirror)
echo "📥 Downloading Noto Sans SC font from jsDelivr CDN..."
if curl -fL --connect-timeout 10 --max-time 120 -o "$FONT_PATH" \
  "https://cdn.jsdelivr.net/gh/notofonts/noto-cjk@Sans2.004/Sans/OTF/SimplifiedChinese/NotoSansCJKsc-Regular.otf"; then
  echo "✅ Font downloaded via jsDelivr CDN"
  ls -lh "$FONT_PATH"
  exit 0
fi

echo "⚠️  jsDelivr failed, trying GitHub release..."

# Option 2: Try GitHub release (zip with all OTF files)
FONT_ZIP="/tmp/NotoSansCJKsc.zip"
FONT_DIR="/tmp/NotoSansCJKsc"

if curl -fL --connect-timeout 10 --max-time 120 -o "$FONT_ZIP" \
  "https://github.com/notofonts/noto-cjk/releases/download/Sans2.004/08_NotoSansCJKsc.zip"; then
  echo "📦 Extracting..."
  rm -rf "$FONT_DIR"
  unzip -q -d "$FONT_DIR" "$FONT_ZIP"
  find "$FONT_DIR" -name "$FONT_NAME" -exec cp {} "$FONTS_DIR/" \;
  rm -rf "$FONT_DIR" "$FONT_ZIP"
  echo "✅ Font extracted from GitHub release"
  ls -lh "$FONT_PATH"
  exit 0
fi

echo "❌ All download methods failed."
echo ""
echo "Please manually download the font:"
echo ""
echo "  1. Open: https://fonts.google.com/noto/specimen/Noto+Sans+SC"
echo "  2. Click 'Download font'"
echo "  3. Extract the OTF file to: $FONT_PATH"
echo ""
echo "Or on macOS, copy the system CJK font:"
echo "  cp /System/Library/Fonts/Supplemental/NISC18030.ttf $FONTS_DIR/"
echo "  (Note: NISC18030 is a bitmap font, quality may be lower)"
exit 1
