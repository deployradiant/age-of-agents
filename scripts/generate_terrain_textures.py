#!/usr/bin/env python3
"""Generate the eight production terrain textures through FAL.

Outputs are converted to seamless 96x96, 8-bit sRGB RGBA PNGs. Credentials are
read from FAL_KEY or ~/.hermes/.env and are never written to the repository.
"""

from __future__ import annotations

import io
import os
from pathlib import Path

import httpx
from PIL import Image, ImageCms

FAL_URL = "https://fal.run/fal-ai/flux/schnell"
OUTPUT_DIR = Path("assets/game")
STYLE = (
    "Locked mood_board_v3 art direction: warm pastoral Japanese cel animation "
    "with refined European-comic charcoal-brown ink marks, flat muted colors, "
    "restrained texture, NOT realistic, NOT 3D, NOT painterly. "
)
TERRAINS = {
    "meadow": "pale sage meadow, tiny sparse cream wildflower flecks and short fine grass strokes",
    "grassland": "medium warm sage grassland, gently varied short grass strokes and restrained olive patches",
    "forest": "cool deep sage forest floor, small leaf silhouettes, moss marks and restrained fern-like strokes, no trees",
    "deep_forest": "dark muted blue-green forest floor, dense tiny leaf-litter curls, moss marks and fine ink hatching, no trees",
    "dirt": "warm light ochre packed earth, sparse fine dry cracks and short brown contour scratches",
    "scrub": "dry olive-tan ground, tiny low thorny scrub marks and sparse straw-colored grass tufts, no large bushes",
    "rock": "muted warm gray-brown ground, many small flat angular stone motifs with thin charcoal contours, no boulders",
    "highland": "cool desaturated slate-green highland ground, fine wind-swept hatching, tiny pale stone flecks and hardy grass marks",
}


def fal_key() -> str:
    key = os.environ.get("FAL_KEY", "")
    if key:
        return key
    env_file = Path.home() / ".hermes" / ".env"
    if env_file.exists():
        for line in env_file.read_text().splitlines():
            if line.startswith("FAL_KEY="):
                return line.split("=", 1)[1].strip()
    raise RuntimeError("FAL_KEY is not configured")


def generate(key: str, description: str) -> tuple[bytes, str]:
    prompt = (
        STYLE
        + "Single top-down production terrain texture for a 2D isometric RTS: "
        + description
        + ". Edge-to-edge square texture, subtle detail readable at 96x96, "
        "no perspective, horizon, paths, objects, text, border, frame, matte, "
        "lighting gradient, or cast shadows. Seamless/tileable on all four edges."
    )
    response = httpx.post(
        FAL_URL,
        headers={"Authorization": f"Key {key}", "Content-Type": "application/json"},
        json={
            "prompt": prompt,
            "image_size": {"width": 1024, "height": 1024},
            "num_images": 1,
            "enable_safety_checker": False,
        },
        timeout=180,
    )
    response.raise_for_status()
    url = response.json()["images"][0]["url"]
    download = httpx.get(url, timeout=60)
    download.raise_for_status()
    return download.content, url


def seamless_rgba(source: bytes) -> Image.Image:
    image = Image.open(io.BytesIO(source)).convert("RGB")
    side = min(image.size)
    left = (image.width - side) // 2
    top = (image.height - side) // 2
    image = image.crop((left, top, left + side, top + side))

    # A mirrored 2x2 composition guarantees matching opposite edges without
    # inventing transparent overlap or runtime edge bleed.
    quarter = image.resize((48, 48), Image.Resampling.LANCZOS)
    output = Image.new("RGB", (96, 96))
    output.paste(quarter, (0, 0))
    output.paste(quarter.transpose(Image.Transpose.FLIP_LEFT_RIGHT), (48, 0))
    output.paste(quarter.transpose(Image.Transpose.FLIP_TOP_BOTTOM), (0, 48))
    output.paste(quarter.transpose(Image.Transpose.ROTATE_180), (48, 48))
    return output.convert("RGBA")


def main() -> None:
    key = fal_key()
    OUTPUT_DIR.mkdir(parents=True, exist_ok=True)
    srgb = ImageCms.ImageCmsProfile(ImageCms.createProfile("sRGB")).tobytes()
    provenance = []
    for name, description in TERRAINS.items():
        print(f"Generating {name}...", flush=True)
        source, url = generate(key, description)
        destination = OUTPUT_DIR / f"terrain_{name}.png"
        seamless_rgba(source).save(destination, "PNG", icc_profile=srgb, compress_level=9)
        provenance.append(f"- `{destination}` — {url}")
        print(f"Saved {destination}", flush=True)
    Path("assets/generated/terrain_voronoi_sources.md").write_text(
        "# Voronoi terrain texture sources\n\n"
        "Generated through FAL with `scripts/generate_terrain_textures.py`. "
        "Temporary result URLs are retained only as provenance; runtime files "
        "were downloaded immediately.\n\n" + "\n".join(provenance) + "\n"
    )


if __name__ == "__main__":
    main()
