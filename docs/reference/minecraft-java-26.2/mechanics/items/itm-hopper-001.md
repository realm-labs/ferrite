# Items mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ITM-HOPPER-001` — A hopper deterministically pushes before pulling and cools down across reload

**Parent:** `ITM-002`, `ITM-005`, `ITM-006`, `PLY-005`, `SIM-004`, `BLK-003`, `RED-003`,
`ENT-001`, `CLI-006`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked source fixes all ten block states, five-slot storage/menu, redstone
enablement, server-tick and collision ingress, push/pull/container/entity selection, sided
admission, rollback, cooldown propagation/persistence, loot/components/removal and client model.
Generic loot-table evaluation remains owned by unfinished `ITM-LOOT-001`; minecart-hopper
movement, activation and Hopper-interface cadence remain an unreviewed entity subtype.

**Applies when:**

A `minecraft:hopper` block is placed, powered, used, ticked, collided with by an item entity, read
by a comparator, used as either side of another hopper transfer, saved/loaded/componentized,
replaced, or projected to a client.

**Authoritative state:**

The block has `FACING` in down/north/south/west/east and boolean `ENABLED`, default down/true: ten
states with report IDs 11313..11322. Placement uses the clicked face's opposite, except either
vertical result becomes down, and always starts enabled before placement-time power checking.
Rotation/mirroring transform facing. The block has stone map color, requires the correct tool,
hardness 3, explosion resistance 4.8, metal sound and no occlusion; it is never pathfindable.

The collision outline is the full-width Y10..16 upper body plus centered 8-wide Y4..10 funnel,
with a centered 12-wide Y11..16 void removed and a facing-rotated 4-wide outlet added. The
interaction shape is a centered 12-wide Y11..16 column plus the facing outlet; down has only the
column. Shape does not depend on enabled state. The block entity owns five slots, an optional
loot-table key/seed, custom name, lock, signed transfer cooldown initially -1, last-ticked game time
initially 0, and a cached facing derived from its current state. Default title is
`container.hopper`; menu type and block-entity protocol IDs are 16 and 18.

**Transition and ordering:**

On placement from a different block and on every neighbor change, the hopper computes
`enabled = !level.hasNeighborSignal(pos)`. It offers a flags-2 state write only when that differs;
the result is ignored. Power neither stops the server block-entity ticker nor blocks other
containers from inserting into or extracting from this hopper; it only prevents this hopper's own
transfer transaction. A state replacement refreshes output neighbors.

Only the server installs the ticker. Every tick first decrements cooldown by one, then records the
current level game time. A still-positive cooldown stops. Otherwise cooldown is normalized to zero
and, if enabled, one transaction runs: public `isEmpty` first materializes this hopper's pending
loot with a null-player context; if nonempty, push is attempted; after that result and any mutation,
the hopper recomputes whether all five slots are exactly at each stack's own maximum. Unless full,
it always evaluates pull as well—the boolean combination is non-short-circuit—so one transaction
can push one item and then pull one item. If either operation reports success, cooldown becomes 8,
then the hopper is marked changed and its comparator neighbors update. Failure leaves cooldown zero,
so the next tick decrements to -1, renormalizes and retries. Disabled ticks likewise count an
existing cooldown down to zero and retry immediately on the first later enabled tick.

**Destination and source selection:**

Push resolves the block position in cached facing. Pull resolves the block containing hopper center
at Y+1. At either position, a block implementing `WorldlyContainerHolder` takes first precedence;
otherwise a container block entity is used. A chest block entity is expanded to its obstruction-
ignoring right-first compound container. Only when no block container exists does a 1x1x1 box
centered at the lookup coordinates select a container entity; an empty list yields none, otherwise
one list member is selected by `level.random.nextInt(size)`.

Pull with a source container traverses its `DOWN` face slots in returned order, or ascending all
slots for a nonsided container. Push traverses hopper slots 0..4 and enters the destination through
`hopperFacing.opposite`, using that face's returned slots or ascending all slots. Extraction requires
the source's generic `canTakeItem(hopper,slot,stack)` and, when sided, `canTakeItemThroughFace`.
Insertion requires generic `canPlaceItem` and, when sided with a nonnull face,
`canPlaceItemThroughFace`. The hopper's own destination face is null during pull and item-entity
collection, so only its generic admission applies. Locks and player menu admission are never
consulted by automation.

If no source container exists, a grid-aligned block hopper suppresses loose-item search when the
above block has a full collision shape and is not in `minecraft:does_not_block_hoppers`. In locked
data that tag delegates to `minecraft:beehives`, containing bee nest and beehive. Otherwise it
queries live `ItemEntity`s in the full block-width suction AABB from local Y 11/16 through 2 and
tries them in entity-list order. The same AABB intersection gates an item entity's immediate
`entityInside` callback; that callback invokes the entire push-then-pull transaction, not a special
absorb-only path, and still obeys cooldown and enabled state.

**One-item container transfer:**

Push preflights the destination as full when every accessible slot has count greater than or equal
to its stack maximum. It then examines source slots 0..4. For the first nonempty candidate it saves
the original count, destructively removes one, and traverses destination slots. Pull performs the
same one-item operation from each eligible source slot in order. An empty target slot receives the
incoming stack object; a merge requires same item and components and grows only up to the incoming
stack's maximum. Every successful target-slot mutation calls destination `setChanged`.

An empty remainder commits. Push then calls destination `setChanged` a second time and returns;
pull calls source `setChanged` and returns. A nonempty one-item remainder restores the original
source count; when the original count was one it also reinstalls that stack in the source slot.
These restore paths deliberately do not mark either side changed. Source selection therefore stops
at the first committed one-item move, while a rejected slot/item leaves the transaction available
to later slots.

**Receiving-hopper cooldown:**

When insertion changes a destination that was empty at the start of that slot attempt and the
destination is a hopper with cooldown at most 8, it receives cooldown 8. If the source is also a
hopper and `destination.tickedGameTime >= source.tickedGameTime`, it instead receives 7. A custom
cooldown above 8 is preserved, and a nonempty destination receives no such adjustment. The actor's
own successful transaction subsequently sets its own cooldown to 8, so the 7-tick branch matters
for a different empty hopper receiving another hopper's push. Enabled/locked state is not a gate.

**Loose-item partial insertion:**

Loose-item collection copies the entity stack, traverses all hopper slots with null face, and writes
the entity to the final remainder. Full absorption replaces its stack with empty then discards it
and reports success. Partial absorption reports false even though hopper slots, dirty/output calls,
the entity remainder, and possibly an empty-hopper cooldown already changed. Consequently the
loose-item loop may continue to later entities after partial mutation, and the outer transaction
does not apply its final cooldown/output call unless some entity is fully absorbed or push already
succeeded. Repeated collision callbacks can therefore observe partial state when no cooldown was
installed.

**Player menu and loot boundary:**

Block use always returns success. Client use and wrong/missing subtype do nothing. Correct server
use calls `openMenu`, ignores its result, then unconditionally awards `INSPECT_HOPPER`. As with other
randomizable containers, opening closes an old non-inventory menu and consumes the next ID 1..100;
pending loot rejects spectators, nonspectators must satisfy the main-hand lock predicate, and a
failed attempt sends the corresponding locked or spectator overlay/sound while still awarding the
statistic. Success materializes player-context loot and creates the five-slot hopper menu.

Slots 0..4 appear at `(44+18i,20)`; the standard player inventory begins at `(8,51)`. Quick move
from hopper uses all player slots in reverse, while player-to-hopper uses 0..4 forward. Construction
calls the container's no-op default `startOpen`; close calls its no-op `stopOpen`. Validity requires
the identical block entity and strict squared eye-to-block-AABB distance below
`(blockInteractionRange+4)^2`. Generic state-ID, click and synchronization behavior remains
`ITM-CONTAINER-*`.

Player opening resolves pending loot with player/luck/criterion chest context. Tick transfer,
comparator, other automation and removal access public methods and therefore use the null-player,
origin-only context; the first caller fixes the materialized result. Destination/source lookups can
likewise materialize another randomizable container when their first public slot read occurs.
`ITM-LOOT-001` owns evaluation/RNG after these exact contexts.

**Persistence, components and removal:**

Load initializes five empty slots, loads a pending table/seed instead of `Items`, and reads
`TransferCooldown` with default -1. Save writes the pending table or items, and always writes the
current signed cooldown. Custom name/lock persist independently. Last-ticked game time and cached
facing are not saved; facing reconstructs from block state. Components carry custom name, nonempty
lock, contents and pending `CONTAINER_LOOT`, but not cooldown. The stack-64 common hopper item has a
default empty container component.

The block loot yields one hopper subject to explosion survival and copies only custom name.
Contents are a separate generic pre-removal transaction. Unless flag 256 suppresses block-entity
side effects, it public-reads all five slots—materializing null-player loot—and consumes 15 position
doubles even when empty, then uses the split/velocity/entity-admission behavior in
`ITM-BARREL-001`. `block_drops=false` does not suppress contents. Lock, cooldown and inventory are
not copied into the block-loot item.

**Client projection:**

Flags-2 redstone writes synchronize `ENABLED`, but blockstate JSON selects solely by facing, so
enabled and disabled are visually identical. Down uses `block/hopper`; horizontal facings rotate
`block/hopper_side`. Both are ordinary baked models with no block-entity renderer or update packet;
contents and cooldown reach a client only through an open menu's generic container synchronization.

**Branches and aborts:**

Client ticker, active transfer while cooldown-positive/disabled, push with no/full destination,
pull with a full hopper, blocked loose-item search, failed sided admission and wrong/missing menu
subtype are no-ops at their stated boundary. A disabled or locked hopper remains a valid passive
source/destination. An above block container takes precedence over collision blocking and loose
items; an attached/source block container takes precedence over a randomly chosen container entity.
Removal makes later menu validity fail through the ordinary server close path.

**Constants and randomness:**

Five slots; ten states; IDs 16/18; initial/absent cooldown -1; normal cooldown 8 and receiving offset
7; retry normalization 0; suction local Y 11/16..2; entity-container box 1x1x1; menu coordinates
above; interaction buffer 4; item/menu ID ranges and removal constants above. Transfer slot choice,
sided order and loose-item iteration consume no RNG. Each nonempty container-entity candidate list
consumes one bounded level-random draw; loot/removal RNG is separately owned.

**Side effects:**

Flags-2 enabled writes; cached facing; old-menu close, ID/screen/stat/overlay/sound; loot criterion
and fill; slot/entity mutations; source/destination dirty and comparator-neighbor updates; cooldown
and tick-time changes; entity discard; persistence/components; removal item entities; baked-model
state and menu projection.

**Gates:**

Server ticker; cooldown positive; enabled state; own empty/full state; destination/source existence;
block-holder/entity precedence; accessible slots; generic and sided take/place admission; same item
and components; stack maximum; full collision shape and `does_not_block_hoppers`; live item/AABB;
menu subtype, pending loot/spectator/lock/range; flag 256 and entity admission.

**Boundary cases and quirks:**

Push does not short-circuit pull. Counts above their own maximum make the hopper's equality-based
`inventoryFull` return false, while destination preflight uses greater-than-or-equal. A target hopper
receiving into a nonempty slot gets no propagated cooldown. Loose-item partial absorption can mutate
without overall success. Random container-entity choice happens only after block-container failure.
Redstone and locks stop neither passive access nor cooldown countdown, and saved cooldown resumes
from its exact integer rather than elapsing while unloaded.

**Evidence:**

`OFF-SERVER-001`, `OFF-CLIENT-001`, `OFF-DATA-001`, `OFF-REPORT-001`;
`net.minecraft.world.level.block.HopperBlock#getStateForPlacement(net.minecraft.world.item.context.BlockPlaceContext)`,
`net.minecraft.world.level.block.HopperBlock#getTicker(net.minecraft.world.level.Level,net.minecraft.world.level.block.state.BlockState,net.minecraft.world.level.block.entity.BlockEntityType)`,
`net.minecraft.world.level.block.HopperBlock#useWithoutItem(net.minecraft.world.level.block.state.BlockState,net.minecraft.world.level.Level,net.minecraft.core.BlockPos,net.minecraft.world.entity.player.Player,net.minecraft.world.phys.BlockHitResult)`,
`net.minecraft.world.level.block.HopperBlock#neighborChanged(net.minecraft.world.level.block.state.BlockState,net.minecraft.world.level.Level,net.minecraft.core.BlockPos,net.minecraft.world.level.block.Block,net.minecraft.world.level.redstone.Orientation,boolean)`,
`net.minecraft.world.level.block.HopperBlock#entityInside(net.minecraft.world.level.block.state.BlockState,net.minecraft.world.level.Level,net.minecraft.core.BlockPos,net.minecraft.world.entity.Entity,net.minecraft.world.entity.InsideBlockEffectApplier,boolean)`,
`net.minecraft.world.level.block.entity.HopperBlockEntity#pushItemsTick(net.minecraft.world.level.Level,net.minecraft.core.BlockPos,net.minecraft.world.level.block.state.BlockState,net.minecraft.world.level.block.entity.HopperBlockEntity)`,
`net.minecraft.world.level.block.entity.HopperBlockEntity#suckInItems(net.minecraft.world.level.Level,net.minecraft.world.level.block.entity.Hopper)`,
`net.minecraft.world.level.block.entity.HopperBlockEntity#addItem(net.minecraft.world.Container,net.minecraft.world.Container,net.minecraft.world.item.ItemStack,net.minecraft.core.Direction)`,
`net.minecraft.world.level.block.entity.HopperBlockEntity#addItem(net.minecraft.world.Container,net.minecraft.world.entity.item.ItemEntity)`,
`net.minecraft.world.level.block.entity.HopperBlockEntity#getItemsAtAndAbove(net.minecraft.world.level.Level,net.minecraft.world.level.block.entity.Hopper)`,
`net.minecraft.world.level.block.entity.HopperBlockEntity#getContainerAt(net.minecraft.world.level.Level,net.minecraft.core.BlockPos)`,
`net.minecraft.world.level.block.entity.HopperBlockEntity#entityInside(net.minecraft.world.level.Level,net.minecraft.core.BlockPos,net.minecraft.world.level.block.state.BlockState,net.minecraft.world.entity.Entity,net.minecraft.world.level.block.entity.HopperBlockEntity)`,
`net.minecraft.world.level.block.entity.HopperBlockEntity#loadAdditional(net.minecraft.world.level.storage.ValueInput)`,
`net.minecraft.world.level.block.entity.HopperBlockEntity#saveAdditional(net.minecraft.world.level.storage.ValueOutput)`,
`net.minecraft.world.inventory.HopperMenu#quickMoveStack(net.minecraft.world.entity.player.Player,int)`,
`net.minecraft.world.level.block.entity.BlockEntity#setChanged(net.minecraft.world.level.Level,net.minecraft.core.BlockPos,net.minecraft.world.level.block.state.BlockState)`;
`reports/blocks.json#minecraft:hopper`,
`reports/registries.json#minecraft:block_entity_type/minecraft:hopper`,
`reports/minecraft/components/item/hopper.json`, `data/minecraft/tags/block/does_not_block_hoppers.json`,
`data/minecraft/loot_table/blocks/hopper.json`, bundled hopper block/item models; `EXP-ITM-014`.

**Test vectors:**

All ten states and placement faces; power edges during cooldown; cooldown -2/-1/0/1/7/8/9 and
save/reload; empty/full/overstack five slots; same-tick push+pull; every sided slot/admission order;
empty/nonempty hopper chains in both tick orders; block holder, single/double blocked chest and
multiple container entities; full/nonfull tagged/untagged above blocks; full/partial/rejected loose
stacks through tick and collision ingress; player/null loot race; normal/spectator/locked menu;
comparator fractions; components and five-slot removal with block_drops/flag-256/admission variants.
