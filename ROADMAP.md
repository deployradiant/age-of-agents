# Roadmap

## Product principle

Grow the proven gather/build demo into one compact, coherent RTS scenario. Every slice must add an end-to-end player decision, remain deterministic and authoritative in Rust, preserve fog/collision/persistence rules, and be playable through the real Canvas UI on desktop and phone. Do not build a generic engine, ECS, recipe language, or broad technology matrix.

## Released baseline — Gather, build, research, and route

- [x] Persisted 600-cell isometric world with eight biomes and explored/visible/unseen fog privacy.
- [x] Seven biome-compatible raw resources: wood, food, stone, gold, iron, clay, and fiber.
- [x] Bounded villager carrying, deposits, resumption, depletion, construction, training, and five gathering technologies.
- [x] Deterministic four-neighbor routing, occupancy, reserved destinations/build sites, and blocked-spawn rejection.
- [x] Typed sequenced WebSocket commands/snapshots, SQLite round-trip, authoritative 0×/1×/2× speed.
- [x] Canvas terrain/entity presentation, directional movement/gathering animation, fog memory, capability popover, and desktop/mobile controls.

## Slice A — Expandable domain foundation

Status: implemented on `feature/expanded-domain-foundation`; not released.

Deliver the smallest explicit catalogs and persisted state needed by later slices.

Gameplay acceptance:

1. The snapshot exposes a stable catalog of the actual resources, building kinds, unit kinds, recipes, and technologies used by this roadmap; unknown persisted enum values fail explicitly rather than resetting the world.
2. Existing saves migrate or deserialize with intentional defaults and retain units, stockpiles, orders, buildings, fog, navigation, and research.
3. Scenario state has an explicit identifier, authoritative tick limit, objective progress, and running/won/lost outcome without yet claiming objectives are playable.
4. Existing released gameplay remains behaviorally unchanged.

Engineering acceptance:

- Split `game.rs` before it or any frontend file grows beyond 1,000 lines; catalogs are direct typed constants/data, not a generic content engine.
- Focused migration, catalog-integrity, serialization, and deterministic tick-boundary regressions pass.

## Slice B — Multi-unit control

Status: planned.

Gameplay acceptance:

1. Mouse drag from empty ground draws a readable selection rectangle and selects all visible friendly units whose projected feet are enclosed; click selection still works.
2. Touch keeps pan/tap semantics and offers additive unit selection without accidental box selection.
3. A group ground order is one typed authoritative command. Validation is atomic: one invalid/busy/member mismatch rejects the whole order without moving any unit.
4. Accepted group movement assigns deterministic distinct reachable destinations, respects reservations/occupancy, and visibly moves every selected unit without stacking.
5. The HUD reports the selected count and group orders survive snapshots/reconnects.

## Slice C — Steel economy vertical slice

Status: planned.

Gameplay acceptance:

1. Coal appears only in compatible terrain and is extracted by villagers only after a Mining Camp is constructed beside it.
2. Iron extraction also requires a Mining Camp; legacy iron remains discoverable but cannot be hand-gathered.
3. A Smelter/Forge can queue steel batches; each batch atomically reserves iron plus coal, progresses visibly, and deposits steel exactly once.
4. Invalid placement, missing inputs, occupied job slots, and inaccessible spawn/interaction cells reject without partial cost/input mutation.
5. The building popover and stockpile HUD make prerequisites, costs, queue progress, blocked reasons, coal, and steel understandable.

## Slice D — Coherent broader economy and progression

Status: planned.

Target economy (13 total resources/products): wood, food, stone, gold, iron ore, coal, clay, fiber, timber, steel, bricks, cloth, and rations.

Gameplay acceptance:

1. Extraction buildings are limited to Mining Camp (iron/coal/gold/stone) and Farm (food/fiber); villagers still directly gather wood and clay.
2. Lumber Mill makes timber from wood; Smelter makes steel from iron+coal; Kiln makes bricks from clay+wood; Weaver makes cloth from fiber; Kitchen makes rations from food.
3. Town Center, Mining Camp, Farm, Lumber Mill, Smelter, Kiln, Weaver, Kitchen, Barracks, Range, Workshop, Infirmary, Watchtower, and Monument form the complete useful building roster. Buildings not yet active in combat may appear only in the slice that makes them useful.
4. Building upgrades are bounded to two levels and improve one visible property. Research remains building-specific and every technology enables or improves an immediately playable action.
5. Costs, one-job-slot queues, prerequisites, progress, and completion are authoritative, persisted, and visible. No giant recipe/technology matrix.

## Slice E — Bounded scenarios

Status: planned.

Deliver four selectable deterministic challenges, each with visible progress, a tick/time limit, and terminal won/lost state:

1. Foundry Town: construct a Smelter and hold 20 steel.
2. Frontier Survey: reveal at least 70% of terrain and construct a Watchtower before the limit.
3. Monument Works: construct the Monument using timber, bricks, cloth, gold, and steel.
4. Hold the Coast: survive the raid schedule until the final tick with the Town Center alive (enabled with Slice G).

A combined default prototype scenario requires steel production, exploration, and survival. Terminal scenarios reject further simulation-changing commands except reset/select-scenario. HUD presents objective progress and remaining authoritative time.

## Slice F — Combat foundation

Status: planned.

Gameplay acceptance:

1. Units/buildings have faction and health; snapshots omit unseen hostiles under existing fog privacy.
2. Barracks trains melee Guards; Range trains Archers. Group attack/target commands validate ownership, visibility, range/path viability, and members atomically.
3. Guards close to melee range; Archers hold bounded range. Attack cadence, damage, target loss, and death cleanup are deterministic.
4. Dead units release occupied/reserved cells and are removed from selection/orders/persistence exactly once.
5. The player can train, select, group-order, fight, and win a small deterministic skirmish through the real UI.

## Slice G — Defense, support, siege, and raids

Status: planned.

Gameplay acceptance:

1. Infirmary trains a Healer that restores friendly health without exceeding maximum health and cannot heal hostiles or dead targets.
2. Workshop trains a slow Siege Cart with bonus structure damage; blocked production spawns reject atomically.
3. Watchtowers automatically attack visible hostile units in range with deterministic cadence and target choice.
4. Pirate raids use a seeded fixed schedule plus bounded deterministic interval jitter, spawn at valid coastal/edge cells, and pursue explicit scenario targets.
5. Raid warnings, current wave, losses, and survive progress are visible. The player can build defenses, position a mixed group, survive, and complete Hold the Coast.

## Slice H — Balance and release polish

Status: planned.

Gameplay acceptance:

1. Starting resources and timings let a new run reach steel, field a mixed defense, and finish the default scenario in a bounded play session without developer shortcuts.
2. Desktop and phone controls remain legible; selection, commands, queue state, scenario progress, combat feedback, and raids have clear visual feedback without frontend emojis.
3. New sprites follow the established hand-drawn cel-shaded dark-ink direction, are valid sRGB RGBA PNGs, and share verified ground anchors.
4. README documents the actual playable loop and controls; this roadmap marks only production-verified slices delivered.

## Gate for every released slice

1. Focused Rust regressions for every domain rule and bounded transition.
2. `cargo fmt --check`, `cargo test`, and `cargo clippy --all-targets --all-features -- -D warnings`.
3. Frontend syntax plus focused presentation/control contract checks; Python asset checks; `git diff --check`.
4. Independent specification review, code-quality review, and thermonuclear maintainability review with all blockers fixed.
5. Real isolated local server plus Chromium: use actual controls/WebSocket, inspect intermediate authoritative state, browser console, desktop and phone screenshots, and play the delivered loop.
6. Push `master`, observe CI, deploy through `python3 scripts/modal_manage.py`, then verify production assets byte-for-byte, state/protocol, hard-reloaded Chromium interactions/screenshots, and restoration of reversible controls.
7. Refresh `OPEN_WORK.md`, README, and this roadmap so committed, pushed, deployed, and production-verified states are never conflated.

## Explicitly deferred

- LLM-controlled villagers, autonomous planning, multiplayer, mod/plugin APIs, generic ECS/content engines, procedural world generation, and a large branching tech tree.
