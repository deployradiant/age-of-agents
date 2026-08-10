#!/usr/bin/env python3
"""Normalize generated two-panel resource activity sheets into runtime sprites."""

from pathlib import Path
from urllib.request import urlopen

from PIL import Image, ImageChops, ImageCms, ImageFilter

ROOT = Path(__file__).resolve().parents[1]
GENERATED = ROOT / "assets" / "generated"
OUTPUT = ROOT / "assets" / "game"
SHEETS = {
    "wood": "https://v3b.fal.media/files/b/0aa5b646/iOzIVtCUWVSX2GLn4UJlh_QmQOuvYt.png",
    "food": "https://v3b.fal.media/files/b/0aa5b646/51-tfLyH_fcsYXOSTS2ul_odkBRZhq.png",
    "stone": "https://v3b.fal.media/files/b/0aa5b646/T7zdJ5gh1aH4zt4vnK69q_w2STZYuO.png",
    "gold": "https://v3b.fal.media/files/b/0aa5b646/BWC2PNyo22z02ySfEqKXr_1QlANsYH.png",
    "iron": "https://v3b.fal.media/files/b/0aa5b647/tTWV79u8qR0V_70TXwEVX_qXxgIueb.png",
    "clay": "https://v3b.fal.media/files/b/0aa5b647/nx_lGewoUwjjdQeEvBIbk_dDWR5DZi.png",
    "fiber": "https://v3b.fal.media/files/b/0aa5b647/AU_Te_ULCZRQwgq_4WXAC_fFcwwanW.png",
}


def remove_white(image: Image.Image) -> Image.Image:
    rgb = image.convert("RGB")
    difference = ImageChops.difference(rgb, Image.new("RGB", rgb.size, "white"))
    alpha = difference.convert("L").point(lambda value: 0 if value < 18 else min(255, (value - 18) * 8))
    alpha = alpha.filter(ImageFilter.MedianFilter(3))
    rgba = rgb.convert("RGBA")
    rgba.putalpha(alpha)
    return rgba


def normalize(cell: Image.Image) -> Image.Image:
    bounds = cell.getchannel("A").getbbox()
    if bounds is None:
        raise ValueError("empty generated frame")
    sprite = cell.crop(bounds)
    scale = min(220 / sprite.width, 214 / sprite.height)
    sprite = sprite.resize((round(sprite.width * scale), round(sprite.height * scale)), Image.Resampling.LANCZOS)
    sprite.putalpha(sprite.getchannel("A").point(lambda value: 0 if value < 18 else value))
    canvas = Image.new("RGBA", (256, 256), (0, 0, 0, 0))
    canvas.alpha_composite(sprite, ((256 - sprite.width) // 2, 224 - sprite.height))
    # Dilate visible edge color into transparent texels to avoid dark/white fringes.
    pixels = canvas.load()
    alpha = canvas.getchannel("A")
    for _ in range(2):
        prior = canvas.copy()
        prior_pixels = prior.load()
        prior_alpha = prior.getchannel("A")
        for y in range(1, 255):
            for x in range(1, 255):
                if alpha.getpixel((x, y)) != 0:
                    continue
                for nx, ny in ((x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)):
                    if prior_alpha.getpixel((nx, ny)) > 8:
                        r, g, b, _ = prior_pixels[nx, ny]
                        pixels[x, y] = (r, g, b, 0)
                        break
        alpha = canvas.getchannel("A")
    return canvas


def main() -> None:
    GENERATED.mkdir(parents=True, exist_ok=True)
    OUTPUT.mkdir(parents=True, exist_ok=True)
    profile = ImageCms.ImageCmsProfile(ImageCms.createProfile("sRGB")).tobytes()
    for kind, url in SHEETS.items():
        sheet_path = GENERATED / f"resource_activity_{kind}_sheet.png"
        if not sheet_path.exists():
            sheet_path.write_bytes(urlopen(url, timeout=60).read())
        sheet = Image.open(sheet_path)
        width, height = sheet.size
        for index in range(2):
            cell = sheet.crop((index * width // 2, 0, (index + 1) * width // 2, height))
            output = normalize(remove_white(cell))
            output.save(OUTPUT / f"agent_gather_{kind}_{index + 1:02d}.png", optimize=True, icc_profile=profile)


if __name__ == "__main__":
    main()
