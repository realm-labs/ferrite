# Goal 07 Launch Prompt

Use this prompt to start or resume persistent execution of
[Goal 07](07-distributed-production-closure.md).

```text
Create and start a persistent goal with this objective:

Complete Ferrite Goal 07: connect the formal Minecraft gateway to real Lattice Region ownership,
finish production remoting, placement, routing, handoff, persistence, recovery, security,
location-independent durable storage, observability, deployment, remaining required 26.2 services,
capacity, fault, soak, and exact-client distributed acceptance. Continue batch by batch until every
terminal gate has committed evidence.
Do not set a token budget.

Before changing code, verify Goal 06 is Complete. Read AGENTS.md, the Goal 07 plan and ledger, all
Goals 03–06 completion records and production manifest, Goal 01 Lattice/deployment/protocol evidence
and supported contracts, Goal 02 MCP operations, ADR-0026, persistence recovery,
architecture/deployment/cache runbooks, and every Goal 07 decision. Inspect Git status and preserve
user work. Read the performance engineering contract and inherit the frozen Goals 04–06 workloads
and thresholds.

Select the earliest unblocked pending batch and keep exactly one batch InProgress. Require actual
multi-process production paths for distributed claims. Preserve one current Region owner,
activation-generation fencing, storage-side writer fencing, durable handoff, committed-state
projection, bounded queues, and observable overload outcomes. Treat node-local files only as cache
or migration input. Require immutable durable payloads plus linearizable Region/checkpoint heads,
and prove a newly selected worker can recover after permanent loss of the former worker and disk.
Use MinIO for immutable payloads and a dedicated etcd namespace for fenced heads in the local
multi-process and CI reference profile. Never store world payloads in etcd, and never promote a
passing MinIO run into a production-backend claim; P0-B1 must select the formal backend and that
backend must pass the same matrix.
Measure remoting, storage, handoff, recovery, and projection overhead in the batch that introduces
it. Preserve the same workload semantics and publish topology/hardware-specific scale limits; do
not reclassify synthetic Region timings as real generation, gameplay, or player capacity.
Do not treat socket probes, in-process tests, per-node persistent volumes, direct handoff streaming,
or synthetic benchmarks as production durability evidence. Keep optional C4 services default-closed
unless their enabled implementation receives explicit scope and acceptance.

Run focused distributed, recovery, hostile-input, deployment, performance, and universal Rust gates.
Run applicable Goal 02 exact-client scenarios against the deployed topology. Use isolated
workspace-owned build targets and guarded cache maintenance. Update the production manifest and
ledger with exact evidence, then commit the batch with a Conventional Commit subject and this exact
trailer:

Ferrite-Batch: <batch-id>

Do not push, publish, deploy outside task-owned test environments, rewrite history, or open a pull
request without separate authorization. Continue to the next unblocked batch. Mark Goal 07 Complete
only after every required production row, multi-process gameplay, handoff/recovery, security,
observability, capacity/fault/soak, exact-client, image/deployment, universal, and clean-worktree gate
passes with committed evidence.
```
