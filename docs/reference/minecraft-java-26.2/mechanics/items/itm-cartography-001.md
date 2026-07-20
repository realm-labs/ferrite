# Items mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ITM-CARTOGRAPHY-001` — Cartography consumes a map plus one material before post-processing the taken copy

**Parent:** `ITM-003`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — slot admission, three transformation previews, saved-map gates, take-time
post-processing and map-data allocation are explicit in locked source.

**Applies when:**

A player opens a cartography table, changes its map/material inputs, takes or clones the result, or
closes the menu.

**Authoritative state:**

Map-component input `0`, material `1`, result `2`, referenced saved-map data, current result, table
access, level map-ID allocator, game time and last result-sound time.

**Transition and ordering:**

The map slot accepts any stack carrying `MAP_ID`; the material slot accepts paper, an empty map or
glass pane. With both inputs present and resolvable saved data, paper previews a one-count
component-preserving copy marked `MAP_POST_PROCESSING=SCALE` only when the map is unlocked and scale
is below `4`; glass previews the analogous `LOCK` copy only when unlocked; an empty map always
previews two unchanged component-preserving copies. Taking removes one map and one material first,
then invokes the output item's crafted post-process. A scale/lock marker is removed and, on a server
level with still-resolvable source data, a fresh map ID is allocated: scaling installs a fresh map
at clamped `oldScale + 1`, passing the old center through the ordinary new-scale grid-centering
formula while retaining dimension/tracking flags but no copied pixels/markers; locking copies
center, scale, flags, dimension, colors, banners, decorations and tracked count into immutable
locked data. The output stack is repointed to that new ID. The take sound then emits at most once
per game-time value.

**Branches and aborts:**

Missing inputs clear a current preview. Paper at scale `4`, paper/glass on a locked map, an
unrecognized material or unresolved saved data creates no new preview. Empty-map duplication is
allowed for locked, maximum-scale and exploration maps. The result is excluded from pickup-all.
Closing discards the preview and returns/drops both inputs.

**Constants and randomness:**

Slots `0..2`; maximum scale `4`; paper/glass output count `1`; empty-map output count `2`; one fresh
map ID for scale or lock. For new scale `s`, grid size is `128 * 2^s` and each center coordinate
becomes `floor((oldCenter + 64) / size) * size + size / 2 - 64`. No branch consumes RNG.

**Side effects:**

Input consumption, output component mutation, fresh saved-map allocation, interaction stat, result
sound and close-time input return/drop; no recipe holder, recipe award or XP exists.

**Gates:**

Server table interaction and range, `MAP_ID`, resolvable map data, exact material ID, locked/scale
rules, result pickup or quick-move destination capacity.

**Boundary cases and quirks:**

Preview only marks deferred post-processing; allocation occurs on the actual taken stack after
inputs are consumed. Quick-move explicitly post-processes before copying into the destination, while
ordinary take post-processes in the result slot hook. If both inputs remain present but their map ID
changes from resolvable to unresolved, setup returns without clearing an already nonempty preview;
this stale-preview branch can therefore commit the prior copy while consuming the current inputs,
and post-processing follows the stale output's prior map ID when that data still exists. Creative
result cloning post-processes the cloned stack without consuming inputs.

**Evidence:**

`OFF-SERVER-001`, `OFF-DATA-001`;
`net.minecraft.world.level.block.CartographyTableBlock#useWithoutItem(net.minecraft.world.level.block.state.BlockState,net.minecraft.world.level.Level,net.minecraft.core.BlockPos,net.minecraft.world.entity.player.Player,net.minecraft.world.phys.BlockHitResult)`,
`net.minecraft.world.inventory.CartographyTableMenu#slotsChanged(net.minecraft.world.Container)`,
`net.minecraft.world.inventory.CartographyTableMenu#setupResultSlot(net.minecraft.world.item.ItemStack,net.minecraft.world.item.ItemStack,net.minecraft.world.item.ItemStack)`,
`net.minecraft.world.inventory.CartographyTableMenu$5#onTake(net.minecraft.world.entity.player.Player,net.minecraft.world.item.ItemStack)`,
`net.minecraft.world.inventory.CartographyTableMenu$5#safeClone(net.minecraft.world.entity.player.Player)`,
`net.minecraft.world.item.MapItem#onCraftedPostProcess(net.minecraft.world.item.ItemStack,net.minecraft.world.level.Level)`,
`net.minecraft.world.level.saveddata.maps.MapItemSavedData#scaled()`,
`net.minecraft.world.level.saveddata.maps.MapItemSavedData#locked()`; `EXP-ITM-003`.

**Test vectors:**

Missing/unresolved map data; paper at unlocked scales `0/3/4` and locked; glass unlocked/locked;
empty map on exploration/locked/max-scale data; component-bearing input;
ordinary/quick/creative-clone take; stale preview after resolvable-to-missing ID; repeated takes in
one/adjacent game times; full destination; close.
