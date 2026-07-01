#!/usr/bin/env bash
set -euo pipefail

# Regenerate the CJK font subset for PDF export.
#
# Prerequisites:
#   pip install fonttools
#
# Usage:
#   cd ai_resume_aid          # project root (where backend/ and scripts/ live)
#   bash scripts/subset-font.sh
#
# The original NotoSansCJKsc-Regular.otf (16MB) is expected at:
#   backend/fonts/NotoSansCJKsc-Regular.otf
#
# Output:
#   backend/fonts/NotoSansCJKsc-Regular-subset.otf  (≈11MB)

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

INPUT="$PROJECT_DIR/backend/fonts/NotoSansCJKsc-Regular.otf"
OUTPUT="$PROJECT_DIR/backend/fonts/NotoSansCJKsc-Regular-subset.otf"

if [ ! -f "$INPUT" ]; then
  echo "ERROR: Original font not found at $INPUT"
  echo "Download from: https://github.com/notofonts/noto-cjk/releases/"
  exit 1
fi

# Unicode ranges for the subset:
#   U+0020-007E   ASCII (numbers, basic Latin, resume-friendly)
#   U+3000-303F   CJK Symbols and Punctuation
#   U+2000-206F   General Punctuation (bullet •, em dash —, etc.)
#   U+4E00-9FFF   CJK Unified Ideographs (all common Chinese characters)
UNICODES="U+0020-007E,U+3000-303F,U+2000-206F,U+4E00-9FFF"

echo "→ Generating subset font..."
echo "  Input:  $INPUT"
echo "  Output: $OUTPUT"
echo "  Ranges: $UNICODES"

pyftsubset "$INPUT" \
  --unicodes="$UNICODES" \
  --output-file="$OUTPUT" \
  --flavor=

echo "✓ Done"
ls -lh "$OUTPUT"
