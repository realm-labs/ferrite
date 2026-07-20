# Client-observable behavior mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `CLI-UI-001` — Screen gestures translate to menu click operations; the server owns results

**Parent:** `CLI-005`, `ITM-002`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceInconclusive` — the server replay, 15-bit state-ID wrap, all menu routes/controls and
correction packets are source-specified by `ITM-CONTAINER-*`; screen gesture mapping, touchscreen
branches, double-click timing and client drag cancellation remain unexpanded.

**Applies when:**

A container screen converts mouse/keyboard/touch-like gestures into inventory actions.

**Authoritative state:**

Client screen drag/double-click state and predicted slots; server menu ID/state ID, slots, carried
stack and click algorithm.

**Transition and ordering:**

Client maps the gesture to the 26.2 `ContainerInput` semantic operation plus slot/button arguments,
predicts changed slots/carried stack, sends it with menu and state identifiers; server invokes
`ITM-CONTAINER-001` and `ITM-CONTAINER-CLICK-001`. Current-state replay produces deltas; stale-state
replay still commits and then produces a full resync. Closing and dedicated controls follow
`ITM-CONTAINER-CLOSE-001` and `ITM-CONTAINER-CONTROL-001`. Recipe book placement remains a separate
semantic action.

**Branches and aborts:**

Outside click, touchscreen mode, quick-craft drag phases, double click pickup-all, hotbar swap,
clone, throw, stale menu, screen closes during gesture or dedicated control invalid.

**Constants and randomness:**

Slot coordinates are presentation only; semantic slot indices and click enums are gameplay input.
Double-click/drag timing is client UI state and must match only where it changes emitted operations.
No RNG.

**Side effects:**

Client prediction, click/dedicated request, server slot/container mutations, sounds for UI/device
results, resynchronization and carried-stack return/drop on close.

**Gates:**

Screen/menu type, slot hover/index, mouse button/modifiers, touchscreen, player game mode, server
state ID and slot policies.

**Boundary cases and quirks:**

Quick-craft is a multi-packet state machine; a non-drag input received while it is active resets the
state and consumes that input. Client visual slot coordinates may differ with resource packs without
changing operations. Client changed-slot hashes seed the server's remote comparison mirrors and are
never authoritative slot writes.

**Evidence:**

`Confirmed`; `OFF-CLIENT-001`, `OFF-SERVER-001`; menu registry mapping; gesture table `EXP-CLI-002`.

**Test vectors:**

Every click type from mouse and keyboard; drag interrupted by close; double-click with matching
stacks across containers; stale state; anvil rename concurrent with output take; creative clone in
noncreative mode.
