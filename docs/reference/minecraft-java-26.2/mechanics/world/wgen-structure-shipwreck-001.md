# World mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `WGEN-STRUCTURE-SHIPWRECK-001` — Shipwrecks defer live height, choose one of eight palettes, and seed marker chests twice

**Parent:** `WGEN-003`

**FidelityClass:** `EquivalentPlayerVisibleBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked 26.2 server source fixes ocean/beached start stubs, rotation and template
selection, the X/Y-only size gate, live height adjustment, template processing and marker-to-loot
transaction; all 20 NBT inputs, eight palettes per input, both structure records, the set and three
loot tables are audited data-only. Shared random-spread admission and generic start/reference
lifecycle remain owned by `WGEN-PIPELINE-001`.

**Applies when:**

The equal-weight shipwreck set selects `minecraft:shipwreck` or `minecraft:shipwreck_beached`, or a
retained `minecraft:shipwreck` template piece intersects a placement chunk.

**Authoritative state:**

Both records use `surface_structures`, default `none` terrain adaptation and no spawn overrides.
Ocean shipwrecks select `#is_ocean`: deep frozen/cold/ordinary/lukewarm ocean plus frozen, ordinary,
cold, lukewarm and warm ocean. Beached shipwrecks select beach and snowy beach. Their shared set
contains the two records at weight `1` each, with random-spread spacing `24`, separation `4`, salt
`165745295`, and default frequency fields; `#minecraft:shipwreck` contains both records. A piece
persists template name/position, base box, rotation, `isBeached`, and `height_adjusted`; missing
booleans decode false. Placement settings use mirror none, pivot `(4,0,15)`, the saved rotation,
default waterlogging/entity/shape settings, and the structure-and-air ignore processor.

**Transition and ordering:**

The start stub samples local `(8,8)` from `WORLD_SURFACE_WG` when beached and `OCEAN_FLOOR_WG`
otherwise, then applies that record's exact 3-D biome gate. Deferred generation first consumes a
uniform rotation draw over none, clockwise `90`, clockwise `180`, and counterclockwise `90`; it
anchors at `(chunkMinX,90,chunkMinZ)`, then consumes `nextInt(11)` for a beached template or
`nextInt(20)` for an ocean template and adds one piece. Beached choice order is mast, sideways
full/front/back, right-side-up full/front/back, degraded mast, then degraded right-side-up
full/front/back. Ocean order is mast; upside-down full/front/back; sideways full/front/back;
right-side-up full/front/back; then the degraded counterpart of each in that same shape order. Every
entry is distinct and uniformly weighted.

After addition, the source calls `isTooBigToFitInWorldGenRegion`: true exactly when raw template X
is greater than `32` **or raw Y is greater than `32`**. Z is not tested. When true, start generation
samples generator `WORLD_SURFACE_WG` at four positions derived from the pivot-rotated box:
`(minX,minZ)`, `(minX,minZ+zSpan)`, `(minX+xSpan,minZ)`, `(minX+xSpan,minZ+zSpan)`. Because span is
the inclusive box size, the latter probes lie one cell beyond its maximum. Beached takes their
minimum, subtracts integer `templateY/2` and `nextInt(3)`; ocean takes their truncating mean. It
writes the chosen template-position Y and sets `height_adjusted=true`.

When that flag is false and the same X/Y-only test remains false, the first intersecting
`postProcess` invocation owns height instead. It samples the unrotated rectangle from template
position through `+(sizeX-1,0,sizeZ-1)`, regardless of rotation, pivot or current processing box. A
positive base area uses every cell to compute both minimum and truncating mean from
`WORLD_SURFACE_WG` if beached or `OCEAN_FLOOR_WG` otherwise. Ocean Y is the mean; beached Y is
minimum minus integer `sizeY/2` minus caller-RNG `nextInt(3)`. If either horizontal size is zero,
ocean instead samples one height at template position, while beached retains initialized minimum
`level.maxY+1` and still subtracts half-height and its draw. Adjustment sets the flag and
template-position Y but does not otherwise consume RNG. Later chunks skip all height work. A missing
persisted flag can therefore re-anchor a loaded piece from its saved X/Z and current terrain.

**Locked template audit:**

Every official input has X `9`, Y `9` or `21`, and Z `16..28`, so all 20 take the deferred branch;
the blind Z edge is not triggered. Every input contains eight palettes, no entities, no jigsaw
blocks, one chest block below every data marker, and no palette air state. Block counts below
include the marker blocks that the processor later removes:

| Template pair | Size | Blocks regular / degraded | Ordered data markers |
|---|---:|---:|---|
| `with_mast` | `9×21×28` | `729 / 652` | supply `(4,3,9)`, map `(5,3,18)`, treasure `(6,5,24)` |
| `rightsideup_full` | `9×9×28` | `662 / 619` | supply `(4,3,8)`, map `(5,3,18)`, treasure `(6,5,24)` |
| `rightsideup_fronthalf` | `9×9×24` | `355 / 299` | supply `(4,3,8)` |
| `rightsideup_backhalf` | `9×9×16` | `414 / 385` | map `(5,3,6)`, treasure `(6,5,12)` |
| `sideways_full` | `9×9×28` | `641 / 583` | treasure `(3,3,24)`, supply `(5,4,8)`, map `(6,4,19)` |
| `sideways_fronthalf` | `9×9×24` | `321 / 246` | supply `(5,4,8)` |
| `sideways_backhalf` | `9×9×17` | `381 / 340` | treasure `(3,3,13)`, map `(6,4,8)` |
| `upsidedown_full` | `9×9×28` | `601 / 546` | treasure `(2,3,24)`, map `(3,6,17)`, supply `(4,6,8)` |
| `upsidedown_fronthalf` | `9×9×22` | `332 / 300` | map `(3,6,17)`, supply `(4,6,8)` |
| `upsidedown_backhalf` | `9×9×16` | `387 / 362` | treasure `(2,3,12)`, map `(3,6,5)` |

All degraded partners retain the same size and marker/chest positions as their regular partner. The
eight palette indices are also positionally compatible across every input. In order they use: oak
hull with spruce accents; jungle with spruce; dark oak with jungle; dark oak with spruce; spruce
hull with oak accents; spruce with jungle; spruce with dark oak; and oak hull with birch accents.
“Hull” supplies that palette's logs, doors and trapdoors as present; accents supply the secondary
planks/fences/slabs/stairs. Chest and structure-marker states are invariant.

**Template placement:**

Before each intersecting-chunk transaction, the current processing box is installed and the rotated
bounding box is recomputed from the adjusted position. Because settings contain no explicit RNG,
each palette lookup creates a fresh RNG from the adjusted template position and consumes
`nextInt(8)`; placement and later marker filtering therefore independently reproduce the same
palette in every chunk. Rotation is around `(4,0,15)`: none and `180` retain the 9-wide axis with
different Z offsets, while quarter-turns exchange the long axis and can extend west of the
source-chunk origin.

The structure-and-air processor removes structure markers and air before writes. Since it does not
request whole-piece state, block processing is clipped to the current chunk before processor
application. Each retained state is pivot-transformed, then mirrored/rotated. An NBT-bearing chest
first receives a barrier with flags `820`; its actual state is offered with flags `2`. A rejected
state write ends that cell, and a successfully written barrier can therefore remain when the
following chest write fails. A successful chest write obtains the resulting block entity; if it
implements `RandomizableContainer`, the generic template loader consumes caller `nextLong`, adds
that value as `LootTableSeed` to the copied NBT, and loads the empty `Items` payload. Successful
writes are tracked for bounding extents, neighbor-shape repair and neighbor notification. Default
liquid handling preserves prior source fluid in waterloggable containers, locks newly placed
source-fluid states, and repeatedly fills eligible retained positions from an unlocked adjacent
source in up/north/east/south/west search order. All locked templates have zero entities, so the
generic entity branch is inert.

Placement returns true for these nonempty, positive-size templates even if a hostile clip retained
no processed block. The piece then filters structure blocks using the same position-seeded palette
and current processing box. Only `DATA` markers are handled. `map_chest`, `supply_chest`, and
`treasure_chest` target the world position one block below the marker and map to the corresponding
shipwreck table; every other marker is a no-op. The helper does not re-test the below position
against the processing box. Only a block entity implementing `RandomizableContainer` consumes caller
`nextLong` and installs the table plus that final seed. In normal placement, each marker's audited
chest lies directly below it, so a successfully written typed chest advances once during generic NBT
load and again during marker initialization; the marker seed is the final observable loot seed.
Within a chunk, all successful NBT-container draws occur in processed block order before marker
draws occur in filtered marker order. Chunk visitation therefore affects the global caller-RNG trace
without changing position-seeded palette identity.

**Locked loot records:**

All three are chest tables with matching random-sequence IDs. `shipwreck_map` has four pools: one
buried-treasure exploration map with red-X decoration, zoom `1`, `skip_existing_chunks=false`, and
translated item name; exactly three rolls among compass/map/clock at weight `1`, paper weight `20`
count `1..10`, feather `10` count `1..5`, and book `5` count `1..5`; empty weight `5` versus two
coast templates weight `1`; then empty/copper/iron/golden/diamond nautilus armor weights
`148/20/10/5/2`, armor count `1`.

`shipwreck_supply` rolls uniformly `3..10` over paper `8` count `1..12`, potato `7` count `2..6`,
moss block `7` count `1..4`, poisonous potato `7` count `2..6`, carrot `7` count `4..8`, wheat `7`
count `8..21`, suspicious stew `10`, coal `6` count `2..8`, rotten flesh `5` count `5..24`, pumpkin
`2` count `1..3`, bamboo `2` count `1..3`, gunpowder `3` count `1..5`, TNT weight `1` count `1..2`,
and randomly enchanted leather helmet/chestplate/leggings/boots weight `3` each. Stew uniformly
selects one declared effect: night vision or jump boost duration `7..10`, weakness `6..8`, blindness
`5..7`, poison `10..20`, or saturation `7..10`. Its second pool is the same coast-template choice
and its third the same nautilus-armor choice.

`shipwreck_treasure` rolls uniformly `3..6` over iron ingot weight `90`, gold ingot `10`, emerald
`40`, each count `1..5`, or diamond/experience bottle weight `5` each; then uniformly `2..5` rolls
over iron nugget weight `50`, gold nugget `10`, lapis lazuli `20`, each count `1..10`; its third and
fourth pools are the same coast-template and nautilus-armor choices. Loot evaluation is deferred
until container unpack and is owned generically by `ITM-LOOT-001`; this leaf owns table selection
and exact data linkage.

**Branches and aborts:**

Ocean/beached record and stub biome; four rotations; every 11/20 template endpoint; raw X/Y
below/equal/above `32` with arbitrary Z; start-adjusted/deferred/persisted/missing flag;
zero/nonzero base area; every sampled height and beached offset; all eight palettes; current-chunk
clip empty/nonempty; processor retain/drop; state write fail/succeed; NBT absent/present and
resulting entity wrong/typed; previous fluid none/source/flowing and placed state
source/container/other; shape changes; template empty/nonempty and size zero/positive; marker
mode/ID and below-marker entity wrong/typed.

**Constants and randomness:**

Start consumes rotation `nextInt(4)` then template `nextInt(11|20)`. Only beached height adjustment
consumes `nextInt(3)`, during start for X/Y-oversized inputs and otherwise during the first
placement invocation. Each palette lookup uses a fresh position-seeded `nextInt(8)`, not caller RNG.
Caller placement RNG supplies one `nextLong` per successfully written typed NBT container and one
per typed marker target; all official templates have one to three pairs. Shape repair may use level
RNG inside block-specific neighbor updates.

**Side effects:**

One retained template piece; persisted adjusted position/flag; position-seeded selection of one of
eight palettes; chunk-clipped structure-and-air-ignored block placement, barrier/NBT loading,
waterlogging and shape/neighbor updates; one to three chests with map/supply/treasure tables and
final seeds; no locked template entities or family-specific spawn pass.

**Gates:**

Caller-owned placement/start/reference lifecycle; record-specific stub biome/heightmap; raw template
X/Y size; first placement order; live heightmaps; current processing box; processor and state-write
result; resulting block-entity interfaces; marker mode/ID.

**Boundary cases and quirks:**

The size gate ignores Z, and every locked long-Z template is deferred. Deferred height sampling
ignores rotation, pivot and clip, while the actual placement box uses all three. Beached zero-area
inputs use `maxY+1`, not their separately sampled mean. The first intersecting chunk latches height
and supplies the beached offset draw. Marker lookup is clipped at the marker but acts one block
below without another clip test. A normal chest consumes a generic seed that is immediately
superseded by a second marker seed; failed writes or hostile pre-existing typed entities change that
draw trace. Palette selection changes with final Y because the seed uses the adjusted full position.

**Evidence:**

`Confirmed`; `OFF-SERVER-001`; `OFF-DATA-001`; `OFF-REPORT-001`. Anchors:
`net.minecraft.world.level.levelgen.structure.structures.ShipwreckStructure#findGenerationPoint`,
`net.minecraft.world.level.levelgen.structure.structures.ShipwreckStructure#generatePieces`,
`net.minecraft.world.level.levelgen.structure.structures.ShipwreckPieces#addRandomPiece`,
`net.minecraft.world.level.levelgen.structure.structures.ShipwreckPieces$ShipwreckPiece#postProcess`,
`net.minecraft.world.level.levelgen.structure.structures.ShipwreckPieces$ShipwreckPiece#isTooBigToFitInWorldGenRegion`,
`net.minecraft.world.level.levelgen.structure.structures.ShipwreckPieces$ShipwreckPiece#handleDataMarker`,
`net.minecraft.world.level.levelgen.structure.TemplateStructurePiece#postProcess`,
`net.minecraft.world.level.levelgen.structure.templatesystem.StructureTemplate#placeInWorld`,
`net.minecraft.world.level.levelgen.structure.templatesystem.StructureTemplate#filterBlocks`,
`net.minecraft.world.level.levelgen.structure.templatesystem.StructurePlaceSettings#getRandomPalette`,
`net.minecraft.world.level.levelgen.structure.templatesystem.BlockIgnoreProcessor#processBlock`,
`net.minecraft.world.RandomizableContainer#setBlockEntityLootTable`, all 20
`data/minecraft/structure/shipwreck/*.nbt` inputs, both structure records, the set, biome/structure
tags, and all three `data/minecraft/loot_table/chests/shipwreck_*.json` inputs.

**Test vectors:**

Cross record, biome, stub/live heightmaps, rotations, all choice endpoints and pivot boxes; assert
the complete template table, eight palette mappings, zero entities and every marker/chest
coordinate. Exercise raw X/Y/Z `0,1,32,33`, first-chunk orders, height/offset extrema, position-seed
changes and all clip/processor/barrier/write/NBT/fluid/shape branches. Trace generic and marker seed
order for typed/wrong/missing entities, every marker mode/ID and all three exact loot-table decodes;
use `EXP-WGEN-001` only for separately owned distribution/locate equivalence.
