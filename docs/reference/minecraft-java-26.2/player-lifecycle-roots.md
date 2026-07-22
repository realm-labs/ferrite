# Player Lifecycle Root Inventory

**Surface:** `SURFACE-PLAYER-LIFECYCLE-001`
**Status:** `InProgress`
**Primary evidence:** `OFF-SERVER-001`, `OFF-REPORT-001`

This is the recoverable source-root inventory for creation, restoration, play entry, death,
replacement respawn, relocation, sleep, mode/ability changes, disconnect, removal and persistence
of a server player. It establishes phase ownership and joins to already specified mechanics and
protocol families. It does not claim that every persisted field, exceptional branch or reconnect
vector has been audited.

| Phase | Locked source roots | Existing semantic owners and protocol joins | Remaining audit |
|---|---|---|---|
| Admission, construction and restoration | `net.minecraft.server.players.PlayerList#canPlayerLogin`, `net.minecraft.server.players.PlayerList#loadPlayerData`, `net.minecraft.server.level.ServerPlayer#readAdditionalSaveData` | Login admission is protocol-owned; restored movement, entity, item and progression state delegates to the named gameplay owners rather than making serialized fields the simulation API. | Enumerate every restored field and missing/corrupt/default branch, duplicate-profile handling, admission rejection, level selection and pre-play failure cleanup. |
| Initial play entry | `net.minecraft.server.players.PlayerList#placeNewPlayer`, `net.minecraft.server.players.PlayerList#sendLevelInfo`, `net.minecraft.server.players.PlayerList#sendAllPlayerInfo`, `net.minecraft.server.players.PlayerList#sendPlayerPermissionLevel` | `PROTO-PLAY-CLIENTBOUND-ENTRY-001` owns the exact login/core projection order; `PROTO-PLAY-SERVERBOUND-ENTRY-001` owns the client-loaded gate. `PLY-001` owns authoritative player state after placement. | Complete every join publication, existing-player fan-out, transferred-session difference, add-to-level/list ordering, failure rollback and notification hook. |
| Death and client-unloaded gate | `net.minecraft.server.level.ServerPlayer#die`, `net.minecraft.server.network.ServerGamePacketListenerImpl#markClientUnloadedAfterDeath`, `net.minecraft.server.network.ServerGamePacketListenerImpl#handleClientCommand` | `ENT-005`/`ENT-007` own damage and death; `PROTO-PLAY-CLIENTBOUND-PLAYER-PROJECTION-001` owns death presentation; `PROTO-PLAY-SERVERBOUND-ENTITY-SESSION-001` owns the respawn request. | Cross-check immediate-respawn, hardcore and won-game branches, duplicate/early requests, inventory/progression keep rules and the exact point that movement/action admission reopens. |
| Replacement respawn | `net.minecraft.server.players.PlayerList#respawn`, `net.minecraft.server.level.ServerPlayer#findRespawnPositionAndUseSpawnBlock`, `net.minecraft.server.level.ServerPlayer#restoreFrom`, `net.minecraft.server.network.ServerGamePacketListenerImpl#restartClientLoadTimerAfterRespawn` | `ENT-008` and `WGEN-005` own relocation/spawn selection. `PROTO-PLAY-CLIENTBOUND-ENTITY-SESSION-001` owns respawn then position and state reprojection. | Audit keep-everything masks, old/new entity identity and list membership, missing/invalid spawn fallback, anchor depletion, effect/permission/menu restoration, rollback and post-respawn client-loaded timeout. |
| Dimension, spawn and sleep transitions | `net.minecraft.server.level.ServerPlayer#teleportTo`, `net.minecraft.server.level.ServerPlayer#setRespawnPosition`, `net.minecraft.server.level.ServerPlayer#startSleepInBed`, `net.minecraft.server.level.ServerPlayer#stopSleepInBed` | `ENT-008`, `WGEN-005`, `PLY-001` and the bed/portal mechanics own authoritative transitions; ordinary convergence uses the entry/entity-session protocol families. | Inventory every portal/command/credits caller, same- versus cross-level failure, passenger/camera policy, bed problem result, sleep-vote/weather join and spawn-message branch. |
| Mode, abilities and permissions | `net.minecraft.server.level.ServerPlayer#setGameMode`, `net.minecraft.server.level.ServerPlayer#onUpdateAbilities`, `net.minecraft.server.players.PlayerList#sendPlayerPermissionLevel` | `PLY-001` owns movement authority; `PROTO-PLAY-CLIENTBOUND-ENTRY-001`, `PROTO-PLAY-CLIENTBOUND-PLAYER-PROJECTION-001` and administration families own wire projection and authorized changes. | Audit no-change results, creative/spectator modifiers, flight invalidation, command-tree resend, operator changes and ordering against movement/menu state. |
| Disconnect and world removal | `net.minecraft.server.network.ServerGamePacketListenerImpl#onDisconnect`, `net.minecraft.server.network.ServerGamePacketListenerImpl#removePlayerFromWorld`, `net.minecraft.server.players.PlayerList#remove` | Entity removal delegates to `ENT-001`; player-info removal is `PROTO-PLAY-CLIENTBOUND-PLAYER-INFO-REMOVE-001`; connection-only acknowledgements and throttlers are discarded. | Audit idempotence, vehicle/passenger/menu/shoulder/pearl cleanup, leave notification, scoreboard/team visibility, save-before-removal order, abrupt transport loss and shutdown-wide removal. |
| Save and later reconnect | `net.minecraft.server.players.PlayerList#save`, `net.minecraft.server.level.ServerPlayer#addAdditionalSaveData`, `net.minecraft.server.players.PlayerList#getPlayerStats`, `net.minecraft.server.players.PlayerList#getPlayerAdvancements` | Persistent player, inventory and progression values re-enter through the first phase; transport IDs, pending acknowledgements, latency and projection mirrors do not. | Build the exhaustive persisted-field ledger, join it to `PersistenceReload`, and replay clean disconnect, crash/restart, cross-dimension save, death-before-respawn and partially missing auxiliary files. |

## Current ordering conclusions

- Initial play entry creates and installs the play listener before emitting the locked join
  projection. The exact packet sequence remains owned by the cited protocol family rather than this
  surface inventory.
- Replacement respawn removes the old player from its list and level, constructs a new
  `ServerPlayer`, transfers the existing connection, restores selected state, publishes respawn and
  position/state convergence, adds the replacement to its level and both player indexes, then
  reinitializes its inventory menu. The precise keep mask and every side branch remain outstanding.
- Disconnect handling converges transport closure into world removal and `PlayerList#remove`; the
  latter owns saving, level/list/index removal and player-info removal publication. The exact
  save/cleanup/notification ordering remains part of the audit rather than an inferred guarantee.

## Recovery procedure

1. For each row, enumerate every caller and branch from the locked class bytecode and add any newly
   discovered root before narrowing its remaining audit.
2. Cross every read/written field with the `PersistenceReload` state-domain ledger and label it
   persistent, reconstructed, projected or connection-local.
3. Replay initial join, transfer join, normal/hardcore death, immediate/manual respawn, invalid
   spawn, same/cross-dimension relocation, sleep/wake, mode/permission change, graceful disconnect,
   abrupt loss and restart.
4. Promote this surface only after all rows have exact admission, ordering, rollback, persistence
   and projection conclusions; structural phase ownership alone is not completion.
