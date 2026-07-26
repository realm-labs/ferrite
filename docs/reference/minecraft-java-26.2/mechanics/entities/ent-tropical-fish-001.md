# Entities mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ENT-TROPICAL-FISH-001` — Tropical Fish share common variants in eight-member schools but rare variants spawn alone

**Parent:** `ENT-001`, `ENT-LIFECYCLE-001`, `ENT-002`,
`ENT-VEHICLE-001`, `ENT-004`, `ENT-PROJECTILE-001`, `ENT-005`,
`ENT-DAMAGE-001`, `ENT-BLOCK-001`, `ENT-DAMAGE-REDUCE-001`,
`ENT-KNOCKBACK-001`, `ENT-006`, `ENT-EFFECT-001`, `ENT-007`,
`ENT-DEATH-001`, `MOB-001`, `MOB-AI-001`, `MOB-002`,
`MOB-SPAWN-001`, `MOB-003`, `MOB-DESPAWN-001`, `MOB-005`,
`ITM-MOB-BUCKET-001`, `ITM-TROPICAL-FISH-001`, `ITM-ENCHANT-001`,
`PLY-AUTOJUMP-001`, `WGEN-005`, `WGEN-PORTAL-001`, `CLI-001`,
`CLI-006`, `CLI-EFFECT-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked registration, packed variant/component paths,
common/rare finalization, schooling/fish/water superclasses, placement and
five biome rows, five direct tags, loot, Spawn Egg, seven migration/schema
contexts, all 1,212 templates and exact two-mesh/fourteen-texture client
projection close protocol entity ID `137`.

**Applies when:**

`minecraft:tropical_fish` is constructed, finalized, naturally selected,
spawned by a bucket, Egg, spawner or command, loaded, schooling, swimming,
flopping, drying, captured, released, targeted by an Axolotl, killed,
synchronized, described in a bucket tooltip or rendered.

**Authoritative state:**

Protocol entity ID `137` constructs `TropicalFish` in `WATER_AMBIENT`.
Registration fixes width/height `0.5×0.4`, eye height `0.26`, tracking range
`4` and default update interval `3`. Tropical Fish are allowed in Peaceful.
Attributes are maximum health `3`, inherited movement speed `0.7` and follow
range `16`; eligible death supplies XP `1+nextInt(3)`.

`AbstractFish` supplies BOOLEAN metadata slot `16`, serializer ID `8`,
default false, and the always-written/default-false `FromBucket` persistence.
Tropical Fish add INT slot `17`, serializer ID `1`, default `0`. Its packed
layout is:

```text
bits  0..15 = pattern packed ID
bits 16..23 = base DyeColor numeric ID
bits 24..31 = pattern DyeColor numeric ID
```

Packing masks the three fields with `65535/255/255`. The default is
Kob/White/White and therefore integer zero. Pattern IDs are sparse:

| serialized pattern | base mesh | local ID | packed ID |
|---|---|---:|---:|
| `kob` | small | 0 | 0 |
| `sunstreak` | small | 1 | 256 |
| `snooper` | small | 2 | 512 |
| `dasher` | small | 3 | 768 |
| `brinely` | small | 4 | 1024 |
| `spotty` | small | 5 | 1280 |
| `flopper` | large | 0 | 1 |
| `stripey` | large | 1 | 257 |
| `glitter` | large | 2 | 513 |
| `blockfish` | large | 3 | 769 |
| `betty` | large | 4 | 1025 |
| `clayfish` | large | 5 | 1281 |

Unknown low-16-bit pattern IDs decode as Kob. Each extracted color byte uses
`DyeColor.byId`; the continuous-ID map's zero fallback makes every unknown
color ID decode as White. A direct metadata write retains its arbitrary raw
integer, while all entity getters expose the decoded semantic triple.

Save decodes the current raw integer into a `Variant` record and stores that
record through its integer codec under uppercase `Variant`; encoding repacks
the semantic values, so malformed raw bits canonicalize. Load accepts any
integer, decodes and repacks it, while missing or wrong-type input selects
Kob/White/White. Pattern has a string codec and ID stream codec; both colors
use their own codecs. Variant does not change dimensions, and slot `17`
updates do not request a dimension refresh.

The entity exposes three implicit components:
`minecraft:tropical_fish/pattern`,
`minecraft:tropical_fish/base_color` and
`minecraft:tropical_fish/pattern_color`. Reads return the semantic triple.
Application is ordered pattern, base color, pattern color, then superclass;
each setter preserves the other two currently decoded fields and repacks the
whole integer.

School `leader`, `schoolSize` and the Tropical-Fish-only `isSchool` boolean
remain transient and unsynchronized. Construction initializes `isSchool`
true; loading does not restore a rare fish's false value. Every fresh or
loaded fish otherwise begins with no leader and school size one.
`FromBucket=true`, generic persistence or a custom name suppress ordinary
distance removal; otherwise Water-Ambient thresholds are `32/64`. Tropical
Fish cannot be leashed and are not pushed by fluid.

**Transition and ordering:**

### Goals, movement, flop and air

Tropical Fish inherit the exact fish goal graph:

- priority `0`, panic at speed `1.25`;
- priority `2`, avoid non-spectator Players within `8`, speeds `1.6/1.4`;
- priority `4`, interval-`40` random swimming at speed `1`, only while not a
  live follower; and
- priority `5`, follow a flock leader.

There is no target, attack, temptation or breeding goal. Water-Bound
navigation, movement control, travel, flop and air handling are identical to
Cod's audited kernel. Water-tag eye fluid first adds Y `0.005`; active
movement lerps speed by `0.125`, adds normalized vertical steering at factor
`0.1` and turns by at most `90` degrees. Water travel applies input `0.01`,
moves, scales velocity by `0.9` and sinks Y `0.005` only without a target.

Outside water while on ground and vertically colliding, AI step consumes two
floats for X/Z `(2f-1)*0.05`, adds Y `0.4000000059604645`, clears on-ground
state, requests impulse sync and plays Tropical-Fish flop. Server air captures
the pre-super value, decrements while alive and dry, and at `-20` resets to
zero before offering `2` Drown damage; water or death resets it to `300`.
Fish step sound is a no-op.

### Eight-member exact-class schools

Tropical Fish inherit maximum school size `8`. A live non-null leader marks a
follower; leader counters increment/decrement on start/stop, following
continues within squared distance `121`, and path requests use speed `1`.
The exact-class query admits every packed variant, so schools formed later by
the follow goal can mix patterns and colors.

Follower addition limits its stream to `8-schoolSize` before filtering out
the leader itself, retaining the capacity-loss quirk. A leader above size one
consumes `nextInt(200)` every tick and only result `1` queries Tropical Fish
in its box inflated by `8`; a list of at most one repairs size to one, while
any unrelated nearby Tropical Fish suppresses repair.

The follow goal has nominal start countdown `200..219`, refuses leaders that
already have followers, admits current live followers immediately and
otherwise selects exact-class nonfollowers/partial leaders after countdown.
Continuation needs a live leader within squared distance `121`; path timer
starts at zero and resets to adjusted `10`. Stopping decrements even a dead
leader through the retained reference. It declares no control flags.

Tropical Fish override the natural-pack stop predicate to return
`!isSchool`, ignoring the count argument. Natural spawning independently
stops at the inherited cluster maximum `8`, so this override makes rare fish
solitary without enlarging common schools.

### Generic-first common/rare finalization

Every finalization first enters schooling finalization. Generic Mob
finalization installs the permanent triangular `random_spawn_bonus`
follow-range modifier when absent and consumes the later float that makes a
Mob left-handed below `0.05`. Schooling then creates plain typed group data
with this fish as leader when input is null, or casts existing data and
attaches this fish to its stored leader.

After those effects, Tropical Fish finalization behaves as follows:

1. Existing `TropicalFishGroupData` supplies its stored variant with no
   variant RNG.
2. Otherwise `nextFloat()<0.9` selects one of the following 22 records with
   `nextInt(22)` and returns new Tropical-Fish group data storing this fish
   and that variant.
3. The remaining `0.1` branch sets `isSchool=false`, independently selects
   one of 12 patterns and two of 16 colors with `nextInt(12/16/16)`, and
   retains the plain schooling group data.
4. The selected triple is packed into slot `17`.

The 22 list positions also select predefined English tooltip names:

| index | predefined name | pattern / base / pattern color |
|---:|---|---|
| 0 | Anemone | Stripey / Orange / Gray |
| 1 | Black Tang | Flopper / Gray / Gray |
| 2 | Blue Tang | Flopper / Gray / Blue |
| 3 | Butterflyfish | Clayfish / White / Gray |
| 4 | Cichlid | Sunstreak / Blue / Gray |
| 5 | Clownfish | Kob / Orange / White |
| 6 | Cotton Candy Betta | Spotty / Pink / Light Blue |
| 7 | Dottyback | Blockfish / Purple / Yellow |
| 8 | Emperor Red Snapper | Clayfish / White / Red |
| 9 | Goatfish | Spotty / White / Yellow |
| 10 | Moorish Idol | Glitter / White / Gray |
| 11 | Ornate Butterflyfish | Clayfish / White / Orange |
| 12 | Parrotfish | Dasher / Cyan / Pink |
| 13 | Queen Angelfish | Brinely / Lime / Light Blue |
| 14 | Red Cichlid | Betty / Red / White |
| 15 | Red Lipped Blenny | Snooper / Gray / Red |
| 16 | Red Snapper | Blockfish / Red / White |
| 17 | Threadfin | Flopper / White / Yellow |
| 18 | Tomato Clownfish | Kob / Red / White |
| 19 | Triggerfish | Sunstreak / Gray / White |
| 20 | Yellowtail Parrotfish | Dasher / Cyan / Yellow |
| 21 | Yellow Tang | Flopper / Yellow / Yellow |

A natural pack begins with null data. The 90% common first member becomes the
leader and every later member copies its exact variant without another
variant draw. Every locked biome asks for exactly eight, so successful common
packs are eight identical fish. A 10% rare first member trips the stop
predicate after one successful spawn.

Supplying plain non-null school data has a separate edge: superclass
finalization first attaches the fish to the old leader, then a common branch
returns new Tropical-Fish group data naming that already-following fish as
the leader for subsequent members. Wrong non-null group data throws during
the superclass cast after generic Mob effects and before the common/rare
draw. Reload restores `isSchool=true` while preserving the packed variant,
so the rare-solitary marker does not persist.

### Placement and baseline natural selection

Tropical Fish register `IN_WATER` with heightmap
`MOTION_BLOCKING_NO_LEAVES`. The placement-type gate requires a non-null
type, world-border inclusion, Water-tag candidate fluid and nonconducting
block above. The species predicate, without RNG, first requires Water-tag
fluid below and exact `Blocks.WATER` above. It then admits either:

- a candidate biome in
  `#minecraft:allows_tropical_fish_spawns_at_any_height`; or
- the ordinary inclusive `seaLevel-13..seaLevel` surface-water range.

The tag contains only Lush Caves, so only that biome bypasses the Y range.
Spawn obstruction later requires the entity to be unobstructed. There is no
light or difficulty predicate.

Exactly five of 66 locked biomes select Tropical Fish in `water_ambient`:
Deep Lukewarm Ocean, Lukewarm Ocean, Warm Ocean, Mangrove Swamp and Lush
Caves. Every row has weight `25` and group `8..8`. The category is
friendly/nonpersistent, cap `20`, distances `32/64`, and cluster/school
maximum `8`. Generic selection, attempts, insertion and cap accounting
retain `MOB-SPAWN-001`.

### Bucket components and tooltip

Capture uses exact Water Bucket and an alive Tropical Fish. After common
bucket fields, `saveToBucketTag` copies pattern, base color and pattern color
in that order into raw item ID `1050` (`tropical_fish_bucket`). Every
captured bucket therefore carries a canonical triple, while the default
bucket report has only empty `minecraft:bucket_entity_data`.

Release constructs with reason `BUCKET`, performs common/rare finalization,
then applies stack configuration. Variant components apply in pattern,
base-color, pattern-color order and overwrite the already-selected result.
Common bucket payload loads afterward, then `FromBucket` becomes true before
insertion and ambient invocation. A componentless bucket exposes the
finalization result. School links and `isSchool` do not round-trip.

The Pattern component itself supplies the bucket tooltip. It reads absent
color components as White and compares the complete triple with the 22
common records. A match emits one gray italic predefined name. Otherwise it
emits a gray italic pattern name and a second gray italic color line; that
line is one color when both references are identical or
`baseColor,patternColor` when distinct (the component appends a literal comma
without an inserted space). Hiding the Pattern component through
`minecraft:tooltip_display` suppresses this projection.

`AbstractFish.mobInteract` retains the eager `Optional.orElse` quirk:
generic Mob interaction evaluates to `PASS` even after successful capture
has replaced the hand and discarded the fish. Exact sounds, hand/inventory
changes, criteria, payload, insertion failure and event order remain with
`ITM-MOB-BUCKET-001`.

### Loot, tags and sounds

The entity loot table has type `entity`, sequence
`minecraft:entities/tropical_fish`. Its first independent pool emits exactly
one raw Tropical Fish, raw item ID `1088`; it has no furnace-smelt or Looting
function. The second independent pool emits one Bone Meal with probability
`0.05`. Eligible death separately emits XP `1..3`.

Tropical Fish belong directly to exactly five entity-type tags:

- `aquatic`, transitively selecting `sensitive_to_impaling`;
- `axolotl_hunt_targets`, allowing a hunt-ready Axolotl to select a visible,
  attackable, in-water fish within squared distance `64`;
- `can_breathe_under_water`;
- `cannot_be_pushed_onto_boats`, preventing collision auto-mount but not the
  physical push; and
- `not_scary_for_pufferfish`, excluding it from the scary-Mob predicate.

Advancements containing the Tropical-Fish string select raw or bucket items,
not the exact entity type. Common Tropical Fish Spawn Egg is raw item ID
`1190`, stack `64`, with
`entity_data.id=minecraft:tropical_fish`; generic Egg construction,
component application, finalization and insertion retain their owner.

Ambient, death, flop and hurt use protocol IDs `1637..1640`; fish swim is ID
`638`. Ambient has an empty sound list and no subtitle. Death uses the four
fish-hurt clips at resource pitch `0.8`; hurt uses those four clips at
default pitch. Flop uses the four fish-flop clips at volume `0.3` and default
pitch. Swim has seven clips. English subtitles are `Tropical Fish dies`,
`Tropical Fish hurts`, `Tropical Fish flops` and `Splashes`. Generic voice
admission, range, volume and runtime pitch retain their owners.

Exact UTF scanning finds zero Tropical-Fish occurrence in all `1,212`
structure templates.

### Legacy migration

Seven exact schema/fix contexts own Tropical-Fish entity or item
compatibility:

- `V1470` registers `minecraft:tropical_fish` as a Mob schema;
- `EntityTheRenameningFix` renames item ID `minecraft:clownfish` to
  `minecraft:tropical_fish`;
- `EntityUUIDFix` includes the current entity in Mob UUID conversion;
- `V705` maps the current Tropical Fish Spawn Egg to its entity shape;
- `ItemStackComponentizationFix` moves legacy bucket-mob fields, including
  `BucketVariantTag`, into `minecraft:bucket_entity_data`;
- `EntitySpawnerItemVariantComponentFix` accepts only numeric
  `BucketVariantTag`, removes it from bucket entity data and writes the three
  pattern/base/pattern-color string components. Its pattern switch defaults
  every unknown low-16 value to Kob and its color converter supplies the
  legacy dye-ID mapping; and
- `TooltipDisplayComponentFix` includes the Pattern component when converting
  legacy `minecraft:hide_additional_tooltip` into tooltip-display hidden
  components.

Non-numeric/missing `BucketVariantTag` is left untouched by the variant
split. No fix creates persistent school or `isSchool` state, and no entity
fix rewrites uppercase `Variant`.

### Two-mesh, two-tint client projection

`EntityRenderers` binds Tropical Fish to `TropicalFishRenderer`, fixed shadow
`0.15`. Render-state extraction decodes slot `17`, stores Pattern plus the
two dye texture-diffuse colors, and defaults a fresh state to Flopper with
both tint integers `-1`. Pattern base selects the small or large base model
and respectively `tropical_a.png` or `tropical_b.png`. The base pass uses
base-color tint. A second colored-cutout layer selects the matching model,
one of twelve pattern textures and pattern-color tint; its duplicate mesh is
expanded by `0.008`.

Both `32×32` meshes use root parts. Small has body
`2×3×6 @ (0,22,0)`, tail `0×3×6 @ (0,22,3)`, two `2×2×0` fins at
`(-1,22.5,0)/(1,22.5,0)` with Y rotations `±0.7853982`, and top fin
`0×3×6 @ (0,20.5,-3)`. Large has body `2×6×6 @ (0,19,0)`, tail
`0×6×5 @ (0,19,3)`, matching side fins at Y `20`, top fin
`0×4×6 @ (0,16,-3)` and bottom fin `0×4×6 @ (0,22,-3)`.

Renderer body yaw is always `4.3*sin(0.6*age)` degrees. Tail yaw is
`-0.45*sin(0.6*age)` in water and
`-0.675*sin(0.6*age)` out of water for both meshes. Dry rendering then
translates `(0.2,0.1,0)` and rotates `90` degrees about positive Z.

All fourteen entity textures are `32×32`. Their byte sizes and SHA-256,
ordered as the renderer dispatches them, are:

| texture | bytes | SHA-256 |
|---|---:|---|
| `tropical_a.png` | 159 | `8039af1f96db7edb991657984be187364ed2bad7bfa7748780ec697106e02c09` |
| `tropical_b.png` | 195 | `152794eb34ba2bb81bcbaa8e0be193481655aef93cfd7d268b8bff1ae428b6fa` |
| `tropical_a_pattern_1.png` | 135 | `95201a0b3a1787e53a7b96e943cce0a237e5e23513b2cd7f488b776c6d2f1b58` |
| `tropical_a_pattern_2.png` | 147 | `763b06293f37f2894f530c0a094450272b3e07c52ca804f89981b6b21b685218` |
| `tropical_a_pattern_3.png` | 148 | `cbe3fb903267ca0a34a336882ffb6873f0de25193207ec315e7bf349c2d95f68` |
| `tropical_a_pattern_4.png` | 148 | `1b26f5b0a5e2f8923c45d65e8c4f13956317f01846538365c5e2e025251b64b6` |
| `tropical_a_pattern_5.png` | 152 | `0fb25e99437ee93a0041dddfd59d73908d60196ba05b4f2d696e2e3bd574a128` |
| `tropical_a_pattern_6.png` | 149 | `fa1d5fe7881ff1f1b96a16ff62c97582fad113049f1fa0f2b6413952b6038a5e` |
| `tropical_b_pattern_1.png` | 166 | `e8e3fbfa8a21be1faac4ac132b68ea36f50036930f170fd1f8062027280af932` |
| `tropical_b_pattern_2.png` | 172 | `14c7d7b73df038a4794e856290608988a92e535ec641b28c1464df3a30cc748f` |
| `tropical_b_pattern_3.png` | 171 | `79399b7742c31ad5cbd72c82c339856c774a074cb847c517505ec8845e570afd` |
| `tropical_b_pattern_4.png` | 178 | `4bf34acff34569d228d63b73c4dfa2322773742a0622ef212884891ede5f32aa` |
| `tropical_b_pattern_5.png` | 166 | `df577694472da03a35fc6391357b708d961a8f71f2da82bc28369038f12c22ba` |
| `tropical_b_pattern_6.png` | 165 | `9a442db8ece0a79501782632e42439e29ee0f6b79b7c4c74d9be700dd1c376f3` |

The generated Spawn-Egg texture is `16×16`, `254` bytes, SHA-256
`867d8af2af0faae40d56003001de55a6aa5a0d1e4b4d6bb9cdcbc205bd50c3f5`.
English names are `Tropical Fish`, `Tropical Fish Spawn Egg` and
`Bucket of Tropical Fish`; the twelve pattern and 22 predefined names are
fixed by the tables above.

**Branches and aborts:**

- Raw metadata remains arbitrary, but semantic reads and persistence default
  unknown fields to Kob/White/White.
- Generic/school finalization precedes the common/rare draw; an existing
  Tropical-Fish group spends no variant RNG.
- Common natural packs copy one variant up to eight; a rare first fish stops
  its pack after one.
- Captured components override the release finalization result; componentless
  buckets preserve it.
- The rare-solitary boolean resets to true on reload.
- Ambient invocation selects a registered event with no clip.

**Constants and randomness:**

Entity/Egg/bucket/raw-item IDs `137/1190/1050/1088`; dimensions/eye
`0.5×0.4/0.26`; tracking/update `4/3`; health/speed/follow
`3/0.7/16`; metadata `16 BOOLEAN, 17 INT`; patterns/colors/common variants
`12/16/22`; common/rare `0.9/0.1`, rare draws `12/16/16`; goals
`0/2/4/5`; school `8/121/8/nextInt(200)==1`, start `200..219`, repath
`10`; movement `0.005/0.125/0.01/0.9`; flop
`±0.05/0.4000000059604645`; air `300/-20/2`; spawn depth `13`,
rows `5/66`, weight/group `25/8`, category `20/32/64`; Bone Meal
`0.05`, XP `1..3`; sounds `1637..1640/638`; tags/templates/migrations
`5/0 of 1212/7`; meshes/textures/deformation/shadow `2/14/0.008/0.15`.

**Side effects:**

Packed variant, `FromBucket`, common durable state and metadata; transient
rare/school state, counters and paths; RNG cursor, motion, impulse and air;
sound, damage, loot and XP; cap/pack state; bucket
hand/discard/components/tooltip/insertion; tag-selected Axolotl, Pufferfish,
boat and Impaling behavior; client mesh, texture and two-pass tint selection.

**Gates:**

Raw integer/component codec validity; logical side and
water/ground/collision/air; leader state/class/distance/countdown; group
subtype, common threshold, modifier presence and RNG; border/fluid/block/
biome-or-Y/cap; bucket/aliveness/components/tooltip hiding; death chance;
tags and client pattern/water state.

**Boundary cases and quirks:**

Arbitrary raw variant bits can be observed on the wire while all semantic
consumers see defaults. A plain-group common finalization can make a
following fish the next group leader. Rare solitude is transient across
reload. Natural common packs are same-variant, but later exact-class follow
selection can mix variants. Stream self-order and unrelated-neighbor stale
repair remain observable. Wrong group data fails before variant RNG.
Captured buckets erase release randomization; componentless buckets expose
it. Ambient is silent.

**Failure semantics:**

Rejected placement prevents natural insertion. Failed insertion does not
roll back finalization or component application under its generic owner.
Invalid NBT defaults the whole variant; arbitrary numeric fields normalize
independently. Rejected Drown damage does not undo air reset. Loot, XP, Egg
and bucket owners retain their commit boundaries.

**Client/server authority split:**

The server owns packed variant, transient rare/school state, AI, placement,
finalization, bucket transfer, damage, loot and XP. Slots `16/17` synchronize
bucket origin and raw packed variant; school and `isSchool` state do not cross
the wire. The client decodes slot `17`, chooses both mesh/texture passes and
applies dye tints and water-dependent tail transforms.

**Observability:**

Observe slots `16/17`, raw versus decoded/canonical variant, uppercase
`Variant`, three component codecs and application order; common/rare and
group RNG cursors; same-variant natural packs versus dynamic mixed schools;
rare reload; movement/flop/air; five-biome selection and Lush-Cave height
bypass; capture/release/tooltip order; loot/XP/tags/Egg;
silent/pitched/default-pitch sounds; zero-template and seven-fix closure;
two base and twelve overlay resources, meshes, tints and transforms.

**Persistence and reload:**

Generic Mob state, `FromBucket` and canonical uppercase integer `Variant`
persist; `isSchool`, school links, paths and counters do not. Code fixes
registration, variant, components, goals, placement and schemas. Biomes,
tags and loot reload through their owners; sounds, language, layers and
textures are client resources.

**Evidence:**

`EntityTypes`, `DefaultAttributes`, `SpawnPlacements`,
`SpawnPlacementTypes`, `MobCategory`, `Mob`, `NaturalSpawner`;
`net.minecraft.world.entity.animal.fish.{WaterAnimal,AbstractFish,AbstractSchoolingFish,TropicalFish}`;
`TropicalFish.{Base,Pattern,Variant,TropicalFishGroupData}`,
`FollowFlockLeaderGoal`, `Bucketable`, `MobBucketItem`, `DataComponents`,
`SoundEvents`; Axolotl, Pufferfish and AbstractBoat consumers; client
`EntityRenderers`, `TropicalFishRenderer`, `TropicalFishRenderState`,
`TropicalFishPatternLayer`, both Tropical-Fish models and
`LayerDefinitions`; the seven migration/schema classes named above;
reports, five biomes, the any-height biome tag, five entity tags, loot, all
1,212 structures, sounds, language, layers and textures. Complete
compiled/data identity searches find no other exact entity runtime path.

**Test vectors:**

Run `EXP-ENT-017` across arbitrary packed fields and NBT/component variants,
common/rare/group finalization, same/mixed eight-member school and reload
cases, movement/flop/air, five-biome and any-height placement, componentful/
componentless capture-release and tooltip ordering, loot/XP/tags/Egg,
templates/migrations/sounds and all mesh/tint/texture/water states.

**Limits:**

Generic entity lifecycle, navigation, damage/death, natural spawning,
despawn, loot, Spawn Egg, bucket transaction, metadata/component packets,
tooltip assembly and render submission retain their owners. Raw Tropical
Fish and Tropical Fish Bucket item behavior retain their item leaves. This
leaf fixes exact Tropical-Fish entity dispatch and every direct join
selecting it.
