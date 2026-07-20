# Simulation mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `SIM-SCHEDULE-001` — Scheduled block and fluid ticks are bounded priority queues

**Parent:** `SIM-003`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceInconclusive` — all creation, deduplication, collection, cap, execution, activity,
clear/copy, and save/reload branches below are source-specified. The sole unresolved observable is a
cross-chunk tie between restored ticks that have identical priority and reconstructed sub-order; the
source comparator returns equality and the queue/container insertion history is not encoded in the
saved tick.

**Applies when:**

Server code requests a scheduled block or fluid tick, a level drains one of those queues, structure
operations clear/copy queued ticks, or a chunk serializes/reconstructs them. Client-side scheduled
ticks are outside this authoritative rule.

**Authoritative state:**

Each server level owns two independent `LevelTicks` instances: one for block registry objects and
one for fluid registry objects. Each loaded chunk has a `LevelChunkTicks` priority queue plus a
deduplication set. A queued record is
`(type identity, immutable BlockPos, triggerTick: i64, priority, subTickOrder: i64)`. The level also
owns a wrapping signed 64-bit `subTickCount`; ordinary creation returns its old value and then
increments it. During one drain, `toRunThisTick` is a pre-collected FIFO that accepts no later
scheduled insertions (although `clearArea` may remove records), `alreadyRunThisTick` records
consumed entries for area-copy semantics, and a lazily built membership set supports
`willTickThisTick`.

**Transition and ordering:**

1. `LevelAccessor#createTick` computes `triggerTick = gameTime + sign_extended_i32(delay)` with Java
   signed-64 wrap semantics, chooses the supplied priority or `NORMAL`, consumes exactly one
   `nextSubTickCount()`, and stores an immutable position. No delay clamp is applied here.
2. `LevelTicks#schedule` derives the target chunk from X/Z. If that chunk has no registered tick
   container, vanilla logs the invalid request and does not queue it. Otherwise
   `LevelChunkTicks#schedule` inserts only when its custom set has no entry with the same type
   **object identity** and equal position. Trigger, priority, and sub-order do not participate in
   deduplication, so a later duplicate never replaces or accelerates the first request.
3. At `ServerLevel#tick`, only when `TickRateManager#runsNormally()` is true and the level is not a
   debug level, drain block ticks with cap 65,536 and then fluid ticks with a separate cap 65,536.
   The same `gameTime` value is supplied to both queues.
4. Collection snapshots the complete execution batch before any callback runs. A chunk becomes
   eligible only when its head is due (`head.triggerTick <= gameTime`) and
   `DistanceManager#inBlockTickingRange(chunk)` is true. Polling a chunk removes the record from
   both its priority queue and deduplication set, then appends it to the FIFO `toRunThisTick` batch.
5. A chunk's local queue orders by signed `triggerTick`, then enum priority, then signed
   `subTickOrder`. The seven priority enum values, from earliest to latest, are
   `EXTREMELY_HIGH(-3)`, `VERY_HIGH(-2)`, `HIGH(-1)`, `NORMAL(0)`, `LOW(1)`, `VERY_LOW(2)`, and
   `EXTREMELY_LOW(3)`. Among already-due chunk heads, the cross-container merge compares only
   priority then sub-order; the head of each chunk must still respect that chunk's trigger-first
   local queue. Therefore an overdue backlog is not equivalent to globally sorting every entry by
   trigger time.
6. Collection stops exactly when the batch size reaches the caller's cap or no eligible due head
   remains. All collected records execute FIFO. Each record is appended to `alreadyRunThisTick`
   immediately before its consumer. The scheduler itself consumes no RNG.
7. The block consumer rereads the current block state and calls its scheduled-tick callback only if
   `currentState.is(scheduledBlock)`; the fluid consumer similarly requires
   `currentFluidState.is(scheduledFluid)`. A mismatch consumes the scheduled record without invoking
   the type callback. Callback exceptions follow vanilla crash handling and are not a normal
   gameplay branch.
8. Work scheduled by a callback, including delay zero or a negative delay, enters the chunk queue
   after collection has finished and cannot join the current execution batch. Because the old entry
   was removed from the deduplication set during collection, the callback may successfully schedule
   the same type and position for the next eligible level drain. Cleanup clears only the per-drain
   batch/query structures; uncollected chunk queues remain.

### Anchors

`net.minecraft.world.level.LevelAccessor#createTick(net.minecraft.core.BlockPos,java.lang.Object,int,net.minecraft.world.ticks.TickPriority)`,
`net.minecraft.world.level.Level#nextSubTickCount()`,
`net.minecraft.world.ticks.LevelChunkTicks#schedule(net.minecraft.world.ticks.ScheduledTick)`,
`net.minecraft.world.ticks.LevelTicks#tick(long,int,java.util.function.BiConsumer)`,
`net.minecraft.world.ticks.LevelTicks#sortContainersToTick(long)`,
`net.minecraft.world.ticks.LevelTicks#drainContainers(long,int)`,
`net.minecraft.world.ticks.LevelTicks#runCollectedTicks(java.util.function.BiConsumer)`, and
`net.minecraft.server.level.ServerLevel#tick(java.util.function.BooleanSupplier)`.

**Branches and aborts:**

A frozen/non-stepping tick or debug level skips both drains. A due chunk outside block-ticking range
remains queued with its absolute trigger tick and is reconsidered after activity returns. Removing a
container unregisters it from the level without consuming its chunk queue. Scheduling into an
unregistered container is rejected rather than creating/loading a chunk. A current-state/type
mismatch is a successful dequeue with no callback. Each queue has its own cap, so 65,536 block
callbacks do not reduce the fluid allowance. Difficulty, gamerules, and permissions affect callbacks
only if the callback itself checks them.

**Constants and randomness:**

Delay is a signed 32-bit API value added to signed 64-bit game time; trigger/sub-order arithmetic
uses Java two's-complement `long` wrapping and comparisons are signed. The production cap is the
literal 65,536 for blocks and again for fluids. Priorities have integer codec values -3 through 3 in
the enum order listed above. Scheduler operations consume zero random values; scheduled block
callbacks receive the level random source and own any later consumption. Deduplication is
identity-by-type plus value-by-position, not record equality.

**Side effects:**

Collection removes deduplication membership before callbacks. A callback may synchronously mutate
states, neighbors, queues, entities, sounds, particles, or other systems, but no such mutation can
add work to the already-collected batch. `hasScheduledTick(pos,type)` observes only the chunk queue,
so it is false for a collected-not-yet-run record; `willTickThisTick(pos,type)` reports membership
in the remaining collected batch and becomes false as that record is consumed.

**Gates:**

Level tick admission, `runsNormally`, non-debug level, a registered chunk container,
`inBlockTickingRange`, due comparison, the independent per-queue cap, and current block/fluid type
validation. Chunk loading alone is insufficient; entity-ticking range is not the predicate used
here.

**Boundary cases and quirks:**

- Inactive-but-loaded and unloaded time differ. An inactive loaded queue retains its absolute
  trigger and becomes immediately due when activity returns. On chunk serialization, each queued
  trigger is stored as signed 32-bit `delay = (int)(triggerTick - saveGameTime)`; reconstruction
  uses `loadGameTime + delay`, so wall/game time while fully unloaded does not reduce a positive
  saved delay. Already-overdue negative delays remain overdue after reload, subject to 32-bit
  narrowing for extreme differences.
- Loaded pending ticks are reconstructed with per-chunk sub-orders `-N .. -1` in list order, so
  before the level counter wraps they precede ordinary new nonnegative sub-orders on equal
  trigger/priority. Two chunks can reconstruct the same negative sub-order. If their due heads also
  have equal priority, `INTRA_TICK_DRAIN_ORDER` returns equality; the stable cross-chunk choice is
  `SourceInconclusive` because saved data contains no global tie breaker. Reproduce with
  `EXP-SIM-002` by loading the chunks in both orders and recording the first callback. After 2^63
  ordinary creations, `subTickCount` wraps and signed ordering changes; vanilla supplies no overflow
  guard.
- `clearArea` uses an inclusive bounding box, removes matching uncollected chunk entries and
  matching records in the current `toRunThisTick` batch, and removes matching `alreadyRunThisTick`
  history so a subsequent copy cannot reproduce them. Clearing the batch does not rebuild the lazily
  materialized `willTickThisTick` set until normal cleanup, so calling clear after that set was
  built can leave a transient query-only stale membership; this has no callback because the FIFO
  record was removed.
- `copyAreaFrom` gathers matching already-run, collected-not-run, and uncollected records. It
  preserves trigger and priority, offsets the position, and assigns copied sub-orders immediately
  after the source set's maximum while preserving each source record's relative sub-order:
  `copySub = sourceSub - minSub + maxSub + 1`. Normal destination deduplication may reject a copied
  type/position already present.
- Cap overflow is backlog, never discard. Since every type/position pair is deduplicated, producing
  more than 65,536 due records requires distinct pairs across the relevant queue.

**Evidence:**

`Confirmed` for every stated branch except the explicitly isolated restored cross-chunk tie;
`OFF-SERVER-001`. Fields and comparators: `net.minecraft.world.ticks.ScheduledTick#DRAIN_ORDER`,
`net.minecraft.world.ticks.ScheduledTick#INTRA_TICK_DRAIN_ORDER`,
`net.minecraft.world.ticks.ScheduledTick#UNIQUE_TICK_HASH`,
`net.minecraft.world.level.Level#subTickCount`, and `net.minecraft.world.ticks.TickPriority`. Chunk
persistence: `net.minecraft.world.ticks.LevelChunkTicks#pack(long)`,
`net.minecraft.world.ticks.LevelChunkTicks#unpack(long)`, and
`net.minecraft.world.ticks.SavedTick#unpack(long,long)`. The unresolved tie is owned by
`EXP-SIM-002`.

**Test vectors:**

(1) Schedule the same type/position first at delay 20/NORMAL and then delay 1/EXTREMELY_HIGH; assert
one callback at the first record's trigger/priority. (2) In callback A, schedule A at delay 0 and
assert the second callback is in the next admitted level drain, never recursively in the current
batch. (3) Queue 65,537 distinct block pairs plus 65,537 distinct fluid pairs due together; assert
65,536 of each execute, then one of each on the next drain. (4) Make a due chunk inactive while
loaded and assert immediate execution on reactivation; separately unload before due, advance level
time, reload, and assert the saved positive remaining delay is counted from reload. (5) Replace the
block/fluid before due and assert dequeue with no callback. (6) During callback query
`hasScheduledTick` and `willTickThisTick` for the current and next collected records. (7) Clear/copy
an inclusive area before and during drain and assert the queue/history effects above. (8) Restore
equal priority/sub-order ticks in two chunks in both load orders and record the unresolved first
callback exactly as `EXP-SIM-002` specifies.
