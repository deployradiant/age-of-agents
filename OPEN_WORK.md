# Open Work

**Last updated:** 2026-08-13T04:36:10Z
**Branch:** `master`
**Base commit:** `028b226`
**Overall status:** In progress — first two slices implemented in review fix loops

## Current goal

Turn the released gather/build demo into one coherent playable RTS prototype with multi-unit control, a 13-resource production economy, bounded scenarios, combat roles, defenses, and deterministic pirate raids.

## Completed in the working tree

- Rewrote and pushed `ROADMAP.md` as eight vertical slices with explicit gameplay and release acceptance (`028b226`).
- Slice A implementation is isolated in PR #6 / commit `b982a0e`: typed 13-resource/building/unit/recipe/technology foundation, persisted scenario state, migration regressions, and `game.rs` decomposition.
- Slice B implementation is isolated in PR #7 / commit `d31c34e`: drag-box/additive multi-selection, selected-count HUD, atomic typed `group_move`, deterministic distinct destination assignment, and focused JS/Rust checks.
- Parent independently verified PR heads, changed files, 51 Slice A tests and 50 Slice B tests. Both strict Clippy runs passed after a parent fix for the enlarged snapshot enum.
- Independent spec/thermonuclear reviews correctly blocked both PRs; fix agents are active on the same isolated branches.

## Verification completed

- Baseline `028b226`: full local Rust/frontend/Python/asset gate passed; CI run 31666409158 passed quality and production deployment.
- PR #6: `cargo test` — 51 passed; strict Clippy and `git diff --check` passed after boxing the snapshot payload.
- PR #7: `cargo test` — 50 passed; strict Clippy, selection Node contract, and `git diff --check` passed; real local WebSocket accepted `group_move` with `applied_sequence=71`.
- Browser daemon is unhealthy (120-second timeouts), but direct Chromium is available; visual release QA remains mandatory after integration.

## Open work

1. Resolve PR #6 review blockers: split >1,000-line tests, remove duplicate recipe identity, and address remaining review findings.
2. Resolve PR #7 blockers: preserve mouse pan, ensure collision-safe intermediate group movement, add CI/deployment parity, pointer cleanup/tests.
3. Re-review both PRs, integrate on `master`, run full local browser-play gate, push/deploy, and production-verify.
4. Continue through steel economy, broader economy/scenarios, combat/raids, and balance slices.

## Blockers

None external. The browser tool daemon is unhealthy; use direct Chromium CDP/X11 as already established.

## Exact next action

Wait for the two isolated fix commits, rerun independent reviews, then cherry-pick only approved commits into `master` and execute the complete local release gate.
