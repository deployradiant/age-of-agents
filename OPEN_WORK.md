# Open Work

**Last updated:** 2026-08-12
**Branch:** `master`
**Base commit:** `486e54b`
**Overall status:** Release verification in progress

## Current goal

Release deterministic server-authoritative grid collision and routing for Milestone 2.

## Contract

- The existing 30×20 terrain cells form the navigation grid.
- Buildings, live resource nodes, villagers, accepted move destinations, and build sites occupy cells.
- Move orders route to a free destination cell.
- Gather, deposit, and build work happen from a free cardinal neighbor of the occupied target.
- Four-neighbor A* is deterministic; routes are derived from authoritative state rather than persisted.
- Commands reject unreachable or occupied targets without mutating state or reserving costs.
- Trained villagers wait for a free adjacent spawn cell rather than overlap another entity.

## Completed but unreleased

- Added deterministic four-neighbor A* in `src/navigation.rs`.
- Added focused movement/occupancy and gathering domain modules.
- Routed move, gather, deposit, and construction phases around occupied cells.
- Reserved accepted move destinations and active build sites in occupancy.
- Added atomic occupied/unreachable command rejection.
- Made villager production wait for a free adjacent spawn cell.
- Updated README and Milestone 2 roadmap status.

## Verification completed

- `cargo fmt --check`.
- `cargo test --locked` — 45 passed.
- Strict Clippy with all targets/features and warnings denied.
- JavaScript syntax plus directional-walk, building-popover, snapshot-buffer, and activity-presentation checks.
- Python compilation plus depleted-resource and all gathering-frame asset checks.
- `git diff --check`.
- Thermonuclear review: navigation is isolated, gathering was extracted, `src/game.rs` is 941 lines, command changes remain atomic, and no frontend or persistence shape was expanded.
- Local server booted against an isolated non-default SQLite path; `GET /state` returned 600 terrain cells, two units, one building, and authoritative speed 1×.
- Browser automation daemon timed out and snap Chromium crashed its network service; visual browser verification is therefore limited. The frontend is unchanged by this release.

## Open work

- Commit and push `master`.
- Wait for GitHub Actions deployment.
- Verify production artifact parity, state, and WebSocket behavior.

## Blockers

None. Local visual-browser tooling is unavailable, but the release does not change frontend code.

## Exact next action

Commit and push the verified implementation, then verify the deployed production endpoint.
