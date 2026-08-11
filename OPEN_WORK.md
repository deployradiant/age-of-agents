# Open Work

**Last updated:** 2026-08-11 23:37:14 UTC
**Branch:** `master`
**Base commit:** `30854a3`
**Overall status:** In progress

## Current goal

Ship the requested gathering, rendering, simulation-speed, and fog-boundary fixes to production.

## Completed in the working tree

- Reduced base gathering consumption from 10 to 2 units per simulated second.
- Added regression coverage proving villagers remain at a node until their 20-unit load is full or the node depletes.
- Made gathering presentation follow the authoritative server phase rather than infer state from distance.
- Replaced separate resource and villager rendering with combined activity sprites during gathering.
- Preserved 1:n villager-to-resource assignment with deterministic presentation offsets and click behavior.
- Made villagers win equal-depth rendering ties over resources.
- Added authoritative 0×, 1×, and 2× simulation-speed commands and controls.
- Matched the out-of-world canvas and page background to unseen fog.
- Added focused activity-presentation and simulation-speed tests.
- Updated deployment verification, CI, README, ROADMAP, and repository instructions.

## Verification completed

- `cargo fmt --check`
- `cargo test --locked` — 37 passed
- `cargo clippy --all-targets --all-features --locked -- -D warnings`
- JavaScript syntax checks
- Directional-walk, building-popover, snapshot-buffer, and activity-presentation checks
- Python compilation
- Depleted-resource and all 14 gathering-frame asset checks
- `git diff --check`
- `frontend/app.js` remains below 1,000 lines (967)
- Thermonuclear review — approved with no blocking structural or security findings
- Real Chromium smoke test — pause held the authoritative tick fixed; two villagers concurrently remained gathering below capacity; two combined activities rendered with zero standalone copies of the active resource; fog background matched `rgb(2, 5, 4)`; no browser exceptions

## Open work

1. Commit and push `master`.
2. Deploy the pushed commit to Modal.
3. Verify production HTTP state, static assets, WebSocket protocol, and speed command behavior.
4. Refresh this file to the final no-open-work state and ensure that state is committed, pushed, and deployed.

## Blockers

None.

## Exact next action

Commit and push the verified working tree.

## Maintenance rule

Keep this file current rather than appending a diary. Update it at task start, after meaningful milestones, when blockers change, before commit/push/deploy transitions, and before ending a work session. Never store credentials or tokens here.
