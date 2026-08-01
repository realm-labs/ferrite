# Goal 03 Status — Production Runtime Integration

This ledger is the resumable source of truth for
[Goal 03](03-production-runtime-integration.md). Update it in every implementation batch.

## Goal state

| Field | Value |
|---|---|
| State | `InProgress` |
| Active batch | — |
| Next unblocked batch | `G03-P0-B2` |
| Depends on | Goal 01 and Goal 02 `Complete` |
| Goal plan | [Goal 03 plan](03-production-runtime-integration.md) |
| Launch prompt | [Goal 03 prompt](03-production-runtime-integration-prompt.md) |
| Blocker | None |

Allowed states are `Planned`, `Ready`, `InProgress`, `Blocked`, and `Complete`. Only one batch may
be `InProgress`.

## Batch ledger

| Batch | State | Depends on | Evidence | Result |
|---|---|---|---|---|
| `G03-P0-B1` | `Complete` | — | [Goal plan](03-production-runtime-integration.md), [launch prompt](03-production-runtime-integration-prompt.md), roadmap commit `3bb5605`, and this row's containing commit | Production evidence chain, scope, responsibility vocabulary, migration rules, dependency order, batches, and terminal gates are frozen |
| `G03-P0-B2` | `Pending` | P0-B1 | — | Create the production-integration manifest |
| `G03-P1-B1` | `Pending` | P0-B2 | — | Rename active simulation and player runtime architecture |
| `G03-P1-B2` | `Pending` | P1-B1 | — | Rename active entity, world, and service test architecture |
| `G03-P1-B3` | `Pending` | P1-B2 | — | Migrate phase-numbered continuity identities and inspection |
| `G03-P2-B1` | `Pending` | P1-B3 | — | Define composite runtime state and deterministic order |
| `G03-P2-B2` | `Pending` | P2-B1 | — | Integrate simulation and player service boundaries |
| `G03-P2-B3` | `Pending` | P2-B2 | — | Integrate entity, world, transfer, and continuity boundaries |
| `G03-P2-B4` | `Pending` | P2-B3 | — | Install the composite runtime in the formal gateway |
| `G03-P3-B1` | `Pending` | P2-B4 | — | Make serverbound dispatch outcomes explicit |
| `G03-P3-B2` | `Pending` | P3-B1 | — | Route post-commit client projections |
| `G03-P3-B3` | `Pending` | P3-B2 | — | Prove ingress-to-projection replay and faults |
| `G03-P4-B1` | `Pending` | P3-B3 | — | Run exact-client MCP composite-runtime scenarios |
| `G03-P4-B2` | `Pending` | P4-B1 | — | Complete naming, manifest, migration, and acceptance audits |

## Decisions and blockers

| Date | ID | State | Decision or blocker | Evidence / follow-up |
|---|---|---|---|---|
| 2026-08-01 | `G03-D001` | `Accepted` | Goal 01 remains the completed audited baseline; Goal 03 owns production integration truth. | User-requested server gap audit |
| 2026-08-01 | `G03-D002` | `Accepted` | Historical Goal/Phase evidence keeps stable names; active architecture uses responsibility names. | Goal 03 naming contract |
| 2026-08-01 | `G03-D003` | `Accepted` | Persisted phase identities require versioned migration and cannot be cosmetically renamed. | Goal 03 migration contract |
| 2026-08-01 | `G03-D004` | `Accepted` | Player-visible integration requires Goal 02 exact-client MCP evidence. | Goal 02 completion boundary |

## Completion record

| Field | Value |
|---|---|
| Final state | Pending |
| Completion commit | — |
| Remaining required work | `G03-P0-B2` through `G03-P4-B2` |
