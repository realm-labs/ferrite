# Goal 02 Status — Minecraft Java 26.2 Client MCP Automation

This ledger is the resumable source of truth for
[Goal 02](02-client-mcp-automation.md). Update it in every implementation batch.

## Goal state

| Field | Value |
|---|---|
| State | `InProgress` |
| Active batch | `G02-P5-B1` |
| Next unblocked batch | `G02-P5-B2` |
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
| `G02-P3-B3` | `Complete` | P3-B2 | [Inventory-screen evidence](../reports/goal-02/g02-p3-b3-inventory-screen-control.md) | Normal open/close, native cursor, valid slot click, and stale menu-revision rejection pass in the exact client |
| `G02-P4-B1` | `Complete` | P2-B2, P3-B3 | [Launcher evidence](../reports/goal-02/g02-p4-b1-isolated-quick-play-launcher.md) | JDK-only supervisor verifies the exact client, owns isolated state and secrets, reaches reference-server PLAY without clicks, and cleans its process tree on timeout |
| `G02-P4-F001` | `Complete` | P4-B1 | [Fabric payload remediation](../reports/goal-02/g02-p4-f001-fabric-play-custom-payload.md) | Formal entry boundedly decodes and ignores base Play custom payloads instead of faulting the required-family decoder |
| `G02-P4-B2` | `Complete` | P4-F001 | [Scenario evidence](../reports/goal-02/g02-p4-b2-unattended-gameplay-scenarios.md) | One pure-Java runner proves reference movement/interaction/GUI and sustained Ferrite terrain/visual state with secret-free evidence bundles |
| `G02-P5-B1` | `InProgress` | P4-B2 | — | Faulting authentication, framing, overload, disconnect, render, input, process, and artifact boundaries |
| `G02-P5-B2` | `Pending` | P5-B1 | — | — |

## Decisions and blockers

| Date | ID | State | Decision or blocker | Evidence / follow-up |
|---|---|---|---|---|
| 2026-08-01 | `G02-D001` | `Accepted` | Use a pure-Java Fabric mod with MCP embedded in the client; do not add a Rust sidecar merely for language consistency. | User direction |
| 2026-08-01 | `G02-D002` | `Accepted` | Drive normal client input/interaction APIs. Server commands, direct world mutation, and hand-built packets are not gameplay acceptance. | Goal 02 scope and security sections |
| 2026-08-01 | `G02-D003` | `Accepted` | Keep unmodified Quick Play and instrumented MCP acceptance as separate evidence classes. | Goal 02 scope boundary |
| 2026-08-01 | `G02-D004` | `Accepted` | Review the locked CC0 26.2 MCP mod and MIT MCCTP revisions as upstream evidence without vendoring their repositories or binaries. | Goal 02 provenance table |
| 2026-08-01 | `G02-F001` | `Resolved` | The Fabric client sent a bounded Play `minecraft:custom_payload` after terrain entry, but the formal connection driver routed it into the required-family decoder and terminated the session. | `G02-P4-F001` reuses the audited common payload body decoder and applies the base listener's ignore behavior before required-family dispatch |

## Completion record

| Field | Value |
|---|---|
| Final state | `InProgress` |
| Completion commit | — |
| Remaining required work | `G02-P5-B1` through `G02-P5-B2` |
