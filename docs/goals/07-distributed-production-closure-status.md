# Goal 07 Status — Distributed Production Closure

This ledger is the resumable source of truth for
[Goal 07](07-distributed-production-closure.md). Update it in every implementation batch.

## Goal state

| Field | Value |
|---|---|
| State | `Planned` |
| Active batch | — |
| Next unblocked batch | — |
| Depends on | Goal 06 `Complete` |
| Goal plan | [Goal 07 plan](07-distributed-production-closure.md) |
| Launch prompt | [Goal 07 prompt](07-distributed-production-closure-prompt.md) |
| Blocker | Goals 03–06 local production integration is incomplete |

Allowed states are `Planned`, `Ready`, `InProgress`, `Blocked`, and `Complete`. Only one batch may
be `InProgress`.

## Batch ledger

| Batch | State | Depends on | Evidence | Result |
|---|---|---|---|---|
| `G07-P0-B1` | `Pending` | Goal 06 | — | Freeze production and distributed-storage closure truth; select the formal backend while retaining MinIO plus etcd for local conformance |
| `G07-P1-B1` | `Pending` | P0-B1 | — | Install real Lattice remoting and membership |
| `G07-P1-B2` | `Pending` | P1-B1 | — | Install placement, ownership, fencing, and readiness |
| `G07-P1-B3` | `Pending` | P1-B2 | — | Route gateway commands to remote owners |
| `G07-P1-B4` | `Pending` | P1-B3 | — | Route committed projections to gateways |
| `G07-P2-B1` | `Pending` | P1-B4 | — | Implement the MinIO-plus-etcd local adapter, production backend, and storage-side fencing |
| `G07-P2-B2` | `Pending` | P2-B1 | — | Import local stores and integrate handoff through published durable commits |
| `G07-P2-B3` | `Pending` | P2-B2 | — | Preserve player/entity/session continuity after permanent worker and local-disk loss |
| `G07-P2-B4` | `Pending` | P2-B3 | — | Complete recovery, backup, restore, migration, retention, and disaster-recovery tooling |
| `G07-P2-B5` | `Pending` | P2-B4 | — | Prove drain, rolling replacement, and storage faults with gameplay |
| `G07-P3-B1` | `Pending` | P2-B5 | — | Materialize remaining required protocol batches |
| `G07-P3-B2` | `Pending` | P3-B1 | — | Close remaining required service rows |
| `G07-P3-B3` | `Pending` | P3-B2 | — | Audit and close optional C4 gates |
| `G07-P3-B4` | `Pending` | P3-B3 | — | Resolve required reference-differential gaps |
| `G07-P4-B1` | `Pending` | P3-B4 | — | Complete security and abuse resistance |
| `G07-P4-B2` | `Pending` | P4-B1 | — | Complete observability and operations interfaces |
| `G07-P4-B3` | `Pending` | P4-B2 | — | Validate images, deployments, upgrades, and rollback |
| `G07-P5-B1` | `Pending` | P4-B3 | — | Validate distributed scaling/overhead and publish topology-specific capacity limits from frozen real workloads |
| `G07-P5-B2` | `Pending` | P5-B1 | — | Run multi-node fault injection |
| `G07-P5-B3` | `Pending` | P5-B2 | — | Run deployed exact-client soak and recovery acceptance |
| `G07-P5-B4` | `Pending` | P5-B3 | — | Close production contracts and completion evidence |

## Decisions and blockers

| Date | ID | State | Decision or blocker | Evidence / follow-up |
|---|---|---|---|---|
| 2026-08-01 | `G07-D001` | `Accepted` | Production distributed claims require actual multi-process gameplay; probes and in-process topology are insufficient. | Server gap audit |
| 2026-08-01 | `G07-D002` | `Accepted` | Gateway nodes do not own independent local production worlds once distributed routing is enabled. | Goal 07 ownership contract |
| 2026-08-01 | `G07-D003` | `Accepted` | Required C0–C3 rows must close; C4 services remain default-closed unless explicitly implemented and accepted. | Goal 01 supported contract |
| 2026-08-01 | `G07-D004` | `Accepted` | Capacity evidence reports measured workload/hardware limits and is not an unsupported player-count promise. | Existing capacity policy |
| 2026-08-02 | `G07-D005` | `Accepted` | Distributed production uses location-independent durable Region storage. Per-node disks are caches or migration inputs; immutable payloads and linearizable Region/checkpoint heads enforce storage-side activation fencing and allow recovery on any eligible worker. | [ADR-0026](../adr/0026-location-independent-region-storage.md) |
| 2026-08-02 | `G07-D006` | `Accepted` | Local multi-process development and CI use MinIO for immutable payload objects and etcd for Region/checkpoint heads and writer fences. This reference profile is conformance evidence, not the formal production-backend decision owned by G07-P0-B1. | [ADR-0026 reference profiles](../adr/0026-location-independent-region-storage.md#reference-development-profiles) |
| 2026-08-02 | `G07-D007` | `Accepted` | Local generation/gameplay performance is frozen and measured in Goals 04–06. Goal 07 measures distributed overhead, scaling, recovery, and topology-specific limits; synthetic topology timings cannot replace production workloads. | [Performance engineering contract](../development/performance-engineering.md) |

## Completion record

| Field | Value |
|---|---|
| Final state | Pending |
| Completion commit | — |
| Remaining required work | Blocked by Goals 03–06; then all batches |
