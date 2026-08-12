# Open Work

**Last updated:** 2026-08-12
**Branch:** `master`
**Base commit:** `9fc3722`
**Overall status:** Release verification complete; awaiting commit/deploy

## Current goal

Fix the production movement stall, realign villager highlights to their visible feet, and show the authoritative tick in the top bar.

## Root cause

Production showed `villager-2` frozen at `(1120.724, 994.915)` in `to_resource` while ticks advanced. The villager was already inside the selected free adjacent cell, so A* returned an empty route. Interaction movement interpreted that empty route as zero waypoints instead of centering the villager in the destination cell; the phase could therefore never complete.

Villager sprites were drawn using a hard-coded `0.88 × height` ground anchor even though frame alpha bounds differ. The selection ellipse uses the projected authoritative ground point, so visible feet and highlight diverged by frame.

## Work completed locally

- Added a regression reproducing the exact stuck production position.
- Made interaction movement always include the selected destination center as its final waypoint.
- Anchored villager sprite frames using their actual bottommost non-transparent pixel.
- Added the authoritative tick to the persistent top bar.

## Verification completed

- Focused production-state movement regression passes.
- Full Rust suite passes — 46 tests.
- Strict Clippy, formatting, JavaScript syntax, frontend behavior checks, Python compilation, asset checks, and `git diff --check` pass.
- Added a focused top-tick and frame-aware villager-anchor frontend check.
- Thermonuclear review passes: the movement fix is a six-line boundary correction in the canonical module; the rendering fix derives frame data once at asset load; no new state model or branch sprawl was introduced; `src/game.rs` remains 941 lines and `frontend/app.js` remains below 1,000 lines.
- Local isolated server boots and serves the fixed checkout. Browser automation still times out because the host Chromium daemon is unhealthy; the rendering contract is covered by focused static checks and existing presentation checks.

## Open work

- Commit/push `master`, deploy, and verify production.

## Blockers

None. Stale leaked headless Chromium processes caused initial compiler memory starvation; they were terminated and available memory returned to normal.

## Exact next action

Commit and push the verified fix, then verify the production deployment and that the previously stuck villager leaves `to_resource`.
