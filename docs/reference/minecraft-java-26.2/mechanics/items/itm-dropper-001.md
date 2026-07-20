# Items mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ITM-DROPPER-001` — A dropper selects one occupied slot, then inserts one item or ejects it

**Parent:** `ITM-002`, `ITM-006`, `RED-001`, `SIM-003`, `BLK-003`, `PLY-006`, `ENT-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — the dropper-owned trigger, inventory, selection, target lookup, one-item
insertion, default ejection, event and persistence paths are fixed by locked source. A pending loot
table's generic evaluation and emitted stack sequence remain explicitly owned by unfinished
`ITM-LOOT-001`; this rule fixes only the dropper's exact caller context and the transaction after
fill.

**Applies when:**

A `minecraft:dropper` is placed, gains or loses ordinary neighbor power, its scheduled block tick
becomes due, a player opens it, a comparator reads it, or its `minecraft:dropper` block entity
loads/saves.

**Authoritative state:**

The block has six-way `FACING` and boolean `TRIGGERED`, defaulting north/false; placement faces
opposite the nearest look direction, including up/down. The matching block entity has exactly nine
slots, optional loot-table key/seed, lock and custom name; its default name is `container.dropper`,
menu is the 3x3 `DispenserMenu`, generic container maximum is 99, and a slot's effective maximum is
the lesser of 99 and the stack maximum.

**Transition and ordering:**

On each neighbor callback, power is `hasNeighborSignal(pos) || hasNeighborSignal(pos.above())`,
evaluated in that short-circuit order. Powered plus untriggered schedules this block after four
ticks at normal priority with the next sub-tick order, then offers `TRIGGERED=true` with flags 2; it
does not test for an existing scheduled tick, and neither result is used. Unpowered plus triggered
offers false with flags 2 but does not cancel pending work. Every due block tick calls the dispatch
attempt without rechecking power or `TRIGGERED`, so a short pulse is retained and failure to write
the latch after successful scheduling does not revoke the attempt.

**Inventory selection and deferred loot:**

Dispatch first requires a typed `DROPPER` block entity; mismatch logs a warning and returns before
RNG or events. `getRandomSlot` first calls `unpackLootTable(null)`. If a table is pending and
level/server exist, lookup occurs, the stored table key is cleared before fill, and `LootTable.fill`
receives the stored seed and a `CHEST` context containing only block-center `ORIGIN`: there is no
player, luck, `THIS_ENTITY`, or generate-loot advancement trigger. The generic table evaluator is
`ITM-LOOT-001`. Selection then scans slots 0 through 8; empty slots draw nothing, while the kth
nonempty slot consumes `nextInt(k)` and replaces the candidate exactly on zero (including
`nextInt(1)` for the first), producing a uniform occupied-slot choice. No occupied slot yields -1
with no selection draws and emits only level event 1001/data 0.

**Target lookup and insertion:**

With a selected nonempty stack, target direction is read from the live block state after
fill/selection. Lookup at the adjacent position prefers a block-provided `WorldlyContainer`,
otherwise a container block entity; a chest resolves its combined container with obstruction
ignored. Only when no block container exists does lookup collect eligible container entities in the
centered 1x1x1 box and, when nonempty, consume `nextInt(size)` to choose in encounter-list order
(also for size one). A found target receives a copied one-item stack from the face opposite the live
direction. A sided target visits `getSlotsForFace(face)` in returned order; otherwise slots increase
from zero. Every candidate must pass ordinary placement and sided placement. The first empty
eligible slot accepts the item; an occupied slot accepts only identical item/components and
available stack capacity. Success dirties the target; an initially empty hopper target receives
cooldown 8 because the source is not a hopper. Source-side extraction permissions are never
consulted. Success replaces the source slot with a copy shrunk by one; total failure replaces it
with an unchanged copy. The source block entity is dirtied by `setItem` in either branch. This
entire container branch emits no 1000/1001/2000 level event and no block-activate game event,
whether insertion succeeds or fails.

**Default ejection:**

With no target container, dispatch splits exactly one item from the selected live stack. Geometry
uses the block state captured by the scheduled-tick call, not the later live state: start at block
center plus `0.7 * capturedFacing`; subtract 0.125 from Y for vertical facing or 0.15625 otherwise.
The server creates an item entity and overwrites its motion with three triangular samples: first
consume one level `nextDouble` for `p=0.2+0.1d`, then six more level doubles for means
`(faceX*p, 0.2, faceZ*p)` and deviation `0.0172275*6 = 0.103365` per axis. Entity construction has
its separate UUID/private-RNG lifecycle under `ENT-LIFECYCLE-001`. Entity insertion success is
ignored: the source item remains consumed. After execution the server emits event 1000/data 0, then
event 2000/data `capturedFacing.get3DDataValue()`, and finally stores/dirties the remaining source
stack.

**Player, comparator and lifecycle surfaces:**

Empty-hand use always returns success on both sides. Only the server, when the live block entity is
any `DispenserBlockEntity`, calls `openMenu`; it then unconditionally awards `INSPECT_DROPPER` for
the dropper subtype (otherwise `INSPECT_DISPENSER`), even if lock, spectator-loot, or another
menu-opening gate prevents a screen. A wrong block entity gives neither menu nor stat. Comparator
output is zero for no container; otherwise sum `count / getMaxStackSize(stack)` over nonempty slots,
divide by nine, and return zero if the fraction is zero or `floor(fraction*14)+1` otherwise. Removal
requests neighboring-output updates. Rotation/mirroring transform `FACING`. Persistence initializes
nine empty slots, loads a pending loot table/seed instead of `Items` when present, and likewise
saves the pending table plus only a nonzero seed instead of item contents; ordinary items, lock and
custom name use inherited container persistence.

**Client event surface:**

Event 1001 plays `DISPENSER_FAIL` in `BLOCKS` at volume 1/pitch 1.2. Event 1000 plays
`DISPENSER_DISPENSE` at volume 1/pitch 1. Event 2000 decodes its direction and creates exactly ten
smoke particles; each consumes one client double for power `0.01+0.2d`, three doubles for the
direction-dependent position jitter, then three Gaussian samples for velocity around
`direction*power` with deviation 0.01. Sound events consume no client RNG here.

**Branches and aborts:**

A typed block-entity mismatch returns before selection; no occupied slot takes the fail-event
branch; an unexpectedly empty selected slot silently returns; a target container takes insertion
even if every eligible slot rejects the item; only absence of a target takes ejection. A client-side
use returns success without opening or awarding. A wrong live block entity gives neither menu nor
stat.

**Constants and randomness:**

Delay 4, flags 2, nine slots, one transferred item, target entity box 1x1x1, hopper cooldown 8,
ejection offsets 0.7/0.125/0.15625, accuracy 6, motion base 0.2 and deviation 0.103365, server
events 1000/1001/2000, and ten client smoke particles are fixed above. Server RNG comprises the
stated occupied-slot reservoir draws, optional entity-target draw, and seven ejection doubles;
deferred loot owns its evaluator draws. Event 2000 consumes the stated client doubles/Gaussians.

**Side effects:**

Scheduled ticks and latch writes; source/target inventory mutation and dirty hooks; optional hopper
cooldown; item-entity creation; menu open and inspect stat; comparator output; neighbor-output
update on removal; server level events; client sound and smoke particles; warning log for a missing
matching block entity. No dropper branch emits a game event.

**Gates:**

Ordinary power at self/above, latch state, due scheduled tick, typed block entity, occupied
selection, live facing property, target existence, target slot/face predicates, stack
identity/components/capacity, server-side interaction, menu lock/spectator gates, and persistence
presence of a loot table.

**Boundary cases and quirks:**

Target lookup uses live `FACING`, while ejection placement, motion mean and animation use captured
`FACING`; a source-visible callback can therefore create a split state, and a live replacement
lacking that property throws before target/output events. Multiple rising callbacks may enqueue
according to the generic scheduler because the block performs no local deduplication. Failed target
insertion does not fall back to ejection. Entity insertion failure still consumes the item and emits
events. Unlike a dispenser, every non-container item uses the same ejection behavior and the dropper
never emits `GameEvent.BLOCK_ACTIVATE`.

**Evidence:**

`OFF-SERVER-001`, `OFF-CLIENT-001`, `OFF-REPORT-001`;
`net.minecraft.world.level.block.DispenserBlock#neighborChanged(net.minecraft.world.level.block.state.BlockState,net.minecraft.world.level.Level,net.minecraft.core.BlockPos,net.minecraft.world.level.block.Block,net.minecraft.world.level.redstone.Orientation,boolean)`,
`net.minecraft.world.level.block.DispenserBlock#getStateForPlacement(net.minecraft.world.item.context.BlockPlaceContext)`,
`net.minecraft.world.level.block.DispenserBlock#useWithoutItem(net.minecraft.world.level.block.state.BlockState,net.minecraft.world.level.Level,net.minecraft.core.BlockPos,net.minecraft.world.entity.player.Player,net.minecraft.world.phys.BlockHitResult)`,
`net.minecraft.world.level.block.DispenserBlock#getAnalogOutputSignal(net.minecraft.world.level.block.state.BlockState,net.minecraft.world.level.Level,net.minecraft.core.BlockPos,net.minecraft.core.Direction)`,
`net.minecraft.world.level.block.DropperBlock#dispenseFrom(net.minecraft.server.level.ServerLevel,net.minecraft.world.level.block.state.BlockState,net.minecraft.core.BlockPos)`,
`net.minecraft.world.level.block.entity.DispenserBlockEntity#getRandomSlot(net.minecraft.util.RandomSource)`,
`net.minecraft.world.level.block.entity.DispenserBlockEntity#loadAdditional(net.minecraft.world.level.storage.ValueInput)`,
`net.minecraft.world.level.block.entity.DispenserBlockEntity#saveAdditional(net.minecraft.world.level.storage.ValueOutput)`,
`net.minecraft.world.RandomizableContainer#unpackLootTable(net.minecraft.world.entity.player.Player)`,
`net.minecraft.world.level.block.entity.HopperBlockEntity#getContainerAt(net.minecraft.world.level.Level,net.minecraft.core.BlockPos)`,
`net.minecraft.world.level.block.entity.HopperBlockEntity#addItem(net.minecraft.world.Container,net.minecraft.world.Container,net.minecraft.world.item.ItemStack,net.minecraft.core.Direction)`,
`net.minecraft.core.dispenser.DefaultDispenseItemBehavior#dispense(net.minecraft.core.dispenser.BlockSource,net.minecraft.world.item.ItemStack)`,
`net.minecraft.core.dispenser.DefaultDispenseItemBehavior#spawnItem(net.minecraft.world.level.Level,net.minecraft.world.item.ItemStack,int,net.minecraft.core.Direction,net.minecraft.core.Position)`,
`net.minecraft.world.inventory.AbstractContainerMenu#getRedstoneSignalFromContainer(net.minecraft.world.Container)`,
`net.minecraft.client.renderer.LevelEventHandler#levelEvent(int,net.minecraft.core.BlockPos,int)`,
`net.minecraft.client.renderer.LevelEventHandler#shootParticles(int,net.minecraft.core.BlockPos,net.minecraft.util.RandomSource,net.minecraft.core.particles.SimpleParticleType)`;
registry/state membership from `reports/blocks.json` and `reports/registries.json`; `EXP-ITM-007`.

**Test vectors:**

All six facings and both latch states; power at self/above and a one-tick pulse; repeated neighbor
callbacks plus latch-write failure; missing/wrong block entity; zero, one and nine occupied slots
with draw trace; pending loot with seed zero/nonzero; block versus entity target, two entity
targets, sided slot order, full/mismatched/partial/empty target, obstructed double chest and empty
hopper cooldown; target callback replacing/rotating the source; no-target ejection for
horizontal/vertical facing with failed entity insertion; open unlocked/locked/spectator/wrong
subtype; comparator empty, one item and all full; save/load with items versus pending loot; exact
server event order and ten client smoke particles.
