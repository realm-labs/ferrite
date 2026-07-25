# Items mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ITM-BLAZE-MATERIAL-001` — Blaze materials split one player-gated drop into furnace, brewing and recipe consumers

**Parent:** `BLK-SPAWNER-001`, `PLY-005`, `PLY-006`, `PLY-INPUT-001`,
`PLY-INTERACT-001`, `ITM-001`, `ITM-002`, `ITM-003`, `ITM-004`, `ITM-005`,
`ITM-006`, `ITM-007`, `ITM-USE-001`, `ITM-CONTAINER-001`, `ITM-RECIPE-001`,
`ITM-CRAFT-001`, `ITM-FURNACE-001`, `ITM-BREW-001`, `ITM-LOOT-001`,
`ITM-ADVANCEMENT-001`, `ITM-ANVIL-001`, `ENT-001`, `ENT-005`, `ENT-DEATH-001`,
`ENT-ENTITY-DROPS-001`, `MOB-SPAWN-001`, `WGEN-STRUCTURE-FORTRESS-001`, `CLI-001`,
`CLI-006`, `CLI-UI-001`, `CLI-EFFECT-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked plain-item registrations/components, exhaustive code/data references,
the Blaze entity table, fuel-table and brewing joins, fourteen recipes and their progression
records, and direct client assets determine every identity-specific branch. Generic entity death,
loot, furnace, brewing, crafting, advancement, stack and inventory behavior remains with the cited
owners.

**Applies when:**

A `blaze_rod` or `blaze_powder` stack is created, looted, moved, renamed, persisted, synchronized,
offered to a furnace or brewing stand, offered to crafting, selected in a tab, rendered or observed
before and after tag, loot, recipe or resource reload.

**Authoritative state:**

`minecraft:blaze_rod` is raw item ID `1145`; `minecraft:blaze_powder` is raw item ID `1153`. Both
register through the plain-item path with default properties. Each is common, nondamageable and max
stack `64`.

Rod belongs to no direct item tag. Powder's sole direct tag is `minecraft:brewing_fuel`, whose
locked baseline value list contains only `minecraft:blaze_powder`. Their registered components are
only the common empty modifiers/enchantments/lore, item-break sound, translated name, direct
item-model key, repair cost, swing animation, tooltip display and use effects. Neither identity
has food, consumable, cooldown, remainder, tool, equipment or repairable state.

**Transition and ordering:**

Neither identity overrides hand use or block use. A prototype stack's air use returns generic
`PASS`; a block click participates only in ordinary block-first interaction and fallback handling.
A component-patched stack can activate a generic component owner, but no identity-specific hand or
block branch consumes a stack, starts active use, emits a sound/game event/particle, increments
item use or changes the world.

Container movement, pickup, dropping, anvil naming and component patching use their generic owners.
The identities add no dispenser, mob-interaction, equipment, repair, enchantment or villager
branch. Fuel, brewing, loot and recipe consumers own the operational joins below.

**Blaze acquisition and conversion:**

The only locked baseline loot source is `entities/blaze`. Its one-roll pool first requires
`killed_by_player`; a death without player kill credit emits no rod through this table. The sole
entry creates one rod, replaces its count with a uniformly drawn integer `0..1`, then applies
enchanted count increase.

With a living attacking entity whose Looting level is `L > 0`, the second function adds
`round(L * U)` for a fresh uniform float `U` in `[0,1)`. With no living attacker or `L = 0`, it
returns without that draw or bonus. There is no configured count limit, and a zero base count can
be revived by a positive Looting bonus before generic empty-stack filtering. The table uses random
sequence `minecraft:entities/blaze`.

Fortress spawn overrides and throne spawners provide baseline Blaze routes under
`WGEN-STRUCTURE-FORTRESS-001`, `BLK-SPAWNER-001` and `MOB-SPAWN-001`; entity death admission,
player-credit context, loot invocation and world drop placement remain with `ENT-DEATH-001`,
`ENT-ENTITY-DROPS-001` and `ITM-LOOT-001`.

Powder has no baseline loot, trade, block-drop or mob-drop source. Its sole standard creation path
is the shapeless `blaze_powder` recipe: one rod returns two default powder items. There is no
reverse recipe. Administration and custom data can still create ordinary stacks through generic
item and loot boundaries.

**Furnace and brewing joins:**

The vanilla furnace fuel table directly maps one rod to `12 * 200 = 2400` burn ticks. Ignition
consumes one rod with no remainder under `ITM-FURNACE-001`; powder has no furnace-fuel mapping.
At default recipe durations, the rod's timer can cover twelve 200-tick cooks or twenty-four
100-tick cooks when all generic work/output gates remain valid.

Powder's `brewing_fuel` membership admits it to brewing-stand fuel slot `4`. On a server tick with
fuel uses at zero, the stand checks that tag before brewability, sets fuel uses to `20`, consumes
one powder and marks the stand changed—even with no valid ingredient or bottle. Each later brew
start spends one use and starts the owner-defined 400-tick transaction; the item stack stores
neither fuel uses nor brew time.

Powder is independently a brewing ingredient in slot `3`. The baseline feature-enabled mix
builder adds both Water plus powder to Mundane and Awkward plus powder to Strength. A completed
brew transforms every matching bottle slot in owner order, then consumes one ingredient powder
with no remainder and emits the generic brew event. Redstone and Glowstone Dust, not additional
powder, own the later Strength-to-Long-Strength and Strength-to-Strong-Strength edges.

The same physical powder stack cannot occupy fuel and ingredient slots simultaneously. Two powder
stacks can serve both roles, and a fuel refill may consume its stack on the same tick a valid brew
spends the first newly installed fuel use. Slot admission, timer/cancellation, bottle transforms,
automation and the `nether/brew_potion` criterion remain with `ITM-BREW-001` and
`ITM-ADVANCEMENT-001`.

**Recipes:**

Rod is an exact ingredient in eleven locked recipes:

- Shapeless one rod returns two powder.
- Brewing Stand uses shaped ` B /###`: one rod above three members of
  `minecraft:stone_crafting_materials`, returning one stand.
- End Rod uses shaped `/#`: one rod above one popped chorus fruit, returning four End Rods.
- Eight Copper Bulb recipes use shaped ` C /CBC/ R `: one rod at center, three matching copper
  blocks and one redstone return four matching bulbs. The eight exact input/output ages are
  copper, exposed, weathered and oxidized, each unwaxed or waxed.

Powder is an exact ingredient in three further shapeless recipes:

- one Ender Pearl plus one powder returns one Ender Eye;
- one Gunpowder, one powder and either Coal or Charcoal return three Fire Charges;
- one powder plus one Slime Ball returns one Magma Cream.

All results are fixed default stacks, no input component patch is copied, and no ingredient has a
remainder in these joins. Generic shaped/shapeless matching, consumption and result transfer remain
with `ITM-RECIPE-001` and `ITM-CRAFT-001`; the crafted outputs' later behavior remains with their
own owners.

**Progression:**

Possessing a rod independently satisfies three inventory criteria: the sole criterion of
`nether/obtain_blaze_rod`, and the possession side of the one-OR-requirement Blaze Powder and
Brewing Stand recipe advancements. The Nether advancement uses the rod as its display icon and
sends its configured telemetry event under the generic advancement owner.

End Rod unlock instead checks Popped Chorus Fruit, and each Copper Bulb unlock checks its exact
matching copper block; rod possession alone does not unlock those nine recipes. Possessing powder
satisfies the possession side of the one-OR-requirement Ender Eye, Fire Charge and Magma Cream
recipe advancements. Every recipe advancement's other alternative is prior unlock of that exact
recipe, and every reward unlocks only its corresponding recipe.

`nether/brew_potion` has an unfiltered brewed-potion criterion and `nether/obtain_blaze_rod` as its
parent. Taking a powder-produced Mundane or Strength potion from a player-opened Brewing Stand
menu can trigger it, but the criterion neither checks powder nor requires those potion results.
Automation extraction does not run that player-slot take hook.

**Persistence and reload boundary:**

Stacks persist and synchronize identity, count and arbitrary ordinary component patches. They
store no killer credit, Looting draw, loot cursor, furnace burn duration, brewing slot/uses/timer,
potion mix, recipe identity or advancement progress. Those values belong to the death transaction,
machine block entity, server mix table, recipe manager and player advancement/recipe state.

Loot reload can replace the Blaze table for future deaths. Tag reload can remove powder from or add
other identities to `brewing_fuel` without changing the baseline potion ingredient edges or
existing stand fuel uses. Recipe/advancement reload can independently replace crafting and unlock
records without rewriting stacks. A rebuilt baseline fuel/mix table retains the locked direct rod
duration and powder mix edges while their items are feature-enabled. Resource reload independently
controls names and models.

**Client and wire projection:**

Generic item-stack encoding projects raw item IDs `1145` and `1153` plus each stack's component
patch. Their common-rarity names use locked English text `Blaze Rod` and `Blaze Powder`; neither
plain class adds a subtype tooltip.

Rod's direct item definition selects the handheld model `minecraft:item/blaze_rod`; powder selects
the generated model `minecraft:item/blaze_powder`. Each uses its same-named texture. Each appears
exactly once and only in Ingredients: Blaze Rod is ordered Fire Charge, Blaze Rod, Breeze Rod;
Blaze Powder is ordered Fermented Spider Eye, Blaze Powder, Sugar.

**Branches and aborts:**

Identity/count/components; hand/block/container/anvil path; player kill credit, attacker and
Looting level; base `0..1` and bonus draw; furnace work/output/fuel admission; powder tag and
fuel-versus-ingredient slot; Water/Awkward/other potion; fourteen recipe grids, variants and
counts; identity-specific versus alternate unlock criterion; save, tag/loot/recipe reload, wire,
language, model and tab context.

**Constants and randomness:**

Raw IDs rod/powder `1145/1153`; common rarity; max stack `64`; Blaze base count `0..1`; Looting
bonus `round(L * U[0,1))`; rod fuel `2400` ticks; powder fuel batch `20`; owner brew duration `400`;
rod-to-powder `1 -> 2`; Brewing Stand `1 + 3 -> 1`; End Rod `1 + 1 -> 4`; each bulb
`1 + 3 + 1 -> 4`; Ender Eye `1 + 1 -> 1`; Fire Charge `1 + 1 + 1 -> 3`; Magma Cream
`1 + 1 -> 1`. Only the Blaze loot path consumes identity-specific RNG.

**Side effects:**

Blaze loot stacks, world drops and random-sequence cursor under generic owners; furnace fuel
stack/timers; brewing fuel/ingredient stacks, uses/timer/bottles/event; crafting inputs/results;
advancement and recipe known/highlight state; ordinary stack persistence/wire state; names, direct
models and two Ingredients-tab entries.

**Gates:**

Generic stack/container/anvil admission; valid Blaze death table and player kill credit; living
attacker for Looting; furnace recipe/output and fuel table; brewing-fuel tag, slot, brewable bottle
and enabled potion mix; exact crafting ingredients/grid; exact inventory or recipe-unlocked
criterion; valid registry/stack decode; client language/model and tab bootstrap.

**State read/written:**

Reads stack identity/count/components, interaction/container state, Blaze death/attacker context,
furnace slots/timers/fuel table, brewing slots/tag/uses/timer/mix table, recipe/advancement
registries and player progression state, persisted stack and client resources. Writes only the
loot, furnace, brewing, crafting, progression, stack and client projection listed above.

**Failure behavior:**

Use has no subtype success or mutation. Missing player kill credit, a zero final count or failed
death admission emits no rod. Invalid furnace work prevents rod ignition under the generic owner;
powder is not furnace fuel. A nonmember fuel tag rejects powder from the fuel path, while an
unmatched potion rejects its ingredient path. Invalid/insufficient crafting leaves inputs
unchanged. Missing/replaced loot, tag, recipe or advancement data removes those future paths
without rewriting stacks. Client resource absence follows generic missing translation/model
fallback and cannot grant authority.

**Boundary cases and quirks:**

The player-kill condition and Looting attacker are separate gates: player credit can pass while no
living attacking entity supplies a bonus. A zero base rod count can become positive through the
following Looting function. Rod is furnace fuel but not brewing fuel; powder is brewing fuel and
ingredient but not furnace fuel. Brewing fuel may be prepaid and wasted before brewability.
Possessing rod does not unlock End Rod or bulb recipes even though each consumes rod.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-DATA-001`; `OFF-REPORT-001`;
`net.minecraft.world.item.Items`;
`net.minecraft.world.item.CreativeModeTabs`;
`net.minecraft.world.level.storage.loot.functions.EnchantedCountIncreaseFunction`;
`net.minecraft.world.level.storage.loot.providers.number.UniformGenerator`;
`net.minecraft.world.level.block.entity.FuelValues`;
`net.minecraft.world.level.block.entity.BrewingStandBlockEntity`;
`net.minecraft.world.item.alchemy.PotionBrewing`;
`net.minecraft.data.loot.packs.VanillaEntityLoot`;
`net.minecraft.data.tags.VanillaItemTagsProvider`;
`net.minecraft.data.recipes.packs.VanillaRecipeProvider`;
`net.minecraft.data.advancements.packs.VanillaNetherAdvancements`;
`reports/registries.json#minecraft:{item,potion}`;
`reports/minecraft/components/item/{blaze_rod,blaze_powder}.json`;
`data/minecraft/loot_table/entities/blaze.json`;
`data/minecraft/tags/item/brewing_fuel.json`;
`data/minecraft/recipe/{blaze_powder,brewing_stand,end_rod,ender_eye,fire_charge,magma_cream,*copper_bulb}.json`;
`data/minecraft/advancement/nether/{obtain_blaze_rod,brew_potion}.json`;
`data/minecraft/advancement/recipes/{brewing/{blaze_powder,brewing_stand,magma_cream},decorations/end_rod,misc/{ender_eye,fire_charge},redstone/*copper_bulb}.json`;
`assets/minecraft/{items,models/item,textures/item}/{blaze_rod,blaze_powder}.*`;
`ITM-FURNACE-001`; `ITM-BREW-001`; `ITM-LOOT-001`; `ITM-RECIPE-001`;
`ITM-ADVANCEMENT-001`; `ENT-DEATH-001`; `WGEN-STRUCTURE-FORTRESS-001`;
`CLI-UI-001`; `CLI-EFFECT-001`; `EXP-ITM-039`.

**Test vectors:**

Exercise default/patched stacks through hands, blocks, containers and anvil. Evaluate Blaze deaths
with and without player credit across attacker types, Looting levels and every count draw while
recording the exact sequence cursor. Offer rod/powder to every furnace and brewing slot at timer,
tag, bottle and potion boundaries, including simultaneous fuel/ingredient stacks. Match/craft all
fourteen recipes at grid/count/variant boundaries and trigger every inventory/recipe/brew criterion
before/after data reload. Persist/synchronize stacks and capture raw IDs, tags, names, tooltips,
models and exact Ingredients order before/after resource reload.

**Limits:**

This leaf does not duplicate Blaze spawning/AI/death, generic loot emission, furnace tick/cooking,
brewing transaction/automation, crafting consumption, recipe-book/advancement state or the later
behavior of crafted blocks/items and potions. Those remain with their cited owners; this rule fixes
the two material identities and their exact acquisition, fuel, ingredient, recipe and presentation
joins.
