# Blocks mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `BLK-STONE-VARIANT-001` — Granite, diorite and andesite join processing, trades, replacement and world generation

**Parent:** `BLK-001`, `BLK-002`, `BLK-003`, `BLK-005`, `PLY-005`, `PLY-006`, `ITM-004`,
`ITM-006`, `ENT-001`, `ENV-003`, `WGEN-003`, `CLI-006`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — the six locked registrations, reports, recipes, advancements, trades, loot,
tags, feature records, code selectors, decoded templates and client assets exhaust their
identities, states, transitions and observable projections.

**Applies when:**

`minecraft:granite`, `minecraft:polished_granite`, `minecraft:diorite`,
`minecraft:polished_diorite`, `minecraft:andesite` or `minecraft:polished_andesite` is placed,
mined, exploded, crafted, cut, traded, selected by a tag, equipped by a sulfur cube, generated,
serialized or rendered.

**Authoritative state:**

All six identities are property-free ordinary `Block` instances with no block entity and exactly
one state:

| Identity | State | Map color | Destroy speed | Explosion resistance |
|---|---:|---|---:|---:|
| granite | 2 | `DIRT` | 1.5 | 6.0 |
| polished granite | 3 | `DIRT` | 1.5 | 6.0 |
| diorite | 4 | `QUARTZ` | 1.5 | 6.0 |
| polished diorite | 5 | `QUARTZ` | 1.5 | 6.0 |
| andesite | 6 | `STONE` | 1.5 | 6.0 |
| polished andesite | 7 | `STONE` | 1.5 | 6.0 |

Every registration selects bass-drum note instrument, `requiresCorrectToolForDrops` and
two-argument strength `1.5/6.0`. Every state retains stone sound; full selection, collision, visual
and occlusion cubes; emission `0`; friction `0.6`; speed/jump factors `1`; restitution `0`; and
piston reaction `NORMAL`. The solid-render cube does not propagate skylight, has light dampening
`15` and shade brightness `0.2`, and is redstone conducting, suffocating and view blocking. Its
sturdy top passes the default spawn-support predicate because emission is below `14`;
entity-specific spawn admission remains independent.

Each identity has an ordinary common-rarity `BlockItem`, maximum stack size `64`, matching name
and item model, and no special use component.

**Transition and ordering:**

#### Placement, transforms and breaking

Placement selects the identity's only state; rotation and mirror retain it. A legal
`minecraft:block_state` component cannot add a property because the state definition is empty.
The blocks add no fluid, gravity, random/scheduled tick, shape-update, use, attack, entity-contact,
neighbor, redstone-output, comparator or block-event callback.

Correct-tool admission remains with generic player breaking. Each one-roll block loot table offers
the matching item behind `survives_explosion` and uses random sequence
`minecraft:blocks/<identity>`. An admitted ordinary player break returns the matching item; an
incorrect player tool returns nothing. Explosion context may independently suppress the offer.

#### Recipes and advancements

The reloadable full-block recipe graph is exact:

- two cobblestone and two quartz in the `CQ/QC` pattern produce two diorite;
- one diorite and one quartz shapelessly produce one granite;
- one diorite and one cobblestone shapelessly produce two andesite;
- four matching raw blocks in a 2-by-2 pattern produce four polished blocks;
- stonecutting one matching raw block produces one polished block.

The exact joins into separately owned shape outputs are:

- three matching raw or polished blocks in one row produce six matching slabs;
- six matching raw or polished blocks in the stair pattern produce four matching stairs;
- six matching raw blocks in two rows produce six matching walls;
- stonecutting one raw block produces two raw slabs, one raw stair or one raw wall;
- stonecutting one raw or one polished block produces two polished slabs or one polished stair.

This leaf owns the six full-block identities and their input/output values. Granite, diorite and
andesite stairs, slabs and walls retain `shape-family` state and shape behavior. Matching,
allocation, stonecutter menus and result publication remain generic.

Diorite and granite recipes unlock from possessing quartz; the andesite recipe unlocks from
possessing diorite. Each polished recipe and its raw-input stonecutting record unlock from
possessing the matching raw block. Raw shape records unlock from the matching raw block. Polished
shape crafting and polished-input stonecutting unlock from the matching polished block, while their
raw-input stonecutting alternatives unlock from the matching raw block. Every recipe advancement
also admits its own `recipe_unlocked` criterion.

#### Mason trades

The level-three mason tag contains seven ordered records, and its trade set selects amount `2`.
Six candidates join this family:

- each of the three raw identities has a `16` blocks for one emerald offer with maximum uses `16`,
  villager XP `20` and reputation discount `0.05`;
- each polished identity has a one emerald for four blocks offer with maximum uses `16`, villager
  XP `10` and reputation discount `0.05`.

The remaining candidate sells four dripstone blocks for one emerald. Candidate resolution, random
selection, exhaustion, demand, price calculation and publication remain with the generic trade
owners.

#### Tags and their consumers

All six blocks are direct members of reloadable `mineable/pickaxe`. Only the three raw identities
are direct members of both `base_stone_overworld` and `stone_ore_replaceables`; polished variants
do not enter either tag.

The base-stone membership composes into `azalea_root_replaceable`, `bats_spawnable_on`,
`dripstone_replaceable_blocks`, `forest_rock_can_place_on`, `moss_replaceable`,
`nether_carver_replaceables`, `overworld_carver_replaceables` and `sculk_replaceable`. It can
therefore admit the raw identities as root/pointed-dripstone/moss replacement, forest-rock support,
carver/sculk replacement and bat-spawn support when each owning algorithm's other gates pass.
`stone_ore_replaceables` lets the ordinary stone-target branches of the locked ore records replace
the raw identities. Tag expansion and the owning feature, carver, sculk and spawning algorithms
remain independent.

All six items are direct members of `sulfur_cube_archetype/slow_bouncy`. Equipping or swallowing
one can select that archetype; its locked knockback pair is
`(horizontal, vertical)=(0.4125, 0.24)`, hit sound is
`minecraft:entity.sulfur_cube.slow_bouncy.hit`, and the remaining attribute/sound values come from
the same record. Multiple-match ordering, equipment changes and composed attributes remain with
the sulfur-cube owner.

#### Generation and structure joins

The three raw identities join five exact feature families and two code selectors:

- `ore_granite`, `ore_diorite` and `ore_andesite` each target `base_stone_overworld`, write their
  matching state with size `64` and discard-on-air-exposure chance `0`; each lower placed wrapper
  uses count `2` and uniform absolute Y `0..60`, while each upper wrapper uses rarity `6` and
  uniform absolute Y `64..128`;
- `glow_lichen` and `sculk_vein` admit all three as attachment supports in their respective
  ten-state and eight-state support lists, both with search range `20`;
- `spring_lava_overworld` and `spring_water` admit all three in their valid-block lists;
- the copper ore-vein material rule can return granite as its filler at inclusive Y `0..50`;
- buried treasure accepts each raw identity, among sandstone and stone, as its below-chest support
  gate.

Configured-feature, placement-modifier, ore-vein and buried-treasure traversal, RNG, write order and
failure behavior remain with their named worldgen owners.

Decoded locked templates contain exactly `1,797` live full-block cells across `42` inputs:

| Template family | Identity | Inputs with live cells | Cells |
|---|---|---:|---:|
| sulfur spring | granite | 10 | 1,210 |
| desert villages | granite | 1 | 7 |
| snowy villages | diorite | 4 | 104 |
| underwater ruins | polished granite | 22 | 193 |
| underwater ruins | polished diorite | 1 | 4 |
| woodland mansion | polished andesite | 3 | 274 |
| igloo | polished andesite | 1 | 1 |
| trial chambers | polished andesite | 1 | 4 |

The trial-chamber cells are in `chests/supply`; the igloo cell is in `bottom`; the mansion cells are
in `1x2_a3`, `2x2_a1` and `2x2_a4`. No locked template contains a live andesite full-block cell,
and no matching palette-only entry was counted. Feature/template selection, village/ocean-ruin/
trial-chamber pools and processors, igloo and mansion construction, transforms and block-write
ordering remain with their named owners; this leaf owns the resulting six identities and behavior.

**Client projection:**

Each property-free blockstate selects one matching `cube_all` model whose `all` texture is
`minecraft:block/<identity>`. Every item definition selects its matching block model directly.
There is no weighted, conditional, animated, rotated or special-renderer branch. Terrain and
block-update packets publish states `2..7`; ordinary full-cube face culling, breaking
particles/sounds, map shading and opaque rendering consume the selected state/model.

**Branches and aborts:**

Six identities; three map colors; raw versus polished tag/recipe/worldgen roles; correct versus
incorrect tool; surviving versus suppressed explosion loot; every craft/cut/unlock and mason
candidate/selection; direct versus composed tag membership; every feature/code/template selection;
and block versus item projection are distinct.

**Constants and randomness:**

States `2..7`; strength `1.5/6.0`; emission `0`; dampening `15`; shade `0.2`; friction `0.6`;
speed/jump `1`; restitution `0`; stack `64`; recipe outputs `1/2/4/6`; level-three mason selection
`2/7`, uses `16`, raw purchase `16:1/20 XP`, polished sale `1:4/10 XP`, discount `0.05`; stone
variant ore size/discard `64/0`, lower count/Y `2/0..60`, upper rarity/Y `6/64..128`; attachment
search `20`; template cells `1,797`. These blocks consume no RNG directly; generic explosion,
trade, feature, structure and template owners retain their randomness.

**Side effects:**

Generic placement/removal; conditional same-identity loot; recipe, advancement, stonecutting and
mason outputs; tag-selected replacement/support/spawn/archetype behavior; feature, ore-vein,
code-built structure and template writes; ordinary persistence and opaque block/item projection.

**Gates:**

Selected identity; placement/write authority; correct-tool harvest admission; explosion context;
active recipe, advancement, loot, trade, tag and archetype snapshots; feature/structure/template
selection; other replacement, support and spawn predicates; client state/model context.

**Boundary cases and quirks:**

Polishing changes neither strength nor map color. Only raw identities are base stone and ore
replaceable, while all six items are slow-bouncy. The three stone-variant ore features may replace
their own output family because every raw identity is in `base_stone_overworld`. Bat support is the
block below the candidate spawn, after the bat rule's height, random and brightness gates.
Template palette entries that no live cell indexes are not counted. Shape-family stairs, slabs and
walls join recipes without becoming members of this six-ID family.

**Evidence:**

`OFF-SERVER-001`, `OFF-CLIENT-001`, `OFF-REPORT-001`, `OFF-DATA-001`;
`net.minecraft.world.level.block.Blocks`,
`net.minecraft.world.level.block.state.BlockBehaviour$Properties#strength`,
`net.minecraft.world.item.trading.TradeSet#calculateNumberOfTrades`,
`net.minecraft.world.item.trading.VillagerTrade#getOffer`,
`net.minecraft.world.entity.ambient.Bat#checkBatSpawnRules`,
`net.minecraft.world.entity.monster.cubemob.SulfurCube#matchingArchetypes`,
`net.minecraft.world.level.levelgen.OreVeinifier`;
`reports/blocks.json#minecraft:{granite,polished_granite,diorite,polished_diorite,andesite,polished_andesite}`,
`reports/minecraft/components/item/{granite,polished_granite,diorite,polished_diorite,andesite,polished_andesite}.json`,
`data/minecraft/tags/block/{base_stone_overworld,stone_ore_replaceables,mineable/pickaxe,azalea_root_replaceable,bats_spawnable_on,dripstone_replaceable_blocks,forest_rock_can_place_on,moss_replaceable,nether_carver_replaceables,overworld_carver_replaceables,sculk_replaceable}.json`,
`data/minecraft/tags/item/sulfur_cube_archetype/slow_bouncy.json`,
`data/minecraft/sulfur_cube_archetype/slow_bouncy.json`,
`data/minecraft/loot_table/blocks/{granite,polished_granite,diorite,polished_diorite,andesite,polished_andesite}.json`,
`data/minecraft/recipe/*{granite,diorite,andesite}*.json`,
`data/minecraft/advancement/recipes/{building_blocks,decorations}/*{granite,diorite,andesite}*.json`,
`data/minecraft/villager_trade/mason/3/*{granite,diorite,andesite}*.json`,
`data/minecraft/tags/villager_trade/mason/level_3.json`,
`data/minecraft/trade_set/mason/level_3.json`,
`data/minecraft/worldgen/configured_feature/{ore_granite,ore_diorite,ore_andesite,glow_lichen,sculk_vein,spring_lava_overworld,spring_water}.json`,
`data/minecraft/worldgen/placed_feature/ore_{granite,diorite,andesite}_{lower,upper}.json`,
`data/minecraft/structure/{spring,underwater_ruin,village,woodland_mansion,igloo,trial_chambers}/**/*.nbt`,
`assets/minecraft/blockstates/{granite,polished_granite,diorite,polished_diorite,andesite,polished_andesite}.json`,
`assets/minecraft/models/block/{granite,polished_granite,diorite,polished_diorite,andesite,polished_andesite}.json`,
`assets/minecraft/items/{granite,polished_granite,diorite,polished_diorite,andesite,polished_andesite}.json`.

**Test vectors:**

Run `EXP-BLK-046` across all six states, ordinary/component/template placement, transforms,
correct/incorrect tools, explosions, every full-block and shape-joining recipe/unlock, all six
mason records, direct/composed tags and the sulfur archetype, every named feature/code-built join,
all 42 containing templates, save/reload and every block/item model. Assert identity, strength and
physical predicates, writes/drops, recipe/unlock/trade values, tag/replacement/support behavior,
template counts and selected model/texture.

**Limits:**

This leaf does not re-specify generic placement/break packets, state-component parsing, tool speed,
explosion survival, recipe/stonecutter/trade allocation, sulfur-cube composition, feature/carver/
sculk/spawn algorithms, ore-vein resolution, structure/template placement or model loading. Those
remain with `BLK-002`, `BLK-STATE-001`, `PLY-006`, `ITM-LOOT-001`, recipe/stonecutter/trade owners,
`ENT-KNOCKBACK-001`, `MOB-SPAWN-001`, `WGEN-PIPELINE-001`, the named structure and jigsaw leaves and
`CLI-006`. Granite, diorite and andesite stairs, slabs and walls remain under `shape-family`.
