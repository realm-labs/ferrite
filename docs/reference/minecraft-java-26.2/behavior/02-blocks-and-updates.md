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
`BLK-DECORATED-POT-001`, `BLK-BRUSHABLE-001`, `BLK-SCULK-SENSOR-001`, `BLK-JIGSAW-001`,
`BLK-STRUCTURE-001`, `BLK-STRUCTURE-VOID-001`, `BLK-AIR-001`, `BLK-BEDROCK-001`,
`BLK-REINFORCED-DEEPSLATE-001`, `BLK-TINTED-GLASS-001`, `BLK-GLASS-001`,
`BLK-STAINED-GLASS-001`, `BLK-CONCRETE-001`, `BLK-TERRACOTTA-001`,
`BLK-GLAZED-TERRACOTTA-001`, `BLK-QUARTZ-001`,
`BLK-SLIME-001`, `BLK-HONEY-001`, `BLK-SOUL-SAND-001`, `BLK-MAGMA-001`,
`BLK-LAVA-CAULDRON-001`, `BLK-TEST-BLOCK-001`, `BLK-CONDUIT-001`, `BLK-BEACON-001`, `BLK-SIGN-001`,
`BLK-SKULL-001`; state vectors in
`EXP-BLK-001`, `EXP-BLK-008`, `EXP-BLK-009`, `EXP-BLK-010`, `EXP-BLK-011`, `EXP-BLK-012` and
`EXP-BLK-013`, `EXP-BLK-014`, `EXP-BLK-019`, `EXP-BLK-020`, `EXP-BLK-021`, `EXP-BLK-022` and
`EXP-BLK-023`, `EXP-BLK-024`, `EXP-BLK-025`, `EXP-BLK-026`, `EXP-BLK-027`, `EXP-BLK-030`,
`EXP-BLK-031`, `EXP-BLK-032`, `EXP-BLK-033`, `EXP-BLK-034`, `EXP-BLK-035`, `EXP-BLK-036`,
`EXP-BLK-037`, `EXP-BLK-038`, `EXP-BLK-039`, `EXP-BLK-040`, `EXP-BLK-041`, `EXP-BLK-042`,
`EXP-BLK-043`

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
`BLK-JIGSAW-001`/`EXP-BLK-021` fixes its 12 clicked-face/front-top placement outcomes while the
generic game-master item and break owners retain permission and unbreakable-state admission.
`BLK-TEST-BLOCK-001`/`EXP-BLK-022` fixes component-selected start/log/fail/accept placement and
mode-preserving clone stacks under those same game-master gates.
`BLK-CONDUIT-001`/`EXP-BLK-023` fixes the unusual registered `waterlogged=true` default, ordinary
placement's full-water override, centered `6×6×6`-pixel shape, unconditional self loot and special shell
item projection; generic placement/break admission remains here.
`BLK-BEACON-001`/`EXP-BLK-024` fixes propertyless state 9980, strength/light/full-cube properties,
custom-name-only self loot and the ordinary block-model item; generic placement and breaking remain
here.
`BLK-SIGN-001`/`EXP-BLK-025` fixes 48 sign blocks and 1,344 states, context-ordered standing/wall/
ceiling/wall-hanging placement, solid/center/full-face support, attachment, waterlogging, chain
precedence and placement-time editor opening; generic block-item admission and breaking remain here.
`BLK-SKULL-001`/`EXP-BLK-026` fixes 14 floor/wall blocks and 280 states, context-ordered wall
selection, 16-segment floor rotation, exact shapes, support-free continuity and initial power;
generic standing/wall item admission and the surrounding placement transaction remain here.
`BLK-STRUCTURE-001`/`EXP-BLK-027` fixes its four mode states, load default, full cube properties,
game-master item gate and placement-time author copy; generic item placement and breaking remain here.
`BLK-STRUCTURE-VOID-001`/`EXP-BLK-029` fixes one replaceable no-collision/no-loot state, its
ordinary epic block item and centered six-voxel selection shape; generic place/break admission stays here.
`BLK-AIR-001`/`EXP-BLK-030` fixes three property-free `isAir` states with empty shape/collision,
destroy short-circuit and ordinary-air removal, plus the non-placeable AIR item sentinel; generic
state-write admission stays here.
`BLK-BEDROCK-001`/`EXP-BLK-031` fixes property-free state 85, destroy speed -1, zero continuous
progress, no loot, full-cube properties and its ordinary common block item; generic placement,
creative/explicit removal and client correction stay here.
`BLK-REINFORCED-DEEPSLATE-001`/`EXP-BLK-032` fixes property-free state 32085, destroy speed 55,
the exact progress divisor, empty loot, full-cube properties and ordinary creative-tab block item;
generic placement, break-session control, removal and client correction stay here.
`BLK-TINTED-GLASS-001`/`EXP-BLK-033` fixes property-free state 27161, glass-derived strength,
full collision but empty visual shape, self loot without Silk Touch and the ordinary two-count recipe;
generic placement, break/explosion resolution, state publication and correction stay here.
`BLK-GLASS-001`/`EXP-BLK-034` fixes property-free state 562, strength 0.3, full collision but empty
visual shape, Silk Touch-only self loot and the tagged smelting recipe; generic placement,
break/explosion resolution, state publication and correction stay here.
`BLK-STAINED-GLASS-001`/`EXP-BLK-040` fixes sixteen property-free states 7098..7113, matching dye
map colors, strength 0.3, full collision but empty visual shape, Silk Touch-only self loot and
coloring/pane recipes; generic placement, break/explosion resolution and publication stay here.
`BLK-CONCRETE-001`/`EXP-BLK-041` fixes sixteen property-free states 15030..15045, matching dye map
colors, ordinary full-solid shape/predicates, strength 1.8, correct-tool harvest, explosion-survival
self loot and exact paired powder-solidification targets. Powder scheduling/falling and generic
placement, break/explosion resolution, publication and correction stay with their existing owners.
`BLK-TERRACOTTA-001`/`EXP-BLK-042` fixes plain state 12912 and sixteen property-free dyed states
11444..11459, plain-orange or terracotta-dye map colors, ordinary full-solid predicates, strength
1.25/4.2 and correct-tool/explosion-survival self loot. Generic placement, break/explosion
resolution, publication and external replacement stay with their existing owners.
`BLK-GLAZED-TERRACOTTA-001`/`EXP-BLK-043` fixes sixteen four-facing state groups 14966..15029,
opposite-horizontal placement, inherited rotation/mirror, ordinary dye map colors, full-solid
strength 1.4, correct-tool/explosion-survival self loot and push-only piston reaction. Generic
placement, state-component parsing, breaking, piston execution and correction remain with their
existing owners.
`BLK-QUARTZ-001`/`EXP-BLK-044` fixes four property-free full quartz states and pillar
`axis=x/y/z` states 11325..11327, clicked-face axis placement, quarter-turn x/z swaps, the
0.8/0.8 ordinary-family versus 2.0/6.0 smooth-quartz strength split, and correct-tool
explosion-survival self loot. Generic placement, component parsing, breaking and publication
remain with their existing owners; quartz stairs/slabs remain with `shape-family`.
`BLK-SLIME-001`/`EXP-BLK-035` fixes property-free state 12532, zero strength, full shapes,
friction/restitution, dampening 1, explosion-surviving self loot and reversible storage recipes;
generic placement, instant break, loot evaluation, state publication and correction stay here.
`BLK-HONEY-001`/`EXP-BLK-036` fixes property-free state 21816, zero strength, a full selection but
inset 14x15x14 collision/support shape, speed/jump factors, dampening 1, self loot and two reversible
bottle recipes; generic placement, instant break, loot/remainder allocation and correction stay here.
`BLK-SOUL-SAND-001`/`EXP-BLK-037` fixes property-free state 6998, strength 0.5, full selection,
visual and support cubes, a 14/16-high collider, speed 0.4, dampening 15, forced spawn/redstone/view/
suffocation predicates and no path type. It also fixes the generation-region postprocess-above
callback; generic placement, break, explosion, state publication and mutation flags stay here.
`BLK-MAGMA-001`/`EXP-BLK-038` fixes property-free state 14845, the full physical cube, strength
0.5, emission 3, dampening 15, fire-immune-only spawn predicate and the same generation-region
postprocess-above callback. Its hot-floor caller and content selectors are separate joins; generic
placement, break, explosion, light propagation and state publication remain with their parents.
`BLK-LAVA-CAULDRON-001`/`EXP-BLK-039` fixes property-free state 9464, its hollow shell and
15/16-high content contact, strength 2, emission 15, dampening 0, comparator output 3 and lack of an
item mapping. Generic writes, break/explosion, comparator propagation and terrain publication stay
with their parents.

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
`BLK-BELL-001`, `BLK-BANNER-001`, `BLK-SHELF-001`, `BLK-DECORATED-POT-001`,
`BLK-BRUSHABLE-001`, `BLK-SCULK-SENSOR-001`, `BLK-JIGSAW-001`, `BLK-STRUCTURE-001`,
`BLK-TEST-BLOCK-001`, `BLK-CONDUIT-001`, `BLK-BEACON-001`, `BLK-SIGN-001`, `BLK-SKULL-001`,
`ITM-HONEYCOMB-001`, `BLK-VINE-001`;
`EXP-BLK-002`, `EXP-BLK-008`, `EXP-BLK-009`, `EXP-BLK-012`, `EXP-BLK-013`, `EXP-BLK-014`,
`EXP-BLK-015`, `EXP-BLK-017`, `EXP-BLK-018`, `EXP-BLK-019`, `EXP-BLK-020`, `EXP-BLK-021`,
`EXP-BLK-022`, `EXP-BLK-027`,
`EXP-SIM-006`

The generic leaf locks every bit value/named mask, phase order, abort semantics and limits; the
content leaves fix their flags-2/3/11/258/260/818 callers, ignored results, state-family retention and
bell/banner/shelf/pot nested writes. `BLK-VINE-001` fixes every flags-2 growth/support write and its
no-retry boundary. `BLK-COMMAND-AREA-001` fixes clone/fill mutation flags, ordered preclear/write/
neighbor phases and their partial-failure boundaries.
`BLK-SCULK-SENSOR-001` fixes unchecked flags-3 activation/phase writes, flags-18 placement repair,
neighbor updates at the source and below, and water-tick scheduling at shape changes.
`BLK-JIGSAW-001` fixes the packet handler's direct same-state update carrying an unused integer
`3`, including its queued block/entity publication and path-cache invalidation.
`BLK-STRUCTURE-001` fixes mode's immediate flags-2 write, accepted operator edits' captured-old-
state flags-3 publication, scan's additional same-state publication and non-dirty power-latch writes.
`BLK-TEST-BLOCK-001` fixes its edit setter's ignored flags-2 state-write result followed by message,
dirtiness and a direct flags-argument-3 update; edits do not notify redstone neighbors or clear
powered/triggered latches, so state/entity divergence and stale output are intentional branches.
`BLK-TEST-INSTANCE-001` fixes every record/status/marker setter as dirty plus an AIR-to-current-state
flags-3 update, followed by the packet handler's duplicate final update; its test-volume clearing
uses flags 818 and an explicit neighbor update at every cell before template placement.
`BLK-CONDUIT-001` fixes water-tick scheduling on waterlogged shape changes and the target-reference
change's direct same-state flags-2 projection without a corresponding `setChanged` call.
`BLK-BEACON-001` fixes selection success as a chunk-dirty `blockEntityChanged` call without an
immediate state write, block update or block-entity-data projection.
`BLK-SIGN-001` fixes text/applicator setters as dirty plus same-state flags-3 updates, accepted edit
submission's additional unconditional flags-3 update, and water-tick scheduling at shape changes.
`BLK-SKULL-001` fixes server-only neighbor power comparison and its ignored flags-2 write result;
it reads neither above-position power nor any scheduled callback.
`ITM-HONEYCOMB-001` fixes its mapped copper replacement as flags 11, ignored write result and
post-write game/level events without rollback.

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

**Owners:** `BLK-FALL-001`, `BLK-BRUSHABLE-001`, `ENT-ENTITY-DROPS-001`; `EXP-BLK-003`,
`EXP-BLK-019`, `EXP-ENT-006`

The leaf fixes every scheduled delay, transition order, subtype branch, timeout, persistence
boundary, damage formula and RNG draw; the experiment is a regression matrix rather than a
source-unknown owner.
`BLK-BRUSHABLE-001` fixes reset-before-fall ordering, retained dust state, lost archaeology data,
disabled landing drop and failed-landing presentation for both suspicious blocks.
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
`BLK-DECORATED-POT-001`/`EXP-BLK-014`, `BLK-BRUSHABLE-001`/`EXP-BLK-019`,
`BLK-SCULK-SENSOR-001`/`EXP-BLK-020`, `BLK-JIGSAW-001`/`EXP-BLK-021`,
`BLK-TEST-BLOCK-001`/`EXP-BLK-022`, `BLK-CONDUIT-001`/`EXP-BLK-023`,
`BLK-BEACON-001`/`EXP-BLK-024`, `BLK-SIGN-001`/`EXP-BLK-025`,
`BLK-SKULL-001`/`EXP-BLK-026`, `BLK-STRUCTURE-001`/`EXP-BLK-027`,
`BLK-STRUCTURE-VOID-001`/`EXP-BLK-029`, `BLK-AIR-001`/`EXP-BLK-030`,
`BLK-BEDROCK-001`/`EXP-BLK-031`, `BLK-REINFORCED-DEEPSLATE-001`/`EXP-BLK-032`,
`BLK-TINTED-GLASS-001`/`EXP-BLK-033`,
`BLK-GLASS-001`/`EXP-BLK-034`, `BLK-STAINED-GLASS-001`/`EXP-BLK-040`,
`BLK-CONCRETE-001`/`EXP-BLK-041`, `BLK-TERRACOTTA-001`/`EXP-BLK-042`,
`BLK-GLAZED-TERRACOTTA-001`/`EXP-BLK-043`, `BLK-QUARTZ-001`/`EXP-BLK-044`, and
`ENV-GEYSER-001`/`EXP-ENV-005`
own concrete subtype transactions; other callbacks remain content-owned.
