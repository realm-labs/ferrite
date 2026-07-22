# Persistence and Reload Root Inventory

**Surface:** `SURFACE-PERSISTENCE-RELOAD-001`
**Status:** `InProgress`
**Primary evidence:** `OFF-SERVER-001`

This inventory owns observable continuity across chunk unload/reload, player disconnect/rejoin and
full server restart without requiring Minecraft's original file formats. Ferrite may encode state
differently, but every behavior-owned value must either survive, be deterministically reconstructed,
or be explicitly transient with a specified first post-boundary result.

| Continuity family | Locked source roots | Existing semantic owners | Remaining audit |
|---|---|---|---|
| Server bootstrap, save and shutdown | `net.minecraft.server.MinecraftServer#loadLevel`, `net.minecraft.server.MinecraftServer#saveAllChunks`, `net.minecraft.server.MinecraftServer#saveEverything`, `net.minecraft.server.MinecraftServer#stopServer` | `SIM-001`/`SIM-006` own tick admission, autosave and pause timing; this surface owns continuity across the resulting durable boundary. | Audit force/flush/skip-save arguments, per-level and player ordering, pending asynchronous writes, partial failures, clean close versus crash and first restarted tick. |
| Level and chunk lifecycle | `net.minecraft.server.level.ServerLevel#save`, `net.minecraft.server.level.ServerLevel#unload`, `net.minecraft.server.level.ServerChunkCache#save`, `net.minecraft.server.level.ServerChunkCache#close`, `net.minecraft.server.level.ChunkMap#saveAllChunks`, `net.minecraft.server.level.ChunkMap#processUnloads`, `net.minecraft.server.level.ChunkMap#scheduleUnload`, `net.minecraft.server.level.ChunkMap#scheduleChunkLoad` | `WGEN-001`, `SIM-005`, block and entity owners define live state; persistence owns when that state becomes inactive, serialized, reconstructed and visible again. | Enumerate dirty/admission gates, active-write limits, holder futures, unload cancellation, read/write failures, data-fix/default branches, light/height/structure state and first-ready publication. |
| Scheduled block and fluid work | `net.minecraft.world.ticks.LevelChunkTicks#pack`, `net.minecraft.world.ticks.LevelChunkTicks#unpack`, `net.minecraft.world.ticks.SavedTick#unpack` | `SIM-003` and `SIM-SCHEDULE-001` own delay reconstruction, priority, sub-order and the explicit equal-head source-inconclusive case. | Cross chunk unload, full restart, already-overdue and integer-overflow cases with world time, duplicate filtering and first callback order; preserve the existing experiment-owned global tie. |
| Block states and block entities | `net.minecraft.world.level.block.entity.BlockEntity#setChanged`, `net.minecraft.world.level.block.entity.BlockEntity#saveWithFullMetadata`, `net.minecraft.world.level.block.entity.BlockEntity#loadStatic`, `net.minecraft.world.level.block.entity.BlockEntity#loadWithComponents` | `BLK-001`, `BLK-003` and `BLK-007` own state, mutation dirtiness and block-entity lifecycle; `BLK-SCULK-SENSOR-001` fixes frequency plus selector/current vibration/delay continuity, `BLK-JIGSAW-001` fixes all seven connector fields and orientation-dependent defaults, `BLK-STRUCTURE-001` fixes its complete record, divergent fresh/load defaults, load clamps, power latch and full update tag while separating entity saves from memory/disk template-manager continuity, `BLK-STRUCTURE-VOID-001` fixes ordinary world-state continuity versus its deliberately absent captured-template coordinate, `BLK-TEST-BLOCK-001` fixes mode/message/powered continuity plus the transient trigger and divergent state defaults, `BLK-TEST-INSTANCE-001` fixes its complete data/marker codecs, malformed-record retention, dirty/update path and runner-owned entity replacement, `BLK-CONDUIT-001` fixes optional target-UUID continuity plus all reset/derived fields and non-dirty target projection, `BLK-BEACON-001` fixes power/name/lock continuity plus ignored saved Levels and rebuilt beam/base state, `BLK-SIGN-001` fixes both four-line raw/filtered sides, color/glow/wax continuity, component resolution and transient editor authorization, `BLK-SKULL-001` fixes nullable profile/sound/name continuity plus nonserialized client animation retention/reset, `ITM-CHEST-001` fixes per-half items/loot/name/lock, pairing migration and transient opener/lid reconstruction, `ITM-HOPPER-001` fixes five-slot loot/name/lock and exact signed cooldown continuity plus reconstructed facing/tick time, `ITM-DISPENSER-001` fixes nine-slot loot/name/lock continuity while separating persisted facing/triggered state from the live scheduled callback, and other subtype leaves own their observable fields. | Build the remaining exhaustive block-entity field matrix, unknown/wrong type and malformed/default branches, component interaction, ticker reinstallation, cached/transient fields and first update/comparator/client projection. |
| Persistent entities | `net.minecraft.world.level.entity.PersistentEntitySectionManager#processChunkUnload`, `net.minecraft.world.level.entity.PersistentEntitySectionManager#processPendingLoads`, `net.minecraft.world.level.entity.PersistentEntitySectionManager#saveAll` | `ENT-001` and subtype leaves own UUID, section, passenger and lifecycle state; removal reasons distinguish unload from death/discard. | Audit async load inbox order, duplicate UUID rejection, passenger trees, cross-chunk references, brain/goal/transient caches, pending teleports, removal callbacks and first ticking/tracking insertion. |
| Players and reconnect | `net.minecraft.server.players.PlayerList#loadPlayerData`, `net.minecraft.server.players.PlayerList#save`, `net.minecraft.server.level.ServerPlayer#readAdditionalSaveData`, `net.minecraft.server.level.ServerPlayer#addAdditionalSaveData` | `SURFACE-PLAYER-LIFECYCLE-001` and [its root inventory](player-lifecycle-roots.md) own join/remove/replacement phases; item and progression leaves own field meaning. | Complete the persisted-field ledger and clean-loss/crash/restart matrix already named by PlayerLifecycle, including dimension, death-before-respawn and missing stats/advancement files. |
| Saved world data and auxiliary progression | `net.minecraft.world.level.storage.SavedDataStorage#computeIfAbsent`, `net.minecraft.world.level.storage.SavedDataStorage#scheduleSave`, `net.minecraft.world.level.storage.SavedDataStorage#saveAndJoin`, `net.minecraft.server.ServerScoreboard#load`, `net.minecraft.server.ServerScoreboard#setDirty`, `net.minecraft.stats.ServerStatsCounter#save`, `net.minecraft.server.PlayerAdvancements#load`, `net.minecraft.server.PlayerAdvancements#save` | World-border, map, scoreboard/team, statistics and advancement owners define values and mutations; this family owns dirty collection, write completion and reconstruction. | Inventory every saved-data type and auxiliary file, dirty-clear timing, absent/corrupt/default/migration behavior, async write failure, cross-file atomicity and first listener/client convergence. |
| Reconstructed and transient state | `net.minecraft.server.level.ServerChunkCache#onChunkReadyToSend`, `net.minecraft.server.players.PlayerList#placeNewPlayer`, `net.minecraft.server.players.PlayerList#sendLevelInfo` | Client projection and protocol families own chunk/player convergence; simulation leaves identify RNG streams, caches, interpolators and transport/session data that may restart. | Classify every nonserialized behavior field as derived, reset-with-defined-first-result or incorrectly missing; compare uninterrupted controls at unload/reload, reconnect and restart boundaries. |

## Current boundary conclusions

- A compatible Ferrite save need not use NBT, Region, Anvil or Mojang data-fix layouts. The required
  contract is the post-boundary authoritative state and its next observable transition.
- Chunk persistence is asynchronous in the locked implementation, while server/level close paths
  join outstanding work. Ferrite may schedule writes differently only if save admission, failure,
  unload visibility and restart results remain equivalent.
- Saved scheduled ticks reconstruct relative to load time. Fully unloaded wall/game time does not
  silently become accumulated callback work; the exact queue rule and unresolved equal-head case
  remain owned by `SIM-SCHEDULE-001`.
- Runtime-only transport IDs, acknowledgement windows, interpolation caches and client mirrors are
  not persistent identities. Their reset must still lead to the same authoritative reprojection.

## Recovery procedure

1. Enumerate every behavior-owned field under the eight families and label it persisted,
   reconstructed, reset-with-defined-first-result or source-inconclusive.
2. Record its dirty/admission trigger, write ordering, load default/migration branch, reference
   resolution and first post-boundary consumer/projection.
3. Replay each field through chunk unload/reload, player disconnect/rejoin and full restart against
   an uninterrupted control; inject missing, malformed and failed-write inputs where the locked
   source exposes a branch.
4. Join every persistence result to WorldLifecycle, PlayerLifecycle, DataReload and client
   projection before promoting this surface; a list of save methods alone is not completion.
