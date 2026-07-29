# ADR-0023: Keep Replay Encoding and Verification in a Dedicated Crate

## Status

Accepted

## Context

Deterministic replay is required for behavior regression, topology comparison, recovery diagnosis,
and fault reproduction. Putting replay envelopes inside simulation or persistence would make those
domains depend on a diagnostic file format.

## Decision

`ferrite-replay` is an explicit workspace crate. It owns:

- versioned replay headers and record envelopes;
- canonical encoding/decoding of replay-owned records;
- Region and world state-hash orchestration;
- command, committed-event, transfer, and topology-observation records;
- replay execution, verification, divergence diagnostics, and golden fixtures.

It consumes stable IDs, semantic commands, committed events, immutable state projections, and
foundation digest types. It does not own gameplay semantics, Minecraft packet captures, persistence
recovery policy, Lattice state, or production scheduling.

Simulation, gameplay, protocol semantics, and persistence schemas do not depend on replay envelopes
or verifier APIs. Runtime instrumentation projects into replay records at explicit commit
boundaries. Replay verification can drive the same local Region runner used by production semantics.

## Consequences

- Replay files can evolve without contaminating save and gameplay schemas.
- Local and distributed traces use the same canonical projections.
- Projection code must be maintained alongside domain changes.
- Packet-level conformance traces remain separate and may reference, but not replace, semantic replay.

## Alternatives Considered

- Put replay in `simulation`: rejected because simulation would own diagnostics and a file format.
- Reuse persistence journals as replay: rejected because recovery and behavioral input histories have
  different retention and compatibility needs.
- Capture only network packets: rejected because packets omit authoritative internal outcomes.

## Migration or Reversal Plan

Replay versions are additive with explicit readers/migrations. Moving ownership requires a
superseding ADR and dependency-direction, golden-vector, and divergence-report compatibility tests.
