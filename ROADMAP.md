# Roadmap

## Product principle

Build the smallest convincing RTS loop before expanding the simulation. Each milestone must be playable on a phone and understandable without developer explanation.

## Milestone 1 — Gather and Build Vertical Slice

### World

- Deterministic maximum-size 2400×1600 map, generated once as 600 persisted cells.
- Eight meaningful, individually connected Voronoi-style biome regions.
- Persistent explored fog; unit/building sight produces visible, explored-dim, and unseen-dark terrain. Never-seen entities are omitted, while discovered static resources remain as dimmed map memory.
- Two villagers, initially idle, and one starting town-center base.
- Deterministically placed wood, food, stone, gold, iron, clay, and fiber nodes, each on a compatible biome and at least 120 world units from every other node.
- Seven typed shared stockpiles.
- One buildable building type: town center, with one active production/research slot and typed product and technology sets.
- No autonomous task selection, random world generation, resource regeneration, combat, health, or LLM behavior.

### Simulation

- An idle villager does nothing across arbitrary ticks.
- A move order makes an idle villager walk to valid selected ground and reveal terrain along the route.
- A gather order makes the villager walk to the selected resource.
- At the resource, the villager gathers its typed material into a bounded 20-unit load.
- Full and final partial loads return to the nearest town center, deposit exactly once into the matching stockpile, then resume the same order if material remains.
- When the resource is depleted, the villager becomes idle.
- A build order validates the selected villager, position, and wood cost.
- Wood is reserved exactly once when the build order is accepted.
- The villager walks to the site and visibly constructs for four seconds.
- The building appears exactly once after construction finishes.
- The villager becomes idle afterward.
- Busy villagers reject replacement orders for this milestone.
- A town center can produce one villager at a time from its allowed product set, reserving 50 food exactly once and completing after six seconds.
- A town center can research Forestry, Agriculture, Masonry, Mining, and Textiles, enforcing prerequisites and reserving 40 food plus 20 wood exactly once; completed research improves matching gather rates by 20%.

### Persistence and networking

- Rust owns authoritative state.
- Full snapshots stream over WebSocket.
- Commands and command results are typed and include a request ID.
- SQLite round-trips active orders and completed world state.
- Every persistence path honors `AGE_OF_AGENTS_DB`.
- Corrupt saves produce an explicit startup error.

### Frontend

- Native Canvas 2D; no Three.js or UI framework.
- Fixed isometric Canvas tile map with no visible seams and exactly eight production terrain textures.
- Entities draw in stable ground-depth order.
- Tap/click selects a villager.
- With a villager selected, tap/click empty ground to move.
- Tap/click any visible resource to gather with a two-frame resource-specific activity animation.
- Tap/click a town center to train one villager or research one available technology at a time.
- Rendering runs independently of network arrival and interpolates only compatible authoritative movement snapshots without extrapolation.
- Build image button enters placement mode; tap/click ground to build.
- Drag pans; wheel/pinch zooms.
- Touch targets are at least 48×48 CSS pixels and respect mobile safe areas.
- Thin top resource row, selected-unit status, bottom command dock, and an anchored selected-building capability popover.
- No minimap, event chronicle, full agent list, or desktop-only controls.

### Asset contract

All production sprites must be:

- Real 8-bit sRGB RGBA PNG files, not JPEG data with `.png` extensions.
- Tightly framed with transparent corners/margins.
- Free of opaque cream/gray matte rectangles and JPEG halos.
- Edge-color-dilated beneath transparency to prevent fringes.
- Drawn from one fixed three-quarter isometric view with upper-left lighting.
- Readable at actual runtime size.

Required set:

| Asset | Source | Runtime | Anchor |
|---|---:|---:|---|
| `agent_idle.png` | 256×256 | 40×40 | feet at `(0.5, 0.875)` |
| `agent_walk_01.png` | 256×256 | 40×40 | same baseline and identity |
| `agent_walk_02.png` | 256×256 | 40×40 | same baseline and identity |
| six directional walk frames | 256×256 | 40×40 | diagonal-toward, down-front, and up-back two-frame cycles |
| 14 resource-specific gather frames | 256×256 | 40×40 | two frames each for seven resource kinds |
| `agent_build.png` | 256×256 | 40×40 | same baseline and identity |
| `resource_tree.png` | 256×256 | 35×35 | trunk bottom-center |
| `resource_berries.png` | 256×256 | 35×35 | bush bottom-center |
| `resource_stone.png` | 256×256 | 35×35 | outcrop bottom-center |
| `building_town_center.png` | 512×512 | 80×80 | footprint bottom-center |
| eight biome terrain textures | 96×96 | renderer-defined | seamless square source |
| build/cancel controls | 128×128 | 48×48 minimum | centered icon |

Animation gates:

- Agent baselines differ by no more than one source pixel.
- Character height differs by no more than 3% between frames.
- Frame changes show movement, not identity, camera, costume, or scale changes.

### Milestone 1 definition of done

- All Rust tests pass.
- Clippy passes with warnings denied.
- Frontend JavaScript syntax check passes.
- Browser flow is manually verified on desktop and a phone-sized viewport.
- State survives a server restart using a non-default SQLite path.
- Thermonuclear review has no unresolved blockers.
- `master` is pushed and the Modal URL serves the verified build.

## Milestone 2 — RTS Feel

Only after Milestone 1 is stable:

- Grid-aware collision and simple path routing.
- Building placement validity and occupancy.
- Dedicated visible carried-load sprites if they improve clarity beyond the delivered cargo status and gathering cycles.
- More coherent terrain variants and transitions.
- One additional building or resource only if it deepens the loop.
- Audio feedback and tactile mobile interaction polish.

## Milestone 3 — Strategy

- Population and production.
- One military unit and one enemy target.
- Basic combat.
- Additional strategic fog rules (enemy vision and scouting); terrain exploration is delivered in Milestone 1.
- Scenario win/lose condition.

## Explicitly deferred

- LLM-driven agents or autonomous planning.
- Multiplayer.
- General ECS or reusable engine framework.
- Large tech trees or economy matrices.
- Procedural world generation.
- Modding/plugin systems.
