# Entities mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ENT-ENDERMITE-001` — Endermites expire by persisted lifetime and enter baseline worlds through player Ender Pearls

**Parent:** `ENT-001`, `ENT-LIFECYCLE-001`, `ENT-002`,
`ENT-VEHICLE-001`, `ENT-004`, `ENT-PROJECTILE-001`, `ENT-005`,
`ENT-DAMAGE-001`, `ENT-BLOCK-001`, `ENT-DAMAGE-REDUCE-001`,
`ENT-KNOCKBACK-001`, `ENT-006`, `ENT-EFFECT-001`, `ENT-007`,
`ENT-DEATH-001`, `MOB-001`, `MOB-AI-001`, `MOB-002`,
`MOB-SPAWN-001`, `MOB-003`, `MOB-DESPAWN-001`, `MOB-005`,
`ITM-ENCHANT-001`, `PLY-AUTOJUMP-001`, `WGEN-005`,
`WGEN-PORTAL-001`, `CLI-001`, `CLI-006`, `CLI-EFFECT-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked registration, the complete `Endermite` class,
generic goal/spawn/death owners, Enderman and Ender-Pearl callers, item/sound/
tag/loot/advancement/biome records, all `1,212` templates, nine migration/
schema classes and exact client renderer/model/texture/language resources
close protocol entity ID `42`.

**Applies when:**

`minecraft:endermite` is constructed, spawned by a player Ender Pearl, a
Spawn Egg, spawner, command or custom biome row, loaded, persisted or named,
ticked, targeted by a player or Enderman, damaged, killed, despawned,
synchronized or rendered.

**Authoritative state:**

Entity protocol ID `42` constructs `Endermite` in category `MONSTER`.
Registration fixes width/height `0.4/0.3`, eye height `0.13`, one passenger
attachment at Y `0.2375`, client tracking range `8`, builder-default update
interval `3`, and disallowance in Peaceful. Ordinary summon/save, fire, loot
and far-spawn defaults remain.

Attributes are maximum health `8`, movement speed `0.25`, attack damage `2`
and inherited follow range `16`. Construction replaces Monster's reward with
XP reward `3`.

Endermite declares no synchronized accessor, so wire metadata contains only
inherited Entity/Living/Mob slots `0..15`. Its only subtype server field is
signed int `life`, initialized to `0` and persisted as `Lifetime`. Loading
reads missing or wrong-type data as `0`; there is no clamp, subtype metadata or
historical `PlayerSpawned` state in locked 26.2.

**Transition and ordering:**

### Lifetime and pose

Each `aiStep` first runs the complete Monster/Pathfinder/Mob AI transaction.
On the server, if persistence is not required, it increments `life` by one.
It then discards when `life >= 2400`, whether persistent or not. Persistence
therefore pauses future increments but neither clears an accumulated value nor
saves a value already at the removal threshold.

A fresh nonpersistent Endermite is discarded on the AI step that changes
`2399` to `2400`, nominally after `120` seconds at 20 steps per second. A
loaded negative value extends the lifetime; a loaded `2400` or larger value is
discarded on its next AI step even when named/persistent. Java signed-int
increment is not saturated: `2147483647` wraps to `-2147483648` before the
comparison for a nonpersistent entity.

Before each inherited `tick`, Endermite copies entity yaw into body yaw.
`setYBodyRot(v)` first sets entity yaw to `v`, then performs the inherited body
write, keeping body and entity orientation coupled. Movement emission is
`EVENTS`.

On the client, after inherited AI, exactly two Portal particles are added each
tick. Each particle calls randomized X within radius `0.5`, randomized Y
within the bounding height and randomized Z within radius `0.5`, then draws
velocity `(2*(r-0.5), -r, 2*(r-0.5))`. Position helpers consume three doubles
and velocity consumes three more, for twelve client-local uniform-double draws
across the pair. These particles are not subtype state and are not sent by the
server.

### Goals, targets and attacks

The goal selector is registered exactly as:

- priority `1`: `FloatGoal`;
- priority `1`: `ClimbOnTopOfPowderSnowGoal`;
- priority `2`: `MeleeAttackGoal`, speed `1`, no long memory;
- priority `3`: `WaterAvoidingRandomStrollGoal`, speed `1`;
- priority `7`: `LookAtPlayerGoal`, Player range `8`; and
- priority `8`: `RandomLookAroundGoal`.

The target selector registers priority `1` `HurtByTargetGoal` and calls
`setAlertOthers()` with an empty ignored-class array. On start it targets the
last attacker and alerts otherwise idle same-class peers in a box inflated by
follow range `16` on X/Z and `10` on Y, subject to generic alliance and target
checks. Priority `2` is `NearestAttackableTargetGoal<Player>` with the default
cadence, required visibility and follow-range `16`. Generic target admission,
navigation, melee cooldown, damage `2`, enchantment and knockback remain with
their owners.

Enderman independently registers a priority `3`
`NearestAttackableTargetGoal<Endermite>` with default interval `10`, required
visibility and no reachability requirement. It applies to every Endermite:
locked 26.2 has no player-spawned flag for this goal to distinguish.

### Registered placement

Endermite registers `ON_GROUND` with heightmap
`MOTION_BLOCKING_NO_LEAVES` and its species predicate:

1. `Monster.checkAnyLightMonsterSpawnRules` delegates directly to generic Mob
   support validity, with no sky/block/local-light read or RNG;
2. a spawner reason therefore passes both generic support and the later
   species branch immediately; otherwise
3. search from candidate center `(x+0.5,y+0.5,z+0.5)` for the nearest player
   under `NO_CREATIVE_OR_SPECTATOR` within radius `5`; and
4. accept only when that search returns null.

The nearest-player implementation uses squared distance strictly less than
`25`; equality is outside and accepts. Creative and spectator players are
excluded, while survival/adventure players can reject. Player visibility and
line of sight are not tested. A nonspawner candidate must also have a below
state valid for Endermite support.

All `66` locked biome records omit Endermite from every spawn category, and no
locked structure or spawner data selects it. Thus baseline natural/chunk-
generation spawning never calls this placement for Endermite. A custom
reloadable biome row activates it through the generic MONSTER cap `70`,
nonpersistent category, no-despawn/despawn distances `32/128`, inherited
cluster limit `4`, pack walk and insertion.

### Player Ender-Pearl ingress

A thrown Ender Pearl supplies the baseline spontaneous source, independently
of `SpawnPlacements`. Its common hit hook first creates `32` Portal particles;
each consumes one `nextDouble` for Y and two Gaussian draws for X/Z velocity.
Only on a live server projectile whose allowed owner is a `ServerPlayer` with
a connection accepting messages does the Endermite branch run.

It draws `nextFloat` and continues only when the value is strictly below
`0.05`. The draw occurs before checking `ServerLevel.isSpawningMonsters`,
which requires live game rules `spawn_mobs` and `spawn_monsters`, and before
rejecting Peaceful difficulty. When all pass it:

1. calls `EntityTypes.ENDERMITE.create(level, TRIGGERED)` without the
   registered placement predicate or Mob spawn finalization;
2. if construction succeeds, snaps the Endermite to the owner's current
   position and yaw/pitch, not the impact point; and
3. calls `addFreshEntity`, ignoring its Boolean result.

The owner has not yet been teleported, so this is its pre-teleport pose.
Insertion failure does not roll back or suppress the later Pearl transaction,
and later teleport failure does not remove an already admitted Endermite.
Nonplayer Pearl owners never take the chance branch. New Pearl Endermites have
`life=0` and are not made persistent by this path.

The Endermite chance is after the 32 particle calls but before portal-cooldown
copy, teleport, fall reset, Player Pearl damage `5`, teleport sound and Pearl
discard. An invalid owner or disallowed teleport discards the Pearl before the
chance draw.

### Sounds, loot, tags and progression

Ambient, death, hurt and step select sound protocol IDs `600/601/602/603`.
Step playback uses volume `0.15`, pitch `1`; other voice volume/pitch and
admission remain generic. English subtitles are `Endermite scuttles`,
`Endermite dies` and `Endermite hurts`; Step has no distinct Endermite
subtitle. Parrot's hostile-imitation map joins the type to
`entity.parrot.imitate.endermite`, sound ID `1223`, subtitle
`Parrot scuttles`; Parrot proximity/chance/pitch retain their owner.

The entity loot table has type `entity`, random sequence
`minecraft:entities/endermite`, and no pools. Eligible death yields zero item
entries and generic player/recent-hit gates can emit XP reward `3`.

Endermite directly belongs to exactly two entity-type tags:

- `arthropod`, alongside Bee, Silverfish, Spider and Cave Spider, selecting
  live data-driven arthropod enchantment effects; and
- `powder_snow_walkable_mobs`, alongside Rabbit, Silverfish and Fox, allowing
  the generic Powder Snow walk-on branch and composing with its priority-1
  climb goal.

The exact type is a named criterion in both `adventure/kill_a_mob` and
`adventure/kill_all_mobs`: the former places it in the one-of-many requirement,
while the latter gives it its own required group. Generic player-kill trigger
and advancement requirement/reward transactions retain their owners.

Common Endermite Spawn Egg is raw item ID `1245`, stack size `64`, with
`entity_data.id=minecraft:endermite`. Generic use-on, spawner, component
patch, naming, construction and insertion behavior retain the spawn-egg owner.

Exact UTF scanning of all `1,212` structure templates finds zero
`minecraft:endermite` occurrence.

### Legacy migration

Nine exact fix/schema classes select the identity:

- `EntityHealthFix` recognizes legacy `Endermite`;
- `EntityIdFix` maps `Endermite` to `minecraft:endermite`;
- `EntityUUIDFix` includes the modern ID in Mob UUID migration;
- `ItemSpawnEggFix` maps legacy generic Spawn Egg damage `67` to
  `Endermite`;
- `ItemStackSpawnEggFix` maps `minecraft:endermite` to
  `minecraft:endermite_spawn_egg`;
- `StatsCounterFix` maps old Endermite statistics to the modern ID;
- schema `V99` registers the legacy simple entity; and
- schemas `V705` and `V1460` register the modern Mob shape, with V705 also
  joining the Spawn Egg to its entity.

No fix rewrites `Lifetime`; its live missing-key/default and signed-int
behavior are authoritative.

### Client projection

`EntityRenderers` binds Endermite to `EndermiteRenderer`. It uses shadow radius
`0.3`, death flip `180` degrees, a plain Living render state and fixed texture
`textures/entity/endermite/endermite.png`. The texture is exact `64×32`, `505`
bytes, SHA-1 `fcd6c3419a60bbacb67a82a5e58bf8b608095120`.

`EndermiteModel` bakes a `64×32` atlas and four cuboid segments with sizes
`4×3×2`, `6×4×5`, `3×3×1` and `1×2×1`. For segment index `i=0..3`, let
`a=ageInTicks*0.9 + i*0.15*pi`. Each animation setup assigns:

`yRot = cos(a)*pi*0.01*(1+abs(i-2))`

and

`x = sin(a)*pi*0.1*abs(i-2)`.

English names are `Endermite` and `Endermite Spawn Egg`; the Egg uses the
generic spawn-egg item projection.

**Branches and aborts:**

- Persistence pauses Lifetime increment, but the `>=2400` check still runs.
- Client particles and server Lifetime are mutually exclusive sides after
  inherited AI.
- Spawner placement bypasses support and player search.
- Nonspawner placement has no light branch and excludes player distance
  strictly below, not equal to, five.
- Baseline biomes never select the registered placement.
- Pearl owner/teleport validity and connection precede the chance; chance
  precedes both rules and difficulty.
- Pearl creation bypasses placement/finalization and ignores insertion return.

**Constants and randomness:**

Entity/item IDs `42/1245`; dimensions/eye/passenger `0.4×0.3/0.13/0.2375`;
tracking/update `8/3`; health/speed/attack/follow `8/0.25/2/16`; metadata
`0..15 inherited`; Lifetime `0→2400`; XP `3`; goals
`1,1,2,3,7,8`, speeds `1/1`, look `8`; target priorities `1/2`; alert box
`16/10/16`; Enderman priority/default interval `3/10`; spawn player radius
`5`, strict square `25`; biome rows `0/66`; MONSTER cap/distances/cluster
`70/32,128/4`; Pearl `32×(double+2 Gaussian)` then `nextFloat<0.05`; client
particles `2×6` doubles/tick; sounds `600..603/1223`; tags/advancements
`2/2`; loot/template cells `0/0 of 1212`; migrations `9`; renderer/model/
texture `0.3/180/4 segments/64×32,505,fcd6c3419a60bbacb67a82a5e58bf8b608095120`.

**Side effects:**

Lifetime save/increment/discard; yaw coupling; navigation/look/attack/group
alert and Enderman targeting; movement events; local particles and sounds;
spawn candidate player/support reads; Pearl RNG/construction/snap/insertion;
generic damage/death/XP/advancements; synchronization and rendering.

**Gates:**

Logical side and persistence; Lifetime integer; goal controls/targets/
visibility; support/spawner/player mode and strict distance; reloadable biome
selection/caps/cluster; Pearl owner/connection/chance/rules/difficulty/create/
admission; death attribution, tags, loot, client assets.

**Boundary cases and quirks:**

Naming a Lifetime-2400 Endermite does not save it. Maximum signed Lifetime
wraps before comparison when nonpersistent. Placement is any-light yet has no
baseline biome selector. The Pearl source bypasses both support and the
five-block player exclusion and places at the thrower rather than impact.
All Endermen target all Endermites because no player-origin flag remains.

**Failure semantics:**

Lifetime and generic despawn removals discard without drops. Placement failure
prevents caller construction/insertion. Pearl entity creation failure skips
only snap/insertion; admission failure is ignored and does not abort teleport.
Generic goal, damage, death, advancement and Spawn Egg owners retain their
commit rules.

**Client/server authority split:**

The server owns Lifetime, persistence, yaw, AI, targets, navigation, attacks,
placement, Pearl creation, damage, death and XP. It synchronizes only inherited
metadata and transforms. The client owns two visual Portal particles per AI
tick and projects sounds, segmented animation, texture and names.

**Observability:**

Observe registration/attributes, inherited metadata, exact Lifetime NBT and
AI-step order, selector priorities/cadence/alerts, body yaw, player/Enderman
targets, placement support/player equality and zero biome census, Pearl RNG/
rule/create/admission/teleport order, tags/loot/XP/criteria, fixes/templates,
packets and exact particle/sound/model/texture/item projection.

**Persistence and reload:**

Generic entity state plus signed int `Lifetime` persists; client particle
timing does not. Entity registration, attributes, metadata, goals, Pearl
caller and migrations are code-built. Biomes, tags, loot, advancements and
Spawn Egg components reload through their owners; language/texture are client
resources.

**Evidence:**

`net.minecraft.world.entity.EntityTypes`;
`net.minecraft.world.entity.ai.attributes.DefaultAttributes`;
`net.minecraft.world.entity.SpawnPlacements`;
`net.minecraft.world.entity.Mob`;
`net.minecraft.world.entity.monster.Monster`;
`net.minecraft.world.entity.monster.Endermite`;
`net.minecraft.world.entity.monster.EnderMan`;
`net.minecraft.world.entity.projectile.throwableitemprojectile.ThrownEnderpearl`;
`net.minecraft.world.level.EntityGetter`;
`net.minecraft.client.renderer.entity.EntityRenderers`;
`net.minecraft.client.renderer.entity.EndermiteRenderer`;
`net.minecraft.client.model.geom.LayerDefinitions`;
`net.minecraft.client.model.monster.endermite.EndermiteModel`; six fix and
three schema classes; reports, two tags, empty loot, two advancements, all 66
biomes, all `1,212` templates, Spawn Egg components, texture and language.
Complete compiled/data/fix/NBT identity searches find no other exact runtime
path.

**Test vectors:**

Run `EXP-ENT-009` across Lifetime/persistence/signed boundaries, yaw, all goals
and targets including Enderman, combat/tags/death, any-light placement/player
radius under injected biome selection, all Pearl gates and outcomes, baseline
biome/template absence, migrations, Spawn Egg and exact client projection.

**Limits:**

Generic entity lifecycle, navigation, combat, effects, death, natural spawn,
despawn, Ender Pearl teleport, Spawn Egg, metadata packet and rendering retain
their owners. Tag consumers, Enderman behavior and advancement transactions
retain their leaves. This leaf fixes exact Endermite and every direct join
selecting it.
