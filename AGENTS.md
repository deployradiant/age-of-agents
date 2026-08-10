# Age of Agents — Repository Context

## Product

Age of Agents is a deliberately small, mobile-friendly 2D isometric real-time strategy game inspired by late-1990s and early-2000s RTS games. The near-term reference is the clarity and immediacy of an early Age of Empires vertical slice, not a full simulation.

Despite the project name, the initial game contains **no LLM-controlled or autonomous AI agents**. In code and product language, use **villager**, **unit**, or **NPC** for game entities. Idle units remain idle until the player commands them.

## Milestone 1

The current playable demo proves these loops:

1. Isometric terrain tiles render cleanly on desktop and mobile.
2. A player can command a villager through typed gathering, bounded carrying, town-center deposits, and deterministic resumption across seven biome-compatible resources.
3. A player can construct a town center and train villagers through its single authoritative job slot.
4. A player can research five bounded gathering improvements through that same job slot.

Keep the world deterministic and small. Do not add combat, pathfinding frameworks, autonomous task selection, LLM calls, multiplayer, or generalized engine abstractions before this milestone is excellent.

## Architecture

- **Backend:** Rust, Axum, Tokio.
- **Authority:** The Rust server owns all world state and advances a fixed-timestep simulation.
- **Transport:** WebSocket typed commands, command acknowledgements, and full world snapshots. `GET /state` exists for debugging.
- **Persistence:** SQLite stores the authoritative world snapshot. Respect `AGE_OF_AGENTS_DB` everywhere.
- **Frontend:** Native Canvas 2D with plain JavaScript/TypeScript-sized code. Keep it deliberately small; no renderer framework unless the current vertical slice proves it necessary.
- **Rendering:** Fixed isometric perspective. Draw terrain first, then world entities sorted by their ground position. UI buttons are real accessible buttons with image artwork.
- **Deployment:** Modal. Verify locally before deploying.

## Interaction Contract

- Tap/click a villager to select it.
- Tap/click a resource with a villager selected to issue a gather order.
- Villagers carry at most 20 typed units, deposit at a town center, and resume unfinished gathering.
- Tap/click the build button, then valid ground, to issue a build order.
- Tap/click a town center to train a villager or start available research through its capability popover.
- Drag pans. Wheel/pinch zooms.
- Mouse and touch semantics must match.
- A busy villager rejects replacement orders in Milestone 1; this avoids cancellation/refund complexity.

## Art Direction

The target is a warm, hand-drawn cel-animation look inspired by pastoral Japanese animation and thin-line European comics:

- Thin-to-medium dark-brown/charcoal contours; heavier lines only on outer silhouettes.
- Flat cel colors, warm highlights, restrained cool shadows.
- Fixed three-quarter isometric view and upper-left light direction.
- Readability at actual gameplay size matters more than 1024px detail.
- No photorealism, painterly gradients, pseudo-3D materials, opaque matte rectangles, text, signatures, or inconsistent character identity between animation frames.

`assets/mood_board_v3.png` is the closest current directional reference. Existing gameplay sprites are placeholders until they satisfy the asset contract in `ROADMAP.md`.

## Engineering Rules

- Prefer subtraction and direct code over generalized machinery.
- A feature must earn its complexity in the current milestone.
- Keep domain logic deterministic and testable without the server.
- Use typed enums at command boundaries; avoid stringly typed optional-field command bags.
- Reject invalid commands without partially mutating state.
- Build costs are reserved exactly once and completed buildings appear exactly once.
- Corrupt persisted state is an error, not permission to silently reset the world.
- Keep frontend files under 1,000 lines; aim much lower.
- Do not use emojis in the game UI.
- Run the thermonuclear review in `docs/THERMONUCLEAR_REVIEW.md` before shipping meaningful changes.

## Workflow

- Parent-session work may proceed directly on `master` for this solo project.
- Every delegated subagent must work on its own feature branch or git worktree, push that branch, and open a pull request for Jakob to review. Subagents must never commit directly to `master`.
- A subagent PR must describe its scope, verification performed, generated assets, and any known limitations. Do not merge it automatically.
- After changes: format, test, lint, perform the thermonuclear review, verify the browser demo, and redeploy Modal.
- Keep `README.md` and `ROADMAP.md` synchronized with actual behavior.
