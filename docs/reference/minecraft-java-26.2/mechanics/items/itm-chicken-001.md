# Items mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ITM-CHICKEN-001` — Raw and Cooked Chicken join toxic food and fire-converting poultry drops to cooking, gifts, Trial rewards, trades and Wolf feeding

**Parent:** `PLY-005`, `PLY-006`, `PLY-INPUT-001`, `PLY-INTERACT-001`,
`ITM-001`, `ITM-002`, `ITM-003`, `ITM-004`, `ITM-005`, `ITM-006`,
`ITM-007`, `ITM-USE-001`, `ITM-CONTAINER-001`, `ITM-RECIPE-001`,
`ITM-CRAFT-001`, `ITM-FURNACE-001`, `ITM-CAMPFIRE-001`, `ITM-LOOT-001`,
`ITM-ADVANCEMENT-001`, `ITM-HUNGER-001`, `ITM-ENCHANT-001`,
`ITM-ANVIL-001`, `ENT-001`, `ENT-DEATH-001`, `ENT-ENTITY-DROPS-001`,
`ENT-EFFECT-001`, `MOB-AI-001`, `MOB-BREED-001`, `MOB-RAID-001`,
`BLK-TRIAL-SPAWNER-001`, `WGEN-JIGSAW-TRIAL-CHAMBERS-001`, `CLI-001`,
`CLI-006`, `CLI-UI-001`, `CLI-EFFECT-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked registration/components, both exact-item data references, Chicken
death loot with live fire smelting, three cooking recipes/unlocks, Cat and Butcher gifts, normal
Trial consumables, two Butcher records, Wolf tag closure, Balanced Diet and direct client
resources determine every Chicken-item branch. Generic active use, effect merging, death, loot,
machine, Cat/Villager AI, Trial Spawner, merchant, Wolf, progression, stack and client behavior
remains with the cited owners.

**Applies when:**

A `chicken` or `cooked_chicken` stack is eaten, emitted by Chicken death, Cat morning gift,
normal Trial reward or Butcher hero gift, cooked, bought or sold through a Butcher, offered to a
Wolf, moved, renamed, persisted, synchronized or rendered before and after component, tag, recipe,
advancement, loot, trade or resource reload.

**Authoritative state:**

| Item | Raw ID | Food | Consumable |
|---|---:|---|---|
| `minecraft:chicken` | `1141` | nutrition `2`, saturation `1.2` | ordinary `32`-tick eat; one probability-`0.3` Hunger effect |
| `minecraft:cooked_chicken` | `1142` | nutrition `6`, saturation `7.2000003` | ordinary `32`-tick eat; no consume-effect entries |

Both are common, nondamageable plain `Item` instances with maximum stack `64` and
`can_always_eat=false`. Their common remaining components are empty attribute modifiers,
enchantments and lore, item-break sound, translated name, direct item-model key, repair cost,
swing animation, tooltip display and use effects. Raw Chicken's effect is Hunger amplifier `0`
for `600` ticks with visible icon.

Both are direct members of `meat`; `wolf_food` includes the live `meat` tag. They have no direct
`wolf_food` membership and are not in `chicken_food`: the latter tag selects seed-like breeding
food for the Chicken entity, not these meat items.

**Transition and ordering:**

Player consumption:

In-air use enters generic food consumption only below full hunger or when player ability permits
eating at full hunger. Block interaction remains block-first. Interruption, release or live-hand/
component replacement before completion commits no statistic, criterion, food, Hunger effect,
game event or shrink.

At successful server completion, generic active-use ordering emits final eat effects, awards the
item-used statistic, triggers `consume_item` against the live pre-shrink stack, applies food,
runs consume effects, emits the `EAT` game event and shrinks one unless the player has infinite
materials.

Default Raw Chicken adds `2` food and `1.2` saturation subject to clamping, then spends one entity
RNG float `U`. Exactly when `U < 0.3`, it offers a fresh `600`-tick amplifier-zero Hunger instance.
Default Cooked Chicken adds `6` and `7.2000003` and spends no consume-effect RNG. Effect immunity,
stronger/longer Hunger, hidden chains and callbacks can reject or merge the Raw-Chicken offer
under `ENT-EFFECT-001` without rolling back food, progression, event or consumption. Neither item
has a remainder.

Raw Chicken and Cooked Chicken are two independent requirements of telemetry-enabled
`husbandry/balanced_diet`. Because `consume_item` precedes food and Raw Chicken's effect roll, a
completed use advances the matching requirement even if food clamps or the Hunger offer is
rejected. The full advancement requires all `40` food predicates and awards `100` experience.

Chicken death acquisition:

An admitted adult Chicken death table evaluates a Feather pool first and the Chicken-item pool
second under random sequence `minecraft:entities/chicken`. Baby/death-rule/table admission,
equipment and XP remain with the entity owners.

The first pool creates Feather, replaces count with uniform integer `0..2`, then applies Looting
count increase. The second creates exactly one default Raw Chicken, conditionally runs
`furnace_smelt`, then applies the same Looting function. For a living attacking entity and
Looting level `L>0`, each bonus spends its own float `U` and adds `round(L*U)`; absent/nonliving
attacker or level zero returns without that bonus draw. Feather and meat use distinct base/bonus
draws in their fixed pool order.

The smelt function runs when the Chicken is on fire or the direct attacker's main hand has an
enchantment in `smelts_loot`. It resolves the live smelting recipe for Raw Chicken at death-table
execution. The locked recipe converts the base stack to Cooked Chicken before the meat Looting
bonus; a missing recipe leaves Raw Chicken, while a replacement recipe controls the future
converted output. Fire and enchantment conditions are ORed, so satisfying both does not smelt
twice. Looting can increase the final raw or converted count without another smelting lookup.

Cooking and recipe progression:

Three exact recipes accept one Raw Chicken and emit one default Cooked Chicken:

| Recipe | Domain | Time | Recipe XP |
|---|---|---:|---:|
| `cooked_chicken` | Furnace | `200` ticks | `0.35` |
| `cooked_chicken_from_smoking` | Smoker | `100` ticks | `0.35` |
| `cooked_chicken_from_campfire_cooking` | Campfire | `600` ticks | `0.35` |

Each no-display recipe advancement has one OR requirement: exact Raw Chicken possession or the
matching recipe already unlocked grants that recipe. Cooked Chicken possession does not satisfy
these inventory criteria.

Furnace and Smoker recipe completion accumulates `0.35` per completed item. Player extraction
uses the furnace-family owner to create integer XP from accumulated fractions and award recipe/
smelt criteria. Campfire re-resolves the live recipe on completion, emits its result or fallback
input under its owner, and awards neither recipe XP nor recipe unlock. Input patches do not copy
to the default Cooked Chicken result.

Cat morning-gift acquisition:

A tame Cat's relax-on-owner goal can attempt a morning gift only after its qualified owner-sleep
sequence. On goal stop it requires sleep timer at least `100`, a day fraction in `(0.77,0.8)`,
then spends one level RNG float against the live
`minecraft:gameplay/cat_waking_up_gift_chance` environment attribute. The normal wake marker at
tick `0` resolves `0.7`. Goal, sleep and attribute behavior remain with the mob owners.

A passed chance attempts the Cat's random teleport offsets, ignores teleport failure, builds gift
context at the resulting Cat position and evaluates `gameplay/cat_morning_gift`. Six entries,
including Raw Chicken, have weight `10`; Phantom Membrane has weight `2`. Conditional Chicken
probability is therefore `10/62 = 5/31`, count one. At live gift chance `g`, a qualified stop
emits Raw Chicken with probability `5g/31`; under the normal `0.7` value this is `7/62`.
Chance, teleport and table selection use distinct RNG sources, with named gift sequence
`minecraft:gameplay/cat_morning_gift`.

Normal Trial-Spawner Cooked-Chicken acquisition:

All `14` normal Trial-Chamber configurations inherit the builder's equal-weight choice between
`spawners/trial_chamber/key` and `spawners/trial_chamber/consumables`. One table is fixed for an
encounter and every registered UUID evaluates that same table once when its reward is due, so
player outcomes are correlated by the `1/2` consumables choice.

The consumables table makes one roll over total weight `10`: Cooked Chicken `3`, Bread `3`, Baked
Potato `2`, Regeneration Potion `1` and Swiftness Potion `1`. Conditional Cooked-Chicken
probability is `3/10`, count exactly one and sequence
`minecraft:spawners/trial_chamber/consumables`. Marginally each registered-player evaluation emits
it with probability `1/2 * 3/10 = 3/20`. Encounter admission, fixed table, registered-player
cohort, ejection and reset remain with `BLK-TRIAL-SPAWNER-001`; configuration and placement remain
with `WGEN-JIGSAW-TRIAL-CHAMBERS-001`.

Butcher trade joins:

The base level-one Butcher tag contains Raw-Chicken, Raw-Porkchop and Raw-Rabbit purchases plus a
Rabbit-Stew sale. Its trade set selects two distinct candidates with random sequence
`minecraft:trade_set/butcher/level_1`. All four records are predicate-free, so the Raw-Chicken
offer has marginal inclusion probability `2/4 = 1/2`.

`butcher/1/chicken_emerald` wants `14` matching Raw Chicken and gives one default Emerald. It has
maximum uses `16`, Villager XP `2`, reputation discount coefficient `0.05`, and no second cost,
merchant predicate or modifier. Empty component matching accepts ordinary patches.

The level-two tag contains Coal purchase, Cooked-Porkchop sale and Cooked-Chicken sale. Its set
selects two distinct candidates with sequence `minecraft:trade_set/butcher/level_2`; Cooked
Chicken therefore has inclusion probability `2/3`.

`butcher/2/emerald_cooked_chicken` wants one matching Emerald and gives eight default Cooked
Chicken. It has maximum uses `16`, Villager XP `5`, reputation discount `0.05`, no second cost or
predicate and no output modifier. Generic demand, special price, reputation, restock, use and
merchant XP behavior remains with the merchant and Villager owners. Trade Rebalance does not
replace either Butcher tag or record.

Butcher hero-gift acquisition:

An adult Butcher Villager with the Hero-of-the-Village behavior can evaluate
`gameplay/hero_of_the_village/butcher_gift` after locating an admitted visible Hero. The one-roll
table selects uniformly among Cooked Rabbit, Cooked Chicken, Cooked Porkchop, Steak and Cooked
Mutton; Cooked Chicken therefore has conditional probability `1/5` and count one.

The initial eligible cooldown is `600` ticks and later eligible cooldown is
`600 + nextInt(6001)`. The behavior requires the target within five blocks, remains active for
at most `100` ticks and throws only after elapsed time exceeds `20`; target/walk/look memories,
profession/age admission, table invocation, throw motion and stopped-memory cleanup remain with
`MOB-RAID-001`, `MOB-AI-001` and the loot owner.

Wolf feeding:

The live `meat -> wolf_food` closure admits both identities to Wolf food handling. Taming remains
the exact-Bone branch and is not entered.

For a tamed injured Wolf, the server's subtype branch heals twice the live food nutrition before
generic animal feeding: default Raw Chicken heals `4` and default Cooked Chicken heals `12`.
Removing the food component falls back to heal `2`; arbitrary other nutrition values change this
amount. The player-aware consume helper spends one unless materials are infinite, plays the eat
sound and returns success. Raw Chicken's player consume-effect entry is not executed, so feeding
does not give Hunger to player or Wolf.

Other admitted Wolf states continue to generic feeding: an age-locked baby does not consume;
another baby consumes one and reduces remaining growth time by ten percent; an adult able to fall
in love consumes one and starts `600`-tick love; other adult states do not consume. Exact tame,
owner, health, age, love, ability and side ordering remain with `MOB-BREED-001`.

**Persistence and reload boundary:**

Stacks persist identity, count and arbitrary component patches. They store no active-use progress,
hunger/effect state, death/fire/attacker/Looting context, live smelting lookup, machine state,
recipe knowledge, Cat/Trial/Villager state, merchant offer, Wolf state or advancement state.

Recipe reload changes future cooking and death-table `furnace_smelt` resolution. Loot reload
changes future Chicken, Cat, Trial and Butcher-gift evaluations. Tag reload changes future Wolf
admission. Trade reload changes future Butcher offers without rewriting existing ones;
advancement reload changes listeners. Completed uses, drops, cooking, gifts, rewards, offers and
feeds are not replayed. Resource reload independently controls language, models and textures.

**Client and wire projection:**

Generic item-stack encoding projects raw ID `1141` or `1142` plus component patches. Both use
common rarity, no subtype tooltip and no forced glint. Locked English names are `Raw Chicken` and
`Cooked Chicken`.

Each direct item definition selects the same-named generated model and texture. Both appear
exactly once and only in Food & Drinks, ordered Raw Mutton, Cooked Mutton, Raw Chicken, Cooked
Chicken, Raw Rabbit, Cooked Rabbit.

**Branches and aborts:**

Identity/count/food/consumable/tag state; player hunger/ability/use/effect RNG; adult/death rule,
fire/direct attacker/smelts-loot/live recipe/Looting and pool order; three machine recipes and
unlocks; Cat goal/chance/teleport/table; Trial config/fixed table/player/table; Butcher level/
candidate/order/offer lifecycle and hero behavior; Wolf side/tame/owner/health/age/love/ability;
persistence/reload, raw ID, name, model, texture, tab and wire.

**Constants and randomness:**

Raw IDs `1141/1142`; common rarity; max `64`; food `2/1.2` and `6/7.2000003`; eat `32` ticks;
Raw Hunger `600@0` at `0.3`; Feather `0..2`, meat base `1`, each plus `round(L*U)`; cooking
`200/100/600` and `0.35`; Cat weight `10/62`, normal chance result `7/62`; Trial
`1/2 * 3/10 = 3/20`; trade inclusion `1/2` and `2/3`, costs/results/uses/XP
`14→1/16/2` and `1→8/16/5`; hero gift `1/5`.

**Side effects:**

Food/effect/statistic/criterion/event/shrink; Feather then raw/cooked death output and named
cursor; machine progress/output/unlock/XP; Cat teleport/gift item; Trial fixed table/reward;
Butcher offer/economy and hero gift item; Wolf health/growth/love/consumption; persistence, wire
and client projection.

**Gates:**

Food/hunger and uninterrupted use; adult/drop-enabled death; fire/smelts-loot/live recipe and
Looting context; cooking recipe/machine state; Cat owner sleep/goal/chance; normal Trial
encounter/player; Butcher profession/level/offer/hero behavior; live Wolf-food closure and Wolf
state; registry/component decode and client resource bootstrap.

**State read/written:**

Reads stack identity/count/components/tags, player food/use/effects/progression, Chicken death/
attacker/enchantment/recipe state, machine slots/timers, Cat/Trial/merchant/Villager brain and
Wolf state, persistence and resources. Writes only the consumption, effect, loot, cooking,
progression, gift, reward, trade, Wolf, stack and client state listed above.

**Failure behavior:**

Full hunger without permission or interrupted use commits nothing. A failed Raw-Chicken effect
roll or effect admission leaves food and consumption committed. Baby or drop-disabled Chicken
emits neither pool; missing smelting recipe leaves admitted death output raw. Missing/blocked
machine recipe makes no normal result. Failed Cat chance or alternate gift, Trial key selection,
unselected/null/exhausted trade, wrong Villager state and alternate hero gift emit no Chicken
item. Non-food or ineligible Wolf states do not consume. Missing client resources grant no
authority.

**Boundary cases and quirks:**

Raw Chicken's Hunger roll occurs only for player-style consumable completion, not Wolf feeding.
Death smelting precedes the meat Looting bonus and uses a live recipe. Cat-gift Chicken and the
Butcher purchase are Raw; normal Trial reward, level-two sale and hero gift are Cooked. The
normal Trial table choice is encounter-correlated across registered players. Both items satisfy
separate Balanced-Diet requirements.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-DATA-001`; `OFF-REPORT-001`;
`net.minecraft.world.item.Items`; `net.minecraft.world.item.CreativeModeTabs`;
`net.minecraft.world.item.component.Consumable#onConsume`;
`net.minecraft.world.item.consume_effects.ApplyStatusEffectsConsumeEffect#apply`;
`net.minecraft.world.entity.animal.chicken.Chicken`;
`net.minecraft.world.level.storage.loot.functions.SmeltItemFunction#run`;
`net.minecraft.world.entity.animal.feline.Cat$CatRelaxOnOwnerGoal#stop`;
`net.minecraft.world.entity.animal.feline.Cat$CatRelaxOnOwnerGoal#giveMorningGift`;
`net.minecraft.world.level.block.entity.trialspawner.TrialSpawnerConfig$Builder`;
`net.minecraft.world.entity.npc.villager.AbstractVillager#addOffersFromTradeSet`;
`net.minecraft.world.entity.ai.behavior.GiveGiftToHero`;
`net.minecraft.world.entity.animal.wolf.Wolf#mobInteract`;
`net.minecraft.data.loot.packs.VanillaEntityLoot`;
`net.minecraft.data.loot.packs.VanillaChestLoot`;
`net.minecraft.data.loot.packs.VanillaGiftLoot`;
`net.minecraft.data.recipes.packs.VanillaRecipeProvider`;
`net.minecraft.data.advancements.packs.VanillaHusbandryAdvancements`;
`reports/registries.json#minecraft:{item,entity_type,recipe,loot_table,advancement,villager_trade,trade_set,trial_spawner_config}`;
`reports/minecraft/components/item/{chicken,cooked_chicken}.json`;
`data/minecraft/tags/item/{meat,wolf_food}.json`;
`data/minecraft/loot_table/{entities/chicken,gameplay/cat_morning_gift,gameplay/hero_of_the_village/butcher_gift,spawners/trial_chamber/consumables}.json`;
`data/minecraft/recipe/cooked_chicken*.json`;
`data/minecraft/advancement/{recipes/food/cooked_chicken*,husbandry/balanced_diet}.json`;
`data/minecraft/{villager_trade/butcher/{1/chicken_emerald,2/emerald_cooked_chicken},tags/villager_trade/butcher/level_{1,2},trade_set/butcher/level_{1,2}}.json`;
`data/minecraft/trial_spawner/trial_chamber/**/normal.json`;
`assets/minecraft/{items,models/item,textures/item}/{chicken,cooked_chicken}.*`;
`ITM-FURNACE-001`; `ITM-CAMPFIRE-001`; `ITM-LOOT-001`;
`ITM-ADVANCEMENT-001`; `ITM-HUNGER-001`; `ENT-EFFECT-001`;
`ENT-DEATH-001`; `ENT-ENTITY-DROPS-001`; `MOB-AI-001`; `MOB-BREED-001`;
`MOB-RAID-001`; `BLK-TRIAL-SPAWNER-001`;
`WGEN-JIGSAW-TRIAL-CHAMBERS-001`; `CLI-UI-001`; `CLI-EFFECT-001`;
`EXP-ITM-064`.

**Test vectors:**

Exercise default, food/consumable-removed and arbitrary component-patched forms through hunger,
abilities, interrupted/completed use, effect-roll/merge endpoints, containers and anvil. Kill
adult/baby Chickens across death rules, fire, direct attacker, `smelts_loot`, live recipe and
Looting endpoints while tracing Feather-first pool order and the named cursor.

Cook through Furnace, Smoker and every Campfire slot across recipe/time/output/reload/unlock/XP
boundaries. Generate Cat gifts through owner-sleep/chance/teleport/table state, all normal Trial
config/table/player branches, both Butcher candidate sets and full offer lifecycles, and every
hero-gift branch. Feed both identities to every Wolf state; reload every data/resource domain,
persist/reload/synchronize and verify raw IDs, names, generated models/textures and exact tab
neighbors.
