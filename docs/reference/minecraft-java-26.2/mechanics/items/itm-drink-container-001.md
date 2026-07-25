# Items mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ITM-DRINK-CONTAINER-001` — Milk and honey drinks separate effect clearing from container ownership

**Parent:** `PLY-005`, `PLY-006`, `PLY-INPUT-001`, `PLY-INTERACT-001`, `ITM-001`,
`ITM-003`, `ITM-004`, `ITM-005`, `ITM-006`, `ITM-007`, `ITM-USE-001`,
`ITM-CONTAINER-001`, `ITM-DISPENSER-001`, `ITM-RECIPE-001`, `ITM-CRAFT-001`,
`ITM-LOOT-001`, `ITM-ADVANCEMENT-001`, `ITM-HUNGER-001`, `ENT-001`, `ENT-006`,
`MOB-004`, `WGEN-003`, `CLI-001`, `CLI-006`, `CLI-UI-001`, `CLI-EFFECT-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked item/component registration, finish/remainder bytecode, animal and hive
interactions, dispenser behavior, recipes, loot/trade/progression data and client assets close both
identities and their shared and divergent transactions.

**Applies when:**

A milk bucket or honey bottle is obtained, stacked, used, interrupted, consumed, crafted, looted,
traded, persisted, reloaded or rendered; an empty bucket or glass bottle is used on its locked
source; or the owning loot/trade/recipe/advancement data is reloaded.

**Authoritative state:**

| item | raw item ID | max stack | food | consumable | use/craft remainder |
|---|---:|---:|---|---|---|
| `milk_bucket` | `1046` | `1` | absent | drink `1.6` s / `32` ticks; generic drink sound; no particles; clear all effects | default `bucket` |
| `honey_bottle` | `1412` | `16` | nutrition `6`, saturation `1.2`, always edible | drink `2.0` s / `40` ticks; honey-drink sound; no particles; remove Poison | default `glass_bottle` |

Both are common, nondamageable ordinary `Item` instances in no direct item tag. Their use remainder
is a stack component and can be removed or patched. Their crafting remainder is separately fixed
on the registered item and therefore still applies to component-patched ingredient stacks.

Removing `consumable` makes in-air use fall through. Removing milk's remainder makes successful use
end empty in survival. Removing honey's food component retains its consumable and Poison-removal
effect but removes hunger admission/listener work; removing its consumable makes the remaining food
component unusable through ordinary `Item#use`.

**Transition and ordering:**

### Start, cadence and interruption

Milk has no food component and is always admitted. Honey has `can_always_eat=true`, so it is also
admitted at food level `20`; generic player invulnerability would independently admit food even
without that flag. Both begin a `drink` animation with result `CONSUME`.

The per-tick sound predicate is generic: elapsed time must be strictly greater than
`floor(total * 0.21875)` and remaining time divisible by four. Milk therefore emits at remaining
ticks `24/20/16/12/8/4`; honey emits at `28/24/20/16/12/8/4`. Each call requests five item
particles, but the disabled flag emits none. Each still consumes one Boolean, one triangular sample
centered at `1` with deviation `0.2`, and one uniform pitch float in `[0.9,1.0)`, then plays its
configured drink sound at volume `0.5` and that uniform pitch. The Boolean and triangular values do
not affect drink output.

Interruption, release or live-hand replacement before completion commits no final sound, statistic,
criterion, food/effect mutation, game event, input consumption or remainder.

### Shared finish and remainder transaction

At server completion with the same live stack:

1. emit a final consume burst requesting `16` particles; particles remain absent, the three generic
   random operations occur and the configured drink sound plays at volume `0.5` with pitch
   `[0.9,1.0)`;
2. for a server player, award the drink identity's item-used statistic and trigger `consume_item`
   against the live pre-shrink stack;
3. invoke the stack's `ConsumableListener` components;
4. server-side, apply the configured consume-effect list;
5. emit the `DRINK` game event;
6. consume one unless the living entity has infinite materials;
7. after the item returns, inspect the pre-use copy's use-remainder component. Infinite-material
   users retain the original stack and create no remainder. Otherwise, if the returned count fell
   below the pre-use count, create one default remainder.

If consumption empties the stack, the new bucket or glass bottle becomes the hand result. If a
honey stack still contains bottles, it stays in hand and a server player inserts the extra glass
bottle into inventory or drops it when insertion fails. The base nonplayer living hook discards
that extra remainder. The remainder is default and does not inherit arbitrary components from the
consumed drink.

### Milk finish

Milk has no default consumable listener. Its sole server consume effect calls
`removeAllEffects`; empty and nonempty effect maps both continue through `DRINK`, consumption and
remainder. Removal uses `ENT-EFFECT-001` callbacks and synchronization and includes beneficial,
harmful, ambient, hidden-chain and otherwise active effects without selecting by registry tag.
The effect-removal result is ignored. A normal count-one use therefore replaces milk with one
default bucket; infinite-material use performs the statistic, criterion and complete effect/event
transaction but retains the milk bucket.

### Honey finish

Honey's food component is its default listener. Before Poison removal it:

1. plays a second `item.honey_bottle.drink` sound in the neutral category at volume `1`, with pitch
   sampled from a triangular distribution centered at `1` with deviation `0.4`;
2. for a player, clamps food level after adding `6` into `0..20`, then clamps saturation after
   adding `1.2` into `0..new food level`;
3. for a player, plays `entity.player.burp` in the player category at volume `0.5` and uniform
   pitch `[0.9,1.0)`.

The server consume effect then iterates its one-element holder set and removes Poison. Other effects
remain. No Poison present and rejected removal do not roll back the later event, shrink or
remainder. A nonplayer gets the additional neutral honey sound and Poison removal but no hunger or
burp work.

`consume_item` precedes nutrition and Poison removal, so the Balanced Diet criterion observes the
pre-shrink honey identity. Its honey requirement is one of 40 separate AND requirements; consuming
honey progresses only that requirement.

### Milk acquisition

An empty-bucket item identity on an adult cow, mooshroom or goat takes precedence in that entity's
admitted interaction path; arbitrary bucket components are accepted but not transferred. Babies
fall through to generic animal interaction. The transaction changes no animal state and has no
cooldown, so an adult can be milked repeatedly:

- cow and mooshroom use `entity.cow.milk`; an adult mooshroom's bowl/shears/flower-specific paths
  precede the inherited bucket path;
- a normal goat uses `entity.goat.milk`, while a screaming goat uses
  `entity.goat.screaming.milk`.

Each sound plays at volume/pitch `1/1`. The filled-result helper uses its infinite-material-aware
mode. In survival it consumes one bucket: a count-one hand becomes the default milk bucket, while a
remaining bucket stack stays in hand and the milk is inserted or dropped. For an
infinite-material player the empty bucket remains; if the inventory does not already contain a
matching default milk bucket, one insertion is attempted and insertion failure is ignored rather
than dropped. If it already contains one, no additional milk is created.

The trial-chamber supply chest is the only locked milk loot entry. Its pool makes `3..5` independent
rolls over total weight `18`; milk has weight `1`, count one and default components, so multiple
rolls can select it. The wandering-trader buying trade set selects two entries from its six-member
tag under the generic trade-set algorithm; the milk record wants one milk-bucket identity with an
empty component predicate, gives two default emeralds, has maximum uses `2` and reputation discount
`0.05`. Component-patched milk therefore satisfies this cost.

### Honey acquisition

Player block use requires a glass-bottle item identity and a `BeehiveBlock` at honey level `5`;
arbitrary bottle components are accepted but not transferred. On both logical paths it directly
shrinks one glass bottle even for infinite-material players, plays
`item.bottle.fill` in the block category at volume/pitch `1/1`, installs one default honey bottle
in an emptied hand or inserts/drops it beside a remaining bottle stack, and emits `FLUID_PICKUP`.
The server then awards the glass bottle's item-used statistic.

After output is committed, smoke controls the hive consequence:

- a smokey position resets honey to `0` while retaining occupants;
- without smoke, any nearby-bee anger hook runs when the hive contains occupants, then the block
  resets honey and attempts emergency release with the player as target context.

The successful result is not rolled back by empty occupant state or release failure. Honey below
level `5` or wrong input falls through without the scoped mutations.

`Safely Harvest Honey` listens to the generic post-use-on-block event. It requires the pre-use item
to be a glass bottle, the block to be in `#minecraft:beehives`, and the location to be smokey.
Because the block path must first return consuming success, level-5 smokey harvesting satisfies it;
the criterion does not test the produced honey stack.

A dispenser facing a locked beehive/bee nest at honey level `5` instead resets honey and invokes
`BEE_RELEASED` with no player, independent of smoke; it then emits `FLUID_PICKUP` at the dispenser
position and consumes one glass bottle. A single input becomes honey in the selected slot; with
remaining bottles, output is inserted into the dispenser or dispensed outward on insertion failure.
The optional behavior marks success. A nonready hive falls through to the glass-bottle water-fill
test and otherwise default-dispenses the bottle; it creates no honey. No player statistic or
advancement runs on the dispenser path.

Each evaluation of the normal trial-vault `reward_common` table selects one entry over total weight
`25`; honey has weight `3` and a uniform inclusive count `1..2`. A complete normal vault reward
always makes `1..3` common evaluations and has one additional common evaluation with probability
`1/5`, so one transaction can emit multiple honey stacks. No ominous common table contains honey.

### Recipes and progression

The locked recipes and default per-slot crafting remainders are:

- cake: shaped `AAA/BEB/CCC`, where `A` is three milk buckets, `B` sugar, `E`
  `#minecraft:eggs` and `C` wheat; output one cake and three buckets;
- honey bottle: shapeless one honey block plus four glass bottles; output four honey bottles and no
  ingredient remainder;
- honey block: shaped `2x2` honey bottles; output one honey block and four glass bottles;
- sugar from honey bottle: shapeless one honey bottle; output three sugar and one glass bottle.

The cake unlock is driven by possession of any egg tag member, not milk. Possessing a honey block
unlocks the honey-bottle recipe; possessing a honey bottle independently unlocks the honey-block
and sugar recipes. Recipe-unlocked alternatives remain valid under `ITM-ADVANCEMENT-001`.
Component patches on milk/honey inputs are not copied to outputs or default crafting remainders.

### Persistence and client projection

The generic stack codec persists identity, count and component patch. Partially completed use is not
durable. Animal milking creates no source state. Hive honey/occupant persistence remains with its
block entity; generated loot/trades/recipes and advancements use their active data snapshots.
Reload changes future evaluations without rewriting emitted stacks or active effects/hunger.

Both items have direct `generated` models and one matching texture, with no component-based model
selection or specialized tooltip provider. Food & Drinks places default milk then honey after the
generated suspicious stews and before the five ominous bottles. Tools & Utilities also places milk
after powder-snow bucket and before fishing rod. Honey has no second locked tab entry.

**Branches and aborts:**

Milk/honey; default/removed/patched food, consumable and remainder; count `1/16`; both hands and
ability modes; player/nonplayer; every interruption tick; empty/nonempty/merge-sensitive effect
maps; every hunger/saturation boundary; adult/baby cow/mooshroom/normal/screaming goat; bucket
count/inventory match/insertion; hive level `4/5`, smoke and occupants; player/dispenser; all
loot/trade/recipe/advancement and reload branches.

**Constants and randomness:**

Raw IDs `1046/1412`; max stacks `1/16`; durations `32/40`; cadence remaining ticks milk
`24/20/16/12/8/4`, honey `28/24/20/16/12/8/4`; cadence/final particle requests `5/16` with zero
emitted; primary drink volume `0.5`, pitch `[0.9,1.0)`; honey listener sound volume `1`, triangle
`1 +/- 0.4`; burp volume `0.5`, pitch `[0.9,1.0)`; nutrition/saturation `6/1.2`; honey level
`5 -> 0`; supply rolls `3..5`, milk weight/total `1/18`; normal common honey weight/total `3/25`,
count `1..2`; full normal-vault common evaluations `1..3` plus `1/5`; milk trade input/output/max
uses/discount `1/2/2/0.05`.

**Side effects:**

Using state and sounds; statistics/criteria; hunger/saturation/effect callbacks; `DRINK` and
`FLUID_PICKUP` events; held/inventory/dropped inputs, outputs and remainders; hive honey/occupant/bee
state; dispenser contents/output; recipes/loot/trade/progress; durable stacks/effects/hunger/hive
state; model/tab projection.

**Gates:**

Component presence and generic use ownership; unchanged live hand and timer; player ability;
effect/hunger state; exact bucket/bottle and adult/source subtype; inventory capacity/match; hive
class/tag/property/honey/smoke/occupants; dispenser facing/insertion; current loot/trade/recipe/
advancement and resource snapshots.

**State read/written:**

Reads item/count/components, use/hand/ability/RNG, effect and hunger maps, animal age/type, player
inventory, hive state/block entity/environment, dispenser inventory/facing, loot/trade/recipe
contexts and client resources. Writes held/inventory/item entities, statistics/criteria, hunger/
effects/events/sounds, hive honey/occupants/bee targets, dispenser state, crafting/loot/trade outputs
and durable state.

**Failure behavior:**

Missing consumable falls through; interruption commits no finish work. Missing use remainder leaves
the survival result empty or decremented. Effect removal failure does not roll back. Baby/wrong
animal or item falls through. Hive level below five or wrong input produces no honey. Output
insertion falls back to a drop except the explicitly ignored creative-milking insertion. Missing or
replaced data can remove future loot/trades/recipes/progress without rewriting existing items.

**Persistence boundary:**

Stacks, player hunger/effects and hive block entities have separate codecs. Active use, interaction,
craft, loot and trade transactions do not resume after completion. Milking has no persisted
depletion. Data/resource reload replaces future interpretation/projection only.

**Boundary cases and quirks:**

Creative milking retains the bucket and creates at most one inventory milk bucket, while creative
hive harvesting directly consumes the glass bottle. Infinite-material drinking retains the filled
drink and creates no empty container. Honey adds food and its extra sounds before removing Poison.
Milk clears every effect but does not add nutrition. Use remainder is patchable per stack; crafting
remainder belongs to the item registration. The safe-harvest advancement observes the pre-use glass
bottle, not the honey output.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-DATA-001`; `OFF-REPORT-001`;
`net.minecraft.world.item.Items`; `net.minecraft.world.item.Item`;
`net.minecraft.world.item.ItemStack`; `net.minecraft.world.item.ItemUtils`;
`net.minecraft.world.item.component.Consumable`; `net.minecraft.world.item.component.Consumables`;
`net.minecraft.world.item.component.UseRemainder`; `net.minecraft.world.food.FoodProperties`;
`net.minecraft.world.food.FoodData`;
`net.minecraft.world.item.consume_effects.ClearAllStatusEffectsConsumeEffect`;
`net.minecraft.world.item.consume_effects.RemoveStatusEffectsConsumeEffect`;
`net.minecraft.world.entity.animal.cow.AbstractCow`;
`net.minecraft.world.entity.animal.cow.MushroomCow`;
`net.minecraft.world.entity.animal.goat.Goat`;
`net.minecraft.world.level.block.BeehiveBlock`;
`net.minecraft.core.dispenser.DispenseItemBehavior`;
`net.minecraft.world.item.crafting.CraftingRecipe`;
`net.minecraft.world.item.trading.TradeSet`; `net.minecraft.world.item.trading.ItemCost`;
`net.minecraft.world.item.CreativeModeTabs`;
`reports/registries.json#minecraft:item/{milk_bucket,honey_bottle}`;
`reports/minecraft/components/item/{milk_bucket,honey_bottle}.json`;
`data/minecraft/loot_table/chests/trial_chambers/{supply,reward_common,reward}.json`;
`data/minecraft/{trade_set,tags/villager_trade,villager_trade}/wandering_trader/**`;
`data/minecraft/recipe/{cake,honey_bottle,honey_block,sugar_from_honey_bottle}.json`;
`data/minecraft/advancement/{husbandry/{safely_harvest_honey,balanced_diet},recipes/**/{cake,honey_bottle,honey_block,sugar_from_honey_bottle}}.json`;
`assets/minecraft/{items,models/item,textures/item}/{milk_bucket,honey_bottle}.*`;
`ITM-USE-001`; `ITM-CRAFT-001`; `ITM-LOOT-001`; `ITM-ADVANCEMENT-001`;
`ITM-HUNGER-001`; `ITM-DISPENSER-001`; `ENT-EFFECT-001`; `BLK-VAULT-001`;
`CLI-EFFECT-001`; `EXP-ITM-031`.

**Test vectors:**

Use both identities with every relevant component patch, count, hand, ability, interruption, hunger
and effect state; assert sound RNG, statistic/criterion, listener/effect/event/shrink/remainder
order and player/nonplayer extra-container ownership. Exhaust every animal/source/inventory and
hive/smoke/occupant/player/dispenser branch. Exhaust loot/trade/recipe/progression boundaries,
persist/reload all durable states, reload data/resources and inspect models/tabs/tooltips.

**Limits:**

This leaf does not duplicate generic use, effect/hunger, hive occupant release, dispenser,
crafting, loot, trade-set, advancement, persistence or rendering algorithms. Those remain with the
cited owners; this rule fixes the two drink-container identities and their exact joins.
