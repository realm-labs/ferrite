# Blocks mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `BLK-CORAL-PLANT-001` — Coral plants and fans join waterlogging, support and delayed drying

**Parent:** `SIM-003`, `BLK-001`, `BLK-002`, `BLK-003`, `BLK-005`, `PLY-005`, `PLY-006`,
`ITM-003`, `ITM-004`, `ITM-006`, `ENV-001`, `ENV-002`, `ENV-003`, `WGEN-002`, `WGEN-003`,
`CLI-001`, `CLI-006`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked registrations/reports, all seven implementation classes, complete
loot/tag and bone-meal/coral-feature consumers, exhaustive recipe/acquisition/client-reference
sweeps, all 1,212 decoded structure templates and exact client assets close five colors across
live/dead upright plants, floor fans and wall fans. The 30 blocks share waterlogging and support;
live forms additionally schedule drying, while wall forms add horizontal facing and share the
corresponding floor-fan item/loot table.

**Applies when:**

Any live/dead tube, brain, bubble, fire or horn identity ending in `_coral`, `_coral_fan` or
`_coral_wall_fan` is placed, waterlogged, updated, scheduled, broken, piston-moved, selected by
underwater bone meal or coral decoration, persisted, mapped or rendered.

**Authoritative state:**

All 30 blocks have no block entity. Upright plants and floor fans each have
`waterlogged=true/false`; wall fans have `facing=north/south/west/east` outside
`waterlogged=true/false`. Defaults are waterlogged, and wall defaults face north.

| Shape family | Dead state ranges | Live state ranges | Dead/live block IDs |
|---|---|---|---|
| upright plant | `15147..15156` | `15157..15166` | `758..762` / `763..767` |
| floor fan | `15167..15176` | `15177..15186` | `768..772` / `773..777` |
| wall fan | `15187..15226` | `15227..15266` | `778..782` / `783..787` |

Within each range the color order is tube, brain, bubble, fire, horn; each nonwall identity uses
waterlogged true then false, and each wall identity uses facing north, south, west, east with true
then false. The 20 item IDs are:

| Item family | Tube | Brain | Bubble | Fire | Horn |
|---|---:|---:|---:|---:|---:|
| live plant | `687` | `688` | `689` | `690` | `691` |
| dead plant | `696` | `692` | `693` | `694` | `695` |
| live floor fan | `697` | `698` | `699` | `700` | `701` |
| dead floor fan | `702` | `703` | `704` | `705` | `706` |

Wall fans have no item ID. Each floor-fan item is a standing-and-wall block item whose standing
block is that floor fan and whose wall block is the matching wall fan; upright plants are ordinary
block items. All 20 are common, stack to 64 and have standard block-item components.

Every identity is instant-break, no-collision and emission zero. Upright selection shapes are a
centered 12-by-15-by-12 column; floor fans use a centered 12-by-4-by-12 column. A north-facing wall
fan selects X `0..16`, Y `4..12`, Z `5..16`; the other facings rotate that shape horizontally.
Collision is empty, so none has a sturdy face or ordinary spawn-floor support. All propagate
skylight when dry; a waterlogged state exposes source water and follows the fluid/light owner's
water boundary.

Dead registrations use gray map color, `BASEDRUM`, default Stone sounds, correct-tool-required
loot, `forceSolidOn`, strength/resistance zero and normal piston reaction. Live registrations use
blue/pink/purple/red/yellow map colors, default `HARP`, Wet Grass sounds, no correct-tool
requirement, strength/resistance zero and `DESTROY` piston reaction. Wet Grass volume/pitch is
`1/1`, with break/step/place/hit/fall sound IDs `1761/1765/1764/1763/1762`; dead Stone uses
`1596/1604/1601/1600/1599`. None random-ticks, emits a signal, has a comparator value, ignites from
lava, enters ordinary-fire odds or provides vanilla furnace fuel.

**Transition and ordering:**

#### Placement, support, water and drying

Base placement reads the center fluid and sets waterlogged true only when it belongs to `water`
and is full; source water qualifies and flowing nonfull water does not. Upright plants and floor
fans require the block below to have a sturdy upper face. A wall-fan item considers the context's
nearest-looking directions in order, skips vertical directions, sets facing opposite each
horizontal candidate and returns the first state whose backing block exposes the required sturdy
face. If none survives, placement returns null. Rotation and mirror transform facing normally and
never change waterlogged.

Every live `onPlace`, after the generic state write, calls the shared drying helper. Its water scan
first returns true when the state's own waterlogged property is true. Otherwise it scans adjacent
fluid states in Down, Up, North, South, West, East order and returns on the first `water` member.
A wet scan consumes no RNG. A dry scan consumes `nextInt(40)` and requests a normal-priority self
tick after `60 + draw`.

On any update, an upright plant/floor fan first returns AIR when the changed direction is Down and
the floor support is no longer sturdy. A wall fan first returns AIR when the changed direction is
opposite its facing and its backing face no longer supports it. These early removals perform no dry
scan, no delay draw and no explicit water-tick request.

If support remains, a live form runs the same dry scan and scheduling helper. A waterlogged live
form then requests the ordinary water delay once in the live override and again in its base
implementation; scheduler deduplication retains one equivalent fluid tick. A dead upright/floor
form uses only the base implementation and requests one water tick; a dead wall form likewise
requests one before its support test. Dry states request no fluid tick. Generic update propagation
owns AIR replacement, drops and neighbor effects.

At due time the scheduler first validates the current live block type. The callback rescans water;
wet is a no-op. Dry upright plants and floor fans offer the matching dead default with
`waterlogged=false` and flags 2. Dry wall fans also copy the live facing before the flags-2 offer.
The Boolean result is ignored, with no sound, particle, game event, loot or retry. Dead forms never
schedule drying or revive. Pending tick persistence, deduplication, activity and type validation
remain with `SIM-SCHEDULE-001`.

Simple-waterlogging bucket admission/removal remains generic. Because the center property is
checked before neighbors, adding source water to the state prevents drying without inspecting
adjacent fluids; removing it can remain wet through any adjacent flowing or waterlogged-neighbor
fluid. A due callback always rechecks current truth.

#### Loot and block-item dispatch

All 20 floor/upright loot tables contain one roll gated only by Silk Touch level at least one and
emit the matching item; none has `survives_explosion`. Live blocks have no correct-tool gate, so any
Silk Touch tool can obtain them. The 15 dead upright/floor/wall blocks are direct
`mineable/pickaxe` members and require a correct tool, so dead loot additionally needs any pickaxe.
Fortune, waterlogged and facing have no other effect.

Each wall fan overrides its loot table with the corresponding floor fan's table. A Silk Touch break
therefore returns the floor-fan item and uses the floor table's
`minecraft:blocks/<color>_coral_fan` random sequence; wall identity never has a distinct loot file
or item. Without Silk Touch, support loss, and ordinary explosions produce no item.

The standing-and-wall item first follows generic placement direction ordering to select the
admitted floor or wall state. Its one count, statistic, game event and component transfer commit
only through the generic block-item transaction; the wall block's absent registry item cannot be
obtained as a distinct stack.

#### Reloadable tag consumers and negative joins

The exact live-only tag order is:

- `coral_plants`: tube, brain, bubble, fire, horn upright plants;
- `corals`: nested `coral_plants`, then tube, brain, bubble, fire, horn floor fans;
- `wall_corals`: tube, brain, bubble, fire, horn wall fans;
- `underwater_bonemeals`: seagrass, nested `corals`, then nested `wall_corals`.

No dead form enters those tags. Production code reads these identities only in `CoralFeature` and
`BoneMealItem`; data providers do not add runtime consumers.

For an admitted coral feature cell, the already-owned top branch below 0.25 uniformly samples the
ten-member flattened `corals` order and offers its waterlogged live default above with flags 2.
Each admitted exact-water horizontal neighbor whose fixed-order float is below 0.2 uniformly
samples `wall_corals`, sets facing to the visited direction and offers flags 2. The feature owner
retains all branch draws, strict-water gates, traversal and ignored write results.

Underwater bone meal initially requires exact water with a full fluid at the target; the clicked
support face is validated by the item-use owner. On the server it runs 128 indexed attempts. In
the sole warm-ocean (`produces_corals_from_bonemeal`) biome, attempt zero with a horizontal clicked
face uniformly samples `wall_corals` and sets facing to that clicked direction. Other warm-ocean
attempts draw `nextInt(4)`; zero uniformly samples the 16-member `underwater_bonemeals` closure,
otherwise retaining seagrass. A selected wall fan tests its default north support, then makes at
most four random-horizontal facing substitutions until one survives; repeated directions are
allowed. Only a surviving state replaces exact full water with flags 3. The complete 128-attempt
random walk, seagrass-growth branch, stack shrink and level event remain with `ITM-USE-001`.

No family identity occurs exactly in any bundled recipe, advancement, trade or nonblock loot
record. No item is in a sulfur-cube archetype. Fire odds and fuel time are zero.

#### Generation and structure absence

Live plants/fans are generated only through the `corals`/`wall_corals` decoration branches of the
three warm-ocean coral feature types; dead forms are never selected. Underwater bone meal is a
player-triggered mutation, not terrain generation. All 30 identities have zero cells in all 1,212
bundled structure templates. `WGEN-PIPELINE-001` retains exact coral tree/claw/mushroom selection,
geometry, RNG and write order.

**Client projection:**

Every upright plant selects its like-named untinted `cross` model for both waterlogged states.
Every floor fan selects its like-named untinted, ambient-occlusion-off four-plane `coral_fan`
model. Wall states ignore waterlogged in model selection and use the like-named two-plane
`coral_wall_fan` model with north/east/south/west Y rotations `0/90/180/270`; its texture is the
corresponding floor-fan texture. All geometry is unshaded.

The 20 item selectors use flat generated models with the matching upright or floor-fan block
texture. The Natural Blocks tab orders the five live upright plants, five dead upright plants,
five live floor fans and five dead floor fans immediately after the completed live/dead coral
blocks; wall variants add no entry. Authoritative state, fluid, break, support-loss and drying
updates use ordinary block/fluid publication, while sounds and bone-meal effects retain their
protocol owners.

**Branches and aborts:**

Thirty blocks, 120 states and 20 items; live/dead; upright/floor/wall; five colors; every
waterlogged/facing state; source/flowing/no center water; every adjacent water position; valid/
invalid support and changed direction; floor/wall placement direction; rotation/mirror; on-place,
update, due, deduplicated/reloaded/type-mismatch/failed tick; bucket admission; correct/incorrect
tool, Silk Touch/Fortune/explosion/support loss; empty/singleton/full tags; coral top/wall
decoration; all 128 bone-meal attempts and facing retries; zero structures; persistence, sounds,
maps, tabs and every block/item model are distinct branches.

**Constants and randomness:**

States `15147..15266`; block IDs `758..787`; item IDs `687..706` with the table ordering above;
strength/resistance `0/0`; stack `64`; plant/floor shapes `12×15×12` / `12×4×12`; wall north box
`0..16 × 4..12 × 5..16`; emission `0`; dry delay `60+nextInt(40)`; drying flags `2`; water update
requests live/base `2`, dead/base `1`; Wet Grass and Stone sound IDs above; bone-meal attempts
`128`, later coral selection `nextInt(4)==0`, wall retry limit `4`, placement flags `3`; flattened
tag sizes `5/10/5/16`; feature top/wall thresholds `0.25/0.2`; templates/cells `1212/0`.

**Side effects:**

Standing/wall placement; support-loss AIR mutation; water fluid scheduling; delayed silent drying;
Silk-only loot; block-item count/stat/game-event commit; reload-selected feature and bone-meal
mutation; ordinary palette/fluid persistence; sounds, maps, creative inventory and client models.

**Gates:**

Generic placement/write/break authority; source-water placement and simple-waterlogging admission;
floor/backing sturdy face; scheduler queue/activity/type checks; current center/adjacent water;
correct dead pickaxe and Silk Touch; active loot/tag/biome snapshot; coral-feature and bone-meal
water/support/write gates; valid registry, resource-pack and client connection context.

**Boundary cases and quirks:**

A dry forced write that changes block type reaches `onPlace` and can schedule drying, unlike the
full coral block's placement-state prewrite hook. Support loss wins before the live dry scan.
Waterlogged live updates offer the same fluid tick twice through override plus base; scheduler
deduplication hides the duplicate. Drying always clears waterlogged, and wall drying preserves
facing. Dead forms are correct-pickaxe-gated even though they break instantly, while live forms
need Silk Touch but no correct tool. Wall loot and items deliberately collapse to the floor fan.
Model selection ignores waterlogged for every form.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-REPORT-001`; `OFF-DATA-001`;
`net.minecraft.world.level.block.Blocks`;
`net.minecraft.world.level.block.BaseCoralPlantTypeBlock#tryScheduleDieTick`;
`net.minecraft.world.level.block.BaseCoralPlantTypeBlock#scanForWater`;
`net.minecraft.world.level.block.BaseCoralPlantTypeBlock#getStateForPlacement`;
`net.minecraft.world.level.block.BaseCoralPlantTypeBlock#updateShape`;
`net.minecraft.world.level.block.BaseCoralPlantTypeBlock#canSurvive`;
`net.minecraft.world.level.block.BaseCoralPlantTypeBlock#getFluidState`;
`net.minecraft.world.level.block.BaseCoralPlantBlock#getShape`;
`net.minecraft.world.level.block.CoralPlantBlock#onPlace`;
`net.minecraft.world.level.block.CoralPlantBlock#tick`;
`net.minecraft.world.level.block.CoralPlantBlock#updateShape`;
`net.minecraft.world.level.block.CoralPlantBlock#getShape`;
`net.minecraft.world.level.block.BaseCoralFanBlock#getShape`;
`net.minecraft.world.level.block.CoralFanBlock#onPlace`;
`net.minecraft.world.level.block.CoralFanBlock#tick`;
`net.minecraft.world.level.block.CoralFanBlock#updateShape`;
`net.minecraft.world.level.block.BaseCoralWallFanBlock#getStateForPlacement`;
`net.minecraft.world.level.block.BaseCoralWallFanBlock#updateShape`;
`net.minecraft.world.level.block.BaseCoralWallFanBlock#canSurvive`;
`net.minecraft.world.level.block.BaseCoralWallFanBlock#rotate`;
`net.minecraft.world.level.block.BaseCoralWallFanBlock#mirror`;
`net.minecraft.world.level.block.BaseCoralWallFanBlock#getShape`;
`net.minecraft.world.level.block.CoralWallFanBlock#onPlace`;
`net.minecraft.world.level.block.CoralWallFanBlock#tick`;
`net.minecraft.world.level.block.CoralWallFanBlock#updateShape`;
`net.minecraft.world.item.BoneMealItem#growWaterPlant`;
`net.minecraft.world.item.StandingAndWallBlockItem#getPlacementState`;
`net.minecraft.world.level.levelgen.feature.CoralFeature#placeCoralBlock`;
`net.minecraft.world.item.CreativeModeTabs#bootstrap`;
`net.minecraft.world.level.block.FireBlock#bootStrap`;
`net.minecraft.world.level.block.entity.FuelValues#vanillaBurnTimes`;
`reports/blocks.json#minecraft:{live/dead five-color coral,coral_fan,coral_wall_fan}`;
`reports/registries.json#minecraft:{block,item,sound_event}`;
`data/minecraft/loot_table/blocks/{live/dead five-color coral,coral_fan}.json`;
`data/minecraft/tags/block/{coral_plants,corals,wall_corals,underwater_bonemeals,mineable/pickaxe}.json`;
`data/minecraft/tags/worldgen/biome/produces_corals_from_bonemeal.json`;
`data/minecraft/{recipe,advancement,villager_trade,loot_table}/**`;
`data/minecraft/structure/**/*.nbt`;
`assets/minecraft/blockstates/{live/dead five-color coral,coral_fan,coral_wall_fan}.json`;
`assets/minecraft/models/block/{live/dead five-color coral,coral_fan,coral_wall_fan}.json`;
`assets/minecraft/items/{live/dead five-color coral,coral_fan}.json`;
`assets/minecraft/models/item/{live/dead five-color coral,coral_fan}.json`.

**Test vectors:**

Run `EXP-BLK-071` across every identity/state, placement/support/water/update/drying branch,
standing-and-wall item, loot outcome, tag reload, coral-feature decoration, underwater bone-meal
attempt, all 1,212 templates, persistence, sounds, maps, tabs and models. Assert exact IDs,
properties, read/draw/schedule/write order, tag order, negative joins and vanilla-client
convergence.

**Limits:**

Generic placement, breaking, update propagation, scheduler/fluid queues, bone-meal item use, loot,
coral-feature traversal, packet encoding and client rendering remain with `BLK-PLACE-001`,
`PLY-BREAK-001`, `BLK-UPDATE-001`, `SIM-SCHEDULE-001`, `ENV-FLUID-001`, `ITM-USE-001`,
`ITM-LOOT-001`, `WGEN-PIPELINE-001`, `PROTO-PLAY-CLIENTBOUND-BLOCK-001` and `CLI-006`.
Full coral blocks, sea pickles, seagrass and unrelated underwater-bonemeal members remain separate
catalog families.
