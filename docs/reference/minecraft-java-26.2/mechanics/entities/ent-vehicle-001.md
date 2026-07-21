# Entities mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ENT-VEHICLE-001` — Boat and minecart authority branches before family-specific motion

**Parent:** `ENT-002`, `ENT-003`, `PLY-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — common damage, boat media/input physics, both minecart movement implementations,
collision/passenger/dismount rules and every boat/minecart subtype hook are explicit in locked source.

**Applies when:**

A boat/raft/chest variant or minecart family entity ticks, collides, is controlled, mounted,
dismounted, damaged or activated.

**Authoritative state:**

Vehicle transform/velocity, damage/hurt values, passengers/order/controller/input, boat medium and
paddles/bubble timer, rail shape/power and movement feature flag, subtype fuel/fuse/container state.

**Transition and ordering:**

Common server damage rejects removed/invulnerable, flips hurt direction, sets hurt time `10`, adds
`damage*10`, marks hurt and emits entity-damage. A creative attacker discards unless the source
forces destruction; otherwise damage strictly above `40` or a forcing source destroys. Destruction
kills, then drops the named item with custom name only when `entityDrops`; mob explosions are ignored
when mob griefing is false. Each tick decays positive hurt time/damage by one.

Boat tick computes status first (`UNDER_FLOWING_WATER`, `UNDER_WATER`, `IN_WATER`, positive averaged
land friction, else air). Underwater time reaches `60` before server ejection. After base tick and
interpolation, only the local authoritative instance floats; without a player first passenger it
clears paddles, client authority also applies input and sends paddle state, then moves. A
nonauthoritative instance zeros velocity. It applies block effects twice, advances bubble state and
paddles, then scans the box inflated `(0.2,-0.01,0.2)` in level order: a server boat without player
controller auto-mounts eligible living nonpassengers while under capacity/width/tag gates, otherwise
pushes by the asymmetric vertical-box rule.

Boat floating starts with gravity `-0.04`. Water uses buoyancy
`(waterLevel-y)/height`, friction `0.9`; flowing-underwater vertical `-0.0007`, friction `0.9`;
source-underwater buoyancy `0.01`, friction `0.45`; air friction `0.9`; land uses averaged block
friction, halved with player controller. Velocity becomes `(x*f, y+vertical, z*f)`, rotation delta
times `f`, then positive buoyancy changes Y to `(y+buoyancy*(0.04/0.65))*0.75`. Air→water snaps to
`waterAbove-height+0.101` only collision-free, zeros Y and selects water. Input changes rotation by
`-1/+1`; turn-in-place adds `0.005`, forward `0.04`, backward `-0.005`, then accelerates by yaw and
sets paddles. Down bubble expiry adds Y `-0.7` and ejects; up sets Y `2.7` with a player passenger or
`0.6` otherwise. Passenger attachments use indices/order, yaw clamps to ±`105`, and two-seat animals
get ID-parity 90/270 body/head offsets. Mount capacity is two and rejects when eye is in water.
Dismount tries the horizontal escape point at vehicle-top and block below, pose order then target
order, before generic fallback.

Minecart selects `OldMinecartBehavior` unless `MINECART_IMPROVEMENTS` is enabled. Common tick decays
damage, checks void/speed/portal, runs behavior, updates fluid/lava and ends first-tick. Off rail it
clamps X/Z to max speed, halves all motion on ground, moves, then applies air drag `0.95` while
airborne. Natural slowdown is behavior factor (`0.96` empty/`0.997` ridden old;
`0.975` empty/`0.997` ridden new), zeros Y, then multiplies all components by `0.95` in water.

Old rails apply gravity, locate rail/below, then slope acceleration `0.0078125` (×`0.2` water),
project horizontal speed capped at `2` onto the rail exits, allow first-player movement input
`*0.001` only below squared speed `0.01`, and process unpowered rail: below speed `0.03` stop,
otherwise X/Z ×`0.5` and Y zero. The cart is projected onto the rail line, moved with `0.75` scale
while ridden and max `0.4` land/`0.2` water, adjusted for slopes and natural slowdown. Powered rail
adds `0.06` along motion above speed `0.01`, otherwise starts at `0.02` away from an adjacent
conductor. Rotation flips at wrapped difference outside `[-170,170)`; rideable collision auto-mounts
nonplayer/non-golem/non-cart targets at squared speed `>=0.01`, otherwise pushes.

New rails use max speed `maxMinecartSpeed/20` (halved in water), substep across successive rails,
slope acceleration `max(0.0078125,speed*0.02)`, normalized player start impulse `0.001`, the same
halt thresholds, natural slowdown once, and powered addition `0.06` or conductor start `0.2`.
Movement stops at opposing V slopes below squared speed `0.005`; substeps/rotations are weighted into
three client interpolation ticks. Collision first tries pickup in box +`0.2`; only on a collision
does it push in the near-exact box and optionally finish the residual move.

Minecart-to-cart push requires server, physics and squared separation `>=1e-4`; legacy rejects facing
dot `<0.8`, improvements do not. Furnace priority transfers momentum through `0.2/0.95`; peers use
their mean then `0.2` plus impulses. Ordinary entities receive quarter impulse. Minecart dismount
tries direction offsets in pose order and heights standing/crouching `[0,1,-1]`, swimming `[0,1]`,
then the vehicle-top ceiling fallback.

Subtype hooks are part of this transaction: rideable cart activates by ejection and damage `50`;
furnace fuel accepts its tag only when `fuel+3600<=32000`, consumes one through item semantics,
pushes away from interaction, and uses propulsion/friction plus family max-speed factors; TNT primes
for `80`, collision explodes at squared speed `>=0.01`, destruction may shorten fuse to two
`nextInt(20)` draws, and power is `base + factor*nextDouble*1.5*min(sqrt(speedSq),5)` under
`tntExplodes`; hopper, spawner, command, chest and generic container carts execute their located
pickup/ticker/control/menu/loot/drop hooks after/beside common motion.

**Branches and aborts:**

Authority side, controller/passenger eligibility, medium, bubble direction, feature flag, rail
shape/power/conductor, collision type, damage source/creative/gamerules and subtype resources.

**Constants and randomness:**

All shared/family numeric constants are stated above. Ordinary motion consumes no RNG; paddle/
bubble visuals, furnace smoke, TNT fuse/power and subtype loot/spawner behavior consume their source
draws only on their branches.

**Side effects:**

Vehicle/passenger transforms and graph, corrections/input packets, pushes, block/rail effects,
paddles/bubbles, damage/destruction/drops, containers/loot, fuel/fuse/explosion, sounds/particles.

**Gates:**

Server/client/local authority, player controller/input, media/rail/feature state, passenger tags/
width/capacity/cooldown, collision, gamerules and subtype admission.

**Boundary cases and quirks:**

Boat calls block-effect application twice in succession. Boat land-friction division by sample count
relies on status admission having a positive result. Rideable `Minecart.interact` calls
`startRiding` in its outer condition and again on the server success-return branch; the second call
returns false because the player is already riding, so its literal server result follows that path.
Legacy and improved minecart trajectories are intentionally distinct feature-gated engines.

**Evidence:**

`OFF-SERVER-001`, `OFF-CLIENT-001`;
`net.minecraft.world.entity.vehicle.VehicleEntity#hurtServer`,
`net.minecraft.world.entity.vehicle.boat.AbstractBoat#tick`,
`net.minecraft.world.entity.vehicle.boat.AbstractBoat#floatBoat`,
`net.minecraft.world.entity.vehicle.boat.AbstractBoat#controlBoat`,
`net.minecraft.world.entity.vehicle.minecart.AbstractMinecart`,
`net.minecraft.world.entity.vehicle.minecart.OldMinecartBehavior`,
`net.minecraft.world.entity.vehicle.minecart.NewMinecartBehavior` and every registered boat/raft/
minecart subtype; `EXP-ENT-004`.

**Test vectors:**

Damage exactly/above `40`; every boat status transition/input/bubble boundary; two passengers and
auto-mount order; dismount pose search; all 10 rail shapes powered/unpowered/water; legacy versus
improved substeps/collisions; cart/cart furnace priority; fuel max; TNT fuse/collision/gamerule;
container/spawner/command subtype activation.
