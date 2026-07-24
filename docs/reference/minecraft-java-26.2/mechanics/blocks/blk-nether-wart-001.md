# Blocks mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `BLK-NETHER-WART-001` — Nether wart grows by random tick and joins brewing, loot and Nether structures

**Parent:** `SIM-003`, `SIM-004`, `SIM-005`, `SIM-RANDOM-001`, `BLK-001`, `BLK-002`,
`BLK-003`, `BLK-005`, `PLY-005`, `PLY-006`, `ITM-004`, `ITM-006`, `ITM-BREW-001`,
`ITM-ADVANCEMENT-001`, `WGEN-003`, `CLI-001`, `CLI-006`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — the locked block/item/sound reports, complete class-reference set, loot,
recipe, advancement, trade and structure data, all 1,212 decoded templates and exact client assets
close the four-state crop and its ordinary item. The crop owns one support predicate and one
random-tick growth transition; bonemeal is deliberately absent. Its item additionally joins
placement, composting, brewing, crafting, cleric trade and chest loot.

**Applies when:**

`minecraft:nether_wart` is placed or force-written, loses support, receives a selected random tick,
is mined, cloned, exploded, composted, brewed or crafted, participates in a recipe unlock, cleric
offer or chest roll, is written by a Nether fortress or bastion, persists, maps, sounds or renders.

**Authoritative state:**

The identity is a `NetherWartBlock`/`VegetationBlock` with codec type `minecraft:nether_wart`, no
block entity and integer property `age=0..3`. States `9447`, `9448`, `9449` and `9450` map to ages
zero through three; age zero/state `9447` is the default. The block protocol ID is `384`. The
separately registered custom-name `BlockItem` has raw item ID `1148`, common rarity, stack limit
`64`, standard item components, translation `item.minecraft.nether_wart` and no direct item tag.

Registration fixes `COLOR_RED`, default `HARP`, hardness/resistance `0/0`, no collision, random
ticks, `NETHER_WART` sounds and piston reaction `DESTROY`. The age-indexed selection shapes fill
the X/Z footprint and have heights `5`, `8`, `11` and `14` sixteenths:
`(0,0,0)..(16,5+3*age,16)`. Collision and occlusion are empty, emission and light dampening are
zero, skylight propagates through the empty fluid state, and AIR pathfinding is allowed. The
states are not sturdy, suffocating, view-blocking, redstone-conducting, replaceable or spawn
floors, and they add no scheduled tick, use, attack, entity-contact, signal, comparator or
block-event dispatch.

The sound type has volume/pitch `1/1`. It selects Nether-wart break sound ID `1098`, stone step
`1604`, Nether-wart planting `1099`, stone hit `1600` and stone fall `1599`. Rotation and mirror
preserve age. Clone-pick returns the nether-wart item at every age.

**Transition and ordering:**

#### Placement, support and random growth

`supports_nether_wart` contains exactly `minecraft:soul_sand`. Ordinary item placement therefore
selects default age zero only when the block immediately below is soul sand and the generic
placement target is admissible. The custom item name does not change `BlockItem` placement
semantics. A successful player placement can satisfy the `nether_wart` criterion of
`husbandry/plant_seed`; that advancement has one OR requirement across wheat, pumpkin stem, melon
stem, beetroots, nether wart, torchflower crop and pitcher crop.

Every neighbor-shape update rechecks the block below. A non-soul-sand support transforms the crop
to AIR; ordinary `updateOrDestroy` then destroys the old state and evaluates its loot unless drops
were suppressed. Forced state/component writes can create any valid age on invalid support and
leave it until a later qualifying update. The block is not code-built or tag-reloadably
replaceable and retains no fluid state.

Only ages `0..2` report randomly ticking. Each time one of those states is selected by the owning
random-tick scheduler, `randomTick` consumes exactly one `nextInt(10)`. Result zero writes the next
age at the same position with flags `2` and ignores the returned success value; results `1..9`
make no mutation. Age three is excluded before the method and consumes no growth draw. Growth has
no light, biome, dimension, humidity or neighboring-crop predicate and emits no explicit game
event, sound or particle. `NetherWartBlock` does not implement `BonemealableBlock`, so bone meal's
crop dispatcher rejects every age without consuming the item or emitting level event `1505`.

#### Harvest and explosion loot

The one-roll block loot table always offers the nether-wart item. Ages `0..2` retain count one.
At age three, ordered functions first replace the count with inclusive uniform `2..4`, then apply
Fortune's `uniform_bonus_count` with multiplier one: Fortune level `L` adds
`nextInt(L+1)`, yielding `2..4+L`. No correct-tool, Silk Touch or tool identity predicate gates
the table.

The pool then applies `explosion_decay`. Without an explosion radius the count is unchanged, so
ordinary breaking and support loss yield one immature item or `2..4` mature items before Fortune.
With radius `r`, the function iterates every resulting unit and retains it when the next float is
at most `1/r`, allowing a partial stack rather than the all-or-nothing result of
`survives_explosion`. The table uses random sequence `minecraft:blocks/nether_wart`.

#### Composting, brewing, recipes and advancements

`ComposterBlock` registers the item at chance `0.65f`. Player insertion at level `0` succeeds
without RNG; levels `1..6` consume one `nextDouble()` and increment exactly when it is below
`0.65`. Success writes level plus one with flags `3`, emits `BLOCK_CHANGE`, and `6 -> 7` schedules
maturation after `20` ticks; failure preserves state. Either level-`0..6` result emits level event
`1500`, awards the used-item statistic and calls `consume(1, player)`, preserving
infinite-material holders. Level `7` succeeds without insertion or consumption; level `8` falls
through to ordinary item-on-block handling. Automation admits only below level `7`, uses the same
first-level/RNG transition and always shrinks one item, including after chance failure.

The code-built vanilla potion graph contains exactly the nether-wart edge
`water + nether_wart -> awkward`. It applies to each registered potion container while retaining
that container identity. Brewing-stand fuel, 400-tick progress, ingredient consumption, bottle
mutation, inventory effects and client synchronization remain with `ITM-BREW-001`.

Two bundled recipes consume the item:

- shapeless `nether_wart_block` requires nine nether-wart items and yields one Nether-wart block;
- shaped `red_nether_bricks` uses the 2-by-2 pattern `NW/WN`, consuming two nether bricks and two
  nether-wart items to yield one red Nether bricks block.

Each recipe advancement has `has_nether_wart` inventory and `has_the_recipe` criteria in one OR
requirement and grants only its matching recipe. Recipe matching, grid consumption, output
admission and recipe-book publication remain generic and the two output leaves retain ownership
of their result blocks.

#### Cleric and chest acquisition

`cleric/5/nether_wart_emerald` wants `22` nether-wart items and gives one emerald, with maximum
uses `12`, villager XP `30` and reputation discount `0.05`. It is one of exactly two members of
the level-five cleric tag; the trade set requests two distinct entries with random sequence
`minecraft:trade_set/cleric/level_5`, so the wart purchase is included whenever that complete
level-five set is built. Offer construction, pricing, demand, exhaustion, restocking and trade
commit remain with the generic villager/trade owners.

The Nether-bridge chest's first pool makes inclusive uniform `2..4` independent rolls over total
entry weight `78`. Nether wart has weight `5`; when selected, inclusive uniform count `3..7` is
offered. Its separate one-roll armor-trim pool has no wart result. The chest table uses random
sequence `minecraft:chests/nether_bridge`.

#### Nether structure writes

`CastleStalkRoom` is a Nether-fortress castle-piece candidate with weight `5`, maximum placement
count `2` and consecutive repetition disabled. Its clipped `postProcess` writes two soul-sand
beds at local boxes `x=3..4,y=4,z=4..8` and `x=8..9,y=4,z=4..8`, then writes default age-zero
nether wart directly above at the corresponding `y=5` boxes: 20 crop cells in a complete room.
Fortress piece selection, orientation, bounding clips and generic structure writes remain with
the fortress/structure owners.

The exhaustive template scan finds 12 age-three cells in exactly three of the 1,212 bundled
templates:

- `bastion/units/center_pieces/center_0` has six at local
  `(3,2,3),(4,2,3),(4,2,5),(5,2,7),(6,2,3),(6,2,4)`;
- `center_1` has one at `(4,2,3)`; and
- `center_2` has five at
  `(3,2,3),(4,2,3),(4,2,5),(6,2,3),(6,2,4)`.

The rigid `bastion/units/center_pieces` pool gives those three templates equal weight one and uses
the `housing` processor list. Its four rules target polished-blackstone bricks, blackstone or
gilded blackstone, never nether wart, so admitted wart cells survive processor selection unchanged
before the generic jigsaw write. Pool reachability, rotation, clipping, terrain collision and
publication remain with `WGEN-JIGSAW-BASTION-001`.

**Client projection:**

Blockstates select `nether_wart_stage0` for age zero, `stage1` for both ages one and two, and
`stage2` for age three. Each inherits `minecraft:block/crop`: ambient occlusion is disabled and
four shade-disabled double-sided planes stand at X/Z `4` and `12`, spanning the other horizontal
axis and model Y `-1..15`. Only the crop texture changes with the three visual stages; the model
geometry does not follow the age-indexed server selection height. The inventory selector uses a
flat generated item model with `minecraft:item/nether_wart`.

Block updates publish states `9447..9450`, inventory paths use item ID `1148`, material sounds use
IDs `1098/1604/1099/1600/1599`, and map projection uses `COLOR_RED`. This leaf adds no packet
field, acknowledgement or connection-local state.

**Branches and aborts:**

Age `0/1/2/3`; ordinary/forced placement; soul sand/other support; retained/lost support and
drop suppression; selected/missed random tick and failed write; bone meal at every age; hand/tool,
Fortune level, support-loss and explosion-radius loot; finite/infinite player and automation at
composter levels `0`, `1..6`, `7`, `8`; water/other potion inputs and registered containers; both
recipe matches/unlocks; cleric set construction and trade lifecycle; each chest roll; fortress
room selected/rejected/clipped; three bastion templates, rotations, processors and clipping;
save/reload and four server states versus three client stages are distinct branches.

**Constants and randomness:**

States `9447..9450`, block/item IDs `384/1148`; ages `0..3`; selection heights `5/8/11/14`;
strength `0/0`; emission/dampening `0/0`; sounds `1098/1604/1099/1600/1599`; stack `64`; one
support; growth bound/success `10/0`; immature loot `1`, mature base `2..4`, Fortune addition
`0..L`, explosion retention `nextFloat<=1/r`; composter chance `0.65`, maturation `20`, event
`1500`; recipe inputs/results `9/1` and `2+2/1`; cleric wants/gives/uses/XP/discount
`22/1/12/30/0.05`, selection `2/2`; chest rolls `2..4`, entry weight/total `5/78`, count `3..7`;
fortress room weight/max/cells `5/2/20`; bastion template weights `1/1/1`, cells `6/1/5`;
templates/matches/cells `1212/3/12`.

**Side effects:**

Placement and plant-seed criterion; support-loss removal and loot; random-tick state write;
ordinary/Fortune/explosion drops; composter item/stat/state/game-event/level-event/schedule;
brewing potion mutation; two crafting results and unlocks; one cleric purchase offer; chest item
generation; fortress and bastion writes; ordinary persistence; map, sound, crop-stage and flat-item
projection.

**Gates:**

Write/break authority; valid age and exact support; random-tick selection and draw; loot state,
tool enchantment and explosion context; composter level/input/RNG/infinite-material policy;
brewing fuel/progress/container/input; active recipe, advancement, trade, chest, tag, pool,
processor and template snapshots; structure selection/orientation/clip/write admission; registry,
map, sound and client-resource context.

**Boundary cases and quirks:**

Age three stops random ticking, while ages one and two share one visual model despite distinct
server states and selection heights. Bone meal is a complete no-op at every age. Support loss uses
age-sensitive loot, so a mature unsupported plant can drop `2..4` before Fortune rather than one.
Explosion decay tests every resulting unit independently. The fortress writes age zero, but every
bundled bastion cell is age three. The `housing` processor can rot surrounding masonry yet has no
wart-matching rule. Nether wart is both a placeable custom-name block item and a brewing ingredient;
using it on an existing plant does not harvest or advance age.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-REPORT-001`; `OFF-DATA-001`;
`net.minecraft.world.level.block.Blocks`;
`net.minecraft.world.item.Items`;
`net.minecraft.world.level.block.SoundType`;
`net.minecraft.world.level.block.NetherWartBlock#getShape`;
`net.minecraft.world.level.block.NetherWartBlock#mayPlaceOn`;
`net.minecraft.world.level.block.NetherWartBlock#isRandomlyTicking`;
`net.minecraft.world.level.block.NetherWartBlock#randomTick`;
`net.minecraft.world.level.block.NetherWartBlock#getCloneItemStack`;
`net.minecraft.world.level.block.VegetationBlock#updateShape`;
`net.minecraft.world.level.block.VegetationBlock#canSurvive`;
`net.minecraft.world.item.BoneMealItem#growCrop`;
`net.minecraft.world.level.storage.loot.functions.ApplyBonusCount#run`;
`net.minecraft.world.level.storage.loot.functions.ApplyBonusCount$UniformBonusCount#calculateNewCount`;
`net.minecraft.world.level.storage.loot.functions.ApplyExplosionDecay#run`;
`net.minecraft.world.level.block.ComposterBlock#bootStrap`;
`net.minecraft.world.level.block.ComposterBlock#useItemOn`;
`net.minecraft.world.level.block.ComposterBlock#insertItem`;
`net.minecraft.world.level.block.ComposterBlock#addItem`;
`net.minecraft.world.item.ItemStack#consume`;
`net.minecraft.world.item.alchemy.PotionBrewing#addVanillaMixes`;
`net.minecraft.world.item.trading.TradeSet#calculateNumberOfTrades`;
`net.minecraft.world.item.trading.VillagerTrade#getOffer`;
`net.minecraft.world.level.levelgen.structure.structures.NetherFortressPieces$PieceWeight`;
`net.minecraft.world.level.levelgen.structure.structures.NetherFortressPieces$CastleStalkRoom#postProcess`;
`net.minecraft.world.level.levelgen.structure.StructurePiece#generateBox`;
`reports/blocks.json#minecraft:nether_wart`;
`reports/registries.json#minecraft:{block,item,sound_event}`;
`reports/minecraft/components/item/nether_wart.json`;
`data/minecraft/loot_table/{blocks/nether_wart,chests/nether_bridge}.json`;
`data/minecraft/tags/block/supports_nether_wart.json`;
`data/minecraft/recipe/{nether_wart_block,red_nether_bricks}.json`;
`data/minecraft/advancement/{husbandry/plant_seed,recipes/building_blocks/nether_wart_block,recipes/building_blocks/red_nether_bricks}.json`;
`data/minecraft/{villager_trade/cleric/5/nether_wart_emerald,tags/villager_trade/cleric/level_5,trade_set/cleric/level_5}.json`;
`data/minecraft/worldgen/template_pool/bastion/units/center_pieces.json`;
`data/minecraft/worldgen/processor_list/housing.json`;
`data/minecraft/structure/**/*.nbt`;
`assets/minecraft/blockstates/nether_wart.json`;
`assets/minecraft/models/block/{crop,nether_wart_stage0,nether_wart_stage1,nether_wart_stage2}.json`;
`assets/minecraft/{items,models/item}/nether_wart.json`.

**Test vectors:**

Run `EXP-BLK-068` across all four ages, support/update/forced-write and placement paths, controlled
random ticks and failed writes, bone meal, hand/tool/Fortune/explosion loot, every composter and
brewing boundary, both recipes and all three advancements, the cleric set, every Nether-bridge
loot boundary, complete/clipped fortress rooms, all 1,212 templates and bastion processor paths,
persistence, sounds, maps and models. Assert exact constants, conditional draw/read/write order,
absence claims and vanilla-client convergence.

**Limits:**

Random-tick scheduling, generic placement/update/break/loot/explosion, composter
maturation/extraction, brewing-stand transactions, crafting/advancement evaluation, villager offer
selection and trading, chest evaluation, fortress/jigsaw assembly, packet encoding and rendering
remain with `SIM-004`, `BLK-PLACE-001`, `BLK-UPDATE-001`, `PLY-BREAK-001`, `ITM-LOOT-001`,
`ITM-006`, `ITM-BREW-001`, `ITM-CRAFT-001`, `ITM-ADVANCEMENT-001`, the generic trade owners,
`WGEN-STRUCTURE-FORTRESS-001`, `WGEN-JIGSAW-BASTION-001`,
`PROTO-PLAY-CLIENTBOUND-BLOCK-001`, `PROTO-PLAY-CLIENTBOUND-SOUND-001` and `CLI-006`.
