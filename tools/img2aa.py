#!/usr/bin/env python3
"""画像をASCIIアートに変換するスクリプト。

Usage:
    python3 tools/img2aa.py <image_path> --halfblock [--width 20]
    python3 tools/img2aa.py <image_path> [--width 40] [--chars " .:-=+*#%@"]
"""

import argparse
import sys
from PIL import Image, ImageOps


def image_to_halfblock(image_path: str, width: int = 20, threshold: int = 128) -> str:
    """ハーフブロック変換。1文字 = 2ピクセル縦。正方形ピクセル。"""
    img = Image.open(image_path).convert("L")

    # 余白クロップ
    inv = ImageOps.invert(img)
    bbox = inv.getbbox()
    if bbox:
        img = img.crop(bbox)

    aspect = img.height / img.width
    height = round(width * aspect)
    if height % 2 != 0:
        height += 1
    img = img.resize((width, height), Image.NEAREST)

    lines = []
    for y in range(0, height, 2):
        line = []
        for x in range(width):
            top = img.getpixel((x, y)) < threshold
            bot = img.getpixel((x, y + 1)) < threshold if y + 1 < height else False
            if top and bot:
                line.append("█")
            elif top:
                line.append("▀")
            elif bot:
                line.append("▄")
            else:
                line.append(" ")
        lines.append("".join(line))

    # trim
    while lines and lines[-1].strip() == "":
        lines.pop()
    while lines and lines[0].strip() == "":
        lines.pop(0)
    lines = [line.rstrip() for line in lines]

    return "\n".join(lines)


def image_to_aa(image_path: str, width: int = 40, chars: str = " .:-=+*#%@", invert: bool = False) -> str:
    """従来の輝度ベース変換。"""
    img = Image.open(image_path).convert("L")

    inv = ImageOps.invert(img)
    bbox = inv.getbbox()
    if bbox:
        img = img.crop(bbox)

    aspect = img.height / img.width
    height = int(width * aspect * 0.5)
    img = img.resize((width, height))

    lines = []
    for y in range(height):
        line = []
        for x in range(width):
            v = img.getpixel((x, y))
            if invert:
                v = 255 - v
            idx = int((v / 255.0) * (len(chars) - 1))
            idx = max(0, min(idx, len(chars) - 1))
            line.append(chars[idx])
        lines.append("".join(line))

    while lines and lines[-1].strip() == "":
        lines.pop()
    while lines and lines[0].strip() == "":
        lines.pop(0)
    lines = [line.rstrip() for line in lines]

    return "\n".join(lines)


def main():
    parser = argparse.ArgumentParser(description="画像をASCIIアートに変換")
    parser.add_argument("image", help="入力画像パス")
    parser.add_argument("--width", type=int, default=20, help="AA の幅（文字数）")
    parser.add_argument("--halfblock", action="store_true", help="ハーフブロック変換（正方形ピクセル）")
    parser.add_argument("--threshold", type=int, default=128, help="白黒しきい値 (0-255)")
    parser.add_argument("--chars", default=" .:-=+*#%@", help="使用する文字セット（暗→明）")
    parser.add_argument("--invert", action="store_true", help="明暗を反転")
    parser.add_argument("--output", "-o", help="出力ファイル（省略時は stdout）")
    parser.add_argument("--rust", action="store_true", help="Rust 定数形式で出力")

    args = parser.parse_args()

    if args.halfblock:
        aa = image_to_halfblock(args.image, width=args.width, threshold=args.threshold)
    else:
        aa = image_to_aa(args.image, width=args.width, chars=args.chars, invert=args.invert)

    if args.rust:
        # Rust 文字列定数形式に変換
        rust_lines = []
        for line in aa.split("\n"):
            rust_lines.append(f"\\n{line}")
        rust_str = "\\".join(rust_lines)
        aa = f'    "\\{rust_str}\\n"'

    if args.output:
        with open(args.output, "w") as f:
            f.write(aa + "\n")
        print(f"Saved to {args.output}", file=sys.stderr)
    else:
        print(aa)


if __name__ == "__main__":
    main()
