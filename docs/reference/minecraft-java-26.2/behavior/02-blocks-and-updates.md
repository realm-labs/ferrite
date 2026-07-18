# 02 — Block States, Mutation, and Updates

This page defines generic block machinery. Read the properties, default states, and collision/outline shapes of roughly 1,196 blocks from `OFF-REPORT-001` and `OFF-DATA-001`.

## `BLK-001` A block state combines a block type with finite property values

- **FidelityClass:** `ExactObservableBehavior`
- **Evidence status:** `Confirmed`
- **Primary evidence:** `OFF-REPORT-001`; `OFF-DATA-001`; `net.minecraft.world.level.block.state.BlockState`; `net.minecraft.world.level.block.state.StateHolder#setValue(net.minecraft.world.level.block.state.properties.Property,java.lang.Comparable)`; `COM-WIKI-BLK-001`
- **Applies when:** A world position stores or queries a block.
- **Behavior and timing:** A position's primary state is a registered block type plus one allowed discrete property combination. Registration and `reports/blocks.json` lock the default state, legal property values, and state combinations. Runtime code must not smuggle in an unrepresentable value.
- **Boundaries and quirks:** Block-entity data, scheduled ticks, fluid state, and persistent components are not ordinary block-state properties even when players regard them as part of “the same block.”
- **Verification owner (`BLK-UPDATE-001`; `EXP-BLK-*`):** Ferrite's importer should schema-test every reported state instead of hand-writing state IDs.

## `BLK-002` Placement and breaking are server-validated state transactions

- **FidelityClass:** `ExactObservableBehavior`
- **Evidence status:** `Confirmed`
- **Primary evidence:** `OFF-SERVER-001`; `net.minecraft.world.item.BlockItem#place(net.minecraft.world.item.context.BlockPlaceContext)`; `net.minecraft.world.item.BlockItem#getPlacementState(net.minecraft.world.item.context.BlockPlaceContext)`; `net.minecraft.world.item.BlockItem#canPlace(net.minecraft.world.item.context.BlockPlaceContext,net.minecraft.world.level.block.state.BlockState)`; `net.minecraft.server.level.ServerPlayerGameMode#handleBlockBreakAction(net.minecraft.core.BlockPos,net.minecraft.network.protocol.game.ServerboundPlayerActionPacket$Action,net.minecraft.core.Direction,int,int)`; `net.minecraft.server.level.ServerPlayerGameMode#destroyBlock(net.minecraft.core.BlockPos)`
- **Applies when:** A player attempts to place a block item or completes break progress.
- **Behavior and timing:** Placement builds a context-dependent candidate state, then validates replaceability, collision/survival, and permissions. Success writes the state, invokes placement callbacks, consumes or updates the item, and synchronizes the result. Breaking is tracked by the server for progress, reach, and permissions. On success it runs the pre-destroy callback, removes the state, and chooses drops and post-destroy callbacks from tool, mode, and block logic.
- **Boundaries and quirks:** The client may predict animation or state, but a server rejection must restore authoritative blocks and related slots. Creative mode, adventure restrictions, block entities, and two-block structures add branches.
- **Verification owner (`BLK-UPDATE-001`; `EXP-BLK-*`):** Add black-box cases for each placement failure class, prediction rollback, two-block state, and block-entity drop ordering.

## `BLK-003` Mutation flags select the follow-up work

- **FidelityClass:** `ExactObservableBehavior`
- **Evidence status:** `Confirmed`
- **Primary evidence:** `OFF-SERVER-001`; `net.minecraft.world.level.Level#setBlock(net.minecraft.core.BlockPos,net.minecraft.world.level.block.state.BlockState,int,int)`; `net.minecraft.world.level.Level#updateNeighborsAt(net.minecraft.core.BlockPos,net.minecraft.world.level.block.Block,net.minecraft.world.level.redstone.Orientation)`; `net.minecraft.world.level.Level#neighborShapeChanged(net.minecraft.core.Direction,net.minecraft.core.BlockPos,net.minecraft.core.BlockPos,net.minecraft.world.level.block.state.BlockState,int,int)`; `net.minecraft.world.level.redstone.CollectingNeighborUpdater#shapeUpdate(net.minecraft.core.Direction,net.minecraft.world.level.block.state.BlockState,net.minecraft.core.BlockPos,net.minecraft.core.BlockPos,int,int)`
- **Applies when:** Any system writes world state, not only a player action.
- **Behavior and timing:** A state write's flag mask separately selects neighbor notification, client synchronization/rendering, shape update, and suppression branches. A shape update may return a new neighbor state and cause further updates; ordinary notification invokes the receiver's `neighborChanged`. Ferrite must carry notification intent with each write instead of broadcasting one universal update.
- **Boundaries and quirks:** A maximum update depth limits pathological recursion. Suppressed-update worldgen/structure writes must not be confused with normal gameplay writes.
- **Verification owner (`BLK-UPDATE-001`; `EXP-BLK-*`):** Lock a matrix of every flag bit and combination against client synchronization, drops, and comparator updates.

## `BLK-004` A collector runs neighbor updates as ordered work

- **FidelityClass:** `ExactObservableBehavior`
- **Evidence status:** `Cross-checked`
- **Primary evidence:** `OFF-SERVER-001`; `net.minecraft.world.level.redstone.CollectingNeighborUpdater#neighborChanged(net.minecraft.world.level.block.state.BlockState,net.minecraft.core.BlockPos,net.minecraft.world.level.block.Block,net.minecraft.world.level.redstone.Orientation,boolean)`; `net.minecraft.world.level.redstone.CollectingNeighborUpdater#addAndRun(net.minecraft.core.BlockPos,net.minecraft.world.level.redstone.CollectingNeighborUpdater$NeighborUpdates)`; `net.minecraft.world.level.redstone.CollectingNeighborUpdater#runUpdates()`; `COM-WIKI-BLK-001`
- **Applies when:** A state change triggers one or more shape or neighbor notifications.
- **Behavior and timing:** The outer notification starts an update chain. Work added inside the chain is ordered and iterated by the collector instead of consuming an unbounded Java call stack. Callbacks can write more blocks and append work, making enqueue order, direction order, and the depth cap observable.
- **Boundaries and quirks:** Equal final steady state is not a substitute for compatible update order; redstone, pistons, attached blocks, and drops expose the difference.
- **Verification owner (`BLK-UPDATE-001`; `EXP-BLK-*`):** GameTests have not yet locked six-direction order, front/back insertion for nested chains, or over-depth handling, so the rule remains `Cross-checked`.

## `BLK-005` Block events queue and execute before the entity phase

- **FidelityClass:** `ExactObservableBehavior`
- **Evidence status:** `Confirmed`
- **Primary evidence:** `OFF-SERVER-001`; `net.minecraft.world.level.Level#blockEvent(net.minecraft.core.BlockPos,net.minecraft.world.level.block.Block,int,int)`; `net.minecraft.server.level.ServerLevel#runBlockEvents()`; `net.minecraft.world.level.block.state.BlockBehaviour$BlockStateBase#triggerEvent(net.minecraft.world.level.Level,net.minecraft.core.BlockPos,int,int)`
- **Applies when:** A piston, note block, or similar block submits an event ID and parameter to the world queue.
- **Behavior and timing:** The event does not execute directly at the `blockEvent` call. The server drains it after scheduled block/fluid ticks and chunk-source work for that dimension, but before entity ticks. Execution rereads the target state; the current block's `triggerEvent` decides whether to handle and broadcast it.
- **Boundaries and quirks:** If the target is replaced before execution, the old event must not blindly act on an old object. An event queued by an event callback may enter a later drain round and must follow queue semantics.
- **Verification owner (`BLK-UPDATE-001`; `EXP-BLK-*`):** Lock the round behavior for target replacement in the same tick and for an event queuing another event.

## `BLK-006` Falling blocks schedule first, then become entities

- **FidelityClass:** `ExactObservableBehavior`
- **Evidence status:** `Confirmed`
- **Primary evidence:** `OFF-SERVER-001`; `net.minecraft.world.level.block.FallingBlock#onPlace(net.minecraft.world.level.block.state.BlockState,net.minecraft.world.level.Level,net.minecraft.core.BlockPos,net.minecraft.world.level.block.state.BlockState,boolean)`; `net.minecraft.world.level.block.FallingBlock#updateShape(net.minecraft.world.level.block.state.BlockState,net.minecraft.world.level.LevelReader,net.minecraft.world.level.ScheduledTickAccess,net.minecraft.core.BlockPos,net.minecraft.core.Direction,net.minecraft.core.BlockPos,net.minecraft.world.level.block.state.BlockState,net.minecraft.util.RandomSource)`; `net.minecraft.world.level.block.FallingBlock#tick(net.minecraft.world.level.block.state.BlockState,net.minecraft.server.level.ServerLevel,net.minecraft.core.BlockPos,net.minecraft.util.RandomSource)`; `net.minecraft.world.level.block.FallingBlock#getDelayAfterPlace()`; `net.minecraft.world.entity.item.FallingBlockEntity#fall(net.minecraft.world.level.Level,net.minecraft.core.BlockPos,net.minecraft.world.level.block.state.BlockState)`
- **Applies when:** A state with `FallingBlock` behavior is placed or receives a shape update and the block below is passable.
- **Behavior and timing:** Placement/shape update schedules a block tick after `2` game ticks. When due, if the block below is air, fire, fluid, or replaceable and the position is not below the world's minimum height, the source becomes a `FallingBlockEntity`. Entity physics then moves it; on landing it attempts to place the state, otherwise enters break/drop handling.
- **Boundaries and quirks:** Chunk unloading in flight, moving pistons, world border, block-entity data, and a non-replaceable landing position affect the result. Damage and anvil degradation are parameterized by concrete subclasses.
- **Verification owner (`BLK-UPDATE-001`; `EXP-BLK-*`):** Add GameTests for unload at a chunk edge, support returning in the same tick, and an occupied landing site; reproduction follows observed results.

## `BLK-007` Block entities have a separate ticker and invalidation lifecycle

- **FidelityClass:** `ExactObservableBehavior`
- **Evidence status:** `Confirmed`
- **Primary evidence:** `OFF-SERVER-001`; `net.minecraft.world.level.chunk.LevelChunk#addAndRegisterBlockEntity(net.minecraft.world.level.block.entity.BlockEntity)`; `net.minecraft.world.level.Level#tickBlockEntities()`; `net.minecraft.world.level.block.entity.BlockEntity#setRemoved()`
- **Applies when:** A block state corresponds to a block entity with dynamic data or a server ticker.
- **Behavior and timing:** State mutation and block-entity object registration are related but distinct lifecycle operations. A valid ticker runs in the dimension's block-entity phase after entity ticking. When removed or no longer compatible with its state, it must be invalidated and cannot keep mutating the world.
- **Boundaries and quirks:** A chunk not yet ready may hold pending block entities; loading and first tick must not double-register them. A client block-entity ticker does not own server gameplay truth.
- **Verification owner (`BLK-UPDATE-001`; `EXP-BLK-*`):** Lock same-tick block-entity replacement, creating a ticker from a callback, and the last eligible tick during unload.
