# Items mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ITM-BREEZE-ROD-001` — Breeze Rods split one player-gated drop into brewing and three fixed recipes

**Parent:** `BLK-TRIAL-SPAWNER-001`, `PLY-005`, `PLY-006`, `PLY-INPUT-001`,
`PLY-INTERACT-001`, `ITM-001`, `ITM-002`, `ITM-003`, `ITM-004`, `ITM-005`,
`ITM-006`, `ITM-007`, `ITM-USE-001`, `ITM-CONTAINER-001`, `ITM-RECIPE-001`,
`ITM-CRAFT-001`, `ITM-BREW-001`, `ITM-LOOT-001`, `ITM-ADVANCEMENT-001`,
`ITM-ANVIL-001`, `ITM-SMITHING-TEMPLATE-001`, `ITM-POTION-001`, `ENT-001`,
`ENT-005`, `ENT-DEATH-001`, `ENT-ENTITY-DROPS-001`, `ENT-PROJECTILE-001`,
`ENT-EFFECT-001`, `MOB-SPAWN-001`, `WGEN-JIGSAW-TRIAL-CHAMBERS-001`, `CLI-001`,
`CLI-006`, `CLI-UI-001`, `CLI-EFFECT-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — the locked plain-item registration/components, exhaustive code/data
references, Breeze entity table, start-mix brewing registration, three recipes and exact unlock
records, and direct client assets determine every Rod-specific branch. Generic Breeze lifecycle
and death, loot, brewing, crafting, progression, stacks and inventories remain with the cited
owners.

**Applies when:**

A `breeze_rod` stack is created, looted, moved, renamed, persisted, synchronized, offered to a
brewing stand or crafting grid, selected in a tab, rendered or observed before and after loot,
recipe or resource reload.

**Authoritative state:**

`minecraft:breeze_rod` is raw item ID `1252`. It registers through the plain-item path with default
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
branch. Breeze loot, brewing and recipes own the operational joins below.

**Breeze acquisition:**

The only locked baseline source is `entities/breeze`. Its one-roll pool first requires
`killed_by_player`; a death without player kill credit aborts before entry functions and emits no
Rod through this table.

The sole entry creates one Rod, replaces its count with a uniformly drawn integer `1..2`, then
applies enchanted count increase. With a living attacking entity whose Looting level is `L > 0`,
the second function draws a fresh uniform float `V` in `[1,2)` and adds `round(L * V)`. With no
living attacker or `L = 0`, it returns without that draw or bonus. No count limit is configured.
Player credit and the Looting-bearing attacker are separate context inputs, so player credit can
pass while an absent/nonliving attacker supplies only the base count.

The table uses random sequence `minecraft:entities/breeze`. Trial-chamber Breeze admission comes
through configured Trial Spawners under `WGEN-JIGSAW-TRIAL-CHAMBERS-001`,
`BLK-TRIAL-SPAWNER-001` and `MOB-SPAWN-001`; entity death admission, table invocation, large-stack
splitting and world-drop placement remain with `ENT-DEATH-001`, `ENT-ENTITY-DROPS-001` and
`ITM-LOOT-001`.

No bundled chest, fishing, block-drop, trade or other entity table directly emits a Rod.
Administration and custom data can still create ordinary stacks through generic item/loot
boundaries.

**Brewing join:**

The feature-enabled vanilla mix builder registers Rod as a start ingredient. That helper adds two
edges: Water plus Rod becomes Mundane, while Awkward plus Rod becomes Wind Charged. No baseline
Rod edge produces a long or strong variant.

A completed brew transforms every matching bottle slot in owner order, then consumes one
ingredient Rod with no remainder and emits the generic brew event. Rod is not a direct member of
`brewing_fuel`, is not furnace fuel and cannot prepay the stand's fuel uses; a separate valid fuel
source is required to start the transaction.

Slot admission, fuel uses, 400-tick timer/cancellation, bottle transforms, automation and the
unfiltered `nether/brew_potion` player-menu take criterion remain with `ITM-BREW-001` and
`ITM-ADVANCEMENT-001`. The later Wind Charged effect and its killed-entity explosion remain with
`ITM-POTION-001` and `ENT-EFFECT-001`.

**Recipes:**

Rod is an exact ingredient in three locked recipes:

- Shapeless one Rod returns four default Wind Charges.
- Mace uses a centered vertical Heavy Core above Rod in a two-row pattern and returns one default
  Mace.
- Flow Armor Trim Smithing Template duplication uses seven Diamonds around a source Flow template
  above one Rod: shaped `#S#/#C#/###` returns two default Flow templates.

No input component patch is copied and none of these ingredients has a remainder. The Flow recipe
therefore consumes the source template and Rod for a net gain of one default template. Generic
matching, consumption and result transfer remain with `ITM-RECIPE-001` and `ITM-CRAFT-001`;
template semantics remain with `ITM-SMITHING-TEMPLATE-001`, while Wind Charge projectile and later
Mace behavior remain outside this material rule.

**Progression:**

The Wind Charge recipe advancement places Rod possession and exact `wind_charge` recipe-unlocked
criteria in one two-entry OR requirement; either awards only that recipe.

The Mace recipe advancement places Rod possession, Heavy Core possession and exact `mace`
recipe-unlocked criteria in one three-entry OR requirement. Rod alone or Heavy Core alone therefore
unlocks the recipe even though the actual craft requires both.

The Flow duplication advancement does not inspect Rod. It accepts exact Flow-template possession
or prior unlock of the same duplication recipe, then awards that recipe. Rod possession alone
cannot unlock it. A player-opened Brewing Stand can trigger the independent unfiltered brewed-
potion criterion when taking a Rod-produced potion, but automation extraction does not run that
player slot hook.

**Persistence and reload boundary:**

Stacks persist and synchronize identity, count and arbitrary ordinary component patches. They
store no player kill credit, attacker/Looting level, count draws/table cursor, brewing slot/fuel/
timer, potion mix, recipe identity or advancement progress. Those values belong to the death/loot
transaction, machine block entity, server mix/recipe managers and player progression state.

Loot reload can replace the Breeze table for future deaths. A rebuilt baseline mix table retains
the two Rod edges while Wind Charged is feature-enabled; existing bottle/Rod stacks and in-flight
machine state are not retroactively rewritten. Recipe/advancement reload can independently replace
crafting and unlock records. Resource reload independently controls name and model.

**Client and wire projection:**

Generic item-stack encoding projects raw item ID `1252` plus the stack's component patch. Its
common-rarity name uses locked English text `Breeze Rod`; the plain class adds no subtype tooltip
or forced glint.

The direct item definition selects handheld model `minecraft:item/breeze_rod` and its same-named
texture. It appears exactly once and only in Ingredients, ordered Blaze Rod, Breeze Rod, Heavy
Core.

**Branches and aborts:**

Identity/count/components; generic hand/block/container/anvil path; Breeze player credit, attacker,
Looting level, base and bonus draws; brewing fuel/work/bottle/potion state; three recipe patterns,
counts and translations; identity-specific versus alternate unlock criteria; save, loot/recipe/
resource reload, wire, language, model and tab context.

**Constants and randomness:**

Raw item ID `1252`; common rarity; max stack `64`; Breeze base count uniform integer `1..2`;
Looting bonus `round(L * V)` for `V` uniform `[1,2)`; owner brew duration `400`; Rod-to-Wind-Charge
`1 -> 4`; Mace `1 + 1 -> 1`; Flow duplication seven Diamonds plus one source template plus one Rod
to two templates. Only the Breeze loot path consumes Rod-specific RNG.

**Side effects:**

Possible loot stack and named-sequence cursor; generic world drop/pickup; brewing ingredient,
bottles/timer/event; crafting inputs and three results; advancement and recipe known/highlight
state; ordinary stack persistence/wire state; name, direct handheld model and one Ingredients-tab
entry.

**Gates:**

Generic stack/container/anvil admission; valid Breeze death table and player kill credit; living
attacker for Looting; valid brewing fuel, bottle and feature-enabled mix; exact crafting
ingredients/grid; exact inventory or recipe-unlocked criterion; valid registry/stack decode;
client language/model and tab bootstrap.

**State read/written:**

Reads stack identity/count/components, interaction/container state, Breeze death/attacker context,
loot random sequence, brewing slots/fuel/timer/mix table, recipe/advancement registries and player
progression state, persisted stack and client resources. Writes only the loot, brewing, crafting,
progression, stack and client projection listed above.

**Failure behavior:**

Use has no subtype success or mutation. Missing player credit or a suppressed/missing table emits
no Rod. An absent living attacker removes only the Looting bonus. Missing fuel or an unmatched
bottle prevents a brew under the generic owner; Rod is not itself fuel. Invalid or insufficient
crafting leaves inputs unchanged. Rod possession does not unlock Flow duplication. Missing/replaced
loot, recipe, advancement or mix data removes those future paths without rewriting stacks. Client
resource absence follows generic missing translation/model fallback and cannot grant authority.

**Boundary cases and quirks:**

The player-kill gate and Looting attacker are separate. The bonus uses a continuous float followed
by one rounding operation, not an integer `1..2` roll per Looting level. Rod directly brews both a
Mundane dead-end from Water and Wind Charged from Awkward, but cannot fuel the stand. Mace unlock is
an OR despite the recipe requiring both material identities; Flow unlock ignores Rod entirely.

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
`reports/registries.json#minecraft:{item,potion}`;
`reports/minecraft/components/item/breeze_rod.json`;
`data/minecraft/loot_table/entities/breeze.json`;
`data/minecraft/recipe/{wind_charge,mace,flow_armor_trim_smithing_template}.json`;
`data/minecraft/advancement/recipes/{misc/{wind_charge,flow_armor_trim_smithing_template},combat/mace}.json`;
`assets/minecraft/{items,models/item,textures/item}/breeze_rod.*`;
`BLK-TRIAL-SPAWNER-001`; `ITM-BREW-001`; `ITM-LOOT-001`; `ITM-RECIPE-001`;
`ITM-ADVANCEMENT-001`; `ITM-SMITHING-TEMPLATE-001`; `ITM-POTION-001`;
`ENT-DEATH-001`; `ENT-ENTITY-DROPS-001`; `ENT-EFFECT-001`;
`WGEN-JIGSAW-TRIAL-CHAMBERS-001`; `CLI-UI-001`; `CLI-EFFECT-001`;
`EXP-ITM-043`.

**Test vectors:**

Exercise default/patched stacks through hands, blocks, containers and anvil. Evaluate Breeze deaths
with and without player credit across attacker types, Looting levels and every count/float boundary
while tracing the named sequence. Brew Water, Awkward and every other potion with valid/invalid
fuel, bottles and feature state. Match/craft all three recipes and trigger every possession/
recipe-unlocked/brewed-potion criterion before/after data reload. Persist/synchronize stacks and
capture raw ID, name, tooltip, handheld model and exact Ingredients position before/after resource
reload.

**Limits:**

This leaf does not duplicate Trial Spawner or Breeze runtime, generic death/loot emission, brewing
transaction/automation, potion effects, crafting consumption, template/Mace/Wind Charge behavior,
or recipe-book/advancement state. Those remain with their cited owners; this rule fixes the Rod
identity and its exact acquisition, brewing, recipe, progression and presentation joins.
