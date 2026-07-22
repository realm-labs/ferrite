# Mobs mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `MOB-HOSTILE-GATE-001` — `spawn_monsters` refreshes chunk policy and gates four direct spawn transactions

**Parent:** `MOB-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — the default, compound accessor, startup/live cache propagation, every cache
consumer and all four direct accessor callers are explicit in locked source.

**Applies when:**

A server level refreshes hostile-spawn policy, filters natural/custom spawning, or reaches an
ender-pearl endermite, zombie reinforcement, creaking-heart protector or Nether-portal piglin
spawn branch.

**Authoritative state:**

The `spawn_mobs` and `spawn_monsters` gamerules, difficulty, each chunk source's cached
`spawnEnemies` flag, custom-spawner state, branch-local entity/block state, environment attributes
and the RNG stream owned by the caller.

**Transition and ordering:**

**Rule and propagation:** `spawn_monsters` is a `SPAWNING` Boolean rule whose default is true.
`ServerLevel#isSpawningMonsters` is the conjunction of `spawn_mobs` and `spawn_monsters`, so the
broader rule can disable every use of this hostile gate. After initial chunks load,
`MinecraftServer#prepareLevels` calls `updateMobSpawningFlags`; difficulty changes do the same.
Changing `spawn_monsters` calls the server gamerule notification path and then refreshes every
server level. Each refresh passes the level's current conjunction through `Level#setSpawnSettings`
and `ServerChunkCache#setSpawnSettings`, replacing that chunk source's `spawnEnemies` cache.

**Chunk and custom spawning:** The normal chunk tick passes cached `spawnEnemies` to
`NaturalSpawner#getFilteredSpawningCategories`. False excludes only `MONSTER`, because it is the
only nonfriendly natural category; it does not exclude friendly categories. The same cache is
passed to all custom spawners. Patrol and phantom spawners return before their own gamerule,
countdown and RNG when it is false. A village siege that sees daylight or false hostile policy sets
its state to done, clears setup and returns. Cat and wandering-trader spawners ignore this Boolean,
so the cat countdown and the wandering-trader countdown/independent gamerule remain reachable. The
separate `spawn_mobs` caller gate and the rest of the natural-spawn transaction remain owned by
`MOB-SPAWN-001`; admitted patrol and phantom transactions are owned by `MOB-PATROL-001` and
`MOB-PHANTOM-SPAWN-001`, while the independently ruled wandering-trader transaction is owned by
`MOB-WANDERING-TRADER-001`.

**Ender-pearl endermite:** After an accepted server-side impact identifies a connected
`ServerPlayer` owner, the pearl consumes `nextFloat` and tests `<0.05` before testing the live
hostile conjunction and non-Peaceful difficulty. Success creates an endermite with reason
`TRIGGERED`, copies the owner's current position and rotation and calls `addFreshEntity`; null
creation or insertion failure has no replacement attempt. This branch does not gate the ordinary
pearl teleport, damage and discard transaction.

**Zombie reinforcement:** After superclass damage succeeds, use the current target or a living
damage-source entity. Only Hard difficulty proceeds. The hurt zombie consumes its chance draw
against `SPAWN_REINFORCEMENTS_CHANCE` before reading the live hostile conjunction. Failure of that
gate leaves the accepted damage intact. Success constructs one same-type zombie with reason
`REINFORCEMENT`; null construction ends the branch. At most 50 candidates are tried. Each consumes
six inclusive integer draws: independently for X/Y/Z, a distance in `7..40` multiplied by a sign
in `-1..1` is added to the hurt zombie's floored coordinate. A candidate must pass placement
position, registered reinforcement spawn rule, no alive player within 7 blocks, unobstructed AABB,
collision and, unless the zombie subtype can spawn in liquids, no-liquid checks in that order.
The first success sets the target, finalizes at the candidate difficulty, inserts with passengers,
then replaces the caller's permanent add-value charge with its existing amount (or zero) minus
`0.05`, installs the callee's fixed permanent add-value charge, and stops. The insertion call is
not a rollback boundary for those modifiers; 50 failures have no further side effect.

**Creaking-heart protector:** The server tick updates its counters, comparator/emitter work and
`20+nextInt(5)` ticker before this gate. Uprooted hearts and hearts already tracking a protector
return earlier; disabling the rule therefore does not remove an existing protector. For a missing
protector and an awake heart, false hostile policy or Peaceful difficulty returns before the
radius-32 nearest-player query. With a player present, the heart makes one
`SpawnUtil.trySpawnMob(CREAKING, SPAWNER, heartPos, 5, 16, 8,
ON_TOP_OF_COLLIDER_NO_LEAVES, true)` call. Success emits `ENTITY_PLACE`, broadcasts entity event
`60`, binds the transient protector to the heart, stores it and plays the creaking and heart spawn
sounds; empty spawn result commits none of those effects.

**Nether-portal piglin:** A portal random tick checks the live hostile conjunction, non-Peaceful
difficulty and positional `NETHER_PORTAL_SPAWNS_PIGLINS` environment attribute before consuming
`nextInt(2000)`. The draw must be below the difficulty ID, then a player must be close enough for
spawning. The cursor descends through consecutive portal blocks to the first nonportal state;
that ground must be a valid zombified-piglin spawn. One zombified piglin is created immediately
above it with reason `STRUCTURE`; success assigns portal cooldown to the piglin and its vehicle, if
present. There is no retry.

**Branches and aborts:**

False `spawn_mobs`; false `spawn_monsters`; Peaceful difficulty where tested; cached category or
custom-spawner rejection; chance failure; absent target/player; creation, placement, collision or
insertion failure; inactive heart state; environment-attribute or portal-ground failure. Each
branch consumes only RNG already reached above.

**Constants and randomness:**

Endermite chance `0.05`; zombie Hard-only admission, 50 attempts, `7..40` signed offsets and
7-block player exclusion; heart cadence `20..24`, player range 32 and spawn parameters
`5/16/8`; portal denominator 2000 and threshold equal to difficulty ID. Rule-disabled portal,
heart, patrol and phantom paths consume no branch RNG, while pearl and zombie chance draws occur
before their live rule tests.

**Side effects:**

Per-level cache replacement, custom-spawner state reset, spawned/finalized entities, reinforcement
attribute modifiers, heart ownership/events/sounds and portal cooldown. Changing the rule neither
despawns existing hostile mobs nor directly tears down an existing creaking protector.

**Gates:**

The compound live rule, cached chunk-source projection, difficulty and every branch-specific
player, environment, placement, collision and construction condition above.

**Boundary cases and quirks:**

Natural spawning reads a refreshed cache, while the four special branches read the live level
conjunction. A false cache suppresses only natural `MONSTER` and the patrol/phantom/siege custom
paths, not cats or wandering traders. Pearl/zombie draws precede their live rule tests; portal and
heart rule tests precede their spawn-chance/search work.

**Evidence:**

`OFF-SERVER-001`; `OFF-REPORT-001`;
`net.minecraft.world.level.gamerules.GameRules`;
`net.minecraft.server.MinecraftServer#prepareLevels`, `#setDifficulty`,
`#onGameRuleChanged`, `#updateMobSpawningFlags`;
`net.minecraft.server.level.ServerLevel#isSpawningMonsters`, `#tickCustomSpawners`;
`net.minecraft.world.level.Level#setSpawnSettings`;
`net.minecraft.server.level.ServerChunkCache#setSpawnSettings`, `#tickChunks`;
`net.minecraft.world.level.NaturalSpawner#getFilteredSpawningCategories`;
`net.minecraft.world.level.levelgen.PatrolSpawner#tick`;
`net.minecraft.world.level.levelgen.PhantomSpawner#tick`;
`net.minecraft.world.entity.ai.village.VillageSiege#tick`;
`net.minecraft.world.entity.npc.CatSpawner#tick`;
`net.minecraft.world.entity.npc.wanderingtrader.WanderingTraderSpawner#tick`;
`net.minecraft.world.entity.projectile.throwableitemprojectile.ThrownEnderpearl#onHit`;
`net.minecraft.world.entity.monster.zombie.Zombie#hurtServer`;
`net.minecraft.world.level.block.entity.CreakingHeartBlockEntity#serverTick`,
`#spawnProtector`;
`net.minecraft.world.level.block.NetherPortalBlock#randomTick`; `EXP-MOB-005`.

**Test vectors:**

Cross both gamerules independently at startup and after live changes; force every natural category
and all five custom spawners; toggle the rule around an existing siege/protector; force pearl chance
at equality and on either side; replay zombie attempts 0/1/50 with every ordered rejection and
insertion failure; exercise awake/uprooted/tracked hearts and spawn failure; cross each portal gate,
difficulty threshold, portal-column depth, ground validity, creation and vehicle branch. Assert the
branch-local RNG cursor and all cache/entity/state/effect deltas.
