# Entities mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ENT-DAMAGE-001` — Damage is a gated pipeline from damage source to health/death transition

**Parent:** `ENT-005`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — server-player/player wrappers, base immunity, outer `LivingEntity#hurtServer`
ordering, cooldown delta selection, attribution, event/sound/effect/criterion dispatch and boolean
return are complete below. Item blocking resolves a blocked float through `ENT-BLOCK-001`;
armor/magic/absorption/health through `ENT-DAMAGE-REDUCE-001`; velocity through `ENT-KNOCKBACK-001`;
lethal protection/death through `ENT-DEATH-001`. Those explicit subtransactions do not alter the
outer ordering specified here.

**Applies when:**

Server code calls `hurtServer(level,source,amount)` on a living entity. Amount may be negative,
zero, finite, infinite or NaN; wrappers and the base pipeline treat these values differently.

**Authoritative state:**

Damage source/type/tags and direct/causing/source-position entities; entity type,
removed/dead/sleeping/fire state and invulnerable flag; enchantment immunity; player abilities,
connection/dimension/PvP/difficulty/game rules and shoulder entities; use-item snapshot and blocking
result; `invulnerableTime`, `lastHurt`, `hurtTime/duration`; effects, attacker attribution, last
source/stamp and lethal state.

**Transition and ordering:**

Resolve the most-derived wrapper first.

1. `ServerPlayer#hurtServer` rejects when its invulnerability query succeeds. It then rejects a
   causing player whom `canHarmPlayer` disallows, and likewise an `AbstractArrow` whose player owner
   is disallowed; otherwise delegate to `Player`.
2. `ServerPlayer#isInvulnerableTo` is true when the player query is true, when changing dimension
   and the source is not exactly `ender_pearl`, or while the client is not loaded.
   `Player#isInvulnerableTo` first delegates to living immunity, then returns the inverse of the
   matching `drowning_damage`, `fall_damage`, `fire_damage` or `freeze_damage` game rule for the
   first matching tag, otherwise false.
3. Living immunity is base immunity OR `EnchantmentHelper.isImmuneToDamage`. Base immunity is true
   when removed; when the entity invulnerable flag is set unless the source bypasses invulnerability
   or is from a creative player; for fire against a fire-immune entity; or for fall against an
   entity type in `fall_damage_immune`. Because each Java wrapper invokes the virtual query before
   its superclass body, a server player rechecks the same most-derived immunity at the
   server-player, player and living-body boundaries.
4. `Player#hurtServer` rejects ability-invulnerable players unless the source bypasses
   invulnerability. It then sets `noActionTime=0`, rejects dead/dying, removes shoulder entities,
   and difficulty-scales only a source whose data says so: Peaceful `0`; Easy
   `min(amount/2+1,amount)`; Normal unchanged; Hard `amount*3/2`. An exact float zero returns false
   here; any other value, including NaN, delegates to the living body.
5. The living body again rejects virtual immunity, dead/dying, and fire-tagged damage while Fire
   Resistance is active, in that order. It then wakes a sleeping entity, sets `noActionTime=0`,
   clamps only `amount<0` to positive zero (NaN is not clamped), stores this as the criteria
   “original” amount, and snapshots the current use-item reference.
6. Call `ENT-BLOCK-001` with that amount to obtain `blockedAmount`; replace remaining amount by
   `amount-blockedAmount`, and define `blocked = blockedAmount>0`. If the source is freezing and the
   victim type is in `freeze_hurts_extra_types`, multiply remaining by `5.0f`. If the source
   `damages_helmet` and the head slot is nonempty, damage the helmet with the current remaining
   amount and then multiply remaining by `0.75f`. Replace NaN or either infinity with
   `Float.MAX_VALUE`.
7. Let `fresh=true`. If `invulnerableTime>10` and the source does not bypass cooldown, return false
   immediately when remaining `<=lastHurt`; earlier waking, action reset and blocking/helmet side
   effects are retained. Otherwise call `ENT-DAMAGE-REDUCE-001` with `remaining-lastHurt`, store the
   full remaining as `lastHurt`, set `fresh=false`, and do not reset timers. In every other case
   store remaining as `lastHurt`, set `invulnerableTime=20`, call the reduction leaf with full
   remaining, and set `hurtDuration=hurtTime=10`.
8. For either accepted cooldown branch, resolve mob responsibility unless `no_anger` or the
   wind-charge/entity exemption applies. A causing player is remembered for 100 ticks; a tame wolf
   attributes its owner UUID for 100 ticks (or clears attribution when absent).
9. Only when `fresh`: if blocking succeeded and the snapshotted use item still has
   `minecraft:blocks_attacks`, invoke its `onBlocked`; otherwise broadcast the damage event. Unless
   `no_impact`, call `markHurt` when unblocked or when remaining is positive. Unless `no_knockback`,
   invoke `ENT-KNOCKBACK-001` with source, full remaining and blocked flag.
10. If now dead/dying, call `ENT-DEATH-001` protection then death path; otherwise a fresh hit plays
    primary and secondary hurt sounds. Define `meaningful = !blocked || remaining>0`. Only when
    meaningful, store last source and current game time, then invoke every active effect's
    `onMobHurt` in collection order with the **full remaining**, not the cooldown delta.
11. Criteria always follow an accepted cooldown branch. A server-player victim receives
    `(source, original, remaining, blocked)`; positive blocked amount strictly below
    `Float.MAX_VALUE/10` awards `round(blockedAmount*10)` shield-block stat. A causing server player
    receives the mirrored hurt-entity criterion. Return `meaningful`.

`hurtTime` decrements once in living base tick. `invulnerableTime` decrements once there for
non-server-player living entities; `ServerPlayer#tick` performs its own one-step decrement because
the base path excludes it.

**Branches and aborts:**

All wrapper immunity/PvP/arrow-owner/ability/client-loaded/dimension/game-rule/difficulty gates;
removed/dead/fire-resistance/sleep; negative/zero/NaN/infinite amounts; blocking, freezing-extra and
helmet; cooldown bypass and lower/equal/higher repeat hit; fresh/delta event, impact, knockback and
sound; lethal protection/death; meaningful effect/stamp gate; victim/attacker criteria and
blocked-stat bound.

**Constants and randomness:**

Difficulty uses float `/2+1`, `min`, and `*3/2`; negative clamps to `+0.0f`; freezing `5.0f`; helmet
`0.75f`; nonfinite post-transform becomes `Float.MAX_VALUE`; cooldown threshold is strict
`invulnerableTime>10`, fresh timer `20`, hurt timers `10`, attribution memory `100`; blocked stat
upper bound is strict `<3.4028235e37f` and value is `round(blockedAmount*10)`. Outer control flow
consumes no RNG; delegated enchantment, blocking, reduction and death hooks own any RNG.

**Side effects:**

Even a final false result may already wake/reset action time or damage a blocking/helmet item.
Accepted hits update cooldown/attribution, may emit damage/impact/knockback/sounds, enter death,
store last source/time, invoke effects, criteria and stats. Health, absorption, defense durability,
exact blocking, velocity and death outputs are owned by the named leaves.

**Gates:**

Locked damage data/tags, entity tags/type, enchantment immunity, player game
rules/PvP/difficulty/abilities/connection/dimension, Fire Resistance, item components/equipment,
cooldown state, active effects and lethal state.

**Boundary cases and quirks:**

A nonplayer negative amount becomes zero yet can start a fresh 20-tick cooldown, emit
events/knockback and return true when unblocked; the player wrapper rejects exact zero after
difficulty scaling. A stronger cooldown hit reduces health only by the excess but updates
callbacks/criteria with the full current remaining. A rejected weaker cooldown hit retains
pre-cooldown item and wake/action side effects. Fully blocked fresh hits can return false after
`onBlocked`, attribution and criteria.

**Evidence:**

`OFF-SERVER-001`, `OFF-DATA-001`. Anchors:
`net.minecraft.world.entity.Entity#isInvulnerableToBase(net.minecraft.world.damagesource.DamageSource)`,
`net.minecraft.world.entity.LivingEntity#isInvulnerableTo(net.minecraft.server.level.ServerLevel,net.minecraft.world.damagesource.DamageSource)`,
`net.minecraft.world.entity.LivingEntity#hurtServer(net.minecraft.server.level.ServerLevel,net.minecraft.world.damagesource.DamageSource,float)`,
`net.minecraft.world.entity.player.Player#isInvulnerableTo(net.minecraft.server.level.ServerLevel,net.minecraft.world.damagesource.DamageSource)`,
`net.minecraft.world.entity.player.Player#hurtServer(net.minecraft.server.level.ServerLevel,net.minecraft.world.damagesource.DamageSource,float)`,
`net.minecraft.server.level.ServerPlayer#isInvulnerableTo(net.minecraft.server.level.ServerLevel,net.minecraft.world.damagesource.DamageSource)`,
and
`net.minecraft.server.level.ServerPlayer#hurtServer(net.minecraft.server.level.ServerLevel,net.minecraft.world.damagesource.DamageSource,float)`.

**Test vectors:**

(1) Every immunity wrapper including unloaded/changing-dimension server player, each player damage
game rule, ability/base/enchantment immunity and PvP direct/arrow owner. (2) Difficulty-scaled
negative, signed zero, positive, NaN and infinities for player/nonplayer. (3)
Blocking/freezing-extra/helmet combinations through nonfinite sanitization. (4) Timer `10/11`,
bypass/no-bypass and remaining below/equal/above `lastHurt`; assert exact reduction input versus
callback input and retained early side effects. (5) Fresh/delta, blocked/unblocked,
no-impact/no-knockback, lethal/protected, active effects, attribution and both criteria/stat
boundaries. `EXP-ENT-002` is the regression probe.
