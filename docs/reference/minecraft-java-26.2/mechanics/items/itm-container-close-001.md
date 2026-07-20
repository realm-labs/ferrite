# Items mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ITM-CONTAINER-CLOSE-001` — Closing resolves cursor and transient inputs before returning to inventory menu

**Parent:** `ITM-002`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — explicit close, automatic invalidation, cursor disposition, snapshot transfer
and every registered menu's removal override are visible in locked source.

**Applies when:**

The client closes a screen, the server closes/replaces a menu, or the per-player tick finds
`stillValid == false`.

**Authoritative state:**

Current menu, cursor, player inventory/removal/disconnect state, transient input containers,
persistent backing containers, viewer state and inventory-menu local/remote snapshots.

**Transition and ordering:**

A client close packet is moved to the level thread and calls `doCloseContainer` without comparing
its carried container ID. A server-initiated or automatic close first sends a close packet for the
current ID, then calls the same method. `doCloseContainer` invokes current menu `removed(player)`,
transfers listener/remote snapshots for slots sharing the same backing-container identity and
container-slot index into `inventoryMenu`, and finally sets the current menu to `inventoryMenu`.
Normal return uses `Inventory.placeItemBackInInventory`: repeatedly choose a compatible slot with
remaining space, otherwise the first free slot, split up to item maximum into it and send its
player-inventory update; if neither exists, drop the remainder without random motion. During each
server-player tick, current menu changes are broadcast before `stillValid` is tested and an invalid
menu is closed.

**Branches and aborts:**

Base removal acts only for `ServerPlayer`. A nonempty cursor is cleared from the menu after
disposition: normally it is placed back into inventory; if the player is removed for a reason other
than dimension change, or has disconnected, it is dropped without random throw motion. Persistent
chest/dispenser/hopper/shulker/mount containers only stop viewer access. Crafting grid, player 2x2
grid, enchantment inputs, grindstone inputs, cartography inputs, loom inputs and anvil/smithing
inputs are removed slot-by-slot and passed through the same return-or-drop policy; their
preview/result is cleared without granting it. Furnace/brewing/crafter contents persist in their
backing block entity.

**Constants and randomness:**

Transient containers are traversed from index `0` upward. No RNG is consumed by resolution; item
drops use non-random throw mode.

**Side effects:**

Cursor clear; inventory insertion or world item entities; transient input/result clearing; viewer
`stopOpen`; menu-specific behavior: beacon always removes its payment and drops it without random
motion on the server; merchant clears trading player and returns both cost slots to a
connected/alive server player's inventory, but drops them if dead or disconnected; snapshot state
moves to inventory menu; client screen close.

**Gates:**

Explicit/automatic close, server versus client side, player removed reason, disconnection/alive
state, menu subclass and `ContainerLevelAccess` execution.

**Boundary cases and quirks:**

The close packet's container ID is informational in 26.2 server handling: a delayed close for an old
menu closes the current menu. Dimension change is the one removed-player reason that still follows
inventory placement rather than the removed-player drop branch. Beacon payment differs from ordinary
transient inputs by always dropping instead of returning. The inventory menu itself clears its
result and 2x2 inputs when removed.

**Evidence:**

`OFF-SERVER-001`;
`net.minecraft.server.network.ServerGamePacketListenerImpl#handleContainerClose(net.minecraft.network.protocol.game.ServerboundContainerClosePacket)`,
`net.minecraft.server.level.ServerPlayer#closeContainer()`,
`net.minecraft.server.level.ServerPlayer#doCloseContainer()`,
`net.minecraft.world.inventory.AbstractContainerMenu#removed(net.minecraft.world.entity.player.Player)`,
`net.minecraft.world.inventory.AbstractContainerMenu#clearContainer(net.minecraft.world.entity.player.Player,net.minecraft.world.Container)`,
`net.minecraft.world.inventory.AbstractContainerMenu#transferState(net.minecraft.world.inventory.AbstractContainerMenu)`,
`net.minecraft.world.entity.player.Inventory#placeItemBackInInventory(net.minecraft.world.item.ItemStack,boolean)`,
and removal overrides in `net.minecraft.world.inventory.BeaconMenu`,
`net.minecraft.world.inventory.CartographyTableMenu`, `net.minecraft.world.inventory.CraftingMenu`,
`net.minecraft.world.inventory.EnchantmentMenu`, `net.minecraft.world.inventory.GrindstoneMenu`,
`net.minecraft.world.inventory.InventoryMenu`, `net.minecraft.world.inventory.ItemCombinerMenu`,
`net.minecraft.world.inventory.LoomMenu`, `net.minecraft.world.inventory.MerchantMenu`,
`net.minecraft.world.inventory.StonecutterMenu`; `EXP-ITM-002`.

**Test vectors:**

Normal close with cursor and free/full inventory; disconnect and non-dimension removal; dimension
change; delayed old-ID close after opening a new menu; automatic distance invalidation; each
transient input menu; persistent furnace/brewing/crafter; beacon payment; merchant while
alive/connected, dead and disconnected; verify inventory slots retain remote snapshots and do not
spuriously resend after transfer.
