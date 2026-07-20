# World mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `WGEN-STRUCTURE-RUINED-PORTAL-001` — Ruined portals choose one center-owned template transaction and grow an unclipped netherrack apron

**Parent:** `WGEN-003`

**FidelityClass:** `EquivalentPlayerVisibleBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked 26.2 server source fixes setup/template/orientation selection, six
vertical-placement policies, cold-property derivation, the complete processor chain and the
post-template netherrack, drip, vine and overgrowth passes. All seven structure records, the set, 13
NBT templates, biome/structure tags, protected-block tag and loot record are audited data-only.
Shared random-spread admission and generic start/reference lifecycle remain owned by
`WGEN-PIPELINE-001`.

**Applies when:**

The ruined-portals set selects one of its seven records, or the retained `minecraft:ruined_portal`
piece is offered to the placement chunk containing its rotated template-box center.

**Authoritative state:**

All seven records use `surface_structures`, default `none` terrain adaptation and no spawn
overrides. Their equal-weight set orders standard, desert, jungle, swamp, mountain, ocean and Nether
records, with spacing `40`, separation `15`, salt `34222645` and default random-spread fields;
`#minecraft:ruined_portal` contains the same records in desert, jungle, mountain, Nether, ocean,
standard, swamp order. The record setup table is exact:

| Record | Placement / weight | Air-pocket chance | Mossiness | Other properties |
|---|---|---:|---:|---|
| standard | underground `.5`; land surface `.5` | `1`; `.5` | `.2` | both can be cold |
| desert | partly buried `1` | `0` | `0` | — |
| jungle | land surface `1` | `.5` | `.8` | overgrown and vines |
| swamp | ocean floor `1` | `0` | `.5` | vines |
| mountain | in mountain `.5`; land surface `.5` | `1`; `.5` | `.2` | both can be cold |
| ocean | ocean floor `1` | `0` | `.8` | can be cold |
| Nether | in Nether `1` | `.5` | `0` | replace with blackstone |

The exact resolved biome scopes are: standard's 24 beach, snowy beach, river, frozen river, four
taigas, forest, flower/birch/old-growth-birch/dark forest, pale garden, grove, mushroom fields, ice
spikes, all three locked caves, savanna, snowy plains, plains and sunflower plains; desert alone;
all three jungles; mountain's three badlands, three windswept hill/forest/gravelly-hill records,
savanna plateau, windswept savanna, stony shore, meadow, three peaks, snowy slopes and cherry grove;
all five Nether biomes; all nine oceans; and swamp plus mangrove swamp.

**Transition and ordering:**

Start selection first handles the setup. A record with more than one setup sums weights, consumes
one float and subtracts each normalized weight in list order until the remainder is strictly
negative; the two locked pairs therefore split at `.5`. A one-setup record consumes no
setup-selection draw. Air-pocket sampling consumes no draw at exact probability `0` or `1`; only a
fractional probability consumes a float and selects strictly below it. The next float selects a
giant template strictly below `.05`, followed by uniform `nextInt(3)` over `giant_portal_1..3`;
otherwise `nextInt(10)` selects `portal_1..10`. Rotation is a uniform `nextInt(4)` over enum order,
then a float chooses mirror none below `.5` or front-back otherwise. The pivot is
`(templateX/2,0,templateZ/2)`, and the unshifted rotated/mirrored box is computed at the source
chunk's world minimum with Y zero.

The source samples the box center from generator `OCEAN_FLOOR_WG` only for ocean-floor placement and
`WORLD_SURFACE_WG` otherwise, subtracting one. Let `floor=level.minY+15` and `span=templateY`.
Initial projected Y is: that sampled surface for land/ocean-floor; surface minus span plus an
inclusive `2..8` draw for partly buried; an inclusive `70..surface-span` draw for mountain only when
`70<surface-span`, otherwise exactly `surface-span` without a draw; an inclusive
`floor..surface-span` draw underground only when `floor<surface-span`, otherwise exactly
`surface-span` without a draw; and in the Nether, inclusive `32..100` when air-pocketed, otherwise a
float split between inclusive `27..29` and `29..100`. The shared height search obtains generator
columns at rotated-box corners `(minX,minZ)`, `(maxX,minZ)`, `(minX,maxZ)`, `(maxX,maxZ)`. From
projected Y downward while strictly above `floor`, it tests the relevant heightmap's opacity
predicate in that order and stops immediately on the third opaque corner; fewer than three
decrements Y. `floor` itself is returned without being tested. The resulting template position is
`(chunkMinX,Y,chunkMinZ)`.

After the generic stub biome gate accepts that 3-D origin, `cold` is true only when the setup
permits it and the generator biome at the origin's quart coordinates reports
`coldEnoughToSnow(origin,seaLevel)`. The piece persists template identity/position/base box,
rotation, mirror, vertical placement and the six derived properties; required missing rotation,
mirror, placement or properties fail decode rather than defaulting.

**Locked template audit:**

All inputs use one palette, contain one chest whose NBT is exactly `id=minecraft:chest` plus
`LootTable=minecraft:chests/ruined_portal` and no seed, and contain zero entities, structure
markers, jigsaws or structure voids. Block totals include template air; positions omitted from the
block list are untouched.

| Template | Size | Blocks / palette states | Chest | Gold / lava / obsidian / air |
|---|---:|---:|---:|---:|
| `portal_1` | `6×10×6` | `304 / 18` | `(2,2,0)` | `2 / 0 / 11 / 198` |
| `portal_2` | `9×12×9` | `750 / 19` | `(8,2,6)` | `2 / 26 / 11 / 544` |
| `portal_3` | `8×9×9` | `554 / 8` | `(3,3,6)` | `0 / 2 / 11 / 355` |
| `portal_4` | `8×9×9` | `500 / 9` | `(3,3,2)` | `1 / 3 / 11 / 302` |
| `portal_5` | `10×10×7` | `601 / 14` | `(4,3,2)` | `3 / 1 / 15 / 387` |
| `portal_6` | `5×7×7` | `212 / 8` | `(1,1,4)` | `1 / 0 / 16 / 147` |
| `portal_7` | `9×7×9` | `510 / 9` | `(0,1,2)` | `2 / 21 / 12 / 377` |
| `portal_8` | `14×9×9` | `1,054 / 13` | `(4,4,2)` | `3 / 26 / 17 / 836` |
| `portal_9` | `10×8×9` | `640 / 12` | `(4,1,0)` | `2 / 0 / 12 / 549` |
| `portal_10` | `12×8×10` | `880 / 12` | `(2,1,7)` | `2 / 19 / 13 / 700` |
| `giant_portal_1` | `11×17×16` | `2,400 / 21` | `(4,3,3)` | `2 / 11 / 31 / 1,949` |
| `giant_portal_2` | `11×16×16` | `2,266 / 22` | `(9,1,9)` | `2 / 19 / 29 / 1,847` |
| `giant_portal_3` | `16×16×16` | `3,433 / 18` | `(9,2,3)` | `6 / 33 / 25 / 2,919` |

**Center-owned placement:**

On every referenced-chunk invocation, the piece recomputes its transformed template box. It returns
immediately unless the supplied processing box contains that box's integer center. The unique owner
then expands the processing box to encapsulate the entire template box before generic placement, so
the template is one full-box transaction rather than ordinary per-chunk clipping; its later apron
and decoration passes have no processing-box test. Settings use the saved transform/pivot and the
following processor order, default entity/liquid/shape behavior, and no explicit settings RNG.
Air-pocket true ignores only structure blocks and therefore places template air; false ignores
structure blocks and air. Because the locked templates have no structure blocks, air clearing is the
only locked difference.

The ordered rule processor uses a fresh RNG seeded from each transformed world position. It first
changes gold to air on a strict `.3` float gate; then changes lava to magma unconditionally on ocean
floor, to netherrack unconditionally when cold, or to magma on a strict `.2` gate otherwise;
finally, only when noncold, it changes netherrack to magma on a strict `.07` gate. Tests
short-circuit on block mismatch and the first matching rule wins, so only the relevant random block
test consumes that position-local stream.

Block aging independently recreates the settings position RNG rather than continuing the rule
stream. Stone, stone bricks and chiseled stone bricks first pass a strict `.5` gate; on success the
source eagerly generates random facing/half for both an ordinary and mossy stair candidate, then
uses mossiness to choose the candidate pair and a uniform index to choose cracked-or-stair versus
mossy-full-or-stair. Any tagged stair first passes `.5`, then mossiness chooses between two-item
ordinary slab and mossy stair/slab arrays; slabs and walls change to property-preserving mossy
stone-brick variants strictly below mossiness. Obsidian becomes crying obsidian strictly below
`.15`. Other states pass unchanged.

Next, a target whose live prewrite state is one of bedrock, spawner, chest, End portal frame,
reinforced deepslate, trial spawner or vault is discarded. Otherwise, if the live state is lava and
the processed state's shape is not a full block, the processor restores default lava while retaining
NBT. Nether records finally map cobblestone/mossy cobblestone to blackstone; stone to polished
blackstone; stone bricks/mossy stone bricks to polished blackstone bricks; cobble/mossy-cobble
stairs to blackstone stairs, stone stairs to polished-blackstone stairs and
stone-brick/mossy-stone-brick stairs to polished-blackstone-brick stairs; cobble/mossy-cobble slabs
to blackstone slabs, smooth-stone/stone slabs to polished-blackstone slabs and
stone-brick/mossy-stone-brick slabs to polished-blackstone-brick slabs; cobble/mossy-cobble walls to
blackstone walls and stone-brick/mossy-stone-brick walls to polished-blackstone-brick walls;
chiseled/cracked stone bricks to chiseled/cracked polished-blackstone variants; and iron bars to
chain. Only facing, stair half and slab type are copied where present.

Generic template placement then transforms states, writes NBT containers through the
barrier/actual-state transaction, applies default liquid and neighbor-shape repair, and places no
locked entities. A successfully written resulting `RandomizableContainer` consumes one caller
`nextLong`, injects it as `LootTableSeed`, and loads the template's ruined-portal table. There is no
second marker pass because the family marker handler is empty and the templates contain none.
Processor randomness is position-seeded; only that successful typed-container path advances the
caller stream during the template transaction.

**Apron, drips and decoration:**

After generic placement returns—regardless of its internal boolean result—the owner spreads around
the transformed box center. It computes `averageWidth=(xSpan+zSpan)/2`, consumes
`nextInt(max(1,8-averageWidth/2))` as a nonnegative distance adjustment, then visits X outer and Z
inner over the center ±14 square. For Manhattan distance plus adjustment indices `0..13`, it
consumes one double and admits it strictly below `[1,1,1,1,1,1,1,.9,.9,.8,.7,.6,.4,.2]`.
Land-surface and ocean-floor setups use the current matching heightmap minus one; other placements
use the lower of that surface and box minimum Y. Candidates more than three Y from the box minimum,
air, obsidian, a protected-tag block, or non-Nether lava are rejected.

Every admitted candidate is offered magma on a noncold strict `.07` float gate or netherrack
otherwise, with ignored write result. Overgrown setups then consume a `.5` leaf gate before testing
for resulting netherrack with air above and offering persistent jungle leaves. Each admitted
candidate unconditionally starts a drip one below: it offers one netherrack/magma cell without any
replaceability, protection, air, lava, height or write-result gate, then makes at most eight
downward continuations while successive floats are below `.5`; eight successes consume no ninth
terminal gate. After the entire spread, every interior X/Z cell of the box minimum layer that
currently is netherrack starts another unrestricted drip below in X-then-Z order.

If vines or overgrowth is enabled, the source finally scans the full transformed box with X fastest,
Y next and Z slowest. For vines it skips air and existing vines without a draw; otherwise one
uniform horizontal-direction draw precedes the neighbor tests, and an air neighbor receives a
one-face vine only when the source collision shape is full on that face. For overgrowth, after the
vine attempt at that cell, a `.5` float gate precedes the same netherrack/air-above leaf test.
Jungle spread cells inside the box can consequently receive this second independent leaf
opportunity; spread leaves outside the box are not revisited. All vine, leaf and drip writes use
flags `3` and ignore failure.

**Locked loot record:**

`chests/ruined_portal` is a chest table with the matching random-sequence ID. Its first pool makes
uniform `4..8` rolls over obsidian/flint/iron nugget/flint-and-steel/fire-charge at weight `40`
(counts `1..2`, `1..4`, `9..18`, one, one); golden apple, gold nuggets `4..24`, and randomly
enchanted golden sword, axe, hoe, shovel, pickaxe, boots, chestplate, helmet and leggings at weight
`15`; glistering melon `4..12`, golden horse armor, light weighted pressure plate, golden carrots
`4..12`, clock and gold ingots `2..8` at weight `5`; and bell, enchanted golden apple and gold
blocks `1..2` at default weight `1`. Its second one-roll pool chooses default-weight empty against
weight-2 lodestone count `1..2`. Loot evaluation itself is deferred and owned by `ITM-LOOT-001`;
this leaf owns exact table installation and data linkage.

**Branches and aborts:**

Seven records; one/two setup selection and every weight boundary; air probability `0`, fractional or
`1`; giant/ordinary endpoints; four rotations/two mirrors; all six vertical placements and
interval-collapse cases; every corner-opacity sequence; valid/invalid stub biome and cold result;
center/noncenter chunk; air-pocket ignore mode; every processor match/gate/output;
protected/live-lava target; template write/container outcome; every apron
distance/probability/surface/admission/state/drip length; vine direction/support and both leaf
opportunities.

**Constants and randomness:**

Start draws are conditional in the exact order setup, air pocket, giant gate, template index,
rotation, mirror, then placement-specific height draws. Processor and age decisions are independent
transformed-position streams. Caller placement RNG supplies a seed only for a successfully written
typed chest, then every apron/direction/leaf/drip decision in the fixed traversal above; cold
suppresses every `.07` netherrack-versus-magma state draw. Shape repair may use level RNG inside
block-specific neighbor updates.

**Side effects:**

One persisted transformed template piece; optional air clearing; aged, protected, lava-aware and
optionally blackstone-replaced template states; exactly one attempted loot chest; a radius-13
probability apron with unrestricted downward drips; optional persistent jungle leaves and one-face
vines. The unique center-containing chunk owns cross-chunk writes for the entire transaction.

**Gates:**

Caller-owned placement/start/reference lifecycle; record biome and setup; live generator
columns/heightmaps; cold biome; center-containing processing box; processor/live target/write/entity
interfaces; apron live surface/state/protection/distance; vine collision and leaf air.

**Boundary cases and quirks:**

Exact setup probabilities `0` and `1` do not draw. Both collapsed interval helpers return the upper
endpoint, which may be below the preferred minimum. Height search accepts three of four corners and
never tests `minY+15`. The owner expands the chunk box before placement and does not clip the apron.
Position-local rule and age streams restart independently at the same transformed position.
Protected blocks stop the template processor but not later unrestricted drips. A failed netherrack
write can still be followed by a drip and leaf read. Cold suppresses magma in processors and
apron/drips, while ocean-floor lava always becomes magma even when cold.

**Evidence:**

`Confirmed`; `OFF-SERVER-001`; `OFF-DATA-001`; `OFF-REPORT-001`. Anchors:
`net.minecraft.world.level.levelgen.structure.structures.RuinedPortalStructure#findGenerationPoint`,
`net.minecraft.world.level.levelgen.structure.structures.RuinedPortalStructure#findSuitableY`,
`net.minecraft.world.level.levelgen.structure.structures.RuinedPortalStructure#sample`,
`net.minecraft.world.level.levelgen.structure.structures.RuinedPortalStructure#isCold`,
`net.minecraft.world.level.levelgen.structure.structures.RuinedPortalPiece#makeSettings`,
`net.minecraft.world.level.levelgen.structure.structures.RuinedPortalPiece#postProcess`,
`net.minecraft.world.level.levelgen.structure.structures.RuinedPortalPiece#spreadNetherrack`,
`net.minecraft.world.level.levelgen.structure.structures.RuinedPortalPiece#addNetherrackDripColumn`,
`net.minecraft.world.level.levelgen.structure.structures.RuinedPortalPiece#maybeAddVines`,
`net.minecraft.world.level.levelgen.structure.structures.RuinedPortalPiece#maybeAddLeavesAbove`, all
six processor classes named above, generic template/container placement, all 13
`data/minecraft/structure/ruined_portal/*.nbt` inputs, seven structure records and biome tags, the
set/structure/protected-block tags, and `data/minecraft/loot_table/chests/ruined_portal.json`.

**Test vectors:**

Cross every setup/probability endpoint, 13 template choices, transform, pivot box, height interval
collapse and four-corner opacity trace; assert the full template table and exact
structure/setup/tag/loot decodes. Replay every position-seeded processor gate/output and
independence boundary against protected/lava targets, air modes and typed/wrong/missing chest
entities. Vary center chunk, hostile processing boxes, live surfaces, all apron indices/admission
failures, cold state-draw suppression, drip lengths `1..9`, vine faces and both leaf passes; trace
the complete caller RNG and use `EXP-WGEN-001` only for separately owned placement/distribution
calibration.
