# World mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `WGEN-STRUCTURE-DESERT-PYRAMID-001` — Desert pyramids span four chunks and make one global archaeology selection

**Parent:** `WGEN-003`

**FidelityClass:** `EquivalentPlayerVisibleBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked 26.2 server source fixes the four-corner start gate, one-piece geometry,
terrain alignment, ordered block/support/chest writes, cellar collapse and global archaeology
postpass; the structure, set and two loot inputs are audited data-only. Shared random-spread
admission and generic start/reference lifecycle remain owned by `WGEN-PIPELINE-001`.

**Applies when:**

The `minecraft:desert_pyramid` type has passed caller-owned structure-set placement, its retained
piece intersects a placement chunk, or the completed structure start runs its family-specific
archaeology postpass.

**Authoritative state:**

The structure record selects only `minecraft:desert`, uses `surface_structures`, default `none`
terrain adaptation and no spawn overrides. Its set contains only desert pyramid at weight `1`, with
random-spread spacing `32`, separation `8`, salt `14357617`, and default frequency fields. The piece
persists its bounding box, horizontal orientation, width/depth `21`, height `15`, cached `HPos`, and
four direction-indexed chest booleans. Missing chest booleans decode false; missing `HPos` decodes
`0`. Its 83-entry archaeology candidate list and selected collapsed-roof position are transient,
initially empty and zero, and are not serialized.

**Transition and ordering:**

Before consuming structure RNG, the single-piece start gate samples `WORLD_SURFACE_WG`
first-occupied heights at `(chunkMinX,chunkMinZ)`, `(chunkMinX,chunkMinZ+21)`,
`(chunkMinX+21,chunkMinZ)`, and `(chunkMinX+21,chunkMinZ+21)`. The latter three probes can lie one
cell beyond the eventual inclusive `0..20` footprint. If the minimum is below the generator sea
level, the structure has no start. Otherwise the generic top-of-chunk-center stub samples local
`(8,8)` and applies the exact 3-D biome-holder gate there. Deferred generation anchors one piece at
`(chunkMinX,64,chunkMinZ)` and consumes `nextInt(4)` over north, east, south and west. Because the
footprint is square, every orientation occupies source/east/south/southeast chunks over world X/Z
`chunkMin..chunkMin+20`; the nominal pre-alignment box is Y `64..78`.

Every `postProcess` invocation first evaluates offset `-nextInt(3)` from its caller-supplied
placement RNG, even when cached height makes that value unused. If `HPos<0`, the helper scans all
441 current bounding-box columns, Z outer then X inner, without processing-box filtering, samples
`MOTION_BLOCKING_NO_LEAVES`, takes their minimum, stores it, and moves the box so minimum Y is
`HPos+offset`. A nonnegative result is then cached for every later chunk invocation. A negative
result moves the piece but remains the negative sentinel, so a later invocation resamples and can
move it again; missing serialized `HPos=0` is instead treated as cached. Ordinary four-chunk
placement therefore burns four offset draws while only the first invocation with a nonnegative
terrain minimum chooses the vertical offset.

**Main fixed layout:**

All coordinates below are piece-local after terrain alignment. Inclusive boxes iterate Y outermost,
X middle and Z innermost. The base first offers solid sandstone over `(0,-4,0)..(20,0,20)` (`2,205`
offers). For each `p=1..9`, it then offers a solid sandstone square `(p,p,p)..(20-p,p,20-p)` and
overwrites the inner square `(p+1,p,p+1)..(19-p,p,19-p)` with air (`2,298` offers). The remaining
main layout is the following ordered reconstruction; states named north/south/east/west are stair
facings before piece mirror/rotation:

#### Front towers

**Ordered local offers:**

Sandstone-shell/air-interior boxes `(0,0,0)..(4,9,4)` and `(16,0,0)..(20,9,4)`; solid sandstone caps
`(1,10,1)..(3,10,3)` and `(17,10,1)..(19,10,3)`; stairs north/south/east/west at
`(2,10,0)/(2,10,4)/(0,10,2)/(4,10,2)` and `(18,10,0)/(18,10,4)/(16,10,2)/(20,10,2)`

#### Center front and connectors

**Ordered local offers:**

Sandstone-shell/air-interior `(8,0,0)..(12,4,4)`, then air `(9,1,0)..(11,3,4)`; cut-sandstone arch
`(9,1..3,1)`, `(10,3,1)`, `(11,3..1,1)`; sandstone-shell/air-interior connectors `(4,1,1)..(8,3,3)`
and `(12,1,1)..(16,3,3)`, then air corridors `(4,1,2)..(8,2,2)` and `(12,1,2)..(16,2,2)`

#### Court and side masses

**Ordered local offers:**

Solid sandstone `(5,4,5)..(15,4,15)`, then air `(9,4,9)..(11,4,11)`; cut-sandstone pillars at X/Z
`(8,8),(12,8),(8,12),(12,12)`, Y `1..3`; solid sandstone `(1,1,5)..(4,4,11)` and
`(16,1,5)..(19,4,11)`; sandstone `(6,7,9)..(6,7,11)` and `(14,7,9)..(14,7,11)`; cut sandstone
`(5,5,9)..(5,7,11)` and `(15,5,9)..(15,7,11)`; air `(5,5,10),(5,6,10),(6,6,10)` and mirrored
`(15,5,10),(15,6,10),(14,6,10)`

#### Side steps

**Ordered local offers:**

Air `(2,4,4)..(2,6,4)` and `(18,4,4)..(18,6,4)`; north stairs `(2,4,5),(2,3,4),(18,4,5),(18,3,4)`;
sandstone `(1,1,3)..(2,2,3)` and `(18,1,3)..(19,2,3)`, points `(1,1,2),(19,1,2)`, slabs
`(1,2,2),(19,2,2)`, west stair `(2,1,2)` and east stair `(18,1,2)`

#### Side halls

**Ordered local offers:**

Sandstone roof lines `(4,3,5)..(4,3,17)` and `(16,3,5)..(16,3,17)`; air `(3,1,5)..(4,2,16)` and
`(15,1,5)..(16,2,16)`; for Z `5,7,9,11,13,15,17`, cut sandstone at X `4,16`, Y `1`, and chiseled
sandstone at those X/Z, Y `2`

#### Floor glyph

**Ordered local offers:**

Orange terracotta at
`(10,7),(10,8),(9,9),(11,9),(7,10),(8,10),(12,10),(13,10),(9,11),(11,11),(10,12),(10,13)` on Y `0`,
and blue terracotta at `(10,0,10)`

#### Facades

**Ordered local offers:**

On side planes X `0,20`, for Y `2..8` and Z `1..3`, and on front plane Z `0` centered at X `2,18`,
the seven rows are cut/orange/cut; cut/orange/cut; orange/chiseled/orange; cut/orange/cut;
orange/chiseled/orange; all orange; all cut. Finally cut sandstone fills `(8,4,0)..(12,6,0)`, air
overrides `(8,6,0),(12,6,0)`, orange overrides `(9,5,0),(11,5,0)`, and chiseled sandstone overrides
`(10,5,0)`

The foundation, rings, listed upper structure and following trap together contribute `6,478` fixed
offers including overlaps. Before the trap, the piece starts a downward sandstone support at every
local `(x,-5,z)` for X/Z `0..20`, in Z-major then X order. Only the start is processing-box tested;
an admitted column continues below the box through air, liquids, glow lichen, seagrass and tall
seagrass while Y is strictly above `level.minY+1`, stopping at the first other state. Thus ordinary
placement distributes 441 support starts among four chunk clips, but an accepted support is not
cell-clipped.

**Trap and chests:**

The central trap offers solid cut sandstone `(8,-14,8)..(12,-11,12)`, a chiseled layer at Y `-10`,
cut layer at `-9`, and sandstone from Y `-8..-1`, all over X/Z `8..12`; it then hollows
`(9,-11,9)..(11,-1,11)`, places a stone pressure plate at `(10,-11,10)`, and offers nine TNT at X/Z
`9..11`, Y `-13`. Four two-cell side openings lie at west X `8`, east X `12`, north Z `8` and south
Z `12`, Y `-11..-10`; each has chiseled sandstone one cell outward at Y `-10` and cut sandstone
there at Y `-11`. The later cellar's 12 fixed boxes and 33 fixed points contribute another `132`
offers, making `6,610` before the random cellar roof.

The piece then assigns its four persisted flags from chest attempts in north, east, south, west
order at `(10,-11,8)`, `(12,-11,10)`, `(10,-11,12)`, `(8,-11,10)`. An attempt outside the processing
box or against an already-present chest returns false and can retry. Otherwise it offers a
reoriented chest with flags `2`, then, only if the resulting entity is a chest entity, consumes
`nextLong` and initializes it with `minecraft:chests/desert_pyramid`; the helper returns true and
latches even if the write was rejected or no chest block entity exists. The chest table uses random
sequence `minecraft:chests/desert_pyramid`: pool one rolls uniformly `2..4` over diamond weight `5`
count `1..3`, iron ingot `15` count `1..5`, gold ingot `15` count `2..7`, emerald `15` count `1..3`,
bone `25` count `4..6`, spider eye `25` count `1..3`, rotten flesh `25` count `3..7`, leather `20`
count `1..5`, copper/iron/gold/diamond horse armor weights `15/15/10/5`, random-enchanted
`#minecraft:on_random_loot` book weight `20`, golden apple `20`, enchanted golden apple `2`, and
empty `15`; pool two makes exactly four equal-weight rolls of bone, gunpowder, rotten flesh, string
or sand, each count `1..8`; pool three selects empty weight `6` or two dune armor-trim templates
weight `1`.

**Cellar and archaeology candidates:**

A cellar centered at `(16,-4,13)` begins with counterclockwise-90-degree sandstone stairs at
`(13,-1,17),(14,-2,17),(15,-3,17)`. One level-RNG boolean controls the collapse: X `12..16`, Y `0`,
Z `17` and `(14,-1,17),(15,-2,17),(16,-3,17)` are sand; exactly one of `(15,-1,17),(16,-1,17)` is
sand and the other sandstone; `(16,-2,17)` is sandstone. The room perimeter X `13..19`, Z `10..16`
is cut sandstone at Y `-3,-1` and chiseled at `-2`, offered as west/east lines, then north/south
lines.

The piece records, without clipping or writing, every X `14..18`, Y `-3..-1`, Z `11..15` position in
Y/X/Z loop order. Its collapsed roof then visits X `14..18` outer and Z `11..15` inner at Y `0`,
consumes a level-RNG float for every cell even outside the clip, and offers sandstone for `<0.33` or
sand otherwise. Hence every piece invocation consumes one collapse boolean and 25 roof floats; an
ordinary four-chunk placement consumes four booleans and 100 floats, with only clip-surviving writes
visible. A separate world-seed positional RNG rooted at the transformed local `(14,0,11)` makes
inclusive X `14..18` and Z `11..15` draws, selecting the same collapsed-roof world position on every
replay.

The floor at Y `-4` is blue terracotta at `(16,13)`, orange on the four diagonals at distance one,
four axes at distance two and four axes at distance three. The eight distance-three cardinal
positions at Y `-3,-2` are appended as candidates. A normal invocation therefore records 83 unique
candidate positions—75 room cells plus eight spokes—and repeated chunk invocations merely append
duplicates. Outer doorway decor adds cut/chiseled pairs at east X `20`, west X `12`, north Z `9`;
the south side is the stair/collapse entrance.

After all intersecting pieces place, the structure postpass first places suspicious sand at each
piece's deterministic collapsed-roof selection, clip permitting. It inserts all candidate positions
into a sorted array set using world Y, then Z, then X comparison, removing repeated-invocation and
overlap duplicates; a normal one-piece start yields 83. It copies the sorted set, creates a fresh
world-seed positional RNG at the completed nominal piece-box center, and performs a full
Fisher–Yates shuffle, consuming `nextInt(i)` for descending sizes `83..2`. It then draws
upper-exclusive `nextInt(5,8)`, yielding a global target of `5`, `6`, or `7`, capped by list size.
Traversal decrements this global count for the first shuffled candidates whether or not each lies in
the current processing box: selected cells outside the present chunk are not refilled. Once the
count is exhausted, every remaining in-clip candidate is overwritten with ordinary sand. Because
each chunk reconstructs the same sorted population, seed, shuffle and target, all four invocations
agree on one global selection. With successful writes, the structure therefore has one roof plus
`5..7` candidate suspicious-sand cells, not `5..7` per chunk.

Suspicious-sand placement is clip-bounded, offers flags `2`, ignores the write result, then performs
a typed brushable-block-entity lookup. A resulting entity receives
`minecraft:archaeology/desert_pyramid` with loot seed `blockPos.asLong`; this can initialize a
pre-existing typed entity after a rejected write. That archaeology table uses random sequence
`minecraft:archaeology/desert_pyramid` and one uniform roll among archer, miner, prize or skull
pottery sherd, diamond, TNT, gunpowder and emerald.

All direct piece writes transform mirror/rotation, processing-box test, offer flags `2` and ignore
the write result. After every admitted direct offer, the resulting fluid is read and a nonempty
fluid gets a delay-`0` tick even after rejection; known shape-check blocks are marked for
postprocessing. Sandstone stairs do not belong to that shape-check set. Downward supports and the
structure postpass use their separately described paths.

**Branches and aborts:**

Four-corner minimum below/equal/above sea level; center-stub biome accept/reject; four directions;
four placement chunks and every invocation order; cached/uncached, missing and negative `HPos`;
every fixed/random/support/chest/candidate cell outside/inside clip and rejected/accepted write;
postwrite empty/nonempty fluid; support replaceable/terminal/minimum-Y states; each persisted chest
flag, existing chest and resulting block-entity type; collapse boolean and each roof threshold;
duplicate candidate insertion; roof and candidate block-entity lookup; global selected/unselected
and in/out-of-current-clip candidates.

**Constants and randomness:**

Start generation consumes direction `nextInt(4)`. Each piece invocation consumes caller placement
`nextInt(3)` before the height cache gate and one `nextLong` for each admitted unlatchable chest
whose postwrite entity has the chest type; level RNG supplies one collapse boolean and 25 roof
floats. Two fresh world-seed positional streams independently select the roof cell and shuffle/count
the global archaeology candidates. The postpass caller RNG is unused. Loot evaluation is deferred to
its random-sequence/seed owners.

**Side effects:**

One retained piece spanning four chunks; `6,610` ordered fixed offers, 25 random roof offers and 441
variable-depth supports; up to four latched loot chests and `6..8` suspicious-sand cells under
normal successful placement; ordinary-sand normalization of every other in-clip candidate; fluid
ticks, persisted height/chest flags and transient accumulated candidate state.

**Gates:**

Caller-owned placement/start/reference lifecycle; four live start heights and generator sea level;
center-stub biome; processing box; live terrain heightmap; replaceable support material;
existing/resulting chest and brushable block-entity types; position-seeded shuffle selection.

**Boundary cases and quirks:**

Start probes extend to `+21`, while the piece ends at `+20`. Four chunk invocations share cached
geometry but not level/caller random draws. Negative terrain minima remain sentinel-negative.
Candidate collection ignores clipping, making its deduplicated population globally reproducible; the
selection count is consumed by candidates outside each current chunk instead of refilling locally.
Failed writes can still latch a chest, schedule a fluid or initialize an already-present brushable
entity. The nominal piece box excludes the trap's negative-Y extension but its center still seeds
candidate selection.

**Evidence:**

`Confirmed`; `OFF-SERVER-001`; `OFF-DATA-001`; `OFF-REPORT-001`. Anchors:
`net.minecraft.world.level.levelgen.structure.SinglePieceStructure#findGenerationPoint`,
`net.minecraft.world.level.levelgen.structure.structures.DesertPyramidStructure#afterPlace`,
`net.minecraft.world.level.levelgen.structure.structures.DesertPyramidStructure#placeSuspiciousSand`,
`net.minecraft.world.level.levelgen.structure.structures.DesertPyramidPiece#postProcess`,
`net.minecraft.world.level.levelgen.structure.structures.DesertPyramidPiece#addCellar`,
`net.minecraft.world.level.levelgen.structure.ScatteredFeaturePiece#updateHeightPositionToLowestGroundHeight`,
`net.minecraft.world.level.levelgen.structure.StructurePiece#placeBlock`,
`net.minecraft.world.level.levelgen.structure.StructurePiece#fillColumnDown`,
`net.minecraft.world.level.levelgen.structure.StructurePiece#createChest`,
`net.minecraft.util.Util#shuffle`, `net.minecraft.core.Vec3i#compareTo`,
`net.minecraft.world.level.block.entity.BrushableBlockEntity#setLootTable`,
`data/minecraft/worldgen/structure/desert_pyramid.json`,
`data/minecraft/worldgen/structure_set/desert_pyramids.json`,
`data/minecraft/tags/worldgen/biome/has_structure/desert_pyramid.json`,
`data/minecraft/loot_table/chests/desert_pyramid.json`, and
`data/minecraft/loot_table/archaeology/desert_pyramid.json`.

**Test vectors:**

Cross all four start probes, sea-level edge, center biome, directions, placement-chunk orders,
offset draws and cached/negative/missing height. Assert every ordered fixed offer, support start,
transform, overlap, clip/write/fluid effect, collapse boolean/float, chest latch/seed and both
loot-table decodes. Replay duplicate candidate accumulation, exact 83-position set/sort, roof draws,
full shuffle/count and per-chunk no-refill behavior across positive/negative coordinates and failed
writes/entities; use `EXP-WGEN-001` only for separately owned distribution/locate equivalence.
