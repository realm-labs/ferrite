# Blocks mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `BLK-PITCHER-CROP-001` — Pitcher crop becomes double-high at age three and keeps a separate placeable mature plant

**Parent:** `SIM-004`, `SIM-005`, `BLK-001`, `BLK-002`, `BLK-003`, `BLK-005`, `RED-001`,
`PLY-005`, `PLY-006`, `ITM-003`, `ITM-004`, `ITM-005`, `ITM-006`, `ITM-007`, `ENT-001`,
`MOB-001`, `MOB-004`, `MOB-005`, `ENV-003`, `ENV-005`, `WGEN-002`, `WGEN-003`, `WGEN-004`,
`CLI-001`, `CLI-006`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked registration, `PitcherCropBlock`, inherited double-plant code, reports,
loot, recipe, advancements, tags, sniffer digging data, all 1,212 structure templates and client
assets close the five-age crop, separately placeable mature plant and both items.

**Applies when:**

`minecraft:pitcher_crop` is placed, updated, randomly ticked, bone-mealed, entered, broken,
exploded, replanted by a farmer, persisted or rendered; `minecraft:pitcher_plant` is placed,
updated, broken, crafted, composted, fed to bees, replaced by worldgen, persisted or rendered; or
`minecraft:pitcher_pod` is planted, dug up by a sniffer, composted, fed, picked up, persisted or
rendered.

**Authoritative state:**

| Identity | Registry ID | State/item ID | Schema or role |
|---|---:|---:|---|
| pitcher crop | block `663` | states `14799..14808`, default `14800` | `age=0..4`, `half=upper/lower`; lower ages zero through two are the normal short forms |
| mature pitcher plant | block `664` | states `14809..14810`, default lower `14810` | independent `DoublePlantBlock`, `half=upper/lower` |
| mature pitcher plant | item `273` | stack 64 | ordinary same-name block item |
| pitcher pod | item `1316` | stack 64 | custom-named block item targeting pitcher crop |

The crop states alternate upper then lower for each age: `14799/14800`, `14801/14802`,
`14803/14804`, `14805/14806` and `14807/14808`. Neither block has a block entity or fluid
property. Both use Plant map color, no collision or occlusion, instant break, Crop sound, piston
reaction `DESTROY`, emission/dampening zero, skylight propagation, no sturdy face, zero
signal/comparator output and AIR pathfinding. The crop has no positional offset; the mature plant
uses the ordinary XZ vegetation offset and is not randomly ticking.

Sound volume/pitch is `1/1`: Crop Break `482`, Grass Step `759`, Crop Plant `483`, Grass Hit `757`
and Grass Fall `756`. Crop selection shapes are centered columns:

| Crop state | Selection shape in pixels | Collision shape |
|---|---|---|
| lower age 0 | width 6, Y `-1..3` | same |
| lower age 1 | width 10, Y `-1..14` | width 10, Y `-1..5` |
| lower ages 2..4 | width 10, Y `-1..16` | width 10, Y `-1..5` |
| upper ages 0..2 | empty | empty |
| upper age 3 | width 10, Y `0..11` | empty |
| upper age 4 | width 10, Y `0..15` | empty |

The mature plant inherits a full-block selection shape with the half-pair's shared position-seeded
XZ offset; collision remains empty. Pod and plant items have no food, consumable, tool, durability,
equippable or use-remainder component.

**Transition and ordering:**

#### Pod placement, survival and update coupling

Using a pitcher pod enters the custom-named block-item transaction. The crop override returns its
default lower age-zero state without the inherited double-plant top-space check, its placement
callback is a no-op, and it cannot be contextually replaced. Successful placement therefore writes
only the lower cell. Immediate and later lower-half survival requires raw brightness at least `8`
and a `supports_crops` block directly below; that tag contains exactly farmland.

The pod can consequently place the lower crop in the top build layer or below an occupied upper
cell. Growth still requires the above position to be inside build height, so a top-layer crop can
never advance. Ages below three are single-cell forms: every shape update returns the state only
when its own survival predicate still holds, otherwise AIR.

At ages three and four, ordinary double-plant update coupling applies. A vertical counterpart
update returns AIR unless the adjacent state is another pitcher crop with the opposite half; age
equality is not checked. A downward update of the lower half additionally rechecks light and
farmland. The upper half survives over any lower pitcher-crop state, again without checking age.

The crop is a direct `crops` and `maintains_farmland` member, so every age retains its supporting
farmland. The separately placeable mature plant is neither member. It instead survives through
`supports_vegetation`, whose closure is the ten-member `substrate_overworld` set plus farmland.
Farmland below a mature plant may therefore dry into dirt, after which the plant remains supported.

#### Random growth and two-cell writes

Only lower halves below age four random-tick. Each admitted callback first computes the exact shared
3-by-3 `CropBlock.getGrowthSpeed` value, including `grows_crops` support contributions and
same-block crowding, then consumes `nextInt((int)(25.0f/speed)+1)`. A nonzero result returns.

A zero draw calls the local grow transaction with increment one. Only then does it reject a
max-age source, raw brightness below `8`, or an above position outside build height. Target ages
one and two impose no above-cell replaceability predicate. Target ages three and four additionally
require the above state to be AIR or any pitcher-crop state; age and half on an existing crop state
do not matter.

An admitted transaction clamps the target at four and offers the lower target-age state with flags
`2`. For target ages three and four it next offers the same-age upper state above with flags `3`.
The lower write precedes the upper write, both results are ignored and neither failure rolls back
the other. Growth thus differs from ordinary crops in three visible ways: brightness `8` is enough,
the speed scan and draw occur before light/build/top checks, and the double-high transition is two
separate writes.

#### Bone meal, entities and double-plant breaking

Bone meal can target either half. An upper target resolves to a lower pitcher-crop state directly
below; a lower target resolves to itself. The pair need not have matching ages. The target is valid
only when that resolved lower state can grow by one under the same light, build-height and
age-three/four above-cell checks. Success is unconditional, and performance deterministically runs
the same increment-one lower-then-upper transaction without consuming RNG or crop-speed state.

A Ravager entering either half on the server destroys only the contacted cell with drops when
`mobGriefing` is true, then returns without invoking the inherited crop contact path. Ordinary
double-plant updates may subsequently remove the counterpart. Other entities do not mutate it.

In survival, the inherited double-plant hook evaluates loot once for the contacted state before
generic removal and suppresses the later duplicate drop. Breaking a lower age-three/four crop can
therefore emit its lower-half loot while the upper disappears; breaking the upper emits no item and
the lower disappears through coupling. A player who prevents drops and breaks an upper half
directly replaces a matching lower half with its fluid or AIR using flags `35` and emits level event
`2001` for that lower state. The independent mature plant uses the same two-half break transaction.

#### Mature-plant placement and loot

The mature pitcher-plant item uses ordinary double-plant placement: the lower target must have
vegetation support, the target Y must be below the build maximum and the upper position must be
replaceable. After the lower block-item write, `setPlacedBy` writes the upper default state with the
ordinary update transaction. Both halves use the lower position when deriving their random XZ
offset.

The crop loot table has one ordered alternatives entry. Lower ages zero through three select
exactly one pitcher pod; lower age four selects exactly one pitcher plant. Every upper-half state
matches no entry. The table-level `explosion_decay` function then applies; Fortune, Silk Touch and
tool identity add no branch.

The mature-plant table similarly selects one pitcher plant only for the lower half and applies
table-level explosion decay. Its upper half yields nothing. This means breaking the upper half of
either double-high identity in survival loses the ordinary drop when counterpart updates remove the
lower cell.

#### Recipe, advancements and sniffer acquisition

One pitcher plant crafts shapelessly to two cyan dye in group `cyan_dye`. Its recipe advancement
ORs plant possession with direct recipe unlock and rewards that recipe.

`husbandry/plant_seed` and the hidden `plant_any_sniffer_seed` each include placed
`pitcher_crop` in their single OR group. The latter has `feed_snifflet` as parent, uses a pitcher-pod
icon and sends telemetry. The pod itself is not `sniffer_food`.

The `gameplay/sniffer_digging` gift table is the renewable pod source: one roll chooses between
equal default-weight torchflower-seed and pitcher-pod entries under its namespaced random sequence.
Generic sniffer search, dig timing, loot evaluation and item spawning retain their existing owners.

#### Compost, animals, villagers and bees

Pitcher pods compost at chance `0.3`; mature pitcher plants compost at `0.85`. Level zero increments
without an admission draw, levels one through six use the strict generic probability, and an
admitted 6-to-7 increment schedules normal composter maturation. Neither item is furnace fuel.

Pods are direct `chicken_food`, `parrot_food` and `villager_plantable_seeds` members. Chickens use
generic temptation/breeding and parrots use the generic one-in-ten tame attempt. Because
`villager_picks_up` includes the plantable-seeds tag, farmer villagers can collect pods.

Farmer planting scans inventory in slot order, obtains a tagged `BlockItem`'s default state and
offers it directly with `setBlockAndUpdate`, then emits `BLOCK_PLACE`, plays Crop Planted and
shrinks one item without consulting the write result. It therefore plants lower age zero but never
creates an upper half. Farmers cannot harvest pitcher crop at any age because their candidate and
harvest branches require `CropBlock`, while `PitcherCropBlock` extends `DoublePlantBlock`.

The mature plant is a direct `bee_food` item and `flowers` item, plus direct `bee_attractive` and
`flowers` block membership. Bee temptation and pollination remain with their generic owners.

#### Fire, worldgen and client projection

The crop has fire odds `0/0`; the mature plant has encouragement/flammability `60/100` and is
ignited by lava. All four planted/nonplanted crimson/warped huge-fungus configurations explicitly
admit pitcher crop, but not the separate mature plant, in their replaceable-block predicate.
Mature plant instead joins the generic tree and huge-mushroom replacement paths through direct
`replaceable_by_trees` and `replaceable_by_mushrooms` tags.

No other configured feature directly names either block. An exhaustive scan of all 1,212 structure
templates finds zero raw pitcher-crop or pitcher-plant cells.

Crop blockstates map every age/half pair to a dedicated model. Lower stages zero through four carry
the visible custom geometry; upper stages zero through two are deliberately empty models, while
upper stages three and four carry the tall geometry. All are untinted. The mature plant maps its two
halves to separate untinted bottom/top plane models. Pod and plant items use direct generated flat
models from their corresponding item textures.

Natural Blocks orders the mature plant after peony and before big dripleaf. It orders pitcher pod
after torchflower seeds and before glow berries.

**Client projection:**

Observers see only committed lower/upper writes, destruction, item/progression effects and loaded
models. Growth speed scans, private draws, rejected top positions and ignored write results remain
server-private. Palette states and ordinary stack components are the reconnect/reload source.

**Branches and aborts:**

Pod versus mature-plant placement; half and age; support/light; age below or at the double-height
intersection; random-tick lower/max eligibility; speed-derived draw; build-height and top
replaceability; bone-meal half recovery; Ravager/server/gamerule; player drop mode and contacted
half; crop age/half loot; recipe/unlock; sniffer gift; compost; animal/villager/bee; fire; fungus,
tree, mushroom and template absence.

**Constants and randomness:**

Maximum age `4`, double-height intersection `3`, bone-meal increment `1`; random growth bound
`(int)(25/speed)+1`; survival and growth brightness threshold `8`; compost chances `0.3/0.85`;
mature fire odds `60/100`; two sniffer-digging alternatives have equal default weight.

**Side effects:**

Crop lower/upper/air writes, mature-plant placement/removal, Ravager/player destruction and drops,
loot/item entities, recipe and advancement state, composter state/work, animal/villager/bee state,
worldgen replacement and client block/item projection.

**Gates:**

Placement support and light; random-position scheduling; lower half and age below four; exact
speed-derived draw zero; brightness/build-height; target-three/four top AIR-or-crop test. Bone meal
skips RNG but retains lower recovery and the same local grow predicates. Loot, compost, sniffer and
AI paths retain their owning admission gates.

**State read/written:**

Reads block identity, age, half, brightness, build height, support, above and 3-by-3 crop states,
moisture, RNG, entities, gamerules, player drop mode, stacks/components, active
loot/recipe/advancement/tag/worldgen snapshots and client assets. Writes crop/plant/air states,
inventories, composter/animal/villager/progression state, loot entities, generated cells and
client-visible effects.

**Failure behavior:** failed short-crop survival returns AIR; missing mature counterparts return
AIR on the relevant vertical update; nonzero growth draws and failed local grow predicates do
nothing; ignored lower/upper writes do not roll back; unresolved bone-meal lower halves return;
upper-half loot is empty; failed data/AI/worldgen gates retain their parent behavior.

**Persistence boundary:**

Age/half and the independent mature-plant identity persist as ordinary palette state with no block
entity. Pod/plant stacks persist ordinarily. Growth, bone-meal, loot, composter, sniffer and farmer
cursors do not persist or catch up; reload replaces tags, loot, recipe, advancements and worldgen
data without rewriting existing palettes or stacks.

**Boundary cases and quirks:**

Pod placement intentionally creates only one cell and bypasses the top-space gate, even though
growth needs an in-height cell above at every target age. Ages three and four require paired halves,
but neither survival nor update coupling checks matching ages. Growth consumes its speed scan and
draw before checking brightness or the future upper cell, and its lower write is not rolled back if
the upper write fails. Bone meal can resolve a mismatched upper/lower pair. Breaking an upper half
yields nothing. Farmers plant pods but never harvest pitcher crop. The separately placeable mature
plant can let farmland dry and participates in tree/mushroom replacement rather than the four
huge-fungus configuration lists.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-DATA-001`; `OFF-REPORT-001`;
`net.minecraft.world.level.block.Blocks`;
`net.minecraft.world.item.Items#createBlockItemWithCustomItemName(net.minecraft.world.level.block.Block)`;
`net.minecraft.world.level.block.PitcherCropBlock#getStateForPlacement(net.minecraft.world.item.context.BlockPlaceContext)`;
`net.minecraft.world.level.block.PitcherCropBlock#updateShape(net.minecraft.world.level.block.state.BlockState,net.minecraft.world.level.LevelReader,net.minecraft.world.level.ScheduledTickAccess,net.minecraft.core.BlockPos,net.minecraft.core.Direction,net.minecraft.core.BlockPos,net.minecraft.world.level.block.state.BlockState,net.minecraft.util.RandomSource)`;
`net.minecraft.world.level.block.PitcherCropBlock#randomTick(net.minecraft.world.level.block.state.BlockState,net.minecraft.server.level.ServerLevel,net.minecraft.core.BlockPos,net.minecraft.util.RandomSource)`;
`net.minecraft.world.level.block.PitcherCropBlock#isValidBonemealTarget(net.minecraft.world.level.LevelReader,net.minecraft.core.BlockPos,net.minecraft.world.level.block.state.BlockState)`;
`net.minecraft.world.level.block.PitcherCropBlock#performBonemeal(net.minecraft.server.level.ServerLevel,net.minecraft.util.RandomSource,net.minecraft.core.BlockPos,net.minecraft.world.level.block.state.BlockState)`;
`net.minecraft.world.level.block.PitcherCropBlock#entityInside(net.minecraft.world.level.block.state.BlockState,net.minecraft.world.level.Level,net.minecraft.core.BlockPos,net.minecraft.world.entity.Entity,net.minecraft.world.entity.InsideBlockEffectApplier,boolean)`;
`net.minecraft.world.level.block.DoublePlantBlock#getStateForPlacement(net.minecraft.world.item.context.BlockPlaceContext)`;
`net.minecraft.world.level.block.DoublePlantBlock#setPlacedBy(net.minecraft.world.level.Level,net.minecraft.core.BlockPos,net.minecraft.world.level.block.state.BlockState,net.minecraft.world.entity.LivingEntity,net.minecraft.world.item.ItemStack)`;
`net.minecraft.world.level.block.DoublePlantBlock#playerWillDestroy(net.minecraft.world.level.Level,net.minecraft.core.BlockPos,net.minecraft.world.level.block.state.BlockState,net.minecraft.world.entity.player.Player)`;
`net.minecraft.world.entity.ai.behavior.HarvestFarmland`;
`net.minecraft.world.level.block.ComposterBlock#bootStrap`;
`net.minecraft.world.item.CreativeModeTabs#bootstrap(net.minecraft.core.Registry)`;
`net.minecraft.world.level.levelgen.feature.TreeFeature`;
`net.minecraft.world.level.levelgen.feature.AbstractHugeMushroomFeature`;
`reports/blocks.json#minecraft:{pitcher_crop,pitcher_plant}`;
`reports/registries.json#minecraft:{block,item,sound_event}`;
`reports/minecraft/components/item/{pitcher_pod,pitcher_plant}.json`;
`data/minecraft/loot_table/{blocks/{pitcher_crop,pitcher_plant},gameplay/sniffer_digging}.json`;
`data/minecraft/recipe/cyan_dye_from_pitcher_plant.json`;
`data/minecraft/advancement/{recipes/misc/cyan_dye_from_pitcher_plant,husbandry/{plant_seed,plant_any_sniffer_seed}}.json`;
`data/minecraft/tags/block/{supports_crops,supports_vegetation,crops,maintains_farmland,bee_attractive,flowers,replaceable_by_trees,replaceable_by_mushrooms}.json`;
`data/minecraft/tags/item/{chicken_food,parrot_food,villager_plantable_seeds,villager_picks_up,bee_food,flowers}.json`;
`data/minecraft/worldgen/configured_feature/{crimson_fungus,crimson_fungus_planted,warped_fungus,warped_fungus_planted}.json`;
`data/minecraft/structure/**/*.nbt`;
`assets/minecraft/blockstates/{pitcher_crop,pitcher_plant}.json`;
`assets/minecraft/models/{block,item}/pitcher*.json`;
`assets/minecraft/items/{pitcher_pod,pitcher_plant}.json`;
`BLK-STATE-001`; `BLK-PLACE-001`; `BLK-BREAK-001`; `BLK-BREAK-HOOK-001`;
`SIM-RANDOM-001`; `ITM-USE-001`; `ITM-RECIPE-001`; `ITM-CRAFT-001`;
`ITM-FURNACE-001`; `ITM-ADVANCEMENT-001`; `ITM-LOOT-001`; `MOB-AI-001`;
`MOB-BREED-001`; `ENV-FIRE-001`; `WGEN-PIPELINE-001`; `EXP-BLK-080`.

**Test vectors:**

Cross all ten crop states through placement, light 7/8, farmland loss, counterpart mismatch,
top-build-height, occupied-top, exact speed/draw endpoints and save/reload. Bone-meal both halves
of matched and mismatched pairs. Break and Ravager-enter each half under both drop/gamerule modes.
Place the mature plant across support/top-space/farmland-drying cases. Roll every crop/plant
age-half loot state, explosion decay, recipe/unlock, digging, compost, animal, villager and bee
path. Exercise all four fungus configurations, generic tree/mushroom replacement, all templates and
every block/item/model/tab projection.

**Limits:**

Generic random-tick scheduling, crop-speed arithmetic, block-item and double-plant transactions,
neighbor propagation, breaking/loot evaluation, bone-meal item effects, farmland
hydration/trampling, crafting, advancements, composting, sniffer/chicken/parrot/bee and villager AI,
tree/mushroom/fungus geometry, persistence, protocol and rendering remain with their cited owners.
This leaf owns the four identities' selectors, constants, local transitions, coupled data joins and
projection.
