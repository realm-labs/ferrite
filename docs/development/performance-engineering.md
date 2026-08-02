# Performance Engineering Contract

Ferrite treats performance as a continuously enforced production property, not a final capacity
exercise. Correct Minecraft 26.2 semantics, bounded resource use, low interaction latency, and
sustained throughput are simultaneous requirements. An optimization that changes vanilla-significant
state, hides required work behind pre-generation, or moves unbounded work into another queue fails
this contract even if one headline number improves.

The existing [capacity benchmark profiles](capacity-benchmarks.md) remain useful synthetic Region
topology regressions. They do not measure production chunk generation, loading, persistence,
projection, client join, exploration, entities, or distributed storage.
The concrete generation algorithm, builder, task-graph, priority, pool, and authority design is in
the [Minecraft 26.2 worldgen execution architecture](worldgen-execution-architecture.md).

## Measurement layers

| Layer | Required measurements | Purpose |
|---|---|---|
| Algorithm stage | wall time, CPU time, allocations, peak temporary bytes, cache behavior | Find hot generation, lighting, simulation, encoding, and persistence work. |
| Chunk pipeline | cold generation and warm load chunks/s; stage p50/p95/p99/max; queue wait and high-water | Separate useful work from scheduling, dependency, serialization, and I/O delay. |
| Client session | admission-to-first-playable-chunk and admission-to-complete-initial-view latency; bytes and chunks sent | Measure what a player actually waits for. |
| Active gameplay | tick MSPT p50/p95/p99/max, deadline misses, exploration throughput, correction latency | Prove generation and loading do not starve simulation or interaction. |
| Server capacity | CPU, RSS, bytes per active chunk/entity/session, queue bounds, scaling efficiency by worker count | Publish a reproducible hardware/workload envelope. |
| Distributed path | object/head commit latency, cache hit ratio, handoff/recovery latency, remote projection cost | Detect costs hidden by local-only execution. |

Every aggregate must retain sample count, warmup, variance, revision, dirty state, compiler/profile,
OS, CPU model, core allocation, memory, storage, network topology, Java/reference-server identity,
world inputs, and cache state. A single best run or an unspecified `chunks/s` value is not evidence.

## Version-locked workloads

The first real-world baseline batch must freeze a machine-readable workload suite containing:

- fixed Minecraft 26.2 version, world seed, generator settings, enabled data packs, dimension,
  chunk coordinates, requested status, and generation order;
- representative Overworld, Nether, and End populations, including structures, feature-dense
  chunks, empty/sparse chunks, negative coordinates, Region boundaries, and continuation after
  restart;
- explicit cold-cache and warm-cache runs, forward and shuffled request orders, and worker counts
  from one through the hardware limit;
- initial view and sustained outward exploration at declared view/simulation distances with one and
  multiple exact clients;
- simultaneous generation, ticking, persistence, compression, projection, and ordinary gameplay;
- local `RegionFileStore`, local MinIO-plus-etcd conformance, and the selected production backend at
  the Goal that owns each storage profile.

The official server is the mandatory compatibility and reference baseline on the same eligible
workload. A third-party implementation may be recorded only when it supports the same Minecraft
version and inputs; its number is contextual evidence, not Ferrite's acceptance oracle. Every
Ferrite output population must also pass the normalized semantic differential oracle, so a faster
divergent generator cannot enter the baseline.

## Gates and claim policy

`G04-P6-B4` establishes the first production baseline and freezes reviewed thresholds after actual
measurement. Until that report exists, Ferrite must not publish a real chunk-generation or
first-view performance claim. Subsequent batches compare against the frozen workload and declare
their allowed regression budget before implementation.

A performance batch passes only when:

1. normalized vanilla semantic differences remain zero for its declared population;
2. repeated measurements satisfy the batch's frozen latency, throughput, memory, and tick
   interference thresholds with the raw report retained;
3. all admission and work queues remain bounded and overload is observable;
4. speedup is not obtained by reducing required view/simulation distance, skipping stages,
   disabling content, pre-generating the measured online workload, or delaying durability;
5. deterministic publication and Region ownership/fencing remain intact;
6. a clean baseline and candidate use the same hardware allocation, workload, caches, and build
   class.

Thresholds are workload- and hardware-specific. Capacity reports state measured limits and never
convert them into universal player-count promises.

## Optimization order

Profile before changing architecture. Prefer changes in this order:

1. remove redundant decoding, allocation, hashing, copying, and generic mutation bookkeeping;
2. add generation-only bulk builders for sections, palettes, heightmaps, and immutable candidate
   columns while preserving all required semantic side effects at publication;
3. cache immutable lowered worldgen data and reusable samplers with explicit versioned keys and
   bounded lifetimes;
4. schedule the status dependency graph so independent chunks/stages run concurrently without
   scheduler-thread ping-pong or gameplay dependence on completion order;
5. isolate bounded generation, lighting, persistence, compression, and simulation pools where
   contention evidence justifies it;
6. stream the nearest playable chunks first while continuing the declared initial view in the
   background, without marking an incomplete state complete;
7. reduce authoritative-column-to-persistence and authoritative-column-to-projection copies while
   keeping immutable snapshots and revision fences.

Parallel execution may change wall-clock completion order. It must not change dependency order,
random-stream semantics, commit order, canonical state, or which activation generation may publish.

## External implementation study

The curated source register is
[Minecraft server performance implementation references](../reference/minecraft-java-26.2/performance-implementation-sources.md).
Those projects suggest candidate techniques and profiling questions; they are not normative
Minecraft behavior and their public throughput claims are not copied into Ferrite acceptance.
