# Blocks mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `BLK-CONCRETE-001` — Concrete is a solid dye-colored block and powder-solidification target

**Parent:** `BLK-001`, `BLK-002`, `BLK-003`, `BLK-005`, `PLY-005`, `PLY-006`, `ENV-003`,
`ENT-001`, `CLI-006`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — the locked color-collection registration, ordinary-block inheritance, paired
concrete-powder callbacks, reports, tags, loot tables and client assets exhaust the sixteen
identities, state schema, physical behavior, acquisition targets and projection.

**Applies when:**

Any `minecraft:<color>_concrete` identity is placed, mined, exploded, queried for shape/light/spawn
support, written by its paired concrete powder, serialized, rendered or used as a sulfur cube's
body item.

**Authoritative state:**

`Blocks.CONCRETE` registers one ordinary `Block` for each `DyeColor` in white, orange, magenta,
light blue, yellow, lime, pink, gray, light gray, cyan, purple, blue, brown, green, red and black
order. Every identity has one property-free state and no block entity:

| Color | State |
|---|---:|
| white | 15030 |
| orange | 15031 |
| magenta | 15032 |
| light blue | 15033 |
| yellow | 15034 |
| lime | 15035 |
| pink | 15036 |
| gray | 15037 |
| light gray | 15038 |
| cyan | 15039 |
| purple | 15040 |
| blue | 15041 |
| brown | 15042 |
| green | 15043 |
| red | 15044 |
| black | 15045 |

Registration selects the matching dye map color and supplies bass-drum note instrument,
`requiresCorrectToolForDrops`, destroy speed and explosion resistance `1.8`. All other properties
remain ordinary defaults: stone sound, a full selection/collision/visual/occlusion cube, emission
`0`, friction `0.6`, speed/jump factors `1`, restitution `0`, and piston reaction `NORMAL`.

The full solid-render cube does not propagate skylight, has light dampening `15` and shade
brightness `0.2`. Its default full-motion predicates make it a redstone conductor, suffocating and
view-blocking block. The default spawn-support predicate accepts its sturdy upper face because its
emission is below `14`; entity-type placement rules remain an additional independent gate.

Each registered ordinary `BlockItem` has common rarity, maximum stack size `64`, its corresponding
block item name/model and no special use component or placement gate. Placement writes the
property-free state paired with the item color.

**Transition and ordering:**

#### Placement, powder solidification and removal

Direct item placement and explicit state writes use generic block placement. Concrete itself has
no water, fluid, gravity, scheduled-tick or shape-update override: after it exists, adjacent or
occupying water neither changes it nor turns it into a falling entity.

The transition into concrete belongs to the paired `ConcretePowderBlock`. The sixteen powder
instances retain a direct reference to the matching member of `Blocks.CONCRETE`:

1. Powder placement or a neighbor-shape update tests water at its own position, then neighboring
   water whose facing side is not sturdy; a match returns the paired concrete state.
2. A fast falling powder entity can collider-raycast its previous/current path for source water.
   Its landing callback applies the same solidification predicate and replaces the powder with the
   paired concrete.
3. The committed result is exactly the property-free concrete state for the powder's dye color.
   It does not inherit falling-entity data or retain a powder marker.

Scheduling, falling-entity motion, water-hit ordering, placement failure and drop fallback remain
with `BLK-FALL-001`; this leaf owns the resulting concrete identity and its later behavior.

Correct-tool admission remains with generic player breaking. When harvest admission succeeds,
each block's one-roll loot table returns its own color subject to `survives_explosion`. An admitted
ordinary player break therefore returns the matching block item; a player break without an
effective correct tool returns nothing. Explosion loot evaluates the generic survival condition
using the active explosion context and may suppress the self item.

There is no direct recipe whose output is a finished concrete block. Locked coloring recipes
instead return eight concrete powder blocks from four sand, four gravel and the matching dye;
water-driven powder solidification is the code-built conversion into this family.

#### Tags and entity consumer

All sixteen block identities are direct members of reloadable `minecraft:concrete` and
`minecraft:mineable/pickaxe`; all sixteen ordinary block items are direct members of the matching
item `minecraft:concrete` tag. The pickaxe tag participates in generic tool speed/harvest
selection. No locked gameplay class branches on the block `concrete` tag.

The item tag is included by `minecraft:sulfur_cube_archetype/slow_bouncy`. When a sulfur cube
equips any concrete color in its body slot and rebuilds matching archetypes, that item therefore
selects the slow-bouncy archetype's attributes, knockback settings and sounds. The locked
knockback pair is `(horizontal, vertical)=(0.4125, 0.24)` and its hit sound is
`minecraft:entity.sulfur_cube.slow_bouncy.hit`; matching order, other archetype effects, equipment
state and knockback calculation remain with `ENT-KNOCKBACK-001`.

The ordinary block class adds no use, attack, entity-contact, random/scheduled tick, neighbor,
redstone, comparator or block-event callback.

**Client projection:**

Every property-free blockstate selects its matching `block/<color>_concrete` model. Each model
inherits opaque `cube_all` and selects the same-color concrete texture; the item definition selects
the same block model. There is no weighted, conditional, animated or special-renderer branch.

Terrain packets publish states `15030..15045`. Ordinary full-cube face culling, map shading,
breaking particles/sounds and item rendering consume the selected state/model through their
generic owners.

**Branches and aborts:**

Sixteen identity/color pairs; direct placement versus paired-powder conversion; dry/ordinary water
contact after conversion versus powder water admission; correct versus incorrect harvest tool;
surviving versus suppressed explosion loot; generic full-cube spawn support versus entity-type
placement rejection; block-tag versus item-tag consumers; empty versus concrete-equipped sulfur
cube; and one opaque model per color are distinct.

**Constants and randomness:**

States `15030..15045` in dye order; hardness/resistance `1.8`; emission `0`; dampening `15`; shade
brightness `0.2`; friction `0.6`; speed/jump `1`; restitution `0`; common stack `64`; full
selection/collision/visual cube; spawn-light ceiling `14`; slow-bouncy knockback
`(0.4125, 0.24)`. Concrete consumes no RNG directly. Generic explosion survival and sulfur-cube
transactions own any randomness they invoke.

**Side effects:**

Generic placement/removal and state publication; color-matched powder-to-concrete replacement;
conditional same-color block-item loot; solid light/shape/spawn support; ordinary persistence and
model projection; reload-selected mining and sulfur-archetype membership. Concrete has no
independent fluid or falling side effect after conversion.

**Gates:**

Selected color identity; placement/removal authority; paired powder solidification predicate;
correct-tool harvest admission; loot explosion context; sturdy upper face and entity placement
rules; active block/item tag and archetype snapshots; sulfur-cube body equipment; client
state/model context.

**Boundary cases and quirks:**

Water converts concrete powder, not finished concrete. The resulting full cube blocks skylight and
uses ordinary stone sound even though its note-block instrument is bass drum. `requiresCorrectTool`
suppresses player harvest independently of the loot table's explosion-survival condition. The
block `concrete` tag has no current code consumer, while the same-named item tag materially selects
the sulfur cube's slow-bouncy archetype.

**Evidence:**

`OFF-SERVER-001`, `OFF-CLIENT-001`, `OFF-REPORT-001`, `OFF-DATA-001`;
`net.minecraft.world.level.block.Blocks`,
`net.minecraft.world.level.block.ColorCollection#registerBlocks`,
`net.minecraft.world.item.DyeColor#getMapColor`,
`net.minecraft.world.level.block.state.BlockBehaviour#getShape`,
`net.minecraft.world.level.block.state.BlockBehaviour#getCollisionShape`,
`net.minecraft.world.level.block.state.BlockBehaviour#getVisualShape`,
`net.minecraft.world.level.block.state.BlockBehaviour#propagatesSkylightDown`,
`net.minecraft.world.level.block.state.BlockBehaviour#getLightDampening`,
`net.minecraft.world.level.block.state.BlockBehaviour#getShadeBrightness`,
`net.minecraft.world.level.block.ConcretePowderBlock#getStateForPlacement`,
`net.minecraft.world.level.block.ConcretePowderBlock#updateShape`,
`net.minecraft.world.level.block.ConcretePowderBlock#onLand`,
`net.minecraft.world.entity.monster.cubemob.SulfurCube#collectEquipmentChanges`,
`net.minecraft.world.entity.monster.cubemob.SulfurCube#matchingArchetypes`;
`reports/blocks.json#*_concrete`,
`reports/minecraft/components/item/*_concrete.json`,
`data/minecraft/tags/block/concrete.json`,
`data/minecraft/tags/block/mineable/pickaxe.json`,
`data/minecraft/tags/item/concrete.json`,
`data/minecraft/tags/item/sulfur_cube_archetype/slow_bouncy.json`,
`data/minecraft/sulfur_cube_archetype/slow_bouncy.json`,
`data/minecraft/loot_table/blocks/*_concrete.json`,
`data/minecraft/recipe/*_concrete_powder.json`,
`assets/minecraft/blockstates/*_concrete.json`,
`assets/minecraft/models/block/*_concrete.json`,
`assets/minecraft/items/*_concrete.json`.

**Test vectors:**

Run `EXP-BLK-041` across all sixteen identities, direct placement, explicit writes, powder
placement/neighbor/fast-fall water conversion, dry/wet finished-concrete updates, correct and
incorrect tools, explosion powers, shape/light/spawn queries, tag reload, sulfur-cube equipment,
save/reload and every block/item model. Assert state/color pairs, conversion timing, later
inertness, writes/drops, physical predicates, archetype values and selected models.

**Limits:**

This leaf does not re-specify generic placement/break packets, tool speed, explosion loot
probability, light propagation, falling-entity motion, concrete-powder recipes, entity spawn
placement, sulfur-cube equipment/knockback or model loading. Those remain with `BLK-002`,
`PLY-006`, `ITM-LOOT-001`, `ENV-LIGHT-001`, `BLK-FALL-001`, `ITM-RECIPE-001`, entity owners,
`ENT-KNOCKBACK-001` and `CLI-006`.
