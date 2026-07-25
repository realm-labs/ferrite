# Items mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ITM-OMINOUS-BOTTLE-001` — Ominous bottles bind a bounded component level to Bad Omen and trial-vault acquisition

**Parent:** `PLY-005`, `PLY-006`, `PLY-INPUT-001`, `ITM-001`, `ITM-003`,
`ITM-004`, `ITM-005`, `ITM-006`, `ITM-007`, `ITM-USE-001`, `ITM-LOOT-001`,
`ITM-ADVANCEMENT-001`, `ENT-006`, `MOB-001`, `WGEN-003`, `CLI-001`, `CLI-006`,
`CLI-EFFECT-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked item/component registration, consumable and effect-listener bytecode,
loot functions/tables, vault reward composition, effect owners and client assets close the bounded
level, finish transaction, acquisition and projection.

**Applies when:**

An ominous bottle is created, looted, used, interrupted, consumed, persisted, reloaded, described
or rendered; its amplifier or consumable component is patched; or the owning entity/vault loot data
is reloaded.

**Authoritative state:**

`minecraft:ominous_bottle` is raw item ID `1536`, an ordinary nondamageable `Item` of uncommon
rarity and maximum stack size `64`. Its default semantic components are:

- `minecraft:ominous_bottle_amplifier = 0`, whose persistent codec admits only integers `0..4`
  and whose network codec is a VarInt;
- a `minecraft:consumable` with `1.6` seconds (`32` ticks), `drink` animation,
  `entity.generic.drink` sound, no consume particles and one server-side
  `play_sound(item.ominous_bottle.dispose)` finish effect.

It has no `food` or use-remainder component, no recipe remainder and no direct item tag. Removing
or replacing either semantic component changes the generic component-driven behavior: without a
consumable it is no longer usable, while a consumable without the amplifier can finish and consume
without adding Bad Omen. The amplifier codec rejects persisted values outside `0..4`; the locked
loot function evaluates its number provider as an integer, clamps the result into that range and
sets the component.

**Transition and ordering:**

### Start, cadence and interruption

Generic in-air use calls the consumable. Because the stack has no food component, its admission is
not gated by hunger and begins a `32`-tick use in either hand with result `CONSUME`. The ordinary
using-item identity/hand checks and replacement/cancellation behavior remain with `ITM-USE-001`.

Each admitted use tick calls the stack before decrementing remaining time. The consumable emits at
remaining ticks `24`, `20`, `16`, `12`, `8` and `4`: elapsed ticks must be strictly greater than
`floor(32 * 0.21875) = 7`, and remaining time must be divisible by four. Each call requests five
particles but `has_consume_particles=false` suppresses them. It still consumes the generic sound
randomness in order — one Boolean, one triangular sample centered at `1` with deviation `0.2`, and
one uniform float in `[0.9,1.0)` — then plays `entity.generic.drink` at volume `0.5` with the
uniform value as pitch; the first two sampled values do not affect drink output.

Release, hand replacement, death or another generic interruption before server completion performs
none of the finish transaction: no final sound burst, statistic, consume criterion, Bad Omen,
disposal sound, `DRINK` event or count change.

### Finish transaction

When the server decrements remaining time from `1` to `0` with the same live hand stack, it invokes
the default components in this observable order:

1. emit one final consume burst requesting `16` particles; particles remain suppressed, the same
   three random operations occur and the generic drink sound plays at volume `0.5` and uniform
   pitch `[0.9,1.0)`;
2. for a server player, award `minecraft.used:minecraft.ominous_bottle`, then trigger
   `consume_item` against the still-unshrunk live stack;
3. enumerate stack component values implementing `ConsumableListener`; the default amplifier
   offers Bad Omen with duration `120000`, amplifier `a`, `ambient=false`,
   `visible=false`, `show_icon=true`, and ignores the Boolean returned by `addEffect`;
4. apply configured server-side consume effects; the default disposal effect broadcasts
   `item.ominous_bottle.dispose` at the user's block position and sound category, volume `1` and
   pitch `1`;
5. emit the `DRINK` game event from the user;
6. consume one from the same stack unless the living entity has infinite materials, then return
   that same stack object.

There is no glass-bottle output. Infinite-material players receive the statistic, criterion, effect,
both finish sounds and game event but retain the ominous bottle. A rejected or merged Bad Omen
effect also does not roll back later steps. `ENT-EFFECT-001` owns effect admission, hidden-chain
merging, replacement, expiration and synchronization; direct nonstandard invocation on another
logical side follows the component method's lack of a side guard, while ordinary timed completion
is server authoritative.

The consume criterion sees the live stack before shrink, including its amplifier and arbitrary
component patch. No locked advancement directly selects ominous-bottle identity or amplifier, but
data packs can attach listeners to that generic trigger.

### Bad Omen downstream joins

The bottle does not itself start a raid or convert a trial spawner. It only offers Bad Omen. When
the effect later ticks:

- `MOB-RAID-001` owns village admission and conversion to Raid Omen for `600` ticks while retaining
  amplifier `a`;
- `BLK-TRIAL-SPAWNER-001` owns player scanning and conversion. At that join, a qualifying Bad Omen
  amplifier `a` is removed and Trial Omen amplifier `0` is installed for
  `18000 * (a + 1)` ticks, so the five locked bottle levels yield `18000`, `36000`, `54000`,
  `72000` or `90000` ticks.

Those consumers read the current effect state, not the consumed stack, and therefore remain subject
to generic effect rejection, replacement, removal and intervening tick order.

### Acquisition

The locked direct loot occurrences are exhaustive:

- a pillager entity-loot pool runs one count-one ominous-bottle entry only when that pillager is a
  raid captain, with amplifier drawn uniformly and inclusively from `0..4`; it has no
  killed-by-player or looting condition and uses random sequence `minecraft:entities/pillager`;
- each evaluation of `chests/trial_chambers/reward_common` makes one weighted choice over total
  weight `25`; the bottle has weight `2`, count one and uniform inclusive amplifier `0..1`;
- each evaluation of `chests/trial_chambers/reward_ominous_common` makes one weighted choice over
  total weight `15`; the bottle has weight `1`, count one and uniform inclusive amplifier `2..4`.

The normal vault reward table always evaluates its common subtable `1..3` times and independently
selects that same common table once more with weight `2/10 = 1/5` in its rare/common pool. The
ominous reward table has the identical `1..3` plus `1/5` common-evaluation structure using its
ominous subtables. Consequently each common evaluation, not each complete vault reward, has bottle
chance `2/25` normal or `1/15` ominous; one reward transaction can emit multiple count-one bottle
stacks. The unique pools do not add bottles. `BLK-VAULT-001` owns key admission, reward evaluation
and reverse-order timed ejection, and `ITM-LOOT-001` owns generic table evaluation and output
materialization.

There is no locked recipe, trade, direct advancement entry or other direct loot-table occurrence
for this item.

### Persistence and client projection

The generic item-stack codec persists identity, count and component patch. The bounded amplifier
component supplies its own codec; already emitted bottles retain their level across data reload.
Reloaded loot tables affect only future pillager deaths and vault reward evaluations. Bad Omen,
Raid Omen, Trial Omen and vault state persist or reload under their cited owners rather than through
the consumed item.

The amplifier component supplies the standard potion-effect tooltip for Bad Omen level `a + 1`,
duration `120000` ticks, scaled by the current tooltip tick rate (displayed as `100:00` at `20`
ticks per second). The five Food & Drinks tab entries are generated in ascending amplifier order
`0..4`, visible in both parent and search tabs, after milk bucket and honey bottle and before
generated potion entries. All levels use the same direct `generated` model and one
`ominous_bottle` texture; the amplifier does not select a model.

**Branches and aborts:**

Default/amplifier-removed/consumable-removed/patched stack; amplifier `0..4`; count `1..64`;
survival/infinite materials; main/off hand; uninterrupted/interrupted/replaced use; player/nonplayer
living user; effect accepted/rejected/merged; captain/noncaptain pillager; every normal/ominous
common-table call and weighted boundary; live/reloaded data and resources.

**Constants and randomness:**

Raw item ID `1536`; maximum stack `64`; amplifier bounds `0..4`; Bad Omen duration `120000`; use
duration `32`; cadence threshold `7`; cadence remaining ticks `24/20/16/12/8/4`; cadence/final
particle requests `5/16` but emitted count `0`; drink volume `0.5`, pitch `[0.9,1.0)`; disposal
volume/pitch `1/1`; normal common bottle weight/total `2/25`, amplifier `0..1`; ominous common
`1/15`, amplifier `2..4`; captain amplifier `0..4`; per-vault guaranteed common evaluations `1..3`
plus one with probability `1/5`.

**Side effects:**

Using-item state and animation; drink/disposal sounds; item-used statistic and generic consume
criterion; Bad Omen/effect synchronization; `DRINK` game event; held count; captain/vault loot
stacks; downstream omen transitions; durable stack/effect/vault state; tooltip/model/tab projection.

**Gates:**

Consumable presence and generic use ownership; remaining time and unchanged hand stack; player
ability; amplifier presence and effect admission; pillager captain predicate; current loot table,
pool condition, rolls, weights and number provider; vault transaction; current data/resource
snapshots.

**State read/written:**

Reads held stack identity/count/components, hand/use timer, user RNG/ability/type/effects, pillager
captain state, loot context/random sequences, vault reward configuration and client resources.
Writes using state, statistic/criterion progress, effect map, game event/sounds, held count, generated
loot stacks and durable component/effect state.

**Failure behavior:**

No consumable component means ordinary item use falls through. Interruption before completion
commits nothing from the finish path. Missing amplifier completes without adding Bad Omen.
Effect rejection does not cancel disposal/event/consumption. Missing/replaced loot tables or failed
pool conditions emit no bottle. Out-of-range persisted amplifier data fails its component codec;
loot-function outputs are clamped before construction.

**Persistence boundary:**

Item identity/count/components use the stack codec; active effects use the living-entity effect
codec; vault reward state uses the vault owner. A partially completed use is not durable. Loot
random choices and finish transactions do not resume after completion. Data/resource reload affects
future evaluation/projection without rewriting existing stacks or effects.

**Boundary cases and quirks:**

The level is stored as zero-based `0..4` but projects as effect levels I..V. Stacks merge only when
their component maps are equal, so different amplifier levels do not stack together. Drink cadence
consumes randomness for particle-oriented values even though particles are disabled. The consume
criterion precedes effect application and shrink. Effect failure is nontransactional.
Creative/infinite-material use still produces the complete finish transaction but consumes no
item, and no mode produces a glass bottle.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-DATA-001`; `OFF-REPORT-001`;
`net.minecraft.world.item.Items`; `net.minecraft.world.item.ItemStack`;
`net.minecraft.world.item.component.Consumables`;
`net.minecraft.world.item.component.Consumable`;
`net.minecraft.world.item.component.ConsumableListener`;
`net.minecraft.world.item.component.OminousBottleAmplifier`;
`net.minecraft.world.item.consume_effects.PlaySoundConsumeEffect`;
`net.minecraft.world.entity.LivingEntity`;
`net.minecraft.world.level.storage.loot.functions.SetOminousBottleAmplifierFunction`;
`net.minecraft.world.item.CreativeModeTabs`;
`reports/registries.json#minecraft:item/minecraft:ominous_bottle`;
`reports/minecraft/components/item/ominous_bottle.json`;
`data/minecraft/loot_table/entities/pillager.json`;
`data/minecraft/loot_table/chests/trial_chambers/{reward,reward_common,reward_ominous,reward_ominous_common}.json`;
`assets/minecraft/{items,models/item,textures/item}/ominous_bottle.*`;
`ITM-USE-001`; `ITM-LOOT-001`; `ITM-ADVANCEMENT-001`; `ENT-EFFECT-001`;
`MOB-RAID-001`; `BLK-TRIAL-SPAWNER-001`; `BLK-VAULT-001`; `CLI-EFFECT-001`;
`EXP-ITM-030`.

**Test vectors:**

Use every amplifier/default/removed-component stack at count boundaries in both hands and ability
modes; interrupt/replace at every remaining tick; assert cadence RNG, sounds and exact finish order
with accepted/rejected/merged effects and generic criterion listeners. Exhaust captain and both
common-table weights/amplifiers plus nested vault common-call counts. Persist/reload stacks/effects,
reload data/resources, and inspect all five tooltip/model/Food & Drinks projections.

**Limits:**

This leaf does not duplicate generic use cancellation, effect merge/tick/sync, raids, trial-spawner
conversion, vault timing/ejection, loot evaluation, item-stack serialization or resource rendering.
Those remain with the cited owners; this rule fixes the ominous-bottle identity, bounded component
and their exact joins.
