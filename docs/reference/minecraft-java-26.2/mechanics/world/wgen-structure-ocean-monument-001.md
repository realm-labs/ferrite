# World mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `WGEN-STRUCTURE-OCEAN-MONUMENT-001` — Ocean monuments prune a connected room lattice inside a fixed flooded shell

**Parent:** `WGEN-003`

**FidelityClass:** `EquivalentPlayerVisibleBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked 26.2 source fixes the two-stage biome gate, fixed-height top piece,
complete 46-room graph/pruning/fitter transaction, all 12 registered piece families, shell and room
geometry, water/ice preservation, foundations, sponge and simple-room placement randomness,
eight-block gold core, three elder-guardian transactions and deterministic load regeneration. The
monument record, triangular random-spread set and exact start/surrounding biome tags are audited
data-only. Shared set admission, start/reference/placement scheduling and natural-spawn override
consumption remain with `WGEN-PIPELINE-001` and the mob-spawning owner; later guardian behavior
remains with the entity subsystem.

**Applies when:**

The `ocean_monuments` set selects `minecraft:monument`, its surrounding-biome precheck and generic
start-biome gate pass, its sole building piece intersects a processing chunk, a monument start
reloads, or natural spawning queries the retained full-box overrides.

**Authoritative state:**

The record uses `surface_structures`, default `none` terrain adaptation, and
`#has_structure/ocean_monument -> #is_deep_ocean`: deep frozen, deep cold, deep ordinary and deep
lukewarm ocean. Its full-box overrides replace the monster list with guardian weight `1` count
`2..4` and replace both axolotl and underground-water-creature lists with empty lists. The one-entry
set has monument weight `1`; triangular random spread uses spacing `32`, separation `5`, salt
`10387313` and default frequency/locate offset. The required-surrounding tag is `#is_ocean` plus
`#is_river`: those four deep oceans, frozen/ocean/cold/lukewarm/warm ocean, river and frozen river.

**Transition and ordering:**

Before the generic biome predicate, source samples `BiomeSource.getBiomesWithin` around source-chunk
local `(9,seaLevel,9)` with radius `29` and rejects if any returned holder is outside the 11-biome
surrounding tag. Success returns an `OCEAN_FLOOR_WG` top-of-chunk-center stub at local X/Z `8` and
the first occupied height; that stub's 3-D noise biome must be one of the four start biomes.
Deferred construction places exactly one `omb` building. It consumes one uniform horizontal
direction, anchors its `58×23×58` box at source-chunk minimum X/Z minus `29` and fixed Y `39..61`,
and ignores the stub height for masonry.

Only the building is in the `PiecesContainer`. It persists generic ID/box/orientation/depth and no
extra fields; its room graph and child list are transient. Loading an empty container returns it
unchanged. Otherwise `StructureStart.loadStaticStart` discards every loaded child and rebuilds from
the first piece's box minimum and world seed/source chunk. A fresh legacy worldgen stream is reset
by `setLargeFeatureSeed(seed,chunkX,chunkZ)`; it consumes a default horizontal-direction draw even
when the first piece has a nonnull saved orientation, uses saved orientation when present, and
reconstructs the entire building. Thus canonical reload reproduces graph/design state from the
structure seed rather than NBT, while a null orientation adopts that already-drawn default.

**Room lattice and pruning:**

Room index is `y*25+z*5+x`. The ordinary lattice contains exactly 46 nodes: X `0..4`, Z `0..3` on Y
`0` and `1`, plus X `1..3`, Z `0..1` on Y `2`. Neighbor links are bidirectional; vertical and X
directions retain their names, while Z-neighbor construction stores the opposite direction, so a
room's logical north points toward increasing lattice Z. Source index `(2,0,0)` is marked source.
Three claimed special nodes are attached above `(2,2,0)` and logically south of `(0,1,0)`/`(4,1,0)`
for penthouse/left/right wing.

The first graph draw chooses core X uniformly `0..3` at Y `0`, Z `2`. It atomically claims that node
and its east, north, up, east+north, east+up, north+up and east+north+up neighbors: a `2×2×2` core.
All ordinary nodes snapshot their six openings, the roof special snapshots its link, and the 46
ordinary nodes begin in ascending room-index order. Their Fisher-Yates shuffle consumes
`nextInt(46),nextInt(45),...,nextInt(2)`, swapping the selected element with the current last
element each time. In shuffled order, each node makes at most five `nextInt(6)` closure attempts
until two closures have succeeded. A missing/already-closed direction wastes the attempt. A proposed
edge is closed at both ends; the current endpoint depth-first searches the live openings in 3-D slot
order down, up, north, south, west, east under the next scan ID, and only a successful first search
evaluates the other endpoint under the following scan ID. The closure commits only when both remain
source-connected; otherwise both bits restore. Special links cannot be stranded and the result is a
connected live opening graph.

After the source is claimed, children start with entry then core. The same shuffled node order skips
claimed/special nodes and selects the first fitting family in this exact order:

| Fitter | Required unclaimed live shape | Claimed cells / piece |
|---|---|---|
| double XY | east and up, with east-up also open/unclaimed | base, east, up, east-up / `omdxyr` |
| double YZ | north and up, with north-up also open/unclaimed | base, north, up, north-up / `omdyzr` |
| double Z | north open/unclaimed | base+north / `omdzr` |
| double X | east open/unclaimed | base+east / `omdxr` |
| double Y | up open/unclaimed | base+up / `omdyr` |
| simple top | no west/east/north/south/up opening | base / `omsimplet` |
| simple | unconditional | base / `omsimple` and one `nextInt(3)` design draw |

The double-Z factory contains a south-rebase fallback, but its canonical fitter precondition makes
that branch unreachable. Room boxes are `8×4×8` per lattice cell, transformed by orientation and
then moved by building-local `(9,0,22)`. Entry is `omentry`; the core is `omcr`. One full-width
`nextInt()` then gives the left `omwr` design bit and its incremented value gives the right design
bit, so the two wings always use opposite designs. Their boxes are building-local corners
`(1,1,1)..(23,8,21)` and `(34,1,1)..(56,8,21)`. `ompenthouse` uses `(22,13,22)..(35,17,35)`. Child
order is entry, core, fitted rooms in shuffled-node order, left wing, right wing, penthouse; this is
also overwrite and placement-RNG order.

**Write primitives and order:**

Let `G/L/D/S/W` denote prismarine, prismarine bricks, dark prismarine, sea lantern and source water.
All ranges below are inclusive local coordinates and rotate with the building/piece. Solid cuboids
use inherited Y-X-Z traversal, flags `2` writes, live-fluid reread/schedule at delay `0` and no
replaceability rejection; offers outside the processing box do nothing. `water(box)` also traverses
Y-X-Z, but its clipped read reports air outside the processing box. Live ice, packed ice, blue ice
and every water-block state are preserved. Every other state below sea level becomes W; every other
state at or above sea level becomes air. `fill-only(box,state)` replaces only cells whose clipped
live state is the canonical W state by identity.

The building first applies `water(x=0..58,y=0..max(seaLevel,64)-39,z=0..58)`, then left/right façade
wings, entrance arches/wall, roof, lower/middle/upper walls, foundations, a five-step outer water
skirt, and finally intersecting children. Building helper RNG parameters are unused. The main water
call deliberately reaches local X/Z `58`, one beyond the `0..57` box. For each pad origin
`(9px,9pz)` it writes a `4×4` L pad at Y `0` then direct L columns from Y `-1` downward while live
state is replaceable and world Y exceeds `minY+1`. Pads are all `pz=0..6` for `px=0|6`; `pz=0|6` for
`px=1,2,4,5`; and only `pz=6` for `px=3`. Direct supports require their Y=-1 start in the processing
box, then continue vertically unclipped and bypass fluid scheduling.

The outer skirt runs `i=0..4` in order: `water(x=-1-i,y=2i..23,z=-1-i..58+i)`, the symmetric X
`58+i` face, then Z `-1-i` and `58+i` faces over X `-i..57+i`. These recipes intentionally reach
local `-5..62` but remain processing-box clipped. Each shell subsection first performs its exact
horizontal `chunkIntersects` guard; child placement separately requires child-box intersection.

**Fixed shell recipes:**

For wing offset `o=0`/`33` and flip false/true, the subsection guard covers X `o..o+23`, Z `0..20`
even though its recipe reaches X `o+24`. It writes G floor `o..o+24,0,0..20` and water interior
`o..o+24,1..10,0..20`. For `i=0..3` write L ribs `(o+i,i+1,i..20)`, `(o+7+i,i+5,7+i..20)`,
`(o+17-i,i+5,7+i..20)`, `(o+24-i,i+1,i..20)`, front span X `o+1+i..o+23-i` at Y/Z `i+1/i`, and upper
span X `o+8+i..o+16-i` at Y/Z `i+5/7+i`. Add G bands `o+4..6,4,4..20`, `o+7..17,4,4..6`,
`o+18..20,4,4..20`, and `o+11..13,8,11..20`. Add L center cross X `o+11..13`, Y `1..7`, Z `12` plus
X `o+12`, Z `11..13`. L decoration points are `(o+12,9,12|15|18)`; one side at X `o+(flip?19:5)`, Y
`5`, Z `20..5` step `-3`; the other X `o+(flip?5:19)` at Z `19..7` step `-3` plus Z `5`; and four
front points derived from X `o+17-3i`, reflected when flipped.

Entrance arches are guarded by local X/Z `22..35/5..17`. They first water X `25..32`, Y `0..8`, Z
`0..20`. At each Z `5+4i`, `i=0..3`, symmetric L posts/arms occupy X `24,22..23,25,26` and
`33,34..35,32,31` at Y `2..6`, with S at X `26|31` Y `5` and a G beam X `27..30` Y `6`. The entrance
wall guard `15..42/20..21` writes G base X `15..42` Y `0` Z `21`, central water doorway X `26..31` Y
`1..3`, and symmetric G stepped bands: X `21..36` Y `12`, `17..40` Y `11`, `16..41` Y `10`, `15..42`
Y `7..9`, `16..41` Y `6`, `17..40` Y `5`, `21..36` Y `4`, then X `22..26|31..35` Y `3` and
`23..25|32..34` Y `2`. L trims occupy X `28..29` Y `4` Z `20..21` and the six single staircase
points down to X `25|32` Y `1`. D diagonals are `(28-i,6+i)`/`(29+i,6+i)` for `i=0..6`, a second
`i=0..3` pair from Y `9+i`, X `28|29` Y `12`, and paired side marks X `22-2i|35+2i` Y `8..9` for
`i=0..2`. The remaining wall void is water-cleared by the exact symmetric side staircase: full X
`15..42` Y `13..15`, then X `15/42` Y `1..6`, `16/41` Y `1..5`, `17..20/37..40` Y `1..4`, `21/36` Y
`1..3`, `22/35` Y `1..2` and `23..24/33..34` Y `1`.

The roof guard `21..36/21..36` writes G floor X `21..36` Y `0` Z `22..36` and water through Y `23`.
For `i=0..3`, L frames the square `21+i..36-i` at Y `13+i` and Z `21+i|36-i` plus X sides over Z
`22+i..35-i`. G caps X/Z `25..32` at Y `16`; L corner pillars are `(25|32,17..19,25|32)`. Four
symmetric finials use L at the outer Y `20` cell, L at the inner Y `21` cell and S below it; G
closes the four two-cell inner arms at Y `21`.

Lower, middle and upper rear/side shell bands retain their exact stepped symmetry. Lower left/right
guards fill G bases X `0..6`/`51..57`, Z `21..57`, water Y `1..7`, G inner shelves at X
`4..6`/`51..53` Y `4` Z `21..53`, four ascending L edge lines, L points every third Z `23..50` plus
`52`, and G crosses centered at `(5,52)`/`(52,52)`. The left side deliberately repeats its identical
four-line L pass after those points; the right does not. The rear lower guard fills G X `7..50` Y
`0` Z `51..57`, water Y `1..10` and four ascending L Z lines from `57` inward. Middle side guards
repeat at X `7..13`/`44..50`, water through Y `10`, G shelves Y `8` and four L lines beginning Y
`5`; L points mark the inner edge every third Z `21..45`. The middle rear is G X `14..43` Y `0` Z
`44..50`, water Y `1..10`, L at every third X `12..45` on Z `45|52` with the source-coded extra
stepped L motif for X `12,18,24,33,39,45`, three G lines X `8+i..49-i` at Y `5+i` Z `54`, an L line
X `11..46` Y `8` Z `54`, and G roof X `14..43` Y `8` Z `44..53`. Upper side guards use X
`14..20`/`37..43`, G base, water Y `1..14`, G shelf Y `12`, four L lines beginning Y `9`, and L
points every third Z `23..39`; the upper rear uses G X `21..36` Z `37..43`, water Y `1..14`, G shelf
Y `12` Z `37..39`, four L rear lines and L points every third X `21..36` at Y/Z `13/38`.

**Room floors, roofs and openings:**

Any ordinary room above lattice floor zero first writes its `8×8` G default floor. With a down
opening it leaves central X/Z `3..4` untouched, uses G on the remaining floor, and overwrites four
two-cell rim pairs with L. A missing room above causes a G interior roof through `fill-only` at
local Y `4`, or Y `8` for double-height/core pieces; these roofs are one block above the declared
room box and can write when clipped. Every live horizontal opening is a water portal two cells wide
and two high at the corresponding wall; double pieces map portals to each constituent room.

`omentry` is an `8×4×8` stepped L arch: roof strips X `0..2|5..7` Y `3`, side strips X `0..1|6..7` Y
`2`, outer columns X `0|7` Y `1`, full back wall Z `7` Y `1..3`, and front columns X `1..2|5..6` Y
`1..3`. Its north/west/east live openings water-clear the matching portals; the south face remains
the entrance.

`omcr` is `16×8×16`. It writes a fill-only G roof at Y `8`, an L perimeter at Y `7`, and perimeter
courses Y `1..6` that are L except G at Y `2|6`, with the source-coded X-side segments and full rear
line. Its center first becomes a solid D cube X/Z `6..9` Y `3..6`; eight gold blocks overwrite X/Z
`7..8` Y `4..5`, and S overwrites its eight X/Z corner cells at Y `3|6`. L posts, roof spokes and
four corner-foot motifs surround that sealed core exactly.

`omdxr` and `omdzr` are `16×4×8` and `8×4×16`. Both use L/G/L perimeter courses at Y `1/2/3`. Double
X adds a south-centered L/G/L dais over X `5..10`, Z `0..4` with S at `(6|9,2,3)`. Double Z adds L
end-corner pairs at X `1|6`, Z `1..2|13..14`, four L columns X `2|5` Y `1..3` Z `6|9`, L cross-links
X `3..4` at Z `6|9` and Z `7..8` at X `2|5`, S at X `2|5` Y `2` Z `5|10`, and L caps above those
lamps. Their six constituent-room horizontal openings water-clear independently.

`omdxyr` and `omdyzr` are `16×8×8` and `8×8×16` with perimeter courses Y `1..7`, G at Y `2|6` and L
otherwise. Double XY builds symmetric full-height inner buttresses at X `2..5` and `10..13`, a high
crossbeam X `5..10`, four S at X `5|10`, Y `4`, Z `2|5`, and exposes six bottom plus six upper
horizontal portals. Double YZ instead builds a central D divider X `3..4`, Z `7..8` through Y
`1..7`, replacing divider cells with S at Y `2|6`; its 12 portals are independent, and every upper
east/west portal also adds the exact adjacent L shelf and two vertical supports on that half.

`omdyr` is `8×8×8`. Its middle Y `4` ring is L with the eight source-coded corner notches. For the
lower definition at Y `1..3` and upper at Y `5..7`, each open side becomes two three-high L posts
with a two-cell top lintel; each closed side becomes a full three-high L wall whose middle course is
G.

`omsimple` stores no design NBT because it is transient. Design `0` makes four L/G/S corner shrines
and gives each open side only its top L lintel while a closed side receives L/G/L courses two cells
deep. Design `1` makes four internal three-high L posts at X/Z `2|5` with S centers, four accented L
corner elbows, and fills only closed sides with full L/G/L courses. Design `2` makes an L/D/L
perimeter with D portal accents and water-clears every live opening. For designs `1|2`, every
placement invocation consumes one `nextBoolean` before testing down/up/count gates; a true result
adds a central `2×3×2` L/G/L pillar only when neither vertical opening exists and total opening
count exceeds one. Design `0` consumes no Boolean. Because the child may be invoked for multiple
processing chunks, the caller placement stream and invocation order, not saved graph state, select
the clipped pillar offers.

`omsimplet` has an L/D/L perimeter and D portal accents; its fitter guarantees no canonical
horizontal/up opening, so its source-coded south water-portal branch is dead canonically. Before
walls, each of 36 interior X/Z cells always consumes `nextInt(3)`. Nonzero consumes `nextInt(4)` and
writes wet sponge from Y `2` when zero or Y `3` otherwise through Y `3`. The complete 36-cell draw
walk repeats per placement invocation regardless clipping, so chunk scheduling can splice
independently sampled sponge columns.

**Wings, penthouse and elders:**

In `omwr` design `0`, `i=0..3` makes descending rear L shelves X `10-i..12+i`, Y `3-i`, Z
`20-i..20`. L then forms floor X `7..15` Y `0` Z `6..16`, side walls X `6|16` Y `0..3` Z `6..20`,
inner ledges X `7|15` Y `1` Z `7..20`, front blocks X `7..9|13..15` Y `1..3` Z `6` and their inner
ledges, plus rear towers X `6..7|15..16` Y `0..4` Z `21`. D marks X `10..12` Z `7` and X `8|14` Z
`10..12` at Y `0`. S appears on the side walls at Y `3`, Z `18,15,12,9`, in the four X/Z `10|12`
floor cells and at X `8|14` Y/Z `3/6`. Four corner pedestals at X/Z `4|18` are L-S-L from Y `2` down
to `0`; rear L points are `(9|13,7,20)`. Its elder point is `(11,2,16)`.

Design `1` writes L rear cap X `9..13` Y `3` Z `18..20` and rear pillars X `9|13` Y `0..2` Z `18`; X
`9|13` at Z `20` have L-S-L at Y `6/5/4`. A raised L deck is X `7..15` Y `3` Z `7..14`. Four L posts
at X/Z `10|12` run Y `0..6`, with S at Y `0|4`. Short L edge posts stand at X `8|14`, Z `7|14`
through Y `2`, and D strips lie X `8|14` Y `3` Z `8..13`. Its elder point is `(11,5,13)`. Exactly
one wing has each design.

`ompenthouse` extends below its own box at local Y `-1`. L fills X/Z `2..11` there; G fills the four
surrounding two-wide strips X `0..13`/Z `0..13`. Its Y `0` L perimeter has S at coordinates
`2,5,8,11` on both X sides and the front Z side. L floor islands are X `2..4|9..11` Z `3..9` and X
`4..9` Z `9..11`, plus `(5,8)`, `(8,8)`, `(10,10)` and `(3,10)`; D marks X `3|10` Z `3..7` and X
`6..7` Z `10`. L posts rise Y `0..2` at X `3|10`, Z `2,5,8` and X `5|8`, Z `10`. A D `2×2` patch
sits X `6..7` Y `-1` Z `7..8`; `water` clears X `6..7` Y `-1` Z `3..4`. Its elder point is
`(6,1,6)`.

Each elder transaction has no latch. If its transformed point is in the processing box, it tries
`ELDER_GUARDIAN.create(level,STRUCTURE)`. A nonnull entity heals to maximum, snaps to centered X/Z
with exact Y and zero yaw/pitch, finalizes against current local difficulty with reason `STRUCTURE`
and null spawn data, then is offered with passengers; the add result is ignored. Reprocessing the
point can therefore offer another elder. The intended graph has exactly the two wing points and
penthouse point.

**Branches and aborts:**

Every surrounding holder; generic deep-ocean biome; four start/reload directions and null/non-null
saved orientation; empty/nonempty saved pieces; core X; every shuffle permutation and closure
direction/connectivity result; every fitter admission/claim; three simple designs, placement Boolean
and sponge draw; wing parity; sea level above/below `64`; subsection/child intersection;
full/partial/adversarial clip; keep-water/ice versus below-sea fill versus above-sea air; fill-only
identity; live write/fluid/support state; three elder clip/create/finalize/add outcomes.

**Constants and randomness:**

New construction draws direction; core X; the locked 46-node shuffle; up to five six-way closure
draws per node until two successes; one three-way draw per simple room in fitted order; and one full
integer for both wing designs. Placement consumes no graph RNG. It consumes only design-eligible
simple-room Booleans, 36 mandatory plus conditional sponge integers per simple-top invocation, and
downstream elder/world behavior. Reload resets a legacy stream to the large-feature seed, consumes a
default direction, then repeats the constructor sequence.

**Side effects:**

One persisted top box; a regenerated connected room graph; chunk-clipped
prismarine/bricks/dark-prismarine/lantern shell, flooded/air-cleared interior, direct foundations,
wet sponges and eight gold blocks; fluid ticks; up to three unlatching elder offers per processing
pass; full-box guardian/empty aquatic spawn overrides.

**Gates:**

Caller-owned set/start/reference/placement scheduling; 29-block surrounding-biome set and deep-ocean
stub biome; shuffled connectivity and claims; live room openings; processing/intersection boxes; sea
level and live block identity; placement RNG; entity creation/world admission; later natural-spawn
consumers.

**Boundary cases and quirks:**

The surrounding probe uses local `9` at sea level while the generic stub uses local `8` at
ocean-floor height. The actual shell ignores that height and stays at Y `39`. Only the top piece
saves; every load discards children and deterministically rebuilds. Roof writes sit above declared
room boxes, the penthouse writes below its box, and building water/skirt writes extend outside the
top box. Water and three ice blocks survive clearing. Simple-room pillars and sponge fields are
placement-time and repeat per intersecting chunk. Elder spawning has no saved latch.

**Evidence:**

`Confirmed`; `OFF-SERVER-001`; `OFF-DATA-001`; `OFF-REPORT-001`. Anchors:
`OceanMonumentStructure#findGenerationPoint`, `#createTopPiece`, `#generatePieces` and
`#regeneratePiecesAfterLoad`; `StructureStart#loadStaticStart`;
`OceanMonumentPieces$MonumentBuilding` constructor, room graph, shell helpers and `postProcess`; all
seven fitters, `RoomDefinition`, all 11 child classes and `OceanMonumentPiece` helpers;
`Structure#onTopOfChunkCenter`; monument record/set, four biome-tag files and all 12 structure-piece
registry entries.

**Test vectors:**

Replay exact large-feature streams across all directions, core X values, shuffle/closure attempts,
fitter claims/designs/wing parity and save-load rebuilding. Assert the complete 46-node opening
graph, all 12 IDs/boxes/order and every fixed shell/room coordinate under four orientations. Cross
sea level `63/64/65`, canonical/noncanonical water identity, all keep blocks, fill-only roofs,
support bottoms, outer-box and chunk edges, repeat placement RNG and all elder outcomes; assert
exact record/set/tags/spawn overrides. Use `EXP-WGEN-001` only for separately owned
random-spread/distribution calibration.
