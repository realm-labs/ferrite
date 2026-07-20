# Items mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ITM-LOOM-001` — Loom selection appends one ordered pattern layer while preserving a reusable pattern item

**Parent:** `ITM-003`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — all 43 official pattern IDs, tag/component selection, selected-holder remapping,
six-layer limit, output assembly and consumption are explicit in locked source/data.

**Applies when:**

A player opens a loom, changes banner/dye/pattern inputs, selects a pattern, takes the result or
closes the menu.

**Authoritative state:**

Banner `0`, dye `1`, optional pattern item `2`, result `3`; selected index, ordered selectable
holder list, banner-pattern registry/tags, item `DYE` and `PROVIDES_BANNER_PATTERNS` components,
existing ordered `BANNER_PATTERNS`, game time and table access.

**Transition and ordering:**

Banner admission requires `BannerItem`; dye requires both `LOOM_DYES` membership and a `DYE`
component; a pattern item requires both `LOOM_PATTERNS` and `PROVIDES_BANNER_PATTERNS`. With banner
and dye present, an empty pattern slot exposes the locked `NO_ITEM_REQUIRED` tag in tag order; a
pattern item exposes its component holder-set order. The official 43 IDs partition into 32 no-item
choices, ten pattern-item choices and `base`, which is not addable by either loom selection path. A
one-entry list auto-selects index `0`. Otherwise an old valid selected holder is remapped to its new
index only if it remains in the new list; invalid/removed selection becomes `-1`. A valid button
selection stores its index and creates a one-count banner copy, preserving components and appending
`(selectedHolder,dyeColor)` after all existing layers. Taking consumes one banner then one dye,
never the pattern item; callbacks refresh the result when both counts remain, otherwise selection
becomes `-1`. The take sound is coalesced to one per game-time value.

**Branches and aborts:**

Missing banner/dye clears result, choices and selection. A missing dye component or invalid button
creates no result. Six or more existing layers clears result and selection even with an otherwise
valid holder. The pattern item is a reusable selector, not a crafting ingredient. Closing
returns/drops all three inputs and abandons the preview.

**Constants and randomness:**

Slots `0..3`; selection sentinel `-1`; maximum preexisting layer count `5`, producing layer six;
official partition `32 + 10 + 1 = 43`. No branch consumes RNG.

**Side effects:**

Selection/list/result synchronization, one banner and dye consumed, appended pattern component,
interaction stat, result sound and close-time input return/drop; no recipe/XP or pattern-item
consumption.

**Gates:**

Server table interaction/range, exact slot predicates, dynamic registry/tag/component membership,
valid selection, fewer than six layers and destination capacity.

**Boundary cases and quirks:**

A component can expose multiple patterns, so pattern-item presence does not imply auto-selection.
Selection preservation is by holder equality, not by numeric index, across list replacement.
Re-clicking any valid index is handled and recomputes the result. Unlike the other audited result
menus, loom does not exclude its result slot from pickup-all, so a matching carried banner can take
one replenishing result during the generic single traversal. Existing banner components other than
the appended layer remain intact.

**Evidence:**

`OFF-SERVER-001`, `OFF-DATA-001`, `OFF-REPORT-001`;
`net.minecraft.world.level.block.LoomBlock#useWithoutItem(net.minecraft.world.level.block.state.BlockState,net.minecraft.world.level.Level,net.minecraft.core.BlockPos,net.minecraft.world.entity.player.Player,net.minecraft.world.phys.BlockHitResult)`,
`net.minecraft.world.inventory.LoomMenu#isPatternItem(net.minecraft.world.item.ItemStack)`,
`net.minecraft.world.inventory.LoomMenu#isDyeItem(net.minecraft.world.item.ItemStack)`,
`net.minecraft.world.inventory.LoomMenu#getSelectablePatterns(net.minecraft.world.item.ItemStack)`,
`net.minecraft.world.inventory.LoomMenu#slotsChanged(net.minecraft.world.Container)`,
`net.minecraft.world.inventory.LoomMenu#clickMenuButton(net.minecraft.world.entity.player.Player,int)`,
`net.minecraft.world.inventory.LoomMenu#setupResultSlot(net.minecraft.core.Holder)`,
`net.minecraft.world.inventory.LoomMenu$6#onTake(net.minecraft.world.entity.player.Player,net.minecraft.world.item.ItemStack)`;
`data/minecraft/banner_pattern/*.json`, `data/minecraft/tags/banner_pattern/**/*.json`,
`data/minecraft/tags/item/loom_dyes.json`, `data/minecraft/tags/item/loom_patterns.json`;
`EXP-ITM-003`.

**Test vectors:**

All 43 IDs by the three scopes; no item/one-holder/multi-holder selector; invalid/missing tag or
component; remap preserved/removed holder across list order changes; invalid and repeat button;
existing layers `0/5/6`; banner/dye counts `1/2`; verify pattern item remains; pickup-all result;
same/adjacent-tick takes; close.
