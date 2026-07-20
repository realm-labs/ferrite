# Items mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ITM-SMITHING-001` — Smithing previews the first matching recipe and consumes each occupied role after take

**Parent:** `ITM-004`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — role admission, first-match preview, error state, recipe credit, three-slot
consumption and event order are explicit in locked source.

**Applies when:**

A player opens a smithing table, changes template/base/addition slots, takes the result or closes
the menu.

**Authoritative state:**

Template `0`, base `1`, addition `2`, result `3`; feature-filtered role property sets, the full
key-ordered smithing recipe domain, stored result recipe holder, synchronized recipe-error flag,
player and table access.

**Transition and ordering:**

Server opening awards the table interaction stat. Each input change first constructs the three-role
input and selects the first matching smithing recipe in key order, with no retained-holder fast
path. A match assembles according to `ITM-RECIPE-SERIALIZER-001`, stores its holder and replaces the
result; no match clears both. After recomputation, the error flag becomes true only when all three
input slots are nonempty and the result is empty. Taking invokes the result crafted hook, awards the
stored recipe using a snapshot list of all three current role stacks, then shrinks each nonempty
role by one in template/base/addition order; each replacement can trigger an intermediate result
recomputation. Finally level event `1044` fires at the table.

**Branches and aborts:**

Role slots admit only items in the synchronized template/base/addition property set. Optional recipe
roles may remain empty and are not shrunk. Assembly may itself return empty, which leaves no
takeable result even though a holder was selected. The result cannot be placed into or included in
pickup-all. Closing returns/drops all remaining inputs and discards the preview.

**Constants and randomness:**

Four menu slots with three ordered input roles; interaction validity uses the ordinary block range
with distance parameter `4.0`. No smithing menu branch consumes RNG.

**Side effects:**

Result and error synchronization, result crafted hook, recipe award/criterion, up to three input
decrements, level event, interaction stat and close-time input return/drop.

**Gates:**

Server side, smithing-table/range validity, role property admission, recipe ingredient matching,
assembly nonempty result, pickup and quick-move destination admission.

**Boundary cases and quirks:**

The synchronized role sets are insertion aids, not a recipe choice: a stack can belong to multiple
roles, while quick-move considers template then base then addition emptiness and the slot predicates
decide the actual destination. The error indicator deliberately stays false for incomplete two-slot
combinations. Preview lookup does not preserve the previously credited recipe, so overlapping
smithing inputs always use the first key-ordered match after any input callback.

**Evidence:**

`OFF-SERVER-001`, `OFF-DATA-001`;
`net.minecraft.world.level.block.SmithingTableBlock#useWithoutItem(net.minecraft.world.level.block.state.BlockState,net.minecraft.world.level.Level,net.minecraft.core.BlockPos,net.minecraft.world.entity.player.Player,net.minecraft.world.phys.BlockHitResult)`,
`net.minecraft.world.inventory.SmithingMenu#createResult()`,
`net.minecraft.world.inventory.SmithingMenu#slotsChanged(net.minecraft.world.Container)`,
`net.minecraft.world.inventory.SmithingMenu#canMoveIntoInputSlots(net.minecraft.world.item.ItemStack)`,
`net.minecraft.world.inventory.SmithingMenu#onTake(net.minecraft.world.entity.player.Player,net.minecraft.world.item.ItemStack)`,
`net.minecraft.world.inventory.ItemCombinerMenu#quickMoveStack(net.minecraft.world.entity.player.Player,int)`,
`net.minecraft.world.inventory.ItemCombinerMenu#removed(net.minecraft.world.entity.player.Player)`;
`EXP-ITM-003`.

**Test vectors:**

Transform and trim recipes; overlapping lexical keys; optional/required role emptiness; an item
eligible for multiple roles; empty assembly/equal trim; all three invalid versus incomplete error
state; count/component-bearing base; output destination full; close with each role occupied.
