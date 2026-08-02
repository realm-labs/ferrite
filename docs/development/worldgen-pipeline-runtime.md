# Minecraft 26.2 Worldgen Pipeline Runtime

Ferrite's `WGEN-001` implementation lives under
`ferrite-world::generation`. It separates scheduling and write admission, biome selection, noise
and density evaluation, chunk material fill, surface replacement, carvers, configured features,
trees, upgrade blending, and locked data projection. The separation keeps the algorithms
independently testable while the status task remains the owner of execution order.

This document records the current audited pipeline surface. The target production algorithm,
generation-only builder, Chunk Status DAG, shared execution pools, player-centered priority, Region
commit boundary, and distributed task policy are defined by the
[worldgen execution architecture](worldgen-execution-architecture.md).

## Deterministic boundaries

- Chunk status dependencies and write radii are explicit. A failed task does not publish its target
  status, and feature writes alone may use the one-chunk write radius.
- Biome fill, noise fill, surface, carvers, and features retain the source encounter order. Random
  streams are passed through the owning algorithm rather than reconstructed from execution timing.
- Density composition, normal and legacy noise, aquifers, ore veins, runtime caches, interpolation,
  upgrade blending, structure beardification, and base-column queries are distinct production
  modules with explicit lifecycle APIs.
- The 7,594-point Overworld preset is constructed from the locked partition tables. Its emitted
  points are compared, after Minecraft climate quantization, against the optional local official
  report. Nether and all seven world-preset selectors are likewise projected without committing
  official payload files.
- The imported content bundle remains the authoritative data boundary. `WorldgenCatalog` exposes
  the 681 records owned by this batch, and strict decoders project behavior-bearing noise settings
  and world presets. Data-only density, noise, feature, biome, and placement records remain named
  graph inputs to their implemented algorithms.

## Region and mutation boundary

Generation algorithms operate through narrow world traits. Callers supply Region-owned reads,
bounded write offers, heightmap updates, post-processing marks, section locks, block-entity
creation, and lifecycle hooks. Algorithms cannot acquire mutable world authority or write through a
foreign Region directly.

The material pipeline is aquifer, then ore vein, then configured default block. Nonair writes update
both worldgen heightmaps; a returned fluid is marked for post-processing only when the immediately
preceding aquifer result requests it. Surface and carver restoration reuse the audited surface-rule
evaluator rather than maintaining a second rule implementation.

## Vanilla exactness boundary

The current implementation proves source-specified gates, ordering dependencies, bounds, codec
projections, and Ferrite replay, but that is only partial evidence under the production contract.
Completion additionally requires same-input semantic identity against the locked official 26.2
server. `EXP-WGEN-001`, `EXP-WGEN-005`, and `EXP-WGEN-006` remain useful coverage and diagnostic
plans; statistical thresholds cannot replace the required zero-unexplained-divergence differential
suite.
