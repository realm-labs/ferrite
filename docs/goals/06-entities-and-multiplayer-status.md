# Goal 06 Status — Entities and Multiplayer

This ledger is the resumable source of truth for
[Goal 06](06-entities-and-multiplayer.md). Update it in every implementation batch.

## Goal state

| Field | Value |
|---|---|
| State | `Planned` |
| Active batch | — |
| Next unblocked batch | — |
| Depends on | Goal 05 `Complete` |
| Goal plan | [Goal 06 plan](06-entities-and-multiplayer.md) |
| Launch prompt | [Goal 06 prompt](06-entities-and-multiplayer-prompt.md) |
| Blocker | Goal 05 authoritative player and survival state is incomplete |

Allowed states are `Planned`, `Ready`, `InProgress`, `Blocked`, and `Complete`. Only one batch may
be `InProgress`.

## Batch ledger

| Batch | State | Depends on | Evidence | Result |
|---|---|---|---|---|
| `G06-P0-B1` | `Pending` | Goal 05 | — | Freeze entity and multiplayer truth |
| `G06-P1-B1` | `Pending` | P0-B1 | — | Integrate player list and lifecycle replication |
| `G06-P1-B2` | `Pending` | P1-B1 | — | Project committed player state to observers |
| `G06-P1-B3` | `Pending` | P1-B2 | — | Add bounded interest and tracking |
| `G06-P2-B1` | `Pending` | P1-B3 | — | Integrate durable entity lifecycle and metadata |
| `G06-P2-B2` | `Pending` | P2-B1 | — | Integrate nonliving entity families |
| `G06-P2-B3` | `Pending` | P2-B2 | — | Integrate fenced entity transfer and recovery |
| `G06-P3-B1` | `Pending` | P2-B3 | — | Integrate melee combat and damage joins |
| `G06-P3-B2` | `Pending` | P3-B1 | — | Integrate projectiles and explosions |
| `G06-P3-B3` | `Pending` | P3-B2 | — | Integrate vehicles, mounts, and passengers |
| `G06-P4-B1` | `Pending` | P3-B3 | — | Integrate mob spawning and despawn |
| `G06-P4-B2` | `Pending` | P4-B1 | — | Integrate navigation and AI scheduling |
| `G06-P4-B3` | `Pending` | P4-B2 | — | Integrate passive, ownership, and trading behavior |
| `G06-P4-B4` | `Pending` | P4-B3 | — | Integrate hostile, raid, boss, and remaining mob behavior |
| `G06-P5-B1` | `Pending` | P4-B4 | — | Run mutual-visibility and combat scenarios |
| `G06-P5-B2` | `Pending` | P5-B1 | — | Run entity, vehicle, mob, transfer, and fault scenarios |
| `G06-P5-B3` | `Pending` | P5-B2 | — | Complete audits and completion evidence |

## Decisions and blockers

| Date | ID | State | Decision or blocker | Evidence / follow-up |
|---|---|---|---|---|
| 2026-08-01 | `G06-D001` | `Accepted` | Shared-world claims require at least two client observation points where the behavior is multiplayer-visible. | Server gap audit |
| 2026-08-01 | `G06-D002` | `Accepted` | Stable entity identity is independent from session-local network IDs. | Goal 01 entity continuity architecture |
| 2026-08-01 | `G06-D003` | `Accepted` | Tracking and projection use committed state and bounded per-observer queues. | Goal 06 observer contract |

## Completion record

| Field | Value |
|---|---|
| Final state | Pending |
| Completion commit | — |
| Remaining required work | Blocked by Goal 05; then all batches |
