# Blocks mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `BLK-CLAY-001` — Clay and Clay Ball join block loot, processing, villages, archaeology, tags and lush-cave generation

**Parent:** `SIM-004`, `SIM-005`, `SIM-RANDOM-001`, `BLK-001`,
`BLK-STATE-001`, `BLK-002`, `BLK-PLACE-001`, `BLK-BREAK-001`,
`BLK-BREAK-HOOK-001`, `PLY-005`, `PLY-006`, `PLY-INPUT-001`,
`PLY-INTERACT-001`, `PLY-BREAK-001`, `BLK-003`, `BLK-004`, `BLK-005`,
`BLK-007`, `BLK-UPDATE-001`, `PLY-002`, `PLY-COLLISION-001`,
`PLY-AUTOJUMP-001`, `RED-001`, `RED-UPDATE-001`, `RED-COMPARATOR-001`,
`ITM-001`, `ITM-002`, `ITM-003`, `ITM-004`, `ITM-005`, `ITM-006`,
`ITM-007`, `ITM-USE-001`, `ITM-CONTAINER-001`, `ITM-RECIPE-001`,
`ITM-CRAFT-001`, `ITM-FURNACE-001`, `ITM-LOOT-001`,
`ITM-ADVANCEMENT-001`, `ITM-ENCHANT-001`, `ITM-ANVIL-001`, `ENT-001`,
`ENT-KNOCKBACK-001`, `MOB-001`, `MOB-004`, `MOB-AI-001`, `MOB-SPAWN-001`,
`MOB-RAID-001`,
`ENV-001`, `ENV-002`, `ENV-003`, `ENV-FLUID-001`, `ENV-FIRE-001`,
`ENV-LIGHT-001`, `BLK-BRUSHABLE-001`, `BLK-DRIPSTONE-BLOCK-001`,
`WGEN-PIPELINE-001`, `WGEN-JIGSAW-TRAIL-RUINS-001`,
`WGEN-JIGSAW-VILLAGES-001`, `CLI-001`, `CLI-006`, `CLI-UI-001`,
`CLI-EFFECT-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — the locked block/item registrations, state report, tags, loot, recipes,
advancements, village records, trade, archaeology, configured/placed features, structure
templates and client resources determine every Clay- and Clay-Ball-specific selector. The generic
block, item, loot, furnace, crafting, trade, AI, archaeology, worldgen and client algorithms remain
with their cited owners.

**Applies when:**

`minecraft:clay` is placed, updated, mined, exploded, converted from Mud, selected by a block tag,
generated or projected; or a `minecraft:clay_ball` stack is looted, crafted, smelted, traded,
moved, renamed, persisted, synchronized or rendered before and after data/resource reload.

**Authoritative state:**

Clay is a property-free plain `Block` with sole state ID `6946`, raw block ID `281` and block-item
ID `370`. Its registration fixes map color `CLAY`, note instrument `FLUTE`, hardness/resistance
`0.6/0.6`, `GRAVEL` sound, no correct-tool requirement and the inherited full unit
selection/collision/visual/occlusion cube. Inherited constants are emission `0`, light dampening
`15`, shade `0.2`, friction `0.6`, speed/jump `1`, restitution `0`, normal piston reaction,
sound volume/pitch `1/1`, solid/full sturdy faces, conductor and ordinary spawn support. It has no
block entity, properties, ticker, random tick, scheduled tick, signal, comparator, use, contact,
fall or projectile override.

Clay Ball is raw item ID `1055`, a common nondamageable plain `Item` with maximum stack `64`. Its
default components are the common empty modifiers/enchantments/lore, item-break sound, translated
name, direct item-model key, repair cost, swing animation, tooltip display and use effects. It has
no food, consumable, remainder, durability, equipment, tool, projectile, cooldown, inventory tick
or identity-specific use hook. Arbitrary ordinary component patches persist but do not change the
exact-identity recipe, loot-entry or trade-output records described below.

**Transition and ordering:**

Block placement, update and removal:

Placing the block item proposes the sole default Clay state. Its inherited placement and
neighbor-shape paths neither add state nor schedule work. Hand, wrong-tool and shovel mining all
reach the same loot table because Clay does not require a correct tool; the direct
`mineable/shovel` membership changes suitable-tool speed only. Ordinary placement/removal,
survival, explosion and update ordering remain the block and player owners.

The block loot table has one roll and one sequence entry. With a tool carrying Silk Touch level
at least one, its first alternative emits one default Clay block. Otherwise it emits one Clay Ball
entry, sets count exactly `4`, then applies explosion decay. Fortune has no effect. Without an
explosion, every admitted non-Silk break therefore yields exactly four Clay Balls. At explosion
radius `r`, decay makes exactly four independent `nextFloat() <= 1/r` survival tests, allowing
counts `0..4`; the retained balls form the resulting stack. Failure to admit or evaluate block
loot emits nothing.

Mud-to-Clay drip conversion:

In the admitted downward pointed-dripstone water-transfer path owned by
`BLK-DRIPSTONE-BLOCK-001`, the water branch uses threshold `0.17578125`. After its orientation,
source-fluid and at-most-eleven-block tip search succeeds, exact source state Mud selects the Clay
conversion instead of cauldron filling. The server writes default Clay at the source Mud position,
emits a `BLOCK_CHANGE` game event there with Clay context and level event `1504` at the tip, then
returns. Failed gates, another source block/fluid or a rejected transfer draw leave the source
unchanged.

Clay/Clay-Ball processing and progression:

- Shaped recipe `clay` is an exact `2×2` square of four Clay Balls and emits one default Clay
  block. It fits the inventory grid and every admitted offset in a `3×3` grid.
- Furnace-only smelting recipe `brick` consumes one Clay Ball, emits one default Brick after the
  omitted/default `200` cooking ticks and records `0.3` recipe experience.
- Furnace-only smelting recipe `terracotta` consumes one Clay block, emits one default Terracotta
  after `200` ticks and records `0.35` recipe experience.

Blast Furnace, Smoker and Campfire recipe maps reject both smelting records. Successful recipes
copy no arbitrary input patches and leave no remainder. Invalid shape, extra cells, wrong
identity/machine, unavailable recipe or an inadmissible output commits no result.

The Clay recipe advancement has one OR requirement: exact Clay-Ball possession or already knowing
`minecraft:clay` grants the Clay recipe. The Brick recipe uses the same OR form with Clay Ball or
known `minecraft:brick`. The Terracotta recipe instead accepts exact Clay-block possession or
known `minecraft:terracotta`. Possessing the output does not substitute for the listed input
criterion. Furnace fuel/progress/result/XP and recipe-book/listener transactions remain with
`ITM-FURNACE-001`, `ITM-CRAFT-001` and `ITM-ADVANCEMENT-001`.

Village chests, Mason exchange and Hero gift:

Two village chest pools can emit Clay Balls:

- `village_desert_house` pool zero takes uniform `3..8` replacement rolls over total weight `36`;
  the weight-one Clay-Ball row emits one default ball per selection;
- `village_mason` pool zero takes uniform `1..5` replacement rolls over total weight `13`; its
  weight-one row emits a uniform `1..3` default Clay Balls per selection.

The named random sequences are the table IDs. Other pools and rows are independent; selection is
not guaranteed and no patches are added.

Mason level one contains exactly two predicate-free records and chooses amount `2` without
duplicates, so its Clay-Ball purchase and Brick sale are each guaranteed once in every default
fresh level-one offer set. The Clay record accepts ten Clay Balls under an empty component
predicate and gives one default Emerald, with maximum uses `16`, villager XP `2` and discount
coefficient `0.05`. Arbitrary Clay-Ball patches satisfy that empty predicate. It has no second
cost, modifier or double-price enchantment, and Trade Rebalance does not replace Mason records.

An admitted adult Mason running the Hero-of-the-Village gift path evaluates
`gameplay/hero_of_the_village/mason_gift`: its sole one-roll entry emits exactly one default Clay
block. Targeting, cooldown, navigation, throw timing and pickup remain `MOB-RAID-001`; no Clay Ball
is in this gift table.

Archaeology and raw village templates:

`archaeology/trail_ruins_common` takes one roll over total eligible weight `45`; Clay has weight
`2`, giving probability `2/45` and one default Clay block when selected. Trail-Ruins processors
can install at most six common suspicious Gravel blocks per house, two per road and two per tower
top, subject to their processor, transform, clipping and write gates. The ten brushing strokes,
loot seed/table invocation, item exposure and later pickup remain the brushable/Trail-Ruins
owners.

Across all `1,212` locked structure templates, Clay occurs in exactly `19` raw cells across five
files: `desert_mason_1` has `1`, `plains_masons_house_1` has `4`,
`savanna_mason_1` has `6`, and the normal and zombie Taiga
`taiga_fisher_cottage_1` files have `4` each. None of those templates embeds an entity. These raw
cells are inputs to the village template/processor/transform/clip/write transaction, not a
guarantee that every generated village retains every cell.

Block-tag selectors:

Clay is a direct member of exactly nine locked block tags:

- `mineable/shovel` selects suitable-tool mining speed but no loot gate;
- `enderman_holdable` admits empty-handed, mob-griefing-enabled Endermen to the generic
  take/carry/place state machine; taking stores default Clay and removes the world block without
  loot, while admitted placement writes the sole Clay state;
- `axolotls_spawnable_on` satisfies the block-below member test in the Axolotl spawn predicate;
- `supports_azalea` and `supports_mangrove_propagule` satisfy their direct placement-support
  branches;
- `supports_small_dripleaf` satisfies its first support branch without needing the alternative
  water-source support path;
- `azalea_root_replaceable` admits replacement by the rooted-azalea feature;
- `lush_ground_replaceable` admits replacement by the two Clay vegetation-patch records;
- `sculk_replaceable` admits the ordinary Sculk spread/vein substrate path and composes into the
  worldgen replacement tag.

The membership determines only those selector results. Enderman timing/rays/game-rule/placement,
mob spawning, plant placement, Sculk propagation and feature writes remain with their generic
owners and active tag snapshots.

Clay block items are direct members of `sulfur_cube_archetype/regular`. A matching equipped body
stack is buoyant; uses horizontal/vertical knockback powers `0.4125/0.09`, regular hit/push
behavior, cooldown `0.5` and threshold `0.2`; and contributes additive knockback resistance `-1`,
explosion knockback resistance `-1`, bounciness `0.5`, multiplied-total friction
`-0.699999988079071` and air drag `-0.8999999985098839`. Reloaded archetype selection and the
equipment/contact/knockback transaction remain `ENT-KNOCKBACK-001` and the sulfur-cube owners.
Clay Ball has no direct item-tag membership.

World generation:

Four configured records write Clay:

- `disk_clay` is a disk of Clay with radius uniform `2..3`, half-height `1` and target Dirt or
  Clay. Its placed record applies in-square, `OCEAN_FLOOR_WG`, current-fluid Water and biome
  modifiers and is referenced by exactly `55` locked Overworld biomes.
- `ore_clay` is an ore configuration of size `33`, discard-on-air-exposure `0`, replacing
  `base_stone_overworld` with Clay. Its placed record uses count `46`, in-square and uniform height
  from `above_bottom:0` through absolute `256`; only Lush Caves references it.
- `lush_caves_clay` is a random-boolean selector between `clay_with_dripleaves` and
  `clay_pool_with_dripleaves`, referenced only by Lush Caves.
- the ordinary/waterlogged vegetation patches both use Clay ground, floor surface, depth `3`,
  bottom chance `0.8`, edge chance `0.7`, horizontal radius uniform `4..7` and the live
  `lush_ground_replaceable` target. The ordinary record uses vegetation chance `0.05` and vertical
  range `2`; the waterlogged pool record uses `0.1` and `5`. Both delegate vegetation to
  Dripleaf.

Exact disk/ore/vegetation-patch candidate scans, RNG, target evaluation, ordering, failed writes
and biome feature scheduling remain `WGEN-PIPELINE-001`; the values and identity joins above are
the Clay-owned records.

**Persistence and reload boundary:**

Clay persists only its property-free state identity. Clay and Clay-Ball stacks persist identity,
count and arbitrary valid patches. They store no mining, drip, furnace, crafting, recipe,
archaeology, merchant, AI or worldgen cursor; those values persist with their owners.

Recipe reload changes future matching/output and progression listeners. Loot reload changes future
block, chest, archaeology and gift evaluation. Tag/archetype reload changes future selector and
equipment admission. Trade and advancement reload change future offers/listeners. Worldgen reload
changes future configured/placed feature evaluation. Existing blocks, stacks, completed work,
offers and generated chunks are not replayed or rewritten. Resource reload independently controls
names, models and textures.

**Client and wire projection:**

Authoritative block updates encode Clay state `6946`; inventory stack encoding uses IDs `370` for
Clay and `1055` for Clay Ball plus generic patches. Gravel sound events use raw IDs
break/step/place/hit/fall `760/764/763/762/761`. The locked English names are `Clay` and `Clay
Ball`.

The sole blockstate variant selects `minecraft:block/clay`; its `cube_all` model maps every face
to `minecraft:block/clay`. The Clay item points to that block model. Clay Ball uses the ordinary
generated `minecraft:item/clay_ball` model and texture. Clay appears exactly once in Natural
Blocks, ordered Mud, Clay, Gravel, Sand. Clay Ball appears exactly once in Ingredients, ordered
Slime Ball, Clay Ball, Prismarine Shard, Prismarine Crystals. This leaf adds no packet field,
acknowledgement or connection-local state.

**Branches and aborts:**

Default/component block placement; hand/wrong-tool/shovel removal; Silk/non-Silk and
ordinary/explosion loot; admitted/rejected drip conversion; crafting and both furnace recipes;
three recipe unlocks; two village chest pools; fresh/exhausted Mason offer; admitted Hero gift;
archaeology selection/installation/brushing; nineteen raw cells and village transforms; nine
block-tag and one item-tag selector families; four configured/placed generation paths;
persistence/reload/wire; both item and block client projections.

**Constants and randomness:**

State/block/item IDs `6946/281/370/1055`; strength `0.6/0.6`; sound IDs
`760/764/763/762/761`; stack `64`; non-Silk loot `4`; explosion survival four draws at
`<=1/r`; drip threshold `0.17578125`; recipes `4→1`, `1→1` at `200/0.3` and `200/0.35`;
chest rolls/weights/counts `3..8/36/1/1` and `1..5/13/1/1..3`; Mason
`10→1`, uses/XP `16/2`, discount `0.05`; archaeology `2/45`; raw templates/cells `5/19`;
worldgen values as listed above; regular-archetype values as listed above.

**Side effects:**

Ordinary full-block placement/removal and Silk/explosion-selected loot; Mud replacement and its
game/level events; crafting/furnace inputs, results, recipe/XP/progression; chest, archaeology,
trade and gift outputs; Enderman/Sculk/plant/spawn/equipment selector admission; village and
feature palette writes; ordinary persistence/wire; Gravel sounds, Clay map color and exact client
models/tabs.

**Gates:**

World-write and break authority; tool enchantment and explosion context; admitted pointed
dripstone water transfer over exact Mud; active recipe/advancement/loot/trade/tag/archetype/
worldgen snapshots; crafting/furnace output admission; village/structure/archaeology/gift
generation; merchant validity; AI/game-rule and spawn predicates; registry/decode; valid client
language/model/tab bootstrap.

**State read/written:**

Reads Clay state/tool/loot/environment, Clay/Clay-Ball stacks/components, recipe/furnace,
progression, loot, merchant, AI, tags, archetype, structure/worldgen and client state. Writes only
the block, stack, processing, progression, loot, merchant, AI, structure/worldgen, persistence and
projection state listed above.

**Failure behavior:**

Rejected placement/removal/loot or a failed drip gate writes no Clay result. Invalid recipes and
wrong machines do not process. Unselected loot/trade/gift/archaeology entries emit alternatives or
nothing as defined by their owners. Failed AI, spawn, plant, Sculk, archetype, structure or feature
gates do not gain authority from identity membership. Reloaded data affects only future evaluation;
missing client resources cannot grant server behavior.

**Boundary cases and quirks:**

Shovels are suitable but not required for loot. Silk Touch switches four Clay Balls to one Clay
block; Fortune never changes either branch. Explosion decay tests the four-ball count
individually. Clay Ball possession unlocks both its compacting recipe and its Brick smelting
recipe, while Clay possession unlocks Terracotta. Clay is both a vegetation-patch output and an
allowed replacement input. Raw village cells, suspicious-block opportunities and biome feature
references are not final placed/output counts.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-DATA-001`; `OFF-REPORT-001`;
`net.minecraft.world.level.block.Blocks`;
`net.minecraft.world.item.Items`;
`net.minecraft.world.level.block.PointedDripstoneBlock`;
`net.minecraft.world.level.block.AzaleaBlock#mayPlaceOn`;
`net.minecraft.world.level.block.MangrovePropaguleBlock#mayPlaceOn`;
`net.minecraft.world.level.block.SmallDripleafBlock#mayPlaceOn`;
`net.minecraft.world.entity.animal.axolotl.Axolotl#checkAxolotlSpawnRules`;
`net.minecraft.world.entity.monster.EnderMan$EndermanTakeBlockGoal`;
`net.minecraft.world.entity.monster.EnderMan$EndermanLeaveBlockGoal`;
`net.minecraft.world.entity.monster.cubemob.SulfurCube#matchingArchetypes`;
`net.minecraft.world.entity.npc.villager.AbstractVillager#addOffersFromTradeSet`;
`net.minecraft.world.entity.npc.villager.AbstractVillager#addOffersFromItemListingsWithoutDuplicates`;
`net.minecraft.world.item.trading.VillagerTrade#getOffer`;
`net.minecraft.world.item.trading.TradeSet#calculateNumberOfTrades`;
`net.minecraft.world.item.CreativeModeTabs`;
`reports/blocks.json#minecraft:clay`;
`reports/registries.json#minecraft:{block,item,sound_event}`;
`reports/minecraft/components/item/{clay,clay_ball}.json`;
`data/minecraft/loot_table/blocks/clay.json`;
`data/minecraft/recipe/{clay,brick,terracotta}.json`;
`data/minecraft/advancement/recipes/{building_blocks/{clay,terracotta},misc/brick}.json`;
`data/minecraft/loot_table/chests/village/{village_desert_house,village_mason}.json`;
`data/minecraft/{villager_trade/mason/1/clay_ball_emerald,tags/villager_trade/mason/level_1,trade_set/mason/level_1}.json`;
`data/minecraft/loot_table/{archaeology/trail_ruins_common,gameplay/hero_of_the_village/mason_gift}.json`;
`data/minecraft/tags/block/{axolotls_spawnable_on,azalea_root_replaceable,enderman_holdable,lush_ground_replaceable,mineable/shovel,sculk_replaceable,supports_azalea,supports_mangrove_propagule,supports_small_dripleaf}.json`;
`data/minecraft/tags/item/sulfur_cube_archetype/regular.json`;
`data/minecraft/sulfur_cube_archetype/regular.json`;
`data/minecraft/worldgen/{configured_feature/{clay_pool_with_dripleaves,clay_with_dripleaves,disk_clay,ore_clay},placed_feature/{disk_clay,lush_caves_clay,ore_clay},biome/*.json}`;
`data/minecraft/structure/village/**/*.nbt`;
`assets/minecraft/blockstates/clay.json`;
`assets/minecraft/models/block/clay.json`;
`assets/minecraft/items/clay.json`;
`assets/minecraft/textures/block/clay.png`;
`assets/minecraft/{items,models/item,textures/item}/clay_ball.*`;
`BLK-DRIPSTONE-BLOCK-001`; `BLK-BRUSHABLE-001`; `ITM-FURNACE-001`;
`ITM-RECIPE-001`; `ITM-LOOT-001`; `ITM-ADVANCEMENT-001`; `MOB-AI-001`;
`MOB-RAID-001`; `WGEN-PIPELINE-001`; `WGEN-JIGSAW-TRAIL-RUINS-001`;
`WGEN-JIGSAW-VILLAGES-001`; `CLI-EFFECT-001`; `EXP-BLK-084`.

**Test vectors:**

Run `EXP-BLK-084` across state/registry identity, ordinary/component placement, hand/wrong-tool/
shovel mining, Silk/Fortune/explosion loot and all pointed-dripstone gates. Exercise all three
recipes/unlocks/machines, both village chests, complete Mason offer lifecycles, Hero gifting,
Trail-Ruins selection/installation/brushing and all five raw templates. Evaluate every direct
block/item tag before/after reload; run disk, ore and both vegetation patches across candidate,
RNG and write boundaries; then persist/synchronize and verify all IDs, sounds, map/name/model and
tab projections.

**Limits:**

Generic placement, breaking, loot/explosion decay, crafting, furnace/XP, advancements, merchant
economy, Hero gifting, Enderman/spawn/plant/Sculk/sulfur-cube behavior, archaeology, structure and
feature algorithms, packet encoding and client rendering remain with `BLK-PLACE-001`,
`PLY-BREAK-001`, `ITM-LOOT-001`, `ITM-RECIPE-001`, `ITM-FURNACE-001`,
`ITM-ADVANCEMENT-001`, `MOB-AI-001`, `MOB-RAID-001`, `ENT-KNOCKBACK-001`,
`MOB-SPAWN-001`, `BLK-BRUSHABLE-001`, `WGEN-PIPELINE-001`,
`WGEN-JIGSAW-TRAIL-RUINS-001`,
`WGEN-JIGSAW-VILLAGES-001`, `PROTO-PLAY-CLIENTBOUND-BLOCK-001`,
`PROTO-PLAY-CLIENTBOUND-SOUND-001` and `CLI-006`.
