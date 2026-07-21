# Redstone mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `RED-UPDATE-001` — Power changes propagate through component callbacks, not a global circuit solve

**Parent:** `RED-001`, `RED-002`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — the locked signal-query functions, default dust topology/decay and neighbor
dispatch are deterministic; individual receiver state machines remain in their named leaf owners.

**Applies when:**

A source, conductor, wire, or component changes a state that can alter redstone signal.

**Authoritative state:**

Installed block states, directional weak/direct signal functions, conductor status, component
internal state, scheduled ticks, neighbor-update orientation and block-event queue.

**Transition and ordering:**

Commit the initiating state; notify the defined neighbors; each receiver recomputes only through its
own callback and may immediately write state or schedule a delayed tick; secondary writes
recursively notify according to their flags. Query signal directionally at the instant of each
callback. Vanilla does not first solve a stable graph and atomically commit it. Anchors:
`net.minecraft.world.level.Level#updateNeighborsAt(net.minecraft.core.BlockPos,net.minecraft.world.level.block.Block,net.minecraft.world.level.redstone.Orientation)`,
`net.minecraft.world.level.block.state.BlockBehaviour$BlockStateBase#getSignal(net.minecraft.world.level.BlockGetter,net.minecraft.core.BlockPos,net.minecraft.core.Direction)`,
and
`net.minecraft.world.level.redstone.ExperimentalRedstoneUtils#initialOrientation(net.minecraft.world.level.Level,net.minecraft.core.Direction,net.minecraft.core.Direction)`.

`SignalGetter#getSignal(pos, direction)` first asks the block at `pos` for ordinary signal in that
direction. If that state is a redstone conductor it returns the maximum of that value and direct
signal into `pos`; otherwise it returns only the ordinary value. Direct aggregation queries below,
above, north, south, west, east in that exact order and stops at 15. Best-neighbor signal iterates
the locked `Direction.values()` order, asks each neighbor using that same direction, retains the
maximum and also stops at 15. Control-input queries either admit only diode direct signal, or special-
case redstone block to 15 and dust to its stored power before falling back to another signal source's
direct signal.

Default dust recomputation temporarily disables the wire's own output, takes best neighboring block
signal, restores output and returns 15 immediately for a block signal of 15; otherwise it takes the
maximum of that signal and incoming adjacent-wire power minus one, floored at zero. For each
horizontal direction it samples same-height dust, then samples dust above a conducting neighbor when
the position above this wire is not a conductor, or below a nonconducting neighbor. A changed power
writes the same state with flag 2 only if that exact state object remains installed, then constructs a
hash set containing the wire and all six adjacent positions and calls `updateNeighborsAt` for each
set entry. The default evaluator ignores the incoming orientation and shape-update Boolean; therefore
the set's iteration order is deliberately not promoted to a specified direction sequence.

Connection shape is separate from strength. A horizontal neighbor connects upward when this wire's
top is open, the neighbor permits placement/trapdoor routing, the block above it connects and the
neighbor face is sturdy; otherwise it connects sideways. It also connects sideways directly to
dust, to a repeater along its facing axis, to an observer on its facing side, or to any signal source
when a direction is supplied; a nonconductor can route down to dust. Missing connections normalize
a placed cross/dot so an isolated ordinary placement becomes the four-side cross, while player
interaction with build permission toggles only cross/dot and emits conductor-neighbor updates for
connection booleans that changed.

Placement recomputes power, updates vertical neighbors, then horizontal wire corners and the
above-conductor/below-nonconductor alternatives. Non-piston removal first updates all six neighbors,
recomputes with the old state but no local shape pass, then repeats the corner traversal. Neighbor
callbacks on the server recompute if supported, otherwise drop and remove the wire. A callback whose
source is this wire is suppressed only under the experimental evaluator.

**Branches and aborts:**

Non-signal source; face not powered; conductor relays direct signal; receiver already in desired
state; stale exact-state guard; scheduled transition already pending; unsupported wire; update
budget or component lock suppresses a state change. When `redstone_experiments` is enabled the block
constructs `ExperimentalRedstoneWireEvaluator` per recomputation and the default transaction in this
rule no longer applies.

**Constants and randomness:**

Signal strength is integer 0–15 and default wire loss is exactly one per traversed dust step. Generic
signal queries and default evaluator arithmetic use no RNG. Neighbor collection/dispatch and nested
callback order—not a random draw—define transient order. Component delays live in their leaf rules.

**Side effects:**

State writes, further neighbor and comparator updates, scheduled ticks, piston/block events,
block-entity mutations, sounds, particles and client block-state updates.

**Gates:**

Chunk availability/ticking for queued work, component direction, conductor rules, update flags,
feature/data-pack selection and component-specific lock/power predicates.

**Boundary cases and quirks:**

`getDirectSignalTo`'s fixed face order does not make default dust's hash-set update order stable.
`shouldSignal=false` prevents a wire from feeding itself while computing block input and must be
restored before notifications. Quasi-connectivity arises only from receiver-specific checked
positions/update paths; do not introduce a generic distance rule. Transient signals can be observed.

**Evidence:**

`Confirmed`; `OFF-SERVER-001`; `SignalGetter`, `RedStoneWireBlock`, `RedstoneWireEvaluator`,
`DefaultRedstoneWireEvaluator`, neighbor-updater owners; `EXP-RED-001`.

**Test vectors:**

All six direct-aggregation early exits; conductor/nonconductor ordinary/direct matrix; block input
0/14/15 against adjacent dust 0/1/15; level/up/down dust routes under each conductor/top condition;
exact-state replacement during recompute; place/remove/support-loss; isolated dot/cross toggle;
branching short pulse and nested rollback; default versus explicitly enabled experimental pack.
