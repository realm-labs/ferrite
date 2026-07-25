# Items mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ITM-BEEF-001` — Raw Beef and Steak join bovine fire-converting drops and cooking to Wolf food, Butcher trade and hero gifts

**Parent:** `PLY-005`, `PLY-006`, `PLY-INPUT-001`, `PLY-INTERACT-001`,
`ITM-001`, `ITM-002`, `ITM-003`, `ITM-004`, `ITM-005`, `ITM-006`,
`ITM-007`, `ITM-USE-001`, `ITM-CONTAINER-001`, `ITM-RECIPE-001`,
`ITM-CRAFT-001`, `ITM-FURNACE-001`, `ITM-CAMPFIRE-001`, `ITM-LOOT-001`,
`ITM-ADVANCEMENT-001`, `ITM-HUNGER-001`, `ITM-ENCHANT-001`,
`ITM-ANVIL-001`, `ENT-001`, `ENT-DEATH-001`, `ENT-ENTITY-DROPS-001`,
`ENT-EFFECT-001`, `MOB-AI-001`, `MOB-BREED-001`, `MOB-RAID-001`,
`BLK-TRIAL-SPAWNER-001`, `WGEN-JIGSAW-TRIAL-CHAMBERS-001`,
`WGEN-JIGSAW-VILLAGES-001`, `CLI-001`, `CLI-006`, `CLI-UI-001`,
`CLI-EFFECT-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked registration/components and tag closure, every exact Beef/Steak class
and data reference, Cow and Mooshroom tables with live fire-smelting, three cooking recipes and
unlocks, village chest, ominous Trial, Butcher trade and hero-gift joins, Wolf dispatch, Balanced
Diet and direct client resources determine every identity-specific branch. Generic active use,
death, loot, cooking-machine, structure, spawner, merchant, Villager AI, Wolf, progression, stack
and client behavior remains with the cited owners.

**Applies when:**

A `beef` or `cooked_beef` stack is emitted from bovine death, village loot, an ominous Trial
Spawner or a Butcher gift; Beef is cooked or sold to a Butcher; either item is eaten or offered to
a Wolf; or either stack is moved, renamed, persisted, synchronized, rendered or observed before
and after component, tag, recipe, advancement, loot, trade or resource reload.

**Authoritative state:**

| Item | Raw ID | Food | Other operational state |
|---|---:|---|---|
| `minecraft:beef` | `1139` | nutrition `3`, saturation `1.8000001` | common, maximum `64`, ordinary `32`-tick eat |
| `minecraft:cooked_beef` | `1140` | nutrition `8`, saturation `12.8` | common, maximum `64`, ordinary `32`-tick eat |

Both are nondamageable plain `Item` instances. Each has the default empty consumable with no
consume-effect entries and omitted/default `can_always_eat=false`. Their remaining components are
the common empty modifiers/enchantments/lore, item-break sound, translated name, direct item-model
key, repair cost, swing animation, tooltip display and use effects. Neither has cooldown, use
remainder, tool, equipment, repairable, fuel, compost or identity-specific glint behavior.

Both directly belong only to `#minecraft:meat`. That tag has `11` direct members: raw/cooked Beef,
raw/cooked Chicken, cooked Mutton, cooked Porkchop, cooked Rabbit, raw Mutton, raw Porkchop, raw
Rabbit and Rotten Flesh. `#minecraft:wolf_food` includes the whole `meat` tag plus seven fish
identities and Rabbit Stew, so both scoped items reach Wolf behavior recursively rather than by
direct `wolf_food` membership.

**Transition and ordering:**

Player consumption:

In-air player use enters the consumable path only when the food listener admits it. Ordinary
survival at food level `20` gets `FAIL`; lower hunger admits use, and an ability that permits
full-hunger eating independently admits it. Block clicks retain block-first handling before an
unconsumed result can reach the food fallback.

Removing only food leaves the empty consumable and admits use even at full hunger, but completion
applies no food. Removing only consumable makes plain in-air use pass. Patched food supplies live
nutrition, saturation and always-edible state while the empty consumable retains no effects.

Interruption, release or live-hand/component replacement before completion commits no statistic,
criterion, food change, event or shrink. Successful server completion emits final eat effects,
awards the item-used statistic, triggers `consume_item` against the live pre-shrink stack, applies
food, emits `EAT`, and consumes one unless the player has infinite materials. Raw Beef adds
`3/1.8000001`; Steak adds `8/12.8`, each clamped by the generic food owner. Neither consumes
effect RNG or applies an effect/remainder.

Raw Beef and Steak are separate requirements among the `40` singleton AND rows of
`husbandry/balanced_diet`. Consuming one advances only its own pre-shrink criterion before food
application. The challenge grants `100` experience only after all 40 rows and retains Apple as
its icon.

Cow and Mooshroom death acquisition:

Adult Cow and Mooshroom death tables each have two ordered one-roll pools under their own random
sequences `minecraft:entities/cow` and `minecraft:entities/mooshroom`. A baby or disabled
`mob_drops` fails the generic `shouldDropLoot` gate before either table executes.

The first pool emits Leather: base count is uniform `0..2`, then a living attacker with Looting
level `L > 0` spends a float and adds `round(L * U)` for uniform `U` in `[0,1)`. That pool cannot
emit either scoped item but consumes the first applicable count work on the same named cursor.

The second pool creates Beef and replaces its count with uniform integer `1..3`. It then tests an
OR condition:

1. the dying Cow/Mooshroom is on fire; or
2. the direct attacker's main-hand stack has an enchantment in `#minecraft:smelts_loot`.

If either term succeeds, `furnace_smelt` looks up a live smelting recipe for the complete Beef
stack. Baseline `cooked_beef` assembles one Steak per input, so the entire current count becomes a
default Steak stack. If no matching recipe or empty result exists, the function warns and retains
the Beef unchanged; recipe replacement can instead produce its live assembled output. Successful
assembly copies no Beef component patches and caps the multiplied result at its maximum stack size.
The condition and smelt operation consume no RNG.

After that conditional conversion, enchanted count increase reads the living attacker's Looting
level. At `L > 0` it spends a fresh float `V` and adds `round(L * V)` to whichever raw or cooked
stack now exists; no living attacker or level zero spends no bonus draw. No explicit final count
limit is configured. Player-kill attribution is not required for either pool.

Thus an eligible baseline adult death emits Beef when neither fire term holds and Steak when one
does, with count `uniform(1..3) + round(L*V)`. Both tables perform the independent Leather pool
first. Death admission, context construction, entity-drop placement, Looting lookup, data-function
dispatch and complete cursor ownership remain `ENT-DEATH-001`, `ENT-ENTITY-DROPS-001`,
`ITM-LOOT-001` and `ITM-ENCHANT-001`.

Cooking acquisition and recipe progression:

No bundled recipe creates Raw Beef. Three exact-Beef recipes create one default Steak:

| Recipe | Domain | Cooking time | Recipe XP |
|---|---|---:|---:|
| `cooked_beef` | smelting | omitted/default `200` ticks | `0.35` |
| `cooked_beef_from_smoking` | smoking | omitted/default `100` ticks | `0.35` |
| `cooked_beef_from_campfire_cooking` | campfire cooking | explicit `600` ticks | data field `0.35` |

Each category-`food` recipe consumes one exact Beef, emits one Steak and copies no input component
patch. Its no-display `recipes/root` child has one OR requirement containing Beef possession and
that recipe's own unlock; either criterion grants only that recipe. Obtaining Beef can therefore
grant all three independently. Steak possession grants none.

Furnace/Smoker completion records the recipe. Player extraction invokes crafted/unlock hooks and
awards each accumulated result `floor(count * 0.35)` experience plus one with probability equal to
the fractional part. Hopper extraction performs neither award and leaves recipe counts
accumulated. Fuel, tick, output, sided and extraction behavior remains `ITM-FURNACE-001`.

A lit Campfire moves one component-bearing Beef into the first empty one of four slots and stores
total `600`. At deadline it re-resolves and normally drops one default Steak. Recipe removal falls
back to the retained Beef, while replacement can change output. Campfire completion ignores
declared `0.35`: it awards neither recipe XP nor unlock. Independent slot progression,
extinguishing, retry, break-drop and output motion remain `ITM-CAMPFIRE-001`.

Village Butcher chest acquisition:

`chests/village/village_butcher` makes uniform `1..5` replacement rolls over total weight `28`.
Beef has weight `6`, hence probability `3/14` per roll and uniform count `1..3`. Alternatives are
Emerald `1`, Porkchop `6`, Wheat `6`, Mutton `6` and Coal `3`. Repeated Beef selections are
permitted and remain separate outputs until container insertion. The table uses named sequence
`minecraft:chests/village/village_butcher`.

Template placement, chest marker/materialization, all alternative/repeated rolls and insertion
remain `WGEN-JIGSAW-VILLAGES-001` and `ITM-LOOT-001`. This table emits no Steak.

Ominous Trial-Spawner Steak acquisition:

All `14` ominous Trial-Chamber configurations fix one ejection table for an encounter: ominous key
weight `3`, consumables weight `7`. Every registered UUID then evaluates that same table once when
its reward is due, so player outcomes are correlated by the `7/10` consumables choice.

`spawners/ominous/trial_chamber/consumables` makes one roll over total weight `10`: Cooked Beef
`3`, Baked Potato `3`, Golden Carrot `2`, Regeneration Potion `1` and Strength Potion `1`.
Conditional Steak probability is `3/10`, count is uniform `1..2`, and its sequence is
`minecraft:spawners/ominous/trial_chamber/consumables`. Marginally, each registered-player
evaluation emits Steak with probability `7/10 * 3/10 = 21/100`, but these events are not
independent across the cohort.

Normal Trial consumables contain no Beef/Steak. Encounter admission, fixed-table persistence,
registered-player cohort, one evaluation each, ejection and reset remain
`BLK-TRIAL-SPAWNER-001`; structure/configuration joins remain
`WGEN-JIGSAW-TRIAL-CHAMBERS-001`.

Guaranteed level-three Butcher Beef sink:

The base Butcher level-three tag contains exactly Mutton and Beef purchase records. Its set
requests two, disables duplicates by default and uses random sequence
`minecraft:trade_set/butcher/level_3`. Both predicate-free records are therefore guaranteed; only
their order varies.

`butcher/3/beef_emerald` accepts ten matching Beef and returns one default Emerald. It has maximum
uses `16`, villager XP `20`, reputation discount coefficient `0.05`, and no second cost, merchant
predicate, cost-component predicate or output modifier. The empty component predicate accepts
arbitrary ordinary Beef patches; ten paid stacks and patches are consumed and not copied.

Offer generation consumes no Beef. Successful generic merchant transactions perform the sink
until exhaustion. Butcher level-up/restock, price/demand/reputation adjustment, commit, player
reward and menu synchronization remain with merchant owners. Steak does not satisfy this cost.

Adult Butcher hero-gift Steak acquisition:

The Villager work/play/meet packages include the priority-three, maximum-`100`-tick
`GiveGiftToHero` behavior. It observes the nearest-visible-player memory and requires that player
to carry Hero of the Village. Its initial cooldown is `600`, decremented only on start checks while
such a hero remains visible.

On an admitted run, the Villager targets and looks at that hero. Beyond block distance `5` it sets
a walk/look target at speed `0.5`; once strictly within `5` and more than `20` ticks after start,
it evaluates the profession gift and marks the run gifted even if the reloaded table emitted
nothing. Stopping clears interaction/walk/look memories and samples the next cooldown as
`600 + nextInt(6001)`, uniformly `600..6600`; a timed-out or lost-hero run also resamples.

An adult Butcher selects `gameplay/hero_of_the_village/butcher_gift`. Its one roll chooses equally
among one default Cooked Rabbit, Cooked Chicken, Cooked Porkchop, Cooked Beef or Cooked Mutton, so
Steak probability is `1/5`, count one. The named sequence is
`minecraft:gameplay/hero_of_the_village/butcher_gift`. A baby Villager uses the separate baby gift
table; another profession uses its own mapping or unemployed fallback and cannot emit Steak from
this table.

Each emitted stack becomes an item entity thrown from the Villager toward the hero with the
Villager as thrower and default pickup delay. Hero-effect acquisition/duration remains
`MOB-RAID-001` and `ENT-EFFECT-001`; brain scheduling, memory, pathing, throw kinematics, gift
context and generic loot evaluation remain `MOB-AI-001` and `ITM-LOOT-001`.

Wolf food, healing, growth and love:

Live `wolf_food` closure admits both identities. A tamed injured Wolf takes the specialized heal
branch first, without checking feeder ownership. It consumes one through the player-aware helper,
heals twice the live food nutrition and plays its eating sound. Defaults therefore offer `6` HP
from Raw Beef and `16` HP from Steak; actual health clamps to maximum. Patched nutrition changes
the amount. If the food component is removed but tag membership remains, the Wolf uses its
distinct fallback `2` HP.

Every other Wolf state falls through to generic Animal feeding. An age-unlocked baby consumes one
and advances by ten percent of its remaining whole seconds. An age-zero adult not already in love
consumes one and enters `600`-tick love even when untamed; untamed Wolves still cannot mate because
Wolf mating requires both partners tame. Already-loving adults and age-locked babies consume
nothing; a tamed Wolf can then reach later owner-only interactions. None of these mob paths runs
the player's consumable or changes Wolf hunger/saturation.

Removing either identity from `meat`, removing the nested `meat` entry from `wolf_food`, or
otherwise replacing tag closure blocks these default branches. Wolf state, eventual mating,
offspring, persistence and AI remain `MOB-BREED-001` and `MOB-AI-001`.

**Persistence and reload boundary:**

Beef/Steak stacks persist and synchronize identity, count and arbitrary ordinary component
patches. They store no active-use progress, hunger, death/fire/attacker/Looting context, live
smelting lookup, machine state, recipe knowledge, loot cursor, Trial fixed table, merchant offer,
Villager hero-gift cooldown/target, Wolf state or progression. Those values belong to player,
world, entity, machine, spawner, merchant, Villager brain, Wolf and advancement owners.

Recipe reload changes future explicit cooking and death-table `furnace_smelt` conversion. Removing
the Beef smelting recipe makes fire-qualified future drops remain Beef; replacing it changes both
Furnace output and those drops. Loot reload independently changes bovine, chest, Trial and gift
evaluation. Tag reload changes future Wolf admission. Trade reload changes future Butcher offers
without rewriting existing ones; advancement reload changes listeners. Completed drops, cooking,
loot, offers, gifts, feeds and uses are not replayed. Resource reload independently controls names,
models and textures.

**Client and wire projection:**

Generic item-stack encoding projects raw ID `1139` for Beef or `1140` for Steak plus the component
patch. Locked English names are `Raw Beef` and `Steak`; both are common, add no subtype tooltip and
force no glint.

Each direct item definition selects its same-named generated model and texture. Both appear exactly
once and only in Food & Drinks, ordered Dried Kelp, Raw Beef, Steak, Raw Porkchop. Neither appears
in Ingredients or adds packet layout.

**Branches and aborts:**

Identity/count/components and direct/recursive tags; hand/block/active-use/hunger; Cow/Mooshroom
age/rule/fire/direct-attacker/enchantment/recipe/Looting and both ordered pools; three cooking
domains, machine or Campfire state and extraction; village chest rolls; ominous fixed-table/player
cohort; Butcher set/order/offer lifecycle; Villager age/profession/hero/memory/cooldown/range/
duration/table; Wolf tame/health/age-lock/love/tag/food/ability; save and component/tag/recipe/
advancement/loot/trade/resource reload; wire, language, model, texture and tab.

**Constants and randomness:**

Raw IDs `1139/1140`; common rarity; max stack `64`; food `3/1.8000001` and `8/12.8`; eat `32`;
death base `1..3` plus `round(L*U)`, after an earlier Leather `0..2 + round(L*U)` pool; cooking
times `200/100/600`, XP `0.35/0.35/ignored 0.35`; village chest `1..5` rolls, `3/14`, count
`1..3`; ominous table `7/10`, conditional Steak `3/10`, count `1..2`; Butcher cost/result
`10/1`, uses/XP/discount `16/20/0.05`; hero behavior max `100`, range `5`, delay `>20`, speed
`0.5`, cooldowns initial `600` then `600..6600`, gift Steak `1/5`; Wolf heal `2*nutrition`, love
`600`.

**Side effects:**

Food/statistic/criterion/event/shrink; Cow/Mooshroom Leather and Beef/Steak outputs plus named
cursors; machine/Campfire inputs, timers, outputs, unlocks and possible XP; village/Trial outputs
and fixed Spawner table; Butcher offer/economy; Villager memories/navigation/gift item entity/
cooldown; Wolf health/age/love/sound/consumption; progression, ordinary persistence/wire and direct
client presentation.

**Gates:**

Food/hunger and uninterrupted use; adult plus `mob_drops`; bovine table, fire/smelts-loot/live
recipe and Looting context; exact cooking recipe and machine admission; structure/spawner/table/
player/container context; level-three Butcher and offer validity; adult Butcher plus visible Hero,
cooldown/range/time/table; live Wolf-food closure and Wolf state; registry/stack decode; client
language/model/tab bootstrap.

**State read/written:**

Reads stack identity/count/components/tags, player use/hunger/progression, death/attacker/
enchantment/recipe/loot state, machine/Campfire slots and timers, village/Trial/merchant/Villager
brain context, Wolf state, persistence and client resources. Writes only the consumption, loot,
cooking, progression, trade, gift, Wolf, stack and client state listed above.

**Failure behavior:**

Full-hunger ordinary use returns `FAIL`; interruption commits no finish. Baby/disabled-drop bovines
skip both pools. Failed fire terms retain Beef; missing live smelting recipe warns and also retains
it. Invalid/removed cooking recipes prevent or fallback according to machine owner. Unselected
chest/Trial/gift entries emit alternatives; key-selected ominous encounters emit no Steak. Invalid
or exhausted Butcher offers commit nothing. Lost Hero, range/time expiry or empty gift table emits
no Steak and still reaches stop/cooldown behavior. Ineligible Wolf state consumes nothing unless a
later owned branch succeeds. Reloaded data changes only future evaluation; missing resources cannot
grant authority.

**Boundary cases and quirks:**

Bovine fire conversion is a live smelting-recipe lookup, not a hard-coded Beef-to-Steak rewrite;
recipe reload therefore changes both machine and death output. Looting runs after conversion and
augments whichever identity resulted. Steak has two noncooking sources, while Raw Beef has a chest
source and a guaranteed Villager sink. The Butcher gift behavior marks an admitted close-range run
gifted even if its table returns empty. Wolf healing uses twice live nutrition but falls back to
two HP when food is removed, and mob feeding never invokes the player's food component.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-DATA-001`; `OFF-REPORT-001`;
`net.minecraft.world.item.Items`; `net.minecraft.world.food.Foods`;
`net.minecraft.world.item.component.Consumable#startConsuming`;
`net.minecraft.world.item.component.Consumable#onConsume`;
`net.minecraft.world.food.FoodProperties#onConsume`;
`net.minecraft.world.food.FoodData#eat`;
`net.minecraft.world.entity.LivingEntity#shouldDropLoot`;
`net.minecraft.world.level.storage.loot.functions.SmeltItemFunction#run`;
`net.minecraft.world.level.block.entity.AbstractFurnaceBlockEntity#serverTick`;
`net.minecraft.world.level.block.entity.AbstractFurnaceBlockEntity#createExperience`;
`net.minecraft.world.level.block.CampfireBlock#useItemOn`;
`net.minecraft.world.level.block.entity.CampfireBlockEntity#placeFood`;
`net.minecraft.world.level.block.entity.CampfireBlockEntity#cookTick`;
`net.minecraft.world.entity.npc.villager.AbstractVillager#addOffersFromTradeSet`;
`net.minecraft.world.entity.npc.villager.AbstractVillager#addOffersFromItemListingsWithoutDuplicates`;
`net.minecraft.world.item.trading.VillagerTrade#getOffer`;
`net.minecraft.world.item.trading.TradeSet#calculateNumberOfTrades`;
`net.minecraft.world.entity.ai.behavior.GiveGiftToHero`;
`net.minecraft.world.entity.ai.behavior.VillagerGoalPackages`;
`net.minecraft.world.entity.LivingEntity#dropFromGiftLootTable`;
`net.minecraft.world.entity.ai.behavior.BehaviorUtils#throwItem`;
`net.minecraft.world.entity.animal.wolf.Wolf#mobInteract`;
`net.minecraft.world.entity.animal.wolf.Wolf#isFood`;
`net.minecraft.world.entity.TamableAnimal#feed`;
`net.minecraft.world.entity.animal.Animal#mobInteract`;
`net.minecraft.world.item.CreativeModeTabs`;
`net.minecraft.data.tags.VanillaItemTagsProvider`;
`net.minecraft.data.tags.VillagerTradesTagsProvider`;
`net.minecraft.data.loot.packs.VanillaEntityLoot`;
`net.minecraft.data.loot.packs.VanillaChestLoot`;
`net.minecraft.data.loot.packs.VanillaGiftLoot`;
`net.minecraft.data.recipes.packs.VanillaRecipeProvider`;
`net.minecraft.data.advancements.packs.VanillaHusbandryAdvancements`;
`reports/registries.json#minecraft:{item,entity_type,recipe,loot_table,advancement,villager_trade,trade_set,trial_spawner_config}`;
`reports/minecraft/components/item/{beef,cooked_beef}.json`;
`data/minecraft/tags/item/{meat,wolf_food}.json`;
`data/minecraft/loot_table/{entities/{cow,mooshroom},chests/village/village_butcher,gameplay/hero_of_the_village/butcher_gift,spawners/ominous/trial_chamber/consumables}.json`;
`data/minecraft/recipe/cooked_beef*.json`;
`data/minecraft/advancement/{recipes/food/cooked_beef*,husbandry/balanced_diet}.json`;
`data/minecraft/{villager_trade/butcher/3/beef_emerald,tags/villager_trade/butcher/level_3,trade_set/butcher/level_3}.json`;
`data/minecraft/trial_spawner/trial_chamber/**/ominous.json`;
`assets/minecraft/{items,models/item,textures/item}/{beef,cooked_beef}.*`;
`ITM-FURNACE-001`; `ITM-CAMPFIRE-001`; `ITM-LOOT-001`;
`ITM-ADVANCEMENT-001`; `ITM-HUNGER-001`; `ITM-ENCHANT-001`;
`ENT-DEATH-001`; `ENT-ENTITY-DROPS-001`; `MOB-AI-001`; `MOB-BREED-001`;
`MOB-RAID-001`; `BLK-TRIAL-SPAWNER-001`;
`WGEN-JIGSAW-TRIAL-CHAMBERS-001`; `WGEN-JIGSAW-VILLAGES-001`;
`CLI-UI-001`; `CLI-EFFECT-001`; `EXP-ITM-061`.

**Test vectors:**

Exercise both default, food/consumable-removed and arbitrary component-patched stacks through
hands, hunger/ability, interruption, containers and anvil. Kill adult/baby Cow and Mooshroom
fixtures across `mob_drops`, fire, direct/nonliving/living attacker, `smelts_loot`, Looting levels,
live smelting outputs and every base/bonus draw while tracing Leather-first named cursors.

Cook Beef in Furnace, Smoker and all Campfire slots across time, fuel/lit/output/extraction and
recipe removal/replacement; trigger all grants and XP fractions. Generate every village chest,
ominous fixed-table/player and Butcher gift branch. Generate both level-three offer orders and
transact/exhaust/restock Beef. Feed both items to Wolves across tame/owner/health/age-lock/love/
tag/component/ability states. Reload each data/resource domain, persist/reload/synchronize stacks
and owners, and verify raw IDs, names, models, textures and exact Food-tab order.
