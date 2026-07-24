# Blocks mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `BLK-STEM-CROP-001` — Melon and pumpkin stems mature, choose one fruit side and collapse when that fruit leaves

**Parent:** `SIM-004`, `SIM-005`, `BLK-001`, `BLK-002`, `BLK-003`, `BLK-005`, `RED-001`,
`PLY-005`, `PLY-006`, `ITM-003`, `ITM-004`, `ITM-006`, `ENT-001`, `MOB-001`, `MOB-004`,
`ENV-003`, `WGEN-002`, `WGEN-003`, `WGEN-004`, `CLI-001`, `CLI-006`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked registrations, `StemBlock`, `AttachedStemBlock`, the reused crop-speed
helper, support/consumer tags, recipes, advancements, loot, trades, processor/pool data, all 1,212
structure templates and exact client assets close the four block identities and two seed items.

**Applies when:**

`minecraft:melon_stem`, `minecraft:pumpkin_stem`, `minecraft:attached_melon_stem` or
`minecraft:attached_pumpkin_stem` is placed, updated, randomly ticked, bone-mealed, broken,
exploded, generated, persisted or rendered; or when melon/pumpkin seeds are crafted, looted,
traded, composted, fed, planted, cloned, persisted or rendered.

**Authoritative state:**

The six identities are:

| Identity | Registry ID | State IDs/default | Schema or item role |
|---|---:|---|---|
| attached pumpkin stem | block `362` | `8334..8337`; north `8334` | `facing=north,south,west,east` |
| attached melon stem | block `363` | `8338..8341`; north `8338` | `facing=north,south,west,east` |
| pumpkin stem | block `364` | `8342..8349`; age zero `8342` | `age=0..7` |
| melon stem | block `365` | `8350..8357`; age zero `8350` | `age=0..7` |
| pumpkin seeds | item `1137` | common stack of 64 | block item for pumpkin stem |
| melon seeds | item `1138` | common stack of 64 | block item for melon stem |

The four blocks have no registered block-item identity, block entity or fluid property. The two
custom-named `BlockItem` registrations target their corresponding stem while retaining
`item.minecraft.<fruit>_seeds`, their own generated item model and the ordinary default component
set.

Every block uses map color `PLANT`, no collision/occlusion, instant break, piston reaction
`DESTROY`, emission and light dampening zero, skylight propagation, no sturdy face or redstone
conduction, shade brightness 1, friction 0.6, speed/jump factors 1, no comparator output and AIR
pathfindability. Hardness/resistance are `0/0`. Only unattached stems random-tick.

An unattached stem's selection column is centered with diameter `2/16`, bottom zero and top
`(2+2*age)/16`; collision remains empty. An attached stem's north shape spans
`x=6..10,y=0..10,z=0..10` in sixteenths and is horizontally rotated to its facing; collision
remains empty.

Unattached stems use Hard Crop sounds at volume/pitch `1/1`: break/step/place/hit/fall are Wood
Break `1853`, Wood Step `1857`, Crop Plant `483`, Wood Hit `1855` and Wood Fall `1854`.
Attached stems use ordinary Wood sounds `1853/1857/1856/1855/1854`.

**Transition and ordering:**

#### Seed placement, support and farmland retention

Each seed runs the ordinary block-item placement transaction for default age zero. Placement and
later survival read only the block immediately below through the species-specific support tag.
Both `supports_melon_stem` and `supports_pumpkin_stem` close through `supports_stem_crops` and
`supports_crops` to exactly `farmland`. Light, moisture, air above, biome and horizontal neighbors
add no placement predicate.

Every stem shape update rechecks that support through `VegetationBlock`; failure immediately
returns ordinary air. All four blocks are direct `maintains_farmland` members. A dry
moisture-zero farmland random tick therefore retains farmland while any one is directly above,
and farmland's own survival predicate likewise accepts the tagged block despite its non-solid
shape. Tag reload does not proactively revisit either state; the next relevant update/tick reads
the new snapshot.

Attached-stem update ordering is special. If the supplied neighbor is not its fixed fruit and the
supplied direction equals `facing`, it resolves the corresponding stem holder and immediately
returns default stem with `age=7`. This precedes the inherited support check, so the fruit-loss
update can return a mature stem even when support is simultaneously invalid; a later shape update
then removes it. A change in any other direction, or a still-matching fruit, delegates to the
ordinary support check. Rotation and mirror transform facing normally.

#### Random growth and fruit transaction

An admitted unattached-stem random callback first reads raw brightness at the current position.
Values below `9` return without calculating speed or consuming RNG. At brightness at least `9`,
compute crop speed `f`:

1. start at `1`;
2. inspect the 3-by-3 plane centered immediately below the stem;
3. each direct `grows_crops` member contributes `1` when it has no positive `moisture`, or `3`
   when moisture is positive; divide each of the eight off-center contributions by `4`;
4. halve the final sum when the same stem block exists on both an east/west and north/south axis,
   or when it exists at any diagonal. The other fruit species does not count as the same block.

Locked `grows_crops` contains only farmland. The callback consumes
`nextInt((int)(25.0f/f)+1)` and proceeds only on zero. At age `0..6`, it offers age plus one at
the same position with flags `2`, ignores the Boolean result and stops.

At age `7`, a successful growth trial next consumes one uniform horizontal-direction choice. Let
`target=pos.relative(direction)`. It reads `target.below` before `target`, and proceeds only when
target is air and the below state belongs to the species fruit-support tag. Both fruit-support
tags close through `supports_stem_fruit` and `supports_vegetation` to exactly `dirt`,
`coarse_dirt`, `rooted_dirt`, `mud`, `muddy_mangrove_roots`, `moss_block`, `pale_moss_block`,
`grass_block`, `podzol`, `mycelium` and `farmland`.

It then resolves both fixed block holders. Missing fruit or attached-stem holder aborts with no
writes after the direction and reads. Otherwise it first offers the default melon or pumpkin at
target with `setBlockAndUpdate`, then offers the attached stem at the original position facing the
chosen direction with another `setBlockAndUpdate`; both Boolean results are ignored. The fruit
write therefore precedes attachment and neither write rolls the other back.

#### Bone meal

A stem is a valid target exactly while age is not `7`; success is unconditional and consumes no
separate success RNG. The callback consumes one inclusive integer `2..5`, sets
`newAge=min(7,oldAge+draw)` with flags `2`, and ignores the result. If `newAge` is `7`, it
immediately invokes the new state's random callback with the same RNG. That nested callback still
requires brightness at least `9`, recomputes `f`, and must pass its independent bounded-integer
growth trial before selecting a side. Thus bone meal can produce age seven without fruit.
Generic item shrink, interaction result and level event remain with the bone-meal owner.

#### Breaking, loot, recipes and advancement

Clone-item selection returns the matching seed for every age and attached facing. Each unattached
stem table makes one roll of a matching seed entry, applies the age-selected binomial count
with `n=3` and exact emitted probabilities
`[0.06666667,0.13333334,0.2,0.26666668,0.33333334,0.4,0.46666667,0.53333336]`,
then applies pool-level `explosion_decay`. Attached stems use `n=3,p=0.53333336`. Counts can be
zero; tool, Silk Touch and Fortune add no branch. The four random sequences are
`minecraft:blocks/<block-id>`.

Exactly two shapeless recipes produce seeds:

- one melon slice produces one melon seed;
- one pumpkin produces four pumpkin seeds.

Each recipe advancement has parent `recipes/root`, an inventory criterion for its input and a
matching `recipe_unlocked` criterion in one OR requirement group, and rewards the recipe. The
separate `husbandry/plant_seed` advancement has one OR group over seven placed-block criteria;
placing either age-zero stem satisfies its corresponding criterion, and the display sends
telemetry.

#### Other seed acquisition and consumers

Baseline nonblock loot contains these direct entries:

| Table/pool | Rolls | Seed entry | Pool total weight |
|---|---|---|---:|
| abandoned mineshaft pool 1 | uniform `2..4` | each seed weight 10, count `2..4` | 98 |
| simple dungeon pool 1 | uniform `1..4` | each seed weight 10, count `2..4` | 125 |
| woodland mansion pool 1 | uniform `1..4` | each seed weight 10, count `2..4` | 175 |
| village taiga house pool 0 | uniform `3..8` | pumpkin seed weight 5, count `1..5` | 54 |
| carve pumpkin pool 0 | `1` | four pumpkin seeds, weight 1 | 1 |

The first four use their like-named `minecraft:chests/...` random sequence; carving uses
`minecraft:carve/pumpkin`. The optional trade-rebalance pack is outside this baseline.

Both one-emerald wandering-trader records give one matching seed, permit 12 uses and set reputation
discount `0.05`. They are two of the 76 uniform candidates in the common set, which chooses five
distinct offers with random sequence `minecraft:trade_set/wandering_trader/common`.

Both items are direct `chicken_food` and `parrot_food` members. The former admits chicken
temptation/breeding; the latter admits the parrot's consuming one-in-ten tame attempt, while
parrots remain nonbreedable. Those generic transactions remain with `MOB-BREED-001`.

Both items are code-built composter inputs at chance `0.3f`. Player or automation insertion at
level zero succeeds without RNG; levels `1..6` compare `nextDouble()` strictly against the widened
float chance, with all item/stat/event/scheduling consequences owned by the composter transaction.
Neither item is a fuel. None of the four blocks has fire odds or lava ignition: odds are `0/0`.

#### World generation and structures

The four huge-fungus configurations explicitly admit all four stem identities in their
replaceable-block predicate. Village processors can instead output age-zero stems from template
wheat: desert and zombie-desert try beetroot at `0.2` before melon stem at `0.1`; savanna and
zombie-savanna have the melon `0.1` rule; taiga and zombie-taiga try pumpkin stem at `0.3` before
potato at `0.2`. Rule ordering, position-derived RNG and pool placement remain with
`WGEN-JIGSAW-PROCESSORS-001` and `WGEN-JIGSAW-VILLAGES-001`.

An exhaustive scan of all 1,212 templates finds:

- woodland-mansion `1x2_a8`: eight west- and eight east-facing attached stems of each species,
  32 attached cells total;
- six savanna street templates, ordinary and zombie copies of `crossroad_07`, `straight_06` and
  `straight_11`: 62 melon stems total, all age `0..2`;
- taiga `houses/taiga_small_farm_1`: 17 pumpkin stems, all age `7`.

There are no other raw cells. Template selection, transforms, processors, clipping and accepted
writes remain with the named structure owners; raw cells are not unconditional placements.

**Client projection:**

Unattached ages select eight `stem_growth0..7` crossed-plane models. Their visual height is
`2,4,...,16` pixels and all faces use tint index zero. `BlockTintSources.stem` computes
`RGB=(32*age,255-8*age,4*age)`. Attached stems use `stem_fruit`, the species stem and upper-stem
textures, fixed tint `#E0C71C`, and model Y rotations west/east/north/south
`0/180/90/270` degrees.

Each seed item directly selects its own generated flat item model and has no item tint, special
renderer or conditional model branch. Natural Blocks orders wheat seeds, cocoa beans, pumpkin
seeds, melon seeds, beetroot seeds; neither item appears in Building Blocks or Ingredients.

**Branches and aborts:**

Unsupported placement; support-loss update; brightness below 9; failed growth draw; ordinary age
increment; mature target nonair; fruit-support miss; missing holder; independently failed fruit
or attachment write; attached fruit retained/lost; invalid mature bone-meal target; bone-meal
increase below seven; nested brightness/growth/target failures; zero loot count; explosion decay;
recipe/unlock alternative; loot/trade/compost/animal gates; processor/template write rejection.

**Constants and randomness:**

Maximum age `7`; brightness gate `9`; base growth numerator `25`; crop-speed base/contributions
`1`, dry `1`, moist `3`, off-center divisor `4`, crowding divisor `2`; one bounded growth draw and,
only for a mature success, one four-way direction draw. Bone meal uses one inclusive `2..5` draw
and may then consume the nested growth/direction draws. Loot uses `n=3` and
the eight exact probabilities listed above; attached `p=0.53333336`. Compost chance `0.3`; trade
candidates/offers `76/5`; structure and loot constants are the tables above.

**Side effects:**

Block/item consumption and placement effects; immediate support removal; farmland retention;
random and bone-meal state writes; fruit then attachment writes; neighbor updates; loot/item
entities; recipe/advancement/stat changes; trade offers; composter mutation/events; chicken/parrot
state; worldgen/template writes; client block, tint, sound and item projection.

**Gates:**

Generic placement/reach/build permissions; active support, fruit-support, food, loot, recipe,
advancement, trade and worldgen snapshots; random-tick admission; brightness; exact growth and
direction draws; target air; both holder lookups; bone-meal age; loot context/explosion; composter,
animal, structure and client gates.

**State read/written:**

Reads age/facing, current/below/3-by-3/side/side-below states, farmland moisture, brightness,
registry holders, RNG, loot context, item stacks, advancement/trade/animal state, worldgen input
and client assets. Writes stem age/identity/facing, fruit state, farmland continuity, inventories,
loot, advancement/trade/animal/composter state, generated cells and client-visible effects.

**Persistence boundary:**

Chunk palettes persist only the current stem identity and age/facing; no fruit link, pending growth
trial, crop-speed scan, bone-meal increment or RNG cursor is stored. A committed attached stem and
fruit persist as independent cells and reconcile through later neighbor updates. Items retain
their stack components ordinarily. Reload can replace tags, loot, recipes, advancements, trades
and worldgen data without rewriting existing palettes.

**Boundary cases and quirks:**

Survival needs farmland but fruit may grow over ten additional substrates. Moisture changes speed,
not survival. Mixed melon/pumpkin neighbors do not trigger same-block crowding. The brightness read
precedes all RNG. Attached fruit loss precedes support validation. Fruit placement precedes
attachment with no rollback. Bone meal reaching seven is not guaranteed to fruit. Stem drops use
binomial count plus explosion decay rather than `survives_explosion`. The seed is a block item even
though no block registry entry shares its item ID.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-DATA-001`; `OFF-REPORT-001`;
`net.minecraft.world.level.block.Blocks`;
`net.minecraft.world.item.Items#createBlockItemWithCustomItemName(net.minecraft.world.level.block.Block)`;
`net.minecraft.world.level.block.StemBlock#randomTick(net.minecraft.world.level.block.state.BlockState,net.minecraft.server.level.ServerLevel,net.minecraft.core.BlockPos,net.minecraft.util.RandomSource)`;
`net.minecraft.world.level.block.StemBlock#performBonemeal(net.minecraft.server.level.ServerLevel,net.minecraft.util.RandomSource,net.minecraft.core.BlockPos,net.minecraft.world.level.block.state.BlockState)`;
`net.minecraft.world.level.block.AttachedStemBlock#updateShape(net.minecraft.world.level.block.state.BlockState,net.minecraft.world.level.LevelReader,net.minecraft.world.level.ScheduledTickAccess,net.minecraft.core.BlockPos,net.minecraft.core.Direction,net.minecraft.core.BlockPos,net.minecraft.world.level.block.state.BlockState,net.minecraft.util.RandomSource)`;
`net.minecraft.world.level.block.CropBlock#getGrowthSpeed(net.minecraft.world.level.block.Block,net.minecraft.world.level.BlockGetter,net.minecraft.core.BlockPos)`;
`net.minecraft.world.level.block.VegetationBlock#updateShape(net.minecraft.world.level.block.state.BlockState,net.minecraft.world.level.LevelReader,net.minecraft.world.level.ScheduledTickAccess,net.minecraft.core.BlockPos,net.minecraft.core.Direction,net.minecraft.core.BlockPos,net.minecraft.world.level.block.state.BlockState,net.minecraft.util.RandomSource)`;
`net.minecraft.world.level.block.FarmlandBlock#randomTick(net.minecraft.world.level.block.state.BlockState,net.minecraft.server.level.ServerLevel,net.minecraft.core.BlockPos,net.minecraft.util.RandomSource)`;
`net.minecraft.world.level.block.ComposterBlock#bootStrap`;
`net.minecraft.client.color.block.BlockColors#createDefault`;
`net.minecraft.client.color.block.BlockTintSources#stem`;
`net.minecraft.world.item.CreativeModeTabs#bootstrap(net.minecraft.core.Registry)`;
`data/minecraft/tags/block/{supports_melon_stem,supports_pumpkin_stem,
supports_melon_stem_fruit,supports_pumpkin_stem_fruit,supports_stem_crops,
supports_stem_fruit,supports_crops,supports_vegetation,substrate_overworld,grows_crops,
maintains_farmland,crops}.json`;
`data/minecraft/tags/item/{chicken_food,parrot_food}.json`;
`data/minecraft/loot_table/blocks/{melon_stem,pumpkin_stem,attached_melon_stem,
attached_pumpkin_stem}.json`;
`data/minecraft/loot_table/{chests/abandoned_mineshaft,chests/simple_dungeon,
chests/woodland_mansion,chests/village/village_taiga_house,carve/pumpkin}.json`;
`data/minecraft/recipe/{melon_seeds,pumpkin_seeds}.json`;
`data/minecraft/advancement/{recipes/misc/{melon_seeds,pumpkin_seeds},
husbandry/plant_seed}.json`;
`data/minecraft/{trade_set/wandering_trader/common,tags/villager_trade/wandering_trader/common,
villager_trade/wandering_trader/{emerald_melon_seeds,emerald_pumpkin_seeds}}.json`;
`data/minecraft/worldgen/{configured_feature/{crimson_fungus,crimson_fungus_planted,
warped_fungus,warped_fungus_planted},processor_list/{farm_desert,farm_savanna,farm_taiga,
zombie_desert,zombie_savanna,zombie_taiga},template_pool/village/**/*.json}.json`;
`data/minecraft/structure/**/*.nbt`;
`assets/minecraft/blockstates/{melon_stem,pumpkin_stem,attached_melon_stem,
attached_pumpkin_stem}.json`;
`assets/minecraft/models/block/{melon_stem_stage0,melon_stem_stage1,melon_stem_stage2,
melon_stem_stage3,melon_stem_stage4,melon_stem_stage5,melon_stem_stage6,melon_stem_stage7,
pumpkin_stem_stage0,pumpkin_stem_stage1,pumpkin_stem_stage2,pumpkin_stem_stage3,
pumpkin_stem_stage4,pumpkin_stem_stage5,pumpkin_stem_stage6,pumpkin_stem_stage7,
attached_melon_stem,attached_pumpkin_stem,stem_growth0,stem_growth1,stem_growth2,
stem_growth3,stem_growth4,stem_growth5,stem_growth6,stem_growth7,stem_fruit}.json`;
`assets/minecraft/{items,models/item}/{melon_seeds,pumpkin_seeds}.json`;
`BLK-STATE-001`; `BLK-PLACE-001`; `BLK-BREAK-001`; `BLK-BREAK-HOOK-001`;
`SIM-RANDOM-001`; `ITM-RECIPE-001`; `ITM-ADVANCEMENT-001`; `ITM-LOOT-001`;
`MOB-BREED-001`; `WGEN-PIPELINE-001`; `WGEN-JIGSAW-PROCESSORS-001`;
`WGEN-JIGSAW-VILLAGES-001`; `WGEN-STRUCTURE-WOODLAND-MANSION-001`; `EXP-BLK-077`.

**Test vectors:**

Cross all 24 states and two seed stacks through placement, support-tag reload, every update
direction/fruit/support combination, farmland moisture/retention, brightness 8/9, every crop-speed
contribution and crowding arrangement, growth draw bounds, all four directions, target/support/
holder/write outcomes, age/bone-meal endpoints and exact RNG cursors. Assert clone and all
age/facing/explosion loot distributions, both recipes/unlocks and plant advancement, every
chest/carve/trade/compost/chicken/parrot route, all processor/template/fungus inputs, save/reload,
sounds, shapes, tints, models and creative ordering.

**Limits:**

Generic random-tick scheduling, block-item admission/commit, neighbor propagation, block breaking,
loot evaluation, bone-meal item effects, farmland hydration/trampling, composter execution,
crafting, advancements, trade-set selection, animal goals/taming, huge-fungus geometry, structure
selection/placement, persistence, protocol and rendering remain with their cited owners. This leaf
owns the six identities' selectors, constants, local transitions, data joins and projection.
