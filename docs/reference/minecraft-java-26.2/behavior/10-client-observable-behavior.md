# 10 — Client Input, Prediction, Correction, and Presentation

This page includes client internals only through their observable semantics. Ferrite need not copy
vanilla packets or renderer internals, but input-to-action, prediction-to-correction, and
gameplay-event-to-presentation results must remain equivalent.

## `CLI-001` Client ticks and render frames are separate time domains

- **FidelityClass:** `EquivalentPlayerVisibleBehavior`
- **Evidence status:** `Confirmed`

### Primary evidence

`OFF-CLIENT-001`; `net.minecraft.client.Minecraft#tick()`;
`net.minecraft.client.renderer.GameRenderer#tick()`;
`net.minecraft.client.renderer.GameRenderer#render(net.minecraft.client.DeltaTracker,boolean)`;
`net.minecraft.client.multiplayer.ClientLevel#tick(java.util.function.BooleanSupplier)`;
`net.minecraft.client.multiplayer.ClientLevel#tickEntities()`

### Applies when

A client is in a world, whether or not FPS equals logical tick rate.

### Behavior and timing

`Minecraft#tick` advances discrete client input, game mode, client level, entities, UI, and sound.
`GameRenderer#render` may run zero, one, or many times between client ticks and interpolates through
`DeltaTracker`. A render frame must not commit server gameplay state or make per-frame animation
change attack/use cooldown.

### Boundaries and quirks

Pause menus, focus loss, resource reload, and client stalls can gate client-world ticking or change
partial tick; a dedicated server advances independently.

### Verification

**Owners:** `CLI-PREDICT-001`, `BLK-CONDUIT-001`, `BLK-BEACON-001`, `BLK-SIGN-001`,
`BLK-SKULL-001`; `EXP-CLI-*`, `EXP-BLK-023`, `EXP-BLK-024`, `EXP-BLK-025`, `EXP-BLK-026`

Ferrite must define an equivalent pause/focus matrix and interpolation reset points rather than
reuse the same main-loop implementation.
The conduit leaf fixes per-client-tick counters, frame/target particles and active rotation versus
partial-tick cage/eye rendering; none of those local clocks commits authoritative server state.
The skull leaf fixes its client-only dragon/piglin counter, freeze-on-unpower behavior and
partial-tick sampling; durable profile/sound/name data remains server-authoritative.
The beacon leaf fixes independent client beam scanning plus game-time/partial-tick beam animation;
the client derives sections and levels locally rather than receiving those runtime values.
The sign leaf fixes its client-local four-line preview and editor range/removal checks; preview
mutation never substitutes for the final server-authorized text commit.

## `CLI-002` Raw key/mouse events update state; client ticks consume gameplay actions

- **FidelityClass:** `ExactObservableBehavior`
- **Evidence status:** `Confirmed`

### Primary evidence

`OFF-CLIENT-001`;
`net.minecraft.client.KeyboardHandler#keyPress(long,int,net.minecraft.client.input.KeyEvent)`;
`net.minecraft.client.KeyboardHandler#tick()`;
`net.minecraft.client.MouseHandler#onMove(long,double,double)`;
`net.minecraft.client.MouseHandler#turnPlayer(double)`;
`net.minecraft.client.Minecraft#handleKeybinds()`; `net.minecraft.client.Minecraft#startAttack()`;
`net.minecraft.client.Minecraft#continueAttack(boolean)`;
`net.minecraft.client.Minecraft#startUseItem()`

### Applies when

The window receives keyboard/mouse events and screen, focus, player state, and key mappings permit
gameplay input.

### Behavior and timing

Event callbacks update click counts/pressed state and accumulated mouse delta; a client tick
consumes them through screen capture, spectator, cooldown, and player-state gates. An attack edge
calls `startAttack`, held attack maintains breaking through `continueAttack`, and use calls
`startUseItem` under press/repeat gates. Mouse look applies accumulated delta to player rotation
rather than sending a gameplay action for every OS event.

### Boundaries and quirks

UI may consume the same key; focus loss must release/clear stuck input. Simultaneous attack/use,
touchscreen mode, and continuous item use add exclusion branches.

### Verification

**Owners:** `CLI-PREDICT-001`; `EXP-CLI-*`

Extract all keybind priority, repeat counts, focus-release behavior, and screen pass-through into an
automated state table.

## `CLI-003` Block actions are sequence-predicted and converge on server-verified state

- **FidelityClass:** `ExactObservableBehavior`
- **Evidence status:** `Confirmed`

### Primary evidence

`OFF-CLIENT-001`;
`net.minecraft.client.multiplayer.MultiPlayerGameMode#startPrediction(net.minecraft.client.multiplayer.ClientLevel,net.minecraft.client.multiplayer.prediction.PredictiveAction)`;
`net.minecraft.client.multiplayer.prediction.BlockStatePredictionHandler#startPredicting()`;
`net.minecraft.client.multiplayer.prediction.BlockStatePredictionHandler#retainKnownServerState(net.minecraft.core.BlockPos,net.minecraft.world.level.block.state.BlockState,net.minecraft.client.player.LocalPlayer)`;
`net.minecraft.client.multiplayer.prediction.BlockStatePredictionHandler#updateKnownServerState(net.minecraft.core.BlockPos,net.minecraft.world.level.block.state.BlockState)`;
`net.minecraft.client.multiplayer.prediction.BlockStatePredictionHandler#endPredictionsUpTo(int,net.minecraft.client.multiplayer.ClientLevel)`;
`net.minecraft.client.multiplayer.ClientLevel#setServerVerifiedBlockState(net.minecraft.core.BlockPos,net.minecraft.world.level.block.state.BlockState,int)`

### Applies when

The client predicts that placement, destruction, or use-on may mutate a local block.

### Behavior and timing

Each predictive action takes an increasing sequence. Within its prediction scope, the first local
write retains known server state and player position, then the client immediately displays its
prediction and sends the sequenced action. Server block updates first update retained records;
acknowledgement closes predictions through that sequence and synchronizes final server-verified
state into the client world, correcting player position when needed to avoid trapping it in restored
collision.

### Boundaries and quirks

Multiple unacknowledged actions can touch one position. An old server update must neither blindly
overwrite a newer prediction nor be ignored forever. Chunk unload, teleport, and dimension change
must clear/rebase prediction records.

### Verification

**Owners:** `CLI-PREDICT-001`; `EXP-CLI-*`

Use a latency/reordering proxy for multiple same-position sequences, rejection, block entities, and
exact player-correction convergence order.

## `CLI-004` The client selectively sends movement state; server correction has an acknowledgement boundary

- **FidelityClass:** `ExactObservableBehavior`
- **Evidence status:** `Cross-checked`

### Primary evidence

`OFF-CLIENT-001`; `OFF-SERVER-001`; `net.minecraft.client.player.LocalPlayer#tick()`;
`net.minecraft.client.player.LocalPlayer#sendPosition()`;
`net.minecraft.client.player.LocalPlayer#sendIsSprintingIfNeeded()`;
`net.minecraft.client.multiplayer.ClientPacketListener#handleMovePlayer(net.minecraft.network.protocol.game.ClientboundPlayerPositionPacket)`;
`net.minecraft.world.entity.Entity#teleportSetPosition(net.minecraft.world.entity.PositionMoveRotation,java.util.Set)`

### Applies when

The local player predicts movement/rotation/ground/sprint state or receives server position
correction.

### Behavior and timing

Each local-player tick compares current and last-sent state, choosing position+rotation,
position-only, rotation-only, or on-ground updates, with a periodic heartbeat; sprint state uses a
separate change notification. Server correction carries absolute/relative components and a teleport
sequence. The client applies it, clears related prediction, acknowledges, and resumes local
simulation from the authoritative baseline.

### Boundaries and quirks

Movement before teleport acknowledgement, vehicle movement, flight/swim pose, and tiny
floating-point changes have dedicated gates. Render smoothing may conceal a snap, but collision and
interaction must immediately use corrected state.

### Verification

**Owners:** `CLI-PREDICT-001`; `EXP-CLI-*`

Extract send epsilon, heartbeat period, relative flags, unacknowledged-movement policy, and vehicle
branches. This remains `Cross-checked`.

## `CLI-005` UI may be optimistic, but server menu content and state ID overwrite it

- **FidelityClass:** `ExactObservableBehavior`
- **Evidence status:** `Confirmed`

### Primary evidence

`OFF-CLIENT-001`; `OFF-SERVER-001`;
`net.minecraft.client.multiplayer.MultiPlayerGameMode#handleContainerInput(int,int,int,net.minecraft.world.inventory.ContainerInput,net.minecraft.world.entity.player.Player)`;
`net.minecraft.client.multiplayer.ClientPacketListener#handleContainerSetSlot(net.minecraft.network.protocol.game.ClientboundContainerSetSlotPacket)`;
`net.minecraft.client.multiplayer.ClientPacketListener#handleContainerContent(net.minecraft.network.protocol.game.ClientboundContainerSetContentPacket)`;
`net.minecraft.client.multiplayer.ClientPacketListener#handleContainerSetData(net.minecraft.network.protocol.game.ClientboundContainerSetDataPacket)`;
`net.minecraft.client.multiplayer.ClientPacketListener#handleContainerClose(net.minecraft.network.protocol.game.ClientboundContainerClosePacket)`

### Applies when

A menu is open and a player manipulates a slot, or the server sends slot/content/data/close updates.

### Behavior and timing

The client runs the same menu click state machine for immediate feedback and sends state ID plus
hashes of predicted changed slots/cursor. The server executes even a stale-state click, then chooses
a full snapshot instead of deltas. Server single-slot, cursor, full-content and data-slot responses
overwrite matching client menu state. The exact replay, 15-bit state counter, all registered menu
routes, controls and close disposition are in `ITM-CONTAINER-*`.

### Boundaries and quirks

Player inventory container ID `0` and the current open menu have separate application paths; a
slot/content update for another nonzero menu ID is ignored. Prediction hashes alter only the
server's remote comparison baseline. Recipe-book, progress-bar and ghost-result visuals are derived
and cannot commit item truth.

### Verification

**Owners:** `CLI-UI-001`, `BLK-LECTERN-001`, `BLK-JIGSAW-001`, `BLK-STRUCTURE-001`, `BLK-TEST-BLOCK-001`, `BLK-TEST-INSTANCE-001`,
`BLK-BEACON-001`, `BLK-SIGN-001`; `EXP-CLI-002`, `EXP-BLK-011`, `EXP-BLK-021`,
`EXP-BLK-022`, `EXP-BLK-024`, `EXP-BLK-025`, `EXP-BLK-027`, `EXP-BLK-028`

The lectern leaf fixes its one-slot/data menu, page/take controls and immediate broadcast boundary.
The jigsaw leaf fixes its non-menu local screen, identifier-only enablement, numeric fallbacks,
joint/level/keep controls and set-before-generate close transaction.
The structure-block leaf fixes its nonpausing mode-specific local screen, snapshot-only cancel,
parse fallbacks, alternate DATA selection and complete set-before-action operator packet.
The test-block leaf fixes its non-menu mode/message screen, hidden-but-retained start message,
128-code-unit canonical limit, single Done packet and packet-free cancel/close paths.
The test-instance leaf fixes its non-menu ID/size/rotation/entity controls, UI-only size and save
gates, set/action close paths, effective-rotation initialization and positionless/unsequenced status
responses that may update a later screen.
The beacon leaf joins the local tier buttons and Done-before-close path to synchronized three-value
menu data and authoritative payment/power validation; the protocol family owns exact wire order.
The sign leaf fixes ordinary/hanging editor selection, 90/60-pixel line admission, four-line
wrapping, automatic range closure and exactly one packet on screen removal; the protocol family
owns exact packet fields and asynchronous server filtering.
The remaining UI owner is client gesture production and presentation timing: mouse/touch mappings,
double-click threshold, drag cancellation, cross-menu delayed clientbound packets and close-screen
behavior.

## `CLI-006` Presentation packets expose committed outcomes and player-facing rule state

- **FidelityClass:** `EquivalentPlayerVisibleBehavior`
- **Evidence status:** `Confirmed`

### Primary evidence

`OFF-CLIENT-001`;
`net.minecraft.client.multiplayer.ClientPacketListener#handleSoundEvent(net.minecraft.network.protocol.game.ClientboundSoundPacket)`;
`net.minecraft.client.multiplayer.ClientPacketListener#handleSoundEntityEvent(net.minecraft.network.protocol.game.ClientboundSoundEntityPacket)`;
`net.minecraft.client.multiplayer.ClientPacketListener#handleParticleEvent(net.minecraft.network.protocol.game.ClientboundLevelParticlesPacket)`;
`net.minecraft.client.multiplayer.ClientPacketListener#handleLevelEvent(net.minecraft.network.protocol.game.ClientboundLevelEventPacket)`;
`net.minecraft.client.multiplayer.ClientPacketListener#handleDamageEvent(net.minecraft.network.protocol.game.ClientboundDamageEventPacket)`;
`net.minecraft.client.multiplayer.ClientPacketListener#handleLogin(net.minecraft.network.protocol.game.ClientboundLoginPacket)`;
`net.minecraft.client.multiplayer.ClientPacketListener#handleGameEvent(net.minecraft.network.protocol.game.ClientboundGameEventPacket)`;
`net.minecraft.client.multiplayer.ClientPacketListener#handlePlayerCombatKill(net.minecraft.network.protocol.game.ClientboundPlayerCombatKillPacket)`;
`net.minecraft.server.MinecraftServer#onGameRuleChanged`;
`net.minecraft.server.waypoints.ServerWaypointManager#addPlayer(net.minecraft.server.level.ServerPlayer)`;
`net.minecraft.server.waypoints.ServerWaypointManager#breakAllConnections()`;
`net.minecraft.client.multiplayer.ClientLevel#playLocalSound(double,double,double,net.minecraft.sounds.SoundEvent,net.minecraft.sounds.SoundSource,float,float,boolean)`;
`net.minecraft.client.multiplayer.ClientLevel#addParticle(net.minecraft.core.particles.ParticleOptions,double,double,double,double,double,double)`

### Applies when

The server broadcasts a gameplay event, the client emits permitted local feedback for a predicted
action, or the server snapshots or changes a player-facing game rule.

### Behavior and timing

Client-thread handlers turn events into positioned/entity-bound sounds, particle batches, level
events, or damage presentation. Instantiation respects resources, distance, sound category, particle
setting, and budget. Presentation may interpolate or lag, but must not apply damage, drops, or block
mutation again. A rejected prediction cannot retain a persistent effect that misrepresents gameplay
state.

### Boundaries and quirks

A local sound can avoid round trip, so a later server broadcast needs duplicate-avoidance semantics.
Missing resources, distant sounds, and reduced particles may drop presentation instances while
critical gameplay state still needs other feedback.

The `immediate_respawn` and `reduced_debug_info` rules snapshot into play login and project later
changes through game/entity events. `locator_bar` changes rebuild or clear each level's waypoint
connections. These presentation choices neither bypass authoritative respawn admission nor mutate
entity/team/location truth.

### Verification

**Owners:** `CLI-EFFECT-001`, `CLI-PLAYER-RULE-001`, `ITM-ENDER-CHEST-001`, `ITM-BARREL-001`, `ITM-BOOKSHELF-001`,
`ITM-JUKEBOX-001`, `BLK-COPPER-GOLEM-STATUE-001`, `BLK-BELL-001`, `BLK-ENCHANTING-TABLE-001`,
`BLK-LECTERN-001`, `BLK-BANNER-001`, `BLK-SHELF-001`, `BLK-DECORATED-POT-001`,
`BLK-BRUSHABLE-001`, `BLK-SCULK-SENSOR-001`, `BLK-JIGSAW-001`, `BLK-STRUCTURE-001`, `BLK-STRUCTURE-VOID-001`, `BLK-AIR-001`, `BLK-BEDROCK-001`, `BLK-REINFORCED-DEEPSLATE-001`, `BLK-TINTED-GLASS-001`, `BLK-TEST-BLOCK-001`,
`BLK-CONDUIT-001`, `BLK-BEACON-001`, `BLK-SIGN-001`, `BLK-SKULL-001`, `ITM-HONEYCOMB-001`, `BLK-COMMAND-001`,
`CLI-COMMAND-FEEDBACK-001`, `SIM-COMMAND-LIMIT-001`,
`BLK-COMMAND-AREA-001`, `ENT-ENTITY-DROPS-001`, `ENV-GEYSER-001`, `MOB-RAID-001`;
`EXP-CLI-003`, `EXP-CLI-004`, `EXP-SIM-006`, `EXP-BLK-018`, `EXP-ENT-006`,
`EXP-ITM-008`, `EXP-ITM-009`, `EXP-ITM-010`, `EXP-ITM-011`, `EXP-BLK-008`, `EXP-BLK-009`,
`EXP-BLK-010`, `EXP-BLK-011`, `EXP-BLK-012`, `EXP-BLK-013`, `EXP-BLK-014`, `EXP-BLK-017`,
`EXP-BLK-019`, `EXP-BLK-020`, `EXP-BLK-021`, `EXP-BLK-022`, `EXP-BLK-023`, `EXP-BLK-024`,
`EXP-BLK-025`, `EXP-BLK-026`, `EXP-BLK-027`, `EXP-BLK-029`, `EXP-BLK-030`, `EXP-BLK-031`,
`EXP-BLK-032`, `EXP-BLK-033`, `EXP-ITM-012`,
`EXP-ENV-005`, `EXP-MOB-011`

Concrete leaves fix container/statue/bell/table/lectern/banner/shelf/pot presentation and potent-sulfur
cadence. `CLI-PLAYER-RULE-001` fixes join/live projection for the three player-facing rules and
delegates packet codecs plus authoritative lifecycle to their existing owners. Classify every
remaining emission as required, settings-droppable, or prediction-deduplicated, then verify timing.
`BLK-COMMAND-AREA-001` fixes successful clone/fill feedback after block-side effects and the
fill-biome dirty/resend boundary before command feedback.
`MOB-RAID-001` fixes bossbar membership/progress/title, per-player horn packets and victory rewards.
`ENT-ENTITY-DROPS-001` fixes which sounds, entity-link updates, item entities, frame state and
ordinary entity tracking survive each of its seven differently placed live-rule reads.
`BLK-BRUSHABLE-001` fixes predicted stroke sound/dust, hidden-to-materialized item synchronization,
face-dependent item rendering, use-cycle models and completion event ordering.
`BLK-SCULK-SENSOR-001` fixes traveling/reloaded vibration particles, dry sensor clicks,
deterministic resonator pitches, ambient active particles and block-state-only client convergence.
`BLK-JIGSAW-001` fixes all 12 ordinary model orientations and seven-field block-entity-data
convergence without a dedicated renderer or menu-open packet.
`BLK-STRUCTURE-001` fixes four mode-selected cube models, its neutral item cube, complete entity-data
convergence and permission/mode/size-gated 96-distance boundary plus invisible-block rendering.
`BLK-STRUCTURE-VOID-001` fixes the absent ordinary world model, generated item texture and the
admitted structure-block renderer's opaque pale-red 0.1-center-cube outline.
`BLK-AIR-001` fixes the shared invisible/empty block projection for all three state IDs and the AIR
item's zero-count optional-stack projection despite its otherwise present particle-only item asset.
`BLK-BEDROCK-001` fixes state 85's four default-weight base/mirrored/Y-180 world variants, its
fixed base-model item projection and the server map-color fallback at minimum height.
`BLK-REINFORCED-DEEPSLATE-001` fixes state 32085's single top/side/bottom full-cube model and the
ordinary item projection that selects the same model without conditional or block-entity payload.
`BLK-TINTED-GLASS-001` fixes state 27161's single tinted `cube_all` model and the ordinary item
projection that selects the same model; server-side dampening remains independent of that model.
`BLK-TEST-BLOCK-001` fixes four state-selected cube models, mode/message/powered entity-data
convergence and its client-local edit UI; the transient trigger latch is never projected.
`BLK-TEST-INSTANCE-001` fixes its cube model, complete data/marker convergence, local edit UI,
status-colored 2048-high beacon beam, permission-gated bounds and permission-independent red
positional markers with always-on-top labels.
`BLK-CONDUIT-001` fixes independent 40-tick client water/frame derivation, exact frame/target
nautilus-particle RNG, target-only entity data, inactive shell versus active cage/wind/eye rendering
and the base-shell-only special item model. Retained targets may keep emitting particles while the
client renders the conduit inactive.
`BLK-BEACON-001` fixes incremental local color-section publication, level-gated visibility,
game-time beam animation, distance/scoping scale and the 2048-high final section, while its block
and item retain the ordinary three-part model.
`BLK-SIGN-001` fixes both-side ordinary/hanging text layout, filtered visibility, dye and glow
colors, full-bright and strict outline boundaries plus editor presentation. `ITM-HONEYCOMB-001`
fixes event-3003 wax particles/sound at one position or both halves of a copper chest.
`BLK-SKULL-001` fixes block/item special models, floor/wall transforms, fixed/profile textures,
asynchronous skin fallback and dragon/piglin moving-part formulas.
