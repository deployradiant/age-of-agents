# Age of Agents

A deliberately small, mobile-first 2D isometric RTS vertical slice built with a Rust authoritative server, Canvas 2D frontend, WebSocket state streaming, and SQLite persistence.

The initial goal is intentionally narrow: select a villager, gather wood from a tree, and spend that wood to construct one building. There are no LLM agents or autonomous NPC policies in this milestone. Villagers remain idle until commanded.

## Milestone 1

- Persistent deterministic 2400×1600 Voronoi-style world with eight connected biomes
- Server-authoritative fog with visible, explored-dim, and unseen-dark terrain
- Selectable villagers
- Command-driven wood gathering
- Command-driven construction of one building type
- Shared wood stockpile
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

- **Select:** tap/click a villager.
- **Gather:** with a villager selected, tap/click a tree.
- **Build:** select a villager, press the build button, then tap/click valid ground.
- **Pan:** drag with one pointer.
- **Zoom:** pinch or use the mouse wheel.
- **Recover view:** reload to center the camera on the currently visible villagers.
- **Cancel build placement:** press the cancel button or Escape.

Mouse and touch use the same command semantics.

## Development checks

```bash
cargo fmt --check
cargo test
cargo clippy --all-targets -- -D warnings
node --check frontend/app.js
```

Before shipping structural changes, apply [docs/THERMONUCLEAR_REVIEW.md](docs/THERMONUCLEAR_REVIEW.md).

## Deploy to Modal

```bash
modal deploy modal_app.py
```

The production demo is deployed from `master` after local tests and browser verification pass.

## Art direction

The visual target is a warm hand-drawn cel-animation style with thin-to-medium dark contours, flat colors, and a fixed three-quarter isometric view. `assets/mood_board_v3.png` is the closest current directional reference. Existing sprites are placeholders and are being normalized to the asset contract in the roadmap.

The eight production terrain textures are generated through FAL and normalized to seamless 96×96 8-bit sRGB RGBA PNGs by `scripts/generate_terrain_textures.py`. Download provenance is recorded in `assets/generated/terrain_voronoi_sources.md`.
