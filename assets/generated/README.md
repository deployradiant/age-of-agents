# Generated Asset Provenance

Generated on 2026-08-05 with the configured FAL image backend (`FLUX 2 Klein 9B`). These are initial-demo assets, not a final art lock.

## Source sheets

### `agent_sheet.png`

Source URL: `https://v3b.fal.media/files/b/0aa5122c/AyLzBItnrMhcW66qNth9p_LnSNxVLf.png`

Prompt:

> A clean 2x2 character animation sprite sheet for a small 2D isometric real-time strategy game. SAME single young Mediterranean village worker in all four cells, identical face, short wavy dark brown hair, teal short tunic, brown belt pouch, burgundy trousers, same scale and fixed three-quarter isometric camera. Cell 1 idle standing; cell 2 walking contact pose; cell 3 walking passing pose; cell 4 chopping/gathering wood with a small hand axe. Thin dark-brown ink contours like a refined European comic, warm hand-drawn Japanese cel animation, flat muted colors, restrained cool shadows, simple readable silhouette at 40 pixels tall. Pure uniform white background, no scenery, no ground plane, no labels, no borders, no grid lines, no text, no cast shadows. Each figure centered in its quadrant with feet on the same baseline and equal height. Not photorealistic, not painterly, not 3D.

### `agent_build.png`

Generated on 2026-08-07 with the configured FAL image backend (`FLUX 2 Klein 9B`) using the existing idle, walk, and gather frames as identity/style references. The prompt kept the same young Mediterranean villager, teal tunic, burgundy trousers, dark-brown contours, fixed three-quarter view, and upper-left cel lighting while posing him leaning into a two-handed wooden mallet swing.

The selected iteration was cut out with FAL BiRefNet, filtered to remove near-transparent background noise, scaled to the shared 214-pixel character height, anchored to the shared `y = 232` feet baseline, and saved as 8-bit sRGB RGBA. Nearest visible edge colors were dilated beneath fully transparent pixels to prevent texture-filtering fringes. No temporary generation URL is retained.

### `props_sheet.png`

Source URL: `https://v3b.fal.media/files/b/0aa5122c/Q8L4UHCDN6-Gy7QsJy3-N_MzCTHgRL.png`

Prompt:

> A clean 2x2 game asset sheet for a small 2D isometric real-time strategy game, fixed three-quarter isometric camera and upper-left lighting. Top-left: compact mature olive tree resource with readable trunk. Top-right: small ancient Mediterranean stone-and-plaster town center with terracotta roof and clear footprint. Bottom-left: build command icon showing a wooden hammer over a tiny foundation. Bottom-right: cancel command icon showing two simple crossed wooden sticks. Thin dark-brown ink contours like a refined European comic, warm hand-drawn Japanese cel animation, flat muted sage green, ochre, terracotta and cream colors, readable at small game scale. Pure uniform white background, no scenery, no ground plane, no labels, no borders, no grid lines, no text, each object isolated and centered in its quadrant. Not photorealistic, not painterly, not 3D.

### `terrain_sheet.png`

Source URL: `https://v3b.fal.media/files/b/0aa5123d/m7792Kig1jeYkGwXyveja_tAJ8sLkw.png`

Prompt:

> A clean 2x2 seamless top-down terrain texture sheet for a small 2D isometric RTS. Four equal quadrants with no divider lines: three subtly varied warm sage grass textures and one light ochre packed-earth texture. Flat hand-drawn cel-animation color, very subtle thin dark-brown grass strokes, no perspective, no diamond shapes, no borders, no objects, no trees, no rocks, no paths, no shadows, edge-to-edge texture. Every quadrant must tile seamlessly on all four edges. Warm muted pastoral Mediterranean palette: sage green, olive, parchment, ochre. Not photorealistic, not painterly, not 3D.

## Processing

The source sheets were divided into quadrants. Sprite backgrounds were converted to soft alpha from their uniform white matte, low-contrast generation noise was removed, and each sprite was normalized to a shared canvas and bottom-center anchor. Terrain quadrants were inset to remove generated divider lines and downsampled to 96×96.

Runtime assets live under `assets/game/`. The source sheets remain under `assets/generated/` for inspection and future reprocessing.
