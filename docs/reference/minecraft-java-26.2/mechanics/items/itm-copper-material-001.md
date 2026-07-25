# Items mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ITM-COPPER-MATERIAL-001` — Raw Copper, Copper Ingots and Copper Nuggets join ore and mob acquisition to crafting, repair and armor trim

**Parent:** `PLY-005`, `PLY-006`, `PLY-INPUT-001`, `PLY-INTERACT-001`,
`ITM-001`, `ITM-002`, `ITM-003`, `ITM-004`, `ITM-005`, `ITM-006`,
`ITM-007`, `ITM-USE-001`, `ITM-CONTAINER-001`, `ITM-RECIPE-001`,
`ITM-CRAFT-001`, `ITM-FURNACE-001`, `ITM-SMITHING-001`,
`ITM-LOOT-001`, `ITM-ADVANCEMENT-001`, `ITM-ENCHANT-001`,
`ITM-ANVIL-001`, `ENT-001`, `ENT-DEATH-001`, `ENT-ENTITY-DROPS-001`,
`MOB-001`, `MOB-004`, `BLK-BREAK-HOOK-001`, `BLK-RAW-STORAGE-001`,
`WGEN-PIPELINE-001`, `CLI-001`, `CLI-006`, `CLI-UI-001`,
`CLI-EFFECT-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked registrations and components, four direct tags, 54 unique recipe
records and their advancement joins, two ore and two entity loot tables, Copper trim material,
tool/armor material construction, standard ore features, the ore-vein join and direct client
resources determine every Raw-Copper, Copper-Ingot and Copper-Nugget-specific branch. Generic
stack, crafting, cooking, repair, smithing, loot, advancement, worldgen and rendering algorithms
remain with the cited owners.

**Applies when:**

A `minecraft:raw_copper`, `minecraft:copper_ingot` or `minecraft:copper_nugget` stack is created,
matched, cooked, crafted, used as repair or trim material, moved, renamed, persisted, synchronized
or rendered; or when Copper Ore, Deepslate Copper Ore, a Drowned or a Copper Golem is evaluated as
one of the family's acquisition sources before and after recipe, tag, advancement, loot, trim
material or resource reload.

**Authoritative state:**

Raw Copper, Copper Ingot and Copper Nugget have raw item IDs `933`, `934` and `1336`. All three
register as common nondamageable plain `Item` instances with maximum stack `64`. Their defaults
contain the common empty modifiers, enchantments and lore, item-break sound, translated name,
direct item-model key, repair cost, swing animation, tooltip display and use effects.

Copper Ingot additionally has default
`minecraft:provides_trim_material=minecraft:copper`. Raw Copper and Copper Nugget do not. None of
the three has food, consumable, remainder, durability, equipment, tool, projectile, cooldown,
inventory-tick or identity-specific use behavior. The three item identities and arbitrary valid
component patches persist through generic stack owners; recipe and tag tests below do not require
the default component map unless explicitly stated for trim assembly.

The complete direct item-tag memberships are:

| Item | Direct tags |
|---|---|
| Raw Copper | none |
| Copper Ingot | `copper_tool_materials`, `repairs_copper_armor`, `trim_materials` |
| Copper Nugget | `metal_nuggets` |

The first two Ingot tags each contain only Copper Ingot. `trim_materials` contains eleven
identities. `metal_nuggets` contains Copper, Iron and Gold Nuggets. A direct membership is distinct
from a recipe's exact-item ingredient and from the Ingot's trim-material component.

**Transition and ordering:**

The identities do not override air or block use. A prototype stack returns generic `PASS` in air
and participates only in ordinary block-first interaction. Container movement, pickup, dropping,
anvil naming and valid component patching use generic owners. Operational behavior enters through
the material, recipe, loot and generation joins below.

### Copper material tags and repair

`ToolMaterial.COPPER` is constructed with `incorrect_for_copper_tool`, durability `190`, mining
speed `5.0`, attack-damage bonus `1.0`, enchantment value `13` and
`copper_tool_materials` as its repair set. Copper Pickaxe, Shovel, Axe, Hoe, Sword and Spear
therefore store a named repairable holder set whose locked sole member is Copper Ingot.

`ArmorMaterials.COPPER` uses durability multiplier `11`, enchantment value `8`, Copper equip
sound, zero toughness and knockback resistance, and `repairs_copper_armor` as its repair set. The
four humanoid Copper Helmet, Chestplate, Leggings and Boots use that named set. Copper Horse Armor
and Copper Nautilus Armor deliberately have no repairable component, even though both are admitted
by the recycling recipes below.

Anvil material-repair matching tests the offered stack's item holder against the target's stored
repair set. Consequently any Copper-Ingot stack identity, including one with ordinary component
patches or a removed/replaced trim-material component, is admitted for the six tools and four
humanoid armor items. Raw Copper and Copper Nugget are rejected. Per-unit damage removal, prior
work, level price, output copying and commit remain `ITM-ANVIL-001`; this leaf fixes the material
admission only.

### Cooking and material conversion

Six Copper-Ingot cooking records form three exact input pairs:

| Input | Furnace output / time / recipe XP | Blast Furnace output / time / recipe XP |
|---|---|---|
| Copper Ore | one Copper Ingot / `200` / `0.7` | one Copper Ingot / `100` / `0.7` |
| Deepslate Copper Ore | one Copper Ingot / `200` / `0.7` | one Copper Ingot / `100` / `0.7` |
| Raw Copper | one Copper Ingot / `200` / `0.7` | one Copper Ingot / `100` / `0.7` |

The six records omit cooking time and therefore use their serializers' `200`/`100` defaults. They
share group `copper_ingot`. Furnace rejects the blasting records, Blast Furnace rejects the
smelting records, and Smoker and Campfire reject all six.

Two recycling records accept the same exact twelve-identity ingredient: Copper Pickaxe, Shovel,
Axe, Hoe, Sword, Spear, Helmet, Chestplate, Leggings, Boots, Horse Armor or Nautilus Armor.
Furnace processing emits one Copper Nugget after `200` ticks and records XP `0.1`; Blast Furnace
processing emits one after `100` ticks and records XP `0.1`. Remaining durability, enchantments,
trim and other arbitrary input components do not alter identity matching and are discarded.

All eight cooking outputs are default stacks and copy no input component. They leave no remainder.
Fuel, progress/reset, result capacity, recipe-used accounting, extraction and fractional
experience remain `ITM-FURNACE-001`.

### Crafting graph

Seven compacting/decompression records are exact:

- a full `3×3` of nine Copper Ingots emits one Copper Block;
- one Copper Block emits nine Copper Ingots;
- one Waxed Copper Block separately emits nine Copper Ingots;
- a full `3×3` of nine Copper Nuggets emits one Copper Ingot;
- one Copper Ingot emits nine Copper Nuggets;
- a full `3×3` of nine Raw Copper emits one Raw Copper Block;
- one Raw Copper Block emits nine Raw Copper.

The Raw-Copper pair and Raw-Copper-Block behavior are also fixed from the block side by
`BLK-RAW-STORAGE-001`. No recipe accepts an exposed, weathered or oxidized Copper Block in place
of either named full block.

Fifteen direct construction records use the following exact grids:

| Result | Exact pattern and output |
|---|---|
| Brush | vertical Feather / Copper Ingot / Stick; one |
| Copper Bars | two full rows of three Ingots; `16` |
| Copper Boots | `X X / X X`; one from four Ingots |
| Copper Chestplate | `X X / XXX / XXX`; one from eight Ingots |
| Copper Helmet | `XXX / X X`; one from five Ingots |
| Copper Leggings | `XXX / X X / X X`; one from seven Ingots |
| Copper Chain | vertical Nugget / Ingot / Nugget; one |
| Copper Chest | eight Ingots surrounding one Chest; one |
| Copper Door | three rows of two Ingots; `3` |
| Copper Trapdoor | full `2×2` of four Ingots; one |
| Lightning Rod | vertical three Ingots; one |
| Spyglass | centered Amethyst Shard / Ingot / Ingot; one |
| Copper Lantern | eight Nuggets surrounding one Copper Torch; one |
| Copper Torch | vertical Copper Nugget / Coal-or-Charcoal / Stick; `4` |
| Name Tag | diagonal two-by-two ` X / # `, where `X` is live `metal_nuggets` and `#` is Paper; one |

Copper Ingot is also the locked sole `copper_tool_materials` member used at every `X` in six
shaped tool records:

| Tool | Pattern (`#` is Stick) |
|---|---|
| Copper Axe | `XX / X# /  #` |
| Copper Hoe | `XX /  # /  #` |
| Copper Pickaxe | `XXX /  #  /  # ` |
| Copper Shovel | `X / # / #` |
| Copper Spear | `  X /  #  / #  ` |
| Copper Sword | `X / X / #` |

The recipes accept live tag members, so a tag reload can broaden their material identities even
though the locked baseline has only Copper Ingot. The four armor recipes and all other named
Ingot positions above are exact-item ingredients. Copper Torch encodes Coal and Charcoal inline
rather than through `coals`; Name Tag alone uses `metal_nuggets`.

Extra, missing or misplaced inputs fail. Patterns may translate or mirror under the shaped-recipe
owner where applicable; the three-row recipes require a `3×3` grid. Successful assembly emits the
listed default stack, copies no arbitrary input patches and leaves no remainder.

### Copper armor trim

Eighteen smithing-trim records—Bolt, Coast, Dune, Eye, Flow, Host, Raiser, Rib, Sentry, Shaper,
Silence, Snout, Spire, Tide, Vex, Ward, Wayfinder and Wild—each require their exact template, a
live `trimmable_armor` base and a live `trim_materials` addition. Default Copper Ingot satisfies
the addition ingredient.

After ingredient matching, assembly reads `provides_trim_material` from the actual addition stack.
A default Copper Ingot supplies the `minecraft:copper` holder. A component-patched Copper Ingot
can instead supply another valid trim holder; removing the component makes assembly return empty
even though the item still matches `trim_materials`. An already identical material-and-pattern
trim also returns empty. Otherwise assembly copies the base at count one and replaces its trim
component; the Smithing Table consumes the occupied roles only when the preview is taken under
`ITM-SMITHING-001`.

The locked Copper trim-material record has asset name `copper`, description color `#B4684D`,
translation `trim_material.minecraft.copper` (`Copper Material`) and one armor-asset override:
Copper equipment uses `copper_darker`; all other admitted equipment uses `copper`. Thus the item
tag controls recipe admission, the stack component controls the material holder, and the trim
registry/resource records control tooltip and visual projection.

### Recipe progression

Every relevant recipe advancement has a single requirement row, so its listed criteria are OR
routes with the already-known-recipe criterion:

- Copper Ingot possession unlocks Copper Block, all four humanoid armor pieces, Copper Bars,
  Copper Chain, Copper Door, Copper Trapdoor, Lightning Rod, Brush and Copper Nugget;
- any live `copper_tool_materials` member unlocks each of the six Copper tool recipes, making
  default Copper Ingot sufficient for all six;
- Copper Nugget possession unlocks Copper Chain, Copper Ingot from Nuggets and Copper Torch;
- any live `metal_nuggets` member, Paper or Name Tag unlocks Name Tag;
- Raw Copper possession unlocks its Furnace and Blast-Furnace records plus Raw Copper Block;
- the exact corresponding ore unlocks each ore cooking record, Copper Block or Waxed Copper Block
  unlocks its own decompression record, and Raw Copper Block unlocks Raw Copper;
- any one of the twelve Copper gear identities unlocks each Nugget recycling record;
- Copper Torch unlocks Copper Lantern, Amethyst Shard unlocks Spyglass, and Copper Chest unlocks
  its own recipe;
- each of the eighteen trim recipes is unlocked by possession of its corresponding exact smithing
  template, not by Copper Ingot possession.

Possessing Copper Ingot therefore does not by itself unlock Spyglass, Copper Chest, Copper Lantern,
the six cooking outputs or any trim recipe. Listener installation, already-known recipe routes,
knowledge persistence and craft criteria remain `ITM-ADVANCEMENT-001`.

### Copper-ore break acquisition

Copper Ore and Deepslate Copper Ore are property-free `DropExperienceBlock` states `27790` and
`27791`. Both are pickaxe-mineable, require the stone tool tier for drops and specify zero break
experience.

Each one-roll loot table first tests Silk Touch level at least one and emits its own default ore
block on that branch. Otherwise it creates Raw Copper, replaces count with an inclusive uniform
integer `C in 2..5`, applies Fortune's `ore_drops` formula and then explosion decay.

At Fortune zero the bonus stage retains `C` without a bonus draw. At positive Fortune level `L`,
it draws `D=nextInt(L+2)` and multiplies by `M=max(1,D)`: multiplier one has probability
`2/(L+2)`, and each multiplier `2..L+1` has probability `1/(L+2)`. The pre-explosion count is
therefore `C*M`, with the base and multiplier draws independent and ordered. Explosion decay then
tests each unit independently. Silk bypasses the Raw-Copper count and Fortune draws. Wrong-tool
breaks emit neither table output nor experience.

The named sequences are `minecraft:blocks/copper_ore` and
`minecraft:blocks/deepslate_copper_ore`. Correct-tool admission, Silk handling, explosion context,
block removal and item-entity placement remain with `BLK-BREAK-HOOK-001` and `ITM-LOOT-001`.

### Entity acquisition

The second pool of `entities/drowned` runs after Rotten Flesh and requires `killed_by_player`.
After that gate it draws `random_chance_with_enchanted_bonus`. With no positive living-attacker
Looting level it succeeds when one float is below `0.11`; at level `L>0` the threshold is
`0.13+0.02*(L-1)`, equivalently `0.11+0.02L`. Normal levels `1/2/3` therefore yield
`0.13/0.15/0.17`; the threshold is not clamped, so arbitrary level `45` or higher admits every
`[0,1)` draw. Success emits one default Copper Ingot. The full table uses named sequence
`minecraft:entities/drowned`.

The sole pool of `entities/copper_golem` has no player-kill condition. It creates a Copper Ingot,
replaces count with inclusive uniform `B in 1..3`, then, for a living attacking entity with
Looting `L>0`, spends a fresh float `U` and adds `round(L*U)`. Absent/nonliving attacker or level
zero adds nothing and spends no bonus draw. The table uses named sequence
`minecraft:entities/copper_golem`.

Generic death admission, mob-drop gamerule handling, Looting context, table invocation and
world-drop placement remain with the entity and loot owners. Raw Copper and Copper Nugget occur
in no entity table.

### Generation join and absence boundary

Ordinary biome decoration uses one of two Copper feature pairs:

- `ore_copper_small` has vein size `10`, ordered Stone/Deepslate replacement targets and air
  exposure discard `0`; placed `ore_copper` tries count `16`, in-square positions and trapezoid
  absolute height `-16..112` in exactly `54` locked Overworld biomes;
- `ore_copper_large` has the same targets and discard but size `20`; its placed record has the
  same count and height and occurs only in Dripstone Caves.

The `55` locked Overworld biomes therefore each schedule exactly one of these two placed Copper
features; Dripstone Caves substitutes the large record rather than adding it beside the small one.
Ore geometry, replacement reads, failed writes, feature order and biome scheduling remain
`WGEN-PIPELINE-001`.

Noise filling independently joins the existing Copper ore-vein resolver at inclusive Y `0..50`.
After its density/noise/admission gates it can write Copper Ore, Raw Copper Block or Granite; the
strict raw-block decision is the third position-seeded float below `0.02` after prior admission.
Breaking an emitted Copper Ore through the non-Silk path reaches Raw Copper above, while crafting
an emitted Raw Copper Block reaches nine Raw Copper through `BLK-RAW-STORAGE-001`. Generation does
not write loose item stacks.

Exhaustive direct data and server-class reference scans find no chest, archaeology, fishing,
villager-trade, barter, brewing, composting, fuel or dispenser branch for any of the three loose
items. An exhaustive decoded-string scan of all `1,212` locked structure NBT files finds no stored
`copper_ingot`, `copper_nugget` or `raw_copper` item identity. Fixed Copper-colored blocks or gear
elsewhere do not create one of these stacks without a separately specified loot/recipe path.

**Persistence and reload boundary:**

The three stacks persist identity, count and arbitrary valid component patches. They do not own
recipe progress, stored XP, recipe knowledge, anvil or Smithing-Table preview, loot cursor,
entity-death state or worldgen state. Those values persist with their respective owners.

Recipe reload changes future cooking, crafting and smithing matches. Tag reload changes future
Copper-tool crafting, Name-Tag crafting, repair and trim-recipe admission because the named holder
sets remain live; it does not change exact-item recipe positions. Advancement reload changes
future listeners and rewards. Loot reload changes future ore and entity evaluation. Trim-material
registry reload changes future material decode/tooltip/asset selection. Existing stacks, completed
crafts, damage repairs, trims, deaths and generated chunks are not replayed or rewritten. Resource
reload independently changes names, item models, textures and trim palettes.

**Wire and client projection:**

Ordinary item-stack codecs publish raw registry IDs `933`, `934` and `1336`, count and component
patches. No family-specific packet exists. Recipes, tags, trim holders and advancement/loot IDs
use their generic registry and synchronization owners; server authority remains decisive.

The Ingredients tab orders the family in this surrounding sequence:
`Coal, Charcoal, Raw Copper, Raw Iron, Raw Gold, Emerald, Lapis Lazuli, Diamond, Ancient Debris,
Quartz, Amethyst Shard, Copper Nugget, Iron Nugget, Gold Nugget, Copper Ingot, Iron Ingot,
Gold Ingot`. Each family item appears exactly once there and in no second ordinary tab.

The English names are `Raw Copper`, `Copper Ingot` and `Copper Nugget`. Each item definition
selects its like-named `item/generated` model with a single like-named item texture. GUI/ground/
fixed/hand transforms, glint, count text and arbitrary component-driven tooltip additions remain
generic. Copper trim uses the material description and `copper`/`copper_darker` palettes described
above rather than the loose Ingot texture.

**Branches and aborts:**

Three identities and four direct tags; default versus patched/removed trim component; exact versus
live-tag recipe input; eight cooking records and machine mismatch; 28 crafting grids; 18 trim
templates; repairable tools/armor versus nonrepairable horse/nautilus armor; wrong/correct tool,
Silk/non-Silk, Fortune and explosion ore paths; Drowned player/chance gates; Copper-Golem base and
optional Looting; small versus large biome ore and noise-vein endpoints; persistence/reload and
wire/client projections are distinct.

**Constants and randomness:**

Raw IDs `933/934/1336`; stack `64`; Copper tool material `190/5.0/1.0/13`; Copper armor material
`11/8`; compacting `9:1`; Bars `16`, Doors `3`, Torches `4`; cooking `200/100`, Ingot XP `0.7`,
Nugget XP `0.1`; ore states `27790/27791`, base count `2..5`, Fortune multiplier above and break
XP `0`; Drowned chance `0.11+0.02L` for positive `L`; Copper-Golem base `1..3` plus
`round(L*U)`; ore sizes `10/20`, count `16`, trapezoid `-16..112`; ore-vein band `0..50` and raw
block gate `<0.02`; trim color `#B4684D`.

**Side effects:**

Default item outputs; cooking progress and recipe XP; recipe knowledge; crafting consumption;
anvil repair; smithing trim replacement; ore/entity world drops; generated Copper Ore/Raw Copper
Block terrain; ordinary inventory persistence, synchronization, model rendering and trim
projection.

**Gates:**

Selected identity/component patch; live recipe/tag/advancement/loot/trim snapshots; crafting shape
and offset; machine recipe type and capacity; target repairable holder set; Smithing template/base/
addition and material component; correct tool, Silk, Fortune and explosion context; player-kill
and living-attacker Looting context; biome/feature/ore-vein admission; client resource snapshot.

**State read/written:**

Reads stack identity/components/tags, cooking machine and recipe state, crafting grid and
knowledge, target damage/repairability, Smithing roles and existing trim, block/tool/enchantment/
explosion context, death/attacker/Looting context, feature/biome/ore-vein state and client
resources. Writes only the processing, result, knowledge, repair, trim, loot, generated-terrain,
stack, wire and projection state listed above.

**Failure behavior:**

Wrong machines, invalid grids, missing/replaced recipes, full results, nonmembers and invalid or
unchanged trim previews produce no commit. Rejected repair materials do not consume levels or
items. Wrong-tool ore breaks emit no loot; Silk bypasses Raw Copper; failed explosion-decay units
vanish. Non-player Drowned kills and failed chance draws emit no Ingot; failed generation gates
write nothing. Reload changes future evaluation only, and missing client resources cannot grant
server behavior.

**Boundary cases and quirks:**

Copper Ingot's three independent roles can diverge under reload or patching: item tags admit
crafting/repair/smithing, while `provides_trim_material` selects the actual trim holder only after
smithing recipe match. Copper Nugget can craft Name Tag because `metal_nuggets` has three metals,
but only Copper Nugget participates in Copper Chain/Torch/Ingot recipes. Horse and Nautilus armor
can be destroyed into a Nugget yet cannot be repaired with an Ingot. Waxed Copper Block reverses
directly to nine Ingots, while other oxidation stages do not. Copper Ore yields no break XP despite
its `DropExperienceBlock` class. Dripstone Caves substitute size `20` for size `10`; they do not
receive both placed records. No structure-template string or unrelated copper block broadens loose
item acquisition.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-REPORT-001`; `OFF-DATA-001`;
`net.minecraft.world.item.Items`;
`net.minecraft.world.item.ToolMaterial`;
`net.minecraft.world.item.ToolMaterial#applyCommonProperties`;
`net.minecraft.world.item.equipment.ArmorMaterials`;
`net.minecraft.world.item.Item$Properties#repairable(net.minecraft.tags.TagKey)`;
`net.minecraft.world.item.crafting.SmithingTrimRecipe#assemble`;
`net.minecraft.world.item.crafting.SmithingTrimRecipe#applyTrim`;
`net.minecraft.world.item.CreativeModeTabs`;
`net.minecraft.world.level.levelgen.OreVeinifier`;
`reports/registries.json#minecraft:{item,recipe,recipe_serializer,loot_table,advancement,trim_material,worldgen}`;
`reports/blocks.json#minecraft:{copper_ore,deepslate_copper_ore}`;
`reports/minecraft/components/item/{raw_copper,copper_ingot,copper_nugget,copper_pickaxe,copper_helmet,copper_horse_armor,copper_nautilus_armor}.json`;
`data/minecraft/tags/item/{copper_tool_materials,repairs_copper_armor,metal_nuggets,trim_materials}.json`;
`data/minecraft/trim_material/copper.json`;
`data/minecraft/recipe/{brush,copper_bars,copper_block,copper_boots,copper_chain,copper_chest,copper_chestplate,copper_door,copper_helmet,copper_ingot,copper_ingot_from_*,copper_lantern,copper_leggings,copper_nugget,copper_nugget_from_*,copper_torch,copper_trapdoor,lightning_rod,spyglass,raw_copper,raw_copper_block,name_tag,copper_axe,copper_hoe,copper_pickaxe,copper_shovel,copper_spear,copper_sword,*_armor_trim_smithing_template_smithing_trim}.json`;
`data/minecraft/advancement/recipes/{building_blocks,combat,decorations,misc,redstone,tools}/*.json`;
`data/minecraft/loot_table/blocks/{copper_ore,deepslate_copper_ore,raw_copper_block}.json`;
`data/minecraft/loot_table/entities/{drowned,copper_golem}.json`;
`data/minecraft/tags/block/{mineable/pickaxe,needs_stone_tool}.json`;
`data/minecraft/worldgen/configured_feature/{ore_copper_small,ore_copper_large}.json`;
`data/minecraft/worldgen/placed_feature/{ore_copper,ore_copper_large}.json`;
`data/minecraft/worldgen/biome/*.json`;
`data/minecraft/structure/**/*.nbt`;
`assets/minecraft/items/{raw_copper,copper_ingot,copper_nugget}.json`;
`assets/minecraft/models/item/{raw_copper,copper_ingot,copper_nugget}.json`;
`assets/minecraft/textures/item/{raw_copper,copper_ingot,copper_nugget}.png`;
`assets/minecraft/equipment/copper.json`;
`assets/minecraft/textures/trims/color_palettes/{copper,copper_darker}.png`;
`EXP-ITM-077`.

**Test vectors:**

Run `EXP-ITM-077` with default, ordinary-patched, trim-component-removed and alternate-trim-holder
stacks through all four tag baselines. Exercise every admitted/rejected target across six Copper
tools, four humanoid armor pieces and the deliberately nonrepairable Horse/Nautilus armor. Match,
complete and extract all eight cooking records; cover all 28 crafting records at every grid/offset/
mirror and near-miss boundary; then run all eighteen trim templates across absent, new, identical
and replacement material states and every unlock route before/after reload.

Break both ores through wrong/correct tools, Silk, Fortune levels, explosion radii and zero-XP
assertions. Evaluate Drowned player attribution and every chance threshold plus Copper-Golem
`1..3` and optional Looting growth with controlled named cursors. Run size-10 placement across the
54 admitted biomes, size-20 placement in Dripstone Caves, Copper ore-vein result boundaries and
the full 1,212-template decoded-string census. Persist and synchronize every stack variant; assert
raw IDs `933/934/1336`, Ingredients order, like-named generated models and Copper versus
Copper-darker trim projection.

**Limits:**

Generic stack/use, cooking timers/XP, crafting, advancement listeners, anvil pricing/commit,
Smithing-Table slot/preview/commit behavior, block/entity loot, ore geometry, noise filling,
packet encoding and client rendering remain with `ITM-001`, `ITM-FURNACE-001`,
`ITM-RECIPE-001`, `ITM-ADVANCEMENT-001`, `ITM-ANVIL-001`, `ITM-SMITHING-001`,
`ITM-LOOT-001`, `BLK-BREAK-HOOK-001`, `BLK-RAW-STORAGE-001`,
`WGEN-PIPELINE-001`, the generic play item/container protocol families and `CLI-006`.
