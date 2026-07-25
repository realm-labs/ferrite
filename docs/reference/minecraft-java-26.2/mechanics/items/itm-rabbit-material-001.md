# Items mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ITM-RABBIT-MATERIAL-001` — Rabbit meat and Hide share an ordered death table before splitting into cooking, stew/Leather crafting, gifts, trades and Wolf feeding

**Parent:** `PLY-005`, `PLY-006`, `PLY-INPUT-001`, `PLY-INTERACT-001`,
`ITM-001`, `ITM-002`, `ITM-003`, `ITM-004`, `ITM-005`, `ITM-006`,
`ITM-007`, `ITM-USE-001`, `ITM-CONTAINER-001`, `ITM-RECIPE-001`,
`ITM-CRAFT-001`, `ITM-FURNACE-001`, `ITM-CAMPFIRE-001`, `ITM-STEW-001`,
`ITM-LOOT-001`, `ITM-ADVANCEMENT-001`, `ITM-HUNGER-001`,
`ITM-ENCHANT-001`, `ITM-ANVIL-001`, `ITM-RABBIT-FOOT-001`, `ENT-001`,
`ENT-DEATH-001`, `ENT-ENTITY-DROPS-001`, `MOB-AI-001`, `MOB-BREED-001`,
`MOB-RAID-001`, `CLI-001`, `CLI-006`, `CLI-UI-001`, `CLI-EFFECT-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked registrations/components and tag closure, Rabbit's ordered
Hide/meat/Foot table, live fire-smelting join, six recipes and unlocks, Cat/Butcher gifts, two
merchant records, Wolf feeding, Balanced Diet and direct client resources determine every
Rabbit/Hide-specific branch. Generic use, death, loot, machine, recipe, Cat/Villager AI,
merchant, Wolf, progression, stack and client behavior remains with the cited owners.

**Applies when:**

A `rabbit`, `cooked_rabbit` or `rabbit_hide` stack is emitted by Rabbit death or gift loot,
eaten, cooked, consumed by Rabbit-Stew or Leather crafting, bought by a Butcher/Leatherworker,
offered to a Wolf, moved, renamed, persisted, synchronized or rendered before and after
component, tag, recipe, advancement, loot, trade or resource reload.

**Authoritative state:**

| Item | Raw ID | Food | Other state |
|---|---:|---|---|
| `minecraft:rabbit` | `1279` | nutrition `3`, saturation `1.8000001` | common, maximum `64`, ordinary `32`-tick eat |
| `minecraft:cooked_rabbit` | `1280` | nutrition `5`, saturation `6.0` | common, maximum `64`, ordinary `32`-tick eat |
| `minecraft:rabbit_hide` | `1283` | none | common, maximum `64`, inert plain item |

All three are nondamageable plain `Item` instances with common empty attribute modifiers,
enchantments and lore, item-break sound, translated name, direct item-model key, repair cost,
swing animation, tooltip display and use effects. The two meats add empty consumables with no
consume-effect entries and `can_always_eat=false`; Hide has neither food nor consumable.

Both meats directly belong to `meat`; `wolf_food` includes the live `meat` tag. Hide has no direct
item-tag membership.

**Transition and ordering:**

Player consumption and progression:

In-air meat use enters generic consumption only below full hunger or when ability permits full-
hunger eating. Interruption or live-hand/component replacement before completion commits nothing.
Successful server completion emits eat effects, awards the statistic, triggers `consume_item`,
applies food, runs the empty effect list, emits `EAT` and shrinks one unless materials are
infinite.

Default Raw Rabbit adds `3` food and `1.8000001` saturation; Cooked Rabbit adds `5` and `6.0`,
subject to clamps. Neither spends effect RNG or has a remainder. They are two independent
requirements of telemetry-enabled `husbandry/balanced_diet`; the full `40`-food advancement
awards `100` experience. Rabbit Hide is not a food requirement.

Rabbit death acquisition:

An admitted adult Rabbit table uses sequence `minecraft:entities/rabbit` and evaluates:

1. Hide: default Rabbit Hide, base count uniform integer `0..1`, then Looting increase;
2. meat: exactly one Raw Rabbit, conditional live smelting, then independent Looting increase;
3. Rabbit Foot: a killed-by-player and enchanted-chance pool owned in exact detail by
   `ITM-RABBIT-FOOT-001`.

With a living attacking entity and Looting level `L>0`, each count increase spends its own float
`U` and adds `round(L*U)`; absent/nonliving attacker or level zero skips it. Hide total is
`H + round(L*U_h)` for `H in 0..1`, so Looting can revive base zero. Meat total is
`1 + round(L*U_m)`.

Meat smelting runs when the Rabbit is on fire or the direct attacker's main hand has an
enchantment in `smelts_loot`. It resolves the live Raw-Rabbit smelting recipe, converts the base
stack to Cooked Rabbit before its Looting bonus, and cannot double-smelt when both OR terms pass.
A missing recipe leaves Raw Rabbit; replacement output controls future drops. Baby/death-rule
admission and the Foot branch remain with entity/loot/Foot owners.

Cooking and recipe progression:

Three exact recipes consume one Raw Rabbit and emit one default Cooked Rabbit:

| Recipe | Domain | Time | Recipe XP |
|---|---|---:|---:|
| `cooked_rabbit` | Furnace | `200` ticks | `0.35` |
| `cooked_rabbit_from_smoking` | Smoker | `100` ticks | `0.35` |
| `cooked_rabbit_from_campfire_cooking` | Campfire | `600` ticks | `0.35` |

Each corresponding no-display advancement has one OR requirement: Raw-Rabbit possession or
matching recipe unlock grants that recipe. Furnace/Smoker accumulate `0.35` and extraction owns
fractional XP/criteria; Campfire re-resolves and awards neither recipe XP nor unlock. Input
patches do not propagate.

Two shapeless group-`rabbit_stew` recipes consume one each of Baked Potato, Cooked Rabbit, Bowl,
Carrot and respectively Brown or Red Mushroom and emit one default Rabbit Stew. Exactly those
five occupied ingredients are admitted; the Bowl is incorporated into the result rather than
returned. Cooked-Rabbit possession independently unlocks each variant through its OR requirement.
Rabbit Stew's food/container/Wolf/trade behavior remains with `ITM-STEW-001`.

Rabbit-Hide-to-Leather crafting:

Shaped `leather` has pattern `"##","##"` with Rabbit Hide as `#`. Exactly a filled `2×2`
rectangle in any admitted crafting offset consumes four Hide and emits one default Leather. It
copies no component patches and has no remainder. Rabbit-Hide possession or recipe knowledge
unlocks it through one OR requirement. Leather's later consumers remain independently owned.

Cat morning-gift Hide:

After the qualified owner-sleep path, a Cat tests a level RNG float against live
`cat_waking_up_gift_chance`; normal wake resolves `0.7`, then teleport offsets are attempted and
the gift table is evaluated even if teleport failed.

`gameplay/cat_morning_gift` has six weight-`10` rows including Rabbit Hide and one weight-`2`
Phantom-Membrane row. Conditional Hide probability is `10/62 = 5/31`, count one. With live
chance `g`, qualified-stop probability is `5g/31`; normal `g=0.7` yields `7/62`. Chance,
teleport and named table sequence are distinct RNG sources. Cat scheduling/owner state remains
with mob owners.

Butcher hero-gift Cooked Rabbit:

An admitted adult Butcher Hero behavior evaluates one uniform row among Cooked Rabbit, Cooked
Chicken, Cooked Porkchop, Steak and Cooked Mutton. Cooked Rabbit has probability `1/5`, count one
under `minecraft:gameplay/hero_of_the_village/butcher_gift`.

Initial eligible cooldown is `600`; later is `600 + nextInt(6001)`. Target must be within five
blocks, behavior lasts at most `100` ticks and throws only after elapsed time exceeds `20`.
Profession/age, memory, navigation/look, context, throw and cleanup remain with mob owners.

Merchant sinks:

Butcher level one contains Raw-Chicken, Raw-Porkchop and Raw-Rabbit purchases plus Rabbit-Stew
sale and selects two distinct predicate-free records. Raw-Rabbit purchase inclusion is `1/2`
under `minecraft:trade_set/butcher/level_1`. `butcher/1/rabbit_emerald` wants four matching Raw
Rabbit and gives one Emerald, maximum uses `16`, XP `2`, reputation discount `0.05`.

Leatherworker level three contains Rabbit-Hide purchase and Dyed-Leather-Chestplate sale and asks
for both, making the Hide offer guaranteed; only order varies under
`minecraft:trade_set/leatherworker/level_3`. `leatherworker/3/rabbit_hide_emerald` wants nine
matching Hide and gives one Emerald, maximum uses `12`, XP `20`, discount `0.05`.

Both have no second cost, predicate or modifier; empty component predicates accept ordinary
patches. Trade Rebalance replaces neither record/tag. Economy/restock remains with merchant
owners.

Wolf feeding:

The live `meat -> wolf_food` closure admits Raw/Cooked Rabbit. For a tamed injured Wolf, default
healing is twice nutrition: `6/10`; missing food falls back to `2`. One is consumed unless
materials are infinite. Other admitted states use generic baby ten-percent growth or adult
`600`-tick love; ineligible states do not consume. Hide never enters this branch. Taming remains
exact Bone.

**Persistence and reload boundary:**

Stacks persist identity, count and patches. They store no use/hunger, death/attacker/Looting,
recipe/machine/craft, Cat/Villager, merchant, Wolf or advancement state.

Recipe reload changes future cooking, death smelting, stew and Leather crafting. Loot reload
changes Rabbit/Cat/Butcher gift evaluation. Tag reload changes Wolf admission. Trade reload
changes future offers; advancement reload changes listeners. Completed work is not replayed.
Resource reload independently controls names/models/textures.

**Client and wire projection:**

Generic stack encoding projects raw IDs `1279/1280/1283` plus patches. Locked English names are
`Raw Rabbit`, `Cooked Rabbit` and `Rabbit Hide`; all are common with no forced glint or subtype
tooltip. Direct definitions select same-named generated models/textures.

Raw/Cooked Rabbit appear exactly once in Food & Drinks, ordered Raw Chicken, Cooked Chicken, Raw
Rabbit, Cooked Rabbit, Raw Cod, Cooked Cod. Rabbit Hide appears exactly once in Ingredients,
ordered Leather, Rabbit Hide, Honeycomb.

**Branches and aborts:**

Identity/count/food/tags; use; Rabbit age/death/fire/attacker/smelts-loot/live recipe/Looting and
three-pool order; cooking/stew/Leather grids and unlocks; Cat/Butcher gifts; two merchant sets;
Wolf; reload/persistence/client/wire.

**Constants and randomness:**

Raw IDs `1279/1280/1283`; max `64`; food `3/1.8000001`, `5/6.0`; Hide `0..1` plus
`round(LU)`, meat `1+round(LU)`; cooking `200/100/600`, XP `0.35`; stew five ingredients;
Leather four Hide; Cat `5g/31`, normal `7/62`; hero `1/5`; Butcher inclusion `1/2`, `4→1`,
uses/XP `16/2`; Leatherworker guaranteed, `9→1`, `12/20`; Wolf heal `6/10`.

**Side effects:**

Food/use; Hide/meat/Foot death cursor; cooking/crafting/unlocks/XP; Cat/Butcher gift; merchant
offers/economy; Wolf health/growth/love; persistence, wire and client projection.

**Gates:**

Food/hunger/use; adult/drop-enabled Rabbit; fire/smelts-loot/live recipe/Looting; machine/grid;
Cat sleep/chance; Butcher Hero; merchant profession/level; Wolf tag/state; registry/decode/client.

**State read/written:**

Reads stacks/components/tags, player use/progression, Rabbit death/attacker/enchantment/recipe,
machine/grid, Cat/Villager/merchant, Wolf, persistence/resources. Writes only states above.

**Failure behavior:**

Unadmitted use commits nothing. Baby/drop-disabled Rabbit emits no ordinary table; base-zero Hide
can remain empty or be revived by Looting. Missing smelting recipe leaves meat raw. Invalid
machine/grid gives no result. Alternate gifts/offers emit none. Ineligible Wolf does not consume;
Hide is never Wolf food.

**Boundary cases and quirks:**

Hide precedes meat, and Foot follows both. Hide zero can be revived by Looting. Death smelting
precedes the meat bonus. Stew consumes its Bowl into the filled-container result. Cat gift emits
Hide, Butcher gift emits Cooked meat, Butcher buys Raw meat, and Leatherworker buys Hide.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-DATA-001`; `OFF-REPORT-001`;
`net.minecraft.world.item.Items`; `net.minecraft.world.item.CreativeModeTabs`;
`net.minecraft.world.level.storage.loot.functions.SmeltItemFunction#run`;
`net.minecraft.world.entity.animal.feline.Cat$CatRelaxOnOwnerGoal#giveMorningGift`;
`net.minecraft.world.entity.npc.villager.AbstractVillager#addOffersFromTradeSet`;
`net.minecraft.world.entity.ai.behavior.GiveGiftToHero`;
`net.minecraft.world.entity.animal.wolf.Wolf#mobInteract`;
`net.minecraft.data.loot.packs.VanillaEntityLoot`;
`net.minecraft.data.loot.packs.VanillaGiftLoot`;
`net.minecraft.data.recipes.packs.VanillaRecipeProvider`;
`net.minecraft.data.advancements.packs.VanillaHusbandryAdvancements`;
`reports/registries.json#minecraft:{item,entity_type,recipe,loot_table,advancement,villager_trade,trade_set}`;
`reports/minecraft/components/item/{rabbit,cooked_rabbit,rabbit_hide}.json`;
`data/minecraft/tags/item/{meat,wolf_food}.json`;
`data/minecraft/loot_table/{entities/rabbit,gameplay/{cat_morning_gift,hero_of_the_village/butcher_gift}}.json`;
`data/minecraft/recipe/{cooked_rabbit*,rabbit_stew_*,leather}.json`;
`data/minecraft/advancement/{recipes/{food/{cooked_rabbit*,rabbit_stew_*},misc/leather},husbandry/balanced_diet}.json`;
`data/minecraft/{villager_trade/{butcher/1/rabbit_emerald,leatherworker/3/rabbit_hide_emerald},tags/villager_trade/{butcher/level_1,leatherworker/level_3},trade_set/{butcher/level_1,leatherworker/level_3}}.json`;
`assets/minecraft/{items,models/item,textures/item}/{rabbit,cooked_rabbit,rabbit_hide}.*`;
`ITM-FURNACE-001`; `ITM-CAMPFIRE-001`; `ITM-STEW-001`; `ITM-LOOT-001`;
`ITM-ADVANCEMENT-001`; `ITM-HUNGER-001`; `ITM-RABBIT-FOOT-001`;
`ENT-DEATH-001`; `ENT-ENTITY-DROPS-001`; `MOB-AI-001`; `MOB-BREED-001`;
`MOB-RAID-001`; `CLI-UI-001`; `CLI-EFFECT-001`; `EXP-ITM-067`.

**Test vectors:**

Exercise default/removed/patched stacks through use. Kill Rabbits across age/death/fire/attacker/
smelts-loot/live recipe/Looting while tracing Hide/meat/Foot order. Cook in all domains; craft
both stew variants and every Leather-grid offset through unlock/remainder boundaries.

Generate Cat/Butcher gifts and both merchant sets/offers. Feed meats and attempt Hide across every
Wolf state. Reload all domains, persist/synchronize and verify IDs, names, models, textures and
both exact tab neighborhoods.
