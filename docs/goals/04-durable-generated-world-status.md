# Goal 04 Status — Durable Generated World

This ledger is the resumable source of truth for
[Goal 04](04-durable-generated-world.md). Update it in every implementation batch.

## Goal state

| Field | Value |
|---|---|
| State | `Planned` |
| Active batch | — |
| Next unblocked batch | — |
| Depends on | Goal 03 `Complete` |
| Goal plan | [Goal 04 plan](04-durable-generated-world.md) |
| Launch prompt | [Goal 04 prompt](04-durable-generated-world-prompt.md) |
| Blocker | Goal 03 production composition and naming migration are incomplete |

Allowed states are `Planned`, `Ready`, `InProgress`, `Blocked`, and `Complete`. Only one batch may
be `InProgress`.

## Batch ledger

| Batch | State | Depends on | Evidence | Result |
|---|---|---|---|---|
| `G04-P0-B1` | `Pending` | Goal 03 | — | Freeze world production truth |
| `G04-P1-B1` | `Pending` | P0-B1 | — | Add world configuration and migration |
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

## Completion record

| Field | Value |
|---|---|
| Final state | Pending |
| Completion commit | — |
| Remaining required work | Blocked by Goal 03; then all batches |
