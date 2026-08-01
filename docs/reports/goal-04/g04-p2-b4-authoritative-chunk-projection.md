# G04-P2-B4 — Authoritative chunk projection

## Outcome

The formal Minecraft gateway no longer owns or receives a `MinimalTerrain` provider. Play entry
sends only cache center, view distance, and simulation distance controls. Requested chunks remain in
the bounded session queue until the Region-owned world service reports the same authoritative
`ChunkColumn` as `FULL`, accessible, free of generation/unload work, and committed by the current
composite tick.

After that boundary, one immutable `ChunkSnapshot` supplies the Java 26.2 packet's complete section
palettes, biome cells, three client heightmaps, block entities, light layers, position, and source
revision. The gateway deduplicates requested positions across sessions and snapshots each ready
column once per tick. Flow-control preparation remains transactional: projection or registry
failure does not mark a chunk sent, and client feedback still controls later batches. Recenter
continues to emit `ForgetLevelChunk` only for chunks that were actually sent; ticket loss therefore
precedes authoritative save/unload without inventing an unload for unseen data.

## Registry and light boundary

Internal generator states map to air, stone, and grass-block raw IDs. With the exact client fixture,
those IDs are read and validated from the locked 26.2 registry report rather than assumed. Climate
biomes map through the connection's synchronized plains, snowy-plains, and forest registry IDs. The
compact bootstrap carries the same closed three-biome mapping for headless operation.

P3-B2 owns propagated sky and block light. Until then, snapshot construction uses a bounded full-sky
and empty-block-light layer for every generated section, including the two protocol boundary layers.
This is explicit projection state, not a second terrain provider. P3-B1 remains the next blocker
because formal movement still queries `FlatWorldCollision` even though visible terrain is now
generated.

## Verification

- World-service tests prove incomplete, dormant, generating, and unloading columns are not
  projectable, while an accessible `FULL` column yields matching revision and heightmaps.
- Formal lifecycle tests generate and commit a real column, promote it through the ticket lifecycle,
  and retrieve that exact revision through the composite gateway snapshot route.
- Existing chunk-stream tests retain bounded ordering, feedback, stale-preparation, registry,
  heightmap, block-entity, light, and unload coverage.
- Formal network-entry, malformed-client, backpressure, and protocol smoke regressions pass without
  a production `MinimalTerrain` dependency.
- Universal Rust, source-policy, production-manifest, and diff gates run before commit.
