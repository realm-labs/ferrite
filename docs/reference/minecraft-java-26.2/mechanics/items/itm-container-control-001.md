# Items mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ITM-CONTAINER-CONTROL-001` — Menu controls are separate server-validated transactions

**Parent:** `ITM-002`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — generic button admission and all menu-specific non-click control entry points
are located; their downstream recipe/enchantment/trade computations remain in their owning behavior
rules.

**Applies when:**

A menu screen sends a button, anvil rename, beacon selection, merchant selection or crafter
slot-state request rather than a slot click.

**Authoritative state:**

Current menu/container ID and class, `stillValid`, spectator state, menu data slots, inputs/results
and the control's selection/cost state.

**Transition and ordering:**

A generic container-button request resets idle time, requires matching current container ID, rejects
spectators, checks `stillValid`, invokes `clickMenuButton`, and calls `broadcastChanges` only when
it returns true. Concrete button meanings are: enchantment IDs `0..2`; loom pattern index within
current selectable patterns; stonecutter recipe index (same index returns false, a different invalid
index returns true without selecting); lectern `1` previous, `2` next, `3` take book, and every ID
`>=100` requests page `id-100`. For enchantment button `b`, require `0 <= b < 3`, nonempty item,
positive displayed cost, and—unless infinite materials—at least `b+1` lapis plus experience level at
least both `b+1` and displayed cost. Passing those gates returns true; only a nonempty generated
enchantment list commits resource/item/stat/sound effects. Lectern take requires `mayBuild`, removes
the book without update, dirties the lectern, and inserts it into player inventory or drops it
without random motion.

**Branches and aborts:**

Anvil rename is accepted only for current `AnvilMenu` with `stillValid`; the name is filtered,
rejected above 50 characters, ignored when unchanged, removes the result's custom name when blank,
otherwise installs a literal custom name, then recomputes the result. Beacon selection requires
current valid `BeaconMenu`; `updateEffects` must validate the chosen effects and consume an existing
payment, otherwise the connection is disconnected for invalid effects. Merchant selection requires
current valid `MerchantMenu`, sets the hint and invokes input auto-fill. Crafter slot-state change
rejects spectators and wrong container ID, then requires `CrafterMenu` backed by a real
`CrafterBlockEntity`; only an empty slot `0..8` can change, requested enabled stores `0`, disabled
stores `1`, and a change dirties the block entity. Unlike generic buttons, the crafter handler has
no explicit `stillValid` check.

**Constants and randomness:**

Button and selection IDs are integers. Lectern page offset is `100`. The menu transaction itself
consumes no RNG; enchantment choice/sound pitch and other downstream business algorithms own their
RNG.

**Side effects:**

Selection/data updates, result recomputation, inventory auto-fill, payment/input consumption,
block-entity dirty state, sounds/stats from downstream algorithms, delta synchronization, or
disconnection for invalid beacon selection.

**Gates:**

Current menu ID/class, spectator, `stillValid` where stated, player `mayBuild`, selection bounds,
resources/XP/payment, menu-specific data and backing block-entity identity.

**Boundary cases and quirks:**

Stonecutter acknowledges a different invalid recipe ID as successful and therefore broadcasts even
though selection does not change. Lectern page arithmetic is submitted without a menu-level clamp;
the backing data implementation owns any normalization. The generic button path rejects spectators
but has no dead/dying gate. Rename, beacon and merchant packets have neither a spectator gate nor a
packet container ID; they rely on current menu class plus `stillValid`. Crafter has
spectator/container-ID gates but no explicit validity gate. Close packet container IDs are not used
by the close handler and belong to `ITM-CONTAINER-CLOSE-001`.

**Evidence:**

`OFF-SERVER-001`;
`net.minecraft.server.network.ServerGamePacketListenerImpl#handleContainerButtonClick(net.minecraft.network.protocol.game.ServerboundContainerButtonClickPacket)`,
`net.minecraft.server.network.ServerGamePacketListenerImpl#handleRenameItem(net.minecraft.network.protocol.game.ServerboundRenameItemPacket)`,
`net.minecraft.server.network.ServerGamePacketListenerImpl#handleSetBeaconPacket(net.minecraft.network.protocol.game.ServerboundSetBeaconPacket)`,
`net.minecraft.server.network.ServerGamePacketListenerImpl#handleSelectTrade(net.minecraft.network.protocol.game.ServerboundSelectTradePacket)`,
`net.minecraft.server.network.ServerGamePacketListenerImpl#handleContainerSlotStateChanged(net.minecraft.network.protocol.game.ServerboundContainerSlotStateChangedPacket)`,
`net.minecraft.world.inventory.AnvilMenu#setItemName(java.lang.String)`,
`net.minecraft.world.level.block.entity.CrafterBlockEntity#setSlotState(int,boolean)`,
`net.minecraft.world.inventory.EnchantmentMenu#clickMenuButton(net.minecraft.world.entity.player.Player,int)`,
`net.minecraft.world.inventory.LecternMenu#clickMenuButton(net.minecraft.world.entity.player.Player,int)`,
`net.minecraft.world.inventory.LoomMenu#clickMenuButton(net.minecraft.world.entity.player.Player,int)`,
`net.minecraft.world.inventory.StonecutterMenu#clickMenuButton(net.minecraft.world.entity.player.Player,int)`;
`EXP-ITM-002`.

**Test vectors:**

Wrong/right container ID; spectator; invalid distance; enchant IDs `-1,0,2,3`; loom
first/last/out-of-range; stonecutter current, other valid and other invalid; lectern IDs
`1,2,3,99,100` with/without build permission and full inventory; rename after anvil invalidates;
beacon no payment/invalid tier/valid selection; merchant negative/out-of-range selection; crafter
wrong backing container and slot `-1,0,8,9`.
