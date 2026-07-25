# Items, inventories and progression mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ITM-QUARTZ-001` — Nether Quartz joins ore, cooking, barter and Bastion acquisition to crafting, Mason sale and armor trim

**Parent:** `SIM-004`, `SIM-005`, `SIM-RANDOM-001`, `BLK-001`, `BLK-002`,
`BLK-003`, `BLK-005`, `BLK-BREAK-001`, `BLK-BREAK-HOOK-001`, `PLY-002`,
`PLY-005`, `PLY-006`, `PLY-INPUT-001`, `PLY-INTERACT-001`, `PLY-BREAK-001`,
`ITM-001`, `ITM-002`, `ITM-003`, `ITM-004`, `ITM-005`, `ITM-006`, `ITM-007`,
`ITM-USE-001`, `ITM-CONTAINER-001`, `ITM-RECIPE-001`, `ITM-CRAFT-001`,
`ITM-FURNACE-001`, `ITM-SMITHING-001`, `ITM-LOOT-001`, `ITM-ADVANCEMENT-001`,
`ITM-ANVIL-001`, `BLK-QUARTZ-001`, `ENT-001`, `MOB-001`, `MOB-004`,
`MOB-AI-001`, `ENV-001`, `ENV-002`, `ENV-003`, `WGEN-PIPELINE-001`,
`WGEN-JIGSAW-BASTION-001`, `CLI-001`, `CLI-006`, `CLI-UI-001`,
`CLI-EFFECT-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked item components, ore report and registration, complete recipe/unlock,
loot, barter, trade, trim and worldgen records, all 1,212 templates and exact client resources
determine every Nether-Quartz identity branch. Generic block breaking, cooking, crafting,
Smithing, loot, Piglin, merchant, ore placement, stack and rendering algorithms retain the cited
owners.

**Applies when:**

`minecraft:quartz` is mined from Nether Quartz Ore, cooked, bartered, looted, crafted, traded,
used as armor-trim material, moved, renamed, persisted, synchronized or rendered before and after
loot, recipe, advancement, tag, trade, trim, worldgen or resource reload.

**Authoritative state:**

Nether Quartz is raw item ID `929`, a common nondamageable plain `Item` with maximum stack `64`.
Its ordinary components include `provides_trim_material=minecraft:quartz`; its sole direct item
tag is `trim_materials`. It has no food, consumable, remainder, fuel, compost, equipment,
durability, repair, projectile, cooldown, inventory-tick or identity-specific use branch.

The coupled source block is already a generic `break-experience-blocks` member: Nether Quartz Ore
has block/item IDs `476/108`, sole/default state `11312`, map color `NETHER`, `BASEDRUM`, Nether-Ore
sounds, hardness/resistance `3/3`, `requiresCorrectToolForDrops` and direct
`mineable/pickaxe` membership. No tier tag contains it, so any pickaxe is correct. Its full-cube
physics and generic XP hook remain with the block owners.

**Transition and ordering:**

### Ore mining and cooking

The ore table uses one ordered alternative. Silk Touch level at least one emits one default Nether
Quartz Ore item. Otherwise the base entry is one default Quartz, `ore_drops` Fortune multiplication
applies, and explosion decay runs last. A correct non-Silk player break also draws integer XP
`2..5`; Fortune does not change that XP. A nonpickaxe fails correct-tool admission and yields
neither ordinary loot nor break XP. Named sequence is `minecraft:blocks/nether_quartz_ore`.

Two exact cooking records accept Nether Quartz Ore and emit one default Quartz with recipe XP
`0.2`: Furnace `quartz` uses omitted/default time `200`, while Blast Furnace
`quartz_from_blasting` uses omitted/default blasting time `100`. Smoker and Campfire reject both.
Input component patches are ignored and not copied. Only the Furnace record has a recipe
advancement: exact Ore possession or prior `quartz` knowledge in one OR requirement grants that
record. No advancement grants the blasting record.

Configured ore `ore_quartz` has size `14`, discard-on-air-exposure chance `0`, and replaces exact
Netherrack with state `11312`. `ore_quartz_nether` makes `16` in-square attempts per chunk in
Nether Wastes, Soul-Sand Valley, Crimson Forest and Warped Forest; `ore_quartz_deltas` makes `32`
in Basalt Deltas. Both sample uniform height from `above_bottom:10` through `below_top:10`, then
apply the biome gate. Ore geometry, target iteration, exposure, write and scheduling order remain
`WGEN-PIPELINE-001`.

### Direct loose-item acquisition

Piglin bartering rolls the total-weight-`469` table once. Quartz has weight `20` (`20/469`) and,
when selected, receives inclusive uniform count `5..12`.

Bastion Treasure chest pool two has nine equal-weight entries and uniform `3..4` rolls. Quartz is
one entry (`1/9` per roll) and receives inclusive uniform count `8..23`; repeated rolls may select
it again. Bastion placement and marker/container handling remain
`WGEN-JIGSAW-BASTION-001`.

No other chest, archaeology, fishing, gift, entity-drop, villager-sale or wandering-trader table
directly emits Quartz. An exhaustive exact-UTF scan of all `1,212` structure templates finds zero
loose Quartz strings; the Bastion source is its reloadable chest table.

### Six crafting consumers and seven unlocks

Quartz participates in six ordinary crafting records:

| Output | Exact Quartz input and other pattern roles |
|---|---|
| Quartz Block | four Quartz in `2×2` → one |
| Diorite | checkerboard two Quartz plus two Cobblestone → two |
| Granite | shapeless one Quartz plus one Diorite → one |
| Comparator | one center Quartz, three Redstone Torches, three Stone → one |
| Daylight Detector | middle row three Quartz below three Glass and above three `#wooden_slabs` → one |
| Observer | one Quartz, two Redstone and six Cobblestone → one |

Each has its own advancement pairing exact Quartz possession with prior knowledge in one OR
requirement and grants only that recipe. Together with the Furnace-only unlock, Quartz has seven
direct recipe advancements. There is no Quartz-Block decompression recipe. Grid
offset/mirroring, tag expansion, output capacity, atomic consumption and recipe publication remain
generic; the output block/device behavior remains with its existing owner.

### Mason sink and armor trim

Level-four Mason has `33` tagged candidates and selects `2` distinct entries under random sequence
`minecraft:trade_set/mason/level_4`; therefore `quartz_emerald` appears with probability `2/33`.
The offer wants `12` matching default Quartz, gives one default Emerald, has maximum uses `12`,
grants `30` Villager XP and uses reputation discount `0.05`. No Trade-Rebalance replacement exists.

Quartz's default provider resolves trim material `minecraft:quartz`, whose asset name is `quartz`
and translated description color is `#E3D4C4`. As a live `trim_materials` member it fills the
addition slot of all `18` generic armor-trim Smithing recipes, is consumed once and writes the
Quartz material holder into the copied armor result. Removing the tag rejects it; removing or
replacing the provider changes material resolution after recipe admission. Template, armor,
existing-trim, preview, result-capacity and consumption behavior remain `ITM-SMITHING-001`.

**Persistence and reload boundary:**

Stacks persist identity, count and component patches. Machines, recipe knowledge, Piglin state,
containers, offers and trimmed equipment persist with their owners. Recipe, advancement, loot,
tag, trade, trim and worldgen reload changes only future evaluation; completed cooking, barter,
loot, crafts, offers, placed ore and existing trimmed armor are not replayed or rewritten.

**Wire and client projection:**

Generic stack publication uses item ID `929`; no Quartz-specific packet exists. English item name
is `Nether Quartz`; the trim description is `Quartz Material`. The item selects one untinted
same-named `item/generated` flat model and texture.

Ingredients orders Ancient Debris, Nether Quartz, Amethyst Shard. Quartz trim projection uses the
`quartz` palette for `29` compatible armor item-model overlays plus atlas-driven equipped trim.
There is no conditional loose-item model, tint, animation or special renderer.

**Branches and aborts:**

Default/patched stack; Silk versus Fortune/explosion ore loot and XP; Furnace versus Blast;
regular versus Delta ore schedule; barter and Bastion selection/count; six crafting matches and
seven unlocks; Mason candidate selection; live trim tag/provider and 18 Smithing records; zero
template strings; persistence/reload/wire/client paths are distinct.

**Constants and randomness:**

Quartz ID `929`; Ore block/item/state `476/108/11312`; stack `64`; ore `3/3`, XP `2..5`, configured
size/discard `14/0`; placement counts `16/32`; cooking time/XP `200/100/0.2`; barter
weight/total/count `20/469/5..12`; Bastion entries/rolls/count `9/3..4/8..23`; Mason
candidates/amount/chance `33/2/2/33`, offer `12:1`, uses/XP/discount `12/30/0.05`; trim
`#E3D4C4`, recipes/models `18/29`; templates/matches `1212/0`.

**Side effects:**

Ore loot/XP and worldgen state; machine result/XP and knowledge; Piglin/chest loot; six crafting
results; Mason offer; trimmed armor; stack/container persistence, synchronization and exact client
projection.

**Gates:**

Correct pickaxe/Silk/Fortune/explosion; cooking machine/input/capacity; ore placement modifiers;
loot-table selection; exact grid/tag/result capacity and knowledge; profession/level/trade-set
selection; live trim tag/provider; registry/stack/equipment decode and client resources.

**State read/written:**

Reads all gates above and writes only the loot, XP, ore, processing, knowledge, barter, container,
crafting, offer, trimmed-equipment, durable, wire and projection state listed above.

**Failure behavior:**

Wrong tool emits no ore loot or XP. Wrong machine/input/capacity commits no cook. Failed ore
placement writes nothing. Unselected loot/trade candidates emit no Quartz or offer. Wrong grid or
unavailable recipe emits no result. Missing trim tag/provider rejects or invalidates trim output.
Reload affects future evaluation only.

**Boundary cases and quirks:**

Any pickaxe is correct because the Ore has no tier tag. Silk replaces both Quartz and its ordinary
break XP; Fortune changes Quartz count but not XP. The Blast Furnace record works without its own
recipe advancement. Four Quartz compact irreversibly into one Quartz Block. Mason Quartz buying is
only one of 33 candidates, whereas the separate level-five Quartz-block sales remain
`BLK-QUARTZ-001`.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-REPORT-001`; `OFF-DATA-001`;
`net.minecraft.world.item.Items`; `net.minecraft.world.level.block.Blocks`;
`net.minecraft.world.level.block.DropExperienceBlock#spawnAfterBreak`;
`net.minecraft.world.item.CreativeModeTabs`;
`reports/blocks.json#minecraft:nether_quartz_ore`;
`reports/registries.json#minecraft:{block,item,recipe,loot_table,advancement,worldgen}`;
`reports/minecraft/components/item/{quartz,nether_quartz_ore}.json`;
`data/minecraft/loot_table/{blocks/nether_quartz_ore,chests/bastion_treasure,gameplay/piglin_bartering}.json`;
`data/minecraft/recipe/{quartz,quartz_from_blasting,quartz_block,diorite,granite,comparator,daylight_detector,observer}.json`;
`data/minecraft/advancement/recipes/{misc/quartz,building_blocks/{quartz_block,diorite,granite},redstone/{comparator,daylight_detector,observer}}.json`;
`data/minecraft/{trim_material/quartz,tags/item/trim_materials}.json`;
`data/minecraft/{villager_trade/mason/4/quartz_emerald,tags/villager_trade/mason/level_4,trade_set/mason/level_4}.json`;
`data/minecraft/worldgen/{configured_feature/ore_quartz,placed_feature/{ore_quartz_nether,ore_quartz_deltas},biome/{nether_wastes,soul_sand_valley,crimson_forest,warped_forest,basalt_deltas}}.json`;
`data/minecraft/structure/**/*.nbt`; `assets/minecraft/{items,models/item,textures/item}/quartz.*`;
`assets/minecraft/{atlases,models/item,textures/trims}/**/*quartz*`;
`BLK-QUARTZ-001`; `WGEN-PIPELINE-001`; `WGEN-JIGSAW-BASTION-001`; `EXP-ITM-082`.

**Test vectors:**

Run `EXP-ITM-082` across default/patched Quartz, all Ore tool/enchantment/explosion/XP endpoints,
both cooking records, both ore schedules in five biomes, controlled Piglin/Bastion/Mason
selection, six crafting records/seven unlocks and all 18 trim recipes under tag/provider reload.
Scan every template, persist/reload/synchronize all owners and assert IDs, name, generated flat,
trim palette/overlays and tab order.

**Limits:**

Generic breaking/XP, cooking, crafting, Smithing, loot, Piglin, merchant, ore-feature, packet and
renderer control flow remains with cited owners. Full Quartz blocks and derived shapes remain with
`BLK-QUARTZ-001` and their shape owner. This leaf fixes the exact loose item, acquisition,
consumer joins, trim material, absences and projection.
