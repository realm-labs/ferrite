# Items mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ITM-SPIDER-EYE-001` — Spider Eyes join four loot sources to poisonous food, Armadillo feeding, crafting and Poison brewing

**Parent:** `PLY-005`, `PLY-006`, `PLY-INPUT-001`, `PLY-INTERACT-001`,
`ITM-001`, `ITM-002`, `ITM-003`, `ITM-004`, `ITM-005`, `ITM-006`,
`ITM-007`, `ITM-USE-001`, `ITM-CONTAINER-001`, `ITM-RECIPE-001`,
`ITM-CRAFT-001`, `ITM-BREW-001`, `ITM-LOOT-001`, `ITM-ADVANCEMENT-001`,
`ITM-HUNGER-001`, `ITM-ANVIL-001`, `ITM-FERMENTED-SPIDER-EYE-001`,
`ITM-SUGAR-001`, `ITM-POTION-001`, `ENT-001`, `ENT-005`, `ENT-DEATH-001`,
`ENT-ENTITY-DROPS-001`, `ENT-EFFECT-001`, `MOB-SPAWN-001`, `MOB-AI-001`,
`MOB-BREED-001`, `WGEN-STRUCTURE-DESERT-PYRAMID-001`, `CLI-001`, `CLI-006`,
`CLI-UI-001`, `CLI-EFFECT-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked registration/components and tag, exhaustive entity/chest loot,
consumable/effect and animal bytecode, the sole recipe/unlock, vanilla brewing graph, potion
payloads, advancement and client assets determine every Spider-Eye-specific branch. Generic
active use, hunger, effects, death, loot, structure, animal breeding, crafting, brewing, stacks
and inventories remain with the cited owners.

**Applies when:**

A `spider_eye` stack is dropped by a Spider, Cave Spider or Witch, generated in a Desert Pyramid,
eaten by a player, offered to an Armadillo or Brewing Stand, consumed in the Fermented Spider Eye
recipe, moved, renamed, persisted, synchronized, selected in a tab, rendered or observed before
and after tag, loot, built-in-pack, recipe, advancement, mix or resource reload.

**Authoritative state:**

`minecraft:spider_eye` is raw item ID `1151`. It is common, nondamageable and has max stack `64`.
It registers through the plain-item path with these two operational components:

- food nutrition `2`, saturation `3.2` and default `can_always_eat=false`;
- the otherwise-default `1.6`-second (`32`-tick) eat consumable, with one apply-effects consumer
  that offers Poison amplifier `0` for `100` ticks with visible particles/icon.

The remaining registered components are the common empty modifiers/enchantments/lore, item-break
sound, translated name, direct item-model key, repair cost, swing animation, tooltip display and
use effects. There is no cooldown, use remainder, tool, equipment, repairable or
identity-specific glint state.

Its sole direct item tag is `#minecraft:armadillo_food`, whose locked value set contains only
Spider Eye. That membership controls Armadillo temptation/food admission but does not make the
item food for other mobs.

**Transition and ordering:**

In-air player use enters the consumable path only when the food listener admits it. A normal
survival player at food level `20` therefore gets `FAIL`; lower hunger admits use, and a
player ability that permits eating at full hunger independently admits it. An admitted use begins
the default eat animation with `CONSUME`. Block clicks retain block-first handling before an
unconsumed result can reach the edible-item fallback.

Interruption, release or live-hand/component replacement before completion commits no item-used
statistic, consume criterion, nutrition, Poison, game event or shrink. At successful server
completion, generic active-use ordering:

1. emits its configured final eat effects;
2. awards the item-used statistic and triggers `consume_item` against the live pre-shrink stack;
3. runs the food listener, clamping hunger after `+2` and saturation after `+3.2`;
4. runs the apply-effects consumer, which draws one entity RNG float and, at default probability
   `1.0`, always offers a fresh `100`-tick amplifier-zero Poison instance;
5. emits the `EAT` game event and shrinks one unless the user has infinite materials.

The probability-one branch still consumes the float before its strict comparison. Poison immunity,
stronger/longer current effects, hidden chains and callback/synchronization results can reject or
merge the offer under `ENT-EFFECT-001`; none rolls back nutrition, event or consumption. There is
no remainder. Infinite-material use retains the Eye but still performs admitted completion and
the item/effect/progression transaction.

Spider Eye is one of the 40 independent AND requirements in
`husbandry/balanced_diet`. Because `consume_item` precedes nutrition and Poison, a completed use
advances this one requirement from the pre-shrink Eye even if the later effect offer is rejected.

Container movement, ordinary pickup/dropping, anvil naming and component patching use generic
owners. The identity adds no dispenser, equipment, repair, composting, furnace or villager-trade
branch.

**Spider and Cave Spider acquisition:**

`entities/spider` and `entities/cave_spider` have equivalent two-pool layouts under separate named
sequences. Their first one-roll pool generates String independently. Their second one-roll pool
admits its sole Spider Eye entry only when `killed_by_player` passes; a non-player-attributed death
therefore spends no Eye count draw and emits no Eye.

An admitted entry creates one Eye and replaces its count with a uniform integer `B` in `-1..1`.
With no living attacking entity or Looting level `L=0`, the enchanted-count function takes no
bonus draw and only `B=1` survives generic empty-stack filtering, giving baseline conditional
emission probability `1/3`.

With a living attacker and `L>0`, it draws a uniform float `U` in `[0,1)`, calculates
`R=round(L*U)` and grows the stack. As with other negative-base tables, `getCount` reports `0`
after a nonpositive stored base, so the final count is

`max(B,0) + round(L*U)`,

not `B + round(L*U)`. A positive bonus revives both `-1` and `0` equally; no count limit is
configured. The preceding String pool and all of its draws remain part of
`minecraft:entities/spider` or `minecraft:entities/cave_spider` before this conditional branch.

Spider/Cave-Spider spawning, combat, player attribution, death admission, table invocation,
empty-stack filtering and world-drop placement remain with `MOB-SPAWN-001`, `ENT-DEATH-001`,
`ENT-ENTITY-DROPS-001` and `ITM-LOOT-001`.

**Witch acquisition:**

The first `minecraft:entities/witch` pool draws an inclusive uniform `1..3` rolls with replacement.
Each roll selects Glowstone Dust, Sugar, Spider Eye, Glass Bottle or Gunpowder at weight `1`, or
Stick at weight `2`; total weight is `7`, so Eye selection is `1/7` per roll.

A selected Eye entry creates one stack, replaces its count with uniform integer `0..2`, then runs
the same optional Looting growth `round(L*U)`. The table has no `killed_by_player` condition:
attacker absence does not block its base count, but a living attacker with positive Looting adds
the growth draw. Base zero can disappear or be revived by a positive bonus. Multiple first-pool
rolls can select Eye repeatedly.

The later one-roll Redstone pool executes after the first pool under the same
`minecraft:entities/witch` sequence and does not create or alter Eyes. Witch spawning/AI/death and
generic loot execution remain with the entity and loot owners; `ITM-SUGAR-001` owns its distinct
entry in the shared first pool.

**Desert Pyramid chest acquisition and pack override:**

The first pool of base `minecraft:chests/desert_pyramid` draws a uniform integer `2..4` rolls with
replacement. Its sixteen direct entries total weight `247`; Spider Eye has weight `25`, so each
roll selects it with probability `25/247` and then draws an inclusive uniform count `1..3`.
Repeated selection can produce separate stacks subject to generic chest insertion.

Enabling the built-in `trade_rebalance` pack replaces that table. Its first pool retains `2..4`
rolls, Eye weight `25`, count `1..3` and sixteen entries, but the replacement entry weights total
`237`, making the per-roll Eye probability `25/237`. The change comes from the neighboring
Leather/Saddle and Book records, not from the Eye entry.

Both tables use sequence `minecraft:chests/desert_pyramid`. After the Eye-bearing pool, each runs
four equal-weight junk-pool rolls and one total-weight-seven trim pool; these later evaluations
cannot change an emitted Eye but advance the same cursor. Pyramid placement/chest assignment,
pack selection, loot execution and inventory insertion remain
`WGEN-STRUCTURE-DESERT-PYRAMID-001`, the reload owners and `ITM-LOOT-001`.

No other locked chest, entity, fishing, gift, barter or trade output directly emits Spider Eye.

**Fermented Spider Eye recipe and unlock:**

The sole crafting sink is the shapeless Fermented Spider Eye recipe. It requires exactly one
Spider Eye, one Brown Mushroom and one Sugar in any three distinct inputs, consumes all three and
returns one default Fermented Spider Eye with no remainder. Arbitrary input patches are accepted
by item identity and not copied.

Its no-display `recipes/root` advancement has one OR requirement: possess a Spider Eye or unlock
the exact `fermented_spider_eye` recipe. Either criterion awards only that recipe. Brown Mushroom,
Sugar and the crafted output do not satisfy the inventory criterion. Result construction, recipe
take and downstream corruption/trade behavior remain `ITM-FERMENTED-SPIDER-EYE-001`.

**Poison brewing graph:**

The vanilla mix builder registers Spider Eye through the start-mix helper. It adds Water plus Eye
to Mundane and Awkward plus Eye to Poison. Ordinary Poison carries amplifier-zero Poison for `900`
ticks (`45` seconds).

Redstone Dust separately maps Poison to Long Poison at amplifier zero for `1800` ticks (`90`
seconds). Glowstone Dust maps Poison to Strong Poison at amplifier `1` for `432` ticks (`21.6`
seconds). Fermented Spider Eye maps Poison and Long Poison to ordinary Harming and Strong Poison
to Strong Harming; those one-tick Instant-Damage corruption edges remain
`ITM-FERMENTED-SPIDER-EYE-001`.

Every admitted edge works for Potion, Splash Potion and Lingering Potion container items. The
container identity is retained while fresh target contents replace source contents; custom color,
custom effects, custom name and duration scale are not preserved. A holder must be present and
match Water or Awkward. Ingredient admission tests Eye identity, accepting arbitrary component
patches and discarding them when one ingredient is consumed.

A completed brew transforms matching bottle slots `0..2` in order, consumes one Eye for up to
three outputs, leaves unmatched bottles unchanged and emits event `1035`. Eye has no remainder, is
not Brewing Stand fuel and is not furnace fuel. Fuel admission, `400`-tick transaction,
cancellation and player-menu take criterion remain `ITM-BREW-001` and `ITM-ADVANCEMENT-001`.
Potion use/projection and Poison ticking remain `ITM-POTION-001` and `ENT-EFFECT-001`.

**Armadillo feeding and temptation:**

An Armadillo first handles a Brush and then rejects every remaining interaction with `FAIL` while
scared. Only an unscared Armadillo delegates Spider Eye to the generic Animal food transaction.
The live `armadillo_food` tag and `Armadillo#isFood` also supply its food-temptation and breeding
AI predicates.

For an adult server-player target at age zero and not already in love, one Eye is consumed through
the player's ability-aware rule, love time becomes `600` ticks, the player becomes love cause,
entity event `18` requests hearts and the Armadillo Eat sound plays. Armadillo's
`canFallInLove` additionally requires the entity not be scared.

For an age-unlocked baby, one Eye instead advances age by ten percent of remaining whole seconds,
truncated to an integer, records the forced-age delta/timer and plays Armadillo Eat. An adult
already in love or age-locked baby receives no server mutation and consumes nothing. Feeding calls
the animal transaction directly: it does not invoke the Eye's player-consumable listener and
therefore neither nourishes nor Poisons the Armadillo.

Offspring creation, mate search, parent cooldown, XP, temptation navigation, love persistence and
client prediction/correction remain `MOB-BREED-001` and `MOB-AI-001`.

**Persistence and reload boundary:**

Eye stacks persist and synchronize identity, count and arbitrary ordinary component patches. They
store no active-use progress, hunger/effect state, advancement knowledge, entity-death context,
loot sequence, structure/pack context, Armadillo age/love/AI state or brewing slot/fuel/timer/mix.
Those values belong to their player, entity, world, loot, progression and machine owners.

Loot or built-in-pack reload changes future drops/chests; tag reload changes future Armadillo food
tests; recipe/advancement reload changes future matching/listeners; a rebuilt baseline mix retains
both Eye start edges while holders/items are enabled. Completed use, loot, crafting, feeding and
brews are not replayed. Resource reload independently controls name and model.

**Client and wire projection:**

Generic item-stack encoding projects raw item ID `1151` plus the stack's component patch. Its
common-rarity name uses locked English text `Spider Eye`; the plain class adds no subtype tooltip
or forced glint.

The direct definition selects generated model `minecraft:item/spider_eye` and its same-named
texture. It appears exactly once in Ingredients, ordered Glistering Melon Slice, Spider Eye,
Pufferfish, and exactly once in Food & Drinks, ordered Rotten Flesh, Spider Eye, Mushroom Stew.

**Branches and aborts:**

Identity/count/components/tag; hand/block/active-use/hunger/effect state; Spider/Cave-Spider
player attribution/base/attacker/Looting and preceding String pool; Witch rolls/weight/base/
Looting/later Redstone pool; base/rebalanced Pyramid table/roll/weight/count/later pools/container;
recipe/unlock; stand fuel/timer/container/holder/custom contents; Armadillo scared/age/age-lock/
love/player/temptation; save, loot/pack/tag/recipe/advancement/mix/resource reload, wire, language,
model and both tab contexts.

**Constants and randomness:**

Raw ID `1151`; common rarity; max stack `64`; food `2/3.2`; eat `32` ticks; one probability-`1`
float and Poison `100@0`; Spider/Cave player gate, base `-1..1`, bonus
`round(L*U[0,1))`; Witch rolls `1..3`, weight `1/7`, base `0..2`, same bonus; Pyramid rolls
`2..4`, base/rebalanced weights `25/247` and `25/237`, count `1..3`; Poison payloads `900@0`,
`1800@0`, `432@1`; owner brew `400`; Armadillo love `600`, baby growth ten percent of remaining
whole seconds.

**Side effects:**

Possible nutrition/saturation, Poison/effect RNG, item/stat/criterion/event state; three entity
loot outputs and sequences; Pyramid chest outputs and cursor; crafting inputs/result/knowledge;
Brewing Stand ingredient/bottles/timer/event and potion state; Armadillo/player stack, love/cause,
age/forced-age, sound/event/AI state; ordinary persistence/wire state; name, model and two tab
entries.

**Gates:**

Food/hunger and live consumable admission; uninterrupted same-stack completion; player-attributed
Spider/Cave death and optional living-attacker Looting; Witch and Pyramid table admission/weights;
active base or rebalanced pack; exact recipe inputs/snapshot; valid stand fuel and Water/Awkward
holder; unscared Armadillo, live food tag and applicable love/growth consequence; registry/stack
decode; client language/model/tab bootstrap.

**State read/written:**

Reads stack identity/count/components/tags, player active-use/hunger/effects/progression, entity
death/attacker and four loot contexts, structure/pack/chest state, recipe knowledge, brewing
slots/fuel/timer/mix/contents, Armadillo/player state, persistence and client resources. Writes
only the consumption, loot, crafting, progression, brewing, feeding, stack and client state listed
above.

**Failure behavior:**

Full-hunger ordinary survival use returns `FAIL`; interruption commits no finish. A rejected
Poison offer does not undo eating. Non-player-attributed Spider/Cave death emits no Eye;
nonpositive final counts disappear. Unselected Witch/Pyramid entries emit their alternatives.
Invalid recipe inputs produce no result. Missing fuel or unmatched holder prevents brewing.
Scared Armadillos return `FAIL`; already-loving adults and age-locked babies consume nothing.
Missing/replaced components, loot/pack, tag, recipe, advancement or mix data removes future paths
without rewriting completed state. Client resource absence follows generic fallback and cannot
grant authority.

**Boundary cases and quirks:**

Default eating always offers Poison yet still spends one probability float. Spider and Cave Spider
Eyes are player-gated and use negative base counts normalized before Looting growth; Witch Eyes
are not player-gated and start at `0..2`. Trade Rebalance raises the Pyramid per-roll Eye chance
from `25/247` to `25/237` without changing the Eye record. Spider Eye possession unlocks its sole
sink. Brewing creates Mundane from Water as well as Poison from Awkward. Scared Armadillos reject
the only member of their food tag before generic feeding, and successful feeding never applies
the Eye's Poison consumer to them.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-DATA-001`; `OFF-REPORT-001`;
`net.minecraft.world.item.Items`;
`net.minecraft.world.item.ItemStack#getCount`;
`net.minecraft.world.item.ItemStack#grow`;
`net.minecraft.world.item.component.Consumable#startConsuming`;
`net.minecraft.world.item.component.Consumable#onConsume`;
`net.minecraft.world.food.FoodProperties#onConsume`;
`net.minecraft.world.food.FoodData#eat`;
`net.minecraft.world.item.consume_effects.ApplyStatusEffectsConsumeEffect#apply`;
`net.minecraft.world.level.storage.loot.functions.SetItemCountFunction#run`;
`net.minecraft.world.level.storage.loot.functions.EnchantedCountIncreaseFunction#run`;
`net.minecraft.world.level.storage.loot.providers.number.UniformGenerator`;
`net.minecraft.world.level.storage.loot.entries.LootPoolSingletonContainer$EntryBase#getWeight`;
`net.minecraft.world.entity.animal.armadillo.Armadillo#mobInteract`;
`net.minecraft.world.entity.animal.armadillo.Armadillo#isFood`;
`net.minecraft.world.entity.animal.armadillo.Armadillo#canFallInLove`;
`net.minecraft.world.entity.animal.armadillo.Armadillo#playEatingSound`;
`net.minecraft.world.entity.animal.Animal#mobInteract`;
`net.minecraft.world.entity.AgeableMob#getSpeedUpSecondsWhenFeeding`;
`net.minecraft.world.item.alchemy.PotionBrewing#addVanillaMixes`;
`net.minecraft.world.item.alchemy.PotionBrewing$Builder#addStartMix`;
`net.minecraft.world.item.alchemy.Potions`;
`net.minecraft.world.item.CreativeModeTabs`;
`net.minecraft.data.loot.packs.VanillaEntityLoot`;
`net.minecraft.data.loot.packs.VanillaChestLoot`;
`net.minecraft.data.recipes.packs.VanillaRecipeProvider`;
`reports/registries.json#minecraft:{item,entity_type,potion,mob_effect,recipe,loot_table,advancement}`;
`reports/minecraft/components/item/spider_eye.json`;
`data/minecraft/tags/item/armadillo_food.json`;
`data/minecraft/loot_table/entities/{spider,cave_spider,witch}.json`;
`data/minecraft/loot_table/chests/desert_pyramid.json`;
`data/minecraft/datapacks/trade_rebalance/data/minecraft/loot_table/chests/desert_pyramid.json`;
`data/minecraft/recipe/fermented_spider_eye.json`;
`data/minecraft/advancement/{recipes/brewing/fermented_spider_eye,husbandry/balanced_diet}.json`;
`assets/minecraft/{items,models/item,textures/item}/spider_eye.*`;
`ITM-USE-001`; `ITM-HUNGER-001`; `ITM-RECIPE-001`; `ITM-CRAFT-001`;
`ITM-BREW-001`; `ITM-LOOT-001`; `ITM-ADVANCEMENT-001`;
`ITM-FERMENTED-SPIDER-EYE-001`; `ITM-POTION-001`; `ENT-DEATH-001`;
`ENT-ENTITY-DROPS-001`; `ENT-EFFECT-001`; `MOB-AI-001`; `MOB-BREED-001`;
`WGEN-STRUCTURE-DESERT-PYRAMID-001`; `CLI-UI-001`; `CLI-EFFECT-001`;
`EXP-ITM-052`.

**Test vectors:**

Exercise default/removed/patched food and consumable components through both hands, full/nonfull
hunger, infinite materials, interruption and every current-Poison merge result while tracing
effect RNG. Kill Spiders/Cave Spiders across attribution/base/attacker/Looting and preceding String
branches; kill Witches across roll/weight/base/Looting/later-pool branches. Generate base and
rebalanced Pyramid chests at every roll/weight/count/later-pool/container boundary. Match/take the
recipe and both unlock criteria. Brew Water/Awkward/Poison and all controls in every potion
container. Feed/tempt adult/baby Armadillos across scared/love/age-lock/tag/player states.
Persist/synchronize and capture raw ID, name, tooltip, model and both exact tab positions before
and after every reload domain.

**Limits:**

This leaf does not duplicate generic active use, hunger/saturation, Poison merge/ticking, mob
spawning/death/drop insertion, Pyramid generation/chest filling, crafting/result-take, Brewing
Stand/potion transaction, Armadillo AI/breeding, advancement state or stack/resource codecs. Those
remain with their cited owners; this rule fixes Spider Eye identity and its exact consumption,
acquisition, crafting, brewing, feeding and presentation joins.
