# ENT-007 Death Runtime

`G01-P7-S006` implements the protocol-neutral `ENT-DEATH-TRANSACTION-001` transition layer. The
owning Region supplies authoritative entity, inventory, rule and random-stream observations. The
runtime returns ordered decisions for death protection, death entry, drops, experience and removal;
it does not mutate a second entity world or draw ambient randomness.

## Responsibility split

`ferrite-gameplay::entity::runtime::ent_007` has five owners:

- `protection` owns bypass admission, main-before-off hand selection, pre-shrink copying, player and
  non-player effect order, fixed Totem effects and client event 35 presentation;
- `entry` owns ordinary re-entry rejection, kill-score ordering, causing-entity callbacks, zombie
  conversion suppression, wither roses, client event 3, server-player teardown and non-server
  player/dragon killing-blow branches;
- `drops` owns loot gates and context, item construction, equipment order/chance/damage mutation,
  subtype overrides, Nether Star age, and player inventory/item drops;
- `experience` owns player/mob eligibility, equipment reward mutation, greedy XP splitting and
  iteration-order orb merge selection;
- `timelines` owns common, heart-bound Creaking and Ender Dragon death ticks plus the post-player-
  death ender-pearl vanish gate.

## Protection and entry order

Damage types that bypass invulnerability return before either hand is inspected. Otherwise the main
hand wins, then the offhand; the full stack is copied before one held item is consumed. Player
statistics, criterion and interaction vibration precede health/effect mutation. Totem protection
sets health to one, clears effects, consumes the effect-list draw, adds Regeneration II for 900
ticks, Absorption II for 100 and Fire Resistance I for 800, then broadcasts event 35. The client
always emits thirty particles and a local sound, while a local player rescans main then offhand and
constructs a fallback Totem presentation if neither current stack matches.

Ordinary death rejects removed or already-dead entities. Kill score is awarded before sleep/use
shutdown, logging, killing-blow handling and combat recheck. A successful zombie-villager
conversion suppresses `ENTITY_DIE`, drops and wither-rose handling, but event 3 and the dying pose
remain. Normal consumes its skip draw; Hard always attempts. Charged Creeper skull bookkeeping is
inside its loot gate. A valid wither rose uses block flags three and ignores the placement result;
all other credited cases spawn an item with zero pickup delay.

Server players retain their separate path: death-message/team visibility, shoulder timing,
neutral-mob forgiveness, objectives and kill credit precede spectator-gated inventory/XP work.
Inventory destruction visits prevent-drop stacks first, then ordinary indices and equipment enum
order. Player items start at `eyeY-0.30000001192092896`, have pickup delay forty and use exactly two
victim-stream floats. The branch resets death state, fire/frozen state, combat and last-death
location, broadcasts event 3, and marks the client unloaded without setting ordinary dead/dying
state.

## Drops, experience and timelines

Common living loot requires adult plus `mob_drops`; Monster removes the adult gate. Loot context
keeps the entity, origin, source, attacking/direct entities, seed and recent-player luck. Ordinary
items use pickup delay ten and four construction draws. Equipment visits eight stable slots, uses
default chance `0.085`, preserves chances above one, adds `0.01*looting` only for player-held
looting, compares strictly below the adjusted chance, and applies the nested two-draw damage
formula only to non-preserved damageable stacks. Fox, Allay, horses and Copper Golem keep their
unconditional overrides; Piglin, Enderman and Wither remain inside the loot gate. A constructed
Nether Star starts at age `-6000`.

Player XP is `min(level*7,100)` unless keep-inventory or spectator applies. Other owners require
recent-player memory, their owner-specific age predicate, `shouldDropExperience` and `mob_drops`.
Eligible non-saddle equipment with chance at most one adds `1+nextInt(3)` in slot order, except for
Hoglin and Piglin. XP uses the exact greedy thresholds and one `nextInt(40)` per piece; the first
iteration-order candidate with equal value, live state and matching ID residue is merged.

The common timeline removes on increment to tick twenty, broadcasts event 60 and emits twenty poof
particles with sixty Gaussian and sixty position draws. Heart-bound tearing Creaking removes as
discarded on increment from 45 to 46 with its 100+10 particles and death sound. Dragon death
updates the fight every tick, emits event 1028 on the first server non-silent tick, creates one
three-draw particle on ticks 180 through 200, pays periodic eight-percent XP after tick 150 and
also the final twenty percent at tick 200, then notifies the fight, removes as killed and emits
`ENTITY_DIE` in that order.

## Validation

`crates/ferrite-gameplay/tests/slices/entities/ent_007.rs` owns the source-specified death slice.
Its fifteen tests cover protection scans/effects/client fallback; death admission and callback
suppression; conversion and skull gates; wither roses; player branches; loot context; item,
equipment and subtype drops; XP eligibility/reward/split/merge; all three removal timelines; dragon
reward accumulation; and ender-pearl vanishing. `G01-P7-B1` remains responsible for composing these
decisions with Region entity storage, effects, inventories, loot evaluation and protocol projection.
