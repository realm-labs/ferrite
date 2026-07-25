# Items mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ITM-AMETHYST-SHARD-001` — Amethyst Shards join cluster and chest acquisition to four recipes, Allay duplication and amethyst armor trims

**Parent:** `BLK-BREAK-001`, `PLY-005`, `PLY-006`, `PLY-INPUT-001`,
`PLY-INTERACT-001`, `PLY-BREAK-001`, `ITM-001`, `ITM-002`, `ITM-003`,
`ITM-004`, `ITM-005`, `ITM-006`, `ITM-007`, `ITM-CONTAINER-001`,
`ITM-RECIPE-001`, `ITM-CRAFT-001`, `ITM-LOOT-001`,
`ITM-ADVANCEMENT-001`, `ITM-ANVIL-001`, `ITM-SMITHING-TEMPLATE-001`,
`ENT-001`, `MOB-BREED-001`, `WGEN-JIGSAW-ANCIENT-CITY-001`,
`WGEN-JIGSAW-TRIAL-CHAMBERS-001`, `CLI-001`, `CLI-006`, `CLI-UI-001`,
`CLI-EFFECT-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked registration/components, both direct item tags, every exact
`Items.AMETHYST_SHARD` class reference, the cluster table, two base chest tables, one replacement
table, four recipes and unlocks, all 18 trim recipes, the amethyst trim-material record, Allay
consumer and client resources determine every shard-specific branch. Generic loot, crafting,
smithing, entity lifecycle, structures and stack behavior remains with the cited owners.

**Applies when:**

An `amethyst_shard` stack is dropped from an Amethyst Cluster, generated in an Ancient City or
Trial Chambers intersection chest, consumed by one of four crafting recipes, used as an armor-trim
addition, offered to a dancing Allay, moved, renamed, persisted, synchronized, selected in
Ingredients or projected as an armor-trim slot hint, before and after component, tag, loot, recipe,
advancement, trim-material, built-in-pack or resource reload.

**Authoritative state:**

`minecraft:amethyst_shard` is raw item ID `930`. It is a common, nondamageable plain `Item` with
maximum stack `64`, no food, consumable, use, remainder, equipment, tool, cooldown, repairable,
fuel, compost or identity-specific glint behavior.

Alongside the common empty modifiers/enchantments/lore, break sound, name, model, repair cost,
swing animation, tooltip display and use effects, its registration supplies
`minecraft:provides_trim_material = minecraft:amethyst`. The referenced trim-material record has
asset name `amethyst` and translated description color `#9A5CC6`.

Its direct tags are exactly `#minecraft:duplicates_allays` and
`#minecraft:trim_materials`. The first contains only Amethyst Shard. The second contains eleven
materials and is the addition ingredient of all 18 armor-trim smithing recipes. Tag admission and
the `provides_trim_material` component are independent live gates.

**Transition and ordering:**

Amethyst Cluster acquisition:

The one-roll `blocks/amethyst_cluster` table uses random sequence
`minecraft:blocks/amethyst_cluster` and nested alternatives in this exact order:

1. a tool with Silk Touch level at least one emits one default Amethyst Cluster block item and
   never reaches either shard branch;
2. otherwise, a tool in `#minecraft:cluster_max_harvestables` emits four shards and applies the
   Fortune `ore_drops` formula;
3. every other tool or bare hand emits two shards, then applies explosion decay.

The harvestable tag contains Wooden, Stone, Copper, Iron, Golden, Diamond and Netherite Pickaxes.
At Fortune level `0`, the tagged branch emits four without an RNG draw. At positive level `L`, it
draws `D = nextInt(L+2)` and emits `4 * max(1,D)`: count four has probability `2/(L+2)`, and each
count `8,12,...,4*(L+1)` has probability `1/(L+2)`. Thus Fortune I emits four or eight with
probability `2/3` and `1/3`; Fortune II emits four/eight/twelve with probability
`1/2,1/4,1/4`.

The fallback branch's explosion decay independently retains each of its two shards under the
generic explosion-radius rule; outside an explosion it emits both. Fortune on a non-harvestable
tool never reaches the bonus formula. Silk Touch on such a tool still wins the first alternative.
Support loss, stage growth and the Cluster block/item state remain
`BLK-BUDDING-AMETHYST-001`.

Chest acquisition:

`chests/ancient_city` first draws uniform `5..10` rolls with replacement from total weight `84`.
Amethyst Shard has weight `3`, hence probability `1/28` per roll, and a selected entry sets count
uniformly to `1..15`. The later one-roll trim-template pool cannot emit another shard but advances
the same `minecraft:chests/ancient_city` named cursor.

Enabling the built-in `trade_rebalance` pack replaces that whole table. Its first pool swaps
Leather for an equal-weight Saddle while retaining total `84`, shard weight `3`, count `1..15`
and rolls `5..10`; shard odds therefore remain `1/28`. Its later pool adds a Mending Book and
changes Empty weight while keeping a one-roll total of `80`. Replacement changes future cursor
work after the first pool, does not layer both records and does not rewrite existing contents.

`chests/trial_chambers/intersection` has one pool with uniform `1..3` rolls and total weight `86`.
Shard weight is `20`, hence `10/43` per roll, and selection sets count uniformly to `8..20`.
Repeated selection is allowed. It uses matching named sequence
`minecraft:chests/trial_chambers/intersection`.

Ancient-City and Trial-Chambers template placement, lazy table evaluation and bounded container
insertion remain with the structure and generic loot owners.

Four shaped recipe sinks and unlocks:

No bundled recipe creates an Amethyst Shard. Four shaped recipes consume it:

| recipe | exact occupied pattern | result |
| --- | --- | --- |
| `amethyst_block` | a `2x2` square of four shards | one Amethyst Block |
| `tinted_glass` | one Glass surrounded orthogonally by four shards | two Tinted Glass |
| `calibrated_sculk_sensor` | one Sculk Sensor with shards north, west and east | one Calibrated Sculk Sensor |
| `spyglass` | one shard vertically above two Copper Ingots | one Spyglass |

Ordinary shaped trimming permits only the pattern's legal translations and mirror. The symmetric
block, glass and sensor patterns are unchanged by horizontal mirror; the one-column Spyglass
pattern can occupy any grid column. Extra occupied cells, wrong identities or missing cells fail.
Taking a result consumes every listed input, produces a default result and leaves no remainder;
arbitrary shard component patches do not propagate.

Each recipe's no-display `recipes/root` child has one OR requirement containing exact recipe
unlock and possession of an Amethyst Shard. Obtaining one shard therefore grants all four recipes
independently; unlocking one recipe does not unlock the other three.

Armor-trim material join:

Every one of the 18 armor-trim recipes requires its exact template, a
`#minecraft:trimmable_armor` base and a `#minecraft:trim_materials` addition. Live tag membership
therefore admits an Amethyst Shard to every pattern. Assembly separately reads the addition
stack's live `provides_trim_material` component; the default resolves holder
`minecraft:amethyst`, constructs `(amethyst, selected pattern)`, copies the base at count one and
sets its `TRIM` component.

If the base already has that exact material/pattern pair, assembly returns empty and there is no
takeable result. A base with the same material but another pattern, or the same pattern with
another material, remains valid and is replaced by the newly selected pair.

Removing shard from `trim_materials` blocks matching even while its component remains. Retaining
the tag but removing `provides_trim_material` lets the ingredients match but makes assembly
empty. Patching that component to another valid material makes the shard produce that material's
trim rather than amethyst. Smithing take consumes one template, one base and one shard only after
a takeable result; exact recipe/menu/event/stat/advancement ordering remains
`ITM-SMITHING-TEMPLATE-001`.

The trim material's `amethyst` asset name selects generic amethyst trim textures and its
`#9A5CC6` description color in trim text. Data reload can replace the holder record for future
resolution/projection without changing already persisted trim values unless their referenced
registry data is reinterpreted by the owner.

Allay duplication:

Allay interaction tests duplication before ordinary item-giving. When the target is dancing, the
held stack is in `duplicates_allays`, and synchronized `canDuplicate` is true, it requests one new
Allay with spawn reason `BREEDING`. A successful creation snaps the new Allay to the source,
marks it persistent, resets both source and child duplication cooldowns to `6000` ticks and
attempts world insertion.

Whether creation returned an Allay or null, the outer admitted branch then broadcasts entity
event `18`, plays `block.amethyst_block.chime` from the Allay in `NEUTRAL` at volume `2` and pitch
`1`, consumes one shard through the player-ability-aware rule, and returns `SUCCESS`. A failed
factory therefore still consumes and emits but leaves the source cooldown unchanged. Event `18`
renders three heart-particle offers on each receiving client.

Cooldown decrements by one on each server `aiStep` while positive and drives the synchronized
boolean true only at zero. It persists as `DuplicationCooldown`; unload pauses it with no
wall-clock catch-up. A nondancing Allay, a positive cooldown or removed tag falls through to
ordinary Allay item-giving/taking behavior: the shard has no special result there beyond being an
arbitrary held item. Full duplication lifecycle remains `MOB-BREED-001`.

Persistence and reload boundary:

Shard stacks persist and synchronize identity, count and arbitrary ordinary component patches,
including an overridden trim-material holder. They store no loot cursor, recipe knowledge,
craft/smith transaction, Allay dance/cooldown or armor result; those belong to their world,
player, menu and entity owners.

Loot and built-in-pack reload changes future table evaluation; recipe/advancement reload changes
future matching and grants; item-tag reload independently changes Allay and trim admission;
trim-material registry reload changes future holder resolution and presentation. Completed drops,
crafts, trims and duplications are not replayed. Resource reload independently replaces language,
item texture/model, slot hint and trim textures.

**Client and wire projection:**

Generic item-stack encoding projects raw item ID `930` plus the component patch. The common name
uses locked English text `Amethyst Shard`; the plain item adds no subtype tooltip or forced glint.

Its direct item definition selects generated model `minecraft:item/amethyst_shard` and the
same-named texture. It appears once in Ingredients, ordered Quartz, Amethyst Shard, Copper Nugget.
Armor-trim templates also include fixed empty-slot icon
`minecraft:container/slot/amethyst_shard` as the seventh addition hint after
ingot/redstone/lapis/quartz/diamond/emerald. That hint is presentation-only and does not replace
the live tag/component gates. The item adds no packet layout or numeric mapping.

**Branches and aborts:**

Identity/count/components and two tags; Cluster Silk/harvestable/Fortune/explosion alternatives;
base/rebalanced Ancient City and Trial intersection rolls/weights/counts/cursors/insertion; four
recipe shapes/translations/mirror/take/unlock paths; all 18 template/base/addition/equal-trim and
tag/component combinations; Allay dance/cooldown/factory/insertion/ability/event paths; save,
component/tag/loot/recipe/advancement/trim-material/pack/resource reload; wire, language, model,
slot hint, trim textures and tab.

**Constants and randomness:**

Raw ID `930`; common rarity; max stack `64`; Cluster base counts `4/2`; Fortune tagged count
`4*max(1,nextInt(L+2))`; Ancient City `5..10` rolls, `1/28`, count `1..15`; intersection `1..3`
rolls, `10/43`, count `8..20`; recipe shard consumption `4/4/3/1`, outputs `1/2/1/1`; 18 trim
patterns; trim color `#9A5CC6`; Allay cooldown `6000`, event `18`, sound volume/pitch `2/1`.

**Side effects:**

Cluster/chest shard stacks and named cursors; four crafting results and recipe grants; smithing
preview/result, trim component and input removal; Allay child/cooldowns/persistence/insertion,
event/sound/item consumption/particles; ordinary stack persistence/wire; name, model, texture,
trim text/assets, slot hint and creative entry.

**Gates:**

Cluster tool/enchantment/tag/Fortune/explosion context; active table/pack/roll/container;
recipe grid and snapshot; exact template, base tag, addition tag/component and unequal result;
Allay dancing/tag/cooldown/factory/player ability; registry/stack decode; client language/model/
trim/tab context.

**State read/written:**

Reads shard identity/count/components/tags, Cluster tool/loot context, structure/table context,
crafting/smithing inputs, recipe/advancement/trim records, Allay state, persistence and client
resources. Writes only the loot, crafting, progression, smithing, Allay, stack and client state
listed above.

**Failure behavior:**

Silk Touch emits Cluster rather than shards. Wrong harvest tools lose Fortune but still reach the
two-shard fallback; explosion decay can reduce it to zero. Unselected chest entries emit
alternatives. Invalid recipe grids produce no result. Removed trim tag, missing trim component or
equal trim yields no takeable output. Nondancing/cooldown Allays do not duplicate; a failed factory
still consumes and emits as specified. Missing/replaced reloadable data changes only future
attempts. Client-resource absence follows generic fallback and cannot grant authority.

**Boundary cases and quirks:**

Fortune is tool-tag-gated, while Silk Touch is not. The rebalanced Ancient City keeps shard odds
and count fixed but changes later cursor work. One shard possession unlocks four unrelated recipes
but no recipe creates a shard. Armor-trim matching needs the tag while output needs the component;
either can be patched independently. A failed Allay factory still consumes the shard. Amethyst
Shard is both an authoritative trim material and a hard-coded empty-slot hint, but only the former
affects results.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-DATA-001`; `OFF-REPORT-001`;
`net.minecraft.world.item.Items`;
`net.minecraft.world.level.storage.loot.functions.ApplyBonusCount$OreDrops#calculateNewCount`;
`net.minecraft.world.item.crafting.SmithingTrimRecipe#applyTrim`;
`net.minecraft.world.entity.animal.allay.Allay#mobInteract`;
`net.minecraft.world.entity.animal.allay.Allay#duplicateAllay`;
`net.minecraft.world.entity.animal.allay.Allay#updateDuplicationCooldown`;
`net.minecraft.world.item.SmithingTemplateItem#createTrimmableMaterialIconList`;
`net.minecraft.world.item.CreativeModeTabs`;
`net.minecraft.data.tags.VanillaItemTagsProvider`;
`net.minecraft.data.loot.packs.VanillaBlockLoot`;
`net.minecraft.data.loot.packs.VanillaChestLoot`;
`net.minecraft.data.loot.packs.TradeRebalanceChestLoot`;
`net.minecraft.data.recipes.packs.VanillaRecipeProvider`;
`reports/registries.json#minecraft:{item,recipe,loot_table,advancement,trim_material}`;
`reports/minecraft/components/item/amethyst_shard.json`;
`data/minecraft/tags/item/{cluster_max_harvestables,duplicates_allays,trim_materials}.json`;
`data/minecraft/loot_table/{blocks/amethyst_cluster,chests/{ancient_city,trial_chambers/intersection}}.json`;
`data/minecraft/datapacks/trade_rebalance/data/minecraft/loot_table/chests/ancient_city.json`;
`data/minecraft/recipe/{amethyst_block,tinted_glass,calibrated_sculk_sensor,spyglass}.json`;
`data/minecraft/advancement/recipes/{building_blocks/{amethyst_block,tinted_glass},redstone/calibrated_sculk_sensor,tools/spyglass}.json`;
`data/minecraft/recipe/*_armor_trim_smithing_template_smithing_trim.json`;
`data/minecraft/trim_material/amethyst.json`;
`assets/minecraft/{items,models/item,textures/item}/amethyst_shard.*`;
`assets/minecraft/textures/trims/**/amethyst*.png`;
`BLK-BUDDING-AMETHYST-001`; `BLK-AMETHYST-BLOCK-001`;
`BLK-TINTED-GLASS-001`; `BLK-SCULK-SENSOR-001`; `ITM-SMITHING-TEMPLATE-001`;
`MOB-BREED-001`; `WGEN-JIGSAW-ANCIENT-CITY-001`;
`WGEN-JIGSAW-TRIAL-CHAMBERS-001`; `CLI-UI-001`; `CLI-EFFECT-001`;
`EXP-ITM-058`.

**Test vectors:**

Break Cluster with bare, all seven tagged Pickaxes, untagged tools, Silk Touch and Fortune at every
level under controlled explosion contexts; trace both alternatives, exact count formula and named
cursor. Generate base/rebalanced Ancient City and Trial intersection tables through every
roll/weight/count/repeated-selection/later-pool/insertion path.

Match, mirror, translate and take all four recipe patterns with default and patched shards; trigger
possession and individual recipe unlocks. Run all 18 trims across default/removed/patched tag and
component combinations, absent/equal/different base trim, result take, persistence and client
projection.

Interact with dancing/nondancing and cooldown-ready/blocked Allays using finite/infinite holders,
removed tag and successful/null factories; assert child/cooldown/insertion/event/sound/consumption
order, persistence and reload. Capture raw ID, common name, absent forced glint, item model/texture,
amethyst trim color/assets, fixed slot hint and Ingredients neighbors.

**Limits:**

This leaf does not duplicate generic loot execution, shaped crafting, smithing-menu commits,
trimmed-equipment rendering, Allay dance/entity lifecycle, structure placement or stack/resource
codecs. Those remain with their cited owners; this rule fixes Amethyst Shard identity and its exact
acquisition, recipe, trim, duplication and presentation joins.
