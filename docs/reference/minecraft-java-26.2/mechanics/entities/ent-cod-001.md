# Entities mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ENT-COD-001` — Cod school through transient leader links and become persistent after bucket release

**Parent:** `ENT-001`, `ENT-LIFECYCLE-001`, `ENT-002`,
`ENT-VEHICLE-001`, `ENT-004`, `ENT-PROJECTILE-001`, `ENT-005`,
`ENT-DAMAGE-001`, `ENT-BLOCK-001`, `ENT-DAMAGE-REDUCE-001`,
`ENT-KNOCKBACK-001`, `ENT-006`, `ENT-EFFECT-001`, `ENT-007`,
`ENT-DEATH-001`, `MOB-001`, `MOB-AI-001`, `MOB-002`,
`MOB-SPAWN-001`, `MOB-003`, `MOB-DESPAWN-001`, `MOB-005`,
`ITM-MOB-BUCKET-001`, `ITM-COD-001`, `ITM-ENCHANT-001`,
`PLY-AUTOJUMP-001`, `WGEN-005`, `WGEN-PORTAL-001`, `CLI-001`,
`CLI-006`, `CLI-EFFECT-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked registration, the complete `Cod`,
`AbstractSchoolingFish`, `AbstractFish` and `WaterAnimal` paths, placement,
natural-pack and bucket paths, all 66 biomes, five direct tags, loot, Spawn
Egg, six migration/schema contexts, all 1,212 templates and exact client
resources close protocol entity ID `27`. Salmon and Tropical Fish retain
separate leaves because they add independent variant state.

**Applies when:**

`minecraft:cod` is constructed, finalized, naturally selected, spawned by a
bucket, Egg, spawner or command, loaded, schooling, swimming, flopping,
drying, captured, released, targeted by an Axolotl, killed, synchronized or
rendered.

**Authoritative state:**

Protocol entity ID `27` constructs `Cod` in `WATER_AMBIENT`. Registration
fixes width/height `0.5×0.3`, eye height `0.195`, tracking range `4` and the
builder-default update interval `3`. Cod is allowed in Peaceful. Its
attributes are maximum health `3`, inherited movement speed `0.7` and follow
range `16`; eligible death supplies XP `1+nextInt(3)`.

`AbstractFish` adds synchronized BOOLEAN metadata slot `16`, serializer ID
`8`, default `false`, for `fromBucket`. Cod adds no slot. Save always writes
the Boolean `FromBucket`; load uses `getBooleanOr("FromBucket",false)`, so a
missing or wrong-type value becomes false. Values need no normalization.

Schooling state is not synchronized or saved. Every fresh or loaded Cod has
`leader=null` and `schoolSize=1`; save/load therefore dissolves every
existing school. Movement targets, goal counters and navigation paths are
also transient.

Bucket origin extends generic persistence: `requiresCustomPersistence` is
true when generic Mob persistence applies or `fromBucket` is true, and
`removeWhenFarAway` is false for a bucket-origin or custom-named Cod.
Otherwise Water-Ambient despawn uses no-despawn distance `32`, hard distance
`64`, `noActionTime>600` and the generic one-in-800 random check. Cod cannot
be leashed and is not pushed by fluid.

**Transition and ordering:**

### Goal graph and fish movement

Cod registers no target, attack, temptation or breeding goal. Its exact goal
selector is:

- priority `0`, panic at speed `1.25`;
- priority `2`, avoid non-spectator Players within `8`, with walk/sprint
  speeds `1.6/1.4`;
- priority `4`, random swimming at speed `1`, interval `40`, admitted only
  when the Cod is not a live follower; and
- priority `5`, follow a flock leader.

Navigation is `WaterBoundPathNavigation`. On each movement-control tick, an
eye position in Water-tag fluid first adds `(0,0.005,0)` velocity. If the
operation is not `MOVE_TO`, or navigation is done, speed becomes zero.
Otherwise desired speed is `speedModifier*MOVEMENT_SPEED`, current speed
lerps toward it by `0.125`, and nonzero vertical distance adds
`speed*(dy/distance)*0.1`. Nonzero X or Z turns toward
`atan2(dz,dx)*57.2957763671875-90`, capped by `90` degrees, and copies that
yaw to body rotation.

Water travel applies relative input at `0.01`, moves `SELF`, multiplies all
velocity by `0.9`, then adds `(0,-0.005,0)` only when there is no target.
Fish step sound is a no-op. The generic fish-swim sound retains ordinary
movement-event admission.

### School construction, following and stale repair

`startFollowing(leader)` stores the reference and increments the leader's
`schoolSize`; `stopFollowing` decrements that referenced leader and clears
the reference. A Cod is a follower only while its non-null leader is alive.
A leader has followers at size above one, can be followed only at sizes
`2..7`, and has maximum school size `8`. Following remains valid within
squared distance `121`; path requests use speed `1`.

`addFollowers` limits the candidate stream to `8-schoolSize` before filtering
out `this`. Encountering the leader inside that limited prefix can therefore
leave one capacity unused. Every leader with `schoolSize>1` consumes
`nextInt(200)` each tick; only result `1` queries exact runtime-class
neighbors in its axis-aligned box inflated by `8`. A result list of size at
most one resets `schoolSize` to one. Any nearby Cod, even one from an
unrelated school, prevents that repair.

The follow goal initializes its start countdown to reduced
`200+(nextInt(200)%20)`, nominally `200..219`. A leader with followers cannot
start; a live follower starts immediately. Otherwise the countdown must
expire, after which exact-class Cod within the inflated-eight box are
filtered by `canBeFollowed() || !isFollower()`. `findAny` among nonfollowers,
or self when none exists, becomes the leader whose `addFollowers` consumes
the same candidate stream. Existing partial leaders and lone fish can both
be chosen.

Continuation requires a live follower within squared distance `121`.
Following starts with path timer zero; predecrement to zero requests a new
path and resets the adjusted interval to `10`. Stop decrements the retained
leader reference. A leader dying therefore makes the follower fail
continuation before stop repairs the dead leader's counter. The goal declares
no control flags; random swimming cannot newly start for a live follower
because its own predicate fails.

### Spawn finalization and natural pack sharing

Schooling finalization first runs generic Mob finalization. If absent, the
permanent `random_spawn_bonus` follow-range modifier receives a triangular
amount centered at zero with deviation `0.11485000000000001`; a subsequent
float below `0.05` makes the Cod left-handed. The generic method's returned
group object is ignored.

A null supplied group becomes `SchoolSpawnGroupData(this)`, making the first
Cod leader. A non-null value is cast to that exact type and the new Cod
follows its stored leader. A wrong non-null type therefore throws only after
generic modifier/handedness side effects. Natural spawning threads the
returned object through its requested pack, while Bucket and Spawn-Egg
creation with null data produce standalone leaders. Requested natural groups
are `3..6`, below cluster maximum `8`.

### Flopping and air

Before inherited AI step, a Cod that is outside water, on ground and
vertically colliding adds:

- X `(nextFloat*2-1)*0.05`;
- Y exactly `0.4000000059604645`; and
- Z `(nextFloat*2-1)*0.05`.

It then clears on-ground state, requests impulse synchronization and plays
Cod flop. Failure of any gate spends neither float and performs none of
those changes.

On a server, the Water-Animal air handler captures pre-super air. While alive
and outside water it writes that value minus one; at `-20` or below it resets
air to zero and offers `2` Drown damage, ignoring the result. While dead or
in water it writes `300`. Rejected damage therefore does not prevent the air
reset or later pulses. The direct `can_breathe_under_water` membership also
skips the generic underwater-drowning path.

### Placement and baseline natural selection

Cod registers placement `IN_WATER` with heightmap
`MOTION_BLOCKING_NO_LEAVES`. The placement-type gate requires a non-null
type, world-border inclusion, Water-tag fluid at the candidate and a block
above that is not a redstone conductor. The species predicate then requires,
without RNG:

1. Y in the inclusive interval `seaLevel-13..seaLevel`;
2. Water-tag fluid below the candidate; and
3. the block above to be exactly `Blocks.WATER`.

Spawn obstruction later requires `level.isUnobstructed(cod)`. There is no
light or difficulty predicate.

Exactly six of 66 locked biomes select Cod, all in `water_ambient`:

- Cold Ocean and Deep Cold Ocean: weight `15`, group `3..6`;
- Deep Lukewarm Ocean: weight `8`, group `3..6`;
- Deep Ocean: weight `10`, group `3..6`;
- Lukewarm Ocean: weight `15`, group `3..6`; and
- Ocean: weight `10`, group `3..6`.

The category cap is `20`, friendly true, nonpersistent, with distances
`32/64`. Generic candidate selection, pack attempts, insertion and cap
accounting retain `MOB-SPAWN-001`.

### Bucket round trip

`AbstractFish.mobInteract` first offers the exact bucket pickup transaction
from `ITM-MOB-BUCKET-001`. Its `Optional.orElse` eagerly evaluates generic
Mob interaction even after successful capture; that superclass path returns
`PASS`, after the capture has already mutated the hand and discarded Cod.

Only an alive Cod and exact Water Bucket admit capture. The transaction plays
Fish-Bucket-Fill, creates raw item ID `1049` (`cod_bucket`), stores common
bucket entity data and replaces the held stack before discarding Cod. Cod
adds no school or subtype payload. Common data can carry custom name,
`NoAI`, `Silent`, `NoGravity`, `Glowing`, `Invulnerable`,
`PersistenceRequired` and always `Health`.

Release constructs Cod with reason `BUCKET`, finalizes it first as a
standalone school leader, applies default stack configuration, loads common
bucket data, sets `fromBucket=true`, inserts it and invokes ambient sound.
Generic finalization can therefore add the follow-range modifier and
left-handed state before saved bucket fields load. School relations never
round-trip. Exact hand replacement, creative handling, criteria, insertion
failure and sound ordering remain wholly owned by `ITM-MOB-BUCKET-001`.

### Loot, tags, sounds and item projection

The Cod entity loot table has type `entity` and sequence
`minecraft:entities/cod`. Its first independent one-roll pool emits one raw
Cod, raw item ID `1086`. `furnace_smelt` converts it through the live recipe
only when this Cod is on fire or the direct attacker's main-hand enchantments
match `#minecraft:smelts_loot`; exact conversion retains `ITM-COD-001`.
The second independent pool emits one Bone Meal with probability `0.05`.
Looting changes neither count. Eligible death can separately emit XP `1..3`.

Cod belongs directly to exactly five entity-type tags:

- `aquatic`, transitively selecting `sensitive_to_impaling`;
- `axolotl_hunt_targets`, allowing an Axolotl without hunting cooldown to
  select a visible, attackable, in-water Cod within squared distance `64`;
- `can_breathe_under_water`;
- `cannot_be_pushed_onto_boats`, preventing collision auto-mount while the
  physical push branch remains; and
- `not_scary_for_pufferfish`, excluding Cod from the scary-Mob predicate.

No locked advancement names the exact Cod entity. Common Cod Spawn Egg is
raw item ID `1181`, stack `64`, with `entity_data.id=minecraft:cod`; generic
Egg construction, naming, finalization and insertion retain their owner.

Ambient, death, flop and hurt use sound protocol IDs `379..382`; fish swim is
ID `638`. The locked ambient event deliberately has an empty sound list and
no subtitle, so invoking it emits no clip. Death and hurt each select the
same four fish-hurt clips with subtitles `Cod dies` and `Cod hurts`; flop
selects four clips, each resource volume `0.3`, with subtitle `Cod flops`.
Fish swim selects seven clips with subtitle `Splashes`. Generic voice
pitch/volume and sound admission retain their owners.

Exact UTF scanning of all `1,212` structure templates finds zero
`minecraft:cod` occurrence.

### Legacy schema and client projection

Six exact migration/schema contexts contain Cod or its bucket:

- `V1470` registers the legacy simple entity `minecraft:cod_mob`;
- `V1486` moves its schema supplier to `minecraft:cod`;
- `EntityCodSalmonFix` maps `cod_mob` and its Spawn Egg to current IDs;
- `EntityUUIDFix` includes current Cod in Mob UUID migration;
- `V705` maps the current Cod Spawn Egg to its entity shape; and
- `ItemStackComponentizationFix` moves legacy bucket-mob fields into
  `minecraft:bucket_entity_data`.

That componentization list can also move `Age`, `Variant`,
`HuntingCooldown` and `BucketVariantTag`; live Cod ignores those fields. No
fix rewrites `FromBucket`, `leader` or `schoolSize`.

`EntityRenderers` binds Cod to `CodRenderer`, shadow `0.3`, texture
`textures/entity/fish/cod.png`. The renderer always yaws by
`4.3*sin(0.6*ageInTicks)` degrees after inherited rotations. Out of water it
then translates `(0.1,0.1,-0.1)` and rotates `90` degrees about positive Z.

The `32×32` model has body, head, nose, right fin, left fin, tail fin and top
fin root parts. Tail Y rotation is
`-0.45*sin(0.6*ageInTicks)` in water and
`-0.675*sin(0.6*ageInTicks)` out of water. The entity texture is `32×32`,
`243` bytes, SHA-256
`02eb65ffd0a9e1744222c094746e0b0a65f84ed0188ac3e4b78a2e37cd41788a`.
The generated Spawn-Egg texture is `16×16`, `216` bytes, SHA-256
`3331026416fa3d5460fae3a3c47d02ee87a181a461e8547dfb2c701c544f735d`.
English names are `Cod`, `Cod Spawn Egg` and `Bucket of Cod`.

**Branches and aborts:**

- A live leader suppresses random-swim admission; a dead leader is repaired
  only when the follow goal stops.
- School stream limiting occurs before self filtering.
- Generic spawn finalization precedes the school-data cast.
- Flopping requires all three dry/on-ground/vertical-collision gates.
- Capture succeeds before eager generic interaction evaluation.
- Release finalizes before loading bucket data and setting `FromBucket`.
- Ambient invocation is observable as a sound event without an audible clip.

**Constants and randomness:**

Entity/Egg/bucket/raw-item IDs `27/1181/1049/1086`; dimensions/eye
`0.5×0.3/0.195`; tracking/update `4/3`; health/speed/follow
`3/0.7/16`; metadata slot `16`; goals `0/2/4/5`; panic/avoid/random
`1.25/1.6,1.4/1`; school max/range/query/reset
`8/121/8/nextInt(200)==1`; follow start `200..219`, repath `10`; water
buoyancy/travel/sink `0.005/0.01,0.9/0.005`; flop X/Z `±0.05`, Y
`0.4000000059604645`; air `300/-20/2`; spawn depth `13`; biome rows
`6/66`, groups `3..6`; category `20/32/64`; Bone Meal `0.05`, XP
`1..3`; sounds `379..382/638`; tags/templates/migration contexts
`5/0 of 1212/6`; shadow `0.3`.

**Side effects:**

`FromBucket`, generic health/equipment/name persistence and metadata;
transient leader counters and paths; RNG cursor, movement and impulse
synchronization; sound, damage, loot and XP; category-cap and pack state;
bucket hand mutation/discard/insertion; tag-selected Axolotl, Pufferfish,
boat and Impaling behavior; client rotations/model/texture.

**Gates:**

Logical side, water/ground/collision and target state; leader liveness,
distance, exact runtime class and countdowns; spawn reason/group object and
RNG; border/Y/fluid/block/category cap; bucket item/aliveness; death
attacker/fire/enchantment/chance; tags and client water state.

**Boundary cases and quirks:**

School relations vanish on reload and are absent from bucket data. A nearby
unrelated Cod can preserve stale leader count. A self entry before stream
limit can waste school capacity. Wrong non-null spawn group data throws after
generic randomization. Bucket capture still evaluates generic interaction.
The registered ambient event is silent. Bucket-origin state permanently
suppresses ordinary distance removal until changed by an external mutation.

**Failure semantics:**

Rejected placement prevents natural insertion. Generic insertion failure
does not undo finalization or bucket-data application under the owning
transaction. Rejected Drown damage does not undo air reset. Ignored path
results preserve the leader relation. Loot, XP, Spawn Egg and bucket owners
retain their commit boundaries.

**Client/server authority split:**

The server owns school links, AI, movement targets, placement, finalization,
air/damage, bucket mutation, loot and XP. Slot `16` synchronizes bucket
origin; no school state crosses the wire. The client interpolates generic
living state, selects in/out-of-water tail and body transforms and renders
the exact Cod mesh/texture.

**Observability:**

Observe registration/attributes, slot `16` and `FromBucket`, save/load school
dissolution, every leader counter/countdown/path branch, movement and flop
formulas/RNG, air pulses, finalization order, six-biome census and pack/cap,
bucket payload/order, loot/XP, five tag consumers, silent ambient versus
other sounds, zero-template/migration closure and exact client projection.

**Persistence and reload:**

Generic Mob fields plus `FromBucket` persist; school references/counts,
movement targets and goal timers do not. Code fixes registration, goals,
placement and schema. Biomes, tags, loot, recipes and bucket components
reload through their owners; sounds, language, model and texture are client
resources.

**Evidence:**

`net.minecraft.world.entity.EntityTypes`;
`net.minecraft.world.entity.ai.attributes.DefaultAttributes`;
`net.minecraft.world.entity.SpawnPlacements`;
`net.minecraft.world.entity.SpawnPlacementTypes`;
`net.minecraft.world.entity.MobCategory`; `Mob`, `NaturalSpawner`,
`WaterAnimal`, `AbstractFish`, both Abstract-Fish inner classes,
`AbstractSchoolingFish`, its group data, `FollowFlockLeaderGoal`, `Cod`,
`Bucketable`, `MobBucketItem`; Axolotl, Pufferfish and AbstractBoat
consumers; `SoundEvents`; client `EntityRenderers`, `CodRenderer`,
`CodModel`, `LayerDefinitions`; migration/schema classes named above;
reports, six biomes, five tags, loot, all 1,212 structures, sounds, language,
model and textures. Complete compiled/data identity searches find no other
exact Cod runtime path.

**Test vectors:**

Run `EXP-ENT-015` across metadata/save/despawn, school construction,
leader death/stale repair/countdown/stream-order/path cases, all movement,
flop and air branches, finalization/group-data paths, exact placement,
biomes/groups/caps, complete capture/release payload ordering, loot/XP/tags,
Egg, templates/migrations/sounds and exact client water/model/texture/name
projection.

**Limits:**

Generic entity lifecycle, navigation, damage/death, natural spawning,
despawn, loot evaluation, Spawn Egg, bucket transaction, metadata packets
and rendering retain their owners. Item Cod and Cod Bucket behaviors retain
their item leaves. This leaf fixes exact Cod dispatch and every direct join
selecting the entity.
