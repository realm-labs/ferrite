# Blocks mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `BLK-SOUL-SAND-001` — Soul sand joins reduced collision to fire, plants, movement, structures and reloadable selectors

**Parent:** `BLK-001`, `BLK-002`, `BLK-003`, `BLK-005`, `PLY-002`, `PLY-006`, `ITM-004`,
`ITM-006`, `ENT-001`, `MOB-001`, `MOB-004`, `ENV-001`, `ENV-002`, `ENV-003`, `WGEN-003`,
`CLI-006`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked registration and concrete hooks, every direct block/item tag consumer,
world-generation identities, generated reports/data and client assets fix the sole state, physical
surface, reload-selected roles, acquisition, generation and projection.

**Applies when:**

`minecraft:soul_sand` is placed, broken, collided with, queried for support/path/fire/plant/tag
roles, used under water, used by an enchanted mover or sulfur cube, consumed by a recipe, selected
by loot/world generation, serialized or rendered.

**Authoritative state:**

Soul sand is a `SoulSandBlock` extending `Block`, with one property-free default state, ID `6998`,
and no block entity. Registration supplies brown map color, cow-bell note instrument, strength
`0.5`, speed factor `0.4` and soul-sand sounds. It leaves friction `0.6`, jump factor `1`,
restitution `0`, collision and occlusion enabled, and piston reaction `NORMAL`.

Selection and occlusion shapes are full cubes. Collision is a full-X/Z column from Y `0` through
`14/16`; visual and block-support shapes are explicitly full cubes. Shade brightness is `0.2`,
the full occlusion caches light dampening `15`, and the full selection does not propagate skylight.
Registered predicates return true for every spawn entity and make the state redstone-conducting,
view-blocking and suffocating despite the shortened collider. Every path-computation type is
rejected. Generic entity ground movement consumes speed factor `0.4`; there is no `entityInside`,
step or fall override.

Its ordinary common-rarity `BlockItem` stacks to `64`, has no special component/use gate and is
listed in the natural-blocks creative tab. Generic placement writes state `6998`.

**Transition and ordering:**

The registered post-process callback returns the cell immediately above soul sand. A successful
`WorldGenRegion#setBlock` invokes it and marks that returned position unless placement flags contain
bit `16`; ordinary loaded-world placement does not use this generation-region path.

#### Bubble-column base

Soul sand is the sole direct member of `enables_bubble_column_push_up`. On placement or neighbor
change, a `LiquidBlock` that is a full source in `bubble_column_can_occupy` schedules itself after
`20` ticks when the state below belongs to either bubble-base tag. A downward shape update uses the
same current-fluid gate and base test. The due liquid tick calls `BubbleColumnBlock.updateColumn`
with the below state; an existing bubble-column neighbor can independently schedule its own update
after `5` ticks.

Column update first requires the target already be the bubble block, or be a full source liquid
block in the occupiable-fluid tag with amount at least `8`. A soul-sand base selects the bubble
default with `drag_down=false`. The first write uses flags `2` and ignores its result; the updater
then walks upward while cells are occupiable, writes the same state with flags `2`, and stops on the
first rejected upper write. Existing bubble state is preserved; removing every valid base turns an
old bubble state back into water through the consumer-owned fallback. Entity lift, particles and
sounds belong to the bubble-column owner, not to soul sand.

#### Fire, plants, snow and summoning

`BaseFireBlock#getState` chooses soul fire when the state below is in `soul_fire_base_blocks`, and
`SoulFireBlock.canSurviveOnBlock` tests that same tag directly. Soul sand is one of its two locked
members. Nether wart's `mayPlaceOn` tests `supports_nether_wart`, whose only member is soul sand;
wither rose tests `supports_wither_rose`, where soul sand is one admitted identity. Plant update and
growth transactions remain with their plant owners.

Snow-layer survival first rejects `cannot_support_snow_layer`, then immediately accepts
`support_override_snow_layer`. Soul sand takes that override branch. This is observable rather than
redundant: the later geometric fallback reads the shortened **collision** shape, not soul sand's
full support shape, so its upper face is not full.

Both the pre-placement and complete wither patterns use `wither_summon_base_blocks` for every `#`
cell in the `###`/`~#~` base. Soul sand is therefore accepted for either the three-block arm or the
center stem. Skull admission, destructive clearing, entity/criterion order and failure behavior
remain with `BLK-SKULL-001`.

#### Soul Speed, dried ghast and sculk

Soul sand is one of two `soul_speed_blocks`. The locked Soul Speed definition uses that tag in both
location-changed effects and both tick effects. At level `L`, an admitted location effect adds
`0.0405 + 0.0105 * (L - 1)` movement speed and `1` movement efficiency; a separate chance-first
effect damages the feet item by one with probability `0.04 * L` when on ground and movement is
affected by the tagged block. The attribute branch excludes vehicles and flying entities and
preserves its active/inactive and airborne rules from data. Every fifth tick, grounded nonflying
movement with horizontal speed at least `9.999999747378752E-6` over the tag emits the configured
soul particle. The following sound effect tests chance `0.35` before that movement predicate; on
success it plays `particle.soul_escape` at volume `0.6` and uniform pitch `[0.6,1.0)`. Equipment
iteration, predicate execution, RNG and attribute lifetime remain with `ITM-ENCHANT-001`.

For an unwaterlogged dried ghast, each client animation tick first draws `nextInt(40)`; result zero
plays `dried_ghast.ambient` locally at volume/pitch `1` only when the block immediately below is in
`triggers_ambient_dried_ghast_block_sounds`. Soul sand is one of two members. The independent
one-in-six smoke branch follows. A waterlogged dried ghast instead uses its water ambient/sparkle
branches and never tests this block tag.

The ordinary level sculk spreader receives `sculk_replaceable`; the world-generation spreader uses
`sculk_replaceable_world_gen`, which includes that base tag. Sculk-vein replacement probing also
tests `sculk_replaceable` beside an attached face. Soul sand is therefore a replaceable target in
both spreader modes and for that vein probe; charge movement, conversion and write order remain
with the sculk owners.

#### Item-selected roles and acquisition

The soul-sand item is one of two `soul_fire_base_blocks` ingredients. The shaped soul-torch recipe
puts coal/charcoal over a stick over that tag and returns four torches; the shaped soul-campfire
recipe puts the tag at center, three sticks around it and three logs below. A separate 3x3 dried
ghast recipe consumes one soul sand surrounded by eight ghast tears and returns one dried ghast.

The item also belongs to `sulfur_cube_archetype/high_resistance`. A sulfur cube filters every
archetype's holder set against the swallowed/body stack, so soul sand matches this archetype. Its
locked record adds `0.7` knockback resistance, `0.7` explosion-knockback resistance, `0.2`
bounciness, zero total-multiplied friction change and `-0.9900000002235174` total-multiplied air
drag; it supplies knockback powers `(0.4125, 0.09)` and the high-resistance hit/push sounds.
Multiple-match composition and the last matching knockback/sound selection remain with the sulfur
cube and `ENT-KNOCKBACK-001` owners.

The block loot table offers one soul-sand item behind `survives_explosion`. Piglin bartering has a
weight-`40` soul-sand entry whose uniform count provider is `2..8`; the hoglin-stable bastion chest
has a soul-sand entry with count `2..7` inside a `3..4`-roll pool. Pool selection, numeric-provider
rounding, luck, stack insertion and recipe allocation remain generic.

#### Locked world generation

The Nether noise settings select soul sand in three surface-rule branches. In soul-sand valley,
ceiling and floor stone-depth branches choose it when `nether_state_selector >= 0`, after the
floor's higher-priority patch/gravel branch and before the soul-soil fallback. In Nether wastes,
the floor branch requires `soul_sand_layer >= -0.012`, not-hole, and the nested absolute-`30`/
`35` Y checks before selecting soul sand; otherwise its local sequence falls back to netherrack.
The referenced noise record has first octave `-8` and amplitudes
`[1,1,1,1,0,0,0,0,0.013333333333333334]`.

The `ore_soul_sand` ore configuration replaces netherrack with veins of size `12` and air-exposure
discard chance `0`. Its placed feature tries count `12`, in-square positions and a uniform height
from above-bottom `0` through absolute `31`, then applies the biome filter; soul-sand valley lists
it in underground ores. The Nether lava-spring configuration also admits soul sand as one of five
valid surrounding blocks.

Fortress castle-stalk rooms write default soul sand in local boxes X `3..4` and `8..9`, Y `4`,
Z `4..8`, then default nether wart directly above. Nether-fossil anchor scanning short-circuits its
sturdy-face test when the lower state is exact soul sand. Conversely, basalt columns include exact
soul sand in `CANNOT_PLACE_ON`: `canPlaceAt` rejects a candidate supported by it and `findAir`
aborts when its upward scan encounters it. These complete algorithms remain with
`WGEN-STRUCTURE-FORTRESS-001`, `WGEN-STRUCTURE-NETHER-FOSSIL-001` and `WGEN-PIPELINE-001`.

Finally, the locked `nether_cave` carver can replace soul sand through
`nether_carver_replaceables`. Ordinary and worldgen sculk replacement are the distinct tag paths
above. Carving, surface evaluation, feature placement and generation-region postprocessing retain
their owner-defined RNG, traversal, clipping and write behavior.

The block adds no scheduled/random tick, use, attack, entity-inside, step, fall, neighbor,
comparator or block-event callback of its own.

**Client projection:**

The sole empty blockstate variant selects `block/soul_sand`, a `cube_all` model using the
`block/soul_sand` texture on every face. The item definition selects the same model. The rendered
cube is full height even though collision stops at `14/16`; ordinary cull-face and ambient-occlusion
rules apply. Terrain updates project state `6998`; no block entity, conditional model, random
variant or special renderer is involved. Dried-ghast ambient sound is the separate client-side tag
consumer described above.

**Branches and aborts:**

Generic placement/break/explosion; worldgen bit-`16` postprocess suppression; water source/base/
write gates; fire/plant/snow/wither tag snapshots; Soul Speed active/vehicle/flying/ground/movement/
chance/period branches; dried-ghast waterlogged/chance/below-state branches; ordinary/worldgen
sculk spread; each item recipe/archetype/loot source; every surface-rule/ore/spring/carver/structure
admission; and block/item rendering are distinct.

**Constants and randomness:**

State ID `6998`; strength/resistance `0.5`; friction `0.6`; speed `0.4`; jump `1`; restitution `0`;
collision Y `0..14/16`; dampening `15`; shade `0.2`; stack `64`; bubble scheduling `20`/`5` ticks
and update flags `2`; Soul Speed level cap `3`, speed `0.0405 + 0.0105*(L-1)`, efficiency `1`,
durability chance `0.04*L`, period `5`, movement threshold `9.999999747378752E-6`, sound chance
`0.35`, volume `0.6`, pitch `[0.6,1.0)`; dried-ghast sound chance `1/40`; recipe counts `4` and
`1`; barter weight/count `40`/`2..8`; chest rolls/count `3..4`/`2..7`; surface thresholds `0` and
`-0.012`; ore size/count `12`/`12` and heights `0..31`. Consumer-owned effects and worldgen/loot
transactions retain the additional RNG specified by their owners.

**Side effects:**

Generic state placement/removal and self loot; ground movement slowdown; generated-cell
postprocessing mark; bubble-column state writes; fire/plant/snow/wither admission; enchanted
attributes, durability, particles and sounds; dried-ghast local sound; sculk replacement; recipes,
sulfur-cube equipment effects, barter/chest outputs; terrain/carver/feature/structure writes; and
ordinary state/item/model projection.

**Gates:**

Placement/removal authority; collision/support/path query; worldgen region/flags; fluid amount and
base tag; current block/item/tag/enchantment/recipe/loot/registry snapshots; entity equipment,
movement and client/server context; structure/feature/carver admission and coordinates; and client
model/animation context.

**Boundary cases and quirks:**

The visible/selection/support cube is full while collision is two pixels short. Explicit predicates
still accept spawn, conduction, suffocation and view blocking, but every path type is rejected.
Snow needs its tag override because it tests collision, not support shape. Soul sand pushes bubble
columns up while a distinct tag drives downward columns. Soul Speed's sound and durability chance
tests precede their state predicates in locked data. A dried ghast tests the below tag only while
unwaterlogged. Soul sand is both selected by several Nether generators and protected from basalt-
column anchoring. Generated soul sand marks the cell above for postprocessing unless flag `16`
suppresses that callback.

**Evidence:**

`OFF-SERVER-001`, `OFF-CLIENT-001`, `OFF-REPORT-001`, `OFF-DATA-001`;
`net.minecraft.world.level.block.Blocks`,
`net.minecraft.world.level.block.SoulSandBlock#getCollisionShape`,
`net.minecraft.world.level.block.SoulSandBlock#getBlockSupportShape`,
`net.minecraft.world.level.block.SoulSandBlock#getVisualShape`,
`net.minecraft.world.level.block.SoulSandBlock#isPathfindable`,
`net.minecraft.world.level.block.SoulSandBlock#getShadeBrightness`,
`net.minecraft.world.level.block.state.BlockBehaviour$BlockStateBase#getPostProcessPos`,
`net.minecraft.server.level.WorldGenRegion#setBlock`,
`net.minecraft.world.level.block.LiquidBlock#onPlace`,
`net.minecraft.world.level.block.LiquidBlock#neighborChanged`,
`net.minecraft.world.level.block.LiquidBlock#updateShape`,
`net.minecraft.world.level.block.LiquidBlock#tick`,
`net.minecraft.world.level.block.LiquidBlock#tryScheduleBubbleBlockColumn`,
`net.minecraft.world.level.block.BubbleColumnBlock#updateColumn`,
`net.minecraft.world.level.block.BubbleColumnBlock#canOccupy`,
`net.minecraft.world.level.block.BubbleColumnBlock#getColumnState`,
`net.minecraft.world.level.block.BaseFireBlock#getState`,
`net.minecraft.world.level.block.SoulFireBlock#canSurviveOnBlock`,
`net.minecraft.world.level.block.NetherWartBlock#mayPlaceOn`,
`net.minecraft.world.level.block.WitherRoseBlock#mayPlaceOn`,
`net.minecraft.world.level.block.SnowLayerBlock#canSurvive`,
`net.minecraft.world.level.block.WitherSkullBlock#checkSpawn`,
`net.minecraft.world.level.block.WitherSkullBlock#canSpawnMob`,
`net.minecraft.world.level.block.WitherSkullBlock#getOrCreateWitherFull`,
`net.minecraft.world.level.block.WitherSkullBlock#getOrCreateWitherBase`,
`net.minecraft.world.level.block.DriedGhastBlock#animateTick`,
`net.minecraft.world.level.block.SculkSpreader#createLevelSpreader`,
`net.minecraft.world.level.block.SculkSpreader#createWorldGenSpreader`,
`net.minecraft.world.level.block.SculkVeinBlock#hasSubstrateAccess`,
`net.minecraft.world.item.Item$Properties#shovel`,
`net.minecraft.world.entity.monster.cubemob.SulfurCube#matchingArchetypes`,
`net.minecraft.world.level.levelgen.feature.BasaltColumnsFeature#canPlaceAt`,
`net.minecraft.world.level.levelgen.feature.BasaltColumnsFeature#findAir`,
`net.minecraft.world.level.levelgen.structure.structures.NetherFossilStructure#findGenerationPoint`,
`net.minecraft.world.level.levelgen.structure.structures.NetherFortressPieces$CastleStalkRoom#postProcess`,
`net.minecraft.world.item.CreativeModeTabs#bootstrap`;
`reports/blocks.json#minecraft:soul_sand`,
`reports/minecraft/components/item/soul_sand.json`,
`data/minecraft/tags/block/{enables_bubble_column_push_up,mineable/shovel,nether_carver_replaceables,sculk_replaceable,soul_fire_base_blocks,soul_speed_blocks,support_override_snow_layer,supports_nether_wart,supports_wither_rose,triggers_ambient_dried_ghast_block_sounds,wither_summon_base_blocks}.json`,
`data/minecraft/tags/item/soul_fire_base_blocks.json`,
`data/minecraft/tags/item/sulfur_cube_archetype/high_resistance.json`,
`data/minecraft/enchantment/soul_speed.json`,
`data/minecraft/sulfur_cube_archetype/high_resistance.json`,
`data/minecraft/loot_table/blocks/soul_sand.json`,
`data/minecraft/loot_table/gameplay/piglin_bartering.json`,
`data/minecraft/loot_table/chests/bastion_hoglin_stable.json`,
`data/minecraft/recipe/{soul_torch,soul_campfire,dried_ghast}.json`,
`data/minecraft/worldgen/{configured_carver/nether_cave,configured_feature/ore_soul_sand,configured_feature/spring_lava_nether,placed_feature/ore_soul_sand,biome/soul_sand_valley,noise/soul_sand_layer,noise_settings/nether}.json`,
`assets/minecraft/blockstates/soul_sand.json`,
`assets/minecraft/models/block/soul_sand.json`,
`assets/minecraft/items/soul_sand.json`.

**Test vectors:**

Run `EXP-BLK-037` across placement/break/explosion, all shape/light/predicate/path queries,
worldgen postprocess flags, bubble source/base/update/write boundaries, all eleven block-tag and two
item-tag consumers before/after reload, Soul Speed levels/movement/RNG, dried-ghast animation RNG,
recipes/archetype/loot, each Nether surface/ore/spring/carver/fortress/fossil/basalt branch,
save/reload and block/item rendering. Assert state, shapes, movement, schedules/writes, predicates,
RNG order, effects, outputs, generated positions and model selection.

**Limits:**

This leaf does not re-specify generic placement/breaking, movement physics, light propagation,
fluid/bubble entity behavior, fire spread, plant growth, snow updates, wither summoning, enchantment
iteration, sulfur-cube composition, sculk charge propagation, loot/recipe allocation, worldgen
pipelines, state packets or model loading. Those remain with `BLK-002`, `PLY-COLLISION-001`,
`ENV-LIGHT-001`, `ENV-FLUID-001`, `ENV-FIRE-001`, plant owners, `BLK-SKULL-001`,
`ITM-ENCHANT-001`, `ENT-KNOCKBACK-001`, sculk owners, `ITM-LOOT-001`, `ITM-RECIPE-001`,
`WGEN-PIPELINE-001`, the dedicated structure leaves and `CLI-006`.
