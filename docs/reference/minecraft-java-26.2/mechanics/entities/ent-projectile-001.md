# Entities mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ENT-PROJECTILE-001` — Projectile ticks sweep from old to new position and resolve the first accepted hit

**Parent:** `ENT-004`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Cross-checked`

**SourceConclusion:**

`SourceInconclusive` — subtype physics, hit ties, piercing/deflection, owner filters, and
unloaded-chunk edges remain unexpanded.

**Applies when:**

A projectile entity is spawned and receives an entity tick.

**Authoritative state:**

Owner, position/velocity/rotation, gravity/drag, in-ground/piercing/return state, remaining
lifetime, pickup policy and subtype payload.

**Transition and ordering:**

Apply subtype pre-move logic; derive the segment from current position to proposed next position;
clip blocks and eligible entities using projectile predicates and choose the nearest accepted hit;
invoke entity/block hit callback; apply subtype damage/effect/embedding/discard/piercing result;
commit remaining movement and then gravity/drag in subtype source order. Anchor:
`net.minecraft.world.entity.projectile.Projectile#hitTargetOrDeflectSelf(net.minecraft.world.phys.HitResult)`.

**Branches and aborts:**

Owner immunity window; same vehicle/team exclusion; deflection; portal/gateway; block before entity;
pierce continues; no hit; chunk unloaded; in-ground state; lifetime expiry; pickup. Recheck removal
after every hit callback.

**Constants and randomness:**

Width, speed, divergence, gravity, drag, lifetime and damage are subtype-owned. Launch divergence
consumes shooter RNG at spawn; ordinary sweep is deterministic floating-point geometry. Ties/epsilon
and pierce order are `EXP-ENT-003`.

**Side effects:**

Entity damage/effects/knockback/fire, block callbacks, projectile embedding/removal/deflection, item
pickup/drop, sounds/particles/game events, criteria and client velocity/entity updates.

**Gates:**

Entity hit predicate, owner/team/friendly fire, collision shapes, portal rules, chunk activity,
subtype flags, pierce count and damage-type immunity.

**Boundary cases and quirks:**

Collision is swept, not only tested at final position. A hit callback can teleport/remove/deflect
the projectile. Visual interpolation cannot choose the authoritative hit.

**Evidence:**

`Confirmed` sweep/state shape; exact tie behavior `Cross-checked`; `OFF-SERVER-001`; listed locator
and subtype classes; `EXP-ENT-003`.

**Test vectors:**

High-speed thin target; entity behind wall; equal-distance candidates; owner immediately after
launch; piercing line; deflection; portal; hit callback removes projectile; compare
position/velocity sequence exactly.
