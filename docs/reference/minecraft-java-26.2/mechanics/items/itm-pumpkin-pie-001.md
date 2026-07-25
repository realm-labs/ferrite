# Items mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ITM-PUMPKIN-PIE-001` — Pumpkin Pie joins a three-egg-identity recipe, Taiga chest, Farmer sale and gift, and guaranteed composting

**Parent:** `PLY-005`, `PLY-006`, `PLY-INPUT-001`, `PLY-INTERACT-001`,
`ITM-001`, `ITM-002`, `ITM-003`, `ITM-004`, `ITM-005`, `ITM-006`,
`ITM-007`, `ITM-USE-001`, `ITM-CONTAINER-001`, `ITM-RECIPE-001`,
`ITM-CRAFT-001`, `ITM-LOOT-001`, `ITM-ADVANCEMENT-001`, `ITM-HUNGER-001`,
`ITM-ENCHANT-001`, `ITM-ANVIL-001`, `ENT-001`, `MOB-RAID-001`,
`WGEN-PIPELINE-001`, `WGEN-JIGSAW-VILLAGES-001`, `CLI-001`, `CLI-006`,
`CLI-UI-001`, `CLI-EFFECT-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked registration/components, recipe/unlock, one village chest row, Farmer
trade/gift records, code-built Composter chance, advancement and direct client resources determine
every Pumpkin-Pie-specific branch. Generic use, crafting, loot/village, merchants, gifts,
composting, stacks and client algorithms remain with the cited owners.

**Applies when:**

A `pumpkin_pie` stack is crafted, emitted by a Taiga-village chest or Farmer gift, received from a
Farmer offer, eaten, inserted into a Composter, moved, renamed, persisted, synchronized or rendered
before and after component, recipe, item-tag, loot, trade, advancement or resource reload.

**Authoritative state:**

`minecraft:pumpkin_pie` is raw item ID `1271`. It is a common nondamageable plain `Item` with
maximum stack `64`, food nutrition `8` and saturation `4.8`, and the ordinary empty `32`-tick eat
consumable with no consume-effect entries or remainder.

Its other default components are the common empty modifiers/enchantments/lore, item-break sound,
translated name, direct item-model key, repair cost, swing animation, tooltip display and use
effects. Pumpkin Pie has no direct item-tag membership. Composter bootstrap separately maps its
exact identity to chance `1.0f`.

**Transition and ordering:**

Player consumption and progression:

In-air use enters generic consumption only below full hunger or when ability permits full-hunger
eating. Interruption or live-hand/component replacement before completion commits nothing.
Successful server completion emits eat effects, awards the statistic, triggers `consume_item`
against the live pre-shrink stack, applies food, runs the empty effect list, emits `EAT` and
shrinks one unless materials are infinite.

Default Pumpkin Pie adds `8` food and `4.8` saturation subject to generic clamps, spends no
consume-effect RNG and leaves no remainder. Pumpkin Pie is one independent requirement of
telemetry-enabled `husbandry/balanced_diet`; all `40` foods award `100` experience.

Player crafting and recipe progression:

The sole recipe is shapeless and requires exactly one Pumpkin, one Sugar and one member of
`#minecraft:eggs`: Egg, Blue Egg or Brown Egg. It fits both `2×2` inventory crafting and `3×3`
Crafting Tables in any slots/order, rejects duplicate/missing/extra inputs, consumes all three
default inputs and emits one default Pumpkin Pie. Input patches do not propagate.

The no-display recipe advancement has one OR requirement: possessing Pumpkin, possessing Carved
Pumpkin or already knowing `minecraft:pumpkin_pie` grants the recipe. Sugar, any egg and a carved
pumpkin cannot satisfy the recipe itself; Carved Pumpkin is an unlock-only alternative. Recipe
reload changes future matching/output, item-tag reload changes which identities satisfy
`#minecraft:eggs`, and neither rewrites completed output.

Taiga-village chest acquisition:

Pool zero of `chests/village/village_taiga_house` rolls uniformly `3..8` times with replacement
over total weight `54`. Its unconditional Pumpkin-Pie entry has weight `1` and no count function,
so every selected entry emits one default Pie. Conditional probability is `1/54` per pool roll
under random sequence `minecraft:chests/village/village_taiga_house`.

Village start/template selection, chest placement, lazy seed assignment, table invocation, output
shuffle and insertion remain with loot and village owners. No other locked chest, entity, fishing,
Trial-Spawner or gift table emits Pumpkin Pie except the Farmer Hero gift below.

Farmer offer and Hero gift:

Farmer level two contains three predicate-free records and selects two without duplicates, so
`farmer/2/emerald_pumpkin_pie` has inclusion probability `2/3` under
`minecraft:trade_set/farmer/level_2`. It consumes one Emerald and gives four default Pumpkin Pies,
with maximum uses `12`, villager XP `5` and reputation discount `0.05`. There is no second cost,
predicate, output modifier or double-price enchantment.

An admitted adult Farmer Hero gift chooses uniformly among one default Bread, Pumpkin Pie and
Cookie. Pumpkin-Pie probability is `1/3` under
`minecraft:gameplay/hero_of_the_village/farmer_gift`. Initial eligible cooldown is `600`; later
cooldown is `600 + nextInt(6001)`, target range is five blocks, behavior lasts at most `100`
ticks and throws only after elapsed time exceeds `20`. Gift admission/navigation/throw/cleanup
remain `MOB-RAID-001`; offer economy and menu work remain merchant-owned. The optional Trade
Rebalance pack replaces neither Pumpkin-Pie record.

Composter insertion:

Player-held insertion at level `0` succeeds without RNG. Levels `1..6` consume one
`nextDouble()` and always increment because its result is strictly below chance `1.0`. Success
writes level plus one with flags `3`, emits `BLOCK_CHANGE`, and `6 -> 7` schedules maturation
after `20` ticks.

Every admitted level-`0..6` result emits level event `1500` as success, awards the
Pumpkin-Pie-used statistic and calls `consume(1, player)`, preserving infinite-material holders.
Level `7` returns success for held Pumpkin Pie without insertion, event, statistic or consumption.
Level `8` delegates to ordinary item-on-block handling.

Automation exposes one top input slot only below level `7`. It accepts one Pumpkin Pie, runs the
same deterministic level-zero or drawn guaranteed-success transition, emits event `1500`, removes
the one-slot stack and schedules maturation on `6 -> 7`. Maturation, Bone-Meal extraction and
event rendering remain with the Composter/block/client owners.

**Persistence and reload boundary:**

Pumpkin-Pie stacks persist identity, count and patches. Player active-use/hunger, recipe
knowledge, chest cursor/container, merchant offers/uses, Hero-gift behavior and Composter state
persist with their owners.

Recipe and egg-tag reload change future crafts. Loot reload changes future chest/gift evaluation.
Trade and advancement reload change future offers/listeners. The code-built Composter chance does
not reload. Completed work is not replayed. Resource reload independently controls name, model
and texture.

**Client and wire projection:**

Generic stack encoding projects raw ID `1271` plus patches. The locked English name is `Pumpkin
Pie`; it is common with no forced glint or subtype tooltip. Its direct item definition selects the
ordinary generated `item/pumpkin_pie` model and same-named texture.

Pumpkin Pie appears exactly once in Food & Drinks, ordered Cake, Pumpkin Pie, Rotten Flesh,
Spider Eye.

**Branches and aborts:**

Identity/count/components; player use; shapeless crafting/unlock/egg tag; Taiga-village chest;
Farmer offer/gift; Composter direct/automation; Balanced Diet, persistence, reload, wire, model
and tab.

**Constants and randomness:**

Raw ID `1271`; max `64`; player food `8/4.8`; eat `32`; recipe `Pumpkin + Sugar + one of 3 eggs
-> 1`; Taiga pool rolls `3..8`, weight `1/54`, count `1`; Farmer trade inclusion `2/3`,
`1→4`, uses/XP `12/5`; gift `1/3`; Composter chance `1.0`, maturation `20`.

**Side effects:**

Player food/use/progression; crafting output/unlock; chest/gift loot and cursor; merchant
result/economy; Composter level/event/stat/consumption/schedule; persistence, wire and client
projection.

**Gates:**

Player hunger/use; exact shapeless recipe/live egg tag; loot/village/gift admission; Farmer
level/set/offer; Composter level/side; progression listeners; registry/decode/client bootstrap.

**State read/written:**

Reads Pumpkin-Pie stack/components, player state, grid/recipe/tag, loot/village, merchant/gift,
Composter state/RNG and client resources. Writes only the consumption, crafting, loot, trade,
compost and projection state listed above.

**Failure behavior:**

Unadmitted use commits nothing. Invalid grids do not craft. Unselected chest/gift entries emit
alternatives. Invalid/exhausted offers commit nothing. Pumpkin Pie has no admitted Composter
chance failure at levels `0..6`; level `7` reports success without consuming it, and level `8`
falls through.

**Boundary cases and quirks:**

Carved Pumpkin unlocks the recipe but cannot replace Pumpkin in it. Blue and Brown Eggs are exact
recipe alternatives through a live tag. Composter levels `1..6` still consume one random draw
even though chance `1.0` guarantees success; level zero consumes none.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-DATA-001`; `OFF-REPORT-001`;
`net.minecraft.world.item.Items`;
`net.minecraft.world.level.block.ComposterBlock#bootStrap`;
`net.minecraft.world.level.block.ComposterBlock#useItemOn`;
`net.minecraft.world.level.block.ComposterBlock#insertItem`;
`net.minecraft.world.level.block.ComposterBlock#addItem`;
`net.minecraft.world.entity.npc.villager.AbstractVillager#addOffersFromTradeSet`;
`net.minecraft.world.entity.npc.villager.AbstractVillager#addOffersFromItemListingsWithoutDuplicates`;
`net.minecraft.world.item.trading.VillagerTrade#getOffer`;
`net.minecraft.world.item.trading.TradeSet#calculateNumberOfTrades`;
`net.minecraft.world.entity.ai.behavior.GiveGiftToHero`;
`net.minecraft.world.item.CreativeModeTabs`;
`net.minecraft.data.loot.packs.VanillaChestLoot`;
`net.minecraft.data.loot.packs.VanillaGiftLoot`;
`net.minecraft.data.recipes.packs.VanillaRecipeProvider`;
`net.minecraft.data.advancements.packs.VanillaHusbandryAdvancements`;
`reports/registries.json#minecraft:{item,recipe,loot_table,advancement,villager_trade,trade_set}`;
`reports/minecraft/components/item/pumpkin_pie.json`;
`data/minecraft/{recipe/pumpkin_pie,advancement/recipes/food/pumpkin_pie}.json`;
`data/minecraft/tags/item/eggs.json`;
`data/minecraft/loot_table/chests/village/village_taiga_house.json`;
`data/minecraft/{villager_trade/farmer/2/emerald_pumpkin_pie,tags/villager_trade/farmer/level_2,trade_set/farmer/level_2}.json`;
`data/minecraft/loot_table/gameplay/hero_of_the_village/farmer_gift.json`;
`data/minecraft/advancement/husbandry/balanced_diet.json`;
`assets/minecraft/{items,models/item,textures/item}/pumpkin_pie.*`;
`ITM-CRAFT-001`; `ITM-LOOT-001`; `ITM-ADVANCEMENT-001`; `ITM-HUNGER-001`;
`MOB-RAID-001`; `WGEN-PIPELINE-001`; `WGEN-JIGSAW-VILLAGES-001`; `CLI-UI-001`;
`CLI-EFFECT-001`; `EXP-ITM-073`.

**Test vectors:**

Exercise default/food-removed/consumable-removed and arbitrarily patched Pumpkin Pies through
interrupted/completed player use at every hunger/material boundary; verify Balanced Diet,
persistence and synchronization.

Match every slot/order and duplicate/missing/extra Pumpkin/Sugar/Egg grid in `2×2` and `3×3`;
repeat with all three default egg identities and tag removal/addition. Exercise Pumpkin,
Carved-Pumpkin and known-recipe unlock routes.

Materialize every Taiga-village pool-zero entry and cursor; generate fresh level-two Farmer sets
across candidate orders and complete offer lifecycles; force each Farmer gift choice across exact
cooldown/range boundaries.

Exercise every Composter level and automated input side while recording random draws and
transitions. Reload all domains, then verify raw ID, name, ordinary generated model/texture and
exact Food-tab neighborhood.
