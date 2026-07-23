# Blocks mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `BLK-STONE-BRICK-001` — Full stone bricks join processing, infestation hosts, masonry loot and structures

**Parent:** `BLK-001`, `BLK-002`, `BLK-003`, `BLK-005`, `PLY-005`, `PLY-006`, `ITM-004`,
`ITM-006`, `ENT-001`, `ENV-003`, `WGEN-003`, `CLI-006`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — the four locked registrations, reports, recipes, advancements, trades, loot,
tags, code-built structures/processors, decoded templates and client assets exhaust their
identities, states, transitions and observable projections.

**Applies when:**

`minecraft:stone_bricks`, `minecraft:mossy_stone_bricks`,
`minecraft:cracked_stone_bricks` or `minecraft:chiseled_stone_bricks` is placed, transformed,
mined, exploded, crafted, cut, smelted, traded, selected by a tag, emitted by infested loot,
equipped by a sulfur cube, generated, serialized or rendered.

**Authoritative state:**

All four identities are property-free ordinary `Block` instances with no block entity and exactly
one state:

| Identity | State | Map color | Destroy speed | Explosion resistance |
|---|---:|---|---:|---:|
| stone bricks | 7754 | `STONE` | 1.5 | 6.0 |
| mossy stone bricks | 7755 | `STONE` | 1.5 | 6.0 |
| cracked stone bricks | 7756 | `STONE` | 1.5 | 6.0 |
| chiseled stone bricks | 7757 | `STONE` | 1.5 | 6.0 |

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

#### Full-block recipes and advancements

The reloadable full-block processing graph is exact:

- four stone in a 2-by-2 pattern produce four stone bricks, while stonecutting one stone produces
  one stone bricks;
- one stone bricks plus one vine, or one stone bricks plus one moss block, shapelessly produces one
  mossy stone bricks; both recipes share group `mossy_stone_bricks`;
- smelting one stone bricks produces one cracked stone bricks, awards `0.1` experience and uses the
  serializer's omitted-field default of `200` ticks;
- two vertically stacked stone-brick slabs produce one chiseled stone bricks, while stonecutting
  one stone bricks or one stone produces one chiseled stone bricks;
- eight chiseled stone bricks around one iron ingot produce one lodestone.

Stone-bricks crafting and stonecutting unlock from possessing stone. Mossy recipes unlock from
possessing their vine or moss-block ingredient. Cracked smelting and stone-bricks-input
stonecutting unlock from possessing stone bricks; the direct-stone chiseled record unlocks from
stone. The chiseled crafting advancement accepts any member of the item `stone_bricks` tag.
Lodestone unlock accepts its own recipe, an iron ingot or a lodestone. Every other recipe
advancement also admits its own `recipe_unlocked` criterion.

#### Shape-output joins

The exact joins into separately owned shape outputs are:

- three stone bricks in one row produce six stone-brick slabs; six in the stair pattern produce
  four stairs; six in two rows produce six walls;
- stonecutting one stone bricks produces two matching slabs, one stair or one wall; stone also has
  direct stonecutting alternatives with the same outputs;
- the corresponding mossy-block patterns and stonecutting records produce six/two mossy slabs,
  four/one mossy stairs and six/one mossy walls.

The stone-brick slab and stair crafting advancements accept any of the four full-block items through
the item `stone_bricks` tag even though their recipes require ordinary stone bricks; the wall
advancement requires ordinary stone bricks. Mossy shape records require mossy stone bricks.

This leaf owns the four full-block identities and their input/output values. Stone-brick and
mossy-stone-brick stairs, slabs and walls retain `shape-family` state and shape behavior. Matching,
allocation, stonecutter menus, furnace progress, lodestone behavior and result publication remain
generic or separately owned.

#### Trades, chest loot and infestation joins

The level-two mason tag contains exactly two records and its trade set selects amount `2`, so both
offers are retained. The family record exchanges one emerald for four chiseled stone bricks with
maximum uses `16`, villager XP `5` and reputation discount `0.05`; the other record buys stone.
Candidate resolution, offer pricing, exhaustion, restock and publication remain with the generic
trade owners.

The village-mason chest table performs uniform integer `1..5` rolls over total entry weight `13`.
Stone bricks has weight `2` and returns one item on each selection. The table's random sequence is
`minecraft:chests/village/village_mason`; generic weighted selection and container filling remain
with the loot owner.

Four separately cataloged `InfestedBlock` registrations use these four identities as their exact
hosts. Host-to-infested and infested-to-host conversion copies only mutually supported properties;
all eight states are property-free, so each conversion selects the matching default state. Each
matching infested loot table offers the ordinary host item only to a tool whose Silk Touch level is
at least `1`, with its own `minecraft:blocks/infested_<identity>` random sequence. Infested block
breaking, silverfish spawning and the infested identities themselves remain outside this four-ID
family.

#### Tags and their consumers

All four blocks are direct members of both `mineable/pickaxe` and block `stone_bricks`; all four
items are direct members of item `stone_bricks` and
`sulfur_cube_archetype/slow_bouncy`. The item tag drives the three crafting-advancement criteria
described above. No locked dedicated gameplay class consumes the block tag beyond generic holder/
tag tests; it remains a reloadable observable membership rather than evidence of a hidden common
callback.

Equipping or swallowing any of the four items can select the slow-bouncy archetype; its locked
knockback pair is `(horizontal, vertical)=(0.4125, 0.24)`, hit sound is
`minecraft:entity.sulfur_cube.slow_bouncy.hit`, and the remaining attribute/sound values come from
the same record. Multiple-match ordering, equipment changes and composed attributes remain with
the sulfur-cube owner.

#### Code-built generation and processors

Stronghold shell selection consumes one float for each admitted edge cell and chooses cracked stone
bricks below `0.2`, mossy stone bricks below `0.5`, infested stone bricks below `0.55`, otherwise
ordinary stone bricks. Interior cells become cave air without a draw. The stronghold pieces also
perform their separately specified fixed stone-brick, stair/slab, furnishing and opening writes.
Only the ordinary infested variant enters this selector; it is not one of this leaf's four
identities.

The jungle-temple hidden room writes exactly three chiseled stone bricks at local
`(8..10,-2,11)`. Its masonry selector and remaining fixed cells use cobblestone families and stay
outside this leaf.

Every ruined-portal piece runs block aging before optional Nether blackstone replacement. Stone,
stone bricks and chiseled stone bricks first pass the strict `<0.5` full-block gate; admitted
processing can select cracked stone bricks or mossy stone bricks, or a separately owned stair,
according to the eager facing/half draws, mossiness comparison and final two-way draw. A Nether
portal then maps ordinary/mossy stone bricks to polished-blackstone bricks and maps chiseled/cracked
stone bricks to the corresponding polished-blackstone identity. Position-seeded RNG, protected/
lava processors, ordering and template writes remain with `WGEN-STRUCTURE-RUINED-PORTAL-001` and
`WGEN-JIGSAW-PROCESSORS-001`.

#### Template generation

Decoded locked templates contain exactly `4,060` live full-block cells across `62` inputs:

| Template family | Identity | Inputs with live cells | Cells |
|---|---|---:|---:|
| igloo | stone bricks | 2 | 116 |
| igloo | mossy stone bricks | 1 | 17 |
| igloo | cracked stone bricks | 1 | 8 |
| igloo | chiseled stone bricks | 1 | 7 |
| ruined portal | stone bricks | 11 | 319 |
| ruined portal | mossy stone bricks | 2 | 3 |
| ruined portal | cracked stone bricks | 2 | 2 |
| ruined portal | chiseled stone bricks | 7 | 25 |
| trail ruins | stone bricks | 9 | 88 |
| trail ruins | cracked stone bricks | 10 | 72 |
| trial chambers | stone bricks | 1 | 4 |
| underwater ruins | stone bricks | 13 | 1,007 |
| underwater ruins | mossy stone bricks | 12 | 1,001 |
| underwater ruins | cracked stone bricks | 12 | 1,004 |
| underwater ruins | chiseled stone bricks | 24 | 385 |
| snowy villages | stone bricks | 2 | 2 |

The trial-chamber cells are in `spawner/small_melee/silverfish`; the village cells are the ordinary
and zombie `snowy_meeting_point_2` variants. No matching palette-only entry was counted. Structure/
pool selection, integrity and processor traversal, transforms, clipping and write ordering remain
with the igloo, ruined-portal, trail-ruins, trial-chambers, ocean-ruin and village owners; this leaf
owns the resulting four full-block identities and behavior.

**Client projection:**

Each property-free blockstate selects one matching `cube_all` model whose `all` texture is
`minecraft:block/<identity>`. Every item definition selects its matching block model directly.
There is no weighted, conditional, animated, rotated or special-renderer branch. Terrain and
block-update packets publish states `7754..7757`; ordinary full-cube face culling, breaking
particles/sounds, map shading and opaque rendering consume the selected state/model.

**Branches and aborts:**

Four identities; correct versus incorrect tool; surviving versus suppressed explosion loot; every
craft/cut/smelt/unlock, guaranteed mason offer and chest-loot draw; ordinary versus infested host
mapping and Silk Touch gate; block/item/archetype tags; stronghold selector intervals; ruined-portal
age/blackstone outcomes; code-built versus template writes; and block versus item projection are
distinct.

**Constants and randomness:**

States `7754..7757`; strength `1.5/6.0`; emission `0`; dampening `15`; shade `0.2`; friction `0.6`;
speed/jump `1`; restitution `0`; stack `64`; cooking time/XP/output `200/0.1/1`; mason selection
`2/2`, output `4`, uses/XP/discount `16/5/0.05`; mason-chest rolls `1..5`, family weight/total
`2/13`; stronghold thresholds `0.2/0.5/0.55`; ruined-portal age gate `0.5`; template cells `4,060`.
These blocks consume no RNG directly; generic explosion, loot, trade, structure and processor
owners retain their randomness.

**Side effects:**

Generic placement/removal; conditional same-identity loot; recipe, advancement, stonecutting,
furnace, trade and chest-loot outputs; host/infested conversion and Silk Touch host drops;
tag-selected archetype and unlock behavior; code-built/processor/template writes; ordinary
persistence and opaque block/item projection.

**Gates:**

Selected identity; placement/write authority; correct-tool harvest admission; explosion and
Silk Touch contexts; active recipe, advancement, loot, trade, tag and archetype snapshots;
structure/template/processor selection; client state/model context.

**Boundary cases and quirks:**

All four visual variants have identical physical registration properties. The item tag broadens
three unlock criteria without broadening their actual recipes. The block tag has no dedicated
locked consumer. Stronghold's `0.5..0.55` interval writes infested, not mossy/cracked/chiseled
ordinary blocks. Block aging can create this family from source stone or replace its cells, while
Nether blackstone processing consumes every family identity. Template palette entries that no live
cell indexes are not counted. Shape-family stairs, slabs and walls join processing without becoming
members of this four-ID family.

**Evidence:**

`OFF-SERVER-001`, `OFF-CLIENT-001`, `OFF-REPORT-001`, `OFF-DATA-001`;
`net.minecraft.world.level.block.Blocks`,
`net.minecraft.world.level.block.InfestedBlock#infestedStateByHost`,
`net.minecraft.world.level.block.InfestedBlock#hostStateByInfested`,
`net.minecraft.world.item.crafting.AbstractCookingRecipe#cookingMapCodec`,
`net.minecraft.world.item.trading.TradeSet#calculateNumberOfTrades`,
`net.minecraft.world.item.trading.VillagerTrade#getOffer`,
`net.minecraft.world.level.levelgen.structure.structures.StrongholdPieces$SmoothStoneSelector#next`,
`net.minecraft.world.level.levelgen.structure.structures.JungleTemplePiece#postProcess`,
`net.minecraft.world.level.levelgen.structure.templatesystem.BlockAgeProcessor#processBlock`,
`net.minecraft.world.level.levelgen.structure.templatesystem.BlackstoneReplaceProcessor#processBlock`,
`net.minecraft.world.entity.monster.cubemob.SulfurCube#matchingArchetypes`;
`reports/blocks.json#minecraft:{stone_bricks,mossy_stone_bricks,cracked_stone_bricks,chiseled_stone_bricks}`,
`reports/minecraft/components/item/{stone_bricks,mossy_stone_bricks,cracked_stone_bricks,chiseled_stone_bricks}.json`,
`data/minecraft/loot_table/blocks/{stone_bricks,mossy_stone_bricks,cracked_stone_bricks,chiseled_stone_bricks,infested_stone_bricks,infested_mossy_stone_bricks,infested_cracked_stone_bricks,infested_chiseled_stone_bricks}.json`,
`data/minecraft/loot_table/chests/village/village_mason.json`,
`data/minecraft/recipe/*stone_brick*.json`,
`data/minecraft/recipe/lodestone.json`,
`data/minecraft/advancement/recipes/{building_blocks,decorations}/*stone_brick*.json`,
`data/minecraft/advancement/recipes/decorations/lodestone.json`,
`data/minecraft/villager_trade/mason/2/emerald_chiseled_stone_bricks.json`,
`data/minecraft/tags/villager_trade/mason/level_2.json`,
`data/minecraft/trade_set/mason/level_2.json`,
`data/minecraft/tags/block/{mineable/pickaxe,stone_bricks}.json`,
`data/minecraft/tags/item/{stone_bricks,sulfur_cube_archetype/slow_bouncy}.json`,
`data/minecraft/sulfur_cube_archetype/slow_bouncy.json`,
`data/minecraft/structure/{igloo,ruined_portal,trail_ruins,trial_chambers,underwater_ruin,village}/**/*.nbt`,
`assets/minecraft/blockstates/{stone_bricks,mossy_stone_bricks,cracked_stone_bricks,chiseled_stone_bricks}.json`,
`assets/minecraft/models/block/{stone_bricks,mossy_stone_bricks,cracked_stone_bricks,chiseled_stone_bricks}.json`,
`assets/minecraft/items/{stone_bricks,mossy_stone_bricks,cracked_stone_bricks,chiseled_stone_bricks}.json`.

**Test vectors:**

Run `EXP-BLK-047` across all four states, ordinary/component/template placement, transforms,
correct/incorrect tools, explosions, every full-block and shape-joining recipe/unlock, lodestone,
the mason trade/table, direct tags and sulfur archetype, matching infested hosts/Silk Touch loot,
stronghold and jungle-temple code paths, ruined-portal processors, all 62 containing templates,
save/reload and every block/item model. Assert identity, physical predicates, writes/drops,
recipe/unlock/trade/loot values, tag/host behavior, processor outcomes, template counts and selected
model/texture.

**Limits:**

This leaf does not re-specify generic placement/break packets, state-component parsing, tool speed,
explosion/Silk Touch conditions, recipe/stonecutter/furnace/trade/loot allocation, infested-block
callbacks, sulfur-cube composition, stronghold/temple algorithms, structure/template placement,
processor traversal or model loading. Those remain with `BLK-002`, `BLK-STATE-001`, `PLY-006`,
`ITM-LOOT-001`, recipe/stonecutter/furnace/trade owners, `BLK-BREAK-HOOK-001`,
`ENT-KNOCKBACK-001`, the named worldgen leaves and `CLI-006`. Stone-brick stairs, slabs and walls
remain under `shape-family`; the four infested identities remain separately cataloged.
