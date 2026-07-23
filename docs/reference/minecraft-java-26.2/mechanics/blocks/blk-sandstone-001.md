# Blocks mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `BLK-SANDSTONE-001` — Full sandstone blocks join processing, replacement, surface generation and desert structures

**Parent:** `BLK-001`, `BLK-002`, `BLK-003`, `BLK-005`, `PLY-005`, `PLY-006`, `ITM-004`,
`ITM-006`, `ENT-001`, `ENV-003`, `WGEN-003`, `CLI-006`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — the eight locked registrations, reports, recipes, advancements, loot, tags,
feature/processor/preset records, code-built structures, decoded templates and client assets
exhaust their identities, states, transitions and observable projections.

**Applies when:**

`minecraft:sandstone`, `minecraft:chiseled_sandstone`, `minecraft:cut_sandstone`,
`minecraft:smooth_sandstone`, `minecraft:red_sandstone`,
`minecraft:chiseled_red_sandstone`, `minecraft:cut_red_sandstone` or
`minecraft:smooth_red_sandstone` is placed, transformed, mined, exploded, crafted, cut, smelted,
selected by a tag, equipped by a sulfur cube, generated, serialized or rendered.

**Authoritative state:**

All eight identities are property-free ordinary `Block` instances with no block entity and exactly
one state:

| Identity | State | Map color | Destroy speed | Explosion resistance |
|---|---:|---|---:|---:|
| sandstone | 578 | `SAND` | 0.8 | 0.8 |
| chiseled sandstone | 579 | `SAND` | 0.8 | 0.8 |
| cut sandstone | 580 | `SAND` | 0.8 | 0.8 |
| smooth sandstone | 13481 | `SAND` | 2.0 | 6.0 |
| red sandstone | 13247 | `COLOR_ORANGE` | 0.8 | 0.8 |
| chiseled red sandstone | 13248 | `COLOR_ORANGE` | 0.8 | 0.8 |
| cut red sandstone | 13249 | `COLOR_ORANGE` | 0.8 | 0.8 |
| smooth red sandstone | 13483 | `COLOR_ORANGE` | 2.0 | 6.0 |

The three ordinary yellow and three ordinary red identities select bass-drum note instrument,
`requiresCorrectToolForDrops` and one-argument strength `0.8`. Each smooth identity selects the
same family map color, instrument and tool requirement but calls two-argument strength
`2.0/6.0`.

Every state retains stone sound; full selection, collision, visual and occlusion cubes; emission
`0`; friction `0.6`; speed/jump factors `1`; restitution `0`; and piston reaction `NORMAL`. The
solid-render cube does not propagate skylight, has light dampening `15` and shade brightness
`0.2`, and is redstone conducting, suffocating and view blocking. Its sturdy top passes the
default spawn-support predicate because emission is below `14`; entity-specific spawn admission
remains independent.

Each identity has an ordinary common-rarity `BlockItem`, maximum stack size `64`, matching name
and item model, and no special use component.

**Transition and ordering:**

#### Placement, transforms and breaking

Placement selects the identity's only state; rotation and mirror retain it. A legal
`minecraft:block_state` component cannot add a property because the state definition is empty.
The blocks add no fluid, gravity, random/scheduled tick, shape-update, use, attack,
entity-contact, neighbor, redstone-output, comparator or block-event callback.

Correct-tool admission remains with generic player breaking. Each one-roll block loot table offers
the matching item behind `survives_explosion` and uses random sequence
`minecraft:blocks/<identity>`. An admitted ordinary player break returns the matching item; an
incorrect player tool returns nothing. Explosion context may independently suppress the offer.

#### Recipes and advancements

The reloadable full-block recipe graph is exact and symmetric by sand color unless noted:

- four sand or red-sand items in a 2-by-2 pattern produce one matching base sandstone;
- two vertically stacked matching base slabs produce one matching chiseled sandstone;
- four matching base sandstone in a 2-by-2 pattern produce four matching cut sandstone;
- stonecutting one base sandstone produces one matching chiseled or cut sandstone;
- smelting one base sandstone produces one matching smooth sandstone, awards `0.1` experience
  and uses the serializer's omitted-field default of `200` ticks;
- the dune armor-trim-template duplication pattern uses one yellow sandstone, one existing dune
  template and seven diamonds to produce two dune templates.

The exact joins into separately owned shape outputs are:

- three base or chiseled sandstone produce six matching base slabs;
- six base, chiseled or cut sandstone in the stair pattern produce four matching base stairs;
- six base sandstone in two rows produce six matching walls;
- stonecutting one base sandstone produces two matching base slabs, one matching base stair or
  one matching wall;
- three cut sandstone produce six matching cut slabs, while stonecutting one cut or one base
  sandstone produces two matching cut slabs;
- three smooth sandstone produce six matching smooth slabs or four matching smooth stairs by
  their respective patterns, while stonecutting one smooth sandstone produces two smooth slabs
  or one smooth stair.

This leaf owns the eight full-block identities and their input/output values. Sandstone stairs,
slabs and walls retain `shape-family` state and shape behavior. Matching, allocation, stonecutter
menus, furnace progress, later template use and result publication remain generic.

Base-block recipes unlock from possessing matching sand. Smooth records unlock from matching base
sandstone. Yellow chiseled crafting unlocks from yellow base slab; its red analogue accepts red
base, chiseled or cut sandstone, while both stonecutting records unlock from matching base
sandstone. Cut crafting and stonecutting unlock from matching base sandstone. Shape-output
advancements use their exact referenced full-block inputs or recipe criterion. Dune duplication
unlocks from possessing the dune template. Every recipe advancement also admits its own
`recipe_unlocked` criterion.

#### Tags and their consumers

All eight blocks are direct members of reloadable `mineable/pickaxe`. Only base sandstone and base
red sandstone are direct members of both `overworld_carver_replaceables` and
`sculk_replaceable`; carvers and worldgen sculk conversion may therefore replace those two but not
their chiseled, cut or smooth variants through these tags.

All eight items are direct members of `sulfur_cube_archetype/slow_bouncy`. Equipping or swallowing
one can select that archetype; its locked knockback pair is
`(horizontal, vertical)=(0.4125, 0.24)`, hit sound is
`minecraft:entity.sulfur_cube.slow_bouncy.hit`, and the remaining attribute/sound values come from
the same record. Multiple-match ordering, equipment changes and composed attributes remain with
the sulfur-cube owner.

#### Generation and structure joins

The base identities join the generic surface and feature paths:

- the shared surface-rule tree embedded by `overworld`, `large_biomes`, `amplified`, `caves` and
  `floating_islands` noise settings selects sandstone below sand and red sandstone below red sand
  in its ordered beach/desert and badlands branches;
- `disk_sand` targets dirt/grass in a radius `2..6`, half-height `2` disk, normally writes sand and
  instead supplies sandstone when the block one below is air;
- an admitted desert well performs its exact ordered base, perimeter, roof and pillar sandstone
  writes around the separately owned water, sand, slab and archaeology cells;
- the `desert` flat preset has bedrock `1`, stone `3`, sandstone `52`, sand `8`; the
  `redstone_ready` preset has bedrock `1`, stone `3`, sandstone `116`.

Code-built structures add two bounded joins. Buried treasure accepts sandstone, among four other
stone-family identities, as its below-chest support gate. Desert pyramid layout writes sandstone,
cut sandstone and chiseled sandstone in the exact cells and order specified by
`WGEN-STRUCTURE-DESERT-PYRAMID-001`; its sandstone stairs/slabs remain shape-owned.

Decoded locked templates contain exactly `5,833` live full-block cells across `83` inputs:

| Template family | Identity | Inputs with live cells | Cells |
|---|---|---:|---:|
| underwater ruins | sandstone | 3 | 16 |
| underwater ruins | chiseled sandstone | 4 | 32 |
| underwater ruins | cut sandstone | 12 | 685 |
| desert villages | sandstone | 2 | 10 |
| desert villages | chiseled sandstone | 2 | 2 |
| desert villages | cut sandstone | 35 | 1,083 |
| desert villages | smooth sandstone | 66 | 4,001 |
| trial chambers | chiseled sandstone | 1 | 4 |

The trial-chamber cells are in `spawner/melee/husk`. The village zombie-desert processor can
replace each smooth-sandstone input with cobweb at probability `0.08` and each cut-sandstone input
at probability `0.1`; its ordered rule traversal remains with `WGEN-JIGSAW-PROCESSORS-001`.
Village connectors additionally contain 49 `smooth_sandstone` final-state strings whose connector
replacement is owned by `WGEN-JIGSAW-VILLAGES-001`. No locked structure template contains a live
red-sandstone-family full-block cell.

Surface evaluation, disk/well algorithms, flat-world selection, structure/pool selection,
processor traversal, template transforms and block-write ordering remain with their named worldgen
owners; this leaf owns the resulting sandstone identity and behavior.

**Client projection:**

Each property-free blockstate selects one matching model:

| Identity family | Parent | Texture mapping |
|---|---|---|
| base yellow/red | `cube_bottom_top` | separate `<family>_bottom`, side and `<family>_top` |
| chiseled yellow/red | `cube_column` | matching top as `end`, chiseled texture as `side` |
| cut yellow/red | `cube_column` | matching top as `end`, cut texture as `side` |
| smooth yellow/red | `cube_all` | matching sandstone top texture on every face |

Every item definition selects its matching block model directly. There is no weighted,
conditional, animated, rotated or special-renderer branch. Terrain and block-update packets
publish states `578..580`, `13247..13249`, `13481` and `13483`; ordinary full-cube face culling,
breaking particles/sounds, map shading and opaque rendering consume the selected state/model.

**Branches and aborts:**

Eight identities and two colors; ordinary versus smooth strength; yellow versus red map color;
direct versus component/template writes; correct versus incorrect tool; surviving versus
suppressed explosion loot; every craft/cut/smelt/duplication and advancement success/rejection;
pickaxe, carver/sculk and item-archetype memberships; every surface/feature/preset/code/template
selection and processor outcome; and block versus item projection are distinct.

**Constants and randomness:**

States `578..580`, `13247..13249`, `13481`, `13483`; ordinary strength `0.8/0.8`; smooth strength
`2.0/6.0`; emission `0`; dampening `15`; shade `0.2`; friction `0.6`; speed/jump `1`;
restitution `0`; stack `64`; cooking time/XP/output `200/0.1/1`; disk radius/half-height `2..6/2`;
flat sandstone heights `52/116`; template cells `5,833`; zombie processor probabilities
`0.08/0.1`. These blocks consume no RNG directly; generic explosion, feature, structure and
processor owners retain their randomness.

**Side effects:**

Generic placement/removal; conditional same-identity loot; recipe, advancement, furnace and
template-duplication outputs; tag-selected carver/sculk/archetype behavior; surface, feature,
preset, code-built structure and template writes; processor substitutions; ordinary persistence
and opaque block/item projection.

**Gates:**

Selected identity; placement/write authority; correct-tool harvest admission; explosion context;
active recipe, advancement, loot, tag and archetype snapshots; surface/feature/preset/structure
selection; processor and template admission; client state/model context.

**Boundary cases and quirks:**

Smooth variants are substantially harder and more blast resistant than the other six despite
sharing their family's sound, instrument and tool requirement. Only the two base identities are
carver/sculk replaceable, while all eight items are slow-bouncy. Smooth models intentionally use
the top texture on every face. Template palette entries that no live cell indexes are not counted
in the exact cell census. Shape-family stairs/slabs/walls join recipes and structures without
becoming members of this eight-ID family.

**Evidence:**

`OFF-SERVER-001`, `OFF-CLIENT-001`, `OFF-REPORT-001`, `OFF-DATA-001`;
`net.minecraft.world.level.block.Blocks`,
`net.minecraft.world.level.block.state.BlockBehaviour$Properties#strength`,
`net.minecraft.world.item.crafting.AbstractCookingRecipe#cookingMapCodec`,
`net.minecraft.world.level.levelgen.feature.DesertWellFeature#place`,
`net.minecraft.world.level.levelgen.structure.structures.BuriedTreasurePieces$BuriedTreasurePiece#postProcess`,
`net.minecraft.world.level.levelgen.structure.structures.DesertPyramidPiece#postProcess`,
`net.minecraft.world.entity.monster.cubemob.SulfurCube#matchingArchetypes`;
`reports/blocks.json#minecraft:{sandstone,chiseled_sandstone,cut_sandstone,smooth_sandstone,red_sandstone,chiseled_red_sandstone,cut_red_sandstone,smooth_red_sandstone}`,
`reports/minecraft/components/item/{sandstone,chiseled_sandstone,cut_sandstone,smooth_sandstone,red_sandstone,chiseled_red_sandstone,cut_red_sandstone,smooth_red_sandstone}.json`,
`data/minecraft/tags/block/{mineable/pickaxe,overworld_carver_replaceables,sculk_replaceable}.json`,
`data/minecraft/tags/item/sulfur_cube_archetype/slow_bouncy.json`,
`data/minecraft/sulfur_cube_archetype/slow_bouncy.json`,
`data/minecraft/loot_table/blocks/{sandstone,chiseled_sandstone,cut_sandstone,smooth_sandstone,red_sandstone,chiseled_red_sandstone,cut_red_sandstone,smooth_red_sandstone}.json`,
`data/minecraft/recipe/*sandstone*.json`,
`data/minecraft/recipe/dune_armor_trim_smithing_template.json`,
`data/minecraft/advancement/recipes/{building_blocks,decorations}/*sandstone*.json`,
`data/minecraft/advancement/recipes/misc/dune_armor_trim_smithing_template.json`,
`data/minecraft/worldgen/configured_feature/disk_sand.json`,
`data/minecraft/worldgen/flat_level_generator_preset/{desert,redstone_ready}.json`,
`data/minecraft/worldgen/noise_settings/{overworld,large_biomes,amplified,caves,floating_islands}.json`,
`data/minecraft/worldgen/processor_list/zombie_desert.json`,
`data/minecraft/structure/{underwater_ruin,village,trial_chambers}/**/*.nbt`,
`assets/minecraft/blockstates/{sandstone,chiseled_sandstone,cut_sandstone,smooth_sandstone,red_sandstone,chiseled_red_sandstone,cut_red_sandstone,smooth_red_sandstone}.json`,
`assets/minecraft/models/block/{sandstone,chiseled_sandstone,cut_sandstone,smooth_sandstone,red_sandstone,chiseled_red_sandstone,cut_red_sandstone,smooth_red_sandstone}.json`,
`assets/minecraft/items/{sandstone,chiseled_sandstone,cut_sandstone,smooth_sandstone,red_sandstone,chiseled_red_sandstone,cut_red_sandstone,smooth_red_sandstone}.json`.

**Test vectors:**

Run `EXP-BLK-045` across all eight states, ordinary/component/template placement, transforms,
correct/incorrect tools, explosions, every full-block and shape-joining recipe/unlock, dune
duplication, all four tags and the sulfur archetype, every named surface/feature/preset/code-built
and template join, zombie processing, save/reload and every block/item model. Assert identity,
strength and physical predicates, writes/drops, recipe/unlock values, tag/replacement behavior,
template counts, processor outcomes and selected model/textures.

**Limits:**

This leaf does not re-specify generic placement/break packets, state-component parsing, tool speed,
explosion survival, recipe/stonecutter/furnace/smithing allocation, sulfur-cube composition,
surface/feature/carver/sculk algorithms, structure/jigsaw/template placement, processor traversal
or model loading. Those remain with `BLK-002`, `BLK-STATE-001`, `PLY-006`, `ITM-LOOT-001`,
recipe/furnace/smithing owners, `ENT-KNOCKBACK-001`, `WGEN-PIPELINE-001`, the named structure and
jigsaw leaves, `WGEN-JIGSAW-PROCESSORS-001` and `CLI-006`. Sandstone stairs, slabs and walls remain
under `shape-family`.
