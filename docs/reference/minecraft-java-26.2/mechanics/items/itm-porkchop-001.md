# Items mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ITM-PORKCHOP-001` — Raw and Cooked Porkchop join Pig and Hoglin fire-converting drops to cooking, Bastion/village loot, Butcher offers, Piglin eating and Wolf feeding

**Parent:** `PLY-005`, `PLY-006`, `PLY-INPUT-001`, `PLY-INTERACT-001`,
`ITM-001`, `ITM-002`, `ITM-003`, `ITM-004`, `ITM-005`, `ITM-006`,
`ITM-007`, `ITM-USE-001`, `ITM-CONTAINER-001`, `ITM-RECIPE-001`,
`ITM-CRAFT-001`, `ITM-FURNACE-001`, `ITM-CAMPFIRE-001`, `ITM-LOOT-001`,
`ITM-ADVANCEMENT-001`, `ITM-HUNGER-001`, `ITM-ENCHANT-001`,
`ITM-ANVIL-001`, `ENT-001`, `ENT-DEATH-001`, `ENT-ENTITY-DROPS-001`,
`MOB-AI-001`, `MOB-BREED-001`, `MOB-RAID-001`,
`WGEN-JIGSAW-BASTION-001`, `WGEN-JIGSAW-VILLAGES-001`, `CLI-001`,
`CLI-006`, `CLI-UI-001`, `CLI-EFFECT-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked registration/components and tags, Pig and Hoglin death tables, live
fire-smelting join, three recipes/unlocks, three chest rows, two Butcher offers, Butcher hero
gift, exact Piglin pickup/eat dispatch, Wolf tag closure, Balanced Diet and direct client
resources determine every Porkchop-specific branch. Generic use, death, loot, cooking machine,
structure, merchant, Villager/Piglin/Wolf AI, progression, stack and client behavior remains with
the cited owners.

**Applies when:**

A `porkchop` or `cooked_porkchop` stack is eaten, emitted by Pig/Hoglin death, village/Bastion
chest or Butcher gift, cooked, bought or sold through a Butcher, picked up by a Piglin, offered
to a Wolf, moved, renamed, persisted, synchronized or rendered before and after component, tag,
recipe, advancement, loot, trade or resource reload.

**Authoritative state:**

| Item | Raw ID | Food | Other state |
|---|---:|---|---|
| `minecraft:porkchop` | `1011` | nutrition `3`, saturation `1.8000001` | common, maximum `64`, ordinary `32`-tick eat |
| `minecraft:cooked_porkchop` | `1012` | nutrition `8`, saturation `12.8` | common, maximum `64`, ordinary `32`-tick eat |

Both are nondamageable plain `Item` instances with empty consumables, no consume-effect entries
and `can_always_eat=false`. Common remaining components are empty attribute modifiers,
enchantments and lore, item-break sound, translated name, direct item-model key, repair cost,
swing animation, tooltip display and use effects.

Both directly belong to `meat` and `piglin_food`. The live `meat` tag is included by `wolf_food`.
The two identities are the complete locked `piglin_food` tag.

**Transition and ordering:**

Player consumption:

In-air use enters generic consumption only below full hunger or when player ability permits
eating at full hunger. Block interaction remains block-first. Interruption, release or live-hand/
component replacement before completion commits no statistic, criterion, food, event or shrink.

Successful server completion emits final eat effects, awards the item-used statistic, triggers
`consume_item` against the pre-shrink stack, applies food, runs the empty consume-effect list,
emits `EAT` and shrinks one unless materials are infinite. Default Raw Porkchop adds `3` food and
`1.8000001` saturation; Cooked adds `8` and `12.8`, subject to hunger clamps. Neither spends
effect RNG or has a remainder.

Both are independent requirements of telemetry-enabled `husbandry/balanced_diet`. A completed
use advances the matching pre-shrink identity. The full advancement requires `40` foods and
awards `100` experience.

Pig and Hoglin death acquisition:

An admitted adult Pig table makes one meat pool under
`minecraft:entities/pig`: default Raw Porkchop, count uniform integer `1..3`, conditional live
smelting, then Looting count increase.

An admitted adult Hoglin table evaluates meat first under `minecraft:entities/hoglin`: default
Raw Porkchop, count `2..4`, conditional live smelting, then Looting. Its second pool emits
Leather with base `0..1` plus an independent Looting bonus. Leather work cannot alter an already
emitted Porkchop but advances the same named sequence after it.

For both meat pools, smelting runs when the victim is on fire or the direct attacker's main hand
has an enchantment in `smelts_loot`. It resolves the live Raw-Porkchop smelting recipe during
loot evaluation and converts the full base stack to Cooked Porkchop before Looting. A missing
recipe leaves it Raw; a replacement recipe controls future converted output. The OR condition
cannot double-smelt.

With a living attacking entity and Looting level `L>0`, each bonus spends a float `U` and adds
`round(L*U)`; absent/nonliving attacker or level zero skips the draw. Locked final meat count is
therefore `B + round(L*U)` for Pig `B in 1..3` or Hoglin `B in 2..4`. Baby/death-rule admission,
Leather output, XP and generic table work remain with entity/loot owners.

Cooking and recipe progression:

Three exact recipes consume one Raw Porkchop and emit one default Cooked Porkchop:

| Recipe | Domain | Time | Recipe XP |
|---|---|---:|---:|
| `cooked_porkchop` | Furnace | `200` ticks | `0.35` |
| `cooked_porkchop_from_smoking` | Smoker | `100` ticks | `0.35` |
| `cooked_porkchop_from_campfire_cooking` | Campfire | `600` ticks | `0.35` |

Each no-display advancement uses one OR requirement: Raw-Porkchop possession or the matching
recipe unlock grants that recipe. Cooked possession does not satisfy it. Furnace/Smoker
accumulate `0.35` per result and extraction owns fractional XP plus criteria. Campfire
re-resolves at completion and awards neither recipe XP nor unlock. Input patches do not copy.

Chest acquisition:

`chests/village/village_butcher` makes uniform `1..5` rolls over total weight `28`. Raw
Porkchop has weight `6`, probability `3/14` per roll and count uniform `1..3`.

`chests/bastion_hoglin_stable` pool `1` makes uniform `3..4` rolls over `14` equal-weight rows.
Raw and Cooked Porkchop each have per-roll probability `1/14` and count `2..5`. They are
independent alternatives in the same pool and repeated selections are allowed.

`chests/bastion_other` pool `2` makes uniform `3..4` rolls over total weight `13`. Cooked
Porkchop has weight `1`, probability `1/13` per roll and count exactly one. Raw Porkchop is absent.
Earlier/later pools still advance each named table in order.

The sequences are `minecraft:chests/village/village_butcher`,
`minecraft:chests/bastion_hoglin_stable` and `minecraft:chests/bastion_other`. Placement,
container materialization/seed, rolls, insertion and template joins remain with loot and
structure owners. Trade Rebalance replaces none of these records.

Butcher offer joins:

Level one contains Raw-Chicken, Raw-Porkchop and Raw-Rabbit purchases plus Rabbit-Stew sale; its
set selects two distinct predicate-free records. Raw-Porkchop purchase inclusion is therefore
`1/2` under `minecraft:trade_set/butcher/level_1`.

`butcher/1/porkchop_emerald` wants seven matching Raw Porkchop and gives one default Emerald,
with maximum uses `16`, Villager XP `2`, reputation discount `0.05`, no second cost, predicate or
modifier.

Level two contains Coal purchase, Cooked-Porkchop sale and Cooked-Chicken sale; its set selects
two distinct predicate-free records. Cooked-Porkchop sale inclusion is `2/3` under
`minecraft:trade_set/butcher/level_2`.

`butcher/2/emerald_cooked_porkchop` wants one matching Emerald and gives five default Cooked
Porkchop, with maximum uses `16`, XP `5`, discount `0.05`, no second cost, predicate or modifier.
Empty cost-component predicates accept ordinary patches. Generic economy/restock remains with
merchant owners; Trade Rebalance replaces neither tag/record.

Butcher hero gift:

An adult Butcher's admitted Hero behavior evaluates
`gameplay/hero_of_the_village/butcher_gift`. Its one roll is uniform among Cooked Rabbit, Cooked
Chicken, Cooked Porkchop, Steak and Cooked Mutton, so Cooked Porkchop has probability `1/5` and
count one.

Initial eligible cooldown is `600`; later cooldown is `600 + nextInt(6001)`. The Hero must be
within five blocks; behavior lasts at most `100` ticks and throws only after elapsed time exceeds
`20`. Profession/age, memories, movement/look, table, throw and cleanup remain with mob owners.

Piglin ground-item eating:

Both identities match the complete `piglin_food` tag. `wantsToPickup` first applies baby-ignore,
repellent and attack/admiration gates. For food it additionally requires no `ATE_RECENTLY`
memory and `canAddToInventory(stack)`, even though the admitted food will not be stored.

On pickup the Piglin stops walking, takes exactly one from the item entity and updates/discards
the remaining entity stack. Because the split stack is food and the memory is absent, it sets
`ATE_RECENTLY=true` with expiry `200` ticks and returns. The one item is consumed: it is neither
equipped nor inserted nor put in the offhand.

This branch does not execute the stack's food or consumable components, change health/hunger,
award player statistics, emit a barter response or run item-use effects. Tag reload can change
future admission. Generic wanted-item acquisition, inventory capacity, brain memory and entity
pickup events remain with `MOB-AI-001`.

Wolf feeding:

The live `meat -> wolf_food` closure admits both identities. For a tamed injured Wolf, default
Raw/Cooked Porkchop heal twice nutrition: `6/16`; missing food falls back to `2`. One item is
consumed unless materials are infinite, and an eat sound plays. Other admitted states use
generic baby ten-percent growth or adult `600`-tick love rules; ineligible states do not consume.
Taming remains exact Bone. Full side/tame/owner/health/age/love ordering remains with
`MOB-BREED-001`.

**Persistence and reload boundary:**

Stacks persist identity, count and patches. They store no use/hunger, death/attacker/Looting,
live recipe, machine, table/structure, offer, Villager/Piglin brain, Wolf or progression state.
`ATE_RECENTLY` is Piglin memory, not item state.

Recipe reload changes future cooking and death smelting. Loot reload changes future death/chest/
gift evaluation. Tag reload changes Piglin and Wolf admission. Trade reload changes future
offers; advancement reload changes listeners. Completed work is not replayed. Resource reload
independently controls language/models/textures.

**Client and wire projection:**

Generic stack encoding projects raw ID `1011` or `1012` plus component patches. Locked English
names are `Raw Porkchop` and `Cooked Porkchop`; both are common with no subtype tooltip or forced
glint. Direct definitions select same-named generated models/textures.

Both appear exactly once and only in Food & Drinks, ordered Raw Beef, Steak, Raw Porkchop,
Cooked Porkchop, Raw Mutton, Cooked Mutton.

**Branches and aborts:**

Identity/count/food/tags; hunger/use; Pig/Hoglin age/death/fire/attacker/smelts-loot/live recipe/
Looting/pool order; cooking; three chests; two Butcher sets/offers and hero gift; Piglin pickup/
memory/inventory; Wolf state; reload/persistence/client/wire.

**Constants and randomness:**

Raw IDs `1011/1012`; food `3/1.8000001`, `8/12.8`; death Pig `1..3`, Hoglin `2..4`, plus
`round(L*U)`; cooking `200/100/600`, XP `0.35`; village `3/14` over `1..5`; stable each `1/14`
over `3..4`, count `2..5`; other Cooked `1/13` over `3..4`; trade inclusion `1/2`, `2/3`,
transactions `7→1`, `1→5`; hero `1/5`; Piglin memory `200`; Wolf heal `6/16`.

**Side effects:**

Food/use; raw/cooked and Leather death outputs; cooking; chest cursors; offers/economy and hero
gift; ground-item split/removal and Piglin memory; Wolf health/growth/love; persistence, wire and
client projection.

**Gates:**

Food/hunger/use; adult/drop-enabled Pig/Hoglin; fire/smelts-loot/live recipe/Looting; machine;
structure/table; Butcher level/Hero; Piglin tag/brain/pickup/inventory; Wolf tag/state;
registry/decode/resources.

**State read/written:**

Reads stack/components/tags, player food/use/progression, death/attacker/enchantment/recipe,
machine, tables/structures, merchant/Villager, Piglin item/memory/inventory, Wolf, persistence and
resources. Writes only states listed above.

**Failure behavior:**

Unadmitted use commits nothing. Baby/drop-disabled deaths emit no ordinary table; missing smelting
recipe leaves meat raw. Missing machine recipe yields no normal result. Alternate chest/gift or
offer candidate emits none. Piglin admission failure leaves the item entity untouched; recently
fed or inventory-blocked Piglins reject food pickup. Ineligible Wolf state does not consume.

**Boundary cases and quirks:**

Hoglin meat precedes Leather. Death smelting converts the full base count before Looting.
Bastion Stable can independently select both Raw and Cooked rows repeatedly. Piglin eating
requires inventory capacity but discards the split food into a memory-only transaction and runs
no food component. Both are distinct Balanced-Diet requirements.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-DATA-001`; `OFF-REPORT-001`;
`net.minecraft.world.item.Items`; `net.minecraft.world.item.CreativeModeTabs`;
`net.minecraft.world.level.storage.loot.functions.SmeltItemFunction#run`;
`net.minecraft.world.entity.monster.piglin.PiglinAi#pickUpItem`;
`net.minecraft.world.entity.monster.piglin.PiglinAi#wantsToPickup`;
`net.minecraft.world.entity.npc.villager.AbstractVillager#addOffersFromTradeSet`;
`net.minecraft.world.entity.ai.behavior.GiveGiftToHero`;
`net.minecraft.world.entity.animal.wolf.Wolf#mobInteract`;
`net.minecraft.data.loot.packs.VanillaEntityLoot`;
`net.minecraft.data.loot.packs.VanillaChestLoot`;
`net.minecraft.data.loot.packs.VanillaGiftLoot`;
`net.minecraft.data.recipes.packs.VanillaRecipeProvider`;
`net.minecraft.data.advancements.packs.VanillaHusbandryAdvancements`;
`reports/registries.json#minecraft:{item,entity_type,recipe,loot_table,advancement,villager_trade,trade_set}`;
`reports/minecraft/components/item/{porkchop,cooked_porkchop}.json`;
`data/minecraft/tags/item/{meat,piglin_food,wolf_food}.json`;
`data/minecraft/loot_table/{entities/{pig,hoglin},chests/{village/village_butcher,bastion_hoglin_stable,bastion_other},gameplay/hero_of_the_village/butcher_gift}.json`;
`data/minecraft/recipe/cooked_porkchop*.json`;
`data/minecraft/advancement/{recipes/food/cooked_porkchop*,husbandry/balanced_diet}.json`;
`data/minecraft/{villager_trade/butcher/{1/porkchop_emerald,2/emerald_cooked_porkchop},tags/villager_trade/butcher/level_{1,2},trade_set/butcher/level_{1,2}}.json`;
`assets/minecraft/{items,models/item,textures/item}/{porkchop,cooked_porkchop}.*`;
`ITM-FURNACE-001`; `ITM-CAMPFIRE-001`; `ITM-LOOT-001`;
`ITM-ADVANCEMENT-001`; `ITM-HUNGER-001`; `ENT-DEATH-001`;
`ENT-ENTITY-DROPS-001`; `MOB-AI-001`; `MOB-BREED-001`; `MOB-RAID-001`;
`WGEN-JIGSAW-BASTION-001`; `WGEN-JIGSAW-VILLAGES-001`;
`CLI-UI-001`; `CLI-EFFECT-001`; `EXP-ITM-066`.

**Test vectors:**

Exercise default/removed/patched stacks through use. Kill adult/baby Pig/Hoglin across death,
fire, attacker, `smelts_loot`, live recipe and Looting while tracing Hoglin meat-before-Leather.
Cook through all machines and unlock/XP boundaries.

Generate all three chest tables, both Butcher candidate sets/offers and hero gift. Drop both
identities to Piglins across age/ignore/repellent/attack/recent/inventory/item-count state; feed
every Wolf state. Reload all domains, persist/synchronize and verify IDs, names, models, textures
and exact tab neighbors.
