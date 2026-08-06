#!/usr/bin/env python3
"""Build normalized runtime assets from the generated 2x2 source sheets."""

from pathlib import Path

from PIL import Image, ImageChops

ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / "assets" / "generated"
OUTPUT = ROOT / "assets" / "game"


def _matte_alpha(value: int) -> int:
    return 0 if value <= 7 else min(255, (value - 7) * 16)


def _drop_generation_noise(value: int) -> int:
    return 0 if value < 48 else value


def _drop_resize_noise(value: int) -> int:
    return 0 if value < 18 else value


def remove_white_matte(image: Image.Image) -> Image.Image:
    rgb = image.convert("RGB")
    difference = ImageChops.difference(rgb, Image.new("RGB", rgb.size, "white"))
    alpha = difference.convert("L").point(_matte_alpha)
    alpha = alpha.point(_drop_generation_noise)
    rgba = rgb.convert("RGBA")
    rgba.putalpha(alpha)
    return rgba


def normalize(
    image: Image.Image,
    canvas: tuple[int, int],
    maximum: tuple[int, int],
    baseline: float,
) -> Image.Image:
    bounds = image.getchannel("A").getbbox()
    if bounds is None:
        raise ValueError("generated cell contains no visible asset")
    cropped = image.crop(bounds)
    scale = min(maximum[0] / cropped.width, maximum[1] / cropped.height)
    size = (round(cropped.width * scale), round(cropped.height * scale))
    cropped = cropped.resize(size, Image.Resampling.LANCZOS)
    cropped.putalpha(cropped.getchannel("A").point(_drop_resize_noise))

    output = Image.new("RGBA", canvas, (0, 0, 0, 0))
    position = (
        (canvas[0] - cropped.width) // 2,
        round(canvas[1] * baseline) - cropped.height,
    )
    output.alpha_composite(cropped, position)
    return output


def cells(sheet: Image.Image):
    width, height = sheet.size
    for row in range(2):
        for column in range(2):
            yield sheet.crop(
                (
                    column * width // 2,
                    row * height // 2,
                    (column + 1) * width // 2,
                    (row + 1) * height // 2,
                )
            )


def build() -> None:
    OUTPUT.mkdir(parents=True, exist_ok=True)

    agent_sheet = Image.open(SOURCE / "agent_sheet.png")
    agent_specs = [
        ("agent_idle.png", (190, 214)),
        ("agent_walk_01.png", (190, 214)),
        ("agent_walk_02.png", (190, 214)),
        ("agent_gather.png", (242, 214)),
    ]
    for cell, (name, maximum) in zip(cells(agent_sheet), agent_specs, strict=True):
        normalize(remove_white_matte(cell), (256, 256), maximum, 0.91).save(
            OUTPUT / name, optimize=True
        )

    props_sheet = Image.open(SOURCE / "props_sheet.png")
    props_specs = [
        ("resource_tree.png", (256, 256), (220, 220), 0.93),
        ("building_town_center.png", (512, 512), (440, 400), 0.90),
        ("command_build.png", (128, 128), (108, 108), 0.90),
        ("command_cancel.png", (128, 128), (104, 104), 0.88),
    ]
    for cell, (name, canvas, maximum, baseline) in zip(
        cells(props_sheet), props_specs, strict=True
    ):
        normalize(remove_white_matte(cell), canvas, maximum, baseline).save(
            OUTPUT / name, optimize=True
        )

    terrain_sheet = Image.open(SOURCE / "terrain_sheet.png").convert("RGB")
    terrain_names = [
        "tile_grass_01.png",
        "tile_grass_02.png",
        "tile_grass_03.png",
        "tile_dirt.png",
    ]
    for cell, name in zip(cells(terrain_sheet), terrain_names, strict=True):
        inset = 5
        cell = cell.crop((inset, inset, cell.width - inset, cell.height - inset))
        cell.resize((96, 96), Image.Resampling.LANCZOS).save(
            OUTPUT / name, optimize=True
        )


if __name__ == "__main__":
    build()
