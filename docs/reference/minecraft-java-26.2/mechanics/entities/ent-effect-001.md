# Entities mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ENT-EFFECT-001` — Status effects merge, tick, expire, and expose attributes in a defined lifecycle

**Parent:** `ENT-006`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceInconclusive` — effect-specific cadence, hidden-chain promotion, attribute ordering, and
removal callbacks remain unexpanded.

**Applies when:**

A living entity gains, updates, removes, cures, or ticks a mob effect instance.

**Authoritative state:**

Effect ID, duration, amplifier, ambient/visible/icon flags, hidden chained instance, effect-derived
attribute modifiers and source entity.

**Transition and ordering:**

On add, test applicability; if absent install and call add hooks; if present merge
strength/duration/flags while preserving weaker/longer instances through the hidden chain as
defined; each living tick test the duration/amplifier cadence, apply tick effect, decrement duration
and promote/remove on expiry; update attribute modifiers and client-visible metadata on transitions.

**Branches and aborts:**

Immune entity; instant effect uses immediate application rather than storage; stronger/longer/equal
merge; duration infinite; cadence false; cure selects only effects matching cure semantics;
death/removal cleanup.

**Constants and randomness:**

Duration is integer ticks; amplifier is integer. Per-effect cadence and arithmetic are behavior
class/data. Instant health/harm scale by amplifier and context with exact integer/float conversion.
Effects consume RNG only where their implementation explicitly does.

**Side effects:**

Health/damage, attributes, AI/movement/visibility, particles/icon metadata, sounds/game events,
criteria and effect add/update/remove synchronization.

**Gates:**

Entity applicability/immunity, effect category/type, duration/cadence, amplifier, cure item,
difficulty and source/context predicates.

**Boundary cases and quirks:**

A hidden weaker effect can reappear after stronger expiry. Attribute modifiers must be
removed/reapplied when amplifier changes, not stacked. Visual flags do not control mechanics.

**Evidence:**

`Confirmed`; `OFF-SERVER-001`; registry/data snapshot; merge/cadence experiment `EXP-ENT-005`.

**Test vectors:**

Strong-short over weak-long; equal amplifier duration update; infinite; cure; instant effect;
amplifier change attribute exactness; save/reload mid-chain; expire on the boundary tick.
