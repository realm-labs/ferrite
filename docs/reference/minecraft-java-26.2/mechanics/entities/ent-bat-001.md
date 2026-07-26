# Entities mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ENT-BAT-001` — Bats alternate between ceiling rest and transient-target flight under exact spawn and wake gates

**Parent:** `ENT-001`, `ENT-LIFECYCLE-001`, `ENT-002`,
`ENT-VEHICLE-001`, `ENT-004`, `ENT-PROJECTILE-001`, `ENT-005`,
`ENT-DAMAGE-001`, `ENT-BLOCK-001`, `ENT-DAMAGE-REDUCE-001`,
`ENT-KNOCKBACK-001`, `ENT-006`, `ENT-EFFECT-001`, `ENT-007`,
`ENT-DEATH-001`, `MOB-001`, `MOB-AI-001`, `MOB-002`,
`MOB-SPAWN-001`, `MOB-003`, `MOB-DESPAWN-001`, `MOB-005`,
`PLY-AUTOJUMP-001`, `WGEN-005`, `WGEN-PORTAL-001`, `CLI-001`,
`CLI-006`, `CLI-EFFECT-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked entity registration, `Bat`, attribute and spawn
bootstraps, generic Ambient/Mob owners, tags, empty loot, all 66 biome records,
all `1,212` templates, four migration contexts and exact client renderer,
model, animation, texture, sounds and language resources close protocol entity
ID `10`. In 26.2 the spawn predicate contains no date or Halloween branch.

**Applies when:**

`minecraft:bat` is constructed, spawned naturally or explicitly, loaded,
resting, awakened, flying, targeted by a nearby player, damaged, pushed,
leashed, killed, despawned, synchronized or rendered, or when its generic
Spawn Egg is used.

**Authoritative state:**

Entity protocol ID `10` constructs `Bat` in category `AMBIENT`. Registration
fixes width/height `0.5/0.9`, eye height `0.45`, client tracking range `5` and
default update interval `3`; it retains ordinary summon/save, fire and loot
properties. The Ambient category has cap `15`, is friendly and nonpersistent,
and uses no-despawn/despawn distances `32/128`.

Default Mob attributes plus the Bat override yield maximum health `6` and
inherited follow range `16`; flight does not use a movement-speed goal.
`AmbientCreature.canBeLeashed` is false. Bat is not pushable, its direct push
hook and push-nearby-entities hook are no-ops, and it registers no goals or
targets.

Bat adds one synchronized `BYTE` accessor at metadata slot `16`, serializer ID
`0`, default value `0`. Bit `0` means resting. `setResting(true)` ORs `1`;
`setResting(false)` ANDs `-2`, preserving every unrelated bit. Server-side
construction immediately sets resting; client construction retains zero until
metadata arrives.

Subtype persistence is only byte key `BatFlags`. Saving writes the complete
metadata byte. Loading reads it with default `0` and replaces the accessor, so
a legacy/malformed record missing `BatFlags` changes the server-constructed
resting state to flying. The transient flight target and both client animation
timers are not persisted.

**Transition and ordering:**

### Common tick and rest pose

After the inherited Ambient/Mob/Living tick completes, Bat:

- when resting, sets delta movement to exact zero and raw-snaps Y to
  `floor(currentY)+1-getBbHeight()`, hence the ceiling plane minus `0.9`; or
- when flying, multiplies only velocity Y by `0.6`, leaving X and Z unchanged.

It then stops the flying animation and starts the resting animation at current
`tickCount`, or performs the inverse, using `startIfStopped`. State change
therefore restarts only the newly selected animation; continued state does not
restart it.

Movement emission is `EVENTS`. While flying, `isFlapping` is true exactly when
`tickCount % 10 == 0`. The inherited flapping processor has no Bat sound hook
but emits game event `FLAP`; a resting Bat never supplies that pulse.

### Resting server AI

Each server AI step captures `P=blockPosition()`, `U=P.above()` and the silent
flag. It reads the state at `U` and tests that state as a Redstone conductor
with level/position arguments `(level,P)`.

If the ceiling is a conductor:

1. draw `nextInt(200)`; on zero, set head yaw directly to
   `nextInt(360)`, consuming the second draw only on that branch;
2. find the nearest player under noncombat targeting range `4`, including the
   generic visibility-percent distance reduction and line-of-sight tests; and
3. if a player qualifies, clear resting and, unless silent, emit level event
   `1025` at `P` with data `0`.

If the ceiling is not a conductor, it clears resting immediately and, unless
silent, emits the same event; that branch consumes neither head draw nor
player search. Event `1025` is client-projected as Bat Takeoff sound ID `151`,
source `NEUTRAL`, volume `0.05`, pitch
`1 + (nextFloat()-nextFloat())*0.2`. The two pitch draws are client-local.

### Damage wake

`hurtServer` first tests generic source invulnerability. An invulnerable hit
returns false without changing rest. Otherwise a resting Bat clears its bit
before delegating the complete damage pipeline. Thus cooldown, armor/effect or
other downstream rejection can leave a Bat awake even when generic damage
returns false. This path does not emit event `1025`; ordinary hurt/death
sounds and all damage, knockback, attribution and death semantics retain their
owners.

### Flying server AI

If transient `targetPosition=T` exists, it is cleared when `T` is nonempty or
`T.y <= level.minY`. With a remaining target, the Bat draws `nextInt(30)` and
keeps it only when that draw is nonzero and its center is not within distance
`2` of the Bat. A null, invalid, one-in-30, or reached target is replaced by:

- `floor(x + nextInt(7) - nextInt(7))`;
- `floor(y + nextInt(6) - 2)`; and
- `floor(z + nextInt(7) - nextInt(7))`.

X/Z offsets are triangular over `-6..6`; Y offset is uniform over `-2..3`.
Target-null short-circuits the one-in-30 draw; target invalidation happens
before it. Regeneration consumes the five coordinate draws in X/X/Y/Z/Z order.

Let `d` be target center `(T.x+0.5,T.y+0.1,T.z+0.5)` minus current position and
`v` the current velocity. The offered velocity is:

`v + ((sign(d.x)*0.5-v.x)*0.1,
      (sign(d.y)*0.7-v.y)*0.1,
      (sign(d.z)*0.5-v.z)*0.1)`.

It computes target yaw
`atan2(newV.z,newV.x)*57.2957763671875-90`, wraps the difference from current
yaw, sets forward input `zza=0.5`, and adds the wrapped difference, reaching
the target yaw exactly.

Finally it draws `nextInt(100)`. Only on zero does it read the state at `U`;
if that state is a Redstone conductor under `(level,U)`, it sets resting. That
landing branch emits no takeoff event. At the end of the outer Bat tick the
new Y velocity is multiplied by `0.6`.

### Ambient, hurt and death sounds

Bat sound volume is `0.1`; its voice pitch is inherited pitch multiplied by
`0.95`. Flying ambient queries always return Ambient ID `147`. A resting query
first draws `nextInt(4)` and returns null for results `1..3`, so only result
zero selects Ambient. Hurt and death select IDs `149/148`. The sound resource
maps Ambient to four clips, Hurt to four, Death to one, Takeoff to one and
registered Loop ID `150` to one; complete compiled consumer search finds no
Bat-specific code path selecting Loop.

### Falls, blocks, loot and ordinary interaction

`checkFallDamage` is empty and `isIgnoringBlockTriggers` returns true. Bat also
belongs directly to entity-type tag `fall_damage_immune`; no parent tag adds
another membership. These independent routes converge on no Bat fall damage
and no ordinary movement block-trigger processing.

The entity loot table has type `entity`, sequence
`minecraft:entities/bat`, and no pools. Bat sets no XP reward, so its ordinary
death produces zero item rolls and zero experience before generic external
hooks. It has no equipment, pickup, breeding, tame, bucket, food, trade,
attack, projectile or use override. Naming can make the generic nonpersistent
Mob persistence-required; otherwise Ambient despawn remains owned by
`MOB-DESPAWN-001`.

The common Bat Spawn Egg is raw item ID `1171`, stack size `64`, with
`entity_data.id=minecraft:bat`; placement, spawner use, component patches and
construction failure retain the generic spawn-egg owner.

### Natural spawning and worldgen data

Bat registers `ON_GROUND` placement with heightmap
`MOTION_BLOCKING_NO_LEAVES`. Its species predicate runs in this exact order:

1. reject when candidate Y is at or above the `WORLD_SURFACE` heightmap Y;
2. draw `nextBoolean` and reject on true;
3. read maximum local raw brightness, then draw `nextInt(4)`, rejecting when
   brightness is greater than the draw;
4. require the block below in live tag `bats_spawnable_on`; and
5. delegate `Mob.checkMobSpawnRules`, which accepts spawner reasons or
   otherwise requires the below state to be valid spawn support.

The first rejected height consumes no RNG; the Boolean-rejected branch skips
brightness and all later work. Conditional on passing the half gate, raw
brightness `0/1/2/3` passes with probability `1,3/4,1/2,1/4`, and brightness
at least `4` fails. Combined species probabilities before support/generic
gates are therefore `1/2,3/8,1/4,1/8,0`. There is no date, season or
Halloween test in locked 26.2 `Bat`.

`bats_spawnable_on` contains one nested `base_stone_overworld` tag, whose
closure is exactly Stone, Granite, Diorite, Andesite, Tuff and Deepslate.
Reload changes future predicate reads.

Exactly 54 biome records include Bat in `ambient`, each at weight `10` and
declared minimum/maximum group `8/8`. These are every ordinary Overworld biome
except Deep Dark; the five Nether, five End, The Void and Deep Dark records
omit it. The Ambient category cap is `15`, while an instantiated Bat inherits
Mob maximum spawn-cluster size `4`; the natural-spawn owner composes the
data-requested group, local/global caps, candidate walk, species predicate,
cluster limit and insertion rather than treating `8` as an unconditional
spawn count.

Exhaustive decoded/string scans of all `1,212` templates find zero Bat entity
NBT, palette, final-state, marker or block-entity occurrence. There is no
structure-specific Bat spawn override.

### Legacy migration

Four exact fix contexts select Bat:

- `EntityIdFix` maps legacy `Bat` to `minecraft:bat`;
- `ItemStackSpawnEggFix` maps modern Bat entity identity to
  `minecraft:bat_spawn_egg`;
- `EntityUUIDFix` includes `minecraft:bat` in its generic mob UUID migration;
  and
- `StatsCounterFix` maps old `Bat` statistic keys to `minecraft:bat`.

Schemas `V705` and `V1460` also declare the old/modern entity shapes. No fix
rewrites `BatFlags`; the live reader's missing-key default is authoritative.

### Client projection

`EntityRenderers` binds Bat to `BatRenderer`. The renderer uses shadow radius
`0.25`, fixed texture `textures/entity/bat/bat.png`, and a dedicated render
state containing the resting bit plus copied flying/resting animation states.
The texture is exact 32×32, 329 bytes, SHA-1
`29cb376f1ae92afaa836f953532fd5c974a4d68a`.

`BatModel` uses a 32×32 atlas with body/head cuboids and zero-thickness
ear/wing/wing-tip/feet planes. Both resting and flying animation definitions
loop with length `0.5` seconds, matching `10` ticks per flap. Resting setup
also applies render yaw to head Y rotation before both animation channels.

English entity/item names are `Bat` and `Bat Spawn Egg`; Ambient, Death, Hurt
and Takeoff subtitles are `Bat screeches`, `Bat dies`, `Bat hurts` and
`Bat takes off`. The spawn egg uses the generic spawn-egg client model/tab
path.

**Branches and aborts:**

- Server construction sets rest; loading a missing `BatFlags` overwrites it
  with zero.
- Invalid ceiling wakes before any resting RNG/player work.
- A valid ceiling always consumes `nextInt(200)` before player targeting.
- Noninvulnerable damage wakes before downstream damage admission.
- Null/invalid/reached/one-in-30 targets regenerate with five ordered draws.
- The one-in-100 landing draw precedes its ceiling read.
- Spawn height, Boolean, brightness/draw, tag and generic validity short-circuit
  in order.

**Constants and randomness:**

Entity/item IDs `10/1171`; dimensions/eye `0.5×0.9/0.45`; tracking/update
`5/3`; health/follow range `6/16`; AMBIENT cap/persistence/distances
`15/false/32/128`; metadata `slot16/BYTE0/bit0=value1`; target range `4`; head
`1/200` then `0..359`; X/Z `r7-r7`; Y `r6-2`; target distance `2`; velocity
targets `0.5/0.7/0.5` and gain `0.1`; vertical damping `0.6`; landing `1/100`;
flap `10`; volume/pitch factor `0.1/0.95`; takeoff `0.05`,
`1+0.2(r1-r2)`; spawn half gate and `nextInt(4)`; support `1→6`; biomes
`54`, weight/group `10/8/8`, cluster `4`; memberships `1`; loot/XP `0/0`;
templates/cells `1212/0`; shadow/animations/texture `0.25/2×0.5s/32×32`.

**Side effects:**

Metadata/save mutation; raw rest positioning and motion; target sampling,
velocity/yaw/input; animation state; FLAP game and takeoff level/sound events;
damage/death/despawn; spawn candidate reads/RNG/insertion; spawn-egg
construction; synchronization and rendering.

**Gates:**

Logical side and construction/load; resting bit; ceiling conductivity,
silence and player targeting; invulnerability and generic damage; target
validity/proximity/RNG; landing draw; spawn height/Boolean/brightness/tag/
valid support/caps/cluster/insertion; persistence/despawn; loot and client
resource validity.

**Boundary cases and quirks:**

Missing `BatFlags` loads flying despite the server constructor's rest default.
Rejected noninvulnerable damage can still wake. Player/ceiling waking emits
takeoff, but damage waking does not. A flying Bat uses goal-free direct
velocity despite retaining generic ground navigation. Biome group `8` and
Mob cluster limit `4` are distinct layers. There is no 26.2 calendar bonus.

**Failure semantics:**

Generic construction, metadata, movement, damage, death, spawn and despawn
owners retain their commit rules. Bat state changes are direct: wake occurs
before delegated hurt; event suppression by silence does not restore rest;
target/RNG/motion work does not roll back on later movement or insertion
failure.

**Client/server authority split:**

The server owns rest, transient targets, AI, motion, wake events, damage,
spawn, persistence and death. Metadata slot `16`, transforms and level/game
events synchronize. The client projects event-1025 pitch RNG, animation
states, model, texture, sounds, names and spawn egg.

**Observability:**

Observe registry/dimensions/attributes, metadata and `BatFlags`, exact
rest/flight tick order, every RNG cursor/read/mutation/event, player
visibility/line-of-sight, hurt return versus wake, spawn short circuits,
biome/cap/group/cluster, interaction/death outputs, closures/template/fix
census, packets and exact render/sound projection.

**Persistence and reload:**

Generic entity state plus `BatFlags` persists; target and animation timers do
not. Entity type, dimensions, attributes, metadata layout, AI and migrations
are code-built. Biomes, tags, loot, spawn-egg components, sounds, texture and
language reload through their owners.

**Evidence:**

`net.minecraft.world.entity.EntityTypes`;
`net.minecraft.world.entity.ai.attributes.DefaultAttributes`;
`net.minecraft.world.entity.SpawnPlacements`;
`net.minecraft.world.entity.MobCategory`;
`net.minecraft.world.entity.Mob`;
`net.minecraft.world.entity.Entity`;
`net.minecraft.world.entity.ambient.AmbientCreature`;
`net.minecraft.world.entity.ambient.Bat`;
`net.minecraft.client.renderer.LevelEventHandler`;
`net.minecraft.client.renderer.entity.EntityRenderers`;
`net.minecraft.client.renderer.entity.BatRenderer`;
`net.minecraft.client.model.geom.LayerDefinitions`;
`net.minecraft.client.model.ambient.BatModel`;
`net.minecraft.client.animation.definitions.BatAnimation`; four fix classes;
reports, two tags, empty loot, all 66 biomes, all `1,212` templates, Spawn Egg
components, 32×32 texture, sounds and language. Complete compiled/data/fix/NBT
identity searches find no other exact runtime path.

**Test vectors:**

Run `EXP-ENT-007` across construction/load/metadata, every rest/player/ceiling/
silence/damage/sound branch, all target and landing draws, exact motion/yaw/
damping/flap ticks, spawn height/brightness/support/reason in all 54 biomes,
cap/group/cluster/despawn, interactions/death, closures/templates/fixes,
reload, protocol state and client projection.

**Limits:**

Generic entity lifecycle, movement/collision, damage/effects/death, spawn
engine, despawn, spawn egg, metadata packet and rendering retain their owners.
Support blocks and player targeting retain their own rules. This leaf fixes
exact Bat and every direct join selecting it.
