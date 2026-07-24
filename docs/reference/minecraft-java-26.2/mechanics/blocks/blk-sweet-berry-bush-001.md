# Blocks mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `BLK-SWEET-BERRY-BUSH-001` — Sweet berry bushes couple four growth stages to harvest, movement damage and animal AI

**Parent:** `SIM-004`, `SIM-005`, `BLK-001`, `BLK-002`, `BLK-003`, `BLK-005`, `RED-001`,
`PLY-002`, `PLY-005`, `PLY-006`, `ITM-003`, `ITM-004`, `ITM-006`, `ITM-007`, `ENT-001`,
`ENT-006`, `MOB-001`, `MOB-004`, `MOB-005`, `MOB-006`, `ENV-003`, `ENV-005`, `WGEN-002`,
`WGEN-003`, `WGEN-004`, `CLI-001`, `CLI-006`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked registration, `SweetBerryBushBlock`, inherited vegetation behavior,
movement and animal consumers, reports, loot, trade, advancement, tag and worldgen data, all 1,212
structure templates and exact client assets close the bush and berry item.

**Applies when:**

`minecraft:sweet_berry_bush` is placed, updated, randomly ticked, bone-mealed, entered, harvested,
broken, burned, grown by a bee, harvested by a fox, generated, persisted or rendered; or
`minecraft:sweet_berries` is placed, eaten, looted, traded, composted, fed to a fox, persisted or
rendered.

**Authoritative state:**

| Identity | Registry ID | State/item ID | Schema or role |
|---|---:|---:|---|
| sweet berry bush | block `861` | states `20941..20944`, default `20941` | `age=0..3` |
| sweet berries | item `1404` | common stack of 64 | custom-named edible block item targeting the bush |

The block has no block entity or fluid property. Registration gives it Plant map color, random
ticks, no collision, Sweet-Berry-Bush sound, piston reaction `DESTROY`, zero hardness/resistance,
zero emission and zero light dampening. It is not position-offset, replaceable, redstone-conducting
or comparator-producing. Its empty collision also makes inherited vegetation AIR pathfinding
available.

Sound type volume/pitch is `1/1`: Sweet Berry Bush Break `1615`, Grass Step `759`, Sweet Berry Bush
Place `1616`, Grass Hit `757` and Grass Fall `756`. Pick uses Sweet Berry Bush Pick Berries `1617`
at volume `1` and pitch `0.8 + nextFloat()*0.4`.

Selection is a centered column of width 10 and height 8 pixels at age zero, width 14 and height 16
at ages one and two, and a full cube at age three. Collision remains empty at every age. Each age
maps to its own untinted crossed-plane model. The item is an untinted generated flat model.

Sweet berries have food nutrition/saturation `2/0.4` and the otherwise-default consumable: a
1.6-second eat animation with ordinary sound and particles. They have no durability, tool,
equippable, use-remainder or item-specific effect component. Generic use completion, hunger,
saturation and stack shrink remain with `ITM-USE-001` and `ITM-HUNGER-001`.

**Transition and ordering:**

#### Placement, support and updates

A block use with sweet berries first attempts their custom block-item placement. If placement does
not consume the action, the consumable component permits fallback to ordinary edible-item use.
Successful placement offers the age-zero default through the generic block-item transaction.

Placement and later survival require the state directly below in `supports_vegetation`. Its locked
closure is the ten identities in `substrate_overworld` plus farmland. There is no light predicate.
Every inherited vegetation shape update rechecks support and returns AIR when it fails.

The bush is not in `maintains_farmland`; farmland below it may dry to dirt. Dirt remains a valid
support, so that transition does not remove the bush. Reloading the support tags changes later
placement/update reads without proactively revisiting existing states.

Clone-item selection returns one sweet-berries stack at every age.

#### Random growth and bone meal

Only ages zero through two are randomly ticking. Each admitted callback reads age, consumes
`nextInt(5)` first and returns unless the result is zero. Only a zero result reads raw brightness
at the position above with ambient subtraction zero; brightness below `9` returns. Admission
offers age plus one at the bush position with flags `2`, ignores the write result, then emits
`BLOCK_CHANGE` with the requested new state even if the offer failed.

Bone meal is valid exactly below age three and reports unconditional success. Performance consumes
no bush RNG: it offers `min(3,age+1)` with flags `2` and ignores the result. The bush method emits
no block-change game event; generic bone-meal item effects retain their owner.

The item-versus-empty-hand dispatcher is deliberate. On ages zero through two, held bone meal
returns `PASS` from block item interaction so bone meal can run. At mature age three, the block
instead requests empty-hand interaction and harvests before the held item can act.

#### Player harvest and breaking loot

An empty-hand interaction at age zero or one passes. At age two or three it returns shared
`SUCCESS` on both sides. The server performs these operations in order:

1. evaluate `minecraft:harvest/sweet_berry_bush` with block-interact context, null tool and the
   player, spawning each result at the block position;
2. play pick sound `1617` at volume `1` and pitch `0.8 + nextFloat()*0.4`;
3. offer the same bush at age one with flags `2`, ignoring the result;
4. emit `BLOCK_CHANGE` at the position with player and requested age-one state context.

The interaction table has two ordered pools. Its unconditional pool emits uniform `1..2`
berries. A preceding age-three-only pool adds exactly one. Age two therefore emits `1..2` and age
three `2..3`; neither tool nor Fortune participates. Any held item other than the immature-age
bone-meal exception reaches the same empty-hand branch first, so an age-two or age-three bush is
harvested before that item can place, eat or otherwise act.

The block-break table has no result at ages zero or one. At age two it emits uniform `1..2`
berries; at age three it emits uniform `2..3`. Each stack then adds a Fortune uniform bonus in
`0..fortuneLevel` and the table-level explosion-decay function applies. There is no Silk Touch or
tool-identity branch. Its namespaced random sequence is `minecraft:blocks/sweet_berry_bush`;
interaction harvest uses the independent `minecraft:harvest/sweet_berry_bush` sequence.

#### Entity contact, fall distance and damage

Only living entities other than exact fox and bee types enter the bush's contact transaction.
Every admitted entity first receives stuck-speed multiplier
`(0.800000011920929,0.75,0.800000011920929)`. Age zero then stops without damage, and client
levels never apply damage.

At ages one through three, the server selects the horizontal movement sample. A
client-authoritative entity uses `getKnownMovement`; every other entity uses
`oldPosition-position`. If horizontal distance squared is positive and either absolute X or Z is
at least `0.003000000026077032`, the entity is offered exactly `1` damage from
`minecraft:sweet_berry_bush`; the result is ignored. Vertical-only or smaller horizontal movement
still receives the slowdown but no damage.

That damage type has effects `poking`, exhaustion `0.1`, message ID `sweetBerryBush` and scaling
`when_caused_by_living_non_player`. It is directly in `bypasses_shield`, `no_knockback` and
`sulfur_cube_with_block_immune_to`; generic immunity, reduction, death-message and hurt effects
remain with the damage owners.

The block is also a direct `fall_damage_resetting` member. During ordinary entity movement with
nonzero fall distance and at least one block of accepted movement, the movement path is ray-cast
up to eight blocks with tag members represented as full shapes. Hitting the bush resets fall
distance even though its ordinary collision is empty.

#### Bee and fox joins

The bush is a direct `bee_growables` member. A crop-growing bee goal is available only while its
post-pollination crop counter is below ten, its activation float is at least `0.3`, it has nectar
and its hive is valid. Each goal tick consumes `nextInt(adjustedTickDelay(30))`; zero scans the
blocks one and then two positions below the bee. Each sweet berry bush below age three becomes an
age-plus-one candidate without a light check.

For every candidate independently, the bee first emits level event `2011` with data `15`, then
calls `setBlockAndUpdate`, ignores its result and increments the crop counter. Thus one tick can
advance both scanned bushes, and an event/counter increment can survive a failed state offer.

Fox berry-search AI accepts a bush at age at least two. After the generic move-to-block gate,
reaching it and waiting 40 goal ticks, `mob_griefing` must be true. The fox computes
`1+nextInt(2)+(age==3 ? 1 : 0)` berries: `1..2` at age two or `2..3` at age three. If its main hand
is empty, one becomes held and the remainder is spawned; otherwise all are spawned. It then plays
pick sound at volume/pitch `1/1`, offers age one with flags `2`, ignores the result and emits
`BLOCK_CHANGE` with fox context.

Sweet berries are a direct `fox_food` member, shared only with glow berries, so generic fox
temptation/breeding consumes them. The fox entity type's direct `fox_immune_to` tag makes the bush
non-dangerous to its path scan, while the contact hook independently excludes foxes from slowdown
and damage.

The direct `happy_ghast_avoids` tag makes careful Ghast movement reject a traversed bush cell even
though its collision shape is empty. Generic goal arbitration and movement remain with their mob
owners.

#### Compost, advancement, trade and acquisition

Sweet berries compost at chance `0.3`: level zero increments without a draw, levels one through
six use the strict generic probability, and an admitted 6-to-7 increment schedules normal
composter maturation. They are not furnace fuel and no bundled recipe consumes or produces them.

`husbandry/balanced_diet` contains sweet berries as one of its 40 independently required consume
criteria and awards 100 experience only after all requirements. Planting the bush is not a
`husbandry/plant_seed` criterion.

The level-five butcher trade exchanges ten sweet berries for one emerald, permits 12 uses, awards
30 villager XP and has reputation discount `0.05`. The taiga-village house chest has `3..8` rolls
over its first pool; sweet berries have weight `5/54` and uniform count `1..7`.

#### Fire, worldgen and client projection

Ordinary fire registers the bush with encouragement/flammability `60/100`. Its block properties do
not set `ignitedByLava`, and the berry item is not fuel.

The configured feature `minecraft:berry_bush` is a simple-block provider fixed to age three.
Placed feature `minecraft:patch_berry_bush` performs 96 attempts with independent trapezoid
offsets X/Z `-7..7` and Y `-3..3`, then requires target AIR and grass block immediately below.
That placed feature occurs with weight `1/39` in ordinary taiga-village decor and `1/26` in zombie
taiga-village decor.

All four planted/nonplanted crimson/warped huge-fungus configurations explicitly admit the bush
in their replaceable-block predicate. No other configured feature directly names it. An exhaustive
scan of all 1,212 structure templates finds zero raw bush cells; village generation reaches it
through the feature-pool element instead.

Food & Drinks orders sweet berries after melon slice and before glow berries. Natural Blocks
orders them after glow berries and before Nether wart.

**Client projection:**

Observers see only committed age/removal writes, item entities, hurt/movement effects, sound and
game/level events, advancement/trade/composter state and loaded models. Rejected offers, support
and light reads, AI searches and private random draws remain server-private. Palette age and
ordinary item components are the reconnect/reload source.

**Branches and aborts:**

Placement versus edible fallback; support; age; random-tick eligibility/draw/light; bone meal;
held-item interaction priority; client/server; interaction versus break loot; Fortune/explosion;
entity type/age/authority/movement; damage tags; fall-reset ray; bee activation/draw/two-height
scan; fox sleep/search/wait/gamerule/hand; compost; advancement/trade/chest; fire; feature/fungus;
template absence; model and tab projection.

**Constants and randomness:**

Maximum age `3`; random growth bound `5`; growth brightness threshold `9`; bone-meal increment `1`;
stuck multiplier `(0.8f,0.75,0.8f)`; hurt threshold `0.003f`; damage `1`; pick pitch
`0.8+nextFloat()*0.4`; interaction counts age two `1..2`, age three `2..3`; break Fortune bonus
`0..level`; bee activation rejects floats below `0.3`, tick bound adjusted from `30`, scan depths
one/two and crop cap ten; fox wait `40` and count draw bound `2`; compost `0.3`; fire `60/100`;
feature attempts `96`.

**Side effects:**

Bush/air writes, block/item loot, player/fox/bee state, stuck movement, damage, fall-distance reset,
sound, level/game events, advancement/trade/composter state, generated cells and client
projection.

**Gates:**

Vegetation support; nonmature age; exact growth draw zero then brightness nine; nonmature bone meal;
age-two interaction; server authority; living nonfox/nonbee contact, nonzero qualifying horizontal
movement; fall movement/raycast; bee nectar/hive/counter/draw/height; fox target/wait/gamerule;
loot conditions, compost, trade, advancement, fire and generation selectors.

**State read/written:**

Reads block identity/age/support/brightness, RNG, held stack, player/entity identity and movement,
fall distance, gamerules, fox hand, bee nectar/hive/counter, active loot/tag/trade/advancement/
worldgen snapshots and client assets. Writes bush/air states, stacks/item entities, entity
movement/health/fall distance, AI counters/hands, composter/trade/progression state, generated cells
and client-visible effects.

**Failure behavior:** unsupported updates return AIR; failed growth, harvest, bone-meal, bee and
fox state offers are not rolled back and their following effects occur as specified; age/light/RNG
rejection is silent; immature empty-hand use passes; excluded entities get neither slowdown nor
damage; false `mob_griefing` prevents fox harvesting; failed data/AI/worldgen gates retain their
parent behavior.

**Persistence boundary:**

Age persists as ordinary palette state with no block entity; berry stacks, entity health/hands,
advancement/trade and composter state persist through their owners. Growth, loot, sound-pitch,
bee/fox selection and movement-damage draws do not persist or catch up. Reload replaces loot,
tags, trade, advancement and worldgen snapshots without rewriting existing palettes or stacks.

**Boundary cases and quirks:**

The bush needs no light to place or survive, and farmland can dry below it. Random growth consumes
its draw before light, while bee growth ignores light entirely. Age zero slows but cannot damage.
Foxes and bees bypass the entire contact hook. Mature selection is a full cube despite empty
collision and a crossed-plane model. Harvesting occurs before a held item's ordinary action and
always requests age one after producing loot. Player harvest uses a loot table; fox harvest uses
separate code with the same count ranges. Bee event/counter effects precede/follow an ignored
write. Village templates contain no raw cells because decor invokes a placed feature.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-DATA-001`; `OFF-REPORT-001`;
`net.minecraft.world.level.block.Blocks`;
`net.minecraft.world.item.Items#createBlockItemWithCustomItemName(net.minecraft.world.level.block.Block)`;
`net.minecraft.world.level.block.SweetBerryBushBlock#randomTick(net.minecraft.world.level.block.state.BlockState,net.minecraft.server.level.ServerLevel,net.minecraft.core.BlockPos,net.minecraft.util.RandomSource)`;
`net.minecraft.world.level.block.SweetBerryBushBlock#entityInside(net.minecraft.world.level.block.state.BlockState,net.minecraft.world.level.Level,net.minecraft.core.BlockPos,net.minecraft.world.entity.Entity,net.minecraft.world.entity.InsideBlockEffectApplier,boolean)`;
`net.minecraft.world.level.block.SweetBerryBushBlock#useItemOn(net.minecraft.world.item.ItemStack,net.minecraft.world.level.block.state.BlockState,net.minecraft.world.level.Level,net.minecraft.core.BlockPos,net.minecraft.world.entity.player.Player,net.minecraft.world.InteractionHand,net.minecraft.world.phys.BlockHitResult)`;
`net.minecraft.world.level.block.SweetBerryBushBlock#useWithoutItem(net.minecraft.world.level.block.state.BlockState,net.minecraft.world.level.Level,net.minecraft.core.BlockPos,net.minecraft.world.entity.player.Player,net.minecraft.world.phys.BlockHitResult)`;
`net.minecraft.world.level.block.SweetBerryBushBlock#performBonemeal(net.minecraft.server.level.ServerLevel,net.minecraft.util.RandomSource,net.minecraft.core.BlockPos,net.minecraft.world.level.block.state.BlockState)`;
`net.minecraft.world.level.block.VegetationBlock#mayPlaceOn(net.minecraft.world.level.block.state.BlockState,net.minecraft.world.level.BlockGetter,net.minecraft.core.BlockPos)`;
`net.minecraft.world.entity.Entity#move(net.minecraft.world.entity.MoverType,net.minecraft.world.phys.Vec3)`;
`net.minecraft.world.entity.animal.bee.Bee$BeeGrowCropGoal#tick()`;
`net.minecraft.world.entity.animal.fox.Fox$FoxEatBerriesGoal#pickSweetBerries(net.minecraft.world.level.block.state.BlockState)`;
`net.minecraft.world.entity.animal.fox.Fox#isFood(net.minecraft.world.item.ItemStack)`;
`net.minecraft.world.entity.EntityType#isBlockDangerous(net.minecraft.world.level.block.state.BlockState)`;
`net.minecraft.world.entity.monster.Ghast$GhastMoveControl#blockTraversalPossible(net.minecraft.world.level.BlockGetter,net.minecraft.world.phys.Vec3,net.minecraft.world.phys.Vec3,net.minecraft.core.BlockPos,boolean,boolean)`;
`net.minecraft.world.level.block.ComposterBlock#bootStrap`;
`net.minecraft.world.level.block.FireBlock#bootStrap`;
`net.minecraft.world.item.CreativeModeTabs#bootstrap(net.minecraft.core.Registry)`;
`reports/blocks.json#minecraft:sweet_berry_bush`;
`reports/registries.json#minecraft:{block,item,damage_type,sound_event}`;
`reports/minecraft/components/item/sweet_berries.json`;
`data/minecraft/damage_type/sweet_berry_bush.json`;
`data/minecraft/loot_table/{blocks,harvest}/sweet_berry_bush.json`;
`data/minecraft/loot_table/chests/village/village_taiga_house.json`;
`data/minecraft/villager_trade/butcher/5/sweet_berries_emerald.json`;
`data/minecraft/advancement/husbandry/balanced_diet.json`;
`data/minecraft/tags/{block/{supports_vegetation,substrate_overworld,bee_growables,fall_damage_resetting,fox_immune_to,happy_ghast_avoids},item/fox_food,damage_type/{bypasses_shield,no_knockback,sulfur_cube_with_block_immune_to}}.json`;
`data/minecraft/worldgen/{configured_feature/berry_bush,placed_feature/patch_berry_bush,template_pool/village/taiga/{decor,zombie/decor}}.json`;
`data/minecraft/worldgen/configured_feature/{crimson_fungus,crimson_fungus_planted,warped_fungus,warped_fungus_planted}.json`;
`data/minecraft/structure/**/*.nbt`;
`assets/minecraft/blockstates/sweet_berry_bush.json`;
`assets/minecraft/models/{block/sweet_berry_bush_stage*,item/sweet_berries}.json`;
`assets/minecraft/items/sweet_berries.json`;
`BLK-STATE-001`; `BLK-PLACE-001`; `BLK-BREAK-001`; `BLK-UPDATE-001`;
`SIM-RANDOM-001`; `ITM-USE-001`; `ITM-HUNGER-001`; `ITM-ADVANCEMENT-001`;
`ITM-LOOT-001`; `ENT-DAMAGE-001`; `ENT-DAMAGE-REDUCE-001`; `ENT-KNOCKBACK-001`;
`MOB-AI-001`; `MOB-BREED-001`; `ENV-FIRE-001`; `WGEN-PIPELINE-001`; `EXP-BLK-081`.

**Test vectors:**

Cross all four ages through placement/support loss/farmland drying, random draw/light, bone meal,
held-item and empty-hand use, block loot/Fortune/explosion, clone and save/reload. Enter with every
entity class across age, authority and horizontal movement thresholds; cross fall-reset rays and
damage tags. Exercise bee activation/two-height/failed-write order and fox search/wait/gamerule/
hand/count/write order. Roll food, balanced diet, compost, trade, chest, fire, all feature/fungus
paths, all templates and every shape/sound/model/tab projection.

**Limits:**

Generic random-tick admission, block-item and consumable transactions, neighbor propagation,
breaking/loot evaluation, bone-meal item effects, entity movement/damage, bee/fox/Ghast goal
scheduling, compost/trade/advancement systems, fire, jigsaw/feature traversal, persistence,
protocol and rendering remain with their cited owners. This leaf owns the two identities'
selectors, constants, local transitions, coupled data joins and projection.
