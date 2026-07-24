# Blocks mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `BLK-NETHER-SPROUTS-001` — Nether sprouts survive on tagged substrates and join warped vegetation

**Parent:** `SIM-003`, `BLK-001`, `BLK-002`, `BLK-003`, `BLK-005`, `PLY-002`, `PLY-005`, `PLY-006`,
`ITM-004`, `ITM-006`, `MOB-001`, `MOB-004`, `ENV-003`, `WGEN-002`, `WGEN-003`, `CLI-001`,
`CLI-006`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked registration and reports, the complete loot/tag/worldgen data,
server/client class-reference sweeps, all 1,212 decoded structure templates and exact client assets
close this property-free vegetation block. Its identity joins tagged substrate survival,
support-loss destruction, shears-only loot, 0.5 composting, combined player footsteps,
enchantment-power transmission, tree/mushroom replacement, natural and bonemeal warped-forest
vegetation, and an untinted offset cross projection.

**Applies when:**

`minecraft:nether_sprouts` is placed, replaced, loses support, harvested, exploded, composted,
walked through, tested between an enchanting table and power provider, selected or overwritten by
world generation, persisted, mapped or rendered.

**Authoritative state:**

Nether sprouts is a property-free `NetherSproutsBlock`/`VegetationBlock` with codec type
`minecraft:nether_sprouts`, no block entity and sole state `20961`. Its locked block protocol ID is
`870`, and its block-item raw ID is `281`. Registration selects map color `COLOR_CYAN`, the default
note instrument `HARP`, zero hardness/resistance, no collision, replaceability, piston reaction
`DESTROY`, `XZ` positional offset and the `NETHER_SPROUTS` sound type.

The unoffset selection shape is a centered column from `(2,0,2)` through `(14,3,14)` in sixteenths.
For block position `(x,y,z)`, `XZ` offset computes `seed=Mth.getSeed(x,0,z)`,
`dx=((seed&15)/15-0.5)*0.5` and `dz=(((seed>>8)&15)/15-0.5)*0.5`; both already lie in the
configured `[-0.25,0.25]` clamp and Y offset is zero. Collision and occlusion are empty, emission
and light dampening are zero, skylight propagates through its empty fluid state, and AIR
pathfinding is allowed. It is not sturdy, suffocating, view-blocking, signal-producing or a spawn
surface, and adds no random/scheduled tick, use, attack, entity-contact, comparator or block-event
override.

The sound type has volume/pitch `1/1` and selects sound registry IDs break `1131`, step `1132`,
place `1133`, hit `1134` and fall `1135`. The ordinary block item is common, stacks to `64`, has
standard block-item components and belongs to no direct item tag.

**Transition and ordering:**

#### Placement, support loss, replacement and loot

Placement selects state `20961` only when the block immediately below belongs to
`supports_nether_sprouts`. Locked closure admits exactly the three `dirt` members, two `mud`
members, two `moss_blocks` members, three `grass_blocks` members, farmland, both nylium blocks and
soul soil: 14 support identities. Rotation and mirror are identity operations.

Every neighbor-shape update rechecks that support. Failure returns AIR; ordinary server
`updateOrDestroy` then destroys the sprouts with drops unless its caller suppresses drops. Because
that destruction has no shears tool, the locked loot table emits nothing. State/component writes
can still force state `20961`; the next qualifying shape update applies the survival rule.

The code-built replaceable property lets a different held block item replace the sprouts in place;
an empty placement context also passes, while holding the sprouts item itself does not. Fluid
replacement is admitted, but the sprouts have no waterlogged property or retained fluid state.
Piston movement destroys rather than moves the state.

The one-roll loot table offers one sprouts item only when the tool is exactly
`minecraft:shears`, using random sequence `minecraft:blocks/nether_sprouts`. Other tools,
tool-free support loss and explosions produce no item; Silk Touch and Fortune add no branch.
No bundled recipe consumes or produces the item, and no recipe advancement references it.

#### Composting, footsteps and enchanting

`ComposterBlock` registers item `281` at chance `0.5f`. A player-held insertion at level `0`
succeeds without RNG; levels `1..6` consume one `nextDouble()` and increment exactly when the draw
is below `0.5`. Success writes level plus one with flags `3`, emits `BLOCK_CHANGE`, and `6 -> 7`
schedules maturation after `20` ticks; failure preserves state. Either level-`0..6` result emits
event `1500`, awards the used-item statistic and calls `consume(1, player)`, which preserves
infinite-material holders. Level `7` returns success without insertion or consumption; level `8`
delegates to ordinary item-on-block handling. Automation admits only below level `7`, invokes the
same first-level/RNG transition and always shrinks one item, even after chance failure.

Direct membership in `combination_step_sound_blocks` makes a walking player inside the sprouts use
them as the primary step block. Outside the water-specific branch, the player emits sprouts step
sound `1132` at volume `1*0.15` and pitch `1`, then the supporting block's step sound at
`supportVolume*0.05` and `supportPitch*0.8`. Step cadence, water movement and sound transport remain
with their owners.

The direct `replaceable` block tag is nested by `enchantment_power_transmitter`. Consequently a
sprouts state at the halfway offset between an enchanting table and an
`enchantment_power_provider` does not obstruct that provider. The table first requires the far
provider, then tests the halfway sprouts membership; menu option computation remains with
`ITM-006`.

#### Reload-selected replacement and warped generation

Direct `replaceable_by_trees` membership makes state `20961` pass `TreeFeature.validTreePos`; direct
`replaceable_by_mushrooms` membership lets `AbstractHugeMushroomFeature.placeMushroomBlock`
overwrite it wherever the selected trunk/cap algorithm writes. Those replacements do not require
the code-built placement-replaceable property. Geometry, RNG, clipping and final writes remain
with `WGEN-PIPELINE-001`.

The natural `nether_sprouts` configured feature is `nether_forest_vegetation` with a fixed state
`20961`, spread width `8` and height `4`; its placed feature applies
`count_on_every_layer(count=4)` then biome filtering and appears in the warped forest's vegetation
generation step. The bonemeal configuration uses the same fixed state with width `3`, height `1`.

The common vegetation kernel first requires nylium below the origin and the inclusive vertical
bounds. It makes `width*width` attempts, each drawing two width offsets for X, two height offsets
for Y and two width offsets for Z, then requests state `20961`. An empty candidate above minimum Y
that can survive is offered with flags `2`; the feature returns true iff at least one offer occurs,
independent of write results. Thus the ordinary record makes 64 attempts and the bonemeal record
makes 9.

Bonemealing warped nylium with air inside build height above first invokes
`warped_forest_vegetation_bonemeal`, then invokes `nether_sprouts_bonemeal`, and only afterward
draws `nextInt(8)` for the optional twisting-vines feature. Crimson nylium never invokes the
sprouts record. Applying bone meal directly to nether sprouts has no bonemealable-block dispatch.
The exhaustive NBT scan finds zero state-`20961` cells across all 1,212 structure templates.

**Client projection:**

The sole blockstate variant selects `minecraft:block/nether_sprouts`. Its untinted cross model has
ambient occlusion disabled and two shade-disabled crossed planes using the block texture; the
deterministic XZ state offset shifts that projection. The item selector instead uses the flat
`minecraft:item/nether_sprouts` generated model and item texture. Block updates publish state
`20961`, inventory projection uses item ID `281`, material sounds use IDs `1131..1135`, and map
projection uses `COLOR_CYAN`. This leaf adds no packet field, acknowledgement or connection-local
state.

**Branches and aborts:**

Ordinary versus forced placement; each of 14 support identities versus another block; support
retained/lost with drop suppression; same-item versus different-item/fluid replacement; shears,
other tools, support loss and explosion; finite/infinite player versus automation at composter
levels `0`, `1..6`, `7`, `8`; walking combination versus other/water step paths; enchanting
provider/transmitter combinations; tree and mushroom overwrite; natural versus bonemeal vegetation,
origin/height/candidate/survival/write branches; warped versus crimson nylium; zero structure
selection; state versus block/item/sound/map/model projection; reload and persistence are distinct
branches.

**Constants and randomness:**

State/block/item IDs `20961/870/281`; selection column `(2,0,2)..(14,3,14)`; XZ offset mask `15`,
divisor `15`, scale `0.5`, clamp `0.25`; support identities `14`; hardness/resistance `0/0`; sound
volume/pitch `1/1`; sound IDs `1131/1132/1133/1134/1135`; stack `64`; composter chance `0.5`,
maturation delay `20`, event `1500`; natural width/height/attempts `8/4/64`, bonemeal
`3/1/9`, placed layer count `4`; warped-nylium twisting draw bound `8`; templates/cells
`1212/0`. Placement, support, loot admission, footsteps and enchanting consume no identity-owned
RNG. Composter, vegetation placement and their owners retain the stated conditional streams.

**Side effects:**

Supported placement or forced write; support-loss destruction; shears-only item creation;
replacement by block/fluid/tree/mushroom writes; composter call/optional write, game event,
schedule and level event; combined player step sounds; enchanting-provider admission; natural and
bonemeal vegetation writes; ordinary persistence; cyan map shading and offset cross/flat-item
projection.

**Gates:**

World-write and break authority; active support/replacement/loot tags; tool identity; placement
context; composter level/input/RNG and infinite-material policy; player step context; enchanting
offset/provider; biome/feature/build-height/candidate admission; valid registry, map, sound and
client-resource context.

**Boundary cases and quirks:**

The code-built replaceable flag and reloadable `replaceable` tag are distinct. The former controls
ordinary placement/fluid replacement; the latter supplies enchanting transmission. Support is
broader than nylium, but both official vegetation features require nylium below their origin before
candidate survival. Unsupported sprouts normally vanish without an item because the shears
condition cannot pass. Level-zero composting draws nothing, whereas failed level-`1..6` attempts
still consume finite player input and all automated attempts shrink one. The non-solid plant
combines its own step sound with the block below. Zero structure cells does not mean warped-forest
generation is absent.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-REPORT-001`; `OFF-DATA-001`;
`net.minecraft.world.level.block.Blocks`;
`net.minecraft.world.level.block.SoundType`;
`net.minecraft.world.level.block.NetherSproutsBlock#getShape`;
`net.minecraft.world.level.block.NetherSproutsBlock#mayPlaceOn`;
`net.minecraft.world.level.block.VegetationBlock#updateShape`;
`net.minecraft.world.level.block.VegetationBlock#canSurvive`;
`net.minecraft.world.level.block.state.BlockBehaviour$Properties#offsetType`;
`net.minecraft.world.level.block.state.BlockBehaviour#getMaxHorizontalOffset`;
`net.minecraft.world.level.block.state.BlockBehaviour#canBeReplaced`;
`net.minecraft.world.level.block.Block#updateOrDestroy`;
`net.minecraft.world.level.block.ComposterBlock#bootStrap`;
`net.minecraft.world.level.block.ComposterBlock#useItemOn`;
`net.minecraft.world.level.block.ComposterBlock#insertItem`;
`net.minecraft.world.level.block.ComposterBlock#addItem`;
`net.minecraft.world.item.ItemStack#consume`;
`net.minecraft.world.entity.player.Player#playStepSound`;
`net.minecraft.world.entity.Entity#playCombinationStepSounds`;
`net.minecraft.world.entity.Entity#playMuffledStepSound`;
`net.minecraft.world.level.block.EnchantingTableBlock#isValidBookShelf`;
`net.minecraft.world.level.levelgen.feature.TreeFeature#validTreePos`;
`net.minecraft.world.level.levelgen.feature.AbstractHugeMushroomFeature#placeMushroomBlock`;
`net.minecraft.world.level.levelgen.feature.NetherForestVegetationFeature#place`;
`net.minecraft.world.level.block.NyliumBlock#performBonemeal`;
`net.minecraft.client.data.models.BlockModelGenerators`;
`reports/blocks.json#minecraft:nether_sprouts`;
`reports/registries.json#minecraft:{block,item}/minecraft:nether_sprouts`;
`reports/registries.json#minecraft:sound_event/minecraft:block.nether_sprouts.*`;
`reports/minecraft/components/item/nether_sprouts.json`;
`data/minecraft/loot_table/blocks/nether_sprouts.json`;
`data/minecraft/tags/block/{combination_step_sound_blocks,replaceable,replaceable_by_mushrooms,replaceable_by_trees,enchantment_power_transmitter,supports_nether_sprouts,supports_vegetation,substrate_overworld,dirt,mud,moss_blocks,grass_blocks,nylium}.json`;
`data/minecraft/tags/item/**`;
`data/minecraft/worldgen/configured_feature/{nether_sprouts,nether_sprouts_bonemeal}.json`;
`data/minecraft/worldgen/placed_feature/nether_sprouts.json`;
`data/minecraft/worldgen/biome/warped_forest.json`;
`data/minecraft/{recipe,advancement,structure}/**`;
`assets/minecraft/blockstates/nether_sprouts.json`;
`assets/minecraft/models/{block,item}/nether_sprouts.json`;
`assets/minecraft/items/nether_sprouts.json`.

**Test vectors:**

Run `EXP-BLK-066` across state/registry identity, all support and replacement branches, neighbor
loss, tools/explosion, recipe/advancement absence, every composter level/draw/material boundary,
walking and enchanting joins, tree/mushroom replacement, natural and bonemeal vegetation,
warped-nylium call order, all 1,212 templates, persistence, sounds, map and models. Assert exact
constants, draw/read/write order, absences and client convergence.

**Limits:**

Generic placement, update propagation, breaking, loot, composter maturation/extraction, step
cadence, enchanting option generation, vegetation/tree/mushroom algorithms, packet encoding and
rendering remain with `BLK-PLACE-001`, `BLK-UPDATE-001`, `PLY-BREAK-001`, `ITM-LOOT-001`,
`ITM-006`, `WGEN-PIPELINE-001`, `PROTO-PLAY-CLIENTBOUND-BLOCK-001`,
`PROTO-PLAY-CLIENTBOUND-SOUND-001` and `CLI-006`.
