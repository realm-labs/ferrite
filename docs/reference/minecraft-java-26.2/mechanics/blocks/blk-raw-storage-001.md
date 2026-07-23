# Blocks mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `BLK-RAW-STORAGE-001` — Raw material storage blocks join compacting, piglins and ore-vein generation

**Parent:** `BLK-001`, `BLK-002`, `BLK-003`, `BLK-005`, `RED-001`, `PLY-005`, `PLY-006`,
`ITM-004`, `ITM-006`, `ENT-001`, `ENT-005`, `MOB-001`, `ENV-003`, `WGEN-003`, `CLI-006`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked registrations, reports, tags, recipes, advancements, loot, ore-vein and
carver code/data, an exhaustive structure-template scan and client assets exhaust raw iron, copper
and gold storage-block behavior and projection.

**Applies when:**

`minecraft:raw_iron_block`, `minecraft:raw_copper_block` or
`minecraft:raw_gold_block` is placed, mined, exploded, crafted or decompressed, used as a note-block
substrate, selected by a piglin or sulfur cube, admitted to ore-vein/carver generation, serialized
or rendered.

**Authoritative state:**

All three registrations are property-free ordinary `Block` instances with no block entity and one
state:

| Identity | State | Map color | Instrument | Destroy speed | Explosion resistance | Sound |
|---|---:|---|---|---:|---:|---|
| raw iron block | 32070 | `RAW_IRON` | `BASEDRUM` | 5.0 | 6.0 | `STONE` |
| raw copper block | 32071 | `COLOR_ORANGE` | `BASEDRUM` | 5.0 | 6.0 | `STONE` |
| raw gold block | 32072 | `GOLD` | `BASEDRUM` | 5.0 | 6.0 | `STONE` |

Every registration requires a correct tool for drops and is a direct `mineable/pickaxe` member.
Raw iron and copper blocks require the stone-tool tier; raw gold requires the iron-tool tier.
Unspecified properties retain the ordinary full-solid defaults: unit selection, collision and
occlusion shapes; emission zero; light dampening 15; friction 0.6; speed/jump factors 1; solid
redstone conduction; normal piston reaction; no random tick and no block entity.

Each block item is common, stacks to 64 and carries only the ordinary block-item components. Unlike
netherite block, none adds a damage-resistant component.

**Transition and ordering:**

#### Placement, breaking and projection

Ordinary placement writes the identity's sole state; rotation and mirror are identity operations
because no property can change. The generic player break transaction admits the matching self-loot
only when the tool is correct. Each block table then performs one roll, returns the matching item
behind `survives_explosion`, and uses `minecraft:blocks/<identity>` as its random sequence. A
wrong-tier pickaxe or non-tool can remove the block but does not reach that self-drop path.

Every blockstate maps its sole variant to `minecraft:block/<identity>`. Each block model is
`cube_all` with the matching all-face texture, and each item definition directly selects that block
model without a transform or condition.

All three registrations select `BASEDRUM` when the state is below a note block. Note pitch,
power/attack admission, block events, sound/particle projection and statistics remain with the
note-block, redstone and generic effect owners.

#### Compacting and recipe discovery

The reloadable recipe graph consists of exactly two records per identity:

- a 3-by-3 shaped building-category recipe converts nine matching raw items to one raw storage
  block;
- a shapeless recipe converts one matching raw storage block to nine raw items.

The recipe IDs are respectively `raw_iron_block`/`raw_iron`,
`raw_copper_block`/`raw_copper` and `raw_gold_block`/`raw_gold`. No record declares a group.
Each of the six matching recipe advancements contains one OR-requirement group: its own
`recipe_unlocked` criterion or possession of the recipe's input raw item/block. Direct recipe grant
and inventory discovery are therefore alternative unlock paths.

No other locked recipe, loot table or optional built-in-pack record names one of these block
identities. Individual raw items retain their separate smelting, blasting, loot and progression
behavior; compacting does not make the storage blocks furnace inputs.

#### Piglin and sulfur-cube selectors

Raw gold block alone is a direct `guarded_by_piglins` block member. The generic
`Block#playerWillDestroy` hook therefore invokes nearby-piglin anger before removing a player-broken
raw gold block. Other destruction paths and the raw iron/copper identities do not gain that
tag-selected hook.

Raw gold block is also the family's only direct `piglin_loved` item. It is not the exact barter
currency, which remains the gold ingot. Subject to the generic baby-ignore, repellent,
attack/admirer and inventory gates, a piglin can want the item entity. Pickup removes one
non-nugget item, puts it in the off hand and sets `ADMIRING_ITEM=true` for 119 ticks. When holding
ends, an adult does not generate barter loot for this stack: it first attempts equipment replacement
and otherwise stores the item. A player holding raw gold block also satisfies
`isPlayerHoldingLovedItem`; all other piglin sensing, activity arbitration and inventory policy
remain separately owned.

All three items are direct `sulfur_cube_archetype/slow_flat` members. That archetype installs
horizontal/vertical knockback powers `0.4125/0.105`, hit/push sounds
`minecraft:entity.sulfur_cube.slow_flat.{hit,push}`, push cooldown `0.9` and impulse threshold
`0.03`. Its five attribute modifiers add knockback and explosion-knockback resistance `0.5`,
bounciness `0.4000000059604645`, multiplied-total friction `-0.5999999940395355` and air drag
`-0.8999999985098839`. Matching order, installation, equipment changes and contact handling remain
with `ENT-KNOCKBACK-001` and the sulfur-cube owners.

#### Ore-vein output and later carving

Only raw copper and raw iron blocks are code-selected worldgen outputs. The independently owned
ore-vein material resolver enables copper for signed toggle values strictly above zero at inclusive
Y `0..50`, with copper ore/raw copper block/granite outputs; nonpositive toggle selects iron at
inclusive Y `-60..-8`, with deepslate iron ore/raw iron block/tuff outputs.

After band, edge, veininess, solidness, ridged-noise, richness and gap admission, the position-seeded
stream's third float returns the corresponding raw block only when strictly below `0.02`; every
other value returns the ordinary ore. The internal ore debug mode changes early null/filler
outcomes but leaves admitted ore and raw-block results unchanged. Exact density, stream and
fallback ordering remain `WGEN-PIPELINE-001`. Raw gold block is not a `VeinType` output.

Raw iron and copper blocks are direct `overworld_carver_replaceables` members. The locked
`cave`, `cave_extra_underground` and `canyon` configured carvers all use that holder set, so a later
ordinary material-kernel visit can replace either generated raw block after its mask and aquifer
gates. Raw gold is absent from the tag. Source-chunk selection, geometry, RNG, mask, aquifer,
surface restoration, write order and debug markers remain `WGEN-PIPELINE-001`.

An exhaustive decode of all 1,212 locked structure NBT files finds no live, palette-only or jigsaw
final-state occurrence of any family identity. The exhaustive server-class constant-pool sweep
finds no runtime consumer beyond registration and `OreVeinifier$VeinType`; data generators,
creative-tab population and item registration do not add gameplay branches.

**Client projection:**

Terrain and block-update packets publish exact states `32070..32072`. Each selects one opaque
matching `cube_all` block model; the item stack selects the same model directly. Ordinary full-cube
face culling, break particles/sounds and map shading consume the selected state/model. Note,
piglin, sulfur-cube and generated-terrain effects use their existing generic protocol/effect
families; this leaf introduces no packet layout.

**Branches and aborts:**

Three identities and map colors; stone versus iron harvest tier; correct versus incorrect tool;
surviving versus suppressed explosion loot; six recipe/unlock records; guarded/loved raw gold
versus ordinary raw iron/copper; nonbarter completion; slow-flat admission; copper versus iron
ore-vein band and strict raw-block chance; carver-admitted raw iron/copper versus excluded raw gold;
and block versus item projection are distinct.

**Constants and randomness:**

States `32070..32072`; strength `5.0/6.0`; emission `0`; dampening `15`; friction `0.6`;
speed/jump `1`; stack `64`; compacting `9:1`; decompression `1:9`; piglin admiration `119`; copper
band `0..50`; iron band `-60..-8`; raw-block gate `<0.02`; slow-flat powers
`0.4125/0.105`, push cooldown/threshold `0.9/0.03` and five attribute amounts as listed. The blocks
consume no RNG directly; generic explosion, loot, piglin, sulfur-cube, ore-vein and carver owners
retain their documented streams.

**Side effects:**

Generic placement/removal; conditional matching self loot; recipe and advancement outputs;
tag-selected piglin anger/admiration/inventory state; sulfur-cube attribute and knockback selection;
noise-material raw-block writes; later admitted carver replacement; ordinary persistence and opaque
block/item projection.

**Gates:**

Selected identity; placement/write authority; correct-tool harvest; explosion context; active
recipe, advancement, loot, tag and archetype snapshots; piglin brain/inventory state; Overworld
ore-vein setting, Y band, density/noise/random gates; configured-carver selection, mask,
replaceable-holder and aquifer gates; client state/model context.

**Boundary cases and quirks:**

All three use the stone sound and bass-drum instrument despite their metal materials. Raw iron and
copper share the stone harvest tier and carver replacement role, while raw gold uses the iron tier
and piglin roles. None is a beacon-base member, and raw iron does not satisfy the iron-golem body
pattern. Piglin love does not make raw gold block barter currency. The ore-vein raw-block gate is
the third position-seeded float after prior admission, not an unconditional two-percent chance over
all cells. No structure-template occurrence or non-block acquisition table broadens generation or
loot.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-REPORT-001`; `OFF-DATA-001`;
`net.minecraft.world.level.block.Blocks`;
`net.minecraft.world.level.block.state.BlockBehaviour$Properties#strength(float,float)`;
`net.minecraft.world.level.block.state.BlockBehaviour$Properties#instrument(net.minecraft.world.level.block.state.properties.NoteBlockInstrument)`;
`net.minecraft.world.level.block.Block#playerWillDestroy(net.minecraft.world.level.Level,net.minecraft.core.BlockPos,net.minecraft.world.level.block.state.BlockState,net.minecraft.world.entity.player.Player)`;
`net.minecraft.world.entity.monster.piglin.PiglinAi#isLovedItem(net.minecraft.world.item.ItemStack)`;
`net.minecraft.world.entity.monster.piglin.PiglinAi#wantsToPickup(net.minecraft.world.entity.monster.piglin.Piglin,net.minecraft.world.item.ItemStack)`;
`net.minecraft.world.entity.monster.piglin.PiglinAi#pickUpItem(net.minecraft.server.level.ServerLevel,net.minecraft.world.entity.monster.piglin.Piglin,net.minecraft.world.entity.item.ItemEntity)`;
`net.minecraft.world.entity.monster.piglin.PiglinAi#stopHoldingOffHandItem(net.minecraft.server.level.ServerLevel,net.minecraft.world.entity.monster.piglin.Piglin,boolean)`;
`net.minecraft.world.entity.monster.piglin.PiglinAi#isPlayerHoldingLovedItem(net.minecraft.world.entity.LivingEntity)`;
`net.minecraft.world.entity.monster.cubemob.SulfurCube#matchingArchetypes`;
`net.minecraft.world.level.levelgen.OreVeinifier`;
`net.minecraft.world.level.levelgen.OreVeinifier$VeinType`;
`net.minecraft.world.level.levelgen.carver.WorldCarver#carveBlock`;
`reports/blocks.json#minecraft:{raw_iron_block,raw_copper_block,raw_gold_block}`;
`reports/minecraft/components/item/{raw_iron_block,raw_copper_block,raw_gold_block}.json`;
`data/minecraft/tags/block/{mineable/pickaxe,needs_stone_tool,needs_iron_tool,guarded_by_piglins,overworld_carver_replaceables}.json`;
`data/minecraft/tags/item/{piglin_loved,sulfur_cube_archetype/slow_flat}.json`;
`data/minecraft/sulfur_cube_archetype/slow_flat.json`;
`data/minecraft/loot_table/blocks/{raw_iron_block,raw_copper_block,raw_gold_block}.json`;
`data/minecraft/recipe/{raw_iron_block,raw_iron,raw_copper_block,raw_copper,raw_gold_block,raw_gold}.json`;
`data/minecraft/advancement/recipes/building_blocks/{raw_iron_block,raw_copper_block,raw_gold_block}.json`;
`data/minecraft/advancement/recipes/misc/{raw_iron,raw_copper,raw_gold}.json`;
`data/minecraft/worldgen/configured_carver/{cave,cave_extra_underground,canyon}.json`;
`data/minecraft/structure/**/*.nbt`;
`assets/minecraft/blockstates/{raw_iron_block,raw_copper_block,raw_gold_block}.json`;
`assets/minecraft/models/block/{raw_iron_block,raw_copper_block,raw_gold_block}.json`;
`assets/minecraft/items/{raw_iron_block,raw_copper_block,raw_gold_block}.json`.

**Test vectors:**

Run `EXP-BLK-049` across all three states and ordinary/component writes; correct/incorrect tools,
explosion survival, all six recipes and alternative unlock paths; note substrates; player and
nonplayer raw-gold destruction; piglin pickup/holding/player display/nonbarter completion; slow-flat
matching and installed values; copper/iron ore-vein endpoints, all strict/equality gates and debug
mode; each configured carver's admitted/rejected replacement; save/reload and every block/item
model. Assert exact states, physical values, drops, recipe outputs, tag-selected effects,
position-seeded ordering, writes and projection. Re-run the complete template/data/code reference
sweeps and assert the documented absence boundaries.

**Limits:**

Generic placement/breaking, note-block runtime, recipes/advancements/loot, piglin AI outside the
raw-gold membership paths, sulfur-cube runtime, ore-vein/carver algorithms and client rendering
remain with `BLK-PLACE-001`, `BLK-BREAK-001`, `BLK-BREAK-HOOK-001`, `ITM-RECIPE-001`,
`ITM-ADVANCEMENT-001`, `ITM-LOOT-001`, `MOB-UNIVERSAL-ANGER-001`, `ENT-KNOCKBACK-001`,
`WGEN-PIPELINE-001` and `CLI-006`. Individual raw materials and ores, refined storage blocks,
copper oxidation/waxing, other mineral/storage identities and non-family tag members retain their
separate owners or `Unreviewed` status.
