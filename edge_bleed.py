#!/usr/bin/env python3
"""Edge-bleed tile textures to remove white borders. Fast PIL-only approach."""

from PIL import Image, ImageDraw
import os

THRESHOLD = 240
iso_dir = 'assets/isometric'

def edge_bleed(img, threshold=THRESHOLD):
    w, h = img.size
    pixels = img.load()
    result = img.copy()
    draw = ImageDraw.Draw(result)
    rpixels = result.load()
    
    def is_white(p):
        return p[0] > threshold and p[1] > threshold and p[2] > threshold
    
    # Phase 1: Fill white margins on each row by scanning only from edges inward
    for y in range(h):
        # Scan from left edge inward
        left_x = 0
        while left_x < w and is_white(pixels[left_x, y]):
            left_x += 1
        if left_x > 0 and left_x < w:
            color = pixels[left_x, y]
            draw.rectangle([0, y, left_x - 1, y], fill=color)
        
        # Scan from right edge inward
        right_x = w - 1
        while right_x >= 0 and is_white(pixels[right_x, y]):
            right_x -= 1
        if right_x < w - 1 and right_x >= 0:
            color = pixels[right_x, y]
            draw.rectangle([right_x + 1, y, w - 1, y], fill=color)
    
    # Phase 2: Fill white margins on each column (using edge-filled result)
    for x in range(w):
        top_y = 0
        while top_y < h and is_white(rpixels[x, top_y]):
            top_y += 1
        if top_y > 0 and top_y < h:
            color = rpixels[x, top_y]
            draw.rectangle([x, 0, x, top_y - 1], fill=color)
        
        bottom_y = h - 1
        while bottom_y >= 0 and is_white(rpixels[x, bottom_y]):
            bottom_y -= 1
        if bottom_y < h - 1 and bottom_y >= 0:
            color = rpixels[x, bottom_y]
            draw.rectangle([x, bottom_y + 1, x, h - 1], fill=color)
    
    # Verify: count remaining white pixels
    remaining = 0
    for y in range(h):
        for x in range(w):
            if is_white(rpixels[x, y]):
                remaining += 1
    
    # Phase 3: If there are still white pixels (inside content), do iterative fill
    if remaining > 0:
        changed = True
        iteration = 0
        while changed and iteration < 50:
            changed = False
            iteration += 1
            fills = []
            for y in range(1, h - 1):
                for x in range(1, w - 1):
                    if is_white(rpixels[x, y]):
                        for dx, dy in [(0,1),(0,-1),(1,0),(-1,0)]:
                            np_ = rpixels[x + dx, y + dy]
                            if not is_white(np_):
                                fills.append((x, y, np_))
                                break
            for x, y, c in fills:
                rpixels[x, y] = c
                changed = True
            if iteration > 2:
                break
    
    return result


if __name__ == '__main__':
    for fname in ['iso_grass.png', 'iso_water.png']:
        path = os.path.join(iso_dir, fname)
        if not os.path.exists(path):
            continue
        img = Image.open(path)
        w, h = img.size
        pixels = img.load()
        
        # Count white
        white_count = sum(
            1 for y in range(h) for x in range(w)
            if pixels[x, y][0] > THRESHOLD and pixels[x, y][1] > THRESHOLD and pixels[x, y][2] > THRESHOLD
        )
        total = w * h
        print(f'{fname}: {w}x{h}, white={white_count}/{total} ({100*white_count/total:.1f}%)')
        
        if white_count / total < 0.05:
            print(f'  skipping')
            continue
        
        print('  processing...', end=' ', flush=True)
        result = edge_bleed(img)
        result.save(path)
        
        rpixels = result.load()
        white_rem = sum(
            1 for y in range(h) for x in range(w)
            if rpixels[x, y][0] > THRESHOLD and rpixels[x, y][1] > THRESHOLD and rpixels[x, y][2] > THRESHOLD
        )
        print(f'white: {white_count} -> {white_rem}')
    
    # Process dirt (should be minimal)
    path = os.path.join(iso_dir, 'iso_dirt.png')
    img = Image.open(path)
    pixels = img.load()
    w, h = img.size
    white_count = sum(
        1 for y in range(h) for x in range(w)
        if pixels[x, y][0] > THRESHOLD and pixels[x, y][1] > THRESHOLD and pixels[x, y][2] > THRESHOLD
    )
    print(f'iso_dirt.png: white={white_count}/{w*h} ({100*white_count/(w*h):.1f}%)')