# Blocks mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `BLK-BAMBOO-001` — Bamboo saplings and stalks share one item but use distinct survival, growth and generation transactions

**Parent:** `SIM-004`, `SIM-005`, `BLK-001`, `BLK-002`, `BLK-003`, `BLK-005`, `PLY-005`,
`PLY-006`, `ITM-003`, `ITM-004`, `ITM-006`, `ENT-001`, `MOB-001`, `MOB-004`, `ENV-001`,
`ENV-002`, `ENV-003`, `WGEN-002`, `WGEN-003`, `WGEN-004`, `CLI-001`, `CLI-006`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked registrations, `BambooSaplingBlock`, `BambooStalkBlock`,
`BambooFeature`, bone-meal and placement control flow, complete tags/loot/recipe/worldgen data,
an exhaustive scan of all 1,212 structure templates and exact client assets close both bamboo
block identities and their sole item.

**Applies when:**

`minecraft:bamboo_sapling` or `minecraft:bamboo` is placed, updated, randomly ticked,
bone-mealed, grown, generated, broken, exploded, burned or ignited; when the bamboo item is
crafted with, selected by loot, used by a panda, persisted or rendered.

**Authoritative state:**

The sapling is block raw ID `791`, default/global state `15278`, report type
`minecraft:bamboo_sapling`, property-free and has no registered item. Its clone and every drop
are `minecraft:bamboo`. The stalk is block raw ID `792`, item raw ID `297`, report type
`minecraft:bamboo_stalk`, and has the Cartesian schema:

| Property | Values | Meaning |
|---|---|---|
| `age` | `0`, `1` | thin or thick stalk/model choice |
| `leaves` | `none`, `small`, `large` | leaf overlay and selection width |
| `stage` | `0`, `1` | growing or terminal random-tick state |

States `15279..15290` are ordered first by age, then leaves `none/small/large`, then stage;
`15279` (`age=0,leaves=none,stage=0`) is default. Neither block has a block entity or fluid
property.

Both registrations random-tick, have strength `1/1`, emission `0`, friction `0.6`, speed/jump
factors `1/1`, XZ positional offset, lava ignition and piston reaction `DESTROY`. The sapling uses
map color `WOOD`, Bamboo Sapling sounds and no collision/occlusion; its selection column is
centered, diameter `8/16`, height `12/16`. The stalk uses map color `PLANT`, Bamboo sounds,
dynamic/no-occlusion shape and explicitly never conducts redstone; its selection column is
diameter `6/16`, or `10/16` only for `leaves=large`, and full height. Its collision column is
always diameter `3/16` and full height. Both shapes receive the deterministic XZ offset owned by
`BLK-007`; neither supplies a sturdy full face, comparator output or pathfinding route, and both
propagate skylight.

Both sound types have volume/pitch `1/1`. Stalk break/step/place/hit/fall registry IDs are
`117/121/120/119/118`. Sapling break/place/hit are `122/124/123`, while its step and fall reuse
stalk IDs `121/118`. The bamboo item is common, stacks to 64 and has only the standard default
components.

**Transition and ordering:**

#### Shared support and different placement forms

Both blocks survive exactly when the block immediately below belongs to reloadable
`supports_bamboo`. Its locked 17-member closure is `sand`, `red_sand`, `suspicious_sand`,
`dirt`, `coarse_dirt`, `rooted_dirt`, `mud`, `muddy_mangrove_roots`, `moss_block`,
`pale_moss_block`, `grass_block`, `podzol`, `mycelium`, `bamboo`, `bamboo_sapling`, `gravel`
and `suspicious_gravel`. Light, biome, air above and horizontal neighbors add no support test.

The sapling rechecks support on every shape update and immediately returns air on failure. If
supported, an upward update whose supplied neighbor is bamboo instead returns default bamboo;
all other updates delegate to the inert base behavior.

The bamboo item first rejects any nonempty target fluid, then rejects unsupported ground.
Supported placement selects:

1. default bamboo above a bamboo sapling;
2. default bamboo with `age=(below.age>0)` above bamboo;
3. default bamboo with `age=above.age` when filling a supported gap below bamboo;
4. otherwise the property-free bamboo sapling.

Leaves and stage remain default in all stalk placement branches. Thus the single item has no
sapling item identity, and a support block can select either block implementation.

A scheduled stalk tick destroys the current block with drops when support is absent. A stalk
shape update only schedules that self tick at delay `1`; it does not remove immediately. After
that scheduling decision, an upward bamboo neighbor with a greater age cycles the current age
from `0` to `1`. Other updates delegate to the base implementation.

#### Sapling random and bone-meal growth

Every admitted sapling random tick consumes `nextInt(3)` first. Only zero then tests that the cell
above is empty and has raw brightness at least `9`, in that order. Passing all gates offers
default bamboo with `leaves=small` above using flags `3`. The ignored-result neighbor update from
that offer normally transforms the original sapling to default bamboo; a rejected upper write can
leave it unchanged.

A sapling is a valid bone-meal target exactly when the cell above is air and, only then, inside
build height. Success is unconditional and consumes no RNG. The callback performs the same
ignored-result flags-3 growth. Generic bone-meal ownership consumes one item and emits
`ITEM_INTERACT_FINISH` plus level event `1505` with data `15`; there is no ordinary-sapling
`0.45` miss.

#### Stalk random growth

Only `stage=0` stalks are random-tick eligible. The callback checks stage again, then consumes
`nextInt(3)` before any environmental read. Only zero tests empty above, raw brightness above
`>=9`, and contiguous bamboo height below plus the current block `<16`, in that order. Saplings
do not count toward this height.

For an admitted new segment, the current top's below and two-below states select leaves:

- height below `1`: new leaves `none`;
- otherwise, if below is not bamboo or has leaves `none`: new leaves `small`;
- otherwise: new leaves `large`, then any bamboo below is rewritten to `small` and any bamboo
  two below to `none`, each with flags `3` and ignored result.

The new age is `1` when the current top already has age `1` or two-below is bamboo, otherwise
`0`. Heights below `11` select stage `0` without a float. Heights `11..14` consume one float and
select terminal stage `1` only when it is strictly below `0.25`. Height `15` still consumes that
float but always selects stage `1`. The new state is offered above with flags `3`, ignoring the
result. Maximum successful column height is therefore `16`, and terminal tops stop random ticks.

#### Stalk bone meal

Target admission counts contiguous bamboo above and below the selected stalk. It rejects when
total height is already `16`, the current top is stage `1`, the next target is outside build
height, or that target is nonempty. It does not test brightness. Success is unconditional.

The callback consumes `nextInt(2)` once and attempts one or two segments. Before each attempt it
aborts the entire callback on total `>=16`, terminal current top, nonempty target, or outside
build height; the empty read precedes the bounds test. Each admitted attempt invokes the same
leaf/age/stage grower, then increments its local above and total counts regardless of the ignored
write result. Generic bone-meal consumption and event `1505` remain with `ITM-USE-001`.

#### Loot, crafting, fuel, fire and panda use

The sapling and stalk block tables each have one one-roll bamboo-item pool behind
`survives_explosion`, with sequences `minecraft:blocks/bamboo_sapling` and
`minecraft:blocks/bamboo`. Tool, age, leaves, stage, Silk Touch and Fortune add no branch.
Potted bamboo emits the pot and bamboo separately under `BLK-FLOWER-POT-001`.

Three recipes directly consume bamboo:

- nine bamboo shapelessly produce one bamboo block;
- pattern `I~I / I I / I I` consumes six bamboo and one string for six scaffolding;
- two vertical bamboo produce one stick in group `sticks`.

Each matching recipe advancement uses the OR requirement `has_the_recipe` or inventory containing
bamboo and rewards that recipe.

Jungle-temple pool one rolls uniformly `2..6` and offers bamboo weight `15`, count `1..3`;
shipwreck-supply pool one rolls `3..10` and offers weight `2`, count `1..3`. The optional
trade-rebalance jungle-temple table retains the same bamboo entry. Panda entity loot always emits
one bamboo, and fishing junk offers bamboo weight `10` only in jungle, sparse-jungle or
bamboo-jungle biomes. Those tables retain their own generic loot admission and sequences.

Fuel construction directly assigns bamboo `200/4 = 50` ticks. It is absent from the composter
table. `FireBlock` assigns stalk encouragement/flammability `60/60`; the sapling has no odds entry
and therefore `0/0`. Both can nevertheless be ignited by lava because both registrations opt in.

The item tag `panda_food` contains only bamboo. `Panda#isFood` and its held-item targeting
predicate read that tag, joining the generic panda feeding, ageing, breeding, sitting and eating
transactions without introducing a second bamboo-specific item hook. Reloading the tag changes
future reads, not already committed entity state.

#### Worldgen and structures

Configured features `bamboo_no_podzol` and `bamboo_some_podzol` select `BambooFeature` with
probabilities `0` and `0.2`. `bamboo_light`, used by jungle vegetation, applies rarity chance `4`,
in-square spread, `MOTION_BLOCKING` height and biome filter. `bamboo`, used by bamboo jungle,
applies noise count factor `80`, offset `0.3`, ratio `160`, in-square spread,
`WORLD_SURFACE_WG` height and biome filter.

`BambooFeature#place` first requires the origin to be empty. A nonempty origin returns false.
An empty but unsupported origin performs no write yet returns true—a locked success-counter
quirk. A supported origin draws height `nextInt(12)+5`, then always draws a podzol float.
Only a strict hit below the configured probability draws radius `nextInt(4)+1`. It scans the
inclusive X/Z square, admits offsets with `dx²+dz²<=radius²`, reads
`WORLD_SURFACE(x,z)-1`, and offers podzol with flags `2` only where reloadable
`beneath_bamboo_podzol_replaceable` matches. That tag resolves exactly to the ten identities in
`substrate_overworld`: three dirt, two mud, two moss and three grass-block members.

The vertical loop writes static `age=1,leaves=none,stage=0` stalks with flags `2` while cells are
empty, ignoring results. If its traversed height is at least three, it then offers a terminal
large-leaf state at the first nonempty/upper cell—even overwriting an obstacle—large leaves one
below and small leaves two below, all age `1`, flags `2`, and ignored results. The feature reports
true after this supported path even if every write failed.

An exhaustive decode of all 1,212 bundled structure templates finds zero bamboo-sapling and zero
bamboo-stalk cells. No processor directly names either block. Worldgen selection, placement
modifier RNG, clipping and feature write admission remain with `WGEN-PIPELINE-001`.

**Client projection:**

The sapling blockstate unconditionally selects `block/bamboo_sapling`, a tinted-cross parent using
`block/bamboo_stage0`; there is no bamboo `BlockColors` registration, so its tint index has no
biome color provider. The stalk multipart first selects one of four equal model alternatives for
the current age (`bamboo1..4_age0` or `bamboo1..4_age1`), then adds the small- or large-leaf
overlay; `leaves=none` adds none and stage has no visual branch. Leaf tint index `0` likewise has
no bamboo block-color provider.

The item selector always uses handheld `item/bamboo` with the bamboo item texture, never either
block model. In Natural Blocks it appears between `firefly_bush` and `sugar_cane`. XZ offsets move
both render and shapes together; the authoritative state and raw ID remain unchanged.

**Branches and aborts:**

Two block identities and 13 states; 17 supports; four item-placement forms; immediate sapling
versus scheduled stalk support loss; upward conversion/age propagation; random draw, air,
brightness, height, leaf, age and stage branches; one- or two-step bone meal with bounds/write
failures; two block tables, three recipes/unlocks, chest/panda/fishing acquisition, fuel/fire/tag
reload; two configured and placed worldgen paths, podzol radius/support/obstacle/write outcomes;
save/reload and all client models are distinct.

**Constants and randomness:**

Block IDs `791/792`; item ID `297`; states `15278..15290`; strength `1/1`; sapling shape
`8x12`, stalk selection `6x16` or `10x16`, collision `3x16`; random growth
`nextInt(3)==0`; brightness `9`; maximum height `16`; terminal threshold float `<0.25` at heights
`11..14`; stalk bone meal `1+nextInt(2)`; flags `1/2/3`; fuel `50`; fire odds `0/0` and
`60/60`; feature height `5..16`; podzol probability `0/0.2`, radius `1..4`; structure cells `0`.

**Side effects:**

Conditional sapling/stalk placement; immediate or scheduled support loss; sapling conversion;
age/leaf/stage writes; bone-meal shrink/vibration/event; block and acquisition loot; crafting and
unlock; fuel and fire; panda item sensing/use; podzol/stalk feature writes; palette persistence;
sound, map, tab and multipart block/item projection.

**Gates:**

Generic reach/hand/build/placement permissions; target fluid and current support-tag snapshot;
random-tick chunk/activity/rate admission; stage, air, brightness, height and RNG; bone-meal
target and stack; loot context/explosion/biome; recipe and advancement snapshots; fuel/fire/panda
tag snapshots; active biome/configured/placed feature and podzol tag data; save, registry, pack
and client connection context.

**Boundary cases and quirks:**

The same item normally places a sapling but places a stalk when vertically adjoining an existing
bamboo column. Sapling
support loss is immediate while stalk loss is delayed one scheduled tick. Both random callbacks
draw before reading air or light. Height-15 growth always consumes a float and always terminates.
Bone meal increments local height after an ignored failed write. An empty unsupported worldgen
origin reports success without writing, while a supported feature can overwrite the first
obstacle with its terminal top and reports success even if all writes fail.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-REPORT-001`; `OFF-DATA-001`;
`net.minecraft.world.level.block.Blocks`;
`net.minecraft.world.level.block.BambooSaplingBlock#canSurvive`;
`net.minecraft.world.level.block.BambooSaplingBlock#updateShape`;
`net.minecraft.world.level.block.BambooSaplingBlock#randomTick`;
`net.minecraft.world.level.block.BambooSaplingBlock#growBamboo`;
`net.minecraft.world.level.block.BambooSaplingBlock#isValidBonemealTarget`;
`net.minecraft.world.level.block.BambooSaplingBlock#performBonemeal`;
`net.minecraft.world.level.block.BambooSaplingBlock#getCloneItemStack`;
`net.minecraft.world.level.block.BambooStalkBlock#getStateForPlacement`;
`net.minecraft.world.level.block.BambooStalkBlock#tick`;
`net.minecraft.world.level.block.BambooStalkBlock#updateShape`;
`net.minecraft.world.level.block.BambooStalkBlock#randomTick`;
`net.minecraft.world.level.block.BambooStalkBlock#growBamboo`;
`net.minecraft.world.level.block.BambooStalkBlock#getHeightBelowUpToMax`;
`net.minecraft.world.level.block.BambooStalkBlock#getHeightAboveUpToMax`;
`net.minecraft.world.level.block.BambooStalkBlock#isValidBonemealTarget`;
`net.minecraft.world.level.block.BambooStalkBlock#performBonemeal`;
`net.minecraft.world.level.levelgen.feature.BambooFeature#place`;
`net.minecraft.world.item.BoneMealItem#useOn`;
`net.minecraft.world.level.block.entity.FuelValues#vanillaBurnTimes`;
`net.minecraft.world.level.block.FireBlock#bootStrap`;
`net.minecraft.world.entity.animal.panda.Panda#isFood`;
`net.minecraft.world.item.CreativeModeTabs#bootstrap`;
`reports/blocks.json#minecraft:{bamboo_sapling,bamboo}`;
`reports/registries.json#minecraft:{block,item,sound_event}`;
`reports/minecraft/components/item/bamboo.json`;
`data/minecraft/tags/block/{supports_bamboo,beneath_bamboo_podzol_replaceable,substrate_overworld,sand,dirt,mud,moss_blocks,grass_blocks}.json`;
`data/minecraft/tags/item/panda_food.json`;
`data/minecraft/loot_table/{blocks/{bamboo_sapling,bamboo},entities/panda,chests/{jungle_temple,shipwreck_supply},gameplay/fishing/junk}.json`;
`data/minecraft/recipe/{bamboo_block,scaffolding,stick_from_bamboo_item}.json`;
`data/minecraft/advancement/recipes/{building_blocks/bamboo_block,decorations/scaffolding,misc/stick_from_bamboo_item}.json`;
`data/minecraft/worldgen/{configured_feature/{bamboo_no_podzol,bamboo_some_podzol},placed_feature/{bamboo_light,bamboo},biome/{jungle,bamboo_jungle}}.json`;
`data/minecraft/structure/**/*.nbt`;
`assets/minecraft/blockstates/{bamboo_sapling,bamboo}.json`;
`assets/minecraft/models/block/bamboo*.json`;
`assets/minecraft/{items,models/item}/bamboo.json`.

**Test vectors:**

Run `EXP-BLK-075` across all 13 states and every support, placement, update, random-tick and
bone-meal boundary; force all draw endpoints and accepted/rejected writes; exercise every
loot/recipe/fuel/fire/panda/tag/worldgen branch, all 1,212 templates, save/reload and client
projection. Assert exact IDs, constants, read/draw/write/effect order and negative joins.
