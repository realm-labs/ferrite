# Entities mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ENT-SPIDER-001` — Spiders climb, abandon bright fights and finalize into shared-effect packs or skeleton jockeys

**Parent:** `ENT-001`, `ENT-LIFECYCLE-001`, `ENT-002`,
`ENT-VEHICLE-001`, `ENT-004`, `ENT-PROJECTILE-001`, `ENT-005`,
`ENT-DAMAGE-001`, `ENT-BLOCK-001`, `ENT-DAMAGE-REDUCE-001`,
`ENT-KNOCKBACK-001`, `ENT-006`, `ENT-EFFECT-001`, `ENT-007`,
`ENT-DEATH-001`, `MOB-001`, `MOB-AI-001`, `MOB-002`,
`MOB-SPAWN-001`, `MOB-003`, `MOB-DESPAWN-001`, `MOB-005`,
`BLK-COBWEB-001`, `BLK-SPAWNER-001`, `BLK-TRIAL-SPAWNER-001`,
`ITM-STRING-001`, `ITM-SPIDER-EYE-001`, `ITM-ENCHANT-001`,
`PLY-AUTOJUMP-001`, `WGEN-005`, `WGEN-PORTAL-001`,
`WGEN-STRUCTURE-MINESHAFT-001`, `WGEN-STRUCTURE-WOODLAND-MANSION-001`,
`WGEN-JIGSAW-TRIAL-CHAMBERS-001`, `CLI-001`, `CLI-006`,
`CLI-EFFECT-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked registrations, complete `Spider`, its three inner
classes, `CaveSpider`, Mob finalization, placement and natural-insertion paths,
all 66 biomes, three direct tags, both loot tables, four Trial-Spawner
configurations, both structure-spawner sources, Spawn Eggs, ten shared
migration contexts, all 1,212 templates and exact client resources close
protocol entity IDs `22` and `124`.

**Applies when:**

`minecraft:spider` or `minecraft:cave_spider` is constructed, finalized,
naturally selected, emitted by an ordinary or Trial Spawner, spawned by an
egg, command or structure, loaded, moved, climbing, stuck in Cobweb, targeted,
riding or carrying a passenger, affected, attacking, damaged, killed,
synchronized, imitated by a Parrot or rendered.

**Authoritative state:**

Entity protocol ID `124` constructs `Spider` in `MONSTER`. It is unavailable
in Peaceful, has dimensions `1.4×0.9`, explicit eye height `0.65`, one
registered passenger attachment at `0.765`, client tracking range `8` and
default update interval `3`. Its attributes are maximum health `16`, movement
speed `0.30000001192092896`, attack damage `2` and follow range `16`.

Protocol ID `22` constructs `CaveSpider`, a direct Spider subclass, in the
same category and Peaceful gate. It has dimensions `0.7×0.5`, eye height
`0.45`, no registered passenger-attachment list, tracking range `8` and
default update interval `3`. It replaces only maximum health with `12`;
speed, attack damage and follow range remain `0.30000001192092896/2/16`.
Monster construction gives both nominal XP reward `5`.

Spider defines BYTE metadata slot `16`, default `0`. Bit `0` is climbing:
the getter tests `byte&1`, enabling ORs `1`, disabling ANDs `-2`, and both
operations preserve all other bits. Entity, Living and Mob metadata occupies
slots `0..15`; Cave Spider adds no slot.

The climbing byte has no dedicated save/load hook. Every server tick runs
inherited Monster tick first, then replaces bit `0` with the current
`horizontalCollision` Boolean. Clients do not recompute it. `onClimbable`
returns that synchronized bit and selects inherited climb/fall handling;
navigation is `WallClimberNavigation`.

Neither subtype adds a saved scalar. Inherited health, effects, equipment,
target-related Mob state and passenger graph retain their generic owners.
Climbing, goal counters and spawn-group selection are transient. A selected
infinite effect persists afterward as an ordinary active effect, and an
accepted skeleton jockey persists through the ordinary passenger graph.

For a carried entity no wider than the vehicle, Spider returns vehicle
attachment `(0,0.3125*scale,0)` and Cave Spider returns
`(0,0.21875*scale,0)`. A wider entity delegates to the generic attachment
calculation; the Cave override delegates through Spider, whose dynamic width
test still compares against the Cave Spider's `0.7`.

**Transition and ordering:**

### Goal graph, light gate and melee

Both subtypes register this exact goal selector:

- priority `1`: `FloatGoal`;
- priority `2`: avoid an unscared Armadillo within `6`, at speed `1` far and
  `1.2` near;
- priority `3`: `LeapAtTargetGoal`, vertical impulse `0.4`;
- priority `4`: Spider melee at speed `1`, with long memory;
- priority `5`: `WaterAvoidingRandomStrollGoal`, speed `0.8`;
- priority `6`: look at a Player within `8`; and
- priority `6`: random look around.

Its target selector is priority `1` `HurtByTargetGoal` without
`setAlertOthers`, then priority `2` Spider target search for Players and
priority `3` the same search for Iron Golems. Each search has required sight,
no reachability requirement, follow range `16` and the default ten-tick
target-search cadence.

The two custom target goals read
`getLightLevelDependentMagicValue()` first. A value at least `0.5` returns
false without entering generic target search and therefore without spending
its cadence RNG. A lower value delegates to the ordinary nearest-attackable
search.

The melee goal can start only when generic melee admission passes and the
Spider is not a vehicle. A Spider carrying its generated Skeleton therefore
cannot start this attack while the passenger remains, although a Spider
riding another entity is not rejected by this test alone.

For continuation it reads the same light value. At least `0.5` consumes
`nextInt(100)`; zero clears the current target and stops immediately, while
the other 99 results delegate to generic long-memory melee continuation.
Below `0.5` it spends no such draw and delegates directly. Thus bright light
prevents fresh Player/Iron-Golem acquisition and gives an already active
fight a per-continuation-test one-percent abandonment path. Retaliation can
still supply a target independently; generic goal arbitration, paths,
attack reach/cooldown, damage, enchantments and knockback remain with
`MOB-AI-001`, `ENT-DAMAGE-001`, `ENT-KNOCKBACK-001` and
`ITM-ENCHANT-001`.

### Climbing, Cobweb and Poison

Horizontal collision is sampled only after inherited server tick, so the
new bit affects later observation and subsequent climb processing rather
than retroactively changing the completed inherited step. Client packets
project the byte's dirty state. Any qualifying horizontal collision can set
the bit; this is not a wall-block tag test.

`makeStuckInBlock` returns immediately when the exact state is Cobweb and
otherwise delegates. Spiders therefore skip Cobweb's generic stuck
multiplier while Cobweb placement, destruction and other entities remain
with `BLK-COBWEB-001`.

`canBeAffected` rejects an effect whose holder is exactly Poison before
generic effect admission. Every duration and amplifier is rejected, while
all other effects delegate. This immunity composes with the direct
`arthropod` tag, whose locked members are Bee, Endermite, Silverfish, Spider
and Cave Spider and which selects data-driven Bane-of-Arthropods effects.

Both types also directly join `no_anger_from_wind_charge`, suppressing the
generic Wind-Charge anger-attribution path. Spider alone joins
`dismounts_underwater`; generic underwater vehicle handling can therefore
dismount its passengers, while Cave Spider lacks that membership.

### Cave-Spider attack and finalization divergence

Cave Spider first performs the inherited melee transaction. A false result
returns false and offers no effect. After a successful hit, a living victim
gets a fresh amplifier-zero Poison offer for:

- `140` ticks on Normal;
- `300` ticks on Hard; and
- zero ticks on Peaceful or Easy.

Only a positive duration calls `addEffect`, with the Cave Spider as source.
The admission result is ignored and the attack still returns true, so
immunity, a stronger current effect or another rejection cannot undo
successful damage. Spider and Cave Spider themselves reject this Poison
offer through their shared immunity.

Ordinary Spider finalization first runs generic Mob finalization. For a fresh
Spider this installs the permanent `random_spawn_bonus` follow-range
modifier with triangular amount centered at `0` and deviation
`0.11485000000000001`, then consumes one float and makes it left-handed only
when the value is below `0.05`. An already-present modifier skips only the
triangle draw.

Cave Spider overrides finalization by returning its supplied group data
unchanged. It does not call Spider or Mob. Cave Spiders therefore receive no
random follow-range modifier or left-handed roll and can never enter the
Spider jockey or special-effect branches, regardless of spawn reason or
difficulty.

### Skeleton jockey and shared special effects

After generic finalization, every ordinary Spider consumes
`nextInt(100)`. On zero it:

1. creates an exact Skeleton with construction reason `JOCKEY`;
2. if construction succeeds, snaps it to Spider X/Y/Z and yaw, with pitch
   `0`;
3. finalizes the Skeleton at the same local difficulty but with the
   Spider's original spawn reason and null group data; and
4. calls `startRiding(spider,false,false)`.

The finalization and riding results are ignored. The method does not insert
the Skeleton directly; standard natural, chunk-generation and spawner paths
subsequently insert the Spider with passengers, making an accepted rider
part of that same root transaction. Failure to construct or attach leaves no
passenger, but does not reroll.

If the post-generic group data is null, Spider next creates
`SpiderEffectsGroupData`. Only for this new object, on exact Hard difficulty,
it draws one float and admits special selection when

`value < 0.1*localDifficulty.specialMultiplier`.

Admission consumes `nextInt(5)`: values `0|1` select Speed, `2` Strength,
`3` Regeneration and `4` Invisibility. Finally, every Spider receiving that
group-data type and a non-null selection offers itself an amplifier-zero
effect with duration `-1`; the offer result is ignored.

Natural spawning passes the returned group data through the pack. Its first
ordinary Spider therefore selects either no effect or one effect, and every
later member receives that same outcome without another chance/effect draw;
the one-percent Skeleton draw still occurs independently for every member.
Ordinary spawners pass null to each finalization, so separate Spider
emissions select independently. Cave Spider's no-op finalizer keeps natural
group data null and performs none of these draws.

### Placement and baseline natural selection

Both types register `ON_GROUND`, `MOTION_BLOCKING_NO_LEAVES`, and the standard
darkness Monster predicate. The outer placement gate requires world-border,
valid support and empty candidate/above blocks. Outside spawn reasons that
ignore light, the predicate first requires sky brightness no greater than
`nextInt(32)`, then the dimension block-light limit, then raw brightness no
greater than the dimension's sampled monster-light threshold; thunder uses
the darkened local-brightness path. Generic Mob support and the non-Peaceful
type gate still apply.

Spider occurs in exactly 52 of 66 biome Monster lists, always at weight `100`
and fixed group `4..4`. The 14 exclusions are `basalt_deltas`,
`crimson_forest`, `deep_dark`, `end_barrens`, `end_highlands`,
`end_midlands`, `mushroom_fields`, `nether_wastes`, `small_end_islands`,
`soul_sand_valley`, `sulfur_caves`, `the_end`, `the_void` and
`warped_forest`.

Cave Spider occurs naturally only in `sulfur_caves`, at weight `20` and fixed
group `1..1`. Both otherwise use Monster category cap `70`, inherited
per-cluster maximum `4`, hostile/nonpersistent classification and
no-despawn/despawn distances `32/128`.

### Ordinary and Trial Spawners

`WGEN-STRUCTURE-MINESHAFT-001` owns the code-built Cave-Spider spawner. A
nonrail corridor has a `1/23` spider-mode draw; an eligible bay selects one
of three offsets and commits its persisted latch before offering a spawner.
Only a resulting typed spawner is configured for Cave Spider, and later
ordinary-spawner delay, nearby-count, collision, finalization and insertion
remain with `BLK-SPAWNER-001`.

Woodland-Mansion template `1x1_as2.nbt` contains one fixed Spider spawner at
local `(4,1,3)`: delay `0`, minimum/maximum delay `200/800`, spawn count `4`,
maximum nearby `6`, required player range `16` and spawn range `4`. Its
SpawnData and sole weight-one potential produce id-only Spider. Mansion room
selection and template placement remain with
`WGEN-STRUCTURE-WOODLAND-MANSION-001`.

Four locked Trial-Spawner configurations contain the family:

- `trial_chamber/melee/spider/normal` and
  `trial_chamber/small_melee/cave_spider/normal` each use total
  `6+2p`, simultaneous `3+0.5p` and interval `20`;
- their ominous forms each use total `12+2p`, simultaneous `4+0.5p`
  and the default interval `40`.

Here `p=max(0,registeredPlayers-1)` and flooring occurs after each sum. Every
potential list contains only the corresponding id-only entity at weight `1`;
neither ominous record adds equipment. Ominous ejection selects key weight
`3` or consumables weight `7`. All four retain spawn range `4`, required
range `14` and cooldown `36,000` from defaults.

The normal keys occur once respectively in
`trial_chambers/spawner/melee/spider.nbt` and
`trial_chambers/spawner/small_melee/cave_spider.nbt`. Alias selection maps
the virtual melee contents equally among Zombie, Husk and Spider, and small
melee equally among Slime, Cave Spider, Silverfish and Baby Zombie.
`WGEN-JIGSAW-TRIAL-CHAMBERS-001` owns those start-scoped draws and
`BLK-TRIAL-SPAWNER-001` owns encounter execution.

Exact scans of all 1,212 templates find zero exact
`minecraft:cave_spider` payloads. Exact `minecraft:spider` occurs twice,
both in the Mansion spawner's SpawnData/potential; the longer Trial config
keys are counted separately. The Trial templates also contain their
separately owned Cobweb and mushroom decoration.

### Death, loot, progression and sounds

The two entity loot tables have equivalent two-pool layouts but independent
named sequences, `minecraft:entities/spider` and
`minecraft:entities/cave_spider`. The first one-roll pool always samples
String base `0..2`, then optionally grows it by
`round(L*U[0,1))` for a living attacker with positive Looting level `L`.

The second one-roll pool runs only for a player-attributed kill. It samples
Spider Eye base `B=-1..1`, normalizes a nonpositive stored base through stack
emptiness, then applies the analogous optional bonus, giving final count
`max(B,0)+round(L*U[0,1))`. The String pool and its draws always precede the
Eye condition. Exact count mechanics and world-drop placement remain with
`ITM-STRING-001`, `ITM-SPIDER-EYE-001`, `ITM-LOOT-001` and
`ENT-ENTITY-DROPS-001`.

Both nominally drop `5` XP under generic admission. A player kill satisfies
its exact OR criterion in `adventure/kill_a_mob` and its distinct AND
requirement in `adventure/kill_all_mobs`; there is no other entity-identity
advancement branch.

Both return sound events `entity.spider.ambient`, `.hurt`, `.death` and
`.step`, protocol IDs `1578..1581`. Step playback is explicit volume
`0.15`, pitch `1`; the other three retain generic voice admission and pitch.
English subtitles are `Spider hisses`, `Spider hurts` and `Spider dies`;
Step uses generic subtitle `Footsteps`. Parrot maps both entity types to
shared `entity.parrot.imitate.spider`, protocol sound ID `1241`, subtitle
`Parrot hisses`; that sound definition references the Spider ambient event
at pitch `1.8` and volume `0.6`.

### Legacy schema and client projection

Exactly ten migration/schema contexts jointly own both identities:

- `EntityHealthFix` recognizes legacy `Spider` and `CaveSpider`;
- `EntityIdFix` maps them to `minecraft:spider` and
  `minecraft:cave_spider`;
- `EntityUUIDFix` processes both modern Mob shapes;
- `ItemSpawnEggFix` maps legacy generic Spawn Egg damage `52/59` to
  Spider/CaveSpider;
- `ItemStackSpawnEggFix` maps the modern entities to their dedicated Egg
  identities;
- `StatsCounterFix` recognizes their old statistics;
- `V99` registers both legacy simple entities;
- `V705` and `V1460` register their modern Mob/Spawn-Egg shapes; and
- `TrialSpawnerConfigInRegistryFix.VanillaTrialChambers` maps both old
  inline normal/ominous Trial configurations to their registry keys.

Legacy Egg damages `52/59` are unrelated to current entity protocol IDs
`124/22`. No migration rewrites climbing or group selection because neither
is subtype-persisted.

Current Spider and Cave-Spider Spawn Eggs are raw item IDs `1217/1216`.
Their common 64-stack components carry entity-data IDs for their respective
types and direct model keys; generic Egg interaction/finalization remains
with the spawn owners. English names are `Spider Spawn Egg` and
`Cave Spider Spawn Egg`. Each generated model selects its same-named
`16×16` texture; Spider/Cave files are respectively `223/219` bytes with
SHA-256
`307bfe5f1740313b1ecb15eccdd391da8ca0be3fd31322bb0ca6679f877022ff` and
`deb9210ca1b20e1f0ec0b173469df56cc194d285df8f86d417632e0b8587814c`.

`EntityRenderers` binds Spider and Cave Spider to `SpiderRenderer` and
`CaveSpiderRenderer`. Both use the same eleven-part, `64×32` Spider mesh
(head, two body segments and eight legs), the same Living render state and
the same emissive `spider_eyes` layer. Spider shadow radius is `0.8`; Cave
Spider applies mesh scale `0.7` and shadow radius `0.56`. Both death-flip
angles are `180`.

The model maps head yaw/pitch degrees directly through `π/180`. Let
`q=0.6662*walkAnimationPos` and `s=walkAnimationSpeed`. Four mirrored leg
pairs add yaw terms `-0.4s*cos(2q+phase)` and roll terms
`0.4s*abs(sin(q+phase))`, with phases `0,π,π/2,3π/2`, on top of their baked
angles. The climbing byte supplies no separate render-state field or
animation branch.

English entity names are `Spider` and `Cave Spider`. Their base textures are
`textures/entity/spider/spider.png` and `cave_spider.png`; the emissive layer
uses `spider_eyes.png`. All are `64×32`, with respective byte counts
`646/648/130` and SHA-256 values
`12771d3524ad137812b2a94d52b6943401c5e0a942baa6946e783e822c46c554`,
`02386867edcf8af7d4a205360800991ab6be475dc4c80752c9145efb2699a8ac`
and `45bc67083660ea8257b65c849dc1bb457be7ec56ad8fe8951adc9cb221f52ecd`.

**Branches and aborts:**

Subtype and protocol registration; metadata byte/bit/collision/side; passenger
width and water dismount; Armadillo fear; light below/equal/above `0.5`,
target class/cadence/sight and melee vehicle/continuation draw; Cobweb/exact
Poison; generic attack success, victim class, difficulty and effect admission;
generic modifier presence, handedness, Skeleton construction/riding/insertion,
group-data type/null/effect and Hard chance; placement reason/light/support,
biome/category/group/cluster/despawn; mineshaft/Mansion/Trial selection and
spawner outcomes; attacker/player/Looting/base counts; criteria, tags,
migration shape, sound and client render/model/resource state.

**Constants and randomness:**

Entity IDs Spider/Cave `124/22`; Egg IDs `1217/1216`; dimensions
`1.4×0.9/0.7×0.5`; eyes `0.65/0.45`; health `16/12`; shared
speed/attack/follow/XP `0.30000001192092896/2/16/5`; tracking/update `8/3`;
metadata slot/bit `16/1`; vehicle Y `0.3125/0.21875`; goals
`1,2,3,4,5,6,6`, targets `1,2,3`; Armadillo `6/1/1.2`, leap `0.4`,
stroll `0.8`, look `8`; light `0.5`, abandonment `1/100`; Cave Poison
`0/140/300`; generic finalizer triangle `0/0.11485000000000001`, left hand
`0.05`; jockey `1/100`; special chance `0.1*multiplier`, effect weights
`2/1/1/1`, duration `-1`; biome rows `52/1`, weights/groups
`100/4..4` and `20/1..1`; Trial normal/ominous
`6+2p,3+.5p,20` / `12+2p,4+.5p,40`; loot String `0..2`, Eye `-1..1`,
optional `round(LU)`; sounds `1578..1581/1241`; mesh/shadows
`64×32/0.8/0.56/0.7`.

**Side effects:**

Goal, target, navigation and attack state; target clearing and RNG cursor;
climbing metadata/dirty packets and inherited fall/movement state; ignored
Cobweb slowdown and Poison offers; follow-range modifier, handedness,
Skeleton passenger/finalization, group data and active effects; natural and
three spawner-source entity insertion; loot/XP/criteria; Parrot sound; wire,
name, model, texture and emissive projection.

**Gates:**

Logical side and horizontal collision; exact block/effect/tag/type; target
life/attackability/sight/light and vehicle state; attack damage/effect
admission/difficulty; spawn reason/group data/local difficulty/RNG and
passenger insertion; Peaceful/world border/support/light/biome/cap/cluster/
despawn; mineshaft latch, Mansion/template and Trial player/omen/gamerule/
collision state; death attacker/player/Looting; migration schema and client
resource/render state.

**Boundary cases and quirks:**

Equality at light `0.5` is bright for both acquisition and abandonment.
Climbing mirrors collision after inherited tick and is neither persisted nor
wall-tag-based. Both types ignore Cobweb slowdown and Poison, so Cave Spider
cannot Poison either family member. Cave's no-op finalizer skips even generic
follow-range and handedness state. Spider's Skeleton is constructed as
`JOCKEY` but finalized with the caller's reason; a jockey Spider cannot begin
its own melee goal. The first natural pack member fixes one shared special
effect or shared absence, while each member still rolls a jockey. Trial
ominous records omit interval and therefore use default `40`, not the normal
records' explicit `20`.

**Failure semantics:**

A bright target search stops before generic RNG/search. A zero abandonment
draw clears the target before returning false. Failed generic damage offers
no Cave Poison; rejected Poison leaves successful damage committed. Skeleton
construction/riding results and special-effect admission are ignored.
Invalid placement, caps, obstruction or insertion prevent that owning spawn
transaction according to the generic owner. The mineshaft latch can remain
committed after a failed spawner write. Non-player death skips the Eye pool;
nonpositive final stacks disappear. Missing data/resources remove future
selection/projection without inventing subtype fallback.

**Client/server authority split:**

Server code owns AI, collision sampling, metadata mutation, attacks/effects,
finalization, spawning, passengers, loot, criteria and authoritative sounds.
Clients consume synchronized climbing and inherited state, but the model has
no climb-specific branch. Client resources own names, model geometry,
animation, base textures, emissive eyes and Egg projection; they cannot
change collision, immunity, target, damage, spawn or loot authority.

**Observability:**

Observe registrations/attributes; slot-16 byte and dirty timing across
horizontal-collision transitions; path/fall behavior; every goal priority,
light boundary and RNG cursor; Cobweb and all Poison admissions; Cave attack
return/effect order; generic/Cave finalization and pack group sharing;
Skeleton reason/passenger/insertion; every biome and all ordinary/Trial
spawner paths; tags, loot/XP/criteria, Eggs/templates/migrations; sound IDs,
Parrot mapping, mesh scale, shadows, gait, textures and emissive eyes.

**Persistence and reload:**

Inherited entity/Mob/effect/passenger state persists normally. Slot-16
climbing, spawn group data and goal counters do not; climbing is recomputed,
whereas an applied infinite effect or passenger survives through its owner.
Code fixes registration, goals, finalization and migration. Biomes, tags,
loot, advancements and Trial configurations reload through their owners;
templates affect newly placed structures. Language, item models and textures
reload client-side.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-DATA-001`; `OFF-REPORT-001`;
`net.minecraft.world.entity.EntityTypes`;
`net.minecraft.world.entity.ai.attributes.DefaultAttributes`;
`net.minecraft.world.entity.SpawnPlacements`;
`net.minecraft.world.entity.monster.spider.Spider`;
`net.minecraft.world.entity.monster.spider.Spider$SpiderAttackGoal`;
`net.minecraft.world.entity.monster.spider.Spider$SpiderTargetGoal`;
`net.minecraft.world.entity.monster.spider.Spider$SpiderEffectsGroupData`;
`net.minecraft.world.entity.monster.spider.CaveSpider`;
`net.minecraft.world.entity.monster.Monster`;
`net.minecraft.world.entity.Mob`;
`net.minecraft.world.entity.ai.goal.target.NearestAttackableTargetGoal`;
`net.minecraft.world.level.NaturalSpawner`;
`net.minecraft.world.level.BaseSpawner`;
`net.minecraft.world.level.levelgen.structure.structures.MineshaftPieces$MineShaftCorridor`;
`net.minecraft.world.level.levelgen.structure.structures.WoodlandMansionPieces`;
`net.minecraft.world.level.block.entity.trialspawner.TrialSpawnerConfigs`;
`net.minecraft.world.entity.animal.parrot.Parrot`;
`net.minecraft.sounds.SoundEvents`;
`net.minecraft.util.datafix.fixes.EntityHealthFix`;
`net.minecraft.util.datafix.fixes.EntityIdFix`;
`net.minecraft.util.datafix.fixes.EntityUUIDFix`;
`net.minecraft.util.datafix.fixes.ItemSpawnEggFix`;
`net.minecraft.util.datafix.fixes.ItemStackSpawnEggFix`;
`net.minecraft.util.datafix.fixes.StatsCounterFix`;
`net.minecraft.util.datafix.fixes.TrialSpawnerConfigInRegistryFix`;
`net.minecraft.util.datafix.schemas.V99`, `V705` and `V1460`;
`net.minecraft.client.renderer.entity.EntityRenderers`;
`net.minecraft.client.renderer.entity.SpiderRenderer`;
`net.minecraft.client.renderer.entity.CaveSpiderRenderer`;
`net.minecraft.client.renderer.entity.layers.SpiderEyesLayer`;
`net.minecraft.client.model.geom.LayerDefinitions`;
`net.minecraft.client.model.monster.spider.SpiderModel`;
`reports/registries.json#minecraft:{entity_type,item,sound_event}`;
`reports/minecraft/components/item/{spider,cave_spider}_spawn_egg.json`;
`data/minecraft/tags/entity_type/{arthropod,dismounts_underwater,no_anger_from_wind_charge}.json`;
`data/minecraft/loot_table/entities/{spider,cave_spider}.json`;
`data/minecraft/trial_spawner/trial_chamber/{melee/spider,small_melee/cave_spider}/{normal,ominous}.json`;
`data/minecraft/worldgen/biome/*.json`;
`data/minecraft/advancement/adventure/{kill_a_mob,kill_all_mobs}.json`;
`data/minecraft/structure/**/*.nbt`;
`assets/minecraft/{items,models/item,textures/item}/{spider,cave_spider}_spawn_egg.*`;
`assets/minecraft/textures/entity/spider/{spider,cave_spider,spider_eyes}.png`;
`assets/minecraft/{sounds,lang/en_us}.json`;
`ENT-LIFECYCLE-001`; `ENT-VEHICLE-001`; `ENT-DAMAGE-001`;
`ENT-EFFECT-001`; `ENT-DEATH-001`; `ENT-ENTITY-DROPS-001`;
`MOB-AI-001`; `MOB-SPAWN-001`; `MOB-DESPAWN-001`;
`BLK-COBWEB-001`; `BLK-SPAWNER-001`; `BLK-TRIAL-SPAWNER-001`;
`ITM-STRING-001`; `ITM-SPIDER-EYE-001`; `ITM-ENCHANT-001`;
`WGEN-STRUCTURE-MINESHAFT-001`;
`WGEN-STRUCTURE-WOODLAND-MANSION-001`;
`WGEN-JIGSAW-TRIAL-CHAMBERS-001`; `CLI-006`; `CLI-EFFECT-001`.

**Test vectors:**

Run `EXP-ENT-014` across metadata/collision/climb timing, both attachment
width branches, the entire goal graph and light boundaries, Cobweb and effect
admission, every Cave attack difficulty/outcome, generic versus no-op
finalization, shared/null/adversarial group data, all jockey creation/riding/
insertion outcomes, darkness placement and 66-biome census, mineshaft,
Mansion and four Trial configs, loot/XP/criteria, exact tags, Eggs,
templates/migrations, sounds/Parrot and model/texture/emissive projection.

**Limits:**

Generic entity lifecycle, goal arbitration/navigation/melee, damage/effect/
death, natural spawning/despawn, ordinary and Trial Spawners, structure
placement, loot evaluation, Spawn Egg interaction, metadata packets,
passenger mechanics and rendering remain with the cited owners. Their
algorithms are included here only where an exact Spider-family override,
input, ordering join or observable consequence selects them.
