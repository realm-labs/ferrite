# 01 — Ticks, Time, Pausing, and Chunk Activity

See the [source lock](../sources.md) and [evidence method](../methodology.md) for the baseline. This page describes server simulation time; client ticks and render frames are covered by `CLI-*`.

## `SIM-001` A fixed game tick is the state-advancement unit

- **FidelityClass:** `ExactObservableBehavior`
- **Evidence status:** `Confirmed`
- **Primary evidence:** `OFF-SERVER-001`; `net.minecraft.server.MinecraftServer#runServer()`; `net.minecraft.server.MinecraftServer#tickServer(java.util.function.BooleanSupplier)`; `net.minecraft.world.TickRateManager#nanosecondsPerTick()`
- **Applies when:** A dedicated or integrated server is advancing an unfrozen simulation.
- **Behavior and timing:** The default target is 20 game ticks per second, or 50 ms per tick. Gameplay advances in discrete ticks. Wall time grows when the machine falls behind, but one gameplay tick does not become a fractional collection of physics steps.
- **Boundaries and quirks:** `/tick rate` can change the target interval, and sprint can temporarily run a requested number of ticks as fast as possible. Ferrite tests must assert gameplay order by tick number, not substitute wall-clock delay.
- **Open verification:** Build a vanilla timing experiment combining severe overload, tick sprint, and autosave.

## `SIM-002` Major per-dimension tick phases are ordered

- **FidelityClass:** `ExactObservableBehavior`
- **Evidence status:** `Confirmed`
- **Primary evidence:** `OFF-SERVER-001`; `net.minecraft.server.MinecraftServer#tickChildren(java.util.function.BooleanSupplier)`; `net.minecraft.server.level.ServerLevel#tick(java.util.function.BooleanSupplier)`
- **Applies when:** `MinecraftServer` advances a server dimension in the current tick.
- **Behavior and timing:** The level tick handles world border, weather/sleep, and time before draining scheduled block ticks and then scheduled fluid ticks. It then advances raids and the chunk source, executes queued block events, and ticks non-passenger entities, passenger trees, and block entities. Nested calls may cause immediate neighbor updates within a phase, but cannot be used to reorder the top-level phases.
- **Boundaries and quirks:** Server-level player, network, function, and save work surrounds dimension ticks. This rule alone does not establish exact cross-dimension ordering.
- **Open verification:** Use a same-tick probe structure to lock the black-box sequence “scheduled block → block event → entity → block entity.”

## `SIM-003` Scheduled ticks order by trigger time, priority, and sub-order

- **FidelityClass:** `ExactObservableBehavior`
- **Evidence status:** `Confirmed`
- **Primary evidence:** `OFF-SERVER-001`; `net.minecraft.world.ticks.LevelTicks#schedule(net.minecraft.world.ticks.ScheduledTick)`; `net.minecraft.world.ticks.LevelTicks#tick(long,int,java.util.function.BiConsumer)`; `net.minecraft.world.ticks.ScheduledTick#DRAIN_ORDER`; `net.minecraft.world.ticks.ScheduledTick#INTRA_TICK_DRAIN_ORDER`
- **Applies when:** A block or fluid requests a tick at the current or a future game time.
- **Behavior and timing:** Eligible entries order first by `triggerTick`, then tick priority, then `subTickOrder`. Container semantics deduplicate same-position, same-type schedules. The block queue drains before the fluid queue; each category executes at most `65,536` due entries per dimension per tick and retains the remainder.
- **Boundaries and quirks:** Whether a newly added already-due entry can execute in the same tick depends on the drain phase and sub-order. It must not be implemented as unbounded recursion.
- **Open verification:** Add separate GameTests for zero-delay scheduling inside a callback, differing priorities, chunk unload/reload, and overflow beyond the cap.

## `SIM-004` Random ticks sample only eligible states in active chunks

- **FidelityClass:** `ExactObservableBehavior`
- **Evidence status:** `Confirmed`
- **Primary evidence:** `OFF-SERVER-001`; `net.minecraft.server.level.ServerLevel#tickChunk(net.minecraft.world.level.chunk.LevelChunk,int)`; `net.minecraft.world.level.block.state.BlockBehaviour$BlockStateBase#isRandomlyTicking()`; `net.minecraft.world.level.block.state.BlockBehaviour$BlockStateBase#randomTick(net.minecraft.server.level.ServerLevel,net.minecraft.core.BlockPos,net.minecraft.util.RandomSource)`
- **Applies when:** A chunk has the activity level required for random ticking and `randomTickSpeed` is greater than zero.
- **Behavior and timing:** Each eligible section receives position samples according to the tick's `randomTickSpeed`. Only block states reporting `isRandomlyTicking()` receive `randomTick`. Fluid states have their own random-tick test and callback. A random tick is not a guaranteed queued event for every block.
- **Boundaries and quirks:** Unloaded or inactive time does not accumulate “owed random ticks”; sampling resumes after activation. Changing `randomTickSpeed` changes attempts, not the process into a traversal.
- **Open verification:** RNG stream, section traversal order, and block/fluid ordering at one sampled position are not yet locked as Ferrite compatibility requirements.

## `SIM-005` Loaded does not mean ticking

- **FidelityClass:** `ExactObservableBehavior`
- **Evidence status:** `Confirmed`
- **Primary evidence:** `OFF-SERVER-001`; `net.minecraft.server.level.ServerChunkCache#tick(java.util.function.BooleanSupplier,boolean)`; `net.minecraft.server.level.ServerLevel#shouldTickBlocksAt(long)`; `net.minecraft.server.level.ServerLevel#isPositionEntityTicking(net.minecraft.core.BlockPos)`; `COM-WIKI-SIM-001`
- **Applies when:** A chunk is resident, but its tickets, player distance, or simulation distance may not qualify every gameplay system to run.
- **Behavior and timing:** Chunk lifecycle must distinguish at least accessible/loaded, block-ticking, and entity-ticking states. Random environment work, natural spawning, block entities, and ordinary entities check the appropriate activity condition. Scheduled ticks suspend with chunk data while unloaded and become candidates only after the chunk ticks again.
- **Boundaries and quirks:** Forced chunks, portal tickets, entity tickets, and spectator state can change the active set. A single `loaded: bool` cannot approximate all gates.
- **Open verification:** Build a matrix for each ticket and simulation-distance edge, recording the first eligible scheduled tick after reload.

## `SIM-006` Freeze, stepping, daylight time, and empty-server pause are distinct

- **FidelityClass:** `ExactObservableBehavior`
- **Evidence status:** `Cross-checked`
- **Primary evidence:** `OFF-SERVER-001`; `net.minecraft.server.ServerTickRateManager#setFrozen(boolean)`; `net.minecraft.server.ServerTickRateManager#stepGameIfPaused(int)`; `net.minecraft.world.TickRateManager#tick()`; `net.minecraft.server.level.ServerLevel#tickTime()`; `net.minecraft.server.MinecraftServer#pauseWhenEmptySeconds()`; `COM-WIKI-RULE-001`
- **Applies when:** `/tick freeze`/step, `doDaylightCycle`, or the dedicated-server empty-pause setting participates.
- **Behavior and timing:** Freeze controls whether gameplay elements advance; step releases an exact requested count of gameplay ticks. `doDaylightCycle` controls only natural day-time advancement and does not freeze the world. Dedicated-server empty pause is a separate run gate. Client-menu pausing matters only for an integrated server that permits pausing.
- **Boundaries and quirks:** Network maintenance, commands, and some server bookkeeping can continue while gameplay is frozen, so freeze is not “stop the main loop.”
- **Open verification:** Enumerate every timer, block event, weather state, entity, and network state that still advances while frozen. The main conclusion has source and community cross-checks but lacks a complete black-box matrix.
