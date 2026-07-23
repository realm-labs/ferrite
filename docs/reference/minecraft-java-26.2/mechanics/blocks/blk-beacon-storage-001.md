# Blocks mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `BLK-BEACON-STORAGE-001` — Beacon storage blocks join compacted materials to golems, piglins and generated treasure

**Parent:** `BLK-001`, `BLK-002`, `BLK-003`, `BLK-005`, `RED-001`, `PLY-005`, `PLY-006`,
`ITM-004`, `ITM-006`, `ENT-001`, `ENT-005`, `MOB-001`, `ENV-003`, `WGEN-003`, `CLI-006`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked registrations, reports, recipes, advancements, tags, loot, optional
trade-rebalance data, structure code/templates/processors and client assets exhaust the five
beacon-base storage identities and their observable joins.

**Applies when:**

`minecraft:iron_block`, `minecraft:gold_block`, `minecraft:diamond_block`,
`minecraft:emerald_block` or `minecraft:netherite_block` is placed, mined, exploded, crafted,
selected as a beacon base or note-block substrate, consumed by a recipe or optional trade, emitted
by loot, handled as an item by a piglin or sulfur cube, used in an iron-golem pattern, generated,
serialized or rendered.

**Authoritative state:**

All five registrations are property-free ordinary `Block` instances with no block entity and one
state each:

| Identity | State | Map color | Instrument | Destroy speed | Explosion resistance | Sound |
|---|---:|---|---|---:|---:|---|
| gold block | 2338 | `GOLD` | `BELL` | 3.0 | 6.0 | `METAL` |
| iron block | 2339 | `METAL` | `IRON_XYLOPHONE` | 5.0 | 6.0 | `IRON` |
| diamond block | 5309 | `DIAMOND` | `HARP` | 5.0 | 6.0 | `METAL` |
| emerald block | 9727 | `EMERALD` | `BIT` | 5.0 | 6.0 | `METAL` |
| netherite block | 21818 | `COLOR_BLACK` | `HARP` | 50.0 | 1200.0 | `NETHERITE_BLOCK` |

Every registration requires a correct tool for drops. Gold, diamond and emerald blocks require the
iron-tool tier; iron requires the stone-tool tier; netherite requires the diamond-tool tier. All
five are direct `mineable/pickaxe` members. Unspecified registration properties retain the ordinary
full-solid defaults: unit selection/collision/occlusion shape, emission zero, light dampening 15,
friction 0.6, speed/jump factors 1, solid redstone conduction, normal piston reaction, no random
tick and no block entity.

Each block table performs one roll, returns the matching item only behind
`survives_explosion`, and uses `minecraft:blocks/<identity>` as its random sequence. Correct-tool
admission remains the generic break transaction; a wrong-tier pickaxe or non-tool removes the
block but does not reach this self-drop path.

Every blockstate maps its sole variant to the matching `minecraft:block/<identity>` model. Each
block model is `cube_all` with the matching all-face texture, and each item definition directly
selects that block model without a transform. The five block items are common, stack to 64 and
carry the ordinary block-item components. Netherite block additionally has
`minecraft:damage_resistant={types:"#minecraft:is_fire"}`: `ItemEntity#hurtServer` asks
`ItemStack#canBeHurtBy` before changing health, so an item entity holding this default stack rejects
fire-tagged damage while retaining the generic response to other admitted sources.

**Transition and ordering:**

#### Beacon and note-block joins

`beacon_base_blocks` contains exactly these five direct members. The independently owned beacon
base scan accepts mixed identities in each complete 3-by-3 through 9-by-9 layer and stops at the
first nonmember or below-world layer. This leaf owns the exact tag membership; beam scanning,
80-tick refresh, effect selection and criterion publication remain `BLK-BEACON-001`.

The registration instrument is the state value read when one of these blocks is the note block's
substrate. Gold, iron and emerald therefore select bell, iron-xylophone and bit respectively;
diamond and netherite retain the default harp. Note pitch, power/attack admission, block event,
sound/particle projection and statistics remain with the note-block/redstone and generic effect
owners.

Gold block is also a direct `guarded_by_piglins` block member. The generic
`Block#playerWillDestroy` hook therefore invokes nearby-piglin anger before removing a player-broken
gold block. Other destruction paths and the other four identities do not gain that tag-selected
hook.

#### Iron-golem pattern

Iron block is the exact `#` predicate in the lazily built iron-golem base and full patterns:

```text
~^~
###
~#~
```

The four `#` cells must be iron blocks, the four `~` cells must be air, and `^` accepts carved
pumpkin or jack o'lantern. `BlockPattern#find` searches the cube around the supplied head position
and every pair of perpendicular orientation axes, so admission is not restricted to one facing.
Replacing a head by the identical block is ignored; otherwise the carved-pumpkin block's `onPlace`
tries snow, iron and copper full patterns in that order. An iron match creates an iron golem with
spawn reason `TRIGGERED`; a null creation leaves the pattern unchanged.

On successful creation the golem is marked player-created, then the shared spawn transaction:

1. writes air with flags `2` to all nine pattern cells in width-major then height-major order,
   including the four cells already required to be air, and emits level event `2001` with every
   captured old state;
2. snaps the golem to the bottom-center pattern cell plus `(0.5,0.05,0.5)` with yaw/pitch zero and
   calls `addFreshEntity`, ignoring its Boolean result;
3. triggers `summoned_entity` for every server player whose position is selected by the golem AABB
   inflated by 5;
4. calls `updateNeighborsAt(position,AIR)` for all nine cells in the same traversal order.

The carved-pumpkin dispenser behavior first requires an empty target and `canSpawnGolem`; on the
server it writes a north-facing carved pumpkin with flags `3`, emits `BLOCK_PLACE`, shrinks the
stack by one and reports success. The write's placement callback runs the pattern transaction.
When no base pattern matches, the behavior instead tries ordinary equipment dispensing. Dispenser
scheduling, slot selection and level-event residue remain `ITM-DISPENSER-001`.

#### Processing, unlocks and optional trades

Each identity has a 3-by-3 shaped compression recipe from nine matching ingots or gems and a
shapeless decompression recipe from one block to nine source items. Iron, gold and netherite
decompression use groups `iron_ingot`, `gold_ingot` and `netherite_ingot`; diamond and emerald have
no group. Recipe IDs are:

| Block | Compression | Decompression |
|---|---|---|
| iron | `iron_block` | `iron_ingot_from_iron_block` |
| gold | `gold_block` | `gold_ingot_from_gold_block` |
| diamond | `diamond_block` | `diamond` |
| emerald | `emerald_block` | `emerald` |
| netherite | `netherite_block` | `netherite_ingot_from_netherite_block` |

Three iron blocks across the top row, one centered iron ingot and three iron ingots across the
bottom row additionally craft one anvil. Every one of these eleven recipe advancements has one
OR-requirement group: its own `recipe_unlocked` criterion or possession of its input material/block.
Thus direct recipe grant and inventory discovery are alternative unlock paths, not cumulative
requirements.

The built-in `trade_rebalance` pack replaces the armorer level-five trade tag with 16 records while
the shared trade set still selects amount 2. Two optional candidates consume this family:

- a desert, jungle, plains, savanna, snow or swamp armorer takes one iron block and gives four
  emeralds, with maximum uses 12, villager XP 30 and reputation discount 0.05;
- a taiga armorer takes one diamond block and gives 42 emeralds with the same use, XP and discount
  values.

The base pack's armorer tag contains neither record. Pack selection, candidate resolution,
merchant-predicate filtering, random offer selection, price/demand, exhaustion and publication
remain with the generic data-reload and trade owners.

#### Item selectors and piglins

Iron, gold and netherite blocks are direct `sulfur_cube_archetype/slow_flat` item members; diamond
and emerald blocks are direct `slow_bouncy` members. The two locked knockback pairs are respectively
`(horizontal,vertical)=(0.4125,0.105)` and `(0.4125,0.24)`, with hit sounds
`minecraft:entity.sulfur_cube.slow_flat.hit` and
`minecraft:entity.sulfur_cube.slow_bouncy.hit`. Attribute installation, multiple-match order,
equipment changes and swallow/contact handling remain `ENT-KNOCKBACK-001` and the sulfur-cube
owners.

Gold block alone is a direct `piglin_loved` item. It is not the exact barter currency, which remains
the gold ingot. Subject to the generic baby-ignore, repellent, attack/admirer and inventory gates, a
piglin can therefore want a gold-block item entity. Pickup removes exactly one non-nugget item,
puts it in the off hand and sets `ADMIRING_ITEM=true` for 119 ticks. When holding ends, an adult does
not generate barter loot for this stack: it first attempts equipment replacement and otherwise
stores the item. A player holding a gold block also satisfies `isPlayerHoldingLovedItem`, feeding
the nearest-wanted-player memory/look behavior. All other piglin sensing, activity arbitration,
inventory/equipment policy and AI state remain outside this identity leaf.

#### Locked acquisition tables

Beyond block self-loot, the locked tables contain these exact direct item records. Roll ranges are
inclusive uniform integers; weights are relative to the stated direct-entry total:

| Table and pool | Rolls / total weight | Family records |
|---|---|---|
| `chests/bastion_bridge`, pool 1 | 1..2 / 13 | gold w1, count 1 |
| `chests/bastion_hoglin_stable`, pool 0 | 1 / 100 | gold w16, count 2..4 |
| `chests/bastion_other`, pool 1 | 2 / 20 | iron w2, count 1; gold w2, count 1 |
| `chests/bastion_treasure`, pool 1 | 3..4 / 9 | iron w1, count 2..5; gold w1, count 2..5 |
| `chests/ruined_portal`, pool 0 | 4..8 / 398 | gold w1, count 1..2 |
| `chests/trial_chambers/intersection`, pool 0 | 1..3 / 86 | diamond w1, count 1; emerald w5, count 1..3; iron w20, count 1..2 |
| `chests/trial_chambers/reward_ominous_rare`, pool 0 | 1 / 29 | diamond w1, emerald w5, iron w4; count 1 each |
| `pots/trial_chambers/corridor`, pool 0 | 1 / 351 | diamond w1, emerald w5; count 1 each |

Every table uses the matching namespaced table ID as its random sequence. Netherite block is absent
from all non-block locked loot tables. Weighted selection, function evaluation, container filling
and pot breaking remain with their generic loot and owning structure rules.

#### Structure and feature joins

An exhaustive decode of every locked structure NBT finds 161 live family cells in 30 unique
templates and no palette-only occurrence:

| Template family | Identity | Templates | Live cells |
|---|---|---:|---:|
| bastion | gold | 16 | 131 |
| ruined portal | gold | 12 | 28 |
| woodland mansion | diamond | 2 | 2 |

The bastion `blocks/gold` pool separately selects its one-cell air-final connector at weight 3 or
gold-final connector at weight 1. Its gold template stores a jigsaw cell whose final state is
`minecraft:gold_block`; that metadata replacement is not one of the 131 live palette cells.

Five bastion processor lists can turn an admitted gold cell into cracked polished-blackstone
bricks through position-seeded `random_block_match`: `entrance_replacement` uses probability 0.6,
`side_wall_degradation` uses 0.1, and `bastion_generic_degradation`, `high_rampart` and
`rampart_degradation` use 0.3. The ruined-portal code-built rule processor independently turns each
of its gold cells into air on a strict `<0.3` position-seeded float gate.

The code-built ocean-monument core room overwrites the inclusive local box
`x=7..8,y=4..5,z=7..8` with eight gold blocks. The two woodland-mansion secret-room templates
`1x1_as3` and `2x2_s1` contain one diamond block each. Template selection, transforms, clipping,
processor ordering and write transactions remain with
`WGEN-JIGSAW-BASTION-001`, `WGEN-STRUCTURE-RUINED-PORTAL-001`,
`WGEN-STRUCTURE-OCEAN-MONUMENT-001` and `WGEN-STRUCTURE-WOODLAND-MANSION-001`.

Finally, a successful large-dripstone placement calls its diagnostic pass only when
`SharedConstants.DEBUG_LARGE_DRIPSTONE` is enabled. That pass writes a diamond block at the
wind-offset ceiling minus one and a gold block at the wind-offset floor plus one, both with flags
`2`; normal locked generation emits neither marker. The remaining diagnostic column and all
large-dripstone control flow stay `WGEN-PIPELINE-001`.

**Branches and aborts:**

Five identities and tool tiers; correct versus incorrect harvest; surviving versus suppressed
explosion loot; fire-tagged versus other netherite item damage; mixed valid/invalid beacon layers;
five note instruments; player versus dispenser pumpkin placement; every pattern orientation,
creation failure and equipment fallback; guarded versus unguarded destruction; loved nonbarter
pickup gates; two sulfur archetypes; every recipe/unlock, base/optional trade and loot record;
template, connector, processor, code-built and debug-gated generation; and block versus item
projection are distinct.

**Constants and randomness:**

States `2338`, `2339`, `5309`, `9727`, `21818`; destroy/resistance pairs
`(3,6)`, `(5,6)`, `(5,6)`, `(5,6)`, `(50,1200)`; tool tiers stone, iron and
diamond as listed; beacon layers `1..4`; iron pattern four body blocks/four air cells/one head;
pattern clear flags `2`; placement flags `3`; level event `2001`; spawn offset
`(0.5,0.05,0.5)`; criterion inflation `5`; piglin admire expiry `119`; compression/decompression
`9:1` and `1:9`; optional trade amount `2`, uses `12`, XP `30`, discount `0.05`; template census
`30/161`; monument core `2x2x2=8`; processor probabilities `0.1/0.3/0.6`; slow-flat
`0.4125/0.105`; slow-bouncy `0.4125/0.24`. The identities consume no RNG directly; generic loot,
trade, structure, processor and archetype owners retain their documented random selection.

**Side effects:**

The server preserves five exact property-free state identities through ordinary placement,
breaking, recipe/trade/loot acquisition, beacon/note/tag dispatch, golem/piglin/item handling and
generation. It publishes the matching state IDs, item components, entity/block mutations, loot and
models through the already completed block, entity, inventory, effect and terrain protocol
families.

**Gates:**

Selected identity; placement/write authority; correct-tool harvest admission; explosion and item
damage contexts; active block/item/tag/recipe/advancement/loot/trade/archetype snapshots; complete
beacon layer or oriented golem pattern; piglin brain/inventory state; structure/template/processor
selection; locked diagnostic flag; client state/model context.

**State read/written:**

The five block states and their item stacks/components; block/item/tag/recipe/advancement/loot/
trade snapshots; beacon base cells; note instrument selection; piglin brain/offhand/inventory and
anger state through owning callbacks; nine pattern cells, spawned golem/player-created flag and
criterion state; structure/feature cells, processor RNG and chunk palettes; client blockstate/model
selection.

**Persistence boundary:**

All five property-free states persist by registry/state identity. Items persist their components.
Iron-golem pattern consumption and admitted structure writes are immediate world mutations; no
pattern transaction is queued for reload. Tags, recipes, advancements, loot, optional built-in
pack trades and processors rebuild at data reload. Template and code-generated cells thereafter
persist as ordinary block states. Client block/item assets remain resource-pack selected.

**Boundary cases and quirks:**

The beacon tag permits mixed layers, but the iron-golem body accepts only iron blocks. Pattern
search is orientation-complete and clears all nine matched cells, including cells already air.
Dispenser pumpkin placement can fall back to equipment behavior when no golem base matches. Gold
block is piglin-loved but is not barter currency, so admiration does not imply barter output.
The bastion connector's NBT final state is not a live palette cell. Palette-only template entries
are excluded from the 161-cell census. Large-dripstone gold/diamond markers are debug-only, and
netherite's block resistance is independent of its item stack's fire-damage resistance.

**Client projection:**

Terrain/block-update packets select the exact state IDs and ordinary opaque cube models. Inventory
paths select the five direct item models and netherite's component snapshot. Note sounds, block
break event 2001, golem entity spawn/metadata, piglin state effects, beacon consequences and
generated terrain use their completed generic protocol/effect projections; this leaf introduces no
new packet layout.

**Evidence:**

`net.minecraft.world.level.block.Blocks`;
`net.minecraft.world.level.block.state.BlockBehaviour$Properties#strength(float,float)`;
`net.minecraft.world.level.block.state.BlockBehaviour$Properties#instrument(net.minecraft.world.level.block.state.properties.NoteBlockInstrument)`;
`net.minecraft.world.level.block.Block#playerWillDestroy(net.minecraft.world.level.Level,net.minecraft.core.BlockPos,net.minecraft.world.level.block.state.BlockState,net.minecraft.world.entity.player.Player)`;
`net.minecraft.world.level.block.entity.BeaconBlockEntity#updateBase(net.minecraft.world.level.Level,int,int,int)`;
`net.minecraft.world.level.block.state.pattern.BlockPattern#find(net.minecraft.world.level.LevelReader,net.minecraft.core.BlockPos)`;
`net.minecraft.world.level.block.CarvedPumpkinBlock#onPlace(net.minecraft.world.level.block.state.BlockState,net.minecraft.world.level.Level,net.minecraft.core.BlockPos,net.minecraft.world.level.block.state.BlockState,boolean)`;
`net.minecraft.world.level.block.CarvedPumpkinBlock#canSpawnGolem(net.minecraft.world.level.LevelReader,net.minecraft.core.BlockPos)`;
`net.minecraft.world.level.block.CarvedPumpkinBlock#trySpawnGolem(net.minecraft.world.level.Level,net.minecraft.core.BlockPos)`;
`net.minecraft.world.level.block.CarvedPumpkinBlock#spawnGolemInWorld(net.minecraft.world.level.Level,net.minecraft.world.level.block.state.pattern.BlockPattern$BlockPatternMatch,net.minecraft.world.entity.Entity,net.minecraft.core.BlockPos)`;
`net.minecraft.world.level.block.CarvedPumpkinBlock#clearPatternBlocks(net.minecraft.world.level.Level,net.minecraft.world.level.block.state.pattern.BlockPattern$BlockPatternMatch)`;
`net.minecraft.world.level.block.CarvedPumpkinBlock#updatePatternBlocks(net.minecraft.world.level.Level,net.minecraft.world.level.block.state.pattern.BlockPattern$BlockPatternMatch)`;
`net.minecraft.world.level.block.CarvedPumpkinBlock#getOrCreateIronGolemBase()`;
`net.minecraft.world.level.block.CarvedPumpkinBlock#getOrCreateIronGolemFull()`;
`net.minecraft.core.dispenser.DispenseItemBehavior$8#execute(net.minecraft.core.dispenser.BlockSource,net.minecraft.world.item.ItemStack)`;
`net.minecraft.world.entity.monster.piglin.PiglinAi#isLovedItem(net.minecraft.world.item.ItemStack)`;
`net.minecraft.world.entity.monster.piglin.PiglinAi#wantsToPickup(net.minecraft.world.entity.monster.piglin.Piglin,net.minecraft.world.item.ItemStack)`;
`net.minecraft.world.entity.monster.piglin.PiglinAi#pickUpItem(net.minecraft.server.level.ServerLevel,net.minecraft.world.entity.monster.piglin.Piglin,net.minecraft.world.entity.item.ItemEntity)`;
`net.minecraft.world.entity.monster.piglin.PiglinAi#stopHoldingOffHandItem(net.minecraft.server.level.ServerLevel,net.minecraft.world.entity.monster.piglin.Piglin,boolean)`;
`net.minecraft.world.entity.monster.piglin.PiglinAi#isPlayerHoldingLovedItem(net.minecraft.world.entity.LivingEntity)`;
`net.minecraft.world.entity.item.ItemEntity#hurtServer(net.minecraft.server.level.ServerLevel,net.minecraft.world.damagesource.DamageSource,float)`;
`net.minecraft.world.item.ItemStack#canBeHurtBy(net.minecraft.world.damagesource.DamageSource)`;
`net.minecraft.world.item.trading.TradeSet#calculateNumberOfTrades`;
`net.minecraft.world.item.trading.VillagerTrade#getOffer`;
`net.minecraft.world.entity.monster.cubemob.SulfurCube#matchingArchetypes`;
`net.minecraft.world.level.levelgen.structure.structures.OceanMonumentPieces$OceanMonumentCoreRoom#postProcess(net.minecraft.world.level.WorldGenLevel,net.minecraft.world.level.StructureManager,net.minecraft.world.level.chunk.ChunkGenerator,net.minecraft.util.RandomSource,net.minecraft.world.level.levelgen.structure.BoundingBox,net.minecraft.world.level.ChunkPos,net.minecraft.core.BlockPos)`;
`net.minecraft.world.level.levelgen.structure.structures.RuinedPortalPiece#makeSettings(net.minecraft.core.HolderLookup$Provider,net.minecraft.world.level.block.Mirror,net.minecraft.world.level.block.Rotation,net.minecraft.world.level.levelgen.structure.structures.RuinedPortalPiece$VerticalPlacement,net.minecraft.core.BlockPos,net.minecraft.world.level.levelgen.structure.structures.RuinedPortalPiece$Properties)`;
`net.minecraft.world.level.levelgen.feature.LargeDripstoneFeature#placeDebugMarkers(net.minecraft.world.level.WorldGenLevel,net.minecraft.core.BlockPos,net.minecraft.world.level.levelgen.Column$Range,net.minecraft.world.level.levelgen.feature.LargeDripstoneFeature$WindOffsetter)`;
`net.minecraft.SharedConstants#debugFlag(java.lang.String)`;
`reports/blocks.json#minecraft:{iron_block,gold_block,diamond_block,emerald_block,netherite_block}`;
`reports/minecraft/components/item/{iron_block,gold_block,diamond_block,emerald_block,netherite_block}.json`;
`data/minecraft/tags/block/{beacon_base_blocks,guarded_by_piglins,mineable/pickaxe,needs_stone_tool,needs_iron_tool,needs_diamond_tool}.json`;
`data/minecraft/tags/item/{piglin_loved,sulfur_cube_archetype/slow_flat,sulfur_cube_archetype/slow_bouncy}.json`;
`data/minecraft/sulfur_cube_archetype/{slow_flat,slow_bouncy}.json`;
`data/minecraft/loot_table/blocks/{iron_block,gold_block,diamond_block,emerald_block,netherite_block}.json`;
`data/minecraft/loot_table/{chests/bastion_bridge,chests/bastion_hoglin_stable,chests/bastion_other,chests/bastion_treasure,chests/ruined_portal,chests/trial_chambers/intersection,chests/trial_chambers/reward_ominous_rare,pots/trial_chambers/corridor}.json`;
`data/minecraft/recipe/{iron_block,iron_ingot_from_iron_block,gold_block,gold_ingot_from_gold_block,diamond_block,diamond,emerald_block,emerald,netherite_block,netherite_ingot_from_netherite_block,anvil}.json`;
`data/minecraft/advancement/recipes/building_blocks/{iron_block,gold_block,diamond_block,emerald_block,netherite_block}.json`;
`data/minecraft/advancement/recipes/misc/{iron_ingot_from_iron_block,gold_ingot_from_gold_block,diamond,emerald,netherite_ingot_from_netherite_block}.json`;
`data/minecraft/advancement/recipes/decorations/anvil.json`;
`data/minecraft/datapacks/trade_rebalance/data/minecraft/villager_trade/armorer/5/{iron_block_emerald_non_taiga,diamond_block_emerald_taiga}.json`;
`data/minecraft/datapacks/trade_rebalance/data/minecraft/tags/villager_trade/armorer/level_5.json`;
`data/minecraft/trade_set/armorer/level_5.json`;
`data/minecraft/worldgen/template_pool/bastion/blocks/gold.json`;
`data/minecraft/worldgen/processor_list/{bastion_generic_degradation,entrance_replacement,high_rampart,rampart_degradation,side_wall_degradation}.json`;
`data/minecraft/structure/{bastion,ruined_portal,woodland_mansion}/**/*.nbt`;
`assets/minecraft/blockstates/{iron_block,gold_block,diamond_block,emerald_block,netherite_block}.json`;
`assets/minecraft/models/block/{iron_block,gold_block,diamond_block,emerald_block,netherite_block}.json`;
`assets/minecraft/items/{iron_block,gold_block,diamond_block,emerald_block,netherite_block}.json`;
`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-REPORT-001`; `OFF-DATA-001`.

**Test vectors:**

Run `EXP-BLK-048` across all five states and correct/incorrect tools; explosion/self-loot and
netherite fire/nonfire item damage; mixed beacon layers and all five note substrates; ordinary and
dispenser-triggered iron patterns in every orientation, creation/add failure, nine-cell mutation,
events, criterion and neighbors; guarded gold breaking, gold-block item pickup/holding/player
display and nonbarter completion; both sulfur archetypes; all compression/decompression/anvil
recipes and unlock paths; base versus trade-rebalance armorer offers; every listed loot record; all
30 templates, bastion connector/processors, ruined-portal removal, monument core, mansion cells and
disabled/enabled dripstone diagnostics; save/reload and every block/item model. Assert the exact
states, physical values, tag consumers, order, counts, weights, RNG gates, writes and projection.

**Limits:**

Generic block placement/breaking, note-block runtime, beacon runtime, recipes/advancements/loot/
trades, entity spawn/protocol publication, piglin AI outside the gold-block membership paths,
sulfur-cube runtime, structure selection/placement and client rendering remain with
`BLK-PLACE-001`, `BLK-BREAK-001`, `BLK-BREAK-HOOK-001`, `BLK-BEACON-001`, `ITM-RECIPE-001`,
`ITM-ADVANCEMENT-001`, `ITM-LOOT-001`, `ITM-DISPENSER-001`, `ENT-001`,
`MOB-UNIVERSAL-ANGER-001`, `ENT-KNOCKBACK-001`, `WGEN-PIPELINE-001`,
`WGEN-JIGSAW-PROCESSORS-001`, `WGEN-JIGSAW-BASTION-001`,
`WGEN-JIGSAW-TRIAL-CHAMBERS-001`, `WGEN-STRUCTURE-RUINED-PORTAL-001`,
`WGEN-STRUCTURE-OCEAN-MONUMENT-001`, `WGEN-STRUCTURE-WOODLAND-MANSION-001` and `CLI-006`.
Raw material items, anvil behavior, the beacon block/payment items, other guarded or piglin-loved
identities, raw storage blocks, copper storage/weathering, coal/redstone/lapis/bone/amethyst blocks
and non-family loot entries retain their separate owners or `Unreviewed` status.
