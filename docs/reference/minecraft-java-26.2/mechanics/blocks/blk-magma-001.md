# Blocks mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `BLK-MAGMA-001` — Magma joins hot-floor damage to downward bubbles, reloadable selectors and generation

**Parent:** `BLK-001`, `BLK-002`, `BLK-003`, `BLK-005`, `PLY-002`, `PLY-006`, `ITM-004`,
`ITM-006`, `ENT-001`, `MOB-001`, `MOB-004`, `ENV-001`, `ENV-002`, `ENV-003`, `WGEN-003`,
`CLI-006`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked registration and hooks, every direct block/item tag consumer, bundled
recipes/loot/worldgen records, concrete feature and structure callers, generated reports and client
assets fix the sole state, physical behavior, selected roles, acquisition, generation and projection.

**Applies when:**

`minecraft:magma_block` is placed, broken, stepped on, queried for support/path/light/tag roles,
used under water, swallowed by a sulfur cube, consumed by a recipe, selected by world generation,
serialized or rendered.

**Authoritative state:**

Magma is a `MagmaBlock` extending `Block`, with one property-free default state, ID `14845`, and no
block entity. Registration supplies Nether map color, bass-drum note instrument, correct-tool
requirement, light emission `3`, strength/resistance `0.5`, an always-true emissive-rendering
predicate, and a spawn predicate accepting only fire-immune entity types. It leaves friction `0.6`,
speed and jump factors `1`, restitution `0`, piston reaction `NORMAL`, full selection/collision/
occlusion/support shapes, shade brightness `0.2`, light dampening `15`, no skylight propagation and
the ordinary true redstone-conductor, view-blocking and suffocating predicates.

Its common-rarity `BlockItem` stacks to `64`, has no special component/use gate and appears in the
locked functional- and natural-block creative-tab builders. Generic placement writes state `14845`.
The registered post-process callback returns the cell immediately above; successful
`WorldGenRegion#setBlock` marks that position unless flags contain bit `16`. Ordinary loaded-world
placement does not use that generation-region callback.

**Transition and ordering:**

Generic placement commits the property-free state before neighbor/fluid consequences. The concrete
step hook submits damage before calling its base hook; bubble updates are scheduled and executed by
their fluid owners; reload-selected tags/data are read by each consumer at its own transaction
boundary; worldgen features and structures write through their owner-defined traversals and flags.

#### Hot-floor step

`MagmaBlock#stepOn` first tests the entity's careful-step state. When it is not stepping carefully
and is a `LivingEntity`, the block submits exactly `1.0` damage from `hotFloor`; it then calls the
base step hook in every branch. Nonliving or careful entities submit no damage. The damage type has
burning effects, exhaustion `0.1`, message ID `hotFloor`, and belongs to `burn_from_stepping`,
`bypasses_shield`, `is_fire`, `no_knockback` and `panic_environmental_causes`.

Frost Walker's locked `damage_immunity` effect admits a source in `burn_from_stepping` only when it
does not bypass invulnerability, so it can reject this submitted hit. Fire immunity, invulnerability,
cooldown, defense, health, death and the ignored boolean result remain with `ITM-ENCHANT-001` and
`ENT-DAMAGE-001`; this leaf owns the caller, amount, source and careful/living gates.

#### Bubble-column base

Magma is the sole direct member of `enables_bubble_column_drag_down`. A full-source
`bubble_column_can_occupy` liquid above either valid bubble base schedules itself after `20` ticks
on placement, neighbor change or the eligible downward shape update; an existing bubble column can
schedule its update after `5` ticks. The due liquid tick delegates to
`BubbleColumnBlock.updateColumn`.

An admitted magma base selects the bubble default with `drag_down=true`. Existing bubble state is
preserved; removing every valid base turns an old bubble state back into water through the
consumer-owned fallback. The first flags-`2` write is followed by an upward walk through occupiable
cells, stopping on the first rejected upper write. Entity drag, particles and sounds belong to the
bubble-column owner.

#### Plants, fire, AI and geysers

Magma is the sole member of both `cannot_support_kelp` and `cannot_support_seagrass`.
`KelpBlock#canAttachTo` rejects the tagged block below directly. `SeagrassBlock#mayPlaceOn` requires
both a sturdy upper face and nonmembership, so magma is rejected despite its full support cube; the
tall-seagrass lower support path reaches the same rule.

Overworld and overworld-caves fire use `infiniburn_overworld`; Nether fire's tag includes it. Magma
is a direct member, so fire immediately above takes the infiniburn survival branch that bypasses
rain and self-removal. This does not make magma itself flammable or a spread target; the full fire
transaction belongs to `ENV-FIRE-001`.

`NodeEvaluator#isBurningBlock` recognizes exact magma independently of the fire tag, lava and lit
campfires, exposing it as a burning/danger input to path evaluation. In addition, careful
`Ghast$GhastMoveControl` rejects any traversed cell in `happy_ghast_avoids` before fluid handling;
magma is that tag's direct member. Non-careful Ghast movement does not test this tag. Path type,
malus and route selection remain with `MOB-AI-001`.

Magma is the sole `causes_periodic_geyser_eruptions` member. Potent sulfur immediately above it can
enter or preserve the periodic dormant/erupting family only through the source-water and captured-
state rules fixed by `ENV-GEYSER-001`; that owner retains countdown, random selection, eruption and
reset ordering.

#### Item-selected roles and acquisition

The magma item is the sole `sulfur_cube_archetype/hot` member. A sulfur cube matching that
archetype receives additive knockback and explosion-knockback resistance `-1`, additive bounciness
`0.5`, total-multiplied friction change `-0.699999988079071`, total-multiplied air-drag change
`-0.8999999985098839`, buoyancy, contact damage `1.0` from `sulfur_cube_hot` without scaling from
the source attribute, knockback powers `(0.4125, 0.09)`, and the hot hit/push sounds with cooldown
`0.7` and threshold `0.2`. Multi-match composition and actual damage/knockback remain with the
sulfur-cube, damage and knockback owners.

A shaped `2x2` square of four magma cream returns one magma block. Its recipe advancement unlocks
from possessing magma cream or already knowing the recipe. The block loot table returns itself
behind `survives_explosion`, using random sequence `blocks/magma_block`; there is no reverse recipe.
Correct-tool enforcement and `mineable/pickaxe` affect generic player harvesting.

#### Locked world generation

`ore_magma` replaces netherrack with size-`33` veins and air-exposure discard chance `0`. Its placed
feature tries count `4`, in-square positions and uniform absolute Y `27..36`, then applies the biome
filter; all five locked Nether biomes list it.

`underwater_magma` searches from an exact-water origin for a floor within configured range `5`.
Around the resulting floor it scans the inclusive radius-`1` cube. Every candidate first draws
`nextFloat() < 0.5`, then requires a current block that is neither exact water nor air and full
face-occlusion from below plus all four horizontal neighbors; the upper face is not tested. Each
admitted candidate writes default magma with flags `2` and ignores the write result. The feature
reports success when at least one candidate passed the filters, even if its write failed. Its
placed feature uses a uniform count `44..52`, in-square positions, uniform height from above-bottom
`0` through absolute `256`, an `OCEAN_FLOOR_WG` upper threshold `-2`, and the biome filter; `55`
locked biomes list it.

The Basalt Deltas `delta` configured feature uses lava contents and magma rim, uniform rim size
`0..2`, content size `3..7`, `count_on_every_layer=40` and the biome filter. Its concrete feature
first draws the `0.9` rim gate, samples each X/Z size independently, applies the hardcoded clear
test and writes rim/content identities; traversal and write order remain with `WGEN-PIPELINE-001`.

Non-cold ruined-portal spread and drip placements independently choose default magma with
`nextFloat() < 0.07`, otherwise netherrack, using flags `3`; cold portals never take the magma
branch. Drip continuation can add up to eight lower cells while successive `nextFloat() < 0.5`
tests pass. Full structure clipping, decay and mutation remain with
`WGEN-STRUCTURE-RUINED-PORTAL-001`.

Magma is also a hardcoded `BasaltColumnsFeature.CANNOT_PLACE_ON` identity: support on it rejects a
candidate and upward scanning aborts when encountering it. The Nether lava-spring configuration
admits magma among its five valid surrounding blocks. Finally, the bastion treasure bottom-rampart
processor replaces input magma with cracked polished blackstone bricks at probability `0.75`; its
sole locked pool entry is the rigid `bastion/treasure/ramparts/bottom_wall_0` element. Generic
feature, spring and jigsaw processor execution retain their dedicated owners.

The block adds no random/scheduled tick, use, attack, entity-inside, fall, neighbor, comparator or
block-event callback of its own.

**Client projection:**

The empty blockstate variant selects `block/magma_block`, a `cube_all` model using `block/magma` on
every face; the item selects the same model. Its texture animation has frame time `8`, interpolation
enabled and frames `0,1,2`. The registered emissive predicate makes client light-coordinate
resolution use packed full-bright `15728880`; this is distinct from authoritative world light
emission `3`, whose propagation still observes full-cube dampening `15`. There is no block entity,
conditional model, random variant or special renderer.

**Branches and aborts:**

Generic placement/break/explosion; generation bit-`16` postprocess suppression; careful/living/
damage-immunity gates; source/base/schedule/write bubble boundaries; plant/fire/path/Ghast/geyser
tag snapshots; archetype/recipe/loot/tool gates; every ore/underwater/delta/portal/basalt/spring/
bastion admission; and block/item rendering are distinct.

**Constants and randomness:**

State `14845`; strength/resistance `0.5`; friction `0.6`; speed/jump `1`; restitution `0`; emission
`3`; full-bright packed light `15728880`; dampening `15`; shade `0.2`; stack `64`; hot-floor damage
`1.0` and exhaustion `0.1`; bubble schedules `20`/`5` and flags `2`; sulfur-cube values above; recipe
count `1` from four inputs; ore size/count/height `33`/`4`/`27..36`; underwater range/radius/chance/
count `5`/`1`/`0.5`/`44..52`; delta rim/content/count and gate `0..2`/`3..7`/`40`/`0.9`; portal
chance/continuation/max `0.07`/`0.5`/`8`; processor chance `0.75`; animation frame time `8`.

**Side effects:**

Generic state placement/removal and self loot; generated-cell postprocess marking; hot-floor damage
submission; bubble scheduling and state writes; plant/fire/path/Ghast/geyser admission; sulfur-cube
attributes, physics, damage and sounds; recipe output; terrain/feature/structure writes; world-light
updates and full-bright block/item projection.

**Gates:**

Placement/removal authority; worldgen region/flags; entity type and careful-step state; live damage,
enchantment, tag, recipe, loot, archetype and worldgen snapshots; fluid amount/base; support/path/
fire/geyser predicates; feature coordinates and RNG; and client model/light context.

**Boundary cases and quirks:**

Magma has a full collider and ordinary support predicates, yet its spawn predicate admits only
fire-immune entity types and both aquatic plants reject it by tag. Client rendering is full-bright
while server light emission is only `3`. Underwater generation consumes its chance draw before
validity checks and may report success after a rejected write. Magma drives downward bubbles while
soul sand drives upward bubbles. Frost Walker protection is an enchantment-owned damage-type
immunity, not a branch inside `MagmaBlock`.

**Evidence:**

`OFF-SERVER-001`, `OFF-CLIENT-001`, `OFF-REPORT-001`, `OFF-DATA-001`;
`net.minecraft.world.level.block.Blocks`,
`net.minecraft.world.level.block.MagmaBlock#stepOn`,
`net.minecraft.world.level.block.state.BlockBehaviour$BlockStateBase#getPostProcessPos`,
`net.minecraft.server.level.WorldGenRegion#setBlock`,
`net.minecraft.world.level.block.LiquidBlock#tryScheduleBubbleBlockColumn`,
`net.minecraft.world.level.block.BubbleColumnBlock#updateColumn`,
`net.minecraft.world.level.block.BubbleColumnBlock#getColumnState`,
`net.minecraft.world.level.block.KelpBlock#canAttachTo`,
`net.minecraft.world.level.block.SeagrassBlock#mayPlaceOn`,
`net.minecraft.world.level.pathfinder.NodeEvaluator#isBurningBlock`,
`net.minecraft.world.entity.monster.Ghast$GhastMoveControl#canReach`,
`net.minecraft.world.entity.monster.Ghast$GhastMoveControl#blockTraversalPossible`,
`net.minecraft.world.entity.monster.cubemob.SulfurCube#matchingArchetypes`,
`net.minecraft.world.level.levelgen.feature.UnderwaterMagmaFeature#place`,
`net.minecraft.world.level.levelgen.feature.UnderwaterMagmaFeature#getFloorY`,
`net.minecraft.world.level.levelgen.feature.UnderwaterMagmaFeature#isValidPlacement`,
`net.minecraft.world.level.levelgen.feature.UnderwaterMagmaFeature#isWaterOrAir`,
`net.minecraft.world.level.levelgen.feature.UnderwaterMagmaFeature#isVisibleFromOutside`,
`net.minecraft.world.level.levelgen.Column#scan`,
`net.minecraft.world.level.levelgen.Column#scanDirection`,
`net.minecraft.world.level.levelgen.feature.DeltaFeature#place`,
`net.minecraft.world.level.levelgen.feature.BasaltColumnsFeature#canPlaceAt`,
`net.minecraft.world.level.levelgen.feature.BasaltColumnsFeature#findAir`,
`net.minecraft.world.level.levelgen.structure.structures.RuinedPortalPiece#placeNetherrackOrMagma`,
`net.minecraft.world.level.levelgen.structure.structures.RuinedPortalPiece#addNetherrackDripColumn`,
`net.minecraft.world.level.levelgen.structure.structures.RuinedPortalPiece#spreadNetherrack`,
`net.minecraft.util.LightCoordsUtil#getLightCoords`,
`net.minecraft.client.renderer.block.BlockModelResolver#update`,
`net.minecraft.world.item.CreativeModeTabs#bootstrap`;
`reports/blocks.json#minecraft:magma_block`,
`reports/minecraft/components/item/magma_block.json`,
`data/minecraft/tags/block/{cannot_support_kelp,cannot_support_seagrass,causes_periodic_geyser_eruptions,enables_bubble_column_drag_down,happy_ghast_avoids,infiniburn_overworld,mineable/pickaxe}.json`,
`data/minecraft/tags/item/sulfur_cube_archetype/hot.json`,
`data/minecraft/tags/damage_type/{burn_from_stepping,bypasses_shield,is_fire,no_knockback,panic_environmental_causes,sulfur_cube_with_block_immune_to}.json`,
`data/minecraft/damage_type/hot_floor.json`,
`data/minecraft/enchantment/frost_walker.json`,
`data/minecraft/sulfur_cube_archetype/hot.json`,
`data/minecraft/loot_table/blocks/magma_block.json`,
`data/minecraft/recipe/magma_block.json`,
`data/minecraft/advancement/recipes/building_blocks/magma_block.json`,
`data/minecraft/worldgen/{configured_feature/ore_magma,configured_feature/underwater_magma,configured_feature/delta,configured_feature/spring_lava_nether,placed_feature/ore_magma,placed_feature/underwater_magma,placed_feature/delta}.json`,
`data/minecraft/worldgen/processor_list/bottom_rampart.json`,
`data/minecraft/worldgen/template_pool/bastion/treasure/ramparts.json`,
`assets/minecraft/blockstates/magma_block.json`,
`assets/minecraft/models/block/magma_block.json`,
`assets/minecraft/items/magma_block.json`,
`assets/minecraft/textures/block/magma.png.mcmeta`.

**Test vectors:**

Run `EXP-BLK-038` across placement/break/explosion, physical/light/spawn/postprocess queries,
careful/noncareful living/nonliving step and immunity branches, bubble scheduling/writes, every
block/item-tag consumer before and after reload, sulfur-cube composition, recipe/loot/tool paths,
all generation roles, save/reload, emitted-world-light and full-bright block/item rendering. Assert
state, callbacks, damage calls/results, schedules/writes, predicates, RNG order, outputs, generated
positions and light/model selection.

**Limits:**

This leaf does not re-specify generic placement/breaking, damage resolution, enchantment iteration,
fluid/bubble entity behavior, fire propagation, plant growth, pathfinding, geyser timing,
sulfur-cube composition, loot/recipe allocation, worldgen pipelines, structure placement, state
packets or model loading. Those remain with `BLK-002`, `ENT-DAMAGE-001`, `ITM-ENCHANT-001`,
`ENV-FLUID-001`, `ENV-FIRE-001`, plant owners, `MOB-AI-001`, `ENV-GEYSER-001`,
`ENT-KNOCKBACK-001`, `ITM-LOOT-001`, `ITM-RECIPE-001`, `WGEN-PIPELINE-001`, dedicated structure
leaves and `CLI-006`.
