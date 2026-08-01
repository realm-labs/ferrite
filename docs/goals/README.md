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
| Goal 02 — Minecraft 26.2 Client MCP Automation | Pure-Java instrumented-client control, observation, screenshots, launch, and unattended gameplay acceptance | In progress | [Plan](02-client-mcp-automation.md) | [Ledger](02-client-mcp-automation-status.md) | [Prompt](02-client-mcp-automation-prompt.md) |

There is intentionally one active implementation goal. Goal 02 is test infrastructure required to
exercise Goal 01's production integrations; it does not move unfinished server work out of Goal 01's
denominator or redefine gameplay completion.

The plan defines completion. The ledger is the resumable source of truth. The prompt may guide an
executor, but it must not override the plan, ledger, architecture, or version-locked reference.
