# Goal 05 Status — Player Survival Systems

This ledger is the resumable source of truth for
[Goal 05](05-player-survival-systems.md). Update it in every implementation batch.

## Goal state

| Field | Value |
|---|---|
| State | `Planned` |
| Active batch | — |
| Next unblocked batch | — |
| Depends on | Goal 04 `Complete` |
| Goal plan | [Goal 05 plan](05-player-survival-systems.md) |
| Launch prompt | [Goal 05 prompt](05-player-survival-systems-prompt.md) |
| Blocker | Goal 04 reopened Phase 6 authority, exactness, and performance work is incomplete |

Allowed states are `Planned`, `Ready`, `InProgress`, `Blocked`, and `Complete`. Only one batch may
be `InProgress`.

## Batch ledger

| Batch | State | Depends on | Evidence | Result |
|---|---|---|---|---|
| `G05-P0-B1` | `Pending` | Goal 04 | — | Freeze player production truth |
| `G05-P1-B1` | `Pending` | P0-B1 | — | Integrate durable player continuity |
| `G05-P1-B2` | `Pending` | P1-B1 | — | Add admission, identity, and permissions |
| `G05-P1-B3` | `Pending` | P1-B2 | — | Add authoritative inventory and equipment |
| `G05-P2-B1` | `Pending` | P1-B3 | — | Integrate item actions, cooldown, and durability |
| `G05-P2-B2` | `Pending` | P2-B1 | — | Install held-item block placement |
| `G05-P2-B3` | `Pending` | P2-B2 | — | Integrate breaking, loot, drops, and tools |
| `G05-P3-B1` | `Pending` | P2-B3 | — | Install revisioned menu transactions |
| `G05-P3-B2` | `Pending` | P3-B1 | — | Integrate crafting and recipe book |
| `G05-P3-B3` | `Pending` | P3-B2 | — | Integrate furnace, brewing, and workstations |
| `G05-P3-B4` | `Pending` | P3-B3 | — | Integrate advanced menus and text services |
| `G05-P4-B1` | `Pending` | P3-B4 | — | Integrate survival health and hunger |
| `G05-P4-B2` | `Pending` | P4-B1 | — | Integrate death, respawn, and experience |
| `G05-P4-B3` | `Pending` | P4-B2 | — | Integrate progression and statistics |
| `G05-P5-B1` | `Pending` | P4-B3 | — | Integrate chat and text policy |
| `G05-P5-B2` | `Pending` | P5-B1 | — | Integrate commands and typed administration |
| `G05-P5-B3` | `Pending` | P5-B2 | — | Complete operator-facing gameplay configuration |
| `G05-P6-B1` | `Pending` | P5-B3 | — | Run exact-client survival scenarios |
| `G05-P6-B2` | `Pending` | P6-B1 | — | Complete audits and completion evidence |

## Decisions and blockers

| Date | ID | State | Decision or blocker | Evidence / follow-up |
|---|---|---|---|---|
| 2026-08-01 | `G05-D001` | `Accepted` | The exact client is the input/observation boundary; it is never the authority oracle. | Goal 02 contract |
| 2026-08-01 | `G05-D002` | `Accepted` | Inventory, world mutation, loot, and durability use one authoritative transaction where the behavior requires atomicity. | Server gap audit |
| 2026-08-01 | `G05-D003` | `Accepted` | Commands execute typed permission-checked effects and do not bypass Region ownership. | Goal 05 authority contract |
| 2026-08-02 | `G05-D004` | `Accepted` | Each player subsystem carries a frozen performance budget and workload as it lands; Goal 05 may not defer all profiling to final capacity closure. | [Performance engineering contract](../development/performance-engineering.md) |

## Completion record

| Field | Value |
|---|---|
| Final state | Pending |
| Completion commit | — |
| Remaining required work | Blocked by Goal 04 Phase 6; then all Goal 05 batches |
