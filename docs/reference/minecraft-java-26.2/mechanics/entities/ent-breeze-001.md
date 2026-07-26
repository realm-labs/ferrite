# Entities mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ENT-BREEZE-001` — Breezes cycle slide, shot and long-jump memories around explosive wind charges

**Parent:** `ENT-001`, `ENT-LIFECYCLE-001`, `ENT-002`,
`ENT-VEHICLE-001`, `ENT-004`, `ENT-PROJECTILE-001`, `ENT-005`,
`ENT-DAMAGE-001`, `ENT-BLOCK-001`, `ENT-DAMAGE-REDUCE-001`,
`ENT-KNOCKBACK-001`, `ENT-006`, `ENT-EFFECT-001`, `ENT-007`,
`ENT-DEATH-001`, `MOB-001`, `MOB-AI-001`, `MOB-002`,
`MOB-SPAWN-001`, `MOB-003`, `MOB-DESPAWN-001`, `MOB-005`,
`BLK-TRIAL-SPAWNER-001`, `ITM-BREEZE-ROD-001`, `ITM-ENCHANT-001`,
`RED-EXPLOSION-001`, `PLY-AUTOJUMP-001`, `WGEN-005`,
`WGEN-PORTAL-001`, `WGEN-JIGSAW-TRIAL-CHAMBERS-001`, `CLI-001`,
`CLI-006`, `CLI-EFFECT-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked registration, complete `Breeze`, Brain,
slide/shoot/long-jump and wind-charge paths, generic projectile deflection and
explosion code, all 66 biomes, direct tags, loot, both Trial-Spawner
configurations, Spawn Egg, four migration contexts, all 1,212 templates and
exact client resources close protocol entity IDs `17` and `18`.

**Applies when:**

`minecraft:breeze` is constructed, emitted by its Trial Spawner, spawned by an
egg, command or custom selector, loaded, moved, targeted, damaged, killed,
synchronized or rendered; or when its `minecraft:breeze_wind_charge` is
created, deflected, moved, collided, exploded, discarded or rendered.

**Authoritative state:**

Entity protocol ID `17` constructs `Breeze` in `MONSTER`. Registration makes
the type unavailable in Peaceful, with dimensions `0.6×1.77`, explicit eye
height `1.3452`, client tracking range `10` and default update interval `3`.
Default attributes are maximum health `30`, movement speed
`0.6299999952316284`, attack damage `3` and follow range `24`. Monster
construction sets XP reward `10`, and Breeze applies pathfinding malus `-1`
to `ON_TOP_OF_TRAPDOOR` and `FIRE`.

The subtype adds no synchronized metadata or saved scalar. Entity/Living/Mob
metadata remains slots `0..15`; pose in inherited slot `6` drives Standing,
Sliding, Inhaling, Shooting and Long-Jumping projection. The unsaved subtype
fields are jump-trail start tick `0`, sound tick `0`, and six client animation
states. Generic Brain state retains its owner; the subtype has no extra
save/load hook.

Breeze caps head yaw at `30` degrees and head rotation speed at `25`. Fluid
jump threshold is its eye height. Movement emission is `EVENTS`. It can attack
only an entity whose exact runtime type is Player or Iron Golem and which also
passes generic attack admission. Its current target is Brain
`ATTACK_TARGET`; `getHurtBy()` returns Brain `HURT_BY` only when that memory's
causing entity is living.

**Transition and ordering:**

### Brain graph and target retention

The Brain provider installs exactly four sensors:

- nearest living entities;
- hurt by;
- nearest players; and
- Breeze attack entity.

The last sensor filters `NEAREST_LIVING_ENTITIES` through
`NO_CREATIVE_OR_SPECTATOR` and generic sensor attackability, then writes the
first candidate to `NEAREST_ATTACKABLE`. Breeze's exact-type attack hook
therefore reduces candidates to Players and Iron Golems.

Core activity priority `0` runs Swim at speed `0.8` and Look-at-Target Sink
with yaw/pitch limits `45/90`. Idle contains priority `0` Start-Attacking from
`NEAREST_ATTACKABLE`, priority `1` Start-Attacking from the living
`HURT_BY` cause, priority `2` `SlideToTargetSink(20,40)`, and priority `3`
weighted `RunOne`: Do-Nothing for `20..100` ticks at weight `1`, or
Random-Stroll at speed `0.6` at weight `2`.

Fight requires `ATTACK_TARGET` present and `WALK_TARGET` absent. It runs, in
priority order:

1. stop attacking when the target was not attackable during the last `100`
   ticks;
2. Shoot;
3. Long Jump;
4. Shoot When Stuck; and
5. Slide.

Fight is the default activity and is tested before Idle on each server custom
AI step. The Brain ticks, activity selection updates to the first valid one,
then inherited Monster custom AI runs. The `100`-tick sensor predicate means
a temporarily unseen/unattackable target can remain retained until that
window expires.

### Slide and stuck recovery

Slide requires `ATTACK_TARGET`, with `WALK_TARGET`, `JUMP_COOLDOWN` and
`BREEZE_SHOOT` absent. It additionally requires on ground, not in water and
Standing pose. A target is inside the inner circle when it is closer to the
Breeze block center than horizontal `4` and vertical `10`.

For an inner-circle target, Slide first asks for a random position away with
horizontal/vertical radii `5/5`. It accepts the result only when non-null,
visible to Breeze, and farther from the target than Breeze's current squared
distance. If no such point is accepted, or for a target outside the circle,
one Boolean chooses between:

- behind target: target head yaw plus `180` degrees plus
  `nextGaussian()*45`, at distance `lerp(nextFloat(),4,8)`; or
- middle circle: from Breeze toward target, subtracting
  `lerp(nextDouble(),8,4)` from their distance.

It stores a `WALK_TARGET` at the destination-containing block, speed `0.6`,
close-enough distance `1`. That memory invalidates Fight and lets Idle's
priority-two movement sink own travel for `20..40` ticks. Sink start plays
Breeze Slide and sets Sliding; stop restores Standing and, if the attack
target remains, writes `BREEZE_SHOOT` with expiry `60`.

Shoot-When-Stuck requires an attack target while jump-inhaling, jump-target,
walk-target and shoot memories are absent. It admits only while a passenger,
in water or under Levitation. It is a one-shot behavior: start writes
`BREEZE_SHOOT` with expiry `60`, and it cannot continue.

### Shot state machine

Shoot requires `ATTACK_TARGET` and `BREEZE_SHOOT`, with shoot-cooldown,
shoot-charging, shoot-recovering, walk-target and jump-target absent. Its
maximum duration is `20` ticks. Start additionally requires Standing and
strict squared target distance below `256`; an out-of-range check erases
`BREEZE_SHOOT`.

An admitted start sets Shooting, writes `BREEZE_SHOOT_CHARGING` for `15`
ticks and plays Breeze Inhale at volume/pitch `1/1`. Each active tick looks
from eyes at target position. While charging or recovering exists it does
not fire. On the first tick with neither:

1. write `BREEZE_SHOOT_RECOVERING` with expiry `4`;
2. construct one Breeze Wind Charge at
   `(selfX, getFiringYPosition(), selfZ)`, where firing Y is
   `Y+height/2+0.30000001192092896`;
3. aim X/Z at target base coordinates and Y at target relative height `0.8`
   when it is a passenger, otherwise `0.3`;
4. call the projectile shoot/spawn helper with speed `0.7`, empty source
   stack, and literal uncertainty `5-4*difficultyId`; and
5. play Breeze Shoot at volume `1.5`, pitch `1`.

The uncertainty argument is `5/1/-3/-7` on Peaceful/Easy/Normal/Hard. Negative
values are passed literally; they mirror the symmetric spread while
preserving the same RNG consumption. The insertion result is ignored, so
recovery and the later sound remain committed after failed insertion.

Shoot continues only while target and shoot memories remain. Stop restores
Standing only if still Shooting, writes `BREEZE_SHOOT_COOLDOWN` for `10`
ticks, and erases `BREEZE_SHOOT`.

### Long jump

Long Jump has maximum duration `200`. It requires attack target, no
jump-cooldown, registered jump-inhaling/jump-target memories, no
`BREEZE_SHOOT`, no walk-target and registered leaving-water memory.
Admission first requires on ground or in water and rejects the generic Swim
predicate. An already stored jump target then passes without recomputing
target, danger or sight checks.

Otherwise it:

1. erases an attack target not strictly closer than follow range `24`;
2. rejects target distance at or below `4`;
3. rejects an exact Honey Block under Breeze;
4. requires each of four blocks above Breeze to be air or contain Water;
5. samples the behind-target point by the Slide formula, then collider-ray
   casts downward `10` and, on miss, upward `10`, using the block above the
   first hit;
6. rejects a candidate whose supporting block is dangerous for Breeze; and
7. requires a collider/no-fluid line of sight either to candidate center or
   four blocks above it.

Breeze visibility also rejects points farther than
`max(50,followRange)`, which is `50` at baseline. An admitted candidate is
stored as `BREEZE_JUMP_TARGET`.

Start writes `BREEZE_JUMP_INHALING` for `10` ticks if absent, sets Inhaling,
plays Breeze Charge at volume/pitch `1/1`, and looks at attack-target block
center when present. After inhaling expires, it aims at the jump target's
bottom center and shuffles angles `40,55,60,75,80`. It accepts the first
Long-Jump trajectory whose maximum launch velocity is
`0.058333334*followRange`, hence about `1.4` at baseline. Jump Boost adds
`normalizedTrajectoryY*getJumpBoostPower()` to Y.

No valid trajectory restores Standing; behavior stop later erases its jump
memories. A valid one writes leaving-water when currently wet, plays Breeze
Jump, sets Long-Jumping, copies yaw to body yaw, enables discard-friction and
sets velocity. Leaving water erases the marker. Landing is on ground, or in
water after that marker is absent; it plays Breeze Land, restores Standing,
disables discard-friction, writes `JUMP_COOLDOWN` for `2` ticks when
`HURT_BY` exists and `10` otherwise, and writes `BREEZE_SHOOT` for `100`.

Continuation requires non-Standing pose and no jump cooldown. Stop restores
Standing from Inhaling/Long-Jumping and erases jump-target, jump-inhaling and
leaving-water, but does not itself disable discard-friction. A forced
`200`-tick timeout while airborne can therefore retain that flag until
another path clears it.

### Pose ticks, particles and ambient sound

Before inherited Monster tick, Breeze branches on the pose captured at tick
entry:

- Sliding emits exactly `20` ground particles;
- Shooting, Inhaling and Standing reset jump-trail count and emit
  `1+nextInt(1)`, always one, ground particle;
- Long-Jumping starts its animation if stopped and emits the jump trail; and
- other poses perform neither operation.

Ground particles are suppressed for passengers. They use the current
in-block state unless air, then the state below; invisible render shape
aborts. Every particle is a zero-velocity Block particle at the same
`(centerX,baseY,centerZ)`.

Jump trail pre-increments its counter and emits only for values `1..5`. Each
admitted tick emits exactly three zero-velocity Block particles at
`position+deltaMovement+(0,0.1,0)`, again using the current block or the block
below. Relevant non-jump poses reset the counter.

Idle animation starts if stopped on every subtype tick. Leaving Sliding while
its animation is active starts Slide-Back at the current tick and stops
Slide. Pose synchronization otherwise stops shoot, idle, inhale and long-jump
states before starting Shooting, Inhaling or Sliding as selected.

When sound tick is zero, Breeze samples inclusive integer `1..80`; otherwise
it decrements. A resulting zero plays the local whirl. This creates effective
gaps of `2..81` subtype ticks between plays. Whirl samples pitch
`0.7+0.4*nextFloat` then volume `0.8+0.2*nextFloat`.

Ambient sound is Idle Ground when on ground and Idle Air otherwise. Its
override suppresses ambient playback while both targeted and grounded;
otherwise it requests a local entity sound at volume/pitch `1/1`.

### Damage and projectile deflection

Any damage source whose causing entity is a Breeze is invulnerable before
generic admission. Direct `fall_damage_immune` membership suppresses fall
damage, but `causeFallDamage` first plays Breeze Land for distances strictly
above `3`, then delegates; the sound can therefore occur without health loss.

Direct `deflects_projectiles` membership returns a custom deflection for
ordinary projectiles, except exact Breeze-Wind-Charge and Wind-Charge types,
which return `NONE`. The admitted custom path first plays Breeze Deflect at
volume/pitch `1/1`, then applies generic `REVERSE`: velocity is multiplied by
`-0.5`, yaw and previous yaw each add `170+20*nextFloat`, and synchronization
is requested. Generic server deflection then changes projectile owner to the
Breeze and runs the projectile's deflection hook.

The other direct tags are `can_turn_in_boats`, enabling generic controlled-
boat turn copying, and `no_anger_from_wind_charge`, suppressing generic anger
attribution from Wind Charge damage.

### Breeze Wind Charge transaction

Protocol entity ID `18` is `BREEZE_WIND_CHARGE` in `MISC`, dimensions
`0.3125×0.3125`, eye height `0`, tracking range `4`, update interval `10`,
with normal save/summon and no loot table. It inherits owner persistence,
collision sweep and movement from `ENT-PROJECTILE-001`.

The projectile has acceleration power `0`, air/liquid inertia `1`, no trail
particle, no fire and an empty supplied item. It neither collides with nor
hits any Wind Charge, and it cannot hit End Crystal. Its bounding box is
vertically shifted down by `0.15`. `push` is a no-op. Above world maximum Y
plus `30`, a server tick explodes it at its current position and discards it
instead of running the ordinary projectile tick.

On entity impact, inherited projectile handling runs first. Server processing
then:

1. resolves a living owner or null;
2. sets a living owner's last-hurt mob to the struck entity before damage;
3. submits raw damage `1` from Wind-Charge source `(projectile,livingOwner)`;
4. on admitted damage to a living victim, runs post-attack enchantment
   effects; and
5. explodes at projectile position regardless of damage result.

On block impact, inherited handling runs first. The server offsets hit
location by `0.25` along the struck face, explodes there, and discards. The
outer Wind-Charge hit hook also discards after either impact.

The Breeze charge explosion has strength `3`, no fire, interaction `TRIGGER`,
small/large Gust-Emitter particles, an empty weighted block-particle list and
Breeze Wind-Charge Burst sound. Its specialized calculator disables explosion
entity damage but retains default knockback, admits block explosion callbacks,
and assigns resistance `3,600,000` only to
`#blocks_wind_charge_explosions` (locked Barrier and Bedrock). The generic
`TRIGGER` block-interaction path also requires `mobGriefing`. Explosion
ordering, exposure, knockback and block callbacks remain with
`RED-EXPLOSION-001`.

**Placement and Trial Spawners:**

Breeze registers `ON_GROUND`, `MOTION_BLOCKING_NO_LEAVES` and the standard
any-light Monster predicate. The outer placement gate requires world border,
valid support and empty candidate/above blocks. Outside the spawner support
bypass, species admission is generic Mob placement; there is no light draw.
The non-Peaceful type gate still applies.

None of the 66 baseline biome spawn lists contains Breeze. Exactly two locked
Trial-Spawner configurations select it:

- normal: total `2+1p`, simultaneous `1+0.5p`, interval `20`; and
- ominous: total `4+1p`, simultaneous default `2+0.5p`, interval `20`, plus
  ejection weights key `3` and consumables `7`.

Here `p` is additional players. Each potential list contains only an id-only
Breeze at weight `1`; ominous adds no external equipment. Both inherit spawn
range `4` and cooldown `36,000`. Activation, omen conversion, finalization,
collision, persistence and insertion belong to `BLK-TRIAL-SPAWNER-001`.

Template `trial_chambers/spawner/breeze/breeze.nbt` references the normal
configuration key once. Exact UTF scans of all 1,212 templates find zero
`minecraft:breeze` or plain `breeze` entity payloads. Filenames
`trial_chambers/chamber/eruption/breeze_slice_1.nbt` and
`trial_chambers/corridor/atrium/breeze_relief.nbt` are decorative and contain
no Breeze identity.

**Death, progression and migration:**

The entity loot table has type `entity`, random sequence
`minecraft:entities/breeze`, and one roll gated by `killed_by_player`. Its
sole Breeze-Rod entry sets uniform integer count `1..2`; a living attacker
with Looting level `L>0` draws a fresh uniform float `V` in `[1,2)` and adds
`round(L*V)`. The complete material joins remain with
`ITM-BREEZE-ROD-001`.

Generic eligible-kill XP starts at `10` and can add ordinary qualifying-
equipment increments. Exact Breeze conditions occur in
`adventure/kill_a_mob` and `adventure/kill_all_mobs`. Challenge
`adventure/blowback` requires the killed entity be exact Breeze and the
killing blow's direct entity be exact Breeze Wind Charge with
`is_projectile`; it awards `40` XP. It does not require the charge still be
owned by a Breeze.

Common Breeze Spawn Egg is raw item ID `1218`, maximum stack `64`, with
`entity_data.id=minecraft:breeze` and generic Spawn-Egg use, dispenser and
projection.

Exactly four migration contexts own the coupled identities:

- `DataFixers` installs the version-3689 entity `AddNewChoices` fix named
  `Added Breeze`;
- schema `V3689` registers simple entities Breeze, Wind Charge and Breeze
  Wind Charge;
- schema `V705` maps `minecraft:breeze_spawn_egg` to Breeze; and
- `TrialSpawnerConfigInRegistryFix.VanillaTrialChambers` recognizes the old
  inline normal and ominous Breeze configurations and maps them to registry
  keys.

No fix rewrites subtype state because Breeze adds no persisted scalar.

**Sounds and client projection:**

Breeze charge/deflect/inhale/idle-ground/idle-air/shoot/jump/land/slide/death/
hurt/whirl/wind-burst sound protocol IDs are `198..210`. English subtitles
are `Breeze charges/deflects/inhales/whirs/flies/shoots/jumps/lands/slides/
dies/hurts/whirls` and `Wind Charge bursts`. Parrot's exact map selects
imitation sound ID `1216`, subtitle `Parrot whirs`; generic selection,
silence and pitch behavior remain with the Parrot owner. Block particle ID is
`1`; small/large Gust-Emitter IDs are `34/33`.

`EntityRenderers` binds Breeze to `BreezeRenderer`, shadow radius `0.5`, and
both Wind-Charge entity types to the same `WindChargeRenderer`. Breeze base,
eyes and wind use separate model layers:

- base retains head and three rods on a `32×32` sheet;
- eyes retain only the eye child on `32×32`; and
- wind retains nested bottom/middle/top wind parts on `128×128`.

Render extraction copies idle, shoot, slide, slide-back, inhale and long-jump
animation states. The six definitions have durations
`2(looping)/1.125/0.2/0.1/2/0.5` for
idle/shoot/slide/slide-back/inhale/jump, and all active animations apply in
that model order. Base texture `textures/entity/breeze/breeze.png` is
`32×32`, `377` bytes, SHA-1
`31af701087929b04d5e9515f7d64d5800928afe7`. Emissive eyes use
`breeze_eyes.png`, `32×32`, `139` bytes, SHA-1
`d7cd925a9f3c1d5077cd6a973a83cdec1ddc5e43`. Wind uses
`breeze_wind.png`, `128×128`, `465` bytes, SHA-1
`6aa6a18457ad1f897d57ea1cd44cec795a021e12`, with horizontal texture offset
`(0.02*ageInTicks)%1`.

The shared Wind-Charge model has a `4³` core and opposing `6×4×6` and `8×2×8`
wind shells on a `64×32` sheet. Core and wind rotate around Y at
`-16/+16` degrees per age tick. Its renderer scrolls horizontal texture by
`(0.03*ageInTicks)%1`; texture
`textures/entity/projectiles/wind_charge.png` is `64×32`, `205` bytes,
SHA-1 `b8b8ee5dc7eba8a05405e4810deb03196c81b029`. English names are
`Breeze`, `Breeze Spawn Egg` and `Wind Charge`.

**Branches and aborts:**

- Fight is valid only without walk target; Slide deliberately transfers
  movement to Idle before arming a shot.
- Shot range is strict squared distance `<256`; Long Jump rejects target
  distance `<=4` and requires strict follow-range admission.
- Existing jump target bypasses later target, danger and visibility
  recomputation.
- Charge/recovery memories prevent duplicate shots; insertion failure does
  not undo recovery or sound.
- Breeze reverses ordinary projectiles but explicitly returns no deflection
  for either Wind-Charge entity type.
- Charge impact always explodes after the direct-damage attempt, including a
  rejected attempt.
- No biome row selects Breeze; only the two Trial-Spawner records do.

**Constants and randomness:**

Entity/projectile/Egg/Rod IDs `17/18/1218/1252`; dimensions/eye
`0.6×1.77/1.3452`; tracking/update `10/3`; health/speed/attack/follow
`30/0.63/3/24`; XP `10`; sensors `4`; target retention `100`; Slide inner
range `4×10`, away radius `5×5`, behind distance `4..8`, speed/close
`0.6/1`, sink `20..40`; Shoot memory/charge/recovery/cooldown
`60/15/4/10`, range square `256`, speed/uncertainty `0.7/5-4d`; Long Jump
duration/inhale/angles/max factor/cooldowns/shoot
`200/10/[40,55,60,75,80]/.058333334F/2-or-10/100`; particles
`20/1/3×5`; whirl `1..80`, pitch `.7+.4U`, volume `.8+.2U`; reverse
`-.5`, yaw `170+20U`; direct charge damage/explosion `1/3`; trial
`2+1p,1+.5p,20` and `4+1p,2+.5p,20`; loot
`1..2+round(LV)`; sounds `198..210/1216`; migrations/templates
`4/0 exact identities of 1212`.

**Side effects:**

Brain sensors, memories, poses, navigation/look and velocity; animation
states, particles and local sounds; projectile construction, ownership,
deflection, damage, enchantment callbacks, explosion and discard; tag-selected
fall/vehicle/anger behavior; trial selection/finalization; loot, XP,
criteria, Parrot imitation and three-layer client projection.

**Gates:**

Logical side, Brain activity/memory presence and expiry, target exact type,
attackability, pose, ground/water/passenger/effect state, distance, sight,
block danger and trajectory; silence and RNG; damage cause/type; projectile
type/collision/insertion; explosion gamerule/block tag; Peaceful/world border/
support/spawner state; attacker/Looting/player kill; migration shape and
client resources.

**Boundary cases and quirks:**

Normal and Hard pass negative shot uncertainty literally. A failed charge
insertion still produces the shooting recovery and sound. Failed direct
Wind-Charge damage still explodes. Breeze-owned damage is rejected by Breeze,
and Breeze refuses to deflect both Wind-Charge types despite its direct
deflection tag. Fall distance above three can play landing sound while fall
damage is immune. A forced airborne Long-Jump timeout does not clear
discard-friction in its stop hook. Decorative Breeze-named templates contain
no entity payload.

**Failure semantics:**

Rejected memory/pose/range admission performs only the specified erase where
present. Failed path or trajectory selection leaves no launch; stop clears
jump memories. Rejected projectile insertion retains preceding memory and
later sound state. Rejected direct damage retains the following explosion.
Rejected placement prevents natural construction/insertion; Trial-Spawner
failure remains with its owner transaction.

**Client/server authority split:**

The server owns Brain state, targeting, paths, poses, movement, projectile
ownership/insertion, damage, explosion, placement, Trial Spawners, loot, XP
and criteria. The client receives pose/entity state, advances the six
animations, emits local Block particles and whirl/ambient requests, and
projects the three Breeze layers plus shared Wind-Charge model. Ordinary
metadata, velocity, sound and particle delivery retain their generic owners.

**Observability:**

Observe registration and attributes; every Brain memory transition and
expiry; activity priority, exact target type and retained attackability;
Slide candidate RNG and walk transfer; Shot timing, aim, signed uncertainty
and failed insertion; Long-Jump geometry, shuffled angles, water markers and
timeout; pose particles/sounds; deflection ownership/velocity/RNG; charge
collision/damage/explosion/discard; Trial Spawners, loot/XP/criteria; exact
migrations, layers, animations, textures and names.

**Persistence and reload:**

Generic entity/Mob/Brain state persists under its owner; subtype trail/sound
ticks and animation states reset on construction. Code fixes attributes,
Brain graph, behaviors, projectile and migration schemas. Tags, loot,
advancements and Trial-Spawner records reload through their owners; language
and textures are client resources.

**Evidence:**

`net.minecraft.world.entity.EntityTypes`;
`net.minecraft.world.entity.ai.attributes.DefaultAttributes`;
`net.minecraft.world.entity.SpawnPlacements`;
`net.minecraft.world.entity.monster.breeze.Breeze`, `BreezeAi`,
`BreezeUtil`, `Slide`, `Shoot`, `ShootWhenStuck` and `LongJump`;
`net.minecraft.world.entity.ai.sensing.BreezeAttackEntitySensor`;
`net.minecraft.world.entity.projectile.Projectile` and
`ProjectileDeflection`;
`net.minecraft.world.entity.projectile.hurtingprojectile.windcharge.AbstractWindCharge`
and `BreezeWindCharge`;
`net.minecraft.world.level.block.entity.trialspawner.TrialSpawnerConfigs`;
`net.minecraft.client.renderer.entity.EntityRenderers`, `BreezeRenderer` and
`WindChargeRenderer`;
`net.minecraft.client.renderer.entity.layers.BreezeEyesLayer` and
`BreezeWindLayer`;
`net.minecraft.client.model.monster.breeze.BreezeModel`;
`net.minecraft.client.model.object.projectile.WindChargeModel`;
`net.minecraft.client.animation.definitions.BreezeAnimation`;
`net.minecraft.util.datafix.DataFixers`;
`net.minecraft.util.datafix.schemas.V3689` and `V705`;
`net.minecraft.util.datafix.fixes.TrialSpawnerConfigInRegistryFix`; reports,
tags, loot, advancements, Trial-Spawner records, all 66 biomes, all 1,212
templates, Egg components, textures, sounds and language. Complete
compiled/data identity searches find no other direct runtime path.

**Test vectors:**

Run `EXP-ENT-013` across every sensor/activity/memory/pose transition, exact
target/retention branch, Slide destination and Idle transfer, stuck-shot and
Shot timing/aim/signed uncertainty/insertion result, Long-Jump geometry/
trajectory/water/timeout path, particle and sound RNG, damage/fall/deflection
tags, all Breeze-Wind-Charge collision/damage/explosion/discard outcomes,
Trial Spawners, loot/XP/advancements, Spawn Egg, templates/migrations and
exact layered animation/texture/name projection.

**Limits:**

Generic entity lifecycle, Brain scheduling, navigation, damage/death,
projectile sweep, explosion, Trial Spawner, loot evaluation, Spawn Egg,
metadata packets and rendering retain their owners. Breeze-Rod progression
and Trial-Chamber generation retain their leaves. This leaf fixes exact
Breeze/Wind-Charge dispatch and every direct join selecting them.
