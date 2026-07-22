# Items mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ITM-CHEST-001` — Chests pair canonically while each half retains independent storage and opener state

**Parent:** `ITM-002`, `ITM-006`, `PLY-005`, `SIM-003`, `BLK-003`, `RED-003`, `MOB-004`, `ENT-001`,
`CLI-006`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked source fixes ordinary/trapped placement and pairing, obstruction,
single/double menu admission, canonical slot order, independent loot/persistence/removal, opener
recounts, comparator and trapped-chest power, sounds/events and client lid/material projection.
Generic loot-table evaluation remains owned by unfinished `ITM-LOOT-001`.

**Applies when:**

A `minecraft:chest` or `minecraft:trapped_chest` is placed, paired, separated, used, opened/closed,
read by automation or comparator, powered, saved/loaded/componentized, replaced, migrated from a
legacy chunk, or rendered as a block or item.

**Authoritative state:**

Each block has horizontal `FACING`, `TYPE` in `single/left/right`, and `WATERLOGGED`, default
north/single/false: 24 states per block and 48 total. Ordinary and trapped chests are distinct block
identities and never pair. A single uses the centered 14 by 14 column from Y 0 through 14; a paired
half extends its matching side to the block edge. No state is pathfindable. Rotation changes facing,
mirror delegates through rotation, and waterlogged shape updates first schedule a water tick.

Placement initially faces opposite the player's horizontal direction. With secondary use held and
a horizontal clicked face, only the neighbor opposite that face is considered: it must be the same
block, single, and face-axis-perpendicular to its facing; the new state adopts that facing and becomes
right when `candidateFacing.counterClockwise == clickedFace.opposite`, otherwise left. If secondary
use is not held, a matching same-facing single neighbor clockwise is tried first and makes the new
state left; counterclockwise is second and makes it right. Holding secondary use suppresses this
automatic fallback even when forced pairing fails. A compatible update can turn a single into the
opposite half of an adjacent non-single; loss or incompatibility in the connected direction turns a
half back into single.

Left connects in `facing.clockwise`; right connects in `facing.counterClockwise`. Internally right is
`FIRST` and left is `SECOND`, so every valid double container orders right slots 0..26 before left
slots 27..53 regardless of the queried half. Pair validation requires the same block identity,
opposite non-single types, equal facing, both matching block entities and, when requested, neither
half blocked. A missing/blocked queried half returns no container; other malformed partner state
falls back to the queried half as a single container.

**Transition and ordering:**

A half is blocked when the block directly above reports redstone-conductor semantics at that
position, or any `Cat` in `[x,x+1] x [y+1,y+2] x [z,z+1]` is in a sitting pose; taming and ownership
do not matter. Menus and comparators enforce obstruction on both halves. Hopper block-container
discovery deliberately ignores it, as does client rendering.

Client use returns success without mutation. Server use obtains the obstruction-aware menu
provider; no provider returns success without menu, statistic or piglin effect. Otherwise it calls
`openMenu`, ignores its result, awards `OPEN_CHEST` or `TRIGGER_TRAPPED_CHEST`, then invokes guarded-
container piglin anger. Therefore lock or pending-loot failure still consumes the next menu ID,
awards the statistic and angers piglins once an unblocked provider existed.

A single uses inherited randomizable-container admission: pending loot rejects spectators;
otherwise spectators bypass the main-hand lock predicate. Nonspectator lock failure sends
`container.isLocked` plus `CHEST_LOCKED` at the half center. A double checks right admission first,
then left only if right passed. Any failure sends that locked overlay/sound at the midpoint between
halves even for a spectator, returns no menu, and `ServerPlayer.openMenu` then also sends
`container.spectatorCantOpen` for that spectator. Neither half materializes loot unless both checks
pass. Success materializes right then left, selects title right custom name, else left custom name,
else `container.chestDouble`, and constructs `generic_9x6`; a single constructs `generic_9x3` with
its custom/default `container.chest` title. Construction calls `startOpen` before the open-screen
packet and current-menu installation.

**Storage, access and validity:**

Each block entity independently owns 27 slots, optional loot-table key/seed, custom name and lock.
A compound container routes slots, mutations, open/close and clear right then left; its maximum stack
size comes from the right half, validity requires both halves, and `isEmpty` short-circuits right
before left. Thus a double is a 54-slot view, never one 54-slot persistent inventory. Single-menu
validity requires the identical live block entity and strict eye-to-block-AABB squared distance below
`(blockInteractionRange+4)^2`; a double requires that for both halves. Ordinary menu state-ID,
click, quick-move and close convergence remains `ITM-CONTAINER-*`.

Player opening materializes pending loot with stored seed and chest context containing center origin,
player, luck and the generation criterion. Public container reads/writes instead use a null-player
origin-only context, so comparator, hopper and removal may permanently choose that context first.
For a double, successful open resolves right then left; comparator and indexed automation likewise
observe canonical right-first traversal. `ITM-LOOT-001` owns evaluator/RNG output after these inputs.

Load starts with 27 empty slots and loads a pending table/seed instead of `Items`; save writes a
pending table and only nonzero seed instead of `Items`. Custom name and lock persist independently.
Implicit block-entity components carry custom name, nonempty lock, contents, and `CONTAINER_LOOT`
when pending. Each chest/trapped-chest item is common, stackable to 64, and has a default empty
container component. The locked block loot yields the matching item subject to explosion survival
and copies only custom name; contents are a separate pre-removal transaction.

**Open counters, sounds and lid events:**

Each half owns an independent transient signed opener count, maximum observed interaction range and
lid controller; none is saved. Removed block entities and spectators do not count. First open plays
the block's chest-open sound, emits sourced `CONTAINER_OPEN`, schedules a five-tick recount, then
sends block event `(1,newCount)` and updates maximum range. Final close analogously sounds/emits,
clears range and sends the event; additional openers send only the count event/range update.

Left halves never play a sound. A single sounds at its center; a right half sounds at the midpoint,
its center plus half its connected-direction vector. Both use `BLOCKS`, volume 0.5 and pitch
`0.9+0.1*nextFloat`. Since compound start/stop calls right then left, a double produces one midpoint
sound but two game events, block events and independent recount schedules.

Each due recount searches the block AABB inflated by `maxInteractionRange+4` for nonspectator
`ContainerUser`s. A player matches when its current `ChestMenu` directly owns the half or owns a
compound containing it. The pass resets/recomputes maximum range and count; zero/nonzero boundaries
sound and emit a source-less game event, while every pass calls the count-change hook even if count
is unchanged, hence sends a fresh block event. It reschedules while positive. Event 1 targets the lid
open exactly when its parameter is positive. Only the client ticks lid openness by 0.1 toward that
target, clamped to [0,1], and interpolates between previous/current values.

**Comparator, trapped power and automation:**

Both blocks expose the standard container comparator output over the obstruction-aware 27/54-slot
container: sum each nonempty `count/min(99,stackMax)`, divide by capacity, return zero for an empty
fraction or `floor(fraction*14)+1`. Any blocked half makes a double's comparator container absent and
the output zero. Hopper lookup instead requests the obstruction-ignoring compound container.

A trapped chest is a signal source. Its weak signal is its local block entity opener count clamped
to 0..15; direct signal equals that value only toward `UP`, otherwise zero. After the ordinary block
event, a trapped half whose old and new counts differ updates neighbors at itself and below using
experimental redstone orientation derived from `facing.opposite` and up. An unchanged recount still
sends the lid event but performs no neighbor update. Both halves of a double count and power
independently.

**Removal and legacy upgrade:**

Replacing one half runs the generic pre-removal drop transaction only for that half's 27 slots; the
partner becomes single and retains its inventory. Unless flag 256 suppresses block-entity side
effects, public slot access first materializes null-player loot and the transaction consumes three
position doubles for every slot, including empty slots (81 total), then destructively splits each
nonempty stack into 10..30 chunks with the position/velocity/admission behavior specified by
`ITM-BARREL-001`. `block_drops=false` does not suppress contents. Removal also refreshes output
neighbors.

Legacy chunk upgrade may pair adjacent same-block/same-facing singles whose connecting axis is
perpendicular to facing. It writes the neighbor with flags 18, returns the current half, and swaps the
two block entities' entire item-list references only for north/east facing, preserving canonical
right-first legacy slot order. This migration-only swap does not run during ordinary placement or
shape update.

**Client presentation:**

World rendering combines halves while ignoring obstruction, uses the maximum half openness and
maximum packed light, eases openness as `1-(1-open)^3`, and rotates lid plus lock by
`-eased*pi/2`. It selects single/left/right geometry and regular/trapped material. A renderer-
construction-time local-date cache replaces both with Christmas material on December 24..26,
Christmas taking precedence over trapped until renderer reconstruction. Item models independently
select ordinary/trapped versus Christmas special-renderer entries from local `MM-dd`. The ordinary
block model supplies only the oak-planks particle texture; the chest body is block-entity rendered.

**Branches and aborts:**

After every server use with an unblocked provider, the no-RNG guarded-container ingress from
`ITM-BARREL-001` considers idle visible `Piglin`s in the opener box inflated 16 and writes the
600-tick normal/universal anger memories subject to attackability. Wrong/missing block entity,
blocked half and client use stop before statistic/anger. Spectators can open no-pending single or
double storage read-only but do not affect counters. Breaking an open half makes later close a no-op
for that removed entity; the ordinary validity path closes invalid menus.

**Constants and randomness:**

The fixed values are 27 slots per half, menu IDs 1..100, validity/recount buffer 4, recount delay 5,
lid step 0.1, sound volume 0.5/pitch `[0.9,1.0)`, trapped power 0..15, comparator scale 14+1,
piglin range 16/duration 600, and removal constants inherited above. Each first/final sound consumes
one server float; removal consumes 81 position doubles per destroyed half plus split/entity draws.
Pairing, obstruction, menu, counter, comparator and signal selection consume no direct RNG.

**Side effects:**

Menu ID/screen/stat/anger; lock overlays/sound; loot criterion/fill; inventory/dirty state;
scheduled fluid/recount ticks; block/game/sound events; redstone/comparator updates;
persistence/components; item entities; and client lid, light and material projection.

**Gates:**

Exact block/subtype and pair compatibility; secondary-use placement; conductor/sitting-cat blockage;
server side; pending loot/spectator/main-hand lock; live identity and strict range; removed/spectator
opener; current menu containing a half; changed trapped count; public versus raw access; flag 256;
item-entity admission; and idle/visible/attackable piglin plus universal anger.

**Boundary cases and quirks:**

Blockage denies players and comparators but not hoppers/rendering; malformed paired state can degrade
to a queried single; double lock failure can produce two spectator overlays; and two halves share a
screen while retaining independent storage, loot, counters, power and destruction. Right-first
ordering is canonical even when the left half was queried, and an unchanged recount still emits a
lid event without trapped-chest neighbor updates.

**Evidence:**

`OFF-SERVER-001`, `OFF-CLIENT-001`, `OFF-DATA-001`, `OFF-REPORT-001`;
`net.minecraft.world.level.block.ChestBlock#getStateForPlacement(net.minecraft.world.item.context.BlockPlaceContext)`,
`net.minecraft.world.level.block.ChestBlock#updateShape(net.minecraft.world.level.block.state.BlockState,net.minecraft.world.level.LevelReader,net.minecraft.world.level.ScheduledTickAccess,net.minecraft.core.BlockPos,net.minecraft.core.Direction,net.minecraft.core.BlockPos,net.minecraft.world.level.block.state.BlockState,net.minecraft.util.RandomSource)`,
`net.minecraft.world.level.block.ChestBlock#useWithoutItem(net.minecraft.world.level.block.state.BlockState,net.minecraft.world.level.Level,net.minecraft.core.BlockPos,net.minecraft.world.entity.player.Player,net.minecraft.world.phys.BlockHitResult)`,
`net.minecraft.world.level.block.ChestBlock#combine(net.minecraft.world.level.block.state.BlockState,net.minecraft.world.level.Level,net.minecraft.core.BlockPos,boolean)`,
`net.minecraft.world.level.block.ChestBlock#isChestBlockedAt(net.minecraft.world.level.LevelAccessor,net.minecraft.core.BlockPos)`,
`net.minecraft.world.level.block.ChestBlock#getAnalogOutputSignal(net.minecraft.world.level.block.state.BlockState,net.minecraft.world.level.Level,net.minecraft.core.BlockPos,net.minecraft.core.Direction)`,
`net.minecraft.world.level.block.TrappedChestBlock#ownSignal(net.minecraft.world.level.block.state.BlockState,net.minecraft.world.level.BlockGetter,net.minecraft.core.BlockPos)`,
`net.minecraft.world.level.block.TrappedChestBlock#getDirectSignal(net.minecraft.world.level.block.state.BlockState,net.minecraft.world.level.BlockGetter,net.minecraft.core.BlockPos,net.minecraft.core.Direction)`,
`net.minecraft.world.level.block.DoubleBlockCombiner#combineWithNeigbour(net.minecraft.world.level.block.entity.BlockEntityType,java.util.function.Function,java.util.function.Function,net.minecraft.world.level.block.state.properties.Property,net.minecraft.world.level.block.state.BlockState,net.minecraft.world.level.LevelAccessor,net.minecraft.core.BlockPos,java.util.function.BiPredicate)`,
`net.minecraft.world.CompoundContainer#startOpen(net.minecraft.world.entity.ContainerUser)`,
`net.minecraft.world.level.block.entity.ChestBlockEntity#startOpen(net.minecraft.world.entity.ContainerUser)`,
`net.minecraft.world.level.block.entity.ChestBlockEntity#recheckOpen()`,
`net.minecraft.world.level.block.entity.ChestBlockEntity#signalOpenCount(net.minecraft.world.level.Level,net.minecraft.core.BlockPos,net.minecraft.world.level.block.state.BlockState,int,int)`,
`net.minecraft.world.level.block.entity.ChestBlockEntity#swapContents(net.minecraft.world.level.block.entity.ChestBlockEntity,net.minecraft.world.level.block.entity.ChestBlockEntity)`,
`net.minecraft.world.level.block.entity.TrappedChestBlockEntity#signalOpenCount(net.minecraft.world.level.Level,net.minecraft.core.BlockPos,net.minecraft.world.level.block.state.BlockState,int,int)`,
`net.minecraft.world.inventory.ChestMenu#sixRows(int,net.minecraft.world.entity.player.Inventory,net.minecraft.world.Container)`,
`net.minecraft.world.level.block.entity.HopperBlockEntity#getBlockContainer(net.minecraft.world.level.Level,net.minecraft.core.BlockPos,net.minecraft.world.level.block.state.BlockState)`,
`net.minecraft.client.renderer.blockentity.ChestRenderer#extractRenderState(net.minecraft.world.level.block.entity.BlockEntity,net.minecraft.client.renderer.blockentity.state.ChestRenderState,float,net.minecraft.world.phys.Vec3,net.minecraft.client.renderer.feature.ModelFeatureRenderer$CrumblingOverlay)`,
`net.minecraft.client.renderer.blockentity.ChestRenderer#submit(net.minecraft.client.renderer.blockentity.state.ChestRenderState,com.mojang.blaze3d.vertex.PoseStack,net.minecraft.client.renderer.SubmitNodeCollector,net.minecraft.client.renderer.state.level.CameraRenderState)`;
`reports/blocks.json#minecraft:{chest,trapped_chest}`,
`reports/registries.json#minecraft:block_entity_type/minecraft:{chest,trapped_chest}`,
`data/minecraft/loot_table/blocks/{chest,trapped_chest}.json`, bundled item models/textures;
`EXP-ITM-013`.

**Test vectors:**

All 48 states; ordinary/trapped cross-pair rejection; automatic and secondary-use placement in every
direction; valid/missing/malformed pairs; conductor and sitting-cat obstruction on either half;
single/double custom names, locks and pending loot with normal/spectator players; right/left first
materialization and slots 26/27; comparator versus hopper blockage; one/two viewers, recount and
signed counts; trapped power/direction/neighbor calls; per-half save/components/break/RNG; north/east
versus south/west upgrade; client partial frames, delayed events, light and Dec 23/24/26/27 material.
