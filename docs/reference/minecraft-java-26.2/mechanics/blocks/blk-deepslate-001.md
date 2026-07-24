# Blocks mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `BLK-DEEPSLATE-001` — Base deepslate joins axis-aware placement to terrain replacement, ore hosts and ancient-city structure

**Parent:** `BLK-001`, `BLK-002`, `BLK-003`, `BLK-004`, `BLK-005`, `PLY-005`, `PLY-006`,
`ITM-004`, `ITM-006`, `ENT-001`, `ENV-003`, `WGEN-003`, `CLI-006`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — the locked registration and inherited pillar implementation, reports, complete
loot/recipe/advancement/tag/worldgen data, direct class-reference sweep, all 1,212 decoded
structure templates and exact client assets exhaust base deepslate's state, transitions and
observable dispatch joins. Deepslate masonry and deepslate ore identities remain separate owners.

**Applies when:**

`minecraft:deepslate` is placed, transformed, mined, exploded, smelted, cut, selected as a
reloadable tag member, written or replaced during terrain/feature/retrogen/structure generation,
used as bat support, persisted or projected.

**Authoritative state:**

Deepslate is a `RotatedPillarBlock` with no block entity and one `axis` property:

| Axis | State | Default |
|---|---:|---|
| `x` | `30416` | no |
| `y` | `30417` | yes |
| `z` | `30418` | no |

Its locked block protocol ID is `1151` and matching item raw ID is `8`. Registration selects map
color `DEEPSLATE`, `BASEDRUM`, `requiresCorrectToolForDrops`, strength `3.0/6.0` and the
`DEEPSLATE` sound type. That sound type has volume/pitch multipliers `1/1` and uses break, fall,
hit, place and step sound IDs `506`, `507`, `508`, `509` and `510` respectively.

All three states retain full unit selection, collision, visual and occlusion shapes; emission
zero; light dampening `15`; shade brightness `0.2`; friction `0.6`; speed/jump factors `1`;
restitution zero; solid redstone conduction; normal piston reaction; full sturdy faces; and no
random/scheduled tick, use, attack, entity-contact, neighbor, signal, comparator or block-event
override. Deepslate is directly `mineable/pickaxe`, and no incorrect-tier tag names it, so every
pickaxe tier is correct while non-pickaxe tools are not. Its common block item stacks to `64`, has
only standard identity/name/model components and directly selects the reloadable
`sulfur_cube_archetype/slow_bouncy` record; archetype matching and movement remain with that owner.

**Transition and ordering:**

#### Placement, transforms and breaking

Player placement sets `axis` to the clicked face's axis. Explicit component, command, generation
and template writes retain the supplied valid axis, defaulting to `y` where a state omits it.
Quarter-turn rotations exchange `x` and `z`; half-turn and no rotation retain the axis. Mirrors do
not change an axis. This block adds no later state transition of its own.

The one-roll block loot table first tests Silk Touch level at least one. That branch returns one
deepslate without an explosion-survival condition. Otherwise it offers one cobbled deepslate
behind `survives_explosion`; the table uses random sequence `minecraft:blocks/deepslate`.
Incorrect-tool player breaks admit neither branch through the generic harvest gate. Thus a
correct Silk Touch pickaxe preserves deepslate even in a loot context carrying an explosion
radius, while the non-Silk cobbled result can be suppressed by the explosion condition.

#### Cooking, stonecutting and unlocks

One building-category smelting record consumes exact `cobbled_deepslate`, returns one deepslate,
awards `0.1` experience and omits `cookingtime`; `SmeltingRecipe` supplies `200` ticks. Its recipe
advancement has one OR requirement containing exact cobbled-deepslate possession and its own
`recipe_unlocked` criterion, then grants only `minecraft:deepslate`.

Exactly seventeen stonecutting records consume one exact deepslate:

- one chiseled deepslate;
- one each of cobbled deepslate, cobbled stairs and cobbled wall, or two cobbled slabs;
- one each of deepslate bricks, brick stairs and brick wall, or two brick slabs;
- one each of deepslate tiles, tile stairs and tile wall, or two tile slabs; and
- one each of polished deepslate, polished stairs and polished wall, or two polished slabs.

Each paired recipe advancement has one OR requirement containing exact deepslate possession and
that recipe's `recipe_unlocked` criterion, and grants only the paired recipe. This leaf owns the
base-deepslate input/output identity and exact counts; the seventeen masonry and shape results
retain their own block-state behavior. Furnace/stonecutter admission, matching, allocation,
publication and recipe-book synchronization remain with their generic owners.

#### Tags, replacement and feature support

Deepslate's four direct block tags are exactly `ancient_city_replaceable`,
`base_stone_overworld`, `deepslate_ore_replaceables` and `mineable/pickaxe`. The base-stone
membership composes into `azalea_root_replaceable`, `bats_spawnable_on`,
`dripstone_replaceable_blocks`, `forest_rock_can_place_on`, `moss_replaceable`,
`lush_ground_replaceable`, `nether_carver_replaceables`, `overworld_carver_replaceables`,
`sculk_replaceable` and `sculk_replaceable_world_gen`. It can therefore admit deepslate as
root/dripstone/moss/lush-ground replacement, forest-rock support, bat-spawn support and
carver/sculk replacement when each owning algorithm's other gates pass.

The two-member `deepslate_ore_replaceables` tag admits deepslate to the deepslate target branch of
the seventeen coal, copper, diamond, emerald, gold, infested, iron, lapis and redstone configured
ore records. Those branches write the matching deepslate ore; the infested record instead writes
default `infested_deepslate` with axis `y`. Target order, exposure tests, geometry, placement
modifiers and writes remain with `WGEN-PIPELINE-001`.

Deepslate is also an explicit support in `glow_lichen` and `sculk_vein`. Both use search range
`20`; glow lichen admits ceilings and walls with default spreading chance `0.5`, while sculk vein
admits all six faces with chance `1`. The Overworld water and lava springs explicitly include
deepslate in their eleven- and eight-identity valid lists. These support identities add no block
callback.

#### Terrain, flat-world and retrogen writes

All five locked Overworld-like noise settings end their surface sequence with named vertical
gradient `minecraft:deepslate`: it is true at and below absolute Y `0`, false at and above Y `8`,
uses the locked positional random decision between those anchors, and writes state `30417`
(`axis=y`) when reached. Earlier surface branches can shadow it, and later feature/carver/sculk
passes can replace the result.

The `water_world` flat preset writes, bottom-up, one bedrock layer, `64` deepslate layers, five
stone, five dirt, five gravel and `90` water; its omitted state properties select axis `y`.
Below-zero retrogen visits its persisted missing-bedrock positions and replaces a position only
when the live old state is still bedrock, writing default axis-`y` deepslate. Flat expansion,
surface first-match evaluation and retrogen iteration/serialization remain with their existing
world-lifecycle owners.

#### Ancient-city template join

The exhaustive 1,212-template scan finds `11,508` live raw deepslate cells in 25 ancient-city NBT
inputs. One input, `city_center/walls/bottom_right_corner`, has `67` cells but is the locked
unreferenced template already identified by `WGEN-JIGSAW-ANCIENT-CITY-001`; it is not counted as a
reachable pool result. The 24 referenced templates contain `11,441` raw cells:

| Reachable group | Templates | Raw cells | Processor |
|---|---:|---:|---|
| `city_center_{1,2,3}` | 3 | `4,422` (`1,474` each) | start degradation |
| six `city/entrance/*` entries | 6 | `4,120` | generic degradation |
| ten referenced `city_center/walls/*` entries | 10 | `1,450` | generic degradation |
| barracks, two pillars, sauna and small statue | 5 | `1,449` | generic degradation |

Start degradation neither rots out nor substitutes base deepslate. Generic degradation first
applies integrity `0.95` block rot to the `ancient_city_replaceable` tag, so each admitted
deepslate cell can be omitted; its following substitution rules do not otherwise target base
deepslate. Protected-live-target rejection, clipping and write results remain independent.

All raw cells have axis `y` except `structures/medium_pillar_1`, which has `74` axis-`y` cells and
one axis-`z` cell. Template quarter-turns rotate that one horizontal state between `z` and `x`;
the other `11,507` raw axes remain `y`. Pool choice, processor RNG, transforms, protected targets
and commits remain with the jigsaw owners; the raw census is not an unconditional final-world
count.

**Client projection:**

Each axis has four equally weighted blockstate alternatives using `block/deepslate` or
`block/deepslate_mirrored`. Axis `y` uses unrotated and Y-180 alternatives; axis `x` applies
X-90/Y-90; axis `z` applies X-90 and optional Y-180. The two block models inherit
`cube_column`/`cube_column_mirrored`, with side texture `block/deepslate` and end texture
`block/deepslate_top`. The item selector always uses the nonmirrored base block model. Weighted
variant selection, face culling and rendering remain client-owned; terrain/block updates publish
the three locked state IDs without adding a packet layout or connection state.

**Branches and aborts:**

Three axes; clicked face, explicit/default write and transform; correct/incorrect tool; Silk/non-
Silk and explosion survival; smelting/output capacity; seventeen cut outputs and eighteen paired
OR unlocks; four direct and ten composed block-tag joins; seventeen ore target branches;
multiface/spring admission; surface gradient/first-match, flat and retrogen predicates; 24
reachable versus one unreferenced structure input; start/generic degradation, rot, transform,
clip/protection/write and client weighted-model branches are distinct.

**Constants and randomness:**

States `30416..30418`; block/item IDs `1151/8`; strength `3/6`; sound IDs `506..510`; emission
`0`, dampening `15`, shade `0.2`, friction `0.6`, factors `1`, restitution `0`, stack `64`;
smelting `200/0.1/1`; seventeen cut records with slab output `2` and all others `1`; multiface
range `20`, chances `0.5/1`; surface anchors `0/8`; flat layers `1/64/5/5/5/90`; raw/reachable/
unreferenced template cells `11,508/11,441/67`; start/generic reachable cells `4,422/7,019`;
generic integrity `0.95`. The block consumes no RNG itself; loot, surface, feature, structure and
client model owners retain their random streams.

**Side effects:**

Axis-aware ordinary placement and transforms; conditional self/cobbled loot; furnace and
stonecutter outputs plus recipe unlocks; tag-selected replacement/support/spawn behavior;
surface, flat, retrogen, ore, feature and ancient-city terrain writes; ordinary persistence and
weighted opaque block/item/sound projection.

**Gates:**

Write authority and valid axis; correct-tool harvest, Silk Touch and explosion context; active
recipe/advancement/loot/tag/archetype/worldgen snapshots; furnace/stonecutter input and output;
feature/support/replacement/spawn predicates; surface position and earlier match; persisted
retrogen marker plus live bedrock; reachable pool entry, processor RNG, clip/protected target and
client state/model context.

**Boundary cases and quirks:**

Silk Touch preserves the axis-bearing block identity but the dropped item carries no axis until a
later placement chooses one. Surface, flat and retrogen paths always select axis `y`; only one raw
locked structure cell begins horizontal. Start degradation has no block-rot stage, whereas every
reachable non-center cell is rot-eligible through the same direct tag. The `67` cells in the
unreferenced template prove artifact presence but not a selectable generation path. The
stonecutting fan-out does not make its seventeen outputs members of this one-ID family.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-REPORT-001`; `OFF-DATA-001`;
`net.minecraft.world.level.block.Blocks`;
`net.minecraft.world.level.block.RotatedPillarBlock#{createBlockStateDefinition,getStateForPlacement,rotatePillar}`;
`net.minecraft.world.level.block.SoundType`;
`net.minecraft.world.item.crafting.AbstractCookingRecipe#cookingMapCodec`;
`net.minecraft.world.level.levelgen.SurfaceRules$VerticalGradientConditionSource`;
`net.minecraft.data.worldgen.SurfaceRuleData#overworldLike`;
`net.minecraft.world.level.levelgen.BelowZeroRetrogen#replaceOldBedrock`;
`net.minecraft.world.level.levelgen.flat.FlatLevelGeneratorPresets$Bootstrap`;
`net.minecraft.world.level.levelgen.feature.{OreFeature,MultifaceGrowthFeature,SpringFeature}`;
`net.minecraft.world.level.levelgen.structure.templatesystem.StructureTemplate#placeInWorld`;
`net.minecraft.world.entity.ambient.Bat#checkBatSpawnRules`;
`net.minecraft.world.entity.monster.cubemob.SulfurCube#matchingArchetypes`;
`reports/blocks.json#minecraft:deepslate`;
`reports/registries.json#minecraft:{block,item,sound_event}`;
`reports/minecraft/components/item/deepslate.json`;
`data/minecraft/loot_table/blocks/deepslate.json`;
`data/minecraft/recipe/{deepslate,*_from_deepslate_stonecutting}.json`;
`data/minecraft/advancement/recipes/{building_blocks,decorations}/{deepslate,*_from_deepslate_stonecutting}.json`;
`data/minecraft/tags/block/{ancient_city_replaceable,base_stone_overworld,deepslate_ore_replaceables,mineable/pickaxe,azalea_root_replaceable,bats_spawnable_on,dripstone_replaceable_blocks,forest_rock_can_place_on,moss_replaceable,lush_ground_replaceable,nether_carver_replaceables,overworld_carver_replaceables,sculk_replaceable,sculk_replaceable_world_gen}.json`;
`data/minecraft/tags/item/sulfur_cube_archetype/slow_bouncy.json`;
`data/minecraft/worldgen/configured_feature/{glow_lichen,sculk_vein,spring_water,spring_lava_overworld,ore_*}.json`;
`data/minecraft/worldgen/noise_settings/{overworld,large_biomes,amplified,caves,floating_islands}.json`;
`data/minecraft/worldgen/flat_level_generator_preset/water_world.json`;
`data/minecraft/worldgen/template_pool/ancient_city/**/*.json`;
`data/minecraft/worldgen/processor_list/ancient_city_{start,generic}_degradation.json`;
`data/minecraft/structure/**/*.nbt`;
`assets/minecraft/blockstates/deepslate.json`;
`assets/minecraft/models/block/{deepslate,deepslate_mirrored}.json`;
`assets/minecraft/items/deepslate.json`.

**Test vectors:**

Run `EXP-BLK-055` across all axes, placement/transform/write, tool/enchantment/explosion, all
eighteen processing/unlock records, every direct/composed tag and ore target, feature/surface/flat/
retrogen path, all 25 containing templates and all 1,212 inputs, processor/transform/clip/protected
boundaries, persistence, sound and weighted block/item models. Assert exact states, IDs, constants,
RNG ownership, outputs, raw/reachable/final structure outcomes and client convergence.

**Limits:**

Generic state publication, breaking, loot, cooking, stonecutting, advancements, sulfur-cube
movement, spawn/feature/carver/sculk/surface/retrogen algorithms, jigsaw processing, packet
encoding and rendering remain with `BLK-UPDATE-001`, `PLY-BREAK-001`, `ITM-LOOT-001`,
`ITM-RECIPE-001`, `ITM-STONECUTTER-001`, `ITM-ADVANCEMENT-001`, `MOB-SPAWN-001`,
`WGEN-PIPELINE-001`, `WGEN-JIGSAW-PROCESSORS-001`, `WGEN-JIGSAW-ANCIENT-CITY-001`,
`PROTO-PLAY-CLIENTBOUND-TERRAIN-001`, `PROTO-PLAY-CLIENTBOUND-BLOCK-001`,
`PROTO-PLAY-CLIENTBOUND-SOUND-001` and `CLI-006`.
