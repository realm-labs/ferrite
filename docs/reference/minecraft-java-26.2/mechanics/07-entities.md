# Entity and combat leaf rules

## Leaf rule `ENT-LIFECYCLE-001` — Entity insertion, ticking, passenger traversal, transfer, and removal have explicit ownership

**Parent:** `ENT-001`, `ENT-002`, `ENT-008`  
**FidelityClass:** `ExactObservableBehavior`  <br>
**EvidenceStatus:** `Confirmed`  <br>
**SourceConclusion:** `SourceInconclusive` — insertion/removal during iteration, passenger mutation, transfer flags, and failure rollback remain unexpanded.  <br>
**Applies when:** An entity is created, loaded, added to a level, ticked, changes dimensions, mounts/dismounts, dies, or is discarded.  
**Authoritative state:** UUID/numeric ID, owning level/chunk section, position/velocity/rotation, removal reason, passenger/vehicle graph, portal cooldown and tracked gameplay data.  
**Transition and ordering:** Validate unique insertion and chunk ownership; add to section/tracking callbacks; during level tick process only non-passenger roots and recursively tick passenger trees through the vehicle; commit section moves when coordinates cross boundaries; removal marks a reason and invokes untracking/section callbacks exactly once. Dimension transfer removes from the old level and creates/repositions the authoritative entity in the destination according to the transfer path. Anchors: `net.minecraft.server.level.ServerLevel#addEntity(net.minecraft.world.entity.Entity)`, `net.minecraft.server.level.ServerLevel#tickNonPassenger(net.minecraft.world.entity.Entity)`, and `net.minecraft.world.entity.Entity#remove(net.minecraft.world.entity.Entity$RemovalReason)`.  
**Branches and aborts:** Duplicate UUID; already removed; passenger handled by vehicle; destination chunk unavailable; change-dimension denied; player-specific transfer; death versus discard/unload removal reason. An entity removed during its callback must not receive later ordinary tick work.  
**Constants and randomness:** Entity type dimensions/tracking data come from the registry/report. Generic ownership transitions consume no RNG. UUID creation may use randomness only at construction if not supplied.  
**Side effects:** Chunk section/tracker membership, passenger links, scoreboard/team references, leash relations, item/XP drops on death, criteria/game events, sounds/particles and client add/remove/correction updates.  
**Gates:** Chunk entity-ticking status, removal state, passenger root status, portal cooldown, destination rules, peaceful/difficulty removal for mobs and entity-type feature flags.  
**Boundary cases and quirks:** Loaded is not entity-ticking. Passengers do not also run as independent roots. Unload removal must not cause death drops. Dimension transfer identity semantics differ for players and ordinary entities.  
**Evidence:** `Confirmed`; `OFF-SERVER-001`; locators above; unload/transfer graph `EXP-ENT-001`.  
**Test vectors:** Remove during tick; passenger tree three levels deep; unload/reload; duplicate UUID insertion; cross chunk section during movement; portal transfer with leash/passengers and verify no duplicate ticking or drops.

## Leaf rule `ENT-DAMAGE-001` — Damage is a gated pipeline from damage source to health/death transition

**Parent:** `ENT-005`
**FidelityClass:** `ExactObservableBehavior`  <br>
**EvidenceStatus:** `Confirmed`  <br>
**SourceConclusion:** `SourceSpecified` — server-player/player wrappers, base immunity, outer `LivingEntity#hurtServer` ordering, cooldown delta selection, attribution, event/sound/effect/criterion dispatch and boolean return are complete below. Item blocking resolves a blocked float through `ENT-BLOCK-001`; armor/magic/absorption/health through `ENT-DAMAGE-REDUCE-001`; velocity through `ENT-KNOCKBACK-001`; lethal protection/death through `ENT-DEATH-001`. Those explicit subtransactions do not alter the outer ordering specified here.  <br>
**Applies when:** Server code calls `hurtServer(level,source,amount)` on a living entity. Amount may be negative, zero, finite, infinite or NaN; wrappers and the base pipeline treat these values differently.
**Authoritative state:** Damage source/type/tags and direct/causing/source-position entities; entity type, removed/dead/sleeping/fire state and invulnerable flag; enchantment immunity; player abilities, connection/dimension/PvP/difficulty/game rules and shoulder entities; use-item snapshot and blocking result; `invulnerableTime`, `lastHurt`, `hurtTime/duration`; effects, attacker attribution, last source/stamp and lethal state.

**Transition and ordering:** Resolve the most-derived wrapper first.

1. `ServerPlayer#hurtServer` rejects when its invulnerability query succeeds. It then rejects a causing player whom `canHarmPlayer` disallows, and likewise an `AbstractArrow` whose player owner is disallowed; otherwise delegate to `Player`.
2. `ServerPlayer#isInvulnerableTo` is true when the player query is true, when changing dimension and the source is not exactly `ender_pearl`, or while the client is not loaded. `Player#isInvulnerableTo` first delegates to living immunity, then returns the inverse of the matching `drowning_damage`, `fall_damage`, `fire_damage` or `freeze_damage` game rule for the first matching tag, otherwise false.
3. Living immunity is base immunity OR `EnchantmentHelper.isImmuneToDamage`. Base immunity is true when removed; when the entity invulnerable flag is set unless the source bypasses invulnerability or is from a creative player; for fire against a fire-immune entity; or for fall against an entity type in `fall_damage_immune`. Because each Java wrapper invokes the virtual query before its superclass body, a server player rechecks the same most-derived immunity at the server-player, player and living-body boundaries.
4. `Player#hurtServer` rejects ability-invulnerable players unless the source bypasses invulnerability. It then sets `noActionTime=0`, rejects dead/dying, removes shoulder entities, and difficulty-scales only a source whose data says so: Peaceful `0`; Easy `min(amount/2+1,amount)`; Normal unchanged; Hard `amount*3/2`. An exact float zero returns false here; any other value, including NaN, delegates to the living body.
5. The living body again rejects virtual immunity, dead/dying, and fire-tagged damage while Fire Resistance is active, in that order. It then wakes a sleeping entity, sets `noActionTime=0`, clamps only `amount<0` to positive zero (NaN is not clamped), stores this as the criteria “original” amount, and snapshots the current use-item reference.
6. Call `ENT-BLOCK-001` with that amount to obtain `blockedAmount`; replace remaining amount by `amount-blockedAmount`, and define `blocked = blockedAmount>0`. If the source is freezing and the victim type is in `freeze_hurts_extra_types`, multiply remaining by `5.0f`. If the source `damages_helmet` and the head slot is nonempty, damage the helmet with the current remaining amount and then multiply remaining by `0.75f`. Replace NaN or either infinity with `Float.MAX_VALUE`.
7. Let `fresh=true`. If `invulnerableTime>10` and the source does not bypass cooldown, return false immediately when remaining `<=lastHurt`; earlier waking, action reset and blocking/helmet side effects are retained. Otherwise call `ENT-DAMAGE-REDUCE-001` with `remaining-lastHurt`, store the full remaining as `lastHurt`, set `fresh=false`, and do not reset timers. In every other case store remaining as `lastHurt`, set `invulnerableTime=20`, call the reduction leaf with full remaining, and set `hurtDuration=hurtTime=10`.
8. For either accepted cooldown branch, resolve mob responsibility unless `no_anger` or the wind-charge/entity exemption applies. A causing player is remembered for 100 ticks; a tame wolf attributes its owner UUID for 100 ticks (or clears attribution when absent).
9. Only when `fresh`: if blocking succeeded and the snapshotted use item still has `minecraft:blocks_attacks`, invoke its `onBlocked`; otherwise broadcast the damage event. Unless `no_impact`, call `markHurt` when unblocked or when remaining is positive. Unless `no_knockback`, invoke `ENT-KNOCKBACK-001` with source, full remaining and blocked flag.
10. If now dead/dying, call `ENT-DEATH-001` protection then death path; otherwise a fresh hit plays primary and secondary hurt sounds. Define `meaningful = !blocked || remaining>0`. Only when meaningful, store last source and current game time, then invoke every active effect's `onMobHurt` in collection order with the **full remaining**, not the cooldown delta.
11. Criteria always follow an accepted cooldown branch. A server-player victim receives `(source, original, remaining, blocked)`; positive blocked amount strictly below `Float.MAX_VALUE/10` awards `round(blockedAmount*10)` shield-block stat. A causing server player receives the mirrored hurt-entity criterion. Return `meaningful`.

`hurtTime` decrements once in living base tick. `invulnerableTime` decrements once there for non-server-player living entities; `ServerPlayer#tick` performs its own one-step decrement because the base path excludes it.
**Branches and aborts:** All wrapper immunity/PvP/arrow-owner/ability/client-loaded/dimension/game-rule/difficulty gates; removed/dead/fire-resistance/sleep; negative/zero/NaN/infinite amounts; blocking, freezing-extra and helmet; cooldown bypass and lower/equal/higher repeat hit; fresh/delta event, impact, knockback and sound; lethal protection/death; meaningful effect/stamp gate; victim/attacker criteria and blocked-stat bound.
**Constants and randomness:** Difficulty uses float `/2+1`, `min`, and `*3/2`; negative clamps to `+0.0f`; freezing `5.0f`; helmet `0.75f`; nonfinite post-transform becomes `Float.MAX_VALUE`; cooldown threshold is strict `invulnerableTime>10`, fresh timer `20`, hurt timers `10`, attribution memory `100`; blocked stat upper bound is strict `<3.4028235e37f` and value is `round(blockedAmount*10)`. Outer control flow consumes no RNG; delegated enchantment, blocking, reduction and death hooks own any RNG.
**Side effects:** Even a final false result may already wake/reset action time or damage a blocking/helmet item. Accepted hits update cooldown/attribution, may emit damage/impact/knockback/sounds, enter death, store last source/time, invoke effects, criteria and stats. Health, absorption, defense durability, exact blocking, velocity and death outputs are owned by the named leaves.
**Gates:** Locked damage data/tags, entity tags/type, enchantment immunity, player game rules/PvP/difficulty/abilities/connection/dimension, Fire Resistance, item components/equipment, cooldown state, active effects and lethal state.
**Boundary cases and quirks:** A nonplayer negative amount becomes zero yet can start a fresh 20-tick cooldown, emit events/knockback and return true when unblocked; the player wrapper rejects exact zero after difficulty scaling. A stronger cooldown hit reduces health only by the excess but updates callbacks/criteria with the full current remaining. A rejected weaker cooldown hit retains pre-cooldown item and wake/action side effects. Fully blocked fresh hits can return false after `onBlocked`, attribution and criteria.
**Evidence:** `OFF-SERVER-001`, `OFF-DATA-001`. Anchors: `net.minecraft.world.entity.Entity#isInvulnerableToBase(net.minecraft.world.damagesource.DamageSource)`, `net.minecraft.world.entity.LivingEntity#isInvulnerableTo(net.minecraft.server.level.ServerLevel,net.minecraft.world.damagesource.DamageSource)`, `net.minecraft.world.entity.LivingEntity#hurtServer(net.minecraft.server.level.ServerLevel,net.minecraft.world.damagesource.DamageSource,float)`, `net.minecraft.world.entity.player.Player#isInvulnerableTo(net.minecraft.server.level.ServerLevel,net.minecraft.world.damagesource.DamageSource)`, `net.minecraft.world.entity.player.Player#hurtServer(net.minecraft.server.level.ServerLevel,net.minecraft.world.damagesource.DamageSource,float)`, `net.minecraft.server.level.ServerPlayer#isInvulnerableTo(net.minecraft.server.level.ServerLevel,net.minecraft.world.damagesource.DamageSource)`, and `net.minecraft.server.level.ServerPlayer#hurtServer(net.minecraft.server.level.ServerLevel,net.minecraft.world.damagesource.DamageSource,float)`.
**Test vectors:** (1) Every immunity wrapper including unloaded/changing-dimension server player, each player damage game rule, ability/base/enchantment immunity and PvP direct/arrow owner. (2) Difficulty-scaled negative, signed zero, positive, NaN and infinities for player/nonplayer. (3) Blocking/freezing-extra/helmet combinations through nonfinite sanitization. (4) Timer `10/11`, bypass/no-bypass and remaining below/equal/above `lastHurt`; assert exact reduction input versus callback input and retained early side effects. (5) Fresh/delta, blocked/unblocked, no-impact/no-knockback, lethal/protected, active effects, attribution and both criteria/stat boundaries. `EXP-ENT-002` is the regression probe.

## Leaf rule `ENT-BLOCK-001` — Item blocking resolves angle, blocked amount, durability and retaliation

**Parent:** `ENT-005`  <br>
**FidelityClass:** `ExactObservableBehavior`  <br>
**EvidenceStatus:** `Cross-checked`  <br>
**SourceConclusion:** `SourceInconclusive` — the outer call and result use are fixed by `ENT-DAMAGE-001`; component curves, timing, angle, durability, disable and retaliation remain to expand.  <br>
**Applies when:** A living entity may be actively using an item with `minecraft:blocks_attacks`.
**Authoritative state:** Use ticks/hand/stack/component, source tags/position/direct entity, view angle, blocked amount and item durability/cooldown.
**Transition and ordering:** Return a blocked float to `ENT-DAMAGE-001`; own every mutation performed while computing it.
**Branches and aborts:** No blocking item/component, bypass tag, piercing arrow, missing source position, angle/timing/strength curve, nonprojectile living attacker and player disable.
**Constants and randomness:** Pending component/data audit; no placeholder curve is normative.
**Side effects:** Blocking item durability/cooldown, attacker knockback/disable and later outer blocked event/stat.
**Gates:** Component data, damage tags, use state, angle, projectile pierce and attacker type.
**Boundary cases and quirks:** Partial blocking is a float and may coexist with remaining damage.
**Evidence:** `OFF-SERVER-001`; `net.minecraft.world.entity.LivingEntity#applyItemBlocking(net.minecraft.server.level.ServerLevel,net.minecraft.world.damagesource.DamageSource,float)` and `net.minecraft.world.item.component.BlocksAttacks#resolveBlockedDamage(net.minecraft.world.damagesource.DamageSource,float,double)`.
**Test vectors:** `EXP-ENT-002` around every extracted time/angle/curve/durability boundary.

## Leaf rule `ENT-DAMAGE-REDUCE-001` — Defense, absorption and health consume the selected cooldown amount

**Parent:** `ENT-005`  <br>
**FidelityClass:** `ExactObservableBehavior`  <br>
**EvidenceStatus:** `Cross-checked`  <br>
**SourceConclusion:** `SourceInconclusive` — `ENT-DAMAGE-001` fixes the exact selected input and callback boundary; internal defense arithmetic remains to expand.  <br>
**Applies when:** The outer pipeline accepts a fresh amount or cooldown excess.
**Authoritative state:** Armor/toughness/effects/enchantments/absorption/health, equipment durability, combat tracker and stats.
**Transition and ordering:** Apply locked defense and health transaction, returning through side effects to the outer pipeline.
**Branches and aborts:** Armor/magic bypass tags, zero/nonfinite input, resistance/protection, absorption crossing and health boundary.
**Constants and randomness:** Pending exact float/clamp audit.
**Side effects:** Equipment durability, absorption, health, combat tracking and damage stats.
**Gates:** Damage tags, attributes, effects, enchantments and current absorption/health.
**Boundary cases and quirks:** Cooldown excess is reduced independently, while outer callbacks retain the full current amount.
**Evidence:** `OFF-SERVER-001`; `net.minecraft.world.entity.LivingEntity#actuallyHurt(net.minecraft.server.level.ServerLevel,net.minecraft.world.damagesource.DamageSource,float)`.
**Test vectors:** `EXP-ENT-002` across all bypass and float boundaries.

## Leaf rule `ENT-KNOCKBACK-001` — Accepted damage derives and applies source-relative velocity

**Parent:** `ENT-005`  <br>
**FidelityClass:** `ExactObservableBehavior`  <br>
**EvidenceStatus:** `Cross-checked`  <br>
**SourceConclusion:** `SourceInconclusive` — invocation gates are fixed by `ENT-DAMAGE-001`; resistance, scaling and velocity mutation remain to expand.  <br>
**Applies when:** A fresh accepted hit lacks `no_knockback`.
**Authoritative state:** Source/direct entity/position, current velocity, knockback resistance and blocked flag.
**Transition and ordering:** Derive projectile override or source-relative horizontal direction, then apply the locked knockback transaction and indication.
**Branches and aborts:** Projectile/nonprojectile, missing/coincident source, resistance, blocked and client indication.
**Constants and randomness:** Outer base strength is double `0.4000000059604645`; remaining constants pending audit.
**Side effects:** Velocity and hurt-direction indication.
**Gates:** Damage tag outer gate, source geometry, attributes and blocked state.
**Boundary cases and quirks:** A zero remaining amount may still invoke this leaf when unblocked.
**Evidence:** `OFF-SERVER-001`; `net.minecraft.world.entity.LivingEntity#dealDefaultKnockback(net.minecraft.world.damagesource.DamageSource,float,boolean)` and `net.minecraft.world.entity.LivingEntity#knockback(double,double,double,net.minecraft.world.damagesource.DamageSource,float)`.
**Test vectors:** `EXP-ENT-002` with projectile/source/coincident vectors and resistance boundaries.

## Leaf rule `ENT-DEATH-001` — Death protection, death entry, drops and timed removal form one transaction

**Parent:** `ENT-007`  <br>
**FidelityClass:** `ExactObservableBehavior`  <br>
**EvidenceStatus:** `Cross-checked`  <br>
**SourceConclusion:** `SourceInconclusive` — the lethal call boundary is fixed by `ENT-DAMAGE-001`; protection, drops, XP, death tick and removal remain to expand.  <br>
**Applies when:** An accepted damage branch leaves a living entity dead or dying.
**Authoritative state:** Health/dead/death timer, protection items/effects, source, loot/equipment/XP, gamerules and removal reason.
**Transition and ordering:** Attempt protection first; on failure enter death exactly once, then own drops/XP and timed death removal.
**Branches and aborts:** Invulnerability-bypass, protection found/absent, repeated death, loot/gamerule/player overrides and unload versus killed removal.
**Constants and randomness:** Pending protection/loot/death-timer audit.
**Side effects:** Item consumption/effects/event, dead state, score/criteria, drops/XP, sound/event and removal.
**Gates:** Source tags, protection components, loot tables, gamerules and entity/player subtype.
**Boundary cases and quirks:** Unload is not death and must not produce this transaction.
**Evidence:** `OFF-SERVER-001`, `OFF-DATA-001`; `net.minecraft.world.entity.LivingEntity#checkTotemDeathProtection(net.minecraft.world.damagesource.DamageSource)`, `net.minecraft.world.entity.LivingEntity#die(net.minecraft.world.damagesource.DamageSource)`, and `net.minecraft.world.entity.LivingEntity#tickDeath()`.
**Test vectors:** `EXP-ENT-002` for lethal/protection/reentry/drop/removal ordering.

## Leaf rule `ENT-PROJECTILE-001` — Projectile ticks sweep from old to new position and resolve the first accepted hit

**Parent:** `ENT-004`  
**FidelityClass:** `ExactObservableBehavior`  <br>
**EvidenceStatus:** `Cross-checked`  <br>
**SourceConclusion:** `SourceInconclusive` — subtype physics, hit ties, piercing/deflection, owner filters, and unloaded-chunk edges remain unexpanded.  <br>
**Applies when:** A projectile entity is spawned and receives an entity tick.  
**Authoritative state:** Owner, position/velocity/rotation, gravity/drag, in-ground/piercing/return state, remaining lifetime, pickup policy and subtype payload.  
**Transition and ordering:** Apply subtype pre-move logic; derive the segment from current position to proposed next position; clip blocks and eligible entities using projectile predicates and choose the nearest accepted hit; invoke entity/block hit callback; apply subtype damage/effect/embedding/discard/piercing result; commit remaining movement and then gravity/drag in subtype source order. Anchor: `net.minecraft.world.entity.projectile.Projectile#hitTargetOrDeflectSelf(net.minecraft.world.phys.HitResult)`.  
**Branches and aborts:** Owner immunity window; same vehicle/team exclusion; deflection; portal/gateway; block before entity; pierce continues; no hit; chunk unloaded; in-ground state; lifetime expiry; pickup. Recheck removal after every hit callback.  
**Constants and randomness:** Width, speed, divergence, gravity, drag, lifetime and damage are subtype-owned. Launch divergence consumes shooter RNG at spawn; ordinary sweep is deterministic floating-point geometry. Ties/epsilon and pierce order are `EXP-ENT-003`.  
**Side effects:** Entity damage/effects/knockback/fire, block callbacks, projectile embedding/removal/deflection, item pickup/drop, sounds/particles/game events, criteria and client velocity/entity updates.  
**Gates:** Entity hit predicate, owner/team/friendly fire, collision shapes, portal rules, chunk activity, subtype flags, pierce count and damage-type immunity.  
**Boundary cases and quirks:** Collision is swept, not only tested at final position. A hit callback can teleport/remove/deflect the projectile. Visual interpolation cannot choose the authoritative hit.  
**Evidence:** `Confirmed` sweep/state shape; exact tie behavior `Cross-checked`; `OFF-SERVER-001`; listed locator and subtype classes; `EXP-ENT-003`.  
**Test vectors:** High-speed thin target; entity behind wall; equal-distance candidates; owner immediately after launch; piercing line; deflection; portal; hit callback removes projectile; compare position/velocity sequence exactly.

## Leaf rule `ENT-VEHICLE-001` — Vehicle control, physics, collision, and passenger placement are server-owned

**Parent:** `ENT-002`, `ENT-003`, `PLY-001`  
**FidelityClass:** `ExactObservableBehavior`  <br>
**EvidenceStatus:** `Cross-checked`  <br>
**SourceConclusion:** `SourceInconclusive` — boat/minecart family constants, collision traversal, input transfer, and passenger placement remain unexpanded.  <br>
**Applies when:** A boat/raft/minecart family entity is ticked, controlled, collided, mounted, or dismounted.  
**Authoritative state:** Vehicle position/velocity/rotation/status, controlling passenger/input, passenger order, track/fluid/ground context, damage/hurt state and interpolation target.  
**Transition and ordering:** Determine control source and environment status; apply propulsion or rail logic; move through entity collision; resolve vehicle/entity pushes; update rotation/status; position every passenger from the committed vehicle transform in passenger order; validate dismount location on exit. Client-controlled prediction is reconciled to server accepted motion.  
**Branches and aborts:** No controller; non-player controller; underwater/air/land status; powered/activator rail; derailment; collision; destroyed vehicle; passenger cannot ride; dismount pose has no collision-free location.  
**Constants and randomness:** Acceleration, drag, max speed, buoyancy and rail projection are family/source constants with double/float rounding. Generic movement consumes no RNG. Exact numeric trajectories are `EXP-ENT-004`.  
**Side effects:** Vehicle/passenger movement, collisions/pushes, fall or impact consequences, block/rail callbacks, damage/drops, sounds/particles/game events, chunk tracking and corrections.  
**Gates:** Controller identity, vehicle family/status, input, rail/fluid state, collision, riding permission/cooldown, entity-ticking chunk and gamerules for drops.  
**Boundary cases and quirks:** Passenger position derives after vehicle motion and is not independent player movement. Dismount searches legal poses/locations. Minecart and boat physics are different families despite shared riding semantics.  
**Evidence:** `Confirmed` ownership/state order; numeric parity `Implementation blocker`; `OFF-SERVER-001`, `OFF-CLIENT-001`; `EXP-ENT-004`.  
**Test vectors:** Empty/controlled boat on land/water/air; two passengers; collide with entity/wall; minecart slopes/curves/powered rails; unload; destroy while occupied; dismount with only one legal pose.

## Leaf rule `ENT-EFFECT-001` — Status effects merge, tick, expire, and expose attributes in a defined lifecycle

**Parent:** `ENT-006`  
**FidelityClass:** `ExactObservableBehavior`  <br>
**EvidenceStatus:** `Confirmed`  <br>
**SourceConclusion:** `SourceInconclusive` — effect-specific cadence, hidden-chain promotion, attribute ordering, and removal callbacks remain unexpanded.  <br>
**Applies when:** A living entity gains, updates, removes, cures, or ticks a mob effect instance.  
**Authoritative state:** Effect ID, duration, amplifier, ambient/visible/icon flags, hidden chained instance, effect-derived attribute modifiers and source entity.  
**Transition and ordering:** On add, test applicability; if absent install and call add hooks; if present merge strength/duration/flags while preserving weaker/longer instances through the hidden chain as defined; each living tick test the duration/amplifier cadence, apply tick effect, decrement duration and promote/remove on expiry; update attribute modifiers and client-visible metadata on transitions.  
**Branches and aborts:** Immune entity; instant effect uses immediate application rather than storage; stronger/longer/equal merge; duration infinite; cadence false; cure selects only effects matching cure semantics; death/removal cleanup.  
**Constants and randomness:** Duration is integer ticks; amplifier is integer. Per-effect cadence and arithmetic are behavior class/data. Instant health/harm scale by amplifier and context with exact integer/float conversion. Effects consume RNG only where their implementation explicitly does.  
**Side effects:** Health/damage, attributes, AI/movement/visibility, particles/icon metadata, sounds/game events, criteria and effect add/update/remove synchronization.  
**Gates:** Entity applicability/immunity, effect category/type, duration/cadence, amplifier, cure item, difficulty and source/context predicates.  
**Boundary cases and quirks:** A hidden weaker effect can reappear after stronger expiry. Attribute modifiers must be removed/reapplied when amplifier changes, not stacked. Visual flags do not control mechanics.  
**Evidence:** `Confirmed`; `OFF-SERVER-001`; registry/data snapshot; merge/cadence experiment `EXP-ENT-005`.  
**Test vectors:** Strong-short over weak-long; equal amplifier duration update; infinite; cure; instant effect; amplifier change attribute exactness; save/reload mid-chain; expire on the boundary tick.
