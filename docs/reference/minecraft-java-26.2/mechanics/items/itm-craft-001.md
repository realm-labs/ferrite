# Items mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ITM-CRAFT-001` — Manual result take revalidates, awards, consumes, and places per-slot remainders

**Parent:** `ITM-004`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — result recomputation, limited-crafting admission, achievement order, take-time
revalidation, cropped-grid consumption and remainder disposition are explicit in locked source.

**Applies when:**

A player crafting grid changes, or pickup/quick-move removes an item from its result slot.

**Authoritative state:**

Full grid and cropped input/offset, retained recipe holder, result slot, player recipe book,
`limitedCrafting`, removed-result count, inventory and current recipe manager.

**Transition and ordering:**

On a server grid change, build the cropped input, prefer the retained recipe while it still matches,
otherwise use `ITM-RECIPE-001`; a non-special recipe is displayable only when `limitedCrafting` is
false or the player has unlocked it. Assemble and require an enabled result, store the recipe
holder, replace result slot `0`, set its remote baseline, increment menu state ID and send that slot
immediately. Taking first applies `onCraftedBy` for the accumulated removed count, triggers the
stored recipe-crafted criterion with a snapshot of full input slots, awards the non-special recipe,
and clears the stored holder. It then rebuilds the cropped current input, re-runs first-match lookup
to obtain per-position remainders (or default item remainders if no longer matched), and walks the
cropped rectangle row-major using its saved left/top offset. For each cell: remove one current
ingredient if present; then place a nonempty remainder into the now-empty grid cell, combine it with
a same-item/same-components residual stack by adding that residual count to the remainder, or try
player inventory and finally drop it without random throw motion.

**Branches and aborts:**

No match, limited-crafting rejection or disabled result produces an empty result slot. Take-time
remainder selection may differ from the recipe holder credited immediately before it if callbacks or
concurrent container changes altered the grid; it deliberately performs a fresh lookup. Cells
outside the cropped rectangle are untouched. Quick-move repeats only through
`ITM-CONTAINER-MOVE-001` while the replenished result item ID stays the same and destination space
remains.

**Constants and randomness:**

Traversal is cropped row-major. Each participating nonempty cell loses exactly one. Manual crafting
and remainder placement consume no RNG.

**Side effects:**

Result/menu state, recipe unlock/criterion/stat hooks, item `onCraftedBy`, ingredient counts,
remainder grid/inventory/drop placement and subsequent result recomputation/synchronization.

**Gates:**

Matching recipe, limited-crafting rule and unlock, enabled output, result-slot pickup, destination
space for quick-move and ordinary container transaction admission.

**Boundary cases and quirks:**

Output preview is non-consuming. The stored credited recipe and the take-time remainder recipe are
separate lookups. When a remainder matches a residual ingredient, the implementation constructs the
remainder stack with the combined count rather than checking its item maximum at that point. A
failed inventory insert drops the remainder; it never cancels the already-consumed craft.

**Evidence:**

`OFF-SERVER-001`, `OFF-DATA-001`;
`net.minecraft.world.inventory.CraftingMenu#slotChangedCraftingGrid(net.minecraft.world.inventory.AbstractContainerMenu,net.minecraft.server.level.ServerLevel,net.minecraft.world.entity.player.Player,net.minecraft.world.inventory.CraftingContainer,net.minecraft.world.inventory.ResultContainer,net.minecraft.world.item.crafting.RecipeHolder)`,
`net.minecraft.world.inventory.RecipeCraftingHolder#setRecipeUsed(net.minecraft.server.level.ServerPlayer,net.minecraft.world.item.crafting.RecipeHolder)`,
`net.minecraft.world.inventory.ResultSlot#checkTakeAchievements(net.minecraft.world.item.ItemStack)`,
`net.minecraft.world.inventory.ResultSlot#getRemainingItems(net.minecraft.world.item.crafting.CraftingInput,net.minecraft.world.level.Level)`,
`net.minecraft.world.inventory.ResultSlot#onTake(net.minecraft.world.entity.player.Player,net.minecraft.world.item.ItemStack)`;
`EXP-ITM-003` remains a mutation-order regression probe.

**Test vectors:**

Preview versus take; locked ordinary and unlocked/special recipe under limited crafting; retained
overlapping recipe; callback changes grid between preview and take; cropped recipe at every offset;
each remainder disposition; duplicate book/banner source remainder; shift-craft output item change
and destination exhaustion.
