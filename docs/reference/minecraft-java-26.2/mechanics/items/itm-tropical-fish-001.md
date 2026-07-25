# Items mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ITM-TROPICAL-FISH-001` — Tropical Fish join invariant fish death, Guardian and fishing loot, a Fisherman sink and three mob-food paths

**Parent:** `PLY-005`, `PLY-006`, `PLY-INPUT-001`, `PLY-INTERACT-001`,
`ITM-001`, `ITM-002`, `ITM-003`, `ITM-004`, `ITM-005`, `ITM-006`,
`ITM-007`, `ITM-USE-001`, `ITM-CONTAINER-001`, `ITM-RECIPE-001`,
`ITM-CRAFT-001`, `ITM-LOOT-001`, `ITM-ADVANCEMENT-001`, `ITM-HUNGER-001`,
`ITM-ENCHANT-001`, `ITM-ANVIL-001`, `ITM-PRISMARINE-MATERIAL-001`,
`ITM-COD-001`, `ITM-SALMON-001`, `ITM-PUFFERFISH-001`, `ITM-MOB-BUCKET-001`,
`ENT-001`, `ENT-005`, `ENT-DEATH-001`, `ENT-ENTITY-DROPS-001`,
`ENT-PROJECTILE-001`, `MOB-AI-001`, `MOB-BREED-001`, `CLI-001`, `CLI-006`,
`CLI-UI-001`, `CLI-EFFECT-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked registration/components, direct and recursive item-tag closure,
Tropical-Fish/Guardian/Elder-Guardian and fishing loot, hook retrieval, the sole level-four
Fisherman record and set-selection bytecode, advancements and direct client resources determine
every Tropical-Fish-item-specific branch. Tropical Fish entity variants and bucket payloads,
generic use, death, loot, fishing, merchant, mob, progression, stack and client algorithms remain
with the cited owners.

**Applies when:**

A `tropical_fish` stack is emitted by Tropical Fish, Guardian or Elder Guardian death or by
fishing; eaten; sold to a level-four Fisherman; offered to a Dolphin, Wolf or Nautilus; moved,
renamed, persisted, synchronized or rendered before and after component, tag, loot, recipe,
trade, advancement or resource reload.

**Authoritative state:**

`minecraft:tropical_fish` is raw item ID `1088`. It is a common nondamageable plain `Item` with
maximum stack `64`, food nutrition `1` and saturation `0.2`, and the ordinary empty `32`-tick eat
consumable with no consume-effect entries or remainder.

Its other default components are the common empty modifiers/enchantments/lore, item-break sound,
translated name, direct item-model key, repair cost, swing animation, tooltip display and use
effects. The default stack has no `tropical_fish/pattern`, base-color, pattern-color, bucket data
or entity data; arbitrary patches remain generic component work.

The direct tags are `fishes` and `wolf_food`. `nautilus_food` recursively includes the complete
`fishes` tag, so the item is also Nautilus food and a Nautilus temptation. It is not in
`cat_food`, `ocelot_food`, `axolotl_food` or `nautilus_taming_items`: Axolotl food is specifically
the Tropical Fish Bucket, and only Pufferfish identities tame an adult Nautilus in the locked
fish family.

**Transition and ordering:**

Player consumption and progression:

In-air use enters generic consumption only below full hunger or when ability permits full-hunger
eating. Interruption or live-hand/component replacement before completion commits nothing.
Successful server completion emits eat effects, awards the statistic, triggers `consume_item`
against the live pre-shrink stack, applies food, runs the empty effect list, emits `EAT` and
shrinks one unless materials are infinite.

The default stack adds `1` food and `0.2` saturation subject to generic clamps, spends no
consume-effect RNG and leaves no remainder. Tropical Fish is one independent requirement of
telemetry-enabled `husbandry/balanced_diet`; all `40` foods award `100` experience.

Direct Tropical-Fish death:

The `entities/tropical_fish` table has two ordered one-roll pools under random sequence
`minecraft:entities/tropical_fish`. The first emits one default Tropical Fish and redundantly
sets count to constant `1`. It has no age, variant, killed-by-player, attacker, fire, Looting or
chance condition. Every admitted normal table evaluation therefore emits the same unpatterned
item; entity pattern and colors are not copied.

The later pool draws one `0.05` random-chance float and emits one default Bone Meal on success.
It cannot alter the already emitted fish. Direct death has no furnace-smelt function, so fire or
`smelts_loot` never converts this item even if data reload adds a Tropical-Fish smelting recipe.
Spawn, death admission, rule gates, table invocation, item insertion and removal paths that
bypass normal death remain with entity and loot owners.

Guardian and Elder-Guardian rare-fish death:

Both Guardian tables have a later pool gated first by killed-by-player and then by an enchanted
chance. Admission is `0.025` without Looting and
`0.035 + 0.01*(L-1)` for Looting level `L>=1`. Once admitted, the pool invokes
`gameplay/fishing/fish`, where Tropical Fish has weight `2/100`, and applies one outer
conditional furnace-smelt function to the nested result.

On the locked vanilla recipe set, Tropical Fish has no smelting recipe, so even a burning
Guardian or direct attacker with `smelts_loot` leaves the selected one-stack unchanged. Recipe
reload can make only this Guardian-origin branch convert in future evaluations. Elder Guardian
evaluates its killed-by-player Wet-Sponge pool before rare fish; its later independent
Tide-Trim/empty pool still follows. Prismarine shards, the Cod/crystal/empty pool, wet sponge,
other fish and trim outputs retain their cited owners.

The outer sequences are `minecraft:entities/guardian` and
`minecraft:entities/elder_guardian`; nested fish selection uses
`minecraft:gameplay/fishing/fish`. Killed-by-player, attacker/enchantment context, live recipe
resolution and output placement remain generic death/loot work.

Fishing acquisition and retrieval:

The root `gameplay/fishing` table selects junk, treasure or fish. For loot-context luck `l`, its
effective integer weights are

`J = max(floor(10 - 2l), 0)`, `T = max(floor(5 + 2l), 0)` and
`F = max(floor(85 - l), 0)`.

Treasure is absent unless the hook is in open water. The nested fish table gives Tropical Fish
weight `2/100 = 1/50`, so when the denominator is positive its conditional probability is

`F / (J + F + (open_water ? T : 0)) * 1/50`.

At `l=0` this is `17/1000` in open water and `17/950` outside it. The root and nested tables use
separate sequences `minecraft:gameplay/fishing` and `minecraft:gameplay/fishing/fish`. Fishing
emits one default stack and applies no smelt or count function.

Retrieval first triggers `fishing_rod_hooked`; Tropical Fish is one of four alternatives in
telemetry-enabled `husbandry/fishy_business`. It then creates and inserts the item entity,
creates one XP orb of uniform value `1..6`, and increments `fish_caught` once because the result
is in `fishes`. The criterion therefore observes the generated stack before item, XP and
statistic commits. A data-reloaded result still satisfies the tag-driven statistic only while it
remains in `fishes`; obtaining the item elsewhere triggers neither fishing progress nor the
statistic.

Hook admission/open-water history, insertion, motion, XP, rod damage and removal remain with
`ENT-PROJECTILE-001` and `ITM-LOOT-001`.

Guaranteed level-four Fisherman purchase:

`fisherman/4/tropical_fish_emerald` is predicate-free. It consumes six matching Tropical Fish
and gives one default Emerald, with maximum uses `12`, villager XP `30` and reputation discount
`0.05`. It has no additional cost, output modifier or double-price enchantment.

The level-four tag contains only this record while its trade set requests amount `2`, disables
duplicates by default and uses random sequence `minecraft:trade_set/fisherman/level_4`.
No-duplicate selection removes the sole candidate after its first attempt and then stops because
the list is empty. The default record always creates an offer, so exactly one Tropical-Fish
purchase is guaranteed; the amount does not duplicate it.

The empty item-cost predicate accepts ordinary component patches, which are consumed rather than
copied to the Emerald. Trade Rebalance replaces neither tag nor record. Offer creation consumes
nothing; profession leveling, pricing, transaction, exhaustion, restock and menu synchronization
remain merchant-owner work.

Dolphin, Wolf and Nautilus interactions:

A Dolphin accepts Tropical Fish through `fishes`, consumes one and plays its eating sound. An
age-unlocked baby advances ten percent of remaining whole seconds; otherwise it sets
`gotFish=true`, enabling later treasure-search AI. It never executes the held stack's consumable.

A tamed injured Wolf consumes Tropical Fish before growth/love, regardless feeder ownership, and
heals twice live nutrition: default healing is `2`; absent food also falls back to `2`. Other
admitted states use generic age-unlocked baby growth or age-zero adult `600`-tick love. Untamed
adults may enter love but cannot mate until tame.

Every Nautilus interaction first marks persistence. Babies accept Tropical Fish through recursive
`nautilus_food` and use generic growth. An untamed adult instead tests
`nautilus_taming_items`; Tropical Fish is absent, so it does not consume or tame. For a tamed
adult, secondary use opens inventory first; otherwise an injured state consumes and heals twice
nutrition (default `2`, absent-food fallback `1`), while a full-health eligible adult can consume
for `600`-tick love. Tropical Fish is a Nautilus temptation.

Cat and Ocelot predicates reject this identity, and Axolotl growth/love/temptation accepts only
the Tropical Fish Bucket. None of these mob paths applies player food state or invokes the
item's consumable.

**Persistence and reload boundary:**

The stack persists identity, count and arbitrary patches. It stores no entity variant, use/
hunger, death/attacker/Looting, hook/open-water/luck, merchant, mob or advancement state.

Loot reload changes future direct, Guardian and fishing evaluations. Recipe reload affects only
future conditional Guardian rare-fish smelting. Tag reload changes future Dolphin/Wolf/Nautilus
admission and fishing-stat tests. Trade reload changes future level-four offers; advancement
reload changes listeners. Completed work is not replayed. Resource reload independently controls
the name, model, texture and advancement presentation.

**Client and wire projection:**

Generic stack encoding projects raw ID `1088` plus patches. The locked English name is
`Tropical Fish`; it is common with no forced glint or default subtype tooltip. A patched
`tropical_fish/pattern` component can enter the generic component-tooltip path, but the default
item has none.

The direct item definition selects the ordinary generated `item/tropical_fish` model and
same-named texture, with no special display transform. It appears exactly once in Food & Drinks,
ordered Cooked Salmon, Tropical Fish, Pufferfish, Bread.

**Branches and aborts:**

Identity/count/components/two effective tags; use; direct and two Guardian death routes; root/
nested fishing and retrieval; sole Fisherman record/set; Dolphin/Wolf/Nautilus precedence;
negative Cat/Ocelot/Axolotl/taming joins; progression, persistence, reload, wire, model and tab.

**Constants and randomness:**

Raw ID `1088`; max `64`; food `1/0.2`; eat `32`; direct death count `1` then Bone Meal `0.05`;
Guardian rare admission `0.025` or `0.035+0.01(L-1)`, nested Tropical Fish `1/50`; fishing root
formula and nested `1/50`, baseline `17/1000` open and `17/950` nonopen, XP `1..6`; Fisherman
amount `2` over one no-duplicate record, cost/result `6→1`, uses `12`, XP `30`, discount `0.05`;
Wolf/Nautilus default heal `2`.

**Side effects:**

Food/use; entity/fishing loot and named cursors; fishing criterion/item/XP/stat; merchant offer/
cost/result/economy; Dolphin age/`gotFish`, Wolf health/age/love, Nautilus persistence/health/
age/love/temptation; advancement, persistence, wire and client projection.

**Gates:**

Food/hunger/use; direct death and Guardian player-credit/chance/fire/attacker/live-recipe gates;
hook/open-water/luck; Fisherman profession/level/set/offer; live mob tags and subtype state;
progression listeners; registry/decode/client bootstrap.

**State read/written:**

Reads stack/components/tags, player use/progression/statistics, entity death contexts, hook/loot
state, live recipes, Villager offers, three mob states and client resources. Writes only the
consumption, loot, fishing, progression, trade, mob and projection state listed above.

**Failure behavior:**

Unadmitted use commits nothing. Missing direct death table emits nothing; its Bone-Meal failure
does not alter fish. Unadmitted Guardian rare pools or nested selection emit alternatives.
Missing Guardian smelting recipe preserves Tropical Fish. Failed fishing insertion does not
suppress later XP/stat work. Invalid trade data or exhausted offer prevents sale. Ineligible mob
states do not consume except the explicit adult-Dolphin `gotFish` route.

**Boundary cases and quirks:**

Every Tropical Fish entity variant dies into the same componentless item. Direct death ignores
fire and every smelting recipe, while Guardian rare-fish selection conditionally consults one.
Open water lowers the luck-zero fishing probability from `17/950` to `17/1000` by adding treasure
weight. Fishy Business precedes item/XP/stat creation. The Fisherman set requests two offers but
its one-entry no-duplicate list yields exactly one. Tropical Fish feeds but cannot tame an adult
Nautilus, and only the distinct bucket item feeds Axolotls.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-DATA-001`; `OFF-REPORT-001`;
`net.minecraft.world.item.Items`;
`net.minecraft.world.entity.projectile.FishingHook#retrieve`;
`net.minecraft.world.level.storage.loot.functions.SmeltItemFunction#run`;
`net.minecraft.world.entity.animal.dolphin.Dolphin#mobInteract`;
`net.minecraft.world.entity.animal.wolf.Wolf#mobInteract`;
`net.minecraft.world.entity.animal.wolf.Wolf#isFood`;
`net.minecraft.world.entity.animal.nautilus.AbstractNautilus#interact`;
`net.minecraft.world.entity.animal.nautilus.AbstractNautilus#mobInteract`;
`net.minecraft.world.entity.animal.nautilus.AbstractNautilus#isFood`;
`net.minecraft.world.entity.animal.nautilus.NautilusAi#getTemptations`;
`net.minecraft.world.entity.animal.axolotl.Axolotl#isFood`;
`net.minecraft.world.entity.TamableAnimal#feed`;
`net.minecraft.world.entity.npc.villager.AbstractVillager#addOffersFromTradeSet`;
`net.minecraft.world.entity.npc.villager.AbstractVillager#addOffersFromItemListingsWithoutDuplicates`;
`net.minecraft.world.item.trading.VillagerTrade#getOffer`;
`net.minecraft.world.item.trading.TradeSet`;
`net.minecraft.world.item.CreativeModeTabs`;
`net.minecraft.data.loot.packs.VanillaEntityLoot`;
`net.minecraft.data.loot.packs.VanillaFishingLoot`;
`net.minecraft.data.advancements.packs.VanillaHusbandryAdvancements`;
`reports/registries.json#minecraft:{item,entity_type,loot_table,advancement,villager_trade,trade_set}`;
`reports/minecraft/components/item/tropical_fish.json`;
`data/minecraft/tags/item/{fishes,wolf_food,nautilus_food,nautilus_taming_items,cat_food,ocelot_food,axolotl_food}.json`;
`data/minecraft/loot_table/{entities/{tropical_fish,guardian,elder_guardian},gameplay/fishing,gameplay/fishing/fish}.json`;
`data/minecraft/{villager_trade/fisherman/4/tropical_fish_emerald,tags/villager_trade/fisherman/level_4,trade_set/fisherman/level_4}.json`;
`data/minecraft/advancement/husbandry/{balanced_diet,fishy_business,tactical_fishing,kill_axolotl_target}.json`;
`assets/minecraft/{items,models/item,textures/item}/tropical_fish.*`;
`ITM-USE-001`; `ITM-HUNGER-001`; `ITM-LOOT-001`; `ITM-ADVANCEMENT-001`;
`ITM-PRISMARINE-MATERIAL-001`; `ITM-COD-001`; `ITM-SALMON-001`;
`ITM-PUFFERFISH-001`; `ITM-MOB-BUCKET-001`; `ENT-DEATH-001`;
`ENT-ENTITY-DROPS-001`; `ENT-PROJECTILE-001`; `MOB-AI-001`; `MOB-BREED-001`;
`CLI-UI-001`; `CLI-EFFECT-001`; `EXP-ITM-070`.

**Test vectors:**

Exercise default, removed and arbitrary component-patched stacks through both hands, full/
nonfull hunger, infinite materials and interruption. Kill every Tropical Fish variant across
age/fire/attacker/Looting/player-credit states and trace invariant unpatterned fish then the
later Bone-Meal draw.

Kill Guardian/Elder Guardian across player-credit/Looting/fire/smelts-loot and missing/default/
added Tropical-Fish smelting recipes while tracing outer and nested sequences, neighboring pools
and exact smelt preservation/conversion. Retrieve fishing loot across open-water/luck/weight and
insertion boundaries.

Generate the level-four Fisherman set through amount/list exhaustion, transact/exhaust/restock
the offer and test patched costs. Offer the item to every Dolphin/Wolf/Nautilus state and verify
Cat/Ocelot/Axolotl/adult-Nautilus-taming rejection. Reload all domains, persist/synchronize and
verify raw ID, name, default tooltip, ordinary generated model/texture and exact Food-tab
neighborhood.
