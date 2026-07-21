# Entities mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ENT-EFFECT-001` — Effects merge through hidden chains and tick from remaining duration

**Parent:** `ENT-006`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — merge/promotion, cadence, instant scaling, attribute replacement, synchronization,
applicability and every specialized vanilla `MobEffect` subclass are explicit in locked source;
potion compositions are locked DataOnly registry inputs.

**Applies when:**

A living entity adds, forces, removes, saves/loads or ticks an effect, is hurt, or is removed.

**Authoritative state:**

Effect holder, duration (`-1` infinite), clamped amplifier `0..255`, ambient/particle/icon flags,
recursive hidden instance, blend state, active-effect map, attributes, entity type tags and source.

**Transition and ordering:**

Adding first tests applicability. If absent, store the supplied instance, run living add callback
(dirty, add attributes, notify player passengers), then effect `onEffectAdded`. If present, merge it;
a changed merge runs update with attribute refresh. In either accepted case—even an unchanged
merge—the incoming instance then invokes `onEffectStarted`, and the return value reports only map
state change.

Merge replaces amplifier/duration when incoming amplifier is higher; if the stronger effect is
shorter, copy the old visible instance ahead of its old hidden chain. A longer incoming effect of
equal amplifier extends visible duration; a longer weaker effect is inserted/merged into the hidden
chain. Incoming nonambient clears ambient when current is ambient, or any strength/duration change
copies incoming ambient. Particle and icon flags always copy when different. Infinite is longer than
every finite duration. Amplifier is clamped on construction.

Server tick iterates active keys. An instance with no duration is removed without callback execution.
Otherwise cadence sees remaining duration, or entity `tickCount` for infinite. If a scheduled
`applyEffectTick` returns false, the instance expires immediately without decrement/promotion.
Otherwise hidden durations decrement recursively, visible finite duration decrements, and a hidden
instance is promoted exactly when visible becomes zero; promotion triggers update with attribute
refresh. The instance remains only if infinite or duration is now positive. Ordinary removal invokes
one removal callback; every still-present duration divisible by `600` sends an update without
attribute refresh. Concurrent modification aborts the rest of this server pass silently.

Update with refresh removes every effect-owned attribute modifier by ID, adds permanent modifiers
scaled as `baseAmount*(amplifier+1)`, then applies dirty-attribute consequences (health/absorption
clamp, dimensions, waypoint tracking). Remove-all is server-only, copies and clears the map before
removing modifiers. Forced add replaces outright, copies blend state on replacement and runs the
living add/update callback, but does not call effect `onEffectAdded`/`onEffectStarted`.

Special tick implementations are exact: regeneration heals `1` every `50>>amp` ticks; poison deals
`1` above health `1` every `25>>amp`; wither deals `1` every `40>>amp`; nonpositive shifted interval
means every tick. Hunger adds `0.005*(amp+1)` exhaustion every tick. Saturation instantaneously eats
`amp+1` with modifier `1`. Absorption starts at at least `4*(amp+1)` and keeps ticking only while
absorption is positive. Heal/harm uses `4<<amp` or `6<<amp`, inverted by entity tag; instant scale
rounds through `(int)(scale*amount+0.5)` and uses indirect magic when a source exists.

Bad omen checks every tick and, for a nonspectator player in a non-Peaceful village with raid
capacity, adds raid omen duration `600`, saves position, then returns false to remove itself. Raid
omen acts at remaining `1`, creates/extends the raid at the saved position, clears it and returns
false. Infested rolls `<=0.1` on hurt, then `1..2` silverfish and their trajectory/yaw draws. On
`KILLED` only: wind charged explodes at strength `3+nextFloat*2`; weaving attempts `2..3` supported
cobwebs from 15 radius-one cube samples when player or mob griefing; oozing requests two size-2
slimes, capped by nearby slimes and max cramming. Other registered effects are pure flags,
attributes or behavior hooks consumed by their owning mechanics; the locked registry defines all
40 IDs, colors, categories, blend times, sounds and modifiers.

**Branches and aborts:**

Entity tags reject infested, oozing, or poison/regeneration as specified. Instant effects use the
caller instant path rather than requiring storage. Cadence false still decrements. Effect callback
false removes before decrement. Removal-trigger effects require stored reason exactly `KILLED`.

**Constants and randomness:**

Infinite `-1`, amplifier `0..255`, refresh packet cadence `600`; client particle chance is
`1/(4*ambientFactor)` or `1/(15*ambientFactor)` while invisible, ambient factor `5`, using a random
visible particle. Specialized RNG is consumed only in the branches and order above.

**Side effects:**

Active/hidden state, attributes and dependent health/scale/waypoint state, effect packets to riding
players, synchronized particles/ambience/invisibility/glow, health/hunger/raid state, spawned mobs/
blocks/explosion, sounds and removal callbacks.

**Gates:**

Applicability tags, server/client, duration/cadence, effect callback result, effect-specific entity/
difficulty/gamerules/reason, visible/ambient flags and potion composition.

**Boundary cases and quirks:**

Hidden durations run down while hidden and may reach zero before promotion. An unchanged accepted add
still runs `onEffectStarted`. Finite duration `1` may apply, decrement and expire in one call; raid
omen deliberately returns false before decrement. The server catches concurrent effect-map mutation
and leaves all not-yet-visited entries untouched until a later tick.

**Evidence:**

`OFF-SERVER-001`, `OFF-DATA-001`;
`net.minecraft.world.effect.MobEffectInstance#update`,
`net.minecraft.world.effect.MobEffectInstance#tickServer`,
`net.minecraft.world.entity.LivingEntity#tickEffects`,
`net.minecraft.world.entity.LivingEntity#addEffect`,
`net.minecraft.world.entity.LivingEntity#forceAddEffect`,
`net.minecraft.world.entity.LivingEntity#onEffectUpdated`,
`net.minecraft.world.entity.LivingEntity#onEffectsRemoved`,
`net.minecraft.world.effect.MobEffects` and every specialized class in
`net.minecraft.world.effect`; `EXP-ENT-005`.

**Test vectors:**

Strong-short over weak-long with hidden expiry; equal extend/unchanged flags; infinite cadence;
callback false; duration `1` and `600`; mutation during tick; force replacement; amplifier clamp and
attribute-dependent health clamp; every special cadence/instant inversion/removal-reason branch.
