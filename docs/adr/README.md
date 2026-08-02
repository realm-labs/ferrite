# Ferrite Architecture Decision Records

Architecture decision records are immutable decision snapshots. A changed decision receives a new
ADR that supersedes the old one; accepted records are not silently rewritten to match later code.

Goal 01 Phase 0 accepts the decisions required before implementation:

| ADR | Decision |
|---|---|
| [ADR-0003](0003-server-authoritative-dependency-direction.md) | Server authority and one-way crate dependencies |
| [ADR-0006](0006-deterministic-tick-boundary-order.md) | Fixed Region tick and boundary order |
| [ADR-0008](0008-journaled-persistence-recovery.md) | Journaled persistence and committed-tick recovery |
| [ADR-0015](0015-canonical-encoding-state-hash.md) | Canonical encoding and state hashing |
| [ADR-0017](0017-minecraft-java-26.2-target.md) | Minecraft Java 26.2 compatibility target |
| [ADR-0018](0018-versioned-protocol-adapter.md) | Versioned wire adapter boundary |
| [ADR-0019](0019-pinned-lattice-substrate.md) | Exact Lattice revision and adapter policy |
| [ADR-0020](0020-simulation-region-mapping.md) | Stable SimulationRegion ownership mapping |
| [ADR-0023](0023-dedicated-replay-crate.md) | Dedicated replay ownership |
| [ADR-0024](0024-build-profiles-cache-retention.md) | Build profiles and guarded cache retention |
| [ADR-0025](0025-official-data-import-boundary.md) | Official-data import and legal boundary |
| [ADR-0026](0026-location-independent-region-storage.md) | Location-independent durable Region storage |

The remaining candidates listed in `docs/architecture.md` are recorded when their implementation
batch needs the decision. Each ADR uses `Accepted`, `Superseded`, or `Proposed`; only accepted
records authorize Goal 01 implementation.

The machine-readable [Lattice revision lock](lattice.lock.toml) is part of ADR-0019. The eventual
Cargo Git dependency and `Cargo.lock` entry must resolve to that exact revision.
