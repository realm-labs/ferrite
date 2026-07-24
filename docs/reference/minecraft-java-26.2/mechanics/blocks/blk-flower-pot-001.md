# Blocks mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `BLK-FLOWER-POT-001` — Flower pots own code-built contents, interaction ordering and two filled-form exceptions

**Parent:** `SIM-004`, `SIM-005`, `BLK-001`, `BLK-002`, `BLK-003`, `BLK-005`, `PLY-005`,
`PLY-006`, `ITM-003`, `ITM-004`, `ITM-006`, `MOB-004`, `MOB-005`, `ENV-001`, `ENV-002`,
`ENV-003`, `WGEN-002`, `WGEN-003`, `WGEN-004`, `CLI-001`, `CLI-006`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked registrations/reports, the complete `FlowerPotBlock` implementation,
the eyeblossom environment/timeline and particle consumers, the sole hoglin-repellent consumer,
all loot/recipe/acquisition/tag records, all 1,212 decoded structure templates and exact client
assets close the empty flower pot plus 36 filled identities. All are property-free, block-entity-
free states using a code-built content map and one interaction transaction. Only the two potted
eyeblossoms random-tick, and only potted warped fungus gains a content-specific AI role.

**Applies when:**

`minecraft:flower_pot` or any of the 36 filled identities listed below is placed, updated,
interacted with, cloned, broken, exploded, selected by loot or a structure template, persisted,
mapped or rendered. The same implementation admits transitions to/from potted crimson and warped
roots, but those two block identities, states, loot and projections remain owned by
`BLK-NETHER-ROOTS-001`.

**Authoritative state:**

Every identity has one property-free state and no block entity. The empty block stores AIR as its
content. The contiguous early registration segment is:

| Block ID | State ID | Identity/content |
|---:|---:|---|
| `411` | `10629` | `flower_pot` / air |
| `412` | `10630` | `potted_torchflower` / torchflower |
| `413` | `10631` | `potted_oak_sapling` / oak sapling |
| `414` | `10632` | `potted_spruce_sapling` / spruce sapling |
| `415` | `10633` | `potted_birch_sapling` / birch sapling |
| `416` | `10634` | `potted_jungle_sapling` / jungle sapling |
| `417` | `10635` | `potted_acacia_sapling` / acacia sapling |
| `418` | `10636` | `potted_cherry_sapling` / cherry sapling |
| `419` | `10637` | `potted_dark_oak_sapling` / dark-oak sapling |
| `420` | `10638` | `potted_pale_oak_sapling` / pale-oak sapling |
| `421` | `10639` | `potted_mangrove_propagule` / mangrove propagule |
| `422` | `10640` | `potted_fern` / fern |
| `423` | `10641` | `potted_dandelion` / dandelion |
| `424` | `10642` | `potted_golden_dandelion` / golden dandelion |
| `425` | `10643` | `potted_poppy` / poppy |
| `426` | `10644` | `potted_blue_orchid` / blue orchid |
| `427` | `10645` | `potted_allium` / allium |
| `428` | `10646` | `potted_azure_bluet` / azure bluet |
| `429` | `10647` | `potted_red_tulip` / red tulip |
| `430` | `10648` | `potted_orange_tulip` / orange tulip |
| `431` | `10649` | `potted_white_tulip` / white tulip |
| `432` | `10650` | `potted_pink_tulip` / pink tulip |
| `433` | `10651` | `potted_oxeye_daisy` / oxeye daisy |
| `434` | `10652` | `potted_cornflower` / cornflower |
| `435` | `10653` | `potted_lily_of_the_valley` / lily of the valley |
| `436` | `10654` | `potted_wither_rose` / wither rose |
| `437` | `10655` | `potted_red_mushroom` / red mushroom |
| `438` | `10656` | `potted_brown_mushroom` / brown mushroom |
| `439` | `10657` | `potted_dead_bush` / dead bush |
| `440` | `10658` | `potted_cactus` / cactus |

Later additions are `potted_bamboo` block/state `793/15291`,
`potted_crimson_fungus` `919/21826`, `potted_warped_fungus` `920/21827`,
`potted_azalea_bush` `1176/32073` with content `azalea`,
`potted_flowering_azalea_bush` `1177/32074` with content `flowering_azalea`,
`potted_open_eyeblossom` `1193/32363` and `potted_closed_eyeblossom`
`1194/32364`.

Only the empty flower pot has a registry item: raw item ID `1256`, common rarity, stack size `64`,
ordinary block-item components and item-model selector `minecraft:flower_pot`. No filled pot has a
separate item. Rotation and mirror are identity operations because there are no state properties.
Palette persistence stores only the selected block-state ID; the content association is rebuilt
from registrations rather than stored as block-entity data.

All 37 blocks use map color `NONE`, `HARP`, Stone sounds, zero hardness/resistance, no correct-tool
requirement, no occlusion and piston reaction `DESTROY`. Their shared selection and collision
shape is the centered box `(5,0,5)..(11,6,11)` in sixteenths. Thus they collide despite not
occluding, expose no full sturdy face, propagate skylight, damp light by zero, shade at `1.0` and
do not supply an ordinary spawn floor. The inherited survival predicate is always true for these
states: ordinary placement needs no support, and the downward-update removal branch never fires
for vanilla pot states. Pathfinding is explicitly false. None emits a redstone signal or comparator
value, ignites from lava, enters ordinary-fire odds or provides vanilla furnace fuel. Stone
break/step/place/hit/fall sound IDs are `1596/1604/1601/1600/1599`.

**Transition and ordering:**

#### Code-built content dispatch and held-item use

Constructing every `FlowerPotBlock` inserts its content block to filled-pot block mapping into one
static map. This map includes all 36 identities here plus both potted-root identities; data-pack
reload cannot add a pottable content block.

Held-item use first resolves a `BlockItem`'s block through that map, defaulting to AIR. A nonblock
item and an unmapped block item therefore resolve to AIR and return `TRY_WITH_EMPTY_HAND`. The
generic interaction dispatcher then reaches the no-item block hook: on a filled pot this extracts
its content even though the player is holding the nonpottable item; on an empty pot it consumes the
interaction without mutation. Hand ordering, sneak bypass, item fallback and client/server
prediction remain with `PLY-INTERACT-001`.

For a mapped non-AIR content:

- if the current pot is already filled, return `CONSUME` immediately, with no write, event,
  statistic, sound or item consumption; direct content replacement is therefore impossible;
- if the current pot is empty, offer the mapped filled default with flags `3`, ignore the Boolean
  result, emit `BLOCK_CHANGE` with the player as source, award custom statistic `pot_flower`
  (protocol ID `52`), call `consume(1, player)` and return `SUCCESS`, in that order.

The player-aware consume preserves infinite-material holders. The ignored write result means a
failed insertion can still emit game event ID `2`, award the statistic and consume the item.
Insertion emits no explicit sound. Root items take this same transaction but select states
`21828/21829` owned by `BLK-NETHER-ROOTS-001`.

The no-item hook returns `CONSUME` for the empty pot. For a filled pot it constructs one default
content stack, calls `player.addItem`, and on failure calls `player.drop(stack,false)`. Only after
that inventory/drop branch does it offer empty state `10629` with flags `3`, ignore the result,
emit `BLOCK_CHANGE` and return `SUCCESS`. It awards no statistic and emits no explicit sound.
Consequently a failed empty-state write can leave the filled state while the player has already
received or dropped its content.

Clone-pick on the empty state delegates to the ordinary block clone and returns one flower-pot
item. Clone-pick on a filled state returns one default content item, never the pot item and never
state-derived components. Potted cactus, wither rose, mushrooms, fungi, bamboo, saplings and
azaleas do not inherit contact, effect, spread, growth or bone-meal hooks from their content:
their runtime class remains `FlowerPotBlock`. The two exceptions below are explicit pot code/tag
branches.

#### Potted-eyeblossom random ticks

Only states `32363` and `32364` report randomly ticking; the other 35 states never enter this
callback. An admitted callback reads `gameplay/eyeblossom_open` at the position as a `TriState`.
`DEFAULT` preserves the current open/closed identity; equal desired/current state performs no
write, effect or internal random draw. The built-in overworld daily timeline supplies keyframes
`TRUE` at tick `12600` and `FALSE` at `23401`; environment selection, wrapping, activity and
random-position admission remain with the environment and random-tick owners.

When desired state differs, the callback offers the opposite property-free potted state with
flags `3` and ignores the result. It then unconditionally emits the target type's transform
particle and long switch sound, even when that write failed. The particle is one Trail particle
(particle ID `56`) created at block center. It consumes four `nextDouble()` values:
`scale=0.5+d0`, direction `(d1-0.5,d2+1,d3-0.5)`, target
`center+direction*scale`, lifetime `int(20*scale)`, zero speed. Opening uses color
`0xFC7812` and sound `minecraft:block.eyeblossom.open_long` ID `619`; closing uses
`0x5F5F5F` and `minecraft:block.eyeblossom.close_long` ID `621`. Sound source is Blocks,
volume/pitch `1/1` and source entity null. No game event, loot or retry accompanies the switch.

#### Loot, crafting and acquisition

The empty pot's one-roll block loot table offers one flower-pot item through
`survives_explosion`, using random sequence `minecraft:blocks/flower_pot`. Each of the 36 filled
tables has two independent one-roll pools under the same condition: first one flower-pot item,
then one exact content item. An ordinary break therefore yields both; an explosion evaluates each
pool independently and can yield neither, either or both. Tool, Silk Touch, Fortune and state
components do not otherwise affect these tables.

The shaped recipe is two rows, `# #` then ` # `, with `#=brick`, producing one flower pot and no
group. Its recipe advancement grants the recipe when either that recipe is already unlocked or an
inventory-change criterion sees brick. The trail-ruins common archaeology table selects one entry:
flower pot has weight `1` in total weight `45`. The village-mason chest makes a uniform `1..5`
rolls with replacement; flower pot has weight `1` in total `13` on each roll. No villager trade,
other direct nonblock loot record or sulfur-cube archetype names the flower-pot item.

#### Tags and the potted-warped-fungus AI exception

The `flower_pots` tag contains the empty pot, these 36 filled forms and both potted roots, 39
direct members total. No production class or bundled data selector consumes that tag; it remains
observable to tag synchronization, commands and future reload-selected consumers.

`potted_warped_fungus` is additionally one of four direct `hoglin_repellents` members. A hoglin's
specific sensor uses its default 20-tick scan period after phase-random startup and searches for
the closest tagged block within horizontal range `8` and vertical range `4`, writing or erasing
`NEAREST_REPELLENT`. In idle or fight activity, a present memory with no existing `PACIFIED`
memory sets `PACIFIED=true` for `200` ticks and erases `ATTACK_TARGET`; idle behavior also requests
a speed-`1.0` walk target at least `8` blocks away. Repellent presence selects the retreat ambient
sound. The other 36 states do not join this tag, and `potted_warped_fungus` is not a
`piglin_repellents` member.

#### Structure-template selection

Thirteen identities have raw cells in the bundled template inventory:

| Identity | Template pairs | Raw cells |
|---|---:|---:|
| flower pot | 1 | 1 |
| potted spruce sapling | 1 | 1 |
| potted birch sapling | 2 | 4 |
| potted dandelion | 6 | 6 |
| potted poppy | 3 | 3 |
| potted blue orchid | 2 | 3 |
| potted allium | 4 | 9 |
| potted azure bluet | 3 | 3 |
| potted red tulip | 3 | 5 |
| potted white tulip | 2 | 6 |
| potted oxeye daisy | 4 | 4 |
| potted dead bush | 9 | 20 |
| potted cactus | 25 | 55 |

These are 65 identity/template pairs and 120 cells across 55 distinct templates: village
`39/36/80` pairs/templates/cells, woodland mansion `22/15/36`, trial chambers `3/3/3` and igloo
`1/1/1`. The other 24 identities have zero cells. No configured feature or processor list names
one of these pot states directly. These are raw template facts, not unconditional placements:
structure pools, processors, transforms, clipping, placement admission and write results remain
with `WGEN-PIPELINE-001`.

**Client projection:**

Every property-free blockstate selects one like-named model. Empty `flower_pot` is ambient-
occlusion-off pot geometry: four one-unit terracotta walls around X/Z `5..11`, Y `0..6`, plus a
dirt top inside X/Z `6..10`, Y `4`. Twenty-nine filled forms use the shared
`flower_pot_cross` geometry: that pot plus two shade-disabled crossed plant planes from Y `4..16`.
Potted fern alone uses the tint-index-0 variant and the biome grass tint source.

Potted bamboo, cactus and mangrove propagule use bespoke inline stalk/leaf, solid-cactus and
propagule geometry. The two azalea forms use the shared potted-azalea template with plant, side and
top textures. Closed eyeblossom uses the ordinary cross; open eyeblossom adds duplicate emissive
planes whose model `light_emission` is `15`. That is a full-bright render layer only: both
eyeblossom blocks still have server block emission `0`.

Only item ID `1256` has an item selector, a flat generated model over `item/flower_pot`; filled
forms have no item projection. The Functional Blocks tab places it after the lightning-rod
variants and before decorated pot. Authoritative insertion, extraction and eyeblossom transitions
publish ordinary block updates; Trail options and the two sounds retain their protocol encoders.

**Branches and aborts:**

Thirty-seven identities/states, empty versus 36 filled contents, plus cross-leaf root transitions;
block/nonblock and mapped/unmapped held item; empty/occupied pot; finite/infinite material;
successful/failed state write; inventory admission/drop; clone and ordinary/explosion loot;
open/closed/default/true/false environment values; admitted/inactive random tick; unchanged,
successful and failed opposite write; all four particle draws; hoglin sensor present/absent and
activity/memory state; empty/singleton/reloaded tags; recipe/archaeology/chest selection; all 1,212
templates; persistence, map, sound, tab and every block/item model are distinct branches.

**Constants and randomness:**

Block IDs/states `411..440/10629..10658` plus `793/15291`, `919..920/21826..21827`,
`1176..1177/32073..32074`, `1193..1194/32363..32364`; item ID `1256`; stack `64`;
shape `(5,0,5)..(11,6,11)`; strength/resistance/emission/fire odds/fuel `0`; insertion/removal
flags `3`; statistic/event IDs `52/2`; random-eyeblossom timeline keys `12600/23401`; particle ID
`56`, four doubles, scale `[0.5,1.5)`, lifetime `10..29`, colors `0xFC7812/0x5F5F5F`, sound IDs
`619/621`; hoglin scan period/ranges/pacification/away distance/speed `20/8/4/200/8/1.0`;
archaeology and mason weights `1/45` and `1/13`, mason rolls `1..5`; template pairs/distinct/cells
`65/55/120`; Stone sound IDs above.

**Side effects:**

Block placement/collision/piston destruction; pot insertion state/event/stat/count commit; content
inventory/drop then empty-state/event commit; clone and two-pool loot; recipe unlock and container
loot; random open/close write, particle and sound; hoglin memory, aggression and movement changes;
structure-template writes; palette persistence; map, tint, emissive, tab and item projection.

**Gates:**

Generic reach/hand/sneak/use and block-write authority; code-built content membership and current
occupancy; player inventory/infinite-material policy; active loot/recipe/tag/structure snapshots;
explosion radius; random-tick chunk/activity/rate admission; active environment attribute;
hoglin sensor/activity/memory admission; structure pool/processor/write admission; valid registry,
resource-pack and client connection context.

**Boundary cases and quirks:**

Pots require no support despite retaining a generic downward-survival check. Nonpottable held items
fall through to content extraction, while any pottable item on a filled pot consumes the
interaction without replacing or extracting. Both insertion and extraction ignore state-write
failure after irreversible player/event/stat work. Neither transition explicitly plays a sound.
Filled clone-pick returns only content, whereas normal filled loot yields both content and pot.
Explosion survival is independent across those two pools. A forced potted cactus or wither rose
has no content-class contact behavior. Open eyeblossom is visually emissive but emits no server
light, and a failed random-tick write still emits its target particle and sound.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-REPORT-001`; `OFF-DATA-001`;
`net.minecraft.world.level.block.Blocks`;
`net.minecraft.world.level.block.Blocks#flowerPotProperties`;
`net.minecraft.world.level.block.FlowerPotBlock#codec`;
`net.minecraft.world.level.block.FlowerPotBlock#getShape`;
`net.minecraft.world.level.block.FlowerPotBlock#useItemOn`;
`net.minecraft.world.level.block.FlowerPotBlock#useWithoutItem`;
`net.minecraft.world.level.block.FlowerPotBlock#getCloneItemStack`;
`net.minecraft.world.level.block.FlowerPotBlock#updateShape`;
`net.minecraft.world.level.block.FlowerPotBlock#isPathfindable`;
`net.minecraft.world.level.block.FlowerPotBlock#isRandomlyTicking`;
`net.minecraft.world.level.block.FlowerPotBlock#randomTick`;
`net.minecraft.world.level.block.FlowerPotBlock#opposite`;
`net.minecraft.world.level.block.EyeblossomBlock$Type#transform`;
`net.minecraft.world.level.block.EyeblossomBlock$Type#spawnTransformParticle`;
`net.minecraft.world.timeline.Timelines#bootstrap`;
`net.minecraft.util.TriState#toBoolean`;
`net.minecraft.world.entity.ai.sensing.Sensor#tick`;
`net.minecraft.world.entity.ai.sensing.HoglinSpecificSensor#doTick`;
`net.minecraft.world.entity.ai.sensing.HoglinSpecificSensor#findNearestRepellent`;
`net.minecraft.world.entity.monster.hoglin.HoglinAi#initIdleActivity`;
`net.minecraft.world.entity.ai.behavior.BecomePassiveIfMemoryPresent#create`;
`net.minecraft.world.level.storage.loot.predicates.ExplosionCondition#test`;
`net.minecraft.world.item.CreativeModeTabs#bootstrap`;
`net.minecraft.world.level.block.FireBlock#bootStrap`;
`net.minecraft.world.level.block.entity.FuelValues#vanillaBurnTimes`;
`reports/blocks.json#minecraft:{flower_pot,potted_* except potted roots}`;
`reports/registries.json#minecraft:{block,item,sound_event,particle_type,game_event,custom_stat}`;
`reports/minecraft/components/item/flower_pot.json`;
`data/minecraft/loot_table/blocks/{flower_pot,potted_* except potted roots}.json`;
`data/minecraft/loot_table/{archaeology/trail_ruins_common,chests/village/village_mason}.json`;
`data/minecraft/{recipe/flower_pot,advancement/recipes/decorations/flower_pot}.json`;
`data/minecraft/tags/block/{flower_pots,hoglin_repellents,piglin_repellents}.json`;
`data/minecraft/{villager_trade,tags/item,worldgen}/**`;
`data/minecraft/structure/**/*.nbt`;
`assets/minecraft/blockstates/{flower_pot,potted_* except potted roots}.json`;
`assets/minecraft/models/block/{flower_pot,flower_pot_cross,tinted_flower_pot_cross,flower_pot_cross_emissive,template_potted_azalea_bush,potted_* except potted roots}.json`;
`assets/minecraft/{items,models/item}/flower_pot.json`.

**Test vectors:**

Run `EXP-BLK-072` across every state/content, interaction result and failed-write order, clone and
loot outcome, recipe/acquisition/tag reload, traceable eyeblossom tick/particle/sound branch,
hoglin memory/behavior branch, all 1,212 templates, persistence and client projection. Assert
exact IDs, mappings, constants, read/draw/write/effect order, negative content joins and
vanilla-client convergence.

**Limits:**

Generic placement, interaction dispatch/prediction, inventory insertion/drop, breaking, explosion
loot, random-tick sampling, environment timelines, hoglin brain scheduling, structure placement,
packet encoding and rendering remain with `BLK-PLACE-001`, `PLY-INTERACT-001`, `PLY-BREAK-001`,
`ITM-LOOT-001`, `SIM-RANDOM-001`, `MOB-AI-001`, `WGEN-PIPELINE-001`,
`PROTO-PLAY-CLIENTBOUND-BLOCK-001`, `PROTO-PLAY-CLIENTBOUND-SOUND-001` and `CLI-006`. Potted
crimson/warped roots remain owned by `BLK-NETHER-ROOTS-001`; unpotted content identities retain
their own leaves or catalog families.
