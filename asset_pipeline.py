#!/usr/bin/env python3
"""
Age of Agents — Asset Pipeline
===============================
Generates isometric game assets via FAL, then post-processes them.

Pipeline:
  1. Generate image via FAL flux-schnell (1024x1024, white bg)
  2. Download result
  3. Run birefnet background removal → RGBA PNG with transparency
  4. Edge-bleed: fill any remaining white/near-white border pixels
  5. Save as RGBA PNG to assets/isometric/

Usage:
  python3 asset_pipeline.py --prompt "..." --output iso_grass.png
  python3 asset_pipeline.py --prompt "..." --output iso_agent_idle.png --no-crop

Flags:
  --prompt TEXT     FAL generation prompt
  --output NAME     Output filename in assets/isometric/ (e.g. iso_grass.png)
  --no-crop         Skip edge-bleed (for sprites that need chroma key transparency)
  --no-birefnet     Skip background removal (if image is already transparent)
"""

import os, sys, json, tempfile, argparse
from pathlib import Path
import httpx

FAL_KEY = os.environ.get("FAL_KEY", "")
OUTPUT_DIR = Path("assets/isometric")
OUTPUT_DIR.mkdir(parents=True, exist_ok=True)

FAL_GEN_URL = "https://fal.run/fal-ai/flux/schnell"
FAL_BIR_URL = "https://fal.run/fal-ai/birefnet/v2"

# ── Prompt templates ──────────────────────────────────────────────────────

TILE_PROMPT = (
    "Princess Mononoke Studio Ghibli isometric game tile, "
    "dense dark ink outlines, hand-drawn cel animation, flat cel-shaded colors, "
    "{description}, "
    "content fills entire canvas edge to edge, no borders no margins no padding, "
    "seamless tileable texture, 64x64 pixel game art style, "
    "square frame, top-down game tile, NOT transparent NOT isolated on white, "
    "NOT realistic NOT 3D NOT painted"
)

SPRITE_PROMPT = (
    "Princess Mononoke Studio Ghibli style, "
    "dense dark ink outlines, hand-drawn cel animation, flat cel-shaded colors, "
    "visible pencil sketch lines, {description}, "
    "isolated on pure white background, game sprite, centered, "
    "NOT realistic NOT 3D NOT painted"
)


def fal_generate(prompt: str, output_path: str) -> str:
    """Generate image via FAL, returns URL to result."""
    payload = {
        "prompt": prompt,
        "image_size": {"width": 1024, "height": 1024},
        "num_images": 1,
        "enable_safety_checker": False,
    }
    print(f"  Generating via FAL...", flush=True)
    resp = httpx.post(
        FAL_GEN_URL,
        headers={"Authorization": f"Key {FAL_KEY}", "Content-Type": "application/json"},
        json=payload,
        timeout=120,
    )
    resp.raise_for_status()
    data = resp.json()
    img_url = data["images"][0]["url"]
    print(f"  Generated: {img_url[:60]}...", flush=True)
    return img_url


def download_image(url: str) -> bytes:
    """Download image bytes from URL."""
    resp = httpx.get(url, timeout=30)
    resp.raise_for_status()
    return resp.content


def birefnet_remove_bg(image_bytes: bytes) -> bytes:
    """Remove background via birefnet, returns RGBA PNG bytes."""
    print(f"  Removing background via birefnet...", flush=True)
    resp = httpx.post(
        FAL_BIR_URL,
        headers={"Authorization": f"Key {FAL_KEY}"},
        files={"image": ("input.png", image_bytes, "image/png")},
        timeout=60,
    )
    resp.raise_for_status()
    # birefnet returns a URL to the result
    result_url = resp.json()["image"]["url"]
    return download_image(result_url)


def edge_bleed_pil(image_bytes: bytes) -> bytes:
    """Fill white/near-white border pixels with nearest content color."""
    from PIL import Image, ImageDraw
    import io

    img = Image.open(io.BytesIO(image_bytes))
    w, h = img.size
    if img.mode == "RGBA":
        # Use RGB for edge detection
        rgb = img.convert("RGB")
        pixels = rgb.load()
    else:
        pixels = img.load()

    result = img.copy()
    if result.mode == "RGBA":
        rpixels = result.load()
        # Also create RGB version for edge detection
        rgb_result = result.convert("RGB")
        rpixels_rgb = rgb_result.load()
    else:
        result = img.copy()
        rpixels = result.load()
        rpixels_rgb = rpixels

    draw = ImageDraw.Draw(result)
    THRESHOLD = 240

    def is_white(p):
        return p[0] > THRESHOLD and p[1] > THRESHOLD and p[2] > THRESHOLD

    # Phase 1: fill left/right margins per row
    for y in range(h):
        left_x = 0
        while left_x < w and is_white(pixels[left_x, y]):
            left_x += 1
        if 0 < left_x < w:
            draw.rectangle([0, y, left_x - 1, y], fill=pixels[left_x, y])

        right_x = w - 1
        while right_x >= 0 and is_white(pixels[right_x, y]):
            right_x -= 1
        if right_x < w - 1 and right_x >= 0:
            draw.rectangle([right_x + 1, y, w - 1, y], fill=pixels[right_x, y])

    # Phase 2: fill top/bottom margins per column
    for x in range(w):
        top_y = 0
        while top_y < h and is_white(pixels[x, top_y]):
            top_y += 1
        if 0 < top_y < h:
            draw.rectangle([x, 0, x, top_y - 1], fill=pixels[x, top_y])

        bottom_y = h - 1
        while bottom_y >= 0 and is_white(pixels[x, bottom_y]):
            bottom_y -= 1
        if bottom_y < h - 1 and bottom_y >= 0:
            draw.rectangle([x, bottom_y + 1, x, h - 1], fill=pixels[x, bottom_y])

    buf = io.BytesIO()
    result.save(buf, format="PNG")
    return buf.getvalue()


def process_asset(prompt: str, output_name: str, do_birefnet: bool, do_bleed: bool):
    """Full pipeline: generate → (birefnet) → (edge-bleed) → save."""
    out_path = OUTPUT_DIR / output_name

    # 1. Generate
    img_url = fal_generate(prompt, str(out_path))
    img_bytes = download_image(img_url)

    # 2. Background removal (for sprites that need transparency)
    if do_birefnet:
        img_bytes = birefnet_remove_bg(img_bytes)

    # 3. Edge-bleed (for tiles — fills white borders with content)
    if do_bleed:
        from PIL import Image
        import io

        pre = Image.open(io.BytesIO(img_bytes))
        pre_size = pre.size
        img_bytes = edge_bleed_pil(img_bytes)

    # 4. Save
    with open(out_path, "wb") as f:
        f.write(img_bytes)

    from PIL import Image
    import io
    final = Image.open(io.BytesIO(img_bytes))
    print(f"  Saved {output_name} ({final.size[0]}x{final.size[1]}, mode={final.mode})")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Generate and post-process game assets")
    parser.add_argument("--prompt", required=True, help="FAL generation prompt")
    parser.add_argument("--output", required=True, help="Output filename (e.g. iso_grass.png)")
    parser.add_argument("--no-crop", action="store_true", help="Skip edge-bleed for sprites")
    parser.add_argument("--no-birefnet", action="store_true", help="Skip background removal")
    args = parser.parse_args()

    if not FAL_KEY:
        print("ERROR: FAL_KEY not set. Set it in ~/.hermes/.env or export it.")
        sys.exit(1)

    process_asset(
        prompt=args.prompt,
        output_name=args.output,
        do_birefnet=not args.no_birefnet,
        do_bleed=not args.no_crop,
    )
    print("Done.")