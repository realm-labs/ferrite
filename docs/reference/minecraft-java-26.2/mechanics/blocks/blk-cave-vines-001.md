# Blocks mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `BLK-CAVE-VINES-001` — Cave vines preserve berry state while a downward-growing head becomes body

**Parent:** `SIM-004`, `SIM-005`, `BLK-001`, `BLK-002`, `BLK-003`, `BLK-005`, `RED-001`,
`PLY-002`, `PLY-005`, `PLY-006`, `ITM-003`, `ITM-004`, `ITM-006`, `ITM-007`, `MOB-001`,
`MOB-004`, `MOB-005`, `MOB-006`, `ENV-003`, `ENV-005`, `WGEN-002`, `WGEN-003`, `WGEN-004`,
`CLI-001`, `CLI-006`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked registrations, growing-plant and cave-vine classes, reports, loot,
advancement, tag and worldgen data, all 1,212 structure templates and exact client assets close
both block identities and the glow-berries item.

**Applies when:**

`minecraft:cave_vines` or `minecraft:cave_vines_plant` is placed, updated, grown, bone-mealed,
grown by a bee, harvested, broken, burned, climbed, glided through, replaced by worldgen,
generated, persisted or rendered; or `minecraft:glow_berries` is placed, eaten, looted, composted,
fed to a fox, persisted or rendered.

**Authoritative state:**

| Identity | Registry ID | State/item ID | Schema or role |
|---|---:|---:|---|
| cave-vine head | block `1135` | states `30249..30300`, default `30250` | `age=0..25`, `berries=true/false` |
| cave-vine body | block `1136` | states `30301..30302`, default `30302` | `berries=true/false` |
| glow berries | item `1405` | common stack of 64 | custom-named edible block item targeting the head |

The head registers Plant map color, random ticks, no collision, state-dependent light, instant
breaking, Cave-Vines sound and piston reaction `DESTROY`; the body has the same properties without
random ticks. Neither has a block entity, fluid property or waterlogging. Both use one centered
column outline, width 14 and height 16 pixels, and an empty collision shape. `berries=true` emits
block light `14`; false emits `0`.

Sound type volume/pitch is `1/1`: Cave Vines Break `306`, Step `310`, Place `309`, Hit `308` and
Fall `307`. Harvest uses Cave Vines Pick Berries `311` at volume `1` and a uniform
`0.8..1.2` pitch.

The head's 26 ages are client-known states but do not select models. Each blockstate file selects
only by `berries`: unlit/lit head or unlit/lit body, each an untinted crossed-plane model. Glow
berries use an untinted generated flat item model.

Glow berries have food nutrition/saturation `2/0.4` and the otherwise-default consumable: a
1.6-second eat animation with ordinary sound and particles. They have no durability, tool,
equippable, use-remainder or item-specific effect component. Generic use completion, hunger,
saturation and stack shrink remain with `ITM-USE-001` and `ITM-HUNGER-001`.

**Transition and ordering:**

#### Placement, support and head/body repair

A block use with glow berries first participates in the block's empty-hand interaction. A
berry-bearing segment harvests and ends dispatch before the held item acts. Otherwise the custom
block item attempts cave-vine placement; if placement does not consume the action, the consumable
component permits ordinary edible-item fallback.

Both identities grow downward. Survival reads the block immediately above and accepts another
cave-vine head/body or a block whose downward face is sturdy. There is no light predicate.
Loss of that support schedules the affected segment's block tick after one tick; the callback
rechecks survival and, on failure, destroys it with drops. The resulting neighbor updates can
cascade down the remaining chain.

Placement reads the target's block below. If it is already a cave-vine head or body, placement
selects the default unlit body. Otherwise it selects an unlit head and consumes
`nextInt(25)`, producing age `0..24`. Placing/growing a head immediately below an existing head
converts the old head into body while preserving its berry bit.

Conversely, when a body no longer has a head or body below, its downward neighbor update converts
that body into a head. The conversion consumes `nextInt(25)` for age `0..24` and preserves the
body's berry bit. Body replacement refuses the head's item while the ordinary growing-plant
replacement predicate would otherwise accept, ensuring extension targets the chain end.

Clone-item selection returns one glow-berries stack from either identity and either berry state.

#### Random head growth

Only heads at ages zero through 24 are randomly ticking. Each callback reads age, consumes
`nextDouble()` and returns unless it is strictly below `0.1`. The admitted branch then reads the
block below and requires AIR. It cycles the head age by one, consumes `nextFloat()`, sets berries
on the proposed new head exactly when that draw is strictly below `0.11`, and offers the state
below with `setBlockAndUpdate`; the result is ignored.

An accepted write's neighbor propagation converts the previous head to body while preserving its
old berry bit. The new head therefore owns the incremented age and newly drawn berry bit. Age 25,
failed admission and non-air below consume no berry draw and make no offer. Growth has no direct
light, sound, particle or game-event gate.

#### Bone meal and bee growth

Both head and body deliberately override generic growing-plant bone meal. A segment is valid
exactly when its berries bit is false, success is unconditional, no cave-vine RNG is consumed, and
performance offers the same segment with `berries=true` and flags `2`; the result is ignored.
Bone meal never lengthens the chain or changes head age. The segment method emits no direct game
event; generic bone-meal item effects retain their owner.

Both blocks are direct `bee_growables` members. The common bee goal retains its counter-below-ten,
activation-float-at-least-`0.3`, nectar, valid-hive and adjusted-bound-30 tick gates, then scans one
and two blocks below the bee. For each unlit cave-vine segment, the cave-vine branch calls the
segment's bone-meal method first, reads the resulting state, emits level event `2011` with data
`15`, redundantly offers that read state with `setBlockAndUpdate`, ignores the result, and
increments the crop counter. It can process both scanned segments in one admitted tick. Unlike
ordinary crops and the sweet-berry branch, the first cave-vine state offer precedes the level
event.

#### Player harvest and break loot

Empty-hand interaction returns `PASS` when berries are absent. With berries present it returns
shared `SUCCESS` on both sides, and the server performs these operations in order:

1. evaluate `minecraft:harvest/cave_vine` with block-interact context, null tool and the player,
   spawning its one glow-berries result at the block position;
2. draw a uniform pitch in `0.8..1.2` and play pick sound `311`;
3. offer the same identity with `berries=false` and flags `2`, ignoring the result;
4. emit `BLOCK_CHANGE` with player and requested unlit-state context.

Because ordinary block item interaction requests the empty-hand path first, a berry-bearing
segment is harvested before held bone meal, glow berries or another item can act. An unlit segment
passes and allows the held item's normal action.

Each block-break table emits exactly one glow berry only when the matching identity has
`berries=true`; false emits nothing. There is no tool, Fortune, Silk Touch or explosion-decay
condition/function. The two random sequences are independently
`minecraft:blocks/cave_vines` and `minecraft:blocks/cave_vines_plant`; interaction harvest uses
`minecraft:harvest/cave_vine`.

#### Movement, tags and acquisition

Both identities are direct `climbable` members and the `cave_vines` tag composes into
`can_glide_through`. Living climbing/fall reset/velocity and fall-flying traversal retain their
player/entity owners. The blocks add no collision volume or contact damage.

Glow berries compost at chance `0.3`. They are not furnace fuel, and no bundled recipe consumes or
produces them. They are a direct `fox_food` member, shared only with sweet berries, so generic fox
temptation and breeding consume them.

`husbandry/balanced_diet` contains glow berries as one of 40 independently required consume
criteria and awards 100 experience only after all requirements. The three default chest sources
are:

| Chest table | Pool rolls | Entry weight / pool weight | Count |
|---|---:|---:|---:|
| abandoned mineshaft | `2..4` | `15/98` | `3..6` |
| ancient city | `5..10` | `3/84` | `1..15` |
| trial-chambers supply | `3..5` | `2/18` | `2..10` |

The trade-rebalance overlay repeats the mineshaft and ancient-city entries unchanged.

#### Fire, worldgen and client projection

Ordinary fire registers both blocks with encouragement/flammability `15/60`. Their properties do
not set `ignitedByLava`.

Configured feature `cave_vine` builds a downward block column in AIR. Its body-layer height first
selects weight `2/15` uniform `0..19`, weight `3/15` uniform `0..2`, or weight `10/15` uniform
`0..6`; every body independently selects unlit/lit at weight `4/1`. One prioritized tip follows,
independently selecting unlit/lit at `4/1` and age uniformly `23..25`.

Placed feature `cave_vines` makes 188 attempts, chooses in-square X/Z and Y uniformly from
dimension bottom through absolute 256, scans upward at most 12 steps through AIR for a sturdy
downward face, offsets down one, then applies the biome gate. Lush caves include it in the
vegetal-decoration step.

Configured feature `cave_vine_in_moss` uses the same providers and prioritized tip, but selects
body height at weight `5/6` uniform `0..3` or `1/6` uniform `1..7`. It is the vegetation child of
`moss_patch_ceiling` at chance `0.08`; that patch uses X/Z radius `4..7`, vertical range `5`,
depth `1..2` and extra-edge chance `0.3`. Placed `lush_caves_ceiling_vegetation` makes 125 attempts,
chooses the same X/Z and height ranges as the direct path, scans upward at most 12 AIR cells for a
SOLID target, offsets down one and applies the biome gate. The composed `moss_replaceable` tag also
allows a moss patch's ground conversion to replace either vine identity.

All four planted/nonplanted crimson/warped huge-fungus configurations explicitly admit both
identities in their replaceable-block predicate. An exhaustive scan of all 1,212 structure
templates finds zero raw head or body cells.

Food & Drinks orders glow berries after sweet berries and before chorus fruit. Natural Blocks
orders them after pitcher pod and before sweet berries.

**Client projection:**

Observers receive committed head/body, age and berries states. Age changes state IDs but not model;
berries switch texture and block emission `0/14`. They also receive item entities, sound,
game/level events, advancement/composter state and the generated flat item model. Rejected offers,
support/air reads, AI scans and private random draws remain server-private.

**Branches and aborts:**

Placement versus harvest/eating; sturdy ceiling/head/body support; head/body conversion; age;
growth double; below AIR; berry float; bone-meal berry bit; client/server; harvest versus break;
climb/glide tag consumers; bee activation/draw/height/berry state; fox food; compost; advancement;
three chest pools; fire; direct/moss column generation; fungus/moss replacement; model and tab
projection.

**Constants and randomness:**

Maximum age `25`; placement/conversion bound `25`; head-growth probability `0.1`; generated-growth
berry probability `0.11`; outline width/height `14/16`; emission `14`; harvest count `1`, pitch
`0.8..1.2`; bee activation rejects floats below `0.3`, tick bound adjusted from `30`, scan depths
one/two and crop cap ten; compost `0.3`; fire `15/60`; direct placed attempts `188`; moss-ceiling
attempts `125`; provider berry weights `4/1`.

**Side effects:**

Head/body/air writes, scheduled block ticks, block/item loot, player/bee state, climbing/gliding
selection, light, sound, level/game events, advancement/composter state, generated cells and client
projection.

**Gates:**

Ceiling or chain support; placement target below; head/body neighbor identity; head age; strict
growth and berry draws; below AIR; false berry bit for bone meal/bee; held-item dispatch; server
authority; loot state; climb/glide tags; bee nectar/hive/counter/draw/height; fox food, compost,
advancement, chest, fire and generation selectors.

**State read/written:**

Reads block identity/age/berries, above/below state and face sturdiness, build height, RNG, held
stack, player/bee identity and state, active loot/tag/advancement/worldgen snapshots and client
assets. Writes head/body/air states, scheduled ticks, stacks/item entities, AI counters,
composter/progression state, generated cells, light and client-visible effects.

**Failure behavior:**

Unsupported scheduled ticks destroy with drops; failed growth, bone-meal, harvest and bee state
offers are ignored without rollback of their stated following effects; age/RNG/air rejection is
silent; unlit interaction passes; lit interaction succeeds even if its unlit write fails; failed
data/AI/worldgen gates retain their parent behavior.

**Persistence boundary:**

Head age/berries and body berries persist as ordinary palette state with no block entity; berry
stacks, advancement and composter state persist through their owners. Growth, placement-age,
conversion-age, harvest-pitch, loot and AI draws do not persist or catch up. Reload replaces loot,
tags, advancement and worldgen snapshots without rewriting existing palettes or stacks.

**Boundary cases and quirks:**

The initial placed head age is random `0..24`, not always zero. Body-to-head conversion randomizes
age while preserving berries. Only the head lengthens the chain, and its prior berry bit remains
on the converted body while the new tip draws independently. Bone meal and bees illuminate one
existing segment rather than extending it. A bee performs the first flags-2 berry write before its
level event and redundant second write. Age is networked but visually ignored. Breaking a lit
segment yields one berry without explosion decay. Lit interaction preempts every held item.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-DATA-001`; `OFF-REPORT-001`;
`net.minecraft.world.level.block.Blocks`;
`net.minecraft.world.item.Items#createBlockItemWithCustomItemName(net.minecraft.world.level.block.Block)`;
`net.minecraft.world.level.block.CaveVines#use(net.minecraft.world.entity.Entity,net.minecraft.world.level.block.state.BlockState,net.minecraft.world.level.Level,net.minecraft.core.BlockPos)`;
`net.minecraft.world.level.block.CaveVines#emission(int)`;
`net.minecraft.world.level.block.CaveVinesBlock#getGrowIntoState(net.minecraft.world.level.block.state.BlockState,net.minecraft.util.RandomSource)`;
`net.minecraft.world.level.block.CaveVinesBlock#performBonemeal(net.minecraft.server.level.ServerLevel,net.minecraft.util.RandomSource,net.minecraft.core.BlockPos,net.minecraft.world.level.block.state.BlockState)`;
`net.minecraft.world.level.block.CaveVinesPlantBlock#performBonemeal(net.minecraft.server.level.ServerLevel,net.minecraft.util.RandomSource,net.minecraft.core.BlockPos,net.minecraft.world.level.block.state.BlockState)`;
`net.minecraft.world.level.block.GrowingPlantBlock#getStateForPlacement(net.minecraft.world.item.context.BlockPlaceContext)`;
`net.minecraft.world.level.block.GrowingPlantBlock#canSurvive(net.minecraft.world.level.block.state.BlockState,net.minecraft.world.level.LevelReader,net.minecraft.core.BlockPos)`;
`net.minecraft.world.level.block.GrowingPlantBlock#tick(net.minecraft.world.level.block.state.BlockState,net.minecraft.server.level.ServerLevel,net.minecraft.core.BlockPos,net.minecraft.util.RandomSource)`;
`net.minecraft.world.level.block.GrowingPlantHeadBlock#randomTick(net.minecraft.world.level.block.state.BlockState,net.minecraft.server.level.ServerLevel,net.minecraft.core.BlockPos,net.minecraft.util.RandomSource)`;
`net.minecraft.world.level.block.GrowingPlantHeadBlock#updateShape(net.minecraft.world.level.block.state.BlockState,net.minecraft.world.level.LevelReader,net.minecraft.world.level.ScheduledTickAccess,net.minecraft.core.BlockPos,net.minecraft.core.Direction,net.minecraft.core.BlockPos,net.minecraft.world.level.block.state.BlockState,net.minecraft.util.RandomSource)`;
`net.minecraft.world.level.block.GrowingPlantBodyBlock#updateShape(net.minecraft.world.level.block.state.BlockState,net.minecraft.world.level.LevelReader,net.minecraft.world.level.ScheduledTickAccess,net.minecraft.core.BlockPos,net.minecraft.core.Direction,net.minecraft.core.BlockPos,net.minecraft.world.level.block.state.BlockState,net.minecraft.util.RandomSource)`;
`net.minecraft.world.entity.animal.bee.Bee$BeeGrowCropGoal#tick()`;
`net.minecraft.world.level.block.ComposterBlock#bootStrap`;
`net.minecraft.world.level.block.FireBlock#bootStrap`;
`net.minecraft.world.item.CreativeModeTabs#bootstrap(net.minecraft.core.Registry)`;
`reports/blocks.json#minecraft:{cave_vines,cave_vines_plant}`;
`reports/registries.json#minecraft:{block,item,sound_event}`;
`reports/minecraft/components/item/glow_berries.json`;
`data/minecraft/loot_table/{blocks/{cave_vines,cave_vines_plant},harvest/cave_vine}.json`;
`data/minecraft/loot_table/chests/{abandoned_mineshaft,ancient_city,trial_chambers/supply}.json`;
`data/minecraft/advancement/husbandry/balanced_diet.json`;
`data/minecraft/tags/{block/{bee_growables,can_glide_through,cave_vines,climbable,moss_replaceable},item/fox_food}.json`;
`data/minecraft/worldgen/biome/lush_caves.json`;
`data/minecraft/worldgen/configured_feature/{cave_vine,cave_vine_in_moss,moss_patch_ceiling,crimson_fungus,crimson_fungus_planted,warped_fungus,warped_fungus_planted}.json`;
`data/minecraft/worldgen/placed_feature/{cave_vines,lush_caves_ceiling_vegetation}.json`;
`data/minecraft/structure/**/*.nbt`;
`assets/minecraft/blockstates/{cave_vines,cave_vines_plant}.json`;
`assets/minecraft/models/{block/cave_vines*,item/glow_berries}.json`;
`assets/minecraft/items/glow_berries.json`;
`BLK-STATE-001`; `BLK-PLACE-001`; `BLK-BREAK-001`; `BLK-UPDATE-001`;
`SIM-RANDOM-001`; `PLY-MOVE-001`; `PLY-MOVE-SPECIAL-001`; `PLY-INTERACT-001`;
`ITM-USE-001`; `ITM-HUNGER-001`; `ITM-ADVANCEMENT-001`; `ITM-LOOT-001`;
`MOB-AI-001`; `MOB-BREED-001`; `ENV-FIRE-001`; `WGEN-PIPELINE-001`; `EXP-BLK-082`.

**Test vectors:**

Cross every head age and both berry bits plus both body bits through placement, support loss,
head/body conversion, random growth, bone meal, held-item/empty-hand use, block loot and
save/reload. Script exact RNG boundaries for `0.1`, `0.11`, placement/conversion age and harvest
pitch. Exercise bee activation/two-height/write-event order, climb/glide and fox-food joins. Roll
all chest/compost/fire and direct/moss/fungus generation paths, all templates and every
shape/light/sound/model/tab projection.

**Limits:**

Generic random-tick admission, block-item and consumable transactions, neighbor propagation,
breaking/loot evaluation, bone-meal item effects, climbing/gliding, bee/fox goal scheduling,
compost/advancement systems, fire, feature traversal, persistence, protocol and rendering remain
with their cited owners. This leaf owns the three identities' selectors, constants, local
transitions, coupled data joins and projection.
