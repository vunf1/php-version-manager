#!/usr/bin/env python3
"""
Build Tauri bundle icons from a single master PNG.

Source (first match):
  - icon_src.png  — use for repeated runs so exports stay sharp (optional)
  - icon.png      — default master (wide logos are letterboxed to a square)

Writes (tauri.conf.json bundle.icon):
  icon.png (512² for hi-res), 32x32.png, 128x128.png, 128x128@2x.png, icon.ico, php.ico

Pipeline: trim transparent margins → letterbox to square (no crop when _CONTENT_ZOOM is 1.0).
If you set _CONTENT_ZOOM above 1.0, the script scales up then center-crops, which cuts the
sides off wide wordmarks — use 1.0 to keep the full logo. Then LANCZOS to each size; ICO:
32px first, layers through 512px for high-DPI Windows.

Requires: pip install -r requirements.txt  (Pillow)
"""
from __future__ import annotations

import io
import sys
from pathlib import Path

try:
    from PIL import Image
except ImportError:
    print(
        "Missing Pillow. From this folder run:\n"
        "  pip install -r requirements.txt\n"
        "or: pip install pillow",
        file=sys.stderr,
    )
    sys.exit(1)


def _load_source_bytes(script_dir: Path) -> tuple[Path, bytes]:
    src = script_dir / "icon_src.png"
    if not src.is_file():
        src = script_dir / "icon.png"
    if not src.is_file():
        raise FileNotFoundError(
            f"No icon source found. Add {script_dir / 'icon.png'} "
            f"(or high-res {script_dir / 'icon_src.png'})."
        )
    return src, src.read_bytes()


# Pixels with alpha <= this are treated as empty for the bounding box (keeps antialiasing).
_TRIM_ALPHA_THRESHOLD = 10

# 1.0 = letterbox only (full logo visible). >1.0 zooms then crops — trims wide phpvm-style marks.
_CONTENT_ZOOM = 1.0

# Primary bundle PNG (Tauri docs also mention 512 for desktop quality).
_ICON_PNG_PX = 512


def _trim_transparent_margins(im: Image.Image, alpha_threshold: int = _TRIM_ALPHA_THRESHOLD) -> Image.Image:
    """Crop to the bounding box of visible pixels so fixed-size icons use space for art, not blank alpha."""
    im = im.convert("RGBA")
    alpha = im.split()[3]
    mask = alpha.point(lambda p: 255 if p > alpha_threshold else 0)
    bbox = mask.getbbox()
    if bbox is None:
        return im
    w0, h0 = im.size
    if bbox == (0, 0, w0, h0):
        return im
    return im.crop(bbox)


def _to_square_rgba(im: Image.Image, content_zoom: float = _CONTENT_ZOOM) -> Image.Image:
    """Square side S=max(w,h). Scale by min(S/w,S/h)*content_zoom, then center crop or letterbox."""
    im = im.convert("RGBA")
    w, h = im.size
    if w < 1 or h < 1:
        return im
    if content_zoom <= 1.0 and w == h:
        return im
    S = max(w, h)
    if content_zoom <= 1.0:
        side = S
        canvas = Image.new("RGBA", (side, side), (0, 0, 0, 0))
        canvas.paste(im, ((side - w) // 2, (side - h) // 2), im)
        return canvas

    k_fit = min(S / w, S / h)
    k = k_fit * content_zoom
    nw = max(1, int(round(w * k)))
    nh = max(1, int(round(h * k)))
    scaled = im.resize((nw, nh), Image.Resampling.LANCZOS)

    if nw >= S and nh >= S:
        left = (nw - S) // 2
        top = (nh - S) // 2
        return scaled.crop((left, top, left + S, top + S))
    if nw >= S:
        left = (nw - S) // 2
        stripe = scaled.crop((left, 0, left + S, nh))
        out = Image.new("RGBA", (S, S), (0, 0, 0, 0))
        out.paste(stripe, (0, (S - nh) // 2))
        return out
    if nh >= S:
        top = (nh - S) // 2
        stripe = scaled.crop((0, top, nw, top + S))
        out = Image.new("RGBA", (S, S), (0, 0, 0, 0))
        out.paste(stripe, ((S - nw) // 2, 0))
        return out
    out = Image.new("RGBA", (S, S), (0, 0, 0, 0))
    out.paste(scaled, ((S - nw) // 2, (S - nh) // 2), scaled)
    return out


def _resize(im: Image.Image, size: int) -> Image.Image:
    # LANCZOS: high-quality downscale; PNG output stays lossless (no JPEG-style artifacts).
    return im.resize((size, size), Image.Resampling.LANCZOS)


def _save_ico(path: Path, square: Image.Image) -> None:
    # Tauri / Windows: include 16,24,32,48,64,256; 32px first. Extra 512 helps sharp high-DPI shells.
    sizes = (32, 48, 64, 128, 256, 512, 24, 16)
    images: list[Image.Image] = []
    for s in sizes:
        images.append(_resize(square, s))
    images[0].save(
        path,
        format="ICO",
        sizes=[(img.width, img.height) for img in images],
        append_images=images[1:],
    )
    print(f"Wrote {path} ({', '.join(str(s) for s in sizes)} px, 32px first)")


def main() -> int:
    script_dir = Path(__file__).resolve().parent
    try:
        used_path, raw = _load_source_bytes(script_dir)
    except FileNotFoundError as e:
        print(e, file=sys.stderr)
        return 1

    print(f"Source: {used_path.name} ({len(raw)} bytes)")

    im = Image.open(io.BytesIO(raw))
    w0, h0 = im.size
    trimmed = _trim_transparent_margins(im)
    if trimmed.size != (w0, h0):
        print(
            f"Trimmed transparent margins: {w0}x{h0} -> "
            f"{trimmed.size[0]}x{trimmed.size[1]} (alpha > {_TRIM_ALPHA_THRESHOLD})"
        )
    square = _to_square_rgba(trimmed, _CONTENT_ZOOM)
    if _CONTENT_ZOOM <= 1.0:
        print(f"Square canvas {square.size[0]}x{square.size[1]} (letterbox, no crop)")
    else:
        print(
            f"Square canvas {square.size[0]}x{square.size[1]} "
            f"(content zoom {_CONTENT_ZOOM}x, may crop wide logos)"
        )

    png_jobs = [
        (_ICON_PNG_PX, "icon.png"),
        (32, "32x32.png"),
        (128, "128x128.png"),
        (256, "128x128@2x.png"),
    ]
    for size, name in png_jobs:
        out = script_dir / name
        _resize(square, size).save(out, format="PNG", optimize=True)
        print(f"Wrote {name} ({size}x{size})")

    ico_path = script_dir / "icon.ico"
    php_ico_path = script_dir / "php.ico"
    _save_ico(ico_path, square)
    _save_ico(php_ico_path, square)

    print("All Tauri bundle icons updated.")
    if used_path.name == "icon.png":
        print(
            "Tip: for sharpest re-runs, save your master as icon_src.png "
            "and keep icon.png as generated output."
        )
    return 0


if __name__ == "__main__":
    sys.exit(main())
