# Blocks mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `BLK-TINTED-GLASS-001` — Tinted glass is visually transparent but fully light-dampening

**Parent:** `BLK-001`, `BLK-002`, `BLK-003`, `BLK-005`, `PLY-005`, `PLY-006`, `ENV-003`,
`ENT-001`, `MOB-001`, `CLI-006`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked registration, class overrides, report, loot/recipe/tag data and client
assets fix the sole state, inherited glass mechanics, light behavior, drops, acquisition and
projection. Direct beacon and iron-golem-support consumers plus the only `impermeable` consumer fix
the exceptional and non-interaction boundaries.

**Applies when:**

`minecraft:tinted_glass` is placed, mined, exploded, used as entity or iron-golem spawn support,
queried by lighting or a beacon, serialized, rendered, crafted or considered by the bundled
`impermeable` tag consumer.

**Authoritative state:**

Tinted glass is a `TintedGlassBlock` with one property-free default state, ID `27161`, and no block
entity. It legacy-copies ordinary glass, then fixes gray map color and no occlusion. The copied
properties supply hat note instrument, glass sound, destroy speed and explosion resistance `0.3`,
a never-valid entity-spawn predicate, and false redstone-conductor, suffocating and view-blocking
predicates. Collision and selection remain a full cube; `TransparentBlock` returns an empty visual
shape and shade brightness `1.0`. Piston reaction remains `NORMAL`.

Its ordinary common-rarity `BlockItem` stacks to `64` and has no special components or use gate.
The locked shaped building recipe places glass at the center and four amethyst shards north, west,
east and south, producing two tinted glass. Generic item placement writes state 27161.

**Transition and ordering:**

`TintedGlassBlock.propagatesSkylightDown` returns false and `getLightDampening` returns `15`,
overriding transparent glass's true skylight propagation. Lighting therefore treats this
non-occluding, non-view-blocking cube as fully dampening; scheduling, propagation, section dirtiness
and client publication remain with `ENV-LIGHT-001`.

Beacon vertical scanning sees the same dampening value `15`. Because exact tinted glass is not the
bedrock exception, it terminates the scan and clears the current beam sections through the existing
`BLK-BEACON-001` transaction. This is distinct from ordinary transparent glass behavior.

The block loot table has one roll containing the tinted-glass item behind only
`survives_explosion`. A player break therefore returns itself without a Silk Touch requirement.
Explosion-caused loot retains the generic survival probability and RNG owned by the explosion/loot
rules; a failed condition yields no stack.

The copied never-spawn predicate rejects the block as support for every entity type. Independently,
`SpawnUtil.Strategy.LEGACY_IRON_GOLEM` rejects exact tinted glass before testing the above cell and
support solidity. Villager golem-summon searches therefore cannot select it as their floor even
though its collision is full; candidate generation, search bounds and summon commit remain with
the mob owners.

Tinted glass is a locked member of `impermeable`, but the only locked runtime consumer is
`BeehiveBlock.trySpawnDripParticles`. `BeehiveBlock.animateTick` passes the beehive's own state to
that helper, not the state below it. Consequently tinted-glass membership is never the tested
state in the current vanilla call path and does not block or redirect honey-drip particles. The tag
membership is reloadable, while this non-interaction conclusion is code-locked to the current
consumer/caller pair.

The block adds no scheduled tick, random tick, use, attack, entity-contact, neighbor, redstone,
comparator or block-event callback of its own.

**Client projection:**

The sole blockstate variant selects one `cube_all` model using the tinted-glass texture. The item
definition selects the same model. Ordinary terrain and block updates project state ID 27161; no
block-entity, conditional model, random variant or special renderer is involved. The texture/model
provides visual translucency while the server-owned state retains dampening 15 and gray map color.

**Branches and aborts:**

Player break versus explosion survival; full collision versus empty visual shape; transparent
model versus false skylight propagation/full dampening; beacon bedrock exception versus tinted-
glass obstruction; generic spawn predicate versus legacy golem support; present tag membership
versus unreachable tested state; block versus item projection are distinct branches.

**Constants and randomness:**

State ID `27161`; hardness/resistance `0.3`; light dampening `15`; shade brightness `1.0`; common
stack `64`; recipe output `2`; full collision/selection cube and empty visual shape; one world
model and the same item model. The block consumes no RNG. Loot explosion survival, beacon timing,
lighting and spawn search retain their owning RNG/state.

**Side effects:**

Generic placement/removal; conditional self drop; lighting and beacon obstruction; rejected entity
and legacy iron-golem support; ordinary state, item and map projection. The locked impermeable
membership has no tinted-glass runtime side effect in the current consumer path.

**Gates:**

Placement/removal authority; loot context and explosion survival; lighting query; beacon scan;
entity-spawn predicate; legacy golem strategy and above cell; beehive caller state; tag snapshot;
recipe match; client model context.

**Boundary cases and quirks:**

No occlusion and empty visual shape do not imply skylight propagation. Full collision does not make
the state valid spawn support. Its loot table deliberately differs from ordinary glass by requiring
no Silk Touch. An `impermeable` catalog membership is not evidence of a runtime effect when the sole
consumer never tests a tinted-glass state in its locked call path.

**Evidence:**

`OFF-SERVER-001`, `OFF-CLIENT-001`, `OFF-REPORT-001`, `OFF-DATA-001`;
`net.minecraft.world.level.block.Blocks`,
`net.minecraft.world.level.block.TintedGlassBlock#propagatesSkylightDown`,
`net.minecraft.world.level.block.TintedGlassBlock#getLightDampening`,
`net.minecraft.world.level.block.TransparentBlock#getVisualShape`,
`net.minecraft.world.level.block.TransparentBlock#getShadeBrightness`,
`net.minecraft.world.item.Items`,
`net.minecraft.world.level.block.entity.BeaconBlockEntity#tick`,
`net.minecraft.util.SpawnUtil$Strategy`,
`net.minecraft.world.entity.npc.villager.Villager#spawnGolemIfNeeded`,
`net.minecraft.world.level.block.BeehiveBlock#animateTick`,
`net.minecraft.world.level.block.BeehiveBlock#trySpawnDripParticles`,
`net.minecraft.data.loot.packs.VanillaBlockLoot#generate`;
`reports/blocks.json#minecraft:tinted_glass`,
`reports/minecraft/components/item/tinted_glass.json`,
`data/minecraft/loot_table/blocks/tinted_glass.json`,
`data/minecraft/recipe/tinted_glass.json`,
`data/minecraft/tags/block/impermeable.json`,
`assets/minecraft/blockstates/tinted_glass.json`,
`assets/minecraft/models/block/tinted_glass.json`,
`assets/minecraft/items/tinted_glass.json`.

**Test vectors:**

Run `EXP-BLK-033` across placement, player/tool/Silk Touch breaks, explosion-survival boundaries,
all faces of skylight/block-light propagation, beacon scans, entity support, legacy golem support,
beehive honey-level/fluid/RNG/collision paths above and beside tinted glass, tag reload, recipe,
save/reload, map color and block/item rendering. Assert state, shape/predicate/light results, write/
drop/beam/particle order and the shared model.

**Limits:**

This leaf does not re-specify generic break/placement packets, explosion RNG, light propagation,
beacon cadence, entity spawning, villager summon search, beehive particle geometry, recipe matching,
tag publication, map shading or model loading. Those remain with `BLK-002`, `PLY-006`,
`RED-EXPLOSION-001`, `ENV-LIGHT-001`, `BLK-BEACON-001`, entity/mob owners, DataReload,
`ITM-RECIPE-001` and `CLI-006`.
