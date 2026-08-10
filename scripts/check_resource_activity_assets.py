#!/usr/bin/env python3
"""Validate the two-frame resource-specific gathering sprite contract."""

from pathlib import Path

from PIL import Image, ImageChops

ROOT = Path(__file__).resolve().parents[1]
KINDS = ("wood", "food", "stone", "gold", "iron", "clay", "fiber")

for kind in KINDS:
    frames = []
    bounds = []
    for number in (1, 2):
        path = ROOT / "assets" / "game" / f"agent_gather_{kind}_{number:02d}.png"
        image = Image.open(path)
        if image.mode != "RGBA" or image.size != (256, 256):
            raise SystemExit(f"FAIL {path.name}: expected 256x256 RGBA, got {image.size} {image.mode}")
        alpha = image.getchannel("A")
        box = alpha.getbbox()
        if box is None or alpha.getextrema() != (0, 255):
            raise SystemExit(f"FAIL {path.name}: missing full-range transparency")
        if any(alpha.getpixel(corner) != 0 for corner in ((0, 0), (255, 0), (0, 255), (255, 255))):
            raise SystemExit(f"FAIL {path.name}: corners must be transparent")
        frames.append(image)
        bounds.append(box)
    if bounds[0][3] != bounds[1][3]:
        raise SystemExit(f"FAIL {kind}: frame baselines differ: {bounds}")
    if ImageChops.difference(frames[0], frames[1]).getbbox() is None:
        raise SystemExit(f"FAIL {kind}: two frames are identical")
    print(f"PASS {kind}: bounds={bounds}")
