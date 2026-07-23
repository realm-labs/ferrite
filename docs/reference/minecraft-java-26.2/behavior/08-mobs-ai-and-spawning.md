# 08 — Mob Spawning, Despawning, AI, and Husbandry

Concrete mobs' spawn placement, biome list, goals, sensors, memories, breeding items, and taming
chance come from type implementations and `OFF-DATA-001`. This page does not flatten every mob into
one AI.

## `MOB-001` Natural spawning applies both global category and per-player local caps

- **FidelityClass:** `ExactObservableBehavior`
- **Evidence status:** `Confirmed`

### Primary evidence

`OFF-SERVER-001`;
`net.minecraft.world.level.NaturalSpawner#createState(int,java.lang.Iterable,net.minecraft.world.level.NaturalSpawner$ChunkGetter,net.minecraft.world.level.LocalMobCapCalculator)`;
`net.minecraft.world.level.NaturalSpawner#getFilteredSpawningCategories(net.minecraft.world.level.NaturalSpawner$SpawnState,boolean,boolean)`;
`net.minecraft.world.level.LocalMobCapCalculator#canSpawn(net.minecraft.world.entity.MobCategory,net.minecraft.world.level.ChunkPos)`;
`net.minecraft.world.entity.MobCategory#getMaxInstancesPerChunk()`; `COM-WIKI-MOB-001`

### Applies when

The server's active-chunk phase considers natural spawning for a `MobCategory`.

### Behavior and timing

Each round first counts existing entities into `SpawnState`. The global category cap scales its
per-category max by spawnable chunks relative to `289`; a category at cap leaves the round's
candidate list. A local cap also counts per non-spectator player near a candidate chunk. Each
successful spawn immediately increments relevant counts and affects later attempts in the same
round.

### Boundaries and quirks

Persistent categories, misc entities, structures, spawners, and chunk-generation spawning need not
use the same cap. Overlapping player regions affect multiple local counts. One world-wide
`mob_count` is insufficient.

### Verification

**Owners:** `MOB-SPAWN-001`, `MOB-HOSTILE-GATE-001`, `MOB-PATROL-001`,
`MOB-PHANTOM-SPAWN-001`, `MOB-WANDERING-TRADER-001`, `MOB-WARDEN-SPAWN-001`, `MOB-RAID-001`;
`EXP-MOB-*`

Lock spawnable-chunk boundary, rounding formula, overlapping-player local counts, and whether
same-tick removal enters the initial snapshot. Also lock the startup/live `spawnEnemies` projection,
its natural-category effect and custom-spawner consumers. `MOB-RAID-001` owns the distinct
event-spawned raider waves, membership and completion lifecycle rather than natural caps.

## `MOB-002` Natural spawning makes pack attempts and fully validates every individual

- **FidelityClass:** `ExactObservableBehavior`
- **Evidence status:** `Confirmed`

### Primary evidence

`OFF-SERVER-001`;
`net.minecraft.world.level.NaturalSpawner#spawnForChunk(net.minecraft.server.level.ServerLevel,net.minecraft.world.level.chunk.LevelChunk,net.minecraft.world.level.NaturalSpawner$SpawnState,java.util.List)`;
`net.minecraft.world.level.NaturalSpawner#spawnCategoryForChunk(net.minecraft.world.entity.MobCategory,net.minecraft.server.level.ServerLevel,net.minecraft.world.level.chunk.LevelChunk,net.minecraft.world.level.NaturalSpawner$SpawnPredicate,net.minecraft.world.level.NaturalSpawner$AfterSpawnCallback)`;
`net.minecraft.world.level.NaturalSpawner#isValidSpawnPostitionForType(net.minecraft.server.level.ServerLevel,net.minecraft.world.entity.MobCategory,net.minecraft.world.level.StructureManager,net.minecraft.world.level.chunk.ChunkGenerator,net.minecraft.world.level.biome.MobSpawnSettings$SpawnerData,net.minecraft.core.BlockPos$MutableBlockPos,double)`;
`net.minecraft.world.level.NaturalSpawner#isValidEmptySpawnBlock(net.minecraft.world.level.BlockGetter,net.minecraft.core.BlockPos,net.minecraft.world.level.block.state.BlockState,net.minecraft.world.level.material.FluidState,net.minecraft.world.entity.EntityType)`

### Applies when

`MOB-001` allows a category to attempt in a selected natural-spawn chunk.

### Behavior and timing

A random chunk position starts several offset candidates for a pack. Each candidate reselects
biome/structure spawn entry and checks player/world-spawn distance, chunk/border, block/fluid
emptiness, entity-type placement, collision, light/difficulty, and the mob's own
`checkSpawnRules`/`checkSpawnObstruction`. Success invokes `finalizeSpawn`, adds the entity, and
updates caps. Pack/cap limits terminate attempts.

### Boundaries and quirks

Generic natural spawning rejects candidates within `24` blocks of the nearest player or world spawn
and constrains candidates to at most `8` chunks / `128` blocks; structure overrides and concrete
types still alter the spawn list. Other spawn reasons bypass different subsets.

### Verification

**Owners:** `MOB-SPAWN-001`, `BLK-TINTED-GLASS-001`, `BLK-GLASS-001`,
`BLK-STAINED-GLASS-001`, `BLK-CONCRETE-001`, `BLK-TERRACOTTA-001`,
`BLK-GLAZED-TERRACOTTA-001`, `BLK-QUARTZ-001`, `BLK-SANDSTONE-001`,
`BLK-STONE-VARIANT-001`, `BLK-STONE-BRICK-001`, `BLK-SLIME-001`,
`BLK-HONEY-001`, `BLK-SOUL-SAND-001`, `BLK-MAGMA-001`, `BLK-LAVA-CAULDRON-001`;
`EXP-MOB-*`, `EXP-BLK-033`, `EXP-BLK-034`, `EXP-BLK-035`, `EXP-BLK-036`, `EXP-BLK-037`,
`EXP-BLK-038`, `EXP-BLK-039`, `EXP-BLK-040`, `EXP-BLK-041`, `EXP-BLK-042`, `EXP-BLK-043`,
`EXP-BLK-044`, `EXP-BLK-045`, `EXP-BLK-046`, `EXP-BLK-047`

Extract attempts/pack termination, surface selection, and special-structure overrides per
category/type into fixtures.
`BLK-TINTED-GLASS-001` fixes its copied never-spawn support predicate and the separate exact
`LEGACY_IRON_GOLEM` floor rejection used by villager summon searches; generic candidate traversal,
above-cell checks and summon commit stay with the mob owners.
`BLK-GLASS-001` fixes the corresponding registered never-spawn predicate and the strategy's
separate exact plain-glass rejection under the same traversal and commit owners.
`BLK-STAINED-GLASS-001` fixes the never-spawn predicate for all sixteen colors and the strategy's
class-wide `StainedGlassBlock` rejection before above-cell/solidity checks.
`BLK-CONCRETE-001` fixes the opposite ordinary-block boundary for all sixteen colors: the full
sturdy upper face and emission 0 pass the default spawn-support predicate, while entity-specific
placement checks remain here.
`BLK-TERRACOTTA-001` fixes the same generic full-sturdy/emission-0 support for all seventeen
identities. Armadillo's additional floor tag accepts only plain, white, yellow, orange, red, brown
and light-gray terracotta through `badlands_terracotta`, then independently requires sufficient
brightness; caps, biome selection, pack traversal and insertion remain here.
`BLK-GLAZED-TERRACOTTA-001` fixes ordinary full-sturdy/emission-0 spawn support for every color and
facing. Its grouping tag adds no entity-specific placement exception; those predicates remain with
the entity-type owners.
`BLK-QUARTZ-001` fixes the same ordinary full-sturdy/emission-0 support for all seven full-cube
states. Pillar axis and the five items' slow-bouncy sulfur-cube membership do not alter the
world-block spawn-support predicate; entity-specific gates remain with entity owners.
`BLK-SANDSTONE-001` fixes the same ordinary full-sturdy/emission-0 support for all eight
full-cube states. Color, strength profile and the items' slow-bouncy sulfur-cube membership do not
alter the world-block spawn-support predicate; entity-specific gates remain with entity owners.
`BLK-STONE-VARIANT-001` fixes the same ordinary full-sturdy/emission-0 support for states 2..7.
Only raw granite, diorite and andesite additionally enter `bats_spawnable_on` through
`base_stone_overworld`; bat height, random, brightness and generic mob gates remain with their
owners.
`BLK-STONE-BRICK-001` fixes ordinary full-sturdy/emission-0 support for all four states. Matching
infested hosts can spawn silverfish under `BLK-BREAK-HOOK-001`, but that separate block callback
does not alter the ordinary hosts' spawn-support predicate.
`BLK-SLIME-001` fixes the opposite inherited boundary: its full sturdy top face and zero emission
pass the default support predicate, with entity-specific placement checks remaining here.
`BLK-HONEY-001` fixes a reduced support shape that fails the default full-top-face spawn predicate;
its snow-layer override is a separate tag consumer and does not grant entity spawn support.
`BLK-SOUL-SAND-001` deliberately registers an always-true spawn predicate despite its shortened
collider. Entity-type placement, light, collision and category-specific admission remain here.
`BLK-MAGMA-001` has a full support cube but its registered spawn predicate admits only fire-immune
entity types; every remaining placement, collision, light and category gate remains here.
`BLK-LAVA-CAULDRON-001` has only a rim at the top and keeps the default spawn predicate, so it
does not provide a full sturdy upper face; remaining entity/category admission stays here.

## `MOB-003` Despawning combines persistence, player distance, category ranges, and random checks

- **FidelityClass:** `ExactObservableBehavior`
- **Evidence status:** `Confirmed`

### Primary evidence

`OFF-SERVER-001`; `net.minecraft.world.entity.Mob#checkDespawn()`;
`net.minecraft.world.entity.Mob#requiresCustomPersistence()`;
`net.minecraft.world.entity.Mob#isPersistenceRequired()`;
`net.minecraft.world.entity.Mob#removeWhenFarAway(double)`;
`net.minecraft.world.entity.MobCategory#getNoDespawnDistance()`;
`net.minecraft.world.entity.MobCategory#getDespawnDistance()`; `COM-WIKI-MOB-001`

### Applies when

A non-player mob's server AI tick checks natural despawn.

### Behavior and timing

Peaceful difficulty may first remove hostile mobs that should not exist. Required or type-specific
persistence bypasses ordinary despawn. Otherwise nearest-player distance is used: beyond the
category hard despawn distance and when the type permits, remove immediately; beyond the fixed
`32`-block no-despawn distance, `noActionTime` plus random chance may despawn; proximity resets
idleness.

### Boundaries and quirks

Naming, taming, breeding state, riding/passengers, held/equipped items, and special spawn reasons
may require persistence. Chunk unload into storage is not natural despawn.

### Verification

**Owners:** `MOB-DESPAWN-001`; `EXP-MOB-003`

Audit `requiresCustomPersistence` and `removeWhenFarAway` overrides per type; lock random frequency
and exact-threshold positions.

## `MOB-004` GoalSelector and Brain are composable but distinct AI schedulers

- **FidelityClass:** `ExactObservableBehavior`
- **Evidence status:** `Confirmed`

### Primary evidence

`OFF-SERVER-001`; `net.minecraft.world.entity.Mob#serverAiStep()`;
`net.minecraft.world.entity.ai.goal.GoalSelector#addGoal(int,net.minecraft.world.entity.ai.goal.Goal)`;
`net.minecraft.world.entity.ai.goal.GoalSelector#tick()`;
`net.minecraft.world.entity.ai.Brain#tick(net.minecraft.server.level.ServerLevel,net.minecraft.world.entity.LivingEntity)`;
`net.minecraft.world.entity.ai.Brain#setActiveActivityIfPossible(net.minecraft.world.entity.schedule.Activity)`;
`COM-WIKI-MOB-001`

### Applies when

A mob's server AI step runs and its type registered goals and/or Brain behaviors.

### Behavior and timing

`GoalSelector` uses priority and mutually exclusive control flags to stop goals that cannot
continue, replace eligible incumbents, start candidates whose conditions pass, and tick running
goals. `Brain` advances memory expiry and sensors, then starts/stops/ticks behaviors from
core/non-core activity and memory preconditions. A mob may combine navigation, target selector, and
Brain, but their state is not interchangeable.

### Boundaries and quirks

Smaller priority number, equal-priority registration order, non-interruptible goals, disabled flags,
and reduced AI cadence are observable. Ferrite's ECS may differ internally but must preserve
arbitration results.

### Verification

**Owners:** `MOB-AI-001`, `MOB-UNIVERSAL-ANGER-001`, `ITM-ENDER-CHEST-001`,
`ITM-BARREL-001`, `BLK-BELL-001`, `BLK-LAVA-CAULDRON-001`; `EXP-MOB-002`, `EXP-MOB-010`,
`EXP-ITM-008`, `EXP-ITM-009`, `EXP-BLK-009`, `EXP-BLK-039`

The content leaves fix guarded-container piglin anger and bell `HEARD_BELL_TIME` ingress with exact
memory inputs. Lock the remaining equal-priority traversal, every-tick/reduced goal cadence, Brain
behavior ordering, and recovery after inactive-chunk gating.
`MOB-UNIVERSAL-ANGER-001` fixes the live revenge/reset arbitration and both classic-neutral and
Piglin target/memory models without generalizing their different persistence or toggle behavior.
`BLK-HONEY-001` fixes generic and breeze long-jump startup rejection on exact honey, including the
generic half-sampled cooldown write; scheduler admission and later jump phases remain here.

## `MOB-005` Perception caches and paths are consumed incrementally by AI ticks

- **FidelityClass:** `EquivalentPlayerVisibleBehavior`
- **Evidence status:** `Cross-checked`

### Primary evidence

`OFF-SERVER-001`; `net.minecraft.world.entity.ai.sensing.Sensing#tick()`;
`net.minecraft.world.entity.ai.sensing.Sensing#hasLineOfSight(net.minecraft.world.entity.Entity)`;
`net.minecraft.world.entity.ai.navigation.PathNavigation#recomputePath()`;
`net.minecraft.world.entity.ai.navigation.PathNavigation#moveTo(net.minecraft.world.level.pathfinder.Path,double)`;
`net.minecraft.world.entity.ai.navigation.PathNavigation#tick()`; `COM-WIKI-MOB-001`

### Applies when

AI tests target visibility or moves along a path.

### Behavior and timing

`Sensing` clears seen/unseen caches each mob AI tick and fills them lazily with line-of-sight clips.
`PathNavigation` creates a discrete path from node evaluator/pathfinder, stores speed and current
node, then advances, detects stalls, recomputes or stops over later ticks before handing a movement
target to move control/entity physics.

### Boundaries and quirks

Doors, fluids, danger malus, size, chunk boundaries, and dynamic blocks alter node feasibility.
Vanilla compute budgets and tie-breaks may create quirks, but Ferrite targets equivalent
player-visible route, reachability, and response timing rather than an identical internal open set.

### Verification

**Owners:** `MOB-AI-001`, `BLK-HONEY-001`, `BLK-SOUL-SAND-001`, `BLK-MAGMA-001`,
`BLK-LAVA-CAULDRON-001`; `EXP-MOB-002`, `EXP-BLK-036`, `EXP-BLK-037`, `EXP-BLK-038`,
`EXP-BLK-039`

The honey leaf fixes exact `STICKY_HONEY` classification, default malus 8.0 and the no-extra-step
branch. This parent retains node expansion, mob overrides, path selection and incremental use.
The magma leaf fixes exact burning-block recognition and careful Ghast rejection through
`happy_ghast_avoids`; this parent retains path type/malus assignment, traversal and route choice.
The lava-cauldron leaf fixes three distinct inputs: every path type is rejected, exact lava
cauldron is burning, and the reloadable `cauldrons` tag lifts current/eligible following path nodes
by one. Its hardcoded state also belongs to the leatherworker POI; navigation, job claiming and
profession transitions remain with this parent.
The soul-sand leaf makes every queried path-computation type return false at the block hook; node
expansion, entity overrides and route selection remain with this parent.

Define allowed route divergence and add reachability cases for doors/water/narrow spaces, dynamic
blockage, moving targets, and unavailable chunks.

## `MOB-006` Breeding and taming commit type-validated persistent state transitions

- **FidelityClass:** `ExactObservableBehavior`
- **Evidence status:** `Cross-checked`

### Primary evidence

`OFF-SERVER-001`; `OFF-DATA-001`;
`net.minecraft.world.entity.animal.Animal#canMate(net.minecraft.world.entity.animal.Animal)`;
`net.minecraft.world.entity.animal.Animal#spawnChildFromBreeding(net.minecraft.server.level.ServerLevel,net.minecraft.world.entity.animal.Animal)`;
`net.minecraft.world.entity.animal.Animal#finalizeSpawnChildFromBreeding(net.minecraft.server.level.ServerLevel,net.minecraft.world.entity.animal.Animal,net.minecraft.world.entity.AgeableMob)`;
`net.minecraft.world.entity.TamableAnimal#tame(net.minecraft.world.entity.player.Player)`;
`net.minecraft.world.entity.TamableAnimal#setOwner(net.minecraft.world.entity.LivingEntity)`;
`COM-WIKI-MOB-001`

### Applies when

Two compatible animals in love can approach, or a player uses a type-approved item for a taming
attempt.

### Behavior and timing

Breeding validates both ages, love causes, type compatibility, and `canMate`, then creates
offspring, sets parent age/cooldown, clears love, attributes the player criterion, and emits
XP/events. A concrete mob interaction chooses taming chance and consumption; success calls `tame` to
persist owner reference, tame flags, and related AI/events.

### Boundaries and quirks

Variant inheritance, crossbreeding, player disconnect, mob griefing, sit commands, and failed taming
are concrete extensions. Feeding to heal and feeding to enter love are not one generic action.

### Verification

**Owners:** `MOB-BREED-001`; `EXP-MOB-004`

Generate a condition table from source/data for every breedable/tamable type. Generic commit
ordering is cross-checked, but content coverage is incomplete.
