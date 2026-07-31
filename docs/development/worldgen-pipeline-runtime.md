# Minecraft 26.2 Worldgen Pipeline Runtime

Ferrite's `WGEN-001` implementation lives under
`ferrite-world::generation`. It separates scheduling and write admission, biome selection, noise
and density evaluation, chunk material fill, surface replacement, carvers, configured features,
trees, upgrade blending, and locked data projection. The separation keeps the algorithms
independently testable while the status task remains the owner of execution order.

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

## Equivalence boundary

The implementation claims the source-specified distributions, gates, ordering dependencies,
bounds, codec projections, and deterministic behavior. It does not claim block-for-block
same-seed identity with the official server. The unresolved calibration and population-equivalence
observations remain `DeferredExperiment` under `EXP-WGEN-001`, `EXP-WGEN-005`, and
`EXP-WGEN-006`.
