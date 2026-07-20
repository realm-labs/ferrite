# Items mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ITM-RECIPE-SERIALIZER-001` — All 21 serializers reduce to audited matching and assembly families

**Parent:** `ITM-004`, `ITM-005`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — every registered serializer maps to a located class; all match, assembly,
component-copy and special-remainder branches are explicit in locked source.

**Applies when:**

`ITM-RECIPE-001` tests or assembles a decoded recipe.

**Authoritative state:**

Cropped input dimensions and row-major stacks, ingredient holder sets, serializer data, input
components, level map data for map extension, and smithing trim registries/components.

**Transition and ordering:**

`CraftingContainer.asPositionedCraftInput` removes completely empty outer rows and columns and
retains the offset used later by `ITM-CRAFT-001`. Ingredients test item holder/tag membership;
unless a family below names a component gate, input components do not affect matching. The
registered families are:

#### `crafting_shaped`

**Exact match and assembly semantics:**

Cropped dimensions and nonempty count must equal the pattern. Test every cell row-major; empty
pattern cells require empty stacks. Test the horizontal mirror first when the pattern is not
symmetric, then the ordinary orientation. Create the fixed result template.

#### `crafting_shapeless`

**Exact match and assembly semantics:**

Nonempty count must equal ingredient count. One-cell input uses one direct ingredient test;
otherwise a bipartite item-holder allocation must assign every stack to one ingredient. Create the
fixed result template.

#### `crafting_dye`

**Exact match and assembly semantics:**

Require exactly one target and one or more dye-ingredient stacks carrying `DYE`; reject every other
stack. Preserve the target's component patch on the result template, then replace `DYED_COLOR` with
the ordered dye blend.

#### `crafting_imbue`

**Exact match and assembly semantics:**

Require a full 3x3: the source at center and material in all eight other cells. Create the result
template and copy only `POTION_CONTENTS` from the center source.

#### `crafting_transmute`

**Exact match and assembly semantics:**

Require exactly one input and a data-bounded `1..8` material count, with no other stacks. Preserve
the input component patch on the result template. Result count is template count, optionally plus
material count. When result count is one, reject a result identical to the input; larger results may
retain identity.

#### `crafting_decorated_pot`

**Exact match and assembly semantics:**

Require a 3x3 with exactly four ingredients at top-middle, middle-left, middle-right and
bottom-middle, each tested against its face predicate. Create the result with `POT_DECORATIONS` in
back/left/right/front order from those item IDs.

#### `crafting_special_bookcloning`

**Exact match and assembly semantics:**

Require one source with `WRITTEN_BOOK_CONTENT` whose generation is in the recipe's allowed range,
plus one or more material stacks and no others. Preserve source components, advance book generation,
and produce `templateCount + materials - 1`. The source book is returned as a one-count remainder in
addition to ordinary item remainders.

#### `crafting_special_mapextending`

**Exact match and assembly semantics:**

Match the fixed shaped map/material pattern, then require a resolvable non-exploration map with
scale below `4`. Preserve map components and set `MAP_POST_PROCESSING=SCALE`; actual saved-map
scaling is downstream of that component.

#### `crafting_special_firework_rocket`

**Exact match and assembly semantics:**

Require one shell, `1..3` fuel stacks and any number of star ingredients, with no others. Create the
result with `FIREWORKS.flight=fuelCount` and explosions copied in row-major star order only when
their component exists.

#### `crafting_special_firework_star`

**Exact match and assembly semantics:**

Require exactly one fuel and at least one dye carrying `DYE`; allow at most one shape item, one
trail and one twinkle. Create `FIREWORK_EXPLOSION` with default small-ball shape, overridden shape,
ordered dye colors, empty fade colors, and the two flags.

#### `crafting_special_firework_star_fade`

**Exact match and assembly semantics:**

Require exactly one target and at least one component-bearing dye. Preserve target components and
replace only the explosion fade-color list with row-major dye colors.

#### `crafting_special_bannerduplicate`

**Exact match and assembly semantics:**

Require exactly two matching banner items: one with `1..6` pattern layers and one with none.
Preserve the patterned banner's components on the result; return the patterned input as a one-count
remainder.

#### `crafting_special_shielddecoration`

**Exact match and assembly semantics:**

Require exactly one `BannerItem` and one target whose pattern list is empty. Preserve target
components and copy banner patterns plus banner base color.

#### `crafting_special_repairitem`

**Exact match and assembly semantics:**

Require exactly two one-count stacks of the same item, both carrying `MAX_DAMAGE` and `DAMAGE`.
Create a default-component stack, set maximum damage to the greater input maximum, and set remaining
durability to both remaining durabilities plus integer `5%` of that maximum, capped at undamaged.
Merge only crafting-visible enchantments, taking the greater level per enchantment. Other input
components are discarded.

#### `smelting`, `blasting`, `smoking`, `campfire_cooking`

**Exact match and assembly semantics:**

A single input ingredient test creates the fixed result template; recipe data supplies cooking time
and experience. Each ID belongs to its same-named type domain and machine rule.

#### `stonecutting`

**Exact match and assembly semantics:**

A single input ingredient test creates the fixed result template in the `stonecutting` domain.
Selection and take behavior belong to the workstation leaf.

#### `smithing_transform`

**Exact match and assembly semantics:**

Test optional template, required base and optional addition in their fixed slots. Preserve the base
component patch on the result template.

#### `smithing_trim`

**Exact match and assembly semantics:**

Test required template/base/addition. Resolve addition `PROVIDES_TRIM_MATERIAL`; return empty if
absent or if the base already has the same material/pattern pair, otherwise return a one-count base
copy with that `TRIM`.

**Branches and aborts:**

Any extra nonempty stack, missing required component, failed ingredient, impossible shapeless
assignment, invalid map data/scale, duplicate singleton modifier, invalid book generation, or
empty/equal smithing-trim result rejects the recipe. Assembly methods defensively return empty when
their required source cannot be found.

**Constants and randomness:**

Shaped dimensions are at most `3x3`; transmute material bounds are clamped by its codec to `1..8`;
map maximum pre-scale value is `3`; repair bonus is integer `(maxDamage * 5) / 100`. No registered
serializer consumes RNG.

**Side effects:**

Assembly returns a new result stack and, for book/banner duplication, an overridden remainder list.
It does not mutate inputs, award recipes or commit map post-processing.

**Gates:**

Decoded data, exact family match, ingredients/tags, named components, level map lookup, enabled
output item and downstream transaction gates.

**Boundary cases and quirks:**

Ingredient matching is item/tag based even when the result copies input components. Dye/firework
color order follows cropped row-major input. Shaped offset is not part of matching after cropping
but is restored for consumption. `crafting_transmute` can intentionally produce the same item when
its count exceeds one. Repair takes the maximum declared durability but begins from default result
components rather than copying either input.

**Evidence:**

`OFF-SERVER-001`, `OFF-DATA-001`, `OFF-REPORT-001`;
`net.minecraft.world.item.crafting.RecipeSerializers#bootstrap(net.minecraft.core.Registry)`,
`net.minecraft.world.item.crafting.ShapedRecipePattern#matches(net.minecraft.world.item.crafting.CraftingInput)`,
`net.minecraft.world.item.crafting.ShapelessRecipe#matches(net.minecraft.world.item.crafting.CraftingInput,net.minecraft.world.level.Level)`,
and `matches`/`assemble`/`getRemainingItems` in `DyeRecipe`, `ImbueRecipe`, `TransmuteRecipe`,
`DecoratedPotRecipe`, `BookCloningRecipe`, `MapExtendingRecipe`, `FireworkRocketRecipe`,
`FireworkStarRecipe`, `FireworkStarFadeRecipe`, `BannerDuplicateRecipe`, `ShieldDecorationRecipe`,
`RepairItemRecipe`, `SingleItemRecipe`, `SmithingRecipe`, `SmithingTransformRecipe` and
`SmithingTrimRecipe`; IDs from `reports/registries.json#minecraft:recipe_serializer`.

**Test vectors:**

Every one of 21 IDs; shaped offset/mirror and explicit empty; shapeless duplicate alternatives;
every component gate; transmute counts `0,1,8,9`; book generation limits and retained source; map
exploration/scale `3/4`; every fireworks singleton duplicate; banner pattern counts `0,1,6,7`;
repair unequal maxima and enchantments; equal versus changed trim.
