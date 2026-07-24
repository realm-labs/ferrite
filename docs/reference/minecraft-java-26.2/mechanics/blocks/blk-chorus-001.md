# Blocks mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `BLK-CHORUS-001` — Chorus flowers turn connected stems into upward growth, branches, or dead tips

**Parent:** `SIM-004`, `SIM-005`, `BLK-001`, `BLK-002`, `BLK-003`, `BLK-005`, `RED-001`,
`PLY-002`, `PLY-005`, `PLY-006`, `ITM-003`, `ITM-004`, `ITM-005`, `ITM-006`, `ITM-007`,
`ENT-001`, `ENT-005`, `ENT-007`, `MOB-001`, `MOB-004`, `MOB-005`, `MOB-006`, `ENV-003`,
`WGEN-002`, `WGEN-003`, `WGEN-004`, `CLI-001`, `CLI-006`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked registrations, chorus block and consume-effect classes, reports, loot,
recipes, advancements, tags, End worldgen, all 1,212 structure templates and exact client assets
close both blocks and all four chorus items.

**Applies when:**

`minecraft:chorus_plant` or `minecraft:chorus_flower` is placed, updated, randomly ticked, hit by a
projectile, broken, replaced by worldgen, generated, persisted or rendered; or a chorus plant,
chorus flower, `minecraft:chorus_fruit` or `minecraft:popped_chorus_fruit` stack is acquired, used,
processed, persisted or rendered.

**Authoritative state:**

| Identity | Registry ID | State/item ID | Schema or role |
|---|---:|---:|---|
| chorus plant | block `656`, item `352` | states `14642..14705`, default `14705` | six directional connection booleans |
| chorus flower | block `657`, item `353` | states `14706..14711`, default `14706` | `age=0..5`; age five is dead |
| chorus fruit | item `1313` | common stack of 64 | always-edible food plus random teleport and one-second cooldown |
| popped chorus fruit | item `1314` | common stack of 64 | smelting result and crafting ingredient |

Both blocks register purple map color, forced nonsolid behavior, strength `0.4`, Wood sound,
no occlusion and piston reaction `DESTROY`. The flower additionally registers random ticks,
rejects mob spawning and is never a redstone conductor. Neither has a block entity, fluid property,
waterlogging, emitted light, fire registration or lava-ignition property.

The plant's collision/outline is a centered 10-pixel cube joined to a 10-pixel-wide arm for every
true direction; it never pathfinds. The flower retains a full-block collision/outline and exposes a
separate centered 14-pixel-wide, 15-pixel-high block-support shape.

**Transition and ordering:**

#### Placement, connections and support

Plant placement reads all six neighbors. Each direction connects to a plant or flower; downward
also connects to `supports_chorus_plant`, whose only locked member is End stone. A later neighbor
update first rechecks survival. If still valid, it changes only the updated direction's bit using
the same predicate. If invalid, it schedules the plant's block tick after one tick and leaves the
current connection state for ordinary update handling. The callback rechecks and destroys with
drops on failure.

A plant survives directly above a plant or `supports_chorus_plant`. It can instead survive from a
horizontal plant whose own lower neighbor is a plant or support-tag member, provided the candidate
is not simultaneously bracketed by non-air above and below. Non-plant horizontal blocks do not
support it. This predicate follows live neighbors rather than trusting stored connection bits.

A flower survives above a plant or `supports_chorus_flower`; that tag also contains only End stone.
With AIR below, it instead requires at least one and at most one horizontal plant, while every
other horizontal neighbor must be AIR. Any other non-air block below rejects it. Non-up neighbor
updates that invalidate this predicate schedule a one-tick flower callback; an upward update does
not recheck because the upper cell is not a support input. The callback destroys an invalid flower
with drops.

The ordinary block items place their default identities and generic clone selection returns the
matching plant or flower item. Neither block can be bone-mealed.

#### Live flower random growth

Only flower ages zero through four randomly tick. A callback first requires AIR above and an
above-position Y no greater than the level maximum, then reads age. There is no light predicate or
additional chorus-specific probability draw.

The upward-growth decision reads below:

- direct `supports_chorus_flower` makes upward growth eligible without a draw;
- AIR makes it eligible, permitting a horizontally supported branch to rise;
- a plant starts a vertical count at one and scans at most four more cells downward, recording
  whether the first non-plant is support-tagged;
- any other block prevents upward growth.

A plant count below two is eligible. Otherwise the count must be at most `nextInt(5)` when the scan
found tagged ground, or at most `nextInt(4)` when it did not. Upward growth then also requires every
horizontal neighbor of the cell above to be AIR and the cell two above to be AIR. Success offers
the current position as a freshly connection-derived plant with flags `2`, offers a flower of the
same age above with flags `2`, ignores both results, then emits level event `1033` at the new tip.
The event projects Chorus Flower Grow sound `372`, Blocks source, volume/pitch `1/1`.

If upward growth fails and age is below four, the callback draws `nextInt(4)` branch attempts and
adds one attempt only when the downward scan found tagged ground. Each attempt independently draws
a horizontal direction, so duplicates are possible. Its target is beside the current flower, not
beside the cell above, and must have AIR at target and target-below plus AIR at the three horizontal
sides other than the parent direction. Each admitted target receives an age-plus-one flower with
flags `2`, ignored result, followed by event `1033`.

At least one admitted branch converts the original flower to a freshly connected plant with flags
`2`. No admitted branch converts it to age-five flower and emits event `1034`, Chorus Flower Death
sound `371` at volume/pitch `1/1`. Age four that cannot grow upward skips branch RNG and dies by the
same path. These writes and following level events do not roll back when a write result is false.

#### Projectile and ordinary break loot

A projectile hit destroys only the flower, and only on the server when the projectile may interact
at the position and may break blocks. Player-owned projectiles delegate interaction permission to
their owner; a non-player owner requires `mob_griefing`, while an ownerless projectile is admitted.
Breaking additionally requires the projectile type in `impact_projectiles` and
`projectiles_can_break_blocks=true`. Destruction requests drops and supplies the projectile as the
destroying entity.

Plant loot selects an integer count uniformly from zero through one chorus fruit, then applies
explosion decay. It has no state, tool, Fortune or Silk Touch condition. Flower loot selects one
flower independent of age only when the loot context has a `this` entity and the entry survives an
explosion. Consequently ordinary player and admitted projectile destruction can drop the flower,
while entity-less support-tick destruction does not. Random sequences are independently
`minecraft:blocks/chorus_plant` and `minecraft:blocks/chorus_flower`.

#### Chorus-fruit consumption and teleport

Chorus fruit supplies nutrition/saturation `4/2.4`, can always be eaten, uses the default
1.6-second consumable cadence and has a `1.0`-second use cooldown. Generic completion first applies
food, particles, sound, stat, consume criterion and listener effects, then invokes its sole
`teleport_randomly` consume effect on the server. After item behavior and shrink, the generic
after-use path applies the 20-tick item-keyed cooldown.

The teleport effect makes at most 16 attempts. Each consumes three entity-RNG doubles and proposes
X/Y/Z by adding `(draw - 0.5) * 16`; Y is clamped to the dimension minimum through
`minY + logicalHeight - 1`. A passenger stops riding before the first attempted teleport. The
candidate must be in an already loaded chunk; the search descends until the block below blocks
motion, then temporarily moves the entity and requires both no collision and no liquid. Failure
restores the pre-attempt position.

The first success broadcasts entity event `46`, stops path navigation for a pathfinding mob, emits
`TELEPORT` at the old position, plays sound at the new position, resets fall distance and finally
resets current impulse context. A fox selects Fox Teleport sound `658` and Neutral source; every
other living entity selects Chorus Fruit Teleport `373` and Players source. If all attempts fail,
the effect returns false without sound, game event, fall reset or impulse reset, but the fruit is
still consumed, cooldown still applies, and a former passenger remains dismounted.

#### Recipes, tags and progression

Smelting one chorus fruit produces one popped chorus fruit, experience `0.1`, with the serializer's
default 200-tick cooking time. Four popped fruit in a 2-by-2 shape produce four purpur blocks; one
blaze rod over one popped fruit produces four End rods. Their three recipe advancements gate on the
corresponding chorus ingredient plus recipe unlock and award the matching recipe.

Popped chorus fruit has no independent use component. Neither fruit is compostable or fuel.
Chorus fruit is one of 40 independent `husbandry/balanced_diet` consume criteria; completing all
requirements awards 100 experience.

The flower is directly `bee_attractive` and a block/item `flowers` member; its item composes into
`bee_food`. The plant and flower are directly `mineable/axe` and `sword_efficient`. Bee search,
pollination and breeding retain their generic owners; chorus flowers are not `bee_growables`.

#### End generation and replacement

The placed `chorus_plant` feature chooses an inclusive uniform count `0..4`, applies in-square X/Z,
the `MOTION_BLOCKING` heightmap and biome filtering, and runs in End Highlands vegetal decoration.
Its configured feature succeeds only when the origin is AIR and the block below is
`supports_chorus_plant`, then calls the code-built generator with horizontal radius eight.

Generation writes a connected plant at the origin and recursively grows stems. Each recursive stem
draws `1 + nextInt(4)` vertical segments, plus one segment at depth zero. Every segment requires all
four horizontal neighbors to be AIR; failure immediately abandons that branch after any earlier
writes. Each accepted segment and the preceding segment's refreshed connections are flags-2 writes
whose results are ignored.

At depths below four, branch-attempt count is `nextInt(4)`, plus one at depth zero. Each attempt
draws a horizontal direction and targets beside the stem top. The target must stay at strict
absolute X/Z distance below eight from the original root, have AIR at target and below, and have AIR
on all horizontal sides except the parent direction. An admitted branch writes its plant and
parent connection before recursing at depth plus one. A stem with no successful branch ends one
cell above its top in an age-five flower. Thus configured generation creates dead tips, permits
duplicate direction attempts, reaches recursion depth four and can reach 22 cells above the root.

All four planted/nonplanted crimson/warped huge-fungus configurations explicitly admit both chorus
blocks in their replaceable predicate. An exhaustive scan of all 1,212 bundled structure templates
finds zero raw plant or flower cells.

**Client projection:**

Plant state is a six-part multipart model. Every true bit adds the appropriately rotated side arm;
every false bit adds one cap chosen from four untinted variants with weights `2/1/1/1`. The plant
item uses the fixed complete block model. Flower ages zero through four share the live untinted
model, age five selects the dead model, and the flower item always uses the live model. Both fruit
items are untinted generated flats.

Natural Blocks orders plant then flower after small dripleaf and before glow lichen. Food & Drinks
orders chorus fruit after glow berries and before carrot. Ingredients orders popped chorus fruit
after shulker shell and before echo shard.

**Branches and aborts:**

Placement/support direction; stored connection bit; flower age; above AIR/build height; below
support/plant/AIR/other; vertical count/root discovery/draw; upward clearance; branch count,
direction and clearance; write result; projectile type/owner/rules/permission; entity/explosion
loot context; consume eligibility; 16 teleport candidates; chunk/ground/collision/liquid; fox
sound; recipes/tags/advancement; feature count/origin/support; recursive height/depth/radius and
model/tab projection.

**Constants and randomness:**

Flower dead age `5`; support delay `1`; plant core/arm width `10`; flower support shape `14×15`;
growth scan four; rooted/unrooted vertical bounds `5/4`; branch bound `4`; events `1033/1034`;
plant loot `0..1`; food `4/2.4`; consume time `1.6` seconds; cooldown `1.0` second; teleport
diameter/attempts `16/16`; smelting XP/time `0.1/200`; placed count `0..4`; generator radius/depth
`8/4`; generated segment bound `4`.

**Side effects:**

Block states, scheduled ticks, drops/stacks, food and cooldown state, mount/navigation/position/fall
and impulse state, entity/level/game events, sounds, recipe/progression state, generated cells and
client projection.

**Gates:**

Live neighbors and support tags; flower age; generic random-tick admission; build height and AIR;
vertical/branch RNG; projectile permissions/tags/game rules; loot entity/explosion context;
consumable and teleport target safety; recipe/advancement/tag snapshots; feature biome/origin,
recursive clearance/radius/depth and client model selectors.

**State read/written:**

Reads block identity/connections/age, adjacent/downward states, height, RNG, projectile owner/type,
game rules, loot context, living position/mount/bounds/collision/liquid, stacks/components and
active data/worldgen/client snapshots. Writes block states/ticks, drops/stacks, food/cooldown,
entity position/mount/navigation/fall/impulse, progression, generated blocks and visible effects.

**Failure behavior:**

Unsupported scheduled ticks destroy with drops; failed growth/generation writes are ignored without
rolling back their stated following writes/events; blocked growth may branch or die; rejected
projectiles do nothing; failed teleport attempts restore position and continue, while total failure
still consumes the fruit; failed feature admission returns false without generation.

**Persistence boundary:**

Plant connection bits and flower age persist as ordinary palette state; stacks, cooldown and
progression persist through their owners. Support, connection, growth and generation predicates
recompute from live state. Growth, loot, teleport and feature RNG do not persist or catch up.
Reload replaces loot, recipe, advancement, tag and worldgen snapshots without rewriting existing
palettes, stacks or entity positions.

**Boundary cases and quirks:**

A horizontally supported flower requires AIR below and exactly one plant neighbor. A plant's
visual connection can include a flower even though the flower cannot provide plant survival.
Direct End-stone support admits flower rise without setting the rooted branch-attempt bonus.
Repeated branch directions waste attempts. Age four can rise without aging but otherwise dies.
Entity-less support destruction cannot drop a flower. An unsuccessful chorus-fruit teleport can
still dismount its consumer, consume the item and start cooldown. Generated tips are always dead.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-DATA-001`; `OFF-REPORT-001`;
`net.minecraft.world.level.block.Blocks`;
`net.minecraft.world.item.Items`;
`net.minecraft.world.level.block.ChorusPlantBlock#getStateForPlacement(net.minecraft.world.item.context.BlockPlaceContext)`;
`net.minecraft.world.level.block.ChorusPlantBlock#getStateWithConnections(net.minecraft.world.level.BlockGetter,net.minecraft.core.BlockPos,net.minecraft.world.level.block.state.BlockState)`;
`net.minecraft.world.level.block.ChorusPlantBlock#updateShape(net.minecraft.world.level.block.state.BlockState,net.minecraft.world.level.LevelReader,net.minecraft.world.level.ScheduledTickAccess,net.minecraft.core.BlockPos,net.minecraft.core.Direction,net.minecraft.core.BlockPos,net.minecraft.world.level.block.state.BlockState,net.minecraft.util.RandomSource)`;
`net.minecraft.world.level.block.ChorusPlantBlock#canSurvive(net.minecraft.world.level.block.state.BlockState,net.minecraft.world.level.LevelReader,net.minecraft.core.BlockPos)`;
`net.minecraft.world.level.block.ChorusFlowerBlock#randomTick(net.minecraft.world.level.block.state.BlockState,net.minecraft.server.level.ServerLevel,net.minecraft.core.BlockPos,net.minecraft.util.RandomSource)`;
`net.minecraft.world.level.block.ChorusFlowerBlock#canSurvive(net.minecraft.world.level.block.state.BlockState,net.minecraft.world.level.LevelReader,net.minecraft.core.BlockPos)`;
`net.minecraft.world.level.block.ChorusFlowerBlock#generatePlant(net.minecraft.world.level.LevelAccessor,net.minecraft.core.BlockPos,net.minecraft.util.RandomSource,int)`;
`net.minecraft.world.level.block.ChorusFlowerBlock#onProjectileHit(net.minecraft.world.level.Level,net.minecraft.world.level.block.state.BlockState,net.minecraft.world.phys.BlockHitResult,net.minecraft.world.entity.projectile.Projectile)`;
`net.minecraft.world.item.consume_effects.TeleportRandomlyConsumeEffect#apply(net.minecraft.world.level.Level,net.minecraft.world.item.ItemStack,net.minecraft.world.entity.LivingEntity)`;
`net.minecraft.world.entity.LivingEntity#randomTeleport(double,double,double,boolean)`;
`net.minecraft.world.entity.projectile.Projectile#mayInteract(net.minecraft.server.level.ServerLevel,net.minecraft.core.BlockPos)`;
`net.minecraft.world.entity.projectile.Projectile#mayBreak(net.minecraft.server.level.ServerLevel)`;
`net.minecraft.world.level.levelgen.feature.ChorusPlantFeature#place(net.minecraft.world.level.levelgen.feature.FeaturePlaceContext)`;
`net.minecraft.client.renderer.LevelEventHandler#levelEvent(int,net.minecraft.core.BlockPos,int)`;
`net.minecraft.world.item.CreativeModeTabs#bootstrap(net.minecraft.core.Registry)`;
`reports/blocks.json#minecraft:{chorus_plant,chorus_flower}`;
`reports/registries.json#minecraft:{block,item,sound_event}`;
`reports/minecraft/components/item/{chorus_plant,chorus_flower,chorus_fruit,popped_chorus_fruit}.json`;
`data/minecraft/loot_table/blocks/{chorus_plant,chorus_flower}.json`;
`data/minecraft/recipe/{popped_chorus_fruit,purpur_block,end_rod}.json`;
`data/minecraft/advancement/{husbandry/balanced_diet,recipes/{misc/popped_chorus_fruit,building_blocks/purpur_block,decorations/end_rod}}.json`;
`data/minecraft/tags/{block/{supports_chorus_plant,supports_chorus_flower,bee_attractive,flowers,mineable/axe,sword_efficient},item/{bee_food,flowers}}.json`;
`data/minecraft/worldgen/{configured_feature/chorus_plant,placed_feature/chorus_plant,biome/end_highlands}.json`;
`data/minecraft/worldgen/configured_feature/{crimson_fungus,crimson_fungus_planted,warped_fungus,warped_fungus_planted}.json`;
`data/minecraft/structure/**/*.nbt`;
`assets/minecraft/blockstates/{chorus_plant,chorus_flower}.json`;
`assets/minecraft/models/{block/chorus_*,item/{chorus_fruit,popped_chorus_fruit}}.json`;
`assets/minecraft/items/{chorus_plant,chorus_flower,chorus_fruit,popped_chorus_fruit}.json`;
`BLK-STATE-001`; `BLK-PLACE-001`; `BLK-BREAK-001`; `BLK-UPDATE-001`;
`SIM-RANDOM-001`; `PLY-INTERACT-001`; `ITM-USE-001`; `ITM-HUNGER-001`;
`ITM-FURNACE-001`; `ITM-CRAFT-001`; `ITM-ADVANCEMENT-001`; `ITM-LOOT-001`;
`ENT-008`; `MOB-AI-001`; `MOB-BREED-001`; `WGEN-PIPELINE-001`; `EXP-BLK-083`.

**Test vectors:**

Cross all 64 connection states and six flower ages through placement, support and every neighbor
direction. Script every vertical count/root/draw, rise clearance, age and duplicate branch stream
plus failed writes. Break both blocks with entity-less, player, admitted/rejected projectile and
explosion contexts. Consume fruit across mount, chunk, ground, collision, liquid and 16-attempt
boundaries. Exercise recipes, tags, criteria, feature count/height/radius/depth/abort streams,
fungus replacement, every template, save/reload and exact sound/model/tab projection.

**Limits:**

Generic random-tick admission, placement/breaking, projectile impact, loot/explosion evaluation,
consumable/hunger/cooldown order, teleport synchronization, recipe processing, bee AI, feature
traversal, persistence, protocol and rendering remain with their cited owners. This leaf owns the
four identities' selectors, constants, local transitions, data joins and projection.
