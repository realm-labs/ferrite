# Entities mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ENT-PUFFERFISH-001` — Pufferfish inflate around scary living entities and poison successful contacts

**Parent:** `ENT-001`, `ENT-LIFECYCLE-001`, `ENT-002`,
`ENT-VEHICLE-001`, `ENT-004`, `ENT-PROJECTILE-001`, `ENT-005`,
`ENT-DAMAGE-001`, `ENT-BLOCK-001`, `ENT-DAMAGE-REDUCE-001`,
`ENT-KNOCKBACK-001`, `ENT-006`, `ENT-EFFECT-001`, `ENT-007`,
`ENT-DEATH-001`, `MOB-001`, `MOB-AI-001`, `MOB-002`,
`MOB-SPAWN-001`, `MOB-003`, `MOB-DESPAWN-001`, `MOB-005`,
`ITM-MOB-BUCKET-001`, `ITM-PUFFERFISH-001`, `ITM-ENCHANT-001`,
`PLY-AUTOJUMP-001`, `WGEN-005`, `WGEN-PORTAL-001`, `CLI-001`,
`CLI-006`, `CLI-EFFECT-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked registration, complete synchronized inflation and
sting paths, fish/water superclasses, placement and three biome rows, six
direct tags, loot, Spawn Egg, seven migration/schema contexts, all 1,212
templates and exact three-model client projection close protocol entity ID
`107`.

**Applies when:**

`minecraft:pufferfish` is constructed, finalized, naturally selected,
spawned by a bucket, Egg, spawner or command, loaded, inflated, deflated,
swimming, flopping, drying, colliding with a Mob or Player, captured,
released, targeted by an Axolotl or Nautilus, killed, synchronized or
rendered.

**Authoritative state:**

Protocol entity ID `107` constructs `Pufferfish` in `WATER_AMBIENT`.
Registration fixes full-state width/height `0.7×0.7`, eye height `0.455`,
tracking range `4` and default update interval `3`. Pufferfish are allowed in
Peaceful. Attributes are maximum health `3`, inherited movement speed `0.7`
and follow range `16`; eligible death supplies XP `1+nextInt(3)`.

`AbstractFish` supplies BOOLEAN metadata slot `16`, serializer ID `8`,
default false, and the always-written/default-false `FromBucket` persistence.
Pufferfish add INT slot `17`, serializer ID `1`, default `0`, for raw
`PUFF_STATE`. The intended states and dimensions are:

| state | scale | width/height | eye |
|---:|---:|---|---:|
| small `0` | `0.5` | `0.35×0.35` | `0.2275` |
| mid `1` | `0.7` | `0.49×0.49` | `0.3185` |
| full `2` | `1` | `0.7×0.7` | `0.455` |

Scaling also affects attachments. The constructor refreshes dimensions after
superclass initialization. A slot-17 update refreshes dimensions first, then
invokes superclass synchronized-data handling.

The integer is deliberately not clamped by its getter or setter. Dimension
selection uses `0→0.5`, `1→0.7`, and every other integer→`1`, so negative and
above-two metadata use full dimensions. Save always writes the raw integer
under `PuffState`. Load reads default zero and applies only
`min(saved,2)`: positive values above two become full, but every negative
value survives. There is no Puff-State implicit data component.

`inflateCounter` and `deflateTimer` are ordinary zero-initialized, transient,
unsynchronized integers. They never save, so a loaded positive state begins
deflating from timer zero when no threat is present. `FromBucket=true`,
generic persistence or a custom name suppress ordinary distance removal;
otherwise Water-Ambient thresholds are `32/64`. Pufferfish cannot be leashed
and are not pushed by fluid.

**Transition and ordering:**

### Goals, movement, flop and air

Pufferfish call the ordinary fish goal registration and then add their own
goal:

- priority `0`, panic at speed `1.25`;
- priority `1`, Pufferfish puff;
- priority `2`, avoid non-spectator Players within `8`, speeds `1.6/1.4`;
  and
- priority `4`, interval-`40` random swimming at speed `1`.

The puff goal sets no control flags, so it can run concurrently with movement
goals. It has no target, attack, temptation, schooling or breeding goal.

Water-Bound navigation, movement control, travel, flop and air handling are
identical to Cod's audited kernel. Water-tag eye fluid first adds Y `0.005`;
active movement lerps speed by `0.125`, adds normalized vertical steering at
factor `0.1` and turns by at most `90` degrees. Water travel applies input
`0.01`, moves, scales velocity by `0.9` and sinks Y `0.005` only without a
target.

Outside water while on ground and vertically colliding, AI step consumes two
floats for X/Z `(2f-1)*0.05`, adds Y `0.4000000059604645`, clears on-ground
state, requests impulse sync and plays Pufferfish flop. Server air captures
the pre-super value, decrements while alive and dry, and at `-20` resets to
zero before offering `2` Drown damage; water or death resets it to `300`.
Fish step sound is a no-op.

### Scary-entity admission

The priority-one goal scans `LivingEntity` instances whose bounding boxes
intersect the Pufferfish bounding box inflated by `2`. Because that box grows
with Puff State, its half-extents grow from `2.175` through `2.245` to
`2.35`. Admission uses noncombat targeting conditions with no configured
distance, invisibility attenuation disabled and line of sight disabled.
The remaining ordered gates reject:

1. the Pufferfish itself;
2. dead or spectator entities through `canBeSeenByAnyone`;
3. creative Players; and
4. every entity in `#minecraft:not_scary_for_pufferfish`.

Survival/adventure Players and invisible entities can therefore trigger
through solid blocks. The goal starts when the admitted list is nonempty and
its inherited continuation simply reruns the same scan. Start writes
`inflateCounter=1` and `deflateTimer=0`; stop writes only
`inflateCounter=0`.

The current direct safe-tag payload is Turtle, Guardian, Elder Guardian, Cod,
Pufferfish, Salmon, Tropical Fish, Dolphin, Squid, Glow Squid, Tadpole,
Nautilus, Zombie Nautilus and Sulfur Cube. Reloading the tag changes threat
admission without changing the goal algorithm.

### Inflation/deflation tick ordering

Pufferfish run their state transition block before `AbstractFish.tick`.
Consequently goal start/stop performed during that later superclass AI step
cannot affect the transition block until the next entity tick. The block
runs only server-side while alive and effective AI:

- If `inflateCounter>0`:
  - raw state `0` plays blow-up and becomes `1`;
  - otherwise, only `inflateCounter>40 && state==1` plays blow-up and becomes
    `2`; and
  - the counter increments afterward.
- Otherwise, if raw state is nonzero:
  - only `deflateTimer>60 && state==2` plays blow-out and becomes `1`;
  - otherwise, only `deflateTimer>100 && state==1` plays blow-out and becomes
    `0`; and
  - the timer increments afterward.

Thus a newly detected threat leaves state zero for the detection tick, enters
mid on the next transition block and enters full only when that block begins
with counter `41`. After stop, the first deflation block changes timer
`0→1`; full becomes mid when the block begins with `61`, and mid becomes
small when it begins with `101`.

Goal restart resets the deflate timer. A full state paired manually with a
timer above 100 becomes mid and then small on consecutive ticks because the
full-to-mid branch takes precedence. `NoAI`, client-side state and death
freeze transitions. Raw negative and above-two states match neither
transition state and therefore never self-normalize; their deflate timer can
increment indefinitely. Saving/reloading above two finally clamps it to two,
whereas a negative value remains stuck.

### Mob and Player stings

After inherited fish `aiStep`, a live server Pufferfish with raw state
strictly above zero queries `Mob` instances intersecting its current bounding
box inflated by `0.3`. It reuses the scary targeting conditions, then checks
each admitted Mob alive and offers `1+state` Mob-Attack damage.

Only accepted damage applies Poison for `60*state` ticks at amplifier zero
with the Pufferfish as source, then asks the fish to play sting at volume and
pitch `1`. The effect result is ignored, so an immune/rejecting target still
gets the sting sound after accepted damage. Damage invulnerability or any
other rejected hurt suppresses both Poison and sound. Intended mid/full
contacts are therefore damage `2/3` and Poison `60/120`; direct metadata
above two increases both formulas without a cap and can overflow the
duration multiplication.

The automatic Mob query cannot sting non-Mob Living Entities. Player
collision instead calls `playerTouch`. Only a `ServerPlayer` and state above
zero proceed. Accepted `1+state` Mob-Attack damage sends that victim a
`PUFFER_FISH_STING` clientbound game event with value `0` unless the fish is
silent, then applies the same `60*state`, amplifier-zero Poison regardless
of the effect result. The client handles that event by playing Pufferfish
sting at the local Player position in `NEUTRAL`, volume/pitch `1`. Rejected
damage sends no event and applies no Poison. This path does not call the
Mob-sting helper, so it does not also broadcast the fish-origin sound.

Puff state, and therefore dimensions, threat box, sting box, damage and
duration, is read independently at each step. A metadata change between
callbacks takes effect immediately.

### Finalization, placement and natural selection

Pufferfish add no subtype finalizer. Generic Mob finalization installs the
permanent triangular `random_spawn_bonus` follow-range modifier when absent
and consumes the later float that makes a Mob left-handed below `0.05`.
There is no typed group data or shared state; each natural pack member
finalizes independently.

Pufferfish register `IN_WATER`, heightmap
`MOTION_BLOCKING_NO_LEAVES`, with the ordinary surface-water predicate. The
placement-type gate requires a non-null type, world-border inclusion,
Water-tag candidate fluid and nonconducting block above. Species admission,
without RNG, requires:

1. Y in inclusive `seaLevel-13..seaLevel`;
2. Water-tag fluid below; and
3. exact `Blocks.WATER` above.

Spawn obstruction later requires the entity to be unobstructed. There is no
light or difficulty predicate.

Exactly three of 66 locked biomes select Pufferfish in `water_ambient`.
Deep Lukewarm Ocean and Lukewarm Ocean use weight `5`; Warm Ocean uses
weight `15`. All three request group `1..3`. The category is
friendly/nonpersistent, cap `20`, distances `32/64`, and the inherited
cluster maximum is `8`, so the biome group maximum controls. Generic
selection, attempts, insertion and cap accounting retain `MOB-SPAWN-001`.

### Bucket state loss

Capture uses exact Water Bucket and an alive Pufferfish. Raw item ID `1047`
(`pufferfish_bucket`) receives only the common bucket payload: custom name
plus true NoAI/Silent/NoGravity/Glowing/Invulnerable/PersistenceRequired
flags and Health. `PuffState`, `inflateCounter` and `deflateTimer` are not
copied. A captured mid/full or malformed-state fish therefore loses its
inflation state.

Release constructs a default-small Pufferfish with reason `BUCKET`, performs
generic finalization and generic stack configuration, then loads only those
common bucket fields. `minecraft:bucket_entity_data.PuffState` is ignored by
the Pufferfish bucket loader. A separately supplied generic entity-data
configuration remains owned by the common item/entity path. `FromBucket`
becomes true before insertion and ambient invocation; the latter has no
Pufferfish ambient event and produces nothing.

`AbstractFish.mobInteract` retains the eager `Optional.orElse` quirk:
generic Mob interaction evaluates to `PASS` even after successful capture
has replaced the hand and discarded the fish. Exact capture/release sounds,
hand/inventory changes, criteria, payload, insertion failure and event order
remain with `ITM-MOB-BUCKET-001`.

### Loot, tags and sounds

The entity loot table has type `entity`, sequence
`minecraft:entities/pufferfish`. Its first independent pool emits exactly
one raw Pufferfish, raw item ID `1089`; it has no furnace-smelt or Looting
function. The second independent pool emits one Bone Meal with probability
`0.05`. Eligible death separately emits XP `1..3`.

Pufferfish belong directly to exactly six entity-type tags:

- `aquatic`, transitively selecting `sensitive_to_impaling`;
- `axolotl_hunt_targets`, allowing a hunt-ready Axolotl to select a visible,
  attackable, in-water Pufferfish within squared distance `64`;
- `can_breathe_under_water`;
- `cannot_be_pushed_onto_boats`, preventing collision auto-mount but not the
  physical push;
- `nautilus_hostiles`, allowing a Nautilus to consider an in-water
  Pufferfish hostile; and
- `not_scary_for_pufferfish`, making Pufferfish safe to one another.

The safe tag also contains both Nautilus types, so a Nautilus can select a
Pufferfish without itself triggering inflation. Axolotls are not safe and
can trigger/sting while pursuing their tag-selected prey.

Advancements containing the Pufferfish string select raw or bucket items,
not the exact entity type. Common Pufferfish Spawn Egg is raw item ID `1186`,
stack `64`, with `entity_data.id=minecraft:pufferfish`; generic Egg
construction, state configuration, finalization and insertion retain their
owner.

Blow-out, blow-up, death, flop, hurt and sting use protocol sound IDs
`1342..1347`; fish swim is ID `638`. Blow-out has two clips at resource
volume `0.7`; blow-up has two at `0.45`; flop has four at `0.3`; death,
hurt and sting have two each at default volume/pitch; swim has seven.
English subtitles are `Pufferfish deflates`, `Pufferfish inflates`,
`Pufferfish dies`, `Pufferfish flops`, `Pufferfish hurts`,
`Pufferfish stings` and `Splashes`. There is no ambient event override.
Generic voice admission/range and runtime pitch retain their owners.

Exact UTF scanning finds zero Pufferfish occurrence in all `1,212` structure
templates.

### Legacy migration

Seven exact schema/fix contexts own Pufferfish entity or item compatibility:

- `V1483` moves the inherited Mob schema from legacy
  `minecraft:puffer_fish` to `minecraft:pufferfish`;
- `EntityPufferfishRenameFix` renames that entity ID and maps
  `minecraft:puffer_fish_spawn_egg` to the current Egg;
- `EntityUUIDFix` includes current Pufferfish in Mob UUID conversion;
- `V705` maps the current Pufferfish Spawn Egg to its entity shape;
- `ItemStackComponentizationFix` moves legacy bucket-mob fields into
  `minecraft:bucket_entity_data`;
- `ItemStackSpawnEggFix` maps current entity identity to the current Spawn Egg
  item; and
- `ItemStackTheFlatteningFix` maps legacy `minecraft:fish.3` to raw
  `minecraft:pufferfish`.

No fix rewrites `PuffState`, supplies lower-bound clamping or creates
persistent counters.

### Three-model client projection

`EntityRenderers` binds Pufferfish to `PufferfishRenderer`, constructor
shadow `0.2` and shared `textures/entity/fish/pufferfish.png`. Render-state
extraction copies the raw Puff State; a fresh state defaults to zero.
Submission selects small for exactly `0`, mid for exactly `1`, and big for
every other integer, matching server dimension fallback.

The `32×32` small mesh has six named parts: a `3×2×3` body, two `1³` eyes,
a `3×0×3` back fin and two `1×0×2` side fins. The mid mesh has eleven:
a `5³` body, two animated `2×0×2` blue fins and eight static top/side/bottom
spine plates. The big mesh has thirteen: an `8³` body, two animated
`2×1×2` blue fins and ten static spine plates, adding the middle top and
bottom plates.

Every model animates the corresponding right fin Z rotation as
`-0.2+0.4*sin(0.2*age)` and left as
`0.2-0.4*sin(0.2*age)`, regardless of water. Before superclass rotations,
the renderer bobs Y by `0.08*cos(0.05*age)` and does not add the dry
translation/90-degree rotation used by Cod, Salmon and Tropical Fish.

Dynamic shadow radius is `0.1+0.1*rawState`, not the constructor constant.
Thus intended states yield `0.1/0.2/0.3`, while malformed metadata can
produce negative or arbitrarily large values even though mesh and dimensions
have already fallen back to big/full.

The shared texture is `32×32`, `490` bytes, SHA-256
`9403593783cb7b074569c7f977c210ac8cb5b967bd0ad0027e9b44c371d942f1`.
The generated Spawn-Egg texture is `16×16`, `272` bytes, SHA-256
`3d2ecb04a6471e238339920002ff2f68272511285410b96bc31df755687e8ffd`.
English names are `Pufferfish`, `Pufferfish Spawn Egg` and
`Bucket of Pufferfish`; Puff State adds no bucket tooltip.

**Branches and aborts:**

- Metadata is raw; load clamps only above two, not below zero.
- Custom transitions precede the goal start/stop that changes their counters.
- Puff admission ignores walls and invisibility but rejects dead, spectator,
  creative and safe-tag entities.
- State must be strictly positive to sting; accepted damage gates Poison and
  effect sound/event.
- Captured buckets omit Puff State; release starts small unless a separate
  generic configuration changes it.
- There is no ambient sound event.

**Constants and randomness:**

Entity/Egg/bucket/raw-item IDs `107/1186/1047/1089`; full
dimensions/eye `0.7×0.7/0.455`; state scales `0.5/0.7/1`;
tracking/update `4/3`; health/speed/follow `3/0.7/16`; metadata
`16 BOOLEAN, 17 INT`; goals `0/1/2/4`; threat/sting inflation
`2/0.3`; counters `1/>40/>60/>100`; damage/duration
`1+state/60*state`; movement `0.005/0.125/0.01/0.9`; flop
`±0.05/0.4000000059604645`; air `300/-20/2`; spawn depth `13`,
rows `3/66`, weights `5/15`, group `1..3`, category `20/32/64`;
Bone Meal `0.05`, XP `1..3`; sounds `1342..1347/638`;
tags/templates/migrations `6/0 of 1212/7`; parts `6/11/13`; shadow
formula `0.1+0.1*state`.

**Side effects:**

Puff metadata, dimensions, transient counters, `FromBucket`, common durable
state; goal scheduling and path/motion/air; damage, Poison, sting game event
and sound; loot and XP; cap/pack state; bucket
hand/discard/payload/insertion; tag-selected threat, Axolotl, Nautilus, boat
and Impaling behavior; client model, bob, fin and shadow selection.

**Gates:**

Raw state/sign/exact value; logical side, alive/effective-AI/NoAI and
counter thresholds; threat class/box/alive/spectator/creative/tag;
Mob-versus-Player contact, hurt acceptance, silence and effect acceptance;
water/ground/collision/air; border/Y/fluid/block/biome/cap;
bucket/aliveness/configuration; death chance; tags and client raw state.

**Boundary cases and quirks:**

Negative NBT remains negative, full-sized, big-modeled, nonstinging and stuck;
above-two metadata is full-sized/big-modeled, stuck and more damaging until a
save/load clamps it. Shadow radius nevertheless uses the raw value. Goal
updates lag transition logic by one tick. A through-wall invisible
noncreative threat inflates the fish, but only Mob intersection or Player
collision stings. Accepted damage followed by rejected Poison still sounds.
NoAI freezes state transitions but preserves synchronized geometry. Captured
buckets discard inflation.

**Failure semantics:**

Rejected placement prevents natural insertion. Failed insertion does not
roll back finalization or common bucket configuration. Rejected sting damage
commits no Poison, event or sting sound. Rejected Poison does not roll back
accepted damage or sound/event. Rejected Drown damage does not undo air
reset. Loot, XP, Egg and bucket owners retain their commit boundaries.

**Client/server authority split:**

The server owns raw Puff State, dimensions, counters, goals, threat/sting
admission, Poison, placement, bucket transfer, damage, loot and XP. Slots
`16/17` synchronize bucket origin and raw Puff State; counters do not cross
the wire. Accepted nonsilent Player stings send a victim-only game event.
The client selects mesh/shadow from raw state and applies shared bob/fin
animation; it plays the sting event at the local Player.

**Observability:**

Observe slots `16/17`, raw NBT clamping, dimension refresh ordering,
counter/goal one-tick separation and transition sounds; state-dependent
threat/sting AABBs; through-wall/invisible/safe/creative admission; Mob and
Player damage/Poison/sound/event commit boundaries; movement/flop/air;
three-biome selection; bucket state loss; loot/XP/six tags/Egg;
zero-template/seven-fix closure; three abrupt models, shared texture,
water-independent fins/bob and raw shadow.

**Persistence and reload:**

Generic Mob state, `FromBucket` and upper-only-clamped `PuffState` persist;
inflate/deflate counters, goals and paths do not. Code fixes registration,
state machine, goals, contact, placement and schemas. Biomes, safe/consumer
tags and loot reload through their owners; sounds, language, layers and
texture are client resources.

**Evidence:**

`EntityTypes`, `DefaultAttributes`, `SpawnPlacements`,
`SpawnPlacementTypes`, `MobCategory`, `Mob`, `NaturalSpawner`,
`TargetingConditions`; `net.minecraft.world.entity.animal.fish.{WaterAnimal,AbstractFish,Pufferfish}`;
`PufferfishPuffGoal`, `Bucketable`, `MobBucketItem`, `SoundEvents`;
Axolotl, Nautilus-AI and AbstractBoat consumers; client
`ClientPacketListener`, `EntityRenderers`, `PufferfishRenderer`,
`PufferfishRenderState`, all three Pufferfish models and `LayerDefinitions`;
the seven migration/schema classes named above; reports, three biomes, six
entity tags, loot, all 1,212 structures, sounds, language, layers and
textures. Complete compiled/data identity searches find no other exact
entity runtime path.

**Test vectors:**

Run `EXP-ENT-018` across arbitrary Puff State/NBT values, counter and
goal-timing boundaries, scary/safe/creative/invisible/occluded entities,
Mob/Player damage/Poison/sound/event outcomes, movement/flop/air,
three-biome placement, capture/release state loss, loot/XP/tags/Egg,
templates/migrations/sounds and every model/shadow/water state.

**Limits:**

Generic entity lifecycle, navigation, damage/effects/death, natural
spawning, despawn, loot, Spawn Egg, bucket transaction, metadata packets,
game-event packet framing and render submission retain their owners. Raw
Pufferfish and Pufferfish Bucket item behavior retain their item leaves.
This leaf fixes exact Pufferfish entity dispatch and every direct join
selecting it.
