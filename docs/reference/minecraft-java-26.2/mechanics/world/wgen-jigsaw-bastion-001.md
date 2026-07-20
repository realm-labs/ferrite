# World mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `WGEN-JIGSAW-BASTION-001` — Bastion pools compose destructive air masks, loot rooms, spawners, and finalized mobs

**Parent:** `WGEN-003`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — the 60 locked bastion pool records, 12 named processor lists, 167 present NBT
templates and four chest loot records fix the complete bastion payload supplied to the generic
jigsaw, processor and template transactions. Every named template exists and is reachable; explicit
air is an overwrite payload, not an absent-cell inference. Outpost, trail-ruins, trial-chambers and
village payloads are owned by `WGEN-JIGSAW-OUTPOST-001`, `WGEN-JIGSAW-TRAIL-RUINS-001`,
`WGEN-JIGSAW-TRIAL-CHAMBERS-001` and `WGEN-JIGSAW-VILLAGES-001`; the shared `minecraft:jigsaw`
structure type is complete.

**Applies when:**

`bastion_remnant` selects one of the four `bastion/starts` entries, any of its connectors expands a
bastion pool, a single element places its sparse blocks/NBT/entities, or a resulting chest,
magma-cube spawner, hoglin, piglin or piglin brute enters live state.

**Authoritative state:**

The record/core inputs from `WGEN-JIGSAW-RECORDS-001` and `WGEN-JIGSAW-CORE-001`; exact pool order,
weights, projection and processor holder; template size, sparse raw cells, states, NBT and entity
records; connector metadata; current chunk box and live block/fluid/block entity; caller structure
RNG; level RNG/local difficulty; four loot records.

**Transition and ordering:**

All 60 pools fall back to `minecraft:empty`. Their 176 top-level entries are rigid
`single_pool_element` values with expanded weight `189`; no pool uses Empty, feature, list or
legacy-single as a primary entry. The root census is blocks `1 pool/2 entries/weight 4`, bridge
`7/12/12`, hoglin stable `13/53/53`, mobs `3/9/20`, starts `1/4/4`, treasure `20/65/65`, units
`15/31/31`. The four start choices are equal-weight units `air_base`, hoglin-stable `air_base`,
treasure `big_air_full` and bridge `entrance_base`, each using generic degradation.

`blocks/gold` selects its one-cell air-final jigsaw at weight `3` or gold-final jigsaw at weight
`1`. The mob pools select hoglin/empty at `2:1`; ordinary piglin selects the brute-bearing
`melee_piglin`, sword piglin, crossbow piglin and empty templates at `1:4:4:1`; melee selects
`melee_piglin_always`, `melee_piglin` and sword piglin at `1:5:1`. Those 11 block/mob entries use
inline empty processor lists. Every other entry has weight one. Named processor references are
generic degradation `9`, bottom rampart `1`, bridge `1`, entrance replacement `1`, high rampart `1`,
high wall `10`, housing `31`, rampart degradation `5`, roof `3`, side-wall degradation `2`, stable
degradation `51` and treasure rooms `50`; their exact first-match transformations are owned by
`WGEN-JIGSAW-PROCESSORS-001`.

The 176 entries name exactly 167 distinct locators, and all 167 exist. Repeated locators are mob
empty, melee piglin, sword piglin and unit ramparts `ramparts_0` twice each, treasure fire room
twice and treasure extension empty five times. No present bastion template is unreferenced. The
empty-named entries are real nonempty templates, distinct from the fallback Empty element.

**Locked payload census:**

Every input has one palette and no duplicate coordinate. Together they declare 328,359 cells across
447,318 bounding-volume coordinates, leaving 118,959 absent coordinates untouched. They use 28 block
IDs and 80 exact states. Raw cells include 170,429 ordinary air, 2,564 lava, 637 jigsaws, 38
non-jigsaw block-NBT cells and five entities; there is no structure void or structure block. The
directory-level physical audit is:

| Template directory under `bastion/` | Templates | Volume | Cells | Air | Absent | Jigsaws | Other NBT | Entities |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| `blocks` | 2 | 2 | 2 | 0 | 0 | 2 | 0 | 0 |
| `bridge/bridge_pieces` | 1 | 9,424 | 5,182 | 3,236 | 4,242 | 4 | 0 | 0 |
| `bridge/connectors` | 2 | 2,888 | 735 | 556 | 2,153 | 2 | 0 | 0 |
| `bridge/legs` | 2 | 396 | 315 | 0 | 81 | 2 | 0 | 0 |
| `bridge/rampart_plates` | 1 | 608 | 563 | 157 | 45 | 1 | 0 | 0 |
| `bridge/ramparts` | 2 | 13,824 | 10,738 | 1,547 | 3,086 | 11 | 4 | 0 |
| `bridge/starting_pieces` | 3 | 37,632 | 30,346 | 6,897 | 7,286 | 33 | 1 | 0 |
| `bridge/walls` | 2 | 12,288 | 12,288 | 1,949 | 0 | 6 | 0 | 0 |
| `hoglin_stable` | 1 | 34,560 | 29,938 | 29,936 | 4,622 | 2 | 0 | 0 |
| `hoglin_stable/connectors` | 1 | 88 | 2 | 0 | 86 | 2 | 0 | 0 |
| `hoglin_stable/large_stables` | 10 | 6,720 | 5,407 | 2,394 | 1,313 | 31 | 1 | 0 |
| `hoglin_stable/posts` | 2 | 576 | 387 | 99 | 189 | 4 | 0 | 0 |
| `hoglin_stable/rampart_plates` | 1 | 608 | 608 | 202 | 0 | 1 | 0 | 0 |
| `hoglin_stable/ramparts` | 3 | 16,640 | 12,468 | 1,912 | 4,172 | 17 | 6 | 0 |
| `hoglin_stable/small_stables` | 8 | 4,608 | 3,522 | 1,908 | 1,086 | 29 | 1 | 0 |
| `hoglin_stable/stairs` | 15 | 20,160 | 17,494 | 12,984 | 2,666 | 55 | 0 | 0 |
| `hoglin_stable/starting_pieces` | 10 | 14,400 | 13,870 | 10,865 | 530 | 45 | 0 | 0 |
| `hoglin_stable/walls` | 3 | 18,432 | 17,589 | 2,131 | 843 | 19 | 2 | 0 |
| `mobs` | 6 | 25 | 25 | 19 | 0 | 6 | 0 | 5 |
| `treasure` | 1 | 69,312 | 27,650 | 27,648 | 41,662 | 2 | 0 | 0 |
| `treasure/bases` | 1 | 6,336 | 1,903 | 5 | 4,433 | 10 | 1 | 0 |
| `treasure/bases/centers` | 4 | 1,344 | 591 | 20 | 753 | 10 | 5 | 0 |
| `treasure/brains` | 1 | 140 | 10 | 0 | 130 | 10 | 0 | 0 |
| `treasure/connectors` | 3 | 33 | 6 | 0 | 27 | 6 | 0 | 0 |
| `treasure/corners/bottom` | 2 | 800 | 800 | 359 | 0 | 8 | 0 | 0 |
| `treasure/corners/edges` | 3 | 192 | 192 | 0 | 0 | 3 | 0 | 0 |
| `treasure/corners/middle` | 2 | 750 | 750 | 469 | 0 | 8 | 0 | 0 |
| `treasure/corners/top` | 2 | 750 | 750 | 456 | 0 | 10 | 0 | 0 |
| `treasure/entrances` | 1 | 6,840 | 4,047 | 3,740 | 2,793 | 3 | 0 | 0 |
| `treasure/extensions` | 13 | 2,945 | 2,147 | 815 | 798 | 13 | 0 | 0 |
| `treasure/ramparts` | 6 | 43,806 | 39,912 | 14,092 | 3,894 | 24 | 4 | 0 |
| `treasure/roofs` | 3 | 1,442 | 1,442 | 0 | 0 | 3 | 0 | 0 |
| `treasure/stairs` | 1 | 50 | 30 | 0 | 20 | 1 | 0 | 0 |
| `treasure/walls` | 2 | 1,824 | 1,824 | 208 | 0 | 3 | 0 | 0 |
| `treasure/walls/bottom` | 4 | 7,680 | 7,680 | 2,812 | 0 | 31 | 1 | 0 |
| `treasure/walls/mid` | 3 | 5,400 | 5,400 | 3,211 | 0 | 34 | 1 | 0 |
| `treasure/walls/outer` | 6 | 2,784 | 2,784 | 0 | 0 | 12 | 0 | 0 |
| `treasure/walls/top` | 3 | 5,400 | 5,400 | 2,830 | 0 | 32 | 0 | 0 |
| `units` | 1 | 50,784 | 23,447 | 23,444 | 27,337 | 3 | 0 | 0 |
| `units/center_pieces` | 3 | 2,783 | 2,783 | 2,382 | 0 | 24 | 3 | 0 |
| `units/edges` | 1 | 672 | 628 | 343 | 44 | 6 | 0 | 0 |
| `units/fillers` | 1 | 672 | 672 | 386 | 0 | 7 | 0 | 0 |
| `units/pathways` | 2 | 24 | 24 | 8 | 0 | 4 | 0 | 0 |
| `units/rampart_plates` | 1 | 1,520 | 1,103 | 697 | 417 | 1 | 0 | 0 |
| `units/ramparts` | 3 | 16,896 | 14,485 | 3,176 | 2,411 | 9 | 4 | 0 |
| `units/stages` | 14 | 7,872 | 6,900 | 3,759 | 972 | 62 | 2 | 0 |
| `units/stages/rot` | 1 | 672 | 672 | 429 | 0 | 2 | 0 | 0 |
| `units/wall_units` | 2 | 1,428 | 1,423 | 799 | 5 | 9 | 0 | 0 |
| `units/walls` | 2 | 12,288 | 11,425 | 1,549 | 863 | 15 | 2 | 0 |

The exact block counts are polished-blackstone bricks `86,109`, blackstone `55,462`, air `170,429`,
basalt `6,849`, cracked polished-blackstone bricks `4,234`, lava `2,564`, jigsaw `637`,
polished-blackstone-brick stairs `751`, polished basalt `196`, blackstone slabs `185`, lanterns
`141`, chiseled polished blackstone `137`, gold blocks `131`, gilded blackstone `100`, magma blocks
`88`, blackstone stairs `80`, blackstone walls `72`, soul sand `58`, iron chains `54`, chests `37`,
netherrack `16`, nether wart `12`, and four each of fire, glowstone and quartz block, two each of
smooth quartz and smooth-quartz slab, and one spawner.

**Connector and write payload:**

All 637 connectors have selection and placement priority zero; `166` are aligned and `471` rollable.
Their pool fields cover 59 bastion pool IDs plus `minecraft:empty`; final states are
polished-blackstone bricks `299`, air `215`, basalt `61`, blackstone `36`, cracked
polished-blackstone bricks `21`, polished basalt `3`, gold block `1` and north-facing
polished-blackstone-brick stairs `1`. The metadata has 87 distinct names and 83 targets;
`minecraft:empty` accounts for 328 names and 160 targets. Jigsaw replacement occurs before every
named rule list.

Every surviving raw air cell is offered as a real flags-`18` block write and can erase live terrain
or an earlier write inside the chunk clip. No bastion list protects live feature blocks or filters
ordinary air. Absent coordinates make no write. The three large start masks alone contribute units
`air_base` 23,444 air, hoglin-stable `air_base` 29,936 and treasure `big_air_full` 27,648; this is
intentional destructive payload, not sparse empty space. Known-shape placement suppresses
neighbor-shape repair, while default liquid handling preserves or propagates source fluid into
compatible placed states.

**Block NBT and loot:**

The 38 cells are 37 chests plus the lava-basin magma-cube spawner. Chest table counts are
`bastion_other` 29, `bastion_treasure` 5, `bastion_hoglin_stable` 2 and `bastion_bridge` 1; none
stores an initial seed. The spawner fixes delay `0`, minimum/maximum delay `200/800`, spawn count
`4`, nearby cap `6`, player range `16`, spawn range `4`, and a sole weight-one
`minecraft:magma_cube` spawn potential/data. Retained NBT cells first offer a barrier with flags
`820`, then their rotated block with flags `18`; a successfully created chest consumes caller
`nextLong` and loads it as `LootTableSeed`.

The four named-sequence chest tables are exact data-only inputs to `ITM-LOOT-001`. Bridge has five
pools with entry counts `1/13/5/2/2` and rolls `1`, uniform `1..2`, uniform `2..4`, `1`, `1`; hoglin
stable has four with `9/14/2/2` and `1`, uniform `3..4`, `1`, `1`; other has five with
`11/14/10/2/2` and `1`, `2`, uniform `3..4`, `1`, `1`; treasure has four with `18/9/2/1` and `3`,
uniform `3..4`, `1`, `1`. Exact weights, items, counts, damage and enchantment functions—including
snout-trim and Netherite-upgrade pools—remain the queried records.

**Raw entities and finalization:**

The five entity templates contain one hoglin, two piglins and two piglin brutes. Hoglin carries
health `40`, adult age zero, `CannotBeHunted`, no loot pickup and persistence; sword/crossbow
piglins carry health `16`, their named main-hand weapon, `CannotHunt`, loot pickup and persistence;
both `melee_piglin` templates actually contain health-`50` piglin brutes with loot pickup and
persistence. Placement is entity-enabled and finalizing. After all block writes, the template tests
the transformed integer `blockPos` against the chunk box, copies NBT, replaces exact `Pos`, removes
UUID, creates with STRUCTURE reason, applies rotation, and silently skips decode/create failures. It
then calls mob finalization and adds the entity with passengers.

STRUCTURE piglins do not reroll baby status or main-hand weapon. They sample the 30–120-second
hunted-recently memory, independently roll four `0.1` armor chances, apply local-difficulty
weapon/armor enchant chances, then superclass follow-range and `0.05` left-handed state from the
level RNG. Brutes set HOME to their placed global position, replace main hand with a golden axe,
then run superclass follow-range/handed finalization. The hoglin independently becomes a baby on
level-RNG `<0.2`, then runs ageable/superclass finalization. These draws use the level RNG, not the
caller structure RNG that assigns chest seeds.

**Branches and aborts:**

All 189 expanded pool positions and fallback; 12 named/inline processor choices; every
explicit-air/absent/jigsaw-final distinction; processor match/failure;
block/fluid/barrier/chest/spawner write success; four loot tables; mob template/empty choice; entity
chunk clip, decode failure and type-specific finalization.

**Constants and randomness:**

Pool choice, connector shuffle, piece placement and retained chest seeds use the structure stream.
Rule transformations restart position-seeded streams. Entity initialization uses the level RNG and
local difficulty after the piece's block loop. Data order controls pool expansion, block/NBT writes,
chest seeds and entity creation.

**Side effects:**

Rigid blackstone structures and destructive air masks; lava, gold, degradation variants and
connector final states; 37 loot-bearing chests, one magma-cube spawner, and finalized persistent
hoglin/piglin/brute entities.

**Gates:**

Record/core admission; exact pool/template/processor availability; connector/collision/depth/range;
current chunk clip; processor predicates; live block/fluid and setBlock result; resulting
block-entity type; entity decode/type/local difficulty; loot registries.

**Boundary cases and quirks:**

Every locator exists, unlike ancient city's missing locator. Empty-named bastion templates are not
Empty elements. Explicit air overwrites while absent cells do nothing. The nominal ordinary-piglin
pool can choose a brute, and the melee pool can choose an ordinary sword piglin. Finalized entities
consume the level stream independently of structure choice and chest seeds. The entity clip uses
transformed integer `blockPos`, while placement uses transformed fractional `pos`.

**Evidence:**

`Confirmed`; `OFF-SERVER-001`; `OFF-DATA-001`. Anchors: all 60 bastion pool records; 12 bastion
processor lists; all 167 bastion templates; four bastion chest loot records; generic single-element,
processor, template block/NBT/liquid/entity and mob-finalization paths.

**Test vectors:**

Query/decode 60 pools, 12 lists and four loot records; assert 176 entries/189 weight, root topology,
processor counts and all 167 locator identities. Decode all 167 NBT inputs; assert the 49-row
physical census, 28 block IDs/80 states, 637 connector fields, 38 NBT cells, five entity records,
zero duplicate coordinates/structure-void/structure-block/missing/unreferenced templates and all
exact raw payloads. Replay every choice, clip, explicit-air overwrite, absent cell,
processor/final-state rule, fluid/barrier/block-entity/chest-seed/spawner result, entity
decode/transform/finalization draw and loot query through the generic owners.
