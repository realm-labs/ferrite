# Entities mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ENT-BLAZE-001` — Blazes hover, charge and fire a retained-state three-projectile volley

**Parent:** `ENT-001`, `ENT-LIFECYCLE-001`, `ENT-002`,
`ENT-VEHICLE-001`, `ENT-004`, `ENT-PROJECTILE-001`, `ENT-005`,
`ENT-DAMAGE-001`, `ENT-BLOCK-001`, `ENT-DAMAGE-REDUCE-001`,
`ENT-KNOCKBACK-001`, `ENT-006`, `ENT-EFFECT-001`, `ENT-007`,
`ENT-DEATH-001`, `MOB-001`, `MOB-AI-001`, `MOB-002`,
`MOB-SPAWN-001`, `MOB-003`, `MOB-DESPAWN-001`, `MOB-005`,
`BLK-SPAWNER-001`, `ITM-BLAZE-MATERIAL-001`, `ITM-ENCHANT-001`,
`PLY-AUTOJUMP-001`, `WGEN-005`, `WGEN-PORTAL-001`,
`WGEN-STRUCTURE-FORTRESS-001`, `CLI-001`, `CLI-006`,
`CLI-EFFECT-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked registration, complete `Blaze`, attack-goal
and Monster superclass paths, placement/category code, fortress generation
and overrides, all 66 biomes, two direct tags, loot, Spawn Egg, nine migration
contexts, all 1,212 templates and exact client resources close protocol entity
ID `14`.

**Applies when:**

`minecraft:blaze` is constructed, selected by a Nether Fortress override,
spawned by the fortress throne spawner, another spawner, Spawn Egg, command or
custom selector, loaded, moved, targeted, damaged, killed, synchronized or
rendered.

**Authoritative state:**

Entity protocol ID `14` constructs `Blaze` in `MONSTER`. Registration makes
the type fire-immune and unavailable in Peaceful, with dimensions `0.6×1.8`,
default eye height `1.53`, client tracking range `8` and update interval `3`.
Default attributes are maximum health `20`, attack damage `6`, movement speed
`0.23000000417232513` and follow range `48`; the constructor fixes XP reward
`10`.

Blaze assigns pathfinding malus `-1` to Water, `8` to Lava and `0` to both
Fire and Fire-in-neighbor. Its light-dependent magic value is always `1`.
Consequently the Monster no-action update adds two on every server AI step in
addition to Mob's ordinary increment, normally advancing `noActionTime` by
three before generic activity resets.

Entity/Living/Mob occupy synchronized metadata slots `0..15`. Blaze adds a
byte at slot `16`, serializer ID `0`, default `0`. Bit zero is the charged
flag; its setter preserves bits `1..7`. The flag, `allowedHeightOffset`
(initially `0.5`), `nextHeightOffsetChangeTick` (initially `0`) and the attack
goal's step, delay and line-of-sight counters are transient and unsaved.

`isOnFire()` returns the charged flag rather than fire ticks. Thus charge is
observable through generic on-fire predicates and the client fire overlay,
despite the entity type's immunity to fire damage. Extinguishing actual fire
ticks does not clear charge; only the attack goal changes its bit.

**Transition and ordering:**

### Goal graph and retained attack state

The goal selector contains:

- priority `4`, `BlazeAttackGoal`, with Move and Look controls;
- priority `5`, `MoveTowardsRestrictionGoal`, speed `1`;
- priority `7`, `WaterAvoidingRandomStrollGoal`, speed `1`, probability `0`;
- priority `8`, `LookAtPlayerGoal`, range `8`; and
- priority `8`, `RandomLookAroundGoal`.

The target selector contains priority `1` `HurtByTargetGoal` with
`setAlertOthers()` and priority `2`
`NearestAttackableTargetGoal<Player>` with must-see enabled and default
ten-tick target-search cadence. The active priority-four attack goal blocks
lower conflicting movement and look goals.

The attack goal can start only with a non-null, living target the Blaze can
attack. Start resets `attackStep` to zero but deliberately leaves
`attackTime` unchanged. Stop clears charge and `lastSeen`, but also leaves
`attackTime` unchanged. Because the delay decrements only while this goal is
running, stopping freezes a positive remainder; restarting waits that
remainder before beginning a new warmup.

Every required-update tick first decrements `attackTime`. Visible targets
reset `lastSeen` to zero; otherwise it increments. Let `d²` be squared
entity-to-target distance:

1. At strict `d²<4`, absent line of sight aborts the tick. With line of sight,
   an expired delay is reset to `20` and calls generic melee attack; every
   such tick requests movement to the target's exact coordinates at speed
   `1`.
2. At strict `4<=d²<48²` with line of sight, the ranged state machine runs
   and Look Control faces the target with yaw/pitch limits `10`.
3. Otherwise, while `lastSeen<5`, Move Control requests the exact target
   coordinates at speed `1`. A visible target at or beyond `48` therefore
   remains followed indefinitely; after line of sight is lost only counts
   `1..4` follow.

Equality at `d²=4` enters ranged logic. Equality at `d²=2304` enters the
fallback-follow branch. Close combat, temporary line-of-sight loss and
out-of-range movement do not reset attack step or charge, so a Blaze can
remain charged while meleeing and later continue its pending volley.

When the ranged branch finds `attackTime<=0`, it increments `attackStep`:

- step `1` writes delay `60` and charge true, without firing;
- steps `2`, `3` and `4` each write delay `6` and launch one Small Fireball;
- step `5` writes delay `100`, resets step to `0` and clears charge, without
  firing.

The nominal uninterrupted cycle therefore has a 60-tick warmup, three
projectiles six ticks apart and a 100-tick post-volley delay.

### Projectile construction and volley effects

For each shot, target differences are:

- `dx = targetX-blazeX`;
- `dy = target.getY(0.5)-blaze.getY(0.5)`; and
- `dz = targetZ-blazeZ`.

With `s=0.5*sqrt(sqrt(d²))`, X and Z are sampled as
`triangle(dx,2.297*s)` and `triangle(dz,2.297*s)`; Y remains exact. Each
triangle consumes two doubles. The direction is normalized before the
Small-Fireball constructor, whose generic Hurting-Projectile construction
normalizes again and installs acceleration of magnitude `0.1`.

Before construction, a nonsilent Blaze emits level event `1018` at its block
position with data `0`, excluding no player. The new projectile initially
uses the Blaze position and owner, then its position is overwritten to
`(blazeX, blaze.getY(0.5)+0.5, blazeZ)`, normally base Y plus `1.4`.
`addFreshEntity` follows and its Boolean result is ignored: insertion failure
does not roll back the event or four consumed doubles.

Event `1018` makes each admitted client play Blaze Shoot, sound protocol ID
`178`, in `HOSTILE` at volume `2` and pitch
`1+(nextFloat-nextFloat)*0.2`. The owned projectile rule then fixes flight,
collision and impact: an entity hit attempts five seconds of ignition and
five Fireball damage, restoring prior fire ticks if damage fails; a block hit
places adjacent Fire only when the mob-griefing gate admits it; an admitted
server hit discards the projectile.

### Hovering, height pursuit and client ambience

`Blaze.aiStep` first checks descent. When airborne with negative Y velocity,
it replaces Y velocity by `0.6*oldY`, preserving X/Z. On a client it then:

1. consumes `nextInt(24)` every step;
2. on zero and while nonsilent, plays local Blaze Burn at
   `(X+0.5,Y+0.5,Z+0.5)`, with volume `1+nextFloat` and pitch
   `0.3+0.7*nextFloat`; and
3. always requests two Large-Smoke particles, each using
   `getRandomX(0.5)`, `getRandomY()` and `getRandomZ(0.5)`, for six doubles
   total.

A selected but silent burn branch spends no floats. Local particles remain
subject to ordinary client distance and particle-status admission. Large
Smoke has particle protocol ID `62`. Only after these subtype operations does
the complete inherited Monster AI step run.

On each server custom-AI step, Blaze first decrements
`nextHeightOffsetChangeTick`. At zero or below it writes `100` and samples
`allowedHeightOffset=(float)triangle(0.5,6.891)`, consuming two doubles and
approaching the open interval `(-6.391,7.391)`. The initial zero therefore
refreshes on the first server AI step.

When an attackable target's eye Y is strictly above
`blazeEyeY+allowedHeightOffset`, Y velocity changes to
`oldY+0.3*(0.3-oldY)`, equivalently `0.7*oldY+0.09`, and marks movement for
synchronization. X/Z remain unchanged. The inherited Monster custom-AI step
then runs.

### Damage, water and freezing

Fire immunity makes generic `IS_FIRE` damage sources invulnerable. Blaze is
directly in exactly two entity-type tags:

- `fall_damage_immune`, suppressing fall damage and its damage sources; and
- `freeze_hurts_extra_types`, multiplying incoming freezing damage amount by
  `5`.

The type is not freeze-immune and can become fully frozen unless equipment
prevents it. A fully frozen baseline request of one damage every 40 ticks
therefore becomes raw five before ordinary damage mitigation and cooldown.

Blaze also overrides `isSensitiveToWater()` true. During the Living-Entity
server AI step, being in water or rain submits one Drown damage every AI
step; the generic transaction decides which submissions commit. A thrown
Water Potion within squared distance strictly below `16` submits one
Indirect-Magic damage. Snowball impact directly recognizes Blaze and requests
three damage. Those delivery algorithms, cooldowns and side effects remain
with their projectile and damage owners.

### Placement, fortress selection and spawners

Blaze registers `ON_GROUND` placement with
`MOTION_BLOCKING_NO_LEAVES` and `checkAnyLightMonsterSpawnRules`, not the
darkness predicate. The outer on-ground placement requires the candidate
inside the world border, valid support below and valid empty spawn blocks at
the candidate and above. The species predicate delegates only to generic Mob
spawn rules: outside `SPAWNER` reason it again requires valid support below;
for `SPAWNER` it passes without light, support or RNG. The generic type gate
still rejects Blaze in Peaceful.

None of the 66 biome JSONs contains a Blaze spawn row. The Nether Fortress
structure instead defines a `MONSTER` spawn override bounded by `"piece"`.
Its rows are Blaze weight `10`, group `2..3`; Zombified Piglin `5/4..4`;
Wither Skeleton `8/5..5`; Skeleton `2/5..5`; and Magma Cube `3/4..4`.
Baseline natural Blaze selection therefore occurs only within qualifying
fortress piece boxes, in any light, under the Monster cap and distance
pipeline. The inherited per-cluster maximum remains `4`.

The fortress throne piece also places a code-built ordinary spawner at local
`(3,5,5)`. Its saved `Mob` placement latch commits before it offers the
spawner block; only a successfully obtained `SpawnerBlockEntity` is
configured for Blaze. The resulting block uses ordinary defaults: delay
`20`, minimum/maximum delay `200/800`, spawn count `4`, maximum nearby `6`,
required player range `16` and spawn range `4`. Its rule calls Blaze
placement with reason `SPAWNER` after collision checks, then applies generic
obstruction, no-liquid, finalization and insertion. `spawner_blocks_work`,
not `spawn_mobs` or `spawn_monsters`, gates this path.

Spawn Eggs and commands retain their generic placement-bypass behavior.
Exact UTF scanning finds no `minecraft:blaze` in any of the 1,212 structure
templates; the throne spawner is generated by code rather than template NBT.

### Despawn, loot, progression and sounds

Monster category cap is `70`; it is hostile, nonpersistent, with no-despawn
and despawn distances `32/128`. Peaceful immediately discards it. Beyond the
generic distance branches, a nonpersistent Blaze with `noActionTime>600`
becomes eligible for the `nextInt(800)==0` random-despawn branch; its
always-one light score normally accelerates arrival at that idle boundary.

The entity loot table, owned with Blaze material behavior, has type `entity`,
sequence `minecraft:entities/blaze` and one roll gated by
`killed_by_player`. It emits Blaze Rod raw item ID `1145`, base uniform
integer count `0..1`; a living attacker with Looting level `L>0` consumes a
fresh float `U` and adds `round(L*U)`. Generic death admission can award XP
`10`.

Common Blaze Spawn Egg is raw item ID `1233`, maximum stack `64`, with
`entity_data.id=minecraft:blaze` and generic Spawn-Egg use, dispenser and
projection. Exact entity-type conditions occur only in
`adventure/kill_a_mob` and `adventure/kill_all_mobs`; the former is one
alternative in an OR requirement, while the latter has its own required
criterion group. `nether/obtain_blaze_rod` tests item possession rather than
entity identity.

Ambient, burn, death, hurt and shoot use sound protocol IDs
`174/175/176/177/178`, with English subtitles `Blaze breathes`, `Blaze
crackles`, `Blaze dies`, `Blaze hurts` and `Blaze shoots`. Parrot imitation
maps Blaze to sound ID `1214`, subtitle `Parrot breathes`. Ambient, hurt and
death retain generic voice admission, attenuation and pitch behavior; burn
and shoot use the exact local/event paths above.

### Legacy schema and client projection

Exactly nine migration/schema contexts own Blaze identity:

- `EntityHealthFix` recognizes legacy `Blaze`;
- `EntityIdFix` maps `Blaze` to `minecraft:blaze`;
- `EntityUUIDFix` processes the modern Mob shape;
- `ItemSpawnEggFix` maps legacy generic Spawn Egg damage `61` to `Blaze`;
- `ItemStackSpawnEggFix` maps the modern entity to
  `minecraft:blaze_spawn_egg`;
- `StatsCounterFix` recognizes old Blaze statistics;
- `V99` registers the legacy simple entity; and
- `V705` and `V1460` register the modern Mob/Spawn-Egg shapes.

The legacy Spawn-Egg damage `61` is unrelated to current entity protocol ID
`14`. No fix rewrites Blaze subtype state because charge and height/attack
counters are not persisted.

`EntityRenderers` binds Blaze to `BlazeRenderer`, with shadow radius `0.5`.
It always returns block light `15`, independent of world light and charge;
sky light remains generic. Texture `textures/entity/blaze/blaze.png` is
`64×32`, `370` bytes, SHA-1
`98a3c485c6e9ca4032070629497d768870ebb610`.

The `64×32` model has one `8×8×8` head and twelve `2×8×2` rods. Four upper
rods animate at radius `9`, phase speed `-0.1*pi*t` and Y
`-2+cos((2i+t)*0.25)`; four middle rods use radius `7`, initial phase
`pi/4`, speed `0.03*pi*t` and Y `2+cos((2i+t)*0.25)`; four lower rods use
radius `5`, initial phase `0.47123894`, speed `-0.05*pi*t` and Y
`11+cos((1.5i+t)*0.5)`. Each successive rod in a group adds `pi/2` to phase.
Head yaw/pitch convert render-state degrees by `pi/180`.

Charge is not copied into a dedicated Blaze render field. Generic living
render extraction calls the virtual `displayFireAnimation()`, however, which
sees charged `isOnFire()` and enables the overlay unless the entity is a
spectator. English names are `Blaze` and `Blaze Spawn Egg`.

**Branches and aborts:**

- Attack start resets step but retains delay; stop clears charge/visibility
  count while retaining delay.
- Strict distance and line-of-sight branches can preserve a charged,
  partially completed volley while meleeing or following.
- Each shot's event and RNG precede insertion, whose result is ignored.
- Height pursuit uses a strict eye-height comparison and samples on the first
  server AI step.
- Client burn selection precedes particles; silence suppresses two floats,
  not the cadence draw.
- Any-light placement has no random light check, and spawner reason bypasses
  its support predicate.
- Natural selection is a fortress piece override, not a biome row.

**Constants and randomness:**

Entity/Egg/Rod IDs `14/1233/1145`; dimensions/eye `0.6×1.8/1.53`;
tracking/update `8/3`; health/attack/speed/follow `20/6/0.23/48`; XP `10`;
metadata `0..15 inherited, 16 BYTE bit 0`; goal priorities
`4/5/7/8/8`, targets `1/2`; melee square/delay `4/20`; range square `2304`;
seen cutoff `5`; volley `60,6,6,6,100`; spread coefficient `2.297` and four
doubles per shot; projectile acceleration `0.1`; event `1018`; descent
`0.6`; height cadence/triangle/pursuit `100/0.5±6.891/0.7y+0.09`; burn
`1/24`, two floats; smoke `2`, six doubles, particle `62`; water/potion/
snowball/freeze raw damage `1/1/3/5`; fortress Blaze row `10/2..3`;
category `70/32/128`, cluster `4`; loot `0..1+round(LU)`; sounds
`174..178/1214`; tags/templates/schemas `2/0 of 1212/9`; shadow/light
`0.5/15`; texture as above.

**Side effects:**

Charge metadata and fire overlay; retained attack counters; movement and
height synchronization; projectile construction, sound events, ignition,
damage and Fire placement; client RNG, sounds and particles; water/freeze/
fall interactions; spawn selection/finalization/despawn; loot, XP,
advancements, Parrot imitation and client model/texture/light.

**Gates:**

Logical side, target/life/attackability, exact distance and line of sight;
goal controls and retained counters; silence/client RNG; eye-height offset;
damage tags/source/cooldown; Peaceful/world border/support/spawn reason;
fortress piece bounds/category caps; spawner gamerule/player/collision;
death attacker/Looting; advancement requirements, migration shape and client
render state.

**Boundary cases and quirks:**

Charge makes a fire-immune entity report on fire. Goal interruption freezes
the delay but resets the volley step. Equality at two blocks is ranged, while
equality at 48 blocks is follow behavior. A visible target beyond follow
range remains followed because visibility continually zeros `lastSeen`.
Natural Blaze spawning is any-light but still fortress-selected. The
fortress throne installs an ordinary spawner rather than a special Blaze
algorithm.

**Failure semantics:**

Rejected placement prevents natural construction/insertion. Generic spawn
finalization or insertion failure retains owner behavior. Projectile
insertion failure leaves the preceding shoot event and RNG committed.
Rejected damage leaves health and cooldown under the generic transaction;
failed Fireball damage restores the target's earlier fire ticks. A failed
throne block-entity lookup leaves its earlier piece latch/block effects.

**Client/server authority split:**

The server owns AI, target/charge metadata, height sampling and pursuit,
projectile creation, placement, damage, loot and XP. The client applies
descending damping too, consumes its local cadence/particle RNG, plays the
periodic burn sound, requests smoke and renders synced charge through the
generic fire overlay. Shoot-event recipients randomize pitch locally.

**Observability:**

Observe registration and attributes; slot-16 bits and fire predicates;
goal start/stop retained state, distance equality, LOS count and volley
timing; shot RNG/origin/event/insertion order; vertical formulas and both-side
RNG; damage/tag behavior; fortress override/spawner/natural-selection
boundaries; loot/XP/criteria/sounds; template/schema closure and exact
light/model/texture/overlay projection.

**Persistence and reload:**

Only generic entity/Mob state persists; charge, height offset/cadence and all
attack-goal counters reset on load. Code fixes type, attributes, goals,
placement, fortress piece behavior and schemas. Structure override, tags,
loot, advancements and Egg components reload through their owners; language
and texture are client resources.

**Evidence:**

`net.minecraft.world.entity.EntityTypes`;
`net.minecraft.world.entity.ai.attributes.DefaultAttributes`;
`net.minecraft.world.entity.SpawnPlacements`;
`net.minecraft.world.entity.SpawnPlacementTypes`;
`net.minecraft.world.entity.MobCategory`;
`net.minecraft.world.entity.monster.Blaze` and `Blaze$BlazeAttackGoal`;
`net.minecraft.world.entity.monster.Monster`;
`net.minecraft.world.entity.LivingEntity`;
`net.minecraft.world.entity.projectile.SmallFireball`;
`net.minecraft.world.entity.projectile.Snowball`;
`net.minecraft.world.item.ThrowablePotionItem`;
`net.minecraft.world.level.NaturalSpawner`;
`net.minecraft.world.level.levelgen.structure.structures.NetherFortressPieces`;
`net.minecraft.world.level.BaseSpawner`;
`net.minecraft.client.renderer.LevelEventHandler`;
`net.minecraft.client.renderer.entity.EntityRenderers` and `BlazeRenderer`;
`net.minecraft.client.model.monster.BlazeModel`; all named data-fix classes
and schemas above; reports, tags, loot, advancements, fortress structure, all
66 biomes, all 1,212 templates, Egg components, texture, sounds and language.
Complete compiled/data identity searches find no other direct runtime path.

**Test vectors:**

Run `EXP-ENT-011` across metadata/fire/damage state, every attack-goal
start/stop/range/LOS/volley branch, exact projectile RNG/event/origin/
insertion order, both-side vertical and ambience paths, height sampling,
water/freeze/fall behavior, fortress override and throne-spawner boundaries,
generic despawn, loot/XP/advancements, Spawn Egg, templates/migrations and
client sound/model/texture/light/overlay projection.

**Limits:**

Generic entity lifecycle, targeting/navigation, damage/death, natural spawn,
spawner, projectile, loot evaluation, Spawn Egg, metadata packets and
rendering retain their owners. Fortress generation and Blaze material
progression retain their leaves. This leaf fixes exact Blaze dispatch and
every direct join selecting it.
