# Items, inventories and progression mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ITM-DIAMOND-001` — Diamond joins ore, fossil and loot acquisition to equipment, progression, trade, Beacon, firework and armor-trim sinks

**Parent:** `SIM-004`, `SIM-005`, `SIM-RANDOM-001`, `BLK-001`, `BLK-002`,
`BLK-003`, `BLK-005`, `BLK-BREAK-001`, `BLK-BREAK-HOOK-001`, `PLY-002`,
`PLY-005`, `PLY-006`, `PLY-INPUT-001`, `PLY-INTERACT-001`, `PLY-BREAK-001`,
`ITM-001`, `ITM-002`, `ITM-003`, `ITM-004`, `ITM-005`, `ITM-006`, `ITM-007`,
`ITM-USE-001`, `ITM-CONTAINER-001`, `ITM-RECIPE-001`,
`ITM-RECIPE-SERIALIZER-001`, `ITM-CRAFT-001`, `ITM-FURNACE-001`,
`ITM-SMITHING-001`, `ITM-LOOT-001`, `ITM-ADVANCEMENT-001`, `ITM-ANVIL-001`,
`ITM-SMITHING-TEMPLATE-001`, `BLK-BEACON-001`,
`BLK-BEACON-STORAGE-001`, `ENT-001`, `MOB-001`, `MOB-004`, `MOB-AI-001`,
`ENV-001`, `ENV-002`, `ENV-003`, `WGEN-PIPELINE-001`, `CLI-001`, `CLI-006`,
`CLI-UI-001`, `CLI-EFFECT-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked registration/components, both ore blocks and tables, all recipes and
advancements, complete loot, trade and trim records, every ordinary/fossil generation selector,
all `1,212` templates and exact client resources determine every Diamond-specific branch. Generic
block breaking, processing, crafting, Firework-Star, Smithing, loot, merchant, Beacon, anvil,
worldgen, persistence, packet and rendering algorithms retain the cited owners.

**Applies when:**

`minecraft:diamond` is mined, cooked, looted, brushed, crafted, used to craft or repair equipment,
spent in a Beacon, merchant offer, template duplication, Firework Star or armor trim, moved,
renamed, persisted, synchronized or rendered before and after recipe, advancement, loot, tag,
trade, trim, worldgen or resource reload.

**Authoritative state:**

Diamond is raw item ID `926`, a common nondamageable plain `Item` with maximum stack `64`. Its
ordinary components include `provides_trim_material=minecraft:diamond`. Its four direct item tags
are `beacon_payment_items`, `diamond_tool_materials`, `repairs_diamond_armor` and
`trim_materials`; the last three are singleton tags in the locked baseline. It has no food,
consumable, remainder, fuel, compost, equipment, durability, projectile, cooldown,
inventory-tick or identity-specific use branch.

The coupled source blocks are property-free `DropExperienceBlock` instances:

| Block | block/item/state IDs | map/sound | hardness/resistance |
|---|---:|---|---:|
| Diamond Ore | `203/105/5307` | `STONE` / Stone | `3/3` |
| Deepslate Diamond Ore | `204/106/5308` | `DEEPSLATE` / Deepslate | `4.5/3` |

Both use `BASEDRUM`, require a correct tool, and are direct `diamond_ores`,
`mineable/pickaxe` and `needs_iron_tool` members. Their full-cube physics and generic XP hook
remain with the block owners.

**Transition and ordering:**

### Ore break, processing and generation

Each ore table has one ordered alternative. Silk Touch level at least one emits one default
matching Ore block. Otherwise it emits one default Diamond, applies `ore_drops` Fortune
multiplication, then explosion decay. A correct non-Silk player break independently draws integer
XP `3..7`; Fortune does not change it. A tool that is not both pickaxe-effective and admitted for
the live iron-tier requirement yields neither ordinary loot nor break XP. Silk suppresses that
ordinary XP. Named sequences are `minecraft:blocks/{diamond_ore,deepslate_diamond_ore}`.

Four exact cooking records emit one default Diamond with recipe XP `1.0`. Furnace accepts each Ore
in a separate record at omitted/default time `200`; Blast Furnace accepts each in a separate
record at omitted/default time `100`. Smoker and Campfire reject both. Every record has its own
advancement, unlocked by possession of its exact Ore or prior knowledge. Input patches are not
copied.

Four configured/placed pairs run in all `55` Overworld biomes:

| Placed ID | configured size / air-discard | first modifier | height |
|---|---:|---|---|
| `ore_diamond` | small `4 / 0.5` | count `7` | trapezoid above-bottom `-80..80` |
| `ore_diamond_buried` | `8 / 1` | count `4` | same trapezoid |
| `ore_diamond_large` | `12 / 0.7` | rarity `9` | same trapezoid |
| `ore_diamond_medium` | `8 / 0.5` | count `2` | uniform absolute `-64..-4` |

Every wrapper then applies in-square and biome. Configured targets are ordered live
`stone_ore_replaceables` to state `5307`, then `deepslate_ore_replaceables` to `5308`.

Desert, Swamp and Mangrove Swamp also schedule `fossil_lower`: rarity `64`, in-square, uniform
above-bottom `0` through absolute `-8`, then biome. Its `fossil_diamonds` configuration selects
one of eight primary/coal-overlay template pairs. Overlay block-rot integrity `0.1` runs before an
always-true rule converts surviving Coal Ore to state `5308`, then protected-block admission runs.
The fossil transaction and all ore geometry/read/write order remain `WGEN-PIPELINE-001`.

### Direct loose-item acquisition

Every listed pool can select Diamond repeatedly when it has multiple rolls:

| Table / pool | rolls | Diamond weight / pool total | count |
|---|---:|---:|---:|
| archaeology/desert_pyramid `0` | `1` | `1/8` | `1` |
| chests/abandoned_mineshaft `1` | `2..4` | `3/98` | `1..2` |
| chests/bastion_treasure `0` | `3` | `5/112` | `2..6` |
| chests/buried_treasure `2` | `1..3` | `5/15` | `1..2` |
| chests/desert_pyramid `0` | `2..4` | `5/247` | `1..3` |
| chests/end_city_treasure `0` | `2..6` | `5/89` | `2..7` |
| chests/jungle_temple `0` | `2..6` | `3/89` | `1..3` |
| chests/nether_bridge `0` | `2..4` | `5/78` | `1..3` |
| chests/shipwreck_treasure `0` | `3..6` | `5/150` | `1` |
| chests/stronghold_corridor `0` | `2..3` | `3/101` | `1..3` |
| chests/trial_chambers/intersection_barrel `0` | `1..3` | `1/33` | `1..3` |
| chests/trial_chambers/intersection `0` | `1..3` | `10/86` | `1..2` |
| chests/trial_chambers/reward_common `0` | `1` | `1/25` | `1..2` |
| chests/trial_chambers/reward_ominous_common `0` | `1` | `2/15` | `2..3` |
| chests/village/village_toolsmith `0` | `3..8` | `1/53` | `1..3` |
| chests/village/village_weaponsmith `0` | `3..8` | `3/107` | `1..3` |
| pots/trial_chambers/corridor `0` | `1` | `5/351` | `1..2` |

Trade Rebalance replaces the Abandoned-Mineshaft, Desert-Pyramid and Jungle-Temple tables. Their
Diamond rows, rolls, weights and counts remain; only the Desert-Pyramid pool total changes from
`247` to `237`. Loot installation, named sequences, brush admission and container/pot commit
remain with their owners.

No entity, fishing, gift, barter, ordinary Villager sale or Wandering-Trader table directly emits
loose Diamond. An exhaustive exact-UTF scan finds zero loose Diamond strings and zero raw
Diamond-Ore palette cells across all `1,212` templates; container sources remain loot-table
driven and the fossil transformation remains processor driven.

### Fifty-six recipe joins and progression

Diamond participates in `56` recipes:

- six shaped Axe/Hoe/Pickaxe/Shovel/Spear/Sword records use the live singleton
  `diamond_tool_materials` tag and respectively consume `3/2/3/1/1/2` Diamond plus Stick roles;
- four exact armor grids consume `4/8/5/7` Diamond for Boots/Chestplate/Helmet/Leggings;
- Diamond Block compression consumes `9`, while the shapeless reverse emits `9`;
- Enchanting Table consumes `2` Diamond with one Book and four Obsidian;
- Jukebox consumes one Diamond surrounded by eight live `planks`;
- each of `19` Smithing-template duplication grids consumes seven Diamond plus one template and
  its identity-specific core;
- the four Ore-cooking records above emit Diamond;
- `18` generic armor-trim Smithing records admit one Diamond in the addition slot; and
- the always-available special Firework-Star record admits at most one exact Diamond as its trail
  ingredient.

The Firework-Star matcher still requires exactly one Gunpowder and at least one component-bearing
live Dye, permits at most one shape and one Glowstone-Dust twinkle input, and rejects every other
identity. Assembly consumes Diamond once and sets `FIREWORK_EXPLOSION.has_trail=true` while
preserving the generic ordered colors, shape, fade and twinkle behavior. It has no recipe
advancement.

The other `55` records each have an advancement: four cooking, six tools, four armor, two
compression, Enchanting Table, Jukebox, nineteen duplication and eighteen trim records. Diamond
possession itself can satisfy the inventory alternative for the six tools, four armor records,
Diamond Block and Jukebox (`12` recipe unlocks); the other listeners use their exact Ore, Block,
Obsidian or template input. Exact Diamond possession separately completes the sole criterion of
`story/mine_diamond`, subject to its parent/progression owner. Grid placement, tag expansion,
machine/result capacity, atomic consumption, special-recipe control flow and knowledge
publication remain generic.

### Repair, Beacon, merchant and armor-trim sinks

Diamond's live material tag is the repair ingredient for six Diamond tools and its live armor tag
repairs four humanoid Diamond armor pieces. Diamond Horse Armor and Diamond Nautilus Armor have no
repairable component and reject it. Generic anvil combination, quarter-durability restoration,
cost, rename, cap and commit behavior remains `ITM-ANVIL-001`.

The Beacon payment slot and quick-move logic accept Diamond only while it is a live
`beacon_payment_items` member. Selection consumes one payment on successful effect update;
pyramid validation, primary/secondary effect gates, UI synchronization and aborts remain
`BLK-BEACON-001`.

Baseline profession sets consume exact Diamond through three reloadable records:

| profession / level | candidates / selected | inclusion | wants → gives | uses / XP / discount |
|---|---:|---:|---|---:|
| Armorer `3` | `5 / 2` | `2/5` | `1` Diamond → `1` Emerald | `12 / 20 / 0.05` |
| Toolsmith `4` | `3 / 2` | `2/3` | `1` Diamond → `1` Emerald | `12 / 30 / 0.05` |
| Weaponsmith `4` | `2 / 2` | `1` | `1` Diamond → `1` Emerald | `12 / 30 / 0.05` |

Trade Rebalance removes the Armorer level-three purchase; Toolsmith and Weaponsmith remain
unchanged. Rebalanced Armorer level five instead has ten variant-filtered armor offers that take
Diamond as a second cost:

| variant | offers: Emerald + Diamond → output / fixed level-one enchantment |
|---|---|
| Desert | `16+4` → Chestplate / Thorns; `16+3` → Leggings / Thorns |
| Plains | `16+3` → Leggings / Protection; `12+2` → Boots / Protection |
| Savanna | `6+2` → Helmet / Binding Curse; `8+3` → Chestplate / Binding Curse |
| Snow | `12+2` → Boots / Frost Walker; `12+3` → Helmet / Aqua Affinity |
| Taiga | `18+4` → Chestplate / Blast Protection; `18+3` → Leggings / Blast Protection |

Level five selects two successful offers. For each listed variant, the two raw-Diamond offers and
one matching storage-block purchase are the three nonnull candidates, so each raw-Diamond offer
has inclusion probability `2/3`. Jungle and Swamp have no raw-Diamond offer. Every listed armor
offer has maximum uses `3`, Villager XP `30` and reputation discount `0.05`. Generic predicate
filtering, distinct selection, price/demand adjustment, second-cost validation, result modifier,
transaction, exhaustion and restock remain merchant-owned.

Diamond's default provider resolves trim material `minecraft:diamond`, description color
`#6EECD2` and normal asset `diamond`. On Diamond equipment the material overrides itself with
`diamond_darker` so the trim remains visible. As a live `trim_materials` member Diamond fills the
addition slot of all `18` generic trim recipes, is consumed once and writes the Diamond material
holder into copied armor. Removing the tag rejects it; removing or replacing the provider changes
material resolution independently after recipe admission.

**Persistence and reload boundary:**

Stacks persist identity, count and patches. Ores, machines, knowledge, Firework Stars, anvils,
Beacon state, containers, brushable blocks, offers and trimmed equipment persist with their
owners. Recipe, advancement, loot, tag, trade, trim and worldgen reload changes only future
evaluation; completed mining, processing, loot, crafts, repairs, payments, offers, trims and
generated chunks are not replayed or rewritten. Existing merchant offers retain their constructed
costs/results. Resource reload independently changes projection only.

**Wire and client projection:**

Generic stack publication uses item ID `926`; Ore block/item/state IDs are
`203/105/5307` and `204/106/5308`. No Diamond-specific packet exists. English names are
`Diamond`, `Diamond Ore`, `Deepslate Diamond Ore` and `Diamond Material`.

Loose Diamond selects one untinted `item/generated` flat. Both ores select property-free cube
block models and direct block-item models with their same-named textures. Ingredients orders
Emerald, Lapis Lazuli, Diamond, Ancient Debris, Nether Quartz, Amethyst Shard. Natural Blocks
orders the Emerald-Ore pair, Lapis-Ore pair, then Diamond Ore and Deepslate Diamond Ore.

Trim projection has `diamond` and `diamond_darker` palettes, `29` compatible armor item-model
overlays and atlas-driven equipped trim. There is no conditional loose-item model, tint, animation
or special renderer.

**Branches and aborts:**

Default/patched stack; two Ore/Silk/Fortune/explosion/XP paths; four cooking and five generation
paths; seventeen direct loot/archaeology rows and three overlays; 56 recipes/55 listeners plus
story progression; two repair tags, Beacon, baseline/rebalanced merchants; trim tag/provider and
normal/darker asset; zero template identities; persistence/reload/wire/client paths are distinct.

**Constants and randomness:**

Diamond ID `926`; Ore block/item/state `203/105/5307`, `204/106/5308`; stack `64`; Ore
strength `3/3`, `4.5/3`, XP `3..7`; cooking `200/100/1.0`; configured
size/discard `4/0.5`, `8/1`, `12/0.7`, `8/0.5`; placement `7/4/1-in-9/2`; fossil
`1-in-64`, rot `0.1`; direct loot rows `17`; recipes/listeners/direct Diamond unlocks
`56/55/12`; templates/matches `1212/0`; repair targets `10`; baseline offer inclusions
`2/5,2/3,1`; rebalanced raw offers `10` at `2/3`; trim `#6EECD2`, recipes/models `18/29`.

**Side effects:**

Ore loot/XP and worldgen state; machine result/XP; seventeen loot/archaeology outputs; crafted
equipment, devices, templates and Firework Star; knowledge/story progress; repaired equipment;
Beacon effect payment; merchant inputs/outputs; trimmed armor; durable stack/container state,
synchronization and exact client projection.

**Gates:**

Correct live tool/Silk/Fortune/explosion; cooking machine/input/capacity; placement/biome/
processor protection; loot selection; exact grid/tag/special-recipe/result capacity and
knowledge; anvil target/tag/cost/capacity; Beacon tag/pyramid/effects; profession/level/variant/
trade set and both costs; trim tag/provider; registry/stack/equipment decode and client resources.

**State read/written:**

Reads all gates above and writes only the loot, XP, ore, processing, container, crafting,
advancement, repair, Beacon, offer, Firework, trimmed-equipment, durable, wire and projection state
listed above.

**Failure behavior:**

Wrong tools emit no Ore loot or XP. Wrong machine/input/capacity commits no cook. Failed feature,
processor or protected write changes nothing. Unselected loot/trade candidates emit no Diamond or
offer. Wrong grid, unavailable recipe or duplicate Firework modifier emits no result. Rejected
repair/Beacon/merchant/trim input consumes nothing. Reload affects future evaluation only.

**Boundary cases and quirks:**

Silk replaces both loose Diamond and ordinary break XP; Fortune changes Diamond count but not XP.
All four ordinary Ore placements coexist in every Overworld biome, while only the three
desert/swamp biomes add processor-created Deepslate Diamond Ore. Diamond is both an exact
Firework-trail identity and three independently reloadable singleton-tag members. It repairs ten
humanoid/tool targets but neither Diamond mount armor. Trade Rebalance removes one ordinary
Diamond purchase yet adds ten variant-specific second-cost sinks. Diamond trim on Diamond armor
uses the darker override rather than its normal palette.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-REPORT-001`; `OFF-DATA-001`;
`net.minecraft.world.item.Items`; `net.minecraft.world.level.block.Blocks`;
`net.minecraft.world.level.block.DropExperienceBlock#spawnAfterBreak`;
`net.minecraft.world.item.ToolMaterial`;
`net.minecraft.world.item.equipment.ArmorMaterials`;
`net.minecraft.world.inventory.BeaconMenu$PaymentSlot`;
`net.minecraft.world.inventory.BeaconMenu#quickMoveStack`;
`net.minecraft.world.inventory.BeaconMenu#updateEffects`;
`net.minecraft.world.item.crafting.FireworkStarRecipe`;
`net.minecraft.world.item.crafting.SmithingTrimRecipe`;
`net.minecraft.world.entity.npc.villager.AbstractVillager#addOffersFromTradeSet`;
`net.minecraft.world.entity.npc.villager.AbstractVillager#addOffersFromItemListingsWithoutDuplicates`;
`net.minecraft.world.item.trading.VillagerTrade#getOffer`;
`net.minecraft.world.item.trading.TradeSet#calculateNumberOfTrades`;
`net.minecraft.world.item.CreativeModeTabs`;
`reports/blocks.json#minecraft:{diamond_ore,deepslate_diamond_ore}`;
`reports/registries.json#minecraft:{block,item,recipe,recipe_serializer,loot_table,advancement,villager_trade,trade_set,trim_material,worldgen}`;
`reports/minecraft/components/item/{diamond,diamond_pickaxe,diamond_spear,diamond_helmet,diamond_horse_armor,diamond_nautilus_armor}.json`;
`data/minecraft/tags/item/{beacon_payment_items,diamond_tool_materials,repairs_diamond_armor,trim_materials}.json`;
`data/minecraft/trim_material/diamond.json`;
`data/minecraft/loot_table/{blocks/{diamond_ore,deepslate_diamond_ore},archaeology/desert_pyramid,chests/**/*.json,pots/trial_chambers/corridor}.json`;
`data/minecraft/recipe/{diamond*,enchanting_table,firework_star,jukebox,*armor_trim_smithing_template*,*_armor_trim_smithing_template_smithing_trim}.json`;
`data/minecraft/advancement/{story/mine_diamond,recipes/**/*.json}`;
`data/minecraft/{villager_trade/{armorer/3,toolsmith/4,weaponsmith/4}/diamond_emerald,tags/villager_trade/{armorer/level_3,toolsmith/level_4,weaponsmith/level_4},trade_set/{armorer/level_3,toolsmith/level_4,weaponsmith/level_4}}.json`;
`data/minecraft/datapacks/trade_rebalance/data/minecraft/{loot_table/chests/{abandoned_mineshaft,desert_pyramid,jungle_temple},villager_trade/armorer/5/emerald_and_diamond_*,tags/villager_trade/armorer/{level_3,level_5}}.json`;
`data/minecraft/worldgen/{configured_feature/{ore_diamond_*,fossil_diamonds},placed_feature/{ore_diamond*,fossil_lower},processor_list/fossil_diamonds,biome/*.json}`;
`data/minecraft/structure/**/*.nbt`;
`assets/minecraft/{items,models/item,textures/item}/diamond.*`;
`assets/minecraft/{blockstates,models/block,textures/block}/*diamond_ore*`;
`assets/minecraft/{atlases,models/item,textures/trims}/**/*diamond*`;
`BLK-BEACON-001`; `BLK-BEACON-STORAGE-001`; `ITM-SMITHING-TEMPLATE-001`;
`ITM-RECIPE-SERIALIZER-001`; `WGEN-PIPELINE-001`; `EXP-ITM-083`.

**Test vectors:**

Run `EXP-ITM-083` across default/patched Diamond, all Ore tool/enchantment/explosion/XP endpoints,
four cooking and five generation paths, every direct loot row under baseline/rebalanced tables,
all 56 recipes/55 listeners and story criterion, ten repair targets/two mount rejects, Beacon,
baseline/rebalanced merchant sets and all 18 trim recipes under independent tag/provider reload.
Scan every template, persist/reload/synchronize all owners and assert IDs, names, cube/generated
models, normal/darker trim palettes/overlays and both tab orders.

**Limits:**

Generic breaking/XP, processing, crafting, special Firework-Star, Smithing, loot, brushable,
merchant, anvil, Beacon, feature/processor, packet and renderer control flow remains with cited
owners. Diamond Block behavior remains `BLK-BEACON-STORAGE-001`; Diamond tools, armor, templates
and device outputs retain their dedicated owners. This leaf fixes the exact loose item,
source/sink joins, absences and projection.
