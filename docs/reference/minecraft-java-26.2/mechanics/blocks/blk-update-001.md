# Blocks mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `BLK-UPDATE-001` — State writes and neighbor/shape propagation are distinct operations

**Parent:** `BLK-003`, `BLK-004`, `BLK-005`, `BLK-007`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — the locked jars specify storage admission, all ten flag bits,
state/lighting/block-entity side effects, collector ordering and both limits, block-event rounds,
client publication, navigation and POI invalidation, and block-entity ticker rebinding below.
Concrete block callbacks and individual block-entity tick algorithms remain in their content slices;
`EXP-BLK-002` is a conformance vector, not a source gap.

**Applies when:**

Authoritative or predicted code writes a block state, explicitly notifies neighbors, submits a shape
update, queues a block event, or changes a state that owns a block entity.

**Authoritative state:**

A level owns canonical block-state references, chunks/sections, heightmaps and light queues, a
`CollectingNeighborUpdater`, the block-event insertion-ordered set, and the ordered block-entity
ticker lists. Each write carries a bit mask and a signed-32-bit shape-update budget. Server state is
authoritative; client writes use the same storage path but only affect prediction/render state until
corrected.

**Transition and ordering:**

For `Level#setBlock(pos, requested, flags, limit)`:

1. Reject with `false` outside valid level bounds or in a server debug level. Otherwise
   obtain/create the level chunk. The three-argument overload supplies `limit=512`.
2. `LevelChunk#setBlockState` returns null without mutation when an all-air section receives air or
   the old canonical state reference equals `requested`. Otherwise install `requested` in the
   section first, update random-tick counters, non-air/fluid counts, four maintained heightmaps,
   section empty status, sky-light sources, and enqueue a light check when light properties differ.
   Every later callback observes the installed state unless it has already replaced it.
3. Let `blockChanged` mean old and requested block **types** differ, `moved` mean bit 64, and
   `blockEntitySideEffects` mean bit 256 is clear. If a changed old state has a block entity that
   the new state cannot retain, optionally call its pre-removal side effect, then always remove the
   object, game-event listener, debug record and ticker wrapper. Bit 256 suppresses only the
   pre-removal callback, not removal.
4. On a server, if the type changed or the requested type is a rail, call the old state's
   removal-neighbor hook when bit 1 or bit 64 is set, passing `moved`. If that callback has replaced
   the position with another block type, abort the chunk transition and return null; the already
   performed write/callback effects remain.
5. Unless bit 512 is set, call requested-state `onPlace` on the server, passing old state and
   `moved`. If the currently stored block type still matches the requested type and it owns a block
   entity, reuse a valid compatible object (update its state and rebind its ticker) or
   create/register a new object. Mark the chunk unsaved and return the old state.
6. `Level#setBlock` rereads the position. It returns `true` for every non-null chunk result even
   when a callback changed the state; however, it performs the remaining stages only when the reread
   canonical state is exactly `requested`. If old and reread differ, mark blocks dirty. With bit 2,
   publish only on a server chunk at least `BLOCK_TICKING`, or on a client when bit 4 is clear.
   Server publication queues the chunk change, invalidates path-type cache, and synchronously
   recomputes paths whose collision-shape query is affected. Client publication passes bit 8 as the
   immediate-dirty boolean to rendering.
7. With bit 1, submit ordinary notifications around the source using the **old** block type; then,
   server-side, update comparator/output neighbors only when requested has analog output. Unless bit
   16 is set and while `limit>0`, compute `nestedFlags = flags & ~33`, then run old indirect shapes,
   requested direct neighbor shapes, and requested indirect shapes, each with `limit-1`. Finally
   compare old/new POI types and remove then add changed POI ownership through server-executor
   tasks. On the main thread outside a reentrant task those execute immediately; inside a server
   task they queue in remove-before-add order.

Flags: bit 1 `UPDATE_NEIGHBORS`; 2 `UPDATE_CLIENTS`; 4 `UPDATE_INVISIBLE`; 8 `UPDATE_IMMEDIATE`; 16
`UPDATE_KNOWN_SHAPE` (skip generic shape cascade); 32 `UPDATE_SUPPRESS_DROPS`; 64
`UPDATE_MOVE_BY_PISTON`; 128 `UPDATE_SKIP_SHAPE_UPDATE_ON_WIRE`; 256
`UPDATE_SKIP_BLOCK_ENTITY_SIDEEFFECTS`; 512 `UPDATE_SKIP_ON_PLACE`. Named masks are
`UPDATE_NONE=260`, `UPDATE_ALL=3`, `UPDATE_ALL_IMMEDIATE=11`, and `UPDATE_SKIP_ALL_SIDEEFFECTS=816`.
Shape-generated writes clear bits 1 and 32; air results call `destroyBlock` with drops iff bit 32 is
clear, while non-air results call `setBlock` with bit 32 cleared.

**Neighbor collector ordering:**

Ordinary six-side notification order is `WEST, EAST, DOWN, UP, NORTH, SOUTH`; direct shape order is
`WEST, EAST, NORTH, SOUTH, DOWN, UP`. A multi-neighbor work item executes one direction per step and
rereads that receiver. A simple work item also rereads on execution; a full work item uses its
captured receiver state; a shape item captures the changed-neighbor state but rereads the receiver
before calling
`updateShape(level, scheduledTicks, receiverPos, direction, neighborPos, capturedNeighborState, level.random)`,
then applies `updateOrDestroy`. With redstone experiments enabled, multi-neighbor work
derives/updates its `Orientation` per direction.

The first submission pushes a work item and drains synchronously. Submissions made while draining
append to `addedThisLayer` in call order. After the current one-direction/callback step, those items
are pushed in reverse so they execute FIFO before the interrupted multi-item or older stack frames
resume: depth-first by callback layer, FIFO within one added layer. The collector counts submitted
work items, not receiver callbacks. Default/integrated limit is 1,000,000; dedicated
`max-chained-neighbor-updates` defaults to that value and may be negative for unlimited. At
`count >= limit`, the new item is discarded; the first discarded position is logged once, later
submissions in that chain remain discarded, and the count/queues reset after the outer drain or
exception. This chain cap is independent of the per-write shape budget 512.

**Block-event state machine:**

`blockEvent(pos, block, a, b)` inserts the four-field record into an `ObjectLinkedOpenHashSet`, so
an exact duplicate already queued is ignored and distinct records retain insertion order. During the
normal level phase, repeatedly remove the first record until the set is empty. If
`shouldTickBlocksAt(pos)` is false, put it in a side list that is reinserted only after the drain.
Otherwise reread state: a different block type discards the event; a matching type calls the current
state's `triggerEvent`. Only a `true` result broadcasts the event packet within 64 blocks in that
dimension. An active callback's newly queued event joins the same set and can execute later in the
**same** drain; there is no cap, and requeueing the just-removed identical event can loop
indefinitely. Inactive records cannot spin because they remain isolated until the drain ends.

**Block-entity lifecycle:**

Registration validates the current state/type, sets level, clears removed, replaces and invalidates
any different object at the position, registers a game-event listener when loaded, and obtains the
state/type-specific ticker. Existing ticker wrappers are rebound in place, preserving list position;
a null ticker result removes the position's wrapper by rebinding it to the sentinel removed ticker.
A new wrapper goes to the main list unless block-entity iteration is already active, in which case
it goes to a pending list merged only at the next `tickBlockEntities` entry. Each block-entity phase
first merges pending, removes invalid wrappers even while frozen, and otherwise calls a wrapper only
during normal gameplay and when `shouldTickBlocksAt(pos)`. The bound wrapper additionally requires
the object not removed, a level, world-border inclusion, chunk `BLOCK_TICKING`, entities loaded, and
a currently compatible state; an incompatible state logs once and does not call subtype logic. Thus
creation before the phase can tick that same server tick, while creation from a block-entity
callback cannot.

**Constants and randomness:**

State coordinates and limits use Java signed-32 arithmetic; callers may pass any flag combination
and any limit. Direct shape callbacks receive the shared level RNG and consume only what their
subtype code requests; generic mutation/notification/event/BE lifecycle consumes no RNG. Block-event
parameters are unbounded signed ints. The event broadcast radius is 64.0 blocks.

**Side effects:**

Section counters, heightmaps, light section/check queues, dirty/save state, block-entity
pre-remove/on-place/listener/ticker work, old-state removal hooks, client chunk deltas/render
dirtiness, path-cache invalidation and path recomputation, ordinary/shape/comparator notifications,
drops from shape-to-air, POI changes, packets, and every nested callback effect occur in the order
above. Bit 1 does not imply bit 2, and neither implies the shape cascade.

**Branches and aborts:**

Invalid bounds, server debug level, unchanged canonical state, a callback replacing the requested
block type, exhausted shape budget, discarded over-cap collector work, inactive/replaced block-event
targets, and invalid/removed/inactive block entities stop only the layer described above. A
chunk-level callback abort can leave its already installed intermediate state and earlier side
effects. Difficulty and `doTileDrops` do not abort state installation; bit 32 controls shape-removal
drops, while higher-level destroy callers own gamerule/tool drop policy.

**Gates:**

Valid bounds; non-debug server; actual canonical-state change; callbacks retaining requested state;
ten flag bits; positive shape budget; redstone-experiment feature; block-ticking
publication/activity; event-position activity and current block type; BE state/type validity, world
border, entities loaded, freeze and ticker existence.

**Boundary cases and quirks:**

`true` from `setBlock` means the chunk accepted an initial mutation, not that `requested` survived
callbacks. The installed state is visible before old removal and new placement hooks. Same block
type/property changes can retain the BE and rebind its ticker. Directly replacing a registered BE
object invalidates the old object but does not itself unregister that old object's game-event
listener; the ordinary incompatible-state path explicitly removes the listener first. Bits 256/512
are narrowly scoped and do not mean “no side effects.” Ordinary and shape directions differ. A
multi-neighbor submission consumes one chain-count unit for as many as six callbacks. Exact block
events deduplicate only while simultaneously present; self-requeue after removal defeats
deduplication. Update-suppressed states intentionally remain possible.

**Evidence:**

`Confirmed`; `OFF-SERVER-001`. Anchors:
`net.minecraft.world.level.Level#setBlock(net.minecraft.core.BlockPos,net.minecraft.world.level.block.state.BlockState,int,int)`,
`net.minecraft.world.level.chunk.LevelChunk#setBlockState(net.minecraft.core.BlockPos,net.minecraft.world.level.block.state.BlockState,int)`,
`net.minecraft.world.level.block.Block#updateOrDestroy(net.minecraft.world.level.block.state.BlockState,net.minecraft.world.level.block.state.BlockState,net.minecraft.world.level.LevelAccessor,net.minecraft.core.BlockPos,int,int)`,
`net.minecraft.world.level.redstone.CollectingNeighborUpdater#addAndRun(net.minecraft.core.BlockPos,net.minecraft.world.level.redstone.CollectingNeighborUpdater$NeighborUpdates)`,
`net.minecraft.world.level.redstone.CollectingNeighborUpdater#runUpdates()`,
`net.minecraft.world.level.redstone.NeighborUpdater#executeShapeUpdate(net.minecraft.world.level.LevelAccessor,net.minecraft.core.Direction,net.minecraft.core.BlockPos,net.minecraft.core.BlockPos,net.minecraft.world.level.block.state.BlockState,int,int)`,
`net.minecraft.server.level.ServerLevel#runBlockEvents()`,
`net.minecraft.server.level.ServerLevel#doBlockEvent(net.minecraft.world.level.BlockEventData)`,
`net.minecraft.world.level.chunk.LevelChunk#updateBlockEntityTicker(net.minecraft.world.level.block.entity.BlockEntity)`,
and `net.minecraft.world.level.Level#tickBlockEntities()`.

**Test vectors:**

(1) For each single flag bit and named mask, replace stone with a support-sensitive state and record
storage, old/new callbacks, client delta, neighbor/comparator/shape work, drops and BE side effects.
(2) Write the identical canonical state and assert `false` with no work; have `onPlace` replace the
request and assert outer `true` but no outer publication/neighbor/POI cascade. (3) Record ordinary
order `W,E,D,U,N,S` and shape order `W,E,N,S,D,U`. From W enqueue A then B and assert A,B execute
before E. (4) Set shape limit 1 and assert only the first layer is submitted; separately set chain
limit 1 and prove one multi-item can still call six receivers while nested work is discarded. (5)
Compare bits 16, 32, 128, 256 and 512 independently. (6) Queue duplicate, distinct, replaced-block,
inactive, callback-appended and one safely bounded self-requeued block event; assert deduplication,
retry, packet and same-drain semantics. (7) Replace a BE with compatible and incompatible
same/different states and assert retention/removal, pre-remove suppression, listener changes and
wrapper position. (8) Create one BE before its phase and one inside a BE callback; assert same-tick
versus next-phase first tick. (9) Freeze while removing a BE and assert invalid wrapper cleanup
without subtype ticks. (10) Run `EXP-BLK-002` over the flag matrix, direction trace, nested layers,
both limits, event rounds and ticker timing.
