# World mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `WGEN-JIGSAW-TRAIL-RUINS-001` — Trail ruins age ordinary templates before globally capped archaeology

**Parent:** `WGEN-003`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — the seven locked trail-ruins pools, three archaeology processor lists, gravel
replaceable tag, 84 present NBT templates and two archaeology loot records fix the complete payload
supplied to generic jigsaw/template placement. Every locator exists, is reachable and occurs exactly
once. The raw corpus has no entities or structure blocks; ordinary-single air, connector
replacement, whole-piece capped processing and brushable-block-entity NBT are specified here.
Village payloads are owned by `WGEN-JIGSAW-VILLAGES-001`.

**Applies when:**

`trail_ruins` expands from `trail_ruins/tower`, selects any building, grouped-building, decor, road,
tower-addition or tower-top element, replaces its jigsaws, ages gravel/mud bricks, or converts
eligible gravel to common/rare suspicious gravel. Record-owned burial, start height and terrain
adaptation remain with `WGEN-JIGSAW-RECORDS-001`.

**Authoritative state:**

Record/core inputs; seven ordered rigid single-element pools; 84 sparse single-palette templates and
their connector/final-state/NBT fields; the three ordered processor lists and exact
`trail_ruins_replaceable` tag; transformed origin, rotation, world seed, chunk clip and live
block/fluid/block-entity state; both archaeology loot records.

**Transition and ordering:**

All seven pools fall back to `minecraft:empty`; all 84 top-level entries have weight one, rigid
projection and an ordinary `single_pool_element`. There are no Empty, list, feature or legacy-single
entries. `buildings` contains 15 `group_hall`, `large_room` and `one_room` templates numbered
`1..5`; `buildings/grouped` contains 20 `group_full`, `group_lower`, `group_upper` and `group_room`
templates numbered `1..5`; `decor` contains `decor_1..7`; `roads` contains `long_road_end`,
`road_end_1`, `road_section_1..4` and `road_spacer_1`; `tower` contains `tower_1..5`;
`tower/additions` contains `hall`, `large_hall`, `one_room`, `platform` and `stable` additions
numbered `1..5`; and `tower/tower_top` contains `tower_top_1..5`. The first 72 elements select
`trail_ruins_houses_archaeology`, roads select `trail_ruins_roads_archaeology`, and tower tops
select `trail_ruins_tower_top_archaeology`. All 84 distinct locators exist; there are no missing or
unreferenced trail-ruins templates.

**Template and processor ordering:**

Ordinary-single settings begin with structure-block ignore, then jigsaw replacement, the selected
processor list and rigid projection behavior. Unlike outpost legacy singles, ordinary air is not
ignored. Jigsaw replacement therefore turns 154 connectors into their final states before
archaeology: air `28`, cobblestone `28`, dirt `70`, gravel `5`, mud bricks `19` and red terracotta
`4`. The five connector-final gravel cells join the archaeology candidate population, the 19
connector-final mud-brick cells can age, and connector-final air is a real overwrite. Together with
218 raw air cells, up to 246 processed air cells are written subject to the final clip/write gates;
absent coordinates remain untouched.

Houses and roads first apply one first-match rule processor. Gravel becomes dirt when a
position-derived float is below `0.2`; only after that miss, a fresh conditional draw below `0.1`
makes coarse dirt, giving effective probabilities `20%`, `8%` and `72%` unchanged. Mud bricks
independently become packed mud below `0.1`. Tower tops skip aging. The replaceable tag expands to
exactly gravel. Houses then run a common capped replacement with limit `6` followed by rare limit
`3`; roads run common limit `2`; tower tops run common limit `2`. Each successful delegate writes
dusted-zero suspicious gravel and appends its named loot table plus a position-derived
`LootTableSeed`.

The presence of a capped processor disables the initial current-box filtering for the entire
processor chain. Each chunk invocation transforms and processes the whole template, then clips only
the final write loop. A cap seeds from world seed plus the adjusted template origin, shuffles all
current indices, and scans until its limit of actual unequal replacements or list exhaustion. Aging
remains keyed to each transformed world position. For houses, common outputs are no longer gravel
when the rare pass begins; the second cap restarts the origin-keyed shuffle and skips those cells,
so its successful rare set is disjoint and follows the remaining eligible order. Limits are maxima,
not guarantees: aging, earlier replacement and list exhaustion can leave fewer candidates.
Rotation/origin and out-of-chunk cells can therefore change which in-chunk gravel becomes suspicious
even though the caller structure RNG is unused.

**Locked payload census:**

All 84 templates have one palette and no duplicate coordinates. Their combined bounding volume is
40,148 cells: 21,528 encoded sparse cells and 18,620 absent coordinates. They contain 218 raw air
cells, 154 jigsaws, eight other block-NBT cells, no structure void, structure block or data marker,
and no raw entities. They use 49 block IDs and 116 exact states:

| Template directory | Templates | Volume | Encoded | Absent | Raw air | Jigsaws | Other NBT | Gravel |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| `buildings` (including `grouped`) | 35 | 25,029 | 10,451 | 14,578 | 49 | 55 | 5 | 5,054 |
| `decor` | 7 | 610 | 360 | 250 | 0 | 7 | 0 | 132 |
| `roads` | 7 | 1,050 | 821 | 229 | 0 | 31 | 0 | 419 |
| `tower` (including additions/top) | 35 | 13,459 | 9,896 | 3,563 | 169 | 61 | 3 | 4,381 |

The dominant raw counts are gravel `9,986`, mud bricks `3,870`, bricks `2,526`, white terracotta
`768`, cobblestone `656`, blue/cyan/red/orange/light-gray/brown/gray/yellow terracotta
`539/438/335/256/254/172/104/48`, stone bricks `88`, stone `85`, cracked stone bricks `72`, packed
mud `68`, and coarse dirt `46`. Exact decorative and workstation states account for the remaining
cells and are data-only payloads to the generic transaction.

**Connector payload:**

Every selection and placement priority is zero. Ten connectors are aligned and 144 rollable. Pool
fields are empty `5`, buildings `12`, grouped buildings `35`, decor `7`, roads `44`, tower `20`,
additions `26` and tower top `5`. The exact name/target vocabulary is
building/decor/room/road/hall/large-hall/tower anchors and connectors plus empty; attachment still
requires opposite facing and exact target/name equality through `WGEN-JIGSAW-CORE-001`. Final-state
distribution and processor interaction are fixed above.

**Fixed block NBT:**

`buildings/large_room_1` has one empty, unlit west-facing blast furnace and one empty, unlit
north-facing furnace; `large_room_2` has one empty west-facing unlit/non-signal/non-waterlogged
campfire; `large_room_5` has south- and north-facing empty unlit blast furnaces. `tower/platform_5`
has three east-facing empty unlit/non-signal/non-waterlogged campfires. Furnace timing fields and
recipe maps are zero/empty; every campfire has an empty item list and four zero cooking-time/total
entries. No raw archaeology NBT exists. Generic barrier/state/load behavior applies after processing
and clipping.

**Archaeology NBT and loot:**

The append-loot rule uses a fresh position-derived rule RNG and stores its first `nextLong`; it does
not use the caller structure stream. `BrushableBlockEntity` is not a `RandomizableContainer`, so
generic template placement does not replace that stored seed with another caller `nextLong`. The
suspicious-gravel write is still gated by clip, block write and resulting typed block entity before
NBT survives.

Both loot records use their matching random sequence and one roll. Common has 31 entries: emerald,
wheat, wooden hoe, clay, brick, yellow/blue/light-blue/white/orange dyes and red/green/purple/brown
candles each have weight `2`; magenta/pink/blue/light-blue/red/yellow/purple stained-glass panes,
spruce/oak hanging signs, gold nugget, coal, wheat/beetroot seeds, dead bush, flower pot, string and
lead each have weight `1`. Rare has 12 equal entries: burn, danger, friend, heart, heartbreak, howl
and sheaf pottery sherds; wayfinder, raiser, shaper and host armor-trim templates; and relic music
disc. Item production remains owned by `ITM-LOOT-001`.

**Branches and aborts:**

Seven primary/fallback pools and 84 equal choices; connector attachment/final-state branches;
ordinary raw/final air versus absent coordinates; aging first-match outcomes; common/rare cap
success, mismatch and exhaustion; whole-piece processing versus final chunk clip;
block/write/NBT/type gates; both deferred loot records.

**Constants and randomness:**

Core selection/shuffle uses the structure stream. Aging and appended loot seeds are
transformed-position-derived; each cap is world-seed-plus-adjusted-origin-derived. Cap order,
encoded cell order and two-stage house processing are observable. Loot evaluation is deferred.

**Side effects:**

Sparse rigid trail pieces with destructive ordinary air; aged gravel/mud-brick variants; at most six
common plus three rare archaeology blocks per house, two common per road or two common per tower
top; exact brushable loot NBT; eight fixed workstation/campfire block entities; no raw entity
creation.

**Gates:**

Record/core and connector/collision/depth/range; pool/template/processor/tag availability; world
seed/origin/rotation; aging predicates and capped eligible count; current final chunk clip; live
block/fluid and setBlock result; resulting block-entity type; loot registries.

**Boundary cases and quirks:**

Five connector-final gravel and 19 connector-final mud-brick cells are processor inputs, not merely
connector cleanup. Capped processing sees out-of-chunk cells before clipping and is repeated per
placement-chunk invocation. Common and rare limits can underfill. Raw/final air overwrites, while
18,620 absent cells do nothing. Brushable loot seeds are processor-derived rather than caller-stream
chest seeds. There are no raw trail-ruins entities or structure markers.

**Evidence:**

`Confirmed`; `OFF-SERVER-001`; `OFF-DATA-001`. Anchors: all seven trail-ruins pool records;
`trail_ruins_houses_archaeology`, `trail_ruins_roads_archaeology` and
`trail_ruins_tower_top_archaeology`; `trail_ruins_replaceable`; all 84 trail-ruins templates; both
trail-ruins archaeology loot records; generic single-element, jigsaw-replacement,
rule/capped/append-loot and template block/NBT/liquid paths; `BrushableBlockEntity#setLootTable`.

**Test vectors:**

Query/decode seven pools, three processor lists, the replaceable tag and two loot records; assert 84
entries/weight `84`, exact directory/order/processor holders and all 84 locator identities. Decode
every NBT input; assert the four-row census, 49 blocks/116 states, 154 connector fields, eight other
NBT cells, zero duplicate/structure-void/structure-block/entity/missing/unreferenced inputs and
every exact raw payload. Replay rotations/origins/chunk visits across connector-final
gravel/mud/air, raw air/absent cells, every aging equality/short circuit, cap
zero/underfill/full/common-then-rare outcome, appended seed, fixed NBT/write/type failure and both
loot records through generic owners.
