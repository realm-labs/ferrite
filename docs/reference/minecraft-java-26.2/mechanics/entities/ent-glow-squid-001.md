# Entities mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ENT-GLOW-SQUID-001` — Glow Squids combine ageable squid propulsion with a synchronized post-hit darkness clock

**Parent:** `ENT-001`, `ENT-LIFECYCLE-001`, `ENT-002`,
`ENT-VEHICLE-001`, `ENT-004`, `ENT-PROJECTILE-001`, `ENT-005`,
`ENT-DAMAGE-001`, `ENT-BLOCK-001`, `ENT-DAMAGE-REDUCE-001`,
`ENT-KNOCKBACK-001`, `ENT-006`, `ENT-EFFECT-001`, `ENT-007`,
`ENT-DEATH-001`, `MOB-001`, `MOB-AI-001`, `MOB-002`,
`MOB-SPAWN-001`, `MOB-003`, `MOB-DESPAWN-001`, `MOB-005`,
`MOB-BREED-001`, `ITM-ENCHANT-001`, `PLY-AUTOJUMP-001`, `WGEN-005`,
`WGEN-PORTAL-001`, `CLI-001`, `CLI-006`, `CLI-EFFECT-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked registration, complete `GlowSquid`, `Squid` and
ageable-water superclass paths, both Squid goals, placement/category code, all
66 biomes, five direct tags, loot, Spawn Egg, two schema contexts, all 1,212
templates and exact adult/baby client resources close protocol entity ID `61`.

**Applies when:**

`minecraft:glow_squid` is constructed, naturally spawned, spawned by an Egg,
spawner, command or custom selector, loaded, age-locked, leashed, moved,
attacked, targeted by an Axolotl, killed, synchronized or rendered.

**Authoritative state:**

Entity protocol ID `61` constructs `GlowSquid` in
`UNDERGROUND_WATER_CREATURE`. Registration fixes adult width/height
`0.8/0.8`, eye height `0.4`, client tracking range `10` and builder-default
update interval `3`. Squid supplies maximum health `10`, inherited follow
range `16`, movement emission `EVENTS`, leashability, sound volume `0.4`,
gravity `0.08` and Water path malus `0`.

Ageable state changes the live dimensions: a baby is `0.5×0.5` with eye height
`0.37`, not a uniform half-scale of the adult collision box. Inherited signed
`Age`, `ForcedAge` and Boolean `AgeLocked` persistence and their age tick
retain `MOB-BREED-001`.

Entity/Living/Mob occupy synchronized metadata slots `0..15`; `AgeableMob`
adds Boolean baby and age-lock slots `16/17`. Glow Squid adds slot `18`, using
serializer ID `1` (`INT`) with default `0`. It persists that same signed value
under `DarkTicksRemaining`; a missing or wrong-type key reads as `0`. Loading
and the private setter do not clamp it.

Squid also holds nonpersisted body X/Z rotations, previous rotations, tentacle
phase/angle and previous values, rotation speed and a movement vector. Its
constructor initializes the vector to zero and consumes one float to set
`tentacleSpeed = 0.2 / (nextFloat + 1)`, in `(0.1,0.2]`.

**Transition and ordering:**

### Darkness clock, damage and ink

Each Glow-Squid `aiStep` first completes the entire inherited Squid step. It
then reads slot `18`; when positive it writes `value-1`, otherwise it performs
no timer write. Finally it evaluates three randomized position helpers and
calls `Level.addParticle(GLOW, x, y, z, 0, 0, 0)`. The position is:

- `x = entityX + (2*nextDouble-1)*width*0.6`;
- `y = entityY + nextDouble*height`; and
- `z = entityZ + (2*nextDouble-1)*width*0.6`.

On a client, `ClientLevel` requests one local Glow particle each AI step,
subject to its ordinary distance and particle-status admission. On a
dedicated server, the inherited `Level.addParticle` implementation is a
no-op, but all three argument-producing doubles have already been consumed;
there is no particle packet. Glow and Glow-Squid-Ink have locked particle
protocol IDs `107/106`.

`GlowSquid.hurtServer` first calls `Squid.hurtServer`. Squid first invokes the
complete generic damage transaction. Only when that returns true and the
post-transaction `getLastHurtByMob()` is non-null does Squid spawn ink and
return true; otherwise it returns false. Thus a fresh environmental hit can
already have changed health while the Squid/Glow-Squid wrapper returns false,
emits no ink and does not reset darkness. Conversely, a retained non-null last
attacker is the tested state; the current damage source need not itself be a
Mob.

On the admitted branch Squid first plays Glow-Squid squirt, then emits exactly
30 Glow-Squid-Ink particles from `(0,-1,0)` rotated by previous body X and
negative previous body yaw and added to entity position. Each particle:

1. draws X/Z direction components `nextFloat*0.6-0.3`, with Y `-1`;
2. rotates that direction by the same previous rotations;
3. draws a scale of `0.1+2*nextFloat` for a baby or
   `0.3+2*nextFloat` for an adult; and
4. invokes `ServerLevel.sendParticles` with count `0`, the origin raised by
   `0.5`, the scaled vector as offsets and speed
   `0.10000000149011612`.

For an admitted recipient, count zero requests one particle whose client
velocity is `maxSpeed` times each packet offset, so the nominal velocity is
`0.1` times the scaled vector. This is 90 floats and 30 count-zero particle
sends after the sound. Only after Squid returns true does Glow Squid write
darkness to `100`; failed generic damage or a null last attacker does not
overwrite the prior timer.

The timer decrements on both logical sides. Positive loaded values are
unbounded; negative values remain unchanged. The dirty synchronized integer
allows the server to correct the client's locally decremented copy.

### Goals and propulsion

Glow Squid inherits exactly two control-free goals and no target goal:

- priority `0`, `SquidRandomMovementGoal`, whose `canUse` is always true; and
- priority `1`, `SquidFleeGoal`, admitted while in water with a non-null last
  attacker at squared distance strictly below `100`.

Because neither declares a control flag, both can run together; the later
flee tick can replace the random goal's movement vector.

The random goal first zeroes the vector when `noActionTime > 100`. Otherwise
it draws `nextInt(reducedTickDelay(50))`. A zero result, not touching water,
or an absent vector with squared length at most
`9.999999747378752E-6` samples an angle `a=nextFloat*2*pi` and vertical float
and writes `(cos(a)*0.2, -0.1+nextFloat*0.2, sin(a)*0.2)`. Short-circuit order
means the cadence draw precedes the water/vector reads.

Flee starts `fleeTicks=0` and updates every tick. It forms the unnormalized
vector `d = squidPosition-attackerPosition`, then inspects the block/fluid at
`squidPosition+d`. A Water-tag fluid or air block admits a replacement;
anything else leaves the old vector. Although source calls `d.normalize()`,
it discards the immutable return value. Let `D=length(d)`: factor `F` starts
at `3`, and for `D>5` becomes `3-(D-5)/5`. When `F>0`, the original `d` is
scaled by `F`; an air destination then clears its Y component. The movement
vector becomes that result divided by `20`. At flee ticks `5,15,25,...` the
server goal calls `Level.addParticle(BUBBLE, position, 0,0,0)`, which is the
same server-side no-op and sends no particle.

Each Squid AI step copies the four previous animation values, then adds
`tentacleSpeed` to phase. Above `2*pi`, a client clamps phase to the float
`2*pi`; a server subtracts that float, draws `nextInt(10)`, rerandomizes
tentacle speed with the constructor formula only on zero, then broadcasts
entity event `19`. Handling event `19` resets phase to `0`.

In water, phase below `pi` sets
`f=phase/pi` and `tentacleAngle=sin(f*f*pi)*pi*0.25`. Above strict
`f>0.75`, a locally authoritative instance copies the movement vector into
delta movement and sets rotation speed to `1`; otherwise it multiplies
rotation speed by `0.8`. In the other half-cycle it sets tentacle angle to
zero, multiplies authoritative delta by `0.9` and rotation speed by `0.99`.
Body yaw converges by `0.1` toward `-atan2(dx,dz)` in degrees and becomes
entity yaw; Z body rotation adds `pi*rotationSpeed*1.5`; X body rotation
converges by `0.1` toward `-atan2(horizontalSpeed,dy)` in degrees.

Out of water, tentacle angle is `abs(sin(phase))*pi*0.25`. On the server the
Y velocity becomes `0.05*(levitationAmplifier+1)` under Levitation or loses
gravity `0.08`, is multiplied by air drag `0.98`, and X/Z velocity are set to
zero. X body rotation converges by `0.02` toward `-90` degrees. `travel`
ignores its input and moves `SELF` by current delta.

### Age, air and spawn finalization

Glow Squid is an `AgeableMob`, but base `canBreed()` is false, it is not an
`Animal`, has no food selector and registers no breeding goal. Its overridden
offspring factory would create `GLOW_SQUID` with reason `BREEDING` if an
external caller invoked it; the factory alone creates no baseline admission.
A baby can use a Golden Dandelion because Glow Squid is absent from
`cannot_be_age_locked`; the inherited interaction toggles its lock and
resets it to baby age under `MOB-BREED-001`.

Squid spawn finalization supplies `AgeableMobGroupData` with baby chance
`0.05`. The first member has group size zero and cannot be made a baby.
Every later finalized member draws a level float and becomes age `-24000`
when the draw is at most `0.05`, then group size increments. The inherited
cluster maximum remains `4`, even though the biome row requests groups `4..6`.

While alive and outside water, `AgeableWaterCreature` decrements the
pre-super-tick air value. At `-20` or below it resets air to zero and applies
`2` Drown damage; in water or while dead it resets air to `300`. The direct
`can_breathe_under_water` tag also makes the generic Living-Entity underwater
branch skip drowning.

### Registered placement and biome selection

Glow Squid registers placement `IN_WATER` with heightmap
`MOTION_BLOCKING_NO_LEAVES`. The placement-type gate first requires a
non-null type, candidate inside the world border, Water-tag fluid at the
candidate and a block above that is not a redstone conductor. The species
predicate then, without RNG:

1. requires candidate Y less than or equal to `seaLevel-33`;
2. requires `getRawBrightness(position, 0) == 0`; and
3. requires the candidate state to be exactly `Blocks.WATER`.

The final condition is stronger than the placement Water-fluid tag:
waterlogged or other Water-tag states fail. Equality at `seaLevel-33`
passes; any raw brightness above zero fails. Spawn obstruction later requires
the constructed entity to be unobstructed.

Exactly `53` of the `66` locked biomes select Glow Squid in
`underground_water_creature`, always weight `10`, group `4..6`. The omitted
13 are Basalt Deltas, Crimson Forest, Deep Dark, End Barrens, End Highlands,
End Midlands, Small End Islands, The End, The Void, Nether Wastes, Soul Sand
Valley, Sulfur Caves and Warped Forest. Category cap is `5`, friendly is true,
persistence is false and no-despawn/despawn distances are `32/128`. Generic
candidate, cap, pack, insertion and despawn transactions retain their owners.

### Loot, tags, sounds and item projection

The entity loot table has type `entity`, sequence
`minecraft:entities/glow_squid` and one roll. It emits Glow Ink Sac raw item
ID `1093` with base uniform integer count `1..3`. With a living attacking
entity and Looting level `L>0`, one fresh float `U` adds `round(L*U)`;
otherwise that bonus spends no draw. Generic death admission can also yield
XP `1+nextInt(3)`.

Glow Squid belongs directly to exactly five entity-type tags:

- `aquatic`, transitively selecting `sensitive_to_impaling`;
- `axolotl_hunt_targets`, allowing an Axolotl without hunting cooldown to
  select a visible, attackable, in-water target within squared distance at
  most `64`;
- `can_breathe_under_water`;
- `cannot_be_pushed_onto_boats`, preventing the boat collision loop from
  auto-mounting it while still allowing the physical push branch; and
- `not_scary_for_pufferfish`, excluding it from the Pufferfish scary-Mob
  predicate.

No locked advancement names the exact type. Common Glow-Squid Spawn Egg is
raw item ID `1184`, stack size `64`, with
`entity_data.id=minecraft:glow_squid`; generic Egg construction, component
patch, naming, finalization and insertion retain their owner.

Ambient, death, hurt and squirt select sound protocol IDs
`731/732/733/734`. English subtitles are `Glow Squid swims`, `Glow Squid
dies`, `Glow Squid hurts` and `Glow Squid shoots ink`; generic voice pitch,
range and admission remain inherited.

Exact UTF scanning of all `1,212` structure templates finds zero
`minecraft:glow_squid` occurrence.

### Legacy schema and client projection

Exactly two schema classes contain the identity. `V2688` introduces
`minecraft:glow_squid` as a simple entity schema; `V705` maps
`minecraft:glow_squid_spawn_egg` to its entity for Spawn-Egg item shape.
No exact fix rewrites `DarkTicksRemaining`; live default and signed-int
behavior are authoritative.

`EntityRenderers` binds the type to `GlowSquidRenderer`, using adult and baby
`SquidModel`s, shadow radius `0.7`, inherited interpolated tentacle/body
rotation and age-dependent translations. Adult geometry uses a `64×32`
atlas, one `12×16×12` body with deformation `0.02`, and eight
`2×18×2` tentacles placed at radius `5`, Y `15`. Baby geometry uses a
`32×32` atlas, one `8×10×8` body and eight `2×6×2` tentacles at radius `3`,
Y `18.5`. Every tentacle X rotation is the interpolated tentacle angle.

Texture selection is age-dependent:

- adult `glow_squid.png` is `64×32`, `616` bytes, SHA-1
  `137d39e1696fb8600bba2b4acbfc7539fe9285e7`; and
- baby `glow_squid_baby.png` is `32×32`, `353` bytes, SHA-1
  `ebdd31494991140a136f35d81d8bc9e3ad94b4bb`.

For block-light projection the renderer computes
`i=(int)clampedLerp(0,15,1-darkTicks/10)`. It returns `15` immediately when
`i==15`; otherwise it returns the maximum of `i` and ordinary world block
light. Timer values at least `10` therefore add no glow beyond world light;
values `9..0` brighten over the final ten ticks, and negative values clamp to
full light. English names are `Glow Squid` and `Glow Squid Spawn Egg`; the
Egg uses generic spawn-egg item projection.

**Branches and aborts:**

- Darkness decrements only when positive; damage overwrites it only after the
  unusual Squid wrapper returns true.
- Ink sound/particles precede the `100` write.
- Both goals can run together; invalid flee destination preserves the old
  vector.
- Server Glow/Bubble `addParticle` calls are no-ops, but only the Glow call
  evaluates randomized arguments.
- Placement requires both Water-tag fluid and exact Water block, plus
  inclusive depth and zero-light checks.
- First finalized group member is adult; only later members draw the
  `<=0.05` baby chance.
- Offspring construction exists without baseline breeding admission.

**Constants and randomness:**

Entity/item IDs `61/1184`; dimensions adult/baby
`0.8×0.8/0.5×0.5`; eyes `0.4/0.37`; tracking/update `10/3`; health/follow
`10/16`; metadata `0..17 inherited, 18 INT`; dark `100` and render fade
`10`; tick Glow `3` doubles; ink `30×3` floats; tentacle speed
`0.2/(float+1)`; phase reroll `1/10`; goals `0/1`, random adjusted `50`,
idle cutoff `100`, flee square `100`, bubble residues `5 mod 10`; gravity/
drag `0.08/0.98`; air `300/-20/2`; spawn depth/light `33/0`; biome rows
`53/66` at `10/4..6`; category `5/32/128`; baby `<=0.05`, age `-24000`,
cluster `4`; loot `1..3 + round(LU)`, XP `1..3`; particles `107/106`;
sounds `731..734`; tags/templates/schemas `5/0 of 1212/2`; shadow
`0.7`; textures as above.

**Side effects:**

Age/darkness/air persistence and metadata; client/server RNG consumption;
movement, rotations, leash and age lock; sound and particle packets; health,
ink, loot and XP; spawn selection/finalization/despawn; tag-selected Axolotl,
Pufferfish, boat and Impaling behavior; renderer light/model/texture state.

**Gates:**

Logical side, dark signed value and last-attacker state; goal concurrency and
destination block/fluid; water/air/phase/authority; age/group index/RNG;
world border/depth/raw light/exact block; biome/category cap; death attacker/
Looting; tags, Egg, schema and client age/light assets.

**Boundary cases and quirks:**

A generic damage transaction can commit while its Squid override returns
false. A retained old attacker can admit ink for a later non-Mob source.
Server-local Glow positions consume RNG despite producing no particle.
Flee discards the normalization result. Requested natural packs up to six
compose with an inherited per-cluster maximum of four. Loaded negative
darkness renders fully bright forever unless overwritten by an admitted hit.

**Failure semantics:**

Rejected placement prevents natural construction/insertion. Generic insertion
failure does not undo spawn finalization. Failed or null-attacker damage
branches do not reset darkness or emit ink even when the inner damage
transaction already committed. Loot, XP, Egg and generic lifecycle owners
retain their commit/rollback rules.

**Client/server authority split:**

The server owns durable age/darkness, phase event, movement vector, AI,
placement, damage, ink packets, loot and XP. It consumes three position
doubles for a no-op Glow call each AI step. The client locally decrements its
synced timer, requests one Glow particle, interpolates Squid rotations and
selects light/model/texture; metadata packets correct authoritative state.

**Observability:**

Observe exact registration/dimensions, metadata slot/default/dirty values,
DarkTicks NBT, damage return versus health, ink order/vectors, RNG cursors,
both concurrent goals, phase/event/motion formulas, air and age lock, spawn
depth/light/water and 53-biome census, pack babies/cap, tags, loot/XP/sounds,
template/schema closure and adult/baby render/light assets.

**Persistence and reload:**

Generic entity plus inherited age fields and `DarkTicksRemaining` persist;
movement/rotation/phase vectors do not. Code fixes registration, metadata,
goals, placement and schemas. Biomes, tags, loot and Egg components reload
through their owners; language and both textures are client resources.

**Evidence:**

`net.minecraft.world.entity.EntityTypes`;
`net.minecraft.world.entity.ai.attributes.DefaultAttributes`;
`net.minecraft.world.entity.SpawnPlacements`;
`net.minecraft.world.entity.SpawnPlacementTypes`;
`net.minecraft.world.entity.MobCategory`;
`net.minecraft.world.entity.AgeableMob`;
`net.minecraft.world.entity.animal.AgeableWaterCreature`;
`net.minecraft.world.entity.animal.squid.Squid` and both inner goals;
`net.minecraft.world.entity.animal.squid.GlowSquid`;
`AxolotlAttackablesSensor`, `Pufferfish`, `AbstractBoat`, `LivingEntity`;
`net.minecraft.client.renderer.entity.EntityRenderers`;
`GlowSquidRenderer`, `SquidRenderer`, `SquidModel`, `BabySquidModel`,
`LayerDefinitions`; schemas `V2688` and `V705`; reports, five tags, loot, all
66 biomes, all 1,212 templates, Egg components, two textures, sounds and
language. Complete compiled/data identity searches find no other exact
runtime path.

**Test vectors:**

Run `EXP-ENT-010` across signed darkness and damage/last-attacker branches,
server/client particle RNG, ink vectors/order, both goals and all propulsion
phases, age/air/lock/group babies, exact placement/biomes/caps, tag consumers,
loot/XP/Looting, Spawn Egg, templates/schemas and adult/baby light/model/
texture/name projection.

**Limits:**

Generic entity lifecycle, age lock, navigation, damage/death, natural spawn,
despawn, loot evaluation, Spawn Egg, metadata packets and rendering retain
their owners. Tag consumers and Impaling retain their leaves. This leaf fixes
exact Glow-Squid dispatch and every direct join selecting it.
