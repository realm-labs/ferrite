# Items mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ITM-COD-001` — Raw and Cooked Cod join aquatic death/fishing loot, cooking, Fisherman records and five mob-food paths

**Parent:** `PLY-005`, `PLY-006`, `PLY-INPUT-001`, `PLY-INTERACT-001`,
`ITM-001`, `ITM-002`, `ITM-003`, `ITM-004`, `ITM-005`, `ITM-006`,
`ITM-007`, `ITM-USE-001`, `ITM-CONTAINER-001`, `ITM-RECIPE-001`,
`ITM-CRAFT-001`, `ITM-FURNACE-001`, `ITM-CAMPFIRE-001`, `ITM-LOOT-001`,
`ITM-ADVANCEMENT-001`, `ITM-HUNGER-001`, `ITM-ENCHANT-001`,
`ITM-ANVIL-001`, `ITM-PRISMARINE-MATERIAL-001`, `ENT-001`, `ENT-DEATH-001`,
`ENT-ENTITY-DROPS-001`, `ENT-PROJECTILE-001`, `MOB-AI-001`, `MOB-BREED-001`,
`MOB-RAID-001`, `WGEN-STRUCTURE-BURIED-001`, `WGEN-JIGSAW-VILLAGES-001`,
`CLI-001`, `CLI-006`, `CLI-UI-001`, `CLI-EFFECT-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked registrations/components, direct and recursive item-tag closure,
Cod/Dolphin/Guardian/Elder-Guardian/Polar-Bear death tables, both fishing tables, two chest
tables, three cooking recipes, Fisherman gift and trade resources, Cat/Ocelot/Dolphin/Wolf/
Nautilus consumers, advancements and direct client resources determine every Cod-item-specific
branch. Generic use, death, loot, fishing, machine, structure, merchant, mob, progression, stack
and client algorithms remain with the cited owners.

**Applies when:**

A `cod` or `cooked_cod` stack is emitted by entity death, fishing, chest or gift loot; eaten;
cooked; used in either Fisherman offer direction; offered to a Cat, Ocelot, Dolphin, Wolf or
Nautilus; moved, renamed, persisted, synchronized or rendered before and after component, tag,
recipe, loot, trade, advancement or resource reload.

**Authoritative state:**

| Item | Raw ID | Food | Direct item tags |
|---|---:|---|---|
| `minecraft:cod` | `1086` | nutrition `2`, saturation `0.4` | `cat_food`, `fishes`, `ocelot_food`, `wolf_food` |
| `minecraft:cooked_cod` | `1090` | nutrition `5`, saturation `6.0` | `fishes`, `wolf_food` |

Both are common nondamageable plain `Item` instances with maximum stack `64`, the ordinary
empty `32`-tick eat consumable and no consume-effect entries. Their remaining components are
the common empty modifiers/enchantments/lore, item-break sound, translated name, direct
item-model key, repair cost, swing animation, tooltip display and use effects.

`nautilus_food` recursively includes the complete `fishes` tag, so both identities are Nautilus
food and temptations. Only Raw Cod is Cat/Ocelot food. Neither belongs to
`nautilus_taming_items`; both are explicitly Wolf food rather than reaching it through `fishes`.

**Transition and ordering:**

Player consumption and progression:

In-air use enters generic consumption only below full hunger or when ability permits full-hunger
eating. Interruption or live-hand/component replacement before completion commits nothing.
Successful server completion emits eat effects, awards the statistic, triggers `consume_item`
against the live pre-shrink stack, applies food, runs the empty effect list, emits `EAT` and
shrinks one unless materials are infinite.

Default Raw Cod adds `2` food and `0.4` saturation; Cooked Cod adds `5` and `6.0`, subject to
generic clamps. They spend no consume-effect RNG and have no remainder. Raw and Cooked Cod are
independent requirements of telemetry-enabled `husbandry/balanced_diet`; all `40` foods award
`100` experience.

Cod and other aquatic death acquisition:

Every admitted `entities/cod` table first emits exactly one default Raw Cod. Fire or a direct
attacker whose main hand has `smelts_loot` then resolves the live Raw-Cod smelting recipe and
converts that stack before emission. Missing recipe leaves it raw; both OR terms cannot
double-smelt. A later independent pool draws `0.05` and emits one Bone Meal on success. Both
pools share sequence `minecraft:entities/cod`.

`entities/dolphin` creates Raw Cod, replaces its base count with uniform integer `0..1`, applies
Looting increase and only then performs the same conditional live smelt. With living-attacker
Looting level `L>0`, the bonus adds `round(L*U)` for one fresh float; it can revive base zero.
The table uses `minecraft:entities/dolphin`.

The Guardian/Elder-Guardian one-roll material pool follows the unconditional shard pool owned by
`ITM-PRISMARINE-MATERIAL-001`. Guardian chooses Cod/crystals/empty with weights `2/2/1`, giving
Cod probability `2/5`; Elder Guardian uses `3/2/1`, giving `1/2`. Selected Cod starts at one,
receives `round(L*U)`, then conditionally smelts.

Each Guardian table also has a later killed-by-player rare-fish pool. Its admission chance is
`0.025` without Looting and `0.035 + 0.01*(L-1)` for `L>=1`. Once admitted,
`gameplay/fishing/fish` chooses Cod with weight `60/100`; an outer conditional smelt then converts
the selected fish. Elder Guardian evaluates its killed-by-player Wet-Sponge pool before this
rare-fish pool. The outer tables use `minecraft:entities/guardian` and
`minecraft:entities/elder_guardian`; the nested selection uses
`minecraft:gameplay/fishing/fish`.

`entities/polar_bear` chooses Cod versus Salmon with weights `3/1`. A Cod selection conditionally
smelts its initial one-stack first, then replaces count with uniform integer `0..2`, then applies
the independent `round(L*U)` Looting increase. Thus the default Cod branch has probability `3/4`,
and a zero post-smelt base may still be revived. It uses `minecraft:entities/polar_bear`.

Death admission, player credit, attacker/enchantment context, table invocation and stack
placement remain with the entity and loot owners. Salmon, shards, crystals, empty, sponge,
trim-template and Bone-Meal branches retain their own owners.

Fishing acquisition and retrieval:

The root `gameplay/fishing` table selects junk, treasure or fish. For loot-context luck `l`, its
effective integer weights are

`J = max(floor(10 - 2l), 0)`, `T = max(floor(5 + 2l), 0)` and
`F = max(floor(85 - l), 0)`.

Treasure is absent unless the hook is in open water. The nested fish table gives Cod weight
`60/100`, so when the denominator is positive the conditional Cod probability is

`F / (J + F + (open_water ? T : 0)) * 3/5`.

At `l=0` this is `51/100` in open water and `51/95` outside it. Root and nested tables use
separate sequences `minecraft:gameplay/fishing` and `minecraft:gameplay/fishing/fish`.

Retrieval first triggers `fishing_rod_hooked`; Raw Cod is one of four alternatives in
telemetry-enabled `husbandry/fishy_business`. It then inserts the item entity, creates one XP orb
of uniform value `1..6`, and increments `fish_caught` once because the result is in `fishes`.
The criterion therefore observes the generated Raw Cod before item, XP and statistic commits.
A data-reloaded fishing result of Cooked Cod still satisfies the tag-driven statistic but not the
identity-specific Fishy Business criterion.

Hook admission/open-water history, insertion, motion, XP, rod damage and removal remain with
`ENT-PROJECTILE-001` and `ITM-LOOT-001`. Merely obtaining Cod elsewhere triggers neither fishing
progress nor the statistic.

Chest and Fisherman-gift acquisition:

The fifth Buried-Treasure pool rolls exactly twice. Each roll chooses Cooked Cod or Cooked Salmon
at equal weight, then a Cod choice sets count to uniform integer `2..4`. Repeated selection is
with replacement under `minecraft:chests/buried_treasure`; preceding/following pools and chest
distribution remain `WGEN-STRUCTURE-BURIED-001` and generic loot work.

The Village Fisher chest makes uniform integer `1..5` rolls over weights
Emerald/Cod/Salmon/Water Bucket/Barrel/Wheat Seeds/Coal = `1/2/1/1/1/3/2`, total `11`. Each Cod
selection therefore has probability `2/11` and sets count to uniform integer `1..3` under
`minecraft:chests/village/village_fisher`.

An admitted adult Fisherman Hero gift chooses uniformly between one Raw Cod and one Raw Salmon.
Raw Cod probability is `1/2` under
`minecraft:gameplay/hero_of_the_village/fisherman_gift`. Initial eligible cooldown is `600`;
later cooldown is `600 + nextInt(6001)`, target range is five blocks, behavior lasts at most
`100` ticks and throws only after elapsed time exceeds `20`. Admission, navigation, throw and
cleanup remain with `MOB-RAID-001`.

Cooking and recipe progression:

Three exact recipes consume one Raw Cod and emit one default Cooked Cod:

| Recipe | Domain | Time | Recipe XP |
|---|---|---:|---:|
| `cooked_cod` | Furnace | `200` ticks | `0.35` |
| `cooked_cod_from_smoking` | Smoker | `100` ticks | `0.35` |
| `cooked_cod_from_campfire_cooking` | Campfire | `600` ticks | `0.35` |

Each no-display advancement has one OR requirement: Raw-Cod possession or matching recipe
knowledge grants that recipe. Furnace/Smoker accumulate `0.35`, and extraction owns fractional
XP and criteria. Campfire re-resolves its live recipe but awards neither recipe XP nor unlock.
Input patches do not propagate. Recipe reload changes all future machine results and every
death-table smelt described above.

Two Fisherman offer sets:

Fisherman level one contains four predicate-free records and selects two without replacement.
The Cod service offer therefore has inclusion probability `1/2` under
`minecraft:trade_set/fisherman/level_1`. It consumes six matching Raw Cod plus one Emerald and
gives six default Cooked Cod, with maximum uses `16`, default XP `1` and reputation discount
`0.05`.

Level two contains three predicate-free records and selects two, so the Raw-Cod purchase has
inclusion probability `2/3` under `minecraft:trade_set/fisherman/level_2`.
`fisherman/2/cod_emerald` consumes fifteen matching Raw Cod and gives one Emerald, with maximum
uses `16`, XP `10` and discount `0.05`.

Both accept ordinary component patches through empty cost predicates; the service offer does not
copy those patches to Cooked Cod. Neither has output modifiers or double-price enchantments, and
Trade Rebalance replaces neither record/tag. Candidate ordering, economy, exhaustion, restock
and merchant-menu synchronization remain with merchant owners.

Cat and Ocelot Raw-Cod paths:

An untamed Cat accepts Raw Cod regardless age, consumes one, draws `nextInt(3)` and tames/assigns
the player, orders sit and emits event `7` exactly on zero; failure emits event `6`. The attempt
marks persistence and plays the eating sound. For a tamed Cat, only its owner reaches the injured
heal shortcut: Raw Cod heals its live nutrition, default `2`, or fallback `1` when food is absent.
Other admitted baby/love feeding and the Cat temptation predicate use generic owners. Cooked Cod
is not `cat_food` and enters none of these branches.

An untrusting Ocelot takes the Raw-Cod trust attempt only when its tempt goal is absent or running,
player squared distance is strictly below `9`, and the item is live `ocelot_food`. It consumes
one and succeeds on `nextInt(3)==0`, setting trust and emitting event `41`; failure emits `40`.
When that precedence branch does not run, generic eligible baby-growth or adult-love feeding can
consume Raw Cod. Trust, breeding, temptation and persistence remain `MOB-BREED-001` work.

Dolphin, Wolf and Nautilus shared fish paths:

A Dolphin accepts either Cod identity through `fishes`, consumes one and plays its eating sound.
An age-unlocked baby advances ten percent of remaining whole seconds; otherwise it sets
`gotFish=true`, enabling later treasure-search AI. It never executes the held stack's consumable.

A tamed injured Wolf consumes either identity before growth/love, regardless feeder ownership,
and heals twice live nutrition: default Raw/Cooked healing is `4/10`; absent food falls back to
`2`. Other admitted states use generic age-unlocked baby growth or age-zero adult `600`-tick
love. Untamed adults may enter love but cannot mate until tame.

Every Nautilus interaction first marks persistence. Babies accept both identities through
recursive `nautilus_food` and use generic growth. An untamed adult instead tests
`nautilus_taming_items`; neither Cod identity belongs, so it does not consume or tame. For a
tamed adult, secondary use opens inventory first; otherwise an injured state consumes and heals
twice nutrition (`4/10`, fallback `1`), while a full-health eligible adult can consume for
`600`-tick love. Both identities are Nautilus temptations. These mob paths apply no player food
state and invoke no item consume effects.

**Persistence and reload boundary:**

Stacks persist identity, count and patches. They store no use/hunger, death/attacker/Looting,
hook/open-water/luck, chest/gift, recipe/machine, merchant, mob or advancement state.

Loot reload changes future entity, fishing, chest and gift evaluations. Recipe reload changes
future cooking and death smelting. Tag reload changes future Cat/Ocelot/Dolphin/Wolf/Nautilus
admission and fishing-stat tests. Trade reload changes future offers; advancement reload changes
listeners. Completed work is not replayed. Resource reload independently controls names/models/
textures and advancement presentation.

**Client and wire projection:**

Generic stack encoding projects raw IDs `1086/1090` plus patches. Locked English names are
`Raw Cod` and `Cooked Cod`; both are common with no forced glint or subtype tooltip.

Each direct item definition selects its same-named model and texture. Cooked Cod is an ordinary
generated model. Raw Cod is generated but additionally defines head display rotation
`[0,90,-60]`, translation `[-7,-4,-7]` and scale `[0.8,0.8,0.8]`.

They appear exactly once in Food & Drinks, ordered Raw Rabbit, Cooked Rabbit, Raw Cod, Cooked Cod,
Raw Salmon, Cooked Salmon. Raw Cod is also the display icon of `husbandry/complete_catalogue`;
the advancement's eleven Cat-variant criteria consume or inspect no Cod.

**Branches and aborts:**

Identity/count/components/five effective tags; use; five entity tables and smelting-function
orders; two fishing sequences/retrieval order; two chest tables; Fisherman gift/trade sets;
three cooking domains; Cat/Ocelot/Dolphin/Wolf/Nautilus precedence; progression, persistence,
reload, wire, models and tab.

**Constants and randomness:**

Raw IDs `1086/1090`; max `64`; food `2/0.4`, `5/6.0`; eat `32`; Cod Bone Meal `0.05`;
Dolphin `0..1+round(LU)`; Guardian/Elder normal Cod `2/5`, `1/2` plus
`1+round(LU)`; rare admission `0.025` or `0.035+0.01(L-1)`, Cod `3/5`; Polar Bear Cod `3/4`,
count `0..2+round(LU)`; fishing formula and Cod `3/5`, XP `1..6`; Buried Treasure two rolls,
Cod `1/2`, count `2..4`; Village Fisher `1..5` rolls, Cod `2/11`, count `1..3`; gift `1/2`;
cooking `200/100/600`, XP `0.35`; trades inclusion `1/2`, `2/3`, costs/results `6+1→6`,
`15→1`, uses/XP `16/1`, `16/10`; Cat/Ocelot tame/trust `1/3`; Wolf/Nautilus heal `4/10`.

**Side effects:**

Food/use; entity/fishing/chest/gift loot and named cursors; fishing criterion/item/XP/stat;
machine progress/output/unlock/XP; merchant offers/costs/results/economy; Cat tame/heal,
Ocelot trust, Dolphin age/`gotFish`, Wolf health/age/love, Nautilus persistence/health/age/love/
temptation; persistence, wire and client projection.

**Gates:**

Food/hunger/use; entity death and fire/attacker/smelts-loot/live recipe/Looting/player-credit;
hook/open-water/luck; chest/gift admission; machine recipe; Fisherman profession/level/set/offer;
live mob tags and subtype state; progression listeners; registry/decode/client bootstrap.

**State read/written:**

Reads stack/components/tags, player use/progression/statistics, entity death contexts, hook/loot/
structure/gift state, machine recipes, Villager offers, five mob states and client resources.
Writes only the consumption, loot, fishing, progression, cooking, trade, mob and projection state
listed above.

**Failure behavior:**

Unadmitted use commits nothing. Missing death smelting recipe leaves Raw Cod; zero count emits no
stack. Unselected loot entries emit alternatives. Failed fishing insertion does not suppress
later XP/stat work. Invalid machine or offer commits nothing. Cooked Cod cannot tame/feed Cats or
Ocelots; Cod cannot tame an adult Nautilus. Ineligible mob states do not consume except the
explicit Dolphin adult and Cat/Ocelot tame/trust branches.

**Boundary cases and quirks:**

Smelting occurs before count replacement for Polar Bear, after base/Looting count for Dolphin,
and after count bonus for Guardian normal pools; Cod itself smelts before its unrelated Bone-Meal
chance. Guardian rare Cod is a second independent source from the same death. Fishy Business
precedes item/XP/stat creation. The level-one service is a real two-cost conversion offer, not a
recipe. Raw Cod alone tames Cats and wins Ocelot trust; both Cod identities feed but cannot tame
an adult Nautilus. Raw Cod's generated model has a dedicated head transform.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-DATA-001`; `OFF-REPORT-001`;
`net.minecraft.world.item.Items`;
`net.minecraft.world.entity.projectile.FishingHook#retrieve`;
`net.minecraft.world.level.storage.loot.functions.SmeltItemFunction#run`;
`net.minecraft.world.entity.animal.feline.Cat#mobInteract`;
`net.minecraft.world.entity.animal.feline.Cat#isFood`;
`net.minecraft.world.entity.animal.feline.Ocelot#mobInteract`;
`net.minecraft.world.entity.animal.feline.Ocelot#isFood`;
`net.minecraft.world.entity.animal.dolphin.Dolphin#mobInteract`;
`net.minecraft.world.entity.animal.wolf.Wolf#mobInteract`;
`net.minecraft.world.entity.animal.nautilus.AbstractNautilus#interact`;
`net.minecraft.world.entity.animal.nautilus.AbstractNautilus#mobInteract`;
`net.minecraft.world.entity.animal.nautilus.AbstractNautilus#isFood`;
`net.minecraft.world.entity.animal.nautilus.NautilusAi#getTemptations`;
`net.minecraft.world.entity.TamableAnimal#feed`;
`net.minecraft.world.entity.npc.villager.AbstractVillager#addOffersFromTradeSet`;
`net.minecraft.world.item.trading.VillagerTrade#getOffer`;
`net.minecraft.world.item.CreativeModeTabs`;
`net.minecraft.data.loot.packs.VanillaEntityLoot`;
`net.minecraft.data.loot.packs.VanillaFishingLoot`;
`net.minecraft.data.loot.packs.VanillaChestLoot`;
`net.minecraft.data.loot.packs.VanillaGiftLoot`;
`net.minecraft.data.recipes.packs.VanillaRecipeProvider`;
`reports/registries.json#minecraft:{item,entity_type,recipe,loot_table,advancement,villager_trade,trade_set}`;
`reports/minecraft/components/item/{cod,cooked_cod}.json`;
`data/minecraft/tags/item/{cat_food,ocelot_food,fishes,wolf_food,nautilus_food,nautilus_taming_items}.json`;
`data/minecraft/loot_table/{entities/{cod,dolphin,guardian,elder_guardian,polar_bear},gameplay/fishing,gameplay/fishing/fish,gameplay/hero_of_the_village/fisherman_gift,chests/{buried_treasure,village/village_fisher}}.json`;
`data/minecraft/recipe/cooked_cod*.json`;
`data/minecraft/advancement/{recipes/food/cooked_cod*,husbandry/{balanced_diet,fishy_business,complete_catalogue}}.json`;
`data/minecraft/{villager_trade/fisherman/{1/raw_cod_and_emerald_cooked_cod,2/cod_emerald},tags/villager_trade/fisherman/{level_1,level_2},trade_set/fisherman/{level_1,level_2}}.json`;
`assets/minecraft/{items,models/item,textures/item}/{cod,cooked_cod}.*`;
`ITM-FURNACE-001`; `ITM-CAMPFIRE-001`; `ITM-LOOT-001`;
`ITM-ADVANCEMENT-001`; `ITM-HUNGER-001`; `ITM-PRISMARINE-MATERIAL-001`;
`ENT-DEATH-001`; `ENT-ENTITY-DROPS-001`; `ENT-PROJECTILE-001`;
`MOB-AI-001`; `MOB-BREED-001`; `MOB-RAID-001`; `CLI-UI-001`;
`CLI-EFFECT-001`; `EXP-ITM-068`.

**Test vectors:**

Exercise default/removed/patched stacks through use. Kill Cod, Dolphin, Guardian, Elder Guardian
and Polar Bear across every fire/attacker/Looting/player-credit/live-recipe state while tracing
function order, all neighboring pools and named cursors.

Retrieve fishing loot across open-water/luck/weight and insertion boundaries; materialize every
Cod-relevant Buried/Village-Fisher roll and Fisherman gift. Cook in all domains and generate,
transact, exhaust and restock both Fisherman records across complete candidate orders.

Offer both identities to every Cat/Ocelot/Dolphin/Wolf/Nautilus age/tame/trust/owner/health/love/
secondary state. Reload all domains, persist/synchronize and verify IDs, names, raw head transform,
textures, advancement icon and exact Food-tab neighborhood.
