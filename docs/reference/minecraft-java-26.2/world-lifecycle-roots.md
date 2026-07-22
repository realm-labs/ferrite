# World Lifecycle Root Inventory

**Surface:** `SURFACE-WORLD-LIFECYCLE-001`
**Status:** `InProgress`
**Primary evidence:** `OFF-SERVER-001`

This inventory owns the transitions that create a live level, advance a chunk from demand through
generation and activity, remove it from the live world, and close the server. The
[persistence inventory](persistence-reload-roots.md) owns which values survive those boundaries;
this surface owns when each boundary occurs and what can observe it.

| Lifecycle family | Locked source roots | Existing semantic owners | Remaining audit |
|---|---|---|---|
| World input and registry bootstrap | `net.minecraft.server.WorldLoader#load`, `net.minecraft.server.MinecraftServer#loadLevel`, `net.minecraft.server.MinecraftServer#createLevels` | DataReload owns pack and registry snapshot construction; `WGEN-002`, `WGEN-003` and `WGEN-DIMENSION-001` own decoded worldgen and dimension behavior. | Trace registry-layer handoff, world-data/default/error branches, dimension iteration order, shared versus per-level state and cleanup after partial construction. |
| Level creation, spawn and initial readiness | `net.minecraft.server.MinecraftServer#createLevels`, `net.minecraft.server.MinecraftServer#setInitialSpawn`, `net.minecraft.server.MinecraftServer#prepareLevels`, `net.minecraft.server.level.ServerLevel#startTickingChunk` | `WGEN-DIMENSION-001` owns dimension properties and spawn semantics; the simulation pipeline owns admitted ticks after readiness. | Audit initial-spawn search and bonus-chest gates, forced spawn tickets, progress thresholds, initial chunk/entity activation, first autosave/client admission and failure rollback. |
| Chunk demand, storage read and generation | `net.minecraft.server.level.ServerChunkCache#getChunk`, `net.minecraft.server.level.ServerChunkCache#getChunkFuture`, `net.minecraft.server.level.ChunkMap#scheduleChunkLoad`, `net.minecraft.server.level.ChunkMap#scheduleGenerationTask` | `WGEN-PIPELINE-001` owns status-by-status generation and locked data; PersistenceReload owns serialized reconstruction; `BLK-STRUCTURE-001` separates the loaded block entity from manager-owned named templates, `BLK-STRUCTURE-VOID-001` separates omitted captured coordinates and jigsaw skip sentinels from raw template writes, `ITM-CHEST-001` fixes pairing/list migration, `ITM-HOPPER-001` fixes saved cooldown plus reconstructed facing/tick time, and `ITM-DISPENSER-001` fixes inventory/block-state reconstruction while keeping pending scheduled dispatch a separate lifecycle concern when their chunks become live. | Enumerate synchronous/async and create/no-create calls, loaded-versus-generated selection, neighbor status dependencies, task priority/cancellation, read/data-fix failure and caller-visible completion. |
| Tickets, levels and activity promotion | `net.minecraft.server.level.ServerChunkCache#addTicket`, `net.minecraft.server.level.ServerChunkCache#addTicketAndLoadWithRadius`, `net.minecraft.server.level.ServerChunkCache#addTicketWithRadius`, `net.minecraft.server.level.ServerChunkCache#removeTicketWithRadius`, `net.minecraft.server.level.DistanceManager#runAllUpdates`, `net.minecraft.server.level.ChunkHolder#updateFutures` | `SIM-005`, `SIM-RANDOM-001` and the locked `ticket_type` catalog own activity predicates, expiry and ticking eligibility; `BLK-TEST-INSTANCE-001` owns the permanent `setChunkForced` calls made before each template placement. | Map every ticket type and radius to ticket level and `FullChunkStatus`; audit propagation/update order, replacement and expiry, player/view/simulation distance changes, other forced-chunk callers and demotion races. |
| Accessible, ticking and entity-ticking publication | `net.minecraft.server.level.ChunkMap#prepareAccessibleChunk`, `net.minecraft.server.level.ChunkMap#prepareTickingChunk`, `net.minecraft.server.level.ChunkMap#prepareEntityTickingChunk`, `net.minecraft.server.level.ServerChunkCache#onChunkReadyToSend`, `net.minecraft.server.level.ChunkMap#forEachReadyToSendChunk` | Simulation owners define work admitted at each activity level; client projection and terrain protocol families own the resulting chunk view. | Recover promotion callback order, `LevelChunk` replacement/publication, post-load hooks, block-entity/entity insertion, POI/light readiness, watch-set changes and first terrain/entity packets. |
| Save, demotion and unload | `net.minecraft.server.level.ChunkMap#saveAllChunks`, `net.minecraft.server.level.ChunkMap#processUnloads`, `net.minecraft.server.level.ChunkMap#scheduleUnload`, `net.minecraft.server.level.ServerLevel#unload`, `net.minecraft.server.level.ServerChunkCache#save` | PersistenceReload owns field continuity and write results; entity, block-entity, scheduled-tick and POI owners define live state being removed. | Audit dirty/save admission, pending future and unload cancellation, demotion hooks, tracking removal, entity/block-entity/POI teardown, callback queues, write failure and last-observer ordering. |
| Dimension travel, portals and world border | `net.minecraft.server.level.ServerPlayer#teleport`, `net.minecraft.world.level.portal.PortalForcer#findClosestPortalPosition`, `net.minecraft.world.level.portal.PortalForcer#createPortal`, `net.minecraft.world.level.border.WorldBorder#setCenter`, `net.minecraft.world.level.border.WorldBorder#setSize`, `net.minecraft.world.level.border.WorldBorder#lerpSizeBetween`, `net.minecraft.world.level.border.WorldBorder#tick` | `WGEN-PORTAL-001` owns travel/search/creation/safe placement; `WGEN-BORDER-001` owns border geometry and interpolation; PlayerLifecycle owns player replacement and session phases. | Join source/destination ticketing and entity removal/addition to portal/border results; audit failed admission, passenger chains, concurrent unload, shared border settings and exact client convergence order. |
| Save-all and clean shutdown | `net.minecraft.server.MinecraftServer#saveEverything`, `net.minecraft.server.MinecraftServer#stopServer`, `net.minecraft.server.MinecraftServer#close`, `net.minecraft.server.level.ServerChunkCache#deactivateTicketsOnClosing`, `net.minecraft.server.level.ServerChunkCache#close`, `net.minecraft.server.level.ServerLevel#close`, `net.minecraft.server.WorldStem#close` | `SIM-006` owns pause/autosave admission; PersistenceReload owns durable continuity and first restarted observation. | Recover player/level/resource/executor close order, ticket deactivation, outstanding generation/save joins, skip-save and failure branches, disconnect projection, idempotence and partially initialized shutdown. |

## Current boundary conclusions

- Chunk status generation, full-chunk accessibility, block ticking and entity ticking are distinct
  transitions. A single loaded/unloaded boolean cannot reproduce their observable admission gates.
- Ticket state is transient scheduling authority even when its cause is persistent. Reconstruction
  may differ internally only if promotion, demotion, expiry and first admitted work stay equivalent.
- A chunk can finish data generation before it is ready for world callbacks or client terrain
  projection. Publication and tracking therefore remain separate audit families.
- Clean shutdown has explicit ticket deactivation and live level/chunk/resource close roots. Their
  exact cross-owner ordering and partial-failure behavior remain open until traced.

## Recovery procedure

1. Follow one demanded chunk through absent, storage-read, every generation status, accessible,
   block-ticking, entity-ticking, demoted, saved and unloaded states; record every future, queue and
   callback edge.
2. Repeat with each locked ticket type, expiry/replacement, view and simulation-distance changes,
   forced chunks, load failure and cancellation while promotion or unload is pending.
3. At every transition, enumerate entity, block-entity, scheduled-tick, POI/light, tracking and
   terrain observers and join them to their semantic and protocol owners.
4. Trace normal startup/shutdown plus partial level construction, failed read/write, skip-save and
   outstanding generation; compare the first restarted observation through PersistenceReload.
5. Promote this surface only after every transition and cross-domain ordering edge has an
   executable vector. The generation pipeline alone is not lifecycle completion.
