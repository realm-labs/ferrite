# World mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `WGEN-STRUCTURE-END-CITY-001` — End cities transact recursive template groups and share a process-global ship latch

**Parent:** `WGEN-003`

**FidelityClass:** `EquivalentPlayerVisibleBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked 26.2 source fixes the rotated four-corner start-height gate, complete
house/tower/bridge/fat-tower recursion, grouped collision transaction, global ship latch, all 20
template inputs, overwrite modes, marker-created shulkers/Elytra frame and twice-seeded treasure
chests, persistence and the sole loot record. The structure record, random-spread set and exact
two-biome tag are audited data-only. Shared random-spread admission and start/reference/placement
scheduling remain owned by `WGEN-PIPELINE-001`; later entity and loot evaluation remain with
`ENT-LIFECYCLE-001` and `ITM-LOOT-001`.

**Applies when:**

The `end_cities` set selects its sole record, an admitted End-city graph is constructed, or one of
its retained template pieces intersects a placement chunk.

**Authoritative state:**

`minecraft:end_city` uses `surface_structures`, default `none` terrain adaptation, no spawn
overrides and exactly End highlands plus End midlands through `#minecraft:has_structure/end_city`.
Its set contains only that record at weight `1`; triangular random spread uses spacing `20`,
separation `11`, salt `10387313`, default frequency and locate offset. Every piece has registry ID
`ecp`, template namespace `minecraft:end_city/`, mirror none, ignored raw entities and forced
generic orientation north. It persists generic box/depth state, template name and `TPX/TPY/TPZ`,
plus `Rot` and `OW`; load recomputes the rotated box from the template position. `OW=true` selects
the structure-block-only ignore processor, so template air overwrites live blocks; false selects
structure-and-air ignore, preserving live blocks at template-air cells. Missing template fields
default empty/zero and missing `OW` decodes false; missing or invalid required `Rot` fails decode.

**Transition and ordering:**

Construction first consumes `nextInt(4)` in enum order none, clockwise `90`, clockwise `180`,
counterclockwise `90`. The anchor is always source-chunk local `(7,7)`. It queries
`WORLD_SURFACE_WG` at that corner and at the three corners reached by signed `(sizeX,sizeZ)`:
`(5,5)`, `(-5,5)`, `(-5,-5)` or `(5,-5)` for those rotations, and takes the minimum first-occupied
height. A result below `60` rejects before any graph draws; otherwise that exact anchor is the
generic 3-D biome-test stub. Deferred construction resets all four static generators, creates
overwrite `base_floor`, then connects nonoverwrite `second_floor_1` at `(-1,0,-1)`, nonoverwrite
`third_floor_1` at `(-1,4,-1)`, and overwrite `third_roof` at `(-1,8,-1)`. It requests a depth-`1`
tower and ignores rejection, so the four-piece base always remains.

Each child template initially shares its parent's template position. With mirror none and zero
pivots, `calculateConnectedPosition(parentSettings,offset,childSettings,ZERO)` is exactly the parent
rotation applied to `offset`; the child moves by that vector. A recursive request above depth `8`
returns false without draws. Otherwise the generator builds into a private creation-ordered list. On
generator success, one full signed `nextInt()` becomes the group tag. It visits candidates in
creation order, overwrites that candidate's generation depth—including any nested tag—and then tests
the first inclusive-box collider in the already-retained list. No collider or a collider whose
generation depth equals the request parent's tag is permitted; the first differently tagged collider
rejects the **entire** candidate list before later candidates are visited. Acceptance therefore
flattens every candidate to the one outer tag and appends the list atomically. Tags are random
integers, not unique identifiers, so coincidental equality across separately committed transactions
also permits overlap. Generator RNG and static mutations are never rolled back after rejection.

**House and ordinary tower generators:**

A house also rejects above depth `8`. At its supplied offset it adds overwrite `base_floor`, then
consumes `nextInt(3)`. Zero connects overwrite `base_roof` at `(-1,4,-1)`. One connects nonoverwrite
`second_floor_2` at `(-1,0,-1)`, nonoverwrite `second_roof` at `(-1,8,-1)`, and requests a
depth-plus-one tower. Two instead uses `second_floor_2`, nonoverwrite `third_floor_2` at
`(-1,4,-1)`, overwrite `third_roof` at `(-1,8,-1)`, then requests that tower. Both nested results
are ignored and the house returns true.

A tower consumes two `nextInt(2)` values and connects overwrite `tower_base` at `(3+dx,-3,3+dz)`,
then `tower_piece` at `(0,7,0)`. `nextInt(3)==0` remembers that piece as a possible bridge level. It
consumes `1+nextInt(3)` for one through three more `tower_piece` templates at `(0,4,0)`; after each
nonfinal piece, one boolean may replace the remembered bridge level. When one is remembered, four
booleans in fixed order offer overwrite `bridge_end` starts at rotation/offset `(none,(1,-1,0))`,
`(clockwise 90,(6,-1,1))`, `(counterclockwise 90,(0,-1,5))`, `(180,(5,-1,6))`; every admitted start
requests a bridge at depth plus one, with that result ignored. The tower then caps its latest piece
with overwrite `tower_top` at `(-1,4,-1)`. With no remembered level, depth exactly `7` adds that
same cap; another depth returns the depth-plus-one fat-tower recursive result, so a failed/colliding
fat group rejects the whole ordinary-tower candidate group.

**Bridge and ship generator:**

A bridge consumes `1+nextInt(4)` for length. It starts with overwrite `bridge_piece` at `(0,0,-4)`.
Each segment consumes a boolean: true adds another `bridge_piece` at `(0,nextY,-4)` and resets
`nextY=0`; false consumes a second boolean and adds `bridge_steep_stairs` at `(0,nextY,-4)` when
true or `bridge_gentle_stairs` at `(0,nextY,-8)` when false, then sets `nextY=4`. The temporary
first and final endpoints are assigned tag `-1`, but the enclosing successful group later overwrites
it.

The bridge generator owns a process-global `shipCreated` boolean reset at each city start. When
already true it consumes no ship-selection integer and requests a house. Otherwise
`nextInt(10-genDepth)` equal to zero connects an overwrite ship at
`(-8+nextInt(8),nextY,-70+nextInt(10))` and sets the latch; a nonzero result requests a house at
`(-3,nextY+1,-11)`. House rejection makes the bridge generator return false before its far endpoint.
The ship path and every successful house path finish with reverse-rotated overwrite `bridge_end` at
`(4,nextY,0)`. Setting `shipCreated` precedes enclosing collision validation: even a later rejected
ship group suppresses every subsequent ship candidate until another city start resets the singleton.
The latch and four generator singletons are unsynchronized process-global state; noninterleaved
construction admits at most one ship candidate per city, while an interleaving reset can suppress
another city's candidate or let an earlier city select again. A fixed caller schedule observes those
shared values, while permissible scheduling is owned by the pipeline rule.

**Fat towers:**

A fat tower connects overwrite `fat_tower_base` at `(-3,4,-3)` and `fat_tower_middle` at `(0,4,0)`.
It attempts at most two more middle levels: each attempted level consumes `nextInt(3)`, stops at
zero, and otherwise adds a middle at `(0,8,0)`. Each added level consumes four booleans in fixed
rotation/offset order `(none,(4,-1,0))`, `(clockwise 90,(12,-1,4))`,
`(counterclockwise 90,(0,-1,8))`, `(180,(8,-1,12))`; admitted `bridge_end` starts request
depth-plus-one bridges and ignore their results. Finally overwrite `fat_tower_top` connects at
`(-2,8,-2)`.

**Locked template audit:**

All inputs have one palette, block lists that include air, no structure void, no jigsaws and no raw
entities. Every list is dense over its declared volume except `tower_base`, whose `202` entries
leave `141` coordinates absent and therefore untouched even in overwrite mode. Counts include
structure markers, which placement processors remove:

#### `base_floor`, `base_roof`

**Size:**

`10×4×10`, `12×2×12`

**Blocks / air:**

`400/250`, `288/140`

**Marker or fixed-NBT payload:**

Sentry `(3,2,9)`, `(6,2,9)`; none

#### `second_floor_1`, `second_floor_2`, `second_roof`

**Size:**

`12×8×12`, `12×8×12`, `14×2×14`

**Blocks / air:**

`1152/890`, `1152/906`, `392/192`

**Marker or fixed-NBT payload:**

none; Sentry `(8,5,6)`; none

#### `third_floor_1`, `third_floor_2`, `third_roof`

**Size:**

`14×8×14`, `14×8×14`, `16×2×16`

**Blocks / air:**

`1568/1213`, `1568/1235`, `512/252`

**Marker or fixed-NBT payload:**

none; Sentry `(2,5,2)`, `(11,5,2)`, Chest `(6,6,2)` above chest `(6,5,2)`, plus ender chest
`(7,5,2)`; none

#### `tower_base`, `tower_floor`, `tower_piece`, `tower_top`

**Size:**

`7×7×7`, `7×4×7`, `7×4×7`, `9×5×9`

**Blocks / air:**

`202/112`, `196/108`, `196/140`, `405/224`

**Marker or fixed-NBT payload:**

only top has Sentry `(4,3,4)` and eight fixed magenta wall banners with the two black triangle
patterns

#### `bridge_end`, `bridge_piece`

**Size:**

`5×6×2`, `5×6×4`

**Blocks / air:**

`60/29`, `120/86`

**Marker or fixed-NBT payload:**

none

#### `bridge_gentle_stairs`, `bridge_steep_stairs`

**Size:**

`5×7×8`, `5×7×4`

**Blocks / air:**

`280/202`, `140/99`

**Marker or fixed-NBT payload:**

none

#### `fat_tower_base`, `fat_tower_middle`, `fat_tower_top`

**Size:**

`13×4×13`, `13×8×13`, `17×6×17`

**Blocks / air:**

`676/501`, `1352/1098`, `1734/1084`

**Marker or fixed-NBT payload:**

middle Sentries `(2,2,6)`, `(10,2,6)`, `(6,6,2)`, `(6,6,10)`; top Chest `(3,2,11)`, `(5,2,13)` above
matching chests

#### `ship`

**Size:**

`13×24×29`

**Blocks / air:**

`9048/8113`

**Marker or fixed-NBT payload:**

Sentry `(6,4,8)`, `(8,6,27)`, `(4,11,27)`; Chest `(5,5,7)`, `(7,5,7)` above matching chests; Elytra
`(6,5,7)`; fixed brewing stand `(6,7,25)` with strong-healing potions in slots `0` and `2`; dragon
wall head `(6,8,0)`

`tower_floor` is an audited official input but no source edge names it, so it is unreachable from
ordinary locked generation. All other 19 templates are reachable. The eight `tower_top` banner
positions are `(0,0,2)`, `(0,0,6)`, `(2,0,0)`, `(2,0,8)`, `(6,0,0)`, `(6,0,8)`, `(8,0,2)`,
`(8,0,6)`.

**Placement and markers:**

Before each invocation the processing box becomes the template clip and the rotated piece box is
recomputed. The single palette is position-deterministic. Because the sole ignore processor does not
request whole-piece state, coordinates are transformed and clipped to the current processing box
before processor application: structure markers never write; nonoverwrite pieces also drop air,
while overwrite pieces offer every listed air cell. Retained blocks use the generic flags-`2`
template transaction: an NBT cell first offers a barrier with flags `820`, then the actual rotated
state; a failed second offer may leave the barrier. Successful writes participate in default fluid
preservation, iterative source-water fill, neighbor-shape repair and notification. A resulting
`RandomizableContainer` consumes one caller `nextLong`, injects `LootTableSeed` into the copied NBT
and loads it. Thus each successfully written audited ordinary chest consumes a first seed even
though its NBT initially contains only empty `Items`; ender chest, brewing stand, skull and banners
do not. Raw entities are ignored and also absent. Every official input is positive-sized with a
nonempty palette list, so placement returns true even when clipping or processing retained no block.

After a true placement result, marker filtering uses the same clip and handles only `DATA` mode.
`Chest*` targets the cell below and separately requires that target inside the processing box; only
a resulting `RandomizableContainer` consumes another `nextLong` and installs
`chests/end_city_treasure`, overwriting the generic seed as the observable loot seed. `Sentry*`
additionally requires the marker in spawnable world bounds, creates a shulker with `STRUCTURE`, sets
its position to `(x+.5,y,z+.5)` without finalization, and ignores entity-admission failure.
`Elytra*` creates an item frame at the marker facing saved-rotation-applied south, installs one
Elytra without sound, and likewise ignores admission failure. Unknown markers do nothing.
NBT-container draws for a piece occur before its creation-order marker draws; piece invocations
share the placement-chunk caller RNG.

**Locked loot record:**

`chests/end_city_treasure` uses its matching random sequence. Pool one rolls uniformly `2..6`:
diamond weight `5` count `2..7`; iron ingot `10` count `4..8`; gold ingot `15` count `2..7`; emerald
`2` count `2..6`; beetroot seeds `5` count `1..10`; saddle `3`; default-weight copper, iron, golden
and diamond horse armor; and weight-`3` diamond sword, spear, boots, chestplate, leggings, helmet,
pickaxe and shovel plus iron sword, boots, chestplate, leggings, helmet, pickaxe and shovel. Every
one of those weight-`3` equipment entries uses `enchant_with_levels` uniformly `20..39` from
`#minecraft:on_random_loot`. Pool two makes one roll between empty weight `14` and one spire
armor-trim template weight `1`. Elytra is supplied only by the ship marker, not this table. Loot
evaluation stays deferred to `ITM-LOOT-001`; this leaf owns record linkage and seed installation.

**Branches and aborts:**

Four rotation/height footprints; height `59/60`; biome failure; depth `8/9`; every house floor
endpoint, tower offset/height/bridge level and bridge boolean; bridge length/straight/steep/gentle,
ship-latch short circuit, denominators and ship offsets; fat-level stop and bridge booleans;
generator false, first-collider equal/different parent tag, transactional accept/reject and
ignored/propagated nested result; all overwrite/air, clip, barrier/state, fluid/shape and typed-NBT
outcomes; every marker prefix, spawn bounds, creation and admission outcome; saved/missing/invalid
fields.

**Constants and randomness:**

Start rotation precedes height rejection. Accepted graph draws follow the recursive creation order
exactly; every successful generator consumes its group-tag `nextInt()` after all nested generator
draws and mutations. Ship selection consumes `nextInt(10-genDepth)` only while the global latch is
false. Placement consumes one long per successfully written typed NBT container, then one per typed
chest marker; shulker/frame construction and admission consume no caller RNG here. Neighbor-shape
callbacks can use the level RNG, and loot owns its separate sequence.

**Side effects:**

A retained, collision-grouped graph of rotated template pieces; process-global ship
suppression/reset during construction; chunk-clipped
purpur/end-stone/end-rod/stair/slab/fence/banner/furniture placement with overwrite-sensitive air,
water/shape updates and persisted settings; seeded treasure chests; fixed brewing stand contents and
dragon head; marker shulkers; under noninterleaved construction at most one ship candidate and its
Elytra item frame.

**Gates:**

Caller-owned random-spread/start/reference/placement scheduling; four live heightmap probes and
exact start biome; depth, generator return and group collision; static ship state; processing box,
processor, live block/fluid/write and resulting block-entity/entity types; spawnable bounds.

**Boundary cases and quirks:**

The start anchor remains local `(7,7)` while only its signed height footprint rotates. Nested
recursion can consume RNG and set the ship latch before an outer collision discards all generated
pieces. Same-tag overlap is deliberately admitted; all candidate tags are flattened to the outer
group at commit. House/tower bridge results are ignored, but bridge-to-house failure and
tower-to-fat failure propagate. The global ship latch records candidate creation, not successful
graph commit or entity placement, and another city's reset can change the first city's later choices
under interleaving. `tower_floor` is dead locked data. Normal chests consume a generic seed
immediately superseded by the marker seed; rejected writes, wrong entities or different chunk
visitation alter the caller trace.

**Evidence:**

`Confirmed`; `OFF-SERVER-001`; `OFF-DATA-001`; `OFF-REPORT-001`. Anchors:
`net.minecraft.world.level.levelgen.structure.Structure#getLowestYIn5by5BoxOffset7Blocks`;
`net.minecraft.world.level.levelgen.structure.structures.EndCityStructure#findGenerationPoint`,
`#generatePieces`;
`net.minecraft.world.level.levelgen.structure.structures.EndCityPieces#startHouseTower`,
`#addPiece`, `#recursiveChildren`; all four `SectionGenerator` implementations;
`EndCityPieces$EndCityPiece#makeSettings`, persistence and marker handler;
`TemplateStructurePiece#postProcess`; `StructureTemplate#placeInWorld`, `#filterBlocks`;
`RandomizableContainer#setBlockEntityLootTable`; all 20 `data/minecraft/structure/end_city/*.nbt`
inputs; the structure/set/tag records, `ecp` registry entry and
`loot_table/chests/end_city_treasure.json`.

**Test vectors:**

Replay all rotations and four signed height footprints at `59/60`; fixed graph streams over every
house/tower/bridge/fat endpoint, depth `8/9`, nested false propagation, same/different-tag collision
and rejected-ship mutation; assert exact connected positions, group tags and all 20 template audits
including dead `tower_floor`. Across every orientation/overwrite mode and adversarial
clip/write/fluid/entity result, trace NBT and marker draw order, all marker coordinates, ship fixed
payloads, persistence/reload and exact record/set/tag/piece/loot decodes. Use `EXP-WGEN-001` only
for separately owned random-spread/distribution calibration.
