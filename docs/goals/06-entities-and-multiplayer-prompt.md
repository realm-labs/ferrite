# Goal 06 Launch Prompt

Use this prompt to start or resume persistent execution of
[Goal 06](06-entities-and-multiplayer.md).

```text
Create and start a persistent goal with this objective:

Complete Ferrite Goal 06: implement authoritative multiplayer replication, entity lifecycle and
tracking, combat, projectiles, vehicles, mobs, spawning, navigation, AI, Region transfer,
persistence, and multi-client exact-client acceptance. Continue batch by batch until every terminal
gate has committed evidence. Do not set a token budget.

Before changing code, verify Goal 05 is Complete. Read AGENTS.md, the Goal 06 plan and ledger, Goal
03 production manifest, Goal 04 world ownership, Goal 05 player/inventory/combat joins, Goal 01
entity/mob evidence, Goal 02 MCP operations, and every Goal 06 decision. Inspect Git status and
preserve user work.

Select the earliest unblocked pending batch and keep exactly one batch InProgress. Separate stable
entity identity from session network IDs. Source tracking and projection from committed state.
Preserve spawn-before-update and remove ordering, one live Region owner, generation fencing,
idempotent replay, bounded AI/tracking/projection queues, and explicit overload outcomes. Use at
least two isolated exact clients for shared-world claims. Never use direct server mutation as
gameplay acceptance.

Run focused lifecycle, tracking, transfer, replay, fault, performance, and universal Rust gates.
Run applicable Goal 02 multi-client state and screenshot scenarios. Update the production manifest
and ledger with exact evidence, then commit the batch with a Conventional Commit subject and this
exact trailer:

Ferrite-Batch: <batch-id>

Do not push, publish, deploy, rewrite history, or open a pull request without separate authorization.
Continue to the next unblocked batch. Mark Goal 06 Complete only after shared-client convergence,
entity continuity, combat/vehicle/mob behavior, fault and overload evidence, universal gates, and
clean-worktree acceptance all pass.
```
