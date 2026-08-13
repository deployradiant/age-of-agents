# Open Work

**Last updated:** 2026-08-13T18:43:00Z
**Branch:** `master`
**Base commit:** `f46ee17`
**Overall status:** In progress — Slices A+B pushed; local browser and production release gates active

## Current goal

Release the integrated expandable domain foundation and multi-unit controls, then continue with a steel economy vertical slice plus one low-overlap roadmap lane.

## Completed in the working tree

- `master` and `origin/master` both point to `f46ee17`; the primary checkout was clean when this checkpoint was refreshed.
- Slice A is integrated: explicit 13-resource/14-building/5-unit catalogs, five recipes, scenario state, save migration, and regressions.
- Slice B is integrated: Shift-box selection, explicit touch long-press additive selection, atomic typed group move, deterministic distinct destinations, and collision regressions.
- PR #6 (`af6a3fb`) and PR #7 (`c6d0d79`) remain open for audit only. Their reviewed work was integrated directly; neither PR was merged.

## Verification completed

- `cargo fmt --check && cargo test && cargo clippy --all-targets --all-features -- -D warnings` — formatting and strict Clippy passed; 56/56 Rust tests passed at `f46ee17`.
- GitHub Actions run `31731879535` for `f46ee17`: quality job passed; deploy job was still running at this checkpoint.
- Cache-bypassed production `/state` already exposes the integrated 13-key stockpile and scenario/catalog shape.

## Open work

1. Fix the production verifier's stale seven-resource assertion, which now rejects the correct 13-resource Slice A state.
2. Complete real isolated local Chromium gameplay: Shift-box two villagers, issue one group move, sample intermediate authoritative movement, inspect console, and capture desktop plus phone screenshots.
3. Resolve final audit reviews, push the verifier fix, wait for CI/deploy, hard-reload, and repeat hands-on production verification.
4. Implement and gate steel economy plus one low-overlap roadmap lane in isolated delegated worktrees.

## Blockers

None external. The current deployment verifier is internally stale: `scripts/modal_manage.py` still requires exactly seven stockpile keys even though Slice A intentionally exposes thirteen.

## Exact next action

Update the verifier to the explicit 13-resource contract, run its checks, then launch the isolated local server and Chromium interaction harness.
