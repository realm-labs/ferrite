# Player mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `PLY-MOVE-SPECIAL-001` — Fluid, swimming, fall-flying and ability-flight dynamics remain separate modes

**Parent:** `PLY-003`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — the complete fluid, swimming steering, fall-flying, glider maintenance and
ability-flight transactions are fixed below; collision remains delegated to `PLY-COLLISION-001`.

**Applies when:**

The travel dispatcher selects water, lava, another standable/affecting fluid, swimming steering,
fall flying or player ability flight.

**Authoritative state:**

Medium membership/height/flow, pose/look/input, velocity, abilities, effects, attributes, equipment
and collision state.

**Transition and ordering:**

`LivingEntity#travel` selects fluid when `(isInWater || isInLava) && isAffectedByFluids` and the
fluid state at `blockPosition` is not standable; otherwise fall flying precedes ordinary air.

Fluid travel snapshots whether entry velocity Y is nonpositive, old Y and effective gravity. Water
uses horizontal slowdown 0.9 while sprinting, otherwise `getWaterSlowDown` (player default 0.8), and
input acceleration 0.02. Read `water_movement_efficiency`; halve it off ground. When positive,
interpolate slowdown toward `0.54600006F` and acceleration toward current movement speed by that
value. Dolphin's Grace replaces slowdown with 0.96 after this interpolation. Apply relative input,
move through ordinary collision, replace Y with 0.2 on horizontal collision plus climbable, multiply
velocity by `(slowdown,0.8,slowdown)`, then apply fluid falling adjustment. That adjustment is absent
for sprinting or zero gravity; otherwise subtract gravity/16 from Y, except entry-falling motion with
`abs(y-0.005)>=0.003` and `abs(y-gravity/16)<0.003` snaps Y to -0.003. Finally, if horizontally
colliding and the current vector fits after adding `0.6-currentY+oldY` to Y, replace Y with 0.3.
Ridden entities in `can_float_while_ridden` additionally add 0.04 Y when they are vehicles and water
height exceeds the fluid-jump threshold.

Lava first applies relative input at 0.02 and moves. At height less than or equal to its fluid-jump
threshold it multiplies `(0.5,0.8,0.5)` and applies the same fluid-falling adjustment; in deep lava it
multiplies all axes by 0.5 without that adjustment. It then always subtracts effective gravity/4
when gravity is nonzero, including after the shallow gravity/16 step, and runs the same 0.3 exit
test. Liquid jump input itself adds 0.04 Y; the local player's held shift in water independently adds
-0.04 before superclass travel.

Before superclass travel, a nonpassenger swimming player steers Y toward look-vector Y. Multiplier
is 0.085 when look Y is below -0.2, otherwise 0.06. The adjustment is allowed while looking
nonpositive, while jumping, or while the fluid state at `(x,y+0.9,z)` is nonempty; an upward-looking,
nonjumping swimmer at an empty surface does not receive it.

Fall flying on a climbable delegates one ordinary-air tick then clears flag 7 with the deliberate
true→false transition. Otherwise, with look vector `L`, pitch radians `p`, horizontal look length
`h`, old horizontal speed `s`, effective gravity `g` and `c=cos(p)^2`, update velocity in order:

1. add Y `g*(-1+c*0.75)`;
2. when Y is negative and `h>0`, let `q=Y*-0.1*c` and add `(Lx*q/h,q,Lz*q/h)`;
3. when `p<0` and `h>0`, let `q=s*(-sin(p))*0.04` and add `(-Lx*q/h,q*3.2,-Lz*q/h)`;
4. when `h>0`, steer X/Z 10% toward the look-horizontal direction at speed `s`;
5. multiply `(0.99,0.98,0.99)`, then move through collision.

On the server, horizontal collision compares old and post-move horizontal speed and computes float
`damage=(old-new)*10-3`; strictly positive damage plays the integer-selected fall sound at volume/
pitch 1 and deals fly-into-wall damage. Fall-flying validity is checked server-side: not grounded,
not a passenger, no levitation, ability flight off for players, and at least one equipped item whose
`GLIDER` component is paired with an `EQUIPPABLE` component for that exact slot and whose next damage
would not break it. Invalidity clears flight. Every ten flight ticks emits the elytra-glide game
event; every even such interval (20 ticks) chooses uniformly among all currently valid glider slots
and damages the selected stack by one before the event.

Ability flight is a `Player#travel` wrapper, not a fourth `LivingEntity` dispatcher. Local controlled
input adds `(jump-shift)*abilitiesFlyingSpeed*3` to Y before travel. Super travel uses the ordinary
or selected fluid branch and the player's airborne `getFlyingSpeed`: base ability speed, doubled
while sprinting. After it returns, the wrapper restores Y to entry Y×0.6, discarding superclass
gravity/collision-adjusted Y while retaining its position and horizontal result. Ability flight
suppresses swimming/climbable, resets fall distance each player tick, and landing disables flight for
nonspectator local players; the server accepts flight only while `mayfly` as specified by
`PLY-INPUT-001`.

**Branches and aborts:**

Standable or unaffected fluid routes out of fluid travel; water wins when both water/lava flags are
present; shallow/deep lava differs; sprint disables water gravity adjustment; climbable stops fall
flight; ability flight wraps whatever superclass medium branch is selected; glider loss, impending
break, ground, passenger or levitation ends server fall flight.

**Constants and randomness:**

All literal constants and comparisons appear above. Relative movement, collision and look vector
math use the ordinary float/double paths owned by `PLY-MOVE-001`/`PLY-COLLISION-001`. Fluid and
fall-flight integration consume no RNG. Only the 20-tick multiple-valid-glider durability selection
uses entity RNG.

**Side effects:**

Position/velocity via `PLY-COLLISION-001`, pose, fall state, equipment durability, sounds/events and
particles as selected by the audited mode.

**Gates:**

Fluid tags/heights, `isAffectedByFluids`, `canStandOnFluid`, swimming/fall-flying flags, ability
flight, effects and equipment.

**Boundary cases and quirks:**

Fluid selection reads membership flags but standability from the fluid state at block position.
Shallow lava receives gravity twice (1/16 inside adjustment and 1/4 afterward) for a nonsprinting
player. Sprinting water receives neither fluid gravity adjustment nor a replacement gravity path.
Ability flight preserves post-move position but overwrites velocity Y from its pre-super value.
Glider durability can select any valid equipment slot, not only chest.

**Evidence:**

`OFF-SERVER-001`, `OFF-CLIENT-001`:
`net.minecraft.world.entity.player.Player#travel(net.minecraft.world.phys.Vec3)`,
`net.minecraft.world.entity.LivingEntity#travel(net.minecraft.world.phys.Vec3)`,
`net.minecraft.world.entity.LivingEntity#travelInFluid(net.minecraft.world.phys.Vec3)`, and
`net.minecraft.world.entity.LivingEntity#travelInWater`, `#travelInLava`,
`#getFluidFallingAdjustedMovement`, `#travelFallFlying`, `#updateFallFlyingMovement`,
`#handleFallFlyingCollisions`, `#updateFallFlying`, and `#canGlideUsing`.

**Test vectors:**

Replay water sprint/default efficiency 0/1 and airborne halving, Dolphin's Grace, gravity/snap and
climb/exit branches; lava exact threshold and both gravity applications; swimmer look around -0.2
and surface gate; fall-flight pitch/horizontal/vertical endpoints, climb stop and collision damage
zero boundary; valid glider slots, next-break loss, 10/20-tick event/damage; ability jump/shift,
sprint speed, fluid wrapper, Y overwrite, landing and mayfly loss. `EXP-PLY-004` is the executable
bit-level regression trace.
