#!/bin/sh
# img2aa.sh - Convert image to halfblock ASCII art
# Usage: img2aa.sh IMAGE [WIDTH]
# Requires: ImageMagick (magick command)

set -e

IMG="$1"
W="${2:-20}"

if [ -z "$IMG" ] || [ ! -f "$IMG" ]; then
  echo "Usage: $0 IMAGE [WIDTH]" >&2
  exit 1
fi

tmp_pbm=$(mktemp -t img2aa.XXXXXX).pbm

# Strategy (mirrors img2aa.py):
# 1. grayscale → threshold (pure 1-bit)
# 2. trim margin
# 3. get trimmed dims, calculate target height from aspect ratio
# 4. make height even (halfblock = 2 rows/char)
# 5. resize with Box filter (area average)
# 6. re-threshold to prevent gray from reappearing
# 7. output PBM P1 (ASCII)

# Step 1+2: binarize then trim
magick "$IMG" -colorspace Gray -threshold 50% -trim +repage miff:- |
  {
    # Step 3: get trimmed dimensions
    tmp_miff=$(mktemp -t img2aa.XXXXXX).miff
    cat > "$tmp_miff"
    dims=$(magick identify -format "%w %h" "$tmp_miff")
    ow=$(echo "$dims" | awk '{print $1}')
    oh=$(echo "$dims" | awk '{print $2}')
    # Step 4: compute even height from aspect
    H=$(awk -v w="$W" -v ow="$ow" -v oh="$oh" 'BEGIN {
      h = oh * w / ow
      h = int(h + 0.5)
      if (h % 2 != 0) h++
      print h
    }')
    # Step 5+6+7
    magick "$tmp_miff" \
      -filter Box -resize "${W}x${H}!" \
      -threshold 50% \
      -compress none \
      "$tmp_pbm"
    rm -f "$tmp_miff"
  }

# Parse PBM: first line is "P1", second line is "W H", rest is pixels
# Pad height to even by adding a row of zeros if needed

awk -v W="$W" '
  NR==1 { next }  # P1
  NR==2 {
    # width height (may span multiple tokens after comments stripped)
    real_w = $1
    real_h = $2
    next
  }
  {
    for (i=1; i<=NF; i++) {
      pixels[count++] = $i
    }
  }
  END {
    # pad to even height
    H = real_h
    if (H % 2 != 0) {
      for (x=0; x<real_w; x++) pixels[count++] = "0"
      H++
    }
    for (y=0; y<H; y+=2) {
      line = ""
      for (x=0; x<real_w; x++) {
        t = pixels[y*real_w+x]
        b = pixels[(y+1)*real_w+x]
        # In PBM, 1 = black (filled), 0 = white (empty)
        if (t=="1" && b=="1") line = line "█"
        else if (t=="1") line = line "▀"
        else if (b=="1") line = line "▄"
        else line = line " "
      }
      # trim trailing spaces
      sub(/ +$/, "", line)
      print line
    }
  }
' "$tmp_pbm"

rm -f "$tmp_pbm"
