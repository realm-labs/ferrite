# World mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `WGEN-STRUCTURE-OCEAN-RUIN-001` — Ocean ruins restack live-height templates, globally cap archaeology, and spawn marker drowned

**Parent:** `WGEN-003`

**FidelityClass:** `EquivalentPlayerVisibleBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked 26.2 source fixes warm/cold and large/cluster selection, every template
identity and overlay, live-height restacking, the integrity/ignore/capped-archaeology processor
chain, chest and drowned marker transactions, and all four loot records. Both structure records,
their set, nine exact biome holders, 48 NBT inputs and the ice tag are audited data-only. Shared
random-spread admission and start/reference lifecycle remain owned by `WGEN-PIPELINE-001`; generic
post-insertion entity ticking remains owned by `ENT-LIFECYCLE-001`.

**Applies when:**

The `ocean_ruins` set selects its cold or warm record, or any retained `minecraft:ocean_ruin` piece
is offered to an intersecting placement chunk.

**Authoritative state:**

Cold and warm both use `surface_structures`, default `none` terrain adaptation, no spawn overrides,
`large_probability=.3` and `cluster_probability=.9`. Their set orders cold then warm at weight `1`,
with spacing `20`, separation `8`, salt `14357621` and default random-spread fields. Cold's exact
biome holders are frozen ocean, cold ocean, ocean, deep frozen ocean, deep cold ocean and deep
ocean; warm's are lukewarm ocean, warm ocean and deep lukewarm ocean. The structure tag contains
cold then warm.

**Transition and ordering:**

`onTopOfChunkCenter` builds the biome-test stub at the source chunk center and generator
`OCEAN_FLOOR_WG` first-occupied height. After that generic 3-D biome gate, piece construction
anchors at `(chunkMinX,90,chunkMinZ)`. It consumes a uniform rotation `nextInt(4)`, then a float and
makes a large parent when the value is **at most** `.3`; integrity is `.9` for large and `.8`
otherwise. Warm consumes a uniform index over `warm_1..8` or `big_warm_4..7` and adds one piece.
Cold consumes one matching index over suffixes `1..8` or large suffixes `1,2,3,8`, then adds brick,
cracked and mossy pieces at the same position/rotation, in that order, with integrities
`base/.7/.5`.

Only a large parent consumes the cluster float, admitting at **at most** `.9`. Cluster geometry
fixes the parent at Y `90`, transforms `(15,0,15)` about zero, and takes the minimum transformed X/Z
corner as its origin. It then eagerly consumes 16 inclusive integer draws to construct these eight
candidates in order: `(-16+[1..8],16+[1..7])`, `(-16+[1..8],[1..7])`, `(-16+[1..8],-16+[4..8])`,
`([1..7],16+[1..7])`, `([1..7],-16+[4..6])`, `(16+[1..7],16+[3..8])`, `(16+[1..7],[1..7])`, and
`(16+[1..7],-16+[4..8])`, all relative to that origin. An inclusive `4..8` draw chooses the attempt
count. Each attempt removes a uniform remaining candidate, consumes a uniform rotation, and forms
its box from transformed `(5,0,6)`. Intersection with the fixed parent box rejects it after those
draws; candidates never test one another. An accepted candidate adds a small integrity-`.8` warm
piece or the matching cold `.8/.7/.5` triplet and only then consumes its template index.

**Locked template audit:**

Every input has one palette, a dense block list filling its full volume, and zero template entities
or jigsaws. Small inputs are `6×7×7` with 294 raw cells; large inputs are `16×16×16` with 4,096. The
archaeology-candidate count and all `chest` (`C`) and `drowned` (`D`) data-marker coordinates are
exact below; `B/C/M` in the cold table mean brick/cracked/mossy overlays.

| Warm input | Candidate sand | Markers |
|---|---:|---|
| `warm_1` | 47 | C `(3,1,1)` |
| `warm_2` | 42 | C `(3,1,4)`; D `(2,2,2)`, `(3,2,2)` |
| `warm_3` | 36 | C `(3,0,4)`; D `(3,1,2)` |
| `warm_4` | 42 | C `(1,0,2)`; D `(2,1,2)` |
| `warm_5` | 37 | C `(3,0,4)`; D `(2,1,2)` |
| `warm_6` | 48 | C `(4,1,4)` |
| `warm_7` | 34 | C `(3,0,3)` |
| `warm_8` | 56 | C `(3,0,3)`; D `(1,2,3)` |
| `big_warm_4` | 220 | C `(11,0,8)`; D `(3,1,5)`, `(4,2,6)` |
| `big_warm_5` | 222 | C `(7,0,7)`; D `(5,1,5)`, `(7,1,9)`, `(10,1,12)`, `(11,1,4)` |
| `big_warm_6` | 219 | C `(10,0,9)`; D `(5,1,4)`, `(9,1,9)` |
| `big_warm_7` | 202 | C `(11,0,7)`; D `(4,1,5)`, `(10,1,8)` |

#### `1`

**Candidate gravel B/C/M:**

36 / 36 / 38

**Marker audit:**

C B/C `(3,1,5)`; D M `(1,1,2)`

#### `2`

**Candidate gravel B/C/M:**

39 / 30 / 37

**Marker audit:**

C all `(2,0,1)`; D B `(2,1,5)`, `(3,1,2)`

#### `3`

**Candidate gravel B/C/M:**

30 / 35 / 30

**Marker audit:**

C all `(1,0,5)`

#### `4`

**Candidate gravel B/C/M:**

30 / 35 / 27

**Marker audit:**

C all `(1,1,4)`; D B `(2,1,5)`, `(4,1,5)`

#### `5`

**Candidate gravel B/C/M:**

39 / 37 / 39

**Marker audit:**

C all `(4,0,4)`; D M `(3,1,3)`

#### `6`

**Candidate gravel B/C/M:**

41 / 41 / 43

**Marker audit:**

C all `(2,0,2)`; D B `(2,1,3)`, `(1,6,4)`, `(3,6,1)`, `(3,6,4)`; M `(2,6,2)`

#### `7`

**Candidate gravel B/C/M:**

35 / 35 / 35

**Marker audit:**

C all `(1,0,3)`; D C `(2,1,3)`; M `(2,1,2)`

#### `8`

**Candidate gravel B/C/M:**

25 / 28 / 27

**Marker audit:**

C all `(3,1,4)`; D B `(2,1,4)`; M `(4,1,4)`

#### `big_1`

**Candidate gravel B/C/M:**

239 / 215 / 226

**Marker audit:**

C all `(5,0,4)`; D B `(6,1,4)`, `(8,1,11)`; C `(12,1,11)`; M `(6,1,8)`

#### `big_2`

**Candidate gravel B/C/M:**

192 / 206 / 199

**Marker audit:**

C all `(9,1,10)`; D B `(10,1,8)`; C `(9,1,7)`, `(12,1,9)`; M `(4,1,5)`, `(9,1,6)`

#### `big_3`

**Candidate gravel B/C/M:**

205 / 217 / 208

**Marker audit:**

C all `(12,2,2)`; D B `(2,1,7)`, `(6,1,9)`; C `(9,2,2)`, `(9,2,7)`, `(13,2,6)`; M `(5,1,9)`

#### `big_8`

**Candidate gravel B/C/M:**

210 / 224 / 221

**Marker audit:**

C all `(5,0,4)`; D B `(3,1,8)`, `(10,1,9)`; C `(2,1,4)`; M `(7,1,8)`, `(11,1,10)`

**Live height and overlay ordering:**

Every referenced-chunk invocation independently samples `OCEAN_FLOOR_WG` at the piece origin and
resets its Y to that value; no height is latched. It transforms `(sizeX-1,0,sizeZ-1)` about zero and
scans the inclusive origin/corner rectangle with X fastest and Z slowest. Each column starts one
below the sampled origin and descends while the state is air, its fluid is tagged water, or the
state is in `#ice` (ice, packed ice, blue ice and frosted ice), stopping before descending past
`level.minY+1`. Let `min` be the lowest result, `top=originY-1`, `area` count columns whose result
is below `originY-3`, and `width=abs(originX-cornerX)`—the coordinate difference, not the inclusive
span. Only when `top-min>2` and `area>width-2` does the piece move to `min+1`; otherwise it retains
the sampled origin Y.

The generic template transaction then runs with the current chunk box. Cold overlays invoke brick,
cracked and mossy in that order in every chunk, so earlier overlays and earlier chunks can change
the live height read by later pieces. Rotation about zero can extend into negative axes. Because Y
is recomputed rather than persisted, chunk visitation and preceding overlay writes can restack a
logical triplet or make its per-chunk processor decisions disagree.

**Processor and placement transaction:**

Settings use mirror none, the saved rotation, zero pivot, default liquid/entity/known-shape behavior
and processors in this order: untagged block rot at the piece integrity; structure-and-air ignore;
then the warm/cold archaeology cap. Block rot visits every transformed raw cell—including air and
structure-marker cells—with a fresh settings RNG keyed by transformed world position and retains it
when `nextFloat()<=integrity`; ignore then removes retained air and structure blocks. Marker
filtering later reads the raw palette independently, so marker handling is not integrity-gated.

The cap evaluates whole-piece state, disabling the initial current-box clip for all processors. It
creates a thread-local RNG from the world seed, forks positional randomness at the **current
adjusted template origin**, shuffles all surviving processed indices, and scans until five actual
replacements or list exhaustion. Warm changes sand to suspicious sand and cold changes gravel to
suspicious gravel. The delegate rule uses a separate fresh seed derived from each transformed world
position; a selected replacement copies/creates NBT and stores its archaeology table plus one
`nextLong` as `LootTableSeed`. Thus survivor admission is position/integrity deterministic, while
the globally capped candidate subset depends on world seed and the invocation's live Y. The final
write loop clips this whole-piece result back to the supplied chunk box. Re-evaluation under another
Y or visit order can therefore change which in-box cells are suspicious.

Generic placement transforms and offers retained states, applies barrier/NBT handling, default fluid
preservation/filling and neighbor-shape repair, and tracks accepted extents. The locked inputs have
no raw block entities other than generated marker metadata and no entity list. Suspicious block
entities retain the `AppendLoot` table and seed; they are not generic randomizable containers and
consume no caller `nextLong`. Placement returns true for these positive nonempty templates, after
which raw `DATA` markers inside the current box are filtered in palette order and dispatched.
Unknown marker IDs are inert.

**Marker transactions:**

A `chest` marker first reads the existing fluid, then offers a chest with `WATERLOGGED` equal to
water-tag membership using flags `2`, ignoring the write result. It immediately queries the
resulting block entity. Only a `ChestBlockEntity` consumes caller `nextLong` and installs
`chests/underwater_ruin_big` when the persisted piece is large, otherwise
`chests/underwater_ruin_small`; a hostile pre-existing/resulting typed chest can therefore be
initialized even after a rejected write. Each of a cold triplet's marker copies runs independently.

A `drowned` marker creates a drowned for `STRUCTURE`; creation failure leaves the marker cell
untouched and has no later side effects. Success marks persistence, snaps to the marker with
yaw/pitch zero, obtains current local difficulty, invokes the full drowned finalizer with null group
data, and offers the drowned with passengers. Regardless of that admission result, it finally offers
air at marker Y above sea level or water otherwise, flags `2`, ignoring failure. Finalization uses
the level/entity stream rather than the structure caller RNG: generic mob setup gives the absent
follow-range modifier a triangular `±0.11485` bonus and samples 5% left-handedness; zombie setup
samples pickup ability against `.55*special`, then baby below `.05`. A baby with jockey permission
consumes a first `.05` gate: below it the source searches only the first nearby unridden chicken and
does not fall back when none exists; at or above it a second `.05` gate can create, finalize and
separately offer a new chicken before mounting. Door breaking is next sampled against `.1*special`,
followed by the drowned main-hand gate `nextFloat()>.9` and `nextInt(16)` selecting trident for
`0..9` or fishing rod for `10..15`. A nonempty main hand is enchant-eligible below `.25*special`
through the locked mob-spawn provider. Exact October 31 can add a pumpkin head below `.25`, with
jack-o-lantern below a following `.1`. Zombie reinforcement, knockback, follow-range and rare-leader
attributes are then sampled by the inherited finalizer.

Drowned postprocessing next gives an empty offhand a nautilus shell below `.03` and makes it
guaranteed-drop. For `STRUCTURE`, every trident main hand consumes a `.5` jockey gate before age is
tested. Only below that gate, for an adult outside `#more_frequent_drowned_spawns`, a successfully
created zombie nautilus is made persistent, snapped to the drowned, finalized with the same local
difficulty and `STRUCTURE`, mounted by the drowned, and separately offered before the outer marker
offers the drowned and any passengers. Offer and ride booleans are ignored. Later entity lifecycle,
AI, movement, drops and removal belong to `ENT-LIFECYCLE-001`; this leaf owns their exact
construction, initial state and insertion attempts.

**Locked loot records:**

Both archaeology tables have one matching-sequence roll. Cold gives weight `1` to blade, explorer,
mourner and plenty pottery sherds plus iron axe; warm gives weight `1` to angler, shelter and snort
sherds, sniffer egg and iron axe. Both give weight `2` to emerald, wheat, wooden hoe, coal and gold
nugget.

Both chest tables have matching random-sequence IDs. The big table's first pool rolls uniformly
`2..8` over coal weight `10` count `1..4`, gold nugget `10` count `1..3`, emerald `1`, stone spear
`2`, and wheat `10` count `2..3`; the small table instead uses coal `10` count `1..4`, stone axe
`2`, stone spear `2`, rotten flesh `5`, emerald `1`, and wheat `10` count `2..3`. Each second pool
makes one roll: big chooses golden apple weight `1`, enchanted book `5`, leather chestplate `1`,
golden helmet `1`, enchanted fishing rod `5`, or a weight-`10` buried-treasure exploration map with
red-X decoration, zoom `1`, `skip_existing_chunks=false` and translated name; small omits apple/book
and gives the same map weight `5`. Both third pools choose empty/copper/iron/gold/diamond nautilus
armor at weights `148/20/10/5/2`, armor count `1`. Loot evaluation is deferred and owned by
`ITM-LOOT-001`; this leaf owns table installation and exact data linkage.

**Branches and aborts:**

Cold/warm record and stub biome; four rotations; float equality at both probabilities; large/small
and every parent index; cluster accepted/rejected and all attempt counts/candidate
removals/rotations; every template overlay and integrity equality; live height/ice/water/min-Y/area
thresholds; chunk orders; whole-piece survivor/cap/shuffle/output and final clip; write/fluid/shape
outcomes; marker ID, creation/entity/write/admission outcomes; local difficulty,
baby/equipment/enchantment/date/attribute/shell/nautilus-jockey gates.

**Constants and randomness:**

Start order is rotation, large float, parent index, conditional cluster float, then the fixed 16
candidate integers, attempt-count integer and per-attempt candidate/rotation/accepted-template
draws. Position-seeded integrity and delegate streams, and world-seed-plus-origin cap shuffle, do
not advance caller RNG. Caller RNG advances only for successfully queried typed marker chests.
Drowned finalization advances the level/entity stream as gated above and may recursively advance it
for chicken or zombie-nautilus finalization.

**Side effects:**

One warm piece or three cold overlays, optionally plus accepted small cluster pieces; mutable live
Y; chunk-clipped integrity ruins; at most five suspicious blocks per invocation with exact
archaeology NBT; waterlogged big/small loot chests; persistent finalized drowned and possible
chicken/zombie-nautilus mounts; marker air/water cleanup.

**Gates:**

Caller-owned start/reference/placement lifecycle; record biome and probabilities; parent-box-only
cluster collision; live heightmap/state/fluid/ice; processor and chunk clip; resulting block-entity
class; drowned/mount creation, local difficulty, date, biome tag and entity admission.

**Boundary cases and quirks:**

Probability tests are inclusive, so configured zero still admits the one exact zero float. Rejected
cluster candidates still consume removal and rotation but no template index and never block one
another. `width` is one less than the unrotated inclusive X span and changes with quarter-turns for
nonsquare small templates. The cap sees the entire transformed survivor list before final chunk
clipping, and all of its seeds move with recomputed Y. Cold layers and chunks can therefore
influence one another's alignment and archaeology choice. Marker selection bypasses integrity. Chest
and drowned writes ignore failure, and a drowned is offered before its marker cell is cleaned even
when the offer fails.

**Evidence:**

`Confirmed`; `OFF-SERVER-001`; `OFF-DATA-001`; `OFF-REPORT-001`. Anchors:
`net.minecraft.world.level.levelgen.structure.structures.OceanRuinStructure#findGenerationPoint`,
`net.minecraft.world.level.levelgen.structure.structures.OceanRuinStructure#generatePieces`,
`net.minecraft.world.level.levelgen.structure.structures.OceanRuinPieces#addPieces`,
`net.minecraft.world.level.levelgen.structure.structures.OceanRuinPieces#addClusterRuins`,
`net.minecraft.world.level.levelgen.structure.structures.OceanRuinPieces#allPositions`,
`net.minecraft.world.level.levelgen.structure.structures.OceanRuinPieces#addPiece`,
`net.minecraft.world.level.levelgen.structure.structures.OceanRuinPieces$OceanRuinPiece#makeSettings`,
`net.minecraft.world.level.levelgen.structure.structures.OceanRuinPieces$OceanRuinPiece#postProcess`,
`net.minecraft.world.level.levelgen.structure.structures.OceanRuinPieces$OceanRuinPiece#getHeight`,
`net.minecraft.world.level.levelgen.structure.structures.OceanRuinPieces$OceanRuinPiece#handleDataMarker`,
`net.minecraft.world.level.levelgen.structure.templatesystem.BlockRotProcessor#processBlock`,
`net.minecraft.world.level.levelgen.structure.templatesystem.CappedProcessor#finalizeProcessing`,
`net.minecraft.world.level.levelgen.structure.templatesystem.RuleProcessor#processBlock`,
`net.minecraft.world.level.levelgen.structure.templatesystem.rule.blockentity.AppendLoot#apply`,
generic template placement/filtering,
`net.minecraft.world.entity.monster.zombie.Zombie#finalizeSpawn`,
`net.minecraft.world.entity.monster.zombie.Drowned#finalizeSpawn`, all 48
`data/minecraft/structure/underwater_ruin/*.nbt` inputs, both structure records, the set and
biome/structure/ice tags, and all four archaeology/chest loot records.

**Test vectors:**

Cross both records, exact biome holders, all probability endpoints, every template
index/overlay/marker coordinate, cluster candidate and parent-only intersection; assert all 48 dense
single-palette inputs and exact data decodes. Replay rotations, live-height scans at every threshold
and chunk/overlay orders; trace integrity/ignore/cap streams, five-change cap and final clipping
under shifted Y. Exercise chest writes/entities/seeds and drowned
creation/finalizer/admission/cleanup including local difficulty, date, equipment, shell, chicken and
zombie-nautilus branches. Decode every loot pool; use `EXP-WGEN-001` only for separately owned
placement/distribution calibration.
