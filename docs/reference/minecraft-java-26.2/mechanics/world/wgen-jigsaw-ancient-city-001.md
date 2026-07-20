# World mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `WGEN-JIGSAW-ANCIENT-CITY-001` — Ancient-city pools compose sparse redstone ruins and fixed container payloads

**Parent:** `WGEN-003`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — the seven locked ancient-city pool records, three processor lists, sculk feature
record, 58 present NBT templates and two chest loot records fix the complete ancient-city payload
supplied to the generic jigsaw, processor and template transactions. The sole missing locator and
sole present unreferenced template are explicit source boundaries, not inferred assets. The other
five jigsaw payload families are owned by their named leaves, so the shared `minecraft:jigsaw`
structure type is complete.

**Applies when:**

`ancient_city` starts from `city_center`, a connector expands any of its seven pools, a selected
single/list/feature element places, a raw payload cell is processed or written, or a resulting
ancient-city container resolves fixed NBT or loot.

**Authoritative state:**

The record/core inputs from `WGEN-JIGSAW-RECORDS-001` and `WGEN-JIGSAW-CORE-001`; exact pool entry
order/weight/type/projection/processor; template size, sparse raw cells, states and NBT; connector
metadata; current chunk box/live block/fluid/block entity; caller structure RNG; both loot records.

**Transition and ordering:**

The record's named `city_anchor` selects one of three equal center templates. All seven pools have
`minecraft:empty` fallback and rigid projection. Weighted selection and connector/collision
traversal follow the core. Single placement uses known-shape flags `18`, applies structure-block
ignore, jigsaw replacement, the entry's processors and no projection processor, then performs the
generic chunk-clipped block/NBT/liquid transaction. List elements expose only their first child's
connectors, use the union box/max size, and place every child at the same origin in order, stopping
at the first false result. The sculk feature element invokes its configured feature directly and is
not template-box clipped.

**Exact pool composition:**

There are 65 top-level entries and expanded weight `107`:

- `city/entrance` has the connector and paths `1..5`, all weight `1`, generic degradation.
  `city_center` has centers `1..3`, all weight `1`, start degradation. `city_center/walls` has
  bottom `1/2`, bottom-left, bottom-right `1/2`, left, right, top and both top corners, all weight
  `1`, generic degradation.
- `sculk` is configured feature `sculk_patch_ancient_city` weight `6`, then Empty weight `1`.
  `structures` starts Empty weight `7`; barracks, chambers `1..3`, sauna and small statue are each
  weight `4`; large ruin and tall ruins `1/2` are weight `1`, tall ruins `3/4` weight `2`; the
  ordered `[camp_1,camp_2,camp_3]` list is weight `1`; both medium ruins, both small ruins, large
  pillar and medium pillar are weight `1`; a one-child `ice_box_1` list is weight `1`. All ordinary
  children use generic degradation. The camp children do too; they overlay in `1,2,3` order at one
  origin and only `camp_1` supplies connectors. Ice box uses an inline empty processor list, so it
  has neither degradation nor protected-block rejection.
- `walls` lists intact corner, intersection, L-shape, horizontal `1/2`, stairs `1/2/3` at weight
  `1`, stairs `4` at `4`, passage at `3`, ruined corners `1/2` at `1`, ruined stairs `1/2` at `2`
  and ruined stairs `3/4` at `3`; all use walls degradation. `walls/no_corners` gives equal weight
  `1` to horizontal `1/2`, stairs `1..5` and bridge, also walls degradation.

The no-corners `intact_horizontal_wall_stairs_5` resource is absent. `getOrCreate` caches a new
empty zero-sized/no-palette template; it exposes no target connector, so this candidate is skipped
during child matching and never becomes a saved piece. Conversely
`city_center/walls/bottom_right_corner` is present but named by no pool and is unreachable from
locked composition. Six horizontal/stair templates appear once in `walls` and once in
`walls/no_corners`; all other reachable template locators appear once, including nested list
children.

**Locked payload census:**

The 58 present inputs each have one palette and no duplicate coordinates. Together they declare
73,493 cells across 429,492 bounding-volume coordinates, leaving 355,999 coordinates absent and
therefore untouched. They use 72 block IDs and 292 exact states, with no explicit ordinary air,
structure void or structure block; three cells are cave air. There are 157 jigsaws, 52 other
block-NBT cells and no raw entities. The complete per-template physical audit is:

| Template suffix under `ancient_city/` | Size | Cells | Jigsaws | Other block NBT |
|---|---:|---:|---:|---|
| `city/entrance/entrance_connector` | `30×15×39` | 2,652 | 4 | — |
| `city/entrance/entrance_path_1` | `35×15×39` | 2,759 | 3 | — |
| `city/entrance/entrance_path_2` | `35×15×39` | 4,131 | 3 | — |
| `city/entrance/entrance_path_3` | `35×15×39` | 3,533 | 3 | — |
| `city/entrance/entrance_path_4` | `35×15×39` | 3,408 | 3 | — |
| `city/entrance/entrance_path_5` | `35×15×39` | 2,674 | 3 | — |
| `city_center/city_center_1` | `18×31×41` | 7,966 | 5 | comparator 8; lectern 1; sensor 1 |
| `city_center/city_center_2` | `18×31×41` | 7,901 | 5 | comparator 9; furnace, lectern, sensor, chest 1 each |
| `city_center/city_center_3` | `18×31×41` | 7,908 | 5 | comparator 2; lectern 1; sensor 1 |
| `city_center/walls/bottom_1` | `14×24×41` | 1,993 | 4 | sensor 4 |
| `city_center/walls/bottom_2` | `14×24×41` | 1,834 | 4 | sensor 3 |
| `city_center/walls/bottom_left_corner` | `14×24×14` | 639 | 1 | — |
| `city_center/walls/bottom_right_corner` | `14×24×14` | 783 | 1 | —; unreferenced |
| `city_center/walls/bottom_right_corner_1` | `14×24×14` | 601 | 1 | — |
| `city_center/walls/bottom_right_corner_2` | `14×24×14` | 640 | 1 | — |
| `city_center/walls/left` | `18×24×14` | 838 | 2 | — |
| `city_center/walls/right` | `18×24×14` | 841 | 2 | — |
| `city_center/walls/top` | `14×24×41` | 1,974 | 4 | — |
| `city_center/walls/top_left_corner` | `14×24×14` | 642 | 1 | — |
| `city_center/walls/top_right_corner` | `14×24×14` | 647 | 1 | — |
| `structures/barracks` | `21×12×17` | 1,579 | 3 | chest 2; skull 1 |
| `structures/camp_1` | `17×5×17` | 234 | 1 | — |
| `structures/camp_2` | `17×5×17` | 220 | 1 | campfire 1 |
| `structures/camp_3` | `17×5×17` | 155 | 1 | campfire 1 |
| `structures/chamber_1` | `19×10×15` | 665 | 3 | chest 1 |
| `structures/chamber_2` | `12×6×11` | 274 | 2 | chest 1 |
| `structures/chamber_3` | `10×6×11` | 206 | 2 | chest 1 |
| `structures/ice_box_1` | `19×10×15` | 946 | 3 | chest 1 |
| `structures/large_pillar_1` | `6×15×6` | 320 | 1 | — |
| `structures/large_ruin_1` | `17×5×17` | 38 | 1 | — |
| `structures/medium_pillar_1` | `7×11×8` | 130 | 2 | skull 1 |
| `structures/medium_ruin_1` | `8×5×13` | 20 | 1 | — |
| `structures/medium_ruin_2` | `16×5×11` | 29 | 1 | — |
| `structures/sauna_1` | `29×10×37` | 1,847 | 5 | chest 3 |
| `structures/small_ruin_1` | `7×5×8` | 17 | 1 | — |
| `structures/small_ruin_2` | `8×5×4` | 16 | 1 | — |
| `structures/small_statue` | `10×8×8` | 116 | 2 | — |
| `structures/tall_ruin_1` | `17×23×17` | 636 | 1 | chest 1 |
| `structures/tall_ruin_2` | `17×23×17` | 916 | 1 | chest 2 |
| `structures/tall_ruin_3` | `17×23×17` | 288 | 1 | chest 1 |
| `structures/tall_ruin_4` | `17×23×17` | 545 | 1 | chest 1 |
| `walls/intact_corner_wall_1` | `21×15×21` | 1,030 | 4 | — |
| `walls/intact_horizontal_wall_1` | `18×15×21` | 529 | 4 | — |
| `walls/intact_horizontal_wall_2` | `18×15×21` | 523 | 4 | — |
| `walls/intact_horizontal_wall_bridge` | `18×15×21` | 703 | 3 | — |
| `walls/intact_horizontal_wall_passage_1` | `18×15×21` | 360 | 4 | — |
| `walls/intact_horizontal_wall_stairs_1` | `18×15×21` | 533 | 4 | — |
| `walls/intact_horizontal_wall_stairs_2` | `18×15×21` | 541 | 4 | — |
| `walls/intact_horizontal_wall_stairs_3` | `18×15×21` | 538 | 5 | — |
| `walls/intact_horizontal_wall_stairs_4` | `18×15×21` | 466 | 4 | — |
| `walls/intact_intersection_wall_1` | `21×15×21` | 834 | 3 | — |
| `walls/intact_lshape_wall_1` | `21×15×21` | 606 | 2 | — |
| `walls/ruined_corner_wall_1` | `21×15×21` | 1,090 | 4 | — |
| `walls/ruined_corner_wall_2` | `21×15×21` | 888 | 4 | — |
| `walls/ruined_horizontal_wall_stairs_1` | `18×15×21` | 319 | 4 | — |
| `walls/ruined_horizontal_wall_stairs_2` | `18×15×21` | 448 | 4 | — |
| `walls/ruined_horizontal_wall_stairs_3` | `18×15×21` | 263 | 5 | — |
| `walls/ruined_horizontal_wall_stairs_4` | `18×15×21` | 261 | 4 | — |

**Connector payloads:**

All 157 are rollable with selection and placement priority zero. Pool fields are entrance `8`, walls
`38`, empty `38`, center-walls `18`, sculk `12`, no-corners `17`, structures `26`. Final states are
deepslate tiles `49`, deepslate `24`, polished deepslate `5`, air `5`, deepslate bricks `26`,
reinforced deepslate `3`, structure void `34`, cobbled deepslate `6` and deepslate-tile slab `5`.
Names are bottom `12`, city-anchor `3`, connect-bottom `2`, bottom-left `1`, bottom-right `3`,
left/right/top/top-left/top-right `1` each, connect-structure `23`, connect-wall `40`, empty `62`,
entrance-middle `5`, entrance-start `1`; target counts are bottom `12`, connect-bottom `3`,
bottom-left/right `2` each, left/right/top `3` each, top-left/right `1` each, connect-structure
`26`, connect-wall `55`, empty `43`, entrance-middle `1`, entrance-start `2`. Jigsaw replacement
occurs before degradation: structure-void connectors vanish immediately; air/fixed-state
replacements then pass the configured rule/protection chain and, when rottable, its position-stable
integrity gate.

**NBT and container payloads:**

The 52 cells are 19 comparators (18 `OutputSignal=0`, one signal `7`), ten sculk sensors and three
lecterns with ID-only tags, one furnace, 15 chests, two skeleton skulls and two empty campfires with
four zero cooking-time arrays. The center-2 furnace contains one wooden shovel in slot `1`, 24
deepslate in slot `2`, deepslate recipe-use count `24`, cooking total `200`, and zero spent/lit
times. Its center-2 chest contains one golden apple in slot `13`. Thirteen chests name
`chests/ancient_city`; the ice-box chest names `chests/ancient_city_ice_box`; none stores an initial
loot seed.

For every retained NBT cell the template first offers a barrier with flags `820`, then the rotated
state with flags `18`; failure of the second write can leave the barrier. A successfully created
randomizable chest consumes one caller `nextLong` and writes/overwrites `LootTableSeed` even for the
fixed-item chest. Other typed entities load the copied exact NBT without that draw. Known-shape
placement suppresses template neighbor-shape repair. Default ancient-city liquid mode applies
waterlogging/source-fluid preservation. The 13 ordinary-loot chests and one ice-box chest are
independently gated by processing, clip, write and resulting block-entity type; successful
fixed-item chest placement advances the same stream before later chests according to
block-list/piece visitation order.

The main loot record is a chest table with random sequence `chests/ancient_city`: its first pool
rolls uniformly `5..10` across 27 exact entries/functions and its second rolls once across Empty,
ward trim and silence trim. The ice-box record uses sequence `chests/ancient_city_ice_box` and
uniformly rolls `4..10` across suspicious stew, golden carrot, baked potato, packed ice and snowball
entries. Exact weights, counts, enchantments, effects and functions are locked data-only values
returned by `mc-ref query`; generic pool/entry/function execution remains `ITM-LOOT-001`.

**Branches and aborts:**

Every weighted pool entry and fallback; three centers; missing/no-connector template;
ordinary/list/feature/empty elements; camp child success/failure; ice-box unprotected path; each
clip/write/processor/jigsaw-final outcome; 52 NBT typed/wrong/missing entities; 15 chest seed draws;
fixed versus two loot tables; sculk feature success.

**Constants and randomness:**

Pool expansion/choice and connector shuffle use the core stream. Degradation and rule decisions are
world-position seeded; list children repeat those seeds at shared coordinates. Feature placement and
retained chest seeds continue the caller stream in placement order. Subsequent
stored-seed/named-sequence loot evaluation remains under the loot owner.

**Side effects:**

Sparse rotated ruins, replacement final states, source-preserved fluids,
redstone/sculk/container/skull/campfire block entities, fixed furnace/apple state, up to 14
loot-bearing chests, and direct sculk-patch feature writes; no raw template entity creation.

**Gates:**

Record/core admission; exact pool/template availability; connector/collision/depth/range; current
chunk clip; processor/protected live state; block/fluid/barrier writes; resulting block-entity type;
feature and loot registries.

**Boundary cases and quirks:**

Absent sparse coordinates and the missing resource are different: absent coordinates are
intentionally untouched inside real boxes, while the missing resource becomes an empty cached
template and offers no connector. Camp list children overlay at one origin but only camp 1 connects.
Ice box bypasses both degradation and protected-block filtering. No raw template air exists, yet
five jigsaws can become air and three raw cells are cave air. The fixed apple chest still consumes
and stores a loot seed.

**Evidence:**

`Confirmed`; `OFF-SERVER-001`; `OFF-DATA-001`. Anchors: all seven ancient-city pool JSON records;
three ancient-city processor lists; 58 present ancient-city NBT inputs; configured
`sculk_patch_ancient_city`; both ancient-city loot records; generic pool/list/single/feature,
template-manager, processor, template-write and randomizable-container paths.

**Test vectors:**

Assert the exact pool table and 58-row physical census, 72/292 state census, 157 connector metadata,
52 NBT cells, zero duplicate coordinates/ordinary-air/structure-void/structure-block/raw-entity
cells, missing/unreferenced boundaries and all processor holders. Replay all pool weights,
rotations/clips/live protections, camp overlays, ice-box overwrite, sculk feature, jigsaw final
states, fluid/barrier/NBT/chest-seed ordering and exact loot queries.
