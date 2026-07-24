# Blocks mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `BLK-TORCHFLOWER-CROP-001` — Torchflower crop replaces its second age with a mature flower after an outer growth gate

**Parent:** `SIM-004`, `SIM-005`, `BLK-001`, `BLK-002`, `BLK-003`, `BLK-005`, `RED-001`,
`PLY-005`, `PLY-006`, `ITM-003`, `ITM-004`, `ITM-005`, `ITM-006`, `ITM-007`, `ENT-001`,
`MOB-001`, `MOB-004`, `MOB-005`, `ENV-003`, `ENV-005`, `WGEN-002`, `WGEN-003`, `WGEN-004`,
`CLI-001`, `CLI-006`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked registration, `TorchflowerCropBlock`, inherited crop/flower code,
reports, loot, recipes, advancements, tags, sniffer digging data, all 1,212 structure templates
and client assets close the crop, mature flower and seed item.

**Applies when:**

`minecraft:torchflower_crop` is placed, updated, randomly ticked, bone-mealed, entered, broken,
exploded, replanted by a farmer, persisted or rendered; `minecraft:torchflower` is placed,
updated, bone-mealed, broken, crafted, composted, fed to bees, potted, persisted or rendered; or
`minecraft:torchflower_seeds` is planted, dug up by a sniffer, composted, fed, picked up,
persisted or rendered.

**Authoritative state:**

| Identity | Registry ID | State/item ID | Schema or role |
|---|---:|---:|---|
| torchflower crop | block `662` | states `14797..14798`, default `14797` | `age=0..1`; logical crop max age is `2` |
| mature torchflower | block `159` | state `2323` | property-free `FlowerBlock` |
| mature torchflower | item `272` | stack 64 | ordinary same-name block item |
| torchflower seeds | item `1315` | stack 64 | custom-named block item targeting the crop |

Neither block has a block entity or fluid property. Both use map color Plant, no collision or
occlusion, instant break, piston reaction `DESTROY`, emission/dampening zero, skylight propagation,
no sturdy face, zero signal/comparator output and AIR pathfinding. The crop has no positional
offset; the flower uses the ordinary XZ vegetation offset.

Crop sound volume/pitch is `1/1`: Crop Break `482`, Grass Step `759`, Crop Plant `483`, Grass Hit
`757` and Grass Fall `756`. Its centered width-six selection columns are 6 and 10 pixels high for
ages zero and one. The flower uses Grass Break/Step/Place/Hit/Fall `755/759/758/757/756`; its
width-six, height-ten selection column is shifted by the state-position XZ offset.

Torchflower seeds and the flower have no food, consumable, tool, durability, equippable or use
remainder component. The flower's block definition carries one suspicious-stew effect: Night
Vision amplifier zero for `100` ticks. Its bee interaction effect is null.

**Transition and ordering:**

#### Crop placement, survival and farmland

Using torchflower seeds enters the custom-named block-item transaction and offers crop age zero.
Placement and later survival require raw brightness at least `8` at the crop plus a
`supports_crops` block immediately below; the locked support tag contains exactly farmland.
Inherited vegetation updates immediately return air when either predicate fails.

Crop and mature flower are both direct `maintains_farmland` members, while only the crop is a
direct `crops` member. The crop therefore retains farmland throughout both ages, and replacement
by the mature flower continues to retain it.

The mature flower instead survives on `supports_vegetation`: the eleven-member
`substrate_overworld` closure plus farmland. A relevant update returns air after support loss.
It has no light survival requirement.

#### Random growth and replacement

Both stored crop ages are randomly ticking. The logical maximum is `2`, so neither age zero nor
age one satisfies inherited `isMaxAge` even though the crop's property itself ends at one.

Every admitted callback first consumes `nextInt(3)`. Result zero returns before brightness,
support scan or another draw; results one and two delegate to `CropBlock.randomTick`. That
callback returns below brightness `9`. Otherwise it computes the exact shared 3-by-3
`grows_crops` speed and same-block crowding value, then consumes
`nextInt((int)(25.0f/speed)+1)`.

On a zero shared draw, age zero offers age one with flags `2`. Age one asks for logical age two;
`getStateForAge(2)` returns the mature torchflower default state rather than a crop state, and that
replacement is offered with flags `2`. Write results are ignored. No mature flower random-ticks.

#### Bone meal, entities and mature-flower spread

Crop bone meal is valid at both ages and always succeeds. It adds exactly one without RNG: age
zero offers age one, while age one offers the mature flower, both through the inherited flags-2
write. Bone meal bypasses the outer bound-three, brightness and crop-speed gates.

A Ravager entering either crop age on the server destroys it with drops only while
`mobGriefing` is true, then the inherited contact path completes. Other entities do not mutate it.

The mature flower implements ordinary bush bone meal. It is a valid target only when the generic
spreadable-neighbor search finds a supported destination; success is unconditional, and the
generic neighbor transaction offers another mature torchflower. Crop growth never invokes that
spread transaction.

#### Loot, recipes, advancements and acquisition

The crop table always selects one torchflower-seed entry, then applies explosion decay. Age,
Fortune, Silk Touch and tool identity add no branch. The mature flower table emits one
torchflower only behind `survives_explosion`.

One torchflower crafts shapelessly to one orange dye. Bowl, brown mushroom, red mushroom and one
torchflower craft one suspicious stew carrying Night Vision for `100` ticks. Each recipe
advancement ORs torchflower possession with direct recipe unlock and rewards its recipe.

`husbandry/plant_seed` and hidden `plant_any_sniffer_seed` each include placed
`torchflower_crop` in their single OR group. `feed_snifflet` triggers only when a player interacts
with a baby sniffer using `sniffer_food`; that tag contains exactly torchflower seeds.

The `gameplay/sniffer_digging` gift table has one roll over equal default-weight torchflower-seed
and pitcher-pod alternatives, with its own namespaced random sequence. Generic sniffer search,
dig timing, loot evaluation and item spawning retain their entity/loot owners.

#### Compost, animals, villager and other selectors

Torchflower seeds compost at chance `0.3`; mature torchflower composts at `0.85`. Level zero
increments without an admission draw, levels one through six use the strict generic probability,
and an admitted 6-to-7 increment schedules the normal composter maturation work. Neither item is
furnace fuel.

Seeds are direct `chicken_food`, `parrot_food`, `sniffer_food` and
`villager_plantable_seeds` members. Chickens use generic temptation/breeding; parrots consume them
for the generic one-in-ten tame attempt; sniffers use their generic food/breeding interaction.
Farmer villagers can pick up and plant seeds from the first matching tagged block-item slot, but
cannot harvest this family: neither stored crop state is logically mature, and the replacement
flower is not a `CropBlock`.

The mature flower is direct `bee_food` and `small_flowers` item membership and direct
`bee_attractive` and `small_flowers` block membership. Bee temptation/pollination and flower-pot
insertion/extraction remain with their existing owners.

#### Fire, worldgen and client projection

The crop has fire odds `0/0`; the mature flower has encouragement/flammability `60/100`. Neither
item is fuel. All four planted/nonplanted crimson/warped huge-fungus configurations admit both
crop and mature flower in their replaceable predicate.

An exhaustive scan of all 1,212 structure templates finds zero raw crop or mature-flower cells.
No other configured feature directly names either identity.

Crop age selects two untinted crossed-plane models with stage-zero/stage-one textures. Mature
torchflower uses one untinted crossed-plane block model and a direct generated item model sourced
from its block texture; seeds use a direct generated item model from their item texture. Natural
Blocks orders the flower after lily of the valley and before cactus flower, and orders seeds after
beetroot seeds and before pitcher pod.

**Client projection:**

Observers see only committed age-one or mature-flower replacements, destruction/spread and
inventory/progression effects. Growth scans/draws, rejected destinations and failed writes remain
server-private. Palette states and ordinary item components are the reconnect/reload source.

**Branches and aborts:**

Crop placement/support/light 8; random-position and age eligibility; outer bound-three;
brightness 9; crop-speed/crowding draw; age-zero versus age-one replacement; bone-meal crop versus
flower spread; Ravager/server/gamerule; crop versus flower loot; recipe/unlock; sniffer gift;
compost; animal/villager/bee/pot; fire; fungus and template absence.

**Constants and randomness:**

Outer `nextInt(3)` occurs before all inherited growth reads; shared crop speed uses its locked
base/contributions/crowding and `25/speed` bound. Crop bone-meal increment is exactly `1`.
Compost chances are `0.3/0.85`, mature fire odds `60/100`, and the two digging alternatives have
equal default weight.

**Side effects:**

Crop age or flower replacement, Ravager destruction/drops, mature-flower spread, loot/item
entities, recipe and advancement state, composter state/work, animal/villager state, huge-fungus
replacement and client block/item/effect projection.

**Gates:**

Generic random-position selection; crop age `0..1`; outer bound-three result
nonzero; brightness at least `9`; exact speed-derived draw zero. Bone meal instead requires a valid
crop age or a spreadable mature-flower neighbor. Loot, compost, sniffer and animal paths retain
their owning admission gates.

**State read/written:**

Reads crop/flower identity, age, brightness, support and 3-by-3 states, moisture, RNG, entities,
gamerules, item stacks/components, active loot/recipe/advancement/tag/worldgen snapshots and client
assets. Writes crop/flower/air states, inventories, composter/animal/villager/progression state,
loot entities, generated cells and client-visible effects.

**Failure behavior:** failed support returns air; outer zero, insufficient growth light and
nonzero shared draw do nothing; failed flags-2 growth results do not roll back; crop bone meal
cannot target a noncrop mature flower; missing spread destination rejects mature-flower bone meal;
failed loot/compost/AI/worldgen gates retain their parent behavior.

**Persistence boundary:**

Crop age or mature flower identity persists as an ordinary palette state with no block entity.
Seed/flower stacks persist ordinarily. Growth, bone-meal spread, loot, compost and AI draw cursors
do not persist or catch up; reload replaces tags, loot, recipes, advancements and worldgen data
without rewriting existing palettes or stacks.

**Boundary cases and quirks:**

The crop property exposes only ages zero and one, while logical age two is a different block.
Consequently both crop states random-tick and a farmer never sees a harvestable max-age crop.
The outer growth draw occurs before the inherited brightness check. Crop bone meal is deterministic
and can replace the block after one application at age one. The mature flower keeps farmland from
drying but survives on the broader vegetation substrate. Sniffers can dig up seeds but never the
mature flower.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-DATA-001`; `OFF-REPORT-001`;
`net.minecraft.world.level.block.Blocks`;
`net.minecraft.world.item.Items#createBlockItemWithCustomItemName(net.minecraft.world.level.block.Block)`;
`net.minecraft.world.level.block.TorchflowerCropBlock#getMaxAge()`;
`net.minecraft.world.level.block.TorchflowerCropBlock#getStateForAge(int)`;
`net.minecraft.world.level.block.TorchflowerCropBlock#randomTick(net.minecraft.world.level.block.state.BlockState,net.minecraft.server.level.ServerLevel,net.minecraft.core.BlockPos,net.minecraft.util.RandomSource)`;
`net.minecraft.world.level.block.TorchflowerCropBlock#getBonemealAgeIncrease(net.minecraft.world.level.Level)`;
`net.minecraft.world.level.block.CropBlock#getGrowthSpeed(net.minecraft.world.level.block.Block,net.minecraft.world.level.BlockGetter,net.minecraft.core.BlockPos)`;
`net.minecraft.world.level.block.CropBlock#entityInside(net.minecraft.world.level.block.state.BlockState,net.minecraft.world.level.Level,net.minecraft.core.BlockPos,net.minecraft.world.entity.Entity,net.minecraft.world.entity.InsideBlockEffectApplier,boolean)`;
`net.minecraft.world.level.block.FlowerBlock#getSuspiciousEffects()`;
`net.minecraft.world.level.block.BushBlock#performBonemeal(net.minecraft.server.level.ServerLevel,net.minecraft.util.RandomSource,net.minecraft.core.BlockPos,net.minecraft.world.level.block.state.BlockState)`;
`net.minecraft.world.level.block.ComposterBlock#bootStrap`;
`net.minecraft.world.item.CreativeModeTabs#bootstrap(net.minecraft.core.Registry)`;
`reports/blocks.json#minecraft:{torchflower_crop,torchflower}`;
`reports/registries.json#minecraft:{block,item,sound_event}`;
`reports/minecraft/components/item/{torchflower_seeds,torchflower}.json`;
`data/minecraft/loot_table/{blocks/{torchflower_crop,torchflower},gameplay/sniffer_digging}.json`;
`data/minecraft/recipe/{orange_dye_from_torchflower,suspicious_stew_from_torchflower}.json`;
`data/minecraft/advancement/{recipes/{misc/orange_dye_from_torchflower,food/suspicious_stew_from_torchflower},husbandry/{plant_seed,plant_any_sniffer_seed,feed_snifflet}}.json`;
`data/minecraft/tags/block/{supports_crops,supports_vegetation,crops,maintains_farmland,bee_attractive,small_flowers}.json`;
`data/minecraft/tags/item/{chicken_food,parrot_food,sniffer_food,villager_plantable_seeds,bee_food,small_flowers}.json`;
`data/minecraft/worldgen/configured_feature/{crimson_fungus,crimson_fungus_planted,warped_fungus,warped_fungus_planted}.json`;
`data/minecraft/structure/**/*.nbt`;
`assets/minecraft/blockstates/{torchflower_crop,torchflower}.json`;
`assets/minecraft/models/{block,item}/torchflower*.json`;
`assets/minecraft/items/{torchflower_seeds,torchflower}.json`;
`BLK-STATE-001`; `BLK-PLACE-001`; `BLK-BREAK-001`; `SIM-RANDOM-001`; `ITM-USE-001`;
`ITM-RECIPE-001`; `ITM-CRAFT-001`; `ITM-FURNACE-001`; `ITM-ADVANCEMENT-001`; `ITM-LOOT-001`;
`MOB-AI-001`; `MOB-BREED-001`; `ENV-FIRE-001`; `WGEN-PIPELINE-001`; `EXP-BLK-079`.

**Test vectors:**

Cross both crop states through placement, support/light 7/8/9, dry farmland, outer and shared
growth draws, bone meal, Ravager/gamerule contact, clone and explosion loot. Cross the mature
flower through substrate loss, farmland retention, bone-meal spread, fire, loot, both recipes,
advancements, compost, bee/pot joins and projection. Exercise seed digging, compost, each animal
and villager path, all four fungus configurations, all templates and save/reload.

**Limits:**

Generic random-tick scheduling, crop-speed arithmetic, block-item transactions, neighbor
propagation, breaking/loot evaluation, bone-meal item effects and bush-neighbor selection,
farmland hydration/trampling, crafting, advancements, composting, sniffer/chicken/parrot/bee and
villager AI, flower-pot transactions, huge-fungus geometry, persistence, protocol and rendering
remain with their cited owners. This leaf owns the three identities' selectors, constants, local
transitions, coupled data joins and projection.
