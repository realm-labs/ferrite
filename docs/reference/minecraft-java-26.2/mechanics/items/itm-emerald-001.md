# Items, inventories and progression mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ITM-EMERALD-001` — Emerald joins mountain ore, loot and Illager acquisition to the complete merchant economy, Beacon payment and armor trim

**Parent:** `SIM-004`, `SIM-005`, `SIM-RANDOM-001`, `BLK-001`, `BLK-002`,
`BLK-003`, `BLK-005`, `BLK-BREAK-001`, `BLK-BREAK-HOOK-001`, `PLY-002`,
`PLY-005`, `PLY-006`, `PLY-INPUT-001`, `PLY-INTERACT-001`, `PLY-BREAK-001`,
`ITM-001`, `ITM-002`, `ITM-003`, `ITM-004`, `ITM-005`, `ITM-006`, `ITM-007`,
`ITM-USE-001`, `ITM-CONTAINER-001`, `ITM-RECIPE-001`,
`ITM-RECIPE-SERIALIZER-001`, `ITM-CRAFT-001`, `ITM-FURNACE-001`,
`ITM-SMITHING-001`, `ITM-LOOT-001`, `ITM-ADVANCEMENT-001`, `ITM-ANVIL-001`,
`ITM-SMITHING-TEMPLATE-001`, `BLK-BEACON-001`,
`BLK-BEACON-STORAGE-001`, `BLK-BRUSHABLE-001`, `ENT-001`, `MOB-001`,
`MOB-004`, `MOB-AI-001`, `ENV-001`, `ENV-002`, `ENV-003`,
`WGEN-PIPELINE-001`, `WGEN-STRUCTURE-IGLOO-001`, `CLI-001`, `CLI-006`,
`CLI-UI-001`, `CLI-EFFECT-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked registration/components, both ore blocks and tables, all recipes and
advancements, complete loot, trade, trim and mountain-placement records, all `1,212` templates and
exact client resources determine every Emerald-specific branch. Generic breaking, processing,
crafting, Smithing, loot, brush, merchant, Beacon, worldgen, persistence, packet and rendering
algorithms retain the cited owners.

**Applies when:**

`minecraft:emerald` is mined, cooked, killed for, looted, brushed, crafted, used in a Beacon,
merchant offer or armor trim, moved, renamed, persisted, synchronized or rendered before and
after recipe, advancement, loot, tag, trade, trim, worldgen or resource reload.

**Authoritative state:**

Emerald is raw item ID `927`, a common nondamageable plain `Item` with maximum stack `64`. Its
ordinary components include `provides_trim_material=minecraft:emerald`. Its only two direct item
tags are `beacon_payment_items` and `trim_materials`. It has no food, consumable, remainder, fuel,
compost, equipment, durability, projectile, cooldown, inventory-tick, repair-material or
identity-specific use branch.

The coupled source blocks are property-free `DropExperienceBlock` instances:

| Block | block/item/state IDs | map/sound | hardness/resistance |
|---|---:|---|---:|
| Emerald Ore | `398/101/9573` | `STONE` / Stone | `3/3` |
| Deepslate Emerald Ore | `399/102/9574` | `DEEPSLATE` / Deepslate | `4.5/3` |

Both use `BASEDRUM`, require a correct tool, and are direct `emerald_ores`,
`mineable/pickaxe` and `needs_iron_tool` members. Ordinary Emerald Ore alone is additionally a
direct `snaps_goat_horn` member. Their full-cube physics and generic XP hook remain with the block
owners.

**Transition and ordering:**

### Ore break, processing, goat and generation

Each ore table has one ordered alternative. Silk Touch level at least one emits one default
matching Ore block. Otherwise it emits one default Emerald, applies `ore_drops` Fortune
multiplication, then explosion decay. A correct non-Silk player break independently draws integer
XP `3..7`; Fortune does not change it. A tool that is not both pickaxe-effective and admitted for
the live iron-tier requirement yields neither ordinary loot nor break XP. Silk suppresses that
ordinary XP. Named sequences are
`minecraft:blocks/{emerald_ore,deepslate_emerald_ore}`.

When the Goat ram owner reaches its front-position or above-position live
`snaps_goat_horn` test, ordinary Emerald Ore can select the horn-drop/sound/finish path.
Deepslate Emerald Ore cannot. Membership does not itself schedule a ram or otherwise alter either
ore.

Four exact cooking records emit one default Emerald with recipe XP `1.0`. Furnace accepts each Ore
in a separate record at omitted/default time `200`; Blast Furnace accepts each in a separate
record at omitted/default time `100`. Smoker and Campfire reject both. Every record has its own
advancement, unlocked by possession of its exact Ore or prior knowledge. Input patches are not
copied.

Configured feature `ore_emerald` has size `3` and air-exposure discard chance `0`. Its ordered
targets are live `stone_ore_replaceables` to state `9573`, then
`deepslate_ore_replaceables` to state `9574`. Placed feature `ore_emerald` makes `100`
attempts, applies in-square, trapezoid absolute height `-16..480`, then biome admission.

Exactly ten biomes schedule it: Cherry Grove, Frozen Peaks, Grove, Jagged Peaks, Meadow, Snowy
Slopes, Stony Peaks, Windswept Forest, Windswept Gravelly Hills and Windswept Hills. Other biomes
do not. Target traversal, position draws, air test, protected-block admission and writes remain
`WGEN-PIPELINE-001`.

### Thirty-two direct loose-item acquisition tables

Every listed pool can select Emerald repeatedly when it has multiple rolls:

| Table / pool | rolls | Emerald weight / pool total | count |
|---|---:|---:|---:|
| archaeology/desert_pyramid `0` | `1` | `1/8` | `1` |
| archaeology/desert_well `0` | `1` | `1/8` | `1` |
| archaeology/ocean_ruin_cold `0` | `1` | `2/15` | `1` |
| archaeology/ocean_ruin_warm `0` | `1` | `2/15` | `1` |
| archaeology/trail_ruins_common `0` | `1` | `2/45` | `1` |
| chests/buried_treasure `2` | `1..3` | `5/15` | `4..8` |
| chests/desert_pyramid `0` | `2..4` | `15/247` | `1..3` |
| chests/end_city_treasure `0` | `2..6` | `2/89` | `2..6` |
| chests/igloo_chest `0` | `2..8` | `1/63` | `1` |
| chests/jungle_temple `0` | `2..6` | `2/89` | `1..3` |
| chests/shipwreck_treasure `0` | `3..6` | `40/150` | `1..5` |
| chests/trial_chambers/reward_common `0` | `1` | `4/25` | `2..4` |
| chests/trial_chambers/reward_ominous_common `0` | `1` | `5/15` | `4..10` |
| chests/trial_chambers/reward_rare `0` | `1` | `3/23` | `2..4` |
| chests/underwater_ruin_big `0` | `2..8` | `1/33` | `1` |
| chests/underwater_ruin_small `0` | `2..8` | `1/30` | `1` |
| chests/village/village_armorer `0` | `1..5` | `1/8` | `1` |
| chests/village/village_butcher `0` | `1..5` | `1/28` | `1` |
| chests/village/village_desert_house `0` | `3..8` | `1/36` | `1..3` |
| chests/village/village_fisher `0` | `1..5` | `1/11` | `1` |
| chests/village/village_fletcher `0` | `1..5` | `1/23` | `1` |
| chests/village/village_mason `0` | `1..5` | `1/13` | `1` |
| chests/village/village_plains_house `0` | `3..8` | `2/43` | `1..4` |
| chests/village/village_savanna_house `0` | `3..8` | `2/46` | `1..4` |
| chests/village/village_shepherd `0` | `1..5` | `1/23` | `1` |
| chests/village/village_snowy_house `0` | `3..8` | `1/53` | `1..4` |
| chests/village/village_taiga_house `0` | `3..8` | `2/54` | `1..4` |
| chests/village/village_tannery `0` | `1..5` | `1/16` | `1..4` |
| chests/village/village_temple `0` | `3..8` | `1/19` | `1..4` |
| entities/evoker `1` | `1`, player-kill gated | sole entry | `0..1 + Looting` |
| entities/vindicator `0` | `1`, player-kill gated | sole entry | `0..1 + Looting` |
| pots/trial_chambers/corridor `0` | `1` | `125/351` | `1..3` |

Each Illager entry first draws uniform integer base `B in 0..1`. With a living attacking entity
and Looting level `L>0`, it then adds `round(L*U)` for a fresh uniform float `U` in `[0,1)`.
Only a positive final count emits. Both pools require `killed_by_player`; Evoker evaluates its
guaranteed Totem pool first.

Trade Rebalance replaces Desert-Pyramid and Jungle-Temple chest tables. The rows, rolls, weights
and counts remain; only the Desert-Pyramid pool total changes from `247` to `237`. Loot
installation, named sequences, brush admission and container/pot commit remain with their owners.
No fishing, gift or barter table directly emits loose Emerald.

### Twenty-four recipes and progression

Emerald participates in `24` recipes:

- Emerald Block compression consumes `9`, while the shapeless reverse emits `9`;
- four exact Ore-cooking records emit Emerald; and
- `18` generic armor-trim Smithing records admit one Emerald in the addition slot.

All `24` have distinct recipe advancements. Exact Emerald possession is the inventory alternative
for Emerald Block compression; reverse crafting listens for the Block, cooking listens for the
respective Ore and trim records listen for their templates. Grid placement, machine/result
capacity, atomic consumption, Smithing copy semantics and knowledge publication remain generic.

`adventure/trade` and `adventure/trade_at_world_height` use Emerald only as their display icon.
Their criteria are respectively any `villager_trade` and a `villager_trade` while player
`y >= 319`; owning, paying or receiving Emerald is not an admission condition.

### Beacon, complete merchant surface and armor trim

The Beacon payment slot and quick-move logic accept Emerald only while it is a live
`beacon_payment_items` member. Selection consumes one payment on successful effect update;
pyramid validation, primary/secondary effect gates, UI synchronization and aborts remain
`BLK-BEACON-001`.

Every one of the `388` baseline `villager_trade` records references exact Emerald in exactly one
cost/output role. The closed role partition is:

| Profession | first cost (`wants`) | second cost (`additional_wants`) | output (`gives`) |
|---|---:|---:|---:|
| Armorer | `13` | `0` | `2` |
| Butcher | `3` | `0` | `8` |
| Cartographer | `28` | `0` | `3` |
| Cleric | `5` | `0` | `6` |
| Farmer | `8` | `0` | `6` |
| Fisherman | `3` | `2` | `11` |
| Fletcher | `6` | `1` | `5` |
| Leatherworker | `8` | `0` | `4` |
| Librarian | `11` | `0` | `4` |
| Mason | `40` | `0` | `6` |
| Shepherd | `66` | `0` | `20` |
| Smith | `1` | `0` | `2` |
| Toolsmith | `11` | `0` | `2` |
| Wandering Trader | `91` | `0` | `6` |
| Weaponsmith | `4` | `0` | `2` |
| **Total** | **`298`** | **`3`** | **`87`** |

The three baseline second-cost records each require one Emerald: Fisherman level one exchanges six
Cod plus one Emerald for six Cooked Cod, Fisherman level two exchanges six Salmon plus one Emerald
for six Cooked Salmon, and Fletcher level one exchanges ten Gravel plus one Emerald for ten
Flint. Baseline fixed Emerald counts span `0..9`, `12..14`, `16` and `36` for first costs and
`1..3` for outputs; zero base costs are completed by their record modifiers rather than making a
free transaction.

The `68` baseline trade sets select through `73` villager-trade tags. Profession, level, variant,
candidate count, random sequence and per-record predicates determine which records become offers.
Each selected record then preserves its exact declared costs/results, counts, component
predicates, result modifiers, maximum uses, XP, discount and enchantment-price fields. Offer
construction consumes no Emerald. Successful generic transactions validate both costs, consume
the current adjusted amounts, transfer the output, increment uses and apply merchant/player
effects atomically.

Trade Rebalance contributes `81` replacement Armorer/Librarian records, again with exactly one
Emerald role each:

| Profession | first cost | second cost | output |
|---|---:|---:|---:|
| Armorer | `49` | `0` | `3` |
| Librarian | `28` | `0` | `1` |
| **Total** | **`77`** | **`0`** | **`4`** |

Rebalanced fixed first costs include `0`, `2..9`, `11..13`, `16`, `18` and `36`; its dynamic
master-level enchanted-book prices are `11 + uniform(0..35)` or
`8 + uniform(0..25)` before generic price adjustment. Rebalanced Emerald outputs have counts
`1`, `4` or `42`. Overlay tags replace only the affected Armorer/Librarian candidates; unaffected
baseline professions remain. Exact variant predicates, enchantment/result modifiers, selection,
price/demand/reputation adjustment, exhaustion, restock and menu synchronization remain
merchant-owned.

The Igloo bottom template separately persists a Plains novice Cleric with two fixed offers:
`36` Rotten Flesh gives `1` Emerald and `9` Gold Ingots gives `1` Emerald. Each begins at uses
`0`, has maximum uses `7` and `rewardExp=true`. These two stored output stacks account for the only
two exact Emerald UTF occurrences across all `1,212` templates; they are not loose template
items and do not pass through the reloadable trade-set constructor.

Emerald's default provider resolves trim material `minecraft:emerald`, description color
`#11A036` and asset `emerald`; it has no equipment-specific override. As a live
`trim_materials` member Emerald fills the addition slot of all `18` generic trim recipes, is
consumed once and writes the Emerald material holder into copied armor. Removing the tag rejects
it; removing or replacing the provider changes material resolution independently after recipe
admission.

**Persistence and reload boundary:**

Stacks persist identity, count and patches. Ores, machines, knowledge, Beacon state, containers,
brushable blocks, entities, offers and trimmed equipment persist with their owners. Recipe,
advancement, loot, tag, trade, trim and worldgen reload changes only future evaluation; completed
mining, processing, loot, crafts, payments, offers, trims and generated chunks are not replayed or
rewritten. Existing constructed or template-stored offers retain their costs/results. Resource
reload independently changes projection only.

**Wire and client projection:**

Generic stack publication uses item ID `927`; Ore block/item/state IDs are
`398/101/9573` and `399/102/9574`. No Emerald-specific packet exists. English names are
`Emerald`, `Emerald Ore`, `Deepslate Emerald Ore` and `Emerald Material`.

Loose Emerald selects one untinted `item/generated` flat. Both ores select property-free cube
block models and direct block-item models with their same-named textures. Ingredients orders Raw
Gold, Emerald, Lapis Lazuli, Diamond, Ancient Debris, Nether Quartz and Amethyst Shard. Natural
Blocks orders Emerald Ore and Deepslate Emerald Ore immediately before the Lapis-Ore pair.

Trim projection uses the `emerald` palette, `29` compatible armor item-model overlays and
atlas-driven equipped trim. There is no conditional loose-item model, tint, animation, special
renderer or self-material override.

**Branches and aborts:**

Default/patched stack; two Ore/Silk/Fortune/explosion/XP paths; Goat tag; four cooking and one
placement path across ten biomes; 32 direct loot tables and two overlays; 24 recipes/listeners;
Beacon; 388 baseline, 81 overlay and two fixed-template offers; trim tag/provider; persistence,
reload, wire and client paths are distinct.

**Constants and randomness:**

Emerald ID `927`; Ore block/item/state `398/101/9573`, `399/102/9574`; stack `64`; Ore
strength `3/3`, `4.5/3`, XP `3..7`; cooking `200/100/1.0`; ore size/discard
`3/0`, attempts/height `100/-16..480`, biomes `10`; direct loot tables `32`, overlays `2`;
recipes/listeners/direct Emerald unlocks `24/24/1`; baseline records/roles
`388=298+3+87`, sets/tags `68/73`; overlay records/roles `81=77+0+4`; templates/exact
Emerald occurrences/Ore cells `1212/2/0`; trim `#11A036`, recipes/models `18/29`.

**Side effects:**

Ore loot/XP, Goat-horn ram resolution and worldgen state; machine result/XP; 32 loot/archaeology/
entity outputs; crafted storage and knowledge; Beacon effect payment; merchant inputs/outputs;
trimmed armor; durable stack/container/entity/offer state, synchronization and exact client
projection.

**Gates:**

Correct live tool/Silk/Fortune/explosion; Goat ram reach/tag; cooking machine/input/capacity;
placement/biome/target/write; loot selection/player kill; exact grid/tag/result capacity and
knowledge; Beacon tag/pyramid/effects; profession/level/variant/trade set and both costs; trim
tag/provider; registry/stack decode and client resources.

**State read/written:**

Reads all gates above and writes only the loot, XP, ore, Goat, processing, container, crafting,
advancement, Beacon, offer, trimmed-equipment, durable, wire and projection state listed above.

**Failure behavior:**

Wrong tools emit no Ore loot or XP. Failed ram/tag, machine/input/capacity, feature or protected
write has no Emerald-specific effect. Unselected or player-kill-rejected loot emits no Emerald.
Wrong grid or unavailable recipe emits no result. Rejected Beacon, merchant or trim input consumes
nothing. Reload affects future evaluation only.

**Boundary cases and quirks:**

Silk replaces both loose Emerald and ordinary break XP; Fortune changes Emerald count but not XP.
Ordinary Ore alone can finish a Goat ram. One high-altitude placement spans only ten mountain/
windswept biomes. Emerald is the unique common currency across every baseline trade record, yet
three recipes use it as a second cost and 87 use it as the output. Trade Rebalance replaces only
Armorer/Librarian records and adds dynamic book prices. The Igloo's two persisted outputs survive
independently of trade reload. Advancement icons do not make Emerald part of their criteria.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-REPORT-001`; `OFF-DATA-001`;
`net.minecraft.world.item.Items`; `net.minecraft.world.level.block.Blocks`;
`net.minecraft.world.level.block.DropExperienceBlock#spawnAfterBreak`;
`net.minecraft.world.entity.ai.behavior.RamTarget`;
`net.minecraft.world.inventory.BeaconMenu$PaymentSlot`;
`net.minecraft.world.inventory.BeaconMenu#quickMoveStack`;
`net.minecraft.world.inventory.BeaconMenu#updateEffects`;
`net.minecraft.world.item.crafting.SmithingTrimRecipe`;
`net.minecraft.world.entity.npc.villager.AbstractVillager#addOffersFromTradeSet`;
`net.minecraft.world.entity.npc.villager.AbstractVillager#addOffersFromItemListingsWithoutDuplicates`;
`net.minecraft.world.item.trading.VillagerTrade#getOffer`;
`net.minecraft.world.item.trading.TradeSet#calculateNumberOfTrades`;
`net.minecraft.world.item.CreativeModeTabs`;
`reports/blocks.json#minecraft:{emerald_ore,deepslate_emerald_ore}`;
`reports/registries.json#minecraft:{block,item,recipe,recipe_serializer,loot_table,advancement,villager_trade,trade_set,trim_material,worldgen}`;
`reports/minecraft/components/item/{emerald,emerald_ore,deepslate_emerald_ore}.json`;
`data/minecraft/tags/{item/{beacon_payment_items,emerald_ores,trim_materials},block/{emerald_ores,mineable/pickaxe,needs_iron_tool,snaps_goat_horn}}.json`;
`data/minecraft/trim_material/emerald.json`;
`data/minecraft/loot_table/{blocks/{emerald_ore,deepslate_emerald_ore},archaeology/*.json,chests/**/*.json,entities/{evoker,vindicator},pots/trial_chambers/corridor}.json`;
`data/minecraft/recipe/{emerald,emerald_block,emerald_from_*,*_armor_trim_smithing_template_smithing_trim}.json`;
`data/minecraft/advancement/{adventure/{trade,trade_at_world_height},recipes/**/*.json}`;
`data/minecraft/{villager_trade,trade_set,tags/villager_trade}/**/*.json`;
`data/minecraft/datapacks/trade_rebalance/data/minecraft/{loot_table/chests/{desert_pyramid,jungle_temple},villager_trade,trade_set,tags/villager_trade}/**/*.json`;
`data/minecraft/worldgen/{configured_feature/ore_emerald,placed_feature/ore_emerald,biome/*.json}`;
`data/minecraft/structure/**/*.nbt`;
`assets/minecraft/{items,models/item,textures/item}/emerald.*`;
`assets/minecraft/{blockstates,models/block,textures/block}/*emerald_ore*`;
`assets/minecraft/{atlases,models/item,textures/trims}/**/*emerald*`;
`BLK-BEACON-001`; `BLK-BEACON-STORAGE-001`; `ITM-SMITHING-TEMPLATE-001`;
`ITM-RECIPE-SERIALIZER-001`; `WGEN-STRUCTURE-IGLOO-001`;
`WGEN-PIPELINE-001`; `EXP-ITM-084`.

**Test vectors:**

Run `EXP-ITM-084` across default/patched Emerald, all Ore tool/enchantment/explosion/XP and Goat
endpoints, four cooking and ten-biome generation paths, every direct loot row under baseline/
rebalanced tables, all 24 recipes/listeners, Beacon, all 388 baseline/81 overlay/two persisted
offers and all 18 trim recipes under independent tag/provider reload. Scan every template,
persist/reload/synchronize all owners and assert IDs, names, cube/generated models, trim palette/
overlays and both tab orders.

**Limits:**

Generic breaking/XP, Goat AI, processing, crafting, Smithing, loot, brushable, merchant, Beacon,
feature, packet and renderer control flow remains with cited owners. Emerald Block behavior
remains `BLK-BEACON-STORAGE-001`; traded input/output item behavior retains its own leaves. This
leaf fixes the exact loose item, source/sink joins, absences and projection.
