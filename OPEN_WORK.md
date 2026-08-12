# Open Work

**Last updated:** 2026-08-12
**Branch:** `master`
**Base commit:** `ecd6376`
**Overall status:** Visual fix reviewed locally; awaiting release

## Current goal

Restore strong gameplay readability after the grid-routing release: make cell boundaries clearly legible across terrain and fog states, re-audit unit/highlight alignment and movement hands-on, and require captured browser evidence before release.

## Root causes

- Interaction routing could return an empty route when a villager was already inside the chosen adjacent cell but off-center, permanently stalling its phase. Released in `5d7d54c` with an exact production-state regression.
- Sprite frames used a fixed ground anchor despite varying transparent padding, offsetting visible feet from the authoritative selection point. Released in `5d7d54c` with a frame-derived anchor.
- Terrain relied on noisy texture repetition and biome transitions to imply cells. There was no stable, texture-independent projected grid, so the navigable board and fog frontier were difficult to parse.

## Work completed locally

- Added a dedicated projected grid overlay above terrain art and below gameplay entities.
- The grid uses dark structural separation, a restrained warm ink accent, and a stronger boundary only where visible terrain meets unseen fog.
- Strengthened selected-unit rings with dark separation and a bright semantic edge while preserving the shared feet/ground anchor.
- Extracted the grid renderer into `frontend/grid-overlay.js` to keep `frontend/app.js` below the 1,000-line review gate.
- Added a focused grid-legibility contract check covering fog privacy, render order, structural contrast, and selection separation.

## Verification completed

- Captured and reviewed the live production baseline: terrain read as noisy texture patches with no stable navigable-cell structure.
- Captured and reviewed two local grid iterations, toning down the initial warm checkerboard effect while retaining clear dark cell separation.
- Hands-on local gameplay passed: selected a villager, issued a ground move, observed authoritative completion at a new cell, and verified the final ring is centered on visible feet and readable against the grid.
- Final screenshot review found no release-blocking grid, fog-frontier, or selection-alignment defect.
- Full final gate passes: 46 Rust tests, strict Clippy, formatting, JavaScript syntax, all frontend checks including the grid-legibility contract, Python compilation, asset validation, and diff checks.
- Thermonuclear review passes: the visual system is isolated in a 64-line module, `frontend/app.js` is 995 lines, no simulation or persistence shape changed, and unseen terrain is never outlined.

## Open work

Commit/push `master`, deploy, and repeat screenshot/gameplay review against production.

## Blockers

None. X11 screenshot capture and injected pointer input provide a working hands-on review path despite the browser daemon integration remaining unavailable.

## Exact next action

Release the visually reviewed candidate and verify deployed production pixels and behavior hands-on.

## Maintenance rule

Keep this file current rather than appending a diary. Update it at task start, after meaningful milestones, when blockers change, before commit/push/deploy transitions, and before ending a work session. Never store credentials or tokens here.
