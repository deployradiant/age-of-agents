# Age of Agents

A deliberately small, mobile-first 2D isometric RTS vertical slice built with a Rust authoritative server, Canvas 2D frontend, WebSocket state streaming, and SQLite persistence.

The current vertical slice is intentionally bounded: command villagers through a seven-resource gather/carry/deposit economy, construct town centers, train villagers, and research five gathering improvements. There are no LLM agents or autonomous NPC policies. Villagers remain idle until commanded.

## Milestone 1

- Persistent deterministic 2400×1600 Voronoi-style world with eight connected biomes
- Server-authoritative fog with visible, explored-dim, and unseen-dark terrain
- Selectable villagers
- Biome-compatible wood, food, stone, gold, iron, clay, and fiber gathering
- Bounded villager cargo with explicit return and town-center deposit phases
- Command-driven construction of one building type
- Starting town-center base with single-slot villager production
- Seven typed shared stockpiles and a five-technology gathering tree
- Rust-authoritative fixed-timestep simulation
- Typed WebSocket commands and snapshots
- SQLite save/load
- Unified mobile and desktop controls
- Modal deployment

See [ROADMAP.md](ROADMAP.md) for the exact acceptance criteria and later milestones.

## Architecture

```text
Canvas 2D client
  ├─ pointer/touch input
  ├─ isometric rendering
  └─ WebSocket commands/snapshots
              │
              ▼
Rust + Axum server
  ├─ deterministic game domain
  ├─ fixed timestep
  ├─ command validation
  └─ SQLite snapshot persistence
```

The server owns the world. The browser renders snapshots and sends player intent; it does not simulate authoritative outcomes.

## Run locally

Requirements:

- Rust 1.85 or newer
- A modern browser

```bash
cargo run
```

Open <http://localhost:8000>.

The default SQLite file is `age_of_agents.db`. Override it with:

```bash
AGE_OF_AGENTS_DB=/tmp/age-of-agents.db cargo run
```

## Controls

- **Select:** tap/click a villager or town center.
- **Move:** with a villager selected, tap/click empty ground.
- **Gather:** with a villager selected, tap/click a resource or an active combined gathering animation. Villagers gather two units per second, wait for a full 20-unit load unless the node depletes, deposit at the nearest town center, and resume until depletion.
- **Build:** select a villager, press the build button, then tap/click valid ground.
- **Produce:** select a town center and press **Train Villager** in its anchored capability popover. It reserves 50 food and produces one villager over six seconds; each building has one active production slot.
- **Research:** select a town center and choose an available technology in its popover. Research reserves 40 food and 20 wood, occupies the building for eight seconds, and improves matching gather rates by 20%.
- **Pan:** drag with one pointer.
- **Zoom:** pinch or use the mouse wheel.
- **Recover view:** reload to center the camera on the currently visible villagers.
- **Reset world:** press **Reset world** and confirm to erase progress and restore the deterministic starting state.
- **Simulation speed:** use **0×**, **1×**, or **2×** in the top bar to pause or change authoritative simulation speed.
- **Cancel build placement:** press the cancel button or Escape.

Mouse and touch use the same command semantics.

Schema version 4 introduces cargo phases, seven typed resources, building jobs, and technologies. Older persisted worlds are intentionally reset because they cannot represent the new model safely.

## Development checks

```bash
cargo fmt --check
cargo test
cargo clippy --all-targets -- -D warnings
node --check frontend/app.js
node scripts/check_directional_walk.cjs
node scripts/check_building_popover.cjs
node scripts/check_snapshot_buffer.cjs
node scripts/check_activity_presentation.cjs
python3 -m py_compile modal_app.py scripts/modal_manage.py scripts/process_resource_activity_sprites.py scripts/check_resource_activity_assets.py
python3 scripts/check_depleted_asset.py
python3 scripts/check_resource_activity_assets.py
```

Before shipping structural changes, apply [docs/THERMONUCLEAR_REVIEW.md](docs/THERMONUCLEAR_REVIEW.md).

## Deploy to Modal

```bash
python3 scripts/modal_manage.py deploy
```

Useful management commands are `status`, `history`, `logs`, `rollover`, and `verify`. Stopping production requires the explicit `stop --confirm age-of-agents` safeguard.

Every push to `master` runs the same checks in GitHub Actions, deploys through Modal, and verifies that the production HTML, JavaScript, directional sprites, world shape, and unseen-terrain privacy match the committed checkout.

## Art direction

The visual target is a warm hand-drawn cel-animation style with thin-to-medium dark contours, flat colors, and a fixed three-quarter isometric view. `assets/mood_board_v3.png` is the closest current directional reference. Existing sprites are placeholders and are being normalized to the asset contract in the roadmap.

The eight production terrain textures are generated through FAL and normalized to seamless 96×96 8-bit sRGB RGBA PNGs by `scripts/generate_terrain_textures.py`. Download provenance is recorded in `assets/generated/terrain_voronoi_sources.md`.
