# World mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `WGEN-STRUCTURE-WOODLAND-MANSION-001` — Woodland mansions tile a randomized floor graph into saved templates

**Parent:** `WGEN-003`

**FidelityClass:** `EquivalentPlayerVisibleBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked 26.2 source fixes rotation-aware terrain admission, the complete 11×11
base/two-partition/optional-third-floor graph, room IDs/flags and door quirks,
perimeter/roof/corridor/room transform tables, all 73 locked template inputs, template
overwrite/persistence, chest/illager/allay markers, fixed block entities, the woodland-mansion loot
record and the post-placement cobblestone foundation. The mansion record, triangular set and exact
two-biome tag are audited data-only. Shared random-spread admission and start/reference/placement
scheduling remain with `WGEN-PIPELINE-001`; generic loot evaluation and later entity/block-entity
lifecycle retain their subsystem owners.

**Applies when:**

The `woodland_mansions` set selects `minecraft:mansion`, its rotation-aware height gate and generic
biome gate pass, a saved `wmp` template piece intersects a processing chunk, one of its transformed
data markers is handled, or the structure finishes placement in that chunk.

**Authoritative state:**

The record uses `surface_structures`, default `none` terrain adaptation, no spawn override, and
exactly dark forest plus pale garden through `#has_structure/woodland_mansion`. The one-entry
triangular set has mansion weight `1`, spacing `80`, separation `20`, salt `10387319` and default
frequency/locate offset. All generated pieces use sole registry ID `wmp`.

**Transition and ordering:**

Generation first consumes one of four rotations. Start X/Z is source-chunk local `(7,7)`. Let the
rotation-selected signed corner offset `(dx,dz)` be none `(+5,+5)`, clockwise 90 `(-5,+5)`,
clockwise 180 `(-5,-5)` or counterclockwise 90 `(+5,-5)`. The four `WORLD_SURFACE_WG` samples are
`(x,z)`, `(x,z+dz)`, `(x+dx,z)` and `(x+dx,z+dz)`. Start Y is their minimum; `59` rejects and `60`
admits. The generic 3-D biome predicate then samples the start point and requires one of the two
record biomes. Deferred generation uses that point as template origin and continues the same
structure stream.

The piece list is appended in this order: entrance; complete first-floor flat-wall perimeter;
second-floor window-wall perimeter; optional third-floor window-wall perimeter; base/third roof
passes; then floors `0,1,2`, each with all corridor/carpet pieces in row-major grid order followed
by interior wall/door and room pieces in row-major order. No collision filter removes overlaps, so
this list is also the later overwrite and marker-RNG order.

**Base grid:**

`SimpleGrid` is `11×11`, returns blocked value `5` outside, and silently ignores out-of-range
writes. House values are corridor `1`, room `2`, start `3` and test `4`; clear is `0`. Fixed seeds
are start X/Y `7/4`: start rectangle X `7..8`, Y `4..5`; room X `6`, Y `4..5`; blocked X `9..10`, Y
`2..7`; corridor X `8`, Y `2..3|6..7` and `(6,3|6)`; blocked rows Y `0..1` and `9..10`. Four
recursive corridors start at `(7,2,W,6)`, `(7,7,W,6)`, `(5,3,W,3)` and `(5,6,W,3)`.

Each positive-depth corridor marks its cell and conditionally the next heading cell. It makes up to
eight attempts, each consuming `nextInt(4)`. Opposite heading rejects immediately. An east choice
that is not already opposite additionally consumes `nextBoolean` and rejects when true. Otherwise
the two cells one heading step then one/two candidate-direction steps must both be clear; success
recurses from heading+candidate at depth minus one and ends the attempt loop. Only after
recursion/attempt exhaustion, seven clear neighbors—clockwise, counterclockwise, both heading
diagonals, two forward and two to each side—become rooms.

Edge cleaning repeatedly scans Y then X and mutates live cells: a clear cell with at least three
direct house neighbors becomes room; one with exactly two direct neighbors becomes room only when at
most one diagonal neighbor is house. Passes repeat until none changes. The resulting footprint is
independently partitioned twice for floors zero and one. Each partition collects room cells
row-major, Fisher-Yates shuffles them, then assigns IDs from `10` upward. An unassigned cell claims
the first available shape in exact priority: 2×2 east+south, west+south, west+north; 1×2 east,
south, west, north; otherwise 1×1. There is deliberately no east+north 2×2 candidate.

Each room consumes two Booleans to select an X and Y endpoint as tentative door/origin. If it does
not edge a corridor, the search checks the diagonally opposite endpoint, then the
X-opposite/Y-original endpoint, then redundantly the original endpoint; the fourth rectangle corner
is never tested. Failure removes the door flag but still marks X0/Y0 as origin. Every cell receives
size flag `0x10000/0x20000/0x40000` and 16-bit ID; only origin receives `0x100000` and, on success,
`0x200000`. Both floor partitions then overwrite start cells X `8`, Y `4..5` with corridor flag
`0x800000` alone.

**Third floor:**

Candidates are floor-one origin cells that are door-bearing 1×2 rooms. Empty candidates block the
entire third grid. Otherwise one candidate is uniform, its floor-one origin gains stairs flag
`0x400000`, and the first same-ID horizontal neighbor defines the other stair cell. Third-grid
nonhouse base cells become blocked, these two cells become start `3`, and the neighbor receives
floor-two corridor flag `0x800000`. Clear horizontal cells adjacent to that neighbor are collected
in horizontal iteration order. If none exists, the third grid is blocked and the floor-one stairs
bit rolls back; the already-written floor-two corridor bit remains stale but unreachable. Otherwise
one direction is uniform, a depth-four recursive corridor starts there, edges clean, and remaining
third-floor rooms are partitioned by the same algorithm.

**Perimeters and roofs:**

Entrance template position is origin shifted rotated west `9`; the perimeter cursor then moves
rotated south `16`. Base and second perimeters start at grid `(8,5)` heading south and finish only
upon `(8,4,south)`. On an outside boundary, `wall_corner` turns the cursor clockwise; on a concave
occupied diagonal, the cursor takes the source-coded inner turn and rotates counterclockwise;
otherwise `wall_flat`/`wall_window` advances one eight-block segment. The third perimeter finds the
first house cell by ascending Y and descending X, offsets from the same grid anchor, emits its
initial wall segment, and traverses back to that cell/direction.

The lower roof is based at origin Y+`16` over base cells not covered by third-floor house; the upper
roof is at Y+`27` over every third-floor house. Each exposed cell emits `roof` at Y+3 and
`roof_front` on each absent cardinal neighbor with the exact rotated six/seven-block offsets.
Covered lower cells emit `small_wall` on exposed base edges and `small_wall_corner` at exposed
convex corners. A final pass emits `roof_corner` where both adjacent sides are absent and
`roof_inner_corner` where the diagonal is occupied behind an absent side. All decisions use
out-of-grid blocked cells as nonhouse.

**Corridors, boundaries and room choice:**

Floor origins are Y offsets `0,8,19`; floors zero/one use the base footprint and floor two uses the
third grid. Every corridor cell emits `corridor_floor`. North/east/south/west carpet pieces are
added when the neighbor is corridor or carries corridor flag; south/west use suffix `1` only on
floor zero and `2` above.

For each room/start cell, a door-bearing origin collects adjacent corridor directions and, when
nonempty, consumes one uniform direction; otherwise any origin becomes secret direction UP. Boundary
pieces use `indoors_wall_1/door_1` on floor zero and suffix `2` above, placing a door only on the
chosen side and suppressing the source-coded east/north boundaries at the flagged third-floor stair
endpoint. A 1×1 room first consumes its ordinary selector even for a secret; north/west/south rotate
it counterclockwise 90/180/clockwise 90 from east, while null/UP consumes a second secret selector.

The room-template families and draws are exact:

#### first

**1×1 / secret:**

`1x1_a1..a5` / `1x1_as1..as4`

**1×2 side / front / secret:**

`1x2_a1..a9` / `1x2_b1..b5` / `1x2_s1..s2`

**2×2 / secret:**

`2x2_a1..a4` / fixed `2x2_s1`

#### second and third

**1×1 / secret:**

`1x1_b1..b5` / `1x1_as1..as4`

**1×2 side / front / secret:**

`1x2_c1..c4` or fixed `c_stairs` / `1x2_d1..d5` or fixed `d_stairs` / `1x2_se1` via a still-consumed
`nextInt(1)`

**2×2 / secret:**

`2x2_b1..b5` / fixed `2x2_s1`

For 1×2, eight perpendicular door/long-axis combinations select side-entrance placement, four
opposing combinations select front entrance, and only secret-UP with long axis east or south selects
a secret piece; secret-UP west/north silently emits no room. The 12 nonsecret cases use the exact
source offsets, rotations and left-right/front-back mirrors. For 2×2, the eight valid
door/clockwise-room-side combinations map to offsets (east,south)
`(-7,0),(-7,6),(1,14),(7,14),(7,-8),(1,-8),(15,6),(15,0)` with the corresponding
none/left-right/front-back mirror and quarter-turn; UP uses fixed secret at east `1`. Template
transforms use the unrotated room grid offsets followed by mansion rotation.

**Template and persistence transaction:**

The locked corpus contains exactly 73 templates: 52 room templates above and 21
entrance/wall/roof/corridor/carpet templates. Every input has one palette, no template entities and
no structure-void cells. Room sizes are `7×8×7` or `7×11×7`; `7×8×15`, `7×11×15` or the two
`7×22×15` stairs; and `15×8×15` or `15×11×15`. Structural sizes are locked from `2×1×5` carpet
strips through `21×19×16` entrance. Explicit air is placed; only structure blocks are ignored,
entities are disabled, and there is no jigsaw input.

Each `wmp` has depth zero and saves generic box/orientation plus template position/name and
legacy-coded `Rot`/`Mi`. Its generic orientation is north regardless template rotation. Loading
resolves `woodland_mansion/<name>`, restores rotation/mirror, and recomputes the box. Placement sets
the processing box, applies mirror/rotation with flags `2` and
`BlockIgnoreProcessor.STRUCTURE_BLOCK`, and handles transformed in-box DATA markers only after a
successful template placement. Single-palette choice uses a fresh position-seeded settings stream,
not the graph/caller stream.

**Locked markers and fixed NBT:**

The 73 inputs contain 38 DATA markers: ten loot chests—west in `1x1_a4`, `1x1_as1`, `1x2_d2`; south
in `1x2_a1`, `1x2_b3`, `1x2_s1`, `1x2_se1`, `2x2_b5`; and north in `1x2_b4` plus the second
`1x2_se1` marker—20 Warrior vindicators, four Mage evokers and four Group-of-Allays markers, all
latter in `2x2_a1`. Chest facing rotates the named cardinal direction but deliberately ignores piece
mirror. A nonexisting live chest is offered, and only a resulting chest block entity consumes
`nextLong` and receives `chests/woodland_mansion`; no latch prevents a later retry.

Mage/Warrior creates one evoker/vindicator. Each allay marker consumes one world-RNG `nextInt(3)+1`
and creates that many allays. Null creations are skipped. Every nonnull mob is made persistent,
snapped at the marker with zero yaw/pitch, finalized for current local difficulty with
STRUCTURE/null data, offered with passengers, then the marker position is set to air with flags `2`
regardless add success. Unknown markers return without clearing.

Fixed template block entities are independent of marker loot: `1x1_as2` has a spider spawner at
`(4,1,3)` with delay `0`, delays `200..800`, count `4`, nearby `6`, player range `16` and range `4`;
`1x1_b5` has eight alliums in fixed slots; `1x2_a4` has `9/10/9` dark-oak saplings; `1x2_a6` has one
Efficiency-I iron axe; `1x2_a7` has one empty chest; `1x2_a9` has 42 empty decorative chests;
`1x2_s2` has a trapped chest with two ender pearls; `2x2_a2` has 26 empty decorative chests.
`1x2_a1` has one plain gray banner, `1x2_d1` three white-flower light-gray banners, and `1x2_d3` two
plain black banners. These decode as fixed NBT and never receive the marker loot table.

`chests/woodland_mansion` uses its matching random sequence. Pool one rolls `1..3`: lead w20; golden
apple w15; enchanted golden apple w2; discs 13/cat w15; chainmail chestplate w10; diamond hoe w15;
diamond chestplate w5; book w10 with one random enchantment from `#on_random_loot`. Pool two rolls
`1..4`: iron `1..4` w10, gold `1..4` w5, bread w20, wheat `1..4` w20, bucket w10, redstone/coal
`1..4` w15, melon/pumpkin/beetroot seed `2..4` w10, and resin clump `2..4` w50. Pool three rolls
exactly three among equal-weight bone, gunpowder, rotten flesh and string, each `1..8`. Pool four
rolls once between equal-weight empty and one vex trim template. Evaluation stays with
`ITM-LOOT-001`.

**Foundation after-place:**

After all intersecting pieces place, the structure computes their union and its minimum Y. It scans
the supplied chunk box X then Z. A column qualifies only when the live block at union-min-Y is
nonempty, inside the union and inside at least one individual piece box. Starting one below, air or
liquid is directly replaced by cobblestone with flags `2` while Y is strictly above level minY; the
first nonempty nonliquid cell stops that column. Writes may continue through liquid to minY+1,
ignore set results and bypass generic fluid scheduling.

**Branches and aborts:**

Four rotations and height `59/60`; both start biomes; every recursive-corridor direction/East
Boolean/occupancy/depth; every cleaning mutation/pass; both room shuffles, shape priorities, door
Booleans/search failure; no/candidate/success/rollback third floor; wall convex/concave/straight
paths; every roof neighborhood; corridor/carpet flags; zero/one/multiple door directions; every room
family/transform/secret no-piece path; full/partial/failed template placement;
known/unknown/chest/mob markers, null entities and allay counts; fixed/marker chest distinction;
every foundation seed/air/liquid/solid/minY outcome.

**Constants and randomness:**

Draw order is rotation and terrain gate, four base corridor recursions, floor-zero shuffle/door
pairs, floor-one shuffle/door pairs, third-floor candidate/direction/corridor/shuffle/door pairs,
then placement traversal door-direction and room-selector draws. Template palettes use private
position seeds. Placement caller RNG is consumed only by successful typed marker chests; allay
counts use world RNG and entity finalization owns downstream state.

**Side effects:**

A saved ordered `wmp` template list; explicit-air and block/NBT placement; marker loot chests,
persistent evokers/vindicators/allays and marker-air clears; fixed spawner/chests/banners;
post-placement cobblestone foundations; loot seeds and block/entity lifecycle effects.

**Gates:**

Caller-owned set/start/reference/placement scheduling; rotation-specific terrain and generic biome;
graph/room flags; template availability/data and processing box; live write/block-entity/entity
state; world/caller RNG; foundation live blocks and piece boxes.

**Boundary cases and quirks:**

Door search repeats its initial corner and never checks one corner. Floor zero and one repartition
the same footprint independently. Third-floor rollback leaves an unreachable corridor flag. A secret
1×1 wastes its ordinary selector; `nextInt(1)` is still consumed for upper secret 1×2. Secret 1×2
west/north emits no piece. Chest mirror does not mirror facing. Mob markers clear only after nonnull
creation and have no latch. Decorative fixed chests are not loot chests. Foundations write through
liquids but only below a nonempty piece-contained seed at the union floor.

**Evidence:**

`Confirmed`; `OFF-SERVER-001`; `OFF-DATA-001`; `OFF-REPORT-001`. Anchors:
`WoodlandMansionStructure#findGenerationPoint`, `#generatePieces` and `#afterPlace`;
`Structure#getLowestYIn5by5BoxOffset7Blocks`; `WoodlandMansionPieces#generateMansion`;
`MansionGrid`, `SimpleGrid`, `MansionPiecePlacer` and all room collections; `WoodlandMansionPiece`
construction/settings/persistence/marker handler; `TemplateStructurePiece#postProcess`; all 73
`structure/woodland_mansion/*.nbt` inputs, mansion record/set/tag, `chests/woodland_mansion.json`
and `wmp` registry entry.

**Test vectors:**

Replay fixed streams through all four height footprints, recursive attempts, cleaning passes, both
floor partitions, every room-shape/door corner, all third-floor outcomes, perimeter/roof
neighborhoods and complete room transform table. Assert all 73 sizes, block lists, explicit air,
markers, fixed NBT, rotations/mirrors/boxes/save-load and overlap order. Cross full/partial/rejected
writes, all marker entity/chest outcomes, fixed versus loot chests, exact loot decode and foundation
union/piece/live-state/minY boundaries. Use `EXP-WGEN-001` only for separately owned triangular
placement/distribution calibration.
