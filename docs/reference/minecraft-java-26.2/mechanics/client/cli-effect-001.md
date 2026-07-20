# Client-observable behavior mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `CLI-EFFECT-001` — Sounds, particles, entity events, and level events are causal observable outputs

**Parent:** `CLI-006`

**FidelityClass:** `EquivalentPlayerVisibleBehavior`

**EvidenceStatus:** `Cross-checked`

**SourceConclusion:**

`SourceInconclusive` — audience/range, client filtering, prediction deduplication, and every
gameplay emission site remain unexpanded.

**Applies when:**

Server or explicitly local client gameplay emits an audible/visual event.

**Authoritative state:**

Event ID/type, position/source entity, audience/dimension, sound category/volume/pitch/seed,
particle parameters/count/velocity and client option/range state.

**Transition and ordering:**

Gameplay commits its authoritative transition, then invokes the event API at the specified branch;
server selects tracking/audience and sends semantic event; client resolves registry/type and
instantiates sound/particles/entity animation. Purely local feedback is emitted only on paths
designated client-side and must avoid duplicating the later server event.

**Branches and aborts:**

Excluded initiating player; different dimension; outside tracking/range; particle setting/count
suppression; muted category; unknown/removed entity; local prediction later rejected; event maps to
block/entity-specific visualization.

**Constants and randomness:**

Volume affects audible distance by client rules; pitch and particle distributions may consume server
or client RNG depending on event API. A supplied sound seed must be preserved. Cosmetic client RNG
need not match unless it changes observable gameplay timing/count required by a rule.

**Side effects:**

Audible instance, particles, entity animation, subtitles, vibration/game-event listeners only when a
separate server game event is emitted. A sound is not itself a game event.

**Gates:**

Commit branch, side (server/local client), audience/tracking, dimension, options, sound category and
event-specific exclusion.

**Boundary cases and quirks:**

State update, sound, particle and game event are separate side effects; one does not imply the
others. Dedicated servers never synthesize client-only presentation.

**Evidence:**

`Confirmed` separation; exact audience/range per overload `Cross-checked`; `OFF-SERVER-001`,
`OFF-CLIENT-001`; `EXP-CLI-003`.

**Test vectors:**

Initiator-excluded placement sound, two observers at range boundary, dimension mismatch,
muted/particle-minimal clients, predicted event rejected, compare event counts and seeds under
latency.
