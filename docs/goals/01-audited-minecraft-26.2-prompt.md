# Goal 01 Launch Prompt

Use the prompt below to start or resume persistent execution of
[Goal 01](01-audited-minecraft-26.2.md). It instructs the executor to continue one atomic batch at a
time until every terminal gate is evidenced.

```text
Create and start a persistent goal with this objective:

Implement Ferrite Goal 01 completely: a deterministic, Region-native,
multi-node Rust server for an unmodified Minecraft Java Edition 26.2 client,
covering every source-specified behavior, required C0-C3 protocol family,
audited catalog disposition, behavior surface, and cross-system join in the
locked reference. Preserve the four exact source-inconclusive observations as
explicit DeferredExperiment items unless reproducible evidence resolves them.
Continue until every terminal gate in the plan and status ledger is proven.
Do not set a token budget.

Repository execution contract
=============================

1. Before changing code, read these files completely:

   - AGENTS.md
   - docs/goals/01-audited-minecraft-26.2.md
   - docs/goals/01-audited-minecraft-26.2-status.md
   - docs/architecture.md
   - docs/reference/minecraft-java-26.2/README.md
   - docs/reference/minecraft-java-26.2/coverage.md
   - docs/reference/minecraft-java-26.2/methodology.md
   - docs/reference/minecraft-java-26.2/completion.toml
   - docs/reference/minecraft-java-26.2/behavior-surfaces.toml
   - docs/reference/minecraft-java-26.2/cross-system-joins.toml
   - docs/reference/minecraft-java-26.2/catalog/README.md
   - docs/reference/minecraft-java-26.2/catalog/catalog.toml
   - docs/reference/minecraft-java-26.2/protocol/README.md
   - docs/reference/minecraft-java-26.2/protocol/completion.toml

   Before each later batch, read every leaf rule, protocol document, root
   inventory, experiment definition, architecture section, and prior decision
   that owns or constrains that batch.

2. Inspect Git status, recent commits, the active goal state, and the full
   status ledger. Preserve user-owned changes and never use destructive Git
   operations.

3. Reproduce the current reference gates:

   cargo run -q -p mc-reference --bin mc-ref -- readiness
   cargo run -q -p mc-reference --bin mc-ref -- protocol readiness
   cargo run -q -p mc-reference --bin mc-ref -- verify --offline

   The initial baseline is 327 SourceSpecified and four SourceInconclusive
   gameplay slices, 9,078 catalog IDs, ten mapped behavior surfaces, 36 mapped
   cross-system joins, 256 packets, 44 required C0-C3 families, and 14
   GatedOptional C4 families. Trust reproducible current ledgers over stale
   counts in prose. If the denominator changes legitimately, update the plan,
   ledger, and implementation manifest in one reviewed documentation batch
   before continuing runtime work.

4. Treat the Goal plan, ledger, architecture, and locked reference as
   implementation contracts. Do not replace Region ownership with a shared
   global simulation, leak packet/Lattice/Bevy runtime types across boundaries,
   add a JSON-direct or mc-reference runtime dependency, or collapse the
   modular workspace into flat or oversized files.

   G01-P1-B1 must create ferrite-replay explicitly and install the root Cargo
   dev/debugging profiles plus guarded cache policy and maintenance tooling.
   Do not postpone them until performance or hardening work.

5. Never commit Mojang jars, assets, generated reports, decompiled source,
   copied tables, packet dumps, or proprietary data. Keep locked artifacts and
   generated inspection outputs under ignored target paths. Commit only
   project-owned schemas, algorithms, mappings, fixtures, and legal metadata.

Persistent execution loop
=========================

Repeat this loop until the Goal 01 ledger is Complete:

1. Reload the Goal plan and status ledger. If context was compacted or the goal
   resumed later, reread every normative file affected by the next batch.
2. Select the earliest unblocked Pending batch whose dependencies are Complete.
   Keep exactly one batch InProgress and announce its outcome and validation
   gate. Start with G01-P0-B1.
3. For G01-P0-B2, materialize every concrete data, gameplay, surface/join, and
   protocol partition in the machine implementation manifest. Do not begin bulk
   slice implementation while Snn/Fnn/Onn placeholders are the only ledger.
4. Start implementation from the stable slice ID or protocol family. Query each
   concrete content ID through mc-ref. Use source-specified constants, gates,
   ordering, abort paths, side effects, and fidelity class; do not infer from
   memory or a similar ID.
5. When a source-inconclusive branch is encountered, implement all specified
   surrounding behavior. Either run its exact experiment or record a
   deterministic, narrowly scoped project policy with the evidence gap,
   rejected alternatives, affected tests, and replacement condition. Keep the
   observation DeferredExperiment and never label policy as vanilla fact.
6. Implement one responsibility-bounded batch. Include code, tests, migrations,
   generated metadata, ADRs, deployment changes, and documentation belonging to
   that batch. Do not add unrelated future-client, plugin, later-version, or
   optional-service work.
7. Run the batch-specific gates and these universal gates:

   cargo fmt --all -- --check
   cargo clippy --workspace --all-targets --all-features -- -D warnings
   cargo test --workspace --all-features
   cargo run -q -p mc-reference --bin mc-ref -- verify --offline
   git diff --check

   Fix failures in the same loop. Do not waive, hide, macro-bypass, or defer a
   gate merely to create a commit.
8. Update the implementation manifest and status ledger with exact commands,
   evidence paths, coverage counters, decisions, deferrals, and blockers.
9. Review the complete diff for scope, module responsibility, imports,
   visibility, dependency direction, determinism, bounds, source-file size,
   generated drift, copied artifacts, and accidental user changes.
10. Commit exactly the completed batch. The subject must follow AGENTS.md:

    <type>(<optional-scope>): <lowercase imperative description>

    Add this exact trailer to the commit body:

    Ferrite-Batch: <batch-id>

    Example:

    feat(region-runtime): add deterministic local region routing

    Ferrite-Batch: G01-P2-B3

    After committing, verify the subject and trailer with git log. Do not push,
    publish, deploy, open a pull request, amend older commits, or rewrite history
    without separate user authorization.
11. Immediately continue with the next unblocked batch. A commit or phase is
    progress, not a reason to stop the persistent goal.

Execution rules
===============

- Keep one active implementation batch. Parallel read-only inspection is
  allowed only when it cannot create overlapping writes or inconsistent
  decisions.
- Use the machine implementation manifest as the progress denominator after
  Phase 0. Code presence, mocks, TODOs, client connection, catalog names, or a
  green reference-readiness command do not count as implementation completion.
- Preserve the architecture's Region-local ECS, Region-owned voxel state,
  explicit boundary batches, activation-generation fencing, project-owned
  snapshots/journals, and topology-independent semantic messages.
- Pin Lattice to an exact reviewed Git revision behind ferrite-region-runtime.
  Ferrite owns tick semantics, state transfer, recovery points, entity transfer,
  and boundary consistency.
- Use the same server binary and Region model for local and multi-node profiles.
  Complete one-command three-node startup, readiness, graceful drain, Compose,
  and Kubernetes contracts in their planned batches.
- Keep packet structs, wire IDs, connection codecs, and registry raw IDs inside
  the Minecraft 26.2 adapter. Simulation and persistence consume only
  project-owned semantic types.
- Keep replay encoding, canonical hashing, verification and divergence
  diagnostics in ferrite-replay. Simulation and gameplay expose stable semantic
  inputs but do not depend on replay file formats or verification machinery.
- Use ordinary dev builds for iteration and the explicit debugging profile for
  full Ferrite symbols. Keep CI/fuzz/coverage/benchmark target roots isolated.
  Run only the guarded workspace cache tool for periodic maintenance; never run
  unscoped cargo clean or delete global Cargo or mc-reference caches.
- Treat packet round trips as insufficient. Required family completion includes
  golden bytes, bounds, malformed cases, state transitions, semantic mapping,
  ordering/acknowledgement, and end-to-end traces.
- Keep all queues, decoders, journals, mailboxes, replication fan-out, and
  asynchronous work bounded. Stale generations and revisions must fail closed.
- Keep every authoritative order and random stream explicit and replayable.
  Compare local and distributed canonical hashes at phase gates.
- Keep visibility minimal, avoid unnecessary pub use, import clear paths before
  use, never use super::super, and split by responsibility before 1,200 LOC.
- Do not suppress Clippy broadly or use macros/conditional compilation to hide
  findings. A narrow exception needs an adjacent concrete reason.
- Optional C4 families require their audited configuration and
  disabled/refusal/degradation behavior. Do not implement all optional services
  merely because their wire contracts are documented.
- If a genuine external blocker affects one batch, record the exact blocker,
  attempted alternatives, and unblock condition, then continue any independent
  unblocked batch. Mark the persistent goal Blocked only after the goal system's
  repeated-blocker threshold is genuinely met.
- Give concise progress updates during long work. Do not use an ordinary final
  response to terminate execution while required unblocked work remains.

Completion protocol
===================

Do not mark the goal Complete because the workspace compiles, one client joins,
a phase ends, or most rules are implemented.

When all batches appear complete:

1. Run every acceptance suite in Goal 01 section 8 from a clean checkout.
2. Verify exact final denominators: 327 required source-specified slices, four
   source-known inconclusive surfaces, 9,078 catalog IDs, 44 required protocol
   families, 14 optional gates, ten behavior surfaces, and 36 cross-system
   joins, adjusted only by a previously committed baseline revision.
3. Verify local, in-process Lattice, and multi-process Lattice state hashes,
   Region handoff/failure evidence, unmodified-client traces, and cross-platform
   deterministic vectors.
4. Verify named benchmark and capacity reports without claiming unmeasured
   hardware, topology, workload, or platform behavior.
5. Update every terminal checklist item and completion record with committed
   evidence.
6. Commit G01-P10-B6 with a Conventional Commit subject and exact batch trailer.
7. Mark the persistent goal Complete only after the commit succeeds and the
   worktree is clean.
8. Report the completion commit, implementation-manifest digest, final coverage
   totals, validation commands, multi-node fault evidence, and benchmark
   profiles. If the goal system reports final token usage, include it.

Start now with the Next unblocked batch recorded in the status ledger. Do not
respond with another plan; execute the loop.
```
