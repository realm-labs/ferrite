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

The current structural coverage checks 9,078 IDs and reports 309 IDs classified as `Unreviewed`:
75 blocks, 13 items, 37 entity types and 184 worldgen records. All 49 block-entity types now have
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
`BLK-STAINED-GLASS-001` owns all sixteen states 7098..7113, their dye/map/opaque beam colors,
transparent full-cube mechanics, first/raw and later/recursively averaged beacon sections,
Silk Touch loot, coloring/pane recipes, golem exclusion and force-translucent block/item models.
`BLK-CONCRETE-001` owns all sixteen states 15030..15045, their dye-map/full-solid properties,
correct-tool and explosion-survival loot, paired powder-solidification targets, concrete/pickaxe
tags, slow-bouncy sulfur-cube item selection and opaque block/item models.
`BLK-TERRACOTTA-001` owns plain state 12912 and all sixteen dyed states 11444..11459, their
terracotta-specific map colors/full-solid properties, correct-tool self loot, recipes and mason
offers, substrate/badlands/archetype consumers, ambient vegetation sounds, worldgen joins and
opaque block/item models.
`BLK-GLAZED-TERRACOTTA-001` owns all sixteen four-facing state groups 14966..15029, opposite-player
placement and transforms, ordinary dye map colors/full-solid strength 1.4, push-only piston
admission, correct-tool self loot, smelting/unlocks, mason offers, tag/archetype and structure joins,
plus exact patterned blockstate/item projection.
`BLK-QUARTZ-001` owns the five full quartz identities and seven states 11323..11327, 13482 and
23095, including pillar face-axis placement/transforms, their two strength profiles, correct-tool
self loot, processing/unlock graph, guaranteed two-of-two level-five mason offers, slow-bouncy and
bastion joins, plus exact column/cube blockstate and item projection. Quartz and smooth-quartz
stairs/slabs remain with `shape-family`.
`BLK-SANDSTONE-001` owns eight property-free full sandstone identities and states 578..580,
13247..13249, 13481 and 13483, including two strength/map-color profiles, correct-tool self loot,
processing/unlock/template-duplication joins, replacement and slow-bouncy tags, surface, feature,
preset, code-built and 5,833-cell template generation, zombie-village processing and exact
bottom/top, column or all-face cube projection. Sandstone stairs, slabs and walls remain with
`shape-family`.
`BLK-STONE-VARIANT-001` owns granite, polished granite, diorite, polished diorite, andesite and
polished andesite states 2..7, including three map colors, shared 1.5/6.0 full-solid properties,
correct-tool self loot, the complete processing/unlock graph, level-three mason candidates,
base-stone/ore-replacement/bat-support and slow-bouncy tags, feature, ore-vein, buried-treasure and
1,797-cell template joins, plus exact cube-all block/item projection. Their stairs, slabs and walls
remain with `shape-family`.
`BLK-STONE-BRICK-001` owns stone, mossy, cracked and chiseled stone-brick states 7754..7757,
including shared 1.5/6.0 full-solid properties, correct-tool self loot, processing/unlock and shape
joins, the guaranteed level-two mason offer and mason-chest candidate, matching infestation hosts,
stone-bricks/slow-bouncy tags, stronghold/temple/processor and 4,060-cell template joins, plus exact
cube-all block/item projection. Stone-brick stairs, slabs and walls and the four infested identities
remain separately owned.
`BLK-BEACON-STORAGE-001` owns iron, gold, diamond, emerald and netherite block states
2339/2338/5309/9727/21818, including exact physical/tool properties, self loot, beacon and note
membership, the iron-golem transaction, guarded/loved piglin and sulfur-archetype joins, recipes,
unlocks, optional rebalanced armorer trades, acquisition loot, 161 live cells in 30 templates,
processor/code-built generation and matching cube-all projection.
`BLK-RAW-STORAGE-001` owns raw iron, copper and gold block states 32070..32072, including their
map colors, shared 5.0/6.0 full-solid/bass-drum/stone-sound profile, tiered self loot, exact
compacting and unlock records, raw-gold piglin joins, slow-flat membership, copper/iron ore-vein
and carver joins, exhaustive structure-template absence and matching cube-all projection.
`BLK-LAPIS-BLOCK-001` owns lapis-block state 565, including its `LAPIS` map color, default
harp/stone-sound 3.0/3.0 full-solid profile, stone-tier self loot, exact compacting/unlock graph,
rejection from lapis-lazuli-only enchantment and trim gates, slow-bouncy membership, the sole live
cell in first-floor woodland-mansion template `2x2_a3` and matching cube-all projection.
`BLK-REDSTONE-BLOCK-001` owns redstone-block state 11311, including its `FIRE` map color, default
harp/metal-sound 5.0/6.0 full-solid profile, wooden-tier self loot, ordinary/own signal 15, direct
signal zero, explicit nonconductor and exact control-input behavior, compacting/unlock graph, the
two live cells shared by all three ancient-city center templates and matching cube-all projection.
`BLK-AMETHYST-BLOCK-001` owns amethyst-block state 23402, including its purple 1.5/1.5 full-solid
profile, wooden-tier self loot, projectile chime, tagged crystal-footstep accumulator, six-face
sculk resonance admission, shard recipe/unlock, slow-bouncy membership, geode inner-layer identity,
zero structure-template cells and matching cube-all projection.
`BLK-BUDDING-AMETHYST-001` owns budding state 23403 and all 48 directional/waterlogged bud and
cluster states, including exact 1-in-5/six-face stage growth, support and water transitions,
light/sound profiles, empty/Silk Touch/Fortune loot, inherited chimes, geode placements, zero
structure cells and directional models.
`BLK-CALCITE-SMOOTH-BASALT-001` owns states 27160 and 32069, their exact registrations, tool,
loot and material-sound profiles, the smooth-basalt smelting/unlock, slow-bouncy and replacement
memberships, calcite feature/surface joins, geode middle/outer layers, 205 smooth-basalt cells in
the six ancient-city entrance templates, calcite's template absence and both cube-all models.
`BLK-DEEPSLATE-001` owns base-deepslate states 30416..30418, clicked-face and transformed axes,
its 3.0/6.0 full-solid/tool/sound profile, Silk/self versus cobbled loot, the exact smelting and
17-way cutting graph, direct/composed tag consumers, ore/support/surface/flat/retrogen identities,
11,508 raw ancient-city cells with reachable/processor boundaries and weighted column models.
`BLK-DEEPSLATE-MASONRY-001` owns the seven property-free full-block masonry states, their 3.5/6
tool/sound/loot profiles, 63 exact-ID and eight cobbled tag-keyed recipes, replacement tags,
44,739 raw structure cells, ordered ancient-city rot/cracking and seven opaque cube-all models.
`BLK-DRIPSTONE-BLOCK-001` owns property-free state 30208, its exact physical/tool/sound and self-
loot profile, shaped recipe, level-three mason, sculk/multiface and slow-bouncy selectors, the
constructor-supplied live pointed-growth substrate, all three dripstone-cave feature result joins,
zero bundled structure cells and exact cube-all projection.
`BLK-HONEYCOMB-BLOCK-001` owns property-free state 21817, its 0.6-strength coral-sound full-cube
profile, tool-independent self loot, four-honeycomb recipe/unlock, sticky sulfur-cube item
selection, zero generation/structure joins and exact cube-all projection.
`BLK-BRICKS-001` owns property-free state 2340, its 2/6 correct-pickaxe red/base-drum profile,
eight masonry/pattern recipes and unlocks, slow-bouncy item selection, 2,558 raw cells across 31
trail/ocean/village templates and exact cube-all projection.
`BLK-PACKED-MUD-001` owns property-free state 7758, its 1/3 tool-independent dirt/harp profile,
two mud recipes/unlocks, buoyant regular item selection, 68 raw trail cells plus the houses/roads
below-0.1 mud-bricks aging output and exact cube-all projection.
`BLK-MUD-BRICKS-001` owns state 7759, its 1.5/3 correct-pickaxe light-gray/base-drum profile, seven
masonry recipes/unlocks, slow-bouncy selection, 3,870 raw trail cells plus 19 connector finals and
aging, and distinct north/west-mirrored world versus cube-all item projection.
`BLK-PURPUR-BLOCK-001` owns state 14712, its 1.5/6 correct-pickaxe magenta/base-drum profile,
eight recipe joins, seven unlocks, slow-bouncy selection, the End-city advancement icon and 2,233
raw cells across 20 End-city inputs, including the dead `tower_floor`.
`BLK-NETHER-WART-BLOCK-001` owns state 14846, its 1/1 tool-independent red/harp/Wart-sound
profile, self loot, nine-wart compacting, 0.85f composting, nested slow-sliding/tutorial/carver
tags, three exact spawn exclusions, crimson surface/fungus and weeping-vines roles, zero structure
cells and exact cube-all projection.
`BLK-WARPED-WART-BLOCK-001` owns state 20959, its 1/1 tool-independent warped/harp/Wart-sound
profile, self loot and recipe absence, 0.85f composting, the shared nested tags, paired warped
surface/fungus and twisting-support roles, three spawn-veto absences, zero structure cells and
cube-all projection.
`BLK-NETHER-SPROUTS-001` owns state 20961, its offset no-collision vegetation profile, exact
14-identity support closure and support-loss removal, shears-only loot, 0.5 composting, combined
player footsteps, enchanting/tree/mushroom tag consumers, warped vegetation and nylium-bonemeal
roles, zero structure cells and untinted cross/flat-item projection.
`BLK-NETHER-ROOTS-001` owns root states 20960/21031 and potted states 21829/21828, the common
14-identity support closure, support-loss self drop, 0.65 composting, combined footsteps,
enchanting/tree/mushroom/Enderman joins, exact pot insertion/extraction and two-pool loot,
crimson/warped vegetation weights, the soul-sand-valley crimson patch and distinct root/item/pot
projections.
`BLK-NETHER-WART-001` owns age states 9447..9450, soul-sand support, one-in-ten selected-tick
growth and bonemeal absence, age/Fortune/per-unit explosion loot, 0.65 composting, the
water-to-awkward brewing edge, two recipes/unlocks, plant-seed criterion, cleric and Nether-bridge
acquisition, 20 age-zero fortress-room plus 12 mature bastion-template cells and exact three-stage
crop/flat-item projection.
`BLK-NETHER-STEM-001` owns 24 axis states across eight crimson/warped stem and hyphae identities,
clicked-face placement and state-preserving axe stripping, nonflammable `logs` joins across leaf,
parrot, tree, lava-pool, blending, tutorial and sulfur equipment consumers, thirteen recipes and
ten unlocks, exact huge-fungus stem generation, zero structure cells and axis-aware cube columns.
`BLK-CORAL-BLOCK-001` owns property-free states 15137..15146 across five live/five dead coral
blocks, six-neighbor water scanning and delayed live-to-dead scheduling, correct-pickaxe Silk/dead
loot, live-only sea-pickle and warm-ocean tag consumers, five common trades, fast-flat equipment,
zero structure cells and exact cube-all projection.
`BLK-CORAL-PLANT-001` replaces 30 generic simple-waterlogged classifications with exact
live/dead upright, floor-fan and wall-fan semantics: 120 states, support-first updates,
center/adjacent-water drying, standing-and-wall items, Silk-only loot, coral-feature/underwater-
bone-meal tag consumers, zero structure cells and untinted cross/fan projection.
`BLK-FLOWER-POT-001` replaces 37 explicit block fallbacks with the empty pot and 36 non-root
filled forms: property-free IDs, support-free collision, code-built content and failure ordering,
two-pool loot/crafting/acquisition, eyeblossom random ticks, warped-fungus hoglin repulsion, 120
raw structure cells and exact tinted/emissive/bespoke projection.
`BLK-COPPER-FULL-001` replaces 24 explicit block fallbacks with the property-free full, cut and
chiseled copper collections: exact physical/loot/note properties, cross-collection weathering,
honeycomb/axe order, full-block-only golem construction, recipes/advancements, slow-flat selection,
23,354 trial-chamber cells and waxed model aliases.
`BLK-SAPLING-001` replaces eight explicit block fallbacks with the ordinary tree-sapling family:
two-stage support-bound survival, random and bone-meal growth gates, exact small/flower/mega
feature selection and cleanup, loot/trade/compost/fuel joins, 45 worldgen survival predicates,
60 structure cells and stage-insensitive cross projection.
`BLK-BAMBOO-001` replaces the bamboo-sapling fallback and removes bamboo stalk from the broad
fire-fuel family. It owns their shared support and item identity, dual-form placement, distinct
random/bone-meal growth, 16-block leaf/age/stage column, loot/recipe/panda/fuel/fire joins,
zero/0.2-podzol worldgen paths and exact cross/multipart/item projection.
`BLK-ANCIENT-DEBRIS-001` replaces the ancient-debris fallback with one exact full-cube owner. It
joins diamond-tier self loot, fire-resistant item damage, smelting/blasting and discovery,
five bastion chest entries, slow-flat selection, both scattered Nether ore paths, zero structure
cells and exact cube-column/item/tab projection.
`BLK-STEM-CROP-001` replaces four block and two item fallbacks with the coupled pumpkin/melon stem
family. It owns support and dry-farmland retention, exact age and fruit-attachment transitions,
crop-speed/crowding RNG, bone meal, binomial seed loot, recipes/acquisition/trades/compost/animal
tags, fungus and village generation, 111 template cells and exact stage/facing/tint/item projection.
`BLK-OVERWORLD-CROP-001` replaces four block and six remaining item fallbacks with wheat, carrots,
potatoes and beetroots plus their seven coupled items. It owns exact age states, light/support
gates, shared crop-speed/crowding RNG, beetroot's outer gate and scaled bone meal, Ravager and
farmer-villager transactions, crop/secondary loot, recipes, trades, composting, food/effect and
animal joins, ten farm processors, all 722 raw wheat template cells and exact staged models.
`BLK-TORCHFLOWER-CROP-001` replaces the torchflower-crop and seed-item fallbacks while promoting
the coupled mature flower/item into one exact family. It owns logical-age-two flower replacement,
outer/shared growth draws, deterministic crop bone meal, flower spreading, Ravager/farmer,
loot/recipe/advancement/sniffer/compost/animal/bee/pot/fire/fungus joins, zero template cells and
exact crop/flower/item projection.
`BLK-PITCHER-CROP-001` promotes both already-audited double-plant blocks and replaces the remaining
pitcher-pod item fallback. It owns pod-only placement, lower-only five-age growth, the age-three
two-cell transition, deterministic half-resolving bone meal, double-plant break/Ravager/farmer
behavior, half-sensitive loot, dye/advancement/sniffer/compost/animal/bee/fire joins,
fungus/tree/mushroom replacement, zero template cells and exact crop/plant/item projection.
`BLK-SWEET-BERRY-BUSH-001` promotes the already-audited fire block and replaces the sweet-berries
item fallback. It owns four-stage support/growth/bone-meal/harvest behavior, movement slowdown,
fall reset and damage, bee/fox/Ghast joins, food/advancement/trade/chest/compost/fire data,
berry-feature/taiga-decor/fungus generation, zero raw template cells and exact stage/item
projection.
`BLK-CAVE-VINES-001` promotes both blocks from the broad fire family and replaces the glow-berries
item fallback. It owns downward support and head/body conversion, head-only growth, segment-local
bone meal and bee lighting, harvest/break loot, climb/glide/fox/food/chest/compost/fire joins,
direct/moss/fungus generation, zero raw template cells and exact age/berry/light projection.
`BLK-CHORUS-001` replaces both block and both fruit fallback IDs while taking the two block items
from the generic item family. It owns connection/support repair, live-flower rise/branch/death,
projectile destruction, loot, random-teleport consumption, recipes/bee/progression joins, recursive
End generation, fungus replacement, zero raw template cells and exact multipart/live/dead items.
`ITM-STEW-001` replaces the bowl and four filled-food fallbacks. It owns their default components,
player remainder and suspicious-effect listeners, 22 recipes, 17 flower variants, brown-mooshroom
charge/milking, rabbit-stew wolf feeding, loot/trades/progression and exact model/creative projection.
`ITM-BUNDLE-001` replaces the plain and 16 dyed bundle fallbacks. It owns fractional and nested
capacity, ordered click/held/destruction transfers, transient client/server selection, component-
preserving recoloring, eight village pools, progression, persistence and exact tooltip/model/tab
projection.
`ITM-BOAT-001` replaces twenty boat, chest-boat and raft fallbacks. It owns exact item/entity
mapping and placement, passenger/container selection, 27-slot loot/persistence and removal order,
matching destruction items, recipes, fuel, fisherman trades, goat progression, dispenser
delegation and item/entity/menu/tab projection while common vehicle physics retains its owner.
`ITM-POTTERY-SHERD-001` replaces all 23 pottery-sherd fallbacks. It owns their uncommon plain-item
identities, pattern IDs, tags, four-face recipe and cracked recovery joins, twenty weighted
archaeology entries, three fixed trial-chamber-only identities, two advancement matrices,
persistence and exact item/pattern/tab projection while decorated-pot, loot, archaeology and
jigsaw algorithms retain their owners.
`ITM-SMITHING-TEMPLATE-001` replaces all 19 smithing-template fallbacks. It owns exact subclass,
rarity, tooltip and screen-hint identities, nineteen duplication recipes, eighteen trim patterns,
twelve netherite transforms, configured loot/entity/vault acquisition, forty-nine unlocks, two
trim advancements, persistence and item/tab/equipment projection while generic smithing, loot,
entity and worldgen algorithms retain their owners.
`ITM-HARNESS-001` replaces all 16 harness fallbacks and gives Happy Ghast its exact entity family.
It owns colored equippable assets, adult direct/dispenser equip, leash/shear and guaranteed-drop
order, live-tag temptation, four-passenger mount/control, thirty-two recipes/unlocks, persistence
and item/equipment/tab projection while generic dispenser, death, passenger, AI and protocol
algorithms retain their owners.
`ITM-MINECART-001` replaces all six `MinecartItem` fallbacks. It owns held/dispenser placement,
exact subtype interaction/activation joins, matching-versus-ordinary destruction/pick results,
five recipes and six unlock joins, mineshaft chest-cart acquisition, persistence/reload and exact
item/entity/menu/tab projection while generic rail, container, loot, command, worldgen and
protocol algorithms retain their owners.
`ITM-STEERING-STICK-001` replaces both `FoodOnAStickItem` fallbacks and gives pig/strider an exact
steering family. It owns controller/temptation selectors, boost-before-durability and patched-rod
break order, two recipes/unlocks, Nether progression, persistence/reload and handheld-rod/bar/tab
projection while generic durability, recipe, advancement, AI, motion and protocol algorithms
retain their owners.
`ITM-SPEAR-001` replaces all seven spear fallbacks with one component-selected family. It owns the
minimum-charge multi-target STAB and held speed-gated kinetic scans, tier constants, contact/feedback/
criterion quirks, Lunge join, mob equipment/AI, recipes/repair/recycling/fuel/loot and dual-context
item/animation projection while generic damage, enchantment, durability, recipe, loot, AI and
protocol algorithms retain their owners.
`ITM-NAUTILUS-ARMOR-001` replaces all five nautilus-armor fallbacks and splits normal/zombie
nautilus from the broad animal family. It owns exact nondamageable body attributes, live-tagged
direct/dispenser/menu admission, leash and body-before-saddle shear order, guaranteed recovery,
zombie sunlight protection, recipes/loot/unlocks, persistence/reload and item/body/menu/tab
projection while generic interaction, container, damage, loot and protocol algorithms retain their
owners.
`ITM-EGG-001` replaces both egg fallbacks and promotes ordinary egg from the broad projectile
family. It owns three exact temperate/cold/warm component identities, held/dispenser launch,
zero-damage impact and 0/1/4 variant-preserving hatch order, chicken laying, tag recipes/unlock,
persistence/reload and flat/projectile/particle/tab projection while generic projectile, damage,
loot, crafting, advancement and protocol algorithms retain their owners.
`ITM-BAKED-POTATO-001` replaces the Baked-Potato fallback. It owns its food components, three
Potato cooking outputs/unlocks, five loot tables and normal/ominous fixed Spawner joins, two
Rabbit-Stew sinks, Wandering Trader purchase, code-built composting, Balanced Diet and exact
item/tab projection while machine, loot, spawner, crafting, merchant, Composter and progression
algorithms retain their owners.
`ITM-BEEF-001` replaces the Raw-Beef and Steak fallbacks. It owns both food identities, recursive
Wolf-food joins, fire/smelts-loot bovine conversion, three cooking outputs/unlocks, village and
ominous Trial loot, level-three Butcher purchase, adult Butcher hero gift, Balanced Diet and exact
item/tab projection while death, loot, machine, spawner, merchant, Villager, Wolf and progression
algorithms retain their owners.
`ITM-BONE-001` replaces the Bone fallback. It owns six skeletal drop rows, five base chest tables
and three Trade-Rebalance replacements, fishing junk, the Bone-Meal recipe/unlock, exact Wolf
taming and BegGoal joins, tame advancements and direct item/tab projection while death, loot,
fishing, structure, crafting, tame, AI and progression algorithms retain their owners.
`ITM-BOOK-FAMILY-001` replaces the Book, Enchanted Book, Writable Book and Written Book fallbacks.
It owns their exact defaults, recipes/cloning, writing/opening, enchanting/grindstone/anvil,
shelf/lectern, base and Trade-Rebalance loot, Librarian offers and client projection while generic
recipe, protocol, enchantment, block-container, loot, merchant and structure algorithms retain
their owners.
`ITM-CHICKEN-001` replaces the Raw- and Cooked-Chicken fallbacks. It owns their food/effect state,
fire-converting Chicken drop, cooking, Cat/Trial/Butcher gifts and rewards, two Butcher offers,
Wolf-food join, Balanced Diet and exact client projection while generic use, effect, death, loot,
machine, mob AI, spawner, merchant and progression algorithms retain their owners.
`ITM-MUTTON-001` replaces the Raw- and Cooked-Mutton fallbacks. It owns their food state,
meat-first fire-converting Sheep drop, cooking, village loot, guaranteed Butcher purchase, hero
gift, Wolf-food join, Balanced Diet and exact client projection while generic use, death, loot,
machine, structure, merchant, mob AI and progression algorithms retain their owners.
`ITM-PORKCHOP-001` replaces the Raw- and Cooked-Porkchop fallbacks. It owns their food state,
Pig/Hoglin fire-converting drops, cooking, village/Bastion loot, two Butcher offers and hero gift,
exact Piglin-food consumption, Wolf-food join, Balanced Diet and client projection while generic
use, death, loot, machine, structure, merchant, mob AI and progression algorithms retain owners.
`ITM-RABBIT-MATERIAL-001` replaces the Raw Rabbit, Cooked Rabbit and Rabbit Hide fallbacks. It owns
their food or inert defaults, the Hide/meat/Rabbit Foot ordered death join, cooking, Rabbit Stew
and Leather recipes, Cat and Butcher gifts, Butcher and Leatherworker trades, Wolf-food join,
Balanced Diet and split client projection while generic use, death, loot, recipe, machine,
merchant, mob AI and progression algorithms retain owners.
`ITM-COD-001` replaces the Raw- and Cooked-Cod fallbacks. It owns their food state, five
Cod-bearing entity tables, fishing/chest/gift acquisition, cooking, two Fisherman records,
raw-only Cat/Ocelot and shared Dolphin/Wolf/Nautilus paths, progression and exact client
projection while generic use, death, loot, fishing, machine, structure, merchant, mob AI and
progression algorithms retain owners.
`ITM-SALMON-001` replaces the Raw- and Cooked-Salmon fallbacks. It owns their food state,
Salmon/Polar-Bear death joins, fishing/chest/gift acquisition, cooking, level-two/three Fisherman
records, raw-only Cat/Ocelot and shared Dolphin/Wolf/Nautilus paths, progression and exact client
projection while generic use, death, loot, fishing, machine, structure, merchant, mob AI and
progression algorithms retain owners.
`ITM-TROPICAL-FISH-001` replaces the Tropical-Fish fallback. It owns invariant direct death,
Guardian rare-fish and fishing acquisition, the sole level-four Fisherman record, Dolphin/Wolf/
Nautilus food joins, negative Cat/Ocelot/Axolotl/taming selectors, progression and exact client
projection while entity variants, buckets and generic use, death, loot, fishing, merchant, mob AI
and progression algorithms retain owners.
`ITM-BREAD-001` replaces the Bread fallback. It owns player and Farmer crafting, eighteen chest
records, normal Trial consumables, Farmer sale/gift records, code-built Villager pickup/food/
sharing/breeding behavior, direct and automated composting, progression and exact client projection
while generic use, crafting, loot, structure, Trial, merchant, Villager, breeding, block and client
algorithms retain owners.
`ITM-COOKIE-001` replaces the Cookie fallback. It owns the exact shaped recipe, guaranteed
level-three Farmer sale, Farmer gift, direct Parrot-poison tag and ordered remainder/effect/lethal-
damage interaction, composting, progression, Allay-advancement icon and exact client projection
while generic use, crafting, merchant, gift, effect, damage/death, mob, block and client algorithms
retain owners.
`ITM-PUMPKIN-PIE-001` replaces the Pumpkin-Pie fallback. It owns the three-egg-identity shapeless
recipe and unlock, Taiga-village chest row, level-two Farmer sale, Farmer gift, guaranteed
composting, progression and exact client projection while generic use, crafting, loot/village,
merchant, gift, block and client algorithms retain owners.
`ITM-ROTTEN-FLESH-001` replaces the Rotten-Flesh fallback. It owns Hunger-bearing consumption,
nine entity drops, nine rows across eight chest families, fishing junk, cat morning gift,
guaranteed level-one Cleric buying, nested Wolf-food behavior, progression and exact client
projection while generic use/effect, death, loot/fishing/structure, cat, merchant, animal and
client algorithms retain owners.
`ITM-BRICK-001` replaces the Brick fallback. It owns Clay-Ball smelting, Bricks/Flower-Pot and
simple/special Decorated-Pot recipes, two archaeology rows, guaranteed level-one Mason selling,
blank-face component mapping and cracked-pot recovery, progression and exact client projection
while generic furnace, crafting, archaeology, merchant, pot and client algorithms retain owners.
`BLK-CLAY-001` replaces both the Clay-block and Clay-Ball fallbacks. It owns the sole block state,
Silk/four-ball loot, Mud drip conversion, compacting and Furnace recipes, village chest/trade/gift,
Trail archaeology and raw cells, direct tag/archetype selectors, four generation records and exact
client projection while generic block, item, processing, AI, village, worldgen and client
algorithms retain owners.
`ITM-COAL-001` replaces the Coal and Charcoal fallbacks. It owns code-built Furnace and
Furnace-Minecart fuel joins, twelve recipes and twelve unlock records, ore/Campfire/Wither-Skeleton
acquisition, thirteen chest and three archaeology rows, five profession sets, ore/fossil
generation joins and exact client projection while generic processing, loot, vehicle, merchant,
structure, worldgen and client algorithms retain owners.
`BLK-COCOA-001` replaces the Cocoa-block and Cocoa-Beans fallbacks. It owns the twelve age/facing
states, custom block-item key, support/placement/growth/Bone-Meal/shape/path joins, age-sensitive
loot, two recipes and unlocks, composting, natural small-jungle-tree generation and exact client
projection while generic placement, updates, loot, crafting, Composter, path, worldgen and client
algorithms retain owners.
`BLK-MELON-001` replaces the Melon-block and Melon-Slice fallbacks. It owns property-free state
8333, the Silk/self versus capped slice table, food and Balanced-Diet path, three recipes/unlocks,
loose/block composting, guaranteed Farmer sink, stem/natural/pile/17-template-cell acquisition and
exact cube-column/flat projection while generic block, loot, food, crafting, merchant, stem,
worldgen and client algorithms retain owners.
`BLK-NETHER-BRICKS-001` replaces the Nether-Brick item and base, Cracked and Chiseled
Nether-Bricks block fallbacks. It owns their exact registrations, smelting/barter acquisition,
thirteen-recipe graph, twelve family unlocks, correct-tool self loot, slow-bouncy block items,
base-only Fortress terrain/spawn/icon joins, Delta/Basalt protection and exact projection while
generic processing, loot, Piglin, block, spawning, structure, feature and client algorithms retain
owners.
`BLK-RESIN-MATERIAL-001` replaces Resin Brick and the Block, Bricks and Chiseled-Bricks
fallbacks. It owns compacting/smelting/masonry/Heart recipes, Creaking/Mansion Clump inputs,
correct-tool loot, fast-flat items, live Resin trim material and exact projection while generic
multiface, processing, Smithing, loot, structure and client algorithms retain owners.
`ITM-QUARTZ-001` replaces the loose Nether-Quartz fallback. It owns the exact item/component,
Nether-Quartz-Ore loot and processing joins, regular/Delta ore scheduling, Piglin/Bastion/Mason
acquisition, six crafting consumers and seven unlocks, live Quartz trim material and exact
projection while generic breaking/XP, processing, crafting, Smithing, loot, merchant, worldgen
and client algorithms retain owners.
`ITM-DIAMOND-001` replaces the loose Diamond fallback. It owns both Ore loot/processing joins,
ordinary/fossil generation, seventeen loot/archaeology rows, 56 recipes and progression, repair,
Beacon, baseline/rebalanced trade, normal/darker trim assets and exact projection while generic
breaking/XP, processing, crafting, special-recipe, loot, merchant, Beacon, worldgen and client
algorithms retain owners.
`ITM-EMERALD-001` replaces the loose Emerald fallback. It owns both Ore loot/processing and Goat
joins, ten-biome mountain generation, 32 direct loot tables, 24 recipes, Beacon, every
baseline/rebalanced merchant role, two persisted Igloo offers, trim material and exact projection
while generic breaking/XP, Goat, processing, crafting, loot, merchant, Beacon, worldgen and client
algorithms retain owners.
`ITM-FEATHER-001` replaces the loose Feather fallback. It owns Chicken/Parrot death and Cat-gift
joins, three chest rows, Arrow/Brush/Writable-Book and burst-Firework recipes, the guaranteed
Fletcher purchase, zero-template census and exact flat/head projection while generic death, Cat
AI, loot, crafting, Firework, merchant, structure and client algorithms retain owners.
`ITM-FIREWORK-STAR-001` replaces the Firework-Star fallback. It owns default-versus-explosion
component state, base/fade/Rocket special-recipe joins, copy/replace/omit asymmetries, tooltip and
two-layer tint projection and complete acquisition absences while generic crafting, Rocket,
component, packet and client algorithms retain owners.
`ITM-FLINT-001` replaces the loose Flint fallback. It owns Gravel Silk/Fortune/explosion loot,
two chest rows, three recipes and direct unlocks, five baseline merchant offers, zero-template
census and exact flat projection while generic breaking, loot, crafting, merchant, structure and
client algorithms retain owners.
`ITM-GLOWSTONE-DUST-001` replaces the loose Glowstone-Dust fallback. It owns Glowstone and Witch
loot, every natural/Bastion/trade block-to-Dust join, three crafts, ten brewing edges, the exact
Dust/block template census and flat projection while generic breaking, death, loot, crafting,
brewing, merchant, worldgen, structure and client algorithms retain owners.
`BLK-SOUL-SAND-001` owns state 6998, its split full-selection/14-of-16 collision mechanics,
postprocess-above callback, eleven block-tag and two item-tag consumers, Soul Speed and sulfur-cube
roles, recipes/loot, normal Nether generation and full-height cube model.
`BLK-MAGMA-001` owns state 14845, its full-cube/emission-3 yet full-bright projection, hot-floor
caller, seven block-tag and one item-tag consumers, hot sulfur-cube archetype, acquisition and
ore/underwater/delta/portal/basalt/spring/bastion generation roles.
`BLK-LAVA-CAULDRON-001` owns state 9464, its hollow shell/full content shape, emission 15,
ordered lava contact, four bucket paths, comparator 3, path/POI selectors, cauldron-only loot and
full lava model without a dedicated item.
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
