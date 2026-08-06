# Thermonuclear Code Quality Review

Use this review before committing meaningful changes. It is intentionally severe: working code is not enough if the implementation makes the project harder to understand or extend.

## Review prompt

> Perform a deep code-quality audit of the current changes. Rethink the structure and implementation to meaningfully improve quality without changing required behavior. Improve boundaries and modularity, remove spaghetti growth, and make the code shorter and more legible. Be ambitious where a restructuring can delete complexity. Measure twice, cut once.

## Blocking questions

1. Is there a code-judo move that deletes concepts, branches, helpers, or layers?
2. Did this change add behavior outside the current roadmap milestone?
3. Did an idle NPC accidentally acquire autonomous behavior?
4. Is authoritative game behavior implemented only in the Rust domain layer?
5. Are commands typed, validated atomically, and rejected without partial mutation?
6. Can a resource cost be charged twice, lost on an unrelated path, or bypassed?
7. Can an entity or building be created more than once?
8. Does persistence use the configured path on every path and fail clearly on corrupt data?
9. Did any file cross 1,000 lines? If so, decompose it unless there is an exceptional structural reason.
10. Did the frontend add a framework, scene graph, hidden picking system, or interaction mode that direct Canvas code does not need?
11. Are mouse and touch semantics identical?
12. Are generated assets being judged at runtime size and validated for real RGBA transparency?

## Structural standards

- Prefer deleting a layer over polishing it.
- Prefer direct domain methods over repositories, managers, services, policies, or generic engines that have one implementation.
- Do not scatter feature-specific conditionals through shared flows.
- Do not use optional-field command bags when a typed enum expresses valid shapes.
- Avoid thin wrappers and pass-through helpers that add navigation without clarity.
- Keep orchestration separate from domain rules.
- Keep related state changes atomic.
- Use deterministic demo data; avoid randomness that makes behavior and tests hard to inspect.
- Do not silently recover from state corruption by replacing the world.

## Frontend standards

- Canvas renderer, networking, input, and UI state should remain understandable in one reading.
- Use plain data and functions before classes or abstractions.
- Keep rendering order explicit: terrain, previews, depth-sorted entities, selection/status.
- Store simple hit areas during rendering; do not add raycasters or scene graphs.
- Avoid duplicate gesture paths for desktop and mobile.
- UI buttons must remain accessible real buttons, not invisible canvas regions.
- No frontend emojis; use image assets or plain text.

## Asset standards

- File signature must actually be PNG.
- Sprites must be RGBA with transparent corners.
- Check for matte contamination on dark green, parchment, and blue backgrounds.
- Animation frames must share identity, scale, baseline, camera, lighting, and anchor.
- Terrain must pass a repeated seam test.
- Reject obvious generated-art anatomy, perspective, duplicate-detail, or floating-object artifacts.

## Verification gate

A change is approved only when:

- Required behavior is covered by focused tests.
- Formatting, tests, and lint pass without warnings.
- The live browser flow is exercised rather than inferred.
- There is no obvious simpler design that preserves the same behavior.
- No major roadmap or documentation claim is ahead of reality.
- The diff contains no unrelated scope growth.

When blockers are found, fix them and rerun this review. Do not invent cosmetic nits after the structural blockers are resolved.
