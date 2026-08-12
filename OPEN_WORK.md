# Open Work

**Last updated:** 2026-08-12
**Branch:** `master`
**Base commit:** `9fc3722`
**Overall status:** Complete

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
- Commit `5d7d54c` was pushed to `origin/master`; GitHub Actions run `31633329063` passed quality and deployment.
- Modal production matches the checkout, including the updated frontend artifacts.
- The previously stuck production villager left `to_resource`, reached a centered grid position `(1160, 840)`, and remained idle across six advancing production-state samples.

## Open work

None.

## Blockers

None. Browser screenshot automation remains unavailable on this host, but focused rendering checks and deployed artifact parity pass.

## Exact next action

None — work is complete.

## Maintenance rule

Keep this file current rather than appending a diary. Update it at task start, after meaningful milestones, when blockers change, before commit/push/deploy transitions, and before ending a work session. Never store credentials or tokens here.
