# Items, inventories and progression mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ITM-LAPIS-LAZULI-001` — Lapis Lazuli joins ore, loot and trade acquisition to enchanting, dye, compacting and armor trims

**Parent:** `SIM-004`, `SIM-005`, `SIM-RANDOM-001`, `BLK-001`, `BLK-002`,
`BLK-003`, `BLK-005`, `BLK-BREAK-001`, `BLK-BREAK-HOOK-001`, `PLY-002`,
`PLY-005`, `PLY-006`, `PLY-INPUT-001`, `PLY-INTERACT-001`, `PLY-BREAK-001`,
`ITM-001`, `ITM-002`, `ITM-003`, `ITM-004`, `ITM-005`, `ITM-006`, `ITM-007`,
`ITM-USE-001`, `ITM-CONTAINER-001`, `ITM-RECIPE-001`,
`ITM-RECIPE-SERIALIZER-001`, `ITM-CRAFT-001`, `ITM-FURNACE-001`,
`ITM-LOOT-001`, `ITM-ADVANCEMENT-001`, `ITM-ENCHANT-001`,
`ITM-SMITHING-001`, `ITM-SMITHING-TEMPLATE-001`, `ITM-ANVIL-001`,
`BLK-LAPIS-BLOCK-001`, `ENT-001`, `MOB-001`, `MOB-004`, `MOB-AI-001`,
`ENV-001`, `ENV-002`, `ENV-003`, `WGEN-PIPELINE-001`,
`WGEN-STRUCTURE-MINESHAFT-001`, `WGEN-STRUCTURE-SHIPWRECK-001`,
`WGEN-JIGSAW-VILLAGES-001`, `CLI-001`, `CLI-006`, `CLI-UI-001`,
`CLI-EFFECT-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked registration/components, both ore blocks and loot tables, all cooking,
crafting, advancement, trade, trim and worldgen records, the exact enchantment-menu consumer, all
`1,212` decoded templates and client resources determine every Lapis-Lazuli-specific branch.
Generic breaking, processing, crafting, loot, enchanting, Smithing, merchant, worldgen,
persistence, packet and rendering algorithms retain their cited owners.

**Applies when:**

`minecraft:lapis_lazuli` is mined, cooked, looted, gifted, traded, compacted, converted to dye,
placed into an enchantment menu, used as armor-trim material, moved, renamed, persisted,
synchronized or rendered before and after data or resource reload.

**Authoritative state:**

Lapis Lazuli is raw item ID `928`, a common nondamageable plain `Item` with maximum stack `64`.
Its ordinary components include `provides_trim_material=minecraft:lapis`. Its sole direct item tag
is `trim_materials`. It has no food, consumable, remainder, fuel, compost, equipment, durability,
projectile, cooldown, inventory-tick or identity-specific use branch.

The coupled source blocks are property-free `DropExperienceBlock` instances:

| Block | block/item/state IDs | map/sound | hardness/resistance |
|---|---:|---|---:|
| Lapis Lazuli Ore | `102/103/563` | `STONE` / Stone | `3/3` |
| Deepslate Lapis Lazuli Ore | `103/104/564` | `DEEPSLATE` / Deepslate | `4.5/3` |

Both use `BASEDRUM`, require a correct tool, and are direct `lapis_ores`,
`mineable/pickaxe` and `needs_stone_tool` members. Full-cube physics and the generic experience
hook remain with the block owners.

**Transition and ordering:**

### Ore break, cooking and generation

Each ore table has one ordered alternative. Silk Touch level at least one emits one default
matching Ore block. Otherwise the table draws an integer base count `4..9`, applies `ore_drops`
Fortune multiplication and then explosion decay. A correct non-Silk player break independently
draws integer XP `2..5`; Fortune does not change that XP. A tool that is not both pickaxe-effective
and admitted for the live stone-tier requirement yields neither ordinary loot nor break XP. Silk
suppresses the ordinary XP. Named sequences are
`minecraft:blocks/{lapis_ore,deepslate_lapis_ore}`.

Four exact cooking records emit one default Lapis Lazuli and recipe XP `0.2`. Furnace accepts each
Ore in a separate record at omitted/default time `200`; Blast Furnace accepts each at
omitted/default time `100`. Smoker and Campfire reject both. Every record has its own advancement,
unlocked by possession of its exact Ore or prior knowledge. Input patches are not copied.

Two configured/placed pairs run in all `55` locked Overworld biomes:

| Placed ID | configured size / air-discard | first modifier | height |
|---|---:|---|---|
| `ore_lapis` | `7 / 0` | count `2` | trapezoid absolute `-32..32` |
| `ore_lapis_buried` | `7 / 1` | count `4` | uniform above-bottom `0` through absolute `64` |

Each wrapper then applies in-square and biome. Configured targets are ordered live
`stone_ore_replaceables` to state `563`, then `deepslate_ore_replaceables` to state `564`.
Feature geometry, exposure checks, replacement and chunk writes remain `WGEN-PIPELINE-001`.

### Direct loose-item acquisition

Every listed pool can select Lapis Lazuli repeatedly when it has multiple rolls:

| Table / pool | rolls | Lapis weight / pool total | count |
|---|---:|---:|---:|
| chests/abandoned_mineshaft `1` | `2..4` | `5/98` | `4..9` |
| chests/shipwreck_treasure `1` | `2..5` | `20/80` | `1..10` |
| chests/village/village_temple `0` | `3..8` | `1/19` | `1..4` |
| gameplay/hero_of_the_village/cleric_gift `0` | `1` | `1/2` | `1` |

Trade Rebalance replaces the Abandoned-Mineshaft table but preserves this pool, row, denominator,
rolls and count exactly; its extra enchanted-book pool is independent. No entity death, fishing,
barter, archaeology, Wandering-Trader or other bundled loot table directly emits loose Lapis
Lazuli.

An exhaustive decoded scan finds zero loose Lapis-Lazuli and zero Lapis-Ore cells across all
`1,212` templates. The one live Lapis Block cell belongs to `BLK-LAPIS-BLOCK-001`; container
sources remain loot-table driven.

### Twenty-five recipe joins and progression

Lapis Lazuli participates in `25` recipes:

- shaped Lapis Block compression consumes a full `3x3` grid of nine Lapis Lazuli;
- shapeless Blue Dye consumes one Lapis Lazuli and emits one Blue Dye;
- shapeless Lapis Block decompression emits nine Lapis Lazuli;
- the four Ore-cooking records above each emit one; and
- all `18` generic armor-trim Smithing records admit one Lapis Lazuli in the addition slot.

All `25` records have advancements. Direct Lapis-Lazuli possession can satisfy the inventory
alternative for Lapis Block and Blue Dye (`2` direct unlocks); decompression uses the Block,
cooking uses the exact Ore and trim listeners use their exact template. Grid placement, tag
expansion, machine/result capacity, atomic consumption, Smithing copying and knowledge publication
remain generic.

### Enchantment-menu, merchant and armor-trim sinks

Enchantment-menu reagent slot `1` accepts only exact `Items.LAPIS_LAZULI`; the quick-move branch
uses the same identity check. For option index `b=0,1,2`, a noncreative commit requires and consumes
exactly `b+1` Lapis Lazuli after revalidating the input, positive displayed cost and required
experience level. Creative mode bypasses both Lapis sufficiency and consumption. A failed
validation consumes nothing. Offer calculation, enchantment selection, book transmutation,
experience deduction, stat/criterion, seed refresh and sound remain `ITM-ENCHANT-001`.

Baseline Cleric level two selects both of its two candidates, so the exact one-Emerald to one-Lapis
offer is guaranteed. It has maximum uses `12`, Villager XP `5` and reputation discount `0.05`.
Trade Rebalance does not replace this record or set. Generic cost adjustment, transaction,
exhaustion and restock remain merchant-owned.

The default provider resolves trim material `minecraft:lapis`, description color `#416E97` and
asset `lapis`. As a live `trim_materials` member Lapis Lazuli fills the addition slot of all `18`
generic trim recipes, is consumed once and writes the Lapis holder into copied armor. Removing the
tag rejects it; removing or replacing the provider changes material resolution independently
after recipe admission.

**Persistence and reload boundary:**

Stacks persist identity, count and patches. Ores, machines, knowledge, enchantment menus,
containers, offers and trimmed equipment persist with their owners. Recipe, advancement, loot,
tag, trade, trim and worldgen reload changes only future evaluation; completed mining, processing,
loot, gifts, crafts, enchantments, trades, trims and generated chunks are not replayed or
rewritten. Existing merchant offers retain constructed costs/results. Resource reload independently
changes projection only.

**Wire and client projection:**

Generic stack publication uses item ID `928`; Ore block/item/state IDs are `102/103/563` and
`103/104/564`. No Lapis-Lazuli-specific packet exists. English names are `Lapis Lazuli`,
`Lapis Lazuli Ore`, `Deepslate Lapis Lazuli Ore` and `Lapis Material`.

Loose Lapis Lazuli selects one untinted `item/generated` flat. Both Ores select property-free cube
block models and direct block-item models with their same-named textures. The enchantment screen
has the exact Lapis-Lazuli reagent-slot sprite and singular/plural count strings. Ingredients orders
Emerald, Lapis Lazuli, Diamond; Natural Blocks orders the Emerald-Ore pair, then the Lapis-Ore pair,
then the Diamond-Ore pair.

Trim projection has one `lapis` color palette, `29` compatible armor item-model overlays and
atlas-driven equipped trim. There is no conditional loose-item model, tint, animation or special
renderer.

**Branches and aborts:**

Default/patched stack; two Ore/Silk/Fortune/explosion/XP paths; four cooking and two generation
paths; four direct loot/gift rows and one overlay; 25 recipes/listeners plus two direct unlocks;
three exact enchantment costs and creative bypass; guaranteed Cleric offer; trim tag/provider;
zero loose/Ore template identities and one separately owned Block cell; persistence/reload/wire/
client paths are distinct.

**Constants and randomness:**

Item ID `928`; Ore block/item/state `102/103/563`, `103/104/564`; stack `64`; Ore strength
`3/3`, `4.5/3`, drop `4..9`, XP `2..5`; cooking `200/100/0.2`; configured size/discard `7/0`,
`7/1`; placement `2/4`; direct rows `4`; recipes/listeners/direct unlocks `25/25/2`;
enchantment consumption `1/2/3`; templates/loose-or-Ore matches `1212/0`; Cleric inclusion `1`;
trim `#416E97`, recipes/models `18/29`.

**Side effects:**

Ore loot/XP and worldgen state; machine results/XP; four loot/gift outputs; crafted Block, Dye and
knowledge; enchantment-menu reagent consumption and generic enchantment commit; merchant
input/output; trimmed armor; durable stack/container state, synchronization and exact client
projection.

**Gates:**

Correct live tool/Silk/Fortune/explosion; cooking machine/input/capacity; placement/biome/exposure;
loot selection; exact grid/tag/result capacity and knowledge; enchantment exact identity,
option/resources/creative state; profession/level/trade set; trim tag/provider; registry/stack/
equipment decode and client resources.

**Boundary cases and quirks:**

Silk replaces both the `4..9` loose output and ordinary `2..5` break XP; Fortune changes only loose
count. Buried Lapis rejects every air-exposed candidate while the ordinary feature discards none.
The storage Block is not a reagent or trim substitute because both consumers require the loose
identity. Enchanting consumes option index plus one Lapis, not the displayed experience cost, and
creative consumes none. The Cleric sale is guaranteed because level two selects both candidates.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-REPORT-001`; `OFF-DATA-001`;
`net.minecraft.world.item.Items`; `net.minecraft.world.level.block.Blocks`;
`net.minecraft.world.level.block.DropExperienceBlock#spawnAfterBreak`;
`net.minecraft.world.inventory.EnchantmentMenu$3#mayPlace(net.minecraft.world.item.ItemStack)`;
`net.minecraft.world.inventory.EnchantmentMenu#quickMoveStack(net.minecraft.world.entity.player.Player,int)`;
`net.minecraft.world.inventory.EnchantmentMenu#clickMenuButton`;
`net.minecraft.world.item.crafting.SmithingTrimRecipe`;
`net.minecraft.world.entity.npc.villager.AbstractVillager#addOffersFromTradeSet`;
`net.minecraft.world.item.trading.VillagerTrade#getOffer`;
`net.minecraft.world.item.CreativeModeTabs`;
`reports/blocks.json#minecraft:{lapis_ore,deepslate_lapis_ore}`;
`reports/registries.json#minecraft:{block,item,recipe,recipe_serializer,loot_table,advancement,villager_trade,trade_set,trim_material,worldgen}`;
`reports/minecraft/components/item/lapis_lazuli.json`;
`data/minecraft/tags/{block/{lapis_ores,mineable/pickaxe,needs_stone_tool},item/trim_materials}.json`;
`data/minecraft/trim_material/lapis.json`;
`data/minecraft/loot_table/{blocks/{lapis_ore,deepslate_lapis_ore},chests/{abandoned_mineshaft,shipwreck_treasure,village/village_temple},gameplay/hero_of_the_village/cleric_gift}.json`;
`data/minecraft/recipe/{lapis_block,lapis_lazuli,blue_dye,lapis_lazuli_from_*,*_armor_trim_smithing_template_smithing_trim}.json`;
`data/minecraft/advancement/recipes/**/*.json`;
`data/minecraft/{villager_trade/cleric/2/emerald_lapis_lazuli,tags/villager_trade/cleric/level_2,trade_set/cleric/level_2}.json`;
`data/minecraft/datapacks/trade_rebalance/data/minecraft/loot_table/chests/abandoned_mineshaft.json`;
`data/minecraft/worldgen/{configured_feature/ore_lapis*,placed_feature/ore_lapis*,biome/*.json}`;
`data/minecraft/structure/**/*.nbt`;
`assets/minecraft/{items,models/item,textures/item}/lapis_lazuli.*`;
`assets/minecraft/{blockstates,models/block,textures/block}/*lapis_ore*`;
`assets/minecraft/{atlases,models/item,textures/trims}/**/*lapis*`;
`BLK-LAPIS-BLOCK-001`; `ITM-ENCHANT-001`; `ITM-SMITHING-TEMPLATE-001`;
`ITM-RECIPE-SERIALIZER-001`; `WGEN-PIPELINE-001`; `EXP-ITM-090`.

**Test vectors:**

Run `EXP-ITM-090` across default/patched Lapis Lazuli, both Ore
tool/Silk/Fortune/explosion/XP paths, four cooking and two generation paths, every direct
loot/gift row under baseline/Trade Rebalance, all 25 recipes/listeners, exact enchantment slot/
quick-move and option/creative commits, Cleric set and all 18 trims under independent tag/provider
reload. Scan every template, persist/reload/synchronize all owners and assert IDs, names,
ore/generated models, reagent-slot projection, palette/overlays and both tab orders.

**Limits:**

Generic breaking/XP, processing, crafting, Smithing, loot, enchanting, merchant, feature, packet
and renderer control flow remains with cited owners. Lapis Block behavior remains
`BLK-LAPIS-BLOCK-001`; Blue Dye, enchantments, templates and trimmed equipment retain their
dedicated owners. This leaf fixes the exact loose item, Ore source, sink joins, absences and
projection.
