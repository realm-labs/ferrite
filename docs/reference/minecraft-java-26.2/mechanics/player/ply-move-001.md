# Player mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `PLY-MOVE-001` — Ordinary ground and air travel integrates input, jump, gravity and drag in source order

**Parent:** `PLY-001`, `PLY-003`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — the ordinary ground/air branch is complete below. Fluid travel, swimming
steering, fall flying and ability flight are owned by `PLY-MOVE-SPECIAL-001`; movement-packet
admission and correction are owned by `PLY-MOVE-VALIDATE-001`.

**Applies when:**

A live, non-ridden player is locally authoritative or is the effective server simulation, can
simulate movement, is not in the fluid-travel branch, is not fall flying and does not have ability
flight changing the wrapper result. The rule consumes the already selected `xxa`, `yya`, `zza`, yaw,
velocity, effects, attributes and supporting block state for one `LivingEntity#aiStep`.

**Authoritative state:**

Double position and velocity; float yaw and input components; `jumping`, `noJumpDelay`, `onGround`,
fall distance, sprint and climbable flags; current collision box; block below/in-block states;
`movement_speed`, `jump_strength`, `gravity`, `friction_modifier`, `air_drag_modifier` and
`step_height` attribute values; levitation, jump boost and slow falling instances. The server owns
gameplay state. The locally controlled client executes the same dynamics predictively; packet
convergence is not part of this leaf.

**Transition and ordering:**

Within `LivingEntity#aiStep`, decrement a positive `noJumpDelay` first. Before input is applied,
read velocity. For a player, replace both horizontal components with zero only when
`x*x+z*z < 9.0e-6`; for every living entity replace `y` with zero when `abs(y) < 0.003`. Call
`applyInput`; an immobile entity then clears `jumping`, `xxa` and `zza`. Jump dispatch runs before
travel. If `jumping && isAffectedByFluids`, read lava height when in lava, otherwise water height;
let `inWater = isInWater && height>0`, and let fluid-jump threshold be `0` when eye height is
`<0.4`, else `0.4`. Deep water (`inWater && (!onGround || height>threshold)`) adds the separate
water-liquid jump impulse. Otherwise lava with `!onGround` or lava height `>threshold` adds the
lava-liquid impulse. Otherwise, when grounded or in water at/below the threshold, `noJumpDelay==0`
calls `jumpFromGround` and then stores `noJumpDelay=10`. Releasing jump or being unaffected by
fluids resets the delay to zero. Liquid impulses and their subsequent medium dynamics are owned by
`PLY-MOVE-SPECIAL-001`; the ordinary ground jump is complete here.

`jumpFromGround` computes Java float
`J = float(jump_strength) * blockJumpFactor + 0.1f*(jumpBoostAmplifier+1)` (the last term is zero
without jump boost). If `J <= 1.0e-5f`, abort with no mutation. Otherwise set velocity Y to
`max(J,currentY)`. When sprinting, additionally add
`(-sin(yaw*0.017453292f)*0.2, 0, cos(yaw*0.017453292f)*0.2)` using the source float trigonometric
path, then set `needsSync=true`.

After optional fall-flying bookkeeping, construct the travel input as three doubles obtained by
widening the current float `(xxa,yya,zza)`. `Player#travel` delegates unchanged to its superclass
for this ordinary branch. `LivingEntity#travel` reads the fluid state at `blockPosition`; it selects
ordinary air travel unless `(in water || in lava) && isAffectedByFluids && !canStandOnFluid(state)`,
and unless fall flying selects its separate branch.

Ordinary travel executes the following transaction:

1. Let `below = getBlockPosBelowThatAffectsMyMovement()`. If grounded, let
   `F = clamp(1 - (1 - blockBelow.friction) * friction_modifier, 0, 1)` in Java float operations;
   otherwise `F=1.0f`.
2. Select acceleration scale `A`. Grounded: when `F>0.6`,
   `A=float(movement_speed)*0.21600002f/(F*F*F)`; otherwise `A=float(movement_speed)`. Airborne
   player: `A=0.025999999f` while sprinting and `0.02f` otherwise. Ability flight uses another
   branch. The vanilla player base movement speed is `0.10000000149011612`; sprinting is a transient
   `ADD_MULTIPLIED_TOTAL` modifier of `0.30000001192092896`, so an otherwise unmodified grounded
   player reads approximately `0.13f` through `getSpeed`.
3. Let input squared length be `L`. If `L<1.0e-7`, acceleration is zero. Otherwise normalize only
   when `L>1`, scale all three components by `A`, and rotate horizontal components by yaw:
   `ax=s.x*cos-s.z*sin`, `ay=s.y`, `az=s.z*cos+s.x*sin`, where `sin/cos` consume
   `float(yaw*0.017453292f)` and return floats widened to double. Add this vector to current
   velocity.
4. If on a climbable, reset fall distance; clamp velocity X/Z independently to
   `[-0.15000000596046448,+0.15000000596046448]` and Y to at least `-0.15000000596046448`. A
   descending player who is suppressing ladder slide has Y set to zero unless its in-block state is
   scaffolding. Pass that velocity to `Entity#move(SELF,...)`, whose exact clipping transaction is
   `PLY-COLLISION-001`.
5. Read the post-move velocity. If `(horizontalCollision || jumping)` and either currently climbable
   or previously in powder snow while eligible to walk on powder snow, replace only Y with `0.2`.
6. Let `vy` be that result's Y. With levitation amplifier `n`, set `vy += (0.05*(n+1)-resultY)*0.2`.
   Otherwise subtract effective gravity. Effective gravity is the current `gravity` attribute,
   except while `currentY<=0` with slow falling it is `min(gravity,0.01)`. Only a client whose
   `below` chunk is absent substitutes `vy=-0.1` above minimum build height or `0` at/below it.
7. Unless `shouldDiscardFriction`, compute `D=clamp(1-(1-0.91f)*air_drag_modifier,0,1)`, horizontal
   multiplier `F*D`, and vertical multiplier `clamp(1-(1-0.98f)*air_drag_modifier,0,1)` for an
   ordinary player. Store `(resultX*F*D, vy*verticalMultiplier, resultZ*F*D)`. The default modifier
   values are `1`, so defaults reduce to grounded horizontal `F*0.91f`, airborne horizontal `0.91f`,
   and vertical `0.98f`. An omnidirectional air mover uses `D` vertically; an entity discarding
   friction stores the unscaled `(resultX,vy,resultZ)`.

After travel returns, `aiStep` invokes accumulated block-effect processing on the server and on a
locally authoritative client; the generic movement transaction has already multiplied horizontal
velocity by the contacted block speed factor. Animation follows on the client.

**Branches and aborts:**

Cannot-simulate and interpolation paths; immobility; held/released jump; shallow water/lava jump
routing; grounded/airborne; low/high friction; climbable/scaffolding/ladder-slide suppression;
powder-snow climb; levitation; slow falling; missing client chunk; discard-friction and
omnidirectional-air overrides. Fluid dynamics, swimming-look Y steering, fall flying, ability
flight, riding, sneaking edge backoff and packet validation are intentionally delegated to their
named leaves.

**Constants and randomness:**

All literal values and comparison operators are stated above. Attribute values are doubles and are
converted to float at the indicated reads. Input components begin as floats, are widened to double,
normalized/scaled in `Vec3`, and combine with float trigonometric results. Default registered values
relevant here are air-drag modifier `1`, friction modifier `1`, gravity `0.08`, jump strength
`0.41999998688697815`, and step height `0.6`; the player overrides movement speed to
`0.10000000149011612`. There is no RNG consumption.

**Side effects:**

Velocity/position and collision flags through `PLY-COLLISION-001`; jump cooldown and synchronization
flag; fall-distance reset; movement recordings; fall/landing processing, step/swim sounds, game
events and block-contact effects through the generic move/effect pipeline. This leaf does not send
or validate movement packets.

**Gates:**

Effective/local authority, can-simulate/effective-AI, mobility, fluid/fall-flying/ability-flight
dispatch, current effects and attributes, block friction/jump/speed properties, collision shapes and
chunk presence on the client. Difficulty and gameplay RNG do not gate ordinary integration.

**Boundary cases and quirks:**

The player's horizontal dead zone is on squared magnitude and zeroes both axes together; Y uses a
separate absolute threshold. `J` is compared after float arithmetic. Ground acceleration has a
discontinuous formula choice at exactly `F=0.6` (`getSpeed`, not the cubic formula). Extremely
modified friction clamps into `[0,1]`; at `F=0` the low-friction branch avoids division. Input Y
participates in normalization even though ordinary player input normally supplies zero. Gravity is
applied after collision and the ladder/powder-snow Y replacement. Sprint affects both the
movement-speed attribute and airborne acceleration, and adds a separate jump impulse.

**Evidence:**

`OFF-SERVER-001`, `OFF-CLIENT-001`. Anchors: `net.minecraft.world.entity.LivingEntity#aiStep()`,
`net.minecraft.world.entity.LivingEntity#jumpFromGround()`,
`net.minecraft.world.entity.LivingEntity#getJumpPower(float)`,
`net.minecraft.world.entity.LivingEntity#getEffectiveGravity()`,
`net.minecraft.world.entity.LivingEntity#travel(net.minecraft.world.phys.Vec3)`,
`net.minecraft.world.entity.LivingEntity#travelInAir(net.minecraft.world.phys.Vec3)`,
`net.minecraft.world.entity.LivingEntity#handleRelativeFrictionAndCalculateMovement(net.minecraft.world.phys.Vec3,float)`,
`net.minecraft.world.entity.LivingEntity#handleOnClimbable(net.minecraft.world.phys.Vec3)`,
`net.minecraft.world.entity.LivingEntity#getFrictionInfluencedSpeed(float)`,
`net.minecraft.world.entity.player.Player#travel(net.minecraft.world.phys.Vec3)`,
`net.minecraft.world.entity.player.Player#getFlyingSpeed()`,
`net.minecraft.world.entity.player.Player#createAttributes()`,
`net.minecraft.world.entity.Entity#moveRelative(float,net.minecraft.world.phys.Vec3)`,
`net.minecraft.world.entity.Entity#getInputVector(net.minecraft.world.phys.Vec3,float,float)`, and
`net.minecraft.world.entity.ai.attributes.Attributes#register(java.lang.String,net.minecraft.world.entity.ai.attributes.Attribute)`.

**Test vectors:**

(1) With default attributes, replay zero, cardinal and diagonal input on friction `0.6`, immediately
below/above `0.6`, and `1`; compare every double velocity/position. (2) Set horizontal squared
velocity immediately below/equal/above `9e-6`, and each component around `0.003`; assert the
asymmetric thresholds. (3) Jump at `J<=1e-5`, with positive Y, sprinting, jump boost, altered block
jump factor, low ceiling and held jump through the ten-tick delay. (4) Exercise ladder X/Z/Y clamps,
scaffolding and slide suppression, plus the post-collision `0.2` branch. (5) Apply levitation and
slow falling at positive, zero and negative Y; vary gravity, friction and drag beyond their clamp
ranges. (6) Remove the client chunk below above/at minimum Y and show that only the predictive
client substitutes Y. (7) Feed nonzero input Y and squared lengths immediately around `1e-7` and `1`
to lock normalization and float-to-double behavior.
