# Execution Goals

Execution goals translate Ferrite's architecture and version-locked Minecraft reference into
resumable, commit-sized implementation work. Each goal package contains:

- a normative scope and phased execution plan;
- a persistent status ledger updated by every implementation batch;
- a reusable launch prompt for starting or resuming a persistent Goal-mode execution loop.

## Goal packages

| Goal | Scope | State | Plan | Status | Launch prompt |
|---|---|---|---|---|---|
| Goal 01 — Audited Minecraft Java 26.2 Server Baseline | Region-first server runtime, required C0-C3 protocol, and all source-audited gameplay/catalog behavior | Ready | [Plan](01-audited-minecraft-26.2.md) | [Ledger](01-audited-minecraft-26.2-status.md) | [Prompt](01-audited-minecraft-26.2-prompt.md) |

There is intentionally one active implementation goal. New goals must not be created merely to
avoid a difficult Goal 01 batch or to move unfinished required work out of its denominator.

The plan defines completion. The ledger is the resumable source of truth. The prompt may guide an
executor, but it must not override the plan, ledger, architecture, or version-locked reference.
