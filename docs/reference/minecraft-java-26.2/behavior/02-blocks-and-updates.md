# 02 — Block States, Mutation, and Updates

This page defines generic block machinery. Read the properties, default states, and
collision/outline shapes of roughly 1,196 blocks from `OFF-REPORT-001` and `OFF-DATA-001`.

## `BLK-001` A block state combines a block type with finite property values

- **FidelityClass:** `ExactObservableBehavior`
- **Evidence status:** `Confirmed`

### Primary evidence

`OFF-REPORT-001`; `OFF-DATA-001`; `net.minecraft.world.level.block.state.BlockState`;
`net.minecraft.world.level.block.state.StateHolder#setValue(net.minecraft.world.level.block.state.properties.Property,java.lang.Comparable)`;
`COM-WIKI-BLK-001`

### Applies when

A world position stores or queries a block.

### Behavior and timing

A position's primary state is a registered block type plus one allowed discrete property
combination. Registration and `reports/blocks.json` lock the default state, legal property values,
and state combinations. Runtime code must not smuggle in an unrepresentable value.

### Boundaries and quirks

Block-entity data, scheduled ticks, fluid state, and persistent components are not ordinary
block-state properties even when players regard them as part of “the same block.”

### Verification

**Owners:** `BLK-STATE-001`, `BLK-COPPER-GOLEM-STATUE-001`, `BLK-BELL-001`,
`BLK-ENCHANTING-TABLE-001`, `BLK-LECTERN-001`, `BLK-BANNER-001`, `BLK-SHELF-001`,
`BLK-DECORATED-POT-001`; state vectors in
`EXP-BLK-001`, `EXP-BLK-008`, `EXP-BLK-009`, `EXP-BLK-010`, `EXP-BLK-011`, `EXP-BLK-012` and
`EXP-BLK-013` and `EXP-BLK-014`

The generic leaf fixes strict runtime transitions, lenient item-component patches, canonical
identity and exhaustive report-schema checks; content leaves exhaust their exact state/component
projections, including lectern divergence, all 320 banner states and all 768 shelf states.

## `BLK-002` Placement and breaking are server-validated state transactions

- **FidelityClass:** `ExactObservableBehavior`
- **Evidence status:** `Confirmed`

### Primary evidence

`OFF-SERVER-001`;
`net.minecraft.world.item.BlockItem#place(net.minecraft.world.item.context.BlockPlaceContext)`;
`net.minecraft.world.item.BlockItem#getPlacementState(net.minecraft.world.item.context.BlockPlaceContext)`;
`net.minecraft.world.item.BlockItem#canPlace(net.minecraft.world.item.context.BlockPlaceContext,net.minecraft.world.level.block.state.BlockState)`;
`net.minecraft.server.level.ServerPlayerGameMode#handleBlockBreakAction(net.minecraft.core.BlockPos,net.minecraft.network.protocol.game.ServerboundPlayerActionPacket$Action,net.minecraft.core.Direction,int,int)`;
`net.minecraft.server.level.ServerPlayerGameMode#destroyBlock(net.minecraft.core.BlockPos)`

### Applies when

A player attempts to place a block item or completes break progress.

### Behavior and timing

Placement builds a context-dependent candidate state, then validates replaceability,
collision/survival, and permissions. Success writes the state, invokes placement callbacks, consumes
or updates the item, and synchronizes the result. Breaking is tracked by the server for progress,
reach, and permissions. On success it runs the pre-destroy callback, removes the state, and chooses
drops and post-destroy callbacks from tool, mode, and block logic.

### Boundaries and quirks

The client may predict animation or state, but a server rejection must restore authoritative blocks
and related slots. Creative mode, adventure restrictions, block entities, and two-block structures
add branches.

### Verification owners

`BLK-PLACE-001` and `EXP-BLK-001` specify placement admission, interaction precedence, every
block-item dispatch family, multi-position partial commits, components and side effects.
`BLK-COPPER-GOLEM-STATUE-001`/`EXP-BLK-008`, `BLK-BELL-001`/`EXP-BLK-009`,
`BLK-ENCHANTING-TABLE-001`/`EXP-BLK-010`, `BLK-LECTERN-001`/`EXP-BLK-011`,
`BLK-BANNER-001`/`EXP-BLK-012`, `BLK-SHELF-001`/`EXP-BLK-013`, and
`BLK-DECORATED-POT-001`/`EXP-BLK-014` own their exact placement,
component, loot and subtype edges. `BLK-BREAK-001`, `BLK-BREAKING-001` and `EXP-BLK-004` specify the
generic breaking state machine and harvest transaction. `BLK-BREAK-HOOK-001`,
`BLK-BREAK-CONTENT-001` and `EXP-BLK-005` exhaustively map and specify all 110 registered IDs with
concrete break-hook behavior; neither placement nor generic breaking completion may hide those
subtype transactions.

## `BLK-003` Mutation flags select the follow-up work

- **FidelityClass:** `ExactObservableBehavior`
- **Evidence status:** `Confirmed`

### Primary evidence

`OFF-SERVER-001`;
`net.minecraft.world.level.Level#setBlock(net.minecraft.core.BlockPos,net.minecraft.world.level.block.state.BlockState,int,int)`;
`net.minecraft.world.level.Level#updateNeighborsAt(net.minecraft.core.BlockPos,net.minecraft.world.level.block.Block,net.minecraft.world.level.redstone.Orientation)`;
`net.minecraft.world.level.Level#neighborShapeChanged(net.minecraft.core.Direction,net.minecraft.core.BlockPos,net.minecraft.core.BlockPos,net.minecraft.world.level.block.state.BlockState,int,int)`;
`net.minecraft.world.level.redstone.CollectingNeighborUpdater#shapeUpdate(net.minecraft.core.Direction,net.minecraft.world.level.block.state.BlockState,net.minecraft.core.BlockPos,net.minecraft.core.BlockPos,int,int)`

### Applies when

Any system writes world state, not only a player action.

### Behavior and timing

Ten independent bits select ordinary notification, client publication/render immediacy, generic
shape suppression, drops, piston semantics, redstone-wire shape handling, block-entity pre-removal
effects and `onPlace`. Storage, light/heightmaps and core BE compatibility occur before outer
publication/notification. Generic shape writes clear neighbor and suppress-drop bits and consume a
separate depth budget.

### Boundaries and quirks

The default shape budget is 512, independent of the neighbor collector's one-million work-item
limit. `setBlock=true` means an initial mutation was accepted even if a callback replaced the
requested state and suppressed the outer follow-ups.

### Verification

**Owners:** `BLK-UPDATE-001`, `BLK-COMMAND-001`, `SIM-COMMAND-LIMIT-001`,
`BLK-COMMAND-AREA-001`,
`BLK-COPPER-GOLEM-STATUE-001`,
`BLK-BELL-001`, `BLK-BANNER-001`, `BLK-SHELF-001`, `BLK-DECORATED-POT-001`, `BLK-VINE-001`;
`EXP-BLK-002`, `EXP-BLK-008`, `EXP-BLK-009`, `EXP-BLK-012`, `EXP-BLK-013`, `EXP-BLK-014`,
`EXP-BLK-015`, `EXP-BLK-017`, `EXP-BLK-018`, `EXP-SIM-006`

The generic leaf locks every bit value/named mask, phase order, abort semantics and limits; the
content leaves fix their flags-2/3/11/258/260/818 callers, ignored results, state-family retention and
bell/banner/shelf/pot nested writes. `BLK-VINE-001` fixes every flags-2 growth/support write and its
no-retry boundary. `BLK-COMMAND-AREA-001` fixes clone/fill mutation flags, ordered preclear/write/
neighbor phases and their partial-failure boundaries.

## `BLK-004` A collector runs neighbor updates as ordered work

- **FidelityClass:** `ExactObservableBehavior`
- **Evidence status:** `Confirmed`

### Primary evidence

`OFF-SERVER-001`;
`net.minecraft.world.level.redstone.CollectingNeighborUpdater#neighborChanged(net.minecraft.world.level.block.state.BlockState,net.minecraft.core.BlockPos,net.minecraft.world.level.block.Block,net.minecraft.world.level.redstone.Orientation,boolean)`;
`net.minecraft.world.level.redstone.CollectingNeighborUpdater#addAndRun(net.minecraft.core.BlockPos,net.minecraft.world.level.redstone.CollectingNeighborUpdater$NeighborUpdates)`;
`net.minecraft.world.level.redstone.CollectingNeighborUpdater#runUpdates()`; `COM-WIKI-BLK-001`

### Applies when

A state change triggers one or more shape or neighbor notifications.

### Behavior and timing

Ordinary receivers run `WEST,EAST,DOWN,UP,NORTH,SOUTH`; direct shapes run
`WEST,EAST,NORTH,SOUTH,DOWN,UP`. A multi item executes one direction at a time. Work added by that
callback runs FIFO as a child layer before the multi item resumes, giving depth-first layers without
recursive Java calls.

### Boundaries and quirks

The configurable cap counts submitted work items, not receiver callbacks; one six-side multi item
costs one. At the cap, all later submissions in that outer chain are discarded and the first
discarded position is logged.

### Verification

**Owners:** `BLK-UPDATE-001`, `BLK-BELL-001`; `EXP-BLK-002`, `EXP-BLK-009`

The generic leaf specifies reread/capture semantics, direction arrays, nested insertion and limits;
the bell leaf fixes one concrete neighbor-signal edge and opposing-support upgrade/downgrade
receiver.

## `BLK-005` Block events queue and execute before the entity phase

- **FidelityClass:** `ExactObservableBehavior`
- **Evidence status:** `Confirmed`

### Primary evidence

`OFF-SERVER-001`;
`net.minecraft.world.level.Level#blockEvent(net.minecraft.core.BlockPos,net.minecraft.world.level.block.Block,int,int)`;
`net.minecraft.server.level.ServerLevel#runBlockEvents()`;
`net.minecraft.world.level.block.state.BlockBehaviour$BlockStateBase#triggerEvent(net.minecraft.world.level.Level,net.minecraft.core.BlockPos,int,int)`

### Applies when

A piston, note block, or similar block submits an event ID and parameter to the world queue.

### Behavior and timing

Exact `(position, block, a, b)` records deduplicate in insertion order. The server drains after
chunk work and before entities, rereads the block, calls only a matching current type, and
broadcasts within 64 blocks only when the callback returns true. Inactive records are isolated and
reinserted after the drain.

### Boundaries and quirks

An active callback's newly queued event runs later in the same drain, with no cap. Because the
current record was already removed, an identical self-requeue is accepted and can loop indefinitely;
inactive isolation prevents that case from spinning.

### Verification

**Owners:** `BLK-UPDATE-001`, `BLK-BELL-001`, `BLK-DECORATED-POT-001`, `ENV-GEYSER-001`;
`EXP-BLK-002`, `EXP-BLK-009`, `EXP-BLK-014`, `EXP-ENV-005`

The generic leaf locks queue semantics; the content leaves fix bell and decorated-pot event-1
parameters/deduplication, bell cache/reset ordering, and the geyser producer/client eruption epoch.

## `BLK-006` Falling blocks schedule first, then become entities

- **FidelityClass:** `ExactObservableBehavior`
- **Evidence status:** `Confirmed`

### Primary evidence

`OFF-SERVER-001`; `net.minecraft.world.level.block.FallingBlock`;
`net.minecraft.world.entity.item.FallingBlockEntity`; `net.minecraft.world.level.block.AnvilBlock`;
`net.minecraft.world.level.block.ConcretePowderBlock`;
`net.minecraft.world.level.block.DragonEggBlock`; `net.minecraft.world.level.block.BrushableBlock`;
`net.minecraft.world.level.block.ScaffoldingBlock`;
`net.minecraft.world.level.entity.PersistentEntitySectionManager`

### Applies when

Sand, red sand, gravel, concrete powder, an anvil, a brushable block or scaffolding loses support,
or a dragon egg receives its delayed fall tick.

### Behavior and timing

Generic placement/shape changes schedule after `2` ticks (`5` for dragon eggs; `1` for scaffolding).
A due generic fall replaces the origin fluid-first and creates a state-carrying entity. Each active
entity tick applies gravity, collision and drag, then either places, transforms, waits above a
moving piston, breaks/drops, or times out. Concrete powder, anvils, brushable blocks and scaffolding
override distinct parts of this transaction.

### Boundaries and quirks

Entity admission failure does not restore the already removed origin. Generic creation does not
capture origin block-entity data. Unloaded entities are stored without advancing and resume through
UUID-guarded reload. The generic path has no world-border landing gate; only dragon-egg teleport
candidates test the border.

### Verification

**Owners:** `BLK-FALL-001`, `ENT-ENTITY-DROPS-001`; `EXP-BLK-003`, `EXP-ENT-006`

The leaf fixes every scheduled delay, transition order, subtype branch, timeout, persistence
boundary, damage formula and RNG draw; the experiment is a regression matrix rather than a
source-unknown owner.
`ENT-ENTITY-DROPS-001` fixes the live rule's three distinct landing/write-failure/timeout positions,
including the rule-off retry that cannot be inferred from a generic drop suppression.

## `BLK-007` Block entities have a separate ticker and invalidation lifecycle

- **FidelityClass:** `ExactObservableBehavior`
- **Evidence status:** `Confirmed`

### Primary evidence

`OFF-SERVER-001`;
`net.minecraft.world.level.chunk.LevelChunk#addAndRegisterBlockEntity(net.minecraft.world.level.block.entity.BlockEntity)`;
`net.minecraft.world.level.Level#tickBlockEntities()`;
`net.minecraft.world.level.block.entity.BlockEntity#setRemoved()`

### Applies when

A block state corresponds to a block entity with dynamic data or a server ticker.

### Behavior and timing

Compatible state changes reuse the object and rebind its existing ticker wrapper in place;
incompatible changes remove the listener/object and bind a null ticker before creating a
replacement. New wrappers join the active list unless creation occurs during BE iteration, when they
wait in a pending list for the next phase entry.

### Boundaries and quirks

Invalid wrappers are removed even while frozen. Subtype callbacks additionally require normal
gameplay, block activity, world-border inclusion, entities loaded and a currently compatible state.
Creation before the BE phase can tick the same server tick; creation inside a BE callback cannot.

### Verification owners

`BLK-UPDATE-001` and `EXP-BLK-002` lock generic lifecycle semantics.
`BLK-COMMAND-001`/`EXP-BLK-017` owns command-block reuse across packet-driven mode replacement,
its scheduled callbacks, persistence and update hooks.
`BLK-SPAWNER-001`/`EXP-BLK-016`, `BLK-TRIAL-SPAWNER-001`/`EXP-BLK-006`,
`BLK-VAULT-001`/`EXP-BLK-007`,
`BLK-COPPER-GOLEM-STATUE-001`/`EXP-BLK-008`, `BLK-BELL-001`/`EXP-BLK-009`,
`BLK-ENCHANTING-TABLE-001`/`EXP-BLK-010`, `BLK-LECTERN-001`/`EXP-BLK-011`,
`BLK-BANNER-001`/`EXP-BLK-012`, `BLK-SHELF-001`/`EXP-BLK-013`,
`BLK-DECORATED-POT-001`/`EXP-BLK-014`, and `ENV-GEYSER-001`/`EXP-ENV-005`
own concrete subtype transactions; other callbacks remain content-owned.
