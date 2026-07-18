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

**Parent:** `ENT-005`, `ENT-007`  
**FidelityClass:** `ExactObservableBehavior`  <br>
**EvidenceStatus:** `Cross-checked`  <br>
**SourceConclusion:** `SourceInconclusive` — exact defense arithmetic, tag bypasses, invulnerability frames, death protection, drops, and removal timing remain unexpanded.  <br>
**Applies when:** A living entity receives a `DamageSource` with a locked `damage_type` and positive proposed amount.  
**Authoritative state:** Health/absorption, invulnerability and hurt timers, armor/toughness, effects, enchantments, attributes, shield/use state, combat tracker, attacker/direct entity and damage-type tags.  
**Transition and ordering:** Reject invulnerable/immune sources; evaluate shield blocking and blocked side effects; apply cooldown/invulnerability-frame semantics to select effective incoming amount; run armor/toughness unless bypassed; run effects/enchantments and absorption in their hook order; subtract remaining health; update combat/hurt state, knockback and criteria; if health reaches the death boundary, enter death handling once. Anchor: `net.minecraft.world.entity.LivingEntity#hurtServer(net.minecraft.server.level.ServerLevel,net.minecraft.world.damagesource.DamageSource,float)`.  
**Branches and aborts:** Amount nonpositive; removed/dead; damage-type immunity/tag bypass; creative/invulnerable; shield succeeds/fails; hurt cooldown absorbs all or only delta; armor/effect bypass; totem/death protection; death already entered. Return value records whether damage was accepted, not simply whether health changed.  
**Constants and randomness:** Amount is float; armor/toughness and protection calculations clamp/round in their source formulas. Damage-type exhaustion/scaling/death message are locked data. Enchantment/durability/death-protection branches may consume RNG; generic subtraction does not. Numeric parity is `EXP-ENT-002`.  
**Side effects:** Shield/item durability, armor durability, absorption/health, hurt/death timers, knockback/velocity, attacker attribution/combat tracker, statistics/criteria, sounds/particles/game events, drops/XP and client health/entity-event updates.  
**Gates:** Damage-type tags/data, invulnerability, difficulty scaling, shield angle/use, armor/effects/enchantments, gamerules such as friendly fire, team/owner relations and death protection.  
**Boundary cases and quirks:** Bypassing armor does not automatically bypass every later reduction. Invulnerability frames can accept only excess damage. A blocked hit may still cause attacker/defender side effects.  
**Evidence:** `Confirmed` order and gates; exact float vectors `Implementation blocker`; `OFF-SERVER-001`, `OFF-DATA-001`; locator above; `EXP-ENT-002`.  
**Test vectors:** Every bypass tag combination; shield front/back; two hits inside cooldown in both magnitude orders; armor/toughness extremes; absorption crossing; lethal hit with death protection; verify exact health and durability floats/integers.

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
