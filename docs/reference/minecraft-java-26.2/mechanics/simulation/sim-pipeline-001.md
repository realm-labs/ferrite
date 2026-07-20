# Simulation mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `SIM-PIPELINE-001` — One server tick has ordered ownership boundaries

**Parent:** `SIM-001`, `SIM-002`, `SIM-006`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — the locked server and client jars specify the admitted-tick clock, overload
deadline correction, sprint/freeze/step state machines, dimension iteration, level phases, dedicated
empty pause, and integrated pause branches below. `EXP-SIM-001`, `EXP-SIM-004`, and `EXP-SIM-005`
are regression vectors, not evidence required to fill a source gap.

**Applies when:**

The dedicated or integrated server loop is running. Distinguish a **loop iteration** (packet/task
processing plus a possible tick), an **admitted server tick** (`MinecraftServer#tickServer` reaches
`tickCount++`), and a **normal gameplay tick** (`TickRateManager#runsNormally()` is true after its
per-tick state update). A frozen loop iteration can be an admitted server tick without being a
normal gameplay tick.

**Authoritative state:**

The server owns `nextTickTimeNanos`, the signed 32-bit `tickCount`, tick-rate/freeze/step/sprint
fields, the dedicated-server `emptyTicks`, the integrated-server `paused` flag, a `LinkedHashMap` of
levels, each level's time and queues, chunk activity, entities, and block entities. Wall-clock
deadlines use monotonic nanoseconds; gameplay callbacks observe integer ticks. Clients receive
ticking-state/remaining-step packets and may interpolate presentation, but do not advance these
server fields.

**Transition and ordering:**

Execute the following state machine; each numbered stage completes before the next one begins.

1. On server startup set `nextTickTimeNanos = monotonicNow`. At each loop iteration choose a target
   interval. If the server is not integrated-paused and a sprint is scheduled,
   `checkShouldSprintThisTick()` either admits one sprint tick, decrements `remainingSprintTicks`,
   records its start time, sets the interval to zero, and resets both `nextTickTimeNanos` and
   `lastOverloadWarningNanos` to now; or, when no sprint tick remains, finishes the sprint and falls
   back to the configured interval. Otherwise use `nanosecondsPerTick`.
2. For a nonzero interval, compute `behind = now - nextTickTimeNanos`. Only when
   `behind > 1_000_000_000 ns + 20 * interval` and the warning interval has elapsed
   (`nextTickTimeNanos - lastOverloadWarningNanos >= 10_000_000_000 ns + 100 * interval`) calculate
   `missed = behind / interval`, log it, and add `missed * interval` to both the next deadline and
   last-warning deadline. Vanilla discards those missed deadlines; it does not execute `missed`
   simulation ticks or enlarge the current step. Then add exactly one interval to
   `nextTickTimeNanos` for the current loop.
3. Process queued packets before calling the virtual `tickServer`. A zero-interval sprint supplies
   an always-false time-budget predicate to chunk work; an ordinary iteration supplies the server's
   remaining-time predicate. Integrated-server pause handling occurs inside its override before the
   base admitted-tick stages below.
4. Dedicated empty-pause admission runs before `tickCount++`. When configured seconds `P > 0`, let
   `threshold = wrapping_i32(P * 20)`. If player count is zero and no sprint is scheduled, increment
   `emptyTicks`; otherwise reset it to zero. At `emptyTicks >= threshold`, log and autosave only on
   the equality transition, tick network connections, then return. No tick count, tick-rate state,
   functions, level, player-list, autosave countdown, or tick-time sample advances. A subsequently
   accepted player is seen on a later loop and resets the counter. The dedicated default is
   `P = 60`; `P <= 0` disables this gate. The multiplier remains 20 when `/tick rate` changes, so
   the setting is not literal wall seconds at non-default rates.
5. For an admitted base tick, increment the wrapping signed-32-bit `tickCount`, then call
   `TickRateManager#tick()`. It sets `runGameElements = !isFrozen || frozenTicksToRun > 0`; if the
   step counter is positive, it is decremented immediately, even when not frozen. This single
   snapshot gates every level during the tick.
6. `tickChildren` first suspends outbound flushing for every player, ticks command functions
   unconditionally, invokes the server clock manager only when `runGameElements` (the manager
   advances registered clocks only when the global overworld gamerule `advance_time` is true), and
   every 20 admitted server ticks sends time synchronization independent of freeze. It refreshes
   effective respawn data, then ticks every level serially. The levels map preserves insertion
   order: overworld is inserted first, followed by every non-overworld level in level-stem registry
   iteration order. Custom dimensions therefore follow their locked registry order; dimensions are
   not ticked in parallel.
7. Each `ServerLevel#tick` invalidates its environment-attribute tick cache and snapshots
   `runGameElements`. If true, tick world border then weather. Independently of that flag, evaluate
   the sleeping percentage/deep-sleep branch, optionally move the default clock to the wake marker
   when `advance_time` permits it, wake all sleepers, optionally reset rainy weather when
   `advance_weather` permits it, and update sky brightness. If `runGameElements`, call `tickTime`:
   only a level whose `tickTime` ownership flag is true increments the shared signed-64-bit game
   time by one and then drains the server scheduled-function timer queue at that new time. Normal
   construction gives that ownership to overworld and not to derived dimensions, avoiding one
   game-time increment per dimension. World-clock/day progression is the earlier clock-manager
   phase, not this game-time increment.
8. In a non-debug level with `runGameElements`, drain up to 65,536 scheduled block ticks and then
   65,536 fluid ticks using the same post-time-advance game time. Then tick raids only when running
   normally; tick the chunk source unconditionally; run queued block events only when running
   normally. Clear the level's `handlingTick` flag after these phases.
9. Active chunk tickets reset the level `emptyTime`. Only normal gameplay ticks increment it. While
   `emptyTime < 300`, tick the dragon fight only when running normally, traverse entity roots, then
   process block-entity tickers. Removed entities abort. During freeze, ordinary non-player entities
   are skipped, but players and any entity whose passenger tree contains a player remain eligible;
   non-player roots additionally require entity-ticking range. A root with a valid vehicle is left
   for the vehicle's recursive passenger traversal; an invalid vehicle link is detached.
   Block-entity lists still remove invalid tickers while frozen, but invoke valid tickers only when
   `runGameElements` and their position passes `shouldTickBlocksAt`. At `emptyTime >= 300`, the
   entire dragon/entity/block-entity phase is skipped. Persistent entity-section management and
   debug synchronization still tick afterward.
10. After all dimensions, tick connections, the player list, and debug subscribers unconditionally.
    GameTests tick only when running normally. Server GUI tickables, per-player chunk
    sending/resumed flushing, and the server activity monitor run unconditionally. Back in
    `tickServer`, refresh status at a five-second monotonic interval, decrement the autosave
    counter, autosave when it reaches zero, and update the 100-entry tick-time ring plus the
    exponential smoothing value `smooth = old * 0.8f + elapsedMillis * 0.19999999f`.
11. The loop executes available tasks, then waits until its deadline while continuing to admit
    eligible scheduled tasks. For sprint, the zero interval avoids pacing and elapsed work is
    accumulated for the sprint report. Packet emission may batch at connection boundaries, but
    mutations retain the causal server order above.

### Anchors

`net.minecraft.server.MinecraftServer#runServer()`,
`net.minecraft.server.MinecraftServer#processPacketsAndTick(boolean)`,
`net.minecraft.server.MinecraftServer#tickServer(java.util.function.BooleanSupplier)`,
`net.minecraft.server.MinecraftServer#tickChildren(java.util.function.BooleanSupplier)`,
`net.minecraft.server.MinecraftServer#waitUntilNextTick()`,
`net.minecraft.server.dedicated.DedicatedServer#pauseWhenEmptySeconds()`,
`net.minecraft.client.server.IntegratedServer#tickServer(java.util.function.BooleanSupplier)`,
`net.minecraft.client.server.IntegratedServer#tickPaused()`,
`net.minecraft.world.TickRateManager#tick()`,
`net.minecraft.world.TickRateManager#isEntityFrozen(net.minecraft.world.entity.Entity)`,
`net.minecraft.server.ServerTickRateManager#checkShouldSprintThisTick()`,
`net.minecraft.world.clock.ServerClockManager#tick()`,
`net.minecraft.server.level.ServerLevel#tick(java.util.function.BooleanSupplier)`,
`net.minecraft.server.level.ServerLevel#tickTime()`,
`net.minecraft.server.level.ServerLevel#tickTime`, and
`net.minecraft.world.level.Level#tickBlockEntities()`.

**Branches and aborts:**

`/tick rate` accepts command values from 1.0 through 10,000.0 inclusive; the underlying setter
clamps only the lower bound to 1.0. It stores the input `float` and computes
`nanosecondsPerTick = (long)(1_000_000_000.0 / (double)tickrate)`, truncating toward zero.
`/tick freeze` first stops an active sprint and pending step, then freezes; unfreeze only clears the
frozen flag. `/tick step` succeeds only while frozen and replaces, rather than adds to, the
remaining step count; the command's time argument has minimum one tick and the no-argument form
requests one. `/tick step stop` succeeds only if a positive remainder exists. A sprint request
replaces both sprint counters, records the current frozen flag, unfreezes, and broadcasts state.
Starting another sprint while one is active records the already-unfrozen state, so completion of the
replacement sprint does not restore the freeze that preceded the first sprint. Manual sprint stop
and natural completion both restore `previousIsFrozen`, report measured ticks, and recompute the
autosave interval. Integrated pause suspends a sprint without consuming it because the run loop will
not call its sprint admission method while paused.

**Constants and randomness:**

Default `tickrate = 20.0f` and interval = 50,000,000 ns. Command rate range is `[1.0f, 10_000.0f]`;
internal minimum is `1.0f`. Overload threshold is one second plus 20 configured intervals; warning
spacing is ten seconds plus 100 intervals. The server-status expiry is five seconds. Level
scheduled-tick caps are 65,536 each. Level empty-activity cutoff is 300 normal ticks. Dedicated
empty pause defaults to 60 × 20 loop admissions. The autosave counter starts at 6,000; subsequent
ordinary intervals are `max(100, trunc_i32(tickrate * 300.0f))`, while a sprint estimates rate from
average tick work using `1_000_000_000 / (averageTickNanos + 1)` before the same formula. The
pipeline itself consumes no gameplay RNG; status-player sampling and child phases own their separate
documented consumption. Integer tick counters and the dedicated threshold use Java wrapping
arithmetic; deadline calculations use signed `long` arithmetic with no explicit overflow guard.

**Side effects:**

Freeze and rate changes broadcast `ClientboundTickingStatePacket`; step changes broadcast
`ClientboundTickingStepPacket`, and joining players receive both. A sprint sends state when it
unfreezes/restores and emits its TPS/milliseconds-per-tick command report at completion. The first
dedicated empty-pause iteration autosaves and logs, then later paused iterations only maintain
connections. Entering integrated pause autosaves once; leaving it forces game-time synchronization.
Every level mutation, queued update, entity movement, inventory mutation, sound, particle, game
event, and correction produced by an admitted phase is ordered as above even if network flushing
coalesces packets.

**Gates:**

Base tick admission; dedicated empty pause; integrated client pause; sprint state; frozen/step
snapshot; per-level `tickTime` ownership; debug level; gamerules `advance_time` and
`advance_weather`; active chunk tickets and the 300-tick empty cutoff; block/entity ticking range;
entity removal/vehicle relation/player-passenger exemption; block-entity removal and position
activity. `advance_time` gates registered world clocks and the sleep wake-marker move, not shared
game-time increments or the rest of the world. Difficulty and permissions do not alter the pipeline
after a command has been accepted; command execution itself requires the administrator permission
check.

**Boundary cases and quirks:**

Integrated `tickServer` recomputes `paused = Minecraft.isPaused() || playerList.isEmpty()`. On the
false-to-true transition it saves once. While paused it skips the base tick entirely, maintains
connections, and awards `Stats.TOTAL_WORLD_TIME` once per paused loop to each present player;
`tickCount`, tick-rate stepping, dimensions, and autosave countdown do not advance. On the first
unpaused tick it forces time synchronization before entering the base pipeline. Freeze differs
materially: admitted tick count, functions, sleep/wake handling, sky brightness, chunk-source
maintenance, player/player-ridden entity ticks, entity management, connections, player list, chunk
sending, status and autosave bookkeeping continue. A player can therefore trigger observable work
while the world is frozen. Overload correction drops elapsed deadlines instead of catching
simulation up. A synchronous callback may mutate state consumed by a later phase, but it never moves
its caller into a different top-level phase. Cross-dimension shared-state observations must preserve
overworld-first/registry insertion order. Exceptions converted to crash reports are failure
handling, not an alternate gameplay transition.

**Evidence:**

`Confirmed`; `OFF-SERVER-001`. Control-flow and constants are directly specified by all anchors
above plus `net.minecraft.server.ServerTickRateManager#requestGameToSprint(int)`,
`net.minecraft.server.ServerTickRateManager#finishTickSprint()`,
`net.minecraft.server.ServerTickRateManager#stepGameIfPaused(int)`,
`net.minecraft.server.commands.TickCommand#register(com.mojang.brigadier.CommandDispatcher)`, and
`net.minecraft.server.dedicated.DedicatedServerProperties#pauseWhenEmptySeconds`. `EXP-SIM-001`,
`EXP-SIM-004`, and `EXP-SIM-005` are retained as executable conformance probes.

**Test vectors:**

(1) At default rate record 100 admitted ticks and assert 50,000,000-ns target increments; at rates
3.0 and 10,000.0 assert truncated intervals 333,333,333 ns and 100,000 ns. (2) Stall beyond
`1 s + 20 intervals` while satisfying the warning spacing; assert one current gameplay tick,
deadline advancement by `floor(behind / interval)` missed intervals, and no fractional or catch-up
physics. (3) Freeze, step three, and assert three ticks with `runGameElements = true`, followed by
admitted ticks with it false; also assert functions/connections/players continue in the frozen ticks
while scheduled ticks, border, weather, block events, ordinary entities, and valid block-entity
callbacks do not. (4) During freeze put a player in a non-player vehicle and assert both remain
tick-eligible; compare a vehicle without a player passenger. (5) Arrange one deep-sleep transition
during freeze and record the source-specified wake/time/weather branch. (6) In two dimensions append
to shared storage from the same phase and assert overworld first, then the locked non-overworld
registry order. (7) Run `EXP-SIM-004`; at the default 60-second setting assert 1,199 complete empty
base ticks plus the threshold invocation that only autosaves/connections, then repeat at 10 TPS and
confirm the fixed 1,200-loop threshold takes about 120 wall seconds. (8) Run `EXP-SIM-005` and
assert one integrated pause-entry save, connection/stat maintenance but no admitted server tick,
then one time synchronization on resume. (9) Start a sprint while frozen, replace it mid-run, and
assert the replacement finishes un-frozen; separately let an unreplaced sprint finish and assert
freeze restoration. (10) Run `EXP-SIM-001` and assert scheduled block → block event → eligible
entity → block entity within each level, followed by connection/player/chunk-send stages.
