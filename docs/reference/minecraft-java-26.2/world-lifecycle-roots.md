# World Lifecycle Root Inventory

**Surface:** `SURFACE-WORLD-LIFECYCLE-001`
**Status:** `Mapped`
**Primary evidence:** `OFF-SERVER-001`

This inventory owns the transitions that create a live level, advance a chunk from demand through
generation and activity, remove it from the live world, and close the server. The
[persistence inventory](persistence-reload-roots.md) owns which values survive those boundaries;
this surface owns when each boundary occurs and what can observe it.

| Lifecycle family | Locked source roots | Exact ownership and ordering conclusion |
|---|---|---|
| World input and registry bootstrap | `net.minecraft.server.WorldLoader#load`, `net.minecraft.server.MinecraftServer#loadLevel`, `net.minecraft.server.MinecraftServer#createLevels` | DataReload owns the completed registry/resource snapshot handed to the server. Load marks modded state, creates levels, forces difficulty and only then prepares levels. Creation inserts Overworld first, loads scoreboard/command/stopwatch saved state, initializes its spawn once, then inserts every non-Overworld stem in registry order with derived level data. Border limits/listeners are installed in that same stem order. Initialization failure is wrapped with the Overworld crash report and escapes; this method has no partial-construction rollback, so outer server/resource close owns cleanup. |
| Level creation, spawn and initial readiness | `net.minecraft.server.MinecraftServer#createLevels`, `net.minecraft.server.MinecraftServer#setInitialSpawn`, `net.minecraft.server.MinecraftServer#prepareLevels`, `net.minecraft.server.level.ServerLevel#startTickingChunk` | Debug spawn is fixed at `(0,80,0)`; ordinary spawn uses the generator sampler, generator height or surface fallback, then the exact 121-chunk spiral and optional already-specified bonus-chest feature. Preparation tracks all levels, reactivates each saved ticket set at its ready callback, pumps scheduled work until pending count is zero, then updates mob-spawn flags and effective respawn data. A ticking promotion unpacks persisted ticks at current game time before later callbacks. |
| Chunk demand, storage read and generation | `net.minecraft.server.level.ServerChunkCache#getChunk`, `net.minecraft.server.level.ServerChunkCache#getChunkFuture`, `net.minecraft.server.level.ChunkMap#scheduleChunkLoad`, `net.minecraft.server.level.ChunkMap#scheduleGenerationTask` | Off-thread synchronous demand resubmits to the main executor and joins. Four-entry exact position/status caching precedes a main-thread future; create demand adds an UNKNOWN ticket at `ChunkLevel.byStatus`, immediately runs distance updates when the holder is absent, and throws if no holder appears. Noncreate demand returns the shared unloaded result. EMPTY combines async chunk read/parse with POI prefetch, reconstructs on the main executor, and creates an empty ProtoChunk for absent/missing-level or reported non-Error load failure; an `Error` becomes a reported chunk-load crash. Later statuses use the exact dependency/task/publication graph in `WGEN-PIPELINE-001`; queued generation resumes only when each awaited future completes. |
| Tickets, levels and activity promotion | `net.minecraft.server.level.ServerChunkCache#addTicket`, `net.minecraft.server.level.ServerChunkCache#addTicketAndLoadWithRadius`, `net.minecraft.server.level.ServerChunkCache#addTicketWithRadius`, `net.minecraft.server.level.ServerChunkCache#removeTicketWithRadius`, `net.minecraft.server.level.DistanceManager#runAllUpdates`, `net.minecraft.server.level.ChunkHolder#updateFutures` | `SIM-005`, `SIM-RANDOM-001` and the exact nine-entry ticket catalog own levels, flags and expiry. One update pass runs natural-spawn, simulation and player trackers, drains loading distances, updates every changed holder's highest allowed status before updating any holder futures, then defers player-loading ticket release until the entity-ticking future completes. Loaded demand resurrects the identical pending-unload holder when present. Promotion cancels the prior confirmation and publishes its new full status on the main executor only after the corresponding future succeeds; demotion completes the old future with unloaded and publishes the lower status synchronously. |
| Accessible, ticking and entity-ticking publication | `net.minecraft.server.level.ChunkMap#prepareAccessibleChunk`, `net.minecraft.server.level.ChunkMap#prepareTickingChunk`, `net.minecraft.server.level.ChunkMap#prepareEntityTickingChunk`, `net.minecraft.server.level.ServerChunkCache#onChunkReadyToSend`, `net.minecraft.server.level.ChunkMap#forEachReadyToSendChunk` | Accessible requires the radius-one status function around full; block ticking requires radius-one FULL, then postprocesses generation, unpacks ticks and waits for the holder's send dependency before marking every current tracking player pending, registering debug state and broadcasting queued changes. Entity ticking requires radius-two FULL. Terrain enumeration exposes only `getChunkToSend` successes from the visible snapshot; POI/light, tracking and packet contents stay with their named owners. |
| Save, demotion and unload | `net.minecraft.server.level.ChunkMap#saveAllChunks`, `net.minecraft.server.level.ChunkMap#processUnloads`, `net.minecraft.server.level.ChunkMap#scheduleUnload`, `net.minecraft.server.level.ServerLevel#unload`, `net.minecraft.server.level.ServerChunkCache#save` | A flush waits each ever-accessible holder ready, repeatedly saves latest live/imposter chunks until no save succeeds, flushes POI, drains unloads and synchronizes storage. Ordinary processing moves dropped holders from updating to pending, queues an unload after the current save-sync future, and recursively reschedules if that future identity changed. The task proceeds only if the same holder is still pending: mark a LevelChunk unloaded, attempt save, clear block entities and unregister ticks, drop debug state, update/schedule light and clear next-save time. Re-added demand cancels the pending removal by identity. Eager save is capped at 20 per pass, 128 active writes and a successful time supplier. |
| Dimension travel, portals and world border | `net.minecraft.server.level.ServerPlayer#teleport`, `net.minecraft.server.level.ServerLevel#addDuringTeleport`, `net.minecraft.server.level.ServerLevel#addRespawnedPlayer`, `net.minecraft.server.network.config.PrepareSpawnTask#start`, `net.minecraft.server.network.config.PrepareSpawnTask$Ready#spawn`, `net.minecraft.world.level.portal.PortalForcer#findClosestPortalPosition`, `net.minecraft.world.level.portal.PortalForcer#createPortal`, `net.minecraft.world.level.border.WorldBorder#setCenter`, `net.minecraft.world.level.border.WorldBorder#setSize`, `net.minecraft.world.level.border.WorldBorder#lerpSizeBetween`, `net.minecraft.world.level.border.WorldBorder#tick` | Portal transitions own their documented final-position radius-3 ticket. Initial configuration spawn instead owns a radius-0 `SPAWN_SEARCH` ticket, then a radius-3 `PLAYER_SPAWN` ticket and entity-readiness wait. Respawn and direct cross-level teleport have no equivalent chunk-readiness wait: player insertion publishes the always-ticking entity to destination tracking and ticking immediately. Cross-level teleport sends respawn, difficulty and permission before source removal, then changes level, sends position, inserts in the destination and reprojects state; it has no local rollback after a visible or membership prefix. `WGEN-BORDER-001` separately owns border geometry, interpolation, consumers and projection and is not a destination-membership gate. |
| Save-all and clean shutdown | `net.minecraft.server.MinecraftServer#saveEverything`, `net.minecraft.server.MinecraftServer#stopServer`, `net.minecraft.server.MinecraftServer#close`, `net.minecraft.server.level.ServerChunkCache#deactivateTicketsOnClosing`, `net.minecraft.server.level.ServerChunkCache#close`, `net.minecraft.server.level.ServerLevel#close`, `net.minecraft.server.WorldStem#close` | Ordinary save sets `isSaving` in a finally-guard, saves players before chunks, then checks disk space. Shutdown closes packet processing/metrics and network admission, sets saving, saves then removes all players, clears every level's no-save flag, repeatedly deactivates closing tickets and ticks chunk sources until every level reports no work, then performs one flush save. It closes each level independently while logging IO failures, clears saving, closes shared saved-data storage and resources, and finally attempts the storage lock close. A level chunk source itself saves, then closes data storage, light engine, both dispatchers, POI and region storage with the region close in `finally`; `WorldStem.close` closes only its resource manager. |

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
- `BLK-TORCHFLOWER-CROP-001` fixes zero raw crop or mature-flower cells across all 1,212 templates.
  Four planted/nonplanted crimson/warped huge-fungus configurations admit both identities as
  replaceable; generic feature traversal, clipping, write results, publication and later live
  support/growth remain with their owning lifecycle rules.
- `BLK-PITCHER-CROP-001` fixes zero raw crop or mature-plant cells across all 1,212 templates. Four
  planted/nonplanted crimson/warped huge-fungus configurations explicitly admit only crop, while
  generic tree and huge-mushroom replacement admits the separate mature plant through tags;
  traversal, clipping, writes, publication and later live support/growth remain parent-owned.
- `BLK-SWEET-BERRY-BUSH-001` fixes zero raw bush cells across all 1,212 templates. An age-three
  simple provider feeds a 96-attempt AIR-over-grass placed feature selected by both taiga-village
  decor pools, and all four huge-fungus configurations admit the bush as replaceable. Feature/pool
  selection, transforms, clipping, writes, publication and later live behavior remain parent-owned.
- `BLK-CAVE-VINES-001` fixes zero raw head/body cells across all 1,212 templates. Lush caves invoke
  a 188-attempt direct ceiling column and a 125-attempt moss-ceiling path whose child uses the
  second weighted column; moss and all four huge-fungus configurations can replace both identities.
  Feature selection, column truncation, writes, publication and later live behavior stay parent-owned.
- `BLK-CHORUS-001` fixes zero raw plant/flower cells across all 1,212 templates. End Highlands
  selects a count-0..4 heightmap/biome placed feature whose AIR-over-end-stone origin recursively
  builds bounded connected columns and dead tips; all four huge-fungus configurations admit both
  identities as replaceable. Selection, recursion writes, partial failure, publication and later
  live support/growth remain parent-owned.

- Chunk status generation, full-chunk accessibility, block ticking and entity ticking are distinct
  transitions. A single loaded/unloaded boolean cannot reproduce their observable admission gates.
- Ticket state is transient scheduling authority even when its cause is persistent. Reconstruction
  may differ internally only if promotion, demotion, expiry and first admitted work stay equivalent.
- A chunk can finish data generation before it is ready for world callbacks or client terrain
  projection. Publication and tracking are distinct mapped transactions with separate owners.
- Clean shutdown drains work after ticket deactivation, flushes once, then closes each live level
  before shared saved data, resources and the storage lock. Per-level IO close failure is logged
  and iteration continues; the server has no rollback to a live state after shutdown begins.

## Reproduction

Follow one demanded chunk through absent storage, valid/missing/malformed/read-failed reconstruction,
every generation status, accessible, block-ticking, entity-ticking, tracking, demotion, pending
unload cancellation, save and final unload. Repeat with all nine ticket types, every full-status
threshold, expiry/replacement, view/simulation changes and forced chunks. Assert tracker and holder
update order, promotion-versus-demotion publication, send dependencies, live teardown and first
reloaded observation. Separately trace new/existing/debug worlds through partial construction and
normal/error shutdown; assert level insertion order, initial readiness, player-before-chunk saves,
work draining, final flush and the complete close/failure sequence.
