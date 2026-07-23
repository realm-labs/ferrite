# Blocks mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `BLK-QUARTZ-001` — Full quartz blocks join axis placement, processing, mason offers and bastion decoration

**Parent:** `BLK-001`, `BLK-002`, `BLK-003`, `BLK-005`, `PLY-005`, `PLY-006`, `ITM-004`,
`ITM-006`, `ENT-001`, `ENV-003`, `WGEN-003`, `CLI-006`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — the five locked registrations, rotated-pillar implementation, reports,
recipes, advancements, loot and trade records, tags, bastion template and client assets exhaust
their identities, seven states, transitions and observable projections.

**Applies when:**

`minecraft:quartz_block`, `minecraft:chiseled_quartz_block`, `minecraft:quartz_pillar`,
`minecraft:quartz_bricks` or `minecraft:smooth_quartz` is placed, transformed, mined, exploded,
crafted, cut, smelted, traded, selected by a tag, equipped by a sulfur cube, written by the
containing bastion template, serialized or rendered.

**Authoritative state:**

The five identities have no block entity. Four are ordinary property-free `Block` instances;
`quartz_pillar` is a `RotatedPillarBlock` with the three-valued `axis` property:

| Identity | State(s) | Default | Destroy speed | Explosion resistance |
|---|---|---|---:|---:|
| quartz block | 11323 | only state | 0.8 | 0.8 |
| chiseled quartz block | 11324 | only state | 0.8 | 0.8 |
| quartz pillar | 11325 `x`, 11326 `y`, 11327 `z` | `y` | 0.8 | 0.8 |
| quartz bricks | 23095 | only state | 0.8 | 0.8 |
| smooth quartz | 13482 | only state | 2.0 | 6.0 |

Quartz block, chiseled quartz and quartz pillar each register map color `QUARTZ`, bass-drum note
instrument, `requiresCorrectToolForDrops` and strength `0.8`. Quartz bricks copies quartz block's
legacy properties exactly. Smooth quartz independently selects the same map color, instrument and
tool requirement but calls two-argument strength with `2.0/6.0`.

Every state retains ordinary stone sound; full selection, collision, visual and occlusion cubes;
emission `0`; friction `0.6`; speed/jump factors `1`; restitution `0`; and piston reaction
`NORMAL`. The full solid-render cube does not propagate skylight, has light dampening `15` and
shade brightness `0.2`, and is redstone conducting, suffocating and view blocking. Its sturdy top
passes the default spawn-support predicate because emission is below `14`; entity-specific
placement remains an independent gate.

Each identity has an ordinary common-rarity `BlockItem`, maximum stack size `64`, matching name and
item model, and no special use component.

**Transition and ordering:**

#### Pillar placement and transforms

`RotatedPillarBlock` installs `axis=y` as the default. Placement replaces it with the clicked
face's axis: either east/west face writes `x`, up/down writes `y`, and north/south writes `z`.
The selected axis depends on the face, not player yaw. The generic block-state component path may
replace that legal value after successful item placement.

A clockwise or counterclockwise quarter-turn rotation swaps `x` and `z` and retains `y`; no
rotation and 180 degrees retain every axis. The class does not override mirror, so the inherited
ordinary mirror path leaves the axis unchanged. Structure-template and command transforms operate
on the stored state rather than recomputing a placement face. The property has no neighbor-driven
correction.

The other four identities remain property-free under placement, rotation and mirror. All five add
no fluid, gravity, random/scheduled tick, shape-update, use, attack, entity-contact, neighbor,
redstone-output, comparator or block-event callback.

#### Breaking, recipes and advancements

Correct-tool admission remains with generic player breaking. Each one-roll block loot table offers
the matching item behind `survives_explosion` and uses random sequence
`minecraft:blocks/<identity>`. An admitted ordinary player break returns the matching item; an
incorrect player tool returns nothing. Explosion context may independently suppress the offer.

The reloadable recipe graph is exact:

- four quartz items in a 2-by-2 pattern produce one quartz block;
- two vertically stacked quartz slabs produce one chiseled quartz block;
- two vertically stacked quartz blocks produce two pillars;
- four quartz blocks produce four quartz bricks;
- stonecutting one quartz block produces one chiseled block, one pillar or one quartz brick;
- smelting one quartz block produces one smooth quartz, awards `0.1` experience and uses the
  serializer's omitted-field default of `200` ticks;
- three quartz blocks, chiseled blocks or pillars produce six quartz slabs, and the same three
  identities in the stair pattern produce four quartz stairs;
- stonecutting one quartz block produces two quartz slabs or one quartz stair;
- three smooth quartz produce six smooth slabs or four smooth stairs by their respective patterns,
  while stonecutting one smooth quartz produces two smooth slabs or one smooth stair.

This leaf owns the five full-block identities and their input/output values. Quartz and
smooth-quartz stairs/slabs retain their separate `shape-family` owner and shape/state behavior.
Matching, allocation, stonecutter menus, furnace progress and result publication remain generic.

The quartz-block recipe unlocks from possessing quartz or the recipe criterion. Quartz bricks,
smooth quartz and every quartz-block stonecutting record unlock from possessing quartz block or
the recipe criterion. The shaped chiseled-block and pillar records accept possession of chiseled
quartz, quartz block or quartz pillar, or their recipe criterion. Shape-output advancements retain
their shape-family ownership.

#### Trades, tags and generation

The two level-five mason records each want one emerald and give one quartz block or one quartz
pillar, with maximum uses `12`, villager XP `30` and reputation discount factor `0.05`. The
level-five tag contains exactly those two records. Its trade set requests `2` distinct candidates
with random sequence `minecraft:trade_set/mason/level_5`; consequently both records are selected,
although generic selection and offer construction retain ordering, price and restock behavior.

All five blocks are direct members of reloadable `mineable/pickaxe`. All five items are direct
members of `sulfur_cube_archetype/slow_bouncy`. Equipping or swallowing one can therefore select
that archetype; its locked knockback pair is `(horizontal, vertical)=(0.4125, 0.24)` and hit sound
is `minecraft:entity.sulfur_cube.slow_bouncy.hit`. Multiple-match ordering, equipment changes and
composed attributes remain with the sulfur-cube owner.

`bastion/bridge/bridge_pieces/bridge` is the only locked structure template containing these full
blocks. Its single palette contains four quartz blocks and two smooth quartz, plus two
smooth-quartz slabs owned by `shape-family`. The rigid, weight-one pool element uses the
`minecraft:bridge` processor list. Bastion pool selection, transforms, processor traversal and
block writes remain with `WGEN-JIGSAW-BASTION-001`.

**Client projection:**

The four property-free blockstates each choose one matching model. Quartz block and chiseled quartz
inherit `cube_column` with separate side/end textures; quartz bricks inherits `cube_all` with its
brick texture; smooth quartz inherits `cube_all` using `quartz_block_bottom` on every face.

Quartz pillar's blockstate selects:

| Axis | Model | X rotation | Y rotation |
|---|---|---:|---:|
| x | `quartz_pillar_horizontal` | 90 | 90 |
| y | `quartz_pillar` | 0 | 0 |
| z | `quartz_pillar_horizontal` | 90 | 0 |

The vertical model inherits `cube_column`; the horizontal model inherits
`cube_column_horizontal`; both use `quartz_pillar_side` and `quartz_pillar_top`. Every item
definition selects the matching unrotated block model directly, so the pillar item uses the
vertical presentation. There is no weighted, conditional, animated or special-renderer branch.

Terrain and block-update packets publish states `11323..11327`, `13482` and `23095`. Ordinary
full-cube face culling, breaking particles/sounds, map shading and opaque rendering consume the
selected state and model.

**Branches and aborts:**

Five identities; four property-free states versus three pillar axes; each clicked-face axis,
rotation and mirror; direct versus component/template state writes; correct versus incorrect tool;
surviving versus suppressed explosion loot; each craft/cut/smelt and advancement success or
rejection; both guaranteed mason candidates; pickaxe versus item-archetype tags; selected versus
unselected bastion element; and block versus item model projection are distinct.

**Constants and randomness:**

States `11323`, `11324`, `11325..11327`, `13482`, `23095`; pillar default `y`; ordinary-family
strength `0.8/0.8`; smooth strength `2.0/6.0`; emission `0`; dampening `15`; shade `0.2`; friction
`0.6`; speed/jump `1`; restitution `0`; stack `64`; cooking time/XP/output `200/0.1/1`; trade
cost/output/max uses/XP/discount `1/1/12/30/0.05`; trade-set amount/candidates `2/2`; bastion
counts `4/2` quartz/smooth quartz. These blocks consume no RNG directly; generic explosion, trade
and bastion owners retain their randomness.

**Side effects:**

Generic placement/removal and optional pillar-axis publication; conditional same-identity loot;
recipe, advancement and furnace outputs; two mason offers; tag-selected sulfur-cube composition;
bastion palette writes; ordinary persistence and opaque block/item projection.

**Gates:**

Selected identity and legal axis; placement face or transform authority; correct-tool harvest
admission; explosion context; active recipe, advancement, loot, trade, tag and archetype snapshots;
villager profession/level; sulfur-cube body equipment; bastion pool/template selection; client
state/model context.

**Boundary cases and quirks:**

Smooth quartz is substantially harder and more blast resistant than the other four despite sharing
their map color, sound, instrument and tool requirement. Pillar placement follows the clicked
face's axis, while its item model always shows the vertical model. Level-five masons have exactly
two candidates and request two distinct offers, so both block identities survive selection. The
slow-bouncy join is an item tag, not a block tag. The shape-family stairs and slabs participate in
the recipe and bastion joins without becoming members of this five-ID family.

**Evidence:**

`OFF-SERVER-001`, `OFF-CLIENT-001`, `OFF-REPORT-001`, `OFF-DATA-001`;
`net.minecraft.world.level.block.Blocks`,
`net.minecraft.world.level.block.RotatedPillarBlock#RotatedPillarBlock`,
`net.minecraft.world.level.block.RotatedPillarBlock#createBlockStateDefinition`,
`net.minecraft.world.level.block.RotatedPillarBlock#getStateForPlacement`,
`net.minecraft.world.level.block.RotatedPillarBlock#rotate`,
`net.minecraft.world.level.block.RotatedPillarBlock#rotatePillar`,
`net.minecraft.world.level.block.state.BlockBehaviour$Properties#strength`,
`net.minecraft.world.item.crafting.AbstractCookingRecipe#cookingMapCodec`,
`net.minecraft.world.item.trading.TradeSet#calculateNumberOfTrades`,
`net.minecraft.world.item.trading.VillagerTrade#getOffer`,
`net.minecraft.world.entity.monster.cubemob.SulfurCube#matchingArchetypes`;
`reports/blocks.json#minecraft:{quartz_block,chiseled_quartz_block,quartz_pillar,quartz_bricks,smooth_quartz}`,
`reports/minecraft/components/item/{quartz_block,chiseled_quartz_block,quartz_pillar,quartz_bricks,smooth_quartz}.json`,
`data/minecraft/tags/block/mineable/pickaxe.json`,
`data/minecraft/tags/item/sulfur_cube_archetype/slow_bouncy.json`,
`data/minecraft/sulfur_cube_archetype/slow_bouncy.json`,
`data/minecraft/loot_table/blocks/{quartz_block,chiseled_quartz_block,quartz_pillar,quartz_bricks,smooth_quartz}.json`,
`data/minecraft/recipe/{quartz_block,chiseled_quartz_block,chiseled_quartz_block_from_quartz_block_stonecutting,quartz_pillar,quartz_pillar_from_quartz_block_stonecutting,quartz_bricks,quartz_bricks_from_quartz_block_stonecutting,smooth_quartz}.json`,
`data/minecraft/recipe/{quartz_slab,quartz_slab_from_quartz_block_stonecutting,quartz_stairs,quartz_stairs_from_quartz_block_stonecutting,smooth_quartz_slab,smooth_quartz_slab_from_smooth_quartz_stonecutting,smooth_quartz_stairs,smooth_quartz_stairs_from_smooth_quartz_stonecutting}.json`,
`data/minecraft/advancement/recipes/building_blocks/{quartz_block,chiseled_quartz_block,chiseled_quartz_block_from_quartz_block_stonecutting,quartz_pillar,quartz_pillar_from_quartz_block_stonecutting,quartz_bricks,quartz_bricks_from_quartz_block_stonecutting,smooth_quartz}.json`,
`data/minecraft/villager_trade/mason/5/{emerald_quartz_block,emerald_quartz_pillar}.json`,
`data/minecraft/tags/villager_trade/mason/level_5.json`,
`data/minecraft/trade_set/mason/level_5.json`,
`data/minecraft/worldgen/template_pool/bastion/bridge/bridge_pieces.json`,
`data/minecraft/worldgen/processor_list/bridge.json`,
`data/minecraft/structure/bastion/bridge/bridge_pieces/bridge.nbt`,
`assets/minecraft/blockstates/{quartz_block,chiseled_quartz_block,quartz_pillar,quartz_bricks,smooth_quartz}.json`,
`assets/minecraft/models/block/{quartz_block,chiseled_quartz_block,quartz_pillar,quartz_pillar_horizontal,quartz_bricks,smooth_quartz}.json`,
`assets/minecraft/items/{quartz_block,chiseled_quartz_block,quartz_pillar,quartz_bricks,smooth_quartz}.json`.

**Test vectors:**

Run `EXP-BLK-044` across all seven states, clicked faces, explicit writes, rotations/mirrors,
correct/incorrect tools, explosions, every full-block and shape-joining recipe/unlock, both mason
records, tag/archetype reload, the containing bastion template, save/reload and every block/item
model. Assert identity/state/axis, strength and physical predicates, writes/drops, recipe/trade
values, template counts/transforms and selected models/rotations.

**Limits:**

This leaf does not re-specify generic placement/break packets, state-component parsing, tool speed,
explosion survival, recipe/stonecutter/furnace allocation, villager lifecycle, sulfur-cube
composition, bastion jigsaw placement or model loading. Those remain with `BLK-002`,
`BLK-STATE-001`, `PLY-006`, `ITM-LOOT-001`, recipe/furnace owners, villager owners,
`ENT-KNOCKBACK-001`, `WGEN-JIGSAW-BASTION-001` and `CLI-006`. Quartz/smooth-quartz stairs and slabs
remain under `shape-family`.
