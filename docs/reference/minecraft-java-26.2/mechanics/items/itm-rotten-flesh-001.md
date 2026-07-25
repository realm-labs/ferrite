# Items mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ITM-ROTTEN-FLESH-001` — Rotten Flesh joins Hunger-bearing consumption, nine undead drops, eight chest families, fishing, cat gifts, Cleric buying and Wolf food

**Parent:** `PLY-005`, `PLY-006`, `PLY-INPUT-001`, `PLY-INTERACT-001`,
`ITM-001`, `ITM-002`, `ITM-003`, `ITM-004`, `ITM-005`, `ITM-006`,
`ITM-007`, `ITM-USE-001`, `ITM-CONTAINER-001`, `ITM-RECIPE-001`,
`ITM-CRAFT-001`, `ITM-LOOT-001`, `ITM-ADVANCEMENT-001`, `ITM-HUNGER-001`,
`ITM-ENCHANT-001`, `ITM-ANVIL-001`, `ENT-001`, `ENT-DEATH-001`,
`ENT-ENTITY-DROPS-001`, `ENT-EFFECT-001`, `MOB-AI-001`, `MOB-BREED-001`,
`WGEN-PIPELINE-001`, `WGEN-DIMENSION-001`,
`WGEN-STRUCTURE-DESERT-PYRAMID-001`, `WGEN-STRUCTURE-IGLOO-001`,
`WGEN-STRUCTURE-JUNGLE-TEMPLE-001`, `WGEN-STRUCTURE-SHIPWRECK-001`,
`WGEN-STRUCTURE-OCEAN-RUIN-001`, `WGEN-JIGSAW-VILLAGES-001`,
`WGEN-STRUCTURE-WOODLAND-MANSION-001`, `CLI-001`, `CLI-006`, `CLI-UI-001`,
`CLI-EFFECT-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked components, nine entity tables, eight chest-table families, fishing
junk, cat-morning-gift, Cleric trade, item tags, advancement and direct client resources determine
every Rotten-Flesh-specific branch. Generic use/effect merging, death, loot, fishing, structure,
merchant, animal, persistence and client algorithms remain with the cited owners.

**Applies when:**

A `rotten_flesh` stack is eaten, emitted by an applicable entity/chest/fishing/cat table, sold to
a Cleric, fed to a Wolf, moved, renamed, persisted, synchronized or rendered before and after
component, tag, loot, trade, advancement, timeline or resource reload.

**Authoritative state:**

`minecraft:rotten_flesh` is raw item ID `1143`. It is a common nondamageable plain `Item` with
maximum stack `64`, food nutrition `4` and saturation `0.8`. Its ordinary `32`-tick eat consumable
contains one `apply_effects` entry: a fresh Hunger effect of duration `600`, amplifier `0`, ambient
false, particles true and icon true, applied with probability Java float `0.8f`
(`0.800000011920929` as a real value). It has no use remainder.

Its other default components are the common empty modifiers/enchantments/lore, item-break sound,
translated name, direct item-model key, repair cost, swing animation, tooltip display and use
effects. Rotten Flesh directly belongs only to `#minecraft:meat`.
`#minecraft:wolf_food` includes that whole tag plus seven fish identities and Rabbit Stew, so
Rotten Flesh reaches Wolf behavior through nested tag closure.

**Transition and ordering:**

Player consumption and Hunger effect:

In-air use enters generic consumption only below full hunger or when an ability permits
full-hunger eating. Ordinary full-hunger use returns `FAIL`. Removing only food leaves the
consumable and admits use even at full hunger; removing only consumable makes plain in-air use
pass. Interruption, release or live-hand/component replacement before completion commits no
finish work.

Successful server completion emits final eat effects, awards the item-used statistic, triggers
`consume_item` against the live pre-shrink stack, applies food, visits consume-effect entries in
order, emits `EAT`, then shrinks one unless materials are infinite. Default food adds `4` and
`0.8`, subject to generic hunger/saturation clamps.

The Rotten-Flesh effect consumes exactly one entity RNG `nextFloat()`. It constructs and attempts
to add a fresh Hunger-0 instance for `600` ticks exactly when that draw is strictly below `0.8f`.
An existing stronger/equivalent effect can make effect insertion reject the fresh instance; the
draw and earlier food/statistic/criterion work are not rolled back. A rejected probability draw
also leaves the preceding work committed. Patched food, consumable and effect entries are live.

Rotten Flesh is one independent requirement of telemetry-enabled `husbandry/balanced_diet`; all
`40` listed foods award `100` experience. The advancement tests item identity at consumption, not
whether Hunger was successfully added.

Entity-death acquisition:

Each applicable entity table puts Rotten Flesh in pool zero with one roll and evaluates that work
before any later pool. Every count listed below emits a default stack and then, when a living
attacker supplies Looting level `L>0`, adds `round(L*U)` for one uniform `U` in `[0,1)`; without
that context no bonus draw occurs. A zero base can therefore be revived by Looting.

| Entity table | Base count | Additional gate | Random sequence |
|---|---:|---|---|
| `entities/camel_husk` | `2..3` | none inside the admitted generic death table | `minecraft:entities/camel_husk` |
| `entities/drowned` | `0..2` | none inside the admitted generic death table | `minecraft:entities/drowned` |
| `entities/husk` | `0..2` | none inside the admitted generic death table | `minecraft:entities/husk` |
| `entities/zoglin` | `1..3` | none inside the admitted generic death table | `minecraft:entities/zoglin` |
| `entities/zombie_horse` | `2..3` | none inside the admitted generic death table | `minecraft:entities/zombie_horse` |
| `entities/zombie_nautilus` | `0..3` | pool requires `killed_by_player` | `minecraft:entities/zombie_nautilus` |
| `entities/zombie_villager` | `0..2` | none inside the admitted generic death table | `minecraft:entities/zombie_villager` |
| `entities/zombie` | `0..2` | none inside the admitted generic death table | `minecraft:entities/zombie` |
| `entities/zombified_piglin` | `0..1` | none inside the admitted generic death table | `minecraft:entities/zombified_piglin` |

For Zombie Nautilus, failed `killed_by_player` skips the whole pool before count or Looting draws.
Entity death admission, table lookup, attacker/enchantment context, item-entity creation and later
pools remain with `ENT-DEATH-001`, `ENT-ENTITY-DROPS-001` and `ITM-LOOT-001`. None of these Rotten
Flesh entries invokes `furnace_smelt`.

Structure-chest acquisition:

All chest rows select with replacement and use their table's named random sequence. Omitted count
means one default Rotten Flesh; uniform count functions run only after the entry is selected.

| Chest pool | Rolls | Rotten weight / eligible total | Count |
|---|---:|---:|---:|
| `chests/desert_pyramid`, pool `0` | `2..4` | baseline `25/247`; Trade Rebalance `25/237` | `3..7` |
| `chests/desert_pyramid`, pool `1` | `4` | `10/50 = 1/5` in either pack state | `1..8` |
| `chests/igloo_chest`, pool `0` | `2..8` | `10/63` | `1` |
| `chests/jungle_temple`, pool `0` | `2..6` | `16/89`; Trade Rebalance preserves it | `3..7` |
| `chests/shipwreck_supply`, pool `0` | `3..10` | `5/84` | `5..24` |
| `chests/simple_dungeon`, pool `2` | `3` | `10/40 = 1/4` | `1..8` |
| `chests/underwater_ruin_small`, pool `0` | `2..8` | `5/30 = 1/6` | `1` |
| `chests/village/village_temple`, pool `0` | `3..8` | `7/19` | `1..4` |
| `chests/woodland_mansion`, pool `2` | `3` | `10/40 = 1/4` | `1..8` |

The two Desert-Pyramid rows are independent pools and may both emit Rotten Flesh. Trade Rebalance
changes the first Desert denominator by replacing other alternatives; it does not replace either
Rotten-Flesh entry and leaves the Jungle row unchanged. Structure placement, container creation,
lazy loot seed, table opening, output shuffle and insertion remain with the corresponding
structure/worldgen and loot owners.

Fishing acquisition and retrieval:

The root `gameplay/fishing` table makes one weighted selection among junk, treasure and fish. For
loot-context luck `l`, effective integer weights are

`J = max(floor(10 - 2l), 0)`, `T = max(floor(5 + 2l), 0)` and
`F = max(floor(85 - l), 0)`.

Treasure is absent unless the hook's `in_open_water` predicate passes. When junk is selected,
`gameplay/fishing/junk` makes one roll. Rotten Flesh has weight `10`; eligible total weight is
`100` outside Jungle, Sparse Jungle and Bamboo Jungle, or `110` inside them because the
conditional weight-`10` Bamboo entry joins. Conditional junk probability is therefore `1/10`
outside those biomes and `1/11` inside, and the selected entry emits one default stack.

When the root denominator is positive, full conditional probability is

`J / (J + F + (open_water ? T : 0)) * (jungle ? 1/11 : 1/10)`.

At `l=0`, this is `1/100` in open non-jungle water, `1/110` in open jungle water, `1/95`
outside open non-jungle water and `2/209` outside open jungle water. Root and nested work use
distinct random sequences `minecraft:gameplay/fishing` and `minecraft:gameplay/fishing/junk`.

Retrieval still triggers `fishing_rod_hooked`, creates and attempts to insert the item entity,
emits one XP orb of uniform value `1..6`, damages the rod and removes the hook. Rotten Flesh is
not in `fishes`, so this catch does not increment `fish_caught` and does not satisfy
`husbandry/fishy_business`. Hook state, motion, insertion, criterion, XP and rod transaction
remain with the fishing and loot owners.

Cat morning-gift acquisition:

A tame cat's relax-on-owner goal can start only while it is not ordered to sit, its player owner
is sleeping within squared distance `100`, the owner occupies a bed, and no other nearby cat
already occupies the selected relaxation space. Cat tame/owner state and scheduling remain with
`MOB-BREED-001` and `MOB-AI-001`.

When the goal stops, it first clears lying state. Only owner sleep timer at least `100` reaches
one level RNG float and invokes the gift path when it is strictly below live
`minecraft:gameplay/cat_waking_up_gift_chance`. The attribute defaults `0`; the locked circular
`day` timeline supplies constant keyframes tick `362` value `0` and tick `23667` value `0.7`, so
the normal wake marker at tick `0` resolves `0.7`. Timeline resolution remains
`WGEN-DIMENSION-001`.

After chance success, cat RNG chooses attempted teleport offsets `nextInt(11)-5`,
`nextInt(5)-2`, `nextInt(11)-5` around the leash holder or cat. The ignored teleport result is
followed by one gift-table roll: Rabbit Hide, Rabbit Foot, Chicken, Feather, Rotten Flesh and
String each have weight `10`, while Phantom Membrane has weight `2`, total `62`. Conditioned on
table evaluation, Rotten Flesh has probability `10/62 = 5/31` and count one. With live chance
`g`, a qualified stop emits it with probability `5g/31`; normal locked `g=0.7` gives `7/62`.

The table uses sequence `minecraft:gameplay/cat_morning_gift`, distinct from level/cat RNG. Its
callback inserts an item entity one horizontal unit along body rotation from the cat's block
position; insertion failure is ignored.

Cleric purchase:

Cleric level one has exactly two predicate-free records and its trade set requests amount `2`
without duplicates, so `cleric/1/rotten_flesh_emerald` is guaranteed once in every default fresh
level-one set. It consumes `32` Rotten Flesh matching an empty component predicate and gives one
default Emerald, with maximum uses `16`, villager XP `2` and reputation discount `0.05`.

The input permits arbitrary Rotten-Flesh component patches because the predicate constrains no
components. There is no second cost, predicate modifier, output modifier or double-price
enchantment. Trade Rebalance does not replace the record. Offer creation/economy, demand,
reputation, restocking and menu commit remain merchant-owned.

Wolf food, healing, growth and love:

Live `wolf_food` closure admits Rotten Flesh. A tamed injured Wolf takes its specialized heal
branch first without checking feeder ownership. It consumes one through the player-aware helper,
heals twice live food nutrition and plays the eating sound; default Rotten Flesh offers `8` HP,
clamped to maximum. Patched nutrition changes the amount. If food is removed but tag membership
remains, the Wolf uses its distinct fallback `2` HP.

Every other Wolf state falls through to generic Animal feeding. An age-unlocked baby consumes one
and advances by ten percent of its remaining whole seconds. An age-zero adult not already in love
consumes one and enters `600`-tick love even when untamed; untamed Wolves still cannot mate until
tame. Already-loving adults and age-locked babies consume nothing unless a later owned branch
succeeds. No mob path runs the player's consumable, rolls the Hunger effect, or changes Wolf
hunger/saturation.

Removing Rotten Flesh from `meat`, removing nested `meat` from `wolf_food`, or otherwise replacing
tag closure blocks these default branches. Wolf health/age/love, mating, offspring, persistence
and AI remain `MOB-BREED-001` and `MOB-AI-001`.

**Persistence and reload boundary:**

Stacks persist identity, count and arbitrary patches. They store no active-use progress, effect
merge/RNG state, death/loot cursor, fishing hook, cat goal, merchant offer, Wolf state or
advancement listener; those values persist with their owners.

Component defaults are registration-built. Loot reload changes future entity, chest, fishing and
gift evaluation. Tag reload changes future Wolf admission. Trade reload changes future fresh
Cleric offers without rewriting existing offers; advancement reload changes listeners. Timeline
or dimension reload changes future cat gift chance. Completed work is not replayed. Resource
reload independently controls name, model and texture.

**Client and wire projection:**

Generic stack encoding projects raw ID `1143` plus patches. The locked English name is `Rotten
Flesh`; it is common with no forced glint or subtype tooltip. Its direct item definition selects
the ordinary generated `item/rotten_flesh` model and same-named texture.

Rotten Flesh appears exactly once in Food & Drinks, ordered Pumpkin Pie, Rotten Flesh, Spider Eye,
Mushroom Stew.

**Branches and aborts:**

Identity/components/tags; player hunger/use/effect; nine entity tables; nine rows across eight
chest-table families; fishing; cat gift; Cleric offer; Wolf heal/growth/love; Balanced Diet;
persistence/reload/wire; language/model/tab.

**Constants and randomness:**

Raw ID `1143`; max `64`; food `4/0.8`; eat `32`; Hunger `600/0`, probability `0.8f`; entity bases
`2..3`, `0..2`, `0..2`, `1..3`, `2..3`, `0..3`, `0..2`, `0..2`, `0..1` plus
`round(L*U)`; fishing probabilities above; cat selection `5/31`, normal combined `7/62`; Cleric
`32→1`, uses/XP `16/2`; Wolf heal `8` default and love `600`; Balanced Diet `40/100`.

**Side effects:**

Food/statistic/criterion/event/shrink and possible Hunger merge; entity/chest/fishing/cat output
and named cursors; fishing XP/rod/hook; Cleric offer/economy; Wolf health/age/love/sound/consume;
advancement, persistence/wire and direct client projection.

**Gates:**

Food/hunger and uninterrupted use; effect probability/merge; generic entity-death and Nautilus
player-kill admission; active loot tables/contexts; fishing junk/open-water/biome/luck; qualified
cat goal/sleep/chance/table; level-one Cleric offer validity; live Wolf-food closure and Wolf
state; registry/decode; client language/model/tab bootstrap.

**State read/written:**

Reads stack/components/tags, player hunger/use/progression, entity death/attacker/enchantment,
structure/container/loot, hook, cat/timeline, merchant and Wolf state, persistence and client
resources. Writes only the consumption/effect, loot, fishing, cat, trade, Wolf, progression,
stack and projection state listed above.

**Failure behavior:**

Full-hunger ordinary use fails and interruption commits nothing. Probability or effect-merge
rejection does not roll back consumption. Unadmitted death/chest/fishing/gift rows emit nothing or
alternatives; failed Zombie-Nautilus player-kill gate spends no pool work. Invalid/exhausted
Cleric offers commit nothing. Ineligible Wolf states consume nothing unless a later owner branch
succeeds. Reloaded data changes future evaluation only; missing client resources grant no
authority.

**Boundary cases and quirks:**

The comparison uses widened Java `0.8f`, not exact real `0.8`. Effect insertion can fail after the
probability draw and after food/progression commits. Zero-base death counts can become positive
through Looting. Desert Pyramid has two independent Rotten-Flesh pools. Closed-water fishing
removes treasure and therefore increases junk share. Cat teleport failure does not cancel a
successful gift. Arbitrarily patched Rotten Flesh still satisfies the Cleric's empty component
predicate. Wolf feeding never invokes the player's Hunger-bearing consumable.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-DATA-001`; `OFF-REPORT-001`;
`net.minecraft.world.item.Items`; `net.minecraft.world.item.component.Consumables`;
`net.minecraft.world.item.consume_effects.ApplyStatusEffectsConsumeEffect#apply`;
`net.minecraft.world.item.component.Consumable#startConsuming`;
`net.minecraft.world.item.component.Consumable#onConsume`;
`net.minecraft.world.food.FoodProperties#onConsume`;
`net.minecraft.world.food.FoodData#eat`;
`net.minecraft.world.entity.projectile.FishingHook#retrieve`;
`net.minecraft.world.entity.animal.feline.Cat$CatRelaxOnOwnerGoal#stop`;
`net.minecraft.world.entity.animal.feline.Cat$CatRelaxOnOwnerGoal#giveMorningGift`;
`net.minecraft.world.entity.LivingEntity#dropFromGiftLootTable`;
`net.minecraft.world.entity.animal.wolf.Wolf#mobInteract`;
`net.minecraft.world.entity.animal.wolf.Wolf#isFood`;
`net.minecraft.world.entity.TamableAnimal#feed`;
`net.minecraft.world.entity.animal.Animal#mobInteract`;
`net.minecraft.world.entity.npc.villager.AbstractVillager#addOffersFromTradeSet`;
`net.minecraft.world.entity.npc.villager.AbstractVillager#addOffersFromItemListingsWithoutDuplicates`;
`net.minecraft.world.item.trading.VillagerTrade#getOffer`;
`net.minecraft.world.item.trading.TradeSet#calculateNumberOfTrades`;
`net.minecraft.world.item.CreativeModeTabs`;
`net.minecraft.data.tags.VanillaItemTagsProvider`;
`net.minecraft.data.tags.VillagerTradesTagsProvider`;
`net.minecraft.data.loot.packs.VanillaEntityLoot`;
`net.minecraft.data.loot.packs.VanillaChestLoot`;
`net.minecraft.data.loot.packs.TradeRebalanceChestLoot`;
`net.minecraft.data.loot.packs.VanillaFishingLoot`;
`net.minecraft.data.loot.packs.VanillaGiftLoot`;
`net.minecraft.data.advancements.packs.VanillaHusbandryAdvancements`;
`reports/registries.json#minecraft:{item,entity_type,loot_table,advancement,villager_trade,trade_set}`;
`reports/minecraft/components/item/rotten_flesh.json`;
`data/minecraft/tags/item/{meat,wolf_food}.json`;
`data/minecraft/loot_table/entities/{camel_husk,drowned,husk,zoglin,zombie_horse,zombie_nautilus,zombie_villager,zombie,zombified_piglin}.json`;
`data/minecraft/loot_table/chests/{desert_pyramid,igloo_chest,jungle_temple,shipwreck_supply,simple_dungeon,underwater_ruin_small,village/village_temple,woodland_mansion}.json`;
`data/minecraft/loot_table/gameplay/{fishing,fishing/junk,cat_morning_gift}.json`;
`data/minecraft/{villager_trade/cleric/1/rotten_flesh_emerald,tags/villager_trade/cleric/level_1,trade_set/cleric/level_1}.json`;
`data/minecraft/advancement/husbandry/balanced_diet.json`;
`data/minecraft/timeline/day.json`;
`assets/minecraft/{items,models/item,textures/item}/rotten_flesh.*`;
`ITM-LOOT-001`; `ITM-ADVANCEMENT-001`; `ITM-HUNGER-001`; `ITM-ENCHANT-001`;
`ENT-DEATH-001`; `ENT-ENTITY-DROPS-001`; `ENT-EFFECT-001`; `MOB-AI-001`;
`MOB-BREED-001`; `WGEN-PIPELINE-001`; `WGEN-DIMENSION-001`;
`WGEN-STRUCTURE-DESERT-PYRAMID-001`; `WGEN-STRUCTURE-IGLOO-001`;
`WGEN-STRUCTURE-JUNGLE-TEMPLE-001`; `WGEN-STRUCTURE-SHIPWRECK-001`;
`WGEN-STRUCTURE-OCEAN-RUIN-001`; `WGEN-JIGSAW-VILLAGES-001`;
`WGEN-STRUCTURE-WOODLAND-MANSION-001`; `CLI-UI-001`; `CLI-EFFECT-001`;
`EXP-ITM-074`.

**Test vectors:**

Exercise default, food/consumable/effect-removed and arbitrary component-patched Rotten Flesh
through full/lower hunger, finite/infinite materials, interrupted/completed use, probability
boundary draws and every existing-effect merge relation; verify Balanced Diet ordering.

Materialize every base/Looting boundary in all nine entity tables including failed/passed
Zombie-Nautilus admission. Force every chest row/roll/count in baseline and Trade Rebalance.
Enumerate fishing luck/open-water/jungle boundaries with separate root/nested cursors and full
retrieval side effects.

Exercise cat goal admission, sleep/chance/teleport/table boundaries; every level-one Cleric offer
lifecycle with patched inputs; every Wolf tame/health/age/love/tag/food state. Reload all data
domains, persist/synchronize stacks, then verify raw ID, name, ordinary model/texture and exact
Food-tab neighborhood.
