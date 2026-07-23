# Content behavior catalog

The catalog is the second layer of the manual. Algorithms live in
[leaf rules](../mechanics/README.md); [`catalog.toml`](catalog.toml) maps every locked content ID to
one of those algorithms.

The committed file deliberately does not copy Mojang's registries or data pack. Instead each
category records the exact count and SHA-1 of its sorted, newline-terminated ID set.
`mc-ref coverage` regenerates the set from the locked official reports/server jar, verifies the
snapshot, then requires exactly one classification for every ID. Consequently a catch-all family
cannot silently accept content added or removed by an upstream version change.

## Classification meanings

- `BehaviorFamily`: the ID inherits the referenced generic state machine. Its concrete dimensions,
  components, tags, or values are read with `mc-ref query`.
- `Special`: dispatch reaches explicit control flow that must receive a dedicated leaf rule as the
  manual deepens. The family references the current controlling rules.
- `DataOnly`: no independent ID-specific control flow was found. The ID parameterizes the referenced
  algorithm with locked data.
- `Unreviewed`: a temporary, explicit backlog classification. It prevents a broad selector from
  claiming that unaudited control flow is `DataOnly`; `mc-ref readiness` must reject it before the
  reference can be complete.

`Special` may not use a `remaining` selector. A remaining `DataOnly` selector is restricted to the
audited potion, recipe, loot-table, advancement, damage-type and enchantment record collections.
Other fallbacks stay `Unreviewed` until their dispatch or data-only status is demonstrated.
The exact direct-reader sweep and closed classification of every game rule is recorded in
[the game-rule consumer inventory](../game-rule-consumers.md).

Classification is an implementation lookup, not a claim that two IDs have identical data. For
example, all recipe JSON is `DataOnly`, but its serializer chooses the `ITM-CRAFT-001` matching
algorithm and the JSON supplies different ingredients/results.

## Locked breadth

| Kind | IDs | Authoritative source |
|---|---:|---|
| block | 1,196 | `reports/blocks.json` |
| block entity type | 49 | `reports/registries.json` |
| fluid | 5 | `reports/registries.json` |
| ticket type / game rule | 68 | `reports/registries.json` |
| item | 1,537 | item component reports |
| entity type | 158 | `reports/registries.json` |
| mob effect / menu / recipe serializer / potion | 132 | `reports/registries.json` |
| recipe / loot table / advancement | 4,628 | bundled server data |
| worldgen entries | 963 | bundled server data |
| worldgen feature type | 63 | `reports/registries.json` |
| worldgen structure type | 16 | `reports/registries.json` |
| structure processor / rule test / position test / NBT modifier | 24 | `reports/registries.json` |
| density function type | 34 | `reports/registries.json` |
| damage type / enchantment / dimension type | 98 | bundled server data |
| environment attribute | 48 | `reports/registries.json` plus bundled biome data |

The daylight-detector block and its otherwise empty block-entity subtype own their complete 20-tick
sky-light/sun-angle signal transaction in `RED-DAYLIGHT-DETECTOR-001`. Comparator block/cache
behavior is closed by `RED-COMPARATOR-001`. The End-portal subtype owns its contact, particle and
two-face 15-layer render surface in `WGEN-PORTAL-001`.

The current structural coverage checks 9,078 IDs and reports 781 IDs classified as `Unreviewed`:
315 blocks, 245 items, 37 entity types and 184 worldgen records. All 49 block-entity types now have
exact audited owners. Structural coverage and behavioral readiness remain separate gates. The command-block
family now owns all three blocks, its block entity, command minecart and live work rule in
`BLK-COMMAND-001`; `SIM-COMMAND-LIMIT-001` owns both command execution limit rules, and
`BLK-COMMAND-AREA-001` owns the shared clone/fill/fillbiome limit. `MOB-UNIVERSAL-ANGER-001` owns
generic revenge suppression plus classic-neutral and Piglin universal targeting,
`ENT-ENTITY-DROPS-001` owns all seven differently placed entity-drop gates, and `MOB-RAID-001`
owns omen admission, managed lifecycle retirement, waves and persistence. Trial-spawner and vault
block/block-entity runtime own their complete encounter, key, reward, persistence and client-output
transactions in `BLK-TRIAL-SPAWNER-001` and
`BLK-VAULT-001`. `BLK-BRUSHABLE-001` owns brush cadence, shared cooldown, dust regression,
archaeology materialization, completion, falling-data loss, persistence and client item exposure for
both suspicious blocks and the `brushable_block` entity type.
`BLK-SCULK-SENSOR-001` owns same-tick vibration selection, travel, calibrated filtering,
redstone/resonance phases, listener persistence and client effects for both sensor entity types.
`BLK-JIGSAW-001` owns the jigsaw block's 12 orientations, game-master edit UI, seven-field
persistence/synchronization and exact immediate-generation delegation to the existing jigsaw core.
`BLK-STRUCTURE-001` owns the structure block's four modes, complete editable record, operator and
redstone actions, corner scan, template-manager continuity, local UI and boundary/invisible render.
`BLK-TEST-BLOCK-001` owns all four test-block modes, redstone and trigger latches, local operator
editing, persistence divergence and ordered block-based accept/fail/log evaluation.
`BLK-CONDUIT-001` owns the conduit's water/frame activation, tiered power radius, full-frame target
and attack state, ambient/particle clocks, target-only persistence and world/item rendering.
`BLK-BEACON-001` owns incremental colored-beam publication, reloadable pyramid/payment tags,
80-tick level/effect/criterion refresh, power selection, reset-on-load state and client beam
rendering; its serverbound control remains joined to the completed beacon protocol family.
`BLK-SIGN-001` owns all 48 ordinary/hanging sign blocks, both entity types, placement/support and
water states, two-sided filtered text, one-editor admission, click actions, four applicator paths,
persistence and ordinary/hanging UI/rendering; its edit request remains joined to the completed
sign protocol family. `ITM-HONEYCOMB-001` separately owns the item's 15-collection copper wax map,
direct stack shrink, flags-11 replacement and paired copper-chest effect branch.
`ITM-CHEST-001` owns ordinary/trapped placement and pairing, conductor/cat obstruction,
right-first 27/54-slot menus and loot, per-half persistence/removal/recounts, comparator-versus-
hopper access, trapped power and regular/trapped/Christmas lid rendering.
`ITM-HOPPER-001` owns all ten hopper states, redstone disablement, deterministic push-before-pull
automation, sided commit/rollback, block/entity source selection, partial loose-item absorption,
8/7-tick cooldown propagation and persistence, the five-slot menu and facing-only client model.
`ITM-DISPENSER-001` owns all 12 dispenser states, the four-tick redstone latch, nine-slot random
selection and persistence, exact static/component/tag dispatch, residue insertion/ejection,
behavior-specific success quirks and client level events, plus its shared nine-slot
`generic_3x3` menu.
`BLK-SKULL-001` owns all 14 floor/wall skull and head blocks, 280 states, block-entity protocol ID
16, profile/sound/name continuity, player-head naming/fill, custom note sound, client animation and
skin rendering, and the destructive wither-summon transaction.
`BLK-TEST-INSTANCE-001` owns the property-free test-instance block and block-entity protocol ID 46,
its complete editable record, template geometry/capture/placement, GameTest runner replacement,
local operator UI, status responses, persistence and beam/bounds/error rendering.
`BLK-STRUCTURE-VOID-001` owns state 14851, its replaceable/no-collision/piston-destroy block form,
ordinary epic item, fluid rejection, structure-capture omission, jigsaw skip and conditional
structure-block debug projection.
`BLK-AIR-001` owns states 0, 15292 and 15293, their shared empty mechanics, ordinary/cave/void
source roles, exact-palette versus all-air read boundary and the plain AIR item's empty-stack
sentinel semantics.
The 16-ID structure-type
registry is now explicit: `buried_treasure` owns its audited one-piece chest behavior,
`nether_fossil` owns its audited cavity scan, 14 bone templates and dried-ghast postpass, `igloo`
owns its audited terrain-anchored top/optional laboratory templates, `swamp_hut` owns its audited
terrain-averaged cabin, supports and latched occupants, `desert_pyramid` owns its audited four-chunk
fixed piece, trap, chests and global archaeology selection, `jungle_temple` owns its audited
randomized masonry, two tripwire traps and lever-piston hidden chamber, `shipwreck` owns its audited
20-template ocean/beached placement and marker-loot transaction, `ruined_portal` owns its audited
setup/height/processor/apron transaction across 13 templates, `ocean_ruin` owns its audited
warm/cold, cluster, live-height, archaeology and marker transaction across 48 templates,
`stronghold` owns its audited retry-selected weighted graph, 13 piece families, portal-eye timing,
spawner and loot transactions, `mineshaft` owns its audited normal/mesa depth-first graph, live
cancellation, supports, carts and spider transaction, `end_city` owns its audited recursive template
graph, grouped collision transaction, ship latch, markers and treasure, `fortress` owns its audited
quota-weighted bridge/castle graph, 15 pieces, supports, chests, blaze spawner and lava well,
`ocean_monument` owns its audited pruned room lattice, flooded shell, sponges, gold core and three
elders, `woodland_mansion` owns its audited floor graph, 73 templates, marker mobs and chests, fixed
NBT and foundations, and `jigsaw` owns its complete generic core, ten records, processor transaction
and all six locked payload families; all 16 structure types are now explicit. All five
structure-pool-element IDs and all three pool-alias-binding IDs now own their generic jigsaw
transactions in `WGEN-JIGSAW-CORE-001`. All 11 structure processors, six rule tests, three position
tests and four rule NBT modifiers own their generic rewrite behavior in
`WGEN-JIGSAW-PROCESSORS-001`; all 40 processor-list records are exact data-only compositions of that
transaction. The seven ancient-city pools, 58 present sparse template payloads, sculk feature, fixed
block NBT and two loot records are audited in `WGEN-JIGSAW-ANCIENT-CITY-001`, including the missing
and unreferenced template boundaries. The 60 bastion pools, all 167 reachable payloads, destructive
air masks, 37 chests, magma-cube spawner and five finalized mobs are audited in
`WGEN-JIGSAW-BASTION-001`. The four outpost pools, 11 full-cuboid legacy payloads, virtual connector
plates, sparse rot overlay, duplicate tower NBT and three finalized captive mobs are audited in
`WGEN-JIGSAW-OUTPOST-001`. The seven trail-ruins pools, 84 sparse ordinary-single payloads,
destructive air, connector-final processor inputs, whole-piece archaeology caps, eight fixed block
entities and two archaeology loot records are audited in `WGEN-JIGSAW-TRAIL-RUINS-001`. The 47
trial-chamber pools, four structure-wide aliases, 191 copper/tuff payloads, destructive air/water,
degradation/protection, 45 placeable NBT cells, 28 trial-spawner configs, two vault configs, a
24-record transitive loot closure and one standalone trial-chamber loot record are audited in
`WGEN-JIGSAW-TRIAL-CHAMBERS-001`; later trial-spawner and vault runtime dispatch to
`BLK-TRIAL-SPAWNER-001` and `BLK-VAULT-001`. The 62 village pools, 483 legacy payloads, five
unreachable templates, ordinary-air suppression, destructive cave air, malformed lamp final state,
264 fixed block-NBT cells, 54 finalized raw entities, 13 placed features and 16 village loot records
are audited in `WGEN-JIGSAW-VILLAGES-001`. All 25 menu IDs are explicitly classified to the
source-specified slot layout, quick-move route and control transaction in `ITM-CONTAINER-*`; no menu
catch-all remains. All 21 recipe-serializer IDs are explicitly assigned to the shaped/shapeless,
component-special, cooking, stonecutting or smithing algorithms in `ITM-RECIPE-SERIALIZER-001`. All
49 block-entity types inherit the audited generic lifecycle; End gateway owns its exact transition
state, brushable blocks own their complete archaeology runtime, both sculk sensors own their
frequency-bearing vibration runtime, jigsaw owns its editable connector record, beacon owns its
beam/base/effect/menu lifecycle, skull owns profile/sound/name persistence and transient animation,
chest and trapped chest own independent per-half storage/openers
behind their canonical compound view, hopper owns its five-slot transfer/cooldown transaction, and
dispenser owns its nine-slot scheduled dispatch and residue transaction, and trial spawner plus
vault own their full state machines.
Remaining explicit subtype
dispatch is conservatively `Special` and points to its current source-specified lifecycle,
interaction, container, redstone and presentation owners. All nine ticket types are explicitly divided by their simulation
flag. All four dimension types and all 48 environment-attribute IDs now have audited record,
declaration, layer, synchronization and consumer-family ownership in `WGEN-DIMENSION-001`; the three
portal gamerules, portal blocks and End-gateway state are owned by `WGEN-PORTAL-001`. All 34
density-function type IDs are audited behavior families: 18 pure composition, five normal-noise
coordinate, old-blended, End-island, three old/new-generation blend, structure beardifier and five
noise-chunk runtime markers. The shared normal-noise evaluator and all 63 parameter records are
source-specified/data-only. All 35 locked density-function records are also audited: the three
old-blended records parameterize their dedicated evaluator, while the other 32 are data-only generic
composition trees with no ID-specific dispatch. All 11 material-condition and four material-rule IDs
now own the generic SURFACE predicate, caching and ordered-state algorithms; all seven noise-setting
rule trees are audited data-only compositions of that evaluator. Of the 66 biome records,
`eroded_badlands`, `frozen_ocean`, and `deep_frozen_ocean` now own their source-coded surface
extensions as `Special`; the other 63 remain explicitly data-only records. All four
configured-carver records are bound to cave, Nether-cave or canyon behavior and now own the complete
audited CARVERS dispatcher, shared kernel and family-specific start/path algorithm. The 63-ID
feature-type registry is structurally covered: `no_op`, five composite selectors, two direct
block-write types, `simple_block`, both platform types, `vines`, `sea_pickle`, `blue_ice`, `kelp`,
`block_pile`, `freeze_top_layer`, `end_island`, `glowstone_blob`, `block_blob`, `seagrass`,
`nether_forest_vegetation`, `spring_feature`, `bonus_chest`, `disk`, `basalt_pillar`,
`delta_feature`, `netherrack_replace_blobs`, `underwater_magma`, `spike`, `desert_well`, `bamboo`,
`chorus_plant`, `twisting_vines`, `weeping_vines`, `basalt_columns`, `end_gateway`, `coral_claw`,
`coral_mushroom`, `coral_tree`, `huge_brown_mushroom`, `huge_red_mushroom`, `block_column`,
`large_dripstone`, `speleothem`, `speleothem_cluster`, `end_spike`, `scattered_ore`, `ore`,
`multiface_growth`, `lake`, `monster_room`, `fossil`, `template`, `vegetation_patch`,
`waterlogged_vegetation_patch`, `sculk_patch`, `fallen_tree`, `root_system`, `huge_fungus`, `geode`
and `iceberg` own source-specified configured/placed-feature algorithms. The shared `tree`
orchestration, clearance, clipping, placement primitives and leaf-distance repair plus all nine
trunk families, all 11 blob, bush, fancy, mega-jungle, pine, spruce, acacia, dark-oak, cherry,
mega-pine and random-spread foliage families, the sole mangrove root-placer family, and the
trunk-vine, leaf-vine, pale-moss, cocoa, creaking-heart, beehive, attached-to-leaves,
attached-to-logs, alter-ground and place-on-ground decorator families are source-specified, and all
39 locked tree configurations are data-only audited across 19 canonical base signatures, so that
type is a `BehaviorFamily`. The 30 locked top-level selector records, 32 locked top-level
simple-block records, both platform configured/placed record pairs, all three vines records, both
sea-pickle records, both blue-ice records, all three kelp records, all ten block-pile records, both
freeze-top-layer records, both end-island records and all three glowstone-blob records, both
block-blob records, all 12 seagrass records, all nine Nether-forest-vegetation records, all 13
spring records, the bonus-chest record, all ten disk records, both basalt-pillar records, both delta
records, all four replacement-blob records, both underwater-magma records, both spike records, both
desert-well records, the five explicit bamboo-named records, both chorus-plant records, all five
Nether-vines records, all four basalt-columns records, all three End-gateway records, the warm-ocean
placed wrapper, both huge-mushroom configured records, all three block-column records, both
large-dripstone records, the pointed-dripstone placed wrapper, both dripstone-cluster records, both
End-spike records, all four scattered-ore records, all 68 ore records, all four multiface-growth
records and all four lake-family records and all three monster-room records and all seven
fossil-family records and all ten vegetation-patch-family records and all four sculk-patch-family
records and all ten fallen-tree-family records and all four root-system-family records and all six
huge-fungus-family records and both geode-family records and all four iceberg-family records plus
all 39 tree configured records are exact data-only configurations; no feature-type fallback remains.
Within the 963 worldgen entries, the buried-treasure, Nether-fossil, igloo, swamp-hut,
desert-pyramid, jungle-temple, shipwreck, ruined-portal, ocean-ruin, stronghold, mineshaft,
End-city, fortress, ocean-monument and woodland-mansion structure and structure-set records, all ten
jigsaw structure records and their six selecting sets, all 40 structure-processor lists, all seven
world-preset compositions, 63 noise-parameter records, all 35 density-function records, those 30
selector records, those 32 simple-block records, the four platform records, the three vines records,
the two sea-pickle records, the two blue-ice records, the three kelp records, the ten block-pile
records, the two freeze-top-layer records, the two end-island records and the three glowstone-blob
records, the two block-blob records, the 12 seagrass records, the nine Nether-forest-vegetation
records, the 13 spring records, the bonus-chest record, the ten disk records, the two basalt-pillar
records, the two delta records, the four replacement-blob records, the two underwater-magma records,
the two spike records, the two desert-well records, the five bamboo-named records, the two
chorus-plant records, the five Nether-vines records, the four basalt-columns records, the three
End-gateway records, the warm-ocean placed wrapper, both huge-mushroom configured records, all three
block-column records, both large-dripstone records, the pointed-dripstone placed wrapper, both
dripstone-cluster records, both End-spike records, all four scattered-ore records, all 68 ore
records, all four multiface-growth records and all four lake-family records and all three
monster-room records and all seven fossil-family records and all ten vegetation-patch-family records
and all four sculk-patch-family records and all ten fallen-tree-family records and all four
root-system-family records and all six huge-fungus-family records and both geode-family records and
all four iceberg-family records plus all 39 tree configured records are explicitly data-only inputs,
while the two multi-noise parameter-list IDs are special source dispatches owned by
`WGEN-PIPELINE-001`; remaining worldgen records stay explicitly `Unreviewed` until their codec
audits land. Registry entries outside these gameplay categories remain discoverable in
`registries.json` and must receive a scoped completion entry before the manual can be declared
complete.

## Lookup workflow

```sh
cargo run -p mc-reference --bin mc-ref -- query block minecraft:observer
cargo run -p mc-reference --bin mc-ref -- query block_entity_type minecraft:chest
cargo run -p mc-reference --bin mc-ref -- query item minecraft:bow
cargo run -p mc-reference --bin mc-ref -- query fluid minecraft:flowing_water
cargo run -p mc-reference --bin mc-ref -- query ticket_type minecraft:portal
cargo run -p mc-reference --bin mc-ref -- query worldgen/feature minecraft:no_op
cargo run -p mc-reference --bin mc-ref -- query structure_processor minecraft:rule
cargo run -p mc-reference --bin mc-ref -- query game_rule minecraft:random_tick_speed
cargo run -p mc-reference --bin mc-ref -- coverage
```

Queries print normalized official properties plus classification and rule IDs. Raw reports and jars
remain under `target/mc-reference/26.2/` and are never committed.

Block-item lookup is intentionally more specific than “this item maps to a block.” The catalog
distinguishes ordinary, double-high, bed, sign, standing/wall, water-surface, scaffolding,
game-master and solid-bucket dispatch. These selectors are locked to the official 26.2 item
registrations and resolve before the generic `block_items` selector, so a new or moved special item
cannot silently inherit ordinary placement.
