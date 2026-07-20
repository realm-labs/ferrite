# World mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `WGEN-JIGSAW-VILLAGES-001` — Village pools combine terrain-matched legacy buildings, finalized residents, and biome features

**Parent:** `WGEN-003`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — the 62 locked village pools, 16 referenced processor lists, 13 placed-feature
holders, 483 present NBT templates and 16 village chest records fix the complete
desert/plains/savanna/snowy/taiga normal and zombie payload supplied to generic jigsaw/template
placement. All 478 named templates exist; five present templates are explicitly unreachable.
Legacy-single filtering, the malformed savanna lamp final state, fixed block NBT, caller-seeded
containers and finalized raw entities are specified here. Together with the generic core, records,
processors and the other five payload leaves, this completes the locked `minecraft:jigsaw`
structure-type family; shared structure scheduling remains with `WGEN-PIPELINE-001`.

**Applies when:**

Any of the five village records selects a town center, a village connector expands a common, biome,
zombie, house, street, terminator, decor, villager, animal or tree pool, a legacy template or
feature element places, or a retained container/entity payload enters live state.

**Authoritative state:**

Record/core inputs; 62 ordered pools and their primary/fallback weights; 483 single-palette template
sizes, sparse cells, exact states, connector/NBT/entity records; configured processors and
projection; current heightmap/chunk clip/block/fluid/block entity; structure RNG for
pool/features/container seeds; level RNG, biome/position, local difficulty and date inputs for
finalization; 16 loot records.

**Transition and ordering:**

The 62 pools have 649 top-level entries and expanded weight `2,950`. They contain 592 legacy
singles, 35 rigid features and 22 Empty elements; no ordinary single or list occurs. Legacy singles
split `410` rigid and `182` terrain matching. The complete top-prefix census is:

| Prefix below `village/` | Pools | Entries | Expanded weight | Legacy | Feature | Empty |
|---|---:|---:|---:|---:|---:|---:|
| `common` | 6 | 29 | 51 | 27 | 0 | 2 |
| `desert` | 12 | 104 | 544 | 96 | 4 | 4 |
| `plains` | 11 | 133 | 513 | 122 | 7 | 4 |
| `savanna` | 12 | 136 | 792 | 125 | 7 | 4 |
| `snowy` | 11 | 124 | 610 | 113 | 7 | 4 |
| `taiga` | 10 | 123 | 440 | 109 | 10 | 4 |

Forty-two pools fall back to `minecraft:empty`. The other 20 use desert terminators twice,
desert-zombie terminators twice, plains terminators four times across normal/zombie streets and
houses, savanna terminators twice, savanna-zombie terminators twice, snowy terminators four times
and taiga terminators four times. Each biome's town-center pool starts the corresponding normal or
zombie graph; exact entry order, weights and all fallback identities remain locked queryable record
data.

The 592 singles select inline empty processors `198` times and the 16 village lists `394` times:
farm desert/plains/savanna/snowy/taiga `3/2/3/2/2`; mossify 10/20/70 percent `53/2/2`; street
plains/savanna/snowy-or-taiga `36/48/72`; zombie desert/plains/savanna/snowy/taiga `32/40/36/33/28`.
Jigsaw replacement precedes those lists; terrain matching then applies `WORLD_SURFACE_WG` gravity
offset `-1`; legacy filtering runs last. Farm/street/moss/zombie first-match transformations and
their position-stable draws are exactly those of `WGEN-JIGSAW-PROCESSORS-001`. In particular, any
processor-produced ordinary air is removed rather than written.

The 35 feature entries have expanded weight `94` and directly place these placed-feature holders:
acacia `9`, flower plain `2`, oak `3`, berry bush `2`, cactus patch `8`, taiga-grass patch `8`, hay
pile `18`, ice pile `5`, melon pile `2`, pumpkin pile `4`, snow pile `8`, pine `8` and spruce `17`.
A selected feature has point geometry, ignores the piece clip, consumes the continuing structure
stream through its configured/placed algorithm, and returns that algorithm's result; those
algorithms and records remain owned by their feature leaves.

**Template census and legacy writes:**

All 483 present templates have one palette and no duplicate coordinate. Their combined bounding
volume is `204,346`: `161,429` encoded cells and `42,917` absent coordinates. They use 164 block IDs
and 617 exact states, including 97,710 ordinary air, 55 cave air, 447 water, seven lava, 1,917
jigsaws, 264 other NBT-bearing cells and 54 entities; there is no raw structure void or structure
block. Directory totals are:

| Directory | Templates | Volume | Encoded | Absent | Air | Jigsaws | Other NBT | Entities |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| `common` | 21 | 108 | 108 | 0 | 40 | 21 | 0 | 26 |
| `decays` | 3 | 480 | 480 | 0 | 0 | 0 | 0 | 0 |
| `desert` | 76 | 27,494 | 19,491 | 8,003 | 9,950 | 295 | 30 | 6 |
| `plains` | 104 | 54,196 | 42,505 | 11,691 | 25,840 | 446 | 37 | 5 |
| `savanna` | 98 | 44,554 | 33,689 | 10,865 | 19,858 | 376 | 111 | 5 |
| `snowy` | 89 | 33,703 | 30,568 | 3,135 | 19,848 | 383 | 43 | 5 |
| `taiga` | 92 | 43,811 | 34,588 | 9,223 | 22,174 | 396 | 43 | 7 |

Legacy's final ignore matches only ordinary air and structure blocks. Therefore the 97,710 raw air
cells and seven connector-final air states preserve live blocks; absent cells likewise do nothing
for a different reason. Cave air is not ordinary air: 54 savanna cells across two tool/weapon houses
and the normal/zombie meeting-point pair, plus one snowy cartographer-house cell, remain destructive
payload. Water, lava and solid cells also remain payload, subject to processors, clip, write and the
record-default waterlogging/source-fluid transaction.

All 478 named template locators exist. The three `decays/grass_9x9`, `grass_11x13`, `grass_16x16`
resources and normal/zombie snowy `streets/crossroad_01` resources are present but named by no pool.
No named resource is missing. The decays are inert template data, not an implied hidden processor
input.

**Connector payloads:**

All 1,917 connectors have selection and placement priority zero; 1,156 are aligned and 761 rollable.
Pool fields cover 50 identities; the largest counts are empty `261`, common cats `117`, plains
streets `79`, plains zombie streets `68`, savanna zombie streets `67`, savanna streets `66`, snowy
normal/zombie streets `55` each and taiga normal/zombie houses `49` each. Forty-three final-state
strings are present: structure void `1,124`, grass block `373`, dirt `66`, dirt path `53`, smooth
sandstone `49`, sand `47`, cobblestone `36`, plus the exact smaller wood/stair/fence/ice/fire
states. Structure void removes the connector; ordinary final air is subsequently legacy-filtered.
The sole `savanna_lamp_post_01` connector has malformed
`minecraft:acacia_fence[east=false,north=false,south=false,waterlogged=false,west=false]]`; parse
failure logs and removes that cell rather than placing a fence or retaining a jigsaw. Connector
discovery already occurred, so none of these replacement outcomes alters graph topology.

**Block NBT and caller stream:**

The 264 cells are 62 loot chests, 18 empty barrels, 39 bells, eight blast furnaces, nine brewing
stands, 77 empty-pattern brown banners, five empty campfires, 25 furnaces, eight lecterns, nine
smokers and four empty-text spruce signs. Furnaces/smokers have empty item/recipe maps and zero
cooking/lit fields; brewing/campfire arrays and times are zero; barrels have an empty item list;
bells and lecterns are ID-only. Exact positions, states and NBT are locked template data.

Each retained NBT cell follows the barrier/state/resulting-type/load transaction. Chests and barrels
implement `RandomizableContainer`, so every successfully written typed instance consumes one caller
`nextLong` and injects `LootTableSeed`: up to 80 draws, including every empty barrel. Clip
rejection, processing removal, failed state write or missing/wrong block-entity type suppresses the
load and draw. The other 184 block entities consume no structure RNG.

The 62 chests bind 16 tables: armorer `1`, butcher `1`, cartographer `5`, desert house `8`, fisher
`1`, fletcher `1`, mason `1`, plains house `6`, savanna house `10`, shepherd `1`, snowy house `4`,
taiga house `10`, tannery `4`, temple `1`, toolsmith `3` and weaponsmith `5`. No template stores an
initial loot seed. The 16 exact records each use their own matching random sequence and together
contain 24 pools, 152 top-level entries—144 item and eight Empty—and 91 `set_count` functions, with
no nested loot-table reference. Exact rolls, weights, items and counts are locked queryable data
evaluated later by `ITM-LOOT-001`.

**Raw entities and finalization:**

The 54 records are ten cats, two cows, seven horses, two pigs, four sheep, one iron golem, one
minimal camel, 15 villagers, ten zombie villagers and two taiga armor stands. Each biome has baby,
nitwit and unemployed villagers plus nitwit/unemployed zombie villagers; their stored village types
are desert, plains, savanna, snow and taiga. The baby age is `-21,359`; normal villagers are
nonpersistent and zombies persistent. The armor stands carry an iron helmet and chestplate
respectively. Exact health, attributes, variants, motion, rotations, equipment and other saved
fields remain locked payload.

Placement tests transformed integer `blockPos` against the chunk box, transforms the fractional
position, replaces `Pos`, removes UUID, creates with STRUCTURE reason and applies rotation before
addition with passengers. The two armor stands are not mobs and skip finalization. The other 52 run
subtype and superclass finalization on the level RNG: cats/cows/pigs select current position/biome
variants and sound variants, sheep replace saved color from the spawn-color rule, horses replace
saved coat/markings and base health/speed/jump attributes, and the minimal camel initializes
memories and standing-pose time. Saved variants are therefore inputs to load but not necessarily
final outputs. Each independently finalized ageable record receives null group data, so its new
group starts at size zero and does not take the later-group baby branch; the saved villager baby
ages survive.

VillagerData in all 25 resident records marks type finalized on load. Villagers preserve
type/profession, mark STRUCTURE profession assignment for their first AI step, then run
abstract-villager/ageable/mob finalization. Zombie villagers preserve VillagerData but run the full
STRUCTURE zombie finalizer: difficulty-dependent loot pickup, adult/baby group selection, possible
jockey path, door/equipment/enchantment, Halloween headgear and reinforcement/attribute paths remain
live; the stored adult age is not a promise of an adult final result. Mob finalization retains an
existing `random_spawn_bonus` follow-range modifier where present and otherwise samples one, then
replaces handedness. Twenty raw mobs already carry that modifier; the other 32 do not. These
level-stream draws are independent of pool/feature/container RNG and occur in template entity order
after block placement.

**Branches and aborts:**

Five record starts and normal/zombie center choices; all 62 primary/fallback pools, weights and
Empty positions; rigid/terrain-matched legacy paths; 16 processor lists and all rule outcomes; 13
feature holders/results; named/unreachable resources; ordinary/cave air, absent, fluid, solid and
malformed/void/fixed connector finals; 264 NBT clip/write/type/seed paths; all 16 loot records; 54
entity clip/decode/type/finalization/add paths.

**Constants and randomness:**

Pool and connector selection use the structure stream; rule lists restart position-derived streams;
feature placement continues the structure stream; each retained chest/barrel consumes one structure
`nextLong`. Entity finalization uses the level RNG plus live biome, local difficulty and date. Later
stored-seed named-sequence loot is separate. Exact record, block and entity order is observable.

**Side effects:**

Terrain-matched streets and rigid houses/decor, nondestructive ordinary-air masks but destructive
cave air/fluids/materials, farm/moss/street/zombie rewrites, direct vegetation/pile features,
workstation/decor/container block entities, seeded village loot, finalized residents/animals and two
equipped armor stands.

**Gates:**

Record/set/biome and core graph admission; fallback/depth/range/collision;
resource/processor/feature lookup; projection heightmap; current chunk and live block/fluid/entity
state; typed block-entity creation; entity decode and subtype finalization; loot registry/evaluator.

**Boundary cases and quirks:**

Legacy air suppression happens after configured rules and gravity. Cave air does overwrite. Five
real templates are unreachable, while no locator is missing. Feature elements are not chunk-box
clipped. Empty barrels advance the caller stream. Saved animal variants can be replaced by
finalization; zombie villagers can become babies or jockeys. One syntactically invalid final state
removes its lamp connector cell.

**Evidence:**

`Confirmed`; `OFF-SERVER-001`; `OFF-DATA-001`; `OFF-REPORT-001`. Anchors: all 62 village pool
records; all 16 referenced processor lists; 13 placed-feature holders; all 483 village templates; 16
village chest tables; generic legacy/feature/jigsaw/processor/template block-NBT-liquid-entity
transactions; `RandomizableContainer`; `Mob`, cat/cow/pig/sheep/horse/camel/villager/zombie-villager
finalization paths.

**Test vectors:**

Query/decode 62 pool records, 16 processor lists, 13 feature holders and 16 loot records; assert 649
entries/2,950 weight, exact group/fallback/projection/processor/feature topology and all 478 locator
identities. Decode all 483 templates; assert the seven-row physical census, 164 blocks/617 states,
1,917 exact connector fields, 264 NBT cells, 54 entities, zero duplicates/raw
structure-void/structure-block/missing locators and exactly five unreferenced inputs. Replay
ordinary-air suppression versus cave-air overwrite/absence, every processor/gravity/final-state
outcome including malformed lamp state, all 80 container seed/no-seed paths, exact fixed NBT, all
entity transform/finalization branches and all 16 nonrecursive loot records through their generic
owners.
