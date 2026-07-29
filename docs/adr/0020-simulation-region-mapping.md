# ADR-0020: Make SimulationRegion Ownership Stable and Spatial

## Status

Accepted

## Context

Mutable world state needs a unit that can execute locally, move between nodes, recover independently,
and preserve cross-boundary semantics. Hashing every chunk or Region independently would destroy
locality; making one placement domain per Region would overload the control plane.

## Decision

Every chunk belongs to exactly one canonical `SimulationRegionKey`:

```text
(world_id, dimension_id, floor_div(chunk_x, region_side_chunks),
                         floor_div(chunk_z, region_side_chunks))
```

Division is Euclidean floor division, including negative coordinates. The initial world-creation
default is `region_side_chunks = 8`. The value is stored in world metadata, must be positive, and
cannot change online. A different size requires an explicit offline ownership migration.

A Lattice placement domain represents one world, dimension group, or explicitly bounded group of
worlds, never one Region. Ferrite installs a mapper identified as
`ferrite-spatial-region`, version `1`. It groups Region coordinates into persisted placement cells,
canonically encodes `(world, dimension, cell_x, cell_z)`, then computes
`xxh3_64_with_seed(bytes, 0x4645_5252_4954_4531) % shard_count`. Placement-cell span, shard count,
mapper identity, mapper version, encoding version, and fixed seed are persistent operational
compatibility fields.

The initial placement-cell span is configurable at world creation; it is not a capacity claim.
Automatic remapping is disabled until workload traces and handoff tests justify a versioned
migration. Allocation may use adjacency traffic, tick cost, players, loaded chunks, queue pressure,
memory, persistence locality, and failure domains.

Each active Region owns one Region-local ECS world, voxel/chunk state, queues, random streams,
boundary state, activation generation, and persistence revision. Neighbor mutation is always a
typed boundary transaction, even in the local runner.

## Consequences

- Local and distributed modes share an ownership model.
- Adjacent Regions can be colocated without changing chunk ownership.
- Region-size and mapper changes become explicit migrations.
- Hot Regions remain independently measurable and movable, while placement cells amortize locality.

## Alternatives Considered

- One actor per block/entity: rejected due to hot-path message and storage overhead.
- One actor per whole world: rejected because hotspots cannot scale across nodes.
- Default hash of the complete Region ID: rejected because it discards spatial locality.
- Dynamic Region boundaries: deferred because split/merge semantics and persistence migrations are
  not yet proven.

## Migration or Reversal Plan

Stop the affected world, snapshot all Regions, rewrite ownership metadata and mapper version,
validate complete chunk coverage with no overlap, then resume under a new recovery generation.
