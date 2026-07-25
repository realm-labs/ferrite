# Block mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `BLK-COCOA-001` — Cocoa Beans place a three-age jungle-log crop that joins growth, composting, recipes and natural jungle trees

**Parent:** `SIM-004`, `SIM-005`, `SIM-RANDOM-001`, `BLK-001`,
`BLK-STATE-001`, `BLK-002`, `BLK-PLACE-001`, `BLK-BREAK-001`,
`BLK-BREAK-HOOK-001`, `BLK-BREAK-CONTENT-001`, `PLY-002`, `PLY-005`,
`PLY-006`, `PLY-INPUT-001`, `PLY-INTERACT-001`, `PLY-BREAK-001`,
`PLY-COLLISION-001`, `PLY-AUTOJUMP-001`, `BLK-003`, `BLK-004`, `BLK-005`,
`BLK-007`, `BLK-UPDATE-001`, `RED-001`, `RED-UPDATE-001`,
`RED-COMPARATOR-001`, `ITM-001`, `ITM-002`, `ITM-003`, `ITM-004`,
`ITM-005`, `ITM-006`, `ITM-007`, `ITM-USE-001`, `ITM-CONTAINER-001`,
`ITM-RECIPE-001`, `ITM-CRAFT-001`, `ITM-LOOT-001`,
`ITM-ADVANCEMENT-001`, `ITM-ENCHANT-001`, `ITM-ANVIL-001`,
`ITM-COOKIE-001`, `ENT-001`, `MOB-001`, `MOB-004`, `MOB-AI-001`,
`ENV-001`, `ENV-002`, `ENV-003`, `ENV-FLUID-001`, `ENV-FIRE-001`,
`ENV-LIGHT-001`, `BLK-SAPLING-001`, `WGEN-PIPELINE-001`, `CLI-001`,
`CLI-006`, `CLI-UI-001`, `CLI-EFFECT-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked registrations, `CocoaBlock`, the special block/item-key mapping,
Composter and path-type consumers, two recipes and advancements, one loot table, direct tags,
natural-tree records and exact client resources determine every Cocoa/Cocoa-Beans-specific
branch. Generic block-item placement, random-tick selection, block updates, breaking, loot,
crafting, composting, pathfinding, tree placement and rendering remain with the cited owners.

**Applies when:**

`minecraft:cocoa_beans` is placed, composted, crafted, moved, persisted, synchronized or rendered;
or a `minecraft:cocoa` state is placed, updated, randomly ticked, bone-mealed, pushed, queried for
shape/path type, broken, exploded, generated, persisted or rendered before and after block-tag,
recipe, advancement, loot, worldgen or resource reload.

**Authoritative state:**

Cocoa is block protocol ID `396` with twelve states:

| Age | Facing order | State IDs |
|---:|---|---|
| `0` | north, south, west, east | `9481..9484` |
| `1` | north, south, west, east | `9485..9488` |
| `2` | north, south, west, east | `9489..9492` |

Default is state `9481`, `age=0,facing=north`. Age closes at `0..2`; facing is horizontal and
points from the Cocoa cell toward its support. Cocoa Beans is the block's deliberately different
item key and raw item ID `1094`. `BlockItemIds.COCOA_CROP` maps block `cocoa` to item
`cocoa_beans`; `Items.createBlockItemWithCustomItemName` constructs an ordinary `BlockItem` with
the item-description prefix. There is no `minecraft:cocoa` item.

Cocoa Beans is a common nondamageable stack of `64` with the common empty modifiers,
enchantments and lore, item-break sound, translated item name, direct model key, repair cost,
swing animation, tooltip display and use effects. It has no food, consumable, remainder,
durability, equipment, tool, projectile, cooldown or inventory-tick behavior. Arbitrary valid
component patches retain the identity; generic block-state component application remains
`ITM-003` and block-item owned.

The block uses `CocoaBlock`, map color `PLANT`, random ticks, hardness/resistance `0.2/3.0`,
ordinary Wood sounds, no occlusion and piston reaction `DESTROY`. It does not require a correct
tool. Emission, redstone signal/conduction, comparator output, fluid state and block entity are
absent. It has no `FireBlock` flammability/encouragement entry.

For the state facing south, selection and collision pod boxes in sixteenths are:

| Age | Pod box |
|---:|---|
| `0` | `x=6..10,y=7..12,z=11..15` |
| `1` | `x=5..11,y=5..12,z=9..15` |
| `2` | `x=4..12,y=3..12,z=7..15` |

The box rotates with facing: south is unrotated, west/east use `90/270` degrees and north uses
`180`. The visual model adds a zero-thickness stem from the pod to the support, but the
authoritative selection/collision shape is the pod box. `CocoaBlock.isPathfindable` returns false
for every path-computation type.

**Transition and ordering:**

### Cocoa-Beans placement and support

Ordinary block-item use asks `CocoaBlock.getStateForPlacement` for a state at the chosen cell.
Starting from default age zero, it traverses the placement context's nearest-looking directions
in order, skips vertical directions, sets `facing` to each horizontal candidate and returns the
first state whose block at `position.relative(facing)` belongs to live
`#minecraft:supports_cocoa`. The locked tag expands `#minecraft:jungle_logs` to exactly Jungle
Log, Jungle Wood, Stripped Jungle Log and Stripped Jungle Wood. If none survives, placement
returns null and the generic transaction fails without consuming the stack.

An admitted placement applies generic state-component, collision, replaceability, permission and
write checks, consumes one for finite-material players, awards placement work and plays the block
sound through `BLK-PLACE-001`. The custom item name does not alter placement. A horizontal
support-neighbor update is special only when the supplied direction equals the state's facing:
if the live support predicate now fails, `updateShape` immediately returns ordinary Air;
otherwise it delegates to inherited horizontal-state update behavior. Changes on the other five
directions do not run the support test in this override.

Tag reload changes future placement and survival tests. It does not proactively revisit existing
states; a later relevant placement or neighbor update reads the new snapshot.

### Random growth and bone meal

Only ages below two report randomly ticking. Each admitted server random callback ignores its
supplied cursor, reads `ServerLevel.getRandom()` and consumes `nextInt(5)`. Zero advances age by
exactly one with `setBlock(...,2)`; `1..4` preserve the state. The callback rechecks
`age<2` before writing. It reads no light, biome, moisture, nearby crop, support or gamerule beyond
the upstream random-tick scheduler.

Age-zero and age-one states are valid bone-meal targets. `isBonemealSuccess` always returns true,
and `performBonemeal` consumes no Cocoa-specific RNG and writes age plus one with flags `2`.
Age two is rejected before performance. Generic Bone-Meal item admission, consumption, particles,
events and client prediction remain with the item and interaction owners.

### Tool, piston and path joins

Cocoa is directly in `mineable/axe` and `sword_efficient`. Those live tags alter the corresponding
generic tool speeds but do not gate drops; hand, wrong-tool, Axe and Sword breaks all reach the
loot table when the break owner admits them. Fortune and Silk Touch have no Cocoa-specific branch.
Piston reaction `DESTROY` selects the ordinary destructive update rather than movement.

`WalkNodeEvaluator.getPathTypeFromState` recognizes the exact block before later fluid/burning
tests and returns the dedicated `COCOA` path type, whose default malus is `0`.
`FlyNodeEvaluator` preserves `COCOA` when its sampled volume encounters that result. Bee and
Parrot constructors each override their own `COCOA` malus to `-1`, making Cocoa forbidden to
their flight searches; other mobs retain their configured/default value. This identity has no
damage callback. Collision, entity dimensions, volume traversal and path search remain
`MOB-AI-001`.

### Block loot and self-renewal

`blocks/cocoa` has one unconditional Cocoa-Beans entry under named sequence
`minecraft:blocks/cocoa`. It begins at count one; a block-state condition replaces count with
three only for `age=2`, then explosion decay independently retains each unit. Thus an admitted
nonexplosive age-zero/one break emits one default Cocoa Beans, an age-two break emits three, and
an explosion produces binomial survival over count one or three at per-unit probability
`1/explosion_radius`. Tool, facing, Fortune and Silk do not change the count. Results copy no
block or tool components.

Those drops are the sole bundled item-producing Cocoa record. No chest, entity, fishing,
archaeology, gift, trade or structure-template record contains Cocoa Beans, and all `1,212`
locked structure NBT files contain zero raw Cocoa cells and no `cocoa_beans` string. Creative or
command-provided Beans can seed player-grown Cocoa; natural generation supplies the survival
source below.

### Crafting and progression

Two exact records consume Cocoa Beans:

- shapeless `brown_dye` consumes one and emits one default Brown Dye in either grid size;
- shaped `cookie` is the width-three row Wheat, Cocoa Beans, Wheat and emits eight default
  Cookies. It cannot fit the `2×2` grid; in `3×3` it may occupy any one of the three rows.

Extra or missing inputs fail. Neither result copies arbitrary input patches or leaves a remainder.
Brown-Dye and Cookie behavior after assembly remains with their identity owners.

Each matching recipe advancement has two criteria in one OR group: exact Cocoa-Beans possession
or knowledge of that same recipe. Possession can therefore grant both recipes independently;
neither recipe grants the other. Listener registration, persistence and criterion effects remain
`ITM-ADVANCEMENT-001`.

### Composter insertion

`ComposterBlock` directly registers Cocoa Beans at Java float chance `0.65f`. Player-held
insertion at level zero succeeds without RNG. Levels `1..6` consume one `nextDouble()` and
increment only when the draw is strictly below the widened float value
`0.6499999761581421`. Success writes level plus one with flags `3`, emits `BLOCK_CHANGE`, and
`6 -> 7` schedules maturation after `20` ticks; failure preserves state.

Either level-`0..6` result emits level event `1500` with success encoded by state change, awards
the Cocoa-Beans-used statistic and calls `consume(1,player)`, preserving infinite-material
holders. Level `7` succeeds without insertion, event, statistic or consumption. Level `8`
delegates to the remaining interaction path.

Automation exposes one top input slot only below level `7`. It accepts a Cocoa-Beans stack once,
runs the same deterministic-first-level/strict-double transition, emits event `1500`, and removes
the one-slot item whether the chance succeeded or failed. Maturation, Bone-Meal extraction and
event rendering remain with the Composter/block/client owners.

### Natural jungle-tree join

Only configured feature `jungle_tree` contains the Cocoa decorator, ordered before trunk-vine and
leaf-vine decorators with global probability `0.2`. `mega_jungle_tree` and the sapling-selected
`jungle_tree_no_vine` contain no Cocoa decorator, so ordinary or two-by-two player-grown Jungle
Saplings do not create Cocoa.

Natural Jungle and Sparse-Jungle biomes reference `trees_jungle` and `trees_sparse_jungle`.
Their placed counts choose `50/51` and `2/3` with weights `9/1`. The ordered random selectors use
small `jungle_tree` only after earlier alternatives fail:

- Jungle tries Fancy Oak `0.1`, Jungle Bush `0.5`, Mega Jungle `0.33333334` and Fallen Jungle
  `0.0125`;
- Sparse Jungle tries Fancy Oak `0.1`, Jungle Bush `0.5` and Fallen Jungle `0.0125`.

For each successfully placed small tree, the Cocoa decorator consumes one strict
`nextFloat()<0.2` gate, returns on an empty log list, then visits every log no more than two blocks
above the first sorted log. It checks North, East, South and West in order; each consumes a float
and proceeds on inclusive `draw<=0.25`. An admitted face targets the opposite-adjacent cell, and
only air consumes `nextInt(3)` for age `0..2` before offering Cocoa facing back toward the log.
Tree geometry, selector short-circuiting, placement filters, decorator cursor/order and ignored
write results remain `WGEN-PIPELINE-001`.

**Persistence and reload boundary:**

Placed Cocoa persists registry identity, age and facing. Cocoa-Beans stacks persist identity,
count and valid patches. They store no random-tick, recipe, advancement, loot, Composter,
pathfinding or tree cursor state; those values remain with their owners.

Block-tag reload changes future support and tool-speed tests. Recipe/advancement and loot reload
change future matching, listeners and drops. Worldgen reload changes future feature selection and
decoration; existing chunks and states are not replayed. Existing stacks, learned recipes,
Composter contents and generated trees are not rewritten. Resource reload independently controls
language, blockstate, model and texture projection.

**Client and wire projection:**

Server block-state projection uses IDs `9481..9492`; generic stack encoding uses raw item ID
`1094` plus patches. Locked English names are `Cocoa` for the block and `Cocoa Beans` for the
item. The blockstate maps age `0/1/2` to `cocoa_stage0/1/2` and facing south/west/north/east to
Y rotations `0/90/180/270`. Each block model disables ambient occlusion, uses its matching
stage texture and renders the pod plus support stem.

The item uses ordinary generated `minecraft:item/cocoa_beans` with its same-named texture, common
rarity and no forced glint. Natural Blocks contains it exactly once, after Wheat Seeds and before
Pumpkin Seeds; no other vanilla tab contains it. This leaf adds no packet field, acknowledgement
or connection-local state.

**Branches and aborts:**

Identity/components; twelve states; four-direction placement/support/update; age-zero/one/two
random-tick and bone-meal paths; hand/Axe/Sword/other tool, piston and path type; age/explosion
loot; two recipes/grids/unlocks; player/automated Composter levels and draws; Jungle/Sparse-Jungle
selector alternatives, decorator gate/log/face/air/age/write; persistence/reload/wire;
name/blockstate/model/texture/tab.

**Constants and randomness:**

Block/item IDs `396/1094`; state IDs `9481..9492`; ages `0..2`; hardness/resistance `0.2/3`;
four support identities; `COCOA` path malus default `0`, Bee/Parrot `-1`; random growth
`nextInt(5)==0`; bone meal `+1`; loot `1/3` plus per-unit explosion decay; recipe outputs `1/8`;
compost `0.65f`, maturation `20`, event `1500`; natural counts and selector chances as listed;
decorator global `<0.2`, log band `0..2`, face `<=0.25`, age `nextInt(3)`.

**Side effects:**

Block-item consumption, placement statistic/sound and block write; growth writes; support/piston
removal; break loot; crafting inputs/results and recipe knowledge; Composter state, event,
schedule, statistic and consumption; natural tree Cocoa offers; ordinary block/item persistence,
wire and client projection.

**Gates:**

Exact identity; replaceability/permission/write; horizontal live support; selected random tick;
age; Bone-Meal admission; tool and break context; explosion; grid/recipe/result capacity;
advancement listener; Composter level/input/RNG/automation/material policy; biome/feature/
selector/tree/decorator/log/face/air/write; registry/decode; client language/model/tab bootstrap.

**State read/written:**

Reads item/components, placement, support tags, age/facing, tick/Bone-Meal, tool/block tags,
break/loot, grid/recipe/advancement, Composter, pathfinding, worldgen and client state. Writes only
the placement, growth, loot, processing, Composter, worldgen, stack and projection state listed
above.

**Failure behavior:**

No adjacent live support returns null placement; relevant support loss returns Air. Nonzero
growth draw or mature age preserves Cocoa. Mature bone meal is rejected. Invalid break/loot,
recipe, result, Composter, selector, tree, decorator, air or write gates produce only their
owner-defined failure result. Reload affects future evaluation only; missing client resources
cannot grant server behavior.

**Boundary cases and quirks:**

Cocoa Beans is a custom-named ordinary BlockItem for differently keyed block Cocoa. Facing points
toward support. Random growth draws from `ServerLevel.getRandom`, not the callback cursor, while
bone meal is guaranteed and RNG-free after target admission. Tool tags change speed but never
drops. Immature Cocoa drops one and mature drops three; explosion decay is per bean. Cocoa Beans
is compostable but not edible, direct dye behavior or fuel. Natural small Jungle Trees can create
Cocoa, but Jungle Saplings use a different no-vine/no-Cocoa configured feature.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-DATA-001`; `OFF-REPORT-001`;
`net.minecraft.references.BlockItemIds`;
`net.minecraft.world.level.block.Blocks`;
`net.minecraft.world.item.Items#createBlockItemWithCustomItemName(net.minecraft.world.level.block.Block)`;
`net.minecraft.world.level.block.CocoaBlock`;
`net.minecraft.world.level.block.CocoaBlock#isRandomlyTicking`;
`net.minecraft.world.level.block.CocoaBlock#randomTick`;
`net.minecraft.world.level.block.CocoaBlock#canSurvive`;
`net.minecraft.world.level.block.CocoaBlock#getShape`;
`net.minecraft.world.level.block.CocoaBlock#getStateForPlacement`;
`net.minecraft.world.level.block.CocoaBlock#updateShape`;
`net.minecraft.world.level.block.CocoaBlock#isValidBonemealTarget`;
`net.minecraft.world.level.block.CocoaBlock#isBonemealSuccess`;
`net.minecraft.world.level.block.CocoaBlock#performBonemeal`;
`net.minecraft.world.level.block.CocoaBlock#isPathfindable`;
`net.minecraft.world.level.pathfinder.WalkNodeEvaluator#getPathTypeFromState`;
`net.minecraft.world.level.pathfinder.FlyNodeEvaluator`;
`net.minecraft.world.level.pathfinder.PathType`;
`net.minecraft.world.entity.animal.bee.Bee`;
`net.minecraft.world.entity.animal.parrot.Parrot`;
`net.minecraft.world.level.block.ComposterBlock#bootStrap`;
`net.minecraft.world.level.block.ComposterBlock#useItemOn`;
`net.minecraft.world.level.block.ComposterBlock#insertItem`;
`net.minecraft.world.level.block.ComposterBlock#addItem`;
`net.minecraft.world.item.CreativeModeTabs`;
`reports/blocks.json#minecraft:cocoa`;
`reports/registries.json#minecraft:{block,item,worldgen/tree_decorator_type}`;
`reports/minecraft/components/item/cocoa_beans.json`;
`data/minecraft/tags/block/{supports_cocoa,jungle_logs,mineable/axe,sword_efficient}.json`;
`data/minecraft/loot_table/blocks/cocoa.json`;
`data/minecraft/recipe/{brown_dye,cookie}.json`;
`data/minecraft/advancement/recipes/{misc/brown_dye,food/cookie}.json`;
`data/minecraft/worldgen/{configured_feature/{jungle_tree,mega_jungle_tree,jungle_tree_no_vine,trees_jungle,trees_sparse_jungle},placed_feature/{jungle_tree,trees_jungle,trees_sparse_jungle},biome/{jungle,sparse_jungle}}.json`;
`assets/minecraft/blockstates/cocoa.json`;
`assets/minecraft/models/block/cocoa_stage{0,1,2}.json`;
`assets/minecraft/textures/block/cocoa_stage{0,1,2}.png`;
`assets/minecraft/{items,models/item}/cocoa_beans.json`;
`assets/minecraft/textures/item/cocoa_beans.png`;
`BLK-SAPLING-001`; `WGEN-PIPELINE-001`; `ITM-COOKIE-001`;
`ITM-RECIPE-001`; `ITM-LOOT-001`; `EXP-BLK-085`.

**Test vectors:**

Run `EXP-BLK-085` over all twelve states and default/patched Cocoa Beans. Cross every nearest-look
direction order with four support identities, missing/reloaded support, replacement/collision/
permission/write and support-loss updates. Run controlled selected random callbacks, both bone-meal
ages, every tool/piston/path query, all age/explosion loot draws, both recipes and unlock routes,
and every player/automated Composter level/draw/material branch.

Exercise both natural-biome selector lists at every alternative boundary; run empty/nonempty
trees, log Y bands, all face-draw equalities, occupied/air targets, ages and failed writes. Assert
zero cells across `1,212` templates, then persist/reload/synchronize and verify IDs, names, shapes,
blockstate rotations, three block models/textures, flat item model and Natural-Blocks ordering.

**Limits:**

Generic block-item placement/component application, selected random ticks, shape/collision
queries, block update/break/piston/loot, Bone-Meal item work, crafting/progression, Composter
maturation/extraction, path search, tree/biome generation, packet encoding and rendering remain
with `BLK-PLACE-001`, `SIM-RANDOM-001`, `BLK-003`, `BLK-UPDATE-001`,
`BLK-BREAK-001`, `ITM-LOOT-001`, `ITM-RECIPE-001`, `ITM-ADVANCEMENT-001`,
`MOB-AI-001`, `WGEN-PIPELINE-001`, `PROTO-PLAY-CLIENTBOUND-LEVEL-001`,
`PROTO-PLAY-CLIENTBOUND-CONTAINER-001` and `CLI-006`.
