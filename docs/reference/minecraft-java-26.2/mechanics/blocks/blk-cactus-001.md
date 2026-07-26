# Blocks mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `BLK-CACTUS-001` — Cactus ages into height or flower, damages every contact and anchors desert acquisition

**Parent:** `SIM-004`, `SIM-005`, `SIM-RANDOM-001`, `BLK-001`,
`BLK-STATE-001`, `BLK-002`, `BLK-PLACE-001`, `BLK-BREAK-001`,
`BLK-BREAK-HOOK-001`, `BLK-BREAK-CONTENT-001`, `BLK-UPDATE-001`,
`PLY-002`, `PLY-005`, `PLY-006`, `PLY-INTERACT-001`, `PLY-BREAK-001`,
`PLY-COLLISION-001`, `PLY-AUTOJUMP-001`, `ITM-003`, `ITM-004`,
`ITM-006`, `ITM-RECIPE-001`, `ITM-FURNACE-001`, `ITM-LOOT-001`,
`ITM-ADVANCEMENT-001`, `MOB-AI-001`, `ENT-001`, `ENT-DAMAGE-001`,
`ENV-001`, `ENV-002`, `ENV-003`, `ENV-FLUID-001`, `ENV-FIRE-001`,
`ENV-LIGHT-001`, `WGEN-003`, `WGEN-PIPELINE-001`,
`WGEN-JIGSAW-VILLAGES-001`, `WGEN-JIGSAW-TRIAL-CHAMBERS-001`,
`CLI-001`, `CLI-006`, `CLI-UI-001`, `CLI-EFFECT-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked registration, class and exact-identity consumer
sweeps, reports, tags, recipes, loot, trades, features, biome/pool resources,
client assets and exhaustive raw/UTF scans of all 1,212 decoded structure
templates fix every Cactus state and runtime join. Cactus Flower remains owned
by its existing `fire-fuel` catalog family; this leaf records only the concrete
outgoing growth/support boundary needed to specify Cactus.

**Applies when:**

`minecraft:cactus` is placed, supported, updated, randomly ticked, contacted,
carried or placed by an Enderman, eaten by a Camel, composted, cooked, traded,
drawn from a chest, placed by a configured/placed feature or structure,
persisted, synchronized or rendered.

**Authoritative state:**

Cactus is a `CactusBlock` with integer property `age=0..15`, no block entity and
exactly 16 states. State IDs `6929..6944` correspond monotonically to ages
`0..15`; default state `6929` has age `0`. Protocol block ID is `279`. Its
ordinary stack-64 `BlockItem` has raw item ID `368` and no special components.

Registration supplies `MapColor.PLANT`, random ticking, destroy time and
explosion resistance `0.4`, Wool sounds and piston reaction `DESTROY`. Unset
defaults are Harp note instrument, friction `0.6`, speed/jump factors `1`,
restitution and light emission `0`, and no lava ignition.

The outline/selection shape is a column from `(1/16,0,1/16)` to
`(15/16,1,15/16)`. Collision ends at Y `15/16` with the same horizontal inset.
Occlusion follows the outline, support and visual shapes follow collision, and
interaction shape is empty. The nonfull geometry makes solid rendering,
redstone conduction, view blocking and suffocation false; shade brightness is
`1`, skylight propagates, light dampening is `0`, and fluid state is empty.
`isPathfindable` explicitly returns false for every computation type.

Wool sound-event protocol IDs are Break `1858`, Fall `1859`, Hit `1860`, Place
`1861` and Step `1862`. The block has no signal, comparator, use, attack,
projectile, fall-on, block-event or client animation override.

**Transition and ordering:**

### Placement, survival and scheduled removal

Survival first checks every horizontal direction. It rejects the state if any
horizontal neighbor reports `isSolid()` or has fluid tagged `minecraft:lava`.
The block directly below must then be exact Cactus or a member of
`minecraft:supports_cactus`; the locked tag contains only
`#minecraft:sand`. Finally, the state directly above must report
`liquid()==false`.

On any shape update, failure of that predicate schedules this Cactus for a
block tick after `1`; the method still returns the base `updateShape` result and
does not remove the block immediately. At the due tick, a still-invalid block
is passed to `destroyBlock(position,true)` and its Boolean result is ignored.
Recovery before the callback therefore preserves it.

The direct block tags are `enderman_holdable`, `happy_ghast_avoids` and
`support_override_cactus_flower`. The last tag lets a Cactus Flower accept
Cactus as its lower support even though the collision face is inset and only
15/16 high. Flower placement also accepts Farmland through that tag or any
block with a CENTER-sturdy upper face.

### Random-tick age machine

Only a selected random tick runs this machine. It first computes the cell above
and returns immediately unless that cell is empty block; a cap therefore
prevents column scanning, RNG and age mutation.

With an empty upper cell, the block initializes height `1` and scans downward
through exact Cactus blocks. Each additional Cactus increments height. If
height becomes `3` while the current age is `15`, the method returns
immediately. At every other age the scan may continue through a force-built
taller column, but growth never extends a natural height-three age-15 top.

At age `8`, the method asks whether default age-zero Cactus could survive in the
upper cell. If so, it draws `nextDouble()` and succeeds when the result is
`<=0.25` for a height of at least three, otherwise `<=0.1`. Success calls
`setBlockAndUpdate(above, Cactus Flower default)` and ignores the result.
Success or failure then continues into ordinary age handling.

At age `15` with height below three, it:

1. calls `setBlockAndUpdate(above, Cactus default age zero)` and ignores the
   result;
2. resets the current block to age zero with flags `260` and ignores that
   result; and
3. explicitly calls `neighborChanged` on the reset state, naming the upper
   position and this Cactus block, even when the upper write failed.

Flags `260` are `UPDATE_INVISIBLE` (`4`) plus
`UPDATE_SKIP_BLOCK_ENTITY_SIDEEFFECTS` (`256`), also named `UPDATE_NONE`.
At every age below `15`, including age `8` after the flower attempt, it writes
age plus one at the current position with flags `260` and ignores the result.

Starting at age zero, the ninth selected random tick attempts a flower and the
sixteenth grows the next Cactus and resets the top. A successfully written
flower leaves the source at age `9`; its nonempty upper cell caps further
progress until the flower is removed. Only the top block advances because
every lower Cactus has a nonempty Cactus above. Natural height is capped at
three, while a height-three top still receives the age-eight flower attempt
with probability `1/4`.

### Entity contact and damage

`entityInside` ignores the supplied current-versus-swept intersection Boolean
and effect applier and always submits `1.0` Cactus damage to the entity. Cactus
therefore acts for both current and swept contacts dispatched by the generic
block-effect traversal; the generic damage owner decides whether the submitted
hit is accepted.

The `minecraft:cactus` damage type has exhaustion `0.1`, message ID `cactus`
and scaling `when_caused_by_living_non_player`. This source has no causing
living entity, so the scaling condition does not activate. Its direct damage
tags are `bypasses_shield`, `no_knockback`, `panic_environmental_causes` and
`sulfur_cube_with_block_immune_to`; it is not in `bypasses_armor`. Item
entities and other ordinary entities remain damage candidates under their
generic damage rules.

`WalkNodeEvaluator#getPathTypeFromState` classifies exact Cactus as
`DAMAGING`. `EntityType#isBlockDangerous` returns true for exact Cactus after
the entity-type block-immunity and fire branches; no locked block-immunity tag
contains Cactus. `SpawnUtil.Strategy.LEGACY_IRON_GOLEM` independently rejects
exact Cactus as a ground block.

Careful Happy Ghast movement rejects the direct `happy_ghast_avoids` membership
before fluid handling. These identity consumers do not change Cactus state.

### Enderman and Camel consumers

An empty-handed Enderman with mob griefing enabled runs its take-block goal on
the one-in-`reducedTickDelay(20)` branch. It samples the familiar nearby
floor-coordinate box, ray-tests the outline and, when the hit block belongs to
`enderman_holdable`, calls `removeBlock(position,false)`, emits the block-
destroy event and stores the block's *default* state. Every carried Cactus is
therefore normalized to age zero rather than retaining the sampled age.

A carrying Enderman runs its leave-block goal on the one-in-
`reducedTickDelay(2000)` branch. The generic target must be air, have full-solid
non-Bedrock support below, admit the neighbor-updated carried state, survive
and have no entity collision. For Cactus, survival additionally requires Sand-
tag support and the horizontal/no-lava constraints. Accepted placement writes
flags `3`, emits the placement event and clears the carried age-zero state.

Cactus item is a direct member of `camel_food`. `Camel#isFood` therefore
accepts it. Eating heals `2` when damaged; a tamed adult can enter love, and a
baby that can age up emits Happy Villager particles and server-side
`ageUp(10)`. If any effect occurred, the Camel emits Eat game event and plays
Camel Eat (sound-event ID `264`) at pitch
`1 + (nextFloat()-nextFloat())*0.2`, then reports success to the generic feeding
owner. A full-health adult with neither love nor age effect reports false, so
that owner does not consume the offered item.

### Acquisition and consumption

The block loot table has one survives-explosion self entry. It has no tool,
Silk Touch, Fortune or age branch. Cactus has no FireBlock flammability row, no
lava-ignition property and no fuel-values entry.

Composter bootstrap assigns Cactus chance `0.5`; player and automated insertion,
level advancement, ready-state timing, extraction and inventory disposition
remain with the Composter owner. The neighboring Cactus Flower identity has
its independent `0.3` row.

The sole cooking record is the Furnace smelting recipe `minecraft:green_dye`:
one exact Cactus produces one Green Dye, awards `1.0` experience and uses the
default `200` cooking ticks. There is no Blast Furnace, Smoker or Campfire
variant. Its recipe advancement is an OR between possessing Cactus and already
having the recipe and awards that recipe.

The Wandering Trader common pool contains a `3` Emerald to `1` Cactus offer
with `8` maximum uses, price multiplier `0.05` and inherited villager XP `1`.
It is one of 76 uniform common candidates, of which five distinct offers are
selected, so its inclusion probability is `5/76`.

Desert village house chest pool one rolls uniformly `3..8` times. Cactus has
weight `10` of total `36`, hence selection probability `5/18` per roll, and
then count uniformly `1..4`. Its random sequence is
`minecraft:chests/village/village_desert_house`.

An empty Flower Pot maps Cactus to `minecraft:potted_cactus`; accepted generic
pot insertion consumes the Cactus item. Potted-Cactus loot has two independent
survives-explosion pools yielding Flower Pot and Cactus, and its client model
contains the pot plus Cactus. The potted block remains a separate catalog
identity.

Cactus appears once in Natural Blocks, directly after Sugar Cane and before
Crimson Roots.

### Configured and placed features

Configured feature `minecraft:cactus` is a `block_column` directed upward with
allowed placement matching exact Air and `prioritize_tip=false`. Its first
layer uses default age-zero Cactus and `biased_to_bottom` height `1..3`.
Sampling is `1 + nextInt(nextInt(3)+1)`, giving heights one, two and three with
probabilities `11/18`, `5/18` and `1/9`. The second layer uses Cactus Flower
and weighted height zero with weight `3` or one with weight `1`, so a flower is
sampled with probability `1/4`.

`BlockColumnFeature#place` samples both heights before admission. Starting one
cell above origin, it scans the full intended length for allowed Air; outer
placed-feature predicates own admission at origin. With
`prioritize_tip=false`, an obstruction truncates from the tip: flower height
first, then upper Cactus height. It writes remaining layer cells from origin
upward with flags `2` and ignores each result. Because sampled total height is
never zero here, it returns true even after truncation to zero or failed writes.

Three placed features consume this configured feature:

- `patch_cactus` applies count `10`, then independent triangular/trapezoid
  offsets X/Z `[-7,7]` and Y `[-3,3]`, then filters for origin Air and default
  Cactus survival. It is a feature-pool element of weight `4/28=1/7` in both
  `village/desert/decor` and `village/desert/zombie/decor`.
- `patch_cactus_desert` orders rarity `1/6`, in-square, `MOTION_BLOCKING`
  heightmap, biome filter, count `10`, those same offsets and survival filters.
  The Desert biome lists it.
- `patch_cactus_decorated` is identical except rarity `1/13`; Badlands, Eroded
  Badlands and Wooded Badlands list it.

Placement-modifier order is observable: rejected rarity/biome/height/origin
gates avoid all later candidate work, while the feature owner retains its
stated ignored-write and success semantics.

### Structure-template census

Exhaustive decoded-NBT and constant-pool UTF scans over all 1,212 templates find
exactly eight raw Cactus cells, all age zero, in three files and no UTF-only
Cactus references:

| Template | Raw coordinates |
|---|---|
| `trial_chambers/corridor/addon/display_3` | `[3,2,2]`, `[3,3,2]` |
| `village/desert/houses/desert_small_house_7` | `[2,1,4]`, `[2,2,4]`, `[2,3,4]` |
| `village/desert/zombie/houses/desert_small_house_7` | `[2,1,4]`, `[2,2,4]`, `[2,3,4]` |

Trial display 3 is one of three weight-one entrance-display pool elements,
rigid and without processors. Normal Desert small house 7 has pool weight `2`,
legacy-rigid projection and no processors. Zombie Desert small house 7 also
has weight `2` and uses `zombie_desert`; that processor removes doors/torches,
webifies selected sandstone/terracotta states and mutates Wheat, but has no
Cactus predicate, so these three cells remain Cactus before later generic
placement admission. Cactus Flower has zero raw and zero UTF occurrences.

The census counts stored source cells, not guaranteed final-world placements;
pool selection, transforms, integrity, clipping, live targets and writes remain
with the structure owners.

**Client projection:**

The blockstate maps every one of the 16 ages through an unconditional
`minecraft:block/cactus` model; age is visually ignored. That model has a full
`[0,0,0]..[16,16,16]` element contributing only up/down faces with cullfaces,
a north/south element `[0,0,1]..[16,16,15]`, and a west/east element
`[1,0,0]..[15,16,16]`. It uses distinct side, top and bottom textures and the
side texture for particles.

The Cactus side and top texture metadata set `alpha_cutoff_bias=0.1`; the
bottom texture has no such metadata. The item definition directly uses the
same block model. There is no conditional age model, tint, special renderer or
custom tooltip. The English name is `Cactus`.

**Branches and aborts:**

Above empty/nonempty; downward scan age and height; prospective survival;
height-dependent flower chance; flower/write failure; age increment versus
growth/reset; current versus recovered scheduled survival; horizontal
solid/lava, lower support and upper liquid gates; submitted versus accepted
damage; entity/path/spawn special consumers; Enderman take/leave gates; Camel
effect/no-effect feeding; loot/explosion, compost, cooking/unlock, trade/chest
draws; feature modifier/order/truncation/write; pool/processor/template
placement; and all reload/client contexts are distinct.

**Constants and randomness:**

States `6929..6944`, block/item IDs `279/368`, age `0..15`; strengths
`0.4/0.4`; outline inset `1/16`, collision top `15/16`; schedule `1`; writes
`260`, growth writes/flower writes with update-and-neighbors semantics;
flower ages/chances `8`, `0.1`/`0.25`; natural maximum height `3`; damage
`1.0`, exhaustion `0.1`; Camel heal/age `2/10` and two pitch draws; compost
`0.5`; cook `200`, XP `1`; trade `3:1`, `8`, `0.05`, inclusion `5/76`;
chest rolls `3..8`, chance `5/18`, count `1..4`; column heights/probabilities
and placement counts/rarities/offsets above; eight cells in three templates.

**Side effects:**

Scheduled ticks and destruction drops; age, upper-Cactus and Flower writes;
explicit neighbor notification; submitted damage and generic entity response;
path/danger/spawn/movement decisions; Enderman removal/events/carried state and
placement; Camel health/love/age/particles/sound/game event/item disposition;
loot, compost level, cooking knowledge/inventory, offers and chest stacks;
feature/structure writes; persistence, packets, sounds and rendering.

**Gates:**

Loaded state and write/break authority; random-tick selection; live neighbors,
fluids and tags; height/age/RNG; effect traversal and generic damage admission;
entity type/AI/rules; inventory and recipe/loot/trade snapshots; feature biome,
modifier, support, Air, obstruction and write gates; pool/process/transform/
clip/live-target gates; registry, resource, map, sound and render context.

**Boundary cases and quirks:**

- A cap blocks even age scanning; a successful flower leaves age `9` and caps
  later growth until removed.
- Natural growth takes 16 selected random ticks from age zero and never exceeds
  height three; a flower attempt is still possible on a height-three age-eight
  top.
- All state writes ignore their Boolean results, and the age-15 branch sends
  explicit neighbor notification even when its upper write failed.
- Contact damage deliberately ignores the current/swept flag, bypasses shields
  and knockback but not armor.
- Endermen carry default age zero, never the observed age.
- The Flower support, Fire/Composter identities and Potted Cactus are boundary
  joins, not additions to this exact Cactus catalog selector.
- The raw eight-cell census excludes generated columns and processor-created
  states and is not a final-world expectation.

**Failure semantics:**

Invalid scheduled Cactus destroys with ordinary drops only if still invalid.
Rejected growth writes do not roll back draws, current-age writes or explicit
notification. Rejected generic damage commits no health response beyond its
owner's rules. Feeding with no effect reports false. Failed crafting/cooking,
loot, trade, chest, feature or structure admission retains only each owner's
earlier documented reads/draws/side effects.

**Client/server authority split:**

The server owns state, schedules, growth, damage, AI, loot, composting, cooking,
trades, chests, worldgen, structures and persistence. Clients project state and
item IDs, geometry, texture alpha behavior, map color, sounds, name and tab
order.

**Observability:**

Commands/state packets, block updates, shape/light/path probes, contact health,
AI state/events, drops, composters, furnaces, recipe book, offers/chests,
controlled feature/template traces, maps, sounds, tabs and rendering expose
every listed branch.

**Persistence and reload:**

Placed Cactus persists identity and age; carried Enderman state persists its
normalized age-zero block state. Stacks persist ordinary components. Loot,
recipe, advancement, tags, damage types, features, biomes, pools, processors
and templates are reload-selected. Registration, random-tick control flow,
Composter row, trades, creative order and exact consumers remain code-built.
Reload does not retroactively mutate placed ages.

**Evidence:**

`OFF-SERVER-001`, `OFF-CLIENT-001`, `OFF-REPORT-001`, `OFF-DATA-001`;
`net.minecraft.world.level.block.Blocks`;
`net.minecraft.world.level.block.CactusBlock`;
`net.minecraft.world.level.block.CactusBlock#randomTick`;
`net.minecraft.world.level.block.CactusBlock#canSurvive`;
`net.minecraft.world.level.block.CactusBlock#updateShape`;
`net.minecraft.world.level.block.CactusBlock#entityInside`;
`net.minecraft.world.level.block.CactusFlowerBlock#mayPlaceOn`;
`net.minecraft.world.level.pathfinder.WalkNodeEvaluator#getPathTypeFromState`;
`net.minecraft.world.entity.EntityType#isBlockDangerous`;
`net.minecraft.util.SpawnUtil$Strategy`;
`net.minecraft.world.entity.monster.EnderMan$EndermanTakeBlockGoal`;
`net.minecraft.world.entity.monster.EnderMan$EndermanLeaveBlockGoal`;
`net.minecraft.world.entity.animal.camel.Camel`;
`net.minecraft.world.level.block.ComposterBlock#bootStrap`;
`net.minecraft.world.level.block.FireBlock#bootStrap`;
`net.minecraft.world.level.block.entity.FuelValues#vanillaBurnTimes`;
`net.minecraft.world.entity.npc.VillagerTrades`;
`net.minecraft.world.level.levelgen.feature.BlockColumnFeature#place`;
`net.minecraft.world.item.CreativeModeTabs#bootstrap`;
block/item/sound reports and item components; direct/composed tags; Cactus loot,
Green Dye recipe/advancement, Desert Village chest, trade, configured/placed
features, biomes, Village/Trial pools and zombie processor; all 1,212 templates;
exact blockstate/model/item/texture-metadata/name resources. Complete compiled
exact-field and data-reference searches found no other runtime path.

**Test vectors:**

Run `EXP-BLK-108` across all ages, heights and supports; every update/write/
damage/AI/feeding/loot/compost/cooking/trade/chest branch; all three feature
profiles with controlled draws/obstructions/writes; all pool and eight raw
template cells; persistence, reload, sounds and exact client projection. Assert
exact read/draw/write ordering, constants, absence claims and vanilla
convergence.

**Limits:**

Generic random-tick scheduling, state writes, breaking/loot/explosion, damage,
AI goals, feeding, composting, cooking/knowledge, trade/chest selection,
block-column placement, jigsaw/template placement, packet encoding and
rendering remain with their named owners. Cactus Flower, Potted Cactus, Sand,
Farmland and other consumers retain their existing leaves. This rule fixes the
exact Cactus identity, its hooks, all ID-specific joins, locked data and
projection.
