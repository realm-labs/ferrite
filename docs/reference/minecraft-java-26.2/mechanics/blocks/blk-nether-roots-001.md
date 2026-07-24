# Blocks mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `BLK-NETHER-ROOTS-001` — Nether roots share support, potting, Enderman and forest-vegetation behavior

**Parent:** `SIM-003`, `SIM-004`, `SIM-005`, `SIM-RANDOM-001`, `BLK-001`, `BLK-002`,
`BLK-003`, `BLK-005`, `PLY-002`, `PLY-005`, `PLY-006`, `ITM-004`, `ITM-006`, `MOB-004`,
`MOB-005`, `ENV-003`, `WGEN-002`, `WGEN-003`, `CLI-001`, `CLI-006`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked registrations and reports, complete loot/tag/worldgen data, the closed
server/client class-reference set, all 1,212 decoded structure templates and exact client assets
close both property-free root blocks and their two property-free potted forms. The roots share one
support state machine and join composting, combined footsteps, enchanting transmission,
tree/mushroom replacement and Enderman carrying; their colors, IDs, generation weights and
acquisition differ. The potted forms close the code-built insertion/extraction map, loot and
separate pot projection.

**Applies when:**

`minecraft:{warped_roots,crimson_roots,potted_warped_roots,potted_crimson_roots}` is placed,
replaced, loses support, harvested, exploded, composted, walked through, carried by an Enderman,
inserted into or removed from a flower pot, selected by loot or world generation, persisted,
mapped or rendered.

**Authoritative state:**

Warped and crimson roots are property-free `NetherRootsBlock`/`VegetationBlock` instances with
codec type `minecraft:nether_roots`, no block entity and sole states `20960` and `21031`. Their
block protocol IDs are `869` and `882`; their ordinary block-item raw IDs are `280` and `279`.
Both registrations use the default `HARP` note instrument, zero hardness/resistance, no collision,
replaceability, piston reaction `DESTROY`, deterministic `XZ` positional offset and `ROOTS`
sound type. Warped roots use map color `COLOR_CYAN`; crimson roots use `NETHER`.

The unoffset root selection shape is the centered column `(2,0,2)..(14,13,14)` in sixteenths.
For block position `(x,y,z)`, `XZ` offset computes `seed=Mth.getSeed(x,0,z)`,
`dx=((seed&15)/15-0.5)*0.5` and `dz=(((seed>>8)&15)/15-0.5)*0.5`; both lie in the configured
`[-0.25,0.25]` clamp and Y offset is zero. Collision and occlusion are empty, emission and light
dampening are zero, skylight propagates through the empty fluid state, and AIR pathfinding is
allowed. Neither root is sturdy, suffocating, view-blocking, signal-producing or a spawn surface,
and neither adds random/scheduled tick, use, attack, entity-contact, comparator or block-event
dispatch.

The shared root sound type has volume/pitch `1/1` and selects sound registry IDs break `688`, step
`689`, place `690`, hit `691` and fall `692`. Each ordinary item is common, stacks to `64`, has
standard block-item components and belongs to no direct item tag.

The potted counterparts are `FlowerPotBlock` states `21829` warped and `21828` crimson, with block
protocol IDs `922` and `921`, no block entities and no corresponding item registrations.
`flowerPotProperties` gives them zero hardness/resistance, no occlusion, piston reaction `DESTROY`,
default `NONE` map color, `HARP` instrument and `STONE` sound type. Their centered
`(5,0,5)..(11,6,11)` shape is both selection and collision; pathfinding is false for every mode.
They have no support requirement, so the downward-neighbor recheck retains them even when floating.
They are not randomly ticking, sturdy full faces, spawn floors, signals or comparators. Stone
volume/pitch is `1/1` and break/step/place/hit/fall sound IDs are
`1596/1604/1601/1600/1599`.

**Transition and ordering:**

#### Root placement, support loss, replacement and loot

The two constructor instances hold distinct tags, `supports_warped_roots` and
`supports_crimson_roots`, but crimson's tag nests warped's tag and both currently expand to the
same exact 14 identities: dirt, coarse dirt, rooted dirt; mud, muddy mangrove roots; moss block,
pale moss block; grass block, podzol, mycelium; farmland; crimson nylium, warped nylium; and soul
soil. Ordinary root placement admits its sole state only over that closure. Rotation and mirror
are identity operations.

Every root neighbor-shape update rechecks the block below. Failure returns AIR; ordinary server
`updateOrDestroy` destroys the old root and evaluates its loot unless the caller suppresses drops.
With no explosion radius, the root's `survives_explosion` condition passes, so support loss creates
one matching root item. Forced state/component writes may leave an unsupported root until a later
qualifying update.

The code-built replaceable property lets a different held block item replace a root in place; an
empty placement context also passes, while holding that same root item does not. Fluid replacement
is admitted, but roots have no waterlogged property or retained fluid state. Piston movement
destroys rather than moves them. Independently, direct `replaceable_by_trees` membership admits
them through `TreeFeature.validTreePos`, and direct `replaceable_by_mushrooms` membership lets the
ordinary huge-mushroom writer overwrite them.

Each root's one-roll loot table offers one matching item through `survives_explosion`, using random
sequence `minecraft:blocks/<root>`. Ordinary tools, bare hands and tool-free support loss therefore
drop one without Silk Touch or Fortune; an explosion admits the one item with the generic
`1/explosion_radius` draw. No bundled recipe consumes or produces either root, no recipe
advancement references them, and only crimson roots has an additional chest acquisition: the
hoglin-stable bastion table includes count `2..7` as one of 14 equal entries in its uniform
`3..4`-roll second pool.

#### Composting, footsteps, enchanting and Endermen

`ComposterBlock` registers both items at chance `0.65f`. Player insertion at level `0` succeeds
without RNG; levels `1..6` consume one `nextDouble()` and increment exactly when it is below
`0.65`. Success writes level plus one with flags `3`, emits `BLOCK_CHANGE`, and `6 -> 7` schedules
maturation after `20` ticks; failure preserves state. Either level-`0..6` result emits event
`1500`, awards the used-item statistic and calls `consume(1, player)`, preserving
infinite-material holders. Level `7` succeeds without insertion or consumption; level `8` falls
through to ordinary item-on-block handling. Automation admits only below level `7`, uses the same
first-level/RNG transition and always shrinks one item, including after chance failure.

Direct membership in `combination_step_sound_blocks` makes a walking player inside either root use
it as the primary step block. Outside the water-specific branch, the player emits roots step sound
`689` at volume `0.15` and pitch `1`, then the supporting block's step sound at
`supportVolume*0.05` and `supportPitch*0.8`. The direct `replaceable` block tag is nested by
`enchantment_power_transmitter`, so a root at the halfway offset between an enchanting table and
an `enchantment_power_provider` transmits that provider after the table's far-provider check.

Both roots are direct `enderman_holdable` members. An empty-handed Enderman with `mobGriefing`
enabled tests `nextInt(reducedTickDelay(20))`, therefore `nextInt(10)==0`, before one take attempt.
That attempt samples X/Z as `floor(position-2+nextDouble()*4)` and Y as
`floor(positionY+nextDouble()*3)`, then clips from the Enderman's block-column center at the sampled
Y to the candidate center. A tagged candidate whose ray hit resolves to itself is removed with
drops disabled, emits `BLOCK_DESTROY` with Enderman/state context, and stores the matching default
root state as carried state; the ignored removal result does not gate those latter effects.

A carrying Enderman under the same gamerule tests `nextInt(reducedTickDelay(2000))`, hence
`nextInt(1000)==0`, before sampling X/Z in `position-1+[0,2)` and Y in `positionY+[0,2)`.
`Block.updateFromNeighbourShapes` first transforms the carried state at the target. Placement then
requires target air, a nonair/non-bedrock full-collision block below, transformed-state survival
and no entity in the unit target box. A supported full-collision substrate writes the root with
flags `3`, emits `BLOCK_PLACE` and clears carried state. A full block outside the support tags
transforms the root to AIR before those gates; because AIR survives, the same commit clears the
carried root without placing it. A tagged but non-full-collision substrate, such as farmland,
fails the separate full-block gate and retains the carried state. Goal scheduling, persistence,
death drops and packet projection remain with their mob/entity owners.

#### Flower-pot transitions and potted loot

Static construction maps each root block to its corresponding potted block. Using a root
`BlockItem` on an empty flower pot writes potted state `21829` or `21828` with flags `3`, emits
`BLOCK_CHANGE`, awards `Stats.POT_FLOWER`, consumes one through the player-aware infinite-material
path and returns success. The method emits no explicit sound. A pottable item used on an already
occupied pot returns consume without mutation or item consumption. A nonpottable held item returns
`TRY_WITH_EMPTY_HAND`, leaving the generic interaction dispatcher to reach the empty-hand path.

Empty-hand use on either potted root constructs one matching root stack, tries the player's
inventory and otherwise drops that stack with the nonrandom throw mode. It then writes the empty
flower-pot state with flags `3`, emits `BLOCK_CHANGE` and returns success, without awarding the
potting statistic. Clone-pick returns the root item rather than a flower pot. The potted blocks'
two independent `survives_explosion` loot pools offer one flower pot and one matching root; normal
break produces both, while an explosion independently admits each pool and can produce neither,
either or both. Their random sequences are `minecraft:blocks/potted_<root>`. Direct membership in
`flower_pots` has no other locked runtime consumer.

#### Reload-selected world generation

The natural crimson-forest vegetation record has width/height `8/4` and a total weight of `99`:
crimson roots occupy `nextInt(99)` results `0..86`, crimson fungus `87..97`, and warped fungus
`98`. Its placed wrapper applies `count_on_every_layer(6)` then biome filtering in the crimson
forest vegetation step. The natural warped record uses width/height `8/4`, total weight `100` and
ordered intervals warped roots `0..84`, crimson roots `85`, warped fungus `86..98`, crimson fungus
`99`; its wrapper uses layer count `5` in warped forest vegetation.

The shared Nether-forest kernel requires nylium below the origin and vertical admission, performs
`width*width=64` attempts, consumes six positional draws per attempt, then samples the weighted
provider before empty/min-Y/survival gates. An admitted state is offered with flags `2`; return is
whether at least one offer occurred, independent of write results. The bonemeal counterparts use
the same ordered weights at width/height `3/1`, hence nine attempts. Crimson nylium invokes only
the crimson bonemeal record. Warped nylium invokes the warped record, then nether sprouts, then its
one-in-eight optional twisting-vines branch. Neither root is directly bonemealable.

Separately, `patch_crimson_roots` occurs in soul-sand valley's underground-ores step and references
a fixed-state `simple_block` crimson-roots feature. Its ordered placement modifiers are a uniform
full-build-height sample, biome filter, count `96`, independent triangular X/Z offsets `[-7,7]`
and Y offset `[-3,3]`, then an air predicate. Each triangular component is the difference of two
uniform draws over `0..7` or `0..3`. The simple feature selects state `21031`, checks the 14-member
support closure at the candidate, offers flags `2`, ignores the write result and reports true on
survival. There is no corresponding warped-roots patch. The exhaustive NBT scan finds zero cells
of all four states across all 1,212 structure templates.

**Client projection:**

Each root's sole blockstate selects its matching untinted `minecraft:block/<root>` cross model.
Ambient occlusion is disabled and both crossed planes are shade-disabled; the deterministic XZ
state offset shifts this projection. The item selector uses a flat generated model whose layer is
the same block texture. Potted blockstates instead select `potted_<root>` models: the fixed
flower-pot/dirt geometry plus two shade-disabled crossed planes from the dedicated
`<root>_pot` texture, without positional offset or an item model.

Block updates publish states `20960/21031/21829/21828`, inventory paths use root item IDs
`280/279`, root material sounds use `688..692`, pot material sounds use the five stone IDs, and
map projection uses `COLOR_CYAN`/`NETHER`/`NONE`. This leaf adds no packet field,
acknowledgement or connection-local state.

**Branches and aborts:**

Two root identities and two potted identities; ordinary/forced placement; 14 supports versus other
blocks; support retained/lost and drop suppression; same/different item and fluid replacement;
tool, support-loss and explosion loot; finite/infinite player and automation at composter levels
`0`, `1..6`, `7`, `8`; walking combination versus water/other step paths; enchanting join;
Enderman gamerule/hand/draw/ray/tag/removal and placement transform/full-block/support/entity
gates; empty/occupied pot with root/other-pottable/nonpottable/empty hand, inventory-full fallback,
clone and potted explosion pools; crimson/warped natural and bonemeal weights; crimson patch
placement gates; tree/mushroom overwrite; zero template selection; persistence and client
projection are distinct branches.

**Constants and randomness:**

Root states `20960/21031`, root block IDs `869/882`, root item IDs `280/279`; potted states
`21829/21828`, block IDs `922/921`; root selection `(2,0,2)..(14,13,14)`, pot shape
`(5,0,5)..(11,6,11)`; XZ mask/divisor/scale/clamp `15/15/0.5/0.25`; support identities `14`;
strength `0/0`; root sounds `688/689/690/691/692`, pot sounds
`1596/1604/1601/1600/1599`; stack `64`; composter chance `0.65`, maturation `20`, event `1500`;
Enderman bounds `10/1000`, take spans `4/3/4`, leave spans `2/2/2`; vegetation natural
width/height/attempts `8/4/64`, bonemeal `3/1/9`, totals `99/100`, layer counts `6/5`; patch count
`96`, offsets `7/3`; templates/cells `1212/0`.

**Side effects:**

Root placement/support-loss item creation/replacement; root and potted loot; composter transition,
game event, schedule and level event; combined footsteps; enchanting admission; Enderman
drop-suppressed removal, carried state, block events and placement/discard; pot state/stat/item
consume/inventory/drop/game event; chest item generation; natural/bonemeal/patch/tree/mushroom
writes; ordinary persistence; map, sound, offset cross, flat-item and potted-model projection.

**Gates:**

World-write/break authority; active support/replacement/Enderman/loot tags; explosion radius;
placement context; composter level/input/RNG/infinite-material policy; player step and enchanting
contexts; Enderman carried state/gamerule/RNG/ray/full-block/support/entities; pot occupancy and
held item; biome/feature/build-height/provider/candidate admission; registry, map, sound and
client-resource context.

**Boundary cases and quirks:**

The two nominal support tags currently have identical closure but remain separately reloadable.
Code-built replaceability, reloadable `replaceable`, tree replacement and mushroom replacement are
four distinct gates. Unsupported ordinary roots drop themselves, unlike shears-gated nether
sprouts; Enderman taking explicitly suppresses that drop. Enderman pre-placement neighbor
transformation can turn an unsupported carried root into AIR and then clear it. Pot insertion and
empty-hand extraction do not explicitly play sounds; normal potted loot yields both container and
plant, and its explosion tests are independent. Crimson roots is a one-percent warped-vegetation
choice and has both a soul-sand-valley patch and bastion-chest acquisition; warped roots has
neither latter path. Zero structure cells does not imply absence from biome generation.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-REPORT-001`; `OFF-DATA-001`;
`net.minecraft.world.level.block.Blocks`;
`net.minecraft.world.level.block.SoundType`;
`net.minecraft.world.level.block.NetherRootsBlock#getShape`;
`net.minecraft.world.level.block.NetherRootsBlock#mayPlaceOn`;
`net.minecraft.world.level.block.VegetationBlock#updateShape`;
`net.minecraft.world.level.block.VegetationBlock#canSurvive`;
`net.minecraft.world.level.block.state.BlockBehaviour$Properties#offsetType`;
`net.minecraft.world.level.block.state.BlockBehaviour#getMaxHorizontalOffset`;
`net.minecraft.world.level.block.state.BlockBehaviour#canBeReplaced`;
`net.minecraft.world.level.block.Block#updateOrDestroy`;
`net.minecraft.world.level.block.Block#updateFromNeighbourShapes`;
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
`net.minecraft.world.entity.monster.EnderMan$EndermanTakeBlockGoal#canUse`;
`net.minecraft.world.entity.monster.EnderMan$EndermanTakeBlockGoal#tick`;
`net.minecraft.world.entity.monster.EnderMan$EndermanLeaveBlockGoal#canUse`;
`net.minecraft.world.entity.monster.EnderMan$EndermanLeaveBlockGoal#tick`;
`net.minecraft.world.entity.monster.EnderMan$EndermanLeaveBlockGoal#canPlaceBlock`;
`net.minecraft.world.entity.ai.goal.Goal#reducedTickDelay`;
`net.minecraft.world.level.block.FlowerPotBlock#useItemOn`;
`net.minecraft.world.level.block.FlowerPotBlock#useWithoutItem`;
`net.minecraft.world.level.block.FlowerPotBlock#getCloneItemStack`;
`net.minecraft.world.level.block.FlowerPotBlock#updateShape`;
`net.minecraft.world.level.block.FlowerPotBlock#isPathfindable`;
`net.minecraft.world.level.block.FlowerPotBlock#isRandomlyTicking`;
`net.minecraft.world.level.storage.loot.predicates.ExplosionCondition#test`;
`net.minecraft.world.level.levelgen.feature.NetherForestVegetationFeature#place`;
`net.minecraft.world.level.levelgen.feature.stateproviders.WeightedStateProvider#getState`;
`net.minecraft.world.level.levelgen.feature.SimpleBlockFeature#place`;
`net.minecraft.world.level.levelgen.placement.RandomOffsetPlacement#getPositions`;
`net.minecraft.util.valueproviders.TrapezoidInt#sample`;
`net.minecraft.world.level.block.NyliumBlock#performBonemeal`;
`net.minecraft.client.data.models.BlockModelGenerators`;
`reports/blocks.json#minecraft:{warped_roots,crimson_roots,potted_warped_roots,potted_crimson_roots}`;
`reports/registries.json#minecraft:{block,item}/minecraft:{warped_roots,crimson_roots}`;
`reports/registries.json#minecraft:block/minecraft:{potted_warped_roots,potted_crimson_roots}`;
`reports/registries.json#minecraft:sound_event/minecraft:block.{roots,stone}.*`;
`reports/minecraft/components/item/{warped_roots,crimson_roots}.json`;
`data/minecraft/loot_table/blocks/{warped_roots,crimson_roots,potted_warped_roots,potted_crimson_roots}.json`;
`data/minecraft/loot_table/chests/bastion_hoglin_stable.json`;
`data/minecraft/tags/block/{combination_step_sound_blocks,enderman_holdable,replaceable,replaceable_by_mushrooms,replaceable_by_trees,enchantment_power_transmitter,flower_pots,supports_crimson_roots,supports_warped_roots,supports_vegetation,substrate_overworld,dirt,mud,moss_blocks,grass_blocks,nylium}.json`;
`data/minecraft/tags/item/**`;
`data/minecraft/{recipe,advancement,structure}/**`;
`data/minecraft/worldgen/configured_feature/{crimson_roots,crimson_forest_vegetation,warped_forest_vegetation,crimson_forest_vegetation_bonemeal,warped_forest_vegetation_bonemeal}.json`;
`data/minecraft/worldgen/placed_feature/{patch_crimson_roots,crimson_forest_vegetation,warped_forest_vegetation}.json`;
`data/minecraft/worldgen/biome/{crimson_forest,warped_forest,soul_sand_valley}.json`;
`assets/minecraft/blockstates/{warped_roots,crimson_roots,potted_warped_roots,potted_crimson_roots}.json`;
`assets/minecraft/models/block/{warped_roots,crimson_roots,potted_warped_roots,potted_crimson_roots,flower_pot_cross}.json`;
`assets/minecraft/models/item/{warped_roots,crimson_roots}.json`;
`assets/minecraft/items/{warped_roots,crimson_roots}.json`.

**Test vectors:**

Run `EXP-BLK-067` across all four identities, support/replacement/update/loot paths, composters,
walking/enchanting, traceable Enderman take/leave draws and every placement transform, potting and
extraction with finite/infinite/full inventories, potted loot, chest selection, each vegetation
weight boundary and crimson patch modifier, all 1,212 templates, persistence, sounds, maps and
models. Assert exact constants, conditional draw/read/write order, absence claims and client
convergence.

**Limits:**

Generic placement/update/break/loot/explosion, composter maturation/extraction, step cadence,
enchanting option generation, goal scheduling, Enderman persistence/death, container-loot
evaluation, feature/placement modifiers, packet encoding and rendering remain with
`BLK-PLACE-001`, `BLK-UPDATE-001`, `PLY-BREAK-001`, `ITM-LOOT-001`, `ITM-006`, `MOB-004`,
`MOB-005`, `WGEN-PIPELINE-001`, `PROTO-PLAY-CLIENTBOUND-BLOCK-001`,
`PROTO-PLAY-CLIENTBOUND-SOUND-001` and `CLI-006`.
