# Mobs mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `MOB-PHANTOM-SPAWN-001` — Phantom spawning turns a shared timer into ordered per-player difficulty and insomnia trials

**Parent:** `MOB-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — installation scope, timer mutation, sky/difficulty/rest gates, every RNG draw,
group construction, finalization and failure boundary are explicit in locked source.

**Applies when:**

The nondebug Overworld chunk source runs normally, `spawn_mobs` has admitted custom spawners, and
its single `PhantomSpawner` receives the cached hostile-spawn Boolean.

**Authoritative state:**

The spawner's nonpersisted signed `nextTick`, the `spawn_phantoms` gamerule, cached hostile policy,
dimension skylight/sky-darkening state, ordered level player list, each player's position,
spectator status and `time_since_rest` stat, sea level/sky visibility, regional difficulty, level
RNG, candidate block/fluid state and each constructed phantom's state.

**Transition and ordering:**

**Installation and caller gates:** `MinecraftServer#createLevels` installs one phantom spawner in
the five-entry custom-spawner list passed only to the Overworld; other levels receive an empty
list. Debug or frozen chunk ticking does not call it. Normal custom spawning occurs after
block/random chunk ticks and only when `spawn_mobs` is true. Phantom then returns first for false
cached hostile policy and second for false live `spawn_phantoms`; both pause its timer and consume
no RNG. The rule is a `SPAWNING` Boolean whose default is true.

**Timer and level sky gate:** With both gates true, predecrement `nextTick`; a positive result
returns without RNG. A nonpositive result first adds
`(60+nextInt(60))*20`, committing the next schedule before any later failure. A fresh spawner
starts at zero and therefore stores `1199..2379` on its first admitted call; ordinary later expiry
reaches zero after decrement and stores `1200..2380`. The field is not persisted, so restart
recreates zero. After scheduling, a dimension with skylight returns when `skyDarken < 5`; equality
proceeds. A no-skylight type bypasses this level gate.

**Ordered player trials:** Iterate the level player list in encounter order; one player's failure
continues to the next. Spectators consume no RNG and are skipped. In a skylight dimension, require
player block Y at least sea level and `canSeeSky(playerPos)`; a no-skylight type bypasses both.
Capture regional difficulty, consume `nextFloat()*3`, and require effective difficulty to be
strictly greater than that value. Peaceful effective difficulty zero therefore always fails, but
only after this draw.

Clamp the player's `minecraft:custom/time_since_rest` stat to `1..2147483647`, consume
`nextInt(clampedRest)`, and require the result to be at least `72000`. Rest values at or below
`72000` can never pass; above it, the exact chance is `(rest-72000)/rest` after clamping. Failure
continues with the next player.

**Candidate and group:** For an admitted player, consume three draws and form one group position:
Y is player Y plus `20+nextInt(15)` (`20..34`), X is plus `-10+nextInt(21)`, then Z uses the same
`-10..10` range. Require `NaturalSpawner#isValidEmptySpawnBlock` for a phantom at that single
position. There is no retry position.

Set group data to null, then consume
`1+nextInt(baseDifficultyId+1)` as the member count: Easy `1..2`, Normal `1..3`, Hard `1..4`;
Peaceful cannot reach this point. Every member uses the same position. Construct a phantom with
reason `NATURAL`; null construction skips only that member. A nonnull phantom snaps to the exact
block position with yaw/pitch zero, then finalizes with the player's captured regional difficulty.
Phantom finalization sets its anchor five blocks above its snapped position and size zero, then
Mob finalization applies its follow-range random bonus and left-handed draw. The returned group
data remains the supplied null in this chain. The caller adds the phantom with passengers and
does not observe an insertion result.

No per-member placement predicate, difficulty check, AABB collision, obstruction, player-distance
or position change follows the one empty-block test. Finalization consumes level RNG inside the
member loop, so successful earlier members affect later finalization and all later players' trials;
null creation consumes none of that finalization RNG.

**Branches and aborts:**

Absent Overworld/custom-spawner call; debug/frozen tick; false `spawn_mobs`, cached hostile policy
or `spawn_phantoms`; positive countdown; insufficient global sky darkness; spectator; below-sea or
covered skylight player; strict difficulty failure; insomnia draw below 72000; invalid empty spawn
block; null member construction. Post-expiry failures retain the already committed schedule, and
per-player/member failures do not terminate later eligible entries except where no entries remain.

**Constants and randomness:**

Timer addition `(60+nextInt(60))*20`; sky-darken threshold 5; strict difficulty trial
`effectiveDifficulty > nextFloat*3`; rest clamp `1..Integer.MAX_VALUE` and threshold 72000;
candidate offsets Y `20..34`, X/Z `-10..10`; group counts by difficulty ID above; anchor offset 5
and size zero. All caller and Mob-finalization draws share the level RNG in their reached order.

**Side effects:**

Ephemeral timer mutation; level RNG advancement; phantom construction, position, anchor, size,
follow-range/handedness finalization and entity insertion. The rule does not alter the rest stat
and does not own later phantom AI, combat, despawn or drops.

**Gates:**

Overworld installation, normal nondebug chunk ticking, `spawn_mobs`, cached hostile policy,
`spawn_phantoms`, timer, dimension/sky state and the exact per-player/candidate conditions above.

**Boundary cases and quirks:**

Disabling either spawning rule pauses rather than resets cadence; restart resets it to an immediate
due state. Scheduling precedes every sky/player failure. Difficulty RNG precedes insomnia RNG.
Every player is considered rather than one random player, group members stack at one position, and
insertion is unchecked.

**Evidence:**

`OFF-SERVER-001`; `OFF-REPORT-001`;
`net.minecraft.server.MinecraftServer#createLevels`;
`net.minecraft.server.level.ServerChunkCache#tickChunks`;
`net.minecraft.server.level.ServerLevel#tickCustomSpawners`, `#getSkyDarken`, `#canSeeSky`,
`#getCurrentDifficultyAt`;
`net.minecraft.world.level.levelgen.PhantomSpawner#tick`;
`net.minecraft.world.DifficultyInstance#isHarderThan`;
`net.minecraft.stats.StatsCounter#getValue`;
`net.minecraft.world.level.NaturalSpawner#isValidEmptySpawnBlock`;
`net.minecraft.world.entity.monster.Phantom#finalizeSpawn`;
`net.minecraft.world.entity.Mob#finalizeSpawn`;
`WGEN-DIMENSION-001`; `MOB-HOSTILE-GATE-001`; `EXP-MOB-007`.

**Test vectors:**

Cross first/subsequent/restarted timer endpoints and both pausing rules; skylight/no-skylight types,
sky darkness 4/5, sea-level equality and sky visibility; ordered spectator and eligible players;
difficulty strict equality and all base IDs; rest values 0/1/71999/72000/72001/Integer.MAX_VALUE;
every candidate-offset endpoint, valid/invalid empty blocks, group-count endpoints, null creation
at each index and ignored insertion. Assert timer, exact RNG cursor, per-player continuation,
position/anchor/size/finalization and entity count.
