# Goal 03 — Production Runtime Integration

## 1. Objective

Turn the audited Goal 01 behavior and protocol baseline into one truthful production execution path.
The formal `ferrite-server` listener must run responsibility-named Region services rather than
phase-numbered conformance islands, and every claimed production feature must be traceable through
client ingress, authoritative state, durable continuity, client projection, and real-client MCP
acceptance.

Goal 01 remains complete as a locked reference, protocol, and behavior-conformance baseline. Goal
03 does not rewrite its historical reports or batch identities; it establishes a separate
production-integration denominator.

## 2. Required production evidence

A feature is production-integrated only when its manifest row identifies all of:

```text
Minecraft client input or server event
  -> bounded protocol decode
  -> semantic command or service call
  -> authoritative Region-owned mutation
  -> continuity/persistence boundary where state is durable
  -> clientbound projection or observable server effect
  -> focused tests
  -> exact 26.2 client MCP scenario when the behavior is player-visible
```

Codec presence, an isolated gameplay function, a Phase Runtime test, a packet trace, or a
conformance fixture alone is not production completion.

## 3. Scope boundary

### In scope

- a machine-checked production-integration manifest distinct from the Goal 01 implementation
  manifest;
- responsibility-based names for active server-runtime modules, public types, errors, tests, and
  operator diagnostics currently named after Goal 01 phases;
- a versioned compatibility path from persisted `ferrite:phase5/*` through `phase8/*` identities to
  responsibility-owned identities;
- backward-compatible `world-inspector` support for every migrated continuity identity;
- one composite Minecraft Region runtime with explicit, deterministic subsystem order;
- bounded command, event, continuity, and projection interfaces between simulation, player,
  entity, and world services;
- replacement of the production-only `PlayerRegionLogic` tick with the composite runtime;
- an application-layer dispatch boundary that makes ignored serverbound packets explicit;
- local production acceptance through the Goal 02 exact-client MCP;
- documentation that distinguishes historical conformance evidence from active production claims.

### Out of scope

- complete world generation, dimensions, and voxel collision, which belong to Goal 04;
- complete survival inventory, crafting, health, and progression, which belong to Goal 05;
- complete entity, combat, mob, and multiplayer tracking, which belong to Goal 06;
- real multi-process Region ownership and production Lattice remoting, which belong to Goal 07;
- renaming immutable Goal 01 report paths, batch IDs, evidence locators, or historical prose merely
  to remove the word `Phase`;
- treating `TickPhase`, lifecycle phases, protocol states, or other genuine domain state machines
  as planning-phase debt.

## 4. Naming and migration contract

The target responsibility vocabulary is:

| Current active name | Target responsibility |
|---|---|
| `phase5` / `Phase5*` | simulation, block, environment, and redstone runtime |
| `phase6` / `Phase6*` | player, item, inventory, and progression runtime |
| `phase7` / `Phase7*` | entity, combat, mob, AI, and tracking runtime |
| `phase8` / `Phase8*` | world lifecycle, generation, dimension, and durable-world runtime |
| active Phase 9 test owners | service, surface, or cross-system-join responsibility |

Persisted identities are compatibility surfaces. Migration must:

- assign responsibility-owned versioned identities;
- read valid old identities and write only the new version after migration;
- reject ambiguous, mixed, corrupt, or unsupported records;
- preserve canonical hashes or explicitly version the hash contract;
- make migration idempotent and crash-safe;
- keep offline inspection able to explain both generations;
- include clean-old, clean-new, interrupted, mixed, duplicate, and rollback-denied tests.

## 5. Composite runtime contract

The composite runtime owns a fixed dependency order and may not rely on crate registration order.
At minimum it defines boundaries for:

1. network ingress and captured command admission;
2. player/session state;
3. block, scheduled tick, environment, fluid, lighting, and redstone work;
4. item, inventory, container, and progression work;
5. entity, combat, mob, AI, tracking, and transfer work;
6. world lifecycle, chunk tickets, generation results, and dimension events;
7. cross-system reconciliation and Region-boundary effects;
8. continuity preparation and authoritative commit;
9. client projection after commit.

Subsystems may initially expose unsupported outcomes for later Goals, but they must not silently
pretend that an ignored request was applied. Every bounded queue has an owner, capacity, overload
outcome, and deterministic replay behavior.

## 6. Phased batches

### Phase 0 — Freeze production truth

| Batch | Outcome |
|---|---|
| `G03-P0-B1` | Commit this plan, prompt, ledger, responsibility vocabulary, dependency order, and production-completion definition. |
| `G03-P0-B2` | Add the machine-checked production-integration manifest with initial `Integrated`, `Partial`, `Unsupported`, and `Planned` rows derived from the formal server entry. |

### Phase 1 — Remove planning phases from active architecture

| Batch | Outcome |
|---|---|
| `G03-P1-B1` | Rename active Phase 5 and Phase 6 modules, types, errors, and tests by responsibility without compatibility-format changes. |
| `G03-P1-B2` | Rename active Phase 7, Phase 8, and Phase 9-owned nonhistorical modules, types, errors, and tests by responsibility. |
| `G03-P1-B3` | Add responsibility-owned continuity identities, fail-closed old-to-new migration, and dual-generation world inspection. |

### Phase 2 — Compose the production Region runtime

| Batch | Outcome |
|---|---|
| `G03-P2-B1` | Define the composite runtime state, deterministic subsystem order, typed commands/events, budgets, and commit/projection boundary. |
| `G03-P2-B2` | Integrate simulation/block/environment/redstone and player/item service boundaries into the composite tick. |
| `G03-P2-B3` | Integrate entity and world-lifecycle service boundaries, Region transfer joins, and continuity capture into the composite tick. |
| `G03-P2-B4` | Replace the formal gateway's `PlayerRegionLogic` execution with the composite runtime and remove the parallel production path. |

### Phase 3 — Make protocol integration explicit

| Batch | Outcome |
|---|---|
| `G03-P3-B1` | Add responsibility-owned serverbound dispatch with explicit handled, rejected, gated, and unsupported outcomes; eliminate silent application-layer drops. |
| `G03-P3-B2` | Add post-commit client projection routing and bounded per-session delivery for every currently integrated production row. |
| `G03-P3-B3` | Add deterministic local replay and fault tests spanning ingress, composite execution, continuity, and projection. |

### Phase 4 — Real-client acceptance and completion

| Batch | Outcome |
|---|---|
| `G03-P4-B1` | Extend Goal 02 scenarios to prove sustained join, movement, Region transfer, block interaction, explicit unsupported behavior, and visual convergence through the composite runtime. |
| `G03-P4-B2` | Audit active naming, manifest truth, migrations, source boundaries, clean-checkout gates, and publish the Goal 03 completion record. |

## 7. Required verification

Every Rust batch runs focused affected-crate tests plus:

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
git diff --check
```

Migration batches also run clean-old-store upgrade, interrupted upgrade, mixed-version rejection,
and `world-inspector` compatibility tests. Player-visible batches run the applicable Goal 02
exact-client MCP scenario and record structured state plus framebuffer evidence.

## 8. Terminal acceptance

- [x] A machine-checked production manifest covers every formal-entry protocol and gameplay path exactly once. See the [completion record](../reports/goal-03/g03-p4-b2-completion-record.md).
- [x] Active production modules, public types, errors, diagnostics, and nonhistorical tests contain no Goal 01 phase-number ownership names. See the [naming audit](../reports/goal-03/g03-p4-b2-completion-record.md#active-naming-and-source-boundaries).
- [x] Historical evidence remains link-stable and clearly labeled as historical conformance provenance. See the [naming audit](../reports/goal-03/g03-p4-b2-completion-record.md#active-naming-and-source-boundaries).
- [x] Old Phase 5–8 continuity records migrate crash-safely to responsibility-owned versioned identities. See the [migration evidence](../reports/goal-03/g03-p1-b3-continuity-identity-migration.md).
- [x] `world-inspector` explains valid old and new records and fails closed on invalid mixtures. See the [completion rerun](../reports/goal-03/g03-p4-b2-completion-record.md#migration-and-inspection-rerun).
- [x] The formal gateway executes one composite Region runtime with documented deterministic order. See the [runtime contract](../development/composite-region-runtime.md).
- [x] No decoded application packet is silently treated as a successful gameplay operation. See the [dispatch contract](../development/serverbound-dispatch.md).
- [x] Every row claimed `Integrated` covers every applicable production stage; incomplete continuity or gameplay remains explicitly `Partial`, `Planned`, or `Unsupported`. See the [manifest audit](../reports/goal-03/g03-p4-b2-completion-record.md#production-manifest-truth).
- [x] Exact 26.2 MCP scenarios prove the composite path rather than a conformance-only socket. See the [exact-client acceptance](../reports/goal-03/g03-p4-b1-exact-client-composite-acceptance.md).
- [x] Format, Clippy, workspace tests, migration tests, source-size review, and clean-worktree acceptance pass. See the [clean-source proof](../reports/goal-03/g03-p4-b2-completion-record.md#clean-source-terminal-gates).

Goal 03 is complete only when production integration has an executable denominator and the formal
server no longer depends on planning-phase architecture.
