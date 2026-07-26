# Entities mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ENT-SQUID-001` — Squids pulse through water and emit thirty ink packets only after admitted Mob-attributed damage

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

`SourceSpecified` — locked registration, the complete `Squid`,
`AgeableWaterCreature` and ageable-Mob paths, both control-free goals,
placement and category code, all 66 biomes, five direct tags, loot, Spawn
Egg, nine migration/schema contexts, all 1,212 templates and exact
adult/baby client resources close protocol entity ID `127`.

**Applies when:**

`minecraft:squid` is constructed, finalized, naturally selected, spawned by
an Egg, spawner, command or custom selector, loaded, age-locked, leashed,
moved, attacked, targeted by an Axolotl, killed, synchronized or rendered.

**Authoritative state:**

Protocol entity ID `127` constructs `Squid` in `WATER_CREATURE`.
Registration fixes adult width/height `0.8×0.8`, eye height `0.4`, client
tracking range `8` and the builder-default update interval `3`. Squid is
Peaceful-compatible. Its attributes are maximum health `10` and inherited
follow range `16`; movement emission is `EVENTS`, sound volume is `0.4`,
gravity is `0.08`, Water path malus is `0`, it can be leashed and it is not
pushed by fluid.

Ageable state changes the live dimensions. An adult uses the registered
`0.8×0.8` box and `0.4` eye height; a baby instead uses a dedicated
`0.5×0.5` box with eye height `0.37`. This is not a uniform half-scale of
the adult collision box.

Entity, Living-Entity and Mob state occupies synchronized metadata slots
`0..15`. `AgeableMob` adds slot `16` for baby state and slot `17` for
age-lock state, both serializer ID `8` (`BOOLEAN`) with default `false`.
Squid adds no synchronized slot. Signed `Age`, signed `ForcedAge` and
Boolean `AgeLocked` persist through the inherited fields; missing or
wrong-type values use `0/0/false`. Baby status is the synchronized projection
of negative age, and a baby-slot update refreshes dimensions.

The forced-growth timer and age-lock particle timer are transient. Squid's
body X/Z rotations, their previous values, tentacle phase/angle and previous
values, rotation speed and movement vector are likewise neither synchronized
nor persisted. Construction initializes the movement vector to zero and
consumes one float to set
`tentacleSpeed=0.2/(nextFloat+1)`, in `(0.1,0.2]`.

**Transition and ordering:**

### Damage admission and ink

`Squid.hurtServer` first invokes the complete generic damage transaction.
Only when that call returns true and the post-transaction
`getLastHurtByMob()` is non-null does Squid spawn ink and return true.
Otherwise the override returns false. A fresh environmental hit can
therefore change health while the Squid wrapper returns false and emits
nothing. Conversely, a retained prior Mob attacker admits ink for a later
accepted non-Mob damage source because the retained post-hit field, not the
current source type, is tested.

On the admitted branch Squid first emits `entity.squid.squirt`, then sends
exactly `30` Squid-Ink particle requests. The common origin is vector
`(0,-1,0)` rotated by previous body X and negative previous body yaw, added
to entity position. For every particle, in order:

1. two floats form direction `(nextFloat*0.6-0.3, -1,
   nextFloat*0.6-0.3)`;
2. that direction is rotated by the same previous body rotations;
3. a third float gives scale `0.1+2*nextFloat` for a baby or
   `0.3+2*nextFloat` for an adult; and
4. `ServerLevel.sendParticles` is called with count `0`, origin Y raised
   by `0.5`, the scaled vector as offsets and speed
   `0.10000000149011612`.

For each admitted packet recipient, count zero requests one particle whose
client velocity is the packet speed times each offset. The nominal velocity
is therefore `0.1` times the scaled rotated direction. The branch consumes
exactly `90` entity-RNG floats, with the sound before all sends. Squid Ink
has particle protocol ID `73`; its resource cycles
`generic_7, generic_6, ..., generic_0`.

### Concurrent goals and movement-vector selection

Squid registers exactly two goals and no target goal:

- priority `0`, `SquidRandomMovementGoal`, whose `canUse` always returns
  true; and
- priority `1`, `SquidFleeGoal`, admitted while currently in water with a
  non-null retained last Mob attacker at squared distance strictly below
  `100`.

Neither goal declares a control flag, so both may run on the same tick. Goal
order allows the later flee tick to replace the random goal's movement
vector.

The random goal reads `noActionTime` first. Above `100` it sets the movement
vector to zero and consumes no cadence or direction draw. Otherwise it
draws `nextInt(reducedTickDelay(50))`. A zero result, a false inherited
`wasTouchingWater` value, or an existing vector with squared length at most
`9.999999747378752E-6` causes a replacement:

`a=nextFloat*2*pi`,
`(x,y,z)=(cos(a)*0.2, -0.1+nextFloat*0.2, sin(a)*0.2)`.

Short-circuit order means the cadence draw occurs before the remembered
water and vector tests. `wasTouchingWater`, rather than the goal's current
`isInWater` query, is the random-goal water input.

Flee start resets `fleeTicks` to zero, and the goal requests an update every
tick. Each tick increments the counter and returns if the retained attacker
has vanished. Otherwise it forms the unnormalized vector
`d=squidPosition-attackerPosition` and inspects the block and fluid at
`squidPosition+d`. Water-tag fluid or an air block admits a replacement;
any other destination preserves the old movement vector.

Although the code calls `d.normalize()`, it discards the immutable result.
Let `D=length(d)`. Factor `F` starts at `3`; for `D>5` it becomes
`3-(D-5)/5`. When `F>0`, the original unnormalized `d` is scaled by `F`.
An air destination then clears its Y component. The new movement vector is
that result divided componentwise by `20`. At flee ticks
`5,15,25,...`, the server goal invokes `Level.addParticle(BUBBLE, position,
0,0,0)`. That base server implementation is a no-op and sends no particle
packet.

### Pulse phase, propulsion and rotations

Every Squid AI step first completes `AgeableWaterCreature.aiStep`, including
Mob AI and inherited age work. It then copies the four prior body/tentacle
values and adds `tentacleSpeed` to its phase.

When phase exceeds double `2*pi`, a client sets it to float
`6.2831855`. A server subtracts that float, draws `nextInt(10)`, rerandomizes
tentacle speed with the constructor formula only on zero, then broadcasts
entity event `19`. Handling event `19` sets phase to zero; other events
delegate upward.

In water, phase below float `pi` sets `f=phase/pi` and:

`tentacleAngle=sin(f*f*pi)*pi*0.25`.

Above strict `f>0.75`, a locally authoritative instance copies the selected
movement vector into delta movement and rotation speed becomes `1`.
Otherwise rotation speed is multiplied by `0.8`. In the other half-cycle,
tentacle angle becomes zero, locally authoritative delta movement is
multiplied by `0.9`, and rotation speed is multiplied by `0.99`.

Using the resulting delta vector, body yaw converges by factor `0.1` toward
`-atan2(dx,dz)` in degrees and is copied to entity yaw. Z body rotation adds
`pi*rotationSpeed*1.5`. X body rotation converges by factor `0.1` toward
`-atan2(horizontalSpeed,dy)` in degrees.

Out of water, tentacle angle is
`abs(sin(phase))*pi*0.25`. On the server, Y velocity becomes
`0.05*(levitationAmplifier+1)` under Levitation; otherwise gravity `0.08` is
subtracted. The result is multiplied by air drag `0.98`, while X and Z
velocity are set to zero. X body rotation converges by factor `0.02` toward
`-90` degrees. `travel` ignores its input vector and moves `SELF` by current
delta movement.

### Age, air, interaction and group finalization

Squid is an `AgeableMob`, but base `canBreed()` is false, it is not an
`Animal`, has no food predicate and registers no breeding goal. Its
offspring factory would construct `SQUID` with reason `BREEDING` if an
external caller invoked it; the factory itself creates no baseline breeding
admission.

On the server, a live unlocked baby increments its negative age toward zero.
An injected positive age decrements toward zero even though it is not baby
growth. A locked baby remains at its negative age. The client projects baby
state and locally runs only the inherited visual timers.

A baby Squid may use raw item ID `257`, Golden Dandelion, only while the
age-lock particle timer is zero and the type is absent from reloadable
`cannot_be_age_locked`. The current tag contains Zombie Horse, Skeleton
Horse and Villager, not Squid. Successful interaction:

1. toggles slot `17`;
2. resets age exactly to baby start age `-24000`;
3. starts the transient timer at `40`;
4. consumes one item;
5. sets custom persistence when the new state is locked, without clearing
   it when later unlocked; and
6. plays Golden-Dandelion Use when locked or Unuse when unlocked, source
   `PLAYERS`, volume/pitch `1/1`.

The timer prevents another use until zero. A client initialized with the
timer emits one particle at each positive even value, `20` total: locked
uses `PAUSE_MOB_GROWTH` with an extra `0.2` Y offset, while unlocked uses
`RESET_MOB_GROWTH`. Squid has no bucket-capture interaction; all other
items delegate to generic Mob interaction.

Spawn finalization replaces null group data with
`AgeableMobGroupData(0.05)` and then delegates to the generic ageable path.
The first finalized member has group size zero and cannot be made baby.
Each later member draws one level float and becomes age `-24000` when that
draw is at most `0.05`; group size then increments. A supplied compatible
group object retains its own chance, while an incompatible non-null object
follows the inherited cast/failure semantics. The inherited maximum spawn
cluster is `4`.

While alive and outside water, `AgeableWaterCreature` decrements the
pre-super-tick air value. At `-20` or below it resets air to zero and applies
`2` Drown damage. In water or while dead it resets air to `300`. Direct
membership in `can_breathe_under_water` also makes the generic
Living-Entity underwater branch skip drowning.

### Registered placement and natural selection

Squid registers placement `IN_WATER` with heightmap
`MOTION_BLOCKING_NO_LEAVES`. The placement-type gate requires a non-null
type, a candidate inside the world border, Water-tag fluid at the candidate
and an above block that is not a redstone conductor. The species predicate
then consumes no RNG and requires:

- candidate Y inclusively in `[seaLevel-13, seaLevel]`;
- Water-tag fluid one block below; and
- the block one position above to be exactly `Blocks.WATER`.

The exact-Water-above test is stronger than the placement gate's
nonconductor test and is performed after it. The candidate block itself need
not be exactly Water: a waterlogged candidate can pass when its fluid is
Water and the below/above conditions hold. Spawn obstruction later requires
the constructed entity to be unobstructed.

Exactly `11` of the `66` locked biomes select Squid in `water_creature`:

| Biomes | Weight | Group |
|---|---:|---:|
| Deep Frozen Ocean, Frozen Ocean, Deep Ocean, Ocean | `1` | `1..4` |
| Frozen River, River | `2` | `1..4` |
| Cold Ocean, Deep Cold Ocean | `3` | `1..4` |
| Deep Lukewarm Ocean | `8` | `1..4` |
| Lukewarm Ocean | `10` | `1..2` |
| Warm Ocean | `10` | `4..4` |

`WATER_CREATURE` has global cap `5`, is friendly, is not category-persistent,
and uses no-despawn/despawn distances `32/128`. The species cluster maximum
`4` already bounds every requested biome group. Generic candidate walking,
cap accounting, group insertion and distance removal retain their owners.

No bundled structure or other direct baseline producer creates a Squid.
Egg, spawner, command and custom construction remain available through
their generic paths.

### Loot, tags, sounds and item projection

The entity loot table has type `entity`, random sequence
`minecraft:entities/squid` and one roll. It emits Ink Sac raw item ID `1092`
with base uniform integer count `1..3`. With a living attacking entity and
positive Looting level `L`, one fresh float `U` adds `round(L*U)`;
otherwise the bonus spends no draw. Eligible generic death also gives XP
`1+nextInt(3)`.

Squid belongs directly to exactly five entity-type tags:

- `aquatic`, transitively selecting `sensitive_to_impaling`;
- `axolotl_hunt_targets`, allowing an Axolotl without hunting cooldown to
  select a visible, attackable, in-water Squid within squared distance at
  most `64`;
- `can_breathe_under_water`;
- `cannot_be_pushed_onto_boats`, preventing the boat collision loop from
  auto-mounting it while retaining the physical-push branch; and
- `not_scary_for_pufferfish`, excluding it from the Pufferfish scary-Mob
  predicate.

No locked advancement names the exact entity type. Squid Spawn Egg is raw
item ID `1188`, stack size `64`, with
`entity_data.id=minecraft:squid`; generic Egg construction, component patch,
naming, finalization and insertion retain their owners.

Ambient, death, hurt and squirt use sound protocol IDs
`1592/1593/1594/1595`. Their sound definitions select respectively
`5/3/4/3` clips with no per-entry volume or pitch override. English
subtitles are `Squid swims`, `Squid dies`, `Squid hurts` and
`Squid shoots ink`. Ambient checks use the inherited water-creature interval
`120`; generic sound admission, range and voice pitch remain inherited.

Exact UTF scanning of all `1,212` structure templates finds zero
`minecraft:squid` occurrence.

### Legacy migration and client projection

Exactly nine migration/schema contexts own Squid compatibility:

- `EntityHealthFix` recognizes legacy `Squid` health;
- `EntityIdFix` maps `Squid` to `minecraft:squid`;
- `EntityUUIDFix` includes the modern identity in Mob UUID migration;
- `ItemSpawnEggFix` maps legacy generic Spawn Egg damage `94` to `Squid`;
- `ItemStackSpawnEggFix` maps the modern entity to
  `minecraft:squid_spawn_egg`;
- `StatsCounterFix` maps legacy Squid statistics to the current identity;
- `V99` registers the legacy simple `Squid` schema;
- `V705` registers the modern Mob shape and maps the current Spawn Egg to
  it; and
- `V1460` registers the modern Mob schema.

Legacy Egg damage `94` is unrelated to current protocol entity ID `127`.
No fix rewrites age, phase, movement-vector or ink state.

`EntityRenderers` binds Squid to `SquidRenderer`, with adult and baby
`SquidModel` layers, `SquidRenderState` and shadow radius `0.7`. Render-state
extraction linearly interpolates tentacle angle and body X/Z rotations.
Rotation setup translates Y by `0.5` adult or `0.25` baby, rotates around
positive Y by `180-yaw`, around positive X by interpolated body X, then
around positive Y by interpolated body Z, and finally translates Y by
`-1.2` adult or `-0.6` baby.

Adult geometry uses a `64×32` atlas: one `12×16×12` body with deformation
`0.02`, and eight `2×18×2` tentacles placed at radius `5`, Y `15`.
Baby geometry uses a `32×32` atlas: one `8×10×8` body and eight `2×6×2`
tentacles placed at radius `3`, Y `18.5`. Every tentacle X rotation is the
interpolated tentacle angle.

Texture selection follows synchronized baby state:

- adult `textures/entity/squid/squid.png` is `64×32`, `394` bytes,
  SHA-256
  `a95f135fa980a0d712d1c6b1f09327ed829ade41acf37a05695bfe4018d80fbc`;
  and
- baby `textures/entity/squid/squid_baby.png` is `32×32`, `281` bytes,
  SHA-256
  `fa894f4884ad7b2bf5e101628cab3821f49e9d4dd66360af53aef1f8399d4794`.

The renderer has no ink, age-lock or water-state texture branch and uses
ordinary world lighting. English names are `Squid` and `Squid Spawn Egg`.
The generated Egg model selects its same-named `16×16`, `239`-byte texture,
SHA-256
`572db27e4561625af62ec1882cbb501c3da3441782f036cf1b1ce455c691c1d3`.

**Branches and aborts:**

- Generic damage can commit while the Squid wrapper returns false.
- Ink requires the retained post-hit Mob attacker, not a Mob current source.
- Both goals can run together; invalid flee destinations preserve the old
  vector.
- Random movement uses remembered `wasTouchingWater`; flee uses current
  `isInWater`.
- Server Bubble calls are no-ops; admitted ink uses thirty count-zero
  particle sends.
- Placement requires candidate/below Water fluid and exact Water above.
- The first finalized group member is adult; only later members draw the
  inclusive baby chance.
- Only babies can start the Golden-Dandelion interaction.

**Constants and randomness:**

Entity/Egg/Ink-Sac IDs `127/1188/1092`; adult/baby dimensions
`0.8×0.8/0.5×0.5`; eyes `0.4/0.37`; tracking/update `8/3`;
health/follow `10/16`; metadata slots `16/17 BOOLEAN`; goals `0/1`;
constructor/phase speed `0.2/(float+1)`; phase reroll `1/10`; random cadence
adjusted `50`, idle cutoff `100`, vector square epsilon
`9.999999747378752E-6`; flee square `100`, bubble residues `5 mod 10`;
ink `30×3` floats, count/speed `0/0.10000000149011612`, particle ID `73`;
gravity/drag `0.08/0.98`; air `300/-20/2`; age `-24000`, baby chance
`<=0.05`, cluster `4`; spawn vertical `seaLevel-13..seaLevel`; biome rows
`11/66`; category `5/32/128`; loot `1..3+round(LU)`, XP `1..3`; direct
tags/templates/migrations `5/0 of 1212/9`; sounds `1592..1595`; shadow
`0.7`.

**Side effects:**

Age/air persistence and baby/lock metadata; transient phase, movement and
goal state; RNG cursor, movement and rotations; leash and age-lock
persistence; sound and particle packets; health, ink, loot and XP; spawn
selection/finalization/despawn; tag-selected Axolotl, Pufferfish, boat and
Impaling behavior; renderer model/texture state.

**Gates:**

Logical side, retained attacker and generic damage result; goal concurrency,
remembered/current water and destination block/fluid; phase half and local
authority; baby/lock/timer/group index/RNG; world border/Y/three Water
positions/obstruction; biome/category cap; death attacker/Looting; tags,
Egg, migrations and client age state.

**Boundary cases and quirks:**

A successful inner damage transaction can accompany an outer false result.
A stale attacker can admit ink for environmental damage. Flee discards the
normalization result. Its factor becomes nonpositive from distance `20`
onward, although `canUse` already requires distance below `10`. A
waterlogged candidate block can pass the Squid predicate. Golden Dandelion
resets an unlocked or locked baby to `-24000`; unlocking never clears
custom persistence.

**Failure semantics:**

Rejected placement prevents natural construction/insertion. Generic
insertion failure does not undo prior finalization. Failed or
null-attacker damage does not emit squirt or ink even when the inner damage
transaction already changed health. Loot, XP, Egg and generic lifecycle
owners retain their commit/rollback rules.

**Client/server authority split:**

The server owns durable age/lock/air, AI, phase event, movement vector,
placement, damage, ink packets, loot and XP. The client owns visual timer
particles, receives baby/lock metadata and event `19`, locally integrates
the phase/rotation projection and selects the adult or baby layer and
texture. Server metadata and motion packets remain authoritative.

**Observability:**

Observe registration/dimensions, metadata slots/defaults, age/lock/air NBT,
damage return versus health, ink sound/order/vectors and RNG cursor, both
simultaneous goals, remembered water, phase/event/motion formulas,
Golden-Dandelion state, group babies, exact placement and 11-biome census,
caps/despawn, tags, loot/XP/sounds, zero-template and nine-migration closure,
and both client models/textures/transforms.

**Persistence and reload:**

Generic entity state plus inherited `Age`, `ForcedAge`, `AgeLocked` and air
persist. Phase, body/tentacle values, movement vector, goal counters,
age-lock particle timer and retained render interpolation do not. Code fixes
registration, goals, placement and schemas. Biomes, tags, loot and Egg
components reload through their owners; language, particle atlas and
adult/baby textures are client resources.

**Evidence:**

`net.minecraft.world.entity.EntityTypes`;
`net.minecraft.world.entity.ai.attributes.DefaultAttributes`;
`net.minecraft.world.entity.SpawnPlacements`;
`net.minecraft.world.entity.SpawnPlacementTypes`;
`net.minecraft.world.entity.MobCategory`;
`net.minecraft.world.entity.AgeableMob`;
`net.minecraft.world.entity.animal.AgeableWaterCreature`;
`net.minecraft.world.entity.animal.squid.Squid` and both inner goals;
`AxolotlAttackablesSensor`, `Pufferfish`, `AbstractBoat`, `LivingEntity`;
`net.minecraft.client.renderer.entity.EntityRenderers`, `SquidRenderer`,
`SquidRenderState`, `SquidModel`, `BabySquidModel`, `LayerDefinitions`;
the nine migration/schema classes named above; reports, five tags, loot,
all 66 biomes, all 1,212 templates, Egg components/model/texture, two entity
textures, particle resource, locked sounds and language. Complete
compiled/data identity searches find no other exact runtime path.

**Test vectors:**

Run `EXP-ENT-020` across retained/null attacker and generic damage results,
adult/baby ink vectors and exact RNG, both concurrent goals and remembered
water, all propulsion phases, age/air/lock/group babies, exact
placement/biomes/caps, tag consumers, loot/XP/Looting, Spawn Egg,
templates/migrations/sounds and adult/baby client projection.

**Limits:**

Generic entity lifecycle, age locking, goal scheduling, damage/death,
natural spawning, despawn, loot evaluation, Spawn Egg, metadata packets,
particles and rendering retain their owners. Tag consumers and Impaling
retain their leaves. This leaf fixes exact Squid dispatch and every direct
join selecting it.
