# Blocks mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `BLK-SAPLING-001` — Ordinary tree saplings stage once, then run the exact small-or-mega tree transaction

**Parent:** `SIM-004`, `SIM-005`, `BLK-001`, `BLK-002`, `BLK-003`, `BLK-005`, `PLY-005`,
`PLY-006`, `ITM-003`, `ITM-004`, `ITM-006`, `ENV-001`, `ENV-002`, `ENV-003`, `WGEN-002`,
`WGEN-003`, `WGEN-004`, `CLI-001`, `CLI-006`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked registrations, `SaplingBlock`/`VegetationBlock`/`TreeGrower` and bone
meal control flow, complete tag/configured-feature/loot/trade/worldgen data, an exhaustive scan of
all 1,212 structure templates and exact client assets close the eight ordinary tree saplings.
`bamboo_sapling` and `mangrove_propagule` use different block implementations and are excluded.

**Applies when:**

Any `minecraft:{oak,spruce,birch,jungle,acacia,cherry,dark_oak,pale_oak}_sapling` is placed,
updated, randomly ticked, bone-mealed, grown, broken, exploded, composted, burned as fuel, selected
by loot or trade, tested by worldgen, generated from a template, persisted or rendered.

**Authoritative state:**

All eight registrations are `SaplingBlock` instances, report type `minecraft:sapling`, have no
block entity and expose only integer property `stage=0|1`. Stage zero is the default.

| Species | Block ID | Item ID | Stage-zero state | Stage-one state | Map color | Primary small-tree base height |
|---|---:|---:|---:|---:|---|---:|
| oak | 25 | 76 | 29 | 30 | `PLANT` | 4 |
| spruce | 26 | 77 | 31 | 32 | `PLANT` | 5 |
| birch | 27 | 78 | 33 | 34 | `PLANT` | 5 |
| jungle | 28 | 79 | 35 | 36 | `PLANT` | 4 |
| acacia | 29 | 80 | 37 | 38 | `PLANT` | 5 |
| cherry | 30 | 81 | 39 | 40 | `COLOR_PINK` | 7 |
| dark oak | 31 | 82 | 41 | 42 | `PLANT` | 0: no small-tree key |
| pale oak | 32 | 83 | 43 | 44 | `METAL` | 0: no small-tree key |

Every registration uses `noCollision`, random ticks, instant break and piston reaction `DESTROY`.
The fixed selection shape is `(2,0,2)..(14,12,14)` in sixteenths; collision and occlusion shapes
are empty. Thus every state has hardness/resistance `0/0`, emission and light dampening `0`,
skylight propagation, no sturdy face or redstone conduction, shade brightness `1`, friction
`0.6`, speed/jump factors `1`, no comparator output and AIR pathfindability. The block is not
waterloggable and contributes no fluid state.

Seven species use Grass sounds at volume/pitch `1/1`, with registry IDs break/step/place/hit/fall
`755/759/758/757/756`. Cherry alone uses Cherry Sapling sounds, also `1/1`, with IDs
`322/326/325/324/323`. Every standard block item is common, stacks to 64 and carries no
nonstandard default component.

**Transition and ordering:**

#### Support, placement and neighbor updates

`VegetationBlock#canSurvive` reads only the block immediately below and accepts it exactly when it
belongs to reloadable `supports_vegetation`. The locked closure has 11 identities:
`dirt`, `coarse_dirt`, `rooted_dirt`, `mud`, `muddy_mangrove_roots`, `moss_block`,
`pale_moss_block`, `grass_block`, `podzol`, `mycelium` and `farmland`. Stage, light, air above,
fluid, biome and horizontal neighbors add no placement predicate.

Every shape update, irrespective of the supplied direction, rechecks that support. Failure returns
ordinary air immediately; success delegates to the inert base implementation. The shared neighbor
update pipeline then owns update-or-destroy flags and conditional drops. Reloading the support tag
does not proactively revisit existing saplings; the next survival check observes the new snapshot.

#### Random-tick staging

An admitted random callback first reads maximum local raw brightness at `pos.above()`. A value
below `9` returns without RNG. Otherwise it consumes exactly `nextInt(7)` and advances only on
zero. Stage zero cycles to stage one through `ServerLevel.setBlock(pos,stage1,260)` and ignores the
Boolean result. Stage one invokes the species `TreeGrower` with the callback random source and
ignores its Boolean result. Neither branch emits its own sound, particle or game event.

Chunk activity, random-tick speed and position selection remain with `SIM-RANDOM-001`; this leaf
begins after the callback is admitted.

#### Bone meal admission and consumption

The target callback rejects every non-server `LevelReader`. On a server it resolves only the
primary small-tree configured feature, reads its `TreeConfiguration.trunkPlacer.base_height`, and
tests `isInsideBuildHeight(pos.above(baseHeight))`. Missing, non-tree and absent primary keys fall
back to zero. It does not inspect stage, light, support, surrounding clearance, mega-tree height or
secondary/flower feature height. The locked primary values are the table's `4/5/5/4/5/7/0/0`.

For a valid target, `BoneMealItem` calls the success callback and always shrinks the stack by one
on the server. The callback ignores its supplied random parameter and consumes
`level.getRandom().nextFloat()`; only a value strictly below `0.45` advances. Miss and equality
still consume the item, emit `ITEM_INTERACT_FINISH`, project level event `1505` with data `15`,
and return server success. A hit calls `advanceTree` with the level random: stage zero merely
offers stage one, while stage one enters the growth transaction below.

#### Feature selection

The eight code-built growers select these configured-feature keys:

| Species | Secondary chance | Primary/secondary mega | Primary/secondary small | Flower-present primary/secondary |
|---|---:|---|---|---|
| oak | 0.1 | none | `oak` / `fancy_oak` | `oak_bees_005` / `fancy_oak_bees_005` |
| spruce | 0.5 | `mega_spruce` / `mega_pine` | `spruce` / none | none |
| birch | 0 | none | `birch` / none | `birch_bees_005` / none |
| jungle | 0 | `mega_jungle_tree` / none | `jungle_tree_no_vine` / none | none |
| acacia | 0 | none | `acacia` / none | none |
| cherry | 0 | none | `cherry` / none | `cherry_bees_005` / none |
| dark oak | 0 | `dark_oak` / none | none | none |
| pale oak | 0 | `pale_oak_bonemeal` / none | none | none |

Mega selection occurs first. It consumes one float only when a secondary mega key exists, so
spruce always draws: `<0.5` selects `mega_pine`, while equality or larger selects
`mega_spruce`. Jungle, dark oak and pale oak choose their sole mega key without RNG. A missing
mega key, missing registry holder or absent matching square continues to small selection.

Before every small selection, even for growers without a small key or flower variant, the code
searches the inclusive 5-by-3-by-5 volume X/Z `-2..2`, Y `-1..1` around the sapling. The locked
`betweenClosed` iterator advances X fastest, then Y, then Z, and stops at the first reloadable
`flowers` member. It therefore performs at most 75 block/tag reads.

Small selection then always consumes one float. A draw strictly below `secondaryChance` uses the
flower-sensitive secondary key when present, otherwise the ordinary secondary key when present;
if neither exists it falls through. The remaining choice uses the flower-sensitive primary key
when present, otherwise the ordinary primary or null. Consequently oak alone changes small-tree
shape at `<0.1`; spruce consumes an otherwise ineffectual second float after a failed mega search;
and every chance-zero species still consumes a small-selection float. Nearby flowers affect only
oak, birch and cherry, selecting the listed bee-bearing feature.

#### Mega-tree transaction

Spruce, jungle, dark oak and pale oak test four possible 2-by-2 origins relative to the triggering
sapling in exact X/Z order `(0,0)`, `(0,-1)`, `(-1,0)`, `(-1,-1)`. Each test compares only block
identity at its four cells; mixed stage values qualify.

For the first match, all four cells are offered ordinary air with flags `260` in order origin,
east, south, southeast, ignoring every result. The selected configured feature is invoked at the
candidate origin. Success returns true immediately. Failure restores the triggering caller's
original state—not each captured neighbor state—to all four cells in the same order and with the
same ignored flags, then returns false. Thus a failed mixed-stage square becomes four copies of the
triggering stage. It does not test another square or fall back to a small tree after a matched mega
feature fails.

Without a matching square, jungle continues to its small jungle tree; spruce continues to spruce.
Dark and pale oak perform the flower scan and one otherwise unused small-selection float, resolve
null and remain unchanged. Their ordinary growth therefore requires a 2-by-2 square.

#### Small-tree transaction

A null selected key or missing configured-feature holder returns false without clearing the
sapling. Otherwise the grower captures `level.getFluidState(pos).createLegacyBlock()`—ordinary air
for all eight nonwaterlogged states—offers it at the root with flags `260`, ignores the write
result and calls `ConfiguredFeature#place`.

Feature success returns true. If the root's resulting block-state object is still the captured
legacy-fluid state, the grower additionally calls
`sendBlockUpdated(pos,originalSapling,legacyFluid,2)`; a feature-written root skips that explicit
packet. Feature failure restores the original triggering state with flags `260`, ignores the
result and returns false. All actual root, trunk, foliage, decorator, clearance and provider
behavior belongs to the selected configured tree and `WGEN-PIPELINE-001`.

#### Loot, acquisition, trade, fuel and composting

Each sapling's own block table has one one-roll self-item pool behind `survives_explosion`, with
random sequence `minecraft:blocks/<sapling>`. Tool, Silk Touch, Fortune and stage add no branch.
Each corresponding leaves table can also emit one sapling on a non-Silk/non-shears leaf branch:
oak, spruce, birch, acacia, cherry, dark oak and pale oak use Fortune chance vector
`[0.05,0.0625,0.083333336,0.1]`; jungle uses
`[0.025,0.027777778,0.03125,0.041666668,0.1]`. Explosion survival is tested before those
table-bonus chances. The eight potted forms' tables emit their matching content separately from
the pot and remain owned by `BLK-FLOWER-POT-001`.

Four village chest tables directly emit saplings from a uniform `3..8`-roll pool:

- plains house: oak, weight `5`, count `1..2`;
- weaponsmith: oak, weight `5`, count `3..7`;
- taiga house: spruce, weight `5`, count `1..5`;
- savanna house: acacia, weight `10`, count `1..2`.

Their random sequences are their like-named `minecraft:chests/village/*` IDs. No bundled nonblock
loot table directly emits the other five species.

All eight wandering-trader records are members of the 76-candidate common tag. Each wants five
emeralds, gives one matching sapling, permits eight uses and sets reputation discount `0.05`; it
has no second cost, predicate or output modifier. The common set chooses five distinct candidates
with random sequence `minecraft:trade_set/wandering_trader/common`. No recipe or advancement
directly produces or consumes one of these eight items.

Every item is registered in the code-built composter table with chance `0.3`. Direct item
`saplings` membership also makes every one a vanilla fuel for `200/2 = 100` ticks. The matching
block tag has no production runtime consumer, while the item tag's only production class reader is
fuel construction. None appears in `FireBlock`'s odds table and no registration enables lava
ignition: fire odds are `0/0` and lava cannot ignite the block.

#### Worldgen and structures

Exactly 45 placed-feature records use a stage-zero sapling state solely in a reloadable
`would_survive` block predicate: oak `15`, spruce `8`, birch `10`, jungle `3`, acacia `2`, cherry
`3`, dark oak `2` and pale oak `2`. These records do not place the sapling; they reuse its current
support predicate to gate their referenced feature.

The four configured huge-fungus records `crimson_fungus`, `crimson_fungus_planted`,
`warped_fungus` and `warped_fungus_planted` explicitly include all eight identities in their
replaceable-block predicate. Their geometry may therefore replace an encountered sapling.

An exhaustive decode of all 1,212 bundled structure templates finds exactly 60 raw cells and two
identity/template pairs: `village/savanna/houses/savanna_library_1` has two acacia saplings at
stage one, and `woodland_mansion/1x2_a4` has 58 dark-oak saplings at stage zero. The other six
identities have no template cell. Feature/pool selection, processors, transforms, clipping and
write admission remain with `WGEN-PIPELINE-001`; raw cells are not unconditional placements.

**Client projection:**

Terrain and block updates publish states `29..44`. Every blockstate file has one unconditional
variant, so `stage` does not change the model. Each block model inherits `minecraft:block/cross`
with `cross=minecraft:block/<sapling>`. Each item selector chooses a like-named
`minecraft:item/generated` model whose layer zero is the block texture. There is no tint,
rotation, emissive layer or special renderer.

The Natural Blocks tab orders the items oak, spruce, birch, jungle, acacia, dark oak, then
mangrove propagule, cherry and pale oak; azalea and flowering azalea follow. Thus the eight owned
items are split once by the excluded mangrove identity.

**Branches and aborts:**

Eight species and two stages; 11-member support or failure; brightness `<9` versus `>=9` and
seven-way tick draw; bone-meal build-height and strict `0.45` draw; four mega capabilities,
optional spruce mega draw, four square origins and mixed stages; flower/no-flower, oak strict
`0.1` draw and null small key; missing holder, failed clear, feature success/failure and failed
restore; self/leaf/pot/chest/trade acquisition; compost/fuel/tag reload; placed-feature, fungus and
template paths; save/reload and every model/tab projection are distinct.

**Constants and randomness:**

Block IDs `25..32`; item IDs `76..83`; states `29..44`; shape
`(2,0,2)..(14,12,14)`; strength `0/0`; emission/dampening/friction/speed/jump
`0/0/0.6/1/1`; support count `11`; brightness threshold `9`; random growth `nextInt(7)==0`;
bone-meal success `<0.45`; minimum heights `4/5/5/4/5/7/0/0`; flower volume `75`; oak secondary
`<0.1`; spruce mega secondary `<0.5`; flags `260/2`; compost `0.3`; fuel `100`; trade
`5-for-1`, uses `8`, discount `0.05`; placed features `45`; template cells `60`.

**Side effects:**

Conditional placement and support-loss destruction; stage write; bone-meal shrink, vibration and
event; four-cell clear/restore or one-cell clear/restore; configured-feature world mutation;
self/leaf/pot/chest/trade acquisition; compost and fuel use; huge-fungus replacement; structure
writes; palette persistence; map, sound, tab and block/item model projection.

**Gates:**

Generic placement/reach/hand/build permissions; current support-tag snapshot; random-tick chunk,
activity and rate admission; brightness and RNG; bone-meal target/build height; configured-feature
registry and flower-tag snapshots; 2-by-2 identity layout; tree clearance/provider/write behavior;
loot context and explosion; trade/tag/composter/fuel snapshots; active worldgen/structure data;
valid save, registry, pack and client connection context.

**Boundary cases and quirks:**

Any directional shape update can remove an unsupported sapling. Brightness failure consumes no
growth RNG. Bone meal is consumed and its success effect is emitted even when the strict `0.45`
draw misses. Every small selection consumes a float, including chance-zero and null-key growers.
Only spruce draws before the mega square scan. Mixed-stage mega squares qualify, but failed
placement restores all four cells to the triggering state. A matched mega failure does not fall
back. Dark and pale oak cannot grow alone. All material writes in staging and grower cleanup
ignore their Boolean results.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-REPORT-001`; `OFF-DATA-001`;
`net.minecraft.world.level.block.Blocks`;
`net.minecraft.world.level.block.SaplingBlock#codec`;
`net.minecraft.world.level.block.SaplingBlock#randomTick`;
`net.minecraft.world.level.block.SaplingBlock#advanceTree`;
`net.minecraft.world.level.block.SaplingBlock#isValidBonemealTarget`;
`net.minecraft.world.level.block.SaplingBlock#isBonemealSuccess`;
`net.minecraft.world.level.block.SaplingBlock#performBonemeal`;
`net.minecraft.world.level.block.SaplingBlock#createBlockStateDefinition`;
`net.minecraft.world.level.block.VegetationBlock#mayPlaceOn`;
`net.minecraft.world.level.block.VegetationBlock#updateShape`;
`net.minecraft.world.level.block.VegetationBlock#canSurvive`;
`net.minecraft.world.level.block.VegetationBlock#propagatesSkylightDown`;
`net.minecraft.world.level.block.VegetationBlock#isPathfindable`;
`net.minecraft.world.level.block.grower.TreeGrower#getConfiguredFeature`;
`net.minecraft.world.level.block.grower.TreeGrower#getConfiguredMegaFeature`;
`net.minecraft.world.level.block.grower.TreeGrower#growTree`;
`net.minecraft.world.level.block.grower.TreeGrower#isTwoByTwoSapling`;
`net.minecraft.world.level.block.grower.TreeGrower#hasFlowers`;
`net.minecraft.world.level.block.grower.TreeGrower#getMinimumHeight`;
`net.minecraft.world.item.BoneMealItem#useOn`;
`net.minecraft.world.item.BoneMealItem#growCrop`;
`net.minecraft.core.BlockPos#betweenClosed`;
`net.minecraft.world.level.block.ComposterBlock#bootStrap`;
`net.minecraft.world.level.block.entity.FuelValues#vanillaBurnTimes`;
`net.minecraft.world.level.block.FireBlock#bootStrap`;
`net.minecraft.world.item.CreativeModeTabs#bootstrap`;
`reports/blocks.json#minecraft:{oak,spruce,birch,jungle,acacia,cherry,dark_oak,pale_oak}_sapling`;
`reports/registries.json#minecraft:{block,item,sound_event}`;
`reports/minecraft/components/item/{oak,spruce,birch,jungle,acacia,cherry,dark_oak,pale_oak}_sapling.json`;
`data/minecraft/tags/block/{saplings,supports_vegetation,substrate_overworld,dirt,mud,moss_blocks,grass_blocks,flowers}.json`;
`data/minecraft/tags/item/saplings.json`;
`data/minecraft/worldgen/configured_feature/{oak,spruce,birch,jungle_tree_no_vine,acacia,cherry,crimson_fungus,crimson_fungus_planted,warped_fungus,warped_fungus_planted}.json`;
`data/minecraft/worldgen/placed_feature/**/*.json`;
`data/minecraft/loot_table/blocks/*{sapling,leaves}.json`;
`data/minecraft/loot_table/chests/village/{village_plains_house,village_weaponsmith,village_taiga_house,village_savanna_house}.json`;
`data/minecraft/villager_trade/wandering_trader/emerald_*_sapling.json`;
`data/minecraft/{tags/villager_trade,trade_set}/wandering_trader/common.json`;
`data/minecraft/structure/**/*.nbt`;
`assets/minecraft/blockstates/*_sapling.json`;
`assets/minecraft/models/{block,item}/*_sapling.json`;
`assets/minecraft/items/*_sapling.json`.

**Test vectors:**

Run `EXP-BLK-074` across all 16 states and every support/light/stage/bone-meal boundary; all flower
populations and feature-selection float endpoints; each mega-square origin, mixed-stage square,
missing holder and failed clear/place/restore result; all loot/trade/compost/fuel/tag/worldgen
records and 1,212 templates; save/reload and every sound/map/tab/model projection. Assert exact
IDs, constants, read/draw/write/effect order and negative joins.

**Limits:**

Generic random-tick admission, placement/breaking, bone-meal item routing, configured tree feature
geometry, loot/trade execution, compost processing, furnace processing, worldgen placement,
structure placement, block/item protocol and rendering remain with `SIM-RANDOM-001`,
`BLK-PLACE-001`, `BLK-BREAK-001`, `ITM-USE-001`, `WGEN-PIPELINE-001`, `ITM-LOOT-001`,
`ITM-FURNACE-001`, `PROTO-PLAY-CLIENTBOUND-BLOCK-001` and `CLI-006`. Corresponding leaves, potted
forms, generated logs/foliage/decorators, bamboo sapling and mangrove propagule retain their own
owners or catalog status.
