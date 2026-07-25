# Items mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ITM-RABBIT-FOOT-001` — Rabbit's Feet join four acquisition paths to Camel Husk feeding, Cleric trade and Leaping brewing

**Parent:** `PLY-005`, `PLY-006`, `PLY-INPUT-001`, `PLY-INTERACT-001`,
`PLY-MOVE-001`, `PLY-AUTOJUMP-001`, `ITM-001`, `ITM-002`, `ITM-003`,
`ITM-004`, `ITM-005`, `ITM-006`, `ITM-007`, `ITM-USE-001`,
`ITM-CONTAINER-001`, `ITM-BREW-001`, `ITM-LOOT-001`,
`ITM-ADVANCEMENT-001`, `ITM-ANVIL-001`, `ITM-POTION-001`, `ENT-001`,
`ENT-005`, `ENT-DEATH-001`, `ENT-ENTITY-DROPS-001`, `ENT-EFFECT-001`,
`MOB-SPAWN-001`, `MOB-AI-001`, `MOB-BREED-001`, `WGEN-DIMENSION-001`,
`CLI-001`, `CLI-006`, `CLI-UI-001`, `CLI-EFFECT-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked registration/components and tag, exhaustive code/data references,
Rabbit/Husk/cat loot, Fox finalization and death, Camel Husk interaction, Cleric trade, brewing
graph, potion payloads and client assets determine every Foot-specific branch. Generic mob
spawning/death/AI, loot, trade, brewing, effects, stacks and inventories remain with the cited
owners.

**Applies when:**

A `rabbit_foot` stack is created, looted, equipped or released by a Fox, moved, renamed,
persisted, synchronized, used on a Camel Husk, offered to a Cleric or Brewing Stand, selected in a
tab, rendered or observed before and after tag, loot, trade, mix, timeline or resource reload.

**Authoritative state:**

`minecraft:rabbit_foot` is raw item ID `1282`. It registers through the plain-item path with
default properties, is common, nondamageable and has max stack `64`.

Its registered components are only the common empty modifiers/enchantments/lore, item-break sound,
translated name, direct item-model key, repair cost, swing animation, tooltip display and use
effects. It has no food, consumable, cooldown, remainder, tool, equipment, repairable or
identity-specific glint state.

Its sole direct item tag is `#minecraft:camel_husk_food`, whose locked baseline contains only
Rabbit's Foot. The tag affects Camel Husk interaction but does not make a player or Fox able to
consume the componentless item.

**Transition and ordering:**

The identity does not override hand use or block use. A prototype stack's air use returns generic
`PASS`; a block click participates only in ordinary block-first interaction and fallback handling.
A component-patched stack can activate a generic component owner, but the identity itself never
starts active use or provides player food.

Container movement, pickup, dropping, anvil naming and component patching use their generic owners.
The identity adds no dispenser, equipment, repair, composting or crafting-recipe branch. Four
acquisition paths, Camel Husk feeding, Cleric trade and brewing own the operational joins below.

**Rabbit acquisition:**

The third pool of `entities/rabbit` has one roll and first tests `killed_by_player`. Failure skips
the Foot chance draw and entry, though the table's earlier Hide and raw-Rabbit pools already ran
and advanced the same named sequence through their applicable count/Looting branches.

After player-kill admission, `random_chance_with_enchanted_bonus` reads Looting from a living
attacking entity. With no positive level, it draws one float and succeeds when `U < 0.10`. At
positive level `L`, it succeeds when `U < 0.13 + 0.03*(L-1)`, equivalently
`U < 0.10 + 0.03L`. Thus normal levels `1/2/3` give `0.13/0.16/0.19`; arbitrary level `30` or
higher makes every `[0,1)` draw pass. Success emits one default Foot with no count function.

The full table uses random sequence `minecraft:entities/rabbit`. Player-kill attribution and the
living attacker used for Looting are separate context facts: the first is mandatory, while the
second only selects the boosted chance. Rabbit spawning/AI remain with the mob owners; death
admission, table invocation and world-drop placement remain with `ENT-DEATH-001`,
`ENT-ENTITY-DROPS-001` and `ITM-LOOT-001`.

**Camel-Husk rider acquisition:**

The second pool of `entities/husk` has one roll and no player-kill condition. Its sole Foot entry
first requires the dying Husk's immediate vehicle to have entity type `minecraft:camel_husk`.
A dismounted Husk, or one riding any other vehicle, skips the Foot count and bonus draws.

An admitted entry creates one Foot, replaces its count with a uniformly drawn integer `0..1`,
then applies enchanted count increase. With a living attacking entity and Looting `L > 0`, it
draws a fresh uniform float `V` in `[0,1)` and adds `round(L * V)`; absent/nonliving attacker or
level zero returns without that draw. There is no count limit, so a positive bonus can revive a
zero base.

This pool runs after the independent Rotten Flesh pool and before the later player-gated rare
Husk pool. All share random sequence `minecraft:entities/husk`; earlier and later branches do not
change the Foot count but do determine full cursor order. Generic passenger state at death and
loot emission remain with the entity/death owners.

**Cat morning-gift acquisition:**

A qualified tame-cat relax goal and owner sleep timer of at least `100` first compare one level
RNG float against the live
`minecraft:gameplay/cat_waking_up_gift_chance` attribute at the cat. The attribute defaults to
`0`; the locked circular `day` timeline supplies constant keyframes tick `362` value `0` and tick
`23667` value `0.7`, so the normal wake marker at tick `0` resolves `0.7`. Cat goal/owner state and
dimension/timeline resolution remain with `MOB-AI-001`, `MOB-BREED-001` and
`WGEN-DIMENSION-001`.

Chance success next spends cat RNG on an attempted teleport whose result is ignored, then
evaluates the one-roll gift table. Six entries including Foot have weight `10`; Phantom Membrane
has weight `2`, for total `62`. Conditioned on the table, Foot therefore has probability
`10/62=5/31` and emits one default stack. At live gift chance `g`, a qualified goal stop emits Foot
with probability `5g/31`; at the normal locked `g=0.7` value this is `7/62`.

Gift selection uses random sequence `minecraft:gameplay/cat_morning_gift`, distinct from the
chance and teleport RNG. Its callback inserts the result at the cat's final position offset one
horizontal unit along body rotation; insertion failure is ignored. Exact goal-space, owner, sleep,
teleport and output placement behavior remains with the same owners fixed by
`ITM-PHANTOM-MEMBRANE-001`.

**Fox-held acquisition:**

Every Fox finalization invokes default-equipment population. Its first level RNG float must be
strictly below `0.2`; failure equips nothing. Success consumes a second float `W`. Values
`0.2 <= W < 0.4` enter the Rabbit-item branch, where one Boolean selects Foot when true and Rabbit
Hide when false. The resulting default Foot count is one.

The independent probability is therefore `0.2 * 0.2 * 0.5 = 0.02`. Other second-float intervals
select Emerald (`<0.05`), Egg (`0.05..0.2`), Wheat (`0.4..0.6`), Leather (`0.6..0.8`) or Feather
(`>=0.8`); no branch falls through to Foot.

Foot has neither food nor consumable components, so a Fox does not eat it. Fox pickup logic can
later replace a held nonfood with an eligible consumable and spit the Foot into the world. If an
admitted Fox death still has Foot in its main hand, `dropAllDeathLoot` spawns the entire held stack
and clears the slot before generic animal loot, without an equipment-drop chance. Factory,
insertion, pickup and death admission remain with the generic mob/entity owners.

No bundled chest, fishing, block-drop or other loot table directly emits Foot. Administration and
custom data can still create ordinary stacks through generic item/loot boundaries.

**Camel Husk feeding join:**

Camel Husk overrides `isFood` with the one-member tag. Its interaction marks the Camel Husk
persistent before delegating, so even a later nonconsuming result retains that persistence side
effect.

The inherited Camel food transaction heals an injured Camel Husk by `2`, then, unless silent,
plays its eating sound at volume `1` and pitch `1 + (U1-U2)*0.2`, emits `EAT`, succeeds and consumes
one Foot through the player's ability-aware consume rule. Camel Husk cannot be a baby and
overrides love admission false, so Foot never grows or breeds it.

At full health none of heal, love or growth applies. The food transaction returns without sound,
event or consumption, but the earlier persistence mark remains. Tag reload can replace future
admission without changing already consumed stacks or entity state.

**Cleric trade join:**

`cleric/3/rabbit_foot_emerald` wants exactly two plain item-identity Feet and gives one default
Emerald. Its maximum uses are `12`, villager XP is `20`, and reputation discount coefficient is
`0.05`.

The level-three Cleric tag contains exactly that purchase and the four-Emerald-to-Glowstone sale.
Its trade set requests two distinct entries with random sequence
`minecraft:trade_set/cleric/level_3`; because candidate count equals requested amount, every
baseline level-three set includes the Foot purchase, with only offer order randomized.

Offer construction, component predicate, pricing/demand/reputation, exhaustion, restocking and
trade commit remain with the generic villager/trade owners. This is a Foot sink, not an acquisition
source.

**Brewing join:**

The vanilla mix builder registers Foot as a start ingredient. The helper adds Water plus Foot to
Mundane and Awkward plus Foot to Leaping. The latter contains amplifier-zero Jump Boost for `3600`
ticks (`180` seconds).

Redstone Dust, not another Foot, extends Leaping to `9600` ticks (`480` seconds); Glowstone Dust
creates amplifier-one Strong Leaping for `1800` ticks (`90` seconds). Fermented Spider Eye
separately corrupts Leaping to Slowness and Long Leaping to Long Slowness.

A completed brew transforms every matching bottle slot in owner order, then consumes one
ingredient Foot with no remainder and emits the generic brew event. Foot is not a member of
`brewing_fuel`, is not furnace fuel and cannot prepay fuel uses.

Slot admission, fuel uses, 400-tick timer/cancellation, bottle transforms, automation and the
player-menu take criterion remain with `ITM-BREW-001` and `ITM-ADVANCEMENT-001`. Potion
consumption/projection and Jump Boost effect merge/ticks remain with `ITM-POTION-001` and
`ENT-EFFECT-001`; movement/jump/auto-jump consequences remain with `PLY-MOVE-001` and
`PLY-AUTOJUMP-001`.

No locked crafting recipe consumes or emits Foot. Taking a Foot-produced potion from a Brewing
Stand as a server player can satisfy the unfiltered `nether/brew_potion` criterion; automation
extraction does not run that player slot hook.

**Persistence and reload boundary:**

Foot stacks persist and synchronize identity, count and arbitrary ordinary component patches.
They store no death/attacker/vehicle context, Looting level, chance/count draws, cat/Fox/Camel
state, offer lifecycle or brewing slot/fuel/timer/mix. Those values belong to their entity, world,
loot, trade and machine owners. A Fox separately persists its current held stack.

Loot reload can independently replace all three tables for future evaluations. Tag reload changes
future Camel Husk food admission; timeline/dimension reload changes future cat chance; trade reload
changes future offer sets; a rebuilt baseline mix retains the two Foot start edges while their
holders are enabled. Completed loot, feeding, trades and brews are not replayed. Resource reload
independently controls name and model.

**Client and wire projection:**

Generic item-stack encoding projects raw item ID `1282` plus the stack's component patch. Its
common-rarity name uses locked English text `Rabbit's Foot`; the plain class adds no subtype
tooltip or forced glint.

The direct item definition selects generated model `minecraft:item/rabbit_foot` and its same-named
texture. It appears exactly once and only in Ingredients, ordered Sugar, Rabbit's Foot, Glistering
Melon Slice.

**Branches and aborts:**

Identity/count/components/tag; generic hand/block/container/anvil path; Rabbit player attribution,
attacker/Looting/chance; Husk vehicle/attacker/base/bonus and adjacent pools; cat goal/sleep/live
chance/teleport/weight; Fox finalization/equipment/swap/death; Camel Husk health/silence/
persistence; Cleric set/offer/lifecycle; brewing fuel/bottle/potion state; save,
tag/loot/timeline/trade/mix/resource reload, wire, language, model and tab context.

**Constants and randomness:**

Raw item ID `1282`; common rarity; max stack `64`; Rabbit chance `0.10` at `L<=0` and
`0.10+0.03L` at `L>0`; Husk base uniform integer `0..1`, bonus `round(L*U[0,1))`; cat gift
weight `10/62=5/31`, normal combined rate `7/62`; Fox Foot probability `0.02`; Camel Husk heal
`2`, eating volume `1`, pitch `1+(U1-U2)*0.2`; trade `2` Feet to `1` Emerald, uses `12`, XP `20`,
discount `0.05`; Leaping durations/amplifiers `3600@0`, `9600@0`, `1800@1`; owner brew duration
`400`.

**Side effects:**

Possible Rabbit/Husk/cat/Fox item stacks and named-sequence cursors; cat/Fox/Camel position,
equipment, health, persistence, sound/event and held-item state; player count; Cleric offer/use/XP/
economy state; brewing ingredient/bottles/timer/event; brewed-potion progress; ordinary stack
persistence/wire state; name, direct model and one Ingredients-tab entry.

**Gates:**

Generic stack/container/anvil admission; player-attributed Rabbit death and Looting chance;
Husk-on-Camel-Husk vehicle predicate and living attacker for bonus; qualified cat goal/live gift
chance/table; Fox finalization/equipment/current hand and admitted death; current Camel-Husk-food
tag plus missing health; level-three Cleric offer and valid trade inputs; valid brewing fuel and
Water/Awkward mix; registry/stack decode; client language/model and tab bootstrap.

**State read/written:**

Reads stack identity/count/components/tags, interaction/container state, entity death/attacker/
vehicle context, three loot sequences, cat/Fox/Camel/owner/world RNG and state, environment
attribute, Cleric trade registries/economy, brewing slots/fuel/timer/mix, progression, persistence
and client resources. Writes only the acquisition, feeding, trade, brewing, stack and client state
listed above.

**Failure behavior:**

Use has no subtype success or mutation. Rabbit attribution or chance failure emits no Foot; absent
living attacker selects unenchanted chance. A nonmatching Husk vehicle skips Foot draws; zero final
count emits none. Failed cat chance/alternate gift emits none. Fox equipment failure or later
replacement removes that held acquisition path; death insertion failure is ignored. Full-health
Camel Husk feeding consumes nothing but retains the prior persistence mark. Invalid/exhausted
trade commits nothing. Missing fuel or unmatched potion prevents brewing; Foot is not itself fuel.
Missing/replaced tag, loot, timeline, trade or mix data removes future paths without rewriting
stacks. Client resource absence follows generic fallback and cannot grant authority.

**Boundary cases and quirks:**

Rabbit Foot chance uses unenchanted `0.10` unless a living attacker has positive Looting, even
after player-kill admission. Husk Foot has no player gate and Looting can revive zero, but only
while the dead Husk still rides a Camel Husk. Cat acquisition has a much higher table weight than
Membrane. Fox creates a componentless nonfood Foot and guarantees its current main hand out before
ordinary death loot. Camel Husk food admission persists the entity even when full health prevents
consumption. Foot start-mixes Water and Awkward but cannot fuel the stand.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-DATA-001`; `OFF-REPORT-001`;
`net.minecraft.world.item.Items`;
`net.minecraft.world.level.storage.loot.predicates.LootItemRandomChanceWithEnchantedBonusCondition#test`;
`net.minecraft.world.level.storage.loot.functions.EnchantedCountIncreaseFunction#run`;
`net.minecraft.world.level.storage.loot.providers.number.UniformGenerator`;
`net.minecraft.world.entity.animal.feline.Cat$CatRelaxOnOwnerGoal#stop`;
`net.minecraft.world.entity.animal.feline.Cat$CatRelaxOnOwnerGoal#giveMorningGift`;
`net.minecraft.world.entity.LivingEntity#dropFromGiftLootTable`;
`net.minecraft.world.entity.animal.fox.Fox#populateDefaultEquipmentSlots`;
`net.minecraft.world.entity.animal.fox.Fox#dropAllDeathLoot`;
`net.minecraft.world.entity.animal.camel.CamelHusk#interact`;
`net.minecraft.world.entity.animal.camel.CamelHusk#isFood`;
`net.minecraft.world.entity.animal.camel.Camel#handleEating`;
`net.minecraft.world.item.trading.TradeSet#calculateNumberOfTrades`;
`net.minecraft.world.item.trading.VillagerTrade#getOffer`;
`net.minecraft.world.item.alchemy.PotionBrewing#addVanillaMixes`;
`net.minecraft.world.item.alchemy.PotionBrewing$Builder#addStartMix`;
`net.minecraft.world.item.alchemy.Potions`;
`net.minecraft.world.item.CreativeModeTabs`;
`net.minecraft.data.loot.packs.VanillaEntityLoot`;
`net.minecraft.data.loot.packs.VanillaGiftLoot`;
`reports/registries.json#minecraft:{item,entity_type,potion,mob_effect,environment_attribute}`;
`reports/minecraft/components/item/rabbit_foot.json`;
`data/minecraft/tags/item/camel_husk_food.json`;
`data/minecraft/loot_table/{entities/{rabbit,husk},gameplay/cat_morning_gift}.json`;
`data/minecraft/timeline/day.json`;
`data/minecraft/{villager_trade/cleric/3/rabbit_foot_emerald,tags/villager_trade/cleric/level_3,trade_set/cleric/level_3}.json`;
`assets/minecraft/{items,models/item,textures/item}/rabbit_foot.*`;
`ITM-BREW-001`; `ITM-LOOT-001`; `ITM-ADVANCEMENT-001`; `ITM-POTION-001`;
`ENT-DEATH-001`; `ENT-ENTITY-DROPS-001`; `MOB-SPAWN-001`; `MOB-AI-001`;
`MOB-BREED-001`; `WGEN-DIMENSION-001`; `PLY-MOVE-001`; `PLY-AUTOJUMP-001`;
`CLI-UI-001`; `CLI-EFFECT-001`; `EXP-ITM-046`.

**Test vectors:**

Exercise default/patched stacks through hands, blocks, containers and anvil. Evaluate Rabbit deaths
across player attribution, attacker types, Looting levels and exact chance equality; Husk deaths
across every vehicle/attacker/base/bonus and adjacent-pool branch; and cat stops across all
chance/teleport/weight branches while tracing named sequences. Finalize Foxes at every float/
Boolean boundary, then swap/death-drop held Feet. Feed full/injured/silent Camel Husks before/after
tag reload. Build/use/exhaust/reload level-three Cleric sets. Brew Water, Awkward, Leaping and
every downstream/control potion with valid/invalid fuel. Persist/synchronize state and capture raw
ID, name, tooltip, model and exact Ingredients position before/after reload.

**Limits:**

This leaf does not duplicate Rabbit/Husk/cat/Fox/Camel spawning, AI, death or persistence,
environment-attribute resolution, generic loot/trade evaluation, brewing transaction/automation,
potion/effect/movement behavior or stack/resource codecs. Those remain with their cited owners;
this rule fixes the Foot identity and its exact acquisition, feeding, trade, brewing, progression
and presentation joins.
