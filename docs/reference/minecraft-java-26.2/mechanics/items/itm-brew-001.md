# Items mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ITM-BREW-001` — Brewing fuel starts a 400-tick same-ingredient transaction over three bottles

**Parent:** `ITM-005`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — the ordered vanilla mix table, timer/fuel order, cancellation identity,
three-slot commit, ingredient remainder and sided policies are explicit in locked source.

**Applies when:**

A brewing stand block entity ticks or automation inserts/extracts one of its five slots.

**Authoritative state:**

Bottle slots `0..2`, ingredient `3`, fuel `4`; brew time, fuel uses, remembered ingredient item,
ordered container/potion mix tables, bottle-presence block state and item/potion components.

**Transition and ordering:**

Every tick first refills empty fuel uses from one `BREWING_FUEL` item, setting uses to `20` even
when nothing is brewable. Brewability requires a recognized nonempty ingredient and at least one
nonempty bottle for which an ordered mix exists. If a brew is active, decrement time. At zero,
commit only if currently brewable: for slots `0..2` in order, container mixes are tested before
potion mixes and the first edge replaces that bottle; unmatched bottles remain unchanged. Then
shrink ingredient by one; if its item has a crafting remainder, install it when the ingredient
became empty or drop it at the stand when not empty, and emit level event `1035`. Before zero,
cancel immediately when no mix remains or the current ingredient item ID differs from the remembered
one. If idle and brewable with positive fuel uses, decrement uses, set time to `400`, and remember
the ingredient item. Finally recompute three bottle-presence booleans and update the block state if
they changed.

**Branches and aborts:**

Fuel can be prepaid without bottles/ingredient. Same item ID with changed components/count does not
itself cancel, but final brewability still must pass. Empty potion holder or absent edge leaves a
bottle unchanged. A container conversion preserves only the potion holder while changing item; a
potion conversion preserves container item and installs the target potion holder, so
unrelated/custom potion-content details are not copied.

**Constants and randomness:**

Fuel batch `20`; brew duration `400` ticks; three bottles. Mix membership/order is the locked
feature-filtered table constructed by `PotionBrewing.addVanillaMixes`. No brewing branch consumes
RNG.

**Side effects:**

Fuel/ingredient/bottle stacks, timers, bottle block properties, dropped ingredient remainder, stand
dirty state and brew level event. There is no recipe unlock or XP.

**Gates:**

Ticking chunk, fuel tag, recognized container/potion input and ingredient edge, remembered
ingredient item while active, slot policies and feature-filtered mix registration.

**Boundary cases and quirks:**

Fuel refill precedes brewability and can consume fuel uselessly. An active brew consumes no
additional fuel at completion. Up exposes ingredient; down exposes bottle slots plus ingredient;
sides expose bottles plus fuel. Ingredient slot extraction is allowed only when it contains a glass
bottle remainder. Direct bottle placement accepts potion/splash/lingering/glass-bottle items only
into an empty bottle slot.

**Evidence:**

`OFF-SERVER-001`, `OFF-DATA-001`;
`net.minecraft.world.level.block.entity.BrewingStandBlockEntity#serverTick(net.minecraft.world.level.Level,net.minecraft.core.BlockPos,net.minecraft.world.level.block.state.BlockState,net.minecraft.world.level.block.entity.BrewingStandBlockEntity)`,
`net.minecraft.world.level.block.entity.BrewingStandBlockEntity#isBrewable(net.minecraft.world.item.alchemy.PotionBrewing,net.minecraft.core.NonNullList)`,
`net.minecraft.world.level.block.entity.BrewingStandBlockEntity#doBrew(net.minecraft.world.level.Level,net.minecraft.core.BlockPos,net.minecraft.core.NonNullList)`,
`net.minecraft.world.item.alchemy.PotionBrewing#addVanillaMixes(net.minecraft.world.item.alchemy.PotionBrewing$Builder)`,
`net.minecraft.world.item.alchemy.PotionBrewing#mix(net.minecraft.world.item.ItemStack,net.minecraft.world.item.ItemStack)`,
`net.minecraft.world.level.block.entity.BrewingStandBlockEntity#canPlaceItem(int,net.minecraft.world.item.ItemStack)`,
`net.minecraft.world.level.block.entity.BrewingStandBlockEntity#canTakeItemThroughFace(int,net.minecraft.world.item.ItemStack,net.minecraft.core.Direction)`;
`EXP-ITM-003`.

**Test vectors:**

Fuel with empty stand; start at uses `1`; replace ingredient with same/different ID; remove last
valid bottle at `1`; one/two/three valid mixes; container versus potion edge; custom potion
components; ingredient remainder into empty/occupied slot; all automation faces.
