#!/usr/bin/env python3
"""Objective production checks for the depleted-tree sprite."""

from pathlib import Path

import numpy as np
from PIL import Image


ASSET = Path(__file__).parents[1] / "assets/game/resource_tree_depleted.png"


def main() -> None:
    image = Image.open(ASSET)
    assert image.format == "PNG", image.format
    assert image.size == (256, 256), image.size
    assert image.mode == "RGBA", image.mode
    assert image.info.get("icc_profile"), "missing embedded sRGB profile"

    pixels = np.asarray(image)
    alpha = pixels[:, :, 3]
    assert np.all(alpha[[0, -1], :] == 0), "top/bottom margins must be transparent"
    assert np.all(alpha[:, [0, -1]] == 0), "left/right margins must be transparent"
    bbox = image.getchannel("A").getbbox()
    assert bbox is not None
    left, top, right, bottom = bbox
    center = (left + right) / 2
    assert abs(center - 128) <= 2, f"off-center alpha bounds: {bbox}"
    assert 238 <= bottom <= 246, f"bottom anchor outside expected band: {bbox}"
    assert left >= 12 and top >= 12 and right <= 244, f"insufficient margin: {bbox}"

    transparent = alpha == 0
    edge = np.zeros_like(transparent)
    edge[1:, :] |= alpha[:-1, :] > 8
    edge[:-1, :] |= alpha[1:, :] > 8
    edge[:, 1:] |= alpha[:, :-1] > 8
    edge[:, :-1] |= alpha[:, 1:] > 8
    edge &= transparent
    # Fully transparent edge texels must carry one neighboring visible color,
    # including legitimate near-black ink outlines.
    matches_visible_neighbor = np.zeros_like(transparent)
    for dy, dx in ((-1, 0), (1, 0), (0, -1), (0, 1)):
        shifted_alpha = np.roll(alpha, (dy, dx), axis=(0, 1))
        shifted_rgb = np.roll(pixels[:, :, :3], (dy, dx), axis=(0, 1))
        matches_visible_neighbor |= (shifted_alpha > 8) & np.all(
            pixels[:, :, :3] == shifted_rgb, axis=2
        )
    assert np.all(matches_visible_neighbor[edge]), "edge RGB is not color-dilated"

    print(
        f"PASS {ASSET}: 256x256 8-bit sRGB RGBA; alpha_bbox={bbox}; "
        f"alpha_pixels={int(np.count_nonzero(alpha))}; "
        f"semi_transparent={int(np.count_nonzero((alpha > 0) & (alpha < 255)))}; "
        f"anchor=({center:.1f},{bottom})"
    )


if __name__ == "__main__":
    main()