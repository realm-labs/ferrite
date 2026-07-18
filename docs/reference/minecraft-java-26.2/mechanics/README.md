# Leaf-rule manual

The ten behavior pages retain the original 65 stable parent rules. Files in this directory split those boundaries into implementation units. A leaf rule is normative only for Java Edition `26.2`; later versions receive a sibling reference tree.

Every leaf has the same fields. “Authoritative state” names the state Ferrite must own. “Transition and ordering” is the executable sequence. “Constants and randomness” distinguishes locked constants from values that must be read through `mc-ref query`. “Gates” lists activity, game-rule, difficulty, permission, and prediction conditions. Test vectors are observable assertions, not implementation prescriptions.

Evidence statuses retain the meanings in [methodology](../methodology.md). An unresolved branch must point at an `EXP-*` experiment or be marked `Implementation blocker`; absence of a value never licenses invention.

## Index

| Domain | Leaf specification |
|---|---|
| Simulation | [Tick pipeline](01-simulation.md) |
| Blocks | [Placement, updates, and falling blocks](02-blocks.md) |
| Environment | [Fluids, weather, light, and fire](03-environment.md) |
| Redstone | [Power and state changes](04-redstone.md) |
| Player | [Movement and interaction](05-player.md) |
| Items | [Use, containers, crafting, and progression](06-items.md) |
| Entities | [Lifecycle, damage, projectiles, and vehicles](07-entities.md) |
| Mobs | [Spawning and AI](08-mobs.md) |
| World | [Worldgen, dimensions, portals, and border](09-world.md) |
| Client-observable | [Prediction, UI, sound, and particles](10-client.md) |
