# Simulation mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `SIM-RANDOM-001` — Random ticks are sampled attempts, never accumulated obligations

**Parent:** `SIM-004`, `SIM-005`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — the locked server jar specifies the activity graph, ticket filtering and
thresholds, eligible-chunk iteration, section traversal, position generator, block/fluid snapshot
order, and every framework-owned RNG-consumption site below. Concrete block and fluid callbacks
remain owned by their content-family rules; `EXP-SIM-003` is a regression vector rather than an
unresolved source question.

**Applies when:**

A normal gameplay tick reaches chunk work in a non-debug server level. This rule covers admission of
chunks and sections, not the subtype-specific state machine entered by an eligible block or fluid
callback.

**Authoritative state:**

The server level owns `random_tick_speed`, its gameplay `RandomSource`, and a separate signed-32-bit
`randValue` position stream. Each chunk section maintains signed-short counts of randomly ticking
block and fluid states. `TicketStorage` owns tickets; `SimulationChunkTracker` propagates the lowest
numeric level from only ticket types whose simulation flag is set and stores every propagated level
below 33 in a `Long2ByteOpenHashMap` whose absent value is 33. Chunk holders and ticking-chunk
futures determine whether an admitted coordinate has a live `LevelChunk`. Clients neither sample
positions nor invoke these callbacks.

**Transition and ordering:**

The random-tick phase is part of the following source-specified sequence.

1. `ServerChunkCache#tick` purges eligible stale tickets only when the level runs normally (the
   ordinary level call supplies its second argument as true), then runs all distance-manager updates
   before selecting chunks. A frozen admitted tick skips purge and later random work but still
   propagates already queued distance updates.
2. `ServerChunkCache#tickChunks` computes and stores inhabited-time delta, returns immediately for a
   debug level, and calls its gameplay overload only while `runsNormally()`. That overload
   constructs natural-spawn state, reads `spawn_mobs` and `random_tick_speed`, collects and shuffles
   spawning chunks with the level `RandomSource`, and completes spawn/thunder work before
   random-tick chunks. Those earlier phases may therefore advance the same gameplay RNG before the
   first random callback even if no random-tick callback fires.
3. Despite its name, `ChunkMap#forEachBlockTickingChunk` iterates
   `DistanceManager#forEachEntityTickingChunk`; only propagated simulation level `<= 31` is
   admitted. For each key, it skips a missing visible holder or a holder whose `getTickingChunk()`
   is null, then calls `ServerLevel#tickChunk(chunk, speed)`. Level 32 is block-ticking for
   scheduled/block-entity gates but is not admitted to this random-tick loop.
4. For each admitted chunk, perform exactly `speed` precipitation attempts first. Every attempt
   consumes `level.random.nextInt(48)`. Only result zero advances the separate position stream once
   with base `(chunkMinX, 0, chunkMinZ)` and mask 15, then invokes precipitation handling. A miss
   consumes no position-stream value.
5. If `speed <= 0`, section random ticking ends after the precipitation loop (which also has zero
   iterations). Otherwise traverse the chunk's section array by increasing index, from the
   dimension's bottom section upward. Read `section.isRandomlyTicking()` once before that section's
   attempt loop; skip the section without consuming either stream when both maintained eligibility
   counts are zero.
6. For each admitted section, perform exactly `speed` attempts. Each attempt advances the position
   stream once, obtains the block state directly from that section at the selected relative
   coordinates, and keeps that `BlockState` object as the attempt snapshot. If its block predicate
   is true, invoke block `randomTick(level, pos, level.random)`.
7. After the block callback returns, derive the fluid state from the same captured block-state
   object without rereading the section or world. If that fluid predicate is true, invoke its random
   tick with the same position and gameplay RNG. Block therefore precedes fluid, and a block
   callback that replaces the world position does not change which captured fluid callback is
   dispatched in that attempt. Callback-owned RNG draws do affect the later fluid callback and every
   later gameplay-RNG consumer.
8. Continue the already-admitted section attempt count even if an early callback removes its last
   eligible state. Conversely, a callback that makes a later section eligible is visible when that
   later section's one-time eligibility check occurs. A mutation of the current section changes the
   state read by later samples, but does not resample the current attempt.

### Anchors

`net.minecraft.server.level.ServerChunkCache#tick(java.util.function.BooleanSupplier,boolean)`,
`net.minecraft.server.level.ServerChunkCache#tickChunks()`,
`net.minecraft.server.level.ServerChunkCache#tickChunks(net.minecraft.util.profiling.ProfilerFiller,long)`,
`net.minecraft.server.level.ChunkMap#forEachBlockTickingChunk(java.util.function.Consumer)`,
`net.minecraft.server.level.DistanceManager#forEachEntityTickingChunk(it.unimi.dsi.fastutil.longs.LongConsumer)`,
`net.minecraft.server.level.SimulationChunkTracker#getLevelFromSource(long)`,
`net.minecraft.server.level.SimulationChunkTracker#setLevel(long,int)`,
`net.minecraft.world.level.TicketStorage#getTicketLevelAt(long,boolean)`,
`net.minecraft.server.level.ChunkLevel#isEntityTicking(int)`,
`net.minecraft.server.level.ChunkLevel#isBlockTicking(int)`,
`net.minecraft.server.level.ServerLevel#tickChunk(net.minecraft.world.level.chunk.LevelChunk,int)`,
`net.minecraft.world.level.Level#getBlockRandomPos(int,int,int,int)`,
`net.minecraft.world.level.chunk.LevelChunkSection#isRandomlyTicking()`, and
`net.minecraft.server.level.TicketType#doesSimulate()`.

**Branches and aborts:**

A missing simulation ticket source propagates level 33 and is absent from the iteration map.
`getTicketLevelAt(chunk, true)` ignores every loading-only ticket and selects the lowest numeric
simulation-ticket level. `ChunkLevel.isEntityTicking(level)` is `level <= 31`;
`isBlockTicking(level)` is `level <= 32`. The player simulation source level is
`max(0, 31 - simulationDistance)`, then graph propagation increases the level by distance. Forced
chunks use level 31. A debug level, freeze, missing holder/ticking chunk, level 32 or 33, zero
speed, ineligible section, ineligible captured block, or ineligible captured fluid aborts only its
stated layer. No branch creates catch-up samples.

**Constants and randomness:**

Registry `minecraft:random_tick_speed` is an integer gamerule in category `UPDATES`, default 3,
minimum 0, maximum signed-32-bit 2,147,483,647. It is read once for this chunk-work invocation and
used for both precipitation and every admitted section.
`Level#getBlockRandomPos(baseX, baseY, baseZ, mask)` performs wrapping signed-32-bit
`randValue = randValue * 3 + 1_013_904_223`, then arithmetic-shifts `q = randValue >> 2` and returns
`(baseX + (q & 15), baseY + ((q >> 16) & mask), baseZ + ((q >> 8) & 15))`; this rule always passes
mask 15. `randValue` is initialized during level construction from
`RandomSource.createThreadLocalInstance().nextInt()`, is not derived from the world seed, and is not
the gameplay RNG. It is one shared per-level stream across successful precipitation selection and
all admitted section samples. Skipped sections and precipitation misses do not advance it. The level
gameplay RNG is shared with the preceding spawning-chunk shuffle, thunder/precipitation decisions,
block callbacks, fluid callbacks, and later chunk work, so callback-local draws shift subsequent
vanilla results.

**Chunk iteration order:**

The simulation tracker uses fastutil 8.5.18 `Long2ByteOpenHashMap`, default expected size 16 and
load factor 0.75, yielding an initial 32-slot table and resize threshold 24. Packed chunk key zero
is emitted first from its special slot; other entries are scanned from the highest table index down.
Placement uses linear probing from the low bits of `mix(key)`, where wrapping-64
`h = key * -7_046_029_254_386_353_131`, followed by `h ^= h >>> 32` and `h ^= h >>> 16`. Insertions,
removals, cluster shifts, and high-to-low reinsertion during resize therefore make order depend on
the complete activity-map history. Distance updates are drained before iteration, and ticket
listeners queue graph work rather than mutating this map during the traversal. This order assigns
the shared position/RNG streams to chunks and is consequently observable; Ferrite may use another
container only if it reproduces the resulting callback sequence.

**Ticket registry mapping:**

Flags are `persist=1`, `load=2`, `simulate=4`, `keep-dimension-active=8`, and
`expire-if-unloaded=16`. The nine locked types are: `player_spawn` `(timeout 20, flags 2)`,
`spawn_search` `(1,2)`, `dragon` `(0,6)`, `player_loading` `(0,2)`, `player_simulation` `(0,12)`,
`forced` `(0,15)`, `portal` `(300,15)`, `ender_pearl` `(40,14)`, and `unknown` `(1,18)`. Thus
`dragon`, `player_simulation`, `forced`, `portal`, and `ender_pearl` feed simulation; the other four
do not. Timeout zero means no timeout. A timed ticket starts at its timeout, decrements only on an
eligible purge, and is removed only after the value becomes negative, so a newly created timeout-`N`
ticket survives the purge that changes `1` to `0` and expires on the next eligible purge.
`canExpireIfUnloaded` bypasses the loaded/saving guard; otherwise a timed ticket decrements only
when no updating holder exists or the holder is ready for saving. Freeze skips the ordinary purge
and therefore does not consume timeout.

**Side effects:**

Framework sampling itself mutates only the two RNG streams and profiler state. Precipitation and
block/fluid callbacks may write blocks, schedule ticks, enqueue neighbor work, change eligibility
counts, spawn entities/items, mutate inventories, emit game events, sounds and particles, or consume
additional RNG; those effects occur synchronously at the exact callback position above and are
specified by the dispatched content rule. Section counts update on state replacement and can also be
recomputed from all 4,096 palette positions; the count is an admission optimization, not a per-state
callback queue.

**Gates:**

Normal gameplay snapshot; non-debug level; a simulation-ticket graph result at entity-ticking level;
visible holder and non-null ticking chunk; integer gamerule; section eligibility count; captured
block/fluid eligibility predicates. Loading status, block-ticking level 32, view distance,
natural-spawn radius, and the presence of a player are not interchangeable with this exact
conjunction. Difficulty does not gate generic random sampling. Permissions gate changing the
gamerule/tickets, not execution after state is authoritative.

**Boundary cases and quirks:**

Inactive, unloaded, frozen, debug, or speed-zero time accrues no obligation and consumes no
random-tick position samples. Re-entering level 31 begins only that tick's attempts. A very large
legal speed causes a correspondingly large synchronous loop for precipitation and each currently
eligible section; vanilla applies no operational clamp below signed-32 maximum. Repeated samples of
one position are valid. Because the position stream begins from a thread-local random value and is
not saved, identical world seeds or server restarts need not reproduce positions. The stricter
level-31 random gate contradicts what the `forEachBlockTickingChunk` name suggests and must not be
generalized from the level-32 block-ticking predicate.

**Evidence:**

`Confirmed`; `OFF-SERVER-001`. The locked reports supply all 1,196 block state eligibility
properties and registry membership for the five fluid IDs, nine ticket types, and
`minecraft:random_tick_speed`. Control flow and constants are directly specified by the anchors
above plus
`net.minecraft.world.level.chunk.LevelChunkSection#setBlockState(int,int,int,net.minecraft.world.level.block.state.BlockState,boolean)`,
`net.minecraft.world.level.chunk.LevelChunkSection#recalcBlockCounts()`,
`net.minecraft.server.level.DistanceManager#getPlayerTicketLevel()`,
`net.minecraft.world.level.TicketStorage#getLowestTicket(java.util.List,boolean)`,
`net.minecraft.world.level.TicketStorage#purgeStaleTickets(net.minecraft.server.level.ChunkMap)`,
`net.minecraft.server.level.Ticket#decreaseTicksLeft()`,
`net.minecraft.server.level.Ticket#isTimedOut()`, and
`net.minecraft.world.level.gamerules.GameRules#registerInteger(java.lang.String,net.minecraft.world.level.gamerules.GameRuleCategory,int,int)`.

**Test vectors:**

(1) Put an instrumented eligible state in otherwise ineligible sections at the bottom and top, set
speed 3, and assert bottom-to-top section admission with exactly three position advances per
section. (2) Compare propagated levels 30, 31, 32, and 33: random work runs only through 31 even
though the level-32 predicate reports block ticking. (3) Supply each of the nine ticket types at
level 31 and assert only the five simulation types admit the chunk; combine multiple tickets and
assert lowest numeric eligible selection. (4) Set speed 0, 1, 3, and signed-32 maximum in a bounded
harness; assert exact loop bounds and no clamp, aborting the maximum case after observing admission
rather than waiting for completion. (5) Force precipitation RNG results miss, hit, miss; assert
three gameplay-RNG draws but one position-stream advance before section sampling. (6) Use a block
callback that replaces itself and consumes RNG while its captured fluid is eligible; assert
old-snapshot block-before-fluid dispatch and shifted later RNG. (7) Remove the current section's
last eligible state in the first of three admitted attempts and assert two more samples; add
eligibility to a later section and assert that section is admitted. (8) Hold a chunk at level 32 or
unloaded for 1,000 ticks, then move it to 31; assert no burst or RNG consumption for missed time.
(9) Create the same active chunk keys in two insertion/removal histories, including packed key zero
and a resize past 24 entries, and compare vanilla callback order to the specified fastutil
traversal. (10) Run `EXP-SIM-003` for speeds 0/1/3, ticket edges, sampled positions, and captured
block/fluid callback order.
