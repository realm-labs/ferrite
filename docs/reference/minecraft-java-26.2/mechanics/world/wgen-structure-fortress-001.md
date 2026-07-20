# World mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `WGEN-STRUCTURE-FORTRESS-001` — Nether fortresses alternate quota-weighted bridge and castle frontiers

**Parent:** `WGEN-003`

**FidelityClass:** `EquivalentPlayerVisibleBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked 26.2 source fixes the start stub, complete bridge/castle weighted graph,
range/depth/filler behavior, all 15 piece families and geometry, vertical relocation, supports,
chest/spawner/lava transactions, persistence, spawn override and sole loot record. The fortress
record, shared Nether-complexes set and exact five-biome tag are audited data-only; the set's
bastion branch remains owned by the later jigsaw slice. Shared random-spread admission,
start/reference/placement scheduling and natural-spawn override consumption remain with
`WGEN-PIPELINE-001` and the mob-spawning owner.

**Applies when:**

The `nether_complexes` set selects `minecraft:fortress`, an admitted fortress graph is constructed,
a retained fortress piece intersects a placement chunk, or generic natural spawning queries that
retained structure's piece-box override.

**Authoritative state:**

The record uses `underground_decoration`, default `none` terrain adaptation and all five Nether
biomes through `#minecraft:has_structure/nether_fortress -> #minecraft:is_nether`: Nether wastes,
soul-sand valley, crimson forest, warped forest and basalt deltas. Its monster override uses
individual **piece** boxes and the ordered weighted entries blaze weight `10` count `2..3`,
zombified piglin `5` count `4`, wither skeleton `8` count `5`, skeleton `2` count `5`, and magma
cube `3` count `4`. The shared set orders fortress weight `2`, bastion remnant weight `3`; linear
random spread uses spacing `27`, separation `4`, salt `30084232`, default frequency and locate
offset.

**Transition and ordering:**

The generation stub is unconditionally source-chunk minimum X, Y `64`, minimum Z; the generic 3-D
biome gate samples that point. Deferred generation consumes one uniform horizontal-direction draw
and creates a start crossing anchored at chunk-local `(2,64,2)` with oriented size `19×10×19`. It
resets both static weight arrays, adds the start, and immediately requests its forward, left and
right bridge children in that order. Every successful child is appended to the builder and to a
pending list. The main loop repeatedly consumes `nextInt(pending.size)`, removes that entry and
expands it; selection is therefore a random frontier walk rather than depth-first recursion.

The bridge pool and castle pool are fixed and ordered:

| Pool piece | Weight | Maximum | Consecutive admission |
|---|---:|---:|---|
| bridge straight | 30 | unlimited | allowed |
| bridge crossing | 10 | 4 | rejected |
| room crossing | 10 | 4 | rejected |
| stairs room | 10 | 3 | rejected |
| monster throne | 5 | 2 | rejected |
| castle entrance | 5 | 1 | rejected |
| castle small corridor | 25 | unlimited | allowed |
| castle small crossing | 15 | 5 | rejected |
| castle right turn | 5 | 10 | rejected |
| castle left turn | 5 | 10 | rejected |
| castle corridor stairs | 10 | 3 | allowed |
| castle T balcony | 7 | 2 | rejected |
| castle stalk room | 5 | 2 | rejected |

Initial totals are `70` and `72`. Before selection, the total includes every entry still in that
pool, but generation remains enabled only while at least one **finite** entry has quota remaining;
unlimited entries alone return `-1`. A successful finite entry is removed on reaching its maximum.
One start-wide previous-entry pointer spans both pools. The two unlimited entries and castle stairs
allow immediate repetition; every other selected repeat aborts that attempt. The static
`PieceWeight` objects—and their mutable counts—are shared across starts and reset by each start
constructor, while the two available lists and previous pointer are per start. They are
unsynchronized: a fixed interleaving observes cross-start resets/counts; scheduling guarantees
remain with the pipeline owner.

An enabled request makes at most five attempts. Each consumes one `nextInt(totalWeight)`, subtracts
weights in pool order and selects the first negative result. An exhausted entry or disallowed repeat
aborts that whole attempt. A null factory instead continues through later entries with the
already-negative accumulator, so later factories can be tried without another draw; a later
exhausted/repeated entry still aborts the attempt. Success increments quota, stores the previous
entry, possibly removes it and returns. Five failed attempts, depth above `30`, or a pool with no
finite quota falls through to a bridge-end filler. The entry's `doPlace(depth)` parameter is
ignored; quota alone decides.

Every in-range ordinary request passes parent depth plus one into this selection. Thus a depth-`29`
parent can select depth `30`; a depth-`30` parent creates only a depth-`31` filler. Before pool
selection, an anchor whose absolute X or Z distance from the start box minimum exceeds `112`
bypasses weights and proposes a filler using the **parent's** depth, not plus one; equality selects
normally. Every factory first requires proposed minimum Y greater than `10` and no inclusive
intersection with an existing box. Null filler means no append. Right/left-turn constructors consume
`nextInt(3)==0` for their chest latch only after box admission; a valid filler consumes one full
signed `nextInt()` self-seed. Bridge-straight and castle-entrance constructors receive but do not
use their random argument.

After the pending list empties, let `B` be the union and `h=B.ySpan`.
`moveInsideHeights(random,48,70)` computes `r=23-h`. When `r>1`, one `nextInt(r)` moves every piece
so union minimum Y is uniformly `48..70-h`, hence maximum at most `69`. When `r<=1`, no draw occurs
and union minimum becomes exactly `48`, even when its maximum then exceeds `70`. The generation stub
and biome-test point remain at Y `64`.

**Piece graph and persistence:**

`F/L/R` are transformed forward/left/right exits; offsets are local arguments before orientation.
All boxes below use `orientBox(anchor,offset,size)` except the special start:

#### `nestart` start

**Box offset; size:**

anchored `19×10×19` at Y `64`

**Child requests:**

bridge F `(8,3)`, L/R `(3,8)`

#### `nebs` bridge straight

**Box offset; size:**

`(-1,-3,0)`; `5×10×19`

**Child requests:**

bridge F `(1,3)`

#### `nebcr` bridge crossing

**Box offset; size:**

`(-8,-3,0)`; `19×10×19`

**Child requests:**

bridge F `(8,3)`, L/R `(3,8)`

#### `nerc` room crossing

**Box offset; size:**

`(-2,0,0)`; `7×9×7`

**Child requests:**

bridge F `(2,0)`, L/R `(0,2)`

#### `nesr` stairs room

**Box offset; size:**

`(-2,0,0)`; `7×11×7`

**Child requests:**

bridge R `(6,2)`

#### `nemt` monster throne

**Box offset; size:**

`(-2,0,0)`; `7×8×9`

**Child requests:**

none

#### `nece` castle entrance

**Box offset; size:**

`(-5,-3,0)`; `13×14×13`

**Child requests:**

castle F `(5,3)`

#### `nesc` small corridor

**Box offset; size:**

`(-1,0,0)`; `5×7×5`

**Child requests:**

castle F `(1,0)`

#### `nescrt` / `nesclt` right / left turn

**Box offset; size:**

`(-1,0,0)`; `5×7×5`

**Child requests:**

castle R/L `(0,1)`

#### `neccs` castle corridor stairs

**Box offset; size:**

`(-1,-7,0)`; `5×14×10`

**Child requests:**

castle F `(1,0)`

#### `nectb` castle T balcony

**Box offset; size:**

`(-3,0,0)`; `9×7×9`

**Child requests:**

L then R at Y `0`, Z `5` for north/west else `1`; each consumes `nextInt(8)>0` for castle versus
bridge pool

#### `nescsc` castle small crossing

**Box offset; size:**

`(-1,0,0)`; `5×7×5`

**Child requests:**

castle F `(1,0)`, L/R `(0,1)`

#### `necsr` castle stalk room

**Box offset; size:**

`(-5,-3,0)`; `13×14×13`

**Child requests:**

castle F `(5,3)`, then F `(5,11)`

#### `nebef` bridge-end filler

**Box offset; size:**

`(-1,-3,0)`; `5×10×8`

**Child requests:**

none

Every piece persists generic ID, box, horizontal orientation and generation depth. The throne
additionally saves `Mob`, both turns save `Chest`, and filler saves `Seed`; missing values decode
false/zero. Start weight lists, counts, previous entry and pending frontier are transient and reload
empty; ordinary saved starts are already fully expanded.

**Shared placement transaction and notation:**

Local coordinates rotate through piece orientation. Write calls reject only positions outside the
supplied processing box; otherwise flags-`2` state offers ignore their result, then reread fluid and
schedule its type at delay `0` when nonempty. Nether-brick fence and stairs are marked for
postprocessing. `generateBox` traverses Y outermost, X middle, Z inner and every fortress call
passes `skipAir=false` with identical edge/fill state. In the ordered recipes below, `N[x,y,z]`,
`A[...]`, `F_NS/F_WE[...]` mean inclusive nether-bricks, ordinary air, or fence ranges; later
overlapping calls are still offered.

`D[x,z]` means `fillColumnDown` from local Y `-1`. It first requires that start inside the
processing box, then directly writes nether bricks downward with flags `2` while the live state is
air, liquid, glow lichen, seagrass or tall seagrass and world Y is greater than `minY+1`. The
scan/fill is no longer vertically clipped, stops before writing at `minY+1`, and bypasses fluid
scheduling/shape postprocessing. Unless stated otherwise, rectangular support loops visit X outer
then Z inner. Geometry and support placement consume no caller RNG.

**Bridge geometry:**

- Bridge straight offers N floor `x=0..4,y=3..4,z=0..18`, A passage `x=1..3,y=5..7,z=0..18`, N side
  ledges `x=0|4,y=5,z=0..18`, lower end pads `x=0..4,y=2,z=0..5|13..18`, and bottom end blocks
  `y=0..1,z=0..3|15..18`. It supports X `0..4`, near Z `0..2` then matching far Z per iteration.
  Connected north-south fences occupy each side at Z `1` Y `1..4`, Z `4|14` Y `3..4`, and Z `17` Y
  `1..4`, with inward east/west arms.
- Start and ordinary bridge crossings use the same recipe: N intersecting decks
  `x=7..11,y=3..4,z=0..18` then `x=0..18,y=3..4,z=7..11`; A inner cross `x=8..10,y=5..7,z=0..18`
  then `x=0..18,y=5..7,z=8..10`. N one-cell parapets bound each arm at X `7|11`, Y `5`, Z
  `0..7|11..18` and at Z `7|11`, Y `5`, X `0..7|11..18`. North/south lower pads are X `7..11`, Y
  `2`, Z `0..5|13..18`, with bottom blocks Y `0..1`, Z `0..3|15..18`; supports visit X `7..11`, Z
  `0..2`, near then far. West/east pads are Z `7..11`, Y `2`, X `0..5|13..18`, with bottom blocks X
  `0..3|15..18`, and supports visit X offset `0..2`, Z `7..11`, west then east.
- Room crossing offers N whole floor `0..6,0..1,0..6`, A `0..6,2..7,0..6`, then two-wide corner wall
  legs at every horizontal corner through Y `6`. At Y `6`, N lintels span X `2..4` on Z `0|6` and Z
  `2..4` on X `0|6`; Y `5` beneath each lintel is a connected east-west or north-south fence. It
  supports all 49 columns.
- Stairs room offers N floor `0..6,0..1,0..6`, A `0..6,2..10,0..6`, N front corner pillars, full
  side walls through Y `8`, and back wall `x=1..5,y=2..8,z=6`. Its rising internal N profiles at Z
  `5` are X `5` Y `2`, X `4` Y `2..3`, X `3` Y `2..4`, X `2` Y `2..5`, X `1` Y `2..6`, plus N roof
  `x=1..5,y=7,z=1..4`, an air side aperture `x=6,y=8,z=2..4`, front lintel `x=2..4,y=6..8,z=0`, and
  fence below at Y `5`. Fence windows are X `0`, Y `3..5`, Z `2..4`, and X `6`, Y `3..5`, Z `2|4`.
  It supports all 49 columns.
- Monster throne first clears A `x=0..6,y=2..7,z=0..7`, lays N `x=1..5,y=0..1,z=0..7`, then
  stair-step platforms X `1..5` at `(Y,Z starts)=(2,1),(3,2),(4,3)` through Z `7`. N side/front/back
  walls lead into a three-tier fence crown. At Y `6`, `(1,3)` has west, `(5,3)` east, `(0,3)`
  east+north and `(6,3)` west+north; north-south rails continue on X `0|6`, Z `4..7`; `(0,8)` has
  east+south, `(6,8)` west+south, and X `1..5`, Z `8` is east-west. At Y `7`, Z `8`, X `1` has east,
  X `2..4` east-west and X `5` west. At Y `8`, Z `8`, X `2` has east, X `3` east-west and X `4`
  west. Those three Y-`8` cells lie one block above the declared `7×8×9` piece box but remain
  processing-box clipped and writable. After the crown it handles the spawner below, then supports
  every X/Z `0..6` column.

**Castle geometry:**

Define the shared 13-room shell as these ordered offers: N whole `0..12,3..4,0..12`; A
`0..12,5..13,0..12`; N side walls X `0..1|11..12`, Y `5..12`; segmented front/back walls X
`2..4|8..10`, Y `5..12`, plus X `5..7`, Y `9..12`; and N inner roof `2..10,11..12,2..10`. Its
battlement pass visits odd `i=1,3,5,7,9,11`: connected fences at `(x=i,y=10..11,z=0|12)` and
`(x=0|12,y=10..11,z=i)`, N cap blocks at the four matching Y-`13` edge cells, and, except at `i=11`,
fence links at the following even coordinate. Four corner fences join the edge axes. Side windows at
Z `3,5,7,9` use north-south fences with an inward arm at X `1|11`, Y `7..8`.

The shared foundation is N crossing strips `x=4..8,y=2,z=0..12` then `x=0..12,y=2,z=4..8`, bottom
blocks Y `0..1` over the four end rectangles, then support loops: X `4..8`, Z `0..2`, near/far;
followed by X offset `0..2`, Z `4..8`, west/east.

- Castle entrance applies the shell, adds a default-fence front lintel `x=5..7,y=8,z=0`,
  battlements/windows, then the shared foundation. It finally offers N `5..7,5,5..7`, A shaft
  `(6,1..4,6)`, N bottom `(6,0,6)`, lava `(6,5,6)`, and—when that target is in the processing
  box—explicitly schedules lava delay `0` regardless of the write result. A successful lava offer
  also takes the generic live-fluid scheduling path.
- Small corridor offers N floor `0..4,0..1,0..4`, A interior `0..4,2..5,0..4`, N side walls X `0|4`,
  a N roof at Y `6`, and north-south fence windows on both sides at Z `1|3`, Y `3..4`; it supports
  all 25 columns.
- Right turn uses that floor/air/roof, N west wall, N front-east and south-east walls, north-south
  west windows and east-west south windows; left turn mirrors these at the opposite X side. Right
  targets chest `(1,2,3)`, left `(3,2,3)`. Both support all 25 columns. Small crossing uses the same
  floor/air/roof but only the four full-height corner columns, then supports all 25 columns.
- Castle corridor stairs iterates step `s=0..9`, Z=`s`, with `floor=max(1,7-s)` and
  `roof=min(max(floor+5,14-s),13)`. In order it offers N from Y `0..floor` across X `0..4`, A X
  `1..3` through `roof-1`, three south-facing stairs at Y `floor+1` when `s<=6`, N roof across X, N
  side walls X `0|4`, north-south fence pairs at Y `floor+2..floor+3` for even `s`, then supports X
  `0..4` at that Z.
- Castle T balcony offers N floor `0..8,0..1,0..8`, A `0..8,2..5,0..8`, N roof `0..8,6,0..5`, a
  three-wide front wall at both ends with fence windows, N rear deck `0..8,2,4..8`, and deliberate
  air cuts `x=1..2|6..7,y=1..2,z=4`. Its rear rail is east-west fence X `1..7`, Y `3`, Z `8` with
  corner joins, plus north-south side rails at X `0|8`, Z `6..7`; N shoulder blocks fill X `0|8`, Y
  `3..5`, Z `4..5` and X `1..2|6..7`, Y `3..5`, Z `5`, with fence faces at X `1|7`, Y `4..5`. It
  supports X `0..8`, Z `0..5` only, visiting Z outer then X inner rather than the default
  support-loop order.
- Castle stalk room applies the shared shell, battlements and windows without the entrance lintel.
  It builds north-facing stairs X `5..7` for `i=0..6` at `(Y,Z)=(5+i,4+i)`, with N underfill at Z
  `5..8` from Y `5` through `i+4`, at Z `9..10` from Y `8` through `i+4`, and A headroom for `i>=1`
  from Y `6+i..9+i`; three more stairs sit at Y `12`, Z `11`. Fence rails flank local
  `(x=5|7,y=6..7,z=7)`, and A opens `x=5..7,y=13,z=12`. Four N farm rims occupy
  `x=2..3|9..10,y=5,z=2..3|9..10`, their lengthwise sides continue at X `2|10`, Z `4..8`;
  inward-facing stairs border them at X `4|8`, Z `2..3|9..10`. Soul sand fills X `3..4|8..9`, Y `4`,
  Z `4..8`; Nether wart overwrites the same X/Z at Y `5`. The shared foundation/supports run last.

**Terminal filler, containers, spawner and loot:**

A valid bridge-end filler stores one graph-RNG integer. Every placement invocation creates the
locked thread-local random implementation from that seed and repeats exactly 27 draws: for X `0..4`,
Y `3..4`, ten `nextInt(8)` values fill N from Z `0` through the inclusive result; two more
`nextInt(8)` values do X `0|4`, Y `5`; five `nextInt(5)` values do each X at Y `2`; then X `0..4`, Y
`0..1` consumes ten `nextInt(3)` values. Caller placement RNG is unused, and chunk clipping changes
writes but not this local draw sequence.

Each admitted right/left turn begins with its saved chest latch from the graph draw. When latched
and the target lies in the processing box, placement clears the latch **before** calling the generic
chest helper. An existing chest, rejected state offer or wrong/missing resulting block entity
therefore still permanently spends the latch. Otherwise the helper reorients a chest from live
neighbors, offers flags `2`, and only a resulting `RandomizableContainer` consumes `nextLong` and
receives `chests/nether_bridge`; its Boolean result is ignored.

The monster throne similarly checks saved `Mob` false and local `(3,5,5)` in-box, sets the latch
before directly offering a spawner, then configures only a resulting `SpawnerBlockEntity` for blaze.
A fresh spawner's empty spawn-data path consumes no caller RNG; adversarial weighted potentials can
consume their selection draw. Wrong/missing entities leave the latch true.

`chests/nether_bridge` uses the matching random sequence. Pool one rolls uniformly `2..4`: diamond
weight `5` count `1..3`; iron ingot `5` count `1..5`; gold ingot `15` count `1..3`; golden sword,
golden chestplate and flint-and-steel weight `5`; Nether wart `5` count `3..7`; saddle `10`; golden
horse armor `8`; copper and iron horse armor `5`; diamond horse armor `3`; obsidian `2` count
`2..4`. Pool two rolls once between empty weight `14` and one rib armor-trim template weight `1`.
Evaluation remains owned by `ITM-LOOT-001`; this leaf owns exact linkage and seed installation.

**Branches and aborts:**

Stub biome; four start directions; every pending index; bridge/castle pool; finite/unlimited
survival; five weighted endpoints, repeat/allow-row/quota/null-factory cascade; depth `29/30/31`,
range `112/113`, minY `10/11`, collision and filler success; both balcony pool integers; union spans
around relocation `r=1/2`; every piece orientation/clip/write/fluid/support state; chest
constructor/latch/target/helper/entity outcomes; spawner latch/write/entity/potentials; explicit
lava target/schedule; filler seed and every local draw.

**Constants and randomness:**

Graph order is start direction, three start child selections, then one pending-index draw before
each expansion and that piece's child/weight/factory draws. Pool selection uses one integer per
attempt, not per cascaded factory. Turn chest and filler-seed draws occur only after valid boxes.
Relocation draws only when `23-unionYSpan>1`. Placement geometry is deterministic except the
filler's private repeated stream, conditional typed-chest long and adversarial spawner selection;
loot and natural spawning own separate streams.

**Side effects:**

A quota-selected, randomly frontier-expanded and vertically relocated 15-family graph; persisted
orientations/boxes/depth and three special fields; chunk-clipped Nether-brick floors, bridges,
walls, fences, stairs, air, supports, soul sand, wart and lava; fluid ticks and shape
postprocessing; latched turn-piece treasure chests; latched blaze spawners; piece-box monster
spawn-list replacement.

**Gates:**

Caller-owned set/start/reference/placement scheduling and random-spread choice; exact stub biome;
shared weight state, previous/quota/depth/range/Y/collision; union height; processing box and live
block/fluid/write/entity state; later generic natural-spawn and loot eligibility.

**Boundary cases and quirks:**

Unlimited pieces cannot keep a pool alive after all finite quotas exhaust. Null factories can
cascade to later entries without a draw; repeats abort instead. Range fallback keeps parent depth,
while ordinary fallback uses child depth. Large graphs taller than `21` are forced to minimum Y `48`
and may exceed the nominal upper bound. Static quota objects can cross-contaminate interleaved
starts. Saved starts do not reconstruct frontier state. The throne writes three fence cells above
its own box. Filler geometry is seed-replayed per chunk. Chest/spawner latches commit before
successful creation, and the entrance explicitly schedules lava even after a rejected offer.

**Evidence:**

`Confirmed`; `OFF-SERVER-001`; `OFF-DATA-001`; `OFF-REPORT-001`. Anchors:
`net.minecraft.world.level.levelgen.structure.structures.NetherFortressStructure#findGenerationPoint`,
`#generatePieces`, `FORTRESS_ENEMIES`;
`net.minecraft.world.level.levelgen.structure.structures.NetherFortressPieces#findAndCreateBridgePieceFactory`;
`NetherFortressPieces$NetherBridgePiece#updatePieceWeight`, `#generatePiece`, `#generateAndAddPiece`
and all child transforms; all 15 nested piece constructors, factories, child and `postProcess`
methods; `StructurePiecesBuilder#moveInsideHeights`; generic structure write/support/chest helpers;
fortress record, `structure_set/nether_complexes.json`, both biome tags, all 15 piece registry
entries and `loot_table/chests/nether_bridge.json`.

**Test vectors:**

Replay fixed streams across every pool endpoint, quota removal, allowed/disallowed repeat,
null-factory cascade, depth/range/Y/collision/filler path, pending-frontier order, balcony pool
split and relocation span; assert all 15 boxes, exits, IDs, fields and concurrency schedules. For
every orientation and hostile chunk/state/write implementation, trace every ordered geometry range,
overlap, support descent, postprocessing/fluid schedule, turn chest, throne spawner, entrance lava
and all 27 filler draws; assert exact record/set/tags/spawn list/loot data. Use `EXP-WGEN-001` only
for separately owned distribution/locate calibration.
