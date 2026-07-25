# Items mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ITM-MAGMA-CREAM-001` — Magma Cream joins cube and Bastion loot to crafting and Fire Resistance brewing

**Parent:** `PLY-005`, `PLY-006`, `PLY-INPUT-001`, `PLY-INTERACT-001`,
`ITM-001`, `ITM-002`, `ITM-003`, `ITM-004`, `ITM-005`, `ITM-006`, `ITM-007`,
`ITM-USE-001`, `ITM-CONTAINER-001`, `ITM-RECIPE-001`, `ITM-CRAFT-001`,
`ITM-BREW-001`, `ITM-LOOT-001`, `ITM-ADVANCEMENT-001`, `ITM-ANVIL-001`,
`ITM-POTION-001`, `ENT-001`, `ENT-005`, `ENT-DEATH-001`,
`ENT-ENTITY-DROPS-001`, `ENT-EFFECT-001`, `MOB-SPAWN-001`, `MOB-AI-001`,
`BLK-MAGMA-001`, `WGEN-JIGSAW-BASTION-001`, `CLI-001`, `CLI-006`,
`CLI-UI-001`, `CLI-EFFECT-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — the locked plain-item registration/components, exhaustive code/data
references, Magma Cube and two Bastion chest tables, ordered count functions, start-mix brewing
registration, two recipes and unlock records, and direct client assets determine every
Cream-specific branch. Generic mob lifecycle/death/AI, damage-source construction, loot filling,
Bastion placement, brewing, crafting, progression, effects, stacks and inventories remain with
the cited owners.

**Applies when:**

A `magma_cream` stack is created, looted, moved, renamed, persisted, synchronized, offered to a
brewing stand or crafting grid, selected in a tab, rendered or observed before and after loot,
recipe, mix or resource reload.

**Authoritative state:**

`minecraft:magma_cream` is raw item ID `1154`. It registers through the plain-item path with
default properties, is common, nondamageable and has max stack `64`. It belongs to no direct item
tag.

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
branch. Magma Cube loot, two chest pools, crafting and brewing own the operational joins below.

**Magma Cube acquisition and Frog alternative:**

The sole pool of `entities/magma_cube` has one roll. Its first entry can emit Cream only when both
of these ordered conditions pass:

- the damage source's `source_entity` is not entity type `minecraft:frog`;
- the dying cube mob's reported size is at least `2`.

The table has no player-kill condition. A non-Frog source therefore admits the Cream entry for
size `2+` even without player attribution, while size below `2` admits no Cream and spends no
Cream count draw.

An admitted entry creates a Cream and replaces its count with one uniformly drawn integer
`B` in `-2..1`; all four integers are possible. It then applies enchanted count increase. With no
living attacking entity or Looting level `L = 0`, that function returns without a bonus draw, so
only `B = 1` survives generic empty-stack filtering and the baseline emission probability is
`1/4`.

With a living attacking entity and `L > 0`, enchanted count increase draws a fresh uniform float
`U` in `[0,1)`, calculates `R = round(L * U)`, and calls `ItemStack.grow(R)`. Ordering matters:
`grow` reads the existing count through `getCount`, which reports `0` whenever the stack's stored
count is nonpositive. The resulting count is therefore

`max(B, 0) + round(L * U)`,

not `B + round(L * U)`. Any positive bonus revives each of the `-2`, `-1` and `0` base branches
equally; the negative values do not subtract from that bonus. No count limit is configured.

The remaining three entries are mutually exclusive Frog alternatives. When the damage source's
`source_entity` is a Frog, the inverted Cream condition fails and its variant selects exactly one
default count-one output: warm maps to Pearlescent Froglight, cold to Verdant Froglight and
temperate to Ochre Froglight. Those entries do not test cube size or run Cream count/Looting
functions. A non-Frog source cannot select a froglight. Froglight block/item behavior and Frog
attack construction remain outside this item leaf.

The table uses random sequence `minecraft:entities/magma_cube`. Magma Cube spawning and Frog AI
remain with the mob owners; death admission, damage-source context, table invocation,
empty-stack filtering and world-drop placement remain with `ENT-DEATH-001`,
`ENT-ENTITY-DROPS-001` and `ITM-LOOT-001`.

**Bastion chest acquisition:**

Cream is a weighted entry in exactly two locked chest tables:

- `chests/bastion_other` pool `2` draws a uniform integer `3..4` rolls. Its entries have total
  weight `13`; Cream has weight `2`, so each roll selects it with probability `2/13` and then
  draws an integer count `2..6`.
- `chests/bastion_treasure` pool `1` likewise draws `3..4` rolls. Its nine entries all have
  weight `1`; each roll therefore selects Cream with probability `1/9` and then draws an integer
  count `3..8`.

Selection is with the generic per-roll loot procedure, so a pool can select Cream more than once.
In `bastion_other`, this pool follows the one-roll valuables and two-roll equipment pools and
precedes the two template pools. In `bastion_treasure`, it follows the three-roll valuables pool
and precedes its two template pools. Their random sequences are respectively
`minecraft:chests/bastion_other` and `minecraft:chests/bastion_treasure`; every preceding pool
therefore advances the same table sequence before the Cream pool.

Exact Bastion template markers and table assignment remain with `WGEN-JIGSAW-BASTION-001`.
Container placement, roll selection, stack splitting/shuffling and insertion remain with
`ITM-LOOT-001`.

No other locked chest, fishing, block-drop, gift, trade or entity table directly emits Cream.
Administration and custom data can still create ordinary stacks through generic item/loot
boundaries.

**Crafting joins:**

Two locked recipes connect the item:

- `magma_cream` is shapeless: one Blaze Powder plus one Slime Ball returns one default Cream.
- `magma_block` is shaped `##/##`, with every `#` exactly Cream, and returns one default Magma
  Block. The two-by-two square can translate within a larger crafting grid.

No input component patch is copied and neither ingredient has a remainder. Generic matching,
consumption and result transfer remain with `ITM-RECIPE-001` and `ITM-CRAFT-001`; Magma Block
state, placement, self loot, environment and world-generation behavior remain with
`BLK-MAGMA-001`.

**Progression:**

The Cream recipe advancement places Blaze Powder possession and exact `magma_cream`
recipe-unlocked criteria in one two-entry OR requirement; either awards only the Cream recipe.
Slime Ball or Cream possession alone cannot unlock it.

The Magma Block recipe advancement instead places Cream possession and exact `magma_block`
recipe-unlocked criteria in one OR requirement; either awards only the Magma Block recipe.

Taking a completed Cream-produced potion from a player-opened Brewing Stand can independently
trigger the generic unfiltered brewed-potion criterion; automation extraction does not run that
player slot hook. Cream itself is not a criterion in the potion/effect challenges.

**Brewing join:**

The feature-enabled vanilla mix builder registers Cream as a start ingredient. The helper adds two
edges: Water plus Cream becomes Mundane, while Awkward plus Cream becomes Fire Resistance with one
amplifier-zero effect lasting `3600` ticks.

Redstone Dust, not another Cream, owns the later Fire-Resistance-to-Long-Fire-Resistance edge,
whose effect lasts `9600` ticks. No strong Fire Resistance potion or Glowstone edge is registered.

A completed brew transforms every matching bottle slot in owner order, then consumes one Cream
with no remainder and emits the generic brew event. Cream is not a direct member of
`brewing_fuel`, is not furnace fuel and cannot prepay the stand's fuel uses; a separate valid fuel
source is required.

Slot admission, fuel uses, 400-tick timer/cancellation, bottle transforms, automation and the
player-menu take criterion remain with `ITM-BREW-001` and `ITM-ADVANCEMENT-001`. Fire Resistance
damage admission and later potion-container behavior remain with `ENT-EFFECT-001` and
`ITM-POTION-001`.

**Persistence and reload boundary:**

Stacks persist and synchronize identity, count and arbitrary ordinary component patches. They
store no cube size, damage source, attacker, Looting level, count draw/table cursor, chest table
or roll, brewing slot/fuel/timer/potion mix, recipe identity or advancement progress. Those values
belong to the death/loot transaction, container fill, machine block entity, server mix/recipe
managers and player progression state.

Loot reload can independently replace any of the three acquisition tables for future evaluations.
A rebuilt baseline mix table retains both Cream edges while Fire Resistance is feature-enabled;
existing stacks and in-flight machine state are not retroactively rewritten. Recipe/advancement
reload can independently replace crafting and unlock records. Resource reload independently
controls name and model.

**Client and wire projection:**

Generic item-stack encoding projects raw item ID `1154` plus the stack's component patch. Its
common-rarity name uses locked English text `Magma Cream`; the plain class adds no subtype tooltip
or forced glint.

The direct item definition selects generated model `minecraft:item/magma_cream` and its same-named
texture. It appears exactly once and only in Ingredients, ordered Pufferfish, Magma Cream, Golden
Carrot.

**Branches and aborts:**

Identity/count/components; generic hand/block/container/anvil path; admitted cube death,
source-entity type/variant, size, attacker/Looting and base/bonus draws; Bastion table/pool/roll/
weight/count and insertion; two recipe shapes/ingredients; brewing fuel/work/bottle/potion state;
two asymmetric unlock criteria; save, loot/recipe/mix/resource reload, wire, language, model and tab
context.

**Constants and randomness:**

Raw item ID `1154`; common rarity; max stack `64`; cube size minimum `2`; base count uniform integer
`-2..1`; active Looting result `max(B,0)+round(L*U)` for `U` uniform `[0,1)`; no-Looting emission
probability `1/4`; `bastion_other` `3..4` rolls, weight `2/13`, count `2..6`;
`bastion_treasure` `3..4` rolls, weight `1/9`, count `3..8`; owner brew duration `400`; Fire
Resistance durations `3600/9600`; one Powder plus one Ball to one Cream; four Cream to one Magma
Block.

**Side effects:**

Possible Cream or alternate froglight loot stack and named-sequence cursor; possible chest stacks
and container fill; generic world drop/pickup; brewing ingredient, bottles/timer/event; crafting
inputs and two results; advancement and recipe known/highlight state; ordinary stack persistence/
wire state; name, direct model and one Ingredients-tab entry.

**Gates:**

Generic stack/container/anvil admission; valid cube death table, non-Frog source and size `2+` for
Cream or exact Frog variant for alternate output; living attacker for Looting; exact Bastion chest
table and weighted roll; valid brewing fuel, bottle and feature-enabled mix; exact crafting
ingredients/grid; exact inventory or recipe-unlocked criterion; valid registry/stack decode;
client language/model and tab bootstrap.

**State read/written:**

Reads stack identity/count/components, interaction/container state, cube size and death/damage/
attacker context, three loot sequences, Bastion chest context, brewing slots/fuel/timer/mix table,
recipe/advancement registries and player progression state, persisted stack and client resources.
Writes only the loot/container, brewing, crafting, progression, stack and client projection listed
above.

**Failure behavior:**

Use has no subtype success or mutation. A suppressed/missing table, failed source/size condition or
nonpositive final Cream count emits no Cream. A Frog source diverts to a matching froglight instead
of Cream. An absent living attacker removes only the Looting bonus. A failed chest weight emits its
selected alternative. Missing fuel or an unmatched bottle prevents a brew under the generic owner;
Cream is not itself fuel. Invalid or insufficient crafting leaves inputs unchanged. Missing/
replaced loot, recipe, advancement or mix data removes those future paths without rewriting
stacks. Client resource absence follows generic missing translation/model fallback and cannot
grant authority.

**Boundary cases and quirks:**

The cube table is not player-gated and its Cream condition tests the damage source's source entity,
not player credit. Its negative base counts are intentional: without Looting only the `1` draw
emits, while active Looting normalizes every nonpositive base through `getCount` before adding the
bonus. Frog diversion maps warm/cold/temperate to Pearlescent/Verdant/Ochre respectively and does
not test size. Cream directly brews a Mundane dead-end from Water but cannot fuel the stand.
Blaze Powder possession unlocks the shapeless Cream recipe without requiring Slime Ball, whereas
Cream possession unlocks only the downstream Magma Block recipe.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-DATA-001`; `OFF-REPORT-001`;
`net.minecraft.world.item.Items`;
`net.minecraft.world.item.ItemStack#getCount`;
`net.minecraft.world.item.ItemStack#grow`;
`net.minecraft.world.item.CreativeModeTabs`;
`net.minecraft.world.level.storage.loot.functions.SetItemCountFunction#run`;
`net.minecraft.world.level.storage.loot.functions.EnchantedCountIncreaseFunction#run`;
`net.minecraft.world.level.storage.loot.providers.number.UniformGenerator`;
`net.minecraft.world.level.storage.loot.entries.LootPoolSingletonContainer$EntryBase#getWeight`;
`net.minecraft.world.item.alchemy.PotionBrewing#addVanillaMixes`;
`net.minecraft.world.item.alchemy.PotionBrewing$Builder#addStartMix`;
`net.minecraft.world.item.alchemy.Potions`;
`net.minecraft.data.loot.packs.VanillaEntityLoot`;
`net.minecraft.data.loot.packs.VanillaChestLoot`;
`net.minecraft.data.recipes.packs.VanillaRecipeProvider`;
`reports/registries.json#minecraft:{item,potion}`;
`reports/minecraft/components/item/magma_cream.json`;
`data/minecraft/loot_table/{entities/magma_cube,chests/{bastion_other,bastion_treasure}}.json`;
`data/minecraft/recipe/{magma_cream,magma_block}.json`;
`data/minecraft/advancement/recipes/{brewing/magma_cream,building_blocks/magma_block}.json`;
`assets/minecraft/{items,models/item,textures/item}/magma_cream.*`;
`ITM-BREW-001`; `ITM-LOOT-001`; `ITM-RECIPE-001`; `ITM-ADVANCEMENT-001`;
`ITM-POTION-001`; `ENT-DEATH-001`; `ENT-ENTITY-DROPS-001`; `BLK-MAGMA-001`;
`WGEN-JIGSAW-BASTION-001`; `CLI-UI-001`; `CLI-EFFECT-001`; `EXP-ITM-047`.

**Test vectors:**

Exercise default/patched stacks through hands, blocks, containers and anvil. Evaluate cube deaths
across generic loot admission, every source-entity type/Frog variant, size, attacker/Looting level,
base integer and bonus float while tracing stored versus `getCount` count and named sequence.
Force every weight/count/roll boundary in both Bastion pools and their preceding/following pools.
Brew Water, Awkward, Fire Resistance and every other potion with valid/invalid fuel, bottles and
feature state. Match/craft both recipes and trigger every possession/recipe-unlocked/brewed-potion
criterion before/after data reload. Persist/synchronize stacks and capture raw ID, name, tooltip,
model and exact Ingredients position before/after resource reload.

**Limits:**

This leaf does not duplicate Magma Cube or Frog AI/spawning/death, generic damage-source and loot
emission, froglight or Magma Block runtime, Bastion placement/chest filling, brewing transaction/
automation, Fire Resistance effects, crafting consumption or recipe-book/advancement state. Those
remain with their cited owners; this rule fixes the Cream identity and its exact acquisition,
crafting, brewing, progression and presentation joins.
