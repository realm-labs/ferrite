# Chunk Join and Terrain Projection

`G01-P4-B1` establishes the protocol-neutral chunk lifecycle used by a Java 26.2 Play session.
It deliberately separates Region-owned world state, per-session interest and flow control, and the
versioned Java wire projection.

## Ownership boundary

A successful session admission now carries the selected spawn chunk and the client's requested
view distance. The connection owner uses that immutable admission to create a
`ClientChunkSession`; it does not put Java packet types into a Region.

The three layers are:

1. `ferrite-world` produces an immutable `ChunkSnapshot` from authoritative sections, ordered
   block entities, typed heightmaps, and exact light layers.
2. `ferrite-server-runtime::chunk` owns session-local tickets, visible interest, pending/sent
   state, deterministic ordering, and bounded batch flow control.
3. `ferrite-protocol::java_26_2` maps a stream event into the exact Java 26.2 packet identity and
   wire representation. `ServerConnection::enqueue_play` pre-encodes a complete packet group and
   verifies queue and sequence capacity before mutating the outbound queue.

Runtime block-state, biome, and block-entity identities cross the adapter through an explicit
connection-independent `JavaTerrainRegistryMap`. Missing, duplicate, remapped, out-of-range, or
over-capacity entries fail closed. Dense Java IDs never enter authoritative world state.

## Tickets and interest

The ticket book is bounded and keyed by chunk plus source. Replacing all tickets for one source
preflights source consistency, duplicate positions, arithmetic, and final capacity before changing
the book. The strongest numerical ticket level selects four separate activation results:

| Level | Loaded | Client-visible | Block ticking | Entity ticking |
|---:|:---:|:---:|:---:|:---:|
| `<= 31` | yes | yes | yes | yes |
| `32` | yes | yes | yes | no |
| `33` | yes | yes | no | no |
| `> 33` | yes | no | no | no |
| absent | no | no | no | no |

Player view tickets use level 33. A separate center simulation ticket carries the simulation
distance's propagation level. Portal, forced, generation, pending-save, scheduled-block, and
administrative sources are represented without coupling their lifecycle to the player source.
Transient tickets use an explicit `GameTick` expiration and expire at the recorded tick.

The effective client view clamps the signed client request into `2..=server_view_distance`; the
server value itself must be in `2..=32`. The square view is precomputed with checked coordinates
and must fit its configured bound. Recenter is failure-atomic: both the next interest and the
replacement ticket set succeed before the live session changes.

Known chunks have distinct `Pending` and `Sent { revision }` states. Leaving chunks that were only
pending are removed silently. A live session emits `forget_level_chunk` only for chunks previously
sent to that client, with the new center event ordered first.

## Bounded streaming

Initial controls are ordered as cache center, cache radius, and simulation distance. Ready chunks
are selected by squared distance from the center, then X and Z, so ties are stable across runs and
topologies. A nonempty delivery is:

1. `chunk_batch_start`;
2. at most the configured number of full chunk snapshots;
3. `chunk_batch_finished` with the exact count.

The initial sender target is 9 chunks per tick and permits one unacknowledged batch. Client
feedback lowers the count to `0.01` for NaN, otherwise clamps it to `0.01..=64`, restores one unit
of quota when the pipeline drains, and raises the unacknowledged bound to 10. No ready snapshot
means no empty batch.

Batch state is transactional. Preparing a batch operates on a cloned stream generation and leaves
live chunks pending. The connection owner projects and enqueues those events, then commits the
prepared generation. Projection or queue failure can discard it without marking chunks sent; any
intervening recenter, readiness, feedback, or earlier commit makes the token stale and prevents a
late commit.

The Java client projection models its own storage rule. A cache-radius packet uses the client's
wrapping `max(2, radius) + 3` calculation. Full chunks outside the current center and valid storage
radius are not installed; changing center or radius removes no-longer-visible projected chunks.

## Terrain payload

`MinimalTerrain` provides deterministic flat terrain until the audited world-generation pipeline
is implemented. Its surface must end at a section boundary, avoiding a special partial-section
case while retaining exact dimension section counts. The snapshot contains:

- every vertical section, including uniform air sections;
- 4,096 block states and 64 biomes per section;
- independently classified `WORLD_SURFACE`, `MOTION_BLOCKING`, and
  `MOTION_BLOCKING_NO_LEAVES` heightmaps;
- unique block entities ordered by global block position;
- exactly `section_count + 2` sky and block light layers.

Heightmap classification is supplied by the authoritative registry boundary rather than guessed
from a raw ID. The minimal terrain classifier treats its single solid state as included in all
three client heightmaps; later content batches can provide the exact per-heightmap predicates.

The Java adapter implements canonical single, indirect, and direct palettes without values
straddling 64-bit storage words. It validates block-state IDs `0..=32365`, biome registry bounds,
block-entity type IDs `0..=48`, heightmap packing, section blobs, light masks, 2,048-byte light
arrays, and trailing input. Biome refresh only updates an installed chunk with the exact expected
section count.

Respawn reuses the common spawn codec. Retention bit `0x01` independently keeps attributes and bit
`0x02` independently keeps entity data; unknown high bits remain wire data and do not acquire
semantic meaning.

## Current composition point

This batch stops at a complete, tested join-to-wire projection boundary:

`PlayAdmission -> ClientChunkSession -> prepared ChunkStreamEvent -> PlayClientboundPacket -> ServerConnection -> commit`

The node process does not yet claim an end-user playable socket. Player movement and explicit
Region transfer are `G01-P4-B2`; block interaction is `G01-P4-B3`; topology and unmodified-client
C2 acceptance remain `G01-P4-B4` and `G01-P4-B5`.
