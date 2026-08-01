# Goal 04 Status — Durable Generated World

This ledger is the resumable source of truth for
[Goal 04](04-durable-generated-world.md). Update it in every implementation batch.

## Goal state

| Field | Value |
|---|---|
| State | `InProgress` |
| Active batch | `G04-P1-B1` |
| Next unblocked batch | `G04-P1-B2` |
| Depends on | Goal 03 `Complete` |
| Goal plan | [Goal 04 plan](04-durable-generated-world.md) |
| Launch prompt | [Goal 04 prompt](04-durable-generated-world-prompt.md) |
| Blocker | None |

Allowed states are `Planned`, `Ready`, `InProgress`, `Blocked`, and `Complete`. Only one batch may
be `InProgress`.

## Batch ledger

| Batch | State | Depends on | Evidence | Result |
|---|---|---|---|---|
| `G04-P0-B1` | `Complete` | Goal 03 | [production contract](../development/durable-world-production.md), [batch report](../reports/goal-04/g04-p0-b1-world-production-truth.md), production manifest, `cargo ferrite production verify` | Eight world responsibilities, one authoritative representation, versioned formats, migration/failure rules, and terminal acceptance are frozen |
| `G04-P1-B1` | `InProgress` | P0-B1 | — | Implementing server configuration schema 2 and deterministic schema-1 migration |
| `G04-P1-B2` | `Pending` | P1-B1 | — | Add configured durable world bootstrap |
| `G04-P1-B3` | `Pending` | P1-B2 | — | Integrate recovery, save, shutdown, and inspection |
| `G04-P2-B1` | `Pending` | P1-B3 | — | Install the production chunk lifecycle |
| `G04-P2-B2` | `Pending` | P2-B1 | — | Integrate terrain and biome generation stages |
| `G04-P2-B3` | `Pending` | P2-B2 | — | Integrate structures and continuation |
| `G04-P2-B4` | `Pending` | P2-B3 | — | Project committed authoritative chunks |
| `G04-P3-B1` | `Pending` | P2-B4 | — | Install voxel/block-state collision |
| `G04-P3-B2` | `Pending` | P3-B1 | — | Integrate environment and lighting |
| `G04-P3-B3` | `Pending` | P3-B2 | — | Integrate border, spawn, and exploration tickets |
| `G04-P4-B1` | `Pending` | P3-B3 | — | Activate durable dimensions |
| `G04-P4-B2` | `Pending` | P4-B1 | — | Integrate authoritative portal travel |
| `G04-P4-B3` | `Pending` | P4-B2 | — | Prove dimensional restart and fault continuity |
| `G04-P5-B1` | `Pending` | P4-B3 | — | Run exact-client world scenarios |
| `G04-P5-B2` | `Pending` | P5-B1 | — | Complete audits and completion evidence |

## Decisions and blockers

| Date | ID | State | Decision or blocker | Evidence / follow-up |
|---|---|---|---|---|
| 2026-08-01 | `G04-D001` | `Accepted` | One authoritative chunk representation serves simulation, collision, persistence, and projection. | Server gap audit |
| 2026-08-01 | `G04-D002` | `Accepted` | Goal 01 worldgen equivalence boundaries remain explicit; Goal 04 does not guess byte-identical Mojang output. | Goal 01 deferred experiment policy |
| 2026-08-01 | `G04-D003` | `Accepted` | `MinimalTerrain` remains test-only after formal production replacement. | Goal 04 scope |
| 2026-08-01 | `G04-D004` | `Accepted` | Ferrite recovery points are the production format; Mojang Anvil/NBT compatibility is an optional boundary adapter. | [Production contract](../development/durable-world-production.md) |
| 2026-08-01 | `G04-D005` | `Accepted` | Schema 1 migration uses former formal constants and fails on conflicting durable identity. | `G04-P0-B1` compatibility freeze |
| 2026-08-01 | `G04-D006` | `Accepted` | A synced receipt clears dirty state only for the exact captured revision; stale receipts cannot acknowledge newer authority. | Persistence recovery contract |
| 2026-08-01 | `G04-D007` | `Accepted` | Physical stores are contained and sharded by world, dimension, and `SimulationRegionKey`; active logs are never compacted in place. | `G04-P0-B1` storage layout |

## Completion record

| Field | Value |
|---|---|
| Final state | Pending |
| Completion commit | — |
| Remaining required work | All Goal 04 batches; `G04-P0-B1` is ready |
