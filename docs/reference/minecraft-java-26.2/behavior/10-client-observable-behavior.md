# 10 — Client Input, Prediction, Correction, and Presentation

This page includes client internals only through their observable semantics. Ferrite need not copy vanilla packets or renderer internals, but input-to-action, prediction-to-correction, and gameplay-event-to-presentation results must remain equivalent.

## `CLI-001` Client ticks and render frames are separate time domains

- **FidelityClass:** `EquivalentPlayerVisibleBehavior`
- **Evidence status:** `Confirmed`
- **Primary evidence:** `OFF-CLIENT-001`; `net.minecraft.client.Minecraft#tick()`; `net.minecraft.client.renderer.GameRenderer#tick()`; `net.minecraft.client.renderer.GameRenderer#render(net.minecraft.client.DeltaTracker,boolean)`; `net.minecraft.client.multiplayer.ClientLevel#tick(java.util.function.BooleanSupplier)`; `net.minecraft.client.multiplayer.ClientLevel#tickEntities()`
- **Applies when:** A client is in a world, whether or not FPS equals logical tick rate.
- **Behavior and timing:** `Minecraft#tick` advances discrete client input, game mode, client level, entities, UI, and sound. `GameRenderer#render` may run zero, one, or many times between client ticks and interpolates through `DeltaTracker`. A render frame must not commit server gameplay state or make per-frame animation change attack/use cooldown.
- **Boundaries and quirks:** Pause menus, focus loss, resource reload, and client stalls can gate client-world ticking or change partial tick; a dedicated server advances independently.
- **Verification owner (`CLI-PREDICT-001`; `EXP-CLI-*`):** Ferrite must define an equivalent pause/focus matrix and interpolation reset points rather than reuse the same main-loop implementation.

## `CLI-002` Raw key/mouse events update state; client ticks consume gameplay actions

- **FidelityClass:** `ExactObservableBehavior`
- **Evidence status:** `Confirmed`
- **Primary evidence:** `OFF-CLIENT-001`; `net.minecraft.client.KeyboardHandler#keyPress(long,int,net.minecraft.client.input.KeyEvent)`; `net.minecraft.client.KeyboardHandler#tick()`; `net.minecraft.client.MouseHandler#onMove(long,double,double)`; `net.minecraft.client.MouseHandler#turnPlayer(double)`; `net.minecraft.client.Minecraft#handleKeybinds()`; `net.minecraft.client.Minecraft#startAttack()`; `net.minecraft.client.Minecraft#continueAttack(boolean)`; `net.minecraft.client.Minecraft#startUseItem()`
- **Applies when:** The window receives keyboard/mouse events and screen, focus, player state, and key mappings permit gameplay input.
- **Behavior and timing:** Event callbacks update click counts/pressed state and accumulated mouse delta; a client tick consumes them through screen capture, spectator, cooldown, and player-state gates. An attack edge calls `startAttack`, held attack maintains breaking through `continueAttack`, and use calls `startUseItem` under press/repeat gates. Mouse look applies accumulated delta to player rotation rather than sending a gameplay action for every OS event.
- **Boundaries and quirks:** UI may consume the same key; focus loss must release/clear stuck input. Simultaneous attack/use, touchscreen mode, and continuous item use add exclusion branches.
- **Verification owner (`CLI-PREDICT-001`; `EXP-CLI-*`):** Extract all keybind priority, repeat counts, focus-release behavior, and screen pass-through into an automated state table.

## `CLI-003` Block actions are sequence-predicted and converge on server-verified state

- **FidelityClass:** `ExactObservableBehavior`
- **Evidence status:** `Confirmed`
- **Primary evidence:** `OFF-CLIENT-001`; `net.minecraft.client.multiplayer.MultiPlayerGameMode#startPrediction(net.minecraft.client.multiplayer.ClientLevel,net.minecraft.client.multiplayer.prediction.PredictiveAction)`; `net.minecraft.client.multiplayer.prediction.BlockStatePredictionHandler#startPredicting()`; `net.minecraft.client.multiplayer.prediction.BlockStatePredictionHandler#retainKnownServerState(net.minecraft.core.BlockPos,net.minecraft.world.level.block.state.BlockState,net.minecraft.client.player.LocalPlayer)`; `net.minecraft.client.multiplayer.prediction.BlockStatePredictionHandler#updateKnownServerState(net.minecraft.core.BlockPos,net.minecraft.world.level.block.state.BlockState)`; `net.minecraft.client.multiplayer.prediction.BlockStatePredictionHandler#endPredictionsUpTo(int,net.minecraft.client.multiplayer.ClientLevel)`; `net.minecraft.client.multiplayer.ClientLevel#setServerVerifiedBlockState(net.minecraft.core.BlockPos,net.minecraft.world.level.block.state.BlockState,int)`
- **Applies when:** The client predicts that placement, destruction, or use-on may mutate a local block.
- **Behavior and timing:** Each predictive action takes an increasing sequence. Within its prediction scope, the first local write retains known server state and player position, then the client immediately displays its prediction and sends the sequenced action. Server block updates first update retained records; acknowledgement closes predictions through that sequence and synchronizes final server-verified state into the client world, correcting player position when needed to avoid trapping it in restored collision.
- **Boundaries and quirks:** Multiple unacknowledged actions can touch one position. An old server update must neither blindly overwrite a newer prediction nor be ignored forever. Chunk unload, teleport, and dimension change must clear/rebase prediction records.
- **Verification owner (`CLI-PREDICT-001`; `EXP-CLI-*`):** Use a latency/reordering proxy for multiple same-position sequences, rejection, block entities, and exact player-correction convergence order.

## `CLI-004` The client selectively sends movement state; server correction has an acknowledgement boundary

- **FidelityClass:** `ExactObservableBehavior`
- **Evidence status:** `Cross-checked`
- **Primary evidence:** `OFF-CLIENT-001`; `OFF-SERVER-001`; `net.minecraft.client.player.LocalPlayer#tick()`; `net.minecraft.client.player.LocalPlayer#sendPosition()`; `net.minecraft.client.player.LocalPlayer#sendIsSprintingIfNeeded()`; `net.minecraft.client.multiplayer.ClientPacketListener#handleMovePlayer(net.minecraft.network.protocol.game.ClientboundPlayerPositionPacket)`; `net.minecraft.world.entity.Entity#teleportSetPosition(net.minecraft.world.entity.PositionMoveRotation,java.util.Set)`
- **Applies when:** The local player predicts movement/rotation/ground/sprint state or receives server position correction.
- **Behavior and timing:** Each local-player tick compares current and last-sent state, choosing position+rotation, position-only, rotation-only, or on-ground updates, with a periodic heartbeat; sprint state uses a separate change notification. Server correction carries absolute/relative components and a teleport sequence. The client applies it, clears related prediction, acknowledges, and resumes local simulation from the authoritative baseline.
- **Boundaries and quirks:** Movement before teleport acknowledgement, vehicle movement, flight/swim pose, and tiny floating-point changes have dedicated gates. Render smoothing may conceal a snap, but collision and interaction must immediately use corrected state.
- **Verification owner (`CLI-PREDICT-001`; `EXP-CLI-*`):** Extract send epsilon, heartbeat period, relative flags, unacknowledged-movement policy, and vehicle branches. This remains `Cross-checked`.

## `CLI-005` UI may be optimistic, but server menu content and state ID overwrite it

- **FidelityClass:** `ExactObservableBehavior`
- **Evidence status:** `Confirmed`
- **Primary evidence:** `OFF-CLIENT-001`; `OFF-SERVER-001`; `net.minecraft.client.multiplayer.MultiPlayerGameMode#handleContainerInput(int,int,int,net.minecraft.world.inventory.ContainerInput,net.minecraft.world.entity.player.Player)`; `net.minecraft.client.multiplayer.ClientPacketListener#handleContainerSetSlot(net.minecraft.network.protocol.game.ClientboundContainerSetSlotPacket)`; `net.minecraft.client.multiplayer.ClientPacketListener#handleContainerContent(net.minecraft.network.protocol.game.ClientboundContainerSetContentPacket)`; `net.minecraft.client.multiplayer.ClientPacketListener#handleContainerSetData(net.minecraft.network.protocol.game.ClientboundContainerSetDataPacket)`; `net.minecraft.client.multiplayer.ClientPacketListener#handleContainerClose(net.minecraft.network.protocol.game.ClientboundContainerClosePacket)`
- **Applies when:** A menu is open and a player manipulates a slot, or the server sends slot/content/data/close updates.
- **Behavior and timing:** The client runs the same menu click state machine for immediate feedback and sends state ID plus hashes of predicted changed slots/cursor. The server executes even a stale-state click, then chooses a full snapshot instead of deltas. Server single-slot, cursor, full-content and data-slot responses overwrite matching client menu state. The exact replay, 15-bit state counter, all registered menu routes, controls and close disposition are in `ITM-CONTAINER-*`.
- **Boundaries and quirks:** Player inventory container ID `0` and the current open menu have separate application paths; a slot/content update for another nonzero menu ID is ignored. Prediction hashes alter only the server's remote comparison baseline. Recipe-book, progress-bar and ghost-result visuals are derived and cannot commit item truth.
- **Verification owner (`CLI-UI-001`; `EXP-CLI-002`):** The remaining owner is client gesture production and presentation timing: mouse/touch mappings, double-click threshold, drag cancellation, cross-menu delayed clientbound packets and close-screen behavior.

## `CLI-006` Sounds, particles, level events, and damage cues present committed outcomes

- **FidelityClass:** `EquivalentPlayerVisibleBehavior`
- **Evidence status:** `Confirmed`
- **Primary evidence:** `OFF-CLIENT-001`; `net.minecraft.client.multiplayer.ClientPacketListener#handleSoundEvent(net.minecraft.network.protocol.game.ClientboundSoundPacket)`; `net.minecraft.client.multiplayer.ClientPacketListener#handleSoundEntityEvent(net.minecraft.network.protocol.game.ClientboundSoundEntityPacket)`; `net.minecraft.client.multiplayer.ClientPacketListener#handleParticleEvent(net.minecraft.network.protocol.game.ClientboundLevelParticlesPacket)`; `net.minecraft.client.multiplayer.ClientPacketListener#handleLevelEvent(net.minecraft.network.protocol.game.ClientboundLevelEventPacket)`; `net.minecraft.client.multiplayer.ClientPacketListener#handleDamageEvent(net.minecraft.network.protocol.game.ClientboundDamageEventPacket)`; `net.minecraft.client.multiplayer.ClientLevel#playLocalSound(double,double,double,net.minecraft.sounds.SoundEvent,net.minecraft.sounds.SoundSource,float,float,boolean)`; `net.minecraft.client.multiplayer.ClientLevel#addParticle(net.minecraft.core.particles.ParticleOptions,double,double,double,double,double,double)`
- **Applies when:** The server broadcasts a gameplay event or the client emits permitted local feedback for a predicted action.
- **Behavior and timing:** Client-thread handlers turn events into positioned/entity-bound sounds, particle batches, level events, or damage presentation. Instantiation respects resources, distance, sound category, particle setting, and budget. Presentation may interpolate or lag, but must not apply damage, drops, or block mutation again. A rejected prediction cannot retain a persistent effect that misrepresents gameplay state.
- **Boundaries and quirks:** A local sound can avoid round trip, so a later server broadcast needs duplicate-avoidance semantics. Missing resources, distant sounds, and reduced particles may drop presentation instances while critical gameplay state still needs other feedback.
- **Verification owner (`CLI-EFFECT-001`, `ITM-ENDER-CHEST-001`, `ITM-BARREL-001`, `ITM-BOOKSHELF-001`, `ITM-JUKEBOX-001`; `EXP-CLI-003`, `EXP-ITM-008`, `EXP-ITM-009`, `EXP-ITM-010`, `EXP-ITM-011`):** Container leaves fix ender-chest presentation, barrel/bookshelf sounds and jukebox level-event sound/HUD/parrot state. Classify every remaining gameplay event as required, settings-droppable, or prediction-deduplicated, then record a vanilla client to verify relative tick and duplicate suppression.
