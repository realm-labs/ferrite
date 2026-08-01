# Goal 02 Launch Prompt

Use this prompt to start or resume persistent execution of
[Goal 02](02-client-mcp-automation.md).

```text
Create and start a persistent goal with this objective:

Complete Ferrite Goal 02: build the pure-Java Minecraft Java 26.2 Fabric client MCP, real-input and
visual observation surface, isolated Quick Play launcher, deterministic scenarios, hardening, and
CI acceptance defined by the Goal plan and status ledger. Continue batch by batch until every
terminal gate has committed evidence. Do not set a token budget.

Before changing code, read AGENTS.md, the Goal 02 plan, Goal 02 status ledger, docs/architecture.md,
the Goal 01 supported-contract and formal-network-entry documents, and every prior Goal 02 decision.
Inspect Git status and preserve user work.

Select the earliest unblocked pending batch. Keep exactly one batch InProgress. Implement one
responsibility-bounded batch with focused tests and documentation. The client mod must use Java 25,
Fabric, normal Minecraft client input/interaction/render paths, bounded queues and requests,
loopback-only authenticated MCP, isolated ignored client state, and exact 26.2 artifacts. It must
never read HMCL credentials, execute server administration commands, mutate world/player state
directly, hand-build Minecraft packets, or label an instrumented client unmodified.

Run the batch gates and the universal Java/Rust gates recorded in the plan. Do not waive failures.
Update the status ledger with exact evidence, review source size, dependency and license boundaries,
then commit the completed batch using a Conventional Commit subject and this exact trailer:

Ferrite-Batch: <batch-id>

Do not push, publish, deploy, rewrite history, or open a pull request without separate authorization.
Immediately continue with the next unblocked batch; a successful build, listed tools, screenshot, or
single connection does not complete the Goal.

Mark Goal 02 Complete only after every terminal checkbox links to committed evidence, clean-checkout
Java and Rust gates pass, exact-client reference and Ferrite scenarios are recorded, secrets and
artifacts are absent, the completion batch is committed, and the worktree is clean.
```
