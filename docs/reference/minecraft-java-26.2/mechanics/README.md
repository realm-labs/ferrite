# Leaf-rule manual

The ten behavior pages retain the original 65 stable parent rules. Domain directories split those
boundaries into implementation units. A leaf rule is normative only for Java Edition `26.2`; later
versions receive a sibling reference tree.

Each implementation-level leaf rule has its own Markdown document and retains the same stable rule
ID. Behavior pages, the completion ledger, catalog entries, implementation code, and tests should
refer to that ID rather than duplicate the rule text.

Every leaf has the same fields. “Authoritative state” names the state Ferrite must own. “Transition
and ordering” is the executable sequence. “Constants and randomness” distinguishes locked constants
from values that must be read through `mc-ref query`. “Gates” lists activity, game-rule, difficulty,
permission, and prediction conditions. Test vectors are observable assertions, not implementation
prescriptions.

Evidence statuses retain the meanings in [methodology](../methodology.md). An unresolved branch must
point at an `EXP-*` experiment or be marked `Implementation blocker`; absence of a value never
licenses invention.

## Formatting contract

- Prose targets 100 columns and should not normally exceed 120 columns.
- Metadata fields stay short; long field bodies begin in their own paragraph.
- Ordered transitions and test vectors use real Markdown lists.
- Large records use subsections instead of tables with paragraph-sized cells.
- Fully qualified source symbols may exceed the prose limit because inserting whitespace would
  change the locator.
- Rule IDs, evidence IDs, source locators, and conclusions must not change during a formatting-only
  refactor.

## Index

| Domain | Leaf-rule index |
|---|---|
| Simulation | [Tick pipeline, scheduled ticks, and random ticks](simulation/README.md) |
| Blocks | [Placement, updates, block entities, and falling blocks](blocks/README.md) |
| Environment | [Fluids, weather, light, fire, and environmental state](environment/README.md) |
| Redstone | [Power, delayed components, pistons, and explosions](redstone/README.md) |
| Player | [Movement, interaction, mining, and lifecycle](player/README.md) |
| Items | [Use, containers, menus, recipes, and progression](items/README.md) |
| Entities | [Lifecycle, damage, effects, projectiles, and vehicles](entities/README.md) |
| Mobs | [Spawning, AI, goals, and navigation](mobs/README.md) |
| World | [World generation, structures, dimensions, portals, and border](world/README.md) |
| Client-observable | [Prediction, UI, sound, and particles](client/README.md) |
