# World mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `WGEN-STRUCTURE-STRONGHOLD-001` — Strongholds retry a weighted piece graph until one portal room exists

**Parent:** `WGEN-003`

**FidelityClass:** `EquivalentPlayerVisibleBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked 26.2 source fixes the retry seed, complete weighted graph, collision and
range gates, all 13 piece families, vertical relocation, chunk-clipped masonry and furniture, four
door forms, chest/spawner/portal transactions, all three loot records and persistence. The structure
record, concentric-rings set and both biome tags are audited data-only. Shared ring-position
selection, start/reference lifecycle and placement-chunk scheduling remain owned by
`WGEN-PIPELINE-001`; later silverfish ticking remains owned by `ENT-LIFECYCLE-001`.

**Applies when:**

The `strongholds` concentric-rings set selects its sole record, a stronghold start graph is
constructed, or one of its retained pieces intersects a placement chunk.

**Authoritative state:**

`minecraft:stronghold` uses `surface_structures`, `bury` terrain adaptation, no spawn overrides and
biome holder `#minecraft:has_structure/stronghold`; that tag delegates to `#minecraft:is_overworld`
and therefore contains all 55 locked Overworld biomes. Its set contains only stronghold at weight
`1`, uses concentric rings with distance `32`, spread `3`, count `128`, salt `0`, and prefers
`#minecraft:stronghold_biased_to`. The preferred tag's exact 39 holders are plains, sunflower
plains, snowy plains, ice spikes, desert, forest, flower forest, birch forest, dark forest, pale
garden, old-growth birch/pine/spruce forests, taiga, snowy taiga, savanna, savanna plateau,
windswept hills/gravelly hills/forest/savanna, jungle, sparse jungle, bamboo jungle, badlands,
eroded badlands, wooded badlands, meadow, cherry grove, grove, snowy slopes, frozen/jagged/stony
peaks, mushroom fields, dripstone caves, lush caves and sulfur caves.

**Transition and ordering:**

The generation-point stub is unconditionally the source chunk's minimum X, Y `0`, minimum Z, and the
generic structure gate samples that exact 3-D biome position. After admission, attempts are
unbounded until the builder is nonempty and the start piece's transient portal pointer is nonnull.
Attempt `a`, beginning at zero, clears the builder and reseeds the structure RNG with
`setLargeFeatureSeed(worldSeed+a,chunkX,chunkZ)`. It resets the static graph state, creates a source
`StartPiece` at `(chunkBlockX(2),64,chunkBlockZ(2))` with one uniform horizontal direction, adds it,
and expands its children. While pending pieces remain, it removes one uniform list index and expands
that piece. A failed portal search discards the complete graph and retries with `a+1`; successful
construction finally relocates the graph below sea level.

The selection table is fixed and ordered:

| Piece | Weight | Maximum | Extra admission |
|---|---:|---:|---|
| straight | 40 | unlimited | — |
| prison hall | 5 | 5 | — |
| left turn | 20 | unlimited | — |
| right turn | 20 | unlimited | — |
| room crossing | 10 | 6 | — |
| straight stairs down | 5 | 5 | — |
| stairs down | 5 | 5 | — |
| five crossing | 5 | 4 | — |
| chest corridor | 5 | 4 | — |
| library | 10 | 2 | child depth greater than `4` |
| portal room | 20 | 1 | child depth greater than `5` |

Initial total weight is `145`. Reset restores every placement count and the complete list. Before an
ordinary selection, total weight is recomputed across the current list, but expansion remains
enabled only while at least one **finite** entry has quota remaining; unlimited entries alone do not
keep it alive. A finite entry is removed immediately on reaching its maximum. The source stairs
impose a five crossing for their first forward request. If that factory succeeds, the imposed piece
neither increments its quota nor becomes the remembered previous weighted choice.

An ordinary selection makes at most five attempts. Each consumes one `nextInt(totalWeight)` and
subtracts weights in list order. Reaching an entry that fails its depth/quota admission or is the
same `PieceWeight` object as the previous successful weighted choice aborts that entire attempt. If
its factory returns null, iteration continues through later entries with the already-negative
accumulator; this can cascade to later factories without another draw, while an ineligible or
repeated later entry still aborts the attempt. Success increments the entry count, remembers it,
removes an exhausted finite entry and returns. Five failed attempts fall through to a filler
corridor.

Every requested child uses parent depth plus one. A parent above depth `50` is rejected, so depth
`50` may create a terminal depth-`51` piece. A proposed anchor is rejected when
`abs(anchorX-start.minX)>112` or the corresponding Z difference is greater than `112`; equality is
admitted. Ordinary factories also require proposed minimum Y greater than `10` and no intersection
with any existing box. A successful proposal is appended to both the builder and pending list.

**Piece boxes and exits:**

Source stairs use `makeBoundingBox(west,64,north,direction,5,11,5)`, persist the source flag and
open entry, and request one forward five crossing. The remaining factories use these oriented local
boxes and child exits; `F/L/R` mean forward/left/right relative to the piece orientation:

#### straight

**Box offset and size:**

`(-1,-1,0)`, `5x5x7`

**Constructor state:**

random door; two booleans

**Child exits:**

F and optional L/R

#### prison hall

**Box offset and size:**

`(-1,-1,0)`, `9x5x11`

**Constructor state:**

random door

**Child exits:**

F

#### left/right turn

**Box offset and size:**

`(-1,-1,0)`, `5x5x5`

**Constructor state:**

random door

**Child exits:**

orientation-dependent L/R

#### room crossing

**Box offset and size:**

`(-4,-1,0)`, `11x7x11`

**Constructor state:**

random door; `nextInt(5)` room type

**Child exits:**

F/L/R

#### straight stairs down

**Box offset and size:**

`(-1,-7,0)`, `5x11x8`

**Constructor state:**

random door

**Child exits:**

F

#### stairs down

**Box offset and size:**

`(-1,-7,0)`, `5x11x5`

**Constructor state:**

random door

**Child exits:**

F

#### five crossing

**Box offset and size:**

`(-4,-3,0)`, `10x9x11`

**Constructor state:**

random door; three booleans; high-right is `nextInt(3)>0`

**Child exits:**

F plus flagged low/high sides

#### chest corridor

**Box offset and size:**

`(-1,-1,0)`, `5x5x7`

**Constructor state:**

random door

**Child exits:**

F

#### library

**Box offset and size:**

`(-4,-1,0)`, first `14x11x15`, fallback `14x6x15`

**Constructor state:**

collision decides tall; door drawn only after success

**Child exits:**

none

#### portal room

**Box offset and size:**

`(-4,-1,0)`, `11x8x16`

**Constructor state:**

no constructor draw

**Child exits:**

none; records source portal pointer

Filler first proposes a `5x5x4` box and requires its first creation-order collider to have the same
minimum Y. It tests lengths `2` then `1`; the first shortened box that no longer intersects yields a
filler whose `steps` are respectively `3` or `2`, whose minimum Y must exceed `1`, and whose final
layer deliberately overlaps the blocker. Its random argument is unused.

**Relocation and persistence:**

Let `B` be the union box, target `seaLevel-10`, and `j=B.ySpan+minBuildY+1`. When `j<target`,
relocation consumes `nextInt(target-j)` and adds it to `j`; otherwise it consumes no draw. Every
piece moves by `j-B.maxY`, making the final union maximum Y exactly `j` and at most `seaLevel-11` on
the randomized branch. No collision, biome or build-range gate is repeated after movement.

Every piece persists registry ID, box, orientation and generation depth. Stronghold pieces
additionally require `EntryDoor`; missing subclass values decode false or zero. The source flag,
straight side booleans, room type, four five-crossing flags, chest latch, library tall flag, portal
spawner latch and filler steps persist. The start's previous choice, pending list and portal pointer
do not. While the pointer exists, the structure locator is the portal-room center; after reload it
falls back to the start-piece center. The 13 exact piece registry IDs are `shstart`, `shs`, `shph`,
`shlt`, `shrt`, `shrc`, `shssd`, `shsd`, `sh5c`, `shcc`, `shli`, `shpr` and `shfc`. The
weighted-list/imposed/total variables are unsynchronized process-global statics, not per-start
state; a fixed caller schedule follows those shared variables exactly, while scheduling/concurrency
guarantees remain with the pipeline owner.

**Shared placement transaction:**

Intersecting pieces run in creation order and share the placement chunk's caller RNG. Local
coordinates transform through orientation. `placeBlock` rejects positions outside the supplied
processing box, otherwise mirrors then rotates the state and offers flags `2`, ignoring the result.
It rereads fluid afterward and schedules a delay-`0` fluid tick when nonempty. Its shape-sensitive
state set—nether-brick and every listed wood fence, torch, wall torch, ladder and iron bars—is
marked for postprocessing even when the write failed. Stronghold uses oak fence, both torch forms,
ladder and bars from that set. Direct air and furniture offers otherwise follow this same
transaction.

`generateBox` traverses local Y outermost, X middle and Z innermost. For a randomized shell, an
interior cell is always cave air with no draw; an edge consumes one float and selects cracked stone
bricks below `.2`, mossy stone bricks below `.5`, infested stone bricks below `.55`, otherwise stone
bricks. All ordinary outer shells pass `skipAir=true`: an off-chunk read is air and suppresses
selector draw/write, as does existing in-chunk air. Portal-room outer generation uniquely passes
false, so its complete `11x8x16` box is evaluated on every intersecting-chunk invocation—edge draws
occur even for off-chunk cells although writes remain clipped. Internal selector boxes also pass
false and similarly draw over their whole declared range each invocation. Library cobweb placement
visits the full `X=2..11,Y=1..4,Z=1..13` volume (520 floats) on every invocation and admits a cobweb
exactly when `nextFloat()<=.07`; clipping is tested only after that gate. Straight's four torch
gates likewise always consume their strict `<.1` floats before clipping.

The random small-door draw is `nextInt(5)`: `0..1` opening, `2` oak door, `3` grates, `4` iron door.
Opening clears a `3x3` cave-air aperture. Wood and iron use a seven-cell stone-brick U frame plus
lower/upper door states; wood offers the upper-left frame cell twice, so it makes eight frame
writes. Iron additionally places north/south-facing stone buttons at local `(x+2,y+1,z-1)` and
`(x+2,y+1,z+1)`. Grates clear the central lower two cells and build the U from connected iron bars.

**Piece geometry:**

All ranges are inclusive local coordinates before orientation. Every ordinary room first makes its
exact outer shell and entry described below; shell cells retain the skip-air and selector behavior
above.

- Straight shells `(0,0,0)..(4,4,6)`, enters `(1,1,0)`, opens `(1,1,6)`, and gates wall torches at
  `(1,2,1)` east, `(3,2,1)` west, `(1,2,5)` east and `(3,2,5)` west. Its saved booleans optionally
  clear the west/east side apertures `x=0|4,y=1..3,z=2..4`.
- Prison hall shells `(0,0,0)..(8,4,10)`, enters `(1,1,0)`, clears exit `(1..3,1..3,10)`, selects
  masonry ribs at `x=4,y=1..3,z=1,3,7,9`, and creates two cells with connected iron-bar walls at X
  `4`, Z `4..6`, crossbars X `5..7`, Z `5`, bars at `(4,3,2|8)`, and two west-facing iron doors at
  `(4,1..2,2|8)`.
- Left and right turns shell `(0,0,0)..(4,4,4)`, enter `(1,1,0)`, and clear one full side opening
  `y=1..3,z=1..3`; north/east orientations choose the opposite X side from south/west exactly as the
  respective left/right child transform requires.
- Room crossing shells `(0,0,0)..(10,6,10)`, enters `(4,1,0)` and clears exits at `(4..6,1..3,10)`,
  `(0,1..3,4..6)` and `(10,1..3,4..6)`. Type `0` places a three-high center stone-brick pillar, four
  facing wall torches and the eight surrounding smooth slabs at Y `1`. Type `1` builds a stone-brick
  ring at X/Z `3|7`, Y `1`, a three-high center pillar and water at `(5,4,5)`. Type `2` builds the
  perimeter cobblestone balcony at Y `3`, eight cross supports, center wall torch, a plank loft at X
  `2..3|7..8`, Z `2..8` plus X `4..6` only for Z `2..3|7..8`, a west-facing ladder at `(9,1..3,3)`,
  and an unlatched crossing chest at `(3,4,8)`. Types `3` and `4` add no furnishing.
- Straight stairs shell `(0,0,0)..(4,10,7)`, enters `(1,7,0)`, opens `(1,1,7)`, and for `i=0..5`
  places south-facing cobblestone stairs at X `1..3`, `(Y,Z)=(6-i,1+i)`, with stone bricks
  immediately below at Y `5-i` only for `i=0..4`.
- Stairs down shell `(0,0,0)..(4,10,4)`, enters `(1,7,0)`, opens `(1,1,4)`, and places the fixed
  descending brick/slab path through `(2,6,1)`, `(1,5..6,1)`, `(1,4..5,3)`, `(2,4,3)`, `(3,3..4,3)`,
  `(3,2..3,1)`, `(2,2,1)`, `(1,1..2,1)`, `(1,1,2)` and slab `(1,1,3)`.
- Five crossing shells `(0,0,0)..(9,8,10)`, enters `(4,3,0)`, conditionally clears low left/right
  `(x=0|9,y=3..5,z=1..3)` and high left/right `(x=0|9,y=5..7,z=7..9)`, and always clears forward
  `(5..7,1..3,10)`. Its fixed interior is selector floor `(1..8,2,1..6)`, selector uprights
  `(4|8,1..4,5..9)`, selector high platform `(1..3,4,7..9)`, selector lower landing `(5..7,1,7..8)`,
  and smooth-slab/double-slab stair and upper bridge ranges at `(1..3,3..4,4..6)` and
  `(4..8,5,7..9)`, plus a south-facing wall torch `(6,5,6)`.
- Chest corridor shells `(0,0,0)..(4,4,6)`, enters `(1,1,0)`, opens `(1,1,6)`, builds a brick shelf
  `(3,1,2)..(3,1,4)`, slabs `(3,1,1|5)`, `(3,2,2|4)` and `(2,1,2..4)`, then conditionally
  initializes its latched chest at `(3,2,3)`.
- Library shells `(0,0,0)..(13,height-1,14)`, where height is `11` or `6`, and enters `(4,1,0)`. At
  Z `1,5,9,13`, side columns are planks and carry inward wall torches; the other side columns are
  bookshelves, as are three paired-width internal shelf bands at every odd Z from `3` through `11`.
  The tall variant adds the exact Y-`5` plank balcony, connected oak-fence rails at Y `6`,
  south-facing ladder `(10,1..7,13)`, and centered two-column fence-and-torch chandelier at X
  `5..8`, Y `7..9`, Z `6..8`. After the 520 cobweb gates it makes an unlatched library chest at
  `(3,3,5)` and, when tall, clears `(12,9,1)` then makes a second at `(12,8,1)`.
- Portal room shells `(0,0,0)..(10,7,15)` without skip-air, enters through grates `(4,1,0)`, then
  builds selector ceiling beams, side floors, stepped portal dais, wall/end iron bars, two side lava
  trenches and the central `3x3` lava pool. It places north-facing stone-brick stairs at X `4..6`
  and local `(Y,Z)=(1,4),(2,5),(3,6)`. The 12 portal frames are north-facing row `(4..6,3,8)`,
  south-facing row `(4..6,3,12)`, east-facing row `(3,3,9..11)` and west-facing row `(7,3,9..11)`.
  Its spawner target is `(5,3,6)`.
- Filler writes a fixed stone-brick shell and cave-air interior across its `5x5xsteps` extent; the
  overlapping last layer is processed like every other cell.

**Container, portal and spawner transactions:**

The chest helper rejects an out-of-box target or a target already containing a chest. Otherwise it
reorients a chest from neighboring solids, offers flags `2` ignoring failure, then queries the
resulting block entity. Only a resulting `ChestBlockEntity` consumes `nextLong` and receives table
plus seed, but the helper returns true after every admitted attempt even when the write was rejected
or the entity is absent/wrong. Chest-corridor sets its saved latch to that return. Room type `2` and
both library positions are unlatched, but an existing chest prevents repeat initialization. Normal
successful ordering is therefore chest helper draw after all earlier selector/furnishing draws for
that piece invocation.

Portal-room placement consumes 12 floats on **every** intersecting-chunk invocation, one per frame
in north, south, east, west row order. An eye is present exactly when its float is greater than
`.9`. These booleans are neither persisted nor position-seeded: the complete eye pattern is redrawn
even when some frames are outside the processing box. `allEyes` is computed from all 12 booleans
regardless of which frame writes succeed; when true, all nine portal cells at X `4..6`, Y `3`, Z
`9..11` are individually offered and clipped. A multi-chunk room can consequently retain
inconsistent eye patterns and can activate only a clipped subset of its portal interior.

When `hasPlacedSpawner` is false and `(5,3,6)` lies inside the processing box, the flag is set
**before** offering a spawner. The write uses flags `2` and is ignored. A resulting
`SpawnerBlockEntity` is configured for silverfish; the fresh spawner's empty spawn-data path
consumes no RNG, while an adversarial typed spawner with weighted potentials can consume its
weighted-selection draw. Wrong/missing entity still leaves the latch true.

**Locked loot records:**

All three chest tables use matching random-sequence IDs. Corridor pool one rolls uniformly `2..3`:
ender pearl weight `10`; diamond `3` count `1..3`; iron ingot `10` count `1..5`; gold ingot `5`
count `1..3`; redstone `5` count `4..9`; bread and apple `15` count `1..3`; iron pickaxe, sword,
chestplate, helmet, leggings and boots weight `5`; and default-weight golden apple, leather `1..5`,
copper/iron/golden/diamond horse armor, `otherside` music disc, and enchant-with-levels-`30`
`#minecraft:on_random_loot` book. Its second pool chooses empty weight `9` or an eye armor-trim
template weight `1`.

Crossing rolls uniformly `1..4` over iron ingot weight `10` count `1..5`, gold ingot `5` count
`1..3`, redstone `5` count `4..9`, coal `10` count `3..8`, bread and apple `15` count `1..3`, iron
pickaxe weight `1`, and a level-`30` random-loot enchanted book weight `1`. Library rolls uniformly
`2..10` over book and paper weight `20` with counts `1..3` and `2..7`, map and compass weight `1`,
and the same enchanted book weight `10`; its second pool always supplies one eye trim template. Loot
evaluation stays deferred to `ITM-LOOT-001`; this leaf owns exact record linkage and seed
installation.

**Branches and aborts:**

Stub biome accept/reject; every retry count and seed; pending removal; finite/unlimited graph
survival; imposed success/failure; every weighted endpoint, repeated/ineligible/factory-null cascade
and five-attempt filler; depth `4/5/50/51`, range `112/113`, Y `1/2/10/11` and collision order; all
four orientations, door types, constructor flags and room types; tall/fallback library; portal
found/missing; relocation draw/no draw; loaded/transient source state; every shell
existing-air/nonair, interior/edge selector threshold, clip and write/fluid outcome; cobweb/torch
thresholds; chest/spawner latch, target state, write and entity type; every portal-eye pattern, clip
and activation subset.

**Constants and randomness:**

Each attempt begins with the large-feature reseed and direction draw, then
constructor/child-selection draws in pending-removal order. Weighted selection uses one integer per
attempt, not per cascaded factory. Relocation is the final graph draw when admitted. Placement
chunks share caller RNG in piece creation order: selector floats are subject to each call's skip-air
rule, while portal shells, internal selector ranges, 520 library cobweb gates, four straight torch
gates and 12 portal-eye gates consume even outside the clip as specified. Successful typed chest
initialization consumes one long. A hostile weighted spawner may consume its own selection draw.
Loot and later entity RNG have separate owners.

**Side effects:**

A retry-selected graph of persisted oriented pieces; relocated boxes; chunk-clipped cave air,
stone-brick variants, infested blocks, doors/buttons/bars, stairs/slabs, prison cells,
fountains/lofts, bookshelves/cobwebs/fences/torches, lava, portal frames and portal blocks; three
stronghold loot-table families; a latched silverfish spawner; fluid ticks and shape postprocessing;
transient portal-centered locate position.

**Gates:**

Caller-owned concentric-ring selection, start/reference/placement scheduling and chunk RNG; exact
start biome; graph depth/range/collision/quota; live sea/build heights; current
block/clip/fluid/write outcomes; persisted flags and resulting block-entity classes.

**Boundary cases and quirks:**

A graph can retry indefinitely, and a finite quota—not unlimited choices—keeps weighted expansion
alive. A null factory can fall through to later weighted entries without a second draw. The imposed
first five crossing is quota-free. A filler deliberately overlaps its blocker. Relocation can create
pieces below their original factory Y limits. Shared static graph fields expose caller scheduling.
Skip-air changes both masonry and RNG by the live current block, whereas portal-room shells redraw
globally per intersecting chunk. The unlatched portal eyes are the principal timing quirk: eye
pattern and active portal subset may differ by chunk visit. Chest/spawner latches can commit after
failed writes, and reload loses the portal-aware locate pointer.

**Evidence:**

`Confirmed`; `OFF-SERVER-001`; `OFF-DATA-001`; `OFF-REPORT-001`. Anchors:
`net.minecraft.world.level.levelgen.structure.structures.StrongholdStructure#findGenerationPoint`,
`#generatePieces`;
`net.minecraft.world.level.levelgen.structure.structures.StrongholdPieces#resetPieces`,
`#updatePieceWeight`, `#findAndCreatePieceFactory`, `#generatePieceFromSmallDoor`,
`#generateAndAddPiece`; every nested piece `createPiece`, `addChildren`, `postProcess` and save
constructor;
`net.minecraft.world.level.levelgen.structure.pieces.StructurePiecesBuilder#moveBelowSeaLevel`;
`net.minecraft.world.level.levelgen.structure.StructurePiece#generateBox`, `#placeBlock`,
`#createChest`; `net.minecraft.world.level.levelgen.structure.StructureStart#placeInChunk`;
`data/minecraft/worldgen/structure/stronghold.json`,
`data/minecraft/worldgen/structure_set/strongholds.json`, both stronghold biome tags and all three
stronghold chest tables.

**Test vectors:**

Replay fixed seeds across retries, all weighted endpoints and null/collision cascades; assert graph
identities, creation/pending order, quotas, depths, boxes, filler overlap, portal guarantee,
relocation and saved/reloaded locator. For every orientation and piece state, use full, empty and
adversarial partial boxes to trace selector/cobweb/torch/eye draws, exact transformed block states,
failed writes, fluids and postprocessing. Exhaust door/room/five-crossing states, all chest and
spawner result types/latches/seeds, every portal-eye pattern and cross-chunk visitation order;
decode both biome tags, the record/set, all 13 piece IDs and three loot records exactly. Use
`EXP-WGEN-001` only for separately owned concentric-ring/distribution calibration.
