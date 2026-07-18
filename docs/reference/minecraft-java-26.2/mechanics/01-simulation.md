# Simulation leaf rules

## Leaf rule `SIM-PIPELINE-001` — One server tick has ordered ownership boundaries

**Parent:** `SIM-001`, `SIM-002`, `SIM-006`  
**FidelityClass:** `ExactObservableBehavior`  <br>
**EvidenceStatus:** `Confirmed`  <br>
**SourceConclusion:** `SourceInconclusive` — the phase skeleton is source-specified, but overload/freeze coverage and every cross-dimension observable ordering branch are not yet expanded.  <br>
**Applies when:** The server loop admits a gameplay tick; applies once per admitted tick, then once per dimension in the server's dimension iteration order.  
**Authoritative state:** Monotonic server tick count, per-level game/day time, tick-rate state, level queues, chunk source, entities, and block entities are server-owned. Clients may interpolate but never advance these authoritative counters.  
**Transition and ordering:** Measure/record the tick; run server task and connection work; for each level update border, weather/sleep and time; drain scheduled block ticks, then fluid ticks; update raids/chunks; drain block events; tick non-passenger entity roots and their passenger trees; tick block entities; then execute surrounding server work such as commands/functions and network flush according to `net.minecraft.server.MinecraftServer#tickServer(java.util.function.BooleanSupplier)`, `net.minecraft.server.MinecraftServer#tickChildren(java.util.function.BooleanSupplier)`, and `net.minecraft.server.level.ServerLevel#tick(java.util.function.BooleanSupplier)`. A synchronous callback may mutate later phases but does not move its caller to another top-level phase.  
**Branches and aborts:** A frozen gameplay tick skips tick-rate-controlled simulation. Removed entities and invalid block entities abort their own callback. A dimension without eligible chunks still advances level-wide work that is not chunk-gated. Failure in one callback is not a normal branch; vanilla crash-report handling is outside gameplay compatibility.  
**Constants and randomness:** Default target interval is 50 ms (20 Hz). Tick rate changes the target wall interval, not the discrete ordering. No RNG is consumed merely by entering this pipeline; individual phases own their streams.  
**Side effects:** Every queued update, entity movement, inventory mutation, sound, particle, and outbound correction generated during the tick becomes visible in causal order, although networking may batch delivery.  
**Gates:** Tick freeze/step/sprint, integrated-server pause, dedicated-server empty pause, chunk ticking level, entity removal, and block-entity validity. `doDaylightCycle` gates day time only.  
**Boundary cases and quirks:** Falling behind does not create fractional physics steps. Tick sprint intentionally decouples game time from wall time. Cross-dimension ordering is observable only through shared server facilities and must follow the locked dimension iteration rather than be parallelized without an equivalence proof.  
**Evidence:** `Confirmed`; `OFF-SERVER-001`; source locators above. Exact black-box phase probe: `EXP-SIM-001`.  
**Test vectors:** (1) At tick N, schedule a block tick that queues a block event and compare both with an entity and block-entity counter; expect the parent phase order. (2) Overload the server for 100 wall-ms; expect whole ticks, never two half steps. (3) Freeze, step 3, and assert exactly three gameplay transitions.

## Leaf rule `SIM-SCHEDULE-001` — Scheduled block and fluid ticks are bounded priority queues

**Parent:** `SIM-003`  
**FidelityClass:** `ExactObservableBehavior`  <br>
**EvidenceStatus:** `Confirmed`  <br>
**SourceConclusion:** `SourceInconclusive` — queue ordering is source-specified; same-tick insertion, inactive-chunk reinsertion, and cap boundaries still require a dedicated source trace.  <br>
**Applies when:** A block or fluid schedules work for a position in a level.  
**Authoritative state:** Each level owns separate block and fluid schedules whose key includes type and position and whose ordering fields are trigger tick, priority, and insertion sub-order.  
**Transition and ordering:** Scheduling inserts only if an equivalent pending type/position entry is absent. On a level tick, collect entries whose trigger time is due and whose chunks are eligible; order them with `net.minecraft.world.ticks.ScheduledTick#DRAIN_ORDER`, execute block entries before fluid entries, and remove an entry from pending state as it is consumed. An entry scheduled during a callback participates only according to `net.minecraft.world.ticks.LevelTicks#tick(long,int,java.util.function.BiConsumer)`'s current-drain boundary.  
**Branches and aborts:** If the current state/type no longer matches the scheduled type, the consumer produces no type-specific transition. Ineligible/unloaded chunks retain work instead of paying it back as elapsed callbacks. Duplicate requests do not create duplicate executions.  
**Constants and randomness:** The per-level per-queue execution cap is 65,536 entries per tick. Ordering is deterministic from trigger tick, priority and sub-order; no RNG chooses a due entry. Delay units are game ticks and trigger arithmetic is integer.  
**Side effects:** A callback may schedule new work, synchronously update neighbors, mutate blocks/fluids, create entities, and emit level events. Remaining due entries stay queued after the cap.  
**Gates:** Level tick admission, chunk eligibility, freeze, and current type/state validation. Difficulty and player permissions do not directly gate queue execution.  
**Boundary cases and quirks:** Zero-delay is not license for recursive immediate invocation. The block and fluid queues are distinct, so equal timestamps never interleave by a shared global sub-order. Overflow behavior is backlog, not discard.  
**Evidence:** `Confirmed`; `OFF-SERVER-001`; `net.minecraft.world.ticks.LevelTicks#schedule(net.minecraft.world.ticks.ScheduledTick)` and locators in `SIM-003`. Same-tick insertion remains owned by `EXP-SIM-002`.  
**Test vectors:** Schedule the same block twice; expect one callback. Schedule priorities at the same trigger and assert comparator order. Queue 65,537 eligible callbacks; expect 65,536 then one. Unload before due time and reload; expect one delayed callback, not elapsed-time repetition.

## Leaf rule `SIM-RANDOM-001` — Random ticks are sampled attempts, never accumulated obligations

**Parent:** `SIM-004`, `SIM-005`  
**FidelityClass:** `ExactObservableBehavior`  <br>
**EvidenceStatus:** `Cross-checked`  <br>
**SourceConclusion:** `SourceInconclusive` — exact section traversal and RNG-consumption positions remain unexpanded.  <br>
**Applies when:** A ticking chunk section is processed with `randomTickSpeed > 0`.  
**Authoritative state:** The server owns chunk-section state, the per-level random source, gamerule value, and each block/fluid state's random-tick eligibility bit.  
**Transition and ordering:** For every eligible section, perform the configured number of samples; resolve the sampled block position; invoke block random tick only when the current block state reports eligibility, and independently apply the fluid-state check at that position as ordered by `net.minecraft.server.level.ServerLevel#tickChunk(net.minecraft.world.level.chunk.LevelChunk,int)`. The callback observes state at invocation time.  
**Branches and aborts:** Empty/non-ticking sections, an ineligible sampled state, or zero speed yields no callback. A callback that changes the sampled state affects subsequent samples normally. Unloaded time creates no samples.  
**Constants and randomness:** Attempts per eligible section equal the integer gamerule value. Position selection consumes the level's tick RNG; do not substitute a stable per-block timer. Exact RNG consumption and block/fluid order are compatibility-blocking only when deterministic reproduction is required and are assigned to `EXP-SIM-003`.  
**Side effects:** Block/fluid callbacks can change state, schedule work, drop items, grow plants, spread fire, and emit observable events. Failed samples have no gameplay side effect beyond RNG consumption.  
**Gates:** Chunk block-ticking status, section presence, gamerule, state eligibility, and freeze. Natural-spawn distance is not this gate.  
**Boundary cases and quirks:** Increasing speed increases attempts and may sample one position repeatedly; it does not traverse all eligible blocks. Reload resumes sampling with current state rather than catching up.  
**Evidence:** `Confirmed` for sampling/gating; `Cross-checked` for exact random consumption; `OFF-SERVER-001`; locators in `SIM-004`; `EXP-SIM-003`.  
**Test vectors:** Keep an eligible block outside simulation distance for 1,000 ticks then activate; expect no burst. Set speed 0; expect no callbacks. Set a deterministic seed/test random source and compare sampled positions and consumption against vanilla.
