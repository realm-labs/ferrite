# Goal 04 Launch Prompt

Use this prompt to start or resume persistent execution of
[Goal 04](04-durable-generated-world.md).

```text
Create and start a persistent goal with this objective:

Complete Ferrite Goal 04: replace the formal flat terrain and collision fixtures with a configured,
generated, authoritative, durable Minecraft 26.2 world whose same-input normalized semantic state
matches the locked vanilla server; integrate chunk lifecycle, generation, projection, collision,
environment, dimensions, portals, recovery, and exact-client acceptance.
Continue batch by batch until every terminal gate has committed evidence. Do not set a token budget.

Before changing code, verify Goal 03 is Complete. Read AGENTS.md, the Goal 04 plan and ledger, Goal
03 production manifest and completion record, Goal 01 world behavior and deferred experiments,
world/Region/persistence architecture, Goal 02 MCP operations, and all Goal 04 decisions. Inspect Git
status and preserve user work. Read the performance engineering contract and the version-locked
third-party implementation source register before generation performance work. Follow the worldgen
execution architecture for the exact plan, candidate builder, status DAG, priority, pools, Region
commit boundary, and distributed task granularity.

Select the earliest unblocked pending batch and keep exactly one batch InProgress. Use one
authoritative chunk representation for simulation, collision, persistence, and projection. Treat
world/configuration formats as versioned compatibility surfaces. Ferrite persistence may use native
Region logs and snapshots, but must preserve every vanilla-significant field; Anvil/NBT import or
export is a separate adapter, not a live authority. Keep recovery identities and receipts independent
of node filesystem paths. Treat RegionFileStore as the local adapter and do not claim distributed
durability; Goal 07 must integrate the location-independent storage layer and prove recovery on a
different node after source-node loss. Bound generation, tickets, saves, and unload work; fence
asynchronous results; fail closed on corrupt, stale, or mixed-version state.
Do not accept statistical similarity, screenshots, or Ferrite replay hashes as a substitute for the
official-server same-input semantic differential suite.
Treat real generation/load/first-view performance as a Goal 04 release gate. Freeze workloads and
hardware metadata before optimizing; profile generation stages, dependency scheduling, allocation,
memory, persistence, compression, and projection; preserve raw repeated samples. Do not substitute
synthetic Region timings, pre-generated worlds, reduced distance/content, skipped durability, or an
unbounded queue for real online performance. Third-party implementations are design references,
not fidelity or benchmark oracles, and any source reuse requires explicit license provenance.

Run focused tests, durability/fault/replay gates, universal Rust gates, and applicable Goal 02 MCP
scenarios. Update the production manifest and ledger with exact evidence, then commit the batch with
a Conventional Commit subject and this exact trailer:

Ferrite-Batch: <batch-id>

Do not push, publish, deploy, rewrite history, or open a pull request without separate authorization.
Continue to the next unblocked batch. Mark Goal 04 Complete only after the formal server no longer
uses flat production fixtures or block-interaction shadow authority, same- and cross-Region
interaction survives ordinary client input, world restart and migration pass, declared vanilla
worldgen differential populations have zero unexplained semantic divergence, frozen generation,
load, first-view, exploration, tick-interference, memory, persistence, and projection thresholds
pass, exact-client exploration and dimension scenarios pass, universal gates pass, and the worktree
is clean.
```
