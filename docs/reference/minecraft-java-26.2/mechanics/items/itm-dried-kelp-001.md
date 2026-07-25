# Items mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ITM-DRIED-KELP-001` — Dried Kelp joins three Kelp-cooking paths and block compaction to fast food, compost, fuel and a Butcher purchase

**Parent:** `PLY-005`, `PLY-006`, `PLY-INPUT-001`, `PLY-INTERACT-001`,
`ITM-001`, `ITM-002`, `ITM-003`, `ITM-004`, `ITM-005`, `ITM-006`,
`ITM-007`, `ITM-USE-001`, `ITM-CONTAINER-001`, `ITM-RECIPE-001`,
`ITM-CRAFT-001`, `ITM-FURNACE-001`, `ITM-CAMPFIRE-001`,
`ITM-LOOT-001`, `ITM-ADVANCEMENT-001`, `ITM-HUNGER-001`,
`ITM-ANVIL-001`, `ENT-001`, `MOB-WANDERING-TRADER-001`,
`BLK-BREAK-HOOK-001`, `ENV-FIRE-001`, `WGEN-PIPELINE-001`,
`CLI-001`, `CLI-006`, `CLI-UI-001`, `CLI-EFFECT-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked registrations/components, the hard-coded food/consumable, Composter and
fuel entries, five recipes/unlocks, Balanced Diet, the sole Butcher record, Kelp loot/trade/
worldgen inputs, block loot/fire/tag records, the complete structure census and exact client
resources determine every Dried-Kelp-specific branch. Generic active use, processing, crafting,
Composter, furnace fuel, merchant, loot, fire, feature, stack and client algorithms remain with
the cited owners.

**Applies when:**

A `minecraft:dried_kelp` stack is cooked, crafted, eaten, composted, moved, persisted,
synchronized or rendered; when its compacted Dried Kelp Block is placed, broken, burned, used as
fuel, composted, selected by a Sulfur Cube or sold to a Butcher; or when Kelp loot, a Wandering
Trader or either ocean Kelp feature supplies the processing input before and after component,
recipe, advancement, tag, trade, loot or resource reload.

**Authoritative state:**

`minecraft:dried_kelp` is raw item ID `1136`. It is a common, nondamageable plain `Item` with
maximum stack `64`. Its operational defaults are:

- `minecraft:food={nutrition:1,saturation:0.6}` with omitted/default
  `can_always_eat=false`;
- an otherwise-default eat consumable with `consume_seconds=0.8` (`16` ticks) and no consume
  effects.

The remaining defaults are ordinary empty modifiers, enchantments and lore, item-break sound,
translated name, direct item-model key, repair cost, swing animation, tooltip display and use
effects. It has no direct item-tag membership, remainder, equipment, durability, repairable, tool,
projectile, cooldown, inventory tick or identity-specific nonfood use.

Dried Kelp Block is the coupled property-free block/item at block protocol ID `744`, sole state
`15089` and raw item ID `1056`. It is an ordinary full cube with `COLOR_GREEN`, strength
`0.5/2.5`, Grass sounds, no block entity and direct `mineable/hoe` membership; correct-tool
restriction is absent. Its common 64-stack block item has ordinary components and is a direct
`sulfur_cube_archetype/fast_flat` member.

**Transition and ordering:**

### Player consumption and progression

In-air player use enters the consumable path only when the food listener admits it. Ordinary
survival at food level `20` returns `FAIL`; lower hunger admits use. Block interaction remains
block-first. Removing only food leaves the consumable and admits use at full hunger but applies no
nutrition. Removing only the consumable makes in-air use pass. Patched food/consumable values
control later uses.

Interruption, release or live-hand/component replacement before completion commits no statistic,
criterion, food, event or shrink. Successful server completion applies the generic eat transaction:
final effects, used-item statistic, pre-shrink `consume_item` criterion, food, `EAT`, then one-item
consumption unless the user has infinite materials. Additions are nutrition `1` and saturation
`0.6`, clamped by the hunger owner. No probability draw, status-effect offer or remainder occurs.

Dried Kelp is one of the 40 independent AND requirements in the telemetry-enabled
`husbandry/balanced_diet` challenge. Its pre-shrink consume criterion advances its row; all 40 rows
are required for the `100`-XP reward.

### Kelp processing and recipe discovery

Three exact Kelp-input records emit one default Dried Kelp and copy no input patch:

| Recipe | Domain | Time | Recipe XP |
|---|---|---:|---:|
| `dried_kelp_from_smelting` | Furnace | omitted/default `200` | `0.1` |
| `dried_kelp_from_smoking` | Smoker | omitted/default `100` | `0.1` |
| `dried_kelp_from_campfire_cooking` | Campfire | explicit `600` | `0.1` |

All have category `food` and omit group. Blast Furnace rejects them. Each recipe advancement has
one OR requirement containing exact Kelp possession and prior unlock of that same recipe; either
criterion grants only its matching recipe.

Furnace/Smoker progress, capacity, recipe-use accounting and extraction XP remain
`ITM-FURNACE-001`. Campfire insertion chooses the first empty slot and schedules its explicit
timer; completion drops the result at the block before clearing the slot. Its data XP field does
not create a furnace-style extraction award.

Two exact compacting records complete the family:

- `dried_kelp_block` is a full shaped `3×3` of nine exact Dried Kelp and emits one default Dried
  Kelp Block;
- `dried_kelp` is shapeless; one exact Dried Kelp Block emits nine default Dried Kelp.

The compression advancement unlocks from exact Dried Kelp or prior recipe unlock. Decompression
unlocks from exact Dried Kelp Block or prior unlock. Components are ignored for exact ingredient
matching and are not copied. Together the family has five recipes and five ordinary recipe
advancements.

### Loose and compacted Composter, fuel, fire and block loot

Composter admission is code-built by exact identity rather than a tag. Loose Dried Kelp has Java
float chance `0.3f`; Dried Kelp Block has chance `0.5f`. An admitted direct or automated attempt
at level zero succeeds without RNG. At levels `1..6`, success is strict
`nextDouble() < chance`; failed probability leaves level/count unchanged except for the caller's
documented effect path. Level-seven extraction and delayed transition remain generic. The loose
and compacted values do not derive from their `9:1` recipe ratio.

Loose Dried Kelp is not fuel. Dried Kelp Block is registered in the fresh vanilla `FuelValues`
snapshot for `1 + baseCookingTime*20`; at default base `200`, its burn duration is exactly `4001`
ticks. Furnace-family machines consume one block on ignition and retain their ordinary burn/
progress ordering. The one-tick surplus is code-built and is not rounded down to `4000`.

Placed Dried Kelp Block has FireBlock ignite/burn odds `30/60`; it is not lava-ignitable by a
separate property. Fire support/spread/destruction remains `ENV-FIRE-001`. Ordinary block break
reaches one exact self entry behind `survives_explosion`, with sequence
`minecraft:blocks/dried_kelp_block`. Any tool or hand can produce it because the block has no
correct-tool requirement; Hoe merely selects faster mining through its tag.

Its block item selects `fast_flat`. That archetype supplies horizontal/vertical knockback powers
`0.9125/0.09`, hit/push sounds, push cooldown `0.9`, impulse threshold `0.03` and its five
attribute modifiers. Loose Dried Kelp does not match the archetype.

### Trade and Kelp acquisition boundary

Butcher level four has exactly one tagged trade record even though its trade-set amount is two.
The resulting guaranteed single offer wants ten Dried Kelp Blocks, gives one Emerald, has maximum
uses `12`, Villager XP `30` and reputation discount `0.05`. Ordinary offer economy, demand,
restock and input-patch predicates remain merchant-owned. Trade Rebalance does not replace it.
Ten blocks represent 90 loose Dried Kelp before any recipe loss.

No locked loot or merchant record directly emits loose Dried Kelp or its block. Initial Kelp can
instead come from:

- exact Kelp and Kelp-Plant block loot, each one Kelp behind `survives_explosion` and its own
  named block sequence;
- the Wandering Trader common set's three-Emerald-to-one-Kelp offer, one of 76 distinct candidates
  from which five are selected without duplicates (`5/76` inclusion);
- configured feature `kelp`, scheduled as `kelp_cold` in Ocean, Deep Ocean, Cold Ocean and Deep
  Cold Ocean and as `kelp_warm` in Lukewarm Ocean and Deep Lukewarm Ocean.

Both placed features use noise factor `80`, in-square, `OCEAN_FLOOR_WG` and biome. Cold uses
noise-to-count ratio `120`; warm uses `80`. Feature execution ignores origin Y, starts at the
ocean-floor exact-water cell, draws length `1..10`, writes body segments and offers a terminal age
`20..23` head under the exact `WGEN-PIPELINE-001` transaction. It creates Kelp, never Dried Kelp;
processing remains a later transaction.

An exhaustive string census of all 1,212 locked structure NBT files finds no Kelp, Dried Kelp or
Dried Kelp Block identity. There is no direct chest, archaeology, fishing, gift, mob-drop,
brewing, animal-food or dispenser branch for loose Dried Kelp.

**Persistence and reload boundary:**

Stacks persist identity, count and patches. They do not own active-use time, hunger, Composter,
machine/Campfire, knowledge, placed block, fuel, fire, merchant, loot or worldgen state.
Recipe/tag/advancement/trade/loot reload changes only future evaluation in its domain; code-built
food/consumable defaults, Composter chances, fresh vanilla fuel construction and exact fire odds
remain fixed until code/registry reconstruction. Completed work, offers, loot and chunks are not
replayed. Resource reload independently changes names/models/textures.

**Wire and client projection:**

Generic stack codecs publish raw item ID `1136`, count and patches. Placed block publication uses
state `15089`; its item uses ID `1056`. No Dried-Kelp packet exists.

English names are `Dried Kelp` and `Dried Kelp Block`. Loose Dried Kelp selects a like-named
`item/generated` model and texture. The blockstate and block item select one opaque full-cube model
with distinct top, bottom and side textures; south/east faces reverse U coordinates.

Food & Drinks places Dried Kelp after Golden Dandelion and before Beef. Natural Blocks places
Dried Kelp Block after Kelp and before Tube Coral Block. Neither appears in another ordinary tab.

**Branches and aborts:**

Food/consumable component combinations; hunger/full/infinite/interrupted completion; Balanced Diet;
three cooking domains and five unlocks; loose/block Composter values; loose nonfuel versus block
fuel; placed fire and self loot; compact/decompact; Butcher and Kelp-trader selection; cold/warm
features and six biomes; zero templates; persistence/reload/wire/client paths are distinct.

**Constants and randomness:**

Item ID `1136`; block/item/state `744/1056/15089`; stack `64`; food `1/0.6`; use `0.8` seconds/
`16` ticks; Balanced Diet `40` and `100` XP; cooking `200/100/600`, XP `0.1`; compacting `9:1`;
Composter `0.3f/0.5f`; fuel `4001`; block `0.5/2.5`; fire `30/60`; fast-flat
`0.9125/0.09/0.9/0.03`; Butcher `10:1`, uses `12`, XP `30`; Trader `5/76`; Kelp values above.

**Side effects:**

Active food use and advancement; cooking/crafting/knowledge; Composter level/item/effects; furnace
fuel; block placement/break/fire; Sulfur-Cube selection; merchant offer/trade; Kelp loot/worldgen;
stack persistence/synchronization and client projection.

**Gates:**

Identity/components/hunger/use; machine/Campfire/grid/capacity; knowledge; Composter level/draw;
fuel snapshot; block/tool/explosion/fire; archetype tag; profession/trade selection; Kelp
loot/feature/biome; client resources.

**State read/written:**

Reads all gates above and writes only the active-use, hunger, advancement, processing, result,
knowledge, Composter, fuel, placed/fire/loot, archetype, offer, generated-Kelp, stack, wire and
projection state listed above.

**Failure behavior:**

Full-hunger ordinary use fails; interruption commits nothing. Wrong machine/grid, full result or
missing ingredient does not consume. Failed Composter probability leaves its state unchanged.
Loose Kelp cannot ignite a furnace as fuel. Failed explosion survival suppresses block loot.
Failed fire/trade/feature gates emit or write nothing. Reload affects future evaluation only.

**Boundary cases and quirks:**

The loose item is fast food but not fuel; the compact block is fuel but not food. Their Composter
chances are `0.3` and `0.5`, not related by the nine-item compacting ratio. Fuel duration is the
source-specified `4001`, not the commonly rounded `4000`. Butcher amount two collapses to the sole
distinct level-four candidate. Ocean features and the Trader create only raw Kelp, never a
pre-cooked stack.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-REPORT-001`; `OFF-DATA-001`;
`net.minecraft.world.item.Items`;
`net.minecraft.world.food.Foods`;
`net.minecraft.world.item.component.Consumables`;
`net.minecraft.world.level.block.ComposterBlock`;
`net.minecraft.world.level.block.entity.FuelValues`;
`net.minecraft.world.level.block.FireBlock`;
`net.minecraft.world.level.levelgen.feature.KelpFeature`;
`net.minecraft.world.item.CreativeModeTabs`;
`reports/registries.json#minecraft:{block,item,recipe,recipe_serializer,loot_table,advancement,villager_trade,trade_set,worldgen}`;
`reports/blocks.json#minecraft:dried_kelp_block`;
`reports/minecraft/components/item/{dried_kelp,dried_kelp_block}.json`;
`data/minecraft/recipe/{dried_kelp,dried_kelp_block,dried_kelp_from_smelting,dried_kelp_from_smoking,dried_kelp_from_campfire_cooking}.json`;
`data/minecraft/advancement/{husbandry/balanced_diet,recipes/{food,building_blocks}/dried_kelp*}.json`;
`data/minecraft/loot_table/blocks/{kelp,kelp_plant,dried_kelp_block}.json`;
`data/minecraft/{villager_trade/butcher/4/dried_kelp_block_emerald,tags/villager_trade/butcher/level_4,trade_set/butcher/level_4}.json`;
`data/minecraft/{villager_trade/wandering_trader/emerald_kelp,tags/villager_trade/wandering_trader/common,trade_set/wandering_trader/common}.json`;
`data/minecraft/tags/{block/mineable/hoe,item/sulfur_cube_archetype/fast_flat}.json`;
`data/minecraft/sulfur_cube_archetype/fast_flat.json`;
`data/minecraft/worldgen/{configured_feature/kelp,placed_feature/{kelp_cold,kelp_warm},biome/{ocean,deep_ocean,cold_ocean,deep_cold_ocean,lukewarm_ocean,deep_lukewarm_ocean}}.json`;
`data/minecraft/structure/**/*.nbt`;
`assets/minecraft/items/{dried_kelp,dried_kelp_block}.json`;
`assets/minecraft/models/{item/dried_kelp,block/dried_kelp_block}.json`;
`assets/minecraft/textures/{item/dried_kelp,block/dried_kelp_{top,bottom,side}}.png`;
`assets/minecraft/blockstates/dried_kelp_block.json`;
`EXP-ITM-081`.

**Test vectors:**

Run `EXP-ITM-081` with default, food-only, consumable-only and patched stacks at every hunger,
count, interruption and infinite-material boundary; complete the Balanced-Diet row. Exercise all
three cooking and two compacting records, exact/malformed inputs, output capacity and both unlock
routes. Compare loose/block Composter attempts at levels `0..7` and exact threshold draws.

Ignite every furnace family with loose versus compacted stacks and assert `4001`; place/break/burn
the block across hand/Hoe/explosion/fire boundaries and select fast-flat. Generate the level-four
Butcher and Wandering common sets at all selection endpoints. Run cold/warm Kelp features in all
six biomes and scan all 1,212 templates. Persist/synchronize every stack and owner; assert IDs,
names, models, textures and both tab positions.

**Limits:**

Generic stack/use, hunger, active-use completion, furnace/Campfire, crafting, advancement,
Composter, fuel consumption, fire, block lifecycle/loot, Sulfur-Cube behavior, merchant economy,
Kelp block/growth/feature execution, packet encoding and rendering remain with `ITM-001`,
`ITM-HUNGER-001`, `ITM-FURNACE-001`, `ITM-CAMPFIRE-001`, `ITM-RECIPE-001`,
`ITM-ADVANCEMENT-001`, the Composter/fuel owners, `ENV-FIRE-001`, `BLK-BREAK-HOOK-001`,
`ITM-LOOT-001`, `ENT-KNOCKBACK-001`, the merchant owners, `WGEN-PIPELINE-001` and `CLI-001`.
