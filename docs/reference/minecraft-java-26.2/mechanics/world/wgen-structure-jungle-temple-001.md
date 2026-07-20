# World mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `WGEN-STRUCTURE-JUNGLE-TEMPLE-001` — Jungle temples randomize masonry and persist two traps and two chests

**Parent:** `WGEN-003`

**FidelityClass:** `EquivalentPlayerVisibleBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked 26.2 server source fixes the four-corner start gate, one-piece geometry,
terrain average, masonry selector, ordered air/state writes, two tripwire-dispenser traps,
lever-piston mechanism and four container latches; the structure, set and two loot inputs are
audited data-only. Shared random-spread admission and generic start/reference lifecycle remain owned
by `WGEN-PIPELINE-001`.

**Applies when:**

The `minecraft:jungle_temple` type has passed caller-owned structure-set placement or its retained
`minecraft:jungle_pyramid` piece intersects a placement chunk. The family has no custom `afterPlace`
transaction.

**Authoritative state:**

The `structure/jungle_pyramid` record selects exactly `minecraft:jungle` and
`minecraft:bamboo_jungle`, uses `surface_structures`, default `none` terrain adaptation and no spawn
overrides. `structure_set/jungle_temples` contains only `minecraft:jungle_pyramid` at weight `1`,
with random-spread spacing `32`, separation `8`, salt `14357619`, and default frequency fields. The
piece persists its bounding box, orientation, width `12`, height `10`, depth `15`, cached `HPos`,
and independent `placedTrap1`, `placedTrap2`, `placedMainChest` and `placedHiddenChest` booleans.
Missing booleans decode false and missing `HPos` decodes cached `0`.

**Transition and ordering:**

Before structure RNG, the single-piece start gate samples `WORLD_SURFACE_WG` first-occupied heights
at `(chunkMinX,chunkMinZ)`, `(chunkMinX,chunkMinZ+15)`, `(chunkMinX+12,chunkMinZ)`, and
`(chunkMinX+12,chunkMinZ+15)`. The `+12` and `+15` probes lie one cell beyond the corresponding
unrotated `0..11` and `0..14` footprint axes. A minimum below generator sea level rejects the start.
Otherwise the generic top-of-chunk-center stub samples local `(8,8)` and applies the exact 3-D
biome-holder gate. Deferred generation anchors one piece at `(chunkMinX,64,chunkMinZ)` and consumes
`nextInt(4)` over north, east, south and west. North/south make a `12×15` horizontal box and
east/west a `15×12` box, always within the source chunk; the nominal box is Y `64..73`, although
later local writes extend four cells below it.

When `HPos<0`, `postProcess` scans its current world bounding box in Z-major then X order at probe Y
`64`, retaining only positions inside the processing box. It samples `MOTION_BLOCKING_NO_LEAVES`,
stores the truncating integer mean, and moves the box so minimum Y equals that mean. No retained
column aborts before any layout RNG or write. A nonnegative result is cached; a negative mean moves
the box but remains sentinel-negative and is resampled on a later invocation. Ordinary placement
retains all 180 columns and invokes the piece only once because its footprint never crosses the
source chunk. A hostile partial box can instead cache a subset mean or repeatedly resample a
negative one.

**Masonry and open volume:**

Every selector box iterates Y outermost, X middle and Z innermost, never skips air, and consumes one
caller-RNG float before the processing-box test for each offered local cell. The selector ignores
coordinates and edge status: `<0.4` chooses cobblestone and every other value chooses mossy
cobblestone. Its exact ordered boxes are:

#### Foundation/lower shell

**Selector-backed inclusive boxes:**

`(0,-4,0)..(11,0,14)`; `(2,1,2)..(9,2,2)` and Z `12`; side strips `(2,1,3)..(2,2,11)` and X `9`

#### Upper shell/floors

**Selector-backed inclusive boxes:**

`(1,3,1)..(10,6,1)` and Z `13`; `(1,3,2)..(1,6,12)` and X `10`; `(2,3,2)..(9,3,12)` and the same at
Y `6`; `(3,7,3)..(8,7,11)`; `(4,8,4)..(7,8,10)`

#### Facade/roof accents

**Selector-backed inclusive boxes:**

At Z `0,14`, X `2,4,7,9`, vertical Y `4..5`; `(5,6,0)..(6,6,0)`; at X `0,11`, every even Z `2..12`,
vertical Y `4..5`, plus `(x,6,5)` and `(x,6,9)`; columns X `2,9`, Z `2,12`, Y `7..9`; roof points X
`4,7`, Z `4,10`, Y `9`, and `(5,9,7)..(6,9,7)`

#### Interior stairs/landing

**Selector-backed inclusive boxes:**

Points `(4,1,9),(7,1,9)`; `(4,1,10)..(7,2,10)`; `(5,4,5)..(6,4,5)`

#### Basement reinforcement

**Selector-backed inclusive boxes:**

At X `1`, every odd Z `1..13`, Y `-3..-2`; at X `1..3`, every even Z `2..12`, Y `-1`;
`(2,-2,1)..(5,-2,1)`, `(7,-2,1)..(9,-2,1)`, and points `(6,-3,1),(6,-1,1)`; later
`(9,-1,1)..(9,-1,5)`; hidden-mechanism floor strips `(8,-3,8)..(8,-3,10)` and X `10`

Those calls make exactly `1,522` selector offers and floats. Interleaved air boxes make another
`703` fixed offers: `(3,1,3)..(8,2,11)`, `(4,3,6)..(7,3,9)`, `(2,4,2)..(9,5,12)`,
`(4,6,5)..(7,6,9)`, `(5,7,6)..(6,7,8)`, openings `(5,1,2)..(6,2,2)`, `(5,2,12)..(6,2,12)`,
`(5,5,1)..(6,5,1)`, `(5,5,13)..(6,5,13)`, and points `(1,5,5),(10,5,5),(1,5,9),(10,5,9)`; four
staircase-clearance boxes `(5,-i,7+i)..(6,-i,9+i)` for `i=0..3`; basement corridors
`(1,-3,12)..(10,-1,13)`, `(1,-3,1)..(3,-1,13)`, `(1,-3,1)..(9,-1,5)`; and hidden room
`(8,-3,8)..(10,-1,10)`.

Before the basement mechanisms, 24 cobblestone-stair points are offered with the named local facings
before piece transformation: north at `(5,9,6),(6,9,6)`, south at `(5,9,8),(6,9,8)`; north across X
`4..7`, Y `0`, Z `0`; north at X `4,7`, with `(y,z)=(1,8),(2,9),(3,10)`; east `(4,4,5)`, west
`(7,4,5)`; then for `i=0..3`, south at X `5,6`, Y `-i`, Z `6+i`. Those stairs and all mechanism
states below make exactly `73` unconditional non-air direct-state offers; the four earlier direct
air points are already included in the 703-air count. Selector, air and other direct calls therefore
total `2,298` fixed offers before up to four conditional containers.

**Tripwire traps:**

The first line has attached east/west-facing tripwire hooks at `(1,-3,8)` and `(4,-3,8)` with
attached east-west tripwire at X `2,3`. North-south redstone wire runs X `5`, Y `-3`, Z `7..2`,
turns north-west at `(5,-3,1)` and east-west at `(4,-3,1)`, and ends beside fixed mossy cobblestone
`(3,-3,1)`. If `placedTrap1` is false, a north-facing dispenser is attempted at `(3,-2,1)` with
`minecraft:chests/jungle_temple_dispenser`; a south-attached vine at `(3,-2,2)` hides it.

The second line has attached north/south-facing hooks at `(7,-3,1)` and `(7,-3,5)`, with attached
north-south tripwire at Z `2..4`. East-west wire `(8,-3,6)` turns west-south at `(9,-3,6)` and
becomes north-side/south-up wire at `(9,-3,5)`; fixed mossy cobblestone `(9,-3,4)` supports
north-south wire at `(9,-2,4)`. If `placedTrap2` is false, a west-facing dispenser is attempted at
`(9,-2,3)` with the same dispenser table. East-attached vines at `(8,-1,3),(8,-2,3)` conceal it. The
main chest attempt follows at `(8,-3,3)` with `minecraft:chests/jungle_temple`.

Nine fixed mossy cells then shape the corridor:
`(9,-3,2),(8,-3,1),(4,-3,5),(5,-2,5),(5,-1,5),(6,-3,5),(7,-2,5),(7,-1,5),(8,-3,5)`. The hidden room
has three chiseled stone bricks at X `8..10`, Y `-2`, Z `11`, and three north-facing wall levers
immediately behind at Z `12`; their omitted powered field is false. Selector floor strips stand at X
`8,10`, Y `-3`, Z `8..10`; fixed mossy cobblestone `(10,-2,9)` supports the circuit. North-south
wire lies at `(8,-2,9),(8,-2,10)`, and a four-sided wire at `(10,-1,9)` feeds an up-facing sticky
piston `(9,-2,8)`, west-facing sticky pistons `(10,-2,8),(10,-1,8)`, and a north-facing
default-delay repeater `(10,-2,10)`. All begin at their default unpowered/unextended state. The
hidden chest is finally attempted at `(9,-3,10)` with the main table.

Each trap/chest flag is assigned the corresponding helper result. A helper outside the processing
box or finding the same container block returns false, leaving the flag retryable. Otherwise a
dispenser goes through the piece transform while a chest is world-reoriented from neighboring
solids; flags `2` are offered and the helper returns true even after a rejected write or
wrong/missing block entity. Only a resulting typed dispenser/chest entity consumes `nextLong` and
receives its table/seed. Thus ordinary successful placement consumes those seed draws in trap-one,
trap-two, main-chest, hidden-chest order, but adversarial entity outcomes remove individual draws
without preventing the latch.

The main table uses random sequence `minecraft:chests/jungle_temple`. Pool one rolls uniformly
`2..6` over diamond weight `3` count `1..3`, iron ingot `10` count `1..5`, gold ingot `15` count
`2..7`, bamboo `15` count `1..3`, emerald `2` count `1..3`, bone `20` count `4..6`, rotten flesh
`16` count `3..7`, leather `3` count `1..5`, four horse armors and an enchant-with-levels-`30`
`#minecraft:on_random_loot` book each at default weight `1`; pool two selects empty weight `2` or
two wild armor-trim templates weight `1`. The dispenser table uses random sequence
`minecraft:chests/jungle_temple_dispenser` and uniformly `1..2` rolls of its sole arrow entry, count
`2..7` (declared weight `30`).

All direct, air, selector and dispenser offers compute the transformed world position,
processing-box test it, then mirror/rotate the state, use flags `2`, and ignore the write result. A
resulting nonempty fluid receives a delay-`0` tick even after rejection. None of this piece's
offered blocks belongs to the generic structure shape-check set. Chest writes use the separate
world-position/reorientation helper just described.

**Branches and aborts:**

Four-corner minimum below/equal/above sea level; center-stub biome accept/reject; four orientations;
uncached/cached, missing and negative `HPos`; zero/full/partial height probes; each of 1,522
selector thresholds; every selector/air/direct/container cell outside/inside clip and
rejected/accepted write; postwrite empty/nonempty fluid; each persisted latch, same/different
existing container and typed/wrong/missing resulting entity.

**Constants and randomness:**

Start generation consumes only direction `nextInt(4)`. After successful terrain alignment, layout
RNG consumes exactly 1,522 floats in selector-call order regardless of clip, then one `nextLong` for
each attempted container that yields its typed block entity, interleaved trap one, trap two, main
chest and hidden chest. No level RNG, support-column RNG or family-specific postpass RNG exists.
Loot evaluation remains deferred to table random-sequence/seed ownership.

**Side effects:**

One retained/cached single-chunk piece; 2,298 unconditional fixed offers and up to four conditional
container offers; 40/60 randomized masonry; complete tripwire/redstone/dispenser and lever/piston
circuits; two dispenser and two chest latches/loot seeds; fluid ticks and persisted height/flags.

**Gates:**

Caller-owned placement/start/reference lifecycle; four live start heights and sea level; center-stub
biome; processing box at terrain probes and writes; existing/resulting container state and type.

**Boundary cases and quirks:**

The start gate probes one cell beyond both nominal axes, while the rotated piece remains wholly in
one chunk. The foundation/basement extends below the nominal piece box. Selector draws happen before
clipping and ignore edge status. Negative mean height remains the uncached sentinel. Same-type
existing containers keep their flags false, while attempted replacements latch even without a typed
entity; only the latter entity gate controls seed consumption. The temple name differs across
registries: structure type `jungle_temple`, structure/piece `jungle_pyramid`, set `jungle_temples`.

**Evidence:**

`Confirmed`; `OFF-SERVER-001`; `OFF-DATA-001`; `OFF-REPORT-001`. Anchors:
`net.minecraft.world.level.levelgen.structure.SinglePieceStructure#findGenerationPoint`,
`net.minecraft.world.level.levelgen.structure.structures.JungleTempleStructure#type`,
`net.minecraft.world.level.levelgen.structure.structures.JungleTemplePiece#postProcess`,
`net.minecraft.world.level.levelgen.structure.structures.JungleTemplePiece$MossStoneSelector#next`,
`net.minecraft.world.level.levelgen.structure.ScatteredFeaturePiece#updateAverageGroundHeight`,
`net.minecraft.world.level.levelgen.structure.StructurePiece#generateBox`,
`net.minecraft.world.level.levelgen.structure.StructurePiece#generateAirBox`,
`net.minecraft.world.level.levelgen.structure.StructurePiece#placeBlock`,
`net.minecraft.world.level.levelgen.structure.StructurePiece#createDispenser`,
`net.minecraft.world.level.levelgen.structure.StructurePiece#createChest`,
`net.minecraft.world.RandomizableContainer#setLootTable`,
`data/minecraft/worldgen/structure/jungle_pyramid.json`,
`data/minecraft/worldgen/structure_set/jungle_temples.json`,
`data/minecraft/tags/worldgen/biome/has_structure/jungle_temple.json`,
`data/minecraft/loot_table/chests/jungle_temple.json`, and
`data/minecraft/loot_table/chests/jungle_temple_dispenser.json`.

**Test vectors:**

Cross all start probes, sea-level edge, center biome, orientations, full/empty/adversarial partial
boxes, height truncation and cached/negative/missing replay. Assert all 1,522 selector
offers/floats, 703 air offers, 73 other direct offers, exact overlaps/transforms/states/fluids and
every circuit connection. Exhaust four flags against clip, existing block, failed write,
typed/wrong/missing entity, conditional seed order and both loot-table decodes; use `EXP-WGEN-001`
only for separately owned distribution/locate equivalence.
