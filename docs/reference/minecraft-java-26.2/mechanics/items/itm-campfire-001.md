# Items mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ITM-CAMPFIRE-001` — Four campfire slots advance independently and eject at their stored deadlines

**Parent:** `ITM-005`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — placement, four-slot iteration, lit/cooldown tickers, completion fallback and
removal drops are explicit in locked source.

**Applies when:**

A player uses a campfire-cookable item on a campfire, or a lit/unlit campfire block entity ticks.

**Authoritative state:**

Four one-item slots in index order, per-slot progress/total, current campfire recipe manager, block
lit/waterlogged state and the interacting entity's stack.

**Transition and ordering:**

Interaction first tests the synchronized campfire-input property set. On the server, scan slots
`0..3` and use the first empty one; first-match lookup must currently succeed, then store its
cooking time, zero progress, and move exactly one item by `consumeAndReturn`, emitting the
block-change event and interaction stat. While lit, scan slots `0..3`; each nonempty slot increments
progress. At `progress >= storedTotal`, re-run recipe lookup and assemble it, or fall back to the
stored input stack if the recipe disappeared; if that output item is enabled, drop it at integer
block coordinates, clear the slot, send a block update and emit a block-change game event. While
unlit, each positive progress loses `2`, clamped to `[0,storedTotal]`.

**Branches and aborts:**

A non-input item, no empty slot, or no current recipe does not consume. At deadline, a disabled
assembled/fallback item leaves the occupied slot at or beyond its deadline and retries on later lit
ticks. Recipe reload may change the eventual result because completion re-resolves instead of
retaining the placement holder. Waterlogging/extinguishing selects cooldown rather than cook ticks.

**Constants and randomness:**

Four slots; cooldown step `2`; recipe cooking time is locked data. The processing algorithm consumes
no RNG; dropped item-entity motion uses the ordinary container-drop path.

**Side effects:**

Hand count, campfire slots/timers, interaction stat, block update/game event, dropped result, dirty
state; breaking/removing the block entity drops all still-stored inputs.

**Gates:**

Campfire input property set, server side, empty slot, recipe match at placement, lit state, output
feature enablement at completion.

**Boundary cases and quirks:**

The placed one-count stack retains the consumed stack's components. A recipe removed mid-cook causes
the original input—not an empty result—to be ejected at the original stored deadline. Progress is
independent per slot and multiple indices may complete in the same tick in ascending order. Campfire
cooking awards neither recipe unlock nor recipe XP.

**Evidence:**

`OFF-SERVER-001`, `OFF-DATA-001`;
`net.minecraft.world.level.block.CampfireBlock#useItemOn(net.minecraft.world.item.ItemStack,net.minecraft.world.level.block.state.BlockState,net.minecraft.world.level.Level,net.minecraft.core.BlockPos,net.minecraft.world.entity.player.Player,net.minecraft.world.InteractionHand,net.minecraft.world.phys.BlockHitResult)`,
`net.minecraft.world.level.block.entity.CampfireBlockEntity#placeFood(net.minecraft.server.level.ServerLevel,net.minecraft.world.entity.LivingEntity,net.minecraft.world.item.ItemStack)`,
`net.minecraft.world.level.block.entity.CampfireBlockEntity#cookTick(net.minecraft.server.level.ServerLevel,net.minecraft.core.BlockPos,net.minecraft.world.level.block.state.BlockState,net.minecraft.world.level.block.entity.CampfireBlockEntity,net.minecraft.world.item.crafting.RecipeManager$CachedCheck)`,
`net.minecraft.world.level.block.entity.CampfireBlockEntity#cooldownTick(net.minecraft.world.level.Level,net.minecraft.core.BlockPos,net.minecraft.world.level.block.state.BlockState,net.minecraft.world.level.block.entity.CampfireBlockEntity)`,
`net.minecraft.world.level.block.entity.CampfireBlockEntity#preRemoveSideEffects(net.minecraft.core.BlockPos,net.minecraft.world.level.block.state.BlockState)`;
`EXP-ITM-003`.

**Test vectors:**

Fill five attempts; four simultaneous deadlines; extinguish/relight at progress boundaries; recipe
removal/replacement; disabled result then feature enable; component-bearing input; break during
cook; verify no XP/unlock.
