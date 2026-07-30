# Scheduled Tick Runtime

`G01-P5-S018` implements the source-known `SIM-SCHEDULED-TICKS-001` behavior in
`ferrite-simulation::scheduled_tick`. The module separates immutable records and persistence
arithmetic, per-chunk storage, and level-wide due-head merging. Block and fluid queues use separate
instances and separate 65,536-entry drain budgets.

## Identity, creation, and deduplication

`ScheduledTick<I>` stores a canonical type-identity token, immutable block position, signed
trigger tick, seven-value priority, and signed sub-tick order. Callers must supply the
process-local canonical registry identity for a block or fluid type; resource text, block state,
or another value that can alias one registry object is not an admissible identity token.

`SubTickCounter` returns its old signed 64-bit value and then wraps. Creation sign-extends the
signed 32-bit delay and adds it to game time with wrapping arithmetic. No delay clamp occurs.
Priority decoding clamps values below -3 and above 3 to the nearest endpoint exactly like the
source codec.

Each `ChunkTickContainer` combines a trigger/priority/sub-order heap with a `(type identity,
position)` membership set. A duplicate is rejected without replacing or accelerating the first
record. A different identity at the same position is independent. `ScheduledTickQueue::schedule`
derives the chunk from X/Z and returns `UnregisteredChunk` without loading or creating storage.
The operations layer may attach the vanilla-compatible diagnostic log at that explicit outcome.

## Collection and execution

The local chunk heap compares signed trigger first, then priority, then signed sub-order. A level
drain first admits only registered chunks whose head is due and whose chunk passes the supplied
block-ticking-range predicate. It then merges those heads by priority and sub-order only. This
preserves the important overdue-backlog behavior: it is not a global trigger-time sort.

Collection fills a FIFO snapshot before any callback runs. Polling removes the membership key, so
`has_scheduled_tick` is already false in the callback and the same identity/position can be
rescheduled. Work added by a callback, even with delay zero or a negative delay, remains in the
chunk heap until the next level drain. `will_tick_this_tick` lazily indexes only the remaining
snapshot and removes the current record before its callback.

`tick_matching` consumes every collected record but invokes the callback only when the caller's
current-state predicate still matches its scheduled type. The ordinary `tick` entry point exists
for consumers that perform an equivalent typed check in their adapter. Scheduler work consumes no
random stream.

`LevelScheduledTicks` applies the normal-gameplay and non-debug gates, passes the same game time to
both queues, and gives blocks and fluids independent literal 65,536 caps. Reaching a cap retains
all uncollected work and restores its chunk-head index; it never discards backlog.

## Activity, persistence, clear, and copy

A due but inactive loaded chunk retains its absolute trigger and is reconsidered when activity
returns. Unregistering returns the untouched chunk container. `pack` preserves still-pending saved
records, appends live records in sub-order, and narrows
`triggerTick - saveGameTime` to signed 32-bit. `unpack` adds that delay to the new load time with
signed wrapping and assigns the saved list `-N..-1` sub-orders. Fully unloaded time therefore does
not consume a positive saved delay.

`clear_area` uses inclusive block bounds. It removes matching uncollected records, remaining
snapshot records, and already-run history. To retain the audited quirk, it does not rebuild an
already-materialized `will_tick_this_tick` set, so that query alone can remain stale until ordinary
drain cleanup.

`copy_area` and `copy_area_from` collect matching already-run, collected-not-run, and uncollected
live records. Positions use wrapping component offsets; trigger and priority remain unchanged.
Copied sub-orders use
`sourceSub - minimumSub + maximumSub + 1` with signed wrapping. Destination deduplication remains
authoritative.

## Region ownership and the unresolved tie

The queue owns only explicitly registered chunk containers and has no chunk-loading side effect,
which lets the Phase 5 integration batch bind containers to Region activation generations and move
their packed state during durable handoff. Cross-Region requests must travel as bounded semantic
commands and register only at the authoritative destination. The later integration batch owns
boundary collection, persistence transactions, and projection; it must not weaken the queue's
snapshot or backlog rules.

Restored ticks in different chunks can have equal priority and the same reconstructed negative
sub-order. The source comparator returns equality and saved data contains no global tie breaker.
Ferrite uses chunk-coordinate order as its deterministic fallback so topology and replay do not
depend on container admission order. This is a Ferrite determinism policy, not a claim about
vanilla. `EXP-SIM-002` remains `DeferredExperiment`, and only a committed observation may replace
that statement.

## Verification

The committed owner is `crates/ferrite-simulation/tests/slices/sim_003.rs`. Its tests lock creation
overflow, priority decoding, unregistered and duplicate scheduling, both ordering layers,
collection snapshots and queries, activity and current-state gates, independent caps, pause/debug
admission, save/reload arithmetic, unload continuity, inclusive clear, copy sub-orders,
destination deduplication, and the explicitly Ferrite-only equal-head fallback.
