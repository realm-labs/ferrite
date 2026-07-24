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
| Chunk demand, storage read and generation | `net.minecraft.server.level.ServerChunkCache#getChunk`, `net.minecraft.server.level.ServerChunkCache#getChunkFuture`, `net.minecraft.server.level.ChunkMap#scheduleChunkLoad`, `net.minecraft.server.level.ChunkMap#scheduleGenerationTask` | `WGEN-PIPELINE-001` owns status-by-status generation and locked data; PersistenceReload owns serialized reconstruction; `BLK-AIR-001` fixes void-air bounds, ordinary-air empty-section reads, exact mixed palettes and cave-air generation sources; `BLK-BEDROCK-001` fixes exact bedrock identity across surface/flat/End generation, feature protection and below-zero retrogen; `BLK-REINFORCED-DEEPSLATE-001` fixes ancient-city state selection and feature-protected target identity; `BLK-DEEPSLATE-001` fixes axis-Y surface/flat/retrogen identity, ore/support/replacement joins and reachable versus raw ancient-city payloads; `BLK-DEEPSLATE-MASONRY-001` fixes seven full-block masonry identities across sculk, 44,739 raw structure cells and start/generic/trial processor order; `BLK-DRIPSTONE-BLOCK-001` fixes state 30208 across pointed patches, clusters and hard-coded large columns plus zero structure-template cells; `BLK-BRICKS-001` fixes state 2340 across 2,558 raw trail-ruins, cold-ocean-ruin and plains-village template cells while retaining named processor/write owners; `BLK-SLIME-001` and `BLK-HONEY-001` fix the explicitly gated below-sea/at-or-above-sea `DEBUG_AQUIFERS` diagnostic stripe during noise fill; `BLK-NETHER-WART-BLOCK-001` fixes crimson surface, ordinary/planted fungus-hat, weeping-vines producer/support and Nether-carver identity roles plus zero structure-template cells; `BLK-WARPED-WART-BLOCK-001` fixes warped surface, ordinary/planted fungus-hat, twisting-vines support and Nether-carver roles plus zero structure-template cells; `BLK-NETHER-SPROUTS-001` fixes natural and bonemeal warped-vegetation state selection, tree/mushroom replacement and zero structure-template cells; `BLK-NETHER-ROOTS-001` fixes weighted crimson/warped vegetation, the crimson-only soul-sand-valley patch, tree/mushroom replacement and zero root/pot structure cells; `BLK-NETHER-WART-001` fixes 20 age-zero fortress-room writes and 12 age-three cells across three bastion center templates; `BLK-NETHER-STEM-001` fixes unstripped Y-axis crimson/warped stem writes across four huge-fungus records plus zero structure-template cells; `BLK-CORAL-BLOCK-001` fixes uniform five-live-state selection by the warm-ocean tree/claw/mushroom family, excludes all dead states and fixes zero template cells for all ten identities; `BLK-SOUL-SAND-001` fixes normal Nether surface, ore, spring, carver, fortress, fossil and basalt-column identity roles plus generation-region postprocess-above; `BLK-MAGMA-001` fixes ore, underwater, delta, ruined-portal, basalt-column, Nether-spring and bastion-processor identities plus the same postprocess callback; `BLK-STRUCTURE-001` separates the loaded block entity from manager-owned named templates, `BLK-STRUCTURE-VOID-001` separates omitted captured coordinates and jigsaw skip sentinels from raw template writes, `ITM-CHEST-001` fixes pairing/list migration, `ITM-HOPPER-001` fixes saved cooldown plus reconstructed facing/tick time, and `ITM-DISPENSER-001` fixes inventory/block-state reconstruction while keeping pending scheduled dispatch a separate lifecycle concern when their chunks become live. | Enumerate synchronous/async and create/no-create calls, loaded-versus-generated selection, neighbor status dependencies, task priority/cancellation, read/data-fix failure and caller-visible completion. |
| Tickets, levels and activity promotion | `net.minecraft.server.level.ServerChunkCache#addTicket`, `net.minecraft.server.level.ServerChunkCache#addTicketAndLoadWithRadius`, `net.minecraft.server.level.ServerChunkCache#addTicketWithRadius`, `net.minecraft.server.level.ServerChunkCache#removeTicketWithRadius`, `net.minecraft.server.level.DistanceManager#runAllUpdates`, `net.minecraft.server.level.ChunkHolder#updateFutures` | `SIM-005`, `SIM-RANDOM-001` and the locked `ticket_type` catalog own activity predicates, expiry and ticking eligibility; `BLK-TEST-INSTANCE-001` owns the permanent `setChunkForced` calls made before each template placement. | Map every ticket type and radius to ticket level and `FullChunkStatus`; audit propagation/update order, replacement and expiry, player/view/simulation distance changes, other forced-chunk callers and demotion races. |
| Accessible, ticking and entity-ticking publication | `net.minecraft.server.level.ChunkMap#prepareAccessibleChunk`, `net.minecraft.server.level.ChunkMap#prepareTickingChunk`, `net.minecraft.server.level.ChunkMap#prepareEntityTickingChunk`, `net.minecraft.server.level.ServerChunkCache#onChunkReadyToSend`, `net.minecraft.server.level.ChunkMap#forEachReadyToSendChunk` | Simulation owners define work admitted at each activity level; client projection and terrain protocol families own the resulting chunk view. | Recover promotion callback order, `LevelChunk` replacement/publication, post-load hooks, block-entity/entity insertion, POI/light readiness, watch-set changes and first terrain/entity packets. |
| Save, demotion and unload | `net.minecraft.server.level.ChunkMap#saveAllChunks`, `net.minecraft.server.level.ChunkMap#processUnloads`, `net.minecraft.server.level.ChunkMap#scheduleUnload`, `net.minecraft.server.level.ServerLevel#unload`, `net.minecraft.server.level.ServerChunkCache#save` | PersistenceReload owns field continuity and write results; entity, block-entity, scheduled-tick and POI owners define live state being removed. | Audit dirty/save admission, pending future and unload cancellation, demotion hooks, tracking removal, entity/block-entity/POI teardown, callback queues, write failure and last-observer ordering. |
| Dimension travel, portals and world border | `net.minecraft.server.level.ServerPlayer#teleport`, `net.minecraft.world.level.portal.PortalForcer#findClosestPortalPosition`, `net.minecraft.world.level.portal.PortalForcer#createPortal`, `net.minecraft.world.level.border.WorldBorder#setCenter`, `net.minecraft.world.level.border.WorldBorder#setSize`, `net.minecraft.world.level.border.WorldBorder#lerpSizeBetween`, `net.minecraft.world.level.border.WorldBorder#tick` | `WGEN-PORTAL-001` owns travel/search/creation/safe placement; `WGEN-BORDER-001` owns border geometry and interpolation; PlayerLifecycle owns player replacement and session phases. | Join source/destination ticketing and entity removal/addition to portal/border results; audit failed admission, passenger chains, concurrent unload, shared border settings and exact client convergence order. |
| Save-all and clean shutdown | `net.minecraft.server.MinecraftServer#saveEverything`, `net.minecraft.server.MinecraftServer#stopServer`, `net.minecraft.server.MinecraftServer#close`, `net.minecraft.server.level.ServerChunkCache#deactivateTicketsOnClosing`, `net.minecraft.server.level.ServerChunkCache#close`, `net.minecraft.server.level.ServerLevel#close`, `net.minecraft.server.WorldStem#close` | `SIM-006` owns pause/autosave admission; PersistenceReload owns durable continuity and first restarted observation. | Recover player/level/resource/executor close order, ticket deactivation, outstanding generation/save joins, skip-save and failure branches, disconnect projection, idempotence and partially initialized shutdown. |

## Current boundary conclusions

- `BLK-PACKED-MUD-001` fixes state 7758 across 68 raw trail-ruins cells and the houses/roads
  below-`0.1` mud-bricks aging output. The trail-ruins owner retains selection, position-derived
  RNG, transform, clipping and final-write admission across chunk lifecycle boundaries.
- `BLK-MUD-BRICKS-001` fixes state 7759 across 3,870 raw trail cells, 19 connector final states
  and the houses/roads aging-survivor path. The trail owner retains replacement, position-derived
  RNG, transform, clipping and final-write admission across chunk lifecycle boundaries.
- `BLK-PURPUR-BLOCK-001` fixes state 14712 across 2,212 cells in 19 reachable End-city inputs and
  distinguishes 21 raw cells in dead `tower_floor`. The End-city owner retains graph selection,
  transform, overwrite, clipping and final-write admission.
- `BLK-NETHER-WART-BLOCK-001` fixes state 14846 across the crimson surface tree, both crimson
  fungus configurations and weeping-vines roof/support paths, plus wart-tag Nether-carver
  replacement and zero structure-template cells. The pipeline owner retains status order,
  selection, RNG, clipping and final-write admission.
- `BLK-WARPED-WART-BLOCK-001` fixes state 20959 across the warped surface tree and both warped
  fungus configurations, plus exact twisting-vines support, wart-tag Nether-carver replacement and
  zero structure cells. The pipeline owner retains status order, RNG and final-write admission.
- `BLK-NETHER-SPROUTS-001` fixes state 20961 as the fixed provider for natural 8/4 and bonemeal
  3/1 Nether-forest-vegetation records, the warped-biome layer-count-4 placement and the
  warped-nylium bonemeal call between vegetation and twisting-vines selection. Reloadable tags also
  admit tree/mushroom overwrite; the pipeline owner retains attempts, RNG and final writes.
- `BLK-NETHER-ROOTS-001` fixes states 20960/21031 in natural and bonemeal crimson/warped weighted
  vegetation, layer counts 6/5 and nylium call order, plus the crimson-only count-96
  soul-sand-valley patch. Reloadable tags admit tree/mushroom overwrite; the pipeline owner retains
  provider/modifier traversal, RNG and final writes.
- `BLK-NETHER-WART-001` fixes the CastleStalkRoom's two 10-cell default-age crop beds and 12
  mature cells across three equal bastion center templates. Fortress and jigsaw owners retain
  piece/pool reachability, orientation, clipping, processor traversal and final writes.
- `BLK-NETHER-STEM-001` fixes the default-Y crimson or warped stem state selected by all four
  ordinary/planted huge-fungus records, including natural huge-form geometry. The pipeline owner
  retains feature reachability, exact replacement/write order and RNG; all eight identities have
  zero bundled structure-template cells.
- `BLK-CORAL-BLOCK-001` fixes the warm-ocean selector's tree/claw/mushroom order, uniform
  per-invocation selection from five live coral-block states and exclusion of dead states. The
  pipeline owner retains feature reachability, strict water admission, traversal, decoration RNG
  and writes; all ten identities have zero bundled structure-template cells. Once live, ordinary
  chunk scheduling/persistence owns any later dry-conversion tick and its revalidation.
- `BLK-CORAL-PLANT-001` fixes the decorator's five live upright plus five live floor-fan top
  candidates, five live wall-fan horizontal candidates and exclusion of all fifteen dead forms.
  The pipeline retains admitted-cell and `<0.25`/`<0.2` draws, fixed north/east/south/west visits,
  exact-water admission and writes; all thirty identities have zero bundled template cells. Once
  live, support updates and separately persisted dry ticks own later removal or terminal conversion.
- `BLK-FLOWER-POT-001` fixes 120 raw cells across 55 bundled village, mansion, trial-chamber and
  igloo templates, with exact per-identity counts and 24 zero-cell forms. Pool selection,
  processors, transforms, clipping and write admission remain with the structure pipeline. Once
  live, pot interaction, potted-eyeblossom environment ticks and hoglin sensing are independent
  state consumers rather than generation continuation.
- `BLK-COPPER-FULL-001` fixes 23,354 raw cells and 404 identity/template pairs across 149 bundled
  trial-chamber templates, with exact counts for eight identities and sixteen zero-cell forms.
  Pool selection, processors, transforms, clipping and write admission remain with the structure
  pipeline. Once live, generic palette persistence retains age/wax identity while random weathering,
  honeycomb/axe use and pumpkin-driven golem/chest creation are independent consumers.
- `BLK-SAPLING-001` fixes two acacia stage-one cells in
  `village/savanna/houses/savanna_library_1` and 58 dark-oak stage-zero cells in
  `woodland_mansion/1x2_a4`; the other six identities have zero cells. Its 45 placed-feature
  references are stage-zero `would_survive` predicates rather than sapling outputs, while four
  huge-fungus configurations admit all eight as replaceable. Structure/feature selection,
  processors, transforms, clipping and final writes remain with the worldgen pipeline.
- `BLK-BAMBOO-001` fixes zero raw sapling/stalk cells across all 1,212 templates. Jungle selects
  the zero-podzol bamboo configuration through a one-in-four rarity path; bamboo jungle selects
  the 0.2-podzol form through noise count. `BambooFeature` owns its 5..16-segment write and podzol
  geometry, while modifier admission, generation-region writes and later live support/growth
  remain with their generic owners.
- `BLK-ANCIENT-DEBRIS-001` fixes zero raw cells across all 1,212 templates and no direct processor
  reference. Every Nether biome lists the large absolute-Y 8..24 path before the small
  above-bottom/below-top 8 path in underground ores. `ScatteredOreFeature` owns the size-3/2
  attempt, six-float offset, base-stone target, six-neighbor exposure and ignored flags-2 write;
  placement seeding, chunk publication and later persistence remain with generic owners.
- `BLK-STEM-CROP-001` fixes 111 raw cells across the template corpus: 32 attached cells in one
  mansion, 62 melon-stem cells in six ordinary/zombie savanna streets and 17 age-seven
  pumpkin-stem cells in one taiga farm. Six ordinary/zombie farm processors replace wheat with
  age-zero stems at exact ordered probabilities, while four huge-fungus configurations admit all
  forms as replaceable. Pool choice, processor RNG/order, feature writes, chunk publication and
  later live support/growth remain with generic owners.
- `BLK-OVERWORLD-CROP-001` fixes 722 raw wheat cells across 29 templates: 72 desert, 93 plains,
  423 savanna, 37 snowy, 65 taiga and 32 mansion cells. Ten ordinary/zombie village farm
  processors replace wheat with age-zero crop states at ordered probabilities, while four
  huge-fungus configurations admit all four crops as replaceable. Generic template, processor and
  feature owners still control commit, transforms and clipping.

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
