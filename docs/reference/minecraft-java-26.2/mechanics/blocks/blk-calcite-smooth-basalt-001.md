# Blocks mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `BLK-CALCITE-SMOOTH-BASALT-001` — Calcite and smooth basalt join geode shells to replacement, cooking and ancient-city entrances

**Parent:** `BLK-001`, `BLK-002`, `BLK-003`, `BLK-004`, `BLK-005`, `PLY-005`, `PLY-006`,
`ITM-004`, `ITM-006`, `ENT-001`, `ENV-003`, `WGEN-003`, `CLI-006`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — the two locked registrations, registry and block/item reports, loot, recipe,
advancement, tag and worldgen records, all 1,212 decoded structure templates and exact client
assets exhaust the two property-free identities and their observable dispatch joins.

**Applies when:**

`minecraft:calcite` or `minecraft:smooth_basalt` is placed, mined, exploded, selected by a tag or
worldgen record, persisted or projected; basalt is smelted; or an amethyst geode, ancient-city
entrance, surface rule, carver, spring, multiface feature or sculk transaction selects either
identity.

**Authoritative state:**

Both identities are plain property-free `Block` instances with no block entity, random/scheduled
tick, use, attack, entity-contact, neighbor, redstone, comparator or block-event override:

| Identity | State | Block/item raw IDs | Map color | Instrument | Strength | Sound IDs |
|---|---:|---:|---|---|---:|---|
| `calcite` | `27160` | `1025` / `11` | `TERRACOTTA_WHITE` | `BASEDRUM` | `0.75/0.75` | calcite `244..248` |
| `smooth_basalt` | `32069` | `1172` / `392` | `COLOR_BLACK` | `BASEDRUM` | `1.25/4.2` | basalt `142..146` |

Calcite's registration directly selects `CALCITE` sounds and the tabulated properties. Smooth
basalt registers a plain block from `ofLegacyCopy(Blocks.BASALT)`: it copies basalt's map color,
instrument, correct-tool requirement, destroy speed, explosion resistance and `BASALT` sound type,
but not basalt's `RotatedPillarBlock` class or `axis` property. Both sound types have volume/pitch
multipliers `1/1` and use their respective break, step, place, hit and fall events.

Both states have full unit selection, collision and occlusion shapes, emission zero, light
dampening `15`, shade brightness `0.2`, friction `0.6`, speed/jump factors `1`, restitution zero,
solid redstone conduction, normal piston reaction and full sturdy faces. Each is directly
`mineable/pickaxe`; no locked incorrect-tier tag names either identity, so every pickaxe tier is
correct and non-pickaxe tools are not. The matching common block items stack to `64` and contain
only their standard identity/name/model components. Both items directly select the reloadable
`sulfur_cube_archetype/slow_bouncy` record; matching and its exact movement values remain with that
archetype's owner.

**Transition and ordering:**

#### Placement, breaking and processing

Ordinary placement and authoritative component/worldgen/template writes commit the sole state;
rotation and mirror are identity operations. Each one-roll block loot table offers its matching
item behind `survives_explosion` and uses random sequence `minecraft:blocks/<identity>`. An
admitted correct-pickaxe break returns one matching item, an incorrect tool returns nothing, and an
explosion context can independently suppress the offer.

The sole processing record involving either output is a building-category smelting recipe that
consumes one exact `basalt`, returns one `smooth_basalt`, awards `0.1` experience and omits
`cookingtime`; `SmeltingRecipe` supplies the locked default of `200` ticks. Its advancement has one
OR requirement containing exact basalt possession and `recipe_unlocked`, then grants only the
smooth-basalt recipe. Furnace admission, fuel, progress, output allocation and publication remain
with `ITM-RECIPE-001`. No recipe produces calcite, and no other recipe, advancement, trade or
non-block loot record names either output identity.

#### Feature and replacement joins

The amethyst-geode record fixes calcite as `middle_layer_provider` and smooth basalt as
`outer_layer_provider`. Field/layer traversal, protection, crack selection and safe writes remain
with `WGEN-PIPELINE-001`; this leaf owns the exact two provider identities and resulting states.

Calcite is an allowed support for configured `glow_lichen` and `sculk_vein` multiface growth. Both
search range `20`; glow lichen enables ceiling/wall placement with default spread chance `0.5`,
while sculk vein enables all six faces with chance `1`. The support scans and optional spreading
remain with the multiface feature owner. Calcite is also a valid surrounding block for the locked
Overworld water and lava springs. Water's valid list has eleven identities and lava's eight; both
include calcite. Their exact neighbor-count transaction, falling-fluid write and delay-zero tick
remain with the spring owner.

Calcite directly enters `overworld_carver_replaceables`; both identities directly enter
`sculk_replaceable`. Calcite additionally appears as the `calcite` noise selector and result state
in the shared five-noise-settings surface program, whose closed selector interval is
`[-0.0125,0.0125]`. Carver masks, sculk charge traversal and first-match surface evaluation retain
their existing owners. These tag and record memberships can replace the identities later or write
calcite; they do not add callbacks to either block.

#### Ancient-city entrance join

An exhaustive scan of all 1,212 locked structure templates finds no calcite cell. Smooth basalt
has exactly 205 live raw cells, all in the six equal-weight rigid entries of the
`ancient_city/city/entrance` pool:

| Template suffix | Smooth-basalt cells |
|---|---:|
| `entrance_connector` | 40 |
| `entrance_path_1` | 48 |
| `entrance_path_2` | 39 |
| `entrance_path_3` | 31 |
| `entrance_path_4` | 28 |
| `entrance_path_5` | 19 |

Every entry uses `ancient_city_generic_degradation`. Smooth basalt is absent from its rottable tag,
and none of its three substitution rules targets smooth basalt, so all 205 cells pass those two
processor stages unchanged. Live protected-target rejection, transform, clipping, placement and
later world mutation remain with `WGEN-JIGSAW-PROCESSORS-001` and
`WGEN-JIGSAW-ANCIENT-CITY-001`; the raw census is not a claim that every cell is unconditionally
written into an arbitrary live world.

**Client projection:**

Each property-free blockstate selects one unrotated matching `cube_all` block model and matching
texture. Each item selector directly selects that same block model. State updates, terrain chunks,
loot, recipe progress and the ten registered material sounds retain their existing protocol
families; this leaf adds no packet layout or connection state.

**Branches and aborts:**

Two identities; ordinary/component/feature/template placement; correct versus incorrect tool;
surviving versus suppressed explosion loot; smelting admission, blocked output and both unlock
criteria; each exact support/replacement/provider selector versus nonmember; surface interval
inside/outside and first-match shadowing; six entrance choices, transforms, clips and protected
targets; persistence and block/item/sound projection are distinct.

**Constants and randomness:**

States and registry IDs, strength and ten sound IDs are tabulated above; emission `0`, dampening
`15`, shade `0.2`, friction `0.6`, speed/jump `1`, restitution `0`, stack `64`; smelting
time/XP/output `200/0.1/1`; multiface search `20`, spread chances `0.5/1`; calcite noise interval
`[-0.0125,0.0125]`; six entrance weights `1`; template counts `40/48/39/31/28/19`, total `205`.
The blocks consume no RNG directly; loot, surface, feature, structure and archetype owners retain
their random streams.

**Side effects:**

Generic placement/removal and optional self loot; furnace output and recipe advancement; tag-
selected feature, carver, sculk and surface writes; geode shell and ancient-city entrance terrain;
ordinary state persistence plus opaque block/item and material-sound projection.

**Gates:**

Write authority; correct-tool and explosion contexts; furnace recipe/input/output/fuel state;
advancement snapshot; live pickaxe, carver, sculk and slow-bouncy tags; configured feature,
surface/noise, geode, template-pool and processor snapshots; structure clip/protected live state;
client registry/model/sound context.

**Boundary cases and quirks:**

Smooth basalt copies basalt's behavior properties without copying its axis state schema. Calcite is
both a possible surface output and a later carver/sculk replacement input. Smooth basalt's 205 raw
entrance cells are immune to the entrance rot/substitution stages, but protected live targets and
clipping can still prevent writes. The smelting record's absent time is semantically the serializer
default `200`, not an unknown or zero.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-REPORT-001`; `OFF-DATA-001`;
`net.minecraft.world.level.block.Blocks`;
`net.minecraft.world.level.block.state.BlockBehaviour$Properties#ofLegacyCopy`;
`net.minecraft.world.item.crafting.AbstractCookingRecipe#cookingMapCodec`;
`net.minecraft.world.item.crafting.SmeltingRecipe`;
`net.minecraft.world.level.levelgen.feature.MultifaceGrowthFeature#place`;
`net.minecraft.world.level.levelgen.feature.SpringFeature#place`;
`net.minecraft.world.level.levelgen.feature.GeodeFeature#place`;
`net.minecraft.world.level.levelgen.structure.templatesystem.StructureTemplate#placeInWorld`;
`net.minecraft.world.entity.monster.cubemob.SulfurCube#matchingArchetypes`;
`reports/blocks.json#minecraft:{calcite,smooth_basalt}`;
`reports/registries.json#minecraft:{block,item,sound_event}`;
`reports/minecraft/components/item/{calcite,smooth_basalt}.json`;
`data/minecraft/loot_table/blocks/{calcite,smooth_basalt}.json`;
`data/minecraft/{recipe/smooth_basalt,advancement/recipes/building_blocks/smooth_basalt}.json`;
`data/minecraft/tags/{block/{mineable/pickaxe,overworld_carver_replaceables,sculk_replaceable},item/sulfur_cube_archetype/slow_bouncy}.json`;
`data/minecraft/worldgen/configured_feature/{amethyst_geode,glow_lichen,sculk_vein,spring_water,spring_lava_overworld}.json`;
`data/minecraft/worldgen/noise_settings/{overworld,large_biomes,amplified,caves,floating_islands}.json`;
`data/minecraft/worldgen/{template_pool/ancient_city/city/entrance,processor_list/ancient_city_generic_degradation}.json`;
`data/minecraft/structure/**/*.nbt`;
`assets/minecraft/{blockstates,models/block,items}/{calcite,smooth_basalt}.json`.

**Test vectors:**

Run `EXP-BLK-054` across both states, every placement/break/explosion path, recipe/unlock/reload
branch, exact tag and feature membership, surface/noise and later replacement paths, geode layer
writes, all six entrance templates with transforms/clips/protected targets, the complete template
scan, save/reload and both block/item models. Assert exact state/registry/sound IDs, physical and
tool properties, loot, recipe values, selector outcomes, raw and final template results and client
convergence.

**Limits:**

Generic state publication, breaking, loot evaluation, cooking, advancements, sulfur-cube movement,
feature/carver/sculk/surface algorithms, geode traversal, jigsaw processing, packet encoding and
client rendering remain with `BLK-UPDATE-001`, `PLY-BREAK-001`, `ITM-LOOT-001`,
`ITM-RECIPE-001`, `ITM-ADVANCEMENT-001`, `WGEN-PIPELINE-001`,
`WGEN-JIGSAW-PROCESSORS-001`, `WGEN-JIGSAW-ANCIENT-CITY-001`,
`PROTO-PLAY-CLIENTBOUND-TERRAIN-001`, `PROTO-PLAY-CLIENTBOUND-BLOCK-001`,
`PROTO-PLAY-CLIENTBOUND-SOUND-001` and `CLI-006`.
