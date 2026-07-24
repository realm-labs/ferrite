# Data Reload Root Inventory

**Surface:** `SURFACE-DATA-RELOAD-001`
**Status:** `InProgress`
**Primary evidence:** `OFF-SERVER-001`

This inventory owns bootstrap and data-pack reload from pack selection through publication to live
worlds and unmodified clients. Generated registry and bundled-data reports lock the input identities
and values; they do not by themselves specify listener dependencies, validation, failure behavior or
the point at which a replacement snapshot becomes observable.

| Reload family | Locked source roots | Existing semantic owners | Remaining audit |
|---|---|---|---|
| Pack discovery, selection and feature flags | `net.minecraft.server.packs.repository.PackRepository#reload`, `net.minecraft.server.packs.repository.PackRepository#setSelected`, `net.minecraft.server.MinecraftServer#configurePackRepository`, `net.minecraft.world.level.WorldDataConfiguration#enabledFeatures` | This surface owns selection and admission; content and world-generation leaves own the behavior parameterized by the accepted packs and flags. | Enumerate required, missing, incompatible, explicitly disabled and auto-enabled pack branches; preserve selection order, feature closure, warning/error outcomes and persisted world configuration. |
| Bootstrap and worldgen registries | `net.minecraft.resources.RegistryDataLoader#load` | `WGEN-002`, `WGEN-003` and the locked catalog own registry-selected behavior and concrete values; `BLK-JIGSAW-001` fixes the operator block's required generation-time template-pool lookup, `BLK-STRUCTURE-001` fixes named structure-template manager lookup after the registry snapshot is live, and `BLK-TEST-INSTANCE-001` fixes configured test-instance holder lookup plus its template/rotation/padding/required fields. | Partition both overloads by bootstrap/network source; audit codec errors, holder/reference resolution, lifecycle metadata, registry layering, duplicate keys and all-or-nothing visibility. |
| Reloadable registries and loot | `net.minecraft.server.ReloadableServerRegistries#reload`, `net.minecraft.server.ReloadableServerRegistries$LoadResult#lookupWithUpdatedTags`, `net.minecraft.server.ReloadableServerRegistries$Holder#getLootTable`, `net.minecraft.world.level.storage.loot.LootDataType#runValidation` | `ITM-006`, `ITM-007`, `ITM-CHEST-001`, `ITM-HOPPER-001`, `ITM-DISPENSER-001` and loot/content leaves own caller context and evaluation after a validated registry snapshot is published. | Enumerate reloadable registry keys and validation contexts; audit missing/default loot tables, recursive references, validation diagnostics, tag-updated lookup identity and rejected-load retention. |
| Listener construction and dependency barrier | `net.minecraft.server.ReloadableServerResources#loadResources`, `net.minecraft.server.ReloadableServerResources#listeners`, `net.minecraft.server.packs.resources.SimpleReloadInstance#create`, `net.minecraft.server.packs.resources.ReloadInstance#done` | Individual rows below own listener semantics; this family owns prepare/apply dependency edges, executors, barriers and aggregate completion. | Recover the exact ordered listener list and dependency DAG, including parallel preparation, apply serialization, profiler-visible ordering, cancellation and the first exceptional completion. |
| Tags and component rebinding | `net.minecraft.tags.TagLoader#load`, `net.minecraft.tags.TagLoader#build`, `net.minecraft.tags.TagLoader#loadTagsForExistingRegistries`, `net.minecraft.server.ReloadableServerResources#updateComponentsAndStaticRegistryTags` | `BLK-SCULK-SENSOR-001` owns vibration, ignore-sneaking, damping, occlusion and resonator membership effects; `BLK-BEACON-001` owns live base/payment membership consumption; `BLK-SIGN-001` owns standing/wall/ceiling/wall-hanging aggregation and the live hanging-sign support/orientation branch; `BLK-SKULL-001` owns live `wither_summon_base_blocks` membership during full/base-pattern checks; `BLK-STRUCTURE-VOID-001` owns the locked structure-void membership in `replaceable` alongside its code-level replaceable property; `ITM-HOPPER-001` owns the `does_not_block_hoppers` loose-item gate; `ITM-DISPENSER-001` owns the reloadable sulfur-cube swallowable item tag and all 12 archetype delegates; other tag-backed leaves own their memberships, and component-bearing item rules own derived component behavior. | Inventory every remaining tag directory and pending-tag bind; audit optional/required entries, cycles, missing references, replacement versus merge, component reinitialization and holder identity seen by existing objects. |
| Recipes, functions and advancements | `net.minecraft.world.item.crafting.RecipeManager#prepare`, `net.minecraft.world.item.crafting.RecipeManager#apply`, `net.minecraft.world.item.crafting.RecipeManager#finalizeRecipeLoading`, `net.minecraft.server.ServerFunctionLibrary#reload`, `net.minecraft.server.ServerAdvancementManager#apply` | `ITM-004` owns recipe matching and crafting results; command/function and progression leaves own execution and advancement state. | Enumerate resource decode, duplicate/replacement and feature-filter branches; audit function compilation context, advancement parent/display resolution, recipe cache/index rebuild and active-player progression reconciliation. |
| Atomic server publication and live refresh | `net.minecraft.server.MinecraftServer#reloadResources`, `net.minecraft.server.players.PlayerList#reloadResources`, `net.minecraft.commands.Commands#sendCommands` | Simulation and world owners consume the published snapshot; PlayerLifecycle and client projection own observable refresh results. | Locate every field swap and post-publication callback; prove whether worlds observe one coherent snapshot, what remains installed on each failure point, and the order of recipes, advancements, functions, commands, tags and player refresh. |
| Active-session reconfiguration and convergence | `net.minecraft.server.network.ServerGamePacketListenerImpl#switchToConfig`, `net.minecraft.server.players.PlayerList#reloadResources` | Configuration, reconfiguration and live-tag protocol families own wire state and packet layouts. | Audit admission during reload, play-to-configuration transition gates, registry/tag snapshot selection, acknowledgement ordering, disconnect/failure branches and convergence for players joining or changing dimension concurrently. |

## Current boundary conclusions

- Pack order and the enabled feature set are behavior inputs. A compatible implementation may use a
  different pack container, but it must accept, reject and prioritize the same locked inputs.
- Reloadable registries are prepared before the resource listener aggregate is constructed. The
  aggregate exposes a completion future; this does not yet prove atomicity for every later server
  field swap or player refresh, so failure behavior remains explicitly open.
- Loot tables, predicates and modifiers are reloadable registry content in 26.2. They must not be
  modeled as an independent legacy loot manager.
- Existing worlds, objects and sessions can retain holder, tag, command or recipe views. Successful
  data decoding alone is therefore insufficient without publication and convergence checks.
- `BLK-AIR-001` owns the live `air` and `replaceable` memberships shared by all three air states and
  the locked `parrots_spawnable_on` membership held only by ordinary air.
- `BLK-BEDROCK-001` owns the live dragon/wither/wind-charge protection, feature replacement,
  geode-invalid and End infiniburn memberships; direct identity and registered-property branches
  remain code-locked.
- `BLK-REINFORCED-DEEPSLATE-001` owns the live dragon/wither and feature-replacement memberships
  plus wind-charge nonmembership; its registered properties and piston identity gate remain
  code-locked.
- `BLK-TINTED-GLASS-001` owns the live `impermeable` membership and its negative boundary: the only
  locked consumer is invoked with the beehive's state, so current vanilla code never tests tinted
  glass there. Its registered light/spawn properties and golem identity gate remain code-locked.
- `BLK-GLASS-001` owns the live `impermeable` and `smelts_to_glass` memberships. The former has the
  same beehive caller-state non-interaction; the latter selects smelting inputs. Registered light/
  spawn properties, Silk Touch loot and the golem identity gate remain code-locked.
- `BLK-STAINED-GLASS-001` owns all sixteen reloadable Silk Touch loot tables, coloring/pane recipes,
  their unlock advancements and `impermeable` memberships. Registration, dye/beam colors, light
  hooks, recursive beacon averaging and class-wide golem exclusion remain code-built.
- `BLK-CONCRETE-001` owns all sixteen reloadable self-loot tables, `concrete` block/item and
  pickaxe memberships, powder recipes and the slow-bouncy sulfur-archetype join. Registration,
  dye-map/full-solid properties and powder-to-concrete target references remain code-built.
- `BLK-TERRACOTTA-001` owns seventeen self-loot tables, plain/dyed/glazed/template recipes and
  unlock advancements, sixteen non-glazed mason records plus their level-four tag/set, the broad
  terracotta block/item joins, seven-member badlands subset, substrate/replacement memberships and
  slow-bouncy sulfur-archetype join. Registration, physical properties, map-color selectors,
  client RNG gates and surface-band construction remain code-built.
- `BLK-GLAZED-TERRACOTTA-001` owns sixteen self-loot tables, matching smelting/unlock records,
  sixteen glazed mason records plus their shared level-four tag/set, glazed-terracotta block/item
  and pickaxe memberships, and the slow-bouncy sulfur-archetype join. Registration, facing,
  transforms, piston reaction, structure palettes and patterned client models remain code-built or
  resource-pack state rather than data-pack reload inputs.
- `BLK-QUARTZ-001` owns five self-loot tables, its full-block crafting/stonecutting/smelting and
  unlock records, two level-five mason records plus their exact two-entry tag/set, pickaxe
  memberships and five direct slow-bouncy item memberships. Registration, pillar axis transforms,
  bastion palette and client models remain code-built or resource-pack state; shape-output records
  retain their `shape-family` ownership.
- `BLK-SANDSTONE-001` owns eight self-loot tables, its full-block and shape-joining
  crafting/stonecutting/smelting/template-duplication and unlock records, pickaxe/carver/sculk
  memberships, eight direct slow-bouncy item memberships, and sandstone-selecting feature,
  processor, preset and noise-setting records. Registration, code-built well/pyramid/buried-
  treasure paths, structure palettes and client models remain code-built or resource-pack state;
  shape-output records retain their `shape-family` ownership.
- `BLK-STONE-VARIANT-001` owns six self-loot tables, its full-block and shape-joining
  crafting/stonecutting and unlock records, six level-three mason records plus the shared tag/set,
  pickaxe/base-stone/ore-replacement/composed-support memberships, six direct slow-bouncy item
  memberships and the matching ore/attachment/spring feature records. Registration, ore-vein and
  buried-treasure selectors, structure palettes and client models remain code-built or
  resource-pack state; shape-output records retain their `shape-family` ownership.
- `BLK-STONE-BRICK-001` owns four self-loot and four matching infested-host loot tables, its
  crafting/stonecutting/smelting/lodestone/shape-output and unlock records, the level-two mason
  record plus shared tag/set, the village-mason chest table, block/item stone-bricks and pickaxe
  memberships, and four slow-bouncy item memberships. Registration, host maps, stronghold/temple/
  processor selectors, structure palettes and client models remain code-built or resource-pack
  state; shape and infested identities retain their separate ownership.
- `BLK-BEACON-STORAGE-001` owns five self-loot tables, eleven processing/unlock records, eight
  non-block acquisition tables, the optional rebalanced level-five armorer tag and two candidates,
  beacon/pickaxe/tool/guarded/loved/sulfur memberships, two sulfur archetypes, the bastion gold
  connector and five gold-replacement processor lists. Registrations, golem/piglin callbacks,
  code-built structure writes, template palettes and client models remain code-built or
  resource-pack selected.
- `BLK-RAW-STORAGE-001` owns three self-loot tables, six compacting/decompression recipes, six
  unlock advancements, pickaxe/tool/guarded/loved/slow-flat/carver-replaceable memberships, the
  shared slow-flat archetype and the three configured carvers that consume its replacement tag.
  Registration, piglin callbacks, ore-vein output, template absence and client models remain
  code-built, scanned or resource-pack selected.
- `BLK-LAPIS-BLOCK-001` owns its self-loot table, two compacting/decompression recipes, two unlock
  advancements, pickaxe/tool and slow-bouncy memberships plus the shared slow-bouncy archetype.
  Registration, exact lapis-lazuli item checks, mansion selection/template placement and client
  models remain code-built, template-scanned or resource-pack selected.
- `BLK-REDSTONE-BLOCK-001` owns its self-loot table, two compacting/decompression recipes, two
  unlock advancements and pickaxe membership. The ancient-city center pool, start processor and
  template payload retain their existing worldgen owners; registration, signal queries and client
  models remain code-built, template-scanned or resource-pack selected.
- `BLK-AMETHYST-BLOCK-001` owns its self-loot table, shard recipe/unlock, crystal-sound,
  vibration-resonator, pickaxe and slow-bouncy memberships plus the amethyst-geode configured and
  placed records. Registration and projectile/footstep/resonance consumers stay code-built, while
  structure absence is template-scanned and client models remain resource-pack selected.
- `BLK-BUDDING-AMETHYST-001` owns the five block loot tables, crystal/inside-step/pickaxe and
  cluster-harvestable memberships plus the budding alternate and four inner-placement identities
  in the geode configuration. Growth/support code stays code-built, structure absence is scanned
  and directional models remain resource-pack selected.
- `BLK-CALCITE-SMOOTH-BASALT-001` owns both self-loot tables, the smooth-basalt smelting/unlock,
  pickaxe/carver/sculk/slow-bouncy memberships and exact calcite/smooth-basalt identities in five
  configured features and five noise settings. Registration and algorithms stay code-built, the
  ancient-city payload is template/pool/processor selected, and client models stay pack-selected.
- `BLK-DEEPSLATE-001` owns its Silk/cobbled loot table, smelting record, seventeen cutting records,
  eighteen unlocks, four direct plus ten composed block tags, slow-bouncy membership, ore/support/
  spring/surface/flat identities and ancient-city pool/processor/template joins. Axis behavior,
  registration, retrogen and generic algorithms stay code-built; active data snapshots select
  later replacement, acquisition and structure paths without rewriting saved states.
- `BLK-DEEPSLATE-MASONRY-001` owns seven self-loot tables, 63 exact-ID recipes and unlocks, eight
  cobbled tag-keyed recipes, exact crafting/replacement/archetype tags and the ancient-city/trial-
  chambers pool, processor and template identities. Registration and generic algorithms stay
  code-built; reload changes later matching, replacement and structure decisions only.
- `BLK-DRIPSTONE-BLOCK-001` owns self loot, one recipe/unlock, one level-three mason record,
  pickaxe/sculk/archetype tags and the pointed/cluster feature records. Its registration, exact
  pointed-growth substrate and hard-coded large-feature result stay code-built; reload changes
  later matching, offer, replacement and feature decisions without rewriting saved state 30208.
- `BLK-SLIME-001` owns its reloadable loot table and the shaped block/shapeless ball recipes. Its
  two brewing start-mix edges, physical properties, piston identities and debug-generation selector
  are code-built and do not change when data packs replace those resource records.
- `BLK-HONEY-001` owns reloadable loot, shaped block/shapeless bottle recipes and the snow-support
  and bounce-suppression memberships. Physical/slide/piston/AI properties and debug generation are
  code-built; with zero entity bounciness, the bounce membership is outcome-equivalent to the
  registered block restitution zero, while a positive entity input keeps the tag observable.
- `BLK-HONEYCOMB-BLOCK-001` owns self loot, its compacting recipe/unlock and the sticky item
  tag/archetype. Registration and the zero-worldgen identity are code-built or template-scanned;
  reload changes later loot, recipe and equipment matches without rewriting saved state 21817.
- `BLK-BRICKS-001` owns correct-tool self loot, eight recipe/unlock records, pickaxe and
  slow-bouncy memberships, the slow-bouncy archetype and 31 structure-template payloads.
  Registration and derived stair/wall property copies stay code-built; reload changes later
  harvest, recipe, equipment and structure decisions without rewriting saved state 2340.
- `BLK-PACKED-MUD-001` owns tool-independent self loot, two recipe/unlock records, pickaxe and
  regular memberships, the regular archetype, six structure-template payloads and the houses/roads
  aging records. Registration stays code-built; reload changes later loot, recipe, equipment,
  template and processor selection without rewriting saved state 7758.
- `BLK-MUD-BRICKS-001` owns correct-tool self loot, seven recipe/unlock records, pickaxe and
  slow-bouncy memberships, the slow-bouncy archetype, 40 structure payloads and all three trail
  processor lists. Registration stays code-built; reload changes later harvest, recipe, equipment,
  connector, template and aging decisions without rewriting saved state 7759.
- `BLK-PURPUR-BLOCK-001` owns correct-tool self loot, eight recipe records, seven recipe
  advancements, the End-city display record, pickaxe/slow-bouncy memberships and 20 End-city
  payloads. Registration stays code-built; reload changes later matching and display only.
- `BLK-RED-NETHER-BRICKS-001` owns correct-tool self loot, seven recipe/unlock records, the
  Nether-root display record and pickaxe/slow-bouncy memberships. Registration and zero-worldgen
  identity stay code-built or scanned; reload changes later matching and display only.
- `BLK-NETHER-WART-BLOCK-001` owns tool-independent self loot, one recipe/unlock, hoe/wart tag
  closure, the slow-sliding archetype and locked crimson-fungus/surface records. Registration,
  composter chance and exact spawn/weeping/huge-fungus identity tests are code-built; active tags
  select later tutorial, equipment and Nether-carver membership without rewriting state 14846.
- `BLK-WARPED-WART-BLOCK-001` owns tool-independent self loot, hoe/wart tag closure, slow-sliding
  and locked warped-fungus/surface records. Registration, composter chance, twisting support,
  false crimson-vine identity and recipe/spawn-reference absences are code-built or swept; active
  tags select later tutorial, equipment and Nether-carver membership without rewriting state 20959.
- `BLK-NETHER-SPROUTS-001` owns shears-only loot, its 14-member support closure, replacement,
  combination-step and nested enchantment-transmitter tags plus the two sprouts vegetation
  records. Registration, code-built replaceability, composter chance and positional offset stay
  code-built; reload changes later survival, loot, enchanting and worldgen admission without
  rewriting state 20961.
- `BLK-NETHER-ROOTS-001` owns four block-loot tables, the two distinct 14-member support tags,
  replacement/combination-step/enchantment/Enderman/flower-pot tags, hoglin-stable chest entry and
  five vegetation/patch configurations. Static pot mappings, registrations, composter chance,
  Enderman algorithms and positional offset stay code-built; reload changes later survival, loot,
  AI selection and generation without rewriting existing root or potted states.
- `BLK-NETHER-WART-001` owns its age-sensitive block loot, sole support tag, two recipes and
  unlocks, plant-seed criterion, cleric purchase record/set/tag, Nether-bridge chest entry, three
  bastion templates, center pool and housing processor join. Registration, one-in-ten growth,
  bonemeal absence, composter chance, potion edge and fortress-room writer stay code-built; reload
  changes later survival, loot, crafting, progression, trade, chest and bastion selection without
  rewriting states 9447..9450.
- `BLK-NETHER-STEM-001` owns eight self-loot tables, crimson/warped stem and nested log tags,
  nonflammable item membership, thirteen positive recipes, ten direct unlocks, the bouncy sulfur
  archetype and four huge-fungus records. Registration, axe mappings, fire/fuel tables, leaf/parrot/
  tree/blending consumers, creative tabs and models stay code-built or pack-selected; reload changes
  later loot, crafting, tag/equipment selection and fungus placement without rewriting existing
  axis states.
- `BLK-CORAL-BLOCK-001` owns ten loot tables, the five-member live `coral_blocks` tag, direct
  pickaxe and fast-flat item memberships, five common wandering-trader records/set membership and
  warm-ocean configured/placed feature records. Registration, water scans, dry scheduling, pickle
  spread, feature traversal and models stay code-built or pack-selected; reload changes later loot,
  tag/equipment, trade and generation selection without rewriting existing live or dead states.
- `BLK-CORAL-PLANT-001` owns twenty Silk-only upright/floor-fan loot tables and the exact
  `coral_plants`, flattened `corals`, `wall_corals`, `underwater_bonemeals` and dead-form pickaxe
  memberships. Registration, support placement, drying, wall-loot delegation, bonemeal traversal,
  coral-feature decoration and models stay code-built or pack-selected; reload changes later loot
  and tag-selected acquisition without rewriting existing waterlogged or facing state.
- `BLK-FLOWER-POT-001` owns one empty and 36 filled loot tables, the flower-pot recipe/unlock,
  archaeology/mason acquisition, 39-member `flower_pots`, four-member `hoglin_repellents`,
  `piglin_repellents` nonmembership and the scanned structure-template records. Registration,
  content mapping, interaction order, eyeblossom callback and hoglin consumer stay code-built;
  reload changes later data/tag selection without rewriting an existing property-free state.
- `BLK-COPPER-FULL-001` owns 24 self-loot tables, 53 producing and 75 outside-family consuming
  recipes, wax-on/off advancement predicates, block/item `copper`, pickaxe/stone-tier,
  `slow_flat` archetype memberships and 149 trial-chamber template payloads. The fifteen
  weather/age collections, honeycomb/axe maps, copper-golem/chest transaction, fire/fuel registry
  absences, models and creative order stay code-built or asset-selected; reload changes later
  loot/recipe/tag/archetype/structure selection without rewriting an existing age/wax state.
- `BLK-SAPLING-001` owns eight self and eight corresponding-leaves loot paths, four village-chest
  sources, eight common wandering-trader records, `saplings`, `supports_vegetation` and `flowers`
  tags, all selected tree configurations, 45 stage-zero placed-feature survival predicates, four
  huge-fungus replaceability records and two structure templates. Registration, stage/update
  control flow, grower mappings, composter/fire/fuel tables and model identity remain code-built.
  Reload changes later support, flower, feature, loot, trade and worldgen selection without
  rewriting an existing stage state.
- `BLK-BAMBOO-001` owns two block, two chest, panda and biome-gated fishing loot paths; three
  recipes/unlocks; `supports_bamboo`, podzol-replaceable and `panda_food` closures; and both
  bamboo configured/placed feature paths. Registration, growth control flow, fuel/fire tables,
  panda class hooks and client assets stay code-built; reload changes later tag/data reads without
  rewriting an existing age/leaves/stage state.
- `BLK-ANCIENT-DEBRIS-001` owns block and three bastion loot tables; two recipes and their
  advancements plus the possession advancement; pickaxe/tier, fire-damage, slow-flat and
  base-stone-Nether closures; and both configured/placed ore paths plus five biome lists.
  Registration, item component default, cooking serializers, scattered-ore algorithm and client
  assets stay code-built; reload replaces later data reads without rewriting state `21819`.
- `BLK-STEM-CROP-001` owns four block loot tables, five chest/gameplay acquisition tables, two
  recipes and their advancements plus `plant_seed`, chicken/parrot and support/farmland tag
  closures, two wandering-trader records/set membership, four fungus configurations, six farm
  processors and all template payloads. Registration, growth/update/composter code and client
  assets stay code-built; reload changes later reads without rewriting an existing age/facing
  state or seed stack.
- `BLK-OVERWORLD-CROP-001` owns four crop loot tables, the coupled chest, archaeology and zombie
  acquisition pools, fifteen recipe/unlock pairs, farmer and wandering-trader records/sets,
  composter and animal-tag closures, four huge-fungus replaceable lists, ten village farm
  processors and 722 raw wheat template cells. Crop registration, growth and bone-meal code,
  farmer-villager harvesting, default food components and client assets remain code-built or
  pack-selected.
- `BLK-TORCHFLOWER-CROP-001` owns crop/flower and sniffer-digging loot, two recipe/unlock pairs,
  three husbandry advancements, crop/support/animal/bee tags, four fungus configurations and zero
  template payloads. Registration, logical-age replacement, both bone-meal algorithms, compost
  chances, fire table and client assets remain code-built or pack-selected.
- `BLK-PITCHER-CROP-001` owns crop/plant and sniffer-digging loot, the cyan-dye recipe/unlock,
  both husbandry placement advancements, support/animal/villager/bee/tree/mushroom tags, four
  crop-only fungus configurations and zero template payloads. Registration, five-age growth,
  half-resolving bone meal, double-plant/farmer transactions, compost chances, fire table and
  client assets remain code-built or pack-selected.
- `BLK-SWEET-BERRY-BUSH-001` owns block/interact/chest loot, the damage record, balanced-diet
  criterion, butcher trade, block/item/damage tags, berry configured/placed feature, both taiga
  decor pools, four fungus configurations and zero template payloads. Registration, four-age
  transitions, contact/movement and bee/fox/Ghast consumers, compost/fire tables and client assets
  remain code-built or pack-selected.
- `BLK-CAVE-VINES-001` owns both block tables, interaction and three chest tables, balanced-diet
  criterion, block/item tags, both column configurations, both placed ceiling paths, lush-cave
  biome join, four fungus configurations and zero template payloads. Registration, growing-plant
  transitions, bee consumer, compost/fire tables and client assets remain code-built or
  pack-selected.
- `BLK-CHORUS-001` owns both block tables and random sequences, three recipes and unlock
  advancements, balanced-diet criterion, block/item/projectile/support/flower tags, its
  configured/placed feature and End-Highlands join, four fungus configurations and zero template
  payloads. Registration, support/growth/projectile/teleport algorithms, fire/compost/fuel tables
  and client assets remain code-built or pack-selected.
- `BLK-SOUL-SAND-001` owns reloadable loot, three recipes, eleven direct block tags, two direct item
  tags, Soul Speed effects, the sulfur-cube high-resistance archetype and locked Nether worldgen
  records. Registration, shapes, postprocess-above and fortress/fossil/basalt concrete algorithms
  are code-built; active tag/data snapshots select bubble, fire, plant, snow, wither, sculk,
  enchantment, dried-ghast, archetype, acquisition and generation branches.
- `BLK-MAGMA-001` owns reloadable self loot, shaped recipe/advancement, seven direct block tags,
  the hot sulfur-cube item tag/archetype, Frost Walker effect and locked worldgen/processor records.
  Registration, hot-floor caller, postprocess-above and concrete underwater/delta/portal/basalt
  algorithms are code-built; active snapshots select bubble, plants, fire, Ghast, geyser,
  enchantment, archetype, acquisition and generation branches.
- `BLK-LAVA-CAULDRON-001` owns reloadable ordinary-cauldron loot and `cauldrons` membership used by
  path trimming; pickaxe membership includes that tag. Registration, shapes, inside effects,
  bucket dispatcher and hardcoded leatherworker POI states are code-built and do not change when
  those resources reload.

## Recovery procedure

1. Enumerate the concrete listeners returned by `ReloadableServerResources#listeners` and record
   each prepare dependency, apply barrier, executor and publication consumer.
2. For every registry and resource family, record input ordering, decode/validation branches,
   holder/tag binding and the first authoritative consumer after publication.
3. Inject failure at pack open, registry decode, tag bind, listener prepare, listener apply and
   post-publication refresh; compare the retained server/world/session snapshot at every point.
4. Reload before login, during configuration, in active play and while another player joins or
   changes dimension; verify command, registry, tag, recipe and advancement convergence over the
   locked protocol families.
5. Join each conclusion to the owning semantic leaf and executable vector before promoting this
   surface. A listener list or a successful `/reload` smoke test alone is not completion.
