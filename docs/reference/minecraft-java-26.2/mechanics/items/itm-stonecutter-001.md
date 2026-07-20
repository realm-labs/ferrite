# Items mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ITM-STONECUTTER-001` — A selected key-ordered recipe consumes one input per result batch

**Parent:** `ITM-004`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — synchronized recipe filtering/order, selection, preview replacement, take
commit, sound coalescing and close disposition are explicit in locked source.

**Applies when:**

A player opens a stonecutter, changes its input, selects a displayed recipe, takes its result or
closes the menu.

**Authoritative state:**

One input slot, one result container, the input item's identity snapshot, feature-filtered ordered
stonecutter entries, selected index, stored recipe holder, level game time and last result-sound
time.

**Transition and ordering:**

Server recipe finalization walks the key-ordered recipe collection and retains each stonecutter
recipe only when every ingredient item and its result display are enabled. An input item-ID change
clears selection to `-1`, clears the result and filters that ordered collection by ingredient
membership; count or component changes to the same item do not rebuild it. A valid different button
index stores the index and recipe holder, assembles the fixed result from the current input and
broadcasts it. Taking calls the result's crafted hook, awards the stored recipe using the
pre-consumption input, removes exactly one input, and recomputes the same selected result when input
remains. It then plays the take sound only if no take has played at that block during the current
level game time, before the generic slot-take tail.

**Branches and aborts:**

Selecting the already selected index returns unhandled and does nothing. Any other index is reported
handled, but a negative/out-of-range index leaves selection and result unchanged. Empty input or no
enabled matching entries produces no selection/result. Result pickup-all is forbidden. Closing
discards the preview and returns or drops the input through the ordinary menu-close transaction.

**Constants and randomness:**

Slots are input `0`, result `1`; selection starts at `-1`. At most one take sound is emitted per
game-time value, even if quick-move repeats multiple batches in that tick. No branch consumes RNG.

**Side effects:**

Selection/result synchronization, result crafted hook, recipe award/criterion, one input consumed
per assembled result stack, block-position sound, and close-time input return/drop.

**Gates:**

Server-opened menu, block interaction range, enabled ingredient/result projection, ingredient match,
valid selection, nonempty result and ordinary destination capacity for quick-move.

**Boundary cases and quirks:**

Recipe order is reload key order, not output name or count. A recipe output stack of any
data-defined count still costs one input. Same-item component changes intentionally retain the
selected list/index because stonecutting ingredients are item/tag based and assembly is a fixed
template. If the last input is removed, its container callback clears selection before the take path
attempts to refresh, so no ghost preview remains.

**Evidence:**

`OFF-SERVER-001`, `OFF-DATA-001`;
`net.minecraft.world.item.crafting.RecipeManager#finalizeRecipeLoading(net.minecraft.world.flag.FeatureFlagSet)`,
`net.minecraft.world.item.crafting.SelectableRecipe$SingleInputSet#selectByInput(net.minecraft.world.item.ItemStack)`,
`net.minecraft.world.level.block.StonecutterBlock#useWithoutItem(net.minecraft.world.level.block.state.BlockState,net.minecraft.world.level.Level,net.minecraft.core.BlockPos,net.minecraft.world.entity.player.Player,net.minecraft.world.phys.BlockHitResult)`,
`net.minecraft.world.inventory.StonecutterMenu#slotsChanged(net.minecraft.world.Container)`,
`net.minecraft.world.inventory.StonecutterMenu#clickMenuButton(net.minecraft.world.entity.player.Player,int)`,
`net.minecraft.world.inventory.StonecutterMenu#setupResultSlot(int)`,
`net.minecraft.world.inventory.StonecutterMenu$2#onTake(net.minecraft.world.entity.player.Player,net.minecraft.world.item.ItemStack)`,
`net.minecraft.world.inventory.StonecutterMenu#removed(net.minecraft.world.entity.player.Player)`;
`EXP-ITM-003`.

**Test vectors:**

Multiple matching keys in lexical order; disabled ingredient/result; same item with changed
count/components versus different item; same/invalid/valid button index; output count above one;
final versus remaining input; repeated shift-takes in one and adjacent game times; full destination;
menu close.
