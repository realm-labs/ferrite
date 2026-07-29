# ADR-0003: Server Authority and One-Way Dependency Direction

## Status

Accepted

## Context

Ferrite must support an unmodified client, deterministic local execution, and Lattice-backed
multi-node execution without allowing wire, actor-runtime, or persistence details to define
gameplay semantics. A monolithic dependency graph would make those execution modes inseparable and
would make topology-dependent behavior likely.

## Decision

The server owns all authoritative gameplay state and decisions. Clients submit bounded intent and
receive semantic projections plus correction; client prediction never commits authoritative state.

Crate dependencies point inward:

```text
foundation
  <- registry
  <- world
  <- simulation
  <- gameplay

foundation/semantic projections
  <- replay
  <- persistence
  <- protocol
  <- region-runtime
  <- server-runtime
  <- applications
```

This diagram expresses allowed knowledge, not a requirement that every crate depend on every crate
to its left. `region-runtime` is the only production boundary that may depend on Lattice.
Minecraft wire types remain in the versioned protocol adapter. Persistence and replay consume
project-owned stable schemas. Domain crates do not depend on applications, executors, transports,
Lattice types, packet types, or replay file envelopes.

Dependency-direction tests will inspect Cargo metadata. Cross-boundary calls use semantic commands,
immutable projections, committed events, and explicit Region messages.

## Consequences

- The local runner and distributed runtime execute the same gameplay code.
- Adapter replacement is possible without rewriting saves or simulation state.
- Some translation structs and explicit orchestration are required.
- Cycles and convenience re-exports are rejected even when they shorten early implementation.

## Alternatives Considered

- A single server crate: rejected because protocol, storage, and distributed runtime concerns would
  leak into simulation.
- Lattice actors as the gameplay model: rejected because placement and mailbox behavior would define
  game semantics.
- Client-authoritative movement or inventory: rejected because it breaks correction, replay, and
  anti-cheat boundaries.

## Migration or Reversal Plan

Reversal requires a superseding ADR, save/replay compatibility analysis, and local/distributed
equivalence tests. Moving an adapter type inward is an architecture change, not a routine refactor.
