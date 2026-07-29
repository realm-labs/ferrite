# ADR-0019: Use a Pinned Lattice Revision as the Region Substrate

## Status

Accepted

## Context

Ferrite needs placement domains, logical sharded Region references, claims, fencing, bounded
handoff, remoting, discovery, and failure testing. Lattice provides those game-backend primitives,
but its public API is evolving and its internal actor lifecycle is not Ferrite gameplay semantics.

The official repository was reviewed at `main` revision
[`a52c54004c782bd18b70d37d929d54cd7d8205f3`](https://github.com/realm-labs/lattice/commit/a52c54004c782bd18b70d37d929d54cd7d8205f3)
on 2026-07-29. It is an MIT-licensed Rust 2024 workspace at version `0.1.0`, with no tags observed.
The reviewed placement crate exposes domain-qualified placement, versioned mapping, claims,
generation fencing, handoff, bounded routing, in-memory/etcd storage, and fault-test surfaces.

## Decision

The machine lock is `docs/adr/lattice.lock.toml`. `G01-P2-B5` must use Git dependencies with the
exact `rev`:

```text
a52c54004c782bd18b70d37d929d54cd7d8205f3
```

Ferrite initially admits `lattice-core`, `lattice-placement`, and `lattice-remoting` behind
`ferrite-region-runtime`. Config, etcd, discovery, Kubernetes, operations, or simulation crates are
added only by the batch that needs them. Production domain crates never depend on Lattice.

Lattice owns placement control, claims, lease fencing, remoting, and handoff coordination. Ferrite
owns:

- logical world ticks and phase order;
- canonical `SimulationRegionKey` encoding and spatial mapping policy;
- Region state, boundary transactions, and entity/player transfer;
- snapshot/journal-tail state movement and recovery point;
- business-level acknowledgement, deduplication, retry, and overload policy;
- canonical state hashes and local/distributed equivalence.

Cargo must never follow `main`, a branch, or an unqualified version. An upgrade is a named batch that
updates this lock, Cargo dependencies, and `Cargo.lock` together after API review.

## Consequences

- Multi-node ownership uses a purpose-built substrate without surrendering game semantics.
- Ferrite carries an adapter and tests against an exact upstream snapshot.
- Upstream fixes are not received incidentally.
- Optional Lattice operational dependencies may increase build cost and are admitted narrowly.

## Alternatives Considered

- Build placement/remoting from scratch: rejected for Goal 01 because it delays gameplay and repeats
  available control-plane work.
- Depend on Lattice `main`: rejected because non-reproducible API drift can change authority.
- Store Ferrite Region state inside Lattice actors: rejected because handoff/recovery compatibility
  would depend on actor internals.

## Migration or Reversal Plan

The adapter permits replacement. A new revision or substrate must pass compile/API review,
local/in-process/multi-process replay equivalence, claim-loss and stale-owner tests, handoff recovery,
message loss/duplication/reordering, and bounded overload tests before acceptance.
