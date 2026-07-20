# Items mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ITM-CONTAINER-MOVE-001` — Quick-move uses a two-pass transfer primitive and a locked per-menu route

**Parent:** `ITM-002`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — the common transfer primitive, 25 registered menu layouts, player inventory
layout, every concrete quick-move range/direction and slot-policy dispatch are explicit in locked
source.

**Applies when:**

`QUICK_MOVE` invokes a menu's `quickMoveStack`, or a menu implementation otherwise calls
`moveItemStackTo`.

**Authoritative state:**

Source stack object, half-open destination range and direction, ordered slot list, destination
contents and maxima, `mayPlace`, menu-special predicates and result/input slot hooks.

**Transition and ordering:**

`moveItemStackTo(source,start,end,reverse)` first scans every destination in direction order if
`source.isStackable`: for each nonempty same-item/same-components target it merges up to
`slot.getMaxStackSize(target)`, marks that slot changed, and keeps scanning until source empties or
range ends. This merge pass does not call `mayPlace`. If source remains, a second directional scan
stops at the first empty slot that accepts it, splits `min(sourceCount,slotMax(source))`, calls
`setByPlayer`, marks changed, and returns. The method returns whether either pass moved a positive
count. Concrete routes and menu indices are:

#### `generic_9x1` … `generic_9x6`

**Menu slots:**

`C=[0,9N)`, `P=[9N,9N+27)`, `H=[9N+27,9N+36)`

**Quick-move routing and special admission:**

`C -> P+H reverse`; player -> `C` forward. `N` is 1…6 and the backing container size must equal
`9N`. Its quick-move postlude never calls source `onTake`.

#### `generic_3x3`

**Menu slots:**

`C=[0,9)`, `P=[9,36)`, `H=[36,45)`

**Quick-move routing and special admission:**

`C -> player reverse`; player -> `C` forward.

#### `crafter_3x3`

**Menu slots:**

grid `[0,9)`, `P=[9,36)`, `H=[36,45)`, noninteractive preview `45`

**Quick-move routing and special admission:**

grid -> player reverse; player -> grid forward. Disabled grid slots reject empty placement; the
preview rejects pickup, placement, removal and modification. Because the merge pass ignores
`mayPlace`, an occupied slot disabled after insertion can still receive a same-stack merge.

#### `hopper`

**Menu slots:**

`C=[0,5)`, `P=[5,32)`, `H=[32,41)`

**Quick-move routing and special admission:**

`C -> player reverse`; player -> `C` forward.

#### `shulker_box`

**Menu slots:**

`C=[0,27)`, `P=[27,54)`, `H=[54,63)`

**Quick-move routing and special admission:**

`C -> player reverse`; player -> `C` forward. Empty placement in `C` requires
`item.canFitInsideContainerItems()`. Its quick-move postlude never calls source `onTake`.

#### `beacon`

**Menu slots:**

payment `0`, `P=[1,28)`, `H=[28,37)`

**Quick-move routing and special admission:**

payment -> player reverse and `onQuickCraft`; a player stack enters payment only when the slot is
empty, the item is in `BEACON_PAYMENT_ITEMS`, and source count is exactly `1`. Otherwise main and
hotbar toggle; fallback scans all player slots. Payment maximum is `1`.

#### `furnace`, `blast_furnace`, `smoker`

**Menu slots:**

input `0`, fuel `1`, result `2`, `P=[3,30)`, `H=[30,39)`

**Quick-move routing and special admission:**

result -> player reverse plus `onQuickCraft`; input/fuel -> player forward; player stack first
enters input if the menu recipe set accepts it, else fuel if burnable, else toggles main/hotbar.

#### `brewing_stand`

**Menu slots:**

bottles `0..2`, ingredient `3`, fuel `4`, `P=[5,32)`, `H=[32,41)`

**Quick-move routing and special admission:**

machine slots -> player reverse plus `onQuickCraft`; player stack tests brewing fuel first, then
ingredient, then potion/splash/lingering/glass-bottle slots, then main/hotbar toggle. A successful
first fuel transfer returns the empty sentinel immediately rather than running the usual source
cleanup/`onTake`; this also stops the generic repeat loop. Other successful routes invoke source
`onTake` with the original pre-transfer copy, not the remaining source stack. Bottle slots have
maximum `1`; fuel is `BREWING_FUEL`; ingredient uses `PotionBrewing.isIngredient`.

#### `crafting`

**Menu slots:**

result `0`, grid `[1,10)`, `P=[10,37)`, `H=[37,46)`

**Quick-move routing and special admission:**

result -> player reverse, `onCraftedBy`, `onQuickCraft`, `onTake`, then drop any source remainder
without random motion; grid -> player forward. Player first tries grid forward; only if that
transfer moves nothing does it toggle main/hotbar.

#### `cartography_table`

**Menu slots:**

map `0`, addition `1`, result `2`, `P=[3,30)`, `H=[30,39)`

**Quick-move routing and special admission:**

result -> player reverse after `onCraftedBy`, then output hooks/broadcast; inputs -> player. A stack
with `MAP_ID` targets `0`; paper, empty map or glass pane targets `1`; other items toggle
main/hotbar. Result rejects placement.

#### `enchantment`

**Menu slots:**

item `0`, lapis `1`, `P=[2,29)`, `H=[29,38)`

**Quick-move routing and special admission:**

either input -> player reverse; lapis -> slot `1`; any other player stack may move exactly one copy
to empty slot `0` (maximum `1`), otherwise fails.

#### `grindstone`

**Menu slots:**

inputs `0,1`, result `2`, `P=[3,30)`, `H=[30,39)`

**Quick-move routing and special admission:**

result -> player reverse plus `onQuickCraft`; inputs -> player. If either input is empty, a player
stack eligible by damageability or any enchantment tries `[0,2)`; when both inputs are occupied,
player stacks only toggle main/hotbar.

#### `anvil`

**Menu slots:**

inputs `0,1`, result `2`, `P=[3,30)`, `H=[30,39)`

**Quick-move routing and special admission:**

result -> player reverse plus `onQuickCraft`; inputs -> player; any player stack may try both inputs
forward; otherwise main/hotbar toggle. Result pickup is additionally gated by positive cost and
creative or sufficient levels; its consumption/cost transaction is `ITM-CRAFT-001`.

#### `smithing`

**Menu slots:**

template `0`, base `1`, addition `2`, result `3`, `P=[4,31)`, `H=[31,40)`

**Quick-move routing and special admission:**

result -> player reverse plus `onQuickCraft`; inputs -> player. Input routing is enabled only if at
least one matching recipe-property slot is empty; the range scan plus each slot predicate chooses
template/base/addition. Once enabled, the common merge pass can still merge a same-stack earlier
occupied input before filling the empty candidate.

#### `loom`

**Menu slots:**

banner `0`, dye `1`, pattern `2`, result `3`, `P=[4,31)`, `H=[31,40)`

**Quick-move routing and special admission:**

result -> player reverse plus `onQuickCraft`; inputs -> player. Player routes banner item,
`LOOM_DYES` with `DYE`, or `LOOM_PATTERNS` with `PROVIDES_BANNER_PATTERNS` to its single slot; other
items toggle main/hotbar.

#### `merchant`

**Menu slots:**

costs `0,1`, result `2`, `P=[3,30)`, `H=[30,39)`

**Quick-move routing and special admission:**

result -> player reverse, `onQuickCraft`, then trade sound; costs -> player; other player slots
toggle main/hotbar and do not automatically classify currency into cost slots.

#### `stonecutter`

**Menu slots:**

input `0`, result `1`, `P=[2,29)`, `H=[29,38)`

**Quick-move routing and special admission:**

result -> player reverse after `onCraftedBy`, then output hooks and remainder drop; input -> player;
a stack accepted by the stonecutter recipe set targets input; other items toggle main/hotbar.

#### `lectern`

**Menu slots:**

book `0`; no player slots

**Quick-move routing and special admission:**

`quickMoveStack` always returns empty. Slot `0` itself has the ordinary generic slot
pickup/placement policy when addressed by a click packet; the vanilla screen normally removes the
book through the dedicated button in `ITM-CONTAINER-CONTROL-001`.

The always-present, non-registered player `InventoryMenu` uses result `0`, 2x2 grid `[1,5)`, armor
`[5,9)` ordered by equipment mapping, `P=[9,36)`, `H=[36,45)`, offhand `45`. Result routes to
`[9,45)` reverse with crafting hooks/remainder drop; grid and armor route to `[9,45)`; an eligible
armor/offhand stack first targets its empty equipment slot; main/hotbar otherwise toggle, with a
final `[9,45)` fallback. Armor slots accept only stacks equippable in their matching
`EquipmentSlot`, have maximum `1`, and prevent pickup of a stack carrying the prevent-armor-change
enchantment effect by a non-creative player. Offhand is an ordinary-capacity slot that accepts any
item on direct placement; only automatic quick-move targeting requires
`getEquipmentSlotForItem(stack) == OFFHAND`.

**Branches and aborts:**

A missing/empty source or a failed destination returns empty. Except for the table's chest, shulker
and brewing early-return cases, a successful route empties or dirties the source; implementations
with an unchanged-count guard return empty before `onTake`, then call `onTake` after a positive
change. Cartography and stonecutter always dirty the source after routing and explicitly broadcast;
crafting and stonecutter drop any result-stack remainder after `onTake`. The generic outer
quick-move loop can consume multiple replenished result stacks. Result-slot business hooks may abort
or replace results according to their owning workstation rule.

**Constants and randomness:**

All indices and directions are fixed above. Transfer uses integer counts and no RNG. Player
inventory standard ordering is 27 main slots followed by nine hotbar slots in a menu, even though
`Inventory` storage indices differ.

**Side effects:**

Destination/source counts, dirty hooks, `setByPlayer`, result `onQuickCraft`/`onTake`,
crafting/trade sounds and stats where called, result replenishment, and synchronization. Storage
menus call `startOpen`/`stopOpen`; that viewer lifecycle is separate from stack routing.

**Gates:**

Menu `stillValid`, source `mayPickup`, destination emptiness/identity/capacity, destination
`mayPlace` only on empty pass, item tags/components/recipe sets listed above, result pickup policy,
disabled crafter slot and player equipment policy.

**Boundary cases and quirks:**

The destination interval is half-open. Reverse order affects which partial player stack fills first.
A merge can bypass destination `mayPlace`; implement the two passes separately. Returning an
original snapshot does not mean the entire stack moved. Outer repetition compares only item ID at
the replenished source. The lectern deliberately exposes no quick-move route.

**Evidence:**

`OFF-SERVER-001`, `OFF-REPORT-001`;
`net.minecraft.world.inventory.AbstractContainerMenu#moveItemStackTo(net.minecraft.world.item.ItemStack,int,int,boolean)`,
`net.minecraft.world.inventory.ChestMenu#quickMoveStack(net.minecraft.world.entity.player.Player,int)`,
`net.minecraft.world.inventory.DispenserMenu#quickMoveStack(net.minecraft.world.entity.player.Player,int)`,
`net.minecraft.world.inventory.CrafterMenu#quickMoveStack(net.minecraft.world.entity.player.Player,int)`,
`net.minecraft.world.inventory.HopperMenu#quickMoveStack(net.minecraft.world.entity.player.Player,int)`,
`net.minecraft.world.inventory.ShulkerBoxMenu#quickMoveStack(net.minecraft.world.entity.player.Player,int)`,
`net.minecraft.world.inventory.BeaconMenu#quickMoveStack(net.minecraft.world.entity.player.Player,int)`,
`net.minecraft.world.inventory.AbstractFurnaceMenu#quickMoveStack(net.minecraft.world.entity.player.Player,int)`,
`net.minecraft.world.inventory.BrewingStandMenu#quickMoveStack(net.minecraft.world.entity.player.Player,int)`,
`net.minecraft.world.inventory.CraftingMenu#quickMoveStack(net.minecraft.world.entity.player.Player,int)`,
`net.minecraft.world.inventory.CartographyTableMenu#quickMoveStack(net.minecraft.world.entity.player.Player,int)`,
`net.minecraft.world.inventory.EnchantmentMenu#quickMoveStack(net.minecraft.world.entity.player.Player,int)`,
`net.minecraft.world.inventory.GrindstoneMenu#quickMoveStack(net.minecraft.world.entity.player.Player,int)`,
`net.minecraft.world.inventory.ItemCombinerMenu#quickMoveStack(net.minecraft.world.entity.player.Player,int)`,
`net.minecraft.world.inventory.LoomMenu#quickMoveStack(net.minecraft.world.entity.player.Player,int)`,
`net.minecraft.world.inventory.MerchantMenu#quickMoveStack(net.minecraft.world.entity.player.Player,int)`,
`net.minecraft.world.inventory.StonecutterMenu#quickMoveStack(net.minecraft.world.entity.player.Player,int)`,
`net.minecraft.world.inventory.LecternMenu#quickMoveStack(net.minecraft.world.entity.player.Player,int)`,
`net.minecraft.world.inventory.InventoryMenu#quickMoveStack(net.minecraft.world.entity.player.Player,int)`;
registry membership from `reports/registries.json#minecraft:menu`; `EXP-ITM-002`.

**Test vectors:**

One partial same stack before one empty slot in both directions; occupied restricted slot merge
versus empty restricted slot; each of the 25 IDs with one machine-to-player and player-to-machine
transfer; all six chest row counts; crafter disabled occupied/empty slots and preview; beacon source
counts `1` and `2`; brewing dual fuel/ingredient item and successful fuel early-return; replenishing
crafting result; smithing match with empty versus occupied input; shulker-prohibited container item;
player binding armor and offhand.
