# Goal 04 Launch Prompt

Use this prompt to start or resume persistent execution of
[Goal 04](04-durable-generated-world.md).

```text
Create and start a persistent goal with this objective:

Complete Ferrite Goal 04: replace the formal flat terrain and collision fixtures with a configured,
generated, authoritative, durable Minecraft 26.2 world; integrate chunk lifecycle, generation,
projection, collision, environment, dimensions, portals, recovery, and exact-client acceptance.
Continue batch by batch until every terminal gate has committed evidence. Do not set a token budget.

Before changing code, verify Goal 03 is Complete. Read AGENTS.md, the Goal 04 plan and ledger, Goal
03 production manifest and completion record, Goal 01 world behavior and deferred experiments,
world/Region/persistence architecture, Goal 02 MCP operations, and all Goal 04 decisions. Inspect Git
status and preserve user work.

Select the earliest unblocked pending batch and keep exactly one batch InProgress. Use one
authoritative chunk representation for simulation, collision, persistence, and projection. Treat
world/configuration formats as versioned compatibility surfaces. Bound generation, tickets, saves,
and unload work; fence asynchronous results; fail closed on corrupt, stale, or mixed-version state.
Do not claim Mojang same-seed byte identity outside the audited equivalence boundary.

Run focused tests, durability/fault/replay gates, universal Rust gates, and applicable Goal 02 MCP
scenarios. Update the production manifest and ledger with exact evidence, then commit the batch with
a Conventional Commit subject and this exact trailer:

Ferrite-Batch: <batch-id>

Do not push, publish, deploy, rewrite history, or open a pull request without separate authorization.
Continue to the next unblocked batch. Mark Goal 04 Complete only after the formal server no longer
uses flat production fixtures, world restart and migration pass, exact-client exploration and
dimension scenarios pass, universal gates pass, and the worktree is clean.
```
