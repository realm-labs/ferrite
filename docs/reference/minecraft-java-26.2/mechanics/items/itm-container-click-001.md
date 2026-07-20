# Items mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ITM-CONTAINER-CLICK-001` — Seven input variants form one deterministic cursor/slot state machine

**Parent:** `ITM-002`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — all generic `ContainerInput` branches, buttons, counts, traversal directions,
hook sites, drag phases and arithmetic are explicit in `AbstractContainerMenu` and `Slot`.
Item-provided click overrides dispatch at the stated point; their item-specific effects belong to
`ITM-USE-001`.

**Applies when:**

`ITM-CONTAINER-001` admits a click for one of `PICKUP(0)`, `QUICK_MOVE(1)`, `SWAP(2)`, `CLONE(3)`,
`THROW(4)`, `QUICK_CRAFT(5)` or `PICKUP_ALL(6)`; an out-of-range encoded enum ID decodes as
`PICKUP`.

**Authoritative state:**

Ordered menu slots and each slot's container index, item, `mayPickup`, `mayPlace`, maximum and
hooks; cursor stack; player inventory indices `0..8` and offhand `40`; player abilities/drop gate;
quick-craft status/type and selected slot set.

**Transition and ordering:**

Operation semantics are:

#### `PICKUP`

**Accepted arguments and exact transition:**

Only buttons `0` primary and `1` secondary. At slot `-999`, primary drops the entire cursor with
random throw motion then clears it; secondary splits and drops one. For a nonnegative slot, call the
tutorial hook, then try the enabled cursor stack's `overrideStackedOnOther`, then the enabled slot
stack's `overrideOtherStackedOnMe`; the first success consumes normal handling. Empty target inserts
all/one through `safeInsert`. With a nonempty target that may be picked up: an empty cursor removes
all/`ceil(count/2)`; a cursor accepted by the slot merges all/one when item and components match,
otherwise swaps if the whole cursor is at most the slot maximum; a cursor rejected by the slot may
nevertheless pull a matching stack into itself up to its own item maximum. The slot is marked
changed after the branch.

#### `QUICK_MOVE`

**Accepted arguments and exact transition:**

Only buttons `0` or `1`; button does not otherwise alter routing. Slot `-999` follows the same
whole/one cursor-drop branch as `PICKUP`; every other negative slot aborts. For a nonnegative slot
require source `mayPickup`, call the concrete menu route, then call it again while its returned
snapshot is nonempty and the source's current item ID (components ignored) still equals that
snapshot's item ID. See `ITM-CONTAINER-MOVE-001`.

#### `SWAP`

**Accepted arguments and exact transition:**

Button must be hotbar `0..8` or offhand `40`. If the selected inventory stack is empty, a pickable
target moves wholly into that inventory index, invokes `onSwapCraft(count)`, clears target and
invokes `onTake`. If target is empty, an accepted selected stack moves wholly or splits at target
maximum. If both are nonempty, target must be pickable and accept the selected stack: if selected
count exceeds target maximum, split that maximum into target, invoke `onTake` for the old target,
then add the old target to inventory or drop it with random motion; otherwise place old target in
the selected inventory index, replace target with selected, then invoke `onTake` for old target.

#### `CLONE`

**Accepted arguments and exact transition:**

Requires infinite materials, empty cursor and nonnegative occupied slot. It ignores `mayPickup` and
sets cursor to a copy with the source item's own maximum count.

#### `THROW`

**Accepted arguments and exact transition:**

Requires empty cursor and nonnegative slot. Button `0` requests one; every other button requests the
current full count. Require `canDropItems`, remove through `safeTake`, drop with random motion and
invoke creative-drop bookkeeping. Button `1` repeats while each removal is nonempty and the
replenished source has the same item ID, rechecking `canDropItems`; other buttons run once.

#### `PICKUP_ALL`

**Accepted arguments and exact transition:**

Requires nonnegative clicked slot and nonempty cursor; it proceeds only if the clicked slot is empty
or cannot be picked up. Button `0` scans ascending, every other button descending. It makes two
complete passes: pass 0 skips source stacks already at their own maximum, pass 1 includes them. Each
candidate must be occupied, same item and components as cursor, pickable and allowed by
`canTakeItemForPickAll`. Remove up to cursor item maximum through `safeTake` and grow cursor.

`Slot.tryRemove(request,max,player)` first checks `mayPickup`; if the slot does not allow
modification (`mayPickup && mayPlace(current)`), partial removal is rejected unless
`max >= currentCount`; it then removes `min(request,max)`, calls the two-stack
`setByPlayer(empty,removed)` when emptied, and returns the removed stack. `safeTake` additionally
invokes `onTake`. `safeInsert` first requires nonempty input and `mayPlace`, computes
`moved = min(request,inputCount,slotMax-inputCountInSlot)`, and either installs a split or grows a
same-item/same-components target.

**Branches and aborts:**

Any other pickup/quick-move button is a no-op. `SWAP` with any other button falls through to no
operation. `CLONE`, `THROW` and `PICKUP_ALL` silently do nothing when their gates fail. If a
non-quick-craft packet arrives while drag status is nonzero, it only resets drag state and does not
process that packet. Stack/component inequality blocks merging but may allow a whole-stack swap.
Slot subclass hooks may reject pickup, placement or all modification.

**Constants and randomness:**

Counts use signed integer arithmetic. Secondary pickup uses `(count + 1) / 2`. Maximum insertion is
`min(container/slot maximum, item maximum)`. No branch consumes RNG; item entity throw motion is
downstream of `Player.drop` and is specified with entity drop behavior.

**Side effects:**

Slot/container dirty flags; `setByPlayer`, `onTake`, `onQuickCraft` and `onSwapCraft`; tutorial
notifications; cursor mutation; inventory add or world drop; item-specific click override effects;
later synchronization under `ITM-CONTAINER-001`.

**Gates:**

Per-operation button/slot conditions, `mayPickup`, `mayPlace`, `allowModification`, item/component
identity, item and slot maxima, feature flags for override handlers, infinite-material ability,
`canDropItems`, `canDragTo`, and menu override of `canTakeItemForPickAll`.

**Boundary cases and quirks:**

Merge identity uses item plus components, but quick-move and repeated throw loop termination uses
item ID only. `moveItemStackTo` can merge into an occupied restricted slot without consulting
`mayPlace`; empty-slot placement does consult it. A result slot can allow matching output to be
pulled into a cursor even though it rejects placement. Empty stacks are normalized by `isEmpty`
semantics even when a container temporarily retains a zero-count object.

**Evidence:**

`OFF-SERVER-001`; `net.minecraft.world.inventory.ContainerInput`,
`net.minecraft.world.inventory.AbstractContainerMenu#doClick(int,int,net.minecraft.world.inventory.ContainerInput,net.minecraft.world.entity.player.Player)`,
`net.minecraft.world.inventory.AbstractContainerMenu#tryItemClickBehaviourOverride(net.minecraft.world.entity.player.Player,net.minecraft.world.inventory.ClickAction,net.minecraft.world.inventory.Slot,net.minecraft.world.item.ItemStack,net.minecraft.world.item.ItemStack)`,
`net.minecraft.world.inventory.AbstractContainerMenu#canItemQuickReplace(net.minecraft.world.inventory.Slot,net.minecraft.world.item.ItemStack,boolean)`,
`net.minecraft.world.inventory.Slot#tryRemove(int,int,net.minecraft.world.entity.player.Player)`,
`net.minecraft.world.inventory.Slot#safeInsert(net.minecraft.world.item.ItemStack,int)`,
`net.minecraft.world.inventory.Slot#safeTake(int,int,net.minecraft.world.entity.player.Player)`;
`EXP-ITM-002`.

**Test vectors:**

Primary/secondary outside drop; empty/nonempty cursor crossed with empty/same/different/restricted
target; output slot pull; max `1` and partial max `64`; every hotbar/offhand swap branch including
overflow and failed inventory add; clone without `mayPickup`; throw from a replenishing result;
pickup-all in both directions with partial/full sources and a two-pass ordering witness; an override
item in cursor and in target; interrupt an active drag with a normal click.

### Quick-craft phases

The button packs `header = button & 3` and `type = (button >> 2) & 3`; packing is
`(header & 3) | ((type & 3) << 2)`. Types `0` (even distribution) and `1` (one per slot) are always
valid; type `2` (fill to maximum) requires infinite materials; type `3` is invalid.

1. Header `0` is accepted only while status is `0`: capture type, set status `1`, clear selected
   slots. Empty cursor or invalid type resets.
2. Header `1` is accepted while status remains `1`: add the addressed slot to a set only when it can
   quick-replace the cursor, accepts it, passes `canDragTo`, and either type is `2` or cursor count
   is strictly greater than the current selected-set size.
3. Header `2` is accepted only from status `1`. No selected slot merely resets. Exactly one selected
   slot resets first and recursively performs `PICKUP` with button equal to quick-craft type. With
   two or more, copy the cursor and initialize `remaining` to its count. Revalidate every selected
   slot. For existing count `e`, slot cap `M = min(itemMax,slotMax)` and selected count `n`, set
   `target = min(e + q,M)`, subtract `target-e` from `remaining`, and install a copy with `target`;
   `q = floor(float(cursorCount) / float(n))` for type `0`, `1` for type `1`, and item maximum for
   type `2`. Finally set the copied cursor count to `remaining`, install it, and reset. The selected
   collection is a hash set; no stable iteration order is promised by source, although the stated
   per-slot target formula is order-independent.

Only the transition old-status `1` to header `2`, or a header equal to the old status, is accepted;
every other sequence resets. A one-slot type-`2` finish recursively supplies pickup button `2`, for
which ordinary pickup has no branch, so it performs no fill.

The multi-slot type-`2` arithmetic does not clamp `remaining` before assigning it to the cursor
copy. Filling multiple empty slots can therefore make that count zero or negative; subsequent
`ItemStack` empty semantics, rather than an explicit creative-cursor restoration, determine the
visible cursor.
