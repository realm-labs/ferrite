# Blocks mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `BLK-CORAL-BLOCK-001` — Live coral blocks schedule delayed drying while dead coral blocks are terminal

**Parent:** `SIM-003`, `BLK-001`, `BLK-002`, `BLK-003`, `BLK-005`, `PLY-005`, `PLY-006`,
`ITM-004`, `ITM-006`, `ENT-001`, `MOB-001`, `ENV-001`, `ENV-002`, `ENV-003`, `WGEN-002`,
`WGEN-003`, `CLI-001`, `CLI-006`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked registration and reports, complete loot/tag/trade/archetype/worldgen
data, exhaustive server/client class-reference sweeps, all 1,212 decoded structure templates and
exact client assets close the five live and five dead coral-block identities. Live blocks share one
six-neighbor water scan, delayed scheduled conversion and live-only tag; dead blocks are ordinary
terminal full blocks. Both groups join correct-pickaxe loot, acquisition, equipment and projection.

**Applies when:**

Any of `minecraft:{tube_coral_block,brain_coral_block,bubble_coral_block,fire_coral_block,
horn_coral_block,dead_tube_coral_block,dead_brain_coral_block,dead_bubble_coral_block,
dead_fire_coral_block,dead_horn_coral_block}` is placed, receives a shape update, reaches a
scheduled tick, is harvested or exploded, supports sea-pickle bone meal, is selected by warm-ocean
coral generation, traded, equipped by a sulfur cube, persisted, mapped or rendered.

**Authoritative state:**

Every identity is property-free, has no block entity and fixes one block state:

| Family | Identity | State ID | Block ID | Item ID | Map color |
|---|---|---:|---:|---:|---|
| dead | `dead_tube_coral_block` | `15137` | `748` | `677` | `COLOR_GRAY` |
| dead | `dead_brain_coral_block` | `15138` | `749` | `678` | `COLOR_GRAY` |
| dead | `dead_bubble_coral_block` | `15139` | `750` | `679` | `COLOR_GRAY` |
| dead | `dead_fire_coral_block` | `15140` | `751` | `680` | `COLOR_GRAY` |
| dead | `dead_horn_coral_block` | `15141` | `752` | `681` | `COLOR_GRAY` |
| live | `tube_coral_block` | `15142` | `753` | `682` | `COLOR_BLUE` |
| live | `brain_coral_block` | `15143` | `754` | `683` | `COLOR_PINK` |
| live | `bubble_coral_block` | `15144` | `755` | `684` | `COLOR_PURPLE` |
| live | `fire_coral_block` | `15145` | `756` | `685` | `COLOR_RED` |
| live | `horn_coral_block` | `15146` | `757` | `686` | `COLOR_YELLOW` |

All ten registrations select note instrument `BASEDRUM`, require a correct tool for player drops
and set hardness/resistance `1.5/6`. Dead registrations additionally force solid behavior and
retain the default Stone sound type; live registrations construct `CoralBlock` with the
corresponding dead identity and select the Coral Block sound type. Neither group enables random
ticking, lava ignition or a fluid state. These are not waterloggable blocks: placement into water
replaces the center fluid.

Every state is a full unit selection/collision/visual/occlusion cube with emission `0`, light
dampening `15`, shade brightness `0.2`, friction `0.6`, speed/jump factors `1`, restitution `0`,
solid redstone conduction, normal piston reaction, full sturdy faces and ordinary full-face spawn
support. Dead blocks add no tick, shape-update, use, attack, contact, signal, comparator or
block-event override. Live blocks add only the placement, shape-update and scheduled-tick behavior
below.

Live Coral Block sounds have volume/pitch `1/1` and registry IDs break `444`, step `448`, place
`447`, hit `446` and fall `445`. Dead Stone sounds also have volume/pitch `1/1`, with IDs break
`1596`, step `1604`, place `1601`, hit `1600` and fall `1599`. Every ordinary block item is common,
stacks to `64` and has standard block-item components.

**Transition and ordering:**

#### Water scan, scheduling and terminal conversion

The live block's shared scan visits adjacent fluid states in enum order Down, Up, North, South,
West, East and succeeds on the first member of `minecraft:water`. Flowing water and the water
fluid of an adjacent waterlogged block qualify. The center fluid is never inspected, and a visually
near but nonadjacent water cell does not qualify.

`getStateForPlacement` scans the clicked position before returning the default live state. A wet
scan returns without RNG or scheduling. A dry scan consumes `nextInt(40)`, calls
`Level.scheduleTick(clickedPos,this,60+draw)` and then returns the default state. Scheduling occurs
while placement state is being computed, before the generic authoritative block write; an aborted
write can therefore leave an entry which the scheduler later consumes without a callback when the
current block type does not match.

Every live `updateShape` repeats the scan. Wet updates skip RNG and scheduling. Dry updates first
consume the callback random source's `nextInt(40)`, request a normal-priority self tick after
`60+draw`, then delegate to `Block.updateShape` and return that result. The level scheduler owns
chunk admission and identity/position deduplication: a later duplicate request does not replace or
accelerate an earlier one, although the dry callback has already consumed its draw before the
request is rejected.

At due time the scheduler first requires that the current state still belongs to the scheduled live
block identity. `CoralBlock#tick` then rescans the six neighbors. Water makes the callback a no-op
without RNG or rescheduling. If still dry, it calls
`ServerLevel.setBlock(pos,matchingDead.defaultBlockState(),2)` and ignores the Boolean result.
The conversion produces no loot, sound, particle, game event or explicit neighbor update; flag `2`
publishes the authoritative dead state to tracking clients. A failed write is not retried by this
callback. Adding water before due time therefore preserves the live block without cancelling the
queued record, while removing that water again requires a later placement/update trigger to
schedule another attempt.

Dead blocks never scan, schedule, revive or map back to live blocks. Forced live-state writes do
not call `getStateForPlacement`; only a reached shape update can schedule their drying. Pending
tick persistence, activity, deduplication, type validation and queue order remain with
`SIM-SCHEDULE-001`.

#### Correct-tool and Silk Touch loot

All ten blocks are direct `mineable/pickaxe` members and have no minimum-tier exclusion, so any
pickaxe is a correct player tool. The generic player-break gate suppresses loot for an incorrect
tool before these tables are evaluated.

Each live table makes one alternatives roll. Silk Touch level at least one selects the matching
live item with no explosion condition on that first child. Otherwise it selects the matching dead
item behind `survives_explosion`. Each dead table selects its matching dead item behind
`survives_explosion`. Random sequences are `minecraft:blocks/<identity>`. Fortune and state add no
other branch. Thus an ordinary correct Silk Touch pickaxe preserves live coral, an ordinary correct
non-Silk pickaxe yields its dead counterpart, and an admitted explosion can yield the dead
counterpart from either state.

#### Reloadable tag consumers, trade and negative joins

The block tag `coral_blocks` contains the live identities in exact order tube, brain, bubble, fire,
horn. It contains no dead identity and is not nested by another bundled block tag. Its only
production runtime readers are coral feature selection and sea-pickle bone meal:

- a waterlogged sea pickle is a valid bone-meal target only when the block immediately below is a
  live `coral_blocks` member;
- the spread scans two Y layers over a five-row diamond footprint with widths `1,3,5,3,1`,
  excluding the origin. Each of the 25 candidates first consumes `nextInt(6)`; zero then requires
  exact water at the candidate and a live coral block below, consumes `nextInt(4)+1`, and offers
  that pickle-count state with flags `3`. After the scan the origin is offered with count four and
  flags `2`. Failed offers are ignored. A dead support rejects the initial target and all candidate
  support checks.

All ten items are direct members of `sulfur_cube_archetype/fast_flat`. A matching sulfur cube
therefore selects horizontal/vertical knockback powers `0.9125/0.09`, hit sound
`minecraft:entity.sulfur_cube.fast_flat.hit` and that record's attribute/sound settings; registry
order, multiple matching, equipment mutation and knockback remain with `ENT-KNOCKBACK-001`.

Five wandering-trader records, ordered brain, bubble, fire, horn, tube in the 76-member common tag,
each want three emeralds and give one matching live block. They set maximum uses `8`, reputation
discount `0.05`, and inherit XP `1` with no second cost, predicate, output modifier or double-price
enchantment. The common set chooses five distinct candidates using random sequence
`minecraft:trade_set/wandering_trader/common`. Dead blocks have no trade record. Offer selection,
pricing, exhaustion, restock and purchase commit remain generic.

No identity occurs in a bundled recipe or advancement. None is registered in `FireBlock`'s
encouragement/flammability table or as lava-ignitable, and none enters vanilla fuel construction:
ordinary fire odds are `0/0`, lava does not ignite it and furnace burn time is `0`.

#### Warm-ocean generation and structure absence

Each `coral_claw`, `coral_mushroom` or `coral_tree` invocation uniformly samples one member of the
nonempty live `coral_blocks` tag and uses its property-free default state for every admitted coral
block cell in that invocation. An empty tag returns false before geometry. The shared helper
requires exact water or a `corals` plant/fan at the current cell and exact water above, offers the
selected live block with flags `3`, then performs its independently owned top-coral/pickle and
ordered wall-fan decoration. Exact claw, mushroom, tree traversal and RNG remain with
`WGEN-PIPELINE-001`.

The locked `warm_ocean_vegetation` selector orders inline tree, claw and mushroom features and its
placed wrapper uses noise factor/count ratio `400/20`, in-square, `OCEAN_FLOOR_WG`, then biome.
Dead blocks are never selected. Because every admitted helper cell had exact water above, the
written live block has qualifying adjacent water even though worldgen does not call its placement
method.

The exhaustive NBT scan finds zero cells for all ten identities in all 1,212 bundled structure
templates. Live tag selection by the warm-ocean feature family is therefore their only bundled
terrain-generation source, and dead blocks have no bundled generation source.

**Client projection:**

Every property-free blockstate file unconditionally selects its like-named block model. Each model
inherits `minecraft:block/cube_all` and puts its like-named texture on all six faces. Each item
selector directly uses that same block model. No coral-block model is tinted or rotated.

The Natural Blocks tab orders live tube, brain, bubble, fire and horn blocks, immediately followed
by dead tube, brain, bubble, fire and horn blocks, before the separate coral plants and fans.
Authoritative updates publish states `15137..15146`; inventories use item IDs `677..686`; live and
dead material sounds use their distinct ID sets above; map projection uses the six registration
colors. A successful dry conversion publishes the matching state change without a sound or event.
This leaf adds no packet field, acknowledgement or connection-local state.

**Branches and aborts:**

Ten identities; live/dead; every six-direction first-water position, flowing/waterlogged water and
no water; placement versus forced write; wet/dry placement and shape update; every delay draw,
deduplicated/missing-chunk request, active/inactive/reloaded queue, type mismatch, rehydrated/dry due
tick and failed conversion; correct/incorrect tool, Silk Touch level and explosion survival; live/
dead sea-pickle support and all 25 candidate branches; every common-trade selection and offer
lifecycle; fast-flat matching; fire/lava/fuel absence; empty/nonempty coral tag, all five selections,
three feature types and cell/write branches; zero template selection; persistence, tab, sound, map,
block/item/model projection are distinct branches.

**Constants and randomness:**

States `15137..15146`; block IDs `748..757`; item IDs `677..686`; strength `1.5/6`; emission `0`,
dampening `15`, shade `0.2`, friction `0.6`, speed/jump `1`, restitution `0`, stack `64`;
BASEDRUM; Coral sound IDs break/step/place/hit/fall `444/448/447/446/445`; Stone IDs
`1596/1604/1601/1600/1599`; water order Down/Up/North/South/West/East; dry delay
`60+nextInt(40)` or `60..99`, conversion flags `2`; pickle candidates `25`, admission
`nextInt(6)==0`, count `nextInt(4)+1`, candidate/origin flags `3/2`; fire `0/0`, fuel `0`; trade
cost/output/uses/XP/discount `3/1/8/1/0.05`, common selection `5` of `76`; fast-flat powers
`0.9125/0.09`; warm-ocean factor/ratio `400/20`; scanned templates/cells `1212/0`. Wet scans,
dead blocks, loot before explosion decay and fixed client selection consume no RNG.

**Side effects:**

Ordinary full-block placement; dry placement/update tick requests; silent live-to-dead state
publication; correct-tool live/dead loot; reload-selected pickle support, warm-ocean generation,
trade and sulfur-equipment selection; explicit recipe/advancement/fire/fuel absence; ordinary
palette and inventory persistence; material sounds, map colors and cube-all projection.

**Gates:**

Placement/write and break authority; adjacent water-fluid tag; scheduler chunk, deduplication,
activity, due and type gates; correct player tool, Silk Touch and explosion context; active loot,
tag, trade, archetype and worldgen snapshots; sea-pickle waterlogged/support/candidate gates;
trade-set and offer admission; feature selector, water, placement-modifier and write gates; valid
registry, sound, map, creative-tab and client-resource context.

**Boundary cases and quirks:**

The block being placed may replace water yet still die because only adjacent fluids count. A dry
placement draws and requests its tick before the generic placement write, while a forced state does
neither. Repeated dry updates consume repeated delay draws even when scheduler deduplication keeps
the first request. Rehydration suppresses conversion at due time without removing the pending tick;
dead coral never revives. The flags-2 conversion ignores failure and deliberately does not notify
neighbors. Live loot's Silk Touch alternative lacks `survives_explosion`, whereas its dead fallback
and every dead table include it. `coral_blocks` means live full blocks only; the separate `corals`
tag means live plants and fans, not these ten blocks.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-REPORT-001`; `OFF-DATA-001`;
`net.minecraft.world.level.block.Blocks`;
`net.minecraft.world.level.block.SoundType`;
`net.minecraft.world.level.block.CoralBlock#tick`;
`net.minecraft.world.level.block.CoralBlock#updateShape`;
`net.minecraft.world.level.block.CoralBlock#scanForWater`;
`net.minecraft.world.level.block.CoralBlock#getStateForPlacement`;
`net.minecraft.core.Direction`;
`net.minecraft.world.level.block.SeaPickleBlock#isDead`;
`net.minecraft.world.level.block.SeaPickleBlock#isValidBonemealTarget`;
`net.minecraft.world.level.block.SeaPickleBlock#performBonemeal`;
`net.minecraft.world.level.levelgen.feature.CoralFeature#place`;
`net.minecraft.world.level.levelgen.feature.CoralFeature#placeCoralBlock`;
`net.minecraft.world.item.trading.TradeSet#calculateNumberOfTrades`;
`net.minecraft.world.item.trading.VillagerTrade#getOffer`;
`net.minecraft.world.entity.monster.cubemob.SulfurCube#matchingArchetypes`;
`net.minecraft.world.level.block.FireBlock#bootStrap`;
`net.minecraft.world.level.block.entity.FuelValues#vanillaBurnTimes`;
`net.minecraft.world.item.CreativeModeTabs#bootstrap`;
`reports/blocks.json#minecraft:{tube_coral_block,brain_coral_block,bubble_coral_block,
fire_coral_block,horn_coral_block,dead_tube_coral_block,dead_brain_coral_block,
dead_bubble_coral_block,dead_fire_coral_block,dead_horn_coral_block}`;
`reports/registries.json#minecraft:{block,item,sound_event}`;
`reports/minecraft/components/item/{tube_coral_block,brain_coral_block,bubble_coral_block,
fire_coral_block,horn_coral_block,dead_tube_coral_block,dead_brain_coral_block,
dead_bubble_coral_block,dead_fire_coral_block,dead_horn_coral_block}.json`;
`data/minecraft/loot_table/blocks/{tube_coral_block,brain_coral_block,bubble_coral_block,
fire_coral_block,horn_coral_block,dead_tube_coral_block,dead_brain_coral_block,
dead_bubble_coral_block,dead_fire_coral_block,dead_horn_coral_block}.json`;
`data/minecraft/tags/block/{coral_blocks,mineable/pickaxe,corals,coral_plants,wall_corals}.json`;
`data/minecraft/tags/item/sulfur_cube_archetype/fast_flat.json`;
`data/minecraft/sulfur_cube_archetype/fast_flat.json`;
`data/minecraft/villager_trade/wandering_trader/emerald_{tube,brain,bubble,fire,horn}_coral_block.json`;
`data/minecraft/{tags/villager_trade,trade_set}/wandering_trader/common.json`;
`data/minecraft/worldgen/configured_feature/warm_ocean_vegetation.json`;
`data/minecraft/worldgen/placed_feature/warm_ocean_vegetation.json`;
`data/minecraft/structure/**/*.nbt`;
`assets/minecraft/blockstates/{tube_coral_block,brain_coral_block,bubble_coral_block,
fire_coral_block,horn_coral_block,dead_tube_coral_block,dead_brain_coral_block,
dead_bubble_coral_block,dead_fire_coral_block,dead_horn_coral_block}.json`;
`assets/minecraft/models/block/{tube_coral_block,brain_coral_block,bubble_coral_block,
fire_coral_block,horn_coral_block,dead_tube_coral_block,dead_brain_coral_block,
dead_bubble_coral_block,dead_fire_coral_block,dead_horn_coral_block}.json`;
`assets/minecraft/items/{tube_coral_block,brain_coral_block,bubble_coral_block,
fire_coral_block,horn_coral_block,dead_tube_coral_block,dead_brain_coral_block,
dead_bubble_coral_block,dead_fire_coral_block,dead_horn_coral_block}.json`.

**Test vectors:**

Run `EXP-BLK-070` across every identity, every adjacent-water arrangement, placement/forced/update/
scheduled paths, queue deduplication and persistence, all correct-tool/Silk/explosion outcomes,
pickle support/spread, common trades, fast-flat equipment, fire/fuel/recipe absences, every coral
feature selection, all 1,212 templates, persistence, creative tab, sounds, maps and models. Assert
exact IDs, constants, read/draw/schedule/write order, negative memberships, zero template cells and
vanilla-client convergence.

**Limits:**

Generic placement, breaking, scheduler queues, bone-meal item dispatch, loot, villager offer
lifecycle, sulfur equipment/knockback, fire, fluid simulation, worldgen placement, packet encoding
and client rendering remain with `BLK-PLACE-001`, `PLY-BREAK-001`, `SIM-SCHEDULE-001`,
`ITM-USE-001`, `ITM-RECIPE-001`, `ITM-FURNACE-001`, `ITM-ADVANCEMENT-001`, `ITM-LOOT-001`,
`ENT-KNOCKBACK-001`, `ENV-FIRE-001`, `ENV-FLUID-001`, `WGEN-PIPELINE-001`,
`PROTO-PLAY-CLIENTBOUND-BLOCK-001`,
`PROTO-PLAY-CLIENTBOUND-SOUND-001` and `CLI-006`. Coral plants, fans, wall fans and sea-pickle
state behavior remain separate catalog families.
