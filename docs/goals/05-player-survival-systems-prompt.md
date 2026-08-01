# Goal 05 Launch Prompt

Use this prompt to start or resume persistent execution of
[Goal 05](05-player-survival-systems.md).

```text
Create and start a persistent goal with this objective:

Complete Ferrite Goal 05: implement the durable authoritative survival-player loop across identity,
admission, inventory, equipment, items, block interaction, containers, crafting, health, hunger,
experience, progression, death, respawn, chat, commands, operations, and exact-client acceptance.
Continue batch by batch until every terminal gate has committed evidence. Do not set a token budget.

Before changing code, verify Goal 04 is Complete. Read AGENTS.md, the Goal 05 plan and ledger, Goal
03 production manifest, Goal 04 world formats and completion record, Goal 01 player/item/protocol
evidence, Goal 02 MCP operations, and every Goal 05 decision. Inspect Git status and preserve user
work.

Select the earliest unblocked pending batch and keep exactly one batch InProgress. Treat client
packets and revisions as requests, never authority. Preserve one owner for player state and explicit
atomic boundaries for inventory, block, loot, durability, death, and transfer changes. Reject and
correct stale or impossible actions. Use typed permission-checked command effects. Never use server
commands, MCP direct mutation, or hand-built packets as gameplay acceptance.

Run focused transaction, continuity, replay, fault, security, and universal Rust gates. Run
applicable Goal 02 exact-client input, state, GUI, and screenshot scenarios. Update the production
manifest and ledger with exact evidence, then commit the batch with a Conventional Commit subject
and this exact trailer:

Ferrite-Batch: <batch-id>

Do not push, publish, deploy, rewrite history, or open a pull request without separate authorization.
Continue to the next unblocked batch. Mark Goal 05 Complete only after the durable survival loop,
protocol semantics, restart continuity, exact-client scenarios, universal gates, and clean-worktree
acceptance all pass.
```
