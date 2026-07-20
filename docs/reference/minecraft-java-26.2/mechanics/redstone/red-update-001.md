# Redstone mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `RED-UPDATE-001` — Power changes propagate through component callbacks, not a global circuit solve

**Parent:** `RED-001`, `RED-002`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceInconclusive` — dust direction ordering and component-specific signal dispatch remain
unexpanded.

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

**Branches and aborts:**

Non-signal source; face not powered; conductor relays direct signal; receiver already in desired
state; scheduled transition already pending; update budget or component lock suppresses a state
change. Experimental redstone data pack behavior is outside default 26.2 unless explicitly enabled.

**Constants and randomness:**

Signal strength is integer 0–15. Generic propagation uses no RNG; direction/orientation and callback
stack define order. Component delays are integer ticks and live in their rules.

**Side effects:**

State writes, further neighbor and comparator updates, scheduled ticks, piston/block events,
block-entity mutations, sounds, particles and client block-state updates.

**Gates:**

Chunk availability/ticking for queued work, component direction, conductor rules, update flags,
feature/data-pack selection and component-specific lock/power predicates.

**Boundary cases and quirks:**

Quasi-connectivity-like behavior arises from the receiver's checked positions and update paths; do
not introduce a generic distance rule. Transient intermediate signals can be observable and power
other components.

**Evidence:**

`Confirmed`; `OFF-SERVER-001`; locators above; exact direction trace `EXP-RED-001`.

**Test vectors:**

Branching wire with two receivers; power through a conductor from each face; state change that is
reverted within one server tick; compare default and redstone-experiments-disabled worlds.
