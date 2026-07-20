# Items mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ITM-GRINDSTONE-001` — Grindstone commit removes non-curses, resets prior-work cost and grants randomized removal XP

**Parent:** `ITM-003`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — admission, single/two-input result construction, damage repair, curse
merge/removal, book conversion, repair-cost reset, take ordering and XP sampling are explicit in
locked source/data.

**Applies when:**

A player opens a grindstone, changes either input, takes its result by any ordinary menu path, or
closes the menu.

**Authoritative state:**

Inputs `0,1`, result `2`, item/component identity, declared maximum damage and current damage,
ordinary or stored enchantment component, `minecraft:curse` tag membership, each enchantment's
minimum-cost curve, repair-cost component, table access, server RNG and block position.

**Transition and ordering:**

Each input slot admits a damageable stack or any stack carrying at least one ordinary or stored
enchantment. Every input change recomputes and broadcasts the preview. An input count above one
aborts. One occupied input produces a one-count copy only when it has enchantments, then removes
every non-curse. Two occupied inputs require the same item ID. For a damageable pair, remaining
durability is
`(maxDamage1 - damage1) + (maxDamage2 - damage2) + floor(0.05 * max(maxDamage1,maxDamage2))`; the
output copies input one, sets its maximum damage to the larger maximum and sets damage to
`max(maxDamage - remainingDurability,0)`. For a nondamageable pair, input one must support a stack
of at least two and both stacks must fully match; the output is a two-count copy. Curses from input
two are added only when absent, so an existing input-one curse level is never upgraded; all
non-curses are then removed. The output repair cost is rebuilt from zero by doubling-and-adding-one
once per remaining distinct curse, hence `2^curseCount - 1` with integer saturation. An enchanted
book with no remaining curse transmutes to an ordinary book.

**Branches and aborts:**

Empty inputs, over-count inputs, unlike item IDs, a lone unenchanted damageable item, or
nondamageable stacks without full equality produce no result. The first input owns all
non-enchantment output components; the second input contributes no such component. The result has no
recipe holder or crafted-item callback. Closing returns/drops both inputs and discards the preview.

**Constants and randomness:**

Slots `0..2`; repair bonus `floor(5% of larger maximum damage)`; locked official curse tag contains
`binding_curse` and `vanishing_curse`, so its reachable reset costs are `0`, `1` and `3`. For each
input, let its removed-enchantment value be the sum of `Enchantment.getMinCost(level)` over
non-curse entries in the crafting enchantment component (`STORED_ENCHANTMENTS` for enchanted books,
`ENCHANTMENTS` otherwise). With combined value `S > 0`, set `b=ceil(S/2)` and grant
`b + nextInt(b)`, uniformly `b..2b-1` with exactly one RNG draw; `S=0` grants zero without a draw.

**Side effects:**

Taking first runs the table-access callback at block center: on a server level it computes XP from
the still-present original inputs, passes the positive amount to `ExperienceOrb.award`, and always
emits level event `1042`; it then clears both inputs, whose callbacks clear the preview. Opening
awards the grindstone interaction statistic. There is no recipe award.

**Gates:**

Server table interaction/range, input slot predicates, at most one item per occupied input, merge
compatibility, curse tag/data lookup and destination capacity.

**Boundary cases and quirks:**

Pair repair uses each stack's own declared maximum when calculating remaining durability but writes
the larger maximum to the result. Full-match nondamageable merging can create count two; damageable
merging does not require component equality. Because result take computes XP before clearing inputs,
both original enchantment sets contribute even though the preview contains only curses. Curse count,
not curse levels or incoming repair cost, controls the reset. The result slot does not opt out of
generic pickup-all, so a matching carried stack can commit one replenishing result during that
traversal. Grindstone placement has face/facing shapes but survives without a supporting block.

**Evidence:**

`OFF-SERVER-001`, `OFF-DATA-001`;
`net.minecraft.world.level.block.GrindstoneBlock#useWithoutItem(net.minecraft.world.level.block.state.BlockState,net.minecraft.world.level.Level,net.minecraft.core.BlockPos,net.minecraft.world.entity.player.Player,net.minecraft.world.phys.BlockHitResult)`,
`net.minecraft.world.inventory.GrindstoneMenu#computeResult(net.minecraft.world.item.ItemStack,net.minecraft.world.item.ItemStack)`,
`net.minecraft.world.inventory.GrindstoneMenu#mergeItems(net.minecraft.world.item.ItemStack,net.minecraft.world.item.ItemStack)`,
`net.minecraft.world.inventory.GrindstoneMenu#mergeEnchantsFrom(net.minecraft.world.item.ItemStack,net.minecraft.world.item.ItemStack)`,
`net.minecraft.world.inventory.GrindstoneMenu#removeNonCursesFrom(net.minecraft.world.item.ItemStack)`,
`net.minecraft.world.inventory.GrindstoneMenu$4#onTake(net.minecraft.world.entity.player.Player,net.minecraft.world.item.ItemStack)`,
`net.minecraft.world.inventory.GrindstoneMenu$4#getExperienceAmount(net.minecraft.world.level.Level)`,
`net.minecraft.world.inventory.GrindstoneMenu$4#getExperienceFromItem(net.minecraft.world.item.ItemStack)`,
`net.minecraft.world.inventory.AnvilMenu#calculateIncreasedRepairCost(int)`;
`data/minecraft/enchantment/**/*.json`, `data/minecraft/tags/enchantment/curse.json`; `EXP-ITM-003`.

**Test vectors:**

Empty/one/two inputs; counts `1/2`; lone enchanted versus unenchanted damageable item; unequal IDs;
damageable pairs with unequal declared maxima and over-repair; nondamageable full match/mismatch and
max-stack one/two; first/second curse levels; zero/one/two curses; enchanted book with/without a
curse; incoming repair costs; all 43 enchantment minimum-cost entries; XP totals even/odd/zero and
RNG bounds; ordinary/quick/pickup-all take; full destination; close; unsupported placement.
