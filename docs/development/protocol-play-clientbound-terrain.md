# Required Play Clientbound Terrain Protocol

Ferrite implements all ten packets in
`PROTO-PLAY-CLIENTBOUND-TERRAIN-001` for Minecraft Java 26.2:

| ID | Identity | Client projection |
|---:|---|---|
| 0 | `minecraft:bundle_delimiter` | open or close one bounded synthetic bundle |
| 11 | `minecraft:chunk_batch_finished` | finish timing and emit desired-batch feedback |
| 12 | `minecraft:chunk_batch_start` | replace the current batch start time |
| 13 | `minecraft:chunks_biomes` | refresh present biomes and notify every coordinate |
| 37 | `minecraft:forget_level_chunk` | remove chunk and all named light state |
| 45 | `minecraft:level_chunk_with_light` | install a dimension-sized chunk and queue light |
| 48 | `minecraft:light_update` | merge changed sky and block light layers |
| 94 | `minecraft:set_chunk_cache_center` | move the connection-local cache center |
| 95 | `minecraft:set_chunk_cache_radius` | derive the internal view radius |
| 111 | `minecraft:set_simulation_distance` | replace the displayed simulation distance |

Packet IDs, palette indices, raw registry IDs, cache coordinates, masks, batch timers, render dirt,
and readiness state remain inside the Java 26.2 adapter. Authoritative Region chunks continue to
use project-owned snapshots and registry identities.

## Bundles and batches

The delimiter is an individual frame. The first delimiter opens a bundle and the second releases
all held packets in order. An empty pair is valid. A terminal disconnect or a 4,097th subpacket
faults the connection-local assembler.

The chunk-batch calculator starts at 2,000,000 ns per chunk with old-sample weight one. A positive
finish sample is elapsed time divided by packet count, clamped to one third through three times the
aggregate, then combined with a weight capped at 49. Zero and negative batch counts do not update
the sample but still produce `7,000,000 / aggregate` as the ID-10
`chunk_batch_received` value. Repeated starts replace the timestamp; an unmatched finish uses the
timestamp installed at calculator construction.

## Chunk, palette, and block-entity projection

Full chunks decode the configured number of bottom-to-top sections from an isolated byte blob.
Trailing blob bytes are ignored. A negative heightmap count is an empty map, while a negative
block-entity count faults. Block palettes use single, local 4–8-bit, or global 15-bit storage;
biomes use single, local 1–3-bit, or dynamic-registry-width global storage. Fixed storage lengths,
palette indices, and raw registry ranges fail closed.

Heightmaps with the dimension-derived wrong long count enter the explicit recomputation set.
Full-chunk block-entity tags are nullable default-quota compound NBT. On installation, an update
is retained only when the block state at its packed local coordinate maps to the same block-entity
type. The conformance projection accepts the locked block-state/type mapping at initialization and
does not persist packed coordinates or raw types into Region state.

The cache radius follows Java arithmetic exactly: `max(2, requested) + 3`, including signed-int
overflow. Range subtraction and absolute value also use Java signed-int overflow. Projection
collections have explicit capacities and fail closed when an unseen entry would exceed them.

## Light, biome, and unload behavior

Each light channel has one configured layer per light-engine section. In-range data-mask bits
consume exact 2,048-byte arrays; data wins over an empty bit. Empty-only installs zero light and
absent bits leave the layer unchanged. Bits above the configured layer count and surplus arrays
are handler-ignored.

Full-chunk light is applied even when the chunk coordinate is outside the cache. Incremental light
merges independently and enables lighting. Installation and touched light mark the affected render
neighborhood dirty. Forget removes cached chunk data, both light channels, heightmap repair state,
and the lighting-enabled marker even when no cached chunk was present.

Biome records replace data only for an exact present chunk. Every listed coordinate still receives
the loaded notification and dirties the wrapping 3×3 chunk neighborhood.

## Terrain readiness

Client load starts in waiting-for-server with a deadline 30,000 ms later. That state never checks
the deadline. The load-start event carries the same deadline into waiting-for-player-chunk.
Readiness opens only after the strict `now > deadline` test, a compiled player section, spectator
or dead state, or a player/camera position outside build height.

The ready state sends fieldless ID-44 `player_loaded` after the configured close delay: zero for a
normal remote connection and 500 ms for the integrated-world path. Batch finish alone has no
readiness effect.

## Evidence

`crates/ferrite-protocol/tests/c2/play_clientbound_terrain.rs` owns all ten identity goldens,
palette and registry boundaries, malformed isolated blobs, NBT and block-entity matching, bundle
limits, batch timing, Java overflow, light-mask precedence and ignored data, biome/unload state,
bounded projection, and readiness transitions.
