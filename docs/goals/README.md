# Execution Goals

Execution goals translate Ferrite's architecture and version-locked Minecraft reference into
resumable, commit-sized implementation work. Each goal package contains:

- a normative scope and phased execution plan;
- a persistent status ledger updated by every implementation batch;
- a reusable launch prompt for starting or resuming a persistent Goal-mode execution loop.

## Goal packages

| Goal | Scope | State | Plan | Status | Launch prompt |
|---|---|---|---|---|---|
| Goal 01 — Audited Minecraft Java 26.2 Server Baseline | Region-first server runtime, required C0-C3 protocol, and all source-audited gameplay/catalog behavior | Complete | [Plan](01-audited-minecraft-26.2.md) | [Ledger](01-audited-minecraft-26.2-status.md) | [Prompt](01-audited-minecraft-26.2-prompt.md) |
| Goal 02 — Minecraft 26.2 Client MCP Automation | Pure-Java instrumented-client control, observation, screenshots, launch, and unattended gameplay acceptance | Complete | [Plan](02-client-mcp-automation.md) | [Ledger](02-client-mcp-automation-status.md) | [Prompt](02-client-mcp-automation-prompt.md) |
| Goal 03 — Production Runtime Integration | Production manifest, responsibility naming, continuity migration, composite Region runtime, explicit dispatch/projection, and exact-client integration evidence | Complete | [Plan](03-production-runtime-integration.md) | [Ledger](03-production-runtime-integration-status.md) | [Prompt](03-production-runtime-integration-prompt.md) |
| Goal 04 — Durable Generated World | Configured durable worlds, chunk lifecycle, generation, collision, environment, dimensions, portals, and exact-client exploration | Complete | [Plan](04-durable-generated-world.md) | [Ledger](04-durable-generated-world-status.md) | [Prompt](04-durable-generated-world-prompt.md) |
| Goal 05 — Player Survival Systems | Durable players, admission, inventory, items, containers, crafting, survival, progression, chat, commands, and operations | Ready | [Plan](05-player-survival-systems.md) | [Ledger](05-player-survival-systems-status.md) | [Prompt](05-player-survival-systems-prompt.md) |
| Goal 06 — Entities and Multiplayer | Shared-player replication, entities, tracking, combat, vehicles, mobs, AI, transfer, persistence, and multi-client acceptance | Planned | [Plan](06-entities-and-multiplayer.md) | [Ledger](06-entities-and-multiplayer-status.md) | [Prompt](06-entities-and-multiplayer-prompt.md) |
| Goal 07 — Distributed Production Closure | Real Lattice gameplay routing, handoff, recovery, required service closure, security, operations, deployment, capacity, faults, and soak | Planned | [Plan](07-distributed-production-closure.md) | [Ledger](07-distributed-production-closure-status.md) | [Prompt](07-distributed-production-closure-prompt.md) |

There is intentionally one ready implementation goal. Goal 04 now supplies the durable generated
world denominator, so Goal 05 is unblocked; Goals 06 and 07 remain planned until their predecessor
is complete. Goal 01 remains the locked
reference, protocol, and behavior-conformance baseline, while Goals 03 through 07 own the separate
production-integration denominator. Goal 02 supplies reusable exact-client input and observation
evidence for every player-visible production batch.

The production sequence is:

```text
Goal 03 production composition and truthful integration ownership
  -> Goal 04 durable generated world
  -> Goal 05 player survival systems
  -> Goal 06 entities and multiplayer
  -> Goal 07 distributed production closure
```

A production feature is complete only when its active Goal traces ordinary client or server input
through semantic dispatch, authoritative Region state, required durable continuity, client
projection or observable effect, focused tests, and applicable Goal 02 exact-client MCP acceptance.
Goal 01 code or conformance evidence alone does not satisfy that production chain.

The plan defines completion. The ledger is the resumable source of truth. The prompt may guide an
executor, but it must not override the plan, ledger, architecture, or version-locked reference.
