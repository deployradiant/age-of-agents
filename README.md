# Age of Agents

A deliberately small, mobile-first 2D isometric RTS vertical slice built with a Rust authoritative server, Canvas 2D frontend, WebSocket state streaming, and SQLite persistence.

The initial goal is intentionally narrow: command villagers to gather a small three-resource economy, construct town centers, and train villagers one at a time. There are no LLM agents or autonomous NPC policies in this milestone. Villagers remain idle until commanded.

## Milestone 1

- Persistent deterministic 2400×1600 Voronoi-style world with eight connected biomes
- Server-authoritative fog with visible, explored-dim, and unseen-dark terrain
- Selectable villagers
- Command-driven wood, food, and stone gathering
- Command-driven construction of one building type
- Starting town-center base with single-slot villager production
- Shared wood, food, and stone stockpiles
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
- **Gather:** with a villager selected, tap/click a tree, berry bush, or stone deposit. Depleted resources remain visible and cannot receive new gather orders.
- **Build:** select a villager, press the build button, then tap/click valid ground.
- **Produce:** select a town center and press **Villager**. It reserves 50 food and produces one villager over six seconds; each building has one active production slot.
- **Pan:** drag with one pointer.
- **Zoom:** pinch or use the mouse wheel.
- **Recover view:** reload to center the camera on the currently visible villagers.
- **Reset world:** press **Reset world** and confirm to erase progress and restore the deterministic starting state.
- **Cancel build placement:** press the cancel button or Escape.

Mouse and touch use the same command semantics.

Schema version 3 introduces typed resources, bases, and production. Older persisted worlds are intentionally reset to the deterministic starting state because their tree-only snapshots cannot represent the new model safely.

## Development checks

```bash
cargo fmt --check
cargo test
cargo clippy --all-targets -- -D warnings
node --check frontend/app.js
node scripts/check_directional_walk.cjs
python3 -m py_compile modal_app.py scripts/modal_manage.py
python3 scripts/check_depleted_asset.py
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
