# World mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `WGEN-STRUCTURE-MINESHAFT-001` — Mineshafts grow a depth-first wood graph and choose support, carts, and spiders from live chunks

**Parent:** `WGEN-003`

**FidelityClass:** `EquivalentPlayerVisibleBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked 26.2 source fixes normal/mesa start construction, recursive graph
selection, all four piece families, relocation, deep-dark/liquid cancellation, wood replacement,
supports and chains, rails/cobwebs/torches, chest-minecart and cave-spider-spawner transactions,
persistence and the sole loot record. Both records, the set, 54 exact start biomes and blocking tag
are audited data-only. Shared legacy-type-3 random-spread admission and start/reference scheduling
remain owned by `WGEN-PIPELINE-001`; later entity/block-entity lifecycle remains owned by
`ENT-LIFECYCLE-001`.

**Applies when:**

The `mineshafts` set selects its normal or mesa record, the retained graph is constructed, or one of
its room/corridor/crossing/stairs pieces intersects a placement chunk.

**Authoritative state:**

Both records use `underground_structures`, default `none` terrain adaptation and no spawn overrides.
Normal type uses oak logs/planks/fences and its exact 51 biomes are all 55 locked Overworld biomes
except deep dark and the three badlands biomes. Mesa uses dark-oak logs/planks/fences and exactly
badlands, eroded badlands and wooded badlands. Their set orders normal then mesa at weight `1`, with
random-spread spacing `1`, separation `0`, salt `0`, frequency `.004`, legacy-type-3 reduction and
default locate offset. `#minecraft:mineshaft_blocking` contains only deep dark.

**Transition and ordering:**

After generic set admission, `findGenerationPoint` consumes and discards one `nextDouble`. It fixes
the provisional stub at source chunk `(middleX,50,minZ)` and makes a room at
`(chunkBlockX(2),50,chunkBlockZ(2))`. Room maximum X is start X plus `7+nextInt(6)`, maximum Y is
`54+nextInt(6)`, and maximum Z is start Z plus `7+nextInt(6)`, yielding inclusive spans `8..13`,
`5..10`, `8..13` in that draw order. The room is added, then graph creation recurses immediately
depth-first: every successful child is added and fully expands before its caller resumes.

The room scans north, south, west and east walls in order. North/south use the room X span and
west/east use its Z span. Each wall begins `pos=0`; while `pos<span`, it first adds `nextInt(span)`.
If `pos+3>span`, that wall loop ends immediately; otherwise it consumes a Y offset
`1+nextInt(max(1,ySpan-4))`, attempts a child just outside the wall, and then adds `4`. A successful
child records a two-block-thick entrance box spanning the child's vertical and transverse box at
that wall. Failure records nothing. These entrance boxes move with and persist with the room.

Every child attempt first rejects an input depth greater than `8` or an anchor whose strict X/Z
distance from the root room minimum exceeds `80`; equality is admitted. It then consumes
`nextInt(100)` once. Values `0..69` choose corridor, `70..79` stairs and `80..99` crossing. A chosen
factory failure returns null—there is no fallback to another family. The successful piece receives
generation depth `input+1`, is added, and expands immediately.

Corridor creation consumes `nextInt(3)+2` for `2..4` five-block sections, tests that full `3x3x(5n)`
oriented box, and decrements `n` without another draw until a collision-free positive length or
failure. Its constructor consumes `nextInt(3)==0` for rails; only a nonrail corridor consumes
`nextInt(23)==0` for spider mode. It persists both, the section count and spider latch. Crossing
consumes `nextInt(4)`: zero makes a `5x7x5` two-floor box, otherwise `5x3x5`; collision rejects it.
It has no orientation transform, but separately persists its entry direction (missing/invalid
decodes south) and two-floor flag. Stairs are an oriented width-`3`, height-`8`, length-`9`
descending box and consume no factory RNG.

Each corridor first consumes `nextInt(4)` for its far end: `0..1` continues, `2` turns left and `3`
right, then consumes `nextInt(3)-1` for that child's Y relative to the corridor minimum. While its
own depth is below `8`, it next visits every interior five-block bay from minimum plus `3`; each
consumes `nextInt(5)`, with `0` attempting the left side and `1` the right. These side calls pass
current depth plus one into the common helper, so successful side children advance generation depth
by **two**, whereas the far-end child advances by one. Crossing always attempts the three non-entry
lower directions in its direction-specific order. A two-floor crossing then consumes four booleans
in north, west, east, south order and attempts each admitted upper exit at minimum Y plus `4`,
including the entry side. Stairs attempt one forward child at their low end. Pieces at depth `8` can
create depth-`9` far/crossing/stairs children; calls from depth `9` reject before the family draw,
although corridor Y expressions and two-floor booleans are still evaluated by their callers as
applicable.

After the graph is complete, normal relocation applies the same exact below-sea formula as
stronghold: target `seaLevel-10`, `j=unionYSpan+minBuildY+1`, conditional `nextInt(target-j)`, then
offset `j-unionMaxY`. Mesa instead samples generator `WORLD_SURFACE_WG` at the pre-move union
center. A surface at or below sea level targets sea level without a draw; otherwise an inclusive
integer draw selects `[seaLevel,surface]`. Every piece moves by `target-centerY`, placing the union
center exactly at that target. The returned offset also moves the provisional stub before the
generic 3-D biome gate, so its final test position remains `(chunkMiddleX,50+offset,chunkMinZ)`, not
the union center.

All pieces persist the generic ID, box, orientation and generation depth plus `MST` type ordinal;
missing or out-of-range type resolves normal. Exact registry IDs are `msroom`, `mscorridor`,
`mscrossing` and `msstairs`. Corridor additionally persists `hr`, `sc`, `hps`, and `Num`; crossing
persists `tf` and `D`; room persists its entrance-box list; stairs adds no fields.

**Per-chunk cancellation and writes:**

Before any piece-specific work, the piece expands its box by one, clips all six bounds to the
supplied processing box, and samples the biome at the integer midpoint of that clipped volume. Deep
dark aborts the complete invocation. Otherwise it scans block-state `liquid()` on the clipped
expanded boundary: for each X/Z it tests bottom then top, for each X/Y north then south, then for
each Z/Y west then east. The first liquid aborts. This gate is repeated independently for every
piece/chunk invocation against live state, so one chunk may suppress a piece while another places
it.

Oriented local writes use the shared transform, clip, flags-`2`, fluid-tick and shape-postprocessing
transaction from `StructurePiece`. Mineshaft `canBeReplaced` additionally rejects any current plank,
log or fence block belonging to its own normal/mesa palette, and iron chain. Air carving and
generated furniture therefore preserve those blocks. Direct floor, pillar/chain and spawner writes
described below bypass that predicate and ignore their write result. `isInterior(x,y,z)` requires
transformed `(x,y+1,z)` inside the processing box and strictly below live `OCEAN_FLOOR_WG`.

**Room, crossing and stairs geometry:**

Room clears its full horizontal box from minimum Y plus `1` through `min(minY+3,maxY)`, then clears
each saved entrance's top three layers. It finally carves an upper-half ellipsoid over the box from
minimum Y plus `4` through maximum Y, using the generic inclusive spans, half-cell X/Z centers,
normalized Y from the lower bound, and membership squared distance at most `1.05`; every admitted
cell is cave air. No room floor or support is added.

Crossing clears two perpendicular width-three passages over the full `5x5` horizontal box. A
one-floor box clears Y `min..max`; a two-floor box clears lower Y `min..min+2`, upper Y
`max-2..max`, and the center `3x1x3` separator at Y `min+3`. At each of the four inner corners, a
plank pillar from minimum through maximum Y is generated only when the live cell immediately above
its top is nonair; clipping makes an off-box read air and suppresses that pillar. Finally every cell
one below the `5x5` box calls the floor helper: only an interior cell whose existing state lacks a
sturdy top is directly replaced with palette planks.

Stairs only carve cave air. They clear upper mouth `(0..2,5..7,0..1)`, lower mouth
`(0..2,0..2,7..8)`, then for `i=0..4` clear the full width at Z `2+i` from Y `5-i-(i<4?1:0)` through
`7-i`. Orientation maps the local descent.

**Corridor carving and bays:**

Let `length=5*numSections-1`. After cancellation, corridor first clears X `0..2`, Y `0..1`, Z
`0..length`. It then consumes one float for every ceiling cell X `0..2`, Y `2`, Z `0..length`,
clearing exactly values at most `.8`. A spider corridor next consumes one float for every X `0..2`,
Y `0..1`, Z `0..length`; values at most `.6` proceed to the live interior test and place cobweb when
it passes. Both volumes traverse Y outermost, X middle and Z innermost. Nonspider corridors consume
none of those `.6` draws.

For each section in increasing order, bay Z is `2+5*section`. Support admission first requires all
three cells at local Y `3` over that cross-section to be nonair; any clipped/off-chunk read is air,
suppressing the support and its RNG. Admission places connected palette fences at X `0|2`, Y `0..1`,
then consumes `nextInt(4)`. Zero places only the two top endpoint planks; another value places the
complete three-plank beam and consumes two strict `<.05` torch floats for center cells one Z before
(south-facing) and after (north-facing) the beam.

Eight decorative cobweb opportunities follow in order: X `0,2` at Z `bay-1`, then X `0,2` at
`bay+1`, each probability `.1`; then the same pairs at `bay-2` and `bay+2`, each `.05`. An
opportunity consumes its float only when interior. A passing float then scans all six adjacent cells
in enum order and requires at least two neighbors both inside the processing box and sturdy on the
contacting face; failure consumes no further RNG and writes nothing.

Two independent `nextInt(100)==0` chest-cart gates follow, targeting `(2,0,bay-1)` then
`(0,0,bay+1)`. Only a target inside the processing box, currently air, with a nonair block below
proceeds. It consumes one boolean for north-south versus east-west rail, offers that rail through
ordinary replacement, then creates a chest minecart for `CHUNK_GENERATION`. Creation failure still
returns success to the unused caller. Success places its initial position at target center, consumes
`nextLong` for `chests/abandoned_mineshaft`, and offers the entity; admission failure is ignored and
there is no persisted latch. A retained rail prevents a later repeat because the target is no longer
air.

In a spider corridor whose latch remains false, the bay then consumes `nextInt(3)` and targets
center X `1`, Y `0`, Z `bay-1+[0..2]`. Outside-box or noninterior candidates leave the latch false,
allowing later bays and later chunk invocations to try. The first admitted candidate sets the latch
**before** directly offering a spawner, then queries the resulting entity. Only a typed spawner is
configured for cave spider; a normal fresh spawner consumes no selection RNG, while adversarial
weighted potentials may. Thus chunk visitation can choose which bay permanently owns the spawner.

After every bay, all X `0..2`, Z `0..length` floor cells at local Y `-1` use the direct floor helper
in X-outer/Z-inner order: interior admission and a nonsturdy existing top face replace it with
palette planks. The first support bay Z `2`, and also `length-2` when there is more than one
section, then inspect the two endpoint floor blocks. Each endpoint equal by block identity to
palette planks starts the lower-pillar/upper-chain search below.

That search checks increasing distance `d`, lower before upper at every step. Below is open only for
generic structure-replaceable state excluding lava. The first nonopen state sturdy upward ends the
search and directly fills palette logs from one above it through one below the floor origin; at
`d=1` this is an empty range but still wins. An open below path continues under the source condition
`d<=20` and above `minY+1`, so distance `21` is still evaluated for a support before stopping. Above
treats every generic replaceable—including lava—as open. The first nonopen, nonfalling block that
supports center downward writes one palette fence at origin plus `1` and iron chains through one
below that anchor. Its continuation condition `d<=50` and below `maxY` likewise evaluates an anchor
at distance `51`. A lower anchor wins a same-distance tie. These scans/fills are not vertically
clipped after the floor endpoint was admitted.

Rail corridors finish by visiting center X `1`, local Y `0`, every Z in order. A clipped floor read,
air floor or non-solid-render floor skips without a draw. Otherwise the rail chance is `.7` when the
target is interior and `.9` when not; one strict-less-than float gates a local north-south rail,
which rotates with the corridor.

**Locked loot record:**

`chests/abandoned_mineshaft` uses the matching random sequence and three pools. Pool one makes one
roll: golden apple weight `20`, enchanted golden apple `1`, name tag `30`,
random-`#minecraft:on_random_loot` enchanted book `10`, iron pickaxe `5`, or empty `5`. Pool two
rolls uniformly `2..4` over iron ingot weight `10` count `1..5`, gold ingot `5` count `1..3`,
redstone and lapis `5` count `4..9`, diamond `3` count `1..2`, coal `10` count `3..8`, bread `15`
count `1..3`, glow berries `15` count `3..6`, and melon/pumpkin/beetroot seeds `10` count `2..4`.
Pool three makes exactly three rolls over rail weight `20` count `4..8`, powered/detector/activator
rail `5` count `1..4`, torch `15` count `1..16`, and `music_disc_bounce` weight `10` only when the
loot-context location biome is sulfur caves. Loot evaluation is deferred to `ITM-LOOT-001`; this
leaf owns cart seed/table installation and the exact record.

**Branches and aborts:**

Normal/mesa record and exact final stub biome; discarded double; every room dimension/wall step/Y
and child failure; depth `8/9`, range `80/81`, family endpoints and collision/no-fallback; corridor
length fallback, rail/spider split, far/side selection and asymmetric depth; crossing
floor/directions/upper booleans; normal/mesa relocation endpoints; saved/missing fields. Per chunk:
deep-dark/nonblocking midpoint, every boundary-liquid position/order, own-material replacement, room
sphere, support overhead, interior height, sturdy floor, all
carving/cobweb/support/torch/chest/spawner/rail thresholds, target clip/state/entity/write/admission
outcomes, lower/upper anchor limits and tie.

**Constants and randomness:**

Graph RNG order is the discarded double, room X/Y/Z dimensions, then recursive wall and piece draws
exactly as above, followed by normal or mesa relocation. Placement pieces share caller chunk RNG in
creation order; an invalid-location return consumes none. Corridor order is ceiling floats, optional
spider-volume floats, then per bay support/torches, eight cobweb gates, two chest integers with
conditional boolean/long, conditional spider offset, followed after all bays by rail floats. Direct
level/height/biome/state queries and support fills consume no RNG. Loot owns its later sequence.

**Side effects:**

A recursively constructed and relocated four-family graph; persisted room entrances/corridor
modes/spider latch; chunk-dependent cave carving; oak or dark-oak floors, supports, beams, fences
and hanging chains; cobwebs, torches and rails; seeded chest minecarts; one latched cave-spider
spawner per admitted spider corridor; fluid ticks and shape postprocessing for ordinary writes.

**Gates:**

Caller-owned random-spread/start/reference/placement scheduling; record biome and live final stub;
depth/range/collision; relocation height; processing box, midpoint biome and boundary liquids; live
ocean-floor height, block identity/sturdiness/support and resulting entity/block-entity types.

**Boundary cases and quirks:**

The first double is consumed but unused. The start stub uses middle X but minimum Z and moves by a
graph-derived offset. A selected piece family never falls back after collision. Corridor side
branches skip a generation depth. Depth-`9` pieces exist but cannot select descendants.
Invalid-location and overhead-support checks are chunk clipped, while successful pillar/chain fills
extend vertically without that clip. Own palette blocks and chains resist ordinary carving, but
direct floors/supports/spawners bypass the replacement rule. The lower/upper scans evaluate
distances `21/51`; a sturdy block immediately below wins while writing no log. Chest gates are
unlatched, whereas the spider latch commits before write and its bay is visitation-dependent.

**Evidence:**

`Confirmed`; `OFF-SERVER-001`; `OFF-DATA-001`; `OFF-REPORT-001`. Anchors:
`net.minecraft.world.level.levelgen.structure.structures.MineshaftStructure#findGenerationPoint`,
`#generatePiecesAndAdjust`, `MineshaftStructure$Type`;
`net.minecraft.world.level.levelgen.structure.structures.MineshaftPieces#createRandomShaftPiece`,
`#generateAndAddPiece`; all four nested piece constructors, factories, child and postprocess
methods; `MineShaftPiece#canBeReplaced`, `#isSupportingBox`, `#isInInvalidLocation`,
`#setPlanksBlock`; corridor chest/support/cobweb/pillar/chain helpers; generic structure placement
helpers; both structure records, `structure_set/mineshafts.json`, both start-biome tags,
`mineshaft_blocking.json`, the four structure-piece registry entries and
`loot_table/chests/abandoned_mineshaft.json`.

**Test vectors:**

Replay fixed graph streams across all room sizes/wall skips, family/collision endpoints, corridor
length fallbacks, recursion/depth/range edges, two-floor upper exits and both relocation policies;
assert exact boxes, entrance persistence and final biome point. Across orientations and creation
orders, process every piece with full/partial/adversarial chunks and midpoint/deep-dark/liquid
boundaries; trace every ceiling/spider/support/cobweb/cart/spawner/floor/pillar/chain/rail draw and
write. Exhaust typed/null/rejected cart/spawner results, lower/upper support distances
`1/20/21/50/51`, live height/sturdiness, reload/visit orders, exact tags/set/records/piece IDs and
every loot entry. Use `EXP-WGEN-001` only for separately owned placement/distribution calibration.
