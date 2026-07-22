# Mobs mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `MOB-SPAWN-001` — Natural spawning is a counted global/local-cap transaction followed by three pack walks

**Parent:** `MOB-001`, `MOB-002`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — category filtering, global and per-player caps, biome potential, candidate RNG,
pack termination, structure selection, all registered placement families, finalization and accounting
are explicit in locked source and data.

**Applies when:**

The chunk source has selected a spawnable level chunk and invokes natural spawning with its current
enemy/persistent-category policy. Chunk-generation creature spawning uses the separate transaction
below.

**Authoritative state:**

The round's `spawnableChunkCount`, entity snapshot, cached players close enough to each spawning
chunk, global category counts, biome spawn-cost charges, chunk/biome/structure spawn lists, the
level respawn point, placement registry, local block/fluid/light/collision state and level RNG.

**Transition and ordering:**

**Caller admission and order:**

On a normal non-debug chunk tick, the distance manager first drains natural-spawn tracker updates;
`spawnableChunkCount` is the size of that tracker's union of player-distance chunks. The spawn state
is built even when `spawn_mobs` is false. When that gamerule is true, `CREATURE` (the only persistent
category) is admitted only on game times divisible by `400`; hostile admission also follows the
chunk source's `spawnEnemies` flag, whose startup/live projection and custom-spawner effects are
owned by `MOB-HOSTILE-GATE-001`. The chunk map collects spawn-candidate positions that have a
present ticking chunk and at least one nonspectator player at chunk-center squared Euclidean
distance strictly below `16384`, then shuffles that list with level RNG. Each chunk increments
inhabited time, performs entity-range thunder work, and calls this natural pipeline only when its
category list is nonempty and `canSpawnEntitiesInChunk` is true. Block/random chunk ticks follow the
entire shuffled spawn loop; custom spawners follow those ticks.

**Count and category transaction:**

`createState` visits the supplied entity snapshot in iteration order. A `Mob` with
`persistenceRequired` or `requiresCustomPersistence()` is skipped entirely. Every other non-`MISC`
entity is counted globally if its containing chunk can be queried; its biome spawn-cost charge is
added when defined, and a `Mob` is also counted against every player returned by
`ChunkMap#getPlayersCloseForSpawning` for that chunk. Non-mob entities can therefore consume a
global category count and potential charge but never a local player count.

The category iteration order is `MONSTER`, `CREATURE`, `AMBIENT`, `AXOLOTLS`,
`UNDERGROUND_WATER_CREATURE`, `WATER_CREATURE`, `WATER_AMBIENT`; `MISC` is excluded. Enemy filtering
keeps a category when `spawnEnemies` is true or it is friendly. Persistent filtering keeps it when
`spawnPersistent` is true or the category is not persistent. Its global cap is integer arithmetic

`baseMax * spawnableChunkCount / 289`,

and the category qualifies only while its current count is strictly below that result. Base maxima
are respectively `70, 10, 15, 5, 5, 5, 20`; only `CREATURE` is persistent and only `MONSTER` is not
friendly. Per chunk/category, the local cap admits the attempt if at least one nearby spawning
player has no counter yet or has a count strictly below the same category base maximum. No nearby
players means local rejection. Each accepted mob increments the category counter for every nearby
player of its actual containing chunk, so overlapping players share the spawn and a separated
player can keep a chunk eligible after another player reaches their cap.

If a biome declares a spawn cost, a candidate of charge `c` is admitted only when the sum of the
existing point potentials at its position multiplied by `c` is at most that type's energy budget.
The last candidate position/type/charge is cached for the subsequent success callback.

**Candidate and pack transaction:**

For each locally eligible category, choose `x` and `z` independently from the 16 block coordinates
of the selected chunk. Let `top = WORLD_SURFACE(x,z) + 1`; choose `y` uniformly and inclusively from
the level minimum through `top`. Abort below `minY + 1` or when the starting block is a redstone
conductor.

From that fixed `y`, run three independent group walks. Each resets `x/z` to the start, spawn entry,
group data and group count, and first draws `ceil(nextFloat() * 4)` as its provisional attempt count.
Every attempt changes each horizontal coordinate by `nextInt(6) - nextInt(6)`, then finds the nearest
non-spectator player without a range limit. Absence of such a player rejects only that candidate.
The squared player distance must be strictly greater than `576`; a same-dimension respawn point
must not be within `24` blocks of the candidate center; and a candidate outside the original chunk
requires `canSpawnEntitiesInChunk`.

The first candidate that reaches list selection fixes one `SpawnerData` for the rest of that group.
For `WATER_AMBIENT` in a `REDUCED_WATER_AMBIENT_SPAWNS` biome, `nextFloat() < 0.98` yields no entry.
Otherwise a `MONSTER` candidate immediately above nether bricks inside a valid nether fortress uses
the fortress enemy list; all other candidates use the generator's biome/structure list. Empty
weighted selection ends that group. Success replaces the provisional attempt count with an
inclusive random integer from the entry's `minCount` through `maxCount`.

Each later candidate must still find that exact entry in its current biome/structure list. It then
rejects `MISC`, a non-summonable type, a type that cannot spawn far from players beyond its category
hard distance, an invalid placement, failed registered predicate, failed biome potential or type
AABB collision. Entity construction failure or a constructed non-`Mob` ends the entire category
position routine. A constructed mob is snapped to block center with random yaw in `[0,360)`, then
must pass its own far-removal predicate, `checkSpawnRules(NATURAL)` and obstruction check. Success
calls `finalizeSpawn` with group-shared data, increments cluster and group counts, calls
`addFreshEntityWithPassengers`, and unconditionally updates potential/global/local accounting after
that call. The insertion result is not a rollback boundary. Reaching `getMaxSpawnClusterSize()` ends
all three walks; `isMaxGroupSizeReached(groupSize)` ends only the current walk.

**Placement and content families:**

The locked `SpawnPlacements` table has `83` explicit entity registrations and rejects every type not
allowed in Peaceful before its predicate. Unregistered types use `NO_RESTRICTIONS`,
`MOTION_BLOCKING_NO_LEAVES` and a true predicate. Registered placement partitions are:

- `IN_WATER`: axolotl, cod, dolphin, drowned, guardian/elder guardian, pufferfish, salmon, squid,
  glow squid, tropical fish and nautilus;
- `IN_LAVA`: strider;
- `NO_RESTRICTIONS`: evoker, fox, illusioner, panda, phantom, shulker, trader llama, vex, vindicator
  and warden;
- `ON_GROUND`: every other explicit registration, with ocelot and parrot alone using
  `MOTION_BLOCKING` rather than `MOTION_BLOCKING_NO_LEAVES`.

The predicates dispatch exactly to the locked common families (`Animal`, `Monster`, surface
monster, any-light monster, water/surface-water) or the registered species method. Placement types
own medium/support adjustment; the generic empty-block helper rejects a full collision shape,
signal source, nonempty fluid, `PREVENT_MOB_SPAWNING_INSIDE` tag or a block dangerous to the type.
Light, height, biome/tag and species RNG therefore belong to the registered predicate, not a second
generic check.

**Chunk-generation branch:**

Chunk generation considers only the biome `CREATURE` list and requires `spawn_mobs`. While
`nextFloat() < creatureProbability`, it chooses a weighted entry and inclusive group count, then for
each member makes at most four attempts. Position starts randomly in the chunk, uses the placement
heightmap and ceiling adjustment, clamps the entity center inside the chunk by its width, and checks
summonability, placement, collision, the registered `CHUNK_GENERATION` predicate, construction,
mob rules and obstruction. Success shares `SpawnGroupData`, finalizes and adds the mob. Whether an
attempt succeeds or fails, the next horizontal position performs independent
`nextInt(5)-nextInt(5)` walks, resampling around the original start until it is back inside the
chunk. Construction exceptions/nulls continue rather than aborting the outer routine.

**Branches and aborts:**

Zero/filled global cap; no locally under-cap player; excluded category; invalid start; no nearest
player; either 24-block exclusion; inactive destination chunk; empty/changed spawn list; potential,
placement, difficulty, predicate, collision, far-removal, mob-rule or obstruction failure; creation
failure; group/cluster completion. Every failure consumes only RNG already reached on that branch.

**Constants and randomness:**

The spawn radius constants are `8` chunks/`128` blocks, its inscribed-square radius is
`floor(8/sqrt(2))`, global-cap divisor is `17^2 = 289`, player/respawn exclusion is `24`, and the
three walk/count/offset formulas are as stated. Category no-despawn distance is always `32`; hard
distance is `128` except `WATER_AMBIENT = 64`.

**Side effects:**

Construction, spawn-rule RNG, finalization/equipment/passengers, entity insertion, group data,
biome potential, global/local counters and all subtype spawn events. Failed candidates do not
reserve count; a post-finalization insertion failure is nevertheless accounted by this caller.

**Gates:**

Normal chunk ticking, non-debug level, `spawn_mobs`, enemy/persistent cadence, ticking candidate
chunk, nonspectator player proximity, `canSpawnEntitiesInChunk`, both cap layers, category/type
lists, potential, placement, peaceful/species rules, collision and obstruction.

**Boundary cases and quirks:**

The global-cap tracker union is not the shuffled loaded-candidate count. Candidate `y` never changes
within a pack walk. A persistent mob is absent from the initial caps, while a non-mob category
entity can consume only global cap. Empty weighted selection ends a group, construction failure
ends the whole position routine, and entity-add failure after finalization still consumes cap and
potential in this caller.

**Evidence:**

`OFF-SERVER-001`; `OFF-DATA-001`;
`net.minecraft.world.level.NaturalSpawner#createState`, `#getFilteredSpawningCategories`,
`#spawnCategoryForPosition`, `#spawnMobsForChunkGeneration`;
`net.minecraft.world.level.NaturalSpawner$SpawnState`;
`net.minecraft.world.level.LocalMobCapCalculator`;
`net.minecraft.world.level.PotentialCalculator`;
`net.minecraft.world.entity.MobCategory`;
`net.minecraft.world.entity.SpawnPlacements`;
`net.minecraft.server.level.ServerChunkCache#tickChunks`, `#tickSpawningChunk`;
`net.minecraft.server.level.ChunkMap#collectSpawningChunks`, `#playerIsCloseEnoughForSpawning`;
`net.minecraft.server.level.DistanceManager#getNaturalSpawnChunkCount`;
`MOB-HOSTILE-GATE-001`; `EXP-MOB-001`.

**Test vectors:**

Cap results around `0/288/289` chunks; overlapping/separate/spectator-only players; persistent and
non-mob initial counts; strict `24/32/64/128` boundaries; potential equality; all four placements;
fortress and reduced-water branches; provisional zero attempts; list changes within a walk;
creation/finalization/insertion failures; group versus cluster end; fixed-RNG chunk-generation walk.
Also cross cached hostile policy true/false and verify only `MONSTER` leaves the category list; the
cache's origin and live refresh are tested under `MOB-HOSTILE-GATE-001`.
