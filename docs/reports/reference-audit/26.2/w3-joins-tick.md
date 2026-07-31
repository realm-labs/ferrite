# Wave 3 tick cross-system join falsification audit

Date: 2026-08-01

Worktree: `/Users/mikai/CLionProjects/ferrite-worktrees/w1-jigsaw-engine`

Branch: `codex/ref-joins-tick`

Base: `1f655268dd0c5ab980b58d4fcfdcd22e8daf84d1`

## Scope and disposition

This report-only worker audited `JOIN-02`, `JOIN-03`, `JOIN-05`, `JOIN-06`, `JOIN-07`, `JOIN-08`
and `SURFACE-CROSS-SYSTEM-ORDERING-001`. `JOIN-01` and `JOIN-04` were deliberately excluded. No
shared matrix, surface ledger, completion ledger, implementation manifest or Ferrite runtime file
was edited, and no implementation disposition was marked `Verified`.

The audit covered server-executor and reentrant task admission, per-phase capture/live-read
boundaries, callback and failure prefixes, chunk activity and unload, autosave/clean-close/crash
boundaries, packet publication order, data-reload publication and independently executable
reproduction vectors.

## Locked evidence and method

Only repository-locked inputs were used. Their SHA-256 digests are:

- `client.jar`: `40896ee9f1e2bec3c934daac7e93d41e9e3d9c2f8ae0ca366d52ffbfd1afa290`
- `server.jar`: `cdacdfb25898de5e4b4b0e5ddcc2722f77067e46605709c2d886c000ebb63ec5`
- `server-26.2.jar`: `183c0499c5f855570ee487dd38e141a53f0121f83a0b07a3bac2d8b6698823e8`

Azul Java/Javap 25 (`Zulu25.28+85-CA`, `javap 25`) was used against the locked official server
jar. The generated reports and the current simulation, world-lifecycle, persistence/reload and
data-reload references supplied registry/data identities and already-audited leaf boundaries.

Principal bytecode anchors were:

- `net.minecraft.server.MinecraftServer#processPacketsAndTick`, `#tickServer`, `#tickChildren`,
  `#pollTask`, `#runServer`, `#saveAllChunks`, `#saveEverything`, `#stopServer` and
  `#reloadResources`;
- `net.minecraft.util.thread.BlockableEventLoop#managedBlock`, `#pollTask` and
  `net.minecraft.util.thread.ReentrantBlockableEventLoop#doRunTask`;
- `net.minecraft.server.commands.ReloadCommand#reloadPacks`;
- `net.minecraft.server.level.ServerLevel#tick`, `#tickBlock`, `#tickFluid`, `#runBlockEvents`,
  `#tickNonPassenger` and `#unload`;
- `net.minecraft.world.ticks.LevelTicks#tick`, `#collectTicks` and `#runCollectedTicks`;
- `net.minecraft.world.level.entity.EntityTickList#forEach` and
  `net.minecraft.world.level.Level#tickBlockEntities`;
- `net.minecraft.server.level.ServerChunkCache#tick`, `#tickChunks`,
  `#broadcastChangedChunks`, `#save` and `#close`;
- `net.minecraft.server.level.ChunkMap#processUnloads`, `#scheduleUnload`, `#saveAllChunks` and
  `#tick`; and
- `net.minecraft.server.players.PlayerList#reloadResources`.

## Material falsifications and corrections

### 1. A zero-delay scheduled tick is governed by its queue's collection boundary

The existing `JOIN-02` text incorrectly says all command/callback-created zero-delay block/fluid
work waits for a later drain. `MinecraftServer#processPacketsAndTick` processes queued packets
before `tickServer`, command functions run before levels, and each level then drains block ticks
and fluid ticks as two distinct `LevelTicks#tick` calls. Each call performs complete collection
before running its own callbacks.

Consequences:

- a command or server task that schedules before the target queue's collection can be admitted in
  that level tick when its trigger/activity gates pass;
- a block scheduled-tick callback cannot add to the already-collected block batch, but a block
  callback can schedule a zero-delay fluid tick before the later fluid collection and that fluid
  tick can run in the same level tick;
- a fluid callback schedules after both collections, so a block or fluid tick it creates waits for
  a later level drain; and
- command blocks run in the later block-event phase, after both scheduled queues, so their newly
  scheduled block/fluid work cannot enter those completed drains.

This is a queue-specific snapshot, not a generic command/callback snapshot.

### 2. Content phases use different snapshot and live-read rules

The existing `JOIN-03` phrase “captured block/fluid/entity inputs remain captured” is too broad.
The official paths distinguish:

- scheduled ticks capture `ScheduledTick(type, pos, trigger, priority, sub-order)`, then
  `ServerLevel#tickBlock`/`#tickFluid` rereads the live state and consumes a mismatched record
  without invoking the callback;
- a random-tick attempt captures one `BlockState`; its block callback runs first and the later
  fluid callback is derived from that captured state even if the block callback changed the world;
- `EntityTickList#forEach` pins the active linked map for the iteration and copy-on-modify moves
  additions/removals to the next active map, so iteration membership is captured while each entity
  path still performs its documented live gates;
- `Level#tickBlockEntities` merges already-pending tickers before creating its iterator, defers
  ticker additions made while `tickingBlockEntities` is true, and live-checks removed state,
  `runsNormally` and `shouldTickBlocksAt` for each entry; and
- `ServerLevel#runBlockEvents` is not precollected: it repeatedly removes the first event from the
  live linked set, so an event synchronously added by a running block event can join the same drain.
  An event failing `shouldTickBlocksAt` is instead appended to the reschedule list and restored only
  after the live set becomes empty.

An uncaught callback exception aborts the remaining enclosing phase/tick. Already committed world
writes, RNG draws and direct packet/effect sends are not rolled back; later callbacks are not
implicitly retried.

### 3. World activity gates and phase timing are not interchangeable

The existing `JOIN-05` uses one generic “chunk visibility/activity” gate and says suppression
creates no catch-up. Source fixes separate gates and separate outcomes:

- scheduled ticks require a registered container, due head and block-ticking range; an inactive
  due record remains queued with its absolute trigger and runs overdue when activity returns,
  subject to cap/order;
- random ticks require the stricter entity-ticking simulation level plus visible holder and
  ticking chunk; missed random attempts are never accumulated and their skipped sampling consumes
  no random-tick position/RNG values;
- entity and block-entity paths use their own tick-list, removal, player/passenger and
  `shouldTickBlocksAt` gates rather than the scheduled/random predicate; and
- `ServerLevel#tick` drains scheduled queues before `ServerChunkCache#tick` runs distance updates.
  A promotion only applied by that chunk-source call is too late for the current scheduled drain,
  but may admit random work inside the same chunk-source phase and later entity/block-entity work.

Unload processing also occurs inside the chunk-source phase. An unload committed there can remove
later entity/block-entity admission in that same level tick. Work already executing normally
finishes before a later server task, except for the explicit managed-block reentrancy described
under `JOIN-08`.

Loaded inactive queues retain absolute time. Serialized scheduled ticks reconstruct relative to
load time and retain only the `EXP-SIM-002` equal-head ambiguity; random/entity/block-entity missed
attempts do not become durable callbacks.

### 4. Save is an ordered prefix, not one atomic snapshot

The existing `JOIN-06` “snapshot” wording can imply atomicity. `saveEverything` synchronously saves
players before iterating levels/chunks, while chunk and saved-data writes have their documented
asynchronous chains. Autosave is called after `tickChildren`, but an unload save happens inside a
level's chunk-source phase, before that level's later block-event/entity/block-entity callbacks.
An explicit save command/task may occupy another server-work position. Therefore each save sees the
owner reads reached in that ordered invocation; it is not a simultaneous multi-player/multi-level
snapshot and has no cross-file rollback.

Clean shutdown also needs a narrower statement. `stopServer` stops packet/network admission, saves
and removes players, deactivates chunk tickets, repeatedly ticks chunk sources until their closing
work is empty, performs a flush save, closes levels/storage/resources and joins persistence I/O.
It does not advance normal scheduled/random/entity/block-entity gameplay callbacks or drain the
general gameplay task queue as a semantic catch-up.

`MinecraftServer#runServer` catches an ordinary server exception, records the crash and then calls
`stopServer`, so a handled server crash attempts this shutdown/save path. Only an abrupt process
loss is limited to durable writes completed before termination. Neither case resumes an interrupted
callback frame, iterator or RNG cursor after restart. Failure during crash shutdown/save leaves the
durable prefix source- and storage-owner-specific; filesystem atomicity must not be inferred.

### 5. Dirty chunk projection is per-level and can precede later callbacks

The existing `JOIN-07` incorrectly groups ordinary dirty/tracking/chunk deltas into end-of-tick
phases after all dimensions. In each `ServerLevel#tick`, scheduled work precedes
`ServerChunkCache#tick`; that chunk-source call runs chunk/random work and then
`broadcastChangedChunks`. The same level's block events, entities and block entities run later.

Thus a scheduled or random callback's dirty chunk change can be broadcast in that level's current
chunk-source phase, while an ordinary dirty change first made by a later block-event/entity/block-
entity callback cannot use the already-finished broadcast drain and normally waits for a later
owner drain. Successful block events send their `ClientboundBlockEventPacket` directly at the
event site. Other direct sounds, particles, events, corrections and packets retain their exact leaf
send sites. `MinecraftServer#tickConnection` runs only after all levels, and
`PlayerChunkSender#sendNextChunks` later publishes pending terrain chunks; neither location moves
all ordinary dirty-state publication to one global end-of-tick transaction.

Packet enqueue order is not a claim of transport flush, render or cross-stream simultaneity. A
callback exception preserves already-enqueued direct effects and authoritative mutations, aborts
the later prefix, and creates no generic correction unless the leaf/protocol owner sends one.

### 6. Server-thread reload has a source-specified reentrant window

The existing global “server tasks and synchronous callbacks do not interleave” rule and `JOIN-08`
old-or-new-per-callback wording both miss a concrete exception. `MinecraftServer#reloadResources`
constructs candidate work asynchronously and schedules publication with `thenAcceptAsync(...,
this)`. When called on the server thread, it calls `managedBlock` until the returned future is done.
`BlockableEventLoop#managedBlock` increments `blockingCount`; that makes `shouldRunAllTasks` true,
and repeated `pollTask` calls may execute pending server tasks and chunk-source tasks reentrantly.

Therefore a command/content callback that calls `reloadResources` on the server thread can read the
old data before the call, pause while unrelated queued tasks and the publication task execute, then
resume after the call with the new data. The callback as a whole is not guaranteed to use one
snapshot. This is a source-specified reentrancy boundary, not general parallel mutation.

Candidate preparation still does not mutate the live `resources` pointer. The publication lambda
itself is one non-awaiting server task: close old resources, swap `resources`, persist selection,
apply tags/components, finalize recipes, save/reload player data and advancement state, send
tag/recipe convergence, replace functions, reload templates and rebuild fuel values. No other
server task interleaves inside that lambda. If a publication step fails, the already-reached close,
pointer swap, player saves/packets or other prefix is not rolled back. An off-thread caller returns
the incomplete future without managed-blocking the server thread.

## Exact proposed shared-document replacements

The following wording is proposed for the integration coordinator. It was not applied here.

### Global arbitration item 2

Replace item 2 in `cross-system-ordering.md` with:

> Server tasks and ordinary synchronous callbacks do not interleave: the later admitted operation
> reads the earlier committed state unless its owner captured an earlier snapshot. Fixed tick
> phases, each scheduled queue's precollected batch, entity-iteration maps, random-attempt states,
> selector/target snapshots and reload candidates retain their owner-specific revalidation. The
> explicit exception is a server-thread `managedBlock` owner such as `reloadResources`: while the
> calling stack is paused, the event loop may run pending server and chunk tasks reentrantly; each
> such owner must state the resulting before/inside/after observations.

### `JOIN-02`

Replace the row contract with:

> A command executes synchronously at its server-work position and affects each later uncaptured
> phase. Scheduled block and fluid queues collect independently immediately before their own
> callbacks: work created before the target queue's collection can enter that tick, work created by
> a callback in the same already-collected queue cannot, and a block-tick callback may still create
> a fluid tick before the later fluid collection. Command functions precede levels; command blocks
> run in the later block-event phase. Command writes/feedback reached before failure remain, and
> `/reload` additionally uses the `JOIN-08` managed-block exception. Vector: schedule distinguishable
> zero-delay block/fluid work from packet command, function, block-tick callback, fluid callback and
> command block around both collection boundaries.

### `JOIN-03`

Replace the row contract with:

> Content executes at its `SIM-002` phase with owner-specific capture. Scheduled records capture
> type/position/order but live-revalidate current block/fluid identity; random attempts retain one
> captured block state through block-then-fluid dispatch; entity iteration pins its active map;
> block-entity additions during iteration defer and each ticker live-checks removal/activity; block
> events drain a live linked set and may append same-drain events. Same-target leaves overwrite,
> revalidate/abort or compose exactly as owned. Exception preserves the mutation/RNG/direct-effect
> prefix and aborts later callbacks without rollback or retry. Only owner-classified state/queues
> persist. Vector: instrument every capture/live gate and throw after a distinguishable prefix.

### `JOIN-05`

Replace the row contract with:

> World lifecycle uses phase-specific gates: scheduled work requires a registered due container in
> block-ticking range; random work requires the stricter entity-ticking holder/chunk path; entities
> and block entities use their own live gates. Scheduled collection precedes the level's distance
> updates, so a promotion first applied in chunk-source work misses that scheduled drain but may
> admit same-phase random and later entity/block-entity work. Inactive due scheduled records remain
> overdue and can run on reactivation; missed random/entity/block-entity attempts consume no skipped
> samples and are not replayed. Chunk-source unload can suppress later same-level phases after
> saving its reached prefix. Vector: apply promotion/demotion before collection, in distance update
> and before unload, observing queue retention and RNG/callback counts.

### `JOIN-06`

Replace the row contract with:

> Save/autosave/unload reads the authoritative owner prefix reached at its server-work position;
> player-before-level and per-level/chunk reads are ordered, not a multi-object atomic snapshot.
> Autosave follows `tickChildren`, while unload save can occur inside chunk-source work before
> block-event/entity/block-entity phases. Scheduled queues persist with exact absolute/relative
> encoding and `EXP-SIM-002`; callback frames and RNG do not. Clean close drains chunk closing and
> persistence I/O, not gameplay callbacks. A handled server crash attempts `stopServer`; abrupt
> process loss retains only completed durable writes. No restart resumes a callback. Vector: save
> before/after each phase, inject save/shutdown failure, and compare handled crash with process kill.

### `JOIN-07`

Replace the row contract with:

> Direct packets/events occur at their leaf send sites. Per-level chunk work broadcasts its current
> dirty-holder set after chunk/random work but before that level's block events, entities and block
> entities; mutations first dirtied after that drain normally wait for a later owner drain.
> Connection ticking after all levels and later pending-terrain sending are distinct phases, not one
> atomic projection flush. Failure preserves only the authoritative and already-enqueued effect
> prefix named by the leaf; prediction changes no authoritative phase. Vector: dirty one tracked
> chunk in scheduled, random, block-event, entity and block-entity callbacks and capture packet
> enqueue order across two ticks.

### `JOIN-08`

Replace the row contract with:

> Candidate preparation does not mutate live resources. Publication is one ordered server task
> whose close/swap/tag/component/recipe/player/function/template/fuel prefix has no rollback and no
> task interleave. Off-thread callers observe completion through the returned future. A server-thread
> caller managed-blocks and may run pending server/chunk tasks reentrantly, so the calling callback
> can observe old data before the call and new data after it; consumers run before publication see
> old, after the relevant publication prefix see reached new state, and prior commits are never
> reinterpreted. Vector: call reload from an instrumented callback with queued mutations, gate
> candidate completion, inject each publication failure, and sample tags/recipes/loot before and
> after the call.

### Global arbitration item 5

Replace item 5 with:

> Projection follows each committing owner's direct-send or deferred-drain site. Per-level dirty
> chunk broadcast, entity tracking, connection ticking and pending terrain sending are distinct
> ordered phases; they do not form one global atomic flush. Reliable enqueue order preserves
> already-emitted effects relative to later sends on that stream, while prediction correction,
> acknowledgements and client render/interpolation retain their protocol owners.

### `SURFACE-CROSS-SYSTEM-ORDERING-001` reproduction

Replace the single reproduction string with:

> Run every join in both pair orders and one rejection/failure order. For tick joins, additionally
> gate each scheduled queue collection, chunk distance/update/unload boundary, dirty-holder
> broadcast, autosave/flush/close boundary and reload managed-block window. Assert owner-specific
> captured versus live reads, committed mutation/RNG/packet prefix, queue persistence/reconstruction
> and first client convergence. Treat `EXP-SIM-002` as unresolved rather than a pass, and reject any
> result that infers multi-object atomicity, rollback, catch-up, generation fences or same-seed
> equivalence from a round trip.

The surface can remain `Mapped`; this audit proposes adding `EXP-SIM-002` to its evidence list so
the one directly relevant source-inconclusive ordering boundary is machine-visible. No change to
implementation disposition is proposed.

## Independently executable reproduction vectors

1. **Per-queue collection:** before one level tick, install callbacks for block A and fluid F.
   Schedule from a packet command before `tickChildren`, from a function, from A's block callback,
   from F's fluid callback and from a command block. Record exact tick and phase for each target.
2. **Capture/live matrix:** replace a scheduled target before consumption, mutate a random target's
   world state in its block callback while preserving a captured eligible fluid, add/remove an
   entity during `EntityTickList#forEach`, add a block-entity ticker during its iteration and append
   a block event from a block event. Assert the five distinct source rules above.
3. **Activity timing:** queue an already-due scheduled tick, hold the chunk inactive, then apply its
   promotion immediately before scheduled collection versus through the later distance update.
   Compare scheduled callback, random position/RNG counts, entity and block-entity admission.
4. **Unload and save prefix:** make scheduled, random, block-event, entity and block-entity phases
   write distinct fields. Trigger unload/save at each reachable boundary and reload. Assert only
   fields and scheduled queue records read by that owner position survive; do not demand atomicity.
5. **Clean/error/abrupt stop:** throw after a committed callback prefix and observe `runServer`'s
   crash-report then `stopServer` attempt. Compare a normal stop and an externally terminated
   process. Assert no gameplay callback drain/resume and separately record completed persistence
   writes and shutdown failures.
6. **Projection drain:** dirty the same tracked chunk in scheduled, random, block-event, entity and
   block-entity phases while emitting distinct direct effects. Capture server packet enqueue order
   across the current and next level ticks, plus connection and `sendNextChunks` phases.
7. **Reload reentrancy:** from a server-thread callback, read a tag/recipe/loot predicate, enqueue a
   distinguishable server task, call `reloadResources`, and read again after return. Gate candidate
   completion to prove the queued task and publication execute while the caller is paused. Repeat
   off-thread and inject failure before close, after resource swap and at every refresh step.
8. **Restored scheduled tie:** run `EXP-SIM-002` unchanged with equal priority and reconstructed
   sub-order in two chunks and both load histories. Record the first callback without converting
   the observation into a source claim.

## Preserved source-inconclusive experiments

No completion ledger was edited. All four existing `SourceInconclusive` slices and their exact
experiments remain unchanged:

- `SIM-SCHEDULED-TICKS-001` — `EXP-SIM-002`;
- `ENV-LIGHTING-001` — `EXP-ENV-004`;
- `PLY-BLOCK-BREAK-001` — `EXP-PLY-003`; and
- `WGEN-PIPELINE-EQUIVALENCE-001` — `EXP-WGEN-001`, `EXP-WGEN-005`, `EXP-WGEN-006`.

Only `EXP-SIM-002` directly gates an assigned join. The other three source-inconclusive families
were preserved without reinterpretation. No new source ambiguity was guessed into a conclusion.

## Verification

All commands used Azul Java/Javap 25 where Java tooling was relevant.

- `./target/debug/mc-ref surface coverage` — passed: 92 command roots in 12 mapped families, all
  36 unordered cross-system pairs mapped and all 10 behavior surfaces mapped.
- `./target/debug/mc-ref surface readiness` — passed with the same complete inventory and
  `mc-reference behavior-surface readiness complete`.
- `./target/debug/mc-ref surface verify` — passed.
- `./target/debug/mc-ref experiment verify` — passed, including the unchanged
  `SourceInconclusive` experiment ownership above.
- `./target/debug/mc-ref coverage` and `./target/debug/mc-ref readiness` — passed.
- `cargo run -q -p mc-reference --bin mc-ref -- verify --offline` — passed against only the locked
  offline artifacts.
- `git diff --check` — passed.

Rust formatting, Clippy and Rust tests are not applicable because the sole repository change is
this Markdown audit report.
