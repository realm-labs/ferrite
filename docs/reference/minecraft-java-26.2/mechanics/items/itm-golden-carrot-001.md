# Items mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ITM-GOLDEN-CARROT-001` — Golden Carrots join exceptional food, six loot contexts and a Farmer offer to equine/Rabbit feeding, Piglin admiration and Night Vision

**Parent:** `PLY-005`, `PLY-006`, `PLY-INPUT-001`, `PLY-INTERACT-001`,
`ITM-001`, `ITM-002`, `ITM-003`, `ITM-004`, `ITM-005`, `ITM-006`,
`ITM-007`, `ITM-USE-001`, `ITM-CONTAINER-001`, `ITM-RECIPE-001`,
`ITM-CRAFT-001`, `ITM-BREW-001`, `ITM-LOOT-001`, `ITM-ADVANCEMENT-001`,
`ITM-HUNGER-001`, `ITM-ANVIL-001`, `ITM-FERMENTED-SPIDER-EYE-001`,
`ITM-POTION-001`, `ENT-001`, `ENT-005`, `ENT-EFFECT-001`, `MOB-AI-001`,
`MOB-BREED-001`, `BLK-TRIAL-SPAWNER-001`,
`WGEN-JIGSAW-ANCIENT-CITY-001`, `WGEN-JIGSAW-BASTION-001`,
`WGEN-JIGSAW-TRIAL-CHAMBERS-001`, `WGEN-STRUCTURE-RUINED-PORTAL-001`,
`CLI-001`, `CLI-006`, `CLI-UI-001`, `CLI-EFFECT-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked registration/components and four direct tags, the complete code and
data reference set, one recipe and unlock, six loot-table contexts, the guaranteed Farmer offer,
the vanilla brewing graph, equine/Rabbit/Piglin dispatch and client assets determine every
Golden-Carrot-specific branch. Generic food, loot, structure/spawner, merchant, animal, brewing,
effect, stack and inventory behavior remains with the cited owners.

**Applies when:**

A `golden_carrot` stack is crafted, emitted by one of its six loot contexts, offered by a
level-five Farmer, eaten, offered to an equine or Rabbit, noticed or picked up by a Piglin,
supplied to a Brewing Stand, moved, renamed, persisted, synchronized, selected in a tab, rendered
or observed before and after component, tag, loot, recipe, advancement, trade, mix or resource
reload.

**Authoritative state:**

`minecraft:golden_carrot` is raw item ID `1262`. It is common, nondamageable and has maximum stack
`64`. It registers through the plain-item path with these operational components:

- food nutrition `6`, saturation `14.400001` and omitted/default
  `can_always_eat=false`;
- the otherwise-default `1.6`-second (`32`-tick) eat consumable, with no consume-effect entries.

The remaining registered components are the common empty modifiers/enchantments/lore, item-break
sound, translated name, direct item-model key, repair cost, swing animation, tooltip display and
use effects. There is no cooldown, use remainder, tool, equipment, repairable or
identity-specific glint state.

Its direct tags are `#minecraft:horse_food`, `#minecraft:horse_tempt_items`,
`#minecraft:rabbit_food` and `#minecraft:piglin_loved`. The horse-food list has eight entries,
the horse-tempt list has the three golden foods, Rabbit food has Carrot, Golden Carrot and
Dandelion, and Piglin-loved membership does not make this item the exact barter currency.

**Transition and ordering:**

Player consumption:

In-air player use enters the consumable path only when the food listener admits it. An ordinary
survival player at food level `20` therefore gets `FAIL`; lower hunger admits use, and a player
ability that permits full-hunger eating independently admits it. An admitted use begins the
default eat animation with `CONSUME`. Block clicks retain block-first handling before an
unconsumed result can reach the edible-item fallback.

Removing only the food component leaves the empty consumable intact: use is then admitted even at
full hunger, but completion applies no nutrition or saturation. Removing only the consumable
makes plain in-air use pass and prevents food application. A patched food component supplies its
live nutrition, saturation and always-edible flag while the consumable retains its empty
effect list.

Interruption, release or live-hand/component replacement before completion commits no statistic,
criterion, nutrition, game event or shrink. At successful server completion, generic active-use
ordering emits final eat effects, awards the item-used statistic, triggers `consume_item` against
the live pre-shrink stack, applies food, emits `EAT`, and consumes one unless the user has infinite
materials. Food level is clamped after `+6`, and saturation is clamped after `+14.400001`. There
is no consume-effect probability draw, status-effect offer or remainder.

Golden Carrot is one of the `40` independent AND requirements in telemetry-enabled
`husbandry/balanced_diet`. The pre-shrink criterion therefore advances before nutrition. Its
occurrence as the icon of `husbandry/bred_all_animals` is presentation-only: feeding a carrot does
not satisfy any species criterion, while a later successful Horse, Donkey, Mule or Rabbit
breeding event remains with `MOB-BREED-001`.

**Crafting acquisition and progression:**

The sole recipe is shaped:

```text
###
#X#
###
```

`#` is Gold Nugget and `X` is Carrot. It consumes eight nuggets and one centered Carrot, returns
one default Golden Carrot and has no remainder. The complete square is unchanged by mirror or
rotation; an extra occupied cell, missing nugget or displaced center fails ordinary shaped
matching. Input component patches are not copied.

Its no-display `recipes/root` advancement has one OR requirement containing Gold Nugget possession
and exact `golden_carrot` recipe unlock. Either criterion awards only this recipe. Possessing a
Carrot or Golden Carrot does not satisfy the inventory criterion.

**Bastion chest acquisition:**

Two Bastion chest tables place the carrot in their first, one-roll pool:

- `chests/bastion_other` has first-pool total weight `89`; Golden Carrot has weight `12`, so it is
  selected with probability `12/89` and counts uniformly `6..17`;
- `chests/bastion_hoglin_stable` has first-pool total weight `100`; Golden Carrot has weight `10`,
  so it is selected with probability `1/10` and counts uniformly `8..17`.

Each admitted entry creates one default stack and replaces its count. The first table then
evaluates pools with `2`, `3..4`, `1` and `1` rolls; the second evaluates later pools with `3..4`,
`1` and `1` rolls. Those pools cannot emit another Golden Carrot but advance their table's shared
named sequence after the carrot branch. Bastion generation, chest placement, lazy table
evaluation and the table-based `loot_bastion` criterion remain with the generic and structure
owners; opening either table can satisfy that criterion regardless of which first-pool item won.

**Ancient City, Ruined Portal and Trial Chambers chest acquisition:**

`chests/ancient_city_ice_box` has one pool with uniform `4..10` rolls and replacement. Its five
entries total weight `9`; Golden Carrot has default weight `1`, hence probability `1/9` on every
roll and a selected count uniformly `1..10`. Repeated selections are allowed.

`chests/ruined_portal` first draws uniform `4..8` rolls from total direct weight `398`. Golden
Carrot has weight `5`, hence probability `5/398` per roll and count `4..12`. The later one-roll
Empty/Lodestone pool has weights `1/2`, with Lodestone count `1..2`; it cannot alter carrots but
continues the same `minecraft:chests/ruined_portal` cursor.

The root `chests/trial_chambers/reward` first chooses the rare nested table at weight `8/10` or the
common table at `2/10`. `reward_rare` then makes one selection from total weight `23`; Golden
Carrot has weight `2` and count `1..2`. Thus its marginal probability from the root first pool is
`4/5 * 2/23 = 8/115`. The root subsequently evaluates `1..3` common-table rolls and a
`0.25`-conditioned unique-table roll. Root and nested tables use their own named sequences; those
later branches do not emit carrots but remain deterministic work after the first pool.

All selected entries create default stacks, and separate selections remain separate outputs until
generic container insertion. Structure/template placement, loot invocation and insertion remain
with the Ancient City, Ruined Portal, Trial Chambers and loot owners.

**Ominous Trial Spawner reward acquisition:**

All `14` locked ominous Trial Chamber configurations expose the same two-entry
`loot_tables_to_eject` list: ominous key table weight `3` and consumables table weight `7`. The
first admitted reward ejection chooses and stores one table for that encounter. If consumables
wins at probability `7/10`, every registered-player ejection evaluates that same table once.

The consumables table's one roll has total weight `10`: Cooked Beef `3`, Baked Potato `3`, Golden
Carrot `2`, Regeneration Potion `1` and Strength Potion `1`. A carrot therefore wins with
conditional probability `1/5` and counts uniformly `1..2`; marginally, each player evaluation has
probability `7/50`, but outcomes across players share the encounter-level table choice and are not
independent unconditionally. The table uses named sequence
`minecraft:spawners/ominous/trial_chamber/consumables`.

Encounter admission, cohort size, table fixation, one evaluation per registered player, upward
dispensing and persistence remain `BLK-TRIAL-SPAWNER-001`. A key-table encounter cannot emit a
carrot, while an empty consumables result is impossible because every entry emits one positive
count.

**Guaranteed level-five Farmer acquisition:**

The base Farmer level-five tag contains exactly Golden Carrot and Glistering Melon Slice records.
The set requests two records, disables duplicates by default and uses random sequence
`minecraft:trade_set/farmer/level_5`; with two eligible predicate-free records, both offers are
guaranteed and only their order varies.

The carrot offer accepts three matching Emeralds and returns three default Golden Carrots. It has
maximum uses `12`, villager XP `30`, reputation discount multiplier `0.05`, and no additional
cost, merchant predicate or output modifier. Generic Farmer level-up/restock, price/demand/
reputation adjustment, input predicate, atomic trade commit, exhaustion and publication remain
with the merchant owners.

**Night-Vision brewing graph:**

The vanilla mix builder registers exactly Awkward plus Golden Carrot to Night Vision through
direct `addMix`. It does **not** use the start-mix helper: Water plus Golden Carrot does not create
Mundane or any other potion. Ordinary Night Vision carries amplifier-zero Night Vision for `3600`
ticks (`180` seconds).

Redstone Dust separately maps Night Vision to Long Night Vision for `9600` ticks (`480` seconds).
There is no Strong form or Glowstone edge. Fermented Spider Eye separately maps Night Vision to
Invisibility `3600@0` and Long Night Vision to Long Invisibility `9600@0`; Redstone can also extend
ordinary Invisibility to that long form.

Every admitted edge works for Potion, Splash Potion and Lingering Potion containers. Container
identity is retained while fresh target contents replace the source contents; custom color,
effects, name and duration scale are not preserved. A holder must be present and match Awkward.
Ingredient admission tests Golden Carrot identity, accepts arbitrary component patches and
discards them when the ingredient is consumed.

A completed brew transforms matching bottle slots `0..2` in order, consumes one carrot for up to
three outputs, leaves unmatched bottles unchanged and emits event `1035`. Golden Carrot has no
remainder, is not Brewing Stand fuel and is not furnace fuel. Fuel admission, the `400`-tick
transaction, cancellation and player-menu take criterion remain `ITM-BREW-001` and
`ITM-ADVANCEMENT-001`; potion use/projection and Night Vision/Invisibility behavior remain
`ITM-POTION-001` and `ENT-EFFECT-001`.

**Ordinary Horse, Donkey and Mule feeding:**

Unridden Horse and Abstract-Chested-Horse interaction consult live `horse_food` before the shared
hard-coded feed table, so Golden Carrot reaches ordinary Horse, Donkey and Mule unless an earlier
adult-tamed secondary-use inventory branch wins. Llama, Camel and Zombie Horse use other food
selectors; an ordinary unmounted Skeleton Horse bypasses the feed-table check.

Golden Carrot assigns heal `4`, baby growth `60` seconds and temper increment `5`. On the server,
the table first puts a tamed age-zero equine not already in love into `600`-tick love with the
player as cause. It then, in order:

1. heals `4` when health is below maximum;
2. if age-unlocked and baby, emits the happy-particle offer and ages up `60` seconds;
3. if temper is below maximum and either an earlier effect succeeded or the equine is untamed,
   adds `5` through the clamped temper modifier.

Any one success opens the synchronized mouth, plays the subtype eating sound unless silent at
volume `1` and pitch `1+(U1-U2)*0.2`, emits `EAT`, consumes one through the player's ability-aware
rule and returns server success. The registered player-food component is not invoked.

A tamed full-health adult not already in love therefore consumes solely for love; the same use can
also heal and raise temper. An untamed adult never receives love, but can consume for healing or
temper. A full-health, maximum-temper untamed adult and a full-health already-loving tamed adult
return `PASS`. Babies do not enter love but can heal, grow and raise temper. Horse and Donkey can
later use eligible love state to parent with their allowed mates; Mule can retain love but its
mating predicate remains false.

**Vehicle dispatch and equine temptation:**

Horse and Abstract-Chested-Horse delegate vehicle interaction to `AbstractHorse` before the
specialized feed table. That path delegates to generic `Animal.mobInteract`, where live
`horse_food` membership consumes one for `600`-tick adult love or advances an age-unlocked baby by
`floor((remainingBabyTicks/20)*0.1)` seconds. It does not heal, modify temper, open the mouth, play
the equine eating sound or emit the specialized `EAT` event. Already-loving adults and age-locked
babies consume nothing on the server.

A tamed Skeleton Horse also delegates to the generic route: an adult vehicle can enter love even
though it cannot mate, while unmounted adults and its age-locked babies gain no carrot consequence.
Zombie Horse, Llama and Camel selectors reject this direct tag membership. Unnatural untamed
vehicle fixtures can receive generic love because `Animal` tests only age and existing love, not
tame state.

Separately, normal Horse, Donkey and Mule goals test `horse_tempt_items`. The nearest player within
the live Tempt Range attribute (baseline `10`) holding a Golden Carrot in either hand can own the
non-line-of-sight temptation goal. It looks at that player and navigates at speed `1.25` until
within `2.5` blocks, cannot be scared by player movement, and has the generic reduced
`ceil(100/2)=50`-tick cooldown after stopping. Skeleton and Zombie Horse replace this behavior-goal set,
Llama uses `llama_tempt_items`, and Camel uses its brain, so the direct horse-tempt tag does not
admit those subtypes.

**Rabbit feeding and temptation:**

Rabbit `isFood` and its priority-three temptation predicate both test live `rabbit_food`.
Generic Animal interaction consumes one Golden Carrot from an age-zero Rabbit not already in love,
sets love to `600` ticks with the server player as cause and broadcasts hearts. An age-unlocked
baby instead consumes one and advances by ten percent of its remaining whole seconds, with forced
age effects. Already-loving adults and age-locked babies consume nothing. Rabbit's generic eating
sound hook is empty, and the carrot's player-food component is never invoked.

The nearest eligible player inside the Rabbit's live baseline-`10` Tempt Range holding the carrot
in either hand can own its non-line-of-sight goal. It navigates at speed `1.0`, stops within `2.5`
blocks, cannot be scared, and receives the generic reduced `ceil(100/2)=50`-tick restart delay. Evil variant
changes combat and sound-source behavior but not `rabbit_food`, feeding or temptation admission.
Mate search, offspring variant selection and the eventual `bred_animals` trigger remain
`MOB-BREED-001`.

**Piglin-loved pickup and admiration:**

Subject to generic baby-ignore, repellent, attack/admirer, reachability and inventory gates, a
Piglin can want a Golden-Carrot item entity. Pickup removes exactly one non-nugget carrot, leaves
the rest in the entity, erases `TIME_TRYING_TO_REACH_ADMIRE_ITEM`, moves the carrot to the off
hand and sets `ADMIRING_ITEM` for `119` ticks. Moving it first drops any previous offhand stack.

When holding ends, an adult generates no barter loot because the exact currency remains Gold
Ingot. It first attempts equipment replacement and otherwise stores the plain carrot, throwing
overflow toward a generic random position. Baby finalization retains its separate policy. A player
holding the carrot also satisfies `isPlayerHoldingLovedItem`, feeding sensor/look/
nearest-wanted-player and jealous-sound decisions without transfer. All remaining Piglin
activity, memories, navigation, inventory, sound and combat behavior remains `MOB-AI-001`.

**Persistence and reload boundary:**

Golden-Carrot stacks persist and synchronize identity, count and arbitrary ordinary component
patches. They store no active-use progress, hunger, recipe knowledge, loot sequence/structure
context, fixed Trial-Spawner table, offer lifecycle, stand mix/timer, equine or Rabbit
health/age/temper/love/temptation, or Piglin memory/inventory state. Those values belong to their
player, world, structure, spawner, merchant, machine and entity owners.

Loot reload independently changes future evaluations of all six contexts. Recipe/advancement
reload changes future matching and listeners; tag reload changes future animal/Piglin admission
and temptation; trade reload changes future Farmer sets without rewriting existing offers. A
rebuilt baseline mix retains the direct Awkward edge while holders/items are enabled. Completed
uses, crafts, loot, offers, feeds, pickups and brews are not replayed. Resource reload
independently controls name and model.

**Client and wire projection:**

Generic item-stack encoding projects raw item ID `1262` plus the stack's component patch. Its
common-rarity name uses locked English text `Golden Carrot`; the plain class adds no subtype
tooltip or forced glint.

The direct item definition selects generated model `minecraft:item/golden_carrot` and its
same-named texture. It appears exactly once in Ingredients, ordered Magma Cream, Golden Carrot,
Ghast Tear, and once in Food & Drinks, ordered Carrot, Golden Carrot, Potato. Its use as the
client-visible Bred All Animals icon does not add another creative entry.

**Branches and aborts:**

Identity/count/components and four tags; hand/block/active-use/hunger; recipe grid and unlock;
six loot contexts, every roll/weight/count/nested/fixed-table branch and insertion; Farmer
set/order/offer/economy; stand fuel/timer/container/holder/custom contents; equine subtype/
vehicle/tame/secondary-use/health/age-lock/temper/love/temptation; Rabbit age/love/variant/
temptation; Piglin subtype/activity/offhand/equipment/inventory/player-held state; save, component/
tag/loot/recipe/advancement/trade/mix/resource reload, wire, language, model and both tabs.

**Constants and randomness:**

Raw ID `1262`; common rarity; max stack `64`; food `6/14.400001`; eat `32` ticks; recipe
`8` Nuggets plus `1` Carrot to `1`; Bastion Other `12/89`, count `6..17`; Hoglin Stable `1/10`,
count `8..17`; Ice Box `4..10` rolls, `1/9`, count `1..10`; Ruined Portal `4..8` rolls,
`5/398`, count `4..12`; Trial reward `8/115`, count `1..2`; ominous fixed-table choice `7/10`
then carrot `1/5`, count `1..2`; Farmer `3` Emeralds to `3`, uses/XP/discount `12/30/0.05`;
Night Vision `3600@0` and `9600@0`; equine heal/growth/temper `4/60/+5`, love `600`;
animal generic growth ten percent of remaining whole seconds; tempt ranges/speeds/stops
`10/1.25/2.5` equine and `10/1.0/2.5` Rabbit; reduced temptation restart delay `50`; Piglin admiration
`119`.

**Side effects:**

Nutrition/saturation, statistic, criterion, event and shrink; crafting result/knowledge; six
possible loot outputs and named cursors; fixed Trial-Spawner table and ejection; Farmer offers,
inputs/results/uses/XP/economy; stand ingredient/bottles/timer/event and potion state; equine
health/age/temper/love/mouth/sound/particles/event/navigation, Rabbit age/love/hearts/navigation,
Piglin item-entity/offhand/inventory/memory/sensor state; ordinary stack persistence/wire state;
name, direct model, advancement icon and two tab entries.

**Gates:**

Food/hunger and live consumable admission; uninterrupted same-stack completion; exact recipe and
active snapshot; structure/spawner/table/roll/nested/container admission; level-five Farmer set and
valid offer; valid stand fuel plus Awkward holder; live tag plus every equine/Rabbit/Piglin subtype
and state precedence; registry/stack decode; client language/model/tab bootstrap.

**State read/written:**

Reads stack identity/count/components/tags, player use/hunger/progression, recipe state, six loot
and structure/spawner contexts, merchant state, stand slots/fuel/timer/mix/contents, equine/Rabbit/
Piglin state, persistence and client resources. Writes only the consumption, crafting,
progression, loot, trade, brewing, animal/Piglin, stack and client state listed above.

**Failure behavior:**

Full-hunger ordinary survival use returns `FAIL`; interruption commits no finish. Invalid recipe
inputs produce no result. An unselected loot entry emits its alternative; a key-selected ominous
encounter emits no carrot. Missing or exhausted Farmer offer commits nothing. Water, holderless
contents and all non-Awkward holders do not brew. Removed tags, earlier inventory/vehicle
dispatch or ineligible age/health/temper/love state consume nothing unless another specified
generic path succeeds. Rejected Piglin pickup gates transfer nothing. Missing/replaced component,
tag, loot, recipe, advancement, trade or mix data changes only future attempts. Client-resource
absence follows generic fallback and cannot grant authority.

**Boundary cases and quirks:**

The recipe unlocks from Gold Nuggets rather than its center Carrot. Six acquisition contexts use
different denominators, and the Trial-Spawner table choice correlates every player's later
evaluation. Golden Carrot has only an Awkward brewing edge: Water does not become Mundane.
Ordinary tamed equine feeding performs love before heal/growth/temper, while vehicle dispatch
switches to percentage growth/love and loses heal/temper/mouth/event behavior. Mule and Skeleton
Horse can retain unusable love. Animal feeding never applies the registered player-food
component. Piglin admiration does not imply barter. Bred All Animals uses Golden Carrot only as
its icon.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-DATA-001`; `OFF-REPORT-001`;
`net.minecraft.world.item.Items`; `net.minecraft.world.food.Foods`;
`net.minecraft.world.item.component.Consumable#startConsuming`;
`net.minecraft.world.item.component.Consumable#onConsume`;
`net.minecraft.world.food.FoodProperties#onConsume`;
`net.minecraft.world.food.FoodData#eat`;
`net.minecraft.world.entity.animal.equine.Horse#mobInteract`;
`net.minecraft.world.entity.animal.equine.AbstractChestedHorse#mobInteract`;
`net.minecraft.world.entity.animal.equine.AbstractHorse#fedFood`;
`net.minecraft.world.entity.animal.equine.AbstractHorse#handleEating`;
`net.minecraft.world.entity.animal.equine.AbstractHorse#addBehaviourGoals`;
`net.minecraft.world.entity.animal.equine.SkeletonHorse#mobInteract`;
`net.minecraft.world.entity.animal.equine.ZombieHorse#isFood`;
`net.minecraft.world.entity.animal.rabbit.Rabbit#isFood`;
`net.minecraft.world.entity.animal.Animal#mobInteract`;
`net.minecraft.world.entity.ai.goal.TemptGoal`;
`net.minecraft.world.entity.monster.piglin.PiglinAi#isLovedItem`;
`net.minecraft.world.entity.monster.piglin.PiglinAi#wantsToPickup`;
`net.minecraft.world.entity.monster.piglin.PiglinAi#pickUpItem`;
`net.minecraft.world.entity.monster.piglin.PiglinAi#stopHoldingOffHandItem`;
`net.minecraft.world.entity.monster.piglin.PiglinAi#isPlayerHoldingLovedItem`;
`net.minecraft.world.entity.npc.villager.AbstractVillager#addOffersFromTradeSet`;
`net.minecraft.world.entity.npc.villager.AbstractVillager#addOffersFromItemListingsWithoutDuplicates`;
`net.minecraft.world.item.trading.VillagerTrade#getOffer`;
`net.minecraft.world.item.trading.TradeSet#calculateNumberOfTrades`;
`net.minecraft.world.item.alchemy.PotionBrewing#addVanillaMixes`;
`net.minecraft.world.item.alchemy.PotionBrewing$Builder#addMix`;
`net.minecraft.world.item.alchemy.Potions`;
`net.minecraft.world.item.CreativeModeTabs`;
`net.minecraft.data.loot.packs.VanillaChestLoot`;
`net.minecraft.data.recipes.packs.VanillaRecipeProvider`;
`net.minecraft.data.advancements.packs.VanillaHusbandryAdvancements`;
`reports/registries.json#minecraft:{item,potion,mob_effect,loot_table,advancement,villager_trade,trade_set}`;
`reports/minecraft/components/item/golden_carrot.json`;
`data/minecraft/tags/item/{horse_food,horse_tempt_items,rabbit_food,piglin_loved}.json`;
`data/minecraft/recipe/golden_carrot.json`;
`data/minecraft/advancement/{recipes/brewing/golden_carrot,husbandry/{balanced_diet,bred_all_animals}}.json`;
`data/minecraft/loot_table/{chests/{ancient_city_ice_box,bastion_hoglin_stable,bastion_other,ruined_portal,trial_chambers/{reward,reward_rare}},spawners/ominous/trial_chamber/consumables}.json`;
`data/minecraft/trial_spawner/trial_chamber/**/ominous.json`;
`data/minecraft/{villager_trade/farmer/5/emerald_golden_carrot,tags/villager_trade/farmer/level_5,trade_set/farmer/level_5}.json`;
`assets/minecraft/{items,models/item,textures/item}/golden_carrot.*`;
`ITM-RECIPE-001`; `ITM-CRAFT-001`; `ITM-BREW-001`; `ITM-LOOT-001`;
`ITM-ADVANCEMENT-001`; `ITM-HUNGER-001`; `ITM-FERMENTED-SPIDER-EYE-001`;
`ITM-POTION-001`; `MOB-AI-001`; `MOB-BREED-001`; `BLK-TRIAL-SPAWNER-001`;
`WGEN-JIGSAW-ANCIENT-CITY-001`; `WGEN-JIGSAW-BASTION-001`;
`WGEN-JIGSAW-TRIAL-CHAMBERS-001`; `WGEN-STRUCTURE-RUINED-PORTAL-001`;
`CLI-UI-001`; `CLI-EFFECT-001`; `EXP-ITM-054`.

**Test vectors:**

Exercise default, food/consumable-removed and patched stacks through both hands, hunger/ability,
interruption, containers and anvil. Match/take the recipe across all grids, patches and both
unlock criteria. Generate both Bastion tables, Ice Box, Ruined Portal, Trial reward and all
ominous spawner paths through every roll, weight, count, nesting, fixed-table correlation, later
pool and insertion branch while tracing each named cursor.

Generate both Farmer offer orders, then use, exhaust and restock the carrot offer. Brew
Potion/Splash/Lingering containers across Water, Awkward, Night Vision and controls, including
long, Glowstone and both corruption paths. Feed and tempt every Horse/Donkey/Mule/Skeleton/
Zombie/Llama/Camel and Rabbit state across vehicle, tame, secondary use, health, age-lock, temper,
love and tag boundaries. Exercise every Piglin pickup/admiration/offhand/inventory/player-held
gate. Persist/reload/synchronize and verify raw ID, name, tooltip, model, advancement icon and both
tab positions before/after every reload domain.

**Limits:**

This leaf does not duplicate generic food completion, crafting/result take, structure placement,
loot execution/container insertion, Trial Spawner runtime, Farmer/merchant lifecycle, Brewing
Stand, potion/effect, animal mating/offspring/persistence, Piglin activity, or stack/resource
codecs. Those remain with their cited owners; this rule fixes Golden Carrot identity and its exact
food, acquisition, crafting, trade, brewing, animal, Piglin and presentation joins.
