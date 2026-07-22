# 01 — Ticks, Time, Pausing, and Chunk Activity

See the [source lock](../sources.md) and [evidence method](../methodology.md) for the baseline. This
page describes server simulation time; client ticks and render frames are covered by `CLI-*`.

## `SIM-001` A fixed game tick is the state-advancement unit

- **FidelityClass:** `ExactObservableBehavior`
- **Evidence status:** `Confirmed`

### Primary evidence

`OFF-SERVER-001`; `net.minecraft.server.MinecraftServer#runServer()`;
`net.minecraft.server.MinecraftServer#tickServer(java.util.function.BooleanSupplier)`;
`net.minecraft.world.TickRateManager#nanosecondsPerTick()`

### Applies when

A dedicated or integrated server loop chooses a paced, sprinted, frozen, or stepped tick.

### Behavior and timing

The default target is 20 ticks per second (50,000,000 ns). Each loop adds one configured interval to
its deadline and advances gameplay only in integer ticks. `/tick rate` accepts 1.0–10,000.0 and
computes a truncated nanosecond interval; sprint uses interval zero for an exact requested tick
count. When sufficiently overloaded, vanilla advances the deadline by the whole number of missed
intervals and executes only the current tick, never catch-up or fractional physics steps.

### Boundaries and quirks

Tick count still advances during freeze, while integrated and dedicated empty pauses can return
before tick-count admission. Sprint temporarily unfreezes, but replacing an active sprint loses the
first sprint's saved frozen state. Autosave scheduling is tick-rate-adjusted after its initial 6,000
ticks. Ferrite tests must assert both admitted-tick and normal-gameplay-tick boundaries.

### Verification

**Owners:** `SIM-PIPELINE-001`, `SIM-COMMAND-LIMIT-001`; `EXP-SIM-001`, `EXP-SIM-004`,
`EXP-SIM-005`, `EXP-SIM-006`

The leaf specifies deadline correction, rate rounding, sprint measurement, admission gates, and
executable overload/pause vectors. `SIM-COMMAND-LIMIT-001` owns the separately bounded synchronous
command-action queue admitted within server work.

## `SIM-002` Major per-dimension tick phases are ordered

- **FidelityClass:** `ExactObservableBehavior`
- **Evidence status:** `Confirmed`

### Primary evidence

`OFF-SERVER-001`;
`net.minecraft.server.MinecraftServer#tickChildren(java.util.function.BooleanSupplier)`;
`net.minecraft.server.level.ServerLevel#tick(java.util.function.BooleanSupplier)`

### Applies when

`MinecraftServer` advances a server dimension in the current tick.

### Behavior and timing

Before dimensions, functions always tick, clocks tick only during normal gameplay, and time sync
occurs every 20 admitted ticks. Levels tick serially: overworld first, then non-overworld levels in
locked level-stem registry order. Within each level the order is environment-cache invalidation;
border/weather when normal; sleep/wake and sky brightness; time when normal; scheduled block then
fluid ticks; raids; chunk source; block events; eligible entity/passenger trees; block entities;
entity management. Connections, player list, GameTests when normal, GUI work, and chunk sending
follow all dimensions.

### Boundaries and quirks

Freeze gates individual phases rather than the level call. Synchronous callbacks may affect later
phases. Level `emptyTime >= 300` skips dragon/entity/block-entity work, while entity management
remains. Cross-dimension shared-state behavior must preserve the linked insertion order and cannot
be parallelized without an equivalence barrier.

### Verification

**Owners:** `SIM-PIPELINE-001`; `EXP-SIM-001`

The leaf contains the complete top-level order, freeze/activity gates, and a same-tick conformance
trace.

## `SIM-003` Scheduled ticks use per-chunk trigger order and a bounded due-head merge

- **FidelityClass:** `ExactObservableBehavior`
- **Evidence status:** `Confirmed`

### Primary evidence

`OFF-SERVER-001`;
`net.minecraft.world.ticks.LevelTicks#schedule(net.minecraft.world.ticks.ScheduledTick)`;
`net.minecraft.world.ticks.LevelTicks#tick(long,int,java.util.function.BiConsumer)`;
`net.minecraft.world.ticks.ScheduledTick#DRAIN_ORDER`;
`net.minecraft.world.ticks.ScheduledTick#INTRA_TICK_DRAIN_ORDER`

### Applies when

A block or fluid requests a tick at the current or a future game time.

### Behavior and timing

Each chunk orders its queue by `triggerTick`, priority, then `subTickOrder`. The level first admits
due block-ticking-range chunk heads, then merges those heads by priority/sub-order while respecting
each chunk's local head. Type identity plus position deduplicates requests. The complete batch is
collected before callbacks, blocks drain before fluids, and each queue independently executes at
most `65,536` entries per dimension per admitted tick.

### Boundaries and quirks

Callback-created work never joins the current collected batch, even at zero/negative delay. Inactive
loaded queues retain absolute triggers; serialized queues retain relative signed-32-bit delays.
Restored equal priority/sub-order heads in different chunks have no saved global tie breaker.

### Verification

**Owners:** `SIM-SCHEDULE-001`, `BLK-LECTERN-001`; `EXP-SIM-002`, `EXP-BLK-011`

The generic scheduler is fully specified except the restored cross-chunk comparator tie; reproduce
that tie with both chunk load orders and treat the observation as version-locked evidence. The
lectern leaf fixes its concrete deduplicated delay-two page pulse and captured/live-state
boundaries.

## `SIM-004` Random ticks sample only eligible states in active chunks

- **FidelityClass:** `ExactObservableBehavior`
- **Evidence status:** `Confirmed`

### Primary evidence

`OFF-SERVER-001`;
`net.minecraft.server.level.ServerLevel#tickChunk(net.minecraft.world.level.chunk.LevelChunk,int)`;
`net.minecraft.world.level.block.state.BlockBehaviour$BlockStateBase#isRandomlyTicking()`;
`net.minecraft.world.level.block.state.BlockBehaviour$BlockStateBase#randomTick(net.minecraft.server.level.ServerLevel,net.minecraft.core.BlockPos,net.minecraft.util.RandomSource)`

### Applies when

A chunk has the activity level required for random ticking and `randomTickSpeed` is greater than
zero.

### Behavior and timing

Level-31 entity-ticking chunks are visited in simulation-map order. Precipitation attempts run
first; sections run bottom-to-top and are admitted from maintained eligibility counts. Every
admitted section receives exactly `random_tick_speed` samples from a separate wrapping-32-bit
position stream. A captured eligible block callback runs before the eligible fluid derived from that
same captured state; the world is not reread between them.

### Boundaries and quirks

Level 32 is block-ticking but does not receive this random work. Skipped sections and
inactive/unloaded/frozen time consume no position samples and accumulate no obligation. The position
stream is not world-seed-derived or saved, while callbacks share the level gameplay RNG with earlier
spawn/weather work.

### Verification

**Owners:** `SIM-RANDOM-001`, `BLK-COPPER-GOLEM-STATUE-001`; `EXP-SIM-003`, `EXP-BLK-008`

The generic leaf locks traversal, sampling arithmetic, old-snapshot block/fluid order and framework
RNG boundaries; the statue leaf fixes its concrete two-float weathering callback and copper-age
neighborhood scan.

## `SIM-005` Loaded does not mean ticking

- **FidelityClass:** `ExactObservableBehavior`
- **Evidence status:** `Confirmed`

### Primary evidence

`OFF-SERVER-001`;
`net.minecraft.server.level.ServerChunkCache#tick(java.util.function.BooleanSupplier,boolean)`;
`net.minecraft.server.level.ServerLevel#shouldTickBlocksAt(long)`;
`net.minecraft.server.level.ServerLevel#isPositionEntityTicking(net.minecraft.core.BlockPos)`;
`COM-WIKI-SIM-001`

### Applies when

A chunk is resident, but its tickets, player distance, or simulation distance may not qualify every
gameplay system to run.

### Behavior and timing

Simulation tickets propagate numeric levels independently of loading tickets. Entity ticking is
level `<=31`; block ticking is level `<=32`; random chunk work uses the level-31 iterator plus a
visible holder and live ticking chunk. Player simulation sources start at
`max(0, 31 - simulationDistance)`. The lowest numeric ticket with the required flag wins.
`PLY-SPECTATOR-CHUNKS-001` fixes whether a spectator supplies the player loading/simulation sources:
the live rule defaults true, and false removes only that distance contribution rather than the
spectator's client tracking view.

### Boundaries and quirks

Only `dragon`, `player_simulation`, `forced`, `portal`, and `ender_pearl` ticket types simulate.
Loading-only types cannot activate random ticks. Timed tickets expire only after their remaining
value becomes negative on an eligible purge, and ordinary freeze skips that purge. A single
`loaded: bool` cannot approximate these gates.

### Verification

**Owners:** `SIM-RANDOM-001`, `PLY-SPECTATOR-CHUNKS-001`, `BLK-BELL-001`; `EXP-SIM-003`,
`EXP-PLY-008`, `EXP-BLK-009`

The generic leaf supplies thresholds, ticket semantics and activity gates; the bell leaf fixes one
transient block-entity ticker whose shake/resonance clock stops across unload/inactivity instead of
accumulating catch-up. Scheduled-tick persistence remains owned by `SIM-SCHEDULE-001`.

## `SIM-006` Freeze, stepping, world-clock time, and empty-server pause are distinct

- **FidelityClass:** `ExactObservableBehavior`
- **Evidence status:** `Confirmed`

### Primary evidence

`OFF-SERVER-001`; `net.minecraft.server.ServerTickRateManager#setFrozen(boolean)`;
`net.minecraft.server.ServerTickRateManager#stepGameIfPaused(int)`;
`net.minecraft.world.TickRateManager#tick()`;
`net.minecraft.world.TickRateManager#isEntityFrozen(net.minecraft.world.entity.Entity)`;
`net.minecraft.client.server.IntegratedServer#tickServer(java.util.function.BooleanSupplier)`;
`net.minecraft.server.MinecraftServer#tickServer(java.util.function.BooleanSupplier)`;
`net.minecraft.server.dedicated.DedicatedServerProperties#pauseWhenEmptySeconds`

### Applies when

`/tick freeze`/step, the global `advance_time` world-clock gamerule, or the dedicated-server
empty-pause setting participates.

### Behavior and timing

Freeze makes each admitted tick snapshot `runGameElements = !isFrozen || stepsRemaining > 0`, then
consumes one positive step. It suppresses border/weather/game-time and world-clock progression,
scheduled ticks, raids, block events, ordinary entity and block-entity callbacks, and GameTests. It
does not suppress functions, sleep/wake evaluation, sky brightness, chunk-source and entity-manager
maintenance, players or entities carrying players, connections, player-list work, status/autosave
bookkeeping, or chunk sending. `advance_time` gates registered world clocks and sleep clock
movement, not the overworld-owned shared game-time increment. Dedicated empty pause and integrated
pause return before base tick admission.

### Boundaries and quirks

Dedicated `pause-when-empty-seconds` defaults to 60 but counts a fixed 20 loop admissions per
configured second at every tick rate; its threshold iteration autosaves once and thereafter
maintains connections. Integrated pause saves once on entry, maintains connections and
`TOTAL_WORLD_TIME` statistics without server ticks, and time-syncs on resume. Freeze is therefore
neither pause nor a stopped main loop.

### Verification

**Owners:** `SIM-PIPELINE-001`; `EXP-SIM-001`, `EXP-SIM-004`, `EXP-SIM-005`

The leaf enumerates the exact frozen/step/sprint/pause gates, packet side effects, and edge-case
vectors.
