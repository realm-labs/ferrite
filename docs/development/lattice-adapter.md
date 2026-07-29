# Pinned Lattice Adapter

`G01-P2-B5` admits the reviewed Lattice substrate only through `ferrite-region-runtime`. The three
direct Git dependencies are `lattice-core`, `lattice-placement`, and `lattice-remoting`, all pinned
to revision `a52c54004c782bd18b70d37d929d54cd7d8205f3`.

The architecture verifier reads `docs/adr/lattice.lock.toml` and rejects a missing crate, a different
repository or revision, a branch/tag/version selector, or admission outside
`ferrite-region-runtime`. `Cargo.lock` therefore resolves the reviewed revision rather than the
current upstream branch.

## Spatial placement

`SpatialPlacementAdapter` groups canonical `SimulationRegionKey` coordinates into persisted placement
cells. Cell coordinates use Euclidean division by the configured positive cell span, including
negative Regions.

The Lattice entity ID is a versioned canonical byte sequence containing the world ID, persistent
dimension ID, placement-cell coordinates, and Region mapping version. It is bounded by Lattice's
entity-ID contract. The custom `ShardMapper` has these compatibility fields:

- mapper ID: `ferrite-spatial-region`;
- mapper version: `1`;
- encoding version: `1`;
- XXH3 seed: `0x4645_5252_4954_4531`;
- mapping: `xxh3_64_with_seed(entity_id, seed) % shard_count`.

The mapper identity/version participates in Lattice's `EntityConfig` fingerprint. Region size,
placement-cell span, shard count, domain, protocol ID, and mapping versions must therefore be treated
as persisted world/operations metadata, not live tuning knobs.

One Lattice placement slot owns a shard of placement cells, not one actor or slot per Minecraft
Region. Ferrite Region ownership remains the finer gameplay partition.

## Claims and admission

`RegionAuthorityAdapter` translates project-owned node, placement observation, claim, generation, and
action types into Lattice's `PlacementAuthority`. Lattice slot keys, grants, terms, revisions, and
node types stay private to the adapter.

Admission requires both:

1. the requested Ferrite `ActivationGeneration` exactly matches the reconciled assignment
   generation;
2. Lattice authority reports the grant open at the supplied monotonic time.

The latter is checked directly against the installed grant deadline after subtracting a nonzero
safety margin. A frozen process cannot serve once its deadline passes merely because it has not run a
periodic fencing tick. Claim loss maps immediately to Ferrite fence/stop actions.

## Handoff

A graceful move first reconciles Lattice's `BeginHandoff` state. `prepare_handoff` then requires the
durable recovery point to match the active Ferrite generation and requires a strictly newer target
generation. Lattice authority must emit both fence-admission and drain actions before any payload is
returned.

The moved payload is the bounded encoded `RegionRecoveryPoint` plus its digest and endpoint
generations. The target decodes and validates the recovery point, Region identity, source generation,
digest, and target generation before receiving a `RecoveredRegion`. Lattice actor memory is never
treated as Ferrite world state.

## Remoting

`LatticeRemotingAdapter` carries a versioned Ferrite Region envelope inside a bounded Lattice
`EntityTell` frame. The envelope contains semantic message kind, tick, source/target Region keys,
both endpoint generations, source sequence, and bounded payload. Decode rejects wrong frame kinds,
schemas, identities, endpoints, sizes, truncation, and trailing bytes.

The Lattice `Frame` remains a private adapter detail. Simulation, gameplay, persistence, replay, and
server APIs continue to exchange Ferrite semantic types.

`G01-P2-B6` will add the operational multi-node process/configuration contract and endpoint lifecycle.
`G01-P2-B7` will exercise local, in-process, and multi-process topology equivalence plus claim and
transport faults.
