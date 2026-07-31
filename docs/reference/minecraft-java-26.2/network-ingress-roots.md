# Network Ingress Root Inventory

**Surface:** `SURFACE-NETWORK-INGRESS-001`
**Status:** `Mapped`
**Primary evidence:** `OFF-SERVER-001`, `OFF-REPORT-001`

This inventory partitions all 22 serverbound protocol families locked by
[`protocol/completion.toml`](protocol/completion.toml). Protocol specifications own bytes, bounds,
legal connection state and packet-local transitions. This surface owns the handoff from a decoded
packet to connection or gameplay state. `net.minecraft.network.Connection#channelRead0` and
`net.minecraft.network.protocol.PacketUtils#ensureRunningOnSameThread` are the common decode-to-
listener and listener-to-server-thread boundaries.

| Protocol family | Locked listener roots | Semantic owner or transport-only boundary |
|---|---|---|
| `PROTO-HANDSHAKE-SERVERBOUND-001` | `net.minecraft.server.network.ServerHandshakePacketListenerImpl#handleIntention` | Connection-state selection only; gameplay state does not yet exist. |
| `PROTO-STATUS-SERVERBOUND-001` | `net.minecraft.server.network.ServerStatusPacketListenerImpl#handleStatusRequest`, `net.minecraft.server.network.ServerStatusPacketListenerImpl#handlePingRequest` | Cached status read and opaque ping echo only; no gameplay mutation. |
| `PROTO-LOGIN-SERVERBOUND-REQUIRED-001` | `net.minecraft.server.network.ServerLoginPacketListenerImpl#handleHello`, `net.minecraft.server.network.ServerLoginPacketListenerImpl#handleLoginAcknowledgement` | PlayerLifecycle owns identity admission and the terminal configuration handoff. |
| `PROTO-LOGIN-SERVERBOUND-OPTIONAL-001` | `net.minecraft.server.network.ServerLoginPacketListenerImpl#handleKey`, `net.minecraft.server.network.ServerLoginPacketListenerImpl#handleCustomQueryPacket`, `net.minecraft.server.network.ServerLoginPacketListenerImpl#handleCookieResponse` | Gated authentication/extension transport; no gameplay mutation before successful admission. |
| `PROTO-CONFIGURATION-SERVERBOUND-REQUIRED-001` | `net.minecraft.server.network.ServerConfigurationPacketListenerImpl#handleClientInformation`, `net.minecraft.server.network.ServerConfigurationPacketListenerImpl#handleSelectKnownPacks`, `net.minecraft.server.network.ServerConfigurationPacketListenerImpl#handleConfigurationFinished` | DataReload owns registry/tag snapshot selection; PlayerLifecycle owns transition into the live player session. |
| `PROTO-CONFIGURATION-SERVERBOUND-OPTIONAL-001` | `net.minecraft.server.network.ServerConfigurationPacketListenerImpl#handleResourcePackResponse`, `net.minecraft.server.network.ServerConfigurationPacketListenerImpl#handleAcceptCodeOfConduct`, `net.minecraft.server.network.ServerCommonPacketListenerImpl#handleCustomClickAction` | Explicitly gated configuration tasks; absent task ownership rejects or disconnects without gameplay mutation. |
| `PROTO-PLAY-SERVERBOUND-ENTRY-001` | `net.minecraft.server.network.ServerGamePacketListenerImpl#handleAcceptPlayerLoad`, `net.minecraft.server.network.ServerGamePacketListenerImpl#handleClientTickEnd` | `PLY-005`, PlayerLifecycle and WorldLifecycle own loaded-player admission and end-of-client-tick ordering. |
| `PROTO-PLAY-SERVERBOUND-MOVEMENT-001` | `net.minecraft.server.network.ServerGamePacketListenerImpl#handleMovePlayer`, `net.minecraft.server.network.ServerGamePacketListenerImpl#handleMoveVehicle`, `net.minecraft.server.network.ServerGamePacketListenerImpl#handleAcceptTeleportPacket`, `net.minecraft.server.network.ServerGamePacketListenerImpl#handlePlayerInput`, `net.minecraft.server.network.ServerGamePacketListenerImpl#handlePlayerAbilities` | `PLY-001`, `PLY-005`, `CLI-003` and movement leaves own validation, teleport acknowledgement and correction. |
| `PROTO-PLAY-SERVERBOUND-BLOCK-001` | `net.minecraft.server.network.ServerGamePacketListenerImpl#handlePlayerAction`, `net.minecraft.server.network.ServerGamePacketListenerImpl#handleUseItemOn`, `net.minecraft.server.network.ServerGamePacketListenerImpl#handleUseItem` | `PLY-006`, `BLK-003`, `ITM-BOAT-001` and interaction/break/place leaves own sequence admission and authoritative mutation; the boat leaf fixes POV/eye/collision gates and the ignored post-collision entity-admission result. |
| `PROTO-PLAY-SERVERBOUND-CHAT-001` | `net.minecraft.server.network.ServerGamePacketListenerImpl#handleChat`, `net.minecraft.server.network.ServerGamePacketListenerImpl#handleChatCommand`, `net.minecraft.server.network.ServerGamePacketListenerImpl#handleSignedChatCommand`, `net.minecraft.server.network.ServerGamePacketListenerImpl#handleChatAck`, `net.minecraft.server.network.ServerGamePacketListenerImpl#handleChatSessionUpdate` | `CLI-005` plus chat/command protocol specifications own signature, last-seen, command dispatch and disconnect branches. |
| `PROTO-PLAY-SERVERBOUND-ENTITY-SESSION-001` | `net.minecraft.server.network.ServerGamePacketListenerImpl#handleAttack`, `net.minecraft.server.network.ServerGamePacketListenerImpl#handleInteract`, `net.minecraft.server.network.ServerGamePacketListenerImpl#handleAnimate`, `net.minecraft.server.network.ServerGamePacketListenerImpl#handlePlayerCommand`, `net.minecraft.server.network.ServerGamePacketListenerImpl#handleClientCommand` | `ENT-005`, `PLY-005`, `ITM-BOAT-001` and entity/player leaves own target lookup, combat/use, animation and session actions; the boat leaf fixes mount-versus-container selection after ordinary target admission. |
| `PROTO-PLAY-SERVERBOUND-CONTAINER-CONVERGENCE-001` | `net.minecraft.server.network.ServerGamePacketListenerImpl#handleContainerClose`, `net.minecraft.server.network.ServerGamePacketListenerImpl#handleContainerClick`, `net.minecraft.server.network.ServerGamePacketListenerImpl#handlePlaceRecipe`, `net.minecraft.server.network.ServerGamePacketListenerImpl#handleContainerButtonClick` | `ITM-002`, `ITM-BOAT-001`, `ITM-CHEST-001`, `ITM-HOPPER-001`, `ITM-DISPENSER-001` and container leaves own menu ID/state ID, predicted hashes, mutation, close/recount or concurrent entity/automation/scheduled-dispatch consequences and resynchronization; `BLK-AIR-001` fixes AIR as the empty hashed-stack form. |
| `PROTO-PLAY-SERVERBOUND-SIGN-UPDATE-001` | `net.minecraft.server.network.ServerGamePacketListenerImpl#handleSignUpdate` | `PLY-005`, `BLK-003` and `BLK-SIGN-001` own filtering order, completion-time chunk/entity/wax/editor admission, literal/style rebuild, authorization clearing and block-entity projection. |
| `PROTO-PLAY-SERVERBOUND-RECIPE-BOOK-001` | `net.minecraft.server.network.ServerGamePacketListenerImpl#handleRecipeBookSeenRecipePacket`, `net.minecraft.server.network.ServerGamePacketListenerImpl#handleRecipeBookChangeSettingsPacket` | `ITM-002`, `ITM-004` and recipe/progression owners own the per-player book state. |
| `PROTO-PLAY-SERVERBOUND-MERCHANT-001` | `net.minecraft.server.network.ServerGamePacketListenerImpl#handleSelectTrade` | `ITM-002` and merchant/container owners own offer selection and menu convergence. |
| `PROTO-PLAY-SERVERBOUND-ANVIL-BEACON-001` | `net.minecraft.server.network.ServerGamePacketListenerImpl#handleRenameItem`, `net.minecraft.server.network.ServerGamePacketListenerImpl#handleSetBeaconPacket` | `ITM-002`, `BLK-BEACON-001` and the respective menu/block-entity owners own bounded input, payment and result mutation. |
| `PROTO-PLAY-SERVERBOUND-INVENTORY-AUXILIARY-001` | `net.minecraft.server.network.ServerGamePacketListenerImpl#handleSetCarriedItem`, `net.minecraft.server.network.ServerGamePacketListenerImpl#handleBundleItemSelectedPacket`, `net.minecraft.server.network.ServerGamePacketListenerImpl#handlePickItemFromBlock`, `net.minecraft.server.network.ServerGamePacketListenerImpl#handlePickItemFromEntity` | `ITM-002`, `PLY-005`, `ITM-BUNDLE-001` and inventory owners own selection, creative gates and resulting slot projection. Bundle selection resolves the handler-time menu slot, admits any in-list index including tooltip-hidden entries, clears on an out-of-list index and silently ignores an invalid slot or missing component; the packet carries no container/state ID or acknowledgement. |
| `PROTO-PLAY-SERVERBOUND-ADMIN-STATE-001` | `net.minecraft.server.network.ServerGamePacketListenerImpl#handleSetGameRule`, `net.minecraft.server.network.ServerGamePacketListenerImpl#handleChangeDifficulty`, `net.minecraft.server.network.ServerGamePacketListenerImpl#handleLockDifficulty`, `net.minecraft.server.network.ServerGamePacketListenerImpl#handleChangeGameMode`, `net.minecraft.server.network.ServerGamePacketListenerImpl#handleSetCreativeModeSlot`, `net.minecraft.server.network.ServerGamePacketListenerImpl#handleSpectatorAction` | CommandAdministration and affected game-rule, world and player owners define permission and mutation semantics; `BLK-AIR-001` makes every decoded positive AIR stack an empty creative-slot value or no-drop request. |
| `PROTO-PLAY-SERVERBOUND-OPERATOR-BLOCKS-001` | `net.minecraft.server.network.ServerGamePacketListenerImpl#handleSetCommandBlock`, `net.minecraft.server.network.ServerGamePacketListenerImpl#handleSetCommandMinecart`, `net.minecraft.server.network.ServerGamePacketListenerImpl#handleSetStructureBlock`, `net.minecraft.server.network.ServerGamePacketListenerImpl#handleSetTestBlock`, `net.minecraft.server.network.ServerGamePacketListenerImpl#handleTestInstanceBlockAction`, `net.minecraft.server.network.ServerGamePacketListenerImpl#handleSetJigsawBlock`, `net.minecraft.server.network.ServerGamePacketListenerImpl#handleJigsawGenerate` | CommandAdministration, `BLK-COMMAND-001`, `BLK-JIGSAW-001`, `BLK-STRUCTURE-001`, `BLK-TEST-BLOCK-001`, `BLK-TEST-INSTANCE-001`, `BLK-003`, `WGEN-JIGSAW-CORE-001` and `WGEN-005` own operator permission, block/entity lookup and world mutation. |
| `PROTO-PLAY-SERVERBOUND-DEBUG-SUBSCRIPTION-001` | `net.minecraft.server.network.ServerGamePacketListenerImpl#handleDebugSubscriptionRequest` | Gated diagnostic subscription state only; `CLI-006` owns any observable debug projection. |
| `PROTO-PLAY-SERVERBOUND-COMMON-SERVICES-001` | `net.minecraft.server.network.ServerCommonPacketListenerImpl#handleKeepAlive`, `net.minecraft.server.network.ServerCommonPacketListenerImpl#handlePong`, `net.minecraft.server.network.ServerCommonPacketListenerImpl#handleCustomPayload`, `net.minecraft.server.network.ServerCommonPacketListenerImpl#handleResourcePackResponse`, `net.minecraft.server.network.ServerGamePacketListenerImpl#handleChunkBatchReceived`, `net.minecraft.server.network.ServerGamePacketListenerImpl#handlePingRequest` | Liveness, telemetry, payload and task acknowledgements are connection-local unless an explicitly owned extension is enabled. |
| `PROTO-PLAY-SERVERBOUND-RECONFIGURATION-001` | `net.minecraft.server.network.ServerGamePacketListenerImpl#handleConfigurationAcknowledged`, `net.minecraft.server.network.ServerGamePacketListenerImpl#switchToConfig` | DataReload and PlayerLifecycle own snapshot/session convergence; the protocol family owns the terminal play-to-configuration transition. |

## Boundary conclusions

- A decoded packet is not a gameplay mutation. `Connection#channelRead0` snapshots the current
  listener and applies its first `shouldHandleMessage` gate on the channel thread. A handler that
  calls `PacketUtils#ensureRunningOnSameThread` schedules the exact listener-and-packet pair on the
  server's dedicated `PacketProcessor` and throws `RunningOnDifferentThreadException` to end the
  channel-thread attempt. Listener-local handlers without that call, including configuration
  client-information replacement, can mutate connection-local state directly on the channel
  thread.
- `PacketProcessor` owns a `ConcurrentLinkedQueue` separate from the server executor's ordinary
  task queue. `MinecraftServer#processPacketsAndTick` drains it in the
  `scheduledPacketProcessing` stage before `tickServer`. Each queued pair applies the captured
  listener's `shouldHandleMessage` gate again immediately before handling; it is never rebound to a
  replacement listener. Closing the processor prevents later processing, while scheduling after
  closure throws `RejectedExecutionException` and follows the connection's shutdown-disconnect
  branch.
- Play listeners accept ordinary packets only while the connection is connected and
  `waitingForSwitchToConfig` is false. `switchToConfig` sets that flag before removing/saving the
  player and sending start-configuration; while waiting, only configuration acknowledgement is
  accepted. A queued ordinary play packet therefore contributes no mutation after disconnect or
  this transition, even if its first channel-thread gate previously passed.
- Ingress order is not a generic “main-queue order.” Most play handlers complete during the
  dedicated packet drain, but chat/command admission can enqueue a second ordinary server task:
  signed-command last-seen state is applied during packet handling, `tryHandleChat` then calls
  `MinecraftServer#execute`, and authoritative parsing/dispatch occurs when that later task runs.
  Async text filtering additionally retains the completion ordering documented by the chat family.
- Handshake, status, login authentication, liveness and unowned optional services are transport-only
  until a named admission or extension boundary explicitly joins gameplay state.
- Teleport IDs, block sequences, container state IDs, chat last-seen state and keepalive tokens are
  independent acknowledgement domains. Success in one never acknowledges another.
- Block-sequence acknowledgement is a high-water mark, not an inline transaction response.
  `handleUseItemOn` and `handleUseItem` advance it before several semantic rejection branches,
  while the block-action branches advance it after `ServerPlayerGameMode` returns. The listener
  emits one maximum `ClientboundBlockChangedAckPacket` at the start of a later listener tick; a
  negative sequence throws before updating the mark. Consequently a rejected use can still be
  acknowledged, multiple sequences coalesce, and correction packets are owner-branch-specific.
- A container state-ID mismatch does not reject the proposed click. After menu-ID, spectator/dead,
  menu-validity and slot gates, `handleContainerClick` executes `menu.clicked` against the current
  menu, installs the packet's hashed stacks only into remote mirrors, then selects full-state
  broadcast for a mismatched state ID or incremental broadcast for a match. Some earlier invalid
  menu/slot branches return without an immediate correction.
- Packet positions carry no dimension identity. Play handlers re-read `player.level()` when their
  server-thread attempt runs, so a dimension transfer first makes the same coordinates refer to the
  destination level rather than generically making them stale. Operator block handlers then call
  ordinary level block/entity access; `Level#getBlockEntity` reaches `getChunkAt`, which requests a
  `FULL` chunk, so there is no universal preloaded/active-chunk rejection rule.
- Configuration snapshot selection is field-specific. `startConfiguration` captures the current
  known-pack list and `LayeredRegistryAccess` in `SynchronizeRegistriesTask`; the task serializes
  registry data and tags only when the select-known-packs response is handled. Configuration finish
  separately binds play clientbound codecs from the then-current `server.registryAccess()` and uses
  the latest directly replaced client-information record. These points must not be collapsed into
  one atomic old/new reload snapshot.
- Disconnect removal closes the chat chain, removes the player and invokes the player persistence
  owner; only handler prefixes completed before that removal can reach that save. Clean server stop
  closes `PacketProcessor` before stopping connections, saving players and saving worlds, so queued
  ingress is not drained as a durability barrier. Transport queues, listeners, sequences, state IDs
  and acknowledgements are never replayed after reconnect or restart.
- `Mapped` means the complete serverbound inventory and its handoffs are explicit. It does not
  promote protocol-gated optional paths, command-root work, semantic leaves or cross-system joins.

## Regression procedure

Regenerate the locked packet report, require the surface validator to match all serverbound protocol
families exactly, then run every serverbound family's named protocol vectors. Instrument
`Connection#channelRead0`, `PacketProcessor#scheduleIfPossible/#processQueuedPackets`, the ordinary
server task queue, listener replacement and outbound sends, and run these independent vectors:

1. Admit two ordinary play packets, replace or close their captured listener between the first and
   second gates, and prove that neither packet is rebound; repeat with `PacketProcessor#close`
   immediately before clean save.
2. Send an unsigned command and an operator-block edit in both wire orders while gating the ordinary
   server task queue. Record packet-drain order, command-task enqueue/dispatch, the operator mutation,
   feedback and any durable prefix.
3. Send use/use-on/action sequences `n`, `n+1`, a rejected `n+2`, a lower repeat and `-1`; capture the
   high-water mark, correction branches and the single next-tick acknowledgement.
4. Click one current menu with matching and mismatching state IDs, then invalid menu ID, invalid
   menu lifetime and invalid slot. Record authoritative mutation, remote-mirror installation and
   full versus incremental sync.
5. Gate a dimension transfer and an operator position packet at the packet drain. Place different
   block entities at the same coordinates in both levels and test an initially unloaded destination
   chunk to distinguish destination interpretation and synchronous chunk resolution from rejection.
6. Gate successful reload publication before/after configuration start, known-pack response and
   finish. Record the offered pack list, serialized registry/tag data, final play codec registry
   access and latest client-information cookie independently.

Re-run the assigned cross-system joins before changing listener threading, acknowledgement state,
disconnect/save behavior, protocol transitions or reload publication order.
