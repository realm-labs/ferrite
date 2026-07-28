# Entities mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ENT-WANDERING-TRADER-001` - Wandering Traders sample three fixed trade sets and pause their own despawn while trading

**Parent:** `ENT-001`, `ENT-LIFECYCLE-001`, `ENT-002`,
`ENT-VEHICLE-001`, `ENT-004`, `ENT-PROJECTILE-001`, `ENT-005`,
`ENT-DAMAGE-001`, `ENT-BLOCK-001`, `ENT-DAMAGE-REDUCE-001`,
`ENT-KNOCKBACK-001`, `ENT-006`, `ENT-EFFECT-001`, `ENT-007`,
`ENT-DEATH-001`, `ENT-ENTITY-DROPS-001`, `MOB-001`, `MOB-AI-001`,
`MOB-002`, `MOB-SPAWN-001`, `MOB-003`, `MOB-DESPAWN-001`,
`MOB-BREED-001`, `MOB-WANDERING-TRADER-001`, `ITM-001`,
`ITM-CONTAINER-001`, `ITM-CONTAINER-CONTROL-001`, `ITM-LOOT-001`,
`ITM-ADVANCEMENT-001`, `ITM-POTION-001`,
`ITM-DRINK-CONTAINER-001`, `PLY-AUTOJUMP-001`, `WGEN-005`,
`WGEN-DIMENSION-001`, `WGEN-PORTAL-001`, `CLI-001`, `CLI-006`,
`CLI-UI-001`, `CLI-EFFECT-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` - locked registration, `WanderingTrader`,
`AbstractVillager`, age and merchant owners, the two item-use goals,
private wander goal, data-driven trade sets, custom spawner join, loot,
advancements, migrations, sounds and renderer close protocol entity ID
`142`.

**Applies when:**

`minecraft:wandering_trader` is constructed, finalized, produced by the
Wandering-Trader spawner, Spawn Egg, command, spawner or custom selector,
given a wander target, avoiding danger, drinking at an outside-light
transition, opening or closing a trade, sampling or completing an offer,
counting down subtype despawn, saving, loading, killed, synchronized,
heard or rendered.

**Authoritative state:**

Protocol entity ID `142` constructs `WanderingTrader` in `CREATURE`.
It remains available in Peaceful. Registration fixes scalable dimensions
`0.6x1.95`, explicit eye height `1.62`, client tracking range `10` and
default update interval `3`. It is neither fire-immune nor
persistence-required.

Attributes are the Mob defaults: maximum health `20`, movement speed
`0.7` and follow range `16`. No attack-damage attribute is installed.
Construction leaves XP reward `0`.

Generic adult dimensions are `0.6x1.95` with eye height `1.62`.
Ageable-Mob baby scaling would produce `0.3x0.975` and eye height `0.81`,
but ordinary finalization installs `AgeableMobGroupData(false)` and never
randomly creates a baby. `getBreedOffspring` returns null and the generic
ageable breeding gate is false.

Entity, Living-Entity and Mob metadata occupy slots `0..15`. Ageable Mob
adds Boolean slot `16`, `baby=false`, and Boolean slot `17`,
`age_locked=false`. Abstract Villager adds Integer slot `18`,
`unhappy_counter=0`. Wandering Trader adds no synchronized field.

Subtype state is:

- signed Integer `despawn_delay`, initialized to `0` and persisted as
  `DespawnDelay`;
- nullable codec `BlockPos wander_target`, persisted under
  `wander_target`;
- the inherited eight-slot `Inventory`;
- inherited, lazily initialized `Offers`, whose `Recipes` persist only
  after the server has initialized them; and
- inherited age and age-lock data.

The current trading player, goal instances, active navigation, hand-use
state and offer-menu session are transient. Offers retain uses, demand,
special-price and other `MerchantOffer` state through their owner.
Wandering Trader supplies merchant XP `0`; its offer and XP override
hooks are no-ops.

Loading calls the ageable owner and then assigns
`max(0, loaded_age)`. A saved negative baby age therefore becomes adult
age `0`; a positive cooldown age remains positive. The independent
age-lock bit can remain true.

**Transition and ordering:**

### Goal graph

The complete goal selector is:

| Priority | Goal and fixed inputs |
|---:|---|
| `0` | Float |
| `0` | use Invisibility Potion while outside is dark and trader is visible |
| `0` | use Milk Bucket while outside is bright and trader is invisible |
| `1` | trade with the current player |
| `1` | avoid Zombie within `8`, speeds `0.5/0.5` |
| `1` | avoid Evoker within `12`, speeds `0.5/0.5` |
| `1` | avoid Vindicator within `8`, speeds `0.5/0.5` |
| `1` | avoid Vex within `8`, speeds `0.5/0.5` |
| `1` | avoid Pillager within `15`, speeds `0.5/0.5` |
| `1` | avoid Illusioner within `12`, speeds `0.5/0.5` |
| `1` | avoid Zoglin within `10`, speeds `0.5/0.5` |
| `1` | Panic at speed `0.5` |
| `1` | look at the trading player |
| `2` | private wander-to-position, stop distance `2`, speed `0.35` |
| `4` | move toward restriction at speed `0.35` |
| `8` | water-avoiding random stroll at speed `0.35` |
| `9` | interact with Player within `3`, probability `1` |
| `10` | look at Mob within `8`, default probability |

The target selector has no registrations. Wandering Trader has no attack
goal. Shared goal flags, priority arbitration, avoidance, panic,
restriction and stroll behavior remain with `MOB-AI-001`.

`TradeWithPlayerGoal` can start only while the trader is alive, out of
water, on ground, not `hurtMarked`, has a trading player and is within
squared distance `16` of that player. Start stops navigation. Stop clears
the trader's current customer.

### Daylight item use

`isBrightOutside` is true only when the dimension has no fixed time and
`skyDarken < 4`. `isDarkOutside` is true only when the dimension has no
fixed time and bright-outside is false. A fixed-time dimension satisfies
neither predicate.

The two priority-zero item goals have no goal flags and their admission
predicates are mutually exclusive. Starting one:

1. overwrites the main hand with a copy of its fixed item;
2. begins using that item in `MAIN_HAND`; and
3. continues only while `isUsingItem`, without rechecking light or
   invisibility.

The Potion item is protocol item ID `1150`, with `minecraft:invisibility`
and duration `3600`. The Milk Bucket is protocol item ID `1046`. Both
default consumables take `1.6` seconds, or `32` ticks, use the Drink
animation and emit no consume particles. Potion completion applies
Invisibility and uses the Drink-Potion species sound. Milk completion
clears every effect and uses Drink-Milk.

Stop always empties the main hand and plays the goal's completion sound at
volume `1` and pitch `0.9 + nextFloat()*0.2`. Potion uses
`wandering_trader.disappeared`; milk uses
`wandering_trader.reappeared`. A Glass Bottle or Bucket use remainder is
therefore erased by stop. An interrupted use still clears the hand and
plays the completion sound, but applies no consume effect or remainder.
A preexisting main-hand stack is overwritten without a drop.

### Wander target

The private wander goal has the Move flag. It can start when
`wander_target` is nonnull and the trader is not within strict center
distance `2` of that block position.

While navigation is done:

1. if the target center is farther than `10`, normalize the vector from
   the trader toward the integer block position, scale it to `10`, add it
   to current position and navigate to that intermediate point at `0.35`;
2. otherwise navigate directly to the integer block coordinates at
   `0.35`.

Stop clears `wander_target` and stops navigation. The standard producer
sets the same point as the trader's home/restriction center with radius
`16`; clearing this private target does not clear that home, so the
priority-four restriction goal can continue returning toward it.

### Interaction and menu

`mobInteract` first reads the held hand stack. Its merchant branch runs
only if:

- the stack is not exactly `minecraft:villager_spawn_egg`;
- the trader is alive;
- it has no existing trading player; and
- it is not a baby.

A Wandering-Trader Spawn Egg does not suppress this branch; the exception
is specifically Villager Spawn Egg item protocol ID `1200`.

On `MAIN_HAND`, admission awards `minecraft:talked_to_villager`; offhand
does not. On the server, `getOffers` lazily generates offers. An empty
offer list returns `CONSUME` without binding a customer or opening a
menu. A nonempty list binds the player, opens the merchant title at level
`1`, sends XP `0`, hides the progress bar and reports no restocking.
The nonempty server result is `SUCCESS`. The client does not initialize
offers and returns `SUCCESS`.

The delegated Abstract-Villager interaction can age-lock a
command-created baby with Golden Dandelion because Wandering Trader is
not in `minecraft:cannot_be_age_locked`. A later load clamps negative age
to adult `0` while retaining the independent lock and persistence state.

Menu validity requires the same trading-player identity, a living trader
and distance within the generic entity interaction range `4`. Teleport,
death, an invalid trading-goal condition or menu closure clears the
session through the merchant owners.

### Offer construction

First lazy access appends offers in strict order from:

1. `minecraft:wandering_trader/buying`, amount `2`, six candidates;
2. `minecraft:wandering_trader/uncommon`, amount `2`, fifteen candidates;
3. `minecraft:wandering_trader/common`, amount `5`, seventy-six
   candidates.

Each set uses its own named random sequence under
`minecraft:trade_set/wandering_trader/`. `allow_duplicates` is omitted
and defaults false. For each selection, the algorithm removes one
uniformly chosen candidate before instantiating it. A candidate that
returns null consumes that candidate but no output slot; selection
continues until the requested amount is reached or the candidates are
empty. With ordinary valid data, the list contains nine offers.

The buying candidates are exactly:

| Cost | Result | Max uses |
|---|---|---:|
| Water Potion Bottle `1` | Emerald `1` | `2` |
| Water Bucket `1` | Emerald `2` | `2` |
| Milk Bucket `1` | Emerald `2` | `2` |
| Fermented Spider Eye `1` | Emerald `3` | `2` |
| Baked Potato `4` | Emerald `1` | `2` |
| Hay Block `1` | Emerald `1` | `2` |

The Water Potion cost requires its `minecraft:potion_contents` component
to select `minecraft:water`.

The uncommon candidates are Packed Ice, Blue Ice, Gunpowder, Podzol,
Acacia/Birch/Dark-Oak/Jungle/Oak/Spruce/Cherry/Mangrove/Pale-Oak logs,
an enchanted Iron Pickaxe and a Long-Invisibility Potion. The common set
is the exact seventy-six records referenced by the locked common-set tag;
the tag supplies candidates, not a new selection algorithm.

Across all ninety-seven records, ninety-six use
`reputation_discount=0.05`; the enchanted pickaxe uses `0.2`. Max-use
counts are:

| Max uses | Record count |
|---:|---:|
| `12` | `52` |
| `8` | `18` |
| `5` | `11` |
| `2` | `8` |
| `6` | `4` |
| `1` | `2` |
| `7` | `2` |

The enchanted Iron Pickaxe candidate applies `enchant_with_levels` using
uniform levels `5..19`, includes additional cost, and selects options
from `#minecraft:on_traded_equipment`. A filter then requires at least
one enchantment; failure returns null. It has max uses `1`. The special
potion result sets `minecraft:long_invisibility`, duration `9600`, costs
five Emeralds and has max uses `1`. Other candidates are plain item-stack
trades.

### Trade completion

Completing a trade increments that offer's uses, resets the merchant
ambient-sound timer and runs reward handling. Wandering Trader does not
restock, update demand, level up, use gossip, apply Hero pricing or add
merchant XP.

When the offer rewards experience, the trader's entity RNG computes
`3 + nextInt(4)` and inserts one Experience Orb at
`(x,y+0.5,z)`. The insertion result is ignored. Every current record
defaults reward-experience true.

After reward handling, a `ServerPlayer` customer triggers the `TRADE`
criterion. Locked advancement consumers are `adventure/trade` and
`adventure/trade_at_world_height`; the latter additionally requires
player Y at least `319`.

The merchant result sound is Yes for a nonempty result and No for an
empty result, subject to the Abstract-Villager ambient throttle.
`getNotifyTradeSound` is Yes. While trading, the ambient sound resolves
to Trade; otherwise it resolves to Ambient.

### Subtype despawn and llama join

On each server AI step, inherited AI runs first and `maybeDespawn` runs
afterward. If `despawn_delay > 0` and there is no trading player, it
predecrements the value and discards the entity exactly when the result
is `0`. Trading pauses the counter. Initial, loaded or commanded values
of `0` or less never change and never cause subtype discard.

The standard producer sets `despawn_delay=48000` after entity creation
and before ordinary ticking. Its first eligible nontrading AI step
therefore leaves `47999`. If inherited AI clears an invalid customer,
the counter can resume on that same step.

`removeWhenFarAway` is false. The subtype timer does not inspect
`persistenceRequired`; setting generic persistence does not stop a
positive timer. The customer pointer is transient while the delay
persists, so a loaded trader resumes counting when positive.

Production, placement, target/home assignment, the 48,000 value, insertion
and zero-to-two Trader-Llama attempts remain owned by
`MOB-WANDERING-TRADER-001`.

A leashed Trader Llama later sets its own eligible despawn delay to the
holder trader's current delay minus `1`. Trading pauses the trader's
counter, while that llama copies the paused value minus one on its own
eligible tick. The llama defend goal reads its leash-holder trader's
last attacker and timestamp. Trader-Llama lifecycle remains owned by its
own entity family.

Abstract Villager cannot itself be leashed, but supplies the rope-holder
point used by attached llamas: `(0,height-1,0.2)` rotated by body yaw.

### Production, loot and progression

The standard custom spawner is the only baseline producer. Across the
locked data there are:

- zero Wandering-Trader rows in all sixty-six biomes;
- zero rows in all twenty-eight Trial-Spawner configurations; and
- zero literal `minecraft:wandering_trader` or `WanderingTrader` payloads
  across all 1,212 structure templates.

The Spawn Egg is protocol item ID `1201`. Commands, spawners, Spawn Eggs
and custom factories remain alternate producer surfaces.

`entities/wandering_trader` has zero pools and random-sequence ID
`minecraft:entities/wandering_trader`. Death XP is `0`. There are no
direct entity-type tags and no hostile-kill advancement joins. Trading
supplies the two progression joins specified above.

### Protocol and client projection

Sound-event protocol IDs are:

| Event | ID |
|---|---:|
| Ambient | `1725` |
| Death | `1726` |
| Disappeared | `1727` |
| Drink Milk | `1728` |
| Drink Potion | `1729` |
| Hurt | `1730` |
| No | `1731` |
| Reappeared | `1732` |
| Trade | `1733` |
| Yes | `1734` |

`WanderingTraderRenderer` uses `ModelLayers.WANDERING_TRADER`, the shared
Villager model, shadow radius `0.5`, a Custom-Head layer and a
Crossed-Arms-Item layer. It extracts held-item state and projects unhappy
when `unhappy_counter > 0`. There are no profession, biome-type or level
layers. The active Potion or Milk Bucket in the main hand is visible
through the crossed-arms item layer.

The entity texture
`textures/entity/wandering_trader/wandering_trader.png` is `64x64`, 890
bytes, SHA-256
`cdb39102044e3bf4f15adceec509117848fb191752baaf0c643db1280e62d343`.
The Spawn-Egg texture is `16x16`, 279 bytes, SHA-256
`0366be3234cf03fa7ebd9d2007bac086350a72d9cb8a12abb66e1d591a674d83`.

**Gates:**

- Entity identity must be exactly `minecraft:wandering_trader`.
- Offer initialization and menu/customer mutation are server-authoritative.
- Item-use admission requires the non-fixed-time outside-light predicate
  and the opposite current invisibility state.
- The subtype despawn decrement requires a positive delay and no current
  trading player.
- The private target goal requires a nonnull target farther than strict
  center distance `2`.
- Trade interaction excludes only the exact Villager Spawn Egg, then
  requires alive, not already trading and adult.

**Branches and aborts:**

- Fixed-time dimensions admit neither automatic drink goal.
- An item-use interruption skips consumption but still runs destructive
  stop and its completion sound.
- Empty lazy offers consume server interaction without a customer/menu.
- Any invalid trading-goal condition stops navigation ownership and clears
  the customer.
- A null enchanted-pickaxe candidate consumes its candidate position and
  may leave that set short only after exhaustion.
- Private wandering waits while navigation is active, and stop clears the
  target even if navigation failed.
- Delay `0`, negative delay and active trading all bypass decrement.
- Failed orb insertion does not roll back offer use or criterion progress.

**Invariants:**

- No target-selector or attack-goal registration exists.
- Ordinary finalization never creates a baby and breeding creates no
  offspring.
- Exactly three trade sets append in buying, uncommon, common order.
- Each set samples without replacement from an independent named sequence.
- Wandering Trader never restocks and never exposes merchant progress.
- A positive subtype delay discards only on the predecrement to exactly
  zero.
- Generic persistence does not override subtype countdown.
- The existing spawner leaf remains the owner of creation and llama
  transactions.

**Constants and randomness:**

- Adult size/eye: `0.6x1.95/1.62`; hypothetical baby:
  `0.3x0.975/0.81`.
- Attributes/XP: `20/0.7/16/0`.
- Trade-set amounts/candidates: `2/6`, `2/15`, `5/76`.
- Consume time: `32` ticks; potion durations: `3600` and `9600`.
- Finish pitch: `0.9 + nextFloat()*0.2`.
- Trade XP: `3 + nextInt(4)`.
- Private wander: distance `2`, segment threshold/length `10`, speed
  `0.35`; home radius `16`.
- Standard subtype delay: `48000`.

**Side effects:**

- Goals replace and clear main-hand stacks, apply/clear effects, play
  sounds and mutate navigation.
- Interaction awards a stat, binds a customer and opens a merchant menu.
- Offer generation consumes three named random sequences.
- Trade completion increments uses, creates an orb and triggers criteria.
- Despawn discards the entity.
- Spawner-owned creation can create and leash Trader Llamas.

**Observability:**

Observe metadata slots `16..18`, age/lock, inventory, offers,
`despawn_delay`, `wander_target`, current customer, goal winner,
navigation request, main-hand use, effects, sounds, stats, menu packets,
offer order/state, orb value/insertion, criteria, removal and renderer
submissions. Observe named-sequence and entity-RNG cursors separately.

**Boundary cases and quirks:**

- The exception is Villager Spawn Egg, not this entity's own Spawn Egg.
- Loading a baby makes it adult but can leave it age-locked.
- Goal stop erases both a displaced original hand stack and a successful
  consume remainder.
- Light changes after item-use start do not cancel use.
- Trading can pause a trader while its llama keeps synchronizing a
  one-less delay.
- A persistence-required trader still reaches subtype discard.
- Offers are absent from a save until server-side lazy initialization.

**Failure semantics:**

Entity creation, insertion, menu opening, navigation, orb insertion and
spawner/llama failure retain their generic owner semantics. This subtype
adds no transaction or rollback. Failed item consumption still reaches
goal stop; failed offer instantiation consumes that candidate; failed orb
insertion leaves the completed trade committed.

**Client/server authority split:**

The server owns AI, daylight predicates, effects, inventories, offers,
customer identity, menus, stats, trade rewards, criteria, persistence and
discard. Client interaction predicts `SUCCESS` for the merchant branch
without initializing offers. Clients consume ordinary metadata, entity
events, sound and menu packets, then project the Villager model, unhappy
state and held item.

**Interaction with persistence:**

Entity UUID, position/motion/rotation, health/effects, equipment,
inventory, initialized offers, age/lock, custom data, `DespawnDelay` and
`wander_target` use their locked owners. Customer, active use/navigation,
goal state and RNG cursor are not restored. Schema `V1929` registers the
entity's Inventory and Offers/Recipes shapes. `V705` maps its Spawn Egg,
Entity-UUID migration includes it, and Block-Position migration renames
legacy `WanderTarget` to `wander_target`. The saved-data spawner timer is
separate from entity state.

**Evidence:**

- `net.minecraft.world.entity.EntityTypes`
- `net.minecraft.world.entity.ai.attributes.DefaultAttributes`
- `net.minecraft.world.entity.SpawnPlacements`
- `net.minecraft.world.entity.AgeableMob`
- `net.minecraft.world.entity.Mob`
- `net.minecraft.world.entity.npc.AbstractVillager`
- `net.minecraft.world.entity.npc.wanderingtrader.WanderingTrader`
- `net.minecraft.world.entity.npc.wanderingtrader.WanderingTrader$WanderToPositionGoal`
- `net.minecraft.world.entity.ai.goal.UseItemGoal`
- `net.minecraft.world.entity.ai.goal.TradeWithPlayerGoal`
- `net.minecraft.world.item.component.Consumable`
- `net.minecraft.world.item.component.Consumables`
- `net.minecraft.world.item.alchemy.Potions`
- `net.minecraft.world.item.trading.Merchant`
- `net.minecraft.world.item.trading.TradeSet`
- `net.minecraft.world.item.trading.TradeSets`
- `net.minecraft.world.entity.animal.horse.TraderLlama`
- `net.minecraft.world.level.Level`
- `net.minecraft.util.datafix.schemas.V705`
- `net.minecraft.util.datafix.schemas.V1929`
- `net.minecraft.util.datafix.fixes.ItemStackSpawnEggFix`
- `net.minecraft.util.datafix.fixes.EntityUUIDFix`
- `net.minecraft.util.datafix.fixes.BlockPosFormatAndRenamesFix`
- `net.minecraft.client.renderer.entity.EntityRenderers`
- `net.minecraft.client.renderer.entity.WanderingTraderRenderer`
- `net.minecraft.client.renderer.entity.state.HoldingEntityRenderState`
- `net.minecraft.client.model.VillagerModel`
- `reports/registries.json`
- `reports/minecraft/components/item/{milk_bucket,potion,villager_spawn_egg,wandering_trader_spawn_egg}.json`
- `data/minecraft/trade_set/wandering_trader/*.json`
- `data/minecraft/tags/villager_trade/wandering_trader/*.json`
- `data/minecraft/villager_trade/wandering_trader/**/*.json`
- `data/minecraft/loot_table/entities/wandering_trader.json`
- `data/minecraft/worldgen/biome/*.json`
- `data/minecraft/trial_spawner/**/*.json`
- `data/minecraft/structure/**/*.nbt`
- `data/minecraft/advancement/adventure/{trade,trade_at_world_height}.json`
- `assets/minecraft/textures/entity/wandering_trader/wandering_trader.png`
- `assets/minecraft/{items,models/item,textures/item}/wandering_trader_spawn_egg.*`
- `assets/minecraft/sounds.json`
- `assets/minecraft/lang/en_us.json`

**Test vectors:**

1. Finalize ordinary, Egg, command, spawner and custom traders; compare
   dimensions, attributes, ages, metadata, offers and producer-owned
   target/home/delay/llama state.
2. Cross every fixed-time, sky-darken and invisibility state; complete
   and interrupt both item goals with occupied hands and verify effects,
   remainders, sounds and RNG.
3. Tick private wandering on both sides of distances `2` and `10`, with
   active/done/failed navigation, then confirm target clearing and home
   continuation.
4. Interact from both hands with Villager and Wandering-Trader Eggs,
   empty/nonempty offers, adult/baby/locked/dead/trading states and both
   logical sides; verify result, stat, customer and menu fields.
5. Enumerate all ninety-seven candidates, force every draw permutation
   and enchanted-pickaxe null result, save/reload offers and exhaust uses.
6. Complete each offer with orb insertion success/failure and players at
   Y `318/319`; verify uses, XP, criteria and sounds.
7. Tick delay values `48000/1/0/-1` with valid and invalid customers,
   persistence on/off and save/load; join zero, one and two leashed
   Trader Llamas.
8. Census biome, Trial and template absence; exercise loot, migration,
   sound, texture and all unhappy/held-item render states.

**Limits:**

Generic movement, collision, damage, effects, item consumption, menus,
merchant-offer matching, named random sequences, spawner transaction,
Trader-Llama implementation, data reload, migrations and rendering
algorithms remain with their parent leaves. This rule fixes Wandering
Trader selectors, overrides, constants, ordering and cross-owner joins
without re-specifying those engines.
