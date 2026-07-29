# Region-Owned State

`G01-P2-B1` establishes the first mutable world boundary. Authoritative voxel and entity state is
created inside one `SimulationRegionKey`; no mutable whole-world container exists.

## Voxel storage

`ferrite-world` stores chunk sections with typed process-local `BlockStateId` and `BiomeId` values.
The generic palette container has three representations:

1. a single value;
2. an insertion-ordered local palette with packed indices;
3. packed direct values after the 256-entry local limit.

Bit storage supports entries crossing machine-word boundaries and expands without losing values.
Palette representation is runtime state, not a persistence schema. Persistent codecs must lower
through content identities and a registry snapshot in the persistence batch.

Chunk columns use an explicit checked vertical section range and allocate sections sparsely. Section
and chunk revisions advance only for actual changes. Reads from absent sections in a loaded chunk
return the configured default block. Reads and writes to an unloaded chunk fail; ordinary block
writes never load or generate a chunk implicitly.

`RegionVoxelState` validates every admitted chunk against the versioned Euclidean Region mapping.
Cross-Region writes fail before chunk allocation or mutation. Its `RegionVoxelView` exposes only
immutable, deterministically ordered chunk and block queries.

## Entity storage

`ferrite-simulation` owns one private `bevy_ecs::World` per Region partition. Callers address
entities only with `StableEntityId`; Bevy `Entity` handles never cross the module API.

Every spawned entity receives immutable stable-identity, world/dimension, and Region-membership
components. Generic component insertion cannot replace those ownership components, and mutation APIs
accept only Bevy components declared mutable. Stable-ID iteration uses a sorted map so observable
query order does not depend on archetype layout or hash iteration.

`RegionSimulationState` combines exactly one voxel partition and one entity partition. Its
`RegionSimulationView` is the read-only boundary consumed by later gameplay, replay, protocol
projection, and persistence work. Tick admission, structural-command barriers, boundary messages,
transfers, snapshots, and recovery are intentionally owned by later Phase 2 batches.
