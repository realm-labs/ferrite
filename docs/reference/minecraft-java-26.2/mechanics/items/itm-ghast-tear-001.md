# Items mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ITM-GHAST-TEAR-001` — Ghast Tears join an ungated mob drop to regeneration brewing and two shaped recipes

**Parent:** `PLY-005`, `PLY-006`, `PLY-INPUT-001`, `PLY-INTERACT-001`,
`ITM-001`, `ITM-002`, `ITM-003`, `ITM-004`, `ITM-005`, `ITM-006`, `ITM-007`,
`ITM-USE-001`, `ITM-CONTAINER-001`, `ITM-RECIPE-001`, `ITM-CRAFT-001`,
`ITM-BREW-001`, `ITM-LOOT-001`, `ITM-ADVANCEMENT-001`, `ITM-ANVIL-001`,
`ITM-POTION-001`, `ITM-END-CRYSTAL-001`, `ENT-001`, `ENT-005`,
`ENT-DEATH-001`, `ENT-ENTITY-DROPS-001`, `MOB-SPAWN-001`, `BLK-SOUL-SAND-001`,
`CLI-001`, `CLI-006`, `CLI-UI-001`, `CLI-EFFECT-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — the locked plain-item registration/components, exhaustive code/data
references, Ghast entity table, start-mix brewing registration, two recipes and exact progression
records, and direct client assets determine every Tear-specific branch. Generic Ghast lifecycle
and death, loot, brewing, crafting, progression, stacks and inventories remain with the cited
owners.

**Applies when:**

A `ghast_tear` stack is created, looted, moved, renamed, persisted, synchronized, offered to a
brewing stand or crafting grid, selected in a tab, rendered or observed before and after loot,
recipe or resource reload.

**Authoritative state:**

`minecraft:ghast_tear` is raw item ID `1146`. It registers through the plain-item path with default
properties, is common, nondamageable and has max stack `64`. It belongs to no direct item tag.

Its registered components are only the common empty modifiers/enchantments/lore, item-break sound,
translated name, direct item-model key, repair cost, swing animation, tooltip display and use
effects. It has no food, consumable, cooldown, remainder, tool, equipment, repairable or
identity-specific glint state.

**Transition and ordering:**

The identity does not override hand use or block use. A prototype stack's air use returns generic
`PASS`; a block click participates only in ordinary block-first interaction and fallback handling.
A component-patched stack can activate a generic component owner, but the identity itself never
consumes a stack, starts active use, emits a sound/game event/particle, increments item use or
changes the world.

Container movement, pickup, dropping, anvil naming and component patching use their generic owners.
The identity adds no dispenser, mob-interaction, equipment, repair, fuel, composting or villager
branch. Ghast loot, brewing and recipes own the operational joins below.

**Ghast acquisition:**

The only locked baseline source is the first pool of `entities/ghast`. Its one roll has no
condition: every admitted table evaluation creates one Tear, replaces its count with a uniformly
drawn integer `0..1`, then applies enchanted count increase.

With a living attacking entity whose Looting level is `L > 0`, the second function draws a fresh
uniform float `U` in `[0,1)` and adds `round(L * U)`. With no living attacker or `L = 0`, it returns
without that draw or bonus. No count limit is configured, and a zero base count can be revived by
a positive Looting bonus before generic empty-stack filtering.

The table uses random sequence `minecraft:entities/ghast`. It contains no player-kill gate for the
Tear pool, so an environmentally killed Ghast can still run the base path when generic death/loot
admission is open. The later Gunpowder and fireball/player-gated Music Disc pools are independent
and do not change the Tear count, though they advance the same named sequence through their own
branches.

Ghast spawning remains with `MOB-SPAWN-001`; entity death admission, attacker context, table
invocation, empty-stack filtering and world-drop placement remain with `ENT-DEATH-001`,
`ENT-ENTITY-DROPS-001` and `ITM-LOOT-001`.

No bundled chest, fishing, block-drop, trade or other entity table directly emits a Tear.
Administration and custom data can still create ordinary stacks through generic item/loot
boundaries.

**Brewing join:**

The feature-enabled vanilla mix builder registers Tear as a start ingredient. That helper adds two
edges: Water plus Tear becomes Mundane, while Awkward plus Tear becomes Regeneration. Redstone and
Glowstone Dust, not another Tear, own the later Regeneration-to-Long-Regeneration and
Regeneration-to-Strong-Regeneration edges.

A completed brew transforms every matching bottle slot in owner order, then consumes one
ingredient Tear with no remainder and emits the generic brew event. Tear is not a direct member of
`brewing_fuel`, is not furnace fuel and cannot prepay the stand's fuel uses; a separate valid fuel
source is required.

Slot admission, fuel uses, 400-tick timer/cancellation, bottle transforms, automation and the
unfiltered `nether/brew_potion` player-menu take criterion remain with `ITM-BREW-001` and
`ITM-ADVANCEMENT-001`. Regeneration effect ticks and later potion container behavior remain with
`ITM-POTION-001` and the generic effect owner.

**Recipes:**

Tear is an exact ingredient in two locked shaped recipes:

- Dried Ghast uses eight Tears surrounding one Soul Sand in a full three-by-three grid and returns
  one default Dried Ghast.
- End Crystal uses `GGG/GEG/GTG`: seven Glass, one Ender Eye and one Tear return one default End
  Crystal.

No input component patch is copied and no ingredient has a remainder. Generic matching,
consumption and result transfer remain with `ITM-RECIPE-001` and `ITM-CRAFT-001`; Dried Ghast
hydration/growth joins remain with `BLK-SOUL-SAND-001`, while End Crystal placement, entity and
fight behavior remain with `ITM-END-CRYSTAL-001`.

**Progression:**

The Dried Ghast recipe advancement places Tear possession and exact `dried_ghast`
recipe-unlocked criteria in one two-entry OR requirement; either awards only that recipe.

The End Crystal recipe advancement instead accepts Ender Eye possession or exact `end_crystal`
recipe unlock. Tear possession alone cannot unlock it even though the recipe consumes one.

The separate `nether/uneasy_alliance` challenge uses a Tear only as its display icon. Its sole
criterion requires a player-killed Ghast whose location is in the Overworld, rewards 100
experience and sends its configured telemetry event. It neither checks for nor rewards a Tear.
A player-opened Brewing Stand can independently trigger the unfiltered brewed-potion criterion
when taking a Tear-produced potion; automation extraction does not run that player slot hook.

**Persistence and reload boundary:**

Stacks persist and synchronize identity, count and arbitrary ordinary component patches. They
store no death/attacker context, Looting level, count draws/table cursor, brewing slot/fuel/timer,
potion mix, recipe identity or advancement progress. Those values belong to the death/loot
transaction, machine block entity, server mix/recipe managers and player progression state.

Loot reload can replace the Ghast table for future deaths. A rebuilt baseline mix table retains
the two Tear edges while Regeneration is feature-enabled; existing stacks and in-flight machine
state are not retroactively rewritten. Recipe/advancement reload can independently replace
crafting, unlock and icon records. Resource reload independently controls name and model.

**Client and wire projection:**

Generic item-stack encoding projects raw item ID `1146` plus the stack's component patch. Its
common-rarity name uses locked English text `Ghast Tear`; the plain class adds no subtype tooltip
or forced glint.

The direct item definition selects generated model `minecraft:item/ghast_tear` and its same-named
texture. It appears exactly once and only in Ingredients, ordered Golden Carrot, Ghast Tear, Turtle
Helmet.

**Branches and aborts:**

Identity/count/components; generic hand/block/container/anvil path; admitted Ghast death, attacker,
Looting level, base/bonus draws and later table pools; brewing fuel/work/bottle/potion state; two
recipe patterns/counts; Tear versus alternate unlock and icon-only challenge criteria; save,
loot/recipe/resource reload, wire, language, model and tab context.

**Constants and randomness:**

Raw item ID `1146`; common rarity; max stack `64`; Ghast base count uniform integer `0..1`;
Looting bonus `round(L * U)` for `U` uniform `[0,1)`; owner brew duration `400`; Dried Ghast eight
Tears plus one Soul Sand to one block; End Crystal seven Glass plus one Ender Eye plus one Tear to
one Crystal; Uneasy Alliance reward `100` experience. Only the Ghast loot path consumes
Tear-specific RNG.

**Side effects:**

Possible loot stack and named-sequence cursor; generic world drop/pickup; brewing ingredient,
bottles/timer/event; crafting inputs and two results; advancement and recipe known/highlight/
experience/telemetry state; ordinary stack persistence/wire state; name, direct model and one
Ingredients-tab entry.

**Gates:**

Generic stack/container/anvil admission; valid Ghast death table; living attacker for Looting;
valid brewing fuel, bottle and feature-enabled mix; exact crafting ingredients/grid; exact
inventory, recipe-unlocked or player-killed-entity criterion; valid registry/stack decode; client
language/model and tab bootstrap.

**State read/written:**

Reads stack identity/count/components, interaction/container state, Ghast death/attacker context,
loot random sequence, brewing slots/fuel/timer/mix table, recipe/advancement registries and player
progression state, persisted stack and client resources. Writes only the loot, brewing, crafting,
progression, stack and client projection listed above.

**Failure behavior:**

Use has no subtype success or mutation. A suppressed/missing table or zero final count emits no
Tear. An absent living attacker removes only the Looting bonus. Missing fuel or an unmatched bottle
prevents a brew under the generic owner; Tear is not itself fuel. Invalid or insufficient crafting
leaves inputs unchanged. Tear possession does not unlock End Crystal or complete Uneasy Alliance.
Missing/replaced loot, recipe, advancement or mix data removes those future paths without
rewriting stacks. Client resource absence follows generic missing translation/model fallback and
cannot grant authority.

**Boundary cases and quirks:**

The Tear pool has no player-kill condition even though its Looting bonus still depends on a living
attacker. A zero base can become positive through the later bonus. Tear directly brews both a
Mundane dead-end from Water and Regeneration from Awkward but cannot fuel the stand. The End Crystal
recipe consumes Tear without using it as the possession unlock, and Uneasy Alliance's icon is
presentation only.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-DATA-001`; `OFF-REPORT-001`;
`net.minecraft.world.item.Items`;
`net.minecraft.world.item.CreativeModeTabs`;
`net.minecraft.world.level.storage.loot.functions.EnchantedCountIncreaseFunction#run`;
`net.minecraft.world.level.storage.loot.providers.number.UniformGenerator`;
`net.minecraft.world.item.alchemy.PotionBrewing`;
`net.minecraft.world.item.alchemy.PotionBrewing$Builder#addStartMix`;
`net.minecraft.data.loot.packs.VanillaEntityLoot`;
`net.minecraft.data.recipes.packs.VanillaRecipeProvider`;
`net.minecraft.data.advancements.packs.VanillaNetherAdvancements`;
`reports/registries.json#minecraft:{item,potion}`;
`reports/minecraft/components/item/ghast_tear.json`;
`data/minecraft/loot_table/entities/ghast.json`;
`data/minecraft/recipe/{dried_ghast,end_crystal}.json`;
`data/minecraft/advancement/{recipes/{building_blocks/dried_ghast,decorations/end_crystal},nether/uneasy_alliance}.json`;
`assets/minecraft/{items,models/item,textures/item}/ghast_tear.*`;
`ITM-BREW-001`; `ITM-LOOT-001`; `ITM-RECIPE-001`; `ITM-ADVANCEMENT-001`;
`ITM-POTION-001`; `ITM-END-CRYSTAL-001`; `ENT-DEATH-001`;
`ENT-ENTITY-DROPS-001`; `BLK-SOUL-SAND-001`; `CLI-UI-001`; `CLI-EFFECT-001`;
`EXP-ITM-044`.

**Test vectors:**

Exercise default/patched stacks through hands, blocks, containers and anvil. Evaluate Ghast deaths
across generic loot admission, attacker types, Looting levels, every count/float boundary and all
later pool branches while tracing the named sequence. Brew Water, Awkward and every other potion
with valid/invalid fuel, bottles and feature state. Match/craft both recipes and trigger every
possession/recipe-unlocked/brewed-potion/challenge criterion before/after data reload.
Persist/synchronize stacks and capture raw ID, name, tooltip, model and exact Ingredients position
before/after resource reload.

**Limits:**

This leaf does not duplicate Ghast spawning/AI/death, generic loot emission, brewing transaction/
automation, potion effects, crafting consumption, Dried Ghast or End Crystal runtime, or
recipe-book/advancement state. Those remain with their cited owners; this rule fixes the Tear
identity and its exact acquisition, brewing, recipe, progression and presentation joins.
