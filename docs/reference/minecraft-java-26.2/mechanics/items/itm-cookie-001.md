# Items mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ITM-COOKIE-001` — Cookie joins an exact recipe, Farmer sale and gift, composting and the lethal Parrot-food branch

**Parent:** `PLY-005`, `PLY-006`, `PLY-INPUT-001`, `PLY-INTERACT-001`,
`ITM-001`, `ITM-002`, `ITM-003`, `ITM-004`, `ITM-005`, `ITM-006`,
`ITM-007`, `ITM-USE-001`, `ITM-CONTAINER-001`, `ITM-RECIPE-001`,
`ITM-CRAFT-001`, `ITM-LOOT-001`, `ITM-ADVANCEMENT-001`, `ITM-HUNGER-001`,
`ITM-ENCHANT-001`, `ITM-ANVIL-001`, `ENT-001`, `ENT-005`, `ENT-DAMAGE-001`,
`ENT-DAMAGE-REDUCE-001`, `ENT-DEATH-001`, `ENT-EFFECT-001`, `MOB-AI-001`,
`MOB-BREED-001`, `MOB-RAID-001`, `CLI-001`, `CLI-006`, `CLI-UI-001`,
`CLI-EFFECT-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked registration/components, recipe/unlock, Farmer trade/gift records,
the direct Parrot tag, Parrot interaction and generic mob-item bytecode, code-built Composter
chance, advancements and client resources determine every Cookie-specific branch. Generic use,
crafting, merchants, gifts, effects, damage/death, mob interaction, composting, stacks and client
algorithms remain with the cited owners.

**Applies when:**

A `cookie` stack is crafted, received from a Farmer offer or Hero gift, eaten by a player, offered
to a Parrot, inserted into a Composter, moved, renamed, persisted, synchronized or rendered before
and after component, tag, recipe, trade, loot, advancement or resource reload.

**Authoritative state:**

`minecraft:cookie` is raw item ID `1131`. It is a common nondamageable plain `Item` with maximum
stack `64`, food nutrition `2` and saturation `0.4`, and the ordinary empty `32`-tick eat
consumable with no consume-effect entries or remainder.

Its other default components are the common empty modifiers/enchantments/lore, item-break sound,
translated name, direct item-model key, repair cost, swing animation, tooltip display and use
effects. Its sole direct item tag is `parrot_poisonous_food`; it is not in `parrot_food`.

Composter bootstrap separately maps Cookie identity to Java float chance `0.85f`. The Parrot
branch consults the live poisonous-food tag and a live `use_remainder` component, but not Cookie's
food or consumable components.

**Transition and ordering:**

Player consumption and progression:

In-air use enters generic consumption only below full hunger or when ability permits full-hunger
eating. Interruption or live-hand/component replacement before completion commits nothing.
Successful server completion emits eat effects, awards the statistic, triggers `consume_item`
against the live pre-shrink stack, applies food, runs the empty effect list, emits `EAT` and
shrinks one unless materials are infinite.

Default Cookie adds `2` food and `0.4` saturation subject to generic clamps, spends no
consume-effect RNG and leaves no remainder. Cookie is one independent requirement of
telemetry-enabled `husbandry/balanced_diet`; all `40` foods award `100` experience.

Player crafting and recipe progression:

The sole recipe is a shaped one-row pattern `#X#`, where `#` is exact Wheat and `X` is exact Cocoa
Beans. It fits any one row of a `3×3` Crafting Table, is horizontally symmetric, does not fit a
`2×2` grid and rejects every extra occupied cell. A match consumes the three inputs and emits
eight default Cookies; input patches do not propagate and none of the inputs has a remainder.

The no-display recipe advancement has one OR requirement: possessing Cocoa Beans or already
knowing `minecraft:cookie` grants the recipe. Recipe reload changes future matching, output and
unlock listeners without rewriting existing stacks.

Farmer offer and Hero gift:

Farmer level three contains exactly two predicate-free records and requests two offers without
duplicates, so `farmer/3/emerald_cookie` is guaranteed once under
`minecraft:trade_set/farmer/level_3`. It consumes three Emeralds and gives eighteen default
Cookies, with maximum uses `12`, villager XP `10` and reputation discount `0.05`. There is no
second cost, predicate, output modifier or double-price enchantment.

An admitted adult Farmer Hero gift chooses uniformly among one default Bread, Pumpkin Pie and
Cookie. Cookie probability is `1/3` under
`minecraft:gameplay/hero_of_the_village/farmer_gift`. Initial eligible cooldown is `600`; later
cooldown is `600 + nextInt(6001)`, target range is five blocks, behavior lasts at most `100`
ticks and throws only after elapsed time exceeds `20`. Gift admission/navigation/throw/cleanup
remain `MOB-RAID-001`; offer economy and menu work remain merchant-owned. The optional Trade
Rebalance pack replaces neither Cookie record.

Parrot poisonous-food interaction:

With the default tags, every untamed/tamed, adult/baby and owner/nonowner Parrot reaches the
poisonous-food test because Cookie fails the earlier untamed-only `parrot_food` branch. A matching
live hand stack commits in this exact order:

1. `Mob.usePlayerItem` snapshots the original count and live `use_remainder`, consumes one through
   the player-aware stack operation, and resolves that remainder;
2. add a default-amplifier Poison effect for `900` ticks;
3. unless the player is noncreative and the Parrot's invulnerable flag is set, request
   player-attack damage of `Float.MAX_VALUE` (`3.4028235E38f`);
4. return `SUCCESS`.

Default Cookie has no remainder. A patched remainder returns unchanged for infinite-material
players; for finite players it replaces an emptied hand, or is added to inventory/dropped while a
nonempty source stack remains. This branch never starts Cookie's consumable, adds player hunger,
awards the Cookie-used statistic, triggers `consume_item` or plays the Parrot eat sound.

Poison is added before damage, including when the local invulnerable-flag gate skips damage.
`Entity.hurt` invokes actual damage only in a `ServerLevel`; the ordinary damage pipeline may
still reject the request, while an admitted ordinary Parrot is lethally reduced and follows
generic death/drop work. Damage source attribution is the interacting player's attack.

Tag reload can remove future poisonous admission. If reload instead makes the same stack
`parrot_food`, the earlier branch preempts poison for an untamed Parrot, consumes the item, plays
the eat sound and makes one `nextInt(10)` tame attempt; tamed Parrots skip that branch and still
test poisonous food. If neither tag matches, later owner/sit and generic interaction rules apply.
Cookie identity alone never breeds, heals, grows or tames a Parrot, and `Parrot.isFood` remains
false.

Composter insertion:

Player-held insertion at level `0` succeeds without RNG. Levels `1..6` consume one
`nextDouble()` and increment exactly when it is strictly below Java float `0.85f`, widened to
`0.8500000238418579` for comparison. Success writes level plus one with flags `3`, emits
`BLOCK_CHANGE`, and `6 -> 7` schedules maturation after `20` ticks; failure preserves state.

Either level-`0..6` result emits level event `1500` with success encoded by whether state changed,
awards the Cookie-used statistic and calls `consume(1, player)`, preserving infinite-material
holders. Level `7` returns success for held Cookie without insertion, event, statistic or
consumption. Level `8` delegates to ordinary item-on-block handling.

Automation exposes one top input slot only below level `7`. It accepts a compostable stack once,
runs the same deterministic-first-level/strict-double transition, emits event `1500`, and removes
the one-slot stack whether chance succeeded or failed. Maturation, Bone-Meal extraction and event
rendering remain with the Composter/block/client owners.

Advancement display join:

`husbandry/allay_deliver_item_to_player` uses a default Cookie as its hidden display icon. Its
criterion accepts any item thrown by an Allay and picked up by the player, so Cookie does not
narrow or otherwise participate in the trigger. Generic Allay item-template, pickup and delivery
behavior likewise remains identity-agnostic.

**Persistence and reload boundary:**

Cookie stacks persist identity, count and patches. Player active-use/hunger, recipe knowledge,
merchant offers/uses, Hero-gift behavior, Parrot effects/health/death and Composter state persist
with their owners.

Recipe reload changes future crafts. Tag reload changes future Parrot selection only. Trade,
loot and advancement reload change future offers, gifts and listeners/displays. The code-built
Composter chance does not reload. Completed consumption, interaction, damage or compost work is
not replayed. Resource reload independently controls name, model, texture and advancement icon.

**Client and wire projection:**

Generic stack encoding projects raw ID `1131` plus patches. The locked English name is `Cookie`;
it is common with no forced glint or subtype tooltip. Its direct item definition selects the
ordinary generated `item/cookie` model and same-named texture.

Cookie appears exactly once in Food & Drinks, ordered Bread, Cookie, Cake, Pumpkin Pie. The
hidden Allay-delivery advancement independently projects Cookie as its display icon.

**Branches and aborts:**

Identity/count/components/direct poisonous tag; player use; shaped crafting/unlock; Farmer
offer/gift; Parrot earlier-food/poison/remainder/effect/damage paths; Composter direct/automation;
Balanced Diet, advancement icon, persistence, reload, wire, model and tab.

**Constants and randomness:**

Raw ID `1131`; max `64`; player food `2/0.4`; eat `32`; recipe `2 Wheat + 1 Cocoa Beans -> 8`;
Farmer trade guaranteed, `3→18`, uses/XP `12/10`; gift `1/3`; Parrot poison `900`, damage
`3.4028235E38f`; optional earlier tame `nextInt(10)==0`; Composter
`0.8500000238418579`, maturation `20`.

**Side effects:**

Player food/use/progression; crafting output/unlock; merchant result/economy and Hero gift;
Parrot hand consumption/remainder, effect, damage/death; Composter level/event/stat/consumption/
schedule; persistence, wire, advancement display and client projection.

**Gates:**

Player hunger/use; exact grid/recipe; Farmer level/set/offer and Hero status/timing; live Parrot
tags, tame state, player materials/creative state, entity invulnerability and damage/effect
pipeline; Composter level/chance/side; progression listeners; registry/decode/client bootstrap.

**State read/written:**

Reads Cookie stack/components, player state, grid/recipe, merchant/gift state, Parrot tags/state,
effect/damage state, Composter state/RNG and client resources. Writes only the consumption,
crafting, trade/gift, entity/effect/damage, compost and projection state listed above.

**Failure behavior:**

Unadmitted player use commits nothing. Invalid grids do not craft. Invalid/exhausted offers commit
nothing. Unselected gifts emit another entry. A removed poisonous tag skips the branch; a patched
earlier food tag can preempt it for an untamed Parrot. Infinite materials preserve the hand item.
Parrot damage rejection does not undo the already consumed item or Poison effect. Composter chance
failure still consumes an admitted level-`1..6` input.

**Boundary cases and quirks:**

The Parrot branch consumes through the generic mob helper rather than executing Cookie's
consumable, so a poisonous Cookie can be used at full player hunger and honors a patched
`use_remainder` while ignoring patched food/consumable values. Poison precedes maximum-float
damage, and damage failure has no rollback. The default Cookie is poisonous but not Parrot
taming food. Composter level zero is deterministic while a failed later attempt still consumes
Cookie.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-DATA-001`; `OFF-REPORT-001`;
`net.minecraft.world.item.Items`;
`net.minecraft.world.entity.animal.parrot.Parrot#mobInteract`;
`net.minecraft.world.entity.animal.parrot.Parrot#isFood`;
`net.minecraft.world.entity.Mob#usePlayerItem`;
`net.minecraft.world.item.component.UseRemainder#convertIntoRemainder`;
`net.minecraft.world.entity.Entity#hurt`;
`net.minecraft.world.level.block.ComposterBlock#bootStrap`;
`net.minecraft.world.level.block.ComposterBlock#useItemOn`;
`net.minecraft.world.level.block.ComposterBlock#insertItem`;
`net.minecraft.world.level.block.ComposterBlock#addItem`;
`net.minecraft.world.entity.npc.villager.AbstractVillager#addOffersFromTradeSet`;
`net.minecraft.world.entity.ai.behavior.GiveGiftToHero`;
`net.minecraft.world.item.CreativeModeTabs`;
`net.minecraft.data.loot.packs.VanillaGiftLoot`;
`net.minecraft.data.recipes.packs.VanillaRecipeProvider`;
`net.minecraft.data.advancements.packs.VanillaHusbandryAdvancements`;
`reports/registries.json#minecraft:{item,entity_type,recipe,loot_table,advancement,villager_trade,trade_set,mob_effect,damage_type}`;
`reports/minecraft/components/item/cookie.json`;
`data/minecraft/{recipe/cookie,advancement/recipes/food/cookie}.json`;
`data/minecraft/tags/item/{parrot_poisonous_food,parrot_food}.json`;
`data/minecraft/{villager_trade/farmer/3/emerald_cookie,tags/villager_trade/farmer/level_3,trade_set/farmer/level_3}.json`;
`data/minecraft/loot_table/gameplay/hero_of_the_village/farmer_gift.json`;
`data/minecraft/advancement/husbandry/{balanced_diet,allay_deliver_item_to_player}.json`;
`assets/minecraft/{items,models/item,textures/item}/cookie.*`;
`ITM-CRAFT-001`; `ITM-ADVANCEMENT-001`; `ITM-HUNGER-001`; `ENT-DAMAGE-001`;
`ENT-DAMAGE-REDUCE-001`; `ENT-DEATH-001`; `ENT-EFFECT-001`; `MOB-AI-001`;
`MOB-BREED-001`; `MOB-RAID-001`; `CLI-UI-001`; `CLI-EFFECT-001`; `EXP-ITM-072`.

**Test vectors:**

Exercise default/food-removed/consumable-removed and arbitrarily patched Cookies through
interrupted/completed player use at every hunger/material boundary; verify Balanced Diet,
persistence and synchronization.

Match every row/offset/extra/wrong Wheat/Cocoa grid and both unlock routes. Generate, transact,
exhaust and restock fresh level-three Farmer sets; force each gift choice across exact
cooldown/range boundaries.

Offer default and patched Cookies with empty/nonempty/infinite hands to Parrots crossing
untamed/tamed, owner/nonowner, adult/baby, creative, invulnerable and damage/effect states.
Reload poisonous/food tags in every combination; record preemption, count/remainder, effect,
damage/death and absence of hunger/stat/criterion/eat-sound work.

Exercise every Composter level and below/equal/above chance draw on held and automated paths.
Reload all domains, then verify raw ID, name, ordinary generated model/texture, Allay advancement
icon and exact Food-tab neighborhood.
