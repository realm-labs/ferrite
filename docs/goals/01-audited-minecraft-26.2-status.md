# Goal 01 Status — Audited Minecraft Java 26.2 Server Baseline

This ledger is the resumable implementation source of truth for
[Goal 01](01-audited-minecraft-26.2.md). Update it in every implementation batch. Do not mark an
item complete from code presence alone; include commands and committed evidence.

## Goal state

| Field | Value |
|---|---|
| State | `InProgress` |
| Active batch | None |
| Next unblocked batch | `G01-P2-B4` |
| Goal plan | [Goal 01 plan](01-audited-minecraft-26.2.md) |
| Launch prompt | [Goal 01 prompt](01-audited-minecraft-26.2-prompt.md) |
| Baseline verified | 2026-07-29 |
| Frozen baseline | [reference-baseline.toml](../../goals/minecraft-java-26.2/reference-baseline.toml) |
| Baseline SHA-256 | `31f5e58c029337aaf4c7bc8bba253a5ce8ecd6edbee30cd41989e94a9345c678` |
| Implementation manifest | [implementation.toml](../../goals/minecraft-java-26.2/implementation.toml) |
| Manifest SHA-256 | `20ae4b4018e93268a020e77fb1e466065fa3e71b85fd642a88a25246c96c2c89` |
| Completion commit | — |
| Blocker | None |

Allowed goal states are `Ready`, `InProgress`, `Blocked`, and `Complete`. Only one batch may be
`InProgress`.

## Frozen reference denominator

| Denominator | Total | Verified implementation | Deferred | Pending |
|---|---:|---:|---:|---:|
| `SourceSpecified` gameplay slices | 327 | 0 | 0 | 327 |
| Source-known surface of inconclusive slices | 4 | 0 | 4 observations | 4 implementations |
| Catalog IDs | 9,078 | 9,078 | 0 | 0 |
| Required C0-C3 protocol families | 44 | 0 | 0 | 44 |
| C4 configuration gates | 14 | 0 | 0 | 14 |
| Behavior-surface roots | 10 | 0 | 0 | 10 |
| Cross-system joins | 36 | 0 | 0 | 36 |

Reference baseline:

- server SHA-1: `823e2250d24b3ddac457a60c92a6a941943fcd6a`;
- client SHA-1: `2dc72797acbc1b63fc16a11c4ac393605f453754`;
- protocol inventory digest: `f34b0956b6399c749d4638cd6d3c9226685f41fa`;
- source locators: 2,789 across 952 classes;
- planned experiments: 307.

## Gameplay subsystem baseline

| Subsystem | Total slices | `SourceSpecified` | `SourceInconclusive` | Verified |
|---|---:|---:|---:|---:|
| `simulation` | 4 | 3 | 1 | 0 |
| `blocks` | 125 | 125 | 0 | 0 |
| `environment` | 5 | 4 | 1 | 0 |
| `redstone` | 6 | 6 | 0 | 0 |
| `player` | 8 | 7 | 1 | 0 |
| `items` | 95 | 95 | 0 | 0 |
| `entities` | 45 | 45 | 0 | 0 |
| `mobs` | 11 | 11 | 0 | 0 |
| `world` | 28 | 27 | 1 | 0 |
| `client` | 4 | 4 | 0 | 0 |
| **Total** | **331** | **327** | **4** | **0** |

## Deferred experiment register

| Slice | Experiment | Implementation state | Observation state | Policy |
|---|---|---|---|---|
| `SIM-SCHEDULED-TICKS-001` | `EXP-SIM-002` | `Pending` | `DeferredExperiment` | No guessed vanilla tie-break |
| `ENV-LIGHTING-001` | `EXP-ENV-004` | `Pending` | `DeferredExperiment` | No universal latency claim |
| `PLY-BLOCK-BREAK-001` | `EXP-PLY-003` | `Pending` | `DeferredExperiment` | Preserve specified packet order |
| `WGEN-PIPELINE-EQUIVALENCE-001` | `EXP-WGEN-001`, `EXP-WGEN-005`, `EXP-WGEN-006` | `Pending` | `DeferredExperiment` | No same-seed identity claim |

## Phase ledger

| Phase | State | Exit evidence | Notes |
|---|---|---|---|
| Phase 0 — Freeze implementation truth | `Complete` | [Baseline](../../goals/minecraft-java-26.2/reference-baseline.toml), [manifest](../../goals/minecraft-java-26.2/implementation.toml), and [ADRs](../adr/README.md) | All audited records map exactly once into the verified ordered batch DAG |
| Phase 1 — Workspace, identity, data, and deterministic primitives | `Complete` | [build/cache](../development/builds-and-cache.md), [content import](../development/content-import.md), [determinism](../development/determinism-and-replay.md), and [test harness](../development/deterministic-testing.md) | Profiles, guarded caches, locked content, canonical primitives, deterministic replay, and repository gates pass |
| Phase 2 — Region-native local and distributed runtime | `InProgress` | [Region-owned state](../development/region-state.md), [tick pipeline](../development/region-tick-pipeline.md), and [local runtime](../development/local-region-runtime.md) | Region state, tick/boundary semantics, transfer, and the local runner are complete; recovery and distributed topology batches remain |
| Phase 3 — Protocol C0 and C1 | `Pending` | — | Depends on Phase 2 semantic boundary |
| Phase 4 — C2 minimal playable multi-Region world | `Pending` | — | Depends on Phase 3 |
| Phase 5 — Simulation, blocks, environment, and redstone | `Pending` | — | Generated slice batches |
| Phase 6 — Players, items, inventories, and progression | `Pending` | — | Generated slice/family batches |
| Phase 7 — Entities, combat, mobs, AI, and spawning | `Pending` | — | Generated slice/family batches |
| Phase 8 — World generation, dimensions, portals, and durable worlds | `Pending` | — | Generated slice batches |
| Phase 9 — Remaining C3 services, client behavior, and C4 gates | `Pending` | — | Generated slice/family batches |
| Phase 10 — Scale, hardening, and completion | `Pending` | — | Depends on all required coverage |

## Fixed batch ledger

`G01-P0-B2` must add every concrete generated partition to the machine implementation manifest and
record its counts below. Placeholder families such as `Snn`, `Fnn`, and `Onn` are not completion
evidence.

| Batch | State | Depends on | Evidence | Result |
|---|---|---|---|---|
| `G01-P0-B1` | `Complete` | — | `7d81b62`; [frozen baseline](../../goals/minecraft-java-26.2/reference-baseline.toml) | All reference readiness, offline verification, format, Clippy, and workspace tests passed |
| `G01-P0-B2` | `Complete` | P0-B1 | `217f724`; [schema](../../goals/minecraft-java-26.2/README.md); [manifest](../../goals/minecraft-java-26.2/implementation.toml) | 145 concrete batches and 46 surface/join owners materialized; renderer idempotency and full gates passed |
| `G01-P0-B3` | `Complete` | P0-B2 | `32ca2f0`; `implementation-manifest verify` | Missing, duplicate, dead, stale, false-completion, path, and DAG checks pass; counters render and offline verification includes them |
| `G01-P0-B4` | `Complete` | P0-B1 | `3ad6ff3`; [ADR index](../adr/README.md); [Lattice lock](../adr/lattice.lock.toml) | Eleven implementation-boundary ADRs accepted; Lattice pinned to `a52c54004c782bd18b70d37d929d54cd7d8205f3`; full gates passed |
| `G01-P1-B1` | `Complete` | Phase 0 | `050cff7`; [build/cache runbook](../development/builds-and-cache.md) | 18-package modular workspace and 51 allowed edges verified; profiles, isolated targets, dry-run/apply pruning, activity/protection/containment tests, daily hook, and full repository entrypoint passed |
| `G01-P1-B2` | `Complete` | P1-B1 | `6ab4dc7`; `ferrite-foundation` | Checked coordinates and bounds, validated resource/stable identities, directions, activation generations, and versioned Euclidean Region mapping; 21 crate tests and full gates passed |
| `G01-P1-B3` | `Complete` | P1-B2 | `e43817d`; `ferrite-registry` | Contribution-order assembly, persistent/runtime ID separation, validated block-state schemas, BLAKE3 content manifests, and provenance implemented; 11 crate tests and full gates passed |
| `G01-P1-B4` | `Complete` | P1-B3 | `fbb7b1b`; [content import report](../reports/goal-01/g01-p1-b4-content-import.md) | Locked local artifacts produced an ignored schema-validated bundle; all 32 catalog partitions and 9,078 IDs verified, bundle/manifest drift locked, and full gates passed |
| `G01-P1-B5` | `Complete` | P1-B2 | `4cb01a4`; [determinism contract](../development/determinism-and-replay.md) | Named independent RNG streams, snapshot continuation, canonical bounded codec, Region/world hash vectors, semantic envelopes, replay log, verifier, and first-divergence diagnostics implemented; 17 focused tests and full gates passed |
| `G01-P1-B6` | `Complete` | P1-B1 | `105ec22`; [deterministic testing](../development/deterministic-testing.md) | Fake time, named seeds, bounded snapshots/malformed corpora, scenario DSL and runner, CI/repository policy gates, and the source-policy exception removal pass full gates |
| `G01-P2-B1` | `Complete` | Phase 1 | `29daa19`; [Region-owned state](../development/region-state.md) | Typed single/local/direct palettes, sparse checked sections, owned chunk admission, one private Bevy ECS World per Region, stable entity mapping, and immutable views pass full gates |
| `G01-P2-B2` | `Complete` | P2-B1 | `20f77d6`; [tick pipeline](../development/region-tick-pipeline.md) | Fixed 20-phase ticks, fail-closed command/boundary admission, generation fencing, immutable journals, explicit barriers, and locked semantic Region/world hash vectors pass full gates |
| `G01-P2-B3` | `Complete` | P2-B2 | This row's containing commit; [local runtime](../development/local-region-runtime.md) | Stable-order consistency-island execution, same-phase boundary effects, dual-generation entity/player transfer, bounded outputs, preflighted commits, and poisoned-tick refusal pass full gates |
| `G01-P2-B4` | `Pending` | P2-B2 | — | Add snapshots and recovery |
| `G01-P2-B5` | `Pending` | P2-B3, P2-B4 | — | Add Lattice adapter |
| `G01-P2-B6` | `Pending` | P2-B5 | — | Add multi-node deployment contract |
| `G01-P2-B7` | `Pending` | P2-B6 | — | Prove topology and fault behavior |
| `G01-P3-B1` | `Pending` | Phase 2 | — | Add bounded protocol primitives |
| `G01-P3-B2` | `Pending` | P3-B1 | — | Add locked packet catalog |
| `G01-P3-B3` | `Pending` | C0/C1 family batches | — | Complete login/configuration |
| `G01-P3-B4` | `Pending` | P3-B3 | — | Connect semantic Region routing |
| `G01-P3-B5` | `Pending` | P3-B4 | — | Prove C0/C1 conformance |
| `G01-P4-B1` | `Pending` | Phase 3 | — | Add chunk/join projection |
| `G01-P4-B2` | `Pending` | P4-B1 | — | Add movement and transfer |
| `G01-P4-B3` | `Pending` | P4-B2 | — | Add block interaction and correction |
| `G01-P4-B4` | `Pending` | C2 family batches | — | Prove topology trace equivalence |
| `G01-P4-B5` | `Pending` | P4-B4 | — | Prove unmodified-client C2 path |
| `G01-P5-B1` | `Pending` | Phase 5 slice batches | — | Integrate continuity and projection |
| `G01-P5-B2` | `Pending` | P5-B1 | — | Close Phase 5 coverage |
| `G01-P6-B1` | `Pending` | Phase 6 slice/family batches | — | Integrate ownership and resync |
| `G01-P6-B2` | `Pending` | P6-B1 | — | Close Phase 6 coverage |
| `G01-P7-B1` | `Pending` | Phase 7 slice/family batches | — | Integrate entity lifecycle |
| `G01-P7-B2` | `Pending` | P7-B1 | — | Close Phase 7 coverage |
| `G01-P8-B1` | `Pending` | Phase 8 slice batches | — | Integrate durable worlds |
| `G01-P8-B2` | `Pending` | P8-B1 | — | Validate world behavior families |
| `G01-P8-B3` | `Pending` | P8-B2 | — | Record equivalence deferral |
| `G01-P9-B1` | `Pending` | Phase 9 generated batches | — | Close protocol and surface coverage |
| `G01-P10-B1` | `Pending` | Phases 1-9 | — | Run architecture/content audits |
| `G01-P10-B2` | `Pending` | P10-B1 | — | Run long fuzz/property suites |
| `G01-P10-B3` | `Pending` | P10-B1 | — | Run distributed fault injection |
| `G01-P10-B4` | `Pending` | P10-B3 | — | Record benchmark profiles |
| `G01-P10-B5` | `Pending` | P10-B2, P10-B4 | — | Run full acceptance |
| `G01-P10-B6` | `Pending` | P10-B5 | — | Commit completion record |

## Generated batch counters

Populate this table in `G01-P0-B2`.

| Family | Concrete batches | Records | Verified | Pending |
|---|---:|---:|---:|---:|
| Data/catalog partitions | 32 | 9,078 IDs | 9,078 | 0 |
| Gameplay slice partitions | 55 | 331 slices | 0 | 331 |
| Behavior-surface/join partitions | 5 owner batches | 46 owners | 0 | 46 |
| Required protocol partitions | 44 | 44 families | 0 | 44 |
| Optional protocol gate partitions | 14 | 14 families | 0 | 14 |

## Decisions and blockers

| Date | ID | State | Decision or blocker | Evidence / follow-up |
|---|---|---|---|---|
| 2026-07-29 | `G01-D001` | `Accepted` | Use one implementation Goal with resumable internal phases; do not create separate goals for each subsystem. | User direction and Goal 01 plan |
| 2026-07-29 | `G01-D002` | `Accepted` | Implement source-specified portions now; retain four exact observations as `DeferredExperiment`. | Locked readiness output |
| 2026-07-29 | `G01-D003` | `Accepted` | Put batch IDs in a commit trailer so Conventional Commit descriptions remain lowercase and imperative. | `AGENTS.md` commit policy |
| 2026-07-29 | `G01-D004` | `Accepted` | Create `ferrite-replay` as an explicit crate and establish lightweight `dev`, full-symbol `debugging`, and guarded periodic cache maintenance in the first workspace batch. | User direction and architecture sections 5.1–5.2 |
| 2026-07-29 | `G01-D005` | `Accepted` | Pin Lattice revision `a52c54004c782bd18b70d37d929d54cd7d8205f3`; keep it behind `ferrite-region-runtime` and retain Ferrite-owned tick, state-transfer, recovery, and business-delivery semantics. | [ADR-0019](../adr/0019-pinned-lattice-substrate.md) and [revision lock](../adr/lattice.lock.toml) |
| 2026-07-29 | `G01-D006` | `Accepted` | Use versioned Euclidean 8×8-chunk Region ownership by default and a persisted spatial placement mapper; changes require offline migration. | [ADR-0020](../adr/0020-simulation-region-mapping.md) |
| 2026-07-29 | `G01-D007` | `Accepted` | Persist resource identities, ordered manifest entries, content digests, and provenance; keep dense registry and block-state indices process-local and reconstruct them deterministically. | `ferrite-registry` compile-time serialization boundary and manifest tests |
| 2026-07-29 | `G01-D008` | `Accepted` | Generate the runtime content bundle only below ignored `target/ferrite-content`; commit aggregate locks and evidence, never official entries or payloads. | [import runbook](../development/content-import.md), [bundle lock](../reference/minecraft-java-26.2/content-bundle.lock.toml), and [evidence](../reports/goal-01/g01-p1-b4-content-import.md) |
| 2026-07-29 | `G01-D009` | `Accepted` | Version named gameplay streams as `Xoshiro256StarStarV1`; derive each stream from the world seed and stable resource name so creation order is irrelevant, and persist materialized states. | [determinism contract](../development/determinism-and-replay.md) and locked RNG vectors |
| 2026-07-29 | `G01-D010` | `Accepted` | Keep authored behavior scenarios target-neutral; the initial recording target validates the harness but does not count as Minecraft rule evidence. | [deterministic testing contract](../development/deterministic-testing.md) |
| 2026-07-29 | `G01-D011` | `Accepted` | Expose typed process-local block/biome IDs and stable entity IDs at the simulation boundary; keep registry internals and Bevy `Entity` handles inside their owning modules. | [Region-owned state contract](../development/region-state.md) |
| 2026-07-29 | `G01-D012` | `Accepted` | Fix a 20-phase logical tick order; sort commands and boundary batches by stable semantic keys; retain duplicate fences until commit; backpressure instead of dropping accepted work. | [Region tick pipeline contract](../development/region-tick-pipeline.md) |
| 2026-07-29 | `G01-D013` | `Accepted` | Run local consistency islands in Region-key phase lockstep; merge immediate effects after all normal phase work; apply complete semantic entity/player transfers at reconciliation with both endpoint generations fenced. | [Local Region runtime contract](../development/local-region-runtime.md) |

## Terminal acceptance checklist

Change an item to `[x]` only with linked committed evidence.

- [x] Required modular workspace and dependency direction are verified.
- [x] Dedicated replay ownership, Cargo debug profiles, and guarded cache maintenance are verified.
- [ ] Region ownership exists for all mutable authoritative state.
- [ ] Local, in-process Lattice, and multi-process Lattice execution converge.
- [ ] One-command three-node development startup and graceful shutdown pass.
- [ ] Unmodified 26.2 client completes the supported C0-C3 baseline.
- [ ] 327/327 `SourceSpecified` gameplay slices are verified.
- [ ] Source-known behavior for all four inconclusive slices is verified.
- [ ] All four unresolved observations remain honestly recorded or are replaced by experiment evidence.
- [ ] 9,078/9,078 catalog IDs have validated runtime disposition and owners.
- [ ] 44/44 required protocol families pass implementation conformance.
- [ ] 14/14 optional protocol families pass their configuration-gate contract.
- [ ] 10/10 behavior surfaces and 36/36 cross-system joins have implementation evidence.
- [ ] Persistence, crash recovery, handoff, generation fencing, and replay pass fault tests.
- [ ] Cross-platform deterministic vectors and topology-equivalence hashes pass.
- [ ] Source-size, visibility, dependency, lint, format, test, fuzz, and public-API audits pass.
- [ ] Named benchmark profiles support every published capacity claim.
- [ ] Clean-checkout acceptance report is committed.
- [ ] No excluded client, plugin, later-version, or unmeasured-scale scope is claimed.

## Completion record

| Field | Value |
|---|---|
| Final state | `InProgress` |
| Completion commit | — |
| Implementation manifest digest | — |
| Coverage report | — |
| Clean-checkout report | — |
| Multi-node fault report | — |
| Performance report | — |
| Remaining required work | All Goal 01 implementation batches |
