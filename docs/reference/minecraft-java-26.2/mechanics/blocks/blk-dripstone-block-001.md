# Blocks mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `BLK-DRIPSTONE-BLOCK-001` — Dripstone block joins pointed growth, cave features and acquisition

**Parent:** `SIM-004`, `BLK-001`, `BLK-002`, `BLK-003`, `BLK-004`, `BLK-005`, `PLY-005`,
`PLY-006`, `ITM-004`, `ITM-006`, `ENT-001`, `ENV-003`, `WGEN-003`, `CLI-006`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — the locked registration, block/item/sound reports, pointed-dripstone and three
feature implementations, complete recipe/advancement/loot/trade/tag data, all 1,212 decoded
structure templates and exact client assets exhaust the property-free base identity. The pointed
dripstone state machine remains classified by `simple-waterlogged`; this leaf owns the exact base-
block join that admits its growth and generation.

**Applies when:**

`minecraft:dripstone_block` is placed, mined, exploded, crafted, offered by a mason, selected
through a reloadable tag or feature configuration, used as the support above a growing stalactite,
written by dripstone-cave generation, persisted, mapped or rendered.

**Authoritative state:**

The identity is an ordinary property-free `Block`, has no block entity and has one default state,
`30208`. Its block protocol ID is `1132` and its block-item raw ID is `53`. Registration fixes
`TERRACOTTA_BROWN` map color, `BASEDRUM`, `DRIPSTONE_BLOCK` sound, the correct-tool requirement,
hardness `1.5` and explosion resistance `1.0`.

State `30208` is a full unit selection/collision/visual/occlusion cube with emission `0`, light
dampening `15`, shade brightness `0.2`, friction `0.6`, speed/jump factors `1`, restitution `0`,
solid redstone conduction, normal piston reaction and all faces sturdy. It adds no random or
scheduled tick, use, attack, entity-contact, neighbor, signal, comparator or block-event override.
It is directly `mineable/pickaxe`; no incorrect-tier tag names it, so every pickaxe tier is correct
and non-pickaxe tools are not. The block item stacks to `64` and directly selects the reloadable
`sulfur_cube_archetype/slow_bouncy` item family.

The sound type has volume/pitch multipliers `1/1` and sound registry IDs break `544`, step `545`,
place `546`, hit `547` and fall `548`. The ordinary full-block, note-block, spawn, path, redstone,
light, piston, movement and sound dispatch rules remain with their generic owners.

**Transition and ordering:**

#### Placement, breaking, crafting and trade

Ordinary placement, component/command writes and feature writes always select state `30208`;
rotation and mirror cannot alter it. Its one-roll block loot table offers one matching item behind
`survives_explosion` and uses random sequence `minecraft:blocks/dripstone_block`. The generic
correct-tool harvest gate runs before table evaluation; Silk Touch and Fortune do not alter the
self-drop table.

The sole exact recipe is a shaped 2-by-2 square of four `pointed_dripstone`, yielding one
`dripstone_block`. Its advancement has `has_pointed_dripstone` inventory and `has_the_recipe`
criteria in one OR requirement and grants only this recipe. Grid orientation/reflection,
consumption, output admission and recipe-book publication remain generic.

`mason/3/emerald_dripstone_block` wants one emerald and gives four block items, with maximum uses
`16`, villager XP `10` and reputation discount `0.05`. It is the fourth of seven ordered level-
three mason candidates. The level trade set selects two distinct candidates with random sequence
`minecraft:trade_set/mason/level_3`; candidate selection, demand, price calculation, exhaustion and
offer publication remain with the generic trade owners.

#### Live pointed-dripstone growth

The `Blocks` bootstrap constructs `PointedDripstoneBlock` with state `30208` as its immutable
`blockToGrowOn`. A pointed state becomes a growth start only when it is a downward stalactite and
the block immediately above is not another pointed state. The pointed random tick first draws one
float for its separate fluid-transfer path; inherited growth then draws a second float and enters
only below `0.011377778`. `PointedDripstoneBlock#canGrow` requires this exact dripstone block
immediately above the start and a source-water fluid state two blocks above it.

After admission the implementation searches at most seven pointed states for a free, non-
waterlogged downward tip. It rejects a blocked or fluid-filled next cell, then draws one boolean:
true extends the stalactite downward; false searches at most ten cells below for an existing
upward tip to extend or a valid floor on which to start a stalagmite. Opposing tips merge into two
`TIP_MERGE` states; new pointed states preserve waterlogging only when written into water. The
dripstone block itself neither ticks nor consumes RNG: it is the exact support identity read by the
pointed block's admitted random-tick transaction. Fluid transfer, cauldron fill, falling,
collision damage and pointed-state survival remain with the pointed implementation and their
generic owners.

#### Reloadable selectors and world generation

The block is directly in `sculk_replaceable`, which composes into
`sculk_replaceable_world_gen`, and is an exact support entry in both the `glow_lichen` and
`sculk_vein` multiface-growth configurations (search range `20`; wall/ceiling for lichen and all
three face classes with spread chance `1` for the vein). Those selectors can respectively replace
or attach to state `30208` when their owning algorithms' remaining gates pass. It is not a member
of `dripstone_replaceable_blocks`; that tag contains `base_stone_overworld`. Dripstone features
nevertheless recognize an already written dripstone block as their configured or hard-coded base.

The dripstone-caves biome installs three generation joins:

- `pointed_dripstone` performs `192..256` placed attempts, then `1..5` local attempts each. Its
  simple random selector chooses one of equal upward/downward speleothem features after a
  12-block environment scan and one-block vertical offset. An admitted feature accepts an
  existing base or a replaceable stone, writes the center base and probabilistic horizontal
  patches using codec defaults `0.7/0.5/0.5`, chooses height two with probability `0.2` when the
  next cell is air/water, otherwise height one, and writes pointed states with update flag `2`;
- `dripstone_cluster` performs `48..96` placed attempts. One admitted origin must be air or water;
  it samples height `3..6`, two independent radii `2..8`, density uniform `[0.3,0.7)`, wetness
  clamped-normal mean/deviation `0.1/0.3` bounded `0.1..0.9`, and iterates the inclusive rectangle.
  The implementation scans floor/ceiling within `12`, uses edge probability down to `0.1`, height
  deviation `3`, maximum opposing-height difference `1`, and base-layer thickness `2..4`; its
  successful base substitutions and speleothems use configured state `30208`; and
- `large_dripstone` performs `10..48` placed attempts. It scans an air/water cave column, rejects
  heights below four, samples its clamped radius `3..16` and the locked bluntness/scale/wind
  providers, and shrinks or rejects bases outside stone. With `DEBUG_LARGE_DRIPSTONE` false it
  writes hard-coded state `30208` into air/water/lava along accepted wind-offset columns. With the
  debug flag true those column writes become glass and the feature adds diamond/gold endpoint and
  creeper-head path markers; that explicit diagnostic branch writes no dripstone-block result.

Placed-feature modifier order, biome admission, RNG ownership, column scanning and individual
writes remain with `WGEN-PIPELINE-001`. The complete NBT scan found zero dripstone-block cells in
all 1,212 bundled structure templates, so there is no structure-palette or processor join to claim.

**Client projection:**

The only blockstate variant unconditionally selects `minecraft:block/dripstone_block`. That model
inherits `cube_all` and maps every face to `minecraft:block/dripstone_block`; the item selector
points to the same block model. Authoritative block updates publish state `30208`, inventory
projection uses item ID `53`, and sound projection uses IDs `544..548`. This leaf adds no packet
field, acknowledgement, ordering rule or connection-local state beyond the audited registry,
block-update, inventory and sound mappings.

**Branches and aborts:**

Correct/incorrect tool and explosion survival; shaped match/reflection/output capacity and either
unlock criterion; selected/unselected/exhausted mason candidate; exact growth support versus any
other block, source/non-source water, admitted/missed random tick, downward/stalagmite branch,
blocked/fluid cell and opposing tip; direct/composed tag membership; three feature paths, origin/
column/replaceability rejection, normal state-30208/debug glass-marker build, successful/failed
write; save/reload and
client identity are distinct branches.

**Constants and randomness:**

State/block/item IDs `30208/1132/53`; strength `1.5/1`; sound IDs `544..548`; emission `0`,
dampening `15`, shade `0.2`, friction `0.6`, factors `1`, restitution `0`, stack `64`; recipe input/
output `4/1`; mason selection `2/7`, output `4`, uses/XP/discount `16/10/0.05`; live growth
fluid-transfer draw thresholds water/lava `0.17578125/0.05859375`, growth probability
`0.011377778`, maximum tip search `7`, floor search `10`; pointed attempts `192..256`
then `1..5`, scan `12`, defaults `0.2/0.7/0.5/0.5`; cluster attempts `48..96`, height `3..6`,
radii `2..8`, scan `12`, edge chance `0.1`, density `0.3..0.7`, wetness `0.1..0.9`, layer `2..4`;
large attempts `10..48`, radius `3..16`; structure cells `0`. RNG belongs to the pointed, recipe,
trade and feature owners; the base block consumes none.

**Side effects:**

Ordinary full-block placement and self loot; one crafting result/unlock; one possible mason offer;
tag-selected sculk and multiface behavior; pointed growth admission; feature base/column writes;
ordinary palette/inventory persistence; sounds and opaque cube-all projection.

**Gates:**

Write authority; correct-tool harvest and explosion context; active recipe, advancement, loot,
trade, tag, feature and archetype snapshots; crafting/trade admission; exact growth arrangement and
random tick; biome/placement/feature gates; valid registry mapping and client resource context.

**Boundary cases and quirks:**

The base block is not random-ticking even though it controls whether an adjacent pointed block may
grow. The live arrangement requires source water above the base, not waterlogged pointed dripstone.
The generation replaceable tag does not contain the result identity, but feature code separately
accepts an existing base. Large dripstone hard-codes the normal result rather than carrying
`base_block` in its JSON, while its debug branch deliberately substitutes glass/markers. No
structure template contains the state, and the level-three mason family already documents the
other six of seven candidates without owning this remaining record.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-REPORT-001`; `OFF-DATA-001`;
`net.minecraft.world.level.block.Blocks`;
`net.minecraft.world.level.block.SoundType`;
`net.minecraft.world.level.block.SpeleothemBlock#randomTick`;
`net.minecraft.world.level.block.SpeleothemBlock#growStalactiteOrStalagmiteIfPossible`;
`net.minecraft.world.level.block.PointedDripstoneBlock#canGrow`;
`net.minecraft.world.level.levelgen.feature.SpeleothemFeature#place`;
`net.minecraft.world.level.levelgen.feature.SpeleothemClusterFeature#place`;
`net.minecraft.world.level.levelgen.feature.LargeDripstoneFeature#place`;
`net.minecraft.world.level.levelgen.feature.SpeleothemUtils`;
`net.minecraft.world.item.trading.TradeSet#calculateNumberOfTrades`;
`net.minecraft.world.item.trading.VillagerTrade#getOffer`;
`reports/blocks.json#minecraft:dripstone_block`;
`reports/registries.json#minecraft:{block,item,sound_event}`;
`reports/minecraft/components/item/dripstone_block.json`;
`data/minecraft/{loot_table/blocks,recipe,advancement/recipes/building_blocks}/dripstone_block.json`;
`data/minecraft/{villager_trade/mason/3/emerald_dripstone_block,tags/villager_trade/mason/level_3,trade_set/mason/level_3}.json`;
`data/minecraft/tags/block/{mineable/pickaxe,sculk_replaceable,sculk_replaceable_world_gen,dripstone_replaceable_blocks}.json`;
`data/minecraft/tags/item/sulfur_cube_archetype/slow_bouncy.json`;
`data/minecraft/worldgen/{configured_feature,placed_feature}/{pointed_dripstone,dripstone_cluster,large_dripstone}.json`;
`data/minecraft/worldgen/configured_feature/{glow_lichen,sculk_vein}.json`;
`data/minecraft/worldgen/biome/dripstone_caves.json`;
`data/minecraft/structure/**/*.nbt`;
`assets/minecraft/{blockstates/dripstone_block,models/block/dripstone_block,items/dripstone_block}.json`.

**Test vectors:**

Run `EXP-BLK-057` across state and registry identity, physical/tool/loot behavior, the recipe and
both unlock criteria, every level-three mason candidate, direct/composed selectors, exact live
growth arrangements and RNG branches, all three configured/placed feature paths, all 1,212
structure inputs, persistence, sounds and block/item models. Assert IDs, constants, matches,
admission, draw/write order, zero structure cells and vanilla-client convergence.

**Limits:**

Generic placement, breaking, loot, crafting, advancements, trade selection, sulfur-cube movement,
sculk/multiface generation, feature placement, pointed state transitions, packet encoding and
rendering remain with `BLK-PLACE-001`, `PLY-BREAK-001`, `ITM-LOOT-001`, `ITM-RECIPE-001`,
`ITM-ADVANCEMENT-001`, the generic trade owners, `WGEN-PIPELINE-001`, `SIM-RANDOM-001`,
`ENV-FLUID-001`, `PROTO-PLAY-CLIENTBOUND-TERRAIN-001`, `PROTO-PLAY-CLIENTBOUND-BLOCK-001`,
`PROTO-PLAY-CLIENTBOUND-SOUND-001` and `CLI-006`.
