# Items mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ITM-PRISMARINE-MATERIAL-001` — Prismarine materials join guardian, treasure and sea-lantern loot to four fixed recipes

**Parent:** `BLK-002`, `BLK-BREAK-001`, `PLY-005`, `PLY-006`, `PLY-INPUT-001`,
`PLY-INTERACT-001`, `PLY-BREAK-001`, `ITM-001`, `ITM-002`, `ITM-003`, `ITM-004`,
`ITM-005`, `ITM-006`, `ITM-007`, `ITM-USE-001`, `ITM-CONTAINER-001`,
`ITM-RECIPE-001`, `ITM-CRAFT-001`, `ITM-LOOT-001`, `ITM-ADVANCEMENT-001`,
`ITM-ANVIL-001`, `ENT-001`, `ENT-005`, `ENT-DEATH-001`, `ENT-ENTITY-DROPS-001`,
`WGEN-STRUCTURE-BURIED-001`, `WGEN-STRUCTURE-OCEAN-MONUMENT-001`, `CLI-001`,
`CLI-006`, `CLI-UI-001`, `CLI-EFFECT-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked plain-item registrations/components, exhaustive code and data
references, all four acquisition records, all four recipes and advancements, and direct client
assets determine every identity-specific branch. Generic stack, interaction, entity death, block
break, loot, crafting and advancement behavior remains with the cited owners.

**Applies when:**

A `prismarine_shard` or `prismarine_crystals` stack is created, looted, moved, renamed, persisted,
synchronized, offered to crafting, selected in a tab, rendered or observed before and after loot,
recipe or resource reload.

**Authoritative state:**

`minecraft:prismarine_shard` is raw item ID `1277`; `minecraft:prismarine_crystals` is raw item ID
`1278`. Both register through the plain-item path with default properties. Each is common,
nondamageable, max stack `64`, and belongs to no direct item tag.

Their registered components are only the common empty modifiers/enchantments/lore, item-break
sound, translated name, direct item-model key, repair cost, swing animation, tooltip display and
use effects. Neither identity has food, consumable, cooldown, remainder, tool, equipment,
repairable, fuel, brewing, composting or other operational state.

**Transition and ordering:**

Neither identity overrides hand use or block use. A prototype stack's air use returns generic
`PASS`; a block click participates only in ordinary block-first interaction and fallback handling.
A component-patched stack can activate a generic component owner, but no identity-specific branch
consumes a stack, starts active use, emits a sound/game event/particle, increments item use or
changes the world.

Container movement, pickup, dropping, anvil naming and component patching use their generic owners.
The identities add no dispenser, mob-interaction, equipment, villager, enchantment, repair or block
predicate. Their server gameplay joins are exactly the acquisition and recipe records below.

**Guardian acquisition:**

The Guardian and Elder Guardian entity tables each begin with an unconditional one-roll shard
pool. Its only entry creates one shard, replaces its count with a uniformly drawn integer `0..2`,
then applies enchanted count increase. With a living attacking entity whose Looting level is
`L > 0`, that function adds `round(L * U)` for a fresh uniform float `U` in `[0,1)`; with no living
attacker or `L = 0`, it returns without that draw or bonus. There is no killed-by-player gate and
no configured count limit. A final zero count emits no stack under the generic loot owner.

The next one-roll pool chooses Cod, crystals or empty. Guardian weights are `2/2/1`, so crystals
have selection probability `2/5`; Elder Guardian weights are `3/2/1`, so crystals have probability
`1/3`. A selected crystal entry starts at count `1` and applies the same
`round(L * U)` Looting increase. Cod's optional smelting branch does not alter a crystal selection.

The tables use distinct random sequences `minecraft:entities/guardian` and
`minecraft:entities/elder_guardian`. Later rare-fish, wet-sponge and trim-template pools cannot
retroactively alter material output, but complete deterministic replay must retain the selected
table's shared cursor. Entity admission to death loot, stack emission and world placement remain
with `ENT-DEATH-001`, `ENT-ENTITY-DROPS-001` and `ITM-LOOT-001`.

**Buried-treasure and block acquisition:**

Only crystals join `chests/buried_treasure`. Its third pool draws an integer roll count uniformly
from `1..3`; Emerald, Diamond and crystals each have weight `5` of total `15`. Every roll therefore
selects crystals with probability `1/3`, then replaces the selected stack count with a uniformly
drawn integer `1..5`. Other selected entries consume their own count draws, so replay follows
`minecraft:chests/buried_treasure` rather than independently sampling crystal rolls. Structure
placement, chest orientation, seed assignment and deferred loot evaluation belong to
`WGEN-STRUCTURE-BURIED-001`.

Sea Lantern's block table produces no crystals when its tool predicate matches Silk Touch level at
least `1`; that first alternative emits the Sea Lantern block. Otherwise it creates crystals with
base count `B` uniformly drawn from integers `2..3`. If a tool is present with Fortune level `F`,
uniform bonus count changes this to `B + nextInt(F + 1)`; the subsequent range function clamps the
result to `1..5`.

Explosion decay then runs only when an explosion-radius context value `R` exists, tests every unit
with a fresh float, and retains it when that float is at most `1/R`. Without that context, the
clamped count is unchanged. This table uses random sequence `minecraft:blocks/sea_lantern`;
block-break admission, tool context, loot invocation and drop placement remain with their generic
owners.

No other locked baseline loot table, block drop, structure payload, trade, mob drop, recipe result
or advancement reward creates either material. Administration and custom data can still create
ordinary stacks through generic item and loot boundaries.

**Recipes and progression:**

Four building-category recipes consume these identities and return fixed default results:

- Prismarine is shaped `##/##`, consuming four shards and returning one `minecraft:prismarine`.
- Prismarine Bricks is shapeless with nine separate shard ingredients, consuming nine shards and
  returning one `minecraft:prismarine_bricks`.
- Dark Prismarine is shaped `SSS/SIS/SSS`, consuming eight shards around one black dye and returning
  one `minecraft:dark_prismarine`.
- Sea Lantern is shaped `SCS/CCC/SCS`, consuming four corner shards and five cross-position
  crystals and returning one `minecraft:sea_lantern`.

None has a remainder or copies input component patches. The Prismarine, Prismarine Bricks and Dark
Prismarine recipe advancements each use one OR requirement: possession of a shard or prior unlock
of that exact recipe. Sea Lantern instead uses possession of crystals or prior unlock of the Sea
Lantern recipe; shards alone do not satisfy its possession criterion despite being required by the
recipe. Each reward unlocks only its corresponding recipe.

Generic shaped/shapeless matching, consumption, result transfer, recipe-book packets and reentrant
recipe-unlocked criteria remain with `ITM-RECIPE-001`, `ITM-CRAFT-001` and
`ITM-ADVANCEMENT-001`.

**Persistence and reload boundary:**

Stacks persist and synchronize identity, count and arbitrary ordinary component patches. They
store no loot source, attacker, Looting/Fortune level, RNG cursor, chest provenance, block context
or recipe identity. Entity and block loot contexts are transaction state; buried-treasure seed and
deferred table identity belong to the chest; known/highlight recipe state belongs to the player.

Loot reload can independently replace guardian, elder-guardian, buried-treasure and sea-lantern
tables for future evaluations. Recipe/advancement reload can replace ingredient, output and unlock
records without rewriting existing stacks. Resource reload independently controls names and
models.

**Client and wire projection:**

Generic item-stack encoding projects raw item IDs `1277` and `1278` plus each stack's component
patch. Their common-rarity names use locked English text `Prismarine Shard` and
`Prismarine Crystals`; neither class adds a subtype tooltip.

Both direct item definitions select generated models and same-named textures:
`minecraft:item/prismarine_shard` and `minecraft:item/prismarine_crystals`. Each appears exactly
once and only in Ingredients, ordered Clay Ball, Prismarine Shard, Prismarine Crystals, Nautilus
Shell.

**Branches and aborts:**

Identity/count/components; hand/block/container/anvil path; Guardian versus Elder Guardian;
attacking entity and Looting level; shard `0..2`; crystal weighted selection; buried-treasure
placement, roll, weight and `1..5` count; Sea Lantern Silk Touch, tool/Fortune and explosion
context; four recipe grids and counts; possession versus recipe-unlocked criterion; save, reload,
wire, language, model and tab context.

**Constants and randomness:**

Raw IDs shard/crystals `1277/1278`; common rarity; max stack `64`; guardian shard count `0..2`;
Looting bonus `round(L * U[0,1))`; crystal weights Guardian `2/5`, Elder Guardian `2/6`;
buried-treasure rolls `1..3`, crystal weight `5/15`, count `1..5`; Sea Lantern base `2..3`,
Fortune bonus `nextInt(F+1)`, cap `5`, explosion survival threshold `1/R`; recipe inputs shard
`4/9/8/4`, crystals `5`; every output count `1`. There is no item-use randomness.

**Side effects:**

Loot stacks, chest/block/entity drop placement and their random-sequence cursors under generic
owners; crafting inputs/results; recipe advancement, known/highlight and recipe-book projection;
ordinary stack persistence/wire state; names, direct models and two Ingredients-tab entries.

**Gates:**

Generic inventory/container/anvil admission; valid entity death loot context; selected guardian
table and attacking entity; retained buried-treasure chest and valid seed/table; Sea Lantern
block-break/tool/explosion context; exact recipe ingredients/grid; either advancement criterion;
valid stack/registry decode; client language/model and tab bootstrap.

**State read/written:**

Reads stack identity/count/components, interaction/container state, entity death and enchantment
context, block/tool/explosion context, buried-treasure loot state, recipe/advancement registries and
player recipe state, persisted stack and client resources. Writes only the loot, crafting,
progression, stack and client projection listed above.

**Failure behavior:**

Use has no subtype success or mutation. Failed death/block/chest admission or an unselected entry
emits no corresponding material. Silk Touch selects the block alternative and no crystals. Invalid
or insufficient crafting leaves inputs unchanged under the generic owner. Missing or replaced
loot, recipe and advancement data remove those future paths without rewriting stacks. Client
resource absence follows generic missing translation/model fallback and cannot grant authority.

**Boundary cases and quirks:**

A zero shard base count can still become positive when the following Looting increase is nonzero.
Guardian and Elder Guardian share formulas but not their crystal selection probability or random
sequence. Sea Lantern needs both material identities, but its advancement possession criterion
checks only crystals. Silk Touch and the crystal/Fortune branch are alternatives, not cumulative
drops. Neither material can be placed as its corresponding block.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-DATA-001`; `OFF-REPORT-001`;
`net.minecraft.world.item.Items`;
`net.minecraft.world.item.CreativeModeTabs`;
`net.minecraft.world.level.storage.loot.functions.EnchantedCountIncreaseFunction`;
`net.minecraft.world.level.storage.loot.functions.ApplyBonusCount`;
`net.minecraft.world.level.storage.loot.functions.ApplyBonusCount$UniformBonusCount`;
`net.minecraft.world.level.storage.loot.functions.ApplyExplosionDecay`;
`net.minecraft.world.level.storage.loot.providers.number.UniformGenerator`;
`net.minecraft.data.loot.packs.VanillaEntityLoot`;
`net.minecraft.data.loot.packs.VanillaChestLoot`;
`net.minecraft.data.loot.packs.VanillaBlockLoot`;
`net.minecraft.data.recipes.packs.VanillaRecipeProvider`;
`reports/registries.json#minecraft:item`;
`reports/minecraft/components/item/{prismarine_shard,prismarine_crystals}.json`;
`data/minecraft/loot_table/entities/{guardian,elder_guardian}.json`;
`data/minecraft/loot_table/chests/buried_treasure.json`;
`data/minecraft/loot_table/blocks/sea_lantern.json`;
`data/minecraft/recipe/{prismarine,prismarine_bricks,dark_prismarine,sea_lantern}.json`;
`data/minecraft/advancement/recipes/building_blocks/{prismarine,prismarine_bricks,dark_prismarine,sea_lantern}.json`;
`assets/minecraft/{items,models/item,textures/item}/{prismarine_shard,prismarine_crystals}.*`;
`ENT-DEATH-001`; `ENT-ENTITY-DROPS-001`; `WGEN-STRUCTURE-BURIED-001`;
`WGEN-STRUCTURE-OCEAN-MONUMENT-001`; `ITM-LOOT-001`; `ITM-RECIPE-001`;
`ITM-ADVANCEMENT-001`; `CLI-UI-001`; `CLI-EFFECT-001`; `EXP-ITM-038`.

**Test vectors:**

Exercise default/patched stacks through hands, blocks, containers and anvil. Evaluate both entity
tables across attacker types, Looting levels, all shard counts and weighted crystal branches;
buried treasure across every roll/count/entry; and Sea Lantern across Silk Touch, Fortune/tool
absence and explosion radii while recording exact sequence cursors. Match/craft all four recipes
at every grid/count boundary and trigger both criteria before/after data reload. Persist and
synchronize stacks, then capture raw IDs, names, tooltips, direct models and exact Ingredients
order before/after resource reload.

**Limits:**

This leaf does not duplicate generic entity death, block break, loot invocation/drop placement,
buried-treasure or ocean-monument generation, inventory/anvil semantics, crafting consumption,
recipe-book/advancement state, or the behavior of the four crafted blocks. Those remain with their
cited owners; this rule fixes the two material identities and their exact acquisition, ingredient
and presentation joins.
