# Items mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ITM-POTION-001` — Drinkable potion contents drive effects while Water alone owns bottle-to-block transactions

**Parent:** `PLY-005`, `PLY-006`, `PLY-INPUT-001`, `PLY-INTERACT-001`, `ITM-001`,
`ITM-003`, `ITM-004`, `ITM-005`, `ITM-006`, `ITM-007`, `ITM-USE-001`,
`ITM-CONTAINER-001`, `ITM-DISPENSER-001`, `ITM-BREW-001`, `ITM-LOOT-001`,
`ITM-ADVANCEMENT-001`, `BLK-003`, `ENT-001`, `ENT-006`, `MOB-004`, `CLI-001`,
`CLI-006`, `CLI-UI-001`, `CLI-EFFECT-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked registration/components, potion-content and item bytecode, cauldron/
dispenser/bottle consumers, brewing/progression/loot/trade data, mob consumers and client assets
determine the drinkable potion identity and every content-sensitive transaction. Potion registry
effect payloads remain exact data inputs to `ENT-EFFECT-001`, and generic active use remains with
`ITM-USE-001`.

**Applies when:**

A `potion` stack is created, filled, brewed, selected by loot/trade or mob AI, used in air or on a
block, dispensed, consumed, persisted, reloaded, named, described, tinted or projected in a
creative tab.

**Authoritative state:**

`minecraft:potion` is raw item ID `1150`, common, nondamageable, max stack `1`, and in no direct
item tag. Its registered prototype has `potion_contents={}`, the default 1.6-second drink
consumable and `use_remainder=glass_bottle`; it has no food component. In contrast,
`PotionItem#getDefaultInstance` installs the Water potion holder. A direct `ItemStack`/give-style
construction therefore starts empty, while APIs requesting the item default receive Water.

`potion_contents` is an ordered record of optional potion holder, optional integer custom color,
custom effect list and optional custom-name suffix. `potion_duration_scale` is a separate stack
component, absent/default `1`. Removing contents leaves a drinkable, bottle-returning stack with no
effects. Removing consumable prevents in-air use but does not remove cauldron or direct block-use
dispatch. Removing or replacing the use remainder changes only post-drink conversion.

The locked potion registry has 46 protocol-ordered holders, IDs `0..45`: Water, Mundane, Thick,
Awkward; Night Vision/Invisibility/Leaping/Fire Resistance/Swiftness/Slowness/Turtle Master/
Water Breathing/Healing/Harming/Poison/Regeneration/Strength/Weakness and their registered long or
strong forms; Luck; Slow Falling/Long Slow Falling; Wind Charged, Weaving, Oozing and Infested.
Their exact effect instances are `potion-data` owned by `ENT-EFFECT-001` and brewing edges by
`ITM-BREW-001`.

**Transition and ordering:**

In-air base use starts the default drink consumable for `32` ticks, independent of hunger.
Periodic generic drink sound attempts occur at remaining `24,20,16,12,8,4`; consume particles are
disabled, including completion. Natural completion follows `ITM-USE-001`: final drink sound,
used-item stat and criterion, component listeners, consume-effect list, `DRINK`, ability-gated
shrink, then use remainder and cooldown.

The potion-contents listener runs at the component-listener position. It does nothing client-side.
On the server it iterates base-potion effects first and custom effects second. Each instance is
cloned; infinite/zero duration is retained, otherwise duration becomes
`max(1,floor(duration*scale))`. Instantaneous effects call their instantaneous hook with factor
`1`, target equal to the user, and direct/indirect source equal to the user only when the user is a
player (otherwise both are null). Noninstant effects are offered through `addEffect` without a
source. All Boolean results are ignored, so effect rejection does not roll back consumption.

In survival a default count-one stack becomes one default glass bottle. Infinite-material use
retains the potion and creates no bottle. A malformed larger stack shrinks one and routes the
extra bottle through the generic living-entity remainder hook. Component changes during use follow
the exact revalidation/release rules in `ITM-USE-001`.

**Water block transaction:**

`PotionContents.is(WATER)` requires the Water holder and an empty custom-effect list. It ignores
custom color and custom name. This exact predicate is shared by cauldrons, mud conversion and the
dispenser; empty contents and Water plus any custom effect fail.

An empty cauldron accepts matching Water and a nonfull water cauldron at level `1` or `2` accepts
another. The client returns `SUCCESS` without mutation. The server transforms one potion through
`createFilledResult` to a glass bottle, awards `use_cauldron` and the potion item-used stat, writes
empty cauldron to water-cauldron level `1` or cycles `1→2→3`, then plays bottle-empty and emits
`FLUID_PLACE`. A full level-`3` water cauldron returns `TRY_WITH_EMPTY_HAND` before inspecting
contents. Non-Water also returns that result. Block-write success is ignored.

Outside the cauldron dispatcher, `PotionItem#useOn` succeeds only when the clicked face is not
Down, the clicked state is in `#minecraft:convertable_to_mud`, and contents match Water. On both
logical sides it first plays generic splash, immediately transforms the held stack to a glass
bottle result, then the server alone consumes ten doubles to emit five one-particle `SPLASH`
packets at `(x+random,y+1,z+random)`. It next plays bottle-empty, emits `FLUID_PLACE`, replaces the
clicked block with default Mud and ignores write success. Other cases return `PASS`.

`createFilledResult` has a different creative rule from drinking: an infinite-material player
retains the potion and, only if inventory does not already contain an equal glass bottle, attempts
to insert one; failed insertion is ignored. Survival consumes one, returns the bottle when empty,
otherwise inserts or drops it. Cauldron conversion invokes this only server-side; mud conversion
invokes it on both projections.

The potion dispenser owns the same Water predicate against the front
`#minecraft:convertable_to_mud` state. Success consumes ten doubles for five particles, plays
bottle-empty, emits `FLUID_PLACE`, writes Mud, and produces a glass bottle through the dispenser
remainder transaction. Every other potion/target invokes nested default ejection, including its
inner events before the outer dispenser events as specified by `ITM-DISPENSER-001`.

**Water acquisition and brewing:**

A held glass bottle first searches the player's bounding box inflated by `2` for alive area-effect
clouds owned by an Ender Dragon; any such cloud takes precedence and produces dragon breath.
Otherwise a source-fluid-only raycast that hits an interactable block whose fluid is in
`#minecraft:water` plays bottle-fill, emits `FLUID_PICKUP` and transforms one bottle into a Water
potion without removing fluid. Miss, permission failure or nonwater passes.

A glass bottle used on water-cauldron level `1..3` produces Water server-side, awards cauldron and
bottle-use stats, lowers the level, plays bottle-fill and emits `FLUID_PICKUP`; the client reports
success without mutation. A dispenser facing any water fluid likewise produces Water without
removing it. These acquisition paths construct explicit Water contents, not the registered empty
prototype.

Brewing slots admit potion identity directly. Container mixes can change it to splash/lingering
while preserving only the potion holder; potion mixes retain item identity and install the target
holder, discarding unrelated/custom content fields as specified by `ITM-BREW-001`. Taking any
brewing-slot stack with a present potion holder as a server player triggers `brewed_potion` with
that holder before ordinary slot `onTake`; empty contents do not trigger. The locked `brew_potion`
advancement accepts every holder.

**Loot and trade acquisition:**

Generic table selection, count splitting and placement remain with `ITM-LOOT-001`. The complete
standard-pack direct potion records are:

- Ancient-city first pool (`5..10` rolls) has Strong Regeneration potion weight `5/84`, requested
  count `1..3`.
- Buried-treasure potion pool has only Water Breathing potion and requested rolls `0..2`.
- Trial-chamber supply (`3..5` rolls) has Regeneration and Strength potion entries, each weight
  `1/18` and requested count `2`.
- Normal trial-spawner consumables select Regeneration or Swiftness, each `1/10`, in one roll;
  ominous consumables select Regeneration or Strength, each `1/10`, in one roll.
- Piglin bartering has Fire Resistance potion weight `8/469` and Water potion weight `10/469` in
  one roll.
- Fishing junk has Water potion weight `10`: effective denominator `100` outside the three jungle
  biomes and `110` in jungle/sparse-jungle/bamboo-jungle when the conditional bamboo entry joins.
  The parent fishing table's junk admission remains generic loot behavior.

The wandering trader's buying set selects two distinct records from six; its potion record wants
one stack whose `potion_contents` equals pure Water and gives one emerald, max uses `2`, multiplier
`0.05`. Other component types may be present, but custom color/name/effects inside contents fail
the exact specified component. The uncommon set selects two of 15; its potion record wants five
emeralds and gives one Long Invisibility potion, max uses `1`, multiplier `0.05`.

**Mob consumers:**

A wandering trader has a priority-zero item-use goal that starts a copied Invisibility potion when
the level is dark outside and the trader is visible. It uses the generic 32-tick potion transaction
and its own potion-drink sound; completion makes the trader invisible, then goal stop clears the
returned bottle and plays the disappeared sound at volume `1`, pitch `0.9+0.2*nextFloat`.
Its bright-and-invisible milk inverse is owned by `ITM-DRINK-CONTAINER-001`.

A living server witch that is not already drinking evaluates in order, consuming one new float per
reached test: `<0.15` Water Breathing when eyes are in water and the effect is absent; `<0.15`
Fire Resistance when on fire or last damage is fire and the effect is absent; `<0.05` Healing when
below max health; `<0.5` Swiftness when it has a target farther than squared distance `121` and no
Speed. First success installs a count-one explicit potion, sets `usingTime=32`, synchronizes
drinking, plays witch-drink unless silent at volume `1`, pitch `0.8+0.4*nextFloat`, and replaces
the drinking speed modifier with transient add-value `-0.25`.

Each later witch server tick post-decrements `usingTime`; old values above zero continue. The tick
observing old zero clears drinking/hand, clones base then custom effects at stack scale and offers
each directly through `addEffect`, emits `DRINK`, and removes the speed modifier. Thus this custom
path completes after the 32 positive countdown ticks plus the old-zero tick, has no glass-bottle
result and does not use the player's special instantaneous-effect call.

**Persistence boundary:**

Stack codec state retains identity, count, optional contents holder/color/effect list/name,
duration scale, consumable and use-remainder patches. Active generic use separately retains only
live entity/use state and is not a durable restart transaction. Brewing, cauldron, witch and
wandering-trader state persistence remains with their generic owners; potion contents that survive
there remain stack components.

**Client projection:**

If contents is present, the item name is
`item.minecraft.potion.effect.<custom_name-or-potion-name-or-empty>`; absent contents falls back to
the base potion item name. Custom name is a translation-key suffix, not literal display text.
Tooltip lists base effects then custom effects, scales displayed duration by stack duration scale
and client tick rate, suppresses duration when the source instance ends within 20 ticks, and emits
gray `effect.none` when no effect exists. It then lists aggregate effect attribute modifiers under
the when-drank heading with positive blue and negative red formatting.

Tint selects custom color, otherwise amplifier-weighted visible-effect color, otherwise
`-13083194`. The generated two-layer item model tints `potion_overlay` and leaves the bottle layer
untinted.

Food & Drinks order is milk, honey, five ominous levels, then 46 potion entries, 46 splash entries
and 46 lingering entries. Potion generation iterates every enabled registry holder in protocol
order and emits `PARENT_AND_SEARCH_TABS`; all 46 are enabled in the locked baseline. No empty
prototype entry is projected.

**Branches and aborts:**

Contents/consumable/remainder presence; base/custom effect order, duration/scale, instantaneous and
player/nonplayer user; effect acceptance; ability/count/inventory; block dispatcher/state/level,
face/tag/Water predicate/side/write; dispenser target; bottle cloud/raycast/permission/fluid;
brewing holder; loot conditions/weights/counts; trade-set/component predicate; mob state and every
ordered RNG gate; client contents/effects/color/registry holder.

**Constants and randomness:**

Raw ID `1150`; max stack `1`; drink `1.6` seconds/`32` ticks; periodic sound boundaries
`24..4` by four; scale default `1`; potion holders `46`; bottle cloud inflation `2`; cauldron
levels `1..3`; mud particles `5` consuming `10` doubles. Witch tests
`0.15/0.15/0.05/0.5`, distance squared `121`, speed `-0.25`, drink pitch
`0.8+0.4u`; trader finish pitch `0.9+0.2u`. Loot/trade constants are exact above.

**Side effects:**

Use state and sounds; stats/criteria; effects/attributes; stack count/identity/components and
bottle inventory/drop; cauldron/Mud block state; particles/game events; dispenser residue/events;
brewing/loot/trade outputs; mob equipment/flags; saved state; name/tooltip/tint/model/tab entries.

**Gates:**

Interaction/cooldown and active-use equality; server effect side; potion contents and target
effect admission; Water holder plus no custom effects; block state/face/tag and interaction
permission; cauldron level; dispenser state; bottle cloud/raycast/fluid; brewing holder; loot/trade
conditions; mob alive/state/effects/environment/target/RNG; enabled client holder/resource.

**State read/written:**

Reads stack components, hands/inventory/ability, active-use state, living effects and server level,
clicked state/face/fluid/permissions, cauldron level, dispenser slot/front state, brewing slots,
loot/trade records, mob environment/health/fire/target/effects/attributes, registries and client
resources. Writes the stated stack/effect/inventory/block/dispenser/brewing/mob/client projection
state.

**Failure behavior:**

Missing consumable passes in-air; interrupted or component-divergent use follows generic release
without effects/remainder. Client effect application is absent. Rejected effects do not undo
consumption. Nonmatching Water/block/face returns pass or cauldron empty-hand fallback as stated.
Write failure is ignored after item/stat/effect ordering. Failed creative bottle insertion loses
the attempted extra. Failed loot/trade/mob gates emit no replacement result.

**Boundary cases and quirks:**

Registered empty and API-default Water are distinct. Water custom color/name still pours, but one
custom effect blocks every Water transaction. Full water cauldron rejects before reading contents.
Mud conversion transforms the client hand before server authority, whereas cauldron mutation is
server-only. Creative drinking returns no bottle, but creative pouring tries to ensure one bottle
in inventory. Loot may request counts above max one and delegates splitting. Witch drinking is a
custom 33-observation countdown and discards its bottle; wandering trader uses generic active use.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-DATA-001`; `OFF-REPORT-001`;
`net.minecraft.world.item.Items`; `net.minecraft.world.item.PotionItem`;
`net.minecraft.world.item.BottleItem`; `net.minecraft.world.item.ItemUtils#createFilledResult`;
`net.minecraft.world.item.alchemy.PotionContents`;
`net.minecraft.world.item.alchemy.PotionBrewing`;
`net.minecraft.world.item.component.Consumables`;
`net.minecraft.core.cauldron.CauldronInteractions`;
`net.minecraft.core.dispenser.DispenseItemBehavior`;
`net.minecraft.world.inventory.BrewingStandMenu$PotionSlot`;
`net.minecraft.advancements.triggers.BrewedPotionTrigger`;
`net.minecraft.world.entity.ai.goal.UseItemGoal`;
`net.minecraft.world.entity.monster.Witch`;
`net.minecraft.world.entity.npc.wanderingtrader.WanderingTrader`;
`net.minecraft.world.item.CreativeModeTabs`;
`reports/registries.json#minecraft:{item,potion}`;
`reports/minecraft/components/item/potion.json`;
`data/minecraft/loot_table/{chests,gameplay,spawners}/**`;
`data/minecraft/{trade_set,tags/villager_trade,villager_trade}/wandering_trader/**`;
`data/minecraft/advancement/nether/brew_potion.json`;
`assets/minecraft/items/potion.json`; `assets/minecraft/models/item/potion.json`;
`assets/minecraft/textures/item/{potion,potion_overlay}.png`;
`PLY-INTERACT-001`; `ITM-USE-001`; `ITM-CONTAINER-001`; `ITM-DISPENSER-001`;
`ITM-BREW-001`; `ITM-LOOT-001`; `ITM-ADVANCEMENT-001`; `ENT-EFFECT-001`;
`MOB-AI-001`; `CLI-UI-001`; `CLI-EFFECT-001`; `EXP-ITM-033`.

**Test vectors:**

Cross absent/empty/Water/every holder/custom color/name/effect and duration scales with default/
removed/patched consumable/remainder, both hands, sides and abilities through every active-use
boundary. Apply persistent/instant effects to player/nonplayer accepted/rejected targets. Pour into
empty/full water and other cauldrons plus every face/tag/write result; dispense against all front
states and fill from cloud/water/cauldron. Exhaust brewing holder/take, every loot/trade boundary
and witch/trader condition/RNG/countdown. Persist/reload every stack and capture all names,
tooltips, tints, models and 46-entry tab order before/after data/resource reload.

**Limits:**

This leaf does not duplicate generic active use/remainder, effect merge/tick, cauldron block,
dispenser scheduling, brewing graph, loot evaluator, villager economy, mob scheduler, stack codec
or client resource algorithms. Those remain with the cited owners; this rule fixes the potion item
identity, contents projection and every exact identity-sensitive join.
