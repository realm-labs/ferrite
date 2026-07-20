# Items mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ITM-RECIPE-001` — Recipe reload produces ordered, type-partitioned lookup domains

**Parent:** `ITM-004`, `ITM-005`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — reload ordering, the seven type domains, preferred-recipe fast path, first-match
fallback and cache invalidation are explicit in locked source.

**Applies when:**

A data pack reloads recipes, a crafting grid or processor requests a match, or the crafter consults
its recipe cache.

**Authoritative state:**

The active `RecipeManager`, sorted recipe keys, recipe type, decoded serializer instance, input
view, level, optional previously selected recipe and the crafter's ten-entry input cache.

**Transition and ordering:**

Reload decodes `data/<namespace>/recipe/**/*.json` into a key-sorted map, iterates that map in key
order, and appends each holder to its recipe-type collection in that order. Lookup first tests an
explicitly supplied previous holder when it belongs to the requested type; if it still matches, it
wins. Otherwise it scans that type's ordered collection and returns the first match. The seven
disjoint domains are `crafting`, `smelting`, `blasting`, `smoking`, `campfire_cooking`,
`stonecutting` and `smithing`. The crafter caches the first-match result, including no match, by
cropped input width/height plus one-count copies of every stack; a different `RecipeManager`
identity clears all entries.

**Branches and aborts:**

Decode failures omit the invalid resource and are logged during reload. Empty crafter input returns
no recipe without inserting an entry. A previous holder that no longer matches falls through to the
ordinary scan. Feature-disabled recipe ingredients are excluded from synchronized property/display
projections, while authoritative matching still invokes the decoded recipe and output feature checks
at its owning transaction.

**Constants and randomness:**

Lookup and matching consume no RNG. Key order is natural `ResourceKey` order. Crafter cache capacity
is `10`; a hit moves to the front and insertion evicts the least-recent entry.

**Side effects:**

Recipe-manager replacement, cache invalidation, selected recipe holder, recipe-book/display
projections and downstream result recomputation. Lookup itself does not consume inputs or produce an
item.

**Gates:**

Successful codec decode, requested type identity, recipe `matches`, optional prior holder type,
enabled ingredient/output features at the calling boundary and input equality for cache hits.

**Boundary cases and quirks:**

Overlapping recipes are resolved by retained previous match before key order, so changing from one
still-matching recipe to a lexically earlier overlapping recipe does not switch the displayed
result. Data-pack key order is therefore observable only when no retained holder wins. The crafter
cache stores full stack equality after forcing count to one, so count changes hit the same entry but
component changes do not.

**Evidence:**

`OFF-SERVER-001`, `OFF-DATA-001`;
`net.minecraft.world.item.crafting.RecipeManager#prepare(net.minecraft.server.packs.resources.ResourceManager,net.minecraft.util.profiling.ProfilerFiller)`,
`net.minecraft.world.item.crafting.RecipeMap#create(java.lang.Iterable)`,
`net.minecraft.world.item.crafting.RecipeManager#getRecipeFor(net.minecraft.world.item.crafting.RecipeType,net.minecraft.world.item.crafting.RecipeInput,net.minecraft.world.level.Level,net.minecraft.world.item.crafting.RecipeHolder)`,
`net.minecraft.world.item.crafting.RecipeMap#getRecipesFor(net.minecraft.world.item.crafting.RecipeType,net.minecraft.world.item.crafting.RecipeInput,net.minecraft.world.level.Level)`,
`net.minecraft.world.item.crafting.RecipeCache#get(net.minecraft.server.level.ServerLevel,net.minecraft.world.item.crafting.CraftingInput)`;
`EXP-ITM-003` is a regression probe, not needed to resolve lookup order.

**Test vectors:**

Two overlapping keys with and without a retained holder; one recipe in each of seven domains;
malformed resource; reload to a different manager; crafter count-only versus component input
changes; cache a no-match then reload a matching recipe.
