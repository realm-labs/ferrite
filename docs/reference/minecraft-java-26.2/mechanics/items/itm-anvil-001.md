# Items mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ITM-ANVIL-001` — Anvil preview prices repair, enchantment merge and rename before a level-paid damaging commit

**Parent:** `ITM-003`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — all-item admission, both repair paths, enchantment compatibility/level/cost
data, prior-work arithmetic, rename validation, pickup gates, consumption and block-damage sampling
are explicit in locked source/data.

**Applies when:**

A player opens any block in `minecraft:anvil`, changes either input, submits a rename, takes the
result, or closes the menu.

**Authoritative state:**

Base input `0`, addition `1`, result `2`; all item components, especially damage/maximum damage,
repairable holder set, ordinary/stored enchantments, repair cost and custom name; all 43 enchantment
definitions and exclusive sets; submitted filtered name; synchronized level cost; player
levels/infinite-materials ability; table block state, position and player RNG.

**Transition and ordering:**

Both input slots admit every stack and every one of the 1,537 official item defaults can store its
applicable enchantment component, so every ID can at least enter the rename path. Each input change
rebuilds a copy of the entire base stack. An empty base or one unable to store its applicable
enchantment component clears result and cost. The base and addition repair costs are summed as prior
work. If a damageable base accepts the addition through its `REPAIRABLE` holder set, each addition
item removes up to `floor(maxDamage/4)` damage, costs one operation level and repeats until repaired
or the addition count is exhausted; a zero first repair amount aborts the whole preview. This
material path does not merge enchantments. Otherwise, the addition must carry `STORED_ENCHANTMENTS`
or be the same item as a damageable base. A same-item sacrifice adds its own remaining durability
plus `floor(0.12 * baseMaxDamage)`; only a strict damage reduction applies and costs two operation
levels.

**Enchantment merge:**

Addition crafting enchantments are visited in the component's unsorted fastutil-map iteration order.
Equal existing/incoming levels propose one higher; unequal levels propose their maximum; an accepted
level is capped to the definition maximum. Acceptance requires the enchantment's supported-item set
to contain the base, except that an enchanted-book base or an infinite-materials player bypasses
that gate, and it must be compatible with every different enchantment already in the mutable output.
Each incompatible incumbent adds one penalty level even when the candidate is rejected. An accepted
candidate costs `newLevel * anvilCost`; an addition carrying `STORED_ENCHANTMENTS` changes the unit
cost to `max(1,floor(anvilCost/2))`. If any candidate was incompatible and none was accepted, the
entire result aborts, including an otherwise valid sacrifice repair. Accepting an enchantment onto a
base stack count above one sets the running operation cost to `40` before later candidates continue.

**Rename and total cost:**

The server removes disallowed chat characters, rejects the update without changing state when the
filtered Java-string length exceeds `50`, and ignores an unchanged accepted name. Null, empty or
all-whitespace name removes an existing `CUSTOM_NAME` for one operation level; a nonblank string
differing from the base hover-name text installs a literal custom name for one. With no operation
the result is empty and cost is zero. Otherwise displayed cost is the prior-work sum plus operation
levels, clamped to signed-int maximum. A pure one-level rename is marked rename-only: total cost at
least `40` is reduced to `39`, the output repair cost remains the greater input repair cost, and
addition slot `1` will not be consumed. Every other operation rejects the survival preview at total
cost `40` or above and writes output repair cost as `min(2 * max(inputRepairCosts) + 1, INT_MAX)`.

**Side effects:**

Result pickup requires positive cost and either enough experience levels or infinite materials.
Successful take first removes that many levels unless infinite, then consumes exactly the counted
material-repair additions; otherwise it clears the whole addition except for rename-only. It resets
synchronized cost, submits a differing nonblank name to the player's text filter without awaiting or
rewriting the already-taken output, clears the entire base slot, and executes at the table position.
A non-infinite player at an anvil-tag block consumes exactly one `nextFloat`: below `0.12`, anvil
becomes chipped, chipped becomes damaged, or damaged is removed without drops. State damage
preserves facing; transitions emit event `1030`, final removal emits `1029`. Every no-damage,
creative or non-anvil callback emits `1030`; creative and non-anvil callbacks consume no RNG.
Opening awards the anvil interaction statistic. There is no recipe or XP-orb award.

**Branches and aborts:**

An unlike-item addition without `STORED_ENCHANTMENTS`, fully repaired/quarter-zero material repair,
no effective operation, all-incompatible additions, insufficient survival levels and non-rename
total cost at least `40` produce no takeable result. Unsupported but mutually compatible addition
enchantments are silently skipped. The output otherwise retains the base's count and nonmodified
components; taking it clears the whole base stack. Closing returns/drops both inputs and discards
the preview.

**Constants and randomness:**

Slots `0..2`; material repair quarter `floor(maxDamage/4)` and cost `1` per consumed item; sacrifice
bonus `floor(12% of base maximum)` and cost `2` only when it improves damage; incompatibility
penalty `1` per conflicting incumbent; rename cost `1`; survival ceiling `39`; name limit `50`
UTF-16 code units after character filtering; anvil damage probability `0.12` from one player-RNG
float per eligible commit.

**Gates:**

Current anvil-tag block and range, base enchantment-component presence, repairable holder membership
or combine compatibility, enchantment supported/exclusive data, positive affordable level cost,
destination capacity and result pickup.

**Boundary cases and quirks:**

Prior-work values affect displayed/paid cost only after an effective operation. Pure rename can
therefore collapse arbitrarily high accumulated cost to `39` and does not advance prior work. A
same-item addition that performs no repair/enchantment can remain present during such a rename and
is preserved on take. Material repair preempts any enchantments carried by that material stack. The
mutable enchantment map makes earlier accepted incoming enchantments participate in compatibility
checks for later entries. The result slot does not exclude pickup-all, so a matching carried stack
can commit it during generic traversal. Anvils also retain the independent falling-block behavior
specified by `BLK-FALL-001`.

**Evidence:**

`OFF-SERVER-001`, `OFF-DATA-001`, `OFF-REPORT-001`;
`net.minecraft.server.network.ServerGamePacketListenerImpl#handleRenameItem(net.minecraft.network.protocol.game.ServerboundRenameItemPacket)`,
`net.minecraft.world.level.block.AnvilBlock#useWithoutItem(net.minecraft.world.level.block.state.BlockState,net.minecraft.world.level.Level,net.minecraft.core.BlockPos,net.minecraft.world.entity.player.Player,net.minecraft.world.phys.BlockHitResult)`,
`net.minecraft.world.level.block.AnvilBlock#damage(net.minecraft.world.level.block.state.BlockState)`,
`net.minecraft.world.inventory.AnvilMenu#createResult()`,
`net.minecraft.world.inventory.AnvilMenu#mayPickup(net.minecraft.world.entity.player.Player,boolean)`,
`net.minecraft.world.inventory.AnvilMenu#onTake(net.minecraft.world.entity.player.Player,net.minecraft.world.item.ItemStack)`,
`net.minecraft.world.inventory.AnvilMenu#setItemName(java.lang.String)`,
`net.minecraft.world.inventory.AnvilMenu#validateName(java.lang.String)`,
`net.minecraft.world.inventory.AnvilMenu#calculateIncreasedRepairCost(int)`,
`net.minecraft.world.item.ItemStack#isValidRepairItem(net.minecraft.world.item.ItemStack)`,
`net.minecraft.world.item.enchantment.EnchantmentHelper#canStoreEnchantments(net.minecraft.world.item.ItemStack)`,
`net.minecraft.world.item.enchantment.EnchantmentHelper#getEnchantmentsForCrafting(net.minecraft.world.item.ItemStack)`,
`net.minecraft.world.item.enchantment.EnchantmentHelper#setEnchantments(net.minecraft.world.item.ItemStack,net.minecraft.world.item.enchantment.ItemEnchantments)`,
`net.minecraft.world.item.enchantment.Enchantment#canEnchant(net.minecraft.world.item.ItemStack)`,
`net.minecraft.world.item.enchantment.Enchantment#areCompatible(net.minecraft.core.Holder,net.minecraft.core.Holder)`,
`net.minecraft.world.item.enchantment.Enchantment#getAnvilCost()`; item-component reports,
`data/minecraft/enchantment/**/*.json`, `data/minecraft/tags/block/anvil.json`,
`data/minecraft/tags/enchantment/exclusive_set/**/*.json`, material/repair item tags; `EXP-ITM-003`.

**Test vectors:**

Every official item default as base; all 75 repairable defaults and every declared material tag;
damage `0/1`, maximum below/at/above `4`, insufficient/exact/excess material; same-item sacrifice
with differing component maximum/damage; all 43 enchantments at unequal/equal/maximum/over-maximum
levels, stored versus ordinary addition, supported/unsupported and every exclusive-set collision in
both iteration orders; base count `1/2`; repair costs `0/1/39/INT_MAX`;
null/blank/same/filtered/50/51-unit names; combined versus rename-only cost `39/40`;
insufficient/exact/creative levels; ordinary/quick/pickup-all/full-destination take; each anvil
state at RNG `<`, `=` and `>` `0.12`; block replacement and close.

`ITM-CRAFT-PROCESS-001` is source-complete: all recipe, ticked processor and workstation leaves have
no remaining source unknowns; `EXP-ITM-003` remains a regression probe rather than unresolved
evidence.
