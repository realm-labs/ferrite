# Goal 02 Status — Minecraft Java 26.2 Client MCP Automation

This ledger is the resumable source of truth for
[Goal 02](02-client-mcp-automation.md). Update it in every implementation batch.

## Goal state

| Field | Value |
|---|---|
| State | `InProgress` |
| Active batch | `G02-P3-B3` |
| Next unblocked batch | `G02-P4-B1` |
| Goal plan | [Goal 02 plan](02-client-mcp-automation.md) |
| Launch prompt | [Goal 02 prompt](02-client-mcp-automation-prompt.md) |
| Minecraft version | `26.2` |
| Client SHA-1 | `2dc72797acbc1b63fc16a11c4ac393605f453754` |
| Blocker | None |

## Batch ledger

| Batch | State | Depends on | Evidence | Result |
|---|---|---|---|---|
| `G02-P0-B1` | `Complete` | — | This row's containing commit; Goal plan, prompt, and ledger | Pure-Java architecture, upstream locks, security boundary, 12 batches, and terminal gates frozen |
| `G02-P1-B1` | `Complete` | P0-B1 | [Build evidence](../reports/goal-02/g02-p1-b1-client-mod-build.md) | Java 25/MC 26.2 client-only Fabric project builds reproducibly with locked and checksum-verified dependencies |
| `G02-P1-B2` | `Complete` | P1-B1 | [Transport evidence](../reports/goal-02/g02-p1-b2-mcp-transport.md) | Authenticated loopback Streamable HTTP, current/legacy MCP lifecycle, bounded resources, discovery, and shutdown pass |
| `G02-P2-B1` | `Complete` | P1-B2 | [Observation evidence](../reports/goal-02/g02-p2-b1-client-observations.md) | Client-thread connection, player, inventory, crosshair, screen, nearby-block, and redacted-error snapshots publish through MCP |
| `G02-P2-B2` | `Complete` | P2-B1 | [Screenshot evidence](../reports/goal-02/g02-p2-b2-framebuffer-screenshot.md) | Single-flight render-thread capture returns bounded integrity-checked PNG image content and metadata |
| `G02-P3-B1` | `Complete` | P2-B1 | [Client-control evidence](../reports/goal-02/g02-p3-b1-tick-fenced-client-control.md) | Bounded receipts, tick waits, real movement/look/jump/sneak/sprint keys, reference-client motion, and disconnect release pass |
| `G02-P3-B2` | `Complete` | P3-B1 | [Interaction evidence](../reports/goal-02/g02-p3-b2-client-interactions.md) | Original key handlers produce mining, swap, drop, use, and chat effects; hotbar and command rejection pass |
| `G02-P3-B3` | `InProgress` | P3-B2 | — | Implementing inventory open/close, cursor, revision-fenced slot validation, and clicks |
| `G02-P4-B1` | `Pending` | P2-B2, P3-B3 | — | — |
| `G02-P4-B2` | `Pending` | P4-B1 | — | — |
| `G02-P5-B1` | `Pending` | P4-B2 | — | — |
| `G02-P5-B2` | `Pending` | P5-B1 | — | — |

## Decisions and blockers

| Date | ID | State | Decision or blocker | Evidence / follow-up |
|---|---|---|---|---|
| 2026-08-01 | `G02-D001` | `Accepted` | Use a pure-Java Fabric mod with MCP embedded in the client; do not add a Rust sidecar merely for language consistency. | User direction |
| 2026-08-01 | `G02-D002` | `Accepted` | Drive normal client input/interaction APIs. Server commands, direct world mutation, and hand-built packets are not gameplay acceptance. | Goal 02 scope and security sections |
| 2026-08-01 | `G02-D003` | `Accepted` | Keep unmodified Quick Play and instrumented MCP acceptance as separate evidence classes. | Goal 02 scope boundary |
| 2026-08-01 | `G02-D004` | `Accepted` | Review the locked CC0 26.2 MCP mod and MIT MCCTP revisions as upstream evidence without vendoring their repositories or binaries. | Goal 02 provenance table |
| 2026-08-01 | `G02-F001` | `Open` | The current `ferrite-server` Quick Play attempt transitions to the client's generic disconnected screen, while the locked original 26.2 reference server accepts and executes MCP-driven gameplay. | Preserve as a server-side P4-B2 acceptance finding; continue client-tool implementation against the reference server |

## Completion record

| Field | Value |
|---|---|
| Final state | `InProgress` |
| Completion commit | — |
| Remaining required work | `G02-P3-B3` through `G02-P5-B2` |
