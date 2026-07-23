# Blocks mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `BLK-GLAZED-TERRACOTTA-001` — Glazed terracotta couples horizontal pattern orientation to push-only piston mobility

**Parent:** `BLK-001`, `BLK-002`, `BLK-003`, `BLK-004`, `BLK-005`, `RED-001`, `RED-004`,
`PLY-005`, `PLY-006`, `ITM-004`, `ITM-006`, `ENT-001`, `ENV-003`, `WGEN-003`, `CLI-006`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — the locked color-collection registration, horizontal-direction class, piston
resolver, reports, recipes, advancements, loot/trade records, tags, structure templates and client
assets exhaust all sixteen identities, states, transitions and observable projections.

**Applies when:**

Any `minecraft:<color>_glazed_terracotta` identity is placed, rotated, mirrored, moved by a piston,
mined, exploded, smelted, offered by a mason, selected from a tag, swallowed or equipped by a
sulfur cube, written by a structure template, serialized, mapped or rendered.

**Authoritative state:**

`Blocks.GLAZED_TERRACOTTA` registers one `GlazedTerracottaBlock` for each `DyeColor`. Every
identity has the horizontal `facing` property, no block entity and four states in fixed
`north,south,west,east` order. North is the default:

| Color | North | South | West | East |
|---|---:|---:|---:|---:|
| white | 14966 | 14967 | 14968 | 14969 |
| orange | 14970 | 14971 | 14972 | 14973 |
| magenta | 14974 | 14975 | 14976 | 14977 |
| light blue | 14978 | 14979 | 14980 | 14981 |
| yellow | 14982 | 14983 | 14984 | 14985 |
| lime | 14986 | 14987 | 14988 | 14989 |
| pink | 14990 | 14991 | 14992 | 14993 |
| gray | 14994 | 14995 | 14996 | 14997 |
| light gray | 14998 | 14999 | 15000 | 15001 |
| cyan | 15002 | 15003 | 15004 | 15005 |
| purple | 15006 | 15007 | 15008 | 15009 |
| blue | 15010 | 15011 | 15012 | 15013 |
| brown | 15014 | 15015 | 15016 | 15017 |
| green | 15018 | 15019 | 15020 | 15021 |
| red | 15022 | 15023 | 15024 | 15025 |
| black | 15026 | 15027 | 15028 | 15029 |

Registration selects the matching dye's ordinary `getMapColor`, bass-drum note instrument,
`requiresCorrectToolForDrops`, destroy speed and explosion resistance `1.4`, and piston reaction
`PUSH_ONLY`. The map color is not the matching non-glazed terracotta's terracotta-specific color.

All four facing states retain ordinary full-cube selection, collision, visual and occlusion
shapes. Stone sound, emission `0`, friction `0.6`, speed/jump factors `1`, restitution `0`, light
dampening `15` and shade brightness `0.2` remain ordinary defaults. The block is redstone
conducting, suffocating and view blocking. Its sturdy top passes the default spawn-support
predicate because emission is below `14`; entity-specific placement rules remain a separate gate.

Each identity has an ordinary common-rarity `BlockItem` with maximum stack size `64`, matching
item name/model and no special use component. The generic block-state component path may replace a
successfully placed horizontal `facing` value under `BLK-STATE-001`.

**Transition and ordering:**

#### Placement and transform orientation

`getStateForPlacement` reads the context's horizontal direction and writes its opposite. For an
ordinary player placement, a player facing north therefore places `facing=south`; the clicked face
and view pitch do not choose among horizontal values. A context without a player reports north and
therefore produces south. Failed generic placement commits no state.

The inherited horizontal transform owns non-item placement:

- rotation applies the selected `Rotation` to the current facing;
- mirror obtains its facing-relative rotation and delegates to the same state rotation;
- structure-template placement therefore preserves or transforms the palette-facing value through
  the generic template owner rather than recomputing it from a player.

Direct command/component writes may select any of the four legal values. The property has no
post-placement neighbor correction.

#### Push-only piston boundary

The registration's `PUSH_ONLY` reaction is interpreted by `PistonBaseBlock#isPushable`: admission
is true only when the requested movement direction equals the piston-direction argument supplied
for that resolver edge.

- A piston extension can push glazed terracotta in its forward line, subject to the generic world
  bounds, hardness, block-entity and 12-block resolver gates.
- Sticky-piston retraction tests movement opposite the piston direction, so a directly exposed
  glazed block is not pulled.
- When slime or honey scans backward or branches perpendicular to motion, the resolver supplies
  the edge direction rather than the forward piston direction. A glazed block terminates that
  sticky edge and is not carried as its backward or side attachment.
- A glazed block already in the forward push line still moves normally even when another sticky
  block is part of the same resolved structure.

This behavior comes from the push reaction, not from membership in the block
`glazed_terracotta` tag. The locked runtime has no gameplay consumer of that block tag.
Resolution order, collision reordering, the 12-block cap, moving-piston states, neighbor updates,
events and client correction remain with `RED-PISTON-001`.

#### Breaking, smelting and trades

Glazed terracotta adds no water, fluid, gravity, scheduled/random tick, shape-update, use, attack,
entity-contact, neighbor, redstone-output, comparator or block-event callback. Outside piston and
explicit transform/write paths it remains inert.

Correct-tool admission remains with generic player breaking. Every one-roll loot table offers its
own block item behind `survives_explosion` and uses random sequence
`minecraft:blocks/<identity>`. An admitted ordinary player break returns the matching item; an
incorrect player tool returns nothing. Explosion context may independently suppress the item.

Each reloadable smelting recipe consumes the matching non-glazed dyed terracotta, returns one
matching glazed block, awards `0.1` experience and uses the smelting serializer's omitted-field
default of `200` cooking ticks. The paired recipe advancement unlocks from possessing that exact
non-glazed input or from the recipe-unlocked criterion. Furnace admission, fuel, progress,
allocation and publication remain with the recipe/furnace owners.

The sixteen `mason/4/emerald_<color>_glazed_terracotta` records each want one emerald and give one
matching block, with maximum uses `12`, villager XP `15` and reputation discount factor `0.05`.
The level-four mason tag combines these records with sixteen non-glazed terracotta records and
quartz-for-emerald. Its trade set draws `2` distinct candidates from all `33` records using random
sequence `minecraft:trade_set/mason/level_4`. This leaf owns the glazed record values; villager
level-up, offer construction, selection, pricing, restock and purchase remain generic.

#### Tags, sulfur cube and generation

All sixteen blocks are direct members of reloadable block tags `glazed_terracotta` and
`mineable/pickaxe`; all sixteen items are direct members of the item `glazed_terracotta` tag.
The pickaxe tag supplies generic tool-speed and harvest selection. No locked gameplay class reads
the block grouping tag.

The item tag is included by `sulfur_cube_archetype/slow_bouncy`. Swallowing or equipping any
glazed color can therefore match that archetype. Its locked knockback pair is
`(horizontal, vertical)=(0.4125, 0.24)` and its hit sound is
`minecraft:entity.sulfur_cube.slow_bouncy.hit`; multiple-match ordering, equipment changes and
knockback calculation remain with the sulfur-cube and `ENT-KNOCKBACK-001` owners.

Thirty-six locked structure templates contain glazed states:

- 24 trail-ruins templates use black, cyan, light blue, light gray, orange or yellow;
- one trial-chambers ominous-vault template uses red;
- three large underwater-ruin variants use purple;
- eight desert or savanna village templates use white, lime, light blue, orange or yellow.

The relevant trail-ruins, trial-chambers, ocean-ruin and village leaves own pool selection,
processors, rotation/mirror choice, palette traversal and placement. This leaf owns each resulting
color/facing state's behavior and projection after the template write.

**Client projection:**

Every identity has four exact blockstate variants using one
`block/<color>_glazed_terracotta` model:

| Facing | Model Y rotation |
|---|---:|
| south | 0 |
| west | 90 |
| north | 180 |
| east | 270 |

Each color model inherits `template_glazed_terracotta` and supplies its matching pattern texture.
The template is a cull-faced full cube whose north/south/east/west face UVs use rotations
`90/270/180/0`; the blockstate's Y rotation turns that complete pattern with the authoritative
facing. The item definition selects the unrotated block model directly, so item projection uses
the template's south-oriented base rather than a blockstate variant. There is no weighted,
conditional, animated or special-renderer branch.

Terrain and block-update packets publish states `14966..15029`. Ordinary full-cube face culling,
breaking particles/sounds, map shading and opaque rendering consume the selected state, map color,
model and texture.

**Branches and aborts:**

Sixteen colors; four legal facings; player/context placement versus explicit or template state;
each rotation and mirror; forward piston push versus sticky retraction, backward adhesion and
perpendicular adhesion; generic resolver admission/failure; correct versus incorrect tool;
surviving versus suppressed explosion loot; smelting/advancement success or rejection; selected
versus unselected mason candidate; block versus item tag; sulfur-archetype match; four structure
families; and server state versus blockstate/item model projection are distinct.

**Constants and randomness:**

States `14966..15029`; four states per color in `north,south,west,east` order; hardness/resistance
`1.4`; emission `0`; dampening `15`; shade `0.2`; friction `0.6`; speed/jump `1`; restitution `0`;
stack `64`; spawn-light ceiling `14`; piston cap `12`; cooking time/XP/output `200/0.1/1`; trade
cost/output/max uses/XP/discount `1/1/12/15/0.05`; trade-set amount/candidates `2/33`; blockstate Y
rotations `0/90/180/270`; `36` containing templates. Glazed terracotta consumes no RNG directly;
generic explosion, trade, structure and sulfur-cube owners retain their randomness.

**Side effects:**

Generic placement/removal and facing publication; optional piston movement; conditional same-color
loot; furnace output and advancement publication; possible mason offer; sulfur-archetype
selection; structure palette writes; ordinary persistence, map color and opaque patterned
block/item projection.

**Gates:**

Selected identity and facing; placement/transform authority; piston direction, resolver edge,
bounds and cap; correct-tool harvest admission; explosion context; active recipe, advancement,
loot, trade, tag and archetype snapshots; villager profession/level and candidate selection;
structure/pool/template selection; client state/model context.

**Boundary cases and quirks:**

The north default is not the item model's unrotated south presentation. Player placement always
uses the horizontal opposite even when the clicked face is vertical. Glazed blocks use ordinary
dye map colors, while non-glazed dyed terracotta uses terracotta-specific colors. `PUSH_ONLY`
allows forward pushing but rejects sticky pulling and off-axis adhesion without making the block
immovable. The block grouping tag has no locked gameplay consumer, but the same-named item tag
materially selects the sulfur cube's slow-bouncy archetype.

**Evidence:**

`OFF-SERVER-001`, `OFF-CLIENT-001`, `OFF-REPORT-001`, `OFF-DATA-001`;
`net.minecraft.world.level.block.Blocks`,
`net.minecraft.world.level.block.GlazedTerracottaBlock#createBlockStateDefinition`,
`net.minecraft.world.level.block.GlazedTerracottaBlock#getStateForPlacement`,
`net.minecraft.world.level.block.HorizontalDirectionalBlock#rotate`,
`net.minecraft.world.level.block.HorizontalDirectionalBlock#mirror`,
`net.minecraft.world.item.context.UseOnContext#getHorizontalDirection`,
`net.minecraft.world.level.block.state.BlockBehaviour$Properties#strength`,
`net.minecraft.world.level.block.piston.PistonBaseBlock#isPushable`,
`net.minecraft.world.level.block.piston.PistonStructureResolver#resolve`,
`net.minecraft.world.level.block.piston.PistonStructureResolver#addBlockLine`,
`net.minecraft.world.level.block.piston.PistonStructureResolver#addBranchingBlocks`,
`net.minecraft.world.item.crafting.AbstractCookingRecipe#cookingMapCodec`,
`net.minecraft.world.item.trading.VillagerTrades#registerMasonLevelFourTerracotta`,
`net.minecraft.world.item.trading.TradeSet#calculateNumberOfTrades`,
`net.minecraft.world.item.trading.VillagerTrade#getOffer`,
`net.minecraft.world.entity.monster.cubemob.SulfurCube#matchingArchetypes`;
`reports/blocks.json#minecraft:{white,orange,magenta,light_blue,yellow,lime,pink,gray,light_gray,cyan,purple,blue,brown,green,red,black}_glazed_terracotta`,
`reports/minecraft/components/item/{white,orange,magenta,light_blue,yellow,lime,pink,gray,light_gray,cyan,purple,blue,brown,green,red,black}_glazed_terracotta.json`,
`data/minecraft/tags/block/{glazed_terracotta,mineable/pickaxe}.json`,
`data/minecraft/tags/item/glazed_terracotta.json`,
`data/minecraft/tags/item/sulfur_cube_archetype/slow_bouncy.json`,
`data/minecraft/sulfur_cube_archetype/slow_bouncy.json`,
`data/minecraft/loot_table/blocks/{white,orange,magenta,light_blue,yellow,lime,pink,gray,light_gray,cyan,purple,blue,brown,green,red,black}_glazed_terracotta.json`,
`data/minecraft/recipe/{white,orange,magenta,light_blue,yellow,lime,pink,gray,light_gray,cyan,purple,blue,brown,green,red,black}_glazed_terracotta.json`,
`data/minecraft/advancement/recipes/decorations/{white,orange,magenta,light_blue,yellow,lime,pink,gray,light_gray,cyan,purple,blue,brown,green,red,black}_glazed_terracotta.json`,
`data/minecraft/villager_trade/mason/4/emerald_{white,orange,magenta,light_blue,yellow,lime,pink,gray,light_gray,cyan,purple,blue,brown,green,red,black}_glazed_terracotta.json`,
`data/minecraft/tags/villager_trade/mason/level_4.json`,
`data/minecraft/trade_set/mason/level_4.json`,
`data/minecraft/structure/{trail_ruins,trial_chambers,underwater_ruin,village}/**/*.nbt`,
`assets/minecraft/blockstates/{white,orange,magenta,light_blue,yellow,lime,pink,gray,light_gray,cyan,purple,blue,brown,green,red,black}_glazed_terracotta.json`,
`assets/minecraft/models/block/template_glazed_terracotta.json`,
`assets/minecraft/models/block/{white,orange,magenta,light_blue,yellow,lime,pink,gray,light_gray,cyan,purple,blue,brown,green,red,black}_glazed_terracotta.json`,
`assets/minecraft/items/{white,orange,magenta,light_blue,yellow,lime,pink,gray,light_gray,cyan,purple,blue,brown,green,red,black}_glazed_terracotta.json`.

**Test vectors:**

Run `EXP-BLK-043` across every color/facing, playerless and four-yaw placement, explicit
state-component writes, all rotations/mirrors, direct extension/retraction and slime/honey
forward/backward/side piston arrangements, resolver limits, correct/incorrect tools, explosions,
smelting/unlock, every mason record, tag/archetype reload, all 36 containing templates,
save/reload and block/item models. Assert state IDs, facing, map colors, physical predicates,
resolver membership, writes/drops, recipe/trade values, template transforms and selected model
rotations.

**Limits:**

This leaf does not re-specify generic placement/break packets, state-component parsing, piston
transactions, tool speed, explosion survival, recipe/furnace allocation, villager lifecycle,
sulfur-cube composition, structure/pool/processor selection or model loading. Those remain with
`BLK-002`, `BLK-STATE-001`, `PLY-006`, `RED-PISTON-001`, `ITM-LOOT-001`, recipe/furnace and
villager owners, `ENT-KNOCKBACK-001`, the four named world-generation owners and `CLI-006`.
