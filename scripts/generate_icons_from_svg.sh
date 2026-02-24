#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SRC_ICON="${ROOT_DIR}/assets/snapmark-icon.svg"
OUT_ICON="${ROOT_DIR}/assets/icon.png"
OUT_STATUS_ICON="${ROOT_DIR}/assets/status_icon_template.png"
OUT_ICNS="${ROOT_DIR}/assets/icon.icns"
PYTHON_BIN="${PYTHON_BIN:-python3}"

if [[ ! -f "${SRC_ICON}" ]]; then
  echo "Missing source SVG: ${SRC_ICON}" >&2
  exit 1
fi

if ! command -v rsvg-convert >/dev/null 2>&1; then
  echo "Missing dependency: rsvg-convert (librsvg)." >&2
  exit 1
fi

TMP_DIR="$(mktemp -d)"
cleanup() {
  rm -rf "${TMP_DIR}"
}
trap cleanup EXIT

RAW_1024="${TMP_DIR}/raw_1024.png"
RAW_64="${TMP_DIR}/raw_64.png"

rsvg-convert -w 1024 -h 1024 "${SRC_ICON}" -o "${RAW_1024}"
rsvg-convert -w 64 -h 64 "${SRC_ICON}" -o "${RAW_64}"

"${PYTHON_BIN}" - "${RAW_1024}" "${RAW_64}" "${OUT_ICON}" "${OUT_STATUS_ICON}" "${OUT_ICNS}" <<'PY'
from PIL import Image, ImageDraw, ImageFilter
import sys

raw_1024 = sys.argv[1]
raw_64 = sys.argv[2]
out_icon = sys.argv[3]
out_status = sys.argv[4]
out_icns = sys.argv[5]


def symbol_from(path: str) -> Image.Image:
    src = Image.open(path).convert("RGBA")
    bbox = src.getbbox()
    if bbox is None:
        raise RuntimeError(f"Rendered SVG has no visible pixels: {path}")
    return src.crop(bbox)


def make_app_icon(symbol: Image.Image) -> Image.Image:
    size = 1024
    radius = 226
    canvas = Image.new("RGBA", (size, size), (0, 0, 0, 0))

    # Deep-blue rounded background that keeps contrast on both light/dark surroundings.
    bg = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    bg_draw = ImageDraw.Draw(bg, "RGBA")
    top = (13, 18, 44)
    bottom = (20, 31, 70)
    for y in range(size):
        t = y / (size - 1)
        r = int(top[0] * (1 - t) + bottom[0] * t)
        g = int(top[1] * (1 - t) + bottom[1] * t)
        b = int(top[2] * (1 - t) + bottom[2] * t)
        bg_draw.line([(0, y), (size, y)], fill=(r, g, b, 255), width=1)

    mask = Image.new("L", (size, size), 0)
    mask_draw = ImageDraw.Draw(mask)
    mask_draw.rounded_rectangle((64, 64, size - 64, size - 64), radius=radius, fill=255)
    canvas = Image.composite(bg, canvas, mask)

    border = ImageDraw.Draw(canvas, "RGBA")
    border.rounded_rectangle(
        (64, 64, size - 64, size - 64), radius=radius, outline=(255, 255, 255, 60), width=6
    )

    # Recolor source symbol to near-white for high legibility.
    symbol = symbol.resize((600, 600), Image.Resampling.LANCZOS)
    alpha = symbol.split()[-1]
    glyph = Image.new("RGBA", symbol.size, (240, 246, 255, 255))
    glyph.putalpha(alpha)

    # Soft drop shadow to keep edge clarity over any dock background.
    shadow = Image.new("RGBA", symbol.size, (0, 0, 0, 170))
    shadow.putalpha(alpha)
    shadow = shadow.filter(ImageFilter.GaussianBlur(8))
    x = (size - symbol.width) // 2
    y = (size - symbol.height) // 2
    canvas.alpha_composite(shadow, (x, y + 8))
    canvas.alpha_composite(glyph, (x, y))
    return canvas


def make_status_template(symbol: Image.Image) -> Image.Image:
    # Keep menu-bar icon simple and strong, then rely on NSImage template tint.
    out = Image.new("RGBA", (64, 64), (0, 0, 0, 0))
    symbol.thumbnail((50, 50), Image.Resampling.LANCZOS)
    alpha = symbol.split()[-1]
    alpha = alpha.filter(ImageFilter.MaxFilter(3))
    alpha = alpha.point(lambda a: min(255, int(a * 1.4)))

    x = (64 - symbol.width) // 2
    y = (64 - symbol.height) // 2
    black = Image.new("RGBA", symbol.size, (0, 0, 0, 255))
    black.putalpha(alpha)
    out.alpha_composite(black, (x, y))
    return out


symbol_1024 = symbol_from(raw_1024)
symbol_64 = symbol_from(raw_64)
app_icon = make_app_icon(symbol_1024)
status_icon = make_status_template(symbol_64)

app_icon.save(out_icon, format="PNG")
status_icon.save(out_status, format="PNG")

img = app_icon.convert("RGBA")
sizes = [(16, 16), (32, 32), (64, 64), (128, 128), (256, 256), (512, 512), (1024, 1024)]
img.save(out_icns, format="ICNS", sizes=sizes)
PY

echo "Generated:"
echo "  ${OUT_ICON}"
echo "  ${OUT_STATUS_ICON}"
echo "  ${OUT_ICNS}"
