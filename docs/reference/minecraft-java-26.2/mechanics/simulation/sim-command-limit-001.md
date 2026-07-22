# Simulation mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `SIM-COMMAND-LIMIT-001` — Command contexts snapshot strict fork and cost budgets

**Parent:** `SIM-001`, `BLK-003`, `CLI-006`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — rule registration, outer/nested context admission, queue ordering, all automatic
cost sites, strict redirect rejection, source/tracer error routing and the independent command-block
chain counter are explicit in locked source.

**Applies when:**

A parsed command or function enters `Commands#executeCommandInContext`, a standard Brigadier
redirect expands command sources, an execution context drains queued actions, or a command block
begins scanning adjacent chain blocks.

**Authoritative state:**

The live nonnegative `max_command_sequence_length` and `max_command_forks` rules; the thread-local
outer execution context; its snapshotted command/fork limits, remaining quota, overflow flag,
current frame depth, tracer, FIFO deque and newly queued actions; command chain position, facing,
initial counter and each block's power/automatic/condition/execution state.

**Transition and ordering:**

Both `MISC` integer rules default to `65536`, accept `0..Integer.MAX_VALUE`, and have no change
callback. Ordinary gamerule storage owns persistence; all execution-context and chain-counter state
is transient.

**Context creation and reuse:** On the outermost `executeCommandInContext` call, the source level's
rules are read once. The command limit is `max(1, max_command_sequence_length)`; the fork limit is
the raw nonnegative `max_command_forks`. A new context is installed in the thread local, the caller
queues initial work, the queue drains synchronously, the tracer closes, and the thread local is
cleared even on failure. A nested call only invokes its consumer against the existing context: it
does not resample either rule, drain or close independently. Rule changes made during that outer
context therefore affect only a later outer context.

**Queue and sequence budget:** Newly queued actions are inserted ahead of older queued work while
preserving their own insertion order, then the deque is polled from the front. Before every poll,
the context stops if remaining quota is at most zero. Exactly three generic sites debit one unit:
each ordinary redirect-modifier stage, each `CallFunction` action before it schedules function
entries, and each ordinary `ExecuteCommand` action immediately before Brigadier runs the
executable. Custom modifier and command executors are not automatically debited at their dispatch
site; any standard work they enqueue remains subject to its own later debit.

The action that consumes the last unit completes. On the next drain iteration, all still-queued
work is abandoned, the server logs that execution stopped after the configured limit, and no
command-source failure or pending result callback is synthesized. Because the configured limit is
clamped to one, a redirect-free simple command still executes when the live rule is zero. A
redirect can instead consume that sole unit and leave its executable queued but unrun. The counter
measures charged execution actions, not command strings, affected targets or queue entries.

`queueNext` also has a fixed defensive queue boundary independent of either rule. It checks the
existing new-plus-deque size against `10,000,000` before adding, so size `10,000,000` admits one
last entry; the next enqueue sets overflow and clears both collections. Overflow raised by a
dequeued action is logged after that action and stops draining. If initial setup overflows before
the first poll, the cleared empty deque returns through the null-entry branch before that log.

**Fork budget:** Each ordinary redirect stage starts a fresh output list and processes its current
sources in order. After a source's modifier returns a collection but before adding it, the stage
tests `accumulatedSize + returnedSize >= forkLimit`. Reaching equality is failure: admitted output
must remain strictly less than the configured limit. The whole stage aborts with
`command.forkLimit(limit)`; it never truncates, never schedules the executable, discards even
previously accumulated outputs, and invokes no command result callback for them. Limit zero thus
rejects every ordinary redirect evaluation that processes a source, including one whose modifier
returns an empty collection.

The limit error is handled against the original command source. A tracer always receives it; a
nonforked chain also sends ordinary source failure, while a chain already marked forked suppresses
that user-facing failure. Ordinary modifier syntax errors instead belong to their current source:
nonforked execution aborts, while forked execution records the error and continues other sources.
A `CustomModifierExecutor` takes its dedicated control path before the generic debit and fork-size
test, then returns; only standard work that it later queues reaches these generic boundaries.

**Command-block chain budget:** After the initiating command block finishes, `executeChain` reads
the then-live `max_command_sequence_length` once without the context's clamp. The initiator is not
counted. Each permitted iteration decrements first, moves one position in the current facing, then
checks chain-block state, block entity, sequence mode, power/automatic admission, condition and
command result. Skipped unpowered blocks and inspected terminating positions still consume a step;
an admitted block supplies the next facing.

When the counter is at most zero on exit, the server warns that the chain tried to execute more
than the rule value. This includes zero before any adjacent lookup and equality after the final
permitted inspection, even if that inspection itself found a terminating non-chain position. The
warning rereads the live rule and prints `max(currentValue, 0)`, not the initial counter. A root
command can therefore change the value before the chain snapshot; a later chain command cannot
change the remaining traversal counter, but its change affects subsequent blocks' independent
command contexts and the eventual warning text.

Each command block's synchronous ordinary dispatch completes and clears its own outer execution
context before traversal proceeds. The chain-step budget is therefore not pooled with any block's
command-action quota. `BLK-COMMAND-001` owns root/chain admission, carrier state, success counts,
timestamps, comparator updates and same-game-time termination around this counter.

**Branches and aborts:**

Outer/nested context; live values zero/one/greater; ordinary/custom redirect and executable;
redirected/forked flag; modifier success/error and returned cardinality below/equal/above limit;
quota positive/exhausted; queue normal/overflow; chain early termination/equality exhaustion;
power/automatic/condition/command result; rule mutation by root or chain command.

**Constants and randomness:**

Both defaults are `65536`; minimum is `0`; the context command-limit floor is `1`; the defensive
queue constant is `10,000,000`. There is no RNG. Source and queue insertion order are observable.

**Side effects:**

Command/function effects and callbacks admitted before exhaustion, source or tracer errors, server
info/error/warning logs, queued-action abandonment, and command-block carrier/comparator effects
delegated to `BLK-COMMAND-001`. Neither limit rolls back already completed work.

**Gates:**

Outermost context creation; the two snapshotted rules; automatic debit site; redirect kind and
cardinality; forked flag; quota before poll; fixed queue overflow; command-block chain counter and
the block admission gates owned by `BLK-COMMAND-001`.

**Boundary cases and quirks:**

Fork equality fails rather than admitting exactly `limit` outputs. Fork zero rejects even an empty
ordinary redirect result. Sequence zero still permits one redirect-free simple executable but no
chain-position inspection. Exhaustion has no source failure and does not complete abandoned
callbacks. A final permitted chain inspection emits the same warning as a provably longer chain.
Mid-context rule changes do not resize that context; chain traversal, later per-block contexts and
warning text can consequently observe three different snapshots of the same live rule.

**Evidence:**

`OFF-SERVER-001`; `OFF-REPORT-001`; `net.minecraft.world.level.gamerules.GameRules`;
`net.minecraft.commands.Commands#executeCommandInContext`;
`net.minecraft.commands.execution.ExecutionContext#queueNext`;
`net.minecraft.commands.execution.ExecutionContext#runCommandQueue`;
`net.minecraft.commands.execution.ExecutionContext#incrementCost`;
`net.minecraft.commands.execution.tasks.BuildContexts#execute`;
`net.minecraft.commands.execution.tasks.CallFunction#execute`;
`net.minecraft.commands.execution.tasks.ExecuteCommand#execute`;
`net.minecraft.commands.CommandSourceStack#handleError`;
`net.minecraft.world.level.block.CommandBlock#executeChain`; `BLK-COMMAND-001`; `EXP-SIM-006`.

**Test vectors:**

Sweep both rules through `0/1/2/65536`; count every charged action for simple, redirected, forked,
nested and recursive-function inputs. Return redirect collections with `limit-1`, `limit`, greater
and empty cardinalities; assert no truncation, source/tracer/result behavior and source order.
Change both rules inside an active context and compare the next outer context. Exhaust quota with
queued callbacks and force the defensive overflow both during initial setup and a dequeued action.
For command blocks, vary chain length, terminating state on the final inspection, facing turns,
skipped/conditional/same-time branches and root/middle rule mutations; assert visits, executions,
success/output and all three observed rule snapshots.
