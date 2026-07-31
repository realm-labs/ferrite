# Goal 01 Status — Audited Minecraft Java 26.2 Server Baseline

This ledger is the resumable implementation source of truth for
[Goal 01](01-audited-minecraft-26.2.md). Update it in every implementation batch. Do not mark an
item complete from code presence alone; include commands and committed evidence.

## Goal state

| Field | Value |
|---|---|
| State | `InProgress` |
| Active batch | `G01-P9-F006` |
| Next unblocked batch | `G01-P9-F007` |
| Goal plan | [Goal 01 plan](01-audited-minecraft-26.2.md) |
| Launch prompt | [Goal 01 prompt](01-audited-minecraft-26.2-prompt.md) |
| Baseline verified | 2026-07-29 |
| Frozen baseline | [reference-baseline.toml](../../goals/minecraft-java-26.2/reference-baseline.toml) |
| Baseline SHA-256 | `31f5e58c029337aaf4c7bc8bba253a5ce8ecd6edbee30cd41989e94a9345c678` |
| Implementation manifest | [implementation.toml](../../goals/minecraft-java-26.2/implementation.toml) |
| Manifest SHA-256 | `706b22c72e6f730a5c7f2ff569aeb692d018e61853163cfea8a27266874ad5c0` |
| Completion commit | — |
| Blocker | None |

Allowed goal states are `Ready`, `InProgress`, `Blocked`, and `Complete`. Only one batch may be
`InProgress`.

## Frozen reference denominator

| Denominator | Total | Verified implementation | Deferred | Pending |
|---|---:|---:|---:|---:|
| `SourceSpecified` gameplay slices | 327 | 327 | 0 | 0 |
| Source-known surface of inconclusive slices | 4 | 4 | 4 observations | 0 |
| Catalog IDs | 9,078 | 9,078 | 0 | 0 |
| Required C0-C3 protocol families | 44 | 36 | 0 | 8 |
| C4 configuration gates | 14 | 0 | 0 | 14 |
| Behavior-surface roots | 10 | 5 | 0 | 5 |
| Cross-system joins | 36 | 15 | 0 | 21 |

Reference baseline:

- server SHA-1: `823e2250d24b3ddac457a60c92a6a941943fcd6a`;
- client SHA-1: `2dc72797acbc1b63fc16a11c4ac393605f453754`;
- protocol inventory digest: `f34b0956b6399c749d4638cd6d3c9226685f41fa`;
- source locators: 2,789 across 952 classes;
- planned experiments: 307.

## Gameplay subsystem baseline

| Subsystem | Total slices | `SourceSpecified` | `SourceInconclusive` | Verified |
|---|---:|---:|---:|---:|
| `simulation` | 4 | 3 | 1 | 4 |
| `blocks` | 125 | 125 | 0 | 125 |
| `environment` | 5 | 4 | 1 | 5 |
| `redstone` | 6 | 6 | 0 | 6 |
| `player` | 8 | 7 | 1 | 8 |
| `items` | 95 | 95 | 0 | 95 |
| `entities` | 45 | 45 | 0 | 45 |
| `mobs` | 11 | 11 | 0 | 11 |
| `world` | 28 | 27 | 1 | 28 |
| `client` | 4 | 4 | 0 | 4 |
| **Total** | **331** | **327** | **4** | **331** |

## Deferred experiment register

| Slice | Experiment | Implementation state | Observation state | Policy |
|---|---|---|---|---|
| `SIM-SCHEDULED-TICKS-001` | `EXP-SIM-002` | `Verified` | `DeferredExperiment` | No guessed vanilla tie-break |
| `ENV-LIGHTING-001` | `EXP-ENV-004` | `Verified` | `DeferredExperiment` | No universal latency claim |
| `PLY-BLOCK-BREAK-001` | `EXP-PLY-003` | `Verified` | `DeferredExperiment` | Preserve specified packet order |
| `WGEN-PIPELINE-EQUIVALENCE-001` | `EXP-WGEN-001`, `EXP-WGEN-005`, `EXP-WGEN-006` | `Verified` | `DeferredExperiment` | No same-seed identity claim |

## Phase ledger

| Phase | State | Exit evidence | Notes |
|---|---|---|---|
| Phase 0 — Freeze implementation truth | `Complete` | [Baseline](../../goals/minecraft-java-26.2/reference-baseline.toml), [manifest](../../goals/minecraft-java-26.2/implementation.toml), and [ADRs](../adr/README.md) | All audited records map exactly once into the verified ordered batch DAG |
| Phase 1 — Workspace, identity, data, and deterministic primitives | `Complete` | [build/cache](../development/builds-and-cache.md), [content import](../development/content-import.md), [determinism](../development/determinism-and-replay.md), and [test harness](../development/deterministic-testing.md) | Profiles, guarded caches, locked content, canonical primitives, deterministic replay, and repository gates pass |
| Phase 2 — Region-native local and distributed runtime | `Complete` | [Region-owned state](../development/region-state.md), [tick pipeline](../development/region-tick-pipeline.md), [local runtime](../development/local-region-runtime.md), [recovery](../development/persistence-recovery.md), [Lattice adapter](../development/lattice-adapter.md), [multi-node deployment](../development/multi-node-deployment.md), and [topology conformance](../development/topology-conformance.md) | Twelve Regions converge for 10,000 ticks across local, in-process, and three-process topologies; fencing, faults, durable node recovery, and overload outcomes pass |
| Phase 3 — Protocol C0 and C1 | `Complete` | [wire foundation](../development/protocol-wire.md), [packet catalog](../development/protocol-catalog.md), [handshake](../development/protocol-handshake-serverbound.md), [clientbound](../development/protocol-status-clientbound.md)/[serverbound](../development/protocol-status-serverbound.md) status, required [clientbound](../development/protocol-login-clientbound.md)/[serverbound](../development/protocol-login-serverbound.md) login, [clientbound](../development/protocol-play-clientbound-entry.md)/[serverbound](../development/protocol-play-serverbound-entry.md) Play entry, both [clientbound](../development/protocol-configuration-clientbound.md)/[serverbound](../development/protocol-configuration-serverbound.md) configuration directions, [semantic session routing](../development/semantic-session-routing.md), and [C0/C1 conformance](../reports/goal-01/g01-p3-b5-protocol-conformance.md) | Headless malformed/ordering suites, real loopback status/login, full 697-tag projection, and an exact unmodified 26.2 client reaching Play all pass |
| Phase 4 — C2 minimal playable multi-Region world | `Complete` | [Chunk join projection](../development/chunk-join-projection.md), [player movement and transfer](../development/player-movement-and-region-transfer.md), [block interaction and convergence](../development/block-interaction-and-convergence.md), [playable topology conformance](../development/playable-topology-conformance.md), [C2 acceptance and adversity](../development/c2-acceptance-and-adversity.md), [clientbound block protocol](../development/protocol-play-clientbound-block.md), [clientbound session protocol](../development/protocol-play-clientbound-session.md), [clientbound terrain protocol](../development/protocol-play-clientbound-terrain.md), [serverbound block protocol](../development/protocol-play-serverbound-block.md), [serverbound movement protocol](../development/protocol-play-serverbound-movement.md), [P4-B1 report](../reports/goal-01/g01-p4-b1-chunk-join-projection.md), [P4-B2 report](../reports/goal-01/g01-p4-b2-player-movement-and-transfer.md), [P4-B3 report](../reports/goal-01/g01-p4-b3-block-interaction-and-convergence.md), [P4-B4 report](../reports/goal-01/g01-p4-b4-playable-topology-conformance.md), [P4-B5 report](../reports/goal-01/g01-p4-b5-c2-acceptance-and-adversity.md), [P4-F001 report](../reports/goal-01/g01-p4-f001-play-clientbound-block.md), [P4-F002 report](../reports/goal-01/g01-p4-f002-play-clientbound-session.md), [P4-F003 report](../reports/goal-01/g01-p4-f003-play-clientbound-terrain.md), [P4-F004 report](../reports/goal-01/g01-p4-f004-play-serverbound-block.md), and [P4-F005 report](../reports/goal-01/g01-p4-f005-play-serverbound-movement.md) | Exact unmodified client completes terrain, feedback, loaded, movement, and tick-end; delayed/fragmented TCP, malformed bodies, bounded backpressure, cross-Region convergence, canonical state equality, and exact packet traces pass |
| Phase 5 — Simulation, blocks, environment, and redstone | `Complete` | [BLK-001 runtime](../development/block-runtime-blk-001.md), [placement and breaking](../development/block-placement-and-breaking.md), [BLK-003 update/runtime](../development/block-update-and-runtime-blk-003.md), [falling blocks](../development/falling-block-runtime.md), [test-instance runtime](../development/test-instance-runtime.md), [SIM-003 block runtime](../development/sim-003-block-runtime.md), [SIM-004 block runtime](../development/sim-004-block-runtime.md), [SIM-005 block runtime](../development/sim-005-block-runtime.md), [ENV-001 runtime](../development/environment-runtime-env-001.md), [lighting runtime](../development/environment-lighting-runtime.md), [weather runtime](../development/environment-weather-runtime.md), [fire runtime](../development/environment-fire-runtime.md), [redstone signal runtime](../development/redstone-signal-comparator-daylight-runtime.md), [redstone delay runtime](../development/redstone-delay-components-runtime.md), [redstone piston runtime](../development/redstone-piston-runtime.md), [redstone explosion runtime](../development/redstone-explosion-runtime.md), [simulation tick and command runtime](../development/simulation-tick-and-command-runtime.md), [scheduled tick runtime](../development/scheduled-tick-runtime.md), [random tick runtime](../development/random-tick-runtime.md), [Phase 5 Region integration](../development/phase5-region-integration.md), [Phase 5 conformance](../development/phase5-conformance.md), [G01-P5-S001 report](../reports/goal-01/g01-p5-s001-block-runtime.md), [G01-P5-S002 report](../reports/goal-01/g01-p5-s002-placement-and-breaking.md), [G01-P5-S003 report](../reports/goal-01/g01-p5-s003-block-update-and-runtime.md), [G01-P5-S004 report](../reports/goal-01/g01-p5-s004-falling-block-runtime.md), [G01-P5-S005 report](../reports/goal-01/g01-p5-s005-test-instance-runtime.md), [G01-P5-S006 report](../reports/goal-01/g01-p5-s006-sim-003-block-runtime.md), [G01-P5-S007 report](../reports/goal-01/g01-p5-s007-sim-004-block-runtime.md), [G01-P5-S008 report](../reports/goal-01/g01-p5-s008-sim-005-block-runtime.md), [G01-P5-S009 report](../reports/goal-01/g01-p5-s009-env-001-runtime.md), [G01-P5-S010 report](../reports/goal-01/g01-p5-s010-environment-lighting.md), [G01-P5-S011 report](../reports/goal-01/g01-p5-s011-environment-weather.md), [G01-P5-S012 report](../reports/goal-01/g01-p5-s012-environment-fire.md), [G01-P5-S013 report](../reports/goal-01/g01-p5-s013-redstone-signal-runtime.md), [G01-P5-S014 report](../reports/goal-01/g01-p5-s014-redstone-delay-components.md), [G01-P5-S015 report](../reports/goal-01/g01-p5-s015-redstone-piston.md), [G01-P5-S016 report](../reports/goal-01/g01-p5-s016-redstone-explosion.md), [G01-P5-S017 report](../reports/goal-01/g01-p5-s017-simulation-tick-command.md), [G01-P5-S018 report](../reports/goal-01/g01-p5-s018-scheduled-ticks.md), [G01-P5-S019 report](../reports/goal-01/g01-p5-s019-random-ticks.md), [G01-P5-B1 report](../reports/goal-01/g01-p5-b1-region-integration.md), and [G01-P5-B2 report](../reports/goal-01/g01-p5-b2-phase5-conformance.md) | All 140 Phase 5 implementations, the TickScheduler root surface, NetworkIngress capture-order join, Region-boundary equivalence, bounded faults, and replay pass |
| Phase 6 — Players, items, inventories, and progression | `Complete` | [Phase 6 Region integration](../development/phase6-region-integration.md), [Phase 6 conformance](../development/phase6-conformance.md), [G01-P6-B1 report](../reports/goal-01/g01-p6-b1-region-integration.md), and [G01-P6-B2 report](../reports/goal-01/g01-p6-b2-phase6-conformance.md); complete slice/family evidence remains indexed by the implementation manifest | All 103 player/item slices and all ten Phase 6 C3 families pass; PlayerLifecycle golden/property/fuzz/replay/client-trace, NetworkIngress transitions, and TickScheduler capture boundaries close the phase. |
| Phase 7 — Entities, combat, mobs, AI, and spawning | `Complete` | [Region integration](../development/phase7-region-integration.md), [conformance](../development/phase7-conformance.md), [G01-P7-B1 report](../reports/goal-01/g01-p7-b1-region-integration.md), and [G01-P7-B2 report](../reports/goal-01/g01-p7-b2-phase7-conformance.md); complete slice/family evidence remains indexed by the implementation manifest | All 56 entity/mob gameplay slices and all seven Phase 7 protocol families pass deterministic lifecycle, transfer, persistence, tracking, fault, replay, and client-facing trace conformance |
| Phase 8 — World generation, dimensions, portals, and durable worlds | `Complete` | [WGEN-001 pipeline](../reports/goal-01/g01-p8-s001-worldgen-pipeline.md), [structures](../reports/goal-01/g01-p8-s002-worldgen-structures.md), [dimensions](../reports/goal-01/g01-p8-s003-worldgen-dimensions.md), [portals](../reports/goal-01/g01-p8-s004-worldgen-portals.md), [world border](../reports/goal-01/g01-p8-s005-worldgen-border.md), [durable-world Region integration](../reports/goal-01/g01-p8-b1-region-integration.md), [conformance](../reports/goal-01/g01-p8-b2-phase8-conformance.md), and [equivalence boundary](../reports/goal-01/g01-p8-b3-worldgen-equivalence-boundary.md) | All 28 world slices, three assigned surfaces, 12 joins, and durable/conformance gates are verified; the statistical observation remains explicitly deferred |
| Phase 9 — Remaining C3 services, client behavior, and C4 gates | `InProgress` | [command feedback routing](../reports/goal-01/g01-p9-s001-command-feedback.md), [client input/prediction](../reports/goal-01/g01-p9-s002-input-prediction.md), [menu semantics](../reports/goal-01/g01-p9-s003-menu-semantics.md), [observable effects](../reports/goal-01/g01-p9-s004-observable-effects.md), [boss/waypoint projection](../reports/goal-01/g01-p9-f001-boss-waypoint.md), [chat presentation](../reports/goal-01/g01-p9-f002-chat-presentation.md), [completion projection](../reports/goal-01/g01-p9-f003-completions.md), [particle projection](../reports/goal-01/g01-p9-f004-particle.md), and [player-info removal](../reports/goal-01/g01-p9-f005-player-info-remove.md) | `G01-P9-F006` is active; all four client slices and the first five of Phase 9's 13 required protocol families are verified |
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
| `G01-P2-B3` | `Complete` | P2-B2 | `49f7011`; [local runtime](../development/local-region-runtime.md) | Stable-order consistency-island execution, same-phase boundary effects, dual-generation entity/player transfer, bounded outputs, preflighted commits, and poisoned-tick refusal pass full gates |
| `G01-P2-B4` | `Complete` | P2-B2 | `c9317d2`; [persistence and recovery](../development/persistence-recovery.md) | Versioned bounded recovery points, contiguous journal tails, append-and-repoint fsync order, committed-transaction selection, corruption/torn-tail handling, revision acknowledgement, and handoff fencing pass full gates |
| `G01-P2-B5` | `Complete` | P2-B3, P2-B4 | `9c4d679`; [Lattice adapter](../development/lattice-adapter.md) | Exact Git pins, spatial placement cells, custom mapper fingerprint, claim/deadline fencing, durable handoff, bounded remoting envelopes, dependency isolation, and adapter integration tests pass full gates |
| `G01-P2-B6` | `Complete` | P2-B5 | `5373df0`; [multi-node deployment](../development/multi-node-deployment.md) | Versioned role/config schema, UUID-backed incarnations, two-stage readiness, bounded admission/drain accounting, management endpoints, actual three-process launcher smoke, immutable image, Compose, Kubernetes, and deployment drift gates pass |
| `G01-P2-B7` | `Complete` | P2-B6 | This row's containing commit; [topology conformance](../development/topology-conformance.md) | Locked 10,000-tick digest across local/in-process/three-process execution, canonical duplicate/reorder behavior, loss barrier, stale-owner and corruption rejection, durable node recovery, and retained-work overload pass |
| `G01-P3-B1` | `Complete` | Phase 2 | This row's containing commit; [wire foundation](../development/protocol-wire.md) | VarInt21 framing, VarInt/VarLong and structured primitives, Java-compatible UTF bounds, exact zlib envelopes, per-connection buffering, terminal malformed-input handling, independent C0/C1 goldens, and two isolated fuzz targets pass |
| `G01-P3-B2` | `Complete` | P3-B1 | This row's containing commit; [packet catalog](../development/protocol-catalog.md) | A compact 256-packet/9-lane Ferrite lock is independently regenerated from ignored OFF-REPORT-001, verified against `f34b0956b6399c749d4638cd6d3c9226685f41fa`, compiled only through `OUT_DIR`, and exposed through state/direction-local fail-closed lookup |
| `G01-P3-B3` | `Complete` | C0/C1 family batches | This row's containing commit; [required server connection](../development/protocol-server-connection.md) | Bounded framed driver composes status, refusal, offline login, compression callbacks, configuration prelude/tasks/liveness, full registry/tag projection, finish acknowledgement, and split Play installation |
| `G01-P3-B4` | `Complete` | P3-B3 | This row's containing commit; [semantic session routing](../development/semantic-session-routing.md) | Version-neutral ingress/egress, bounded virtual-host routing, two-stage admission, deterministic join commands, local Region execution, and failure atomicity pass full gates |
| `G01-P3-B5` | `Complete` | P3-B4 | This row's containing commit; [C0/C1 conformance](../reports/goal-01/g01-p3-b5-protocol-conformance.md) | Four independent goldens, seven malformed sessions, seven half-duplex checks, 34 ordered packets, real loopback status/login, 697 resolved tags, and an exact unmodified client reaching Play pass |
| `G01-P4-B1` | `Complete` | Phase 3 | This row's containing commit; [chunk join projection](../development/chunk-join-projection.md); [report](../reports/goal-01/g01-p4-b1-chunk-join-projection.md) | Immutable terrain snapshots, tickets, interest, transactional bounded batches, Java palettes/heightmaps/light/block entities, unload, respawn, Play enqueue, and stable loopback closure pass full gates |
| `G01-P4-B2` | `Complete` | P4-B1 | This row's containing commit; [player movement and transfer](../development/player-movement-and-region-transfer.md); [report](../reports/goal-01/g01-p4-b2-player-movement-and-transfer.md) | Stable player spawn, four movement variants, load/known-movement gates, collision admission, floating timeout, Play liveness/correction/disconnect, deterministic state projection, dual-generation transfer, and commit-fenced owner/chunk switch pass full gates |
| `G01-P4-B3` | `Complete` | P4-B2 | This row's containing commit; [block interaction](../development/block-interaction-and-convergence.md); [report](../reports/goal-01/g01-p4-b3-block-interaction-and-convergence.md) | Five interaction codecs, three convergence codecs, tick-local cumulative prediction ACK, strict targeting, Region-owned bootstrap placement/breaking, committed correction, and section aggregation pass full gates |
| `G01-P4-B4` | `Complete` | C2 family batches | This row's containing commit; [playable topology conformance](../development/playable-topology-conformance.md); [report](../reports/goal-01/g01-p4-b4-playable-topology-conformance.md) | Seven committed ticks, two Regions, generation-fenced player transfer, placement/rejection/break, canonical state digest, 16 exact Java packet bodies, and three process-isolated Lattice repetitions converge |
| `G01-P4-B5` | `Complete` | P4-B4 | This row's containing commit; [C2 acceptance and adversity](../development/c2-acceptance-and-adversity.md); [report](../reports/goal-01/g01-p4-b5-c2-acceptance-and-adversity.md) | Unattended and delayed/fragmented C2 TCP smokes, exact unmodified-client terrain/feedback/loaded/movement trace, malformed refusal, bounded rollback, and cross-Region convergence pass |
| `G01-P5-B1` | `Complete` | Phase 5 slice batches | `72149b7`; [Phase 5 Region integration](../development/phase5-region-integration.md); [report](../reports/goal-01/g01-p5-b1-region-integration.md) | Atomic generation-fenced boundary mechanics, bounded queues, scheduled/random continuity, replay receipts, handoff, and retryable Java projection pass full gates |
| `G01-P5-B2` | `Complete` | P5-B1 | This row's containing commit; [Phase 5 conformance](../development/phase5-conformance.md); [report](../reports/goal-01/g01-p5-b2-phase5-conformance.md) | TickScheduler golden/property/fault/replay, five Region-boundary equivalence cases, and NetworkIngress capture-order conformance pass full gates |
| `G01-P6-B1` | `Complete` | Phase 6 slice batches | This row's containing commit; [Phase 6 Region integration](../development/phase6-region-integration.md); [report](../reports/goal-01/g01-p6-b1-region-integration.md) | Generation/session-fenced ownership, canonical continuity, stale action/menu resync, bounded per-player projection, and multiplayer isolation pass |
| `G01-P6-B2` | `Complete` | P6-B1 and C3 family batches | This row's containing commit; [Phase 6 conformance](../development/phase6-conformance.md); [report](../reports/goal-01/g01-p6-b2-phase6-conformance.md) | PlayerLifecycle golden/property/fuzz/replay/client-trace, atomic session join/leave routing, and tick-capture membership conformance pass full gates |
| `G01-P7-B1` | `Complete` | Phase 7 slice/family batches | This row's containing commit; [Phase 7 Region integration](../development/phase7-region-integration.md); [report](../reports/goal-01/g01-p7-b1-region-integration.md) | Generation-fenced lifecycle, two-phase cross-Region transfer, durable replay receipts, canonical save/load, stable tracking, and atomic bounded fan-out pass |
| `G01-P7-B2` | `Complete` | P7-B1 | This row's containing commit; [Phase 7 conformance](../development/phase7-conformance.md); [report](../reports/goal-01/g01-p7-b2-phase7-conformance.md) | 128 ordering properties, 256 fixed-seed operation cases, ten faults, 64 transfer equivalence cases, eight replay frames, and the ten-event client projection golden close Phase 7 |
| `G01-P8-B1` | `Complete` | Phase 8 slice batches | This row's containing commit; [durable-world integration](../development/phase8-durable-world-integration.md); [report](../reports/goal-01/g01-p8-b1-region-integration.md) | Generation/revision/content fencing, exact bounded continuity, commit-receipt teardown, recovery/handoff, control-Region level state, ordered lifecycle, and offline inspection pass full gates |
| `G01-P8-B2` | `Complete` | P8-B1 | This row's containing commit; [Phase 8 conformance](../development/phase8-conformance.md); [report](../reports/goal-01/g01-p8-b2-phase8-conformance.md) | 963 records/14 families, 399 world tests, deterministic dispatch, boundaries, recovery, three surfaces, and 12 joins pass full gates |
| `G01-P8-B3` | `Complete` | P8-B2 | This row's containing commit; [equivalence boundary](../development/worldgen-equivalence-boundary.md); [report](../reports/goal-01/g01-p8-b3-worldgen-equivalence-boundary.md) | Three planned experiments, 8,200 planned runs, deterministic Ferrite seeds, and the no-same-seed-identity policy are executable and explicit |
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
| Gameplay slice partitions | 55 | 331 slices | 331 | 0 |
| Behavior-surface/join partitions | 5 owner batches | 46 owners | 20 | 26 |
| Required protocol partitions | 44 | 44 families | 36 | 8 |
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
| 2026-07-29 | `G01-D014` | `Accepted` | Persist bounded stable Region recovery points with contiguous journal tails through an intent/data/index/commit fsync sequence; accept only committed checksum-verified repoints and require strictly newer handoff generations. | [Persistence and recovery contract](../development/persistence-recovery.md) |
| 2026-07-29 | `G01-D015` | `Accepted` | Keep all Lattice types inside `ferrite-region-runtime`; bind the reviewed custom spatial mapper into Lattice fingerprints; combine deadline authority with Ferrite generation checks; move only durable Ferrite recovery points during handoff. | [Pinned Lattice adapter contract](../development/lattice-adapter.md) |
| 2026-07-29 | `G01-D016` | `Accepted` | Use one immutable server binary with a fail-closed versioned node schema; gate readiness on discovery membership then required placement domains; gate drain completion on sessions, Region authority, and durable commits; verify local, Compose, and Kubernetes profiles as one contract. | [Multi-node deployment contract](../development/multi-node-deployment.md) |
| 2026-07-29 | `G01-D017` | `Accepted` | Make activation generation fencing metadata rather than semantic gameplay input; preflight every partition before a logical tick commits; carry the same bounded Region envelope through local, in-process, and multi-process topology proofs; recover failed nodes only through checksum-verified durable points. | [Topology and fault conformance](../development/topology-conformance.md) |
| 2026-07-29 | `G01-D018` | `Accepted` | Treat every inbound wire-codec failure as terminal and non-resynchronizable; preserve the locked non-minimal VarInt, lossy UTF-8, nonzero-Boolean, raw-compression-envelope, and exact-zlib behaviors behind the isolated 26.2 adapter. | [Minecraft 26.2 wire foundation](../development/protocol-wire.md) |
| 2026-07-29 | `G01-D019` | `Accepted` | Keep OFF-REPORT-001 ignored; commit only a compact Ferrite lane lock whose array positions are wire IDs, regenerate it explicitly through `mc-ref`, and generate Rust descriptors solely into Cargo `OUT_DIR`. | [Minecraft 26.2 packet catalog](../development/protocol-catalog.md) |

## Terminal acceptance checklist

Change an item to `[x]` only with linked committed evidence.

- [x] Required modular workspace and dependency direction are verified.
- [x] Dedicated replay ownership, Cargo debug profiles, and guarded cache maintenance are verified.
- [ ] Region ownership exists for all mutable authoritative state.
- [x] Local, in-process Lattice, and multi-process Lattice execution converge.
- [x] One-command three-node development startup and graceful shutdown pass.
- [ ] Unmodified 26.2 client completes the supported C0-C3 baseline.
- [x] 327/327 `SourceSpecified` gameplay slices are verified.
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
