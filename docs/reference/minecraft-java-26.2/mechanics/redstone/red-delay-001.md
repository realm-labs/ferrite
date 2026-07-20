# Redstone mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `RED-DELAY-001` — Repeaters, observers, and torches schedule component-owned transitions

**Parent:** `RED-003`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Cross-checked`

**SourceConclusion:**

`SourceInconclusive` — repeater delay/lock priority collisions, observer pulses, and torch burnout
need separate slices; comparator behavior is closed by `RED-COMPARATOR-001`.

**Applies when:**

A delayed redstone component receives a relevant neighbor/input change.

**Authoritative state:**

Facing, powered/delay/locked state, pending scheduled tick, input signals and observer/torch
transition history.

**Transition and ordering:**

Neighbor callback samples the remaining component-specific inputs; if desired output differs,
enqueue a tick with component delay and priority; on execution resample inputs, apply
lock/pulse/burnout rules, commit state/output and notify defined outputs. Observer emits a fixed
pulse after detecting its watched-side state change. Anchors include
`net.minecraft.world.level.block.RepeaterBlock`, `net.minecraft.world.level.block.ObserverBlock`,
and `net.minecraft.world.level.block.RedstoneTorchBlock`.

**Branches and aborts:**

Repeater locked; stale scheduled transition; observer pulse already active; torch powered from
attachment or burnout. Each branch may deliberately retain a scheduled tick even if a later input
changes.

**Constants and randomness:**

Signal is clamped to 0–15. Repeater player delay settings correspond to integer game-tick delays;
observer and repeater scheduling use source constants and tick priorities. No RNG selects output.
Exact simultaneous-input waveform is `EXP-RED-002`.

**Side effects:**

Powered/output state, new scheduled ticks, neighbor notifications, click/torch sounds and particles.

**Gates:**

Facing geometry, lock predicate, chunk schedule eligibility, freeze and experimental mode.

**Boundary cases and quirks:**

A scheduled callback must resample rather than blindly applying the state desired when queued.
Pulses shorter than delay can be filtered or transformed depending on component and priority
ordering.

**Evidence:**

`Confirmed` state-machine split; exact collision waveforms `Cross-checked`; `OFF-SERVER-001`; listed
classes; `EXP-RED-002`.

**Test vectors:**

Repeater input pulse shorter/equal/longer than delay; lock before due tick; observer watches two
same-tick changes; torch rapid-toggle burnout sequence.
