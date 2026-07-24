# Blocks mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `BLK-OVERWORLD-CROP-001` — Ordinary overworld crops share farmland growth but diverge at beetroot RNG, harvest and item use

**Parent:** `SIM-004`, `SIM-005`, `BLK-001`, `BLK-002`, `BLK-003`, `BLK-005`, `RED-001`,
`PLY-005`, `PLY-006`, `ITM-003`, `ITM-004`, `ITM-006`, `ITM-007`, `ENT-001`, `ENT-006`,
`MOB-001`, `MOB-004`, `MOB-005`, `MOB-006`, `ENV-003`, `WGEN-002`, `WGEN-003`,
`WGEN-004`, `CLI-001`, `CLI-006`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked registrations, `CropBlock` and its three concrete overrides, block and
item reports, loot, recipe, advancement, trade, tag and worldgen data, all 1,212 structure
templates and exact client assets close the four crop blocks and seven directly coupled item
identities.

**Applies when:**

`minecraft:wheat`, `minecraft:carrots`, `minecraft:potatoes` or `minecraft:beetroots` is placed,
updated, randomly ticked, bone-mealed, entered by an entity, harvested, exploded, farmed by a
villager, generated, persisted or rendered; or when wheat seeds, wheat, carrot, potato, poisonous
potato, beetroot or beetroot seeds are placed, consumed, crafted, cooked, looted, traded,
composted, fed, picked up, persisted or rendered.

**Authoritative state:**

The eleven identities are:

| Identity | Registry ID | State IDs/default | Schema or item role |
|---|---:|---|---|
| wheat crop | block `207` | `5311..5318`; age zero `5311` | `age=0..7` |
| carrots crop | block `441` | `10659..10666`; age zero `10659` | `age=0..7` |
| potatoes crop | block `442` | `10667..10674`; age zero `10667` | `age=0..7` |
| beetroot crop | block `665` | `14811..14814`; age zero `14811` | `age=0..3` |
| wheat seeds | item `979` | common stack of 64 | custom-named block item for wheat |
| wheat | item `980` | common stack of 64 | plain harvested/crafting item |
| carrot | item `1257` | common stack of 64 | custom-named edible block item for carrots |
| potato | item `1258` | common stack of 64 | custom-named edible block item for potatoes |
| poisonous potato | item `1260` | common stack of 64 | plain edible item |
| beetroot | item `1317` | common stack of 64 | plain edible harvested item |
| beetroot seeds | item `1318` | common stack of 64 | custom-named block item for beetroots |

The four blocks have no same-name registered block item, block entity or fluid property. Wheat
seeds, carrot, potato and beetroot seeds retain their own `item.minecraft:<id>` names and flat item
models even though their `BlockItem` target has another registry ID.

Carrot has food nutrition/saturation `3/3.6000001`, potato `1/0.6`, beetroot `1/1.2` and poisonous
potato `2/1.2`. Their consumables otherwise use the default 1.6-second eat animation, sound and
particles. Poisonous potato additionally runs one apply-effects consumer with probability `0.6`;
success offers Poison amplifier zero for `100` ticks with the ordinary visible icon. Wheat and
both seed items have no food or consumable component. Generic use completion, hunger/saturation,
effect merging and item consumption remain with `ITM-USE-001`, `ITM-HUNGER-001` and
`ENT-EFFECT-001`.

Every crop has no collision or occlusion, instant break, random ticks, piston reaction `DESTROY`,
emission and light dampening zero, skylight propagation, no sturdy face, redstone conduction or
comparator output, shade brightness 1, friction 0.6, speed/jump factors 1 and AIR
pathfindability. Hardness/resistance are `0/0`. Carrots, potatoes and beetroots use map color
`PLANT`; wheat uses `PLANT` at ages `0..5` and `COLOR_YELLOW` at ages `6..7`.

Crop sounds have volume/pitch `1/1`: break/step/place/hit/fall are Crop Break `482`, Grass Step
`759`, Crop Plant `483`, Grass Hit `757` and Grass Fall `756`. Selection shapes are full-width
centered columns with empty collision. Wheat top height is `(2+2*age)/16`, carrots and potatoes
`(2+age)/16`, and beetroots `(2+2*age)/16`.

**Transition and ordering:**

#### Placement, support and farmland retention

Wheat seeds and beetroot seeds always enter their block-item placement path. Carrot and potato
first attempt that same path on a block-use; an unconsumed placement result can fall through to
their ordinary edible-item use. Successful placement offers the target crop's default age-zero
state and consumes through the generic block-item transaction.

Placement and later survival require both raw brightness at the crop position at least `8` and the
state immediately below in `supports_crops`. The locked tag contains exactly farmland. Every
inherited vegetation shape update rechecks both predicates and immediately returns ordinary air
when either fails. A light-only change does not itself synthesize a neighbor shape update; the
next placement or relevant update observes the new brightness.

All four crops are direct `crops` and `maintains_farmland` members. Farmland therefore remains
valid and dry farmland's own random tick does not turn to dirt while any one is directly above.
The `grows_crops` speed tag also contains exactly farmland. Reloading any tag changes later reads
without proactively revisiting existing states.

#### Random growth

Wheat, carrots and potatoes random-tick exactly while age is below seven. Their callback first
reads raw brightness at the current position; a value below `9` returns with no speed scan or RNG.
At brightness at least `9`, calculate shared crop speed `f`:

1. start at `1`;
2. inspect the 3-by-3 plane centered immediately below the crop;
3. each `grows_crops` member contributes `1` when it has no positive `moisture`, or `3` when
   moisture is positive; divide each of the eight off-center contributions by `4`;
4. halve the final sum when the same exact crop block appears on both an east/west and
   north/south axis, or at any diagonal. A different crop species does not count.

The callback consumes `nextInt((int)(25.0f/f)+1)`. Only zero offers age plus one at the same
position with flags `2`; the Boolean result is ignored.

Beetroots random-tick exactly while age is below three, but each admitted callback first consumes
`nextInt(3)`. Result zero returns immediately, before brightness or any shared growth work; results
one and two delegate to the ordinary callback and can consume its crop-speed draw. This extra
two-thirds gate remains observable even at brightness below nine when the callback was admitted.

#### Bone meal and entity contact

Each crop is a valid bone-meal target exactly below its maximum age and success is unconditional.
Wheat, carrots and potatoes consume one inclusive `2..5` draw from the level RNG and offer
`min(7,age+draw)` with flags `2`. Beetroot consumes that same draw first, integer-divides it by
three, and offers `min(3,age+increment)`; source draws `2,3,4,5` therefore add `0,1,1,1`. Even the
zero-increment beetroot branch offers the current state and ignores the write result. Generic
bone-meal item shrink, event and interaction result remain with the item-use owner.

On a server, a Ravager entering any crop while `mob_griefing` is true invokes
`destroyBlock(pos,true,ravager)` and ignores its result. Client levels, other entities and a false
rule do no crop-specific destruction. Every branch then delegates the inherited vegetation
contact callback; the attempted destruction is not rolled back.

Clone-item selection returns wheat seeds, carrot, potato and beetroot seeds for their corresponding
crops at every age.

#### Harvest loot

All four block tables apply table-level `explosion_decay` after their pools and use
`minecraft:blocks/<block-id>` as random sequence:

- wheat age seven emits one wheat from pool zero; every earlier age emits one wheat seed. At age
  seven, pool one additionally starts with one wheat seed and adds
  `Binomial(fortuneLevel+3,0.5714286)`;
- carrots and potatoes always emit one matching crop item from pool zero. At age seven, pool one
  adds another base one plus the same Fortune binomial. Mature potatoes independently run a third
  pool that emits one poisonous potato with chance `0.02`;
- beetroot age three emits one beetroot from pool zero; every earlier age emits one beetroot seed.
  At age three, pool one adds one beetroot seed plus the same Fortune binomial.

There is no tool or Silk Touch branch. Explosion decay can reduce each produced stack, including
the independently generated poisonous potato.

#### Farmer-villager farming and pickup

`villager_plantable_seeds` closes to wheat seeds, potato, carrot, beetroot seeds, torchflower seeds
and pitcher pod; the four members in this leaf are all block items. `villager_picks_up` includes
that whole tag plus bread, wheat and beetroot, so every non-poisonous scoped crop/seed item is
eligible when inventory space exists.

With `mob_griefing` true, farmer profession, absent walk/look targets and a secondary job site,
`HarvestFarmland` searches the 3-by-3-by-3 block cube around the villager for either a max-age
`CropBlock` or air immediately over farmland, then chooses one valid position uniformly. Within
one block of its center and after the behavior cooldown:

- an observed mature crop is destroyed with drops and the villager as breaker; the cached old
  state prevents planting in that same behavior tick;
- an observed air cell over farmland scans inventory slots in order for the first tagged
  `BlockItem`, offers its default state with `setBlockAndUpdate`, ignores the result, emits the
  block-place game event, plays Crop Plant `483` at volume/pitch `1/1`, and shrinks one item.
  Because the write result is ignored, a rejected offer still emits and consumes.

An immature crop is removed from the current candidate list and another is selected with a
20-tick work delay. The behavior works for at most 200 ticks and stopping imposes a 40-tick
restart delay. Generic Brain scheduling, navigation, pickup insertion, sharing and breeding
willingness stay with their mob owners. Carrot, potato and beetroot each count as one villager
food point; wheat and seeds count as zero.

#### Recipes and advancement

The direct recipe joins are:

| Input | Locked recipes |
|---|---|
| wheat | shaped bread `3 -> 1`; shaped cake `3 + 3 milk buckets + 2 sugar + #eggs -> 1`; shaped cookie `2 + cocoa -> 8`; shapeless hay block `9 -> 1`; shapeless mud+wheat `-> 1 packed mud` |
| hay block | shapeless `1 -> 9 wheat` |
| carrot | shaped fishing-rod+carrot `-> 1 carrot on a stick`; shaped eight gold nuggets around carrot `-> 1 golden carrot`; either mushroom-color shapeless rabbit-stew recipe |
| potato | smelting `200`, smoking `100` or campfire cooking `600` ticks to one baked potato, each awarding `0.35` recipe experience |
| beetroot | shapeless bowl+six beetroot `-> 1 beetroot soup`; shapeless one beetroot `-> 1 red dye` |

The bread and hay-block unlocks use wheat inventory criteria; carrot-on-a-stick uses carrot; all
three baked-potato unlocks use potato; beetroot soup and red dye use beetroot; wheat decompression
uses hay block. Cake, cookie, packed mud, golden carrot and rabbit stew instead unlock from their
egg, cocoa, mud, gold-nugget or cooked-rabbit criterion. Every recipe advancement ORs its inventory
criterion with its own `recipe_unlocked` criterion and rewards that recipe.

`husbandry/plant_seed` has one OR group over seven placed-block criteria. Wheat and beetroots are
members; carrots and potatoes are deliberately not. `husbandry/balanced_diet` has 40 independent
required consume criteria and awards 100 experience; carrot, potato, poisonous potato and beetroot
are four of them. Generic matching, assembly, cooking, reward and criterion persistence remain
with the recipe, furnace and advancement owners.

#### Other acquisition, trades and consumers

Baseline direct nonblock loot is:

| Item | Table/pool facts |
|---|---|
| wheat seeds | trail-ruins common archaeology: weight `1/45`; fisher chest: rolls `1..5`, weight `3/11`, count `1..3`; savanna-house chest: rolls `3..8`, weight `10/46`, count `1..5`; unemployed hero gift: sole weight-one entry |
| wheat | cold/warm ocean-ruin archaeology: each weight `2/15`; trail ruins: `2/45`; igloo: rolls `2..8`, `10/63`, count `2..3`; pillager outpost: rolls `2..3`, `7/17`, count `3..5`; shipwreck supply: rolls `3..10`, `7/84`, count `8..21`; simple dungeon: rolls `1..4`, `20/125`, count `1..4`; big/small underwater ruins: rolls `2..8`, `10/33` or `10/30`, count `2..3`; butcher/desert/shepherd village chests: rolls `1..5`,`3..8`,`1..5`, weights `6/28`,`10/36`,`6/23`, counts `1..3`,`1..7`,`1..6`; woodland mansion: rolls `1..4`, `20/175`, count `1..4` |
| carrot | pillager outpost: rolls `2..3`, `5/17`, count `3..5`; shipwreck supply: rolls `3..10`, `7/84`, count `4..8` |
| potato | pillager outpost: rolls `2..3`, `5/17`, count `2..5`; shipwreck supply: rolls `3..10`, `7/84`, count `2..6`; plains/snowy/taiga village chests: rolls `3..8`, weight `10/43`,`10/53`,`10/54`, count `1..7` |
| poisonous potato | shipwreck supply: rolls `3..10`, `7/84`, count `2..6` |
| beetroot seeds | trail-ruins common archaeology: `1/45`; abandoned mineshaft: rolls `2..4`, `10/98`, count `2..4`; End city: rolls `2..6`, `5/89`, count `1..10`; simple dungeon: rolls `1..4`, `10/125`, count `2..4`; snowy village: rolls `3..8`, `10/53`, count `1..5`; woodland mansion: rolls `1..4`, `10/175`, count `2..4` |

Short grass and fern choose their shears self-drop first; otherwise chance `0.125` emits one wheat
seed, applies Fortune uniform-bonus multiplier two, then explosion decay. A valid half of tall
grass or large fern instead chooses the two-item shears result first; otherwise
`survives_explosion` then chance `0.125` emits one wheat seed, with no Fortune function.

Zombie, zombie villager and husk each have one killed-by-player rare pool with three equal-weight
entries: iron ingot, carrot and potato. Admission chance is `0.025` without Looting, or
`0.035 + 0.01*(level-1)` with Looting. A selected potato is furnace-smelted to baked potato when
the dying entity is on fire or the direct attacker's main hand has an enchantment in
`smelts_loot`.

Farmer level one chooses two distinct offers from five. Four sell crops for one emerald:
wheat `20`, potato `26`, carrot `22` or beetroot `15`; each permits 16 uses, awards 2 villager XP
and has reputation discount `0.05`. Wheat-seed and beetroot-seed wandering offers are two of 76
uniform common candidates from which five distinct offers are selected; each exchanges one
emerald for one seed, permits 12 uses and has discount `0.05`.

Code-built composter chances are `0.3f` for both seed items and `0.65f` for wheat, carrot, potato
and beetroot. Poisonous potato is absent. Player or automation insertion at level zero succeeds
without RNG; levels `1..6` compare the level RNG against the widened float chance. Generic
consumption, state/event/schedule and automation consequences remain with the composter
transaction. None of the seven items is fuel; none of the four blocks has fire odds or lava
ignition (`0/0`).

Direct animal-food membership is:

- wheat seeds and beetroot seeds: chicken and parrot;
- wheat: cow, goat, horse, llama and sheep;
- carrot: horse, pig and rabbit;
- potato and beetroot: pig.

The generic love, baby-growth and parrot one-in-ten tame paths remain with `MOB-BREED-001`.
The equine feed table gives wheat heal `2`, baby growth `20` seconds and temper `+3`; carrot gives
heal `3`, growth `60` seconds and temper `+3`. The table only consumes after at least one applicable
heal/growth/temper effect. Poisonous potato has no animal-food membership.

#### World generation and structures

All four ordinary/planted crimson/warped huge-fungus configurations admit all four crop identities
in their replaceable-block predicate.

Five ordinary village farm processors and their five zombie counterparts process template wheat
in ordered first-match rules:

| Village processor pair | Ordered wheat replacements |
|---|---|
| desert | beetroot age zero `0.2`, then melon stem age zero `0.1` |
| plains | carrots age zero `0.3`, potatoes age zero `0.2`, then beetroots age zero `0.1` |
| savanna | melon stem age zero `0.1` |
| snowy | carrots age zero `0.1`, then potatoes age zero `0.8` |
| taiga | pumpkin stem age zero `0.3`, then potatoes age zero `0.2` |

Rules later in a row are tried only after the earlier predicate fails. Position-derived RNG,
template selection, transforms, clipping and accepted writes remain with the village processor
and placement owners.

An exhaustive scan of all 1,212 templates finds no raw carrot, potato or beetroot cells and exactly
722 raw wheat cells in 29 templates: desert village `72/3`, plains `93/4`, savanna `423/16`,
snowy `37/2`, taiga `65/3`, and woodland mansion `32/1` cells/files. Their age distribution is
`0:339`, `1:100`, `2:26`, `3:9`, `4:9`, `5:17`, `6:19`, `7:203`. These are processor inputs and
pool candidates, not unconditional live writes.

**Client projection:**

Wheat ages map one-to-one to eight `wheat_stage0..7` models. Carrot and potato ages map to visual
stages `0,0,1,1,2,2,2,3`; beetroot ages map one-to-one to four stages. Every stage inherits the
untinted `crop` model: four crossed double-sided planes at x/z `4` and `12`, from y `-1` through
`15`, with ambient occlusion and face shading disabled. Texture transparency supplies the visible
stage height; there is no block tint registration.

All seven items directly select like-named generated flat models with no tint, special renderer or
conditional branch. Natural Blocks orders wheat seeds, cocoa beans, pumpkin seeds, melon seeds,
beetroot seeds, torchflower seeds and pitcher pod. Ingredients places wheat between flint and bone.
Food and Drinks orders chorus fruit, carrot, golden carrot, potato, baked potato, poisonous potato,
beetroot and golden dandelion. The scoped items add no Building Blocks entry.

**Branches and aborts:**

Unsupported or too-dark placement; later support/light update loss; mature random-tick exclusion;
beetroot outer zero; brightness below nine; failed shared growth draw/write; invalid mature bone
meal; beetroot zero increment; ravager/client/gamerule branches; crop age/Fortune/explosion and
poisonous-drop branches; block-item placement versus edible fallback; food/effect admission;
villager profession/memory/range/candidate/inventory/write branches; recipe/unlock alternatives;
loot, trade, compost and animal gates; processor/template/fungus write rejection.

**Constants and randomness:**

Maximum ages `7/3`; survival light `8`; growth light `9`; crop numerator `25`; speed base/dry/moist
`1/1/3`, off-center divisor `4`, crowding divisor `2`; one shared bounded growth draw and beetroot
outer bound `3`. Bone meal draws inclusive `2..5`, with beetroot divisor `3`. Fortune adds
`Binomial(level+3,0.5714286)`; poisonous crop chance `0.02`. Food, loot, trade, compost, villager,
animal, processor and template constants are fixed above.

**Side effects:**

Placement/eating and item mutation; crop removal/age writes; farmland continuity; bone-meal
effects; ravager/villager destruction and drops; villager planting, sound and game event; loot
entities; recipe, cooking, advancement, trade, compost, animal and effect state; worldgen cells;
client block, sound, map-color, model and tab projection.

**Gates:**

Generic reach/build/use permissions; support, light, farmland and random-tick admission; exact
growth and bone-meal draws; `mob_griefing`; crop age and loot context; hunger/effect admission;
farmer Brain/memory/range/inventory state; active recipe, advancement, loot, trade, tag and
worldgen snapshots; client asset selection.

**State read/written:**

Reads crop identity/age, brightness, below/3-by-3/horizontal states, moisture, entities, gamerules,
RNG, item components/stacks, villager Brain/inventory, loot/crafting/trade/animal state, worldgen
input and client assets. Writes crop state/air, inventories, food/effect/animal/villager state,
loot, advancement/trade/composter state, generated cells and client-visible effects.

**Persistence boundary:**

Chunk palettes persist only crop identity and age; there is no block entity. Growth scans, random
draws, bone-meal increments, ravager contact and farmer work cursor are not stored in the crop.
Committed cells survive; rejected and pending local transactions do not resume. Item stacks retain
their default components normally. Reload can replace tags, loot, recipes, advancements, trades
and worldgen data without rewriting existing palettes or stacks.

**Boundary cases and quirks:**

Survival light eight is lower than growth light nine. Different crop species do not crowd one
another. Beetroot spends its outer draw before the brightness check and may spend bone meal on a
zero-increment flags-two offer. Carrot and potato are both food and seed block items. Crop loot
uses table-level explosion decay, not a simple survival condition. A farmer's mature destruction
and replant occur on different behavior ticks, while a rejected plant offer still consumes and
emits because its result is ignored. Only wheat and beetroot placement advance `plant_seed`.
Templates contain only wheat; processors derive the other crop species.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-DATA-001`; `OFF-REPORT-001`;
`net.minecraft.world.level.block.Blocks`;
`net.minecraft.world.item.Items#createBlockItemWithCustomItemName(net.minecraft.world.level.block.Block)`;
`net.minecraft.world.level.block.CropBlock#randomTick(net.minecraft.world.level.block.state.BlockState,net.minecraft.server.level.ServerLevel,net.minecraft.core.BlockPos,net.minecraft.util.RandomSource)`;
`net.minecraft.world.level.block.CropBlock#canSurvive(net.minecraft.world.level.block.state.BlockState,net.minecraft.world.level.LevelReader,net.minecraft.core.BlockPos)`;
`net.minecraft.world.level.block.CropBlock#entityInside(net.minecraft.world.level.block.state.BlockState,net.minecraft.world.level.Level,net.minecraft.core.BlockPos,net.minecraft.world.entity.Entity,net.minecraft.world.entity.InsideBlockEffectApplier,boolean)`;
`net.minecraft.world.level.block.CropBlock#growCrops(net.minecraft.world.level.Level,net.minecraft.core.BlockPos,net.minecraft.world.level.block.state.BlockState)`;
`net.minecraft.world.level.block.CropBlock#getGrowthSpeed(net.minecraft.world.level.block.Block,net.minecraft.world.level.BlockGetter,net.minecraft.core.BlockPos)`;
`net.minecraft.world.level.block.BeetrootBlock#randomTick(net.minecraft.world.level.block.state.BlockState,net.minecraft.server.level.ServerLevel,net.minecraft.core.BlockPos,net.minecraft.util.RandomSource)`;
`net.minecraft.world.level.block.BeetrootBlock#getBonemealAgeIncrease(net.minecraft.world.level.Level)`;
`net.minecraft.world.entity.ai.behavior.HarvestFarmland#tick(net.minecraft.server.level.ServerLevel,net.minecraft.world.entity.npc.villager.Villager,long)`;
`net.minecraft.world.entity.npc.villager.Villager#hasFarmSeeds()`;
`net.minecraft.world.entity.npc.villager.Villager#wantsToPickUp(net.minecraft.server.level.ServerLevel,net.minecraft.world.item.ItemStack)`;
`net.minecraft.world.entity.animal.equine.AbstractHorse#handleEating(net.minecraft.world.entity.player.Player,net.minecraft.world.item.ItemStack)`;
`net.minecraft.world.level.block.ComposterBlock#bootStrap`;
`net.minecraft.world.level.block.SoundType`;
`net.minecraft.client.color.block.BlockColors#createDefault`;
`net.minecraft.world.item.CreativeModeTabs#bootstrap(net.minecraft.core.Registry)`;
`data/minecraft/tags/block/{supports_crops,grows_crops,crops,maintains_farmland}.json`;
`data/minecraft/tags/item/{chicken_food,cow_food,goat_food,horse_food,llama_food,parrot_food,
pig_food,rabbit_food,sheep_food,villager_picks_up,villager_plantable_seeds}.json`;
`data/minecraft/loot_table/blocks/{wheat,carrots,potatoes,beetroots}.json`;
`data/minecraft/loot_table/{archaeology,blocks,chests,entities,gameplay}/**/*.json`;
`data/minecraft/recipe/{baked_potato*,beetroot_soup,bread,cake,carrot_on_a_stick,cookie,
golden_carrot,hay_block,packed_mud,rabbit_stew*,red_dye_from_beetroot,wheat}.json`;
`data/minecraft/advancement/{husbandry/{plant_seed,balanced_diet},recipes}/**/*.json`;
`data/minecraft/{villager_trade,trade_set,tags/villager_trade}/{farmer,wandering_trader}/**/*.json`;
`data/minecraft/worldgen/{configured_feature/{crimson_fungus,crimson_fungus_planted,
warped_fungus,warped_fungus_planted},processor_list/{farm_*,zombie_*}}.json`;
`data/minecraft/structure/**/*.nbt`;
`assets/minecraft/blockstates/{wheat,carrots,potatoes,beetroots}.json`;
`assets/minecraft/models/block/{crop,wheat_stage*,carrots_stage*,potatoes_stage*,
beetroots_stage*}.json`;
`assets/minecraft/{items,models/item}/{wheat_seeds,wheat,carrot,potato,poisonous_potato,
beetroot,beetroot_seeds}.json`;
`BLK-STATE-001`; `BLK-PLACE-001`; `BLK-BREAK-001`; `BLK-BREAK-HOOK-001`;
`SIM-RANDOM-001`; `ITM-USE-001`; `ITM-HUNGER-001`; `ITM-RECIPE-001`;
`ITM-CRAFT-001`; `ITM-FURNACE-001`; `ITM-ADVANCEMENT-001`; `ITM-LOOT-001`;
`ENT-EFFECT-001`; `MOB-AI-001`; `MOB-BREED-001`; `WGEN-PIPELINE-001`;
`WGEN-JIGSAW-PROCESSORS-001`; `WGEN-JIGSAW-VILLAGES-001`;
`WGEN-STRUCTURE-WOODLAND-MANSION-001`; `EXP-BLK-078`.

**Test vectors:**

Cross all 28 crop states and seven item stacks through placement/light/support/farmland updates,
every 3-by-3 moisture and same/mixed-crop crowding arrangement, brightness 7/8/9, ordinary and
beetroot draw cursors, bone-meal endpoints, ravager/gamerule contact, clone and exact age/Fortune/
explosion loot. Assert edible-placement fallback, food/effect outcomes, every recipe/unlock,
acquisition/trade/compost/animal path, villager pickup/harvest/plant result, all fungus/processors/
templates, save/reload, sounds, shapes, map colors, stage models and creative ordering.

**Limits:**

Generic random-tick scheduling, block-item admission/commit, neighbor propagation, breaking/loot
evaluation, bone-meal item effects, farmland hydration/trampling, food/hunger/effect execution,
Brain scheduling/navigation, inventory pickup, composter execution, crafting/cooking,
advancements, trades, animal breeding/taming, huge-fungus geometry, structure selection/placement,
persistence, protocol and rendering remain with their cited owners. This leaf owns the eleven
identities' selectors, constants, local transitions, coupled data joins and projection.
