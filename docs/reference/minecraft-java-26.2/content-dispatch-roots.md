# Content Dispatch Root Inventory

**Surface:** `SURFACE-CONTENT-DISPATCH-001`
**Status:** `InProgress`
**Primary evidence:** `OFF-SERVER-001`, `OFF-REPORT-001`, `OFF-DATA-001`

This inventory owns the point where a locked registry identity, implementation class, codec value,
tag, component or bundled-data record selects executable behavior. The catalog owns the exact
9,078-ID classification; the completion ledger separately owns all 95 registry scopes. Structural
one-owner coverage does not prove that a remaining subtype has no special control flow.

| Dispatch family | Locked source roots | Existing semantic owners | Remaining audit |
|---|---|---|---|
| Blocks and block entities | `net.minecraft.world.level.block.state.BlockBehaviour$BlockStateBase#getBlock`, `net.minecraft.world.level.block.state.BlockBehaviour$BlockStateBase#tick`, `net.minecraft.world.level.block.state.BlockBehaviour$BlockStateBase#randomTick`, `net.minecraft.world.level.block.state.BlockBehaviour$BlockStateBase#entityInside`, `net.minecraft.world.level.block.state.BlockBehaviour$BlockStateBase#updateShape`, `net.minecraft.world.level.block.entity.BlockEntityType#create` | `BLK-001`, `BLK-002`, `BLK-007`, `BLK-BRUSHABLE-001`, `BLK-SCULK-SENSOR-001`, `BLK-JIGSAW-001`, `BLK-STRUCTURE-001`, `BLK-STRUCTURE-VOID-001`, `BLK-AIR-001`, `BLK-BEDROCK-001`, `BLK-REINFORCED-DEEPSLATE-001`, `BLK-TINTED-GLASS-001`, `BLK-GLASS-001`, `BLK-STAINED-GLASS-001`, `BLK-CONCRETE-001`, `BLK-TERRACOTTA-001`, `BLK-GLAZED-TERRACOTTA-001`, `BLK-QUARTZ-001`, `BLK-SANDSTONE-001`, `BLK-STONE-VARIANT-001`, `BLK-STONE-BRICK-001`, `BLK-BEACON-STORAGE-001`, `BLK-RAW-STORAGE-001`, `BLK-LAPIS-BLOCK-001`, `BLK-REDSTONE-BLOCK-001`, `BLK-AMETHYST-BLOCK-001`, `BLK-BUDDING-AMETHYST-001`, `BLK-CALCITE-SMOOTH-BASALT-001`, `BLK-DEEPSLATE-001`, `BLK-DEEPSLATE-MASONRY-001`, `BLK-DRIPSTONE-BLOCK-001`, `BLK-SLIME-001`, `BLK-HONEY-001`, `BLK-HONEYCOMB-BLOCK-001`, `BLK-BRICKS-001`, `BLK-PACKED-MUD-001`, `BLK-MUD-BRICKS-001`, `BLK-PURPUR-BLOCK-001`, `BLK-RED-NETHER-BRICKS-001`, `BLK-NETHER-WART-BLOCK-001`, `BLK-WARPED-WART-BLOCK-001`, `BLK-NETHER-SPROUTS-001`, `BLK-NETHER-ROOTS-001`, `BLK-NETHER-WART-001`, `BLK-NETHER-STEM-001`, `BLK-CORAL-BLOCK-001`, `BLK-CORAL-PLANT-001`, `BLK-FLOWER-POT-001`, `BLK-COPPER-FULL-001`, `BLK-SAPLING-001`, `BLK-BAMBOO-001`, `BLK-ANCIENT-DEBRIS-001`, `BLK-STEM-CROP-001`, `BLK-OVERWORLD-CROP-001`, `BLK-TORCHFLOWER-CROP-001`, `BLK-PITCHER-CROP-001`, `BLK-SWEET-BERRY-BUSH-001`, `BLK-CAVE-VINES-001`, `BLK-CHORUS-001`, `BLK-SOUL-SAND-001`, `BLK-MAGMA-001`, `BLK-LAVA-CAULDRON-001`, `BLK-TEST-BLOCK-001`, `BLK-TEST-INSTANCE-001`, `BLK-CONDUIT-001`, `BLK-BEACON-001`, `BLK-SIGN-001`, `BLK-SKULL-001`, `ITM-CHEST-001`, `ITM-HOPPER-001`, `ITM-DISPENSER-001` and concrete block leaves own the virtual hooks, lifecycle and state schema. | Resolve 84 block fallback IDs to audited implementation-class families or dedicated leaves; all 49 block-entity types now have exact owners. Include inherited overrides, block items, ticker factories, persistence and projection. |
| Items and data components | `net.minecraft.world.item.ItemStack#use`, `net.minecraft.world.item.ItemStack#inventoryTick`, `net.minecraft.world.item.ItemStack#onUseTick`, `net.minecraft.world.item.ItemStack#applyComponentsAndValidate`, `net.minecraft.world.item.Item#use`, `net.minecraft.core.component.DataComponentMap#makeCodec` | `ITM-001`, `ITM-003`, `BLK-AIR-001`, `ITM-HONEYCOMB-001`, `ITM-STEW-001`, `ITM-BUNDLE-001`, `ITM-BOAT-001`, `ITM-POTTERY-SHERD-001`, `ITM-SMITHING-TEMPLATE-001`, `ITM-HARNESS-001`, `ITM-MINECART-001`, `ITM-STEERING-STICK-001`, `ITM-SPEAR-001`, `ITM-NAUTILUS-ARMOR-001`, `ITM-EGG-001`, `ITM-BAKED-POTATO-001`, `ITM-BEEF-001`, `ITM-BONE-001`, `ITM-BOOK-FAMILY-001`, `ITM-CHICKEN-001`, `ITM-MUTTON-001`, `ITM-PORKCHOP-001`, `ITM-RABBIT-MATERIAL-001`, `ITM-COD-001`, `ITM-SALMON-001`, `ITM-TROPICAL-FISH-001`, `ITM-BREAD-001`, `ITM-COOKIE-001`, `ITM-CHEST-001`, `ITM-HOPPER-001`, `ITM-DISPENSER-001`, `BLK-STEM-CROP-001`, `BLK-OVERWORLD-CROP-001`, `BLK-TORCHFLOWER-CROP-001`, `BLK-PITCHER-CROP-001`, `BLK-SWEET-BERRY-BUSH-001`, `BLK-CAVE-VINES-001`, `BLK-CHORUS-001`, item-use/container/progression leaves and component-selected hooks own authoritative stack behavior; AIR is a plain-item empty-stack sentinel, stew dispatch fixes five component/data/mob-selected identities, bundle dispatch fixes 17 component/click/use/client-selected identities, boat dispatch fixes twenty item-to-vehicle/container identities, pottery-sherd dispatch fixes 23 plain-item identities selected by tags, pot faces, loot, advancements and client resources, smithing-template dispatch fixes nineteen subclass/rarity/recipe/loot/UI identities, harness dispatch fixes sixteen body-equippable identities and their exact Happy Ghast vehicle/AI join, minecart dispatch fixes six exact item-to-rail-vehicle identities and their destruction/pick asymmetry, steering-stick dispatch fixes two exact item-to-mount boost/durability identities, spear dispatch fixes seven component-selected piercing/kinetic identities, nautilus-armor dispatch fixes five body-equippable identities and their normal/zombie mount/menu/shear/sunlight joins, egg dispatch fixes three item-to-projectile/chicken-variant identities and their laying/hatch/recipe joins, Baked-Potato dispatch fixes one plain edible identity selected by cooking recipes, loot, composting, stew, trade and client records, Beef dispatch fixes two plain edible identities selected by bovine fire-converting loot, cooking, Wolf tags, village/Trial/gift loot, Butcher trade and client records, Bone dispatch fixes one plain identity selected by skeletal/chest/fishing loot, crafting and hard-coded Wolf tame/BegGoal paths, Book-family dispatch fixes four component-, use-, recipe-, loot-, trade- and client-selected identities, Chicken dispatch fixes two food/effect-, death/cooking-, gift/Trial-, trade/Wolf- and client-selected identities, Mutton dispatch fixes two food-, Sheep-death/cooking-, village/gift-, trade/Wolf- and client-selected identities, Porkchop dispatch fixes two food-, Pig/Hoglin-death/cooking-, village/Bastion/gift-, trade/Piglin/Wolf- and client-selected identities, Rabbit-material dispatch fixes three food/inert-, ordered death-loot-, cooking/recipe-, gift/trade-, Wolf/Balanced-Diet- and client-selected identities, Cod dispatch fixes two food-, aquatic-death/fishing/chest/gift-loot-, cooking/trade-, five-mob- and client-selected identities, Salmon dispatch fixes two food-, fish-death/fishing/chest/gift-loot-, cooking/trade-, five-mob- and client-selected identities, Tropical-Fish dispatch fixes one food-, direct/Guardian/fishing-loot-, trade-, three-mob- and client-selected identity, Bread dispatch fixes one food-, player/Farmer-crafting-, chest/Trial/gift/trade-, Villager/Composter- and client-selected identity, Cookie dispatch fixes one food-, recipe/trade/gift-, Parrot-poison/Composter- and client-selected identity, and dispenser dispatch additionally fixes its 80 exact item identities and dynamic equippable, sulfur-cube and entity-data branches. | Resolve 37 item fallback IDs; partition implementation subclasses, default components, `use_remainder`/consumable/equippable/tool/projectile hooks, inventory ticks and component-driven feature gates. |
| Entities, mobs and effects | `net.minecraft.world.entity.EntityType#create`, `net.minecraft.world.effect.MobEffectInstance#tickServer`, `net.minecraft.world.effect.MobEffect#applyEffectTick`, `net.minecraft.world.effect.MobEffect#applyInstantaneousEffect` | `ENT-001`, `ENT-004`, `MOB-004` and entity/effect leaves own construction, lifecycle, AI and effect application. | Resolve 37 entity-type fallback IDs; audit factory class, spawn finalization, tracking data, persistence, passengers, goals/brains, interaction/damage hooks and client metadata for every remaining subtype. |
| Menus and recipes | `net.minecraft.world.inventory.MenuType#create`, `net.minecraft.world.item.crafting.RecipeSerializer#codec`, `net.minecraft.world.item.crafting.RecipeSerializer#streamCodec`, `net.minecraft.world.item.crafting.RecipeManager#getRecipeFor`, `net.minecraft.world.item.crafting.RecipeManager#byKey` | `ITM-002`, `ITM-004` and container/recipe leaves own menu layout, controls, matching, assembly and convergence. | Keep the explicit 25-menu and 21-serializer partitions synchronized with new leaves; audit feature filtering, display lookup, recipe-book joins and data reload without treating recipe JSON values as new algorithms. |
| Loot, advancement and progression records | `net.minecraft.server.ReloadableServerRegistries$Holder#getLootTable`, `net.minecraft.world.level.storage.loot.LootTable#getRandomItemsRaw`, `net.minecraft.world.level.storage.loot.LootTable#getRandomItems`, `net.minecraft.server.ServerAdvancementManager#apply` | `ITM-006`, `ITM-007` and loot/progression leaves own context construction, conditions/functions, RNG, reward and criterion effects. | Preserve exact serializer/type dispatch for loot entries, conditions, functions, number/NBT/score providers and advancement triggers; join data reload, malformed references, recursion and per-player persistence. |
| Tags, holders, enchantments and data-selected predicates | `net.minecraft.core.Holder#is`, `net.minecraft.world.item.enchantment.Enchantment#getEffects`, `net.minecraft.world.item.enchantment.Enchantment#modifyUnfilteredValue`, `net.minecraft.world.item.enchantment.Enchantment#tick`, `net.minecraft.tags.TagLoader#build` | Block, item, entity, enchantment and environment leaves own each consumer; DataReload owns snapshot rebinding. | Inventory every behavior-affecting tag/component/type registry consumer, optional versus required references, ordered conditional effects and holder identity across reload; a tag list alone is not an algorithm owner. |
| Game rules and global selectors | `net.minecraft.world.level.gamerules.GameRules#codec`, `net.minecraft.world.level.gamerules.GameRules#get`, `net.minecraft.world.level.gamerules.GameRules#set`, `net.minecraft.world.level.gamerules.GameRule#valueCodec` | Simulation, environment, command, player, entity and world leaves own each rule's read/write consequences. | Keep all 59 IDs synchronized with the closed [game-rule consumer inventory](game-rule-consumers.md), including defaults/validation, indirect callers, callbacks, persistence and client projection; shared storage/codec behavior is insufficient. |
| World generation and structures | `net.minecraft.world.level.levelgen.feature.Feature#configuredCodec`, `net.minecraft.world.level.levelgen.feature.Feature#place`, `net.minecraft.world.level.levelgen.structure.Structure#findGenerationPoint`, `net.minecraft.world.level.levelgen.structure.Structure#afterPlace`, `net.minecraft.world.level.levelgen.DensityFunction#compute`, `net.minecraft.world.level.levelgen.DensityFunction#codec`, `net.minecraft.world.level.levelgen.SurfaceRules$Condition#test` | `WGEN-003`, `WGEN-PIPELINE-001`, structure, jigsaw, feature, dimension and border leaves own executable generation. | Resolve 184 worldgen fallback records by registry key and codec-selected implementation; distinguish genuine parameter trees from structure/biome/source/placement control flow and retain equivalence boundaries. |
| Catalog classification and recovery | `docs/reference/minecraft-java-26.2/catalog/catalog.toml` | Each exact/pattern family names its current rule owners; `mc-ref query` exposes the joined data and classification. | Replace all four `Unreviewed` remaining selectors with exact or proven pattern families or justified `DataOnly`; verify zero stale, zero-match, overlapping or silently broadened selectors before completion. |

`BLK-CORAL-PLANT-001` is an exact block-dispatch owner for the thirty upright, floor-fan and
wall-fan identities previously covered only by the broad `simple-waterlogged` classification. It
did not consume an explicit block fallback; it replaced generic classification with exact state,
support, drying, loot, tag-consumer, worldgen, persistence and projection semantics.

`BLK-FLOWER-POT-001` is the exact dispatch owner for 37 former block fallbacks. It fixes the
code-built content map and uniform pot class, then isolates the only two content-selected
exceptions: environment-driven potted-eyeblossom random ticks and the potted-warped-fungus hoglin
tag consumer. The later copper batch continues from that recovered fallback state.

`BLK-COPPER-FULL-001` is the exact dispatch owner for 24 former block fallbacks. It separates 12
weathering `WeatheringCopperFullBlock` instances from 12 inert waxed `Block` instances, owns their
radius-four cross-collection age scan and full-block-only golem tag branch, and closes the fallback
at 105 IDs.

`BLK-SAPLING-001` is the exact dispatch owner for eight former block fallbacks. It owns
`VegetationBlock` support/update inheritance, both `SaplingBlock` stage callbacks and the complete
small/flower/mega `TreeGrower` selector and cleanup transaction, closing the block fallback at 97
IDs.

`BLK-BAMBOO-001` owns the separate `BambooSaplingBlock`, `BambooStalkBlock` and
`BambooFeature` paths plus their shared item/tag/data consumers, closing the block fallback at 96
IDs.

`BLK-ANCIENT-DEBRIS-001` owns the ordinary property-free block dispatch plus its component,
recipe, advancement, loot, archetype and `ScatteredOreFeature` joins, closing the block fallback
at 95 IDs.

`BLK-STEM-CROP-001` owns four `StemBlock`/`AttachedStemBlock` identities, two custom-named seed
block items and every support, growth, loot, acquisition, consumer, worldgen and projection join,
closing the block and item fallbacks at 91 and 243 IDs.
`BLK-OVERWORLD-CROP-001` owns the four ordinary `CropBlock` identities and seven directly coupled
plain or seed-placement items, including wheat's exact-item override ahead of the generic
same-name block matcher. It closes their support, growth, interaction, loot, acquisition,
villager/animal, worldgen and projection joins at 87 block and 237 item fallback IDs.
`BLK-TORCHFLOWER-CROP-001` owns the torchflower crop and seed fallback identities and promotes the
coupled mature flower/item into the same exact family. It closes logical-age-two replacement, crop/flower bone
meal, loot, sniffer/animal/villager, compost, recipe, fire, worldgen, persistence and projection
dispatch at 86 block and 236 item fallback IDs.
`BLK-PITCHER-CROP-001` promotes both pitcher blocks from generic multi-block placement and owns the
remaining pod item fallback. It closes lower-only five-age growth, the two-cell transition,
half-resolving bone meal and break behavior, loot, sniffer/animal/villager/bee, compost, recipe,
fire, worldgen, persistence and projection dispatch at 86 block and 235 item fallback IDs.
`BLK-SWEET-BERRY-BUSH-001` promotes the bush from the broad fire family and owns the berry item
fallback. It closes support/growth/bone meal, harvest/loot, movement/damage/fall reset, bee/fox/
Ghast, food/advancement/trade/chest/compost/fire, worldgen, persistence and projection dispatch at
86 block and 234 item fallback IDs.
`BLK-CAVE-VINES-001` promotes both vine identities from the broad fire family and owns the
glow-berries item fallback. It closes downward support/conversion, head-only growth, segment
bone meal/bee lighting, harvest/loot, climb/glide/fox/food/chest/compost/fire, worldgen,
persistence and projection dispatch at 86 block and 233 item fallback IDs.
`BLK-CHORUS-001` closes both chorus block fallbacks and the chorus-fruit/popped-fruit item
fallbacks while giving both block items an exact owner. It closes connection/support repair,
flower growth/death, projectile break and loot, random teleport/cooldown, recipes/bee/progression,
End generation, persistence and projection dispatch at 84 block and 231 item fallback IDs.
`ITM-STEW-001` closes the bowl, mushroom-stew, rabbit-stew, beetroot-soup and suspicious-stew
fallbacks. It separates player use-remainder completion from direct mob stack consumption, fixes
ordered effect-component application and generation, mooshroom/wolf interactions, recipes,
acquisition/progression, persistence and client variants at 84 block and 226 item fallback IDs.
`ITM-BUNDLE-001` closes the plain and 16 dyed bundle fallbacks. It fixes fractional/nested capacity,
ordered whole-entry transfers, both click-override directions, transient packet-selected removal,
held emptying and destruction release, component-preserving recolors, acquisition/progression,
persistence and exact tooltip/model/tab projection at 84 block and 210 item fallback IDs.
`ITM-BOAT-001` closes the twenty boat, chest-boat and raft fallbacks. It fixes exact held/dispenser
placement, passenger-versus-container interaction, pending loot and 27-slot persistence,
destructive-removal content release before gamerule-gated matching itemization, recipes, fuel,
fisherman trades, goat progression, reload and item/entity/menu/tab projection at 84 block and 190
item fallback IDs.
`ITM-POTTERY-SHERD-001` closes all 23 pottery-sherd fallbacks. It fixes uncommon plain-item and
pattern identities, tag-selected recipe and advancement joins, exact cracked recovery, twenty
weighted archaeology entries, three fixed trial-chamber sherd sources, persistence, reload and
item/pattern/tab projection at 84 block and 167 item fallback IDs.
`ITM-SMITHING-TEMPLATE-001` closes all nineteen smithing-template fallbacks. It fixes exact
subclass/rarity/UI behavior, duplication and smithing selectors, acquisition and advancement
joins, persistence, reload and item/tab/equipment projection at 84 block and 148 item fallback IDs.
`ITM-HARNESS-001` closes all sixteen harness fallbacks and splits Happy Ghast from the generic
animal family. It fixes exact equippable assets, adult direct/dispenser admission, leash/shear and
guaranteed-drop order, live-tag temptation, four-passenger mount/control, recipes/unlocks,
persistence, reload and item/equipment/tab projection at 84 block and 132 item fallback IDs.
`ITM-MINECART-001` closes all six `MinecartItem` fallbacks. It fixes held/dispenser rail placement,
exact subtype interaction and activator joins, container/fuel/fuse/command state, matching versus
ordinary destruction results, five recipes, six unlock joins, mineshaft acquisition, persistence,
reload and item/entity/menu/tab projection at 84 block and 126 item fallback IDs.
`ITM-STEERING-STICK-001` closes both `FoodOnAStickItem` fallbacks and splits pig/strider from their
generic entity family. It fixes exact controller/temptation joins, boost-before-durability order,
patched fishing-rod break conversion, two recipes/unlocks, Nether progression, persistence, reload
and handheld-rod/bar/tab projection at 84 block and 124 item fallback IDs.
`ITM-SPEAR-001` closes all seven component-built spear fallbacks. It fixes minimum-charge STAB versus
held kinetic ingress, ordered multi-target scans, tier attributes/timings/conditions, contact and
feedback clocks, Lunge, mob equipment/AI, recipes/repair/recycling/fuel/loot/progression,
persistence/reload and dual-context projection at 84 block and 117 item fallback IDs.
`ITM-NAUTILUS-ARMOR-001` closes all five nautilus-armor fallbacks and splits both nautilus entity
identities from their broad animal family. It fixes component-built body attributes/equippability,
direct/dispenser/menu/shear admission, guaranteed recovery, zombie sunlight protection,
recipes/loot/unlocks, persistence/reload and exact item/equipment/menu/tab projection at 84 block
and 112 item fallback IDs.
`ITM-EGG-001` closes both blue/brown egg fallbacks and promotes ordinary egg into one exact family.
It fixes registration-built variant components, held/dispenser launch, flight/impact/hatch order,
chicken laying, tag recipes/unlock, persistence/reload and exact item/projectile/particle/tab
projection at 84 block and 42 item fallback IDs. `ITM-BOOK-FAMILY-001` additionally closes Book,
Enchanted Book, Writable Book and Written Book across component, recipe, use, loot, trade and
client dispatch. `ITM-CHICKEN-001` additionally closes Raw and Cooked Chicken across food/effect,
death-loot/cooking, gift, Trial, trade, Wolf and client dispatch. `ITM-MUTTON-001` additionally
closes Raw and Cooked Mutton across food, Sheep death/cooking, village/gift loot, trade, Wolf and
client dispatch. `ITM-PORKCHOP-001` additionally closes Raw and Cooked Porkchop across food,
Pig/Hoglin death/cooking, village/Bastion/gift loot, trade, Piglin/Wolf and client dispatch.
`ITM-RABBIT-MATERIAL-001` additionally closes Raw Rabbit, Cooked Rabbit and Rabbit Hide across
food/inert defaults, ordered Rabbit death loot, cooking, stew/Leather recipes, gifts, trades, Wolf,
Balanced Diet and split client dispatch.
`ITM-COD-001` additionally closes Raw and Cooked Cod across food, aquatic death/fishing/chest/gift
loot, cooking, Fisherman records, Cat/Ocelot/Dolphin/Wolf/Nautilus paths, progression and exact
client dispatch. `ITM-SALMON-001` additionally closes Raw and Cooked Salmon across food,
Salmon/Polar-Bear death/fishing/chest/gift loot, cooking, Fisherman level-two/three records, the
same five mob-food paths, progression and exact client dispatch. `ITM-TROPICAL-FISH-001`
additionally closes Tropical Fish across food, direct/Guardian/fishing loot, the level-four
Fisherman record, Dolphin/Wolf/Nautilus paths, negative Axolotl and taming joins, progression and
exact client dispatch. `ITM-BREAD-001` additionally closes Bread across player and Farmer
crafting, eighteen chest rows, normal Trial consumables, Farmer sale/gift records, Villager
pickup/food/sharing/breeding, composting, progression and exact client dispatch.
`ITM-COOKIE-001` additionally closes Cookie across crafting, the guaranteed Farmer sale and gift,
Parrot poisonous-food consumption/remainder/effect/damage ordering, composting, progression,
Allay icon and exact client dispatch at 84 block and 37 item fallback IDs.

## Boundary conclusions

- Registry lookup selects an identity or implementation object; later virtual, codec or data-driven
  dispatch selects the behavior. Both boundaries must be represented when they can diverge.
- `DataOnly` means a record supplies values to an already audited algorithm. It cannot be inferred
  from common JSON shape, common base class, absent catalog overlap or lack of a remembered quirk.
- Tags, components and holder references can change the branch taken by generic code without
  creating an ID-specific subclass. Consumer search is therefore part of content dispatch.
- `InProgress` remains required while any of the 342 catalog IDs is `Unreviewed`, even though every
  locked ID has exactly one structural catalog owner.

## Recovery procedure

1. For each fallback ID, resolve its registered implementation/factory and every effective virtual
   hook, codec/type discriminator, default component, tag and bundled-data reference.
2. Compare the trace with an existing audited family. Add an exact/pattern member only when all
   independent behavior is inherited; otherwise create or extend a source-specified leaf.
3. Prove `DataOnly` by tracing every decoded field into an already specified algorithm and showing
   no ID-specific dispatch, callback or consumer branch.
4. Run `mc-ref query`, symbol verification, catalog coverage and readiness after every family; keep
   all raw reports and class inspection under ignored `target/mc-reference/26.2/` paths.
5. Promote this surface only when the catalog has zero `Unreviewed` IDs and all cross-system joins
   from content selection to reload, persistence and projection have terminal ownership.
