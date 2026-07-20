# Items mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ITM-FURNACE-001` — Furnace-family ticks spend burn time before advancing one validated cook

**Parent:** `ITM-005`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — timer order, fuel table construction, output admission, completion, sided access
and delayed XP award are explicit in locked source.

**Applies when:**

A furnace, blast furnace or smoker block entity ticks, its input changes, or a player removes result
items.

**Authoritative state:**

Input `0`, fuel `1`, result `2`; lit remaining/total, cook current/total, the machine's recipe-type
cache, source-defined fuel-duration table, recipe-use counts and block `LIT` state.

**Transition and ordering:**

At the start of every block-entity tick, positive lit time loses one and current lit state is
computed from the remaining value. When either still lit or both fuel and input are present,
nonempty input is matched in the machine's recipe domain and assembled. The result is burnable only
if nonempty and the output is empty or same-item/same-components with
`oldCount + resultCount <= min(containerMax,resultMax)`. If burnable but unlit, read fuel duration,
assign both lit timers, and on a positive duration consume exactly one fuel, installing its crafting
remainder only if the fuel stack became empty. If now lit, increment cook time; equality with total
resets progress to zero, refreshes total from recipe data, installs/grows output, applies the
wet-sponge plus bucket conversion, shrinks input by one and increments that recipe's use count.
Invalid input/recipe/output resets cook time immediately. Only the fully inactive unlit branch cools
positive progress by `2` per tick, clamped to `[0,total]`. A changed input item/components resets
progress and recomputes total; a count-only same-stack change does not.

**Branches and aborts:**

No input, no matching recipe, disabled/empty result, incompatible/full output or zero-duration fuel
prevents completion. A lit machine with invalid work continues spending fuel while progress is held
at zero. Lit time reaching zero can ignite new fuel in the same tick. Fuel exhaustion on the
completion tick does not prevent that tick's cook increment because decrement occurs before
validation and lit is re-evaluated.

**Constants and randomness:**

Default cook total is `200` when no recipe supplies one; cooling step is `2`. Fuel duration is the
26.2 `FuelValues.vanillaBurnTimes` table with standard unit `200`, explicit item/tag multipliers and
final `NON_FLAMMABLE_WOOD` removal. Smelting/blasting/smoking cooking time and experience are recipe
data. Cooking consumes no RNG. When a player extracts output, each accumulated recipe count produces
`floor(count * experience)` XP plus one with probability equal to the fractional part, consuming one
level RNG float only when that fraction is nonzero.

**Side effects:**

Slots/timers, `LIT` block update, recipe-use counts, dirty state, result crafted hook, recipe
unlocks/criteria and delayed XP orbs at the player. Hopper extraction never invokes
`FurnaceResultSlot`, so counts/XP remain accumulated until player extraction or block removal side
effects request them.

**Gates:**

Ticking chunk, recipe type and match, enabled result, fuel duration, output capacity, input identity
and result-slot player take for XP.

**Boundary cases and quirks:**

Up exposes input; sides expose fuel; down exposes result then fuel. Downward extraction from fuel
slot permits only empty or water buckets. An empty bucket may enter fuel only when the fuel slot
does not already contain a bucket, supporting the wet-sponge conversion. Progress increments on the
same tick fuel ignites. Timer increments alone do not mark the block entity dirty every tick;
ignition, completion, input replacement and lit-state change do.

**Evidence:**

`OFF-SERVER-001`, `OFF-DATA-001`;
`net.minecraft.world.level.block.entity.AbstractFurnaceBlockEntity#serverTick(net.minecraft.server.level.ServerLevel,net.minecraft.core.BlockPos,net.minecraft.world.level.block.state.BlockState,net.minecraft.world.level.block.entity.AbstractFurnaceBlockEntity)`,
`net.minecraft.world.level.block.entity.AbstractFurnaceBlockEntity#canBurn(net.minecraft.core.NonNullList,int,net.minecraft.world.item.ItemStack)`,
`net.minecraft.world.level.block.entity.AbstractFurnaceBlockEntity#burn(net.minecraft.core.NonNullList,net.minecraft.world.item.ItemStack,net.minecraft.world.item.ItemStack)`,
`net.minecraft.world.level.block.entity.AbstractFurnaceBlockEntity#setItem(int,net.minecraft.world.item.ItemStack)`,
`net.minecraft.world.level.block.entity.AbstractFurnaceBlockEntity#getRecipesToAwardAndPopExperience(net.minecraft.server.level.ServerLevel,net.minecraft.world.phys.Vec3)`,
`net.minecraft.world.level.block.entity.AbstractFurnaceBlockEntity#createExperience(net.minecraft.server.level.ServerLevel,net.minecraft.world.phys.Vec3,int,float)`,
`net.minecraft.world.level.block.entity.FuelValues#vanillaBurnTimes(net.minecraft.core.HolderLookup$Provider,net.minecraft.world.flag.FeatureFlagSet,int)`,
`net.minecraft.world.inventory.FurnaceResultSlot#checkTakeAchievements(net.minecraft.world.item.ItemStack)`;
`EXP-ITM-003`.

**Test vectors:**

Each of three machine domains; ignition/completion with lit time `0/1`; invalid work while
lit/unlit; count-only versus component input change; full/incompatible output; multi-count result
capacity; wet sponge with bucket; fuel remainder; every side; hopper then player extraction; XP
products below/equal/above an integer.
