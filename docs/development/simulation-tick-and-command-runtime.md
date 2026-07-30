# Simulation Tick and Command Runtime

`G01-P5-S017` implements the `SourceSpecified` `SIM-TICK-PIPELINE-001` and
`SIM-COMMAND-LIMIT-001` slices. `ferrite-simulation::server_tick` owns the vanilla-compatible
server-loop state machine; `ferrite-simulation::command_limit` owns synchronous command execution
budgets. These modules complement the Ferrite-native `RegionTickPipeline`: the former decides
which vanilla work is admitted and in what observable order, while the latter enforces Region
commit barriers and deterministic cross-Region delivery.

## Loop pacing and tick-rate state

`DeadlineClock` starts from monotonic time and advances exactly one configured interval per loop.
Ordinary overload correction uses the strict `1s + 20*interval` threshold and the
`10s + 100*interval` warning spacing. It advances the deadline by
`floor(behind/interval)` intervals, records the corrected deadline as the warning time, and still
admits only the current tick. A sprint-admitted loop resets both deadlines to now, uses interval
zero, and supplies an always-false chunk-work time budget.

Tick rate defaults to `20.0F` and 50,000,000 ns. The internal setter preserves Java `Math.max`
and float-to-long behavior; the command boundary admits only `[1.0F, 10000.0F]`. Every admitted
base tick first wraps the signed 32-bit tick count, then snapshots
`!frozen || frozenTicksToRun > 0`. A positive step count is decremented immediately even when the
server is not frozen.

Sprint requests replace both counters, remember the current frozen bit, and unfreeze. A request
made while the previous run snapshot is false waits one ordinary admitted tick before sprint
admission. Each sprint tick records work time and decrements before execution; natural completion
occurs on the following loop check. Replacing a sprint records the already-unfrozen bit, so it does
not restore an earlier freeze. Freeze commands stop active sprint and step state before setting
frozen. Autosave interval and tick-time smoothing retain the locked float order and signed wrapping
arithmetic.

## Dedicated and integrated pause

Dedicated empty pause multiplies configured seconds by a fixed wrapping 20. With no players and no
scheduled sprint, the threshold invocation autosaves, logs, and ticks connections but does not
increment tick count or enter the base pipeline. Later paused loops tick only connections. A player
or sprint resets the counter; a nonpositive/wrapped-nonpositive threshold disables the gate without
rewriting prior counter state.

Integrated pause recomputes from client pause or an empty player list. Entry saves exactly once.
Paused loops tick connections and award total-world-time once per present player, but do not admit a
server tick. Resume synchronizes time before the base pipeline. A paused integrated server does not
call sprint admission and therefore does not consume sprint work.

## Server and level phases

The child plan suspends player flushing, ticks command functions unconditionally, conditionally
ticks registered clocks, sends time every 20 admitted ticks, and visits dimensions serially in the
caller's insertion order. Connections, players and debug subscribers always follow dimensions;
GameTests require normal gameplay; GUI work, chunk sending/resumed flushing and activity monitoring
remain unconditional.

Each level invalidates its environment cache. Border/weather require normal gameplay, but the
sleep/deep-sleep branch and sky brightness do not. Sleep can wake players while frozen; clock-marker
movement and weather reset retain independent gamerules. Normal gameplay calls `tickTime` in every
dimension, while only its owning level wraps shared game time and drains scheduled functions.

Non-debug normal levels drain at most 65,536 block ticks then 65,536 fluid ticks. Raids and block
events require normal gameplay; chunk-source maintenance does not. Active tickets reset
`emptyTime` before a normal tick increments it to one. Below 300, entity and block-entity traversal
remains present. Removed/frozen entities stop before despawn checks; other eligible entities
despawn-check before range and vehicle tests. Invalid block-entity tickers are removed while frozen,
but callbacks require normal gameplay and block-ticking range. Persistent entity management and
debug synchronization always follow.

## Command execution contexts

Both command gamerules default to 65,536. An outer context snapshots
`max(1, max_command_sequence_length)` and raw `max_command_forks`; nested calls reuse it without
resampling or independently draining/closing. Limits remain immutable while live gamerules change.

Newly queued actions move ahead of older deque work while retaining their own insertion order.
Quota is checked only before polling. Ordinary redirect stages, function calls and executable
commands each debit one; custom dispatch has no automatic debit. Multiple redirects inside one
dequeued action can drive quota below zero and finish that action. The next poll abandons pending
work without source failure or result synthesis.

Queue admission checks the existing size against 10,000,000 before adding, so exactly 10,000,000
admits one more entry. The following attempt marks overflow and clears both collections. Overflow
raised inside an action is logged after that action; initial overflow can return through the empty
queue branch without that log.

## Redirects and command-block chains

Every ordinary redirect stage builds a fresh ordered output list. After each modifier result it
requires `accumulated + returned < forkLimit`; equality fails the whole stage without truncation or
execution. Fork-limit errors route through the original source and always reach the tracer.
Nonforked modifier errors abort and report to their current source; forked errors trace but suppress
user failure and continue. Custom modifier/command executors bypass generic debit and fork checking.

Command-block chain traversal snapshots the then-live sequence rule independently, excludes the
initiator, and decrements before each adjacent inspection. Unpowered, conditional and terminating
positions still consume a step; admitted state supplies the next facing. Counter exhaustion warns
even when the final allowed lookup terminates, and warning text rereads and clamps the later live
rule. Every chain block's ordinary command context remains independent of this traversal counter.

## Region integration

The server-loop coordinator creates one immutable admission record containing the wrapping tick,
normal-gameplay bit, ordered dimension list and loop deadline. Regions execute that record through
their existing 20-phase barrier pipeline. Freeze-exempt connection/player work stays with the
server runtime; dimension work remains Region-owned and is reconciled before the next dimension can
observe shared state.

Synchronous command contexts stay on their initiating Region executor. Cross-Region commands are
queued as bounded semantic commands and execute in a later admitted Region phase; they do not split
or extend the caller's thread-local context. Generation fencing and backpressure therefore preserve
the source budget and action order without depending on mailbox arrival.

## Verification

The committed owner is `crates/ferrite-simulation/tests/slices/sim_001.rs`. Its 26 tests cover rate
and deadline boundaries, freeze/step/sprint replacement, both pause modes, server/level ordering,
sleep/time/entity gates, signed bookkeeping, context snapshots, all cost sites, queue insertion and
overflow boundaries, strict redirects, error routing, and chain traversal/warnings.
