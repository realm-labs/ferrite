# Items mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ITM-BOOKSHELF-001` — Chiseled-bookshelf interaction, automation and comparator state can diverge

**Parent:** `ITM-001`, `ITM-003`, `PLY-005`, `BLK-003`, `RED-003`, `ENT-001`, `CLI-006`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked source fixes all six-slot hit selection, interaction results and
ordering, item policies, occupancy/last-slot updates, comparator projection, persistence/components,
hopper-facing gates, removal drops and visible effects. Generic hopper transaction traversal and
item-entity-private construction remain owned by their generic rules rather than unknown here.

**Applies when:**

A `minecraft:chiseled_bookshelf` is placed, rotated, hit on any face, used with an
empty/nonbook/book stack, accessed as a container, read by a comparator, saved/loaded/componentized,
or replaced.

**Authoritative state:**

The block has horizontal `FACING` plus six booleans `SLOT_0_OCCUPIED` through `SLOT_5_OCCUPIED`,
default north/all false, for 256 states. Placement faces opposite the player's horizontal direction;
rotation/mirroring transform facing. It is wood-map-color, bass-instrument,
chiseled-bookshelf-sound, lava-ignitable and has strength 1.5. A selectable hit exists only on the
outward face matching `FACING`. Looking at that face, north reverses world X, south uses X, west
uses Z and east reverses Z. `floor(coordinate*16/(16/sections))` clamped to range selects three
columns and two rows after vertically replacing `y` by `1-y`: top is slots 0..2 and bottom 3..5,
with exact `x=1/3`, `x=2/3` and `y=1/2` entering the next section under float conversion.

**Interaction dispatch:**

A wrong block-entity subtype returns `PASS`. A held non-book returns `TRY_WITH_EMPTY_HAND` before
hit selection. A held item in the `minecraft:bookshelf_books` tag (`book`, `written_book`,
`enchanted_book`, `writable_book`, `knowledge_book`) returns `PASS` off the front,
`TRY_WITH_EMPTY_HAND` when the captured occupancy property is true, and otherwise inserts then
returns `SUCCESS`. Empty-hand use returns `PASS` off-front, `CONSUME` for captured-empty and removes
then returns `SUCCESS` for captured-occupied. Consequently a nonbook or book used on an occupied
slot reaches empty-hand removal, while empty/nonbook use on a captured-empty slot consumes without
mutation. Client calls return the same result but both mutation helpers stop before side effects.

**Transition and ordering:**

On the server, insertion first awards the held item's `ITEM_USED` statistic, selects the enchanted
insert sound only for exact `minecraft:enchanted_book`, consumes and returns one item using player
abilities, passes that stack to block-entity `setItem`, then broadcasts the chosen block sound at
the block with category `BLOCKS`, volume 1 and pitch 1. Infinite-material players therefore
duplicate one book while retaining the hand stack. `setItem` installs the supplied stack object
without clamping its count, records the slot, derives all six occupancy properties from the live
item list, offers that state with flags 3, and then emits an unsourced `BLOCK_CHANGE` game event
whose context contains the offered state; a failed write does not roll back the item, last slot or
event.

**Removal ordering:**

`removeItem(slot,1)` ignores the requested count, clears and returns the entire stored stack, and
only for a nonempty result performs the same last-slot/state-write/unsourced-event update. The block
then selects the enchanted pickup sound only for exact enchanted book, broadcasts it with
volume/pitch 1, tries player-inventory insertion, and on false calls `player.drop(stack,false)`
before emitting a second `BLOCK_CHANGE` at the shelf sourced by the player. Thus ordinary successful
removal produces two game events. A full creative inventory clears the returned stack inside
inventory addition and reports success, deleting the book; a full ordinary inventory creates an item
entity at `(player.x, player.eyeY-0.3, player.z)` with pickup delay 40, no thrower and the
view/RNG-derived nonrandom-throw velocity. Partial overstack insertion drops only the remainder.
Entity admission is attempted and its result ignored.

**Container policy and state separation:**

The block entity owns exactly six slots, reports maximum stack size 1 and accepts only the
bookshelf-books tag. Public `setItem` silently rejects invalid nonempty stacks; empty delegates to
removal. Inherited `canPlaceItem` therefore admits a tagged stack only into an empty normal slot.
Public set/remove do not call `setChanged`; an occupancy-changing flags-3 block write supplies
chunk/comparator notification, but replacing one occupied book with another offers the same state
and can change items/last slot without dirtying or comparator-neighbor notification while still
emitting the game event. `removeItemNoUpdate` removes at most one, and `clearContent` clears raw
storage, without occupancy reconciliation, last-slot change or dirtying.

**Comparator projection:**

Server comparator reads return `lastInteractedSlot+1` from a matching block entity and zero
otherwise; client reads always return zero. The default `-1` yields zero and normal interactions
yield 1..6. Load accepts any persisted integer without validation or clamping, so comparator output
is exactly that integer plus one rather than an occupancy/count formula.

**Persistence and components:**

Load clears the six-list, applies serialized slot entries and reads `last_interacted_slot`
defaulting to -1; it does not validate item tag/count or reconcile the block properties. Save always
emits `Items`, even empty, and the last integer. Implicit `CONTAINER` components copy directly into
the six slots and collect all slots; component removal discards the legacy `Items` field. Raw
load/component/no-update/clear paths can therefore independently diverge stored items, occupancy
properties and comparator memory.

**Automation:**

The container is not sided, so all six slots are visible from every face. Inbound generic insertion
consults the tag/empty policy and then public `setItem`. Outbound preflight returns true when the
destination has any empty slot, without checking that slot's placement or capacity, or when it has
an equal item-and-components stack whose combined count is at most that destination stack's maximum.
Normal extraction clears occupancy before insertion; generic hopper rollback after a failed one-item
transfer calls public `setItem`, producing an empty then occupied state/event pair. A malformed
overstack is removed whole, and the generic rollback's one-item-only reinsertion condition can leave
it absent; traversal/commit details remain under the generic hopper owner.

**Removal drops and block loot:**

Removing the block entity, unless update flag 256 suppresses pre-removal side effects, drops the
live six-slot list independently of block loot and occupancy properties. Each slot including empty
consumes three level doubles for a position (`x/z + 0.125 + 0.75d`, `y + 0.75`). Every nonempty
stack is destructively split into `10+nextInt(21)` chunks; each entity receives three triangular
velocity samples with means `(0,0.2,0)` and deviation `0.11485000171139836`, consuming six level
doubles, and admission is attempted and ignored. Normal one-count books still consume the bounded
integer and create one entity. The locked block table yields the shelf block item only under Silk
Touch and copies no container contents, so contents still drop beside a Silk Touch block item;
without Silk Touch only contents drop. `block_drops=false` does not suppress the block-entity path,
while flag 256 does. Removal also refreshes analog neighbors.

**Branches and aborts:**

Player interaction trusts the captured occupancy properties, automation/drop trusts the item list,
and comparator trusts only the persisted last slot. Captured occupied plus actual empty therefore
plays an ordinary pickup sound and emits only the final player-sourced event while retaining ghost
occupancy; captured empty plus an actual item allows a tagged insertion to overwrite it, while
empty/nonbook use cannot retrieve it. Same-occupancy writes, failed writes, raw load/components and
no-update mutation preserve these divergences rather than repairing them.

**Constants and randomness:**

Six slots, two rows, three columns, flags 3, sound volume/pitch 1, insertion count 1, comparator
offset 1, drop pickup delay 40, item width 0.25, per-slot position range/half-width 0.75/0.125,
split 10..30 and velocity deviation above are fixed. Interaction itself consumes no bookshelf RNG. A
full ordinary-inventory fallback consumes four player floats for direction/power/jitter; pre-removal
consumes 18 position doubles plus one bounded integer and six level doubles per nonempty one-count
slot. Item-entity-private construction state is owned by `ENT-LIFECYCLE-001`.

**Side effects:**

Hand count/stat; six item slots; last slot; occupancy state and its ordinary neighbor/client
consequences; comparator output/refresh; block sounds; one or two game events; inventory insertion
or item entity; persistence/components; destructive replacement drops.

**Gates:**

Client/server, live subtype, held tag/exact enchanted item, matching front face and slot section,
captured occupancy, public/raw access path, destination preflight, same-state/failed state write,
inventory admission/creative clearing, block-entity-side-effect flag 256, Silk Touch and entity
admission.

**Boundary cases and quirks:**

A bookshelf book on an occupied slot removes rather than swaps. Invalid persisted last-slot values
directly escape through comparator output. `setItem` can store an overstack despite advertised
capacity; remove returns it whole. Empty ghost removal still plays sound and emits the player event.
Component/load truth is not normalized into block state, and ordinary same-occupancy replacement is
not guaranteed to persist because the block entity is never explicitly dirtied.

**Evidence:**

`OFF-SERVER-001`, `OFF-DATA-001`, `OFF-REPORT-001`;
`net.minecraft.world.level.block.ChiseledBookShelfBlock#useItemOn(net.minecraft.world.item.ItemStack,net.minecraft.world.level.block.state.BlockState,net.minecraft.world.level.Level,net.minecraft.core.BlockPos,net.minecraft.world.entity.player.Player,net.minecraft.world.InteractionHand,net.minecraft.world.phys.BlockHitResult)`,
`net.minecraft.world.level.block.ChiseledBookShelfBlock#useWithoutItem(net.minecraft.world.level.block.state.BlockState,net.minecraft.world.level.Level,net.minecraft.core.BlockPos,net.minecraft.world.entity.player.Player,net.minecraft.world.phys.BlockHitResult)`,
`net.minecraft.world.level.block.ChiseledBookShelfBlock#getAnalogOutputSignal(net.minecraft.world.level.block.state.BlockState,net.minecraft.world.level.Level,net.minecraft.core.BlockPos,net.minecraft.core.Direction)`,
`net.minecraft.world.level.block.ChiseledBookShelfBlock#getStateForPlacement(net.minecraft.world.item.context.BlockPlaceContext)`,
`net.minecraft.world.level.block.SelectableSlotContainer#getHitSlot(net.minecraft.world.phys.BlockHitResult,net.minecraft.core.Direction)`,
`net.minecraft.world.level.block.entity.ChiseledBookShelfBlockEntity#setItem(int,net.minecraft.world.item.ItemStack)`,
`net.minecraft.world.level.block.entity.ChiseledBookShelfBlockEntity#removeItem(int,int)`,
`net.minecraft.world.level.block.entity.ChiseledBookShelfBlockEntity#canTakeItem(net.minecraft.world.Container,int,net.minecraft.world.item.ItemStack)`,
`net.minecraft.world.level.block.entity.ChiseledBookShelfBlockEntity#loadAdditional(net.minecraft.world.level.storage.ValueInput)`,
`net.minecraft.world.level.block.entity.ChiseledBookShelfBlockEntity#saveAdditional(net.minecraft.world.level.storage.ValueOutput)`,
`net.minecraft.world.level.block.entity.ChiseledBookShelfBlockEntity#applyImplicitComponents(net.minecraft.core.component.DataComponentGetter)`,
`net.minecraft.world.level.block.entity.ChiseledBookShelfBlockEntity#collectImplicitComponents(net.minecraft.core.component.DataComponentMap$Builder)`,
`net.minecraft.world.level.block.entity.ListBackedContainer#removeItemNoUpdate(int)`,
`net.minecraft.world.level.block.entity.ListBackedContainer#canPlaceItem(int,net.minecraft.world.item.ItemStack)`,
`net.minecraft.world.level.block.entity.BlockEntity#preRemoveSideEffects(net.minecraft.core.BlockPos,net.minecraft.world.level.block.state.BlockState)`,
`net.minecraft.world.Containers#dropItemStack(net.minecraft.world.level.Level,double,double,double,net.minecraft.world.item.ItemStack)`,
`net.minecraft.world.entity.player.Player#drop(net.minecraft.world.item.ItemStack,boolean)`,
`net.minecraft.world.entity.LivingEntity#drop(net.minecraft.world.item.ItemStack,boolean,boolean)`;
`data/minecraft/tags/item/bookshelf_books.json`,
`data/minecraft/loot_table/blocks/chiseled_bookshelf.json`, registry/state membership from
`reports/blocks.json` and `reports/registries.json`; `EXP-ITM-010`.

**Test vectors:**

All 256 states and every face/row/column boundary; client/server and wrong subtype; each of five
accepted IDs plus invalid/empty held stack against captured empty/occupied and actual
empty/occupied; survival/infinite-material insertion; normal/enchanted pickup with
available/full/partially available survival and creative inventories; set/remove/no-update/clear
plus valid/invalid/overstack values; same-state and failed writes; valid/out-of-range last-slot
load; component/load state divergence; inbound/outbound hopper success/preflight mismatch/failure
rollback; empty/populated replacement under Silk Touch, `block_drops=false`, flag 256 and rejected
entity admission.
