# Entities mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ENT-TADPOLE-001` — Tadpoles run fish and Brain AI while age, food and Golden Dandelion state converge on Frog conversion

**Parent:** `ENT-001`, `ENT-LIFECYCLE-001`, `ENT-002`,
`ENT-VEHICLE-001`, `ENT-004`, `ENT-PROJECTILE-001`, `ENT-005`,
`ENT-DAMAGE-001`, `ENT-BLOCK-001`, `ENT-DAMAGE-REDUCE-001`,
`ENT-KNOCKBACK-001`, `ENT-006`, `ENT-EFFECT-001`, `ENT-007`,
`ENT-DEATH-001`, `MOB-001`, `MOB-AI-001`, `MOB-002`,
`MOB-SPAWN-001`, `MOB-003`, `MOB-DESPAWN-001`, `MOB-005`,
`MOB-BREED-001`, `BLK-FROGSPAWN-001`, `ITM-MOB-BUCKET-001`,
`ITM-SLIME-BALL-001`, `ITM-ENCHANT-001`, `PLY-AUTOJUMP-001`,
`WGEN-005`, `WGEN-PORTAL-001`, `CLI-001`, `CLI-006`,
`CLI-EFFECT-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked registration, complete age/lock/feeding and Frog
conversion paths, fish superclass plus Tadpole Brain, Frogspawn and bucket
joins, five direct tags, empty loot, four migration/schema contexts, all
1,212 templates and exact client projection close protocol entity ID `131`.

**Applies when:**

`minecraft:tadpole` is constructed, hatched from Frogspawn, spawned by a
bucket, Egg, spawner or command, loaded, aged, locked or unlocked, fed,
tempted, swimming, flopping, drying, captured, released, hunted, killed,
converted to a Frog, synchronized or rendered.

**Authoritative state:**

Protocol entity ID `131` constructs `Tadpole` in `CREATURE`. Registration
fixes width `0.4`, height `0.3`, eye height `0.19500001`, client tracking
range `10` and default update interval `3`. Tadpoles are Peaceful-compatible.
Attributes come from the Animal/Mob builders but Tadpole is still an
`AbstractFish`: maximum health `6`, movement speed `1`, follow range `16`
and temptation range `10`.

Inherited BOOLEAN metadata slot `16`, serializer ID `8`, is defined false
for `FromBucket`. Tadpole overrides `fromBucket()` to return true
unconditionally and `setFromBucket` to do nothing, so the wire slot remains
false while semantic bucket-origin is always true. Inherited saving
nevertheless calls the override and always writes `FromBucket=true`; loading
that field calls the no-op setter.

Tadpole adds BOOLEAN slot `17`, serializer ID `8`, default false, for
`AGE_LOCKED`. The signed `age` integer and `ageLockParticleTimer` integer
are ordinary zero-initialized fields: neither is synchronized, and the
particle timer is not saved. Save always writes raw `Age` and `AgeLocked`;
load defaults both to zero/false and routes age through the converting
setter. There is no Tadpole age implicit component.

The public mutable static `ticksToBeFrog` defaults to
`abs(-24000)=24000`. It is read afresh at every setter call; age has no
clamp or lower bound.

**Transition and ordering:**

### Goal selector, Brain and water motion

Tadpole does not override `registerGoals`, so the inherited fish selector
remains live:

- priority `0`, `PanicGoal` at speed `1.25`;
- priority `2`, avoid non-spectator Players within `8`, speeds `1.6/1.4`;
  and
- priority `4`, interval-`40` fish swimming at speed `1`.

It simultaneously replaces the fish move/look controls and supplies a
Tadpole Brain. `SmoothSwimmingMoveControl(85,10,0.02,0.1,true)` applies
in-water buoyancy Y `0.005`, caps pitch target at `±85`, turns yaw at most
`10`, uses water/outside speed factors `0.02/0.1` and slows dry forward
speed continuously from full below a `10`-degree turn to zero at `60`.
`SmoothSwimmingLookControl(10)` uses the shared `+20`-yaw/`+10`-pitch look
bias and keeps head/body yaw within `10`.

The Brain provider installs exact sensors `NEAREST_LIVING_ENTITIES`,
`NEAREST_PLAYERS`, `HURT_BY` and `FROG_TEMPTATIONS`. Its activities are:

- `CORE`: `AnimalPanic(2)`, `LookAtTargetSink(45,90)`,
  `MoveToTargetSink`, then temptation-cooldown countdown;
- `IDLE` priority `0`: sometimes look at a Player within `6`, interval
  uniform `30..60`;
- priority `1`: follow temptation at speed `1.25`; and
- priority `2`: when `WALK_TARGET` is absent, an ordered/try-all gate
  containing water stroll speed `0.5`, look-target walking speed `0.5`
  within `3`, and an `isInWater` trigger, with weights `2/3/5`.

Activity update always selects the first valid activity from the singleton
list `[IDLE]`. Server custom AI ticks the Brain, updates activity, then calls
the fish superclass hook. The ordinary goal selector also remains active;
generic Mob scheduling owns contention between its navigation writes and
the later Brain/control phases. Slime Ball is the sole direct
`#minecraft:frog_food` item and supplies the temptation sensor.

Tadpole retains Water-Bound navigation and the shared fish travel, flop and
air kernel. Water travel applies input `0.01`, moves, scales velocity by
`0.9` and sinks Y `0.005` only without a target. Outside water while on
ground and vertically colliding, AI step consumes two floats for X/Z
`(2f-1)*0.05`, adds Y `0.4000000059604645`, clears on-ground state,
requests impulse synchronization and plays Tadpole flop. Server air
decrements while alive and dry, resets at `-20` before offering `2` Drown
damage, and resets to `300` in water or on death. Step sound is a no-op.

### Age tick and direct conversion

After the inherited fish `aiStep`, every server-side unlocked Tadpole calls
`setAge(age+1)`. This does not test `NoAI`, effective AI, water, difficulty
or a synchronized baby flag. Locked Tadpoles do not increment. Client
Tadpoles never receive age, so age has no client interpolation or
size/model consequence.

Every age write first stores the raw integer and then converts when
`age>=ticksToBeFrog`. Natural aging therefore reaches the default boundary
after 24,000 increments. Negative values lengthen the wait; integer
overflow remains ordinary signed overflow. Loading or bucket/entity-data
configuration can invoke the same conversion immediately.

Conversion asks for a Frog with reason `CONVERSION` and
`ConversionParams.single(tadpole,false,false)`: single replacement, no kept
equipment and no preserved pickup ability, retaining the source team
through the generic conversion owner. If the Tadpole is already removed or
Frog construction returns null, conversion returns null and the Tadpole is
not discarded; an unlocked threshold-age Tadpole retries on its next age
write.

For a constructed Frog, the after-conversion callback runs before insertion:

1. finalize the Frog at its current block with reason `CONVERSION`;
2. select warm or cold Frog variant at priority `1` from the corresponding
   biome tags, otherwise temperate at priority `0`; if a reload makes
   multiple highest-priority records match, choose uniformly among them;
3. initialize Frog memories and perform generic Mob finalization;
4. mark the Frog persistence-required;
5. ask it to adjust position for the old Tadpole dimensions; and
6. ask the Tadpole to play grow-up at runtime volume `0.15`, pitch `1`.

Generic single conversion then offers the Frog to the level and discards the
Tadpole. The insertion result is ignored: rejection can therefore consume
the Tadpole after finalization and sound without leaving an inserted Frog.
Frog behavior after successful conversion retains its own catalog owner.

### Slime-Ball acceleration

Interaction checks food before Golden Dandelion or bucket capture. A held
Slime Ball and unlocked state consumes one player item, computes

`seconds=floor((max(0,ticksToBeFrog-age)/20)*0.1)`

under the actual integer-division/float/truncation sequence, then calls
`setAge(age+20*seconds)` and emits one `HAPPY_VILLAGER` particle at a random
X/Z point and Y plus `0.5`. With default age zero the first feed advances
2,400 ticks. Fewer than 200 ticks remaining produces zero seconds but still
consumes and particles. Threshold-minus-age, multiplication and addition
retain signed-int overflow before the float conversion. Locked Tadpoles skip
this branch, so Slime Ball can fall through to ordinary interaction.
Accelerated age uses the same conversion and failure boundaries as ticking.

### Golden Dandelion lock toggle

After food, exact raw item ID `257` (`golden_dandelion`) is admitted only
when `ageLockParticleTimer==0` and Tadpole is absent from reloadable
`#minecraft:cannot_be_age_locked` (currently Zombie Horse, Skeleton Horse
and Villager only). On success it:

1. toggles slot-17 `AgeLocked`;
2. resets age to exactly zero through the converting setter;
3. sets the transient particle timer to `40`;
4. consumes one item;
5. marks persistence-required only when the new state is locked; and
6. plays `GOLDEN_DANDELION_USE` for locking or
   `GOLDEN_DANDELION_UNUSE` for unlocking, source `PLAYERS`,
   volume/pitch `1`.

Unlocking does not clear persistence. The 40-tick timer prevents another
toggle until it reaches zero. Each AI step delegates it to the shared
particle helper; a client on which the local interaction initialized the
timer emits one particle at each even positive value, then the timer
decrements. A locked result emits 20
`PAUSE_MOB_GROWTH` particles with Y offset `+0.2`; an unlocked result emits
20 `RESET_MOB_GROWTH` particles without that offset. Server and client
timers are transient and not synchronized independently.

### Bucket interaction and payload

Only after food and lock branches does Tadpole attempt bucket capture. Its
fallback is eager: the outer `Optional.orElse` evaluates inherited
`AbstractFish.mobInteract`, whose own eager fallback attempts the same
bucket helper again before Water-Animal interaction. A successful outer
capture has already replaced the hand and discarded the Tadpole, so the
second attempt cannot create a second capture.

Raw item ID `1053` (`tadpole_bucket`, stack one) receives the common bucket
payload and always writes raw `Age` plus `AgeLocked`. Release creates and
generically finalizes a zero-age Tadpole, applies stack configuration, then
loads common payload, optional `Age`, and `AgeLocked` default false.
Missing Age preserves zero. `setFromBucket(true)` remains a no-op, but
semantic `fromBucket()` is still true.

An injected bucket `Age>=ticksToBeFrog` converts during payload loading,
before the bucket helper calls the no-op setter and offers its original
Tadpole reference for insertion. Successful conversion has already inserted
the Frog and discarded that reference; generic bucket event, remainder and
fluid commit boundaries retain `ITM-MOB-BUCKET-001`. Normal capture/release
round-trips signed age and lock state but not the particle timer or Brain
runtime state. The default bucket has empty `bucket_entity_data` and no
subtype tooltip.

### Production, loot, tags and sounds

Tadpole has no `SpawnPlacements` registration and occurs in zero of the 66
biome spawn lists. It is therefore not selected by natural spawning.
`BLK-FROGSPAWN-001` owns the built-in production path: after the scheduled
hatch it directly creates `2..5` Tadpoles with reason `BREEDING`, does not
call generic finalization, snaps each below the former block with independent
clamped offsets/yaw, marks persistence-required and ignores insertion
failure. Bucket, Spawn Egg, spawner, command and custom creation use their
generic owners.

`CREATURE` is friendly, cap `10` and category-persistent. Independently,
semantic `fromBucket()==true` makes inherited fish distance removal false
for every Tadpole, including Egg/command-created instances. Maximum cluster
size remains `8`, but no natural Tadpole row requests a cluster. Tadpoles
cannot be leashed and are not pushed by fluid.

The entity loot table is an empty type-`entity` table with random sequence
`minecraft:entities/tadpole`; it emits no item and consumes no loot RNG.
`shouldDropExperience()` is always false, including player-credit deaths.

Tadpole belongs directly to exactly five entity-type tags:

- `aquatic`, transitively selecting `sensitive_to_impaling`;
- `axolotl_hunt_targets`;
- `can_breathe_under_water`;
- `cannot_be_pushed_onto_boats`; and
- `not_scary_for_pufferfish`.

Thus an Axolotl can hunt it, while it does not inflate a Pufferfish.
The filled-bucket advancement selects the Tadpole Bucket item, not the entity.
Spawn Egg raw ID `1189`, stack `64`, has
`entity_data.id=minecraft:tadpole`.

Tadpole death/flop/grow-up/hurt are sound IDs `1618..1621`; fish swim is
`638`. Death has two clips, hurt four, and grow-up reuses eight Frog idle
clips at resource volume `0.75`, pitch `1.2`. Flop aliases the four-clip
Tropical-Fish flop event at resource volume `0.3`. English subtitles are
`Tadpole dies`, `Tadpole flops`,
`Tadpole grows up`, `Tadpole hurts` and `Splashes`. Ambient is explicitly
null. Bucket fill/empty IDs are `236/229`; Golden Dandelion use/unuse are
`753/754`.

Exact UTF scanning finds zero Tadpole identity in all `1,212` structure
templates.

### Legacy migration

Four exact schema/fix contexts own Tadpole compatibility:

- `V3078` registers Frog and Tadpole as Mob schemas;
- its `AddNewChoices` bootstrap adds the named `Added Tadpole` entity choice;
- `V705` maps the current Tadpole Spawn Egg to Tadpole entity shape; and
- `ItemStackComponentizationFix` moves common bucket-mob fields, including
  Age, into `minecraft:bucket_entity_data`.

No fix clamps Tadpole age, synthesizes AgeLocked, preserves the particle
timer or changes conversion state.

### Client projection

`EntityRenderers` binds Tadpole to `TadpoleRenderer`, using ordinary
`LivingEntityRenderState`, shadow radius `0.14` and
`textures/entity/tadpole/tadpole.png`. There is one `16×16` model layer with
two root parts: body box `3×2×3` at `(0,22,-3)` and zero-width tail plane
`0×2×7` at `(0,22,0)`.

The sole animation sets tail Y rotation to
`-0.25*sin(0.3*ageInTicks)` in water and
`-0.375*sin(0.3*ageInTicks)` outside water. Renderer and model have no age
field, lock-state branch, dry translation or subtype layer.

Entity texture is `16×16`, `158` bytes, SHA-256
`31c004940eb00c301cfeb4e7d155e22bebeb181e57506fe9fd57447ec57d8417`.
Spawn-Egg texture is `16×16`, `224` bytes, SHA-256
`557a75a1d8f24c3033b45d337918c105bb8c70bc89f7c4bc58b97d99d94a7f8c`.
Bucket texture is `16×16`, `234` bytes, SHA-256
`869a08f376e19a7ad8345f65a629eeb14e361b171b1dd8892fe282c68ab8de92`.
English names are `Tadpole`, `Tadpole Spawn Egg` and `Bucket of Tadpole`.

**Branches and aborts:**

- Age is server-only raw state; only AgeLocked synchronizes.
- Every age setter stores first and converts at the mutable threshold.
- Food precedes lock toggle, which precedes the double-eager bucket fallback.
- Age lock is the only ordinary aging gate; `NoAI` does not freeze age.
- Frog construction failure retains the threshold Tadpole; Frog insertion
  failure does not.
- There is no natural spawn row, item/XP death output or ambient sound.

**Constants and randomness:**

Entity/Egg/bucket/food/lock-item IDs `131/1189/1053/1059/257`;
dimensions/eye `0.4×0.3/0.19500001`; tracking/update `10/3`;
health/speed/follow/tempt `6/1/16/10`; metadata `16/17 BOOLEAN`;
age `0/24000`; feed factor `0.1` in whole seconds; lock timer/particles
`40/20`; goals `0/2/4`; Brain speeds `2/1.25/0.5`; look interval
`30..60`; controls `85/10/0.02/0.1`; movement/air retain the fish constants;
Frogspawn `2..5`; category/cluster `10/8`; loot/XP `0/0`;
tags/templates/fixes `5/0 of 1212/4`; sounds
`1618..1621/638/236/229/753/754`; shadow `0.14`; tail
`-0.25/-0.375*sin(0.3*age)`.

**Side effects:**

Server age, synchronized lock, transient particle timer and semantic
bucket-origin; goals, Brain memories, navigation, movement/flop/air; player
item consumption, particles, sounds and persistence; Frog construction,
finalization, variant/memory selection, insertion and source discard;
bucket payload and advancement; empty loot/no XP; tag-selected Axolotl,
Pufferfish, boat and Impaling behavior; client tail and texture.

**Gates:**

Logical side, lock and mutable threshold; item exact/tag identity, lock
cooldown and cannot-lock tag; remaining-age rounding; removed/construction/
insertion conversion state; fish movement/water/air gates; interaction
ordering and bucket payload presence; Frog conversion biome; death
eligibility; client water state.

**Boundary cases and quirks:**

Slot 16 says false while every API call says from-bucket true. Age is
invisible to clients and may be negative or overflow. Food inside the last
199 ticks can consume without acceleration. Locking and unlocking both reset
age to zero, and unlocking leaves persistence set. NoAI Tadpoles keep aging.
Threshold NBT or bucket data can convert during load. A rejected converted
Frog insertion loses both forms. Tadpole is both classic-goal- and
Brain-driven despite inheriting the fish movement kernel.

**Failure semantics:**

Null Frog construction keeps the source for a later retry. Once conversion
constructs a Frog, callback mutations and grow sound precede insertion;
ignored insertion failure still discards the Tadpole. Frogspawn and bucket
insertion owners likewise do not roll back their earlier block/fluid/item
effects. Feeding consumes before any later conversion insertion result.
Empty loot and disabled XP commit nothing.

**Client/server authority split:**

The server owns age, conversion, Frog variant/finalization, interaction,
food, lock admission, persistence, Brain, damage, loot and bucket payload.
Slot `17` synchronizes lock; slot `16` remains false and age/timers/Brain do
not cross the wire. The client observes lock through particles initiated by
its transient interaction timer, plus standard position/water/render state.

**Observability:**

Observe slots `16/17` against API `fromBucket`, raw Age/lock save and bucket
payload, tick/feed/load conversion boundaries, Golden Dandelion timer/
particles/sounds/persistence, simultaneous goals and Brain, smooth controls
plus fish movement/flop/air, Frogspawn-only production, empty loot/XP, five
tags/Egg/advancement, four fixes/zero templates, and exact two-part model,
tail coefficients, hashes and names.

**Persistence and reload:**

Generic Mob state, always-true saved FromBucket, raw Age and AgeLocked
persist. Particle timer, navigation, sensors and active Brain/goal state do
not. Frogspawn/bucket/tag/loot/variant data reload through their owners;
sounds, language, layer and textures are client resources.

**Evidence:**

`EntityTypes`, `DefaultAttributes`, `MobCategory`, `Mob`,
`ConversionParams`, `ConversionType`, `AgeableMob`;
`net.minecraft.world.entity.animal.fish.{WaterAnimal,AbstractFish}`;
`net.minecraft.world.entity.animal.frog.{Tadpole,TadpoleAi,Frog}`;
`SmoothSwimmingMoveControl`, `SmoothSwimmingLookControl`, Brain behaviors
and four sensors; `FrogspawnBlock`, `Bucketable`, `MobBucketItem`,
`SoundEvents`; client `EntityRenderers`, `TadpoleRenderer`, `TadpoleModel`
and layer definitions; the four migration/schema contexts; reports, Frog
variants/biome tags, five entity tags, food tag, loot, advancement, all
1,212 structures, locked sounds object, language, items, models and
textures. Complete compiled/data identity searches find no other exact
Tadpole runtime path.

**Test vectors:**

Run `EXP-ENT-019` across arbitrary ages/thresholds, lock and particle
timers, tick/feed/NBT/bucket/Spawn-Egg conversion, null/rejected Frog cases,
all warm/cold/temperate locations, goal/Brain/temptation and smooth-control
branches, fish movement/flop/air, Frogspawn/bucket/Egg production, empty
loot/XP/five tags, templates/migrations/sounds and every client water state.

**Limits:**

Generic entity lifecycle, Brain/goal scheduling, navigation, damage/effects/
death, conversion copying, Frog behavior, Frogspawn block algorithm, bucket
transaction, Spawn Egg/entity-data loading, metadata packets and render
submission retain their owners. Slime Ball, Golden Dandelion and Tadpole
Bucket item behavior retain item/block owners. This leaf fixes exact Tadpole
entity dispatch and every direct join selecting it.
