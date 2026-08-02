# Minecraft 26.2 Worldgen Execution Architecture

This document defines how Ferrite executes exact Minecraft Java 26.2 world generation efficiently.
The version-locked behavior reference owns the required result; this document owns the internal
algorithm representation, task graph, memory layout, scheduling, authority, persistence, and
performance strategy used to produce it.

The design does not replace vanilla generation with a similar Ferrite algorithm. It implements the
same observable 26.2 generation semantics through an independent execution architecture:

```text
locked 26.2 content and world configuration
  -> immutable WorldgenPlan26_2
  -> dependency-aware GenerationTask graph
  -> bounded shared execution pools
  -> GenerationColumnBuilder and ordered cross-chunk patches
  -> immutable generated candidate
  -> current Region owner validates and commits
  -> persistence, collision, simulation, and client projection
```

## 1. Required invariants

- Identical declared inputs produce zero unexplained normalized semantic differences from the
  locked official 26.2 server population.
- Task scheduling, worker count, completion order, node placement, cache hits, and restart do not
  affect the generated result.
- Random implementations, seeds, stream derivation, call order, integer overflow, floating-point
  evaluation, traversal order, and data-pack dispatch preserve the behavior required by the locked
  reference.
- A worker computes an uncommitted candidate. Only the current Region owner may publish it after
  validating ownership generation, request identity, source/dependency revisions, continuation
  version, and content/worldgen digests.
- Generation, lighting, persistence, compression, projection, and Region tick work have separate
  bounded admission. Moving work into an unbounded queue is not an optimization.
- Online performance is measured without pre-generating the measured chunks or reducing required
  content, view distance, simulation distance, durability, or exactness.

## 2. Algorithm representation

### 2.1 Immutable worldgen plan

Startup lowers the locked content and configured world into an immutable `WorldgenPlan26_2` shared
by tasks for the same world-generation identity. Its compatibility key contains at least:

- Minecraft version and official artifact identity;
- generator and dimension settings;
- world seed and enabled data-pack/content digest;
- registry/runtime-ID mapping digest;
- Ferrite plan schema and optimized-executor version.

The plan owns lowered density functions, noise parameters, climate/biome selection, aquifer and ore
rules, surface rules, carvers, placed/configured features, structure sets/templates/processors,
heightmap rules, lighting inputs, and the exact stage operation plans.

Lowering may perform constant folding, compact instruction selection, immutable lookup-table
construction, and common-subexpression elimination for proven-pure nodes. It must retain the
original cache, random, short-circuit, visitation, and side-effect semantics. A graph node cannot be
deduplicated merely because its serialized shape is equal.

### 2.2 Reference and optimized executors

Two executors consume the same plan:

| Executor | Responsibility |
|---|---|
| `ScalarWorldgenOracle` | Straightforward, inspectable execution used for exactness, differential diagnosis, and optimized-path tests. |
| `OptimizedWorldgenExecutor` | Batch evaluation, reusable scratch memory, bounded caches, compact dispatch, safe vectorization, and other measured optimizations. |

The scalar executor remains available in tests and diagnostic tooling. The optimized executor must
match its intermediate checkpoints where defined and the official normalized semantic result.

Rust release optimization does not authorize approximate arithmetic. Fast-math, reassociation,
unreviewed fused operations, reduced precision, or platform-dependent native math is forbidden
where it can change a generation branch or final semantic state. SIMD is accepted only after the
same workload passes scalar, cross-platform, and official-server differential gates.

### 2.3 Evaluation strategy

The optimized density executor uses a compact tape or equivalent typed IR rather than per-sample
dynamic object dispatch. It should:

- evaluate interpolation cells and coordinate batches in the required traversal order;
- implement vanilla-significant `cache_once`, 2D/flat cache, interpolation, and cell-cache
  lifetimes explicitly;
- constant-fold and share only pure work;
- reuse thread-owned samplers and scratch arenas without leaking state between generation inputs;
- record per-node/stage profiling attribution without making instrumentation part of semantics.

GPU/OpenCL generation is not part of the first production path. It may be evaluated later for an
optional offline or proven-exact executor after CPU correctness, data-transfer cost, hardware
fallback, and license provenance are closed.

## 3. Candidate construction and data layout

### 3.1 Generation-only builder

Normal gameplay mutation APIs preserve revisions, listeners, block entities, ticks, lighting, and
other live-world effects per mutation. Noise and terrain fill must not pay that general-purpose cost
for every unpublished block.

`GenerationColumnBuilder` is a task-local, non-authoritative candidate with responsibility-owned
sub-builders:

```text
GenerationColumnBuilder
├── DenseSectionScratch<BlockStateId, 4096>
├── DenseBiomeScratch<BiomeId, 64>
├── GenerationHeightmapBuilder
├── PostProcessingBuilder
├── ScheduledTickBuilder
├── BlockEntityBuilder
├── StructureStateBuilder
└── OrderedPatchLog
```

Dense scratch sections are acquired from a bounded worker-local pool and initialized lazily.
Uniform sections retain a single-value representation. When a section is frozen, one linear pass
builds the frequency/index table, selects the canonical single/local/direct palette form, allocates
packed storage once, and installs the final values. Revision and derived-state changes are recorded
once for the accepted stage/candidate rather than once per generated voxel.

The existing `ChunkColumn::set_block` remains the correct API for ordinary authoritative gameplay
mutation. It is not the bulk noise-fill interface.

### 3.2 Dense base and sparse ordered patches

- Biome and noise/base-terrain stages write dense buffers.
- Surface and carver stages mutate the candidate through stage-specific bulk views.
- Features, structures, block entities, post-processing marks, and ticks emit ordered patches when
  their writes are sparse or cross chunk boundaries.
- Heightmaps are updated during the bulk traversal when the exact rule permits it; they are not
  repeatedly rescanned without profiling evidence.
- Lighting consumes a frozen block/fluid candidate and produces revision-bound immutable light
  state.

Each patch carries a deterministic `VanillaOrderKey` derived from the source chunk, status,
configured feature or structure order, placement index, and local emission sequence. The exact key
shape is frozen with the official differential evidence. Sorting by arbitrary worker completion or
hash-map iteration is forbidden.

## 4. Chunk-status dependency graph

### 4.1 Task key and dependencies

The scheduler materializes the existing `ChunkStatus::direct_requirement` rules as a DAG. A task is
identified by:

```text
(world, dimension, chunk, target status, worldgen identity, activation generation)
```

It contains immutable source state or a validated source reference, neighbor dependency revisions,
priority, estimated CPU/memory cost, request/continuation identity, and cancellation state.

A request for `Full` expands only the required vanilla status pyramid. Dependency halo chunks do
not advance farther than required unless another ticket demands it. Ready tasks may execute in
parallel when their dependency and write-conflict sets are disjoint.

### 4.2 Safe stage fusion

The scheduler may execute a contiguous chain of statuses in one coarse task when:

- every dependency for the final target is already satisfied;
- no intermediate Region publication, durable compatibility boundary, or externally observable
  event is required;
- the chain has one candidate owner and bounded memory/cancellation cost;
- exact status checkpoints remain reconstructible for diagnostics and continuation validation.

Stage fusion removes repeated queue round-trips and full-column copies. It does not remove semantic
operations. `Full` activation and all authoritative publication remain Region-owned boundaries.

### 4.3 Conflict-aware parallelism

Noise, biome, and other radius-zero tasks are normally parallel across chunks. Feature, structure,
lighting, and other neighbor-sensitive tasks require explicit read/write sets. The scheduler may
use disjoint neighborhoods, spatial coloring, or Region-owned ordered merge only when the method is
proven to preserve vanilla ordering. It must serialize an unresolved conflict instead of accepting
nondeterministic output.

Global submission-order publication is unnecessary for independent chunks and causes head-of-line
blocking. Ordering is local to actual semantic dependencies and overlapping effects.

## 5. Priority, admission, and fairness

### 5.1 Priority classes

The scheduler orders ready work by a stable priority tuple whose leading class is:

1. player safety, collision, spawn, portal, or interaction blocker;
2. chunks required for the nearest playable view;
3. chunks near the player and ahead of current view/movement;
4. the remainder of the declared client view;
5. simulation and background continuity work;
6. operator pre-generation and speculative warming.

Within a class, distance, client acknowledgement/backpressure, age, estimated cost, and stable task
identity decide order. Aging prevents starvation. Per-world, per-session, and per-priority admission
prevents a fast-moving client or background pre-generator from consuming the node.

The first-playable set is defined by exact-client acceptance and collision safety, not a hard-coded
marketing radius. Completed `Full + light + collision-ready` chunks stream center-out immediately;
the server does not wait for the complete view before sending useful results.

### 5.2 Separate bounds

Do not use one number for all of these independent controls:

- maximum queued tasks and estimated scratch bytes;
- maximum CPU tasks in flight;
- maximum completions collected per tick;
- maximum Region validation/commit work per tick;
- maximum persistence/compression work;
- maximum chunks/bytes offered to each client.

Admission refills available workers without forcing the Region tick to commit every completion at
once. Overload sheds speculative/background work first, retains authoritative work, and exposes its
reason through metrics.

## 6. Execution pools and locality

One node-level scheduler serves all worlds and dimensions. Fixed pools per dimension can
oversubscribe the machine and strand capacity. The node maintains separately budgeted work classes:

| Work class | Examples |
|---|---|
| Region authority | Tick, command admission, candidate validation, ordered patch merge, activation. |
| Generation CPU | Density, biome, surface, carver, feature, structure candidate computation. |
| Lighting CPU | Initial and incremental light computation with its own queue budget. |
| Storage/I/O | Chunk load/decode, compression, immutable payload upload, head/checkpoint work. |
| Projection/network | Snapshot encoding, compression, per-client chunk delivery. |

The actual worker counts are configured or selected from measured hardware profiles; they are not
hard-coded to four. The controller protects the 20 Hz Region deadline and memory limit, using tick
debt, runnable work, queue latency, and scratch-memory pressure as inputs. A high core count does not
justify starving Region ticks, network, storage, or garbage collection/system work.

Worker-local immutable plan handles, samplers, and scratch arenas maximize locality. Shared mutable
generator state and one receiver mutex in front of all workers are avoided. Scheduling may use a
bounded priority queue plus worker-local ready queues, provided task selection and cancellation
remain observable and deterministic publication does not depend on stealing order.

## 7. Region authority and distributed execution

### 7.1 Local ownership boundary

Generation workers never mutate a live `RegionVoxelState`. The current owner captures or references
immutable source/dependency state, marks the request pending, and submits a candidate task. On
completion it validates:

- Region and activation generation;
- request and continuation identities;
- source chunk and neighbor revisions;
- target status and dependency closure;
- Minecraft/content/worldgen plan digests;
- patch bounds, ordering identities, and resource limits.

Only then does the Region commit the candidate, emit events, make the chunk projectable, and allow
persistence/projection to observe it. Stale or cancelled results are discarded without partial
mutation.

### 7.2 Remote workers

Goal 07 may execute generation remotely, but the default placement is colocated with the Region
owner to avoid moving full candidates and neighbor snapshots. Remote execution is admitted only for
coarse tasks or safely fused stage chains whose compute benefit exceeds transfer/storage cost.

A remote task receives versioned immutable inputs and a plan/content identity already present on
the worker. It returns a bounded candidate or immutable object reference plus semantic digest and
measurements. Object storage is a durable publication boundary, not a fine-grained scratch-message
bus. The Region owner and storage metadata plane independently enforce activation fencing.

## 8. Persistence, caches, and cancellation

- Persist published authoritative milestones and continuation identities, not every scratch write.
- A crash may require deterministic regeneration of unpublished work; it must not publish partial
  state or lose already acknowledged authoritative mutation.
- Unload either cancels an unpublished candidate or finishes and durably publishes it according to
  the frozen cost/continuation policy.
- Immutable lowered plans and read-only generator data are shared. Scratch and intermediate caches
  are bounded, worker-local where practical, and keyed by all compatibility inputs.
- Cached chunks, projections, durable payloads, or density samples never survive a content/worldgen
  identity change under an old key.
- Cancellation is checked at coarse safe points so abandoned exploration does not consume an
  unbounded tail of CPU, while the check frequency does not dominate inner noise loops.

## 9. Required observability

Every generation task reports queue wait, execution time, commit wait, status chain, dependency
wait, cancellation/stale outcome, CPU time, allocation/scratch high-water, cache hits, bytes copied,
worker/node, and source/target revision identities.

The end-to-end workload additionally records:

- cold generation and warm load chunks/s;
- per-status and fused-chain p50/p95/p99/max;
- admission-to-first-playable and complete-view latency;
- exploration throughput and center-distance completion order;
- Region tick MSPT/deadline misses while generating;
- persistence, compression, projection, and chunk-send latency/bytes;
- scaling efficiency and memory at each worker count;
- local versus remote generation and storage overhead.

The [performance engineering contract](performance-engineering.md) owns report metadata, claim
boundaries, thresholds, and regression policy.

## 10. Current implementation risks

The current formal path is a bounded proof of lifecycle integration, not the target executor:

- [`FormalGenerationWorker`](../../crates/ferrite-server-runtime/src/world_service/formal_lifecycle.rs)
  creates at most four workers per dimension and shares a mutex-protected standard-library receiver;
- production configuration admits at most four generation results/requests per tick, coupling worker
  admission to Region commit throughput;
- demand is traversed by coordinate-ordered `BTreeSet`, so chunks nearest the player are not
  necessarily generated first;
- `begin_generation` clones the authoritative column, then the worker clones the request source
  again for every status;
- the formal dimension adapters still use project-owned V1 generators that must be replaced by the
  exact plan rather than optimized as the final algorithm;
- dense generation repeatedly uses live `set_block`, whose palette search/repack and per-mutation
  revision behavior is appropriate for gameplay but expensive for unpublished terrain;
- lighting adapters may recompute the whole chunk at both initialization and light statuses.

For a cold square view distance `d`, the current four-status-per-tick admission alone has the rough
lower bound below before generation CPU, lighting, storage, or network cost:

```text
((2d + 1)^2 chunks × 11 Empty-to-Full transitions) / (4 transitions/tick × 20 ticks/s)
```

At `d = 10` this is about 60.6 seconds. This explains why scheduling and center-first readiness are
first-class performance work rather than thread-count tuning.

## 11. Goal 04 implementation sequence

| Batch | Execution-architecture responsibility |
|---|---|
| `G04-P6-B2` | Connect the official normalized semantic oracle to stage and final-candidate diagnostics. |
| `G04-P6-B3` | Replace project-owned V1 output with the exact scalar `WorldgenPlan26_2` path and preserve all ownership/continuation gates. |
| `G04-P6-B4` | Freeze workloads, instrument every layer above, and publish official/exact-Ferrite baselines and reviewed thresholds. |
| `G04-P6-B5` | Introduce the builder, remove redundant copies, add priority/admission separation, install the DAG and safe fusion, share node pools, then optimize the plan executor in individually measured changes. |
| `G04-P6-B6` | Re-run exactness, determinism, restart, cancellation, overload, cross-Region, first-view, exploration, performance, and exact-client acceptance. |

Within `G04-P6-B5`, each optimization keeps the scalar executor and prior production path available
as a diagnostic oracle until its exactness, failure, and performance evidence is committed. The
batch must not become one unreviewable rewrite; its completion record links focused implementation
commits and raw reports for every material step.

## 12. Rejected shortcuts

- Retaining the simplified V1 terrain because it is faster than exact vanilla generation.
- Treating a whole Region as one indivisible generation task.
- Creating a fixed worker pool for every dimension or Region.
- Increasing threads without measuring scheduler, memory, storage, projection, and tick contention.
- Mutating authoritative chunks from worker threads or remote workers.
- Publishing in arbitrary completion order where features or structures overlap.
- Waiting for the entire view before sending the nearest playable chunks.
- Reporting pre-generated-world join time as online generation performance.
- Copying third-party implementation code without exact revision and license provenance.
- Adding GPU, JIT, SIMD, unsafe mutation, or native math before the scalar exactness oracle passes.
