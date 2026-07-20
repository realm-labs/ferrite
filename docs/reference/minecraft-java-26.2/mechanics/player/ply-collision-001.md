# Player mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `PLY-COLLISION-001` — Generic entity movement clips a swept box, selects a step candidate and derives collision state

**Parent:** `PLY-002`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — the generic `Entity#move` geometry, step selection and post-collision state
transaction are complete below. Registry-specific collision shapes, friction/speed/bounce values and
block contact callbacks remain content-owned inputs rather than hidden branches of this algorithm.

**Applies when:**

Any entity calls `move(moverType,requested)` without `noPhysics`; player ordinary travel calls it
with `SELF`. This rule accepts already computed collision shapes and block properties.
Vehicle-specific movement and server player packet validation are separate leaves.

**Authoritative state:**

Position and AABB; requested and stored velocity; mover type; `noPhysics`, `stuckSpeedMultiplier`,
piston per-axis deltas/time; step-height attribute; staying-on-ground flag; world-border, block and
entity collision shapes; fall/collision/support flags; current block speed and bounce properties;
per-tick movement recordings.

**Transition and ordering:**

With `noPhysics`, add requested XYZ directly to position, clear horizontal, vertical, vertical-below
and minor-horizontal collision flags, and return without ordinary side effects. For `PISTON`, first
reset three accumulated axis deltas when game time changes. For the request's first nonzero axis in
X/Y/Z order, clamp `oldAccumulated+requestedAxis` to `[-0.51,+0.51]`, use
`newAccumulated-oldAccumulated` as the permitted movement, and return zero when its absolute value
is `<=9.999999747378752e-6`; a zero vector returns immediately. If
`stuckSpeedMultiplier.lengthSqr()>1e-7`, multiply non-piston requested movement componentwise by it,
then clear the multiplier and stored velocity. For a player, `maybeBackOffFromEdge` applies only
when not flying, requested Y is nonpositive, mover type is `SELF` or `PLAYER`, the player is staying
on a ground surface and is not already safely above ground by `maxUpStep`. “Above ground” is true
when already grounded, or when `fallDistance<maxUpStep` and the shrunken test box cannot fall by
`maxUpStep-fallDistance`. Otherwise repeatedly reduce X in `0.05*sign(X)` increments to zero while
X-only movement has no support, then Z similarly, then both together while the diagonal has no
support. Each support test shrinks horizontal AABB faces inward by `1e-7`, offsets X/Z, extends its
bottom downward by the tested distance plus `1e-7`, and asks whether collision shapes are absent. Y
is preserved.

Let `box` be the current AABB. Collect entity collision shapes over `box.expandTowards(requested)`.
Unless requested length squared is exactly zero, construct the clipping list in this order: all
collected entity shapes, the world-border shape only when the entity is close enough to it, then
block collision shapes over the same swept box. Clip against the combined list one axis at a time.
Axis order is `Y,Z,X` when `abs(requested.x)<abs(requested.z)`, otherwise (including equality)
`Y,X,Z`. Before clipping each later axis, move the working AABB by already accepted components. For
each shape, if the remaining axis magnitude becomes `<1e-7`, return zero for that axis immediately;
otherwise the shape clips it to the nearest nonpenetrating displacement.

The first clipped vector is the normal candidate. A step attempt occurs only when `maxUpStep>0`, X
or Z was clipped, and either downward Y was clipped or the entity was already on ground. If falling
into the obstacle, shift the step base box downward by accepted normal Y; otherwise use the original
box. Expand that base toward `(requestedX,maxUpStep,requestedZ)`, and additionally by
`-9.999999747378752e-6` Y when the attempt did not begin from a downward collision. Reuse the entity
collision list and collect world-border/block shapes for that volume. Candidate heights are every
collision-shape Y coordinate minus base `minY` that is nonnegative, not exactly equal (float
comparison) to the normal candidate Y, and `<=maxUpStep`; deduplicate as floats and unstable-sort
ascending. For each height, clip `(requestedX,height,requestedZ)` in normal axis order. Return the
first candidate whose horizontal squared displacement is strictly greater than the normal
candidate's; subtract the base-box vertical offset from its Y. If none improves horizontal distance,
keep the normal candidate.

If actual length squared is `>1e-7`, or `requested.lengthSqr-actual.lengthSqr <1e-7`, record
`(from,to,requested)` and set position to `position+actual`; otherwise position is unchanged. At 100
queued movements, adding another first merges/removes the two oldest into one
`from(first)->to(second)` record, keeping the queue bounded before appending. Then:

1. `xClipped` and `zClipped` use `!Mth.equal(requested,actual)`, where equality is absolute
   difference `<9.999999747378752e-6`; horizontal collision is their OR. Vertical collision uses
   exact `requestedY!=actualY` when vertical movement was requested or the entity is locally
   authoritative. Vertical-below is vertical collision with requested Y `<0`.
2. Feed vertical-below, horizontal-collision and actual displacement to support/on-ground
   derivation. Compute minor-horizontal collision only when horizontally collided. Look up the
   legacy on-position block and check fall damage using actual Y, derived on-ground, that block and
   position. Removal aborts remaining work.
3. When movement can be simulated and either requested Y actually collided or horizontal collision
   occurred, restore collision velocity. X/Z collisions reverse their stored velocity components
   times entity bounciness (normally zero). On a downward vertical collision, suppress bounce when
   requested, when the support block has `suppresses_bounce`, or when incoming downward velocity is
   smaller than effective gravity; otherwise use the maximum of entity bounciness and block bounce
   restitution (nonliving entities multiply block restitution by `0.8`). The vertical restitution
   path applies its locked gravity/air-drag interpolation, emits `minecraft:bounce` and sets
   position-sync when any positive bounce occurred. Zero restitution zeros the collided component.
4. On the server, or on a locally authoritative client, continue only when the entity's
   movement-emission mode emits anything and the entity is not a passenger. Compute full distance
   `float(actual.length*0.6000000238418579)` and horizontal distance with the same multiplier; add
   full distance to `flyDist` and add full distance to `moveDist` when climbable, otherwise
   horizontal distance. When `moveDist>nextStep` and the current on-block is nonair, run the
   legacy-on-block vibration/sound callback first, permitting sounds and events according to the
   emission mode and whether both on-positions agree; when positions differ, OR in the
   current-on-block callback with sounds disabled and events gated by the mode. If either callback
   emits, advance `nextStep` to `float(int(moveDist)+1)`. If the current on-block is air, use the
   flapping path instead. The intervening in-water fallback advances `nextStep`, optionally plays
   the swim sound and emits `minecraft:swim` according to the same mode. `aiStep` later applies
   contact effects from the recorded movement segments. Finally multiply stored horizontal velocity
   by `getBlockSpeedFactor`; Y is unchanged.

**Branches and aborts:**

No-physics; piston restriction and zero return; stuck multiplier; sneak/edge backoff; exact-zero
request; empty/nonempty collision lists; world-border proximity; axis-order tie; downward versus
grounded step eligibility; every candidate height; insignificant actual movement; nonauthoritative
vertical-flag preservation; entity removal; movement simulation; bounce suppression/restitution;
passenger/emission/local-authority; block speed factor.

**Constants and randomness:**

Geometry uses doubles except step height/candidate arrays and block/attribute properties, which use
floats. Shape clipping and edge-support shrink use `1e-7`; shape clipping zero is strict `<1e-7`.
Collision-flag equality and piston zeroing use `9.999999747378752e-6`, strict `<` for equality and
`<=` for piston zeroing. Piston accumulated range is `[-0.51,+0.51]`; edge-backoff quantum is
`0.05`; step-volume downward expansion is `9.999999747378752e-6`; movement recording cap is `100`;
nonliving block bounce factor is `0.8`; movement sound/event distance scale is `0.6000000238418579`.
Candidate selection compares horizontal squared doubles strictly. No RNG is consumed by clipping;
selected sound implementations may consume presentation RNG.

**Side effects:**

Position/AABB and movement records; horizontal/vertical/below/minor collision and on-ground/support
state; fall distance/damage; stored velocity restitution and block speed scaling; bounce/step/swim
game events, position synchronization, sounds and block-contact callbacks. Shapes and callbacks are
obtained from the concrete registry content and may have their own rules.

**Gates:**

Mover type, no-physics and local-authority flags, player abilities/sneaking surface retention,
step-height attribute, entity/block/world-border shapes, block tags/properties, bounce suppression,
movement simulation/emission and passenger status. Difficulty does not change geometry.

**Boundary cases and quirks:**

Y always clips before horizontal axes; only X/Z order changes, and equality selects X before Z. A
higher step is accepted as soon as it strictly improves horizontal squared distance, so ascending
candidate order matters and it does not search for the globally longest candidate after that.
Collision booleans intentionally use a looser epsilon than shape clipping, while vertical collision
uses exact comparison. A requested movement may update position even when its actual vector is tiny
if clipping did not remove enough squared length. Edge backoff quantizes X and Z independently
before the diagonal pass. After a server player packet uses this transaction as a validation probe,
`PLY-MOVE-VALIDATE-001` may snap to the packet target and overwrite final on-ground/horizontal flags
with accepted packet bits; that is an explicit caller transaction, not a different clipping result.
Content can have an empty collision shape but nondefault friction/speed/contact behavior; the
generic family mapping does not classify that content as data-only.

**Evidence:**

`OFF-SERVER-001`, `OFF-CLIENT-001`. Anchors:
`net.minecraft.world.entity.Entity#move(net.minecraft.world.entity.MoverType,net.minecraft.world.phys.Vec3)`,
`net.minecraft.world.entity.Entity#collide(net.minecraft.world.phys.Vec3)`,
`net.minecraft.world.entity.Entity#collideBoundingBox(net.minecraft.world.entity.Entity,net.minecraft.world.phys.Vec3,net.minecraft.world.phys.AABB,net.minecraft.world.level.Level,java.util.List)`,
`net.minecraft.world.entity.Entity#collectCollidersIgnoringWorldBorder(net.minecraft.world.entity.Entity,net.minecraft.world.level.Level,java.util.List,net.minecraft.world.phys.AABB)`,
`net.minecraft.world.entity.Entity#collideWithShapes(net.minecraft.world.phys.Vec3,net.minecraft.world.phys.AABB,java.util.List)`,
`net.minecraft.world.entity.Entity#collectCandidateStepUpHeights(net.minecraft.world.phys.AABB,java.util.List,float,float)`,
`net.minecraft.world.entity.Entity#restituteMovementAfterCollisions(net.minecraft.world.level.block.state.BlockState,boolean,boolean,net.minecraft.world.phys.Vec3)`,
`net.minecraft.world.entity.Entity#limitPistonMovement(net.minecraft.world.phys.Vec3)`,
`net.minecraft.world.entity.Entity#addMovementThisTick(net.minecraft.world.entity.Entity$Movement)`,
`net.minecraft.world.entity.player.Player#maybeBackOffFromEdge(net.minecraft.world.phys.Vec3,net.minecraft.world.entity.MoverType)`,
`net.minecraft.core.Direction#axisStepOrder(net.minecraft.world.phys.Vec3)`,
`net.minecraft.world.phys.shapes.Shapes#collide(net.minecraft.core.Direction$Axis,net.minecraft.world.phys.AABB,java.lang.Iterable,double)`,
and `net.minecraft.util.Mth#equal(double,double)`.

**Test vectors:**

(1) Empty, full-cube, slab, stair, fence and entity shapes with cardinal/diagonal/equal XZ requests;
compare clipped doubles and axis order. (2) Values immediately below/equal/above both epsilons;
assert shape zeroing, position update and collision booleans independently. (3) Step heights with
duplicate, normal-Y-equal, negative, over-limit and multiple improving candidates; assert first
improving ascending height. (4) Ground-edge retention with X-only, Z-only and diagonal gaps around
every `0.05` decrement. (5) `noPhysics`, each piston axis/tick cap, stuck multiplier and zero-result
early returns. (6) Downward/upward/horizontal collisions across bounce suppression, living/nonliving
restitution and block-speed factors. (7) Add 101+ movement segments and assert the exact oldest-pair
compaction, then verify contact callback traversal.
