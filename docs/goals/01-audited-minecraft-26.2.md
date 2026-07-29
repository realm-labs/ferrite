# Goal 01 — Audited Minecraft Java 26.2 Server Baseline

## 1. Goal statement

Implement Ferrite's first production server baseline as a deterministic, Region-native Rust
service that can run locally or across multiple Lattice-backed nodes and serve an unmodified
Minecraft Java Edition 26.2 client.

The implementation denominator is the source-audited compatibility reference already committed
under `docs/reference/minecraft-java-26.2/`. Goal 01 does not wait for every possible future
experiment or optional service. It implements all behavior that the locked source already
specifies, preserves the four explicitly inconclusive observations as visible deferred evidence,
and leaves later Minecraft versions and nonessential C4 services for later work.

This is one persistent implementation goal. Phases and batches are resumable units inside the same
goal, not separate goals.

## 1.1 Verified start baseline

The baseline was reproduced on 2026-07-29 with:

```text
cargo run -q -p mc-reference --bin mc-ref -- readiness
cargo run -q -p mc-reference --bin mc-ref -- protocol readiness
cargo run -q -p mc-reference --bin mc-ref -- verify --offline
```

The locked starting denominator is:

| Surface | Verified baseline |
|---|---:|
| Parent behavior rules | 65 |
| Implementation-level leaf rules | 352 |
| Gameplay slices | 331 |
| `SourceSpecified` gameplay slices | 327 |
| `SourceInconclusive` gameplay slices | 4 |
| Behavior-surface roots | 10 mapped |
| Command roots | 92 in 12 mapped families |
| Cross-system joins | 36 mapped unordered pairs |
| Catalog IDs | 9,078, zero `Unreviewed` |
| Protocol packets | 256 |
| Protocol families | 58 |
| Required C0-C3 protocol families | 44 `Specified` |
| C4 protocol families | 14 `GatedOptional` |

The protocol inventory digest is
`f34b0956b6399c749d4638cd6d3c9226685f41fa`. The locked server artifact SHA-1 is
`823e2250d24b3ddac457a60c92a6a941943fcd6a`; the client artifact SHA-1 is
`2dc72797acbc1b63fc16a11c4ac393605f453754`.

Reference readiness is evidence that the behavior is specified. It is not runtime implementation
coverage. Goal 01 must create and maintain a separate implementation ledger.

## 2. Terminal outcome

Goal 01 is complete only when all of the following are true:

1. The responsibility-separated Rust workspace builds from a clean checkout, obeys the dependency
   direction, provides lightweight routine and explicit full-symbol debugging profiles, and has
   bounded workspace-scoped cache maintenance.
2. Every mutable chunk, entity, scheduled operation, world command, and persistence revision has
   exactly one `SimulationRegion` owner.
3. The deterministic local Region runner and Lattice-backed in-process and multi-process runners
   execute the same semantic messages and produce the same canonical committed state hashes.
4. A one-command development launcher starts at least three server nodes, and the same immutable
   server binary supports local, multi-process, container, and Kubernetes deployment profiles.
5. An unmodified 26.2 client completes status, offline-mode login, configuration, join, movement,
   correction, chunk streaming, block interaction, and the supported survival paths.
6. All 44 required C0-C3 protocol families are implemented with golden codecs, bounds, connection
   state, semantic mapping, acknowledgement/order tests, and required end-to-end traces.
7. All 14 C4 families have their specified configuration gate and enablement, refusal, or
   degradation boundary. Enabling every optional C4 service is not required.
8. Every one of the 327 `SourceSpecified` gameplay slices maps to production code and committed
   behavioral evidence with no required `Pending`, `InProgress`, or `Blocked` entry.
9. The source-specified portions of the four `SourceInconclusive` slices are implemented and tested;
   each unresolved observation remains explicitly `DeferredExperiment` and is not reported as
   vanilla-confirmed.
10. Every one of the 9,078 locked catalog IDs is loadable through its audited behavior family,
    explicit special owner, or data-only schema, with no runtime fallback that invents behavior.
11. All ten behavior surfaces and all 36 cross-system joins have implementation owners for
    admission, ordering, atomicity, persistence, and client projection or an explicit verified
    non-interaction.
12. Region snapshots, journal tails, activation generations, handoff, crash recovery, and
    asynchronous revision checks pass fault tests without dual authority or lost committed state.
13. Deterministic replay reproduces the canonical state hash and required client-visible trace
    across local, multi-worker, and multi-node topologies.
14. Formatting, Clippy, tests, reference verification, source-size, dependency-direction,
    public-API, fuzz, fault, and clean-checkout gates pass with committed evidence.
15. Capacity and latency claims are backed by reproducible benchmark profiles. Architectural
    estimates are not reported as production capacity.
16. The status ledger contains evidence for every terminal gate, the final completion batch is
    committed, and the worktree is clean.

An implemented packet codec without semantic behavior, a catalog identity without a behavior
owner, a mock, a TODO, or an unverified guess does not satisfy a gate.

## 3. Normative document order

Implementation decisions follow these sources in order:

1. This goal plan and its [status ledger](01-audited-minecraft-26.2-status.md) define delivery
   scope, batch order, and completion.
2. [Architecture](../architecture.md) defines responsibility boundaries, Region ownership,
   dependency direction, determinism, persistence, and protocol isolation.
3. The version-locked [reference entry point](../reference/minecraft-java-26.2/README.md),
   [methodology](../reference/minecraft-java-26.2/methodology.md), and
   [coverage report](../reference/minecraft-java-26.2/coverage.md) define evidence meaning.
4. The gameplay completion ledger, leaf rules, catalog, behavior-surface ledger, root inventories,
   and cross-system joins define the implementation denominator and observable semantics.
5. The [protocol reference](../reference/minecraft-java-26.2/protocol/README.md), completion ledger,
   codec documents, ordering rules, and conformance vectors define wire compatibility.
6. Repository [Agent guidelines](../../AGENTS.md) define source layout, imports, visibility, lint,
   checks, and commit-message policy.

If normative documents conflict, do not silently choose one. Record a narrow decision, update every
affected document, and add a regression fixture in the same batch. This plan may narrow delivery
scope, but it cannot redefine a source-specified Minecraft rule.

## 4. Included scope

### 4.1 Runtime workspace

Goal 01 creates the architecture's initial crate and application boundaries:

- `ferrite-foundation`;
- `ferrite-registry`;
- `ferrite-world`;
- `ferrite-simulation`;
- `ferrite-gameplay`;
- `ferrite-replay`;
- `ferrite-protocol`;
- `ferrite-persistence`;
- `ferrite-region-runtime`;
- `ferrite-server-runtime`;
- `ferrite-testkit`;
- `ferrite-tooling`;
- `ferrite-server`;
- `ferrite-cluster`;
- `ferrite-behavior-runner`;
- `ferrite-protocol-conformance`;
- `ferrite-world-inspector`.

Crate creation is driven by responsibility, not by one crate per rule. Internal modules must be
split before they become flat or exceed the source limits in `AGENTS.md`.

The future `client-runtime`, `client-bevy`, and native Ferrite protocol remain out of scope.

### 4.2 Build profiles and cache maintenance

The workspace skeleton includes the root Cargo profiles from the architecture:

```toml
[profile.dev]
debug = "line-tables-only"

[profile.dev.package."*"]
debug = false

[profile.debugging]
inherits = "dev"
debug = true
```

Routine builds, tests and Clippy use `dev`. Full-symbol debugging is explicit through
`--profile debugging`, so it has a separate profile output directory and does not replace ordinary
development artifacts.

The same skeleton adds a versioned cache policy and `ferrite-tooling` inspection/pruning command.
Repository bootstrap and supported developer task entry points invoke a rate-limited maintenance
check, with a workspace marker permitting at most one automatic check every 24 hours. Direct Cargo
commands perform no hidden deletion. The initial policy prunes inactive full-debugging and auxiliary
target namespaces before ordinary development artifacts and uses a configurable 40 GiB local
high-water mark. CI, fuzzing, coverage and benchmarks use isolated `CARGO_TARGET_DIR` roots keyed by
their toolchain, target, profile, lockfile and build flags.

Cleanup must be guarded by a workspace cache lock and verified resolved paths. Dry-run is the
default interactive behavior. Automated pruning may touch only declared workspace-owned Cargo
cache roots; it must preserve active builds, the most recent ordinary development artifacts, global
Cargo caches and `target/mc-reference/26.2/`. An unscoped periodic `cargo clean` is forbidden.

### 4.3 Region-native execution

Region ownership is present before the first mutable world:

- `WorldId`, `DimensionId`, and versioned `SimulationRegionKey`;
- one Region-local `bevy_ecs::World` for dynamic entities;
- Region-owned chunks, block entities, scheduled work, random streams, queues, and revisions;
- a deterministic local runner using the same Region messages as distributed execution;
- tick/phase/source/generation/sequence-tagged `BoundaryBatch` messages;
- explicit entity and player transfer records;
- Region snapshots and journal tails at committed tick boundaries;
- Lattice placement domains, versioned space-aware shard mapping, claims, fencing, handoff, and
  remoting behind `ferrite-region-runtime`;
- no Lattice types in world, simulation, gameplay, protocol semantics, or persistence schemas.

The exact Lattice Git revision is selected and recorded in an ADR before integration. An upgrade is
a reviewed milestone, not an incidental dependency update.

### 4.4 Multi-node deployment contract

Goal 01 includes a deployable cluster contract, not only an actor abstraction:

- one immutable `ferrite-server` executable and container image;
- role-based configuration for gateway, Region worker, coordinator candidate, and administration;
- explicit cluster name, node identity/incarnation, remoting bind/advertise addresses, discovery
  providers, placement capacity, storage, management, and Minecraft listener configuration;
- a `ferrite-cluster dev --nodes <N>` launcher with ephemeral local configuration;
- Docker Compose support for local multi-process testing;
- Kubernetes discovery, health probes, readiness, graceful drain, and rolling restart contracts;
- two-stage readiness: membership first, then required placement-domain readiness;
- bounded admission and overload behavior instead of unbounded queues.

Single-node mode uses the same Region ownership and routing model. It is a deployment profile, not
a separate simulation implementation.

### 4.5 Behavior implementation denominator

The implementation manifest is derived from, rather than manually copied from:

- `completion.toml` for 331 gameplay slices;
- `catalog/catalog.toml` for 9,078 content IDs;
- `behavior-surfaces.toml` and root inventories for entry/exit ownership;
- `cross-system-joins.toml` for interaction coverage;
- `protocol/completion.toml` for 58 protocol families and 256 packets.

Phase 0 creates `goals/minecraft-java-26.2/implementation.toml` and a verifier. Every reference
record must map exactly once to an implementation owner, batch, test evidence, and disposition.
Reference documents remain normative; the generated implementation manifest records progress only.

Implementation dispositions are:

| Disposition | Meaning |
|---|---|
| `Pending` | Required and not started |
| `InProgress` | Owned by the single active batch |
| `Implemented` | Production path exists; full batch gates not yet evidenced |
| `Verified` | Production path and required evidence pass |
| `DeferredExperiment` | Only the exact source-inconclusive observation is deferred |
| `NotApplicable` | Proven outside Ferrite's server responsibility with a reference owner |
| `Blocked` | Exact external blocker, attempted alternatives, and unblock condition recorded |

`NotApplicable` cannot be used for difficult work. `DeferredExperiment` cannot be applied to a
source-specified branch.

### 4.6 Source-inconclusive observations

The four incomplete observations are:

| Slice | Deferred observation |
|---|---|
| `SIM-SCHEDULED-TICKS-001` | Cross-chunk restored-tick ties with identical priority and reconstructed sub-order |
| `ENV-LIGHTING-001` | A universal mutation-to-render latency bound under arbitrary load |
| `PLY-BLOCK-BREAK-001` | Whether ACK restoration is rendered before the later authoritative air update |
| `WGEN-PIPELINE-EQUIVALENCE-001` | Experiment-selected statistical equivalence thresholds and allowed divergence |

Their source-specified state machines, ordering, packet publication, and algorithms remain required.
When Ferrite needs behavior at an unresolved branch, it selects a deterministic project policy,
labels it, tests it, and retains the experiment replacement condition. It never claims that policy
as observed vanilla behavior.

### 4.7 Official data and generated artifacts

Ferrite must not commit Mojang jars, assets, generated reports, copied tables, decompiled sources,
or proprietary data. Production schemas, algorithms, mappings, and project-authored fixtures are
committed. A deterministic import/build step consumes the user's locally available locked official
artifacts and produces ignored or distributable outputs according to a documented legal policy.

Runtime code must not depend on `mc-reference`. The tooling crate may invoke or consume verified
reference outputs during development, but production crates consume project-owned schemas and
validated generated bundles.

## 5. Explicitly excluded scope

The following are not Goal 01 completion requirements:

- a Ferrite-native client, Bevy frontend, renderer, UI, audio, or client asset pipeline;
- a Ferrite-native network protocol;
- server plugins, mod-loader APIs, scripting runtimes, or broad public extension SDKs;
- identical vanilla internal architecture or original save format;
- block-for-block same-seed vanilla world generation;
- enabling every C4 optional service, including online-mode authentication, secure chat, transfer,
  cookies, resource packs, dialogs, and diagnostics;
- resolving all 307 planned experiments when the source-specified implementation does not depend on
  their observation;
- support for a Minecraft version other than the locked Java Edition 26.2 target;
- cross-cluster federation or atomic transactions across independent worlds;
- capacity promises that have not been measured on a named reproducible profile.

Excluded paths must fail, refuse, or degrade according to the audited boundary. They must not be
silently accepted.

## 6. Delivery rules

### 6.1 Batch and commit policy

One implementation batch is active at a time. Each batch:

1. changes one responsibility or one bounded compatibility partition;
2. includes its code, tests, migrations, generated metadata, ADR changes, and documentation;
3. updates the status ledger and implementation manifest;
4. records exact commands and evidence;
5. passes its gates before commit;
6. preserves unrelated work and leaves no unrelated working-tree changes;
7. does not push, publish, deploy, or open a pull request without separate authorization.

Commit subjects follow `AGENTS.md`, for example:

```text
feat(region-runtime): add deterministic local region routing
feat(protocol): implement offline login compression
test(gameplay): cover scheduled tick ordering
```

The commit body ends with the exact batch trailer:

```text
Ferrite-Batch: G01-P2-B3
```

Generated rule or protocol partitions receive concrete batch IDs in `G01-P0-B2`. A partition should
normally contain one primary gameplay slice or protocol family. Closely coupled records may share a
batch only when splitting them would prevent an executable end-to-end assertion, and every member
must remain individually accounted for.

### 6.2 Universal gates

Every Rust implementation batch runs:

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo run -q -p mc-reference --bin mc-ref -- verify --offline
git diff --check
```

Use narrower tests while iterating, but the completed batch must pass the full commands unless the
ledger records an objective bootstrap limitation and the exact later batch that closes it.

Applicable batches additionally run:

- implementation-manifest coverage and traceability verification;
- protocol golden, malformed-frame, state-machine, fuzz, and packet-trace tests;
- deterministic replay under multiple eligible Region execution orders;
- persistence corruption, interruption, handoff, and recovery tests;
- local-versus-Lattice topology equivalence;
- unmodified-client smoke or directed differential traces;
- clean generated-data and legal-boundary checks;
- build-profile and workspace-cache policy verification;
- benchmark and fault-injection profiles.

Warnings, ignored failures, blanket lint suppression, hand-edited generated output, or a locally
patched reference artifact do not pass a gate.

### 6.3 Evidence and fidelity policy

- Start every behavior batch from its stable slice ID and leaf rule.
- Query every concrete registry ID through `mc-ref`; do not infer a sibling's data or behavior.
- Use exact source-specified constants, gates, ordering, abort paths, and side effects.
- Preserve each rule's `FidelityClass`. An intentional improvement requires a decision record and
  must not be reported as exact vanilla behavior.
- A packet round trip proves symmetry, not vanilla compatibility. Required protocol work includes
  official golden bytes, bounds, invalid cases, state transitions, semantic effects, and traces.
- The original client is an integration probe, not the only oracle.
- Do not copy Mojang implementation code or generated tables into Ferrite.
- Every behavioral regression test names its reference rule or protocol family.

## 7. Execution phases and batches

Phases are ordered by architectural dependency. A later batch may start only when its dependencies
are complete and the ledger identifies it as the earliest unblocked work. Generated behavior and
protocol partitions are materialized during Phase 0 before bulk implementation begins.

### Phase 0 — Freeze implementation truth

**Exit gate:** every audited record maps exactly once into a machine-verifiable implementation
manifest and an ordered batch DAG.

| Batch | Atomic deliverable |
|---|---|
| `G01-P0-B1` | Reproduce the locked source, gameplay, surface, catalog, experiment, and protocol baselines; record hashes, counts, toolchain identity, and the first immutable Goal 01 baseline report. |
| `G01-P0-B2` | Add the implementation-manifest schema and deterministically materialize concrete data, gameplay-slice, surface/join, and protocol-family batch rows with dependencies and test owners. |
| `G01-P0-B3` | Add coverage tooling that rejects missing, duplicate, dead, falsely completed, or stale reference mappings and renders the status counters used by this ledger. |
| `G01-P0-B4` | Record the initial ADR set: dependency direction, dedicated replay ownership, Cargo profile/cache policy, canonical encoding/hash, Region mapping, tick/boundary order, persistence recovery point, Lattice revision policy, protocol target, and official-data import boundary. |

### Phase 1 — Workspace, identity, data, and deterministic primitives

**Exit gate:** the modular workspace builds with the required debug profiles and guarded cache
maintenance, locked content can be imported without committing official artifacts, and canonical
primitives have golden tests.

| Batch | Atomic deliverable |
|---|---|
| `G01-P1-B1` | Replace the placeholder package with the initial responsibility-separated crates and applications, including `ferrite-replay`; add dependency-direction enforcement, root `dev`/`debugging` profiles, isolated auxiliary target roots, the versioned cache policy, guarded inspect/dry-run/prune tooling, and the daily rate-limited maintenance hook used by repository task entry points. |
| `G01-P1-B2` | Implement coordinates, stable IDs, resource identifiers, checked numeric helpers, bounds, directions, and explicit world/dimension/Region identity. |
| `G01-P1-B3` | Implement deterministic registries, contribution ordering, block-state schemas, persistent/runtime ID separation, content manifests, and provenance. |
| `G01-P1-B4` | Implement the legal local-artifact import pipeline, schema validation, generated-bundle digest, drift checks, and representative catalog fixtures. Materialize data partitions from Phase 0. |
| `G01-P1-B5` | Implement named random streams and deterministic selection, then implement canonical encoding, state hashing, command/event envelopes, replay headers, verification and divergence diagnostics in `ferrite-replay`. |
| `G01-P1-B6` | Add `ferrite-testkit`, behavior runner, scenario DSL, fake clocks, deterministic seeds, snapshot comparison, malformed-input harnesses, and CI/repository checks. |

### Phase 2 — Region-native local and distributed runtime

**Exit gate:** adjacent Regions execute, transfer, snapshot, recover, and move between nodes without
topology-dependent state.

| Batch | Atomic deliverable |
|---|---|
| `G01-P2-B1` | Implement palette-backed chunk/section storage and Region-owned voxel/ECS partitions with immutable query views. |
| `G01-P2-B2` | Implement the fixed Region tick pipeline, command admission, journals, phase barriers, boundary batches, and canonical Region/world hashes. |
| `G01-P2-B3` | Implement cross-Region entity/player transfer, immediate boundary effects, generation fencing, and the deterministic local Region runner. |
| `G01-P2-B4` | Implement Region commit snapshots, journal tails, revision-safe asynchronous work, recovery points, and local crash tests. |
| `G01-P2-B5` | Pin Lattice and implement placement domains, versioned space-aware shard mapping, claims, lease fencing, routing, bounded mailboxes, and handoff behind the adapter. |
| `G01-P2-B6` | Implement the node/role configuration schema, management health/readiness, graceful drain, one-command three-node launcher, Compose profile, and Kubernetes deployment contract. |
| `G01-P2-B7` | Prove 10,000-tick local/in-process/multi-process equivalence, stale-owner rejection, message fault outcomes, node loss recovery, and bounded overload behavior. |

### Phase 3 — Protocol C0 and C1

**Exit gate:** an unmodified client discovers the server, completes offline login/configuration, and
enters the semantic play boundary.

| Batch | Atomic deliverable |
|---|---|
| `G01-P3-B1` | Implement bounded TCP framing, VarInt and structured primitives, compression, buffer limits, malformed-input policy, and fuzz targets. |
| `G01-P3-B2` | Generate and verify the locked state/direction packet catalog without committing Mojang reports; isolate all wire IDs inside the 26.2 adapter. |
| `G01-P3-Fnn` | Implement each concrete C0/C1 protocol-family partition materialized by Phase 0 with golden, boundary, state, and semantic tests. |
| `G01-P3-B3` | Complete handshake, status, ping, wrong-version refusal, offline login, compression, configuration tasks, registry/tag/feature projection, and finish acknowledgement. |
| `G01-P3-B4` | Connect normalized session ingress/egress to Region routing without exposing packet or Lattice types to simulation. |
| `G01-P3-B5` | Add headless conformance, malformed-session, half-duplex transition, packet-order, and unmodified-client C0/C1 smoke suites. |

### Phase 4 — C2 minimal playable multi-Region world

**Exit gate:** an unmodified client moves and edits blocks across Region boundaries while
authoritative correction and topology equivalence hold.

| Batch | Atomic deliverable |
|---|---|
| `G01-P4-B1` | Implement join/respawn semantics, chunk tickets, view position, minimal terrain, sections, biomes, heightmaps, light payloads, block entities, unload, and bounded streaming. |
| `G01-P4-B2` | Implement player spawn, movement variants, collision admission, keepalive, teleport correction/acknowledgement, disconnect, and explicit Region transfer. |
| `G01-P4-B3` | Implement block targeting, placement, breaking, prediction sequence acknowledgement, rejection, authoritative correction, and committed replication aggregation. |
| `G01-P4-Fnn` | Implement every remaining C2 family partition with golden codecs, semantic state, ordering, and end-to-end traces. |
| `G01-P4-B4` | Run the same playable scenario through local and Lattice-backed runners and require equal committed hashes and compatible packet traces. |
| `G01-P4-B5` | Add unmodified-client C2 smoke, adverse latency/batching tests, malformed input, backpressure, and cross-Region movement/block scenarios. |

### Phase 5 — Simulation, blocks, environment, and redstone

**Exit gate:** all audited `simulation`, `blocks`, `environment`, and `redstone` slices are verified,
including Region-interior and Region-boundary variants.

| Batch | Atomic deliverable |
|---|---|
| `G01-P5-Snn` | Implement each Phase 0 partition for tick/time/chunks, scheduled/random ticks, block state/update/interaction behavior, fluids, fire, weather, lighting, redstone, pistons, and explosions. |
| `G01-P5-B1` | Integrate persistence/reload continuity, boundary transactions, queue budgets, and client projection for these domains. |
| `G01-P5-B2` | Run subsystem golden/property/fault suites, deterministic replay, interior-versus-boundary scenarios, and close all mapped slice/surface/join rows for this phase. |

### Phase 6 — Players, items, inventories, and progression

**Exit gate:** all audited `player` and `items` slices are verified and their required C3 protocol
paths converge under prediction and correction.

| Batch | Atomic deliverable |
|---|---|
| `G01-P6-Snn` | Implement each Phase 0 player/item partition, including interaction, movement variants, item use, containers, menus, recipes, crafting, equipment, loot, experience, effects, advancement, and game-rule gates. |
| `G01-P6-Fnn` | Implement the coupled C3 inventory, container, item, command, and progression protocol families. |
| `G01-P6-B1` | Integrate Region ownership, save/reload continuity, stale menu/action resynchronization, client projection, and multi-player isolation. |
| `G01-P6-B2` | Run subsystem golden/property/fuzz/replay/client-trace suites and close all mapped slice/surface/join rows for this phase. |

### Phase 7 — Entities, combat, mobs, AI, and spawning

**Exit gate:** all audited `entities` and `mobs` slices are verified across Region ownership,
lifecycle, persistence, and client projection.

| Batch | Atomic deliverable |
|---|---|
| `G01-P7-Snn` | Implement each Phase 0 entity/mob partition, including lifecycle, physics, vehicles, projectiles, damage, effects, drops, AI, spawning, despawn, breeding, raids, patrols, and special entity families. |
| `G01-P7-Fnn` | Implement the coupled C3 entity, metadata, attribute, equipment, effect, passenger, combat, and spawn protocol families. |
| `G01-P7-B1` | Integrate cross-Region transfer, lifecycle continuity, activation/deactivation, save/load, tracking, and bounded fan-out. |
| `G01-P7-B2` | Run entity/mob golden/property/fault/replay/client-trace suites and close all mapped slice/surface/join rows for this phase. |

### Phase 8 — World generation, dimensions, portals, and durable worlds

**Exit gate:** every source-specified world slice and catalog family is verified under Ferrite's
documented player-visible-equivalence boundary.

| Batch | Atomic deliverable |
|---|---|
| `G01-P8-Snn` | Implement each Phase 0 world partition, including dimension records, terrain pipeline, carvers, features, structures, jigsaw, portals, borders, and cross-dimensional transfer. |
| `G01-P8-B1` | Integrate asynchronous generation with revision checks, Region storage, content locks, world lifecycle, handoff, recovery, and the world inspector. |
| `G01-P8-B2` | Validate every worldgen behavior/data family, deterministic project-owned generation, structural invariants, boundary behavior, save/load, and crash recovery. |
| `G01-P8-B3` | Record the unresolved statistical-equivalence experiment separately while verifying all source-specified control flow and the architecture's non-identical-seed equivalence contract. |

### Phase 9 — Remaining C3 services, client-observable behavior, and C4 gates

**Exit gate:** required protocol and client-observable surfaces are complete, and every optional
service has an explicit audited gate.

| Batch | Atomic deliverable |
|---|---|
| `G01-P9-Snn` | Implement remaining client-observable, command/administration, lifecycle, reload, and cross-system-ordering partitions not closed by earlier gameplay phases. |
| `G01-P9-Fnn` | Implement every remaining C3 protocol-family partition, including ordering, acknowledgement, chat/service boundaries, and refusal behavior required by the baseline. |
| `G01-P9-Onn` | Implement each C4 family's configuration gate and exact disabled/refusal/degradation behavior; optional enabled service implementations require an explicit registered child batch. |
| `G01-P9-B1` | Run the complete 256-packet inventory/coverage verifier, required family conformance, optional-gate matrix, reconnect/reconfiguration traces, and root-surface implementation audit. |

### Phase 10 — Scale, hardening, and completion

**Exit gate:** every terminal condition has committed evidence and the goal is complete.

| Batch | Atomic deliverable |
|---|---|
| `G01-P10-B1` | Add manifest-wide rule reachability, catalog lowering, protocol mapping, surface/join ownership, public-API, dependency, source-size, and generated-artifact audits. |
| `G01-P10-B2` | Expand property and coverage-guided fuzz suites for codecs, commands, ordering, Region boundaries, persistence, corruption, and replay; retain reproducible failure corpora. |
| `G01-P10-B3` | Run multi-node fault injection for owner crash, network partition, message loss/duplication/reordering, control-plane outage, handoff, drain, restart, and rolling upgrade. |
| `G01-P10-B4` | Establish named capacity profiles and benchmark Region tick cost, queue pressure, memory, storage, network fan-out, uneven hotspots, many worlds, and rebalance objectives. |
| `G01-P10-B5` | Run cross-platform deterministic vectors, local/distributed replay equivalence, unmodified-client acceptance, clean-checkout builds, and complete reference/implementation coverage. |
| `G01-P10-B6` | Freeze supported CLI/configuration/library/deployment contracts and commit the final Goal 01 completion record only after the ledger and worktree are clean. |

## 8. Acceptance suites

### 8.1 Architecture and determinism

- dependency-direction and forbidden-type audits pass;
- no packet, Lattice, Bevy `Entity`, runtime registry ID, or executor type leaks across its boundary;
- stable IDs, Region keys, persistent content identity, and activation generations survive restart;
- local and distributed topologies produce equal canonical committed hashes;
- every queue, decoder, journal, mailbox, and replication path has an enforced bound;
- invalid or stale input does not mutate authoritative state or consume the wrong random stream;
- cross-Region ordering is explicit and replayable.

### 8.2 Gameplay and catalog

- 327/327 `SourceSpecified` slices are `Verified`;
- source-known parts of all four inconclusive slices are verified and their exact unknown remains
  `DeferredExperiment`;
- 9,078/9,078 catalog IDs resolve through validated production schemas and behavior owners;
- all 65 parent rules and 352 leaf rules retain implementation reachability;
- Region-interior and Region-boundary variants exist for every mechanic that can cross ownership;
- persistence/reload and client projection agree with committed authoritative outcomes.

### 8.3 Protocol

- all 256 packet identities remain partitioned exactly once;
- 44/44 required C0-C3 families pass their specified implementation gates;
- 14/14 C4 families pass their configuration and disabled/refusal/degradation gates;
- codec tests cover golden bytes, bounds, truncation, malformed values, residual data, and fuzzing;
- session tests cover legal/illegal transitions, reconnect, acknowledgement, and correction;
- semantic tests bind packet traces to Region state and replay hashes;
- unmodified-client smoke tests cover the supported baseline.

### 8.4 Operations and scale

- one command starts and stops a three-node development cluster cleanly;
- readiness does not pass before membership and required placement domains are ready;
- graceful drain stops admission, transfers authority, flushes committed work, and then exits;
- failure tests never permit two generations to commit the same Region;
- overload shedding preserves authoritative work and remains observable;
- published performance/capacity numbers include workload, runner, topology, revision, and variance.

### 8.5 Engineering

- every source file complies with `AGENTS.md` or has a reviewed exception;
- the checked root Cargo profiles match the lightweight `dev` and full-symbol `debugging` contract;
- cache maintenance passes path-containment, active-lock, protected-cache and dry-run tests;
- format, Clippy, workspace tests, offline reference verification, links, and diff checks pass;
- unsafe code, lint suppressions, public re-exports, and public APIs have explicit audits;
- generated outputs reproduce from documented inputs without repository drift;
- no Mojang-owned artifact or copied implementation is committed;
- every batch has one Conventional Commit and exact `Ferrite-Batch` trailer;
- a clean checkout reproduces the final acceptance report.

## 9. Progress accounting

The [status ledger](01-audited-minecraft-26.2-status.md) is updated in every batch. Phase 0 tooling
becomes the machine source of truth for individual data, slice, surface/join, and protocol rows.

Progress is reported with exact denominators:

```text
Audited gameplay: verified SourceSpecified slices / 327
Inconclusive known surface: verified slices / 4
Catalog: runtime-verified IDs / 9,078
Protocol required: verified families / 44
Protocol optional gates: verified families / 14
Behavior surfaces: implementation-verified roots / 10
Cross-system joins: implementation-verified joins / 36
```

Phase completion, a connecting client, or a compiling workspace cannot substitute for these
denominators.
