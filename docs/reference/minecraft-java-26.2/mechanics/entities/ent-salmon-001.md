# Entities mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ENT-SALMON-001` — Salmon randomize three synchronized sizes before forming five-member schools

**Parent:** `ENT-001`, `ENT-LIFECYCLE-001`, `ENT-002`,
`ENT-VEHICLE-001`, `ENT-004`, `ENT-PROJECTILE-001`, `ENT-005`,
`ENT-DAMAGE-001`, `ENT-BLOCK-001`, `ENT-DAMAGE-REDUCE-001`,
`ENT-KNOCKBACK-001`, `ENT-006`, `ENT-EFFECT-001`, `ENT-007`,
`ENT-DEATH-001`, `MOB-001`, `MOB-AI-001`, `MOB-002`,
`MOB-SPAWN-001`, `MOB-003`, `MOB-DESPAWN-001`, `MOB-005`,
`ITM-MOB-BUCKET-001`, `ITM-SALMON-001`, `ITM-ENCHANT-001`,
`PLY-AUTOJUMP-001`, `WGEN-005`, `WGEN-PORTAL-001`, `CLI-001`,
`CLI-006`, `CLI-EFFECT-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked registration, the complete `Salmon` variant and
component paths, schooling/fish/water superclasses, placement and pack paths,
all 66 biomes, five direct tags, loot, Spawn Egg, eight migration/schema
contexts, all 1,212 templates and exact three-scale client resources close
protocol entity ID `110`.

**Applies when:**

`minecraft:salmon` is constructed, finalized, naturally selected, spawned by
a bucket, Egg, spawner or command, loaded, resized, schooling, swimming,
flopping, drying, captured, released, targeted by an Axolotl, killed,
synchronized or rendered.

**Authoritative state:**

Protocol entity ID `110` constructs `Salmon` in `WATER_AMBIENT`.
Registration fixes medium width/height `0.7×0.4`, eye height `0.26`, tracking
range `4` and default update interval `3`. Salmon is allowed in Peaceful.
Attributes are maximum health `3`, inherited movement speed `0.7` and follow
range `16`; eligible death supplies XP `1+nextInt(3)`.

`AbstractFish` supplies BOOLEAN metadata slot `16`, serializer ID `8`,
default false, and the always-written/default-false `FromBucket` persistence.
Salmon adds INT slot `17`, serializer ID `1`, default `1`, for size:

| serialized variant | integer ID | dimension scale | width/height | eye |
|---|---:|---:|---|---:|
| `small` | `0` | `0.5` | `0.35×0.2` | `0.13` |
| `medium` | `1` | `1` | `0.7×0.4` | `0.26` |
| `large` | `2` | `1.5` | `1.05×0.6` | `0.39` |

The variant's ID lookup clamps: any negative slot value reads as small and
any value above two reads as large. `getDefaultDimensions` scales width,
height, eye height and attachments by that clamped factor. Construction
explicitly refreshes dimensions after superclass initialization. Every
slot-17 update invokes superclass handling first and then refreshes
dimensions on both logical sides.

Save always stores the current clamped enum under lowercase `type` through
its string codec. Load reads the same codec and uses medium when the key is
missing, wrong-type or an unknown string, then updates slot `17` and
dimensions. The enum also has an ID-clamping stream codec for its independent
`minecraft:salmon/size` data component.

School `leader` and `schoolSize` remain transient and unsynchronized. Every
fresh or loaded Salmon begins with no leader and size one, so save/load
dissolves schools without changing the persisted body-size variant.
`FromBucket=true`, generic persistence or a custom name suppress ordinary
distance removal as in `ENT-COD-001`; otherwise Water-Ambient thresholds are
`32/64`. Salmon cannot be leashed and is not pushed by fluid.

**Transition and ordering:**

### Goals, movement, flop and air

Salmon inherits the exact fish goal graph:

- priority `0`, panic at speed `1.25`;
- priority `2`, avoid non-spectator Players within `8`, speeds `1.6/1.4`;
- priority `4`, interval-`40` random swimming at speed `1`, only while not a
  live follower; and
- priority `5`, follow a flock leader.

It has no target, attack, temptation or breeding goal. Water-Bound
navigation, movement control, travel, flop and air handling are identical to
Cod's audited kernel: Water-tag eye fluid first adds Y `0.005`; active
movement lerps speed by `0.125`, adds normalized vertical steering at factor
`0.1` and turns by at most `90` degrees. Water travel applies input `0.01`,
moves, scales velocity by `0.9` and sinks Y `0.005` only without a target.

Outside water while on ground and vertically colliding, AI step consumes two
floats for X/Z `(2f-1)*0.05`, adds Y
`0.4000000059604645`, clears on-ground state, requests impulse sync and
plays Salmon flop. Server air captures the pre-super value, decrements while
alive and dry, and at `-20` resets to zero before offering `2` Drown damage;
water or death resets it to `300`. Fish step sound is a no-op.

### Five-member exact-class schools

Salmon overrides maximum school size to `5`. A live non-null leader marks a
follower; leader counters increment/decrement on start/stop, following
continues within squared distance `121`, and path requests use speed `1`.
The exact-class query admits all Salmon variants, so one school can mix
small, medium and large members.

Follower addition limits its stream to `5-schoolSize` before filtering out
the leader itself, retaining the capacity-loss quirk. A leader above size
one consumes `nextInt(200)` every tick and only result `1` queries Salmon in
its box inflated by `8`; a list of at most one repairs size to one, while any
unrelated nearby Salmon suppresses repair.

The follow goal has nominal start countdown `200..219`, refuses leaders that
already have followers, admits current live followers immediately and
otherwise selects exact-class nonfollowers/partial leaders after countdown.
Continuation needs a live leader within squared distance `121`; path timer
starts at zero and resets to adjusted `10`. Stopping decrements even a dead
leader through the retained reference. It declares no control flags.

### Variant-first spawn finalization

Every Salmon finalization first builds weights small `30`, medium `50`,
large `15`, total `95`, and selects exactly one variant with the entity RNG.
Probabilities are therefore `30/95`, `50/95` and `15/95`. Selection writes
slot `17` and refreshes collision dimensions before any superclass
finalization.

It then enters schooling finalization, which first performs generic Mob
finalization: when absent, the permanent `random_spawn_bonus` follow-range
modifier gets a triangular amount centered at zero with deviation
`0.11485000000000001`; the later float below `0.05` makes the Salmon
left-handed. Finally null group data makes this Salmon the school leader, or
typed existing school data attaches it to the stored leader. Wrong non-null
data throws after variant selection and generic side effects.

Natural spawning threads group data through the pack, but every member rolls
its variant independently before joining. Requested groups `1..5` fit the
school/cluster maximum `5`.

### Placement and baseline natural selection

Salmon registers `IN_WATER` with heightmap
`MOTION_BLOCKING_NO_LEAVES` and the same surface-water predicate as Cod. The
placement-type gate requires a non-null type, world-border inclusion,
Water-tag candidate fluid and nonconducting block above. Species admission,
without RNG, requires:

1. Y in inclusive `seaLevel-13..seaLevel`;
2. Water-tag fluid below; and
3. exact `Blocks.WATER` above.

Spawn obstruction later requires the entity to be unobstructed. There is no
light or difficulty predicate.

Exactly six of 66 locked biomes select Salmon in `water_ambient`:

- Cold Ocean, Deep Cold Ocean, Frozen Ocean and Deep Frozen Ocean use weight
  `15`, group `1..5`; and
- River and Frozen River use weight `5`, group `1..5`.

The category is friendly/nonpersistent, cap `20`, distances `32/64`.
Generic selection, pack attempts, insertion and cap accounting retain
`MOB-SPAWN-001`.

### Bucket component override

Capture uses exact Water Bucket and an alive Salmon. After common bucket
fields, `saveToBucketTag` copies the entity's implicit
`minecraft:salmon/size` component into raw item ID `1048`
(`salmon_bucket`). Thus every captured bucket carries canonical small,
medium or large even though a default componentless Salmon Bucket has no
such component.

Release constructs a Salmon with reason `BUCKET`, consumes the weighted
variant choice, performs generic/school finalization as a standalone leader,
then applies stack configuration. A captured size component calls the
implicit-component setter, replacing that random choice and refreshing
dimensions; a componentless bucket retains the roll. Common bucket payload
loads afterward, then `FromBucket` becomes true before insertion and ambient
invocation. School links never round-trip.

`AbstractFish.mobInteract` also retains the eager `Optional.orElse` quirk:
generic Mob interaction evaluates to `PASS` even after successful capture
has replaced the hand and discarded Salmon. Exact capture/release sounds,
hand/inventory changes, criteria, payload, insertion failure and event order
remain with `ITM-MOB-BUCKET-001`.

### Loot, tags and sounds

The entity loot table has type `entity`, sequence
`minecraft:entities/salmon`. Its first independent pool emits one raw Salmon,
raw item ID `1087`; `furnace_smelt` uses the live recipe only when this
Salmon is on fire or the direct attacker's main hand matches
`#minecraft:smelts_loot`. The second independent pool emits one Bone Meal
with probability `0.05`. Looting changes neither count. Eligible death
separately emits XP `1..3`.

Salmon belongs directly to exactly five entity-type tags:

- `aquatic`, transitively selecting `sensitive_to_impaling`;
- `axolotl_hunt_targets`, allowing a hunt-ready Axolotl to select a visible,
  attackable, in-water Salmon within squared distance `64`;
- `can_breathe_under_water`;
- `cannot_be_pushed_onto_boats`, preventing collision auto-mount but not the
  physical push; and
- `not_scary_for_pufferfish`, excluding it from the scary-Mob predicate.

Advancements containing the Salmon string select raw/cooked/bucket items,
not the exact entity type. Common Salmon Spawn Egg is raw item ID `1187`,
stack `64`, with `entity_data.id=minecraft:salmon`; generic Egg construction,
component application, finalization and insertion retain their owner.

Ambient, death, flop and hurt use protocol IDs `1392..1395`; fish swim is ID
`638`. Ambient has an empty sound list and no subtitle, so the registered
event has no audible clip. Death and hurt each use the four fish-hurt clips
at resource pitch `0.8`; flop uses the four fish-flop clips at resource
pitch/volume `0.8/0.3`. English subtitles are `Salmon dies`, `Salmon hurts`,
`Salmon flops` and `Splashes`. Generic voice admission, range, volume and
runtime pitch retain their owners.

Exact UTF scanning finds zero `minecraft:salmon` occurrence in all `1,212`
structure templates.

### Legacy migration

Eight exact schema/fix contexts own Salmon entity or bucket compatibility:

- `V1470` registers legacy `minecraft:salmon_mob`;
- `V1486` moves its schema supplier to `minecraft:salmon`;
- `EntityCodSalmonFix` renames both legacy entity and Spawn Egg IDs;
- `EntityUUIDFix` includes current Salmon in Mob UUID conversion;
- `V705` maps the current Salmon Spawn Egg to its entity shape;
- `ItemStackComponentizationFix` moves legacy bucket-mob fields into
  `minecraft:bucket_entity_data`;
- `EntitySpawnerItemVariantComponentFix` removes legacy lowercase `type`
  from Salmon-Bucket entity data and writes it as
  `minecraft:salmon/size`; and
- `EntitySalmonSizeFix` preserves exact legacy `large` but rewrites every
  other/missing `type` value to `medium`.

The generic bucket componentization can move irrelevant legacy `Age`,
`Variant`, `HuntingCooldown` and `BucketVariantTag`; live Salmon ignores
those common-payload entries. No fix creates persistent school state.

### Three-scale client projection

`EntityRenderers` binds Salmon to `SalmonRenderer`, fixed shadow `0.4` and
one shared `textures/entity/fish/salmon.png`. Render-state extraction copies
the clamped server variant; a fresh render state defaults to medium.
Submission switches exhaustively among separately baked small, medium and
large models.

The base `32×32` mesh contains eight named parts: two `3×5×8` body halves,
a `2×4×3` head, zero-thickness `0×5×6` back fin, `0×2×3` front-top fin,
`0×2×4` back-top fin, and two `2×0×2` side fins rotated
`±0.7853982`. Medium uses the base layer. Small and large transform every
part by `0.5` and `1.5`, then translate Y by respectively `12.008` and
`-12.008` (`24.016*(1-scale)`) to retain floor alignment.

In water, renderer body yaw is `4.3*sin(0.6*age)` degrees and rear-body tail
yaw is `-0.25*sin(0.6*age)`. Out of water both use amplitude factor `1.3`
and frequency factor `1.7`, producing body
`5.59*sin(1.02*age)` and tail `-0.325*sin(1.02*age)`. The renderer then
translates `(0.2,0.1,0)` and rotates `90` degrees about positive Z.

The shared texture is `32×32`, `485` bytes, SHA-256
`de7105cfa87d6845196a3f424e3c3aa811408fb7e2ca806c2d3c583b29b8d5b4`.
The generated Spawn-Egg texture is `16×16`, `268` bytes, SHA-256
`808f0b3f83e645f54d6b484afaf06d143b457ed69176a039ce2b25c89af1c9b3`.
English names are `Salmon`, `Salmon Spawn Egg` and `Bucket of Salmon`;
Salmon size adds no bucket tooltip line.

**Branches and aborts:**

- Slot-17 integers clamp for behavior and persistence; NBT codec failure
  instead selects medium.
- Variant selection and dimension refresh precede generic/school
  finalization and any typed-group failure.
- Every pack member rolls size independently; the school admits all sizes.
- Captured bucket size overrides the release roll after finalization;
  componentless buckets preserve it.
- Ambient invocation selects a registered event with no clip.

**Constants and randomness:**

Entity/Egg/bucket/raw-item IDs `110/1187/1048/1087`; base dimensions/eye
`0.7×0.4/0.26`; variant IDs/scales `0/1/2` and `0.5/1/1.5`; tracking/update
`4/3`; health/speed/follow `3/0.7/16`; metadata `16 BOOLEAN, 17 INT`;
variant weights/total `30/50/15/95`; goals `0/2/4/5`; school
`5/121/8/nextInt(200)==1`, start `200..219`, repath `10`; movement
`0.005/0.125/0.01/0.9`; flop `±0.05/0.4000000059604645`; air
`300/-20/2`; spawn depth `13`, rows `6/66`, group `1..5`, category
`20/32/64`; Bone Meal `0.05`, XP `1..3`; sounds `1392..1395/638`;
tags/templates/migrations `5/0 of 1212/8`; model scales/Y offsets
`0.5/1/1.5` and `12.008/0/-12.008`; shadow `0.4`.

**Side effects:**

Variant, dimensions, `FromBucket`, common durable state and metadata; school
counters/paths; RNG cursor, motion, impulse and air; sound, damage, loot and
XP; cap/pack state; bucket hand/discard/component/insertion; tag-selected
Axolotl, Pufferfish, boat and Impaling behavior; client model selection and
animation.

**Gates:**

Variant integer/string/component validity; logical side and water/ground/
collision/air; leader state/class/distance/countdown; group type, modifier
presence and RNG; border/Y/fluid/block/biome/cap; bucket/aliveness/component;
death fire/attacker/enchantment/chance; tags and client water state.

**Boundary cases and quirks:**

Arbitrary metadata clamps rather than wrapping. Legacy size migration turns
even exact `small` into medium while preserving only large. A school can mix
collision sizes but caps at five. Stream self-order and unrelated-neighbor
stale repair remain observable. Wrong group data fails only after size and
generic randomization. Captured buckets erase the release size roll's result,
whereas newly created componentless buckets expose it. Ambient is silent.

**Failure semantics:**

Rejected placement prevents natural insertion. Failed insertion does not
roll back finalization or component application under its generic owner.
Invalid NBT defaults medium; out-of-range metadata clamps. Rejected Drown
damage does not undo air reset. Loot, XP, Egg and bucket owners retain their
commit boundaries.

**Client/server authority split:**

The server owns canonical variant, dimensions, school links, AI, placement,
finalization, bucket transfer, damage, loot and XP. Slots `16/17` synchronize
bucket origin and integer size; school state does not cross the wire. The
client refreshes dimensions on slot `17`, selects the corresponding baked
mesh and applies water-dependent body/tail transforms.

**Observability:**

Observe slots `16/17`, clamping, `type` codec and dimension refresh; variant
RNG before generic/school draws; mixed-size five-member topology; movement,
flop and air; six-biome selection/group/cap; capture/release component order;
loot/XP/tags/Egg; silent versus pitched sounds; zero-template and eight-fix
closure; three layer scales, floor offsets, texture and names.

**Persistence and reload:**

Generic Mob state, `FromBucket` and canonical lowercase `type` persist;
schools, paths and counters do not. Code fixes registration, variant,
component, goals, placement and schemas. Biomes, tags, loot and recipes
reload through their owners; sounds, language, layers and texture are client
resources.

**Evidence:**

`EntityTypes`, `DefaultAttributes`, `SpawnPlacements`,
`SpawnPlacementTypes`, `MobCategory`, `Mob`, `NaturalSpawner`;
`net.minecraft.world.entity.animal.fish.{WaterAnimal,AbstractFish,AbstractSchoolingFish,Salmon}`;
`FollowFlockLeaderGoal`, `Bucketable`, `MobBucketItem`, `DataComponents`,
`SoundEvents`; Axolotl, Pufferfish and AbstractBoat consumers; client
`EntityRenderers`, `SalmonRenderer`, `SalmonRenderState`, `SalmonModel`,
`LayerDefinitions`, `MeshTransformer`; the eight migration/schema classes
named above; reports, six biomes, five tags, loot, all 1,212 structures,
sounds, language, models and textures. Complete compiled/data identity
searches find no other exact entity runtime path.

**Test vectors:**

Run `EXP-ENT-016` across arbitrary slot-17 and NBT/component variants,
dimension refresh, variant-first group finalization, mixed-size school and
stale-repair cases, movement/flop/air, six-biome placement/groups/caps,
componentful/componentless capture-release ordering, loot/XP/tags/Egg,
templates/migrations/sounds and all three client layers/transforms/resources.

**Limits:**

Generic entity lifecycle, navigation, damage/death, natural spawning,
despawn, loot, Spawn Egg, bucket transaction, metadata/component packets and
render submission retain their owners. Raw/Cooked Salmon and Salmon Bucket
item behavior retain their item leaves. This leaf fixes exact Salmon entity
dispatch and every direct join selecting it.
