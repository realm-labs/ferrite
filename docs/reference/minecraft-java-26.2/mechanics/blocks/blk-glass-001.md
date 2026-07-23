# Blocks mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `BLK-GLASS-001` — Glass is a full collider that propagates skylight

**Parent:** `BLK-001`, `BLK-002`, `BLK-003`, `BLK-005`, `PLY-005`, `PLY-006`, `ENV-003`,
`ENT-001`, `MOB-001`, `CLI-006`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked registration, inherited transparent-block hooks, report, loot, recipe,
tag data and client assets fix the sole state, physical/light behavior, acquisition and projection.
Direct beacon and iron-golem-support consumers plus the only `impermeable` consumer fix the
exceptional and non-interaction boundaries.

**Applies when:**

`minecraft:glass` is placed, mined, used as entity or iron-golem spawn support, queried by lighting
or a beacon, serialized, rendered, smelted or considered by the bundled `impermeable` tag consumer.

**Authoritative state:**

Glass is a `TransparentBlock` with one property-free default state, ID `562`, and no block entity.
Registration supplies hat note instrument, glass sound, destroy speed and explosion resistance
`0.3`, no occlusion, a never-valid entity-spawn predicate, and false redstone-conductor,
suffocating and view-blocking predicates. Collision and selection remain a full cube;
`TransparentBlock` returns an empty visual shape and shade brightness `1.0`. Piston reaction remains
`NORMAL`.

Its ordinary common-rarity `BlockItem` stacks to `64` and has no special components or use gate.
The locked smelting recipe accepts the live `smelts_to_glass` ingredient tag, returns one glass,
awards `0.1` experience and uses the generic omitted cooking-time default. Generic item placement
writes state 562.

**Transition and ordering:**

`TransparentBlock.propagatesSkylightDown` returns true. The base light-dampening calculation first
tests solid rendering and then skylight propagation; glass is not solid-rendering and therefore
caches dampening `0`. Lighting scheduling, propagation, section dirtiness and client publication
remain with `ENV-LIGHT-001`.

Beacon vertical scanning does not take a colored-glass branch for plain glass. Its dampening `0` is
below the obstruction threshold `15`, so an existing beam section increases its height and the scan
continues. This differs from tinted glass, which reaches the obstruction branch.

The block loot table has one roll containing glass only when the tool carries Silk Touch level at
least `1`. A player break without that predicate yields no stack; a matching tool returns one glass.
The table has no explosion-survival condition, while explosion loot normally lacks the matching
tool predicate, so ordinary explosion removal yields no glass.

The registered never-spawn predicate rejects the block as support for every entity type.
Independently, `SpawnUtil.Strategy.LEGACY_IRON_GOLEM` rejects exact glass before testing the above
cell and support solidity. Villager golem-summon searches therefore cannot select it as their floor
even though its collision is full; candidate generation, search bounds and summon commit remain
with the mob owners.

Glass is a locked member of `impermeable`, but the only locked runtime consumer is
`BeehiveBlock.trySpawnDripParticles`. `BeehiveBlock.animateTick` passes the beehive's own state to
that helper, not the state below it. Consequently glass membership is never the tested state in the
current vanilla call path and does not block or redirect honey-drip particles. The tag membership is
reloadable, while this non-interaction conclusion is code-locked to the current consumer/caller pair.

The block adds no scheduled tick, random tick, use, attack, entity-contact, neighbor, redstone,
comparator or block-event callback of its own.

**Client projection:**

The sole blockstate variant selects one `cube_all` model whose glass texture explicitly forces
translucency. The item definition selects the same model. `HalfTransparentBlock.skipRendering`
omits a face when its neighbor is the same glass block, otherwise delegating ordinary face culling.
Terrain and block updates project state ID 562; no block entity, conditional model, random variant
or special renderer is involved.

**Branches and aborts:**

Silk Touch match versus empty loot; full collision versus empty visual shape; skylight propagation
versus ordinary opaque damping; plain beacon continuation versus tinted obstruction/colored-glass
handling; generic spawn predicate versus legacy golem support; present tag membership versus
unreachable tested state; same-glass versus other-neighbor face rendering are distinct branches.

**Constants and randomness:**

State ID `562`; hardness/resistance `0.3`; light dampening `0`; shade brightness `1.0`; Silk Touch
minimum `1`; common stack `64`; smelting output `1`, experience `0.1`; full collision/selection cube
and empty visual shape; one block/item model. The block consumes no RNG, and its loot table has no
random function beyond generic one-roll evaluation.

**Side effects:**

Generic placement/removal; conditional self drop; zero-dampening light and beacon continuation;
rejected entity and legacy iron-golem support; ordinary state, item and model projection. The locked
impermeable membership has no glass runtime side effect in the current consumer path.

**Gates:**

Placement/removal authority; Silk Touch predicate; lighting query; beacon scan; entity-spawn
predicate; legacy golem strategy and above cell; beehive caller state; tag/recipe snapshot; smelting
match; client neighbor/model context.

**Boundary cases and quirks:**

No occlusion and empty visual shape do not remove the full collision cube. Full collision does not
make the state valid spawn support. Plain glass propagates skylight while tinted glass does not.
An `impermeable` catalog membership is not evidence of a runtime effect when the sole consumer
never tests a glass state in its locked call path.

**Evidence:**

`OFF-SERVER-001`, `OFF-CLIENT-001`, `OFF-REPORT-001`, `OFF-DATA-001`;
`net.minecraft.world.level.block.Blocks`,
`net.minecraft.world.level.block.TransparentBlock#propagatesSkylightDown`,
`net.minecraft.world.level.block.TransparentBlock#getVisualShape`,
`net.minecraft.world.level.block.TransparentBlock#getShadeBrightness`,
`net.minecraft.world.level.block.HalfTransparentBlock#skipRendering`,
`net.minecraft.world.level.block.state.BlockBehaviour#getLightDampening`,
`net.minecraft.world.item.Items`,
`net.minecraft.world.level.block.entity.BeaconBlockEntity#tick`,
`net.minecraft.util.SpawnUtil$Strategy`,
`net.minecraft.world.entity.npc.villager.Villager#spawnGolemIfNeeded`,
`net.minecraft.world.level.block.BeehiveBlock#animateTick`,
`net.minecraft.world.level.block.BeehiveBlock#trySpawnDripParticles`;
`reports/blocks.json#minecraft:glass`,
`reports/minecraft/components/item/glass.json`,
`data/minecraft/loot_table/blocks/glass.json`,
`data/minecraft/recipe/glass.json`,
`data/minecraft/tags/block/impermeable.json`,
`data/minecraft/tags/item/smelts_to_glass.json`,
`assets/minecraft/blockstates/glass.json`,
`assets/minecraft/models/block/glass.json`,
`assets/minecraft/items/glass.json`.

**Test vectors:**

Run `EXP-BLK-034` across placement, tools with Silk Touch levels 0/1, explosion removal, all faces
of skylight/block-light propagation, beacon scans, entity support, legacy golem support, beehive
honey-level/fluid/RNG/collision paths above and beside glass, tag reload, smelting inputs,
save/reload, adjacent-face culling and block/item rendering. Assert state, shape/predicate/light
results, write/drop/beam/particle order and model/face selection.

**Limits:**

This leaf does not re-specify generic break/placement packets, light propagation, beacon cadence,
entity spawning, villager summon search, beehive particle geometry, furnace timing/recipe matching,
tag publication, map shading or model loading. Those remain with `BLK-002`, `PLY-006`,
`ENV-LIGHT-001`, `BLK-BEACON-001`, entity/mob owners, DataReload, `ITM-RECIPE-001` and `CLI-006`.
