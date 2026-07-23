# Blocks mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `BLK-TERRACOTTA-001` — Terracotta joins ordinary solid cubes to color, substrate, trade and world-generation selectors

**Parent:** `BLK-001`, `BLK-002`, `BLK-003`, `BLK-005`, `PLY-005`, `PLY-006`, `ITM-004`,
`ITM-006`, `ENT-001`, `MOB-001`, `ENV-003`, `WGEN-003`, `CLI-006`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — the locked ordinary-block and color-collection registrations, reports, every
direct and composed block/item tag, recipe, advancement, loot/trade record, world-generation
consumer and client asset exhaust plain terracotta plus the sixteen non-glazed dyed identities.

**Applies when:**

`minecraft:terracotta` or any `minecraft:<color>_terracotta` identity is placed, broken, exploded,
queried for shape/light/spawn/vegetation support, consumed by a recipe or sulfur cube, offered by a
mason, selected or replaced during world generation, serialized, mapped or rendered.

**Authoritative state:**

Plain terracotta is an ordinary `Block` with property-free state `12912`. `Blocks.DYED_TERRACOTTA`
registers another ordinary `Block` for every `DyeColor`; those sixteen property-free states have no
block entity:

| Identity | State |
|---|---:|
| `terracotta` | 12912 |
| `white_terracotta` | 11444 |
| `orange_terracotta` | 11445 |
| `magenta_terracotta` | 11446 |
| `light_blue_terracotta` | 11447 |
| `yellow_terracotta` | 11448 |
| `lime_terracotta` | 11449 |
| `pink_terracotta` | 11450 |
| `gray_terracotta` | 11451 |
| `light_gray_terracotta` | 11452 |
| `cyan_terracotta` | 11453 |
| `purple_terracotta` | 11454 |
| `blue_terracotta` | 11455 |
| `brown_terracotta` | 11456 |
| `green_terracotta` | 11457 |
| `red_terracotta` | 11458 |
| `black_terracotta` | 11459 |

Plain registration selects `MapColor.COLOR_ORANGE`. Dyed registration selects each
`DyeColor#getTerracottaColor`, which is deliberately distinct from the dye's ordinary map color.
Every identity supplies bass-drum note instrument, `requiresCorrectToolForDrops`, destroy speed
`1.25` and explosion resistance `4.2`.

All other properties remain ordinary defaults: stone sound, full
selection/collision/visual/occlusion shapes, emission `0`, friction `0.6`, speed/jump factors `1`,
restitution `0`, and piston reaction `NORMAL`. The full solid-render cube does not propagate
skylight, has light dampening `15` and shade brightness `0.2`, and is redstone-conducting,
suffocating and view-blocking. Its sturdy upper face passes the default spawn-support predicate
because emission is below `14`; entity-specific placement rules remain an independent gate.

Each identity has an ordinary common-rarity `BlockItem`, maximum stack size `64`, its matching
block item name/model and no special use component. Generic placement writes exactly the
property-free state selected by that item.

**Transition and ordering:**

#### Placement, breaking and acquisition

Terracotta adds no water, fluid, gravity, scheduled/random tick, shape-update, use, attack,
entity-contact, neighbor, redstone, comparator or block-event callback. Direct placement and
explicit writes therefore use generic block transactions and remain inert until another system
explicitly replaces the state.

Correct-tool admission remains with generic player breaking. After admission, every one-roll loot
table offers its own block item behind `survives_explosion` and uses random sequence
`minecraft:blocks/<identity>`. An ordinary admitted player break returns the matching item; an
incorrect player tool returns nothing. Explosion context can independently suppress the item.

The reloadable plain-terracotta smelting recipe consumes one clay block, takes the smelting
serializer's omitted-field default of `200` cooking ticks, returns one terracotta and awards
`0.35` experience. Every dyed recipe is the same centered 3x3 pattern: eight plain terracotta
around the matching dye return eight matching dyed terracotta in group `stained_terracotta`.
Their recipe advancements unlock from possessing plain terracotta or from the recipe-unlocked
criterion.

Each dyed identity is also the exact input to its matching glazed-terracotta smelting recipe. The
same `200`-tick default returns one glazed block and awards `0.1` experience; the unlock
advancement accepts the matching dyed input or prior recipe unlock. Glazed terracotta's facing,
placement and piston-special behavior is outside this leaf.

Plain terracotta is the center material in both host and wayfinder armor-trim template duplication
recipes. Each pattern consumes one existing template, one terracotta and seven diamonds and
returns two copies; matching, consumption, allocation and advancement publication remain with the
generic recipe/progression owners.

#### Mason offers

The sixteen reloadable `mason/4/emerald_<color>_terracotta` records each want one emerald and give
one matching dyed terracotta, with maximum uses `12`, villager XP `15` and reputation discount
factor `0.05`. The level-four mason tag combines those sixteen entries with quartz-for-emerald and
sixteen glazed-terracotta entries. Its trade set draws `2` candidates without duplicates from that
33-entry holder set using random sequence `minecraft:trade_set/mason/level_4`.

This leaf owns the sixteen non-glazed output identities and their record values. Profession
level-up, offer construction, random selection, price mutation, restocking and transaction commit
remain with the villager/trading owners.

#### Reloadable tags and consumers

All seventeen blocks are direct members of `terracotta` and `mineable/pickaxe`; all seventeen block
items are direct members of the item `terracotta` tag. The pickaxe tag supplies generic
tool-speed/harvest selection. The item tag is included by
`sulfur_cube_archetype/slow_bouncy`, so swallowing or equipping any identity can match that
archetype. Its locked knockback pair is `(horizontal, vertical)=(0.4125, 0.24)` and its hit sound is
`minecraft:entity.sulfur_cube.slow_bouncy.hit`; multiple-match ordering, equipment changes and
knockback calculation remain with the sulfur-cube and `ENT-KNOCKBACK-001` owners.

The block `terracotta` tag is composed into six active substrate/replacement tags:

- `azalea_grows_on` and `azalea_root_replaceable` admit all seventeen identities in the locked
  azalea tree/root-system configurations.
- `overworld_carver_replaceables` lets configured Overworld carvers replace them.
- `sculk_replaceable` admits ordinary and world-generation sculk spread plus the sculk-vein
  substrate probe.
- `supports_dry_vegetation` makes `DryVegetationBlock#mayPlaceOn` accept every identity.
- `triggers_ambient_desert_dry_vegetation_block_sounds` supplies the client-local dry-grass and
  dead-bush sound substrate described below.

`badlands_terracotta` is a narrower direct set: plain, white, yellow, orange, red, brown and light
gray only. `armadillo_spawnable_on` includes that tag. `Armadillo#checkArmadilloSpawnRules`
therefore accepts those seven identities as the block below a candidate only when the separate
brightness check also passes; the other ten dyed identities do not enter through this branch.
Natural-spawn caps, packs, biome lists and insertion remain with `MOB-SPAWN-001`.

#### Dry-vegetation client sound

Short and tall dry grass call `playAmbientDryGrassSounds` from their client animation callback.
Each callback draws `nextInt(200)` first. On zero, the block immediately below the grass and the
block below that must both belong to
`triggers_ambient_desert_dry_vegetation_block_sounds`; success plays the player-local
`minecraft:block.dry_grass.ambient` event through `SoundEvents.DRY_GRASS`, ambient source,
volume/pitch `1`.

Dead bush inherits `DryVegetationBlock#animateTick` and first draws `nextInt(130)`. If its immediate
substrate is red sand or any member of `terracotta`, it then draws `nextInt(3)` and aborts unless
that result is zero. The same two-layer trigger-tag test follows; success plays local
`minecraft:block.deadbush.idle`, ambient source, volume/pitch `1`, at the bush position. Thus a
dead bush on terracotta has a `1/390` chance to reach the two-layer test per animation callback,
while dry grass has `1/200`. These are ordered client RNG gates, not server random ticks.

#### Locked world-generation joins

`SurfaceSystem` constructs a 192-entry clay-band array initialized to plain terracotta. A dedicated
hash stream first places separated orange entries, then `6..15` runs each of yellow width `1..3`,
brown width `2..4` and red width `1..3`, followed by `9..15` white entries with adjacent light
gray. Band lookup uses
`(Y + round(clayBandsOffset(X,0,Z) * 4) + 192) % 192`. The Overworld surface-rule trees also
select exact plain, orange and white terracotta states outside that lookup. Surface traversal,
noise setup, RNG ownership and writes remain fully specified by `WGEN-PIPELINE-001`.

The azalea, sculk and carver tag joins above feed the same pipeline. Village zombie processors add
three exact replacement predicates: desert replaces plain terracotta with cobweb at probability
`0.08`, plains replaces white at `0.07`, and savanna independently replaces orange, yellow and red
at `0.05` each. Their rule order and template placement remain with
`WGEN-JIGSAW-PROCESSORS-001` and `WGEN-JIGSAW-VILLAGES-001`.

Desert-pyramid construction writes exact orange and blue terracotta into its tower/floor motifs and
archaeology room. Trail-ruins templates contain plain and multiple dyed identities. Those
coordinate/template inventories and placement algorithms remain with
`WGEN-STRUCTURE-DESERT-PYRAMID-001` and `WGEN-JIGSAW-TRAIL-RUINS-001`; this leaf owns the state
semantics after placement.

**Client projection:**

Every property-free blockstate selects `block/<identity>`. Each block model inherits opaque
`cube_all` and uses the matching `block/<identity>` texture on all faces; each item definition
selects that same block model. There is no weighted, conditional, animated or special-renderer
branch.

Terrain packets publish state `12912` or `11444..11459`. Ordinary full-cube face culling,
breaking particles/sounds and item rendering consume that state/model. Map projection uses plain
orange or the corresponding terracotta-specific dye map color selected at registration.

**Branches and aborts:**

Plain versus sixteen dye identities; direct placement versus external worldgen/replacement writes;
correct versus incorrect tool; surviving versus suppressed explosion loot; clay smelting versus
dye crafting versus glazed smelting/template duplication; selected versus unselected mason
candidate; all-terracotta versus seven-member badlands tag; support, armadillo, sculk, carver and
azalea consumers; dry-grass versus dead-bush ordered sound gates; each worldgen selector; and one
opaque block/item model per identity are distinct.

**Constants and randomness:**

States `12912` and `11444..11459`; hardness `1.25`; resistance `4.2`; emission `0`; dampening `15`;
shade `0.2`; friction `0.6`; speed/jump `1`; restitution `0`; stack `64`; spawn-light ceiling
`14`; cooking time `200`; recipe outputs/XP `1/0.35`, `8`, and `1/0.1`; template output `2`;
trade cost/output/max uses/XP/discount `1/1/12/15/0.05`; trade-set amount/candidates `2/33`;
dry-grass gate `1/200`; terracotta-supported dead-bush pre-test `1/390`; clay-band length and
offset scale `192/4`; zombie-village probabilities `0.08`, `0.07` and `0.05`. Generic explosion,
trade, worldgen and consumer owners retain their additional randomness.

**Side effects:**

Generic state placement/removal and publication; conditional matching self loot; recipe and
furnace outputs; possible mason offer output; substrate/spawn/replacement admissions; sulfur-cube
archetype selection; client-local ambient sound; terrain/structure/template writes; persistence,
map color and opaque block/item projection.

**Gates:**

Selected identity; placement/removal authority; correct-tool and explosion context; active recipe,
advancement, loot, tag, trade and archetype snapshots; villager profession/level and candidate
selection; armadillo brightness; vegetation and two-layer sound substrates; configured
feature/carver/surface/structure context; client animation RNG; and state/model context.

**Boundary cases and quirks:**

Plain terracotta's state ID is not adjacent to the dyed range. Dyed blocks use
`getTerracottaColor`, not each dye's ordinary map color. All seventeen use stone sound despite the
bass-drum instrument. The broad tag supports dry vegetation and several replacement systems, but
only seven identities join armadillo floors through `badlands_terracotta`. Dead bushes on
terracotta receive an extra one-in-three suppression after their one-in-130 draw. Finished dyed
terracotta smelts into glazed terracotta, whose direction and piston behavior is intentionally not
inherited by this family.

**Evidence:**

`OFF-SERVER-001`, `OFF-CLIENT-001`, `OFF-REPORT-001`, `OFF-DATA-001`;
`net.minecraft.world.level.block.Blocks`,
`net.minecraft.sounds.SoundEvents`,
`net.minecraft.world.level.block.ColorCollection#registerBlocks`,
`net.minecraft.world.item.DyeColor#getTerracottaColor`,
`net.minecraft.world.level.block.state.BlockBehaviour#getShape`,
`net.minecraft.world.level.block.state.BlockBehaviour#getCollisionShape`,
`net.minecraft.world.level.block.state.BlockBehaviour#getVisualShape`,
`net.minecraft.world.level.block.state.BlockBehaviour#propagatesSkylightDown`,
`net.minecraft.world.level.block.state.BlockBehaviour#getLightDampening`,
`net.minecraft.world.level.block.state.BlockBehaviour#getShadeBrightness`,
`net.minecraft.world.level.block.DryVegetationBlock#mayPlaceOn`,
`net.minecraft.world.level.block.DryVegetationBlock#animateTick`,
`net.minecraft.world.level.block.ShortDryGrassBlock#animateTick`,
`net.minecraft.world.level.block.TallDryGrassBlock#animateTick`,
`net.minecraft.world.level.block.sounds.AmbientDesertBlockSoundsPlayer#playAmbientDryGrassSounds`,
`net.minecraft.world.level.block.sounds.AmbientDesertBlockSoundsPlayer#playAmbientDeadBushSounds`,
`net.minecraft.world.level.block.sounds.AmbientDesertBlockSoundsPlayer#shouldPlayDesertDryVegetationBlockSounds`,
`net.minecraft.world.entity.animal.armadillo.Armadillo#checkArmadilloSpawnRules`,
`net.minecraft.world.item.crafting.AbstractCookingRecipe#cookingMapCodec`,
`net.minecraft.world.item.trading.VillagerTrades#registerMasonLevelFourTerracotta`,
`net.minecraft.world.item.trading.TradeSet#calculateNumberOfTrades`,
`net.minecraft.world.item.trading.VillagerTrade#getOffer`,
`net.minecraft.world.entity.monster.cubemob.SulfurCube#matchingArchetypes`,
`net.minecraft.world.level.levelgen.SurfaceSystem#generateBands`,
`net.minecraft.world.level.levelgen.SurfaceSystem#getBand`;
`reports/blocks.json#minecraft:terracotta`,
`reports/blocks.json#minecraft:{white,orange,magenta,light_blue,yellow,lime,pink,gray,light_gray,cyan,purple,blue,brown,green,red,black}_terracotta`,
`reports/minecraft/components/item/terracotta.json`,
`reports/minecraft/components/item/{white,orange,magenta,light_blue,yellow,lime,pink,gray,light_gray,cyan,purple,blue,brown,green,red,black}_terracotta.json`,
`data/minecraft/tags/block/{terracotta,badlands_terracotta,armadillo_spawnable_on,azalea_grows_on,azalea_root_replaceable,overworld_carver_replaceables,sculk_replaceable,supports_dry_vegetation,triggers_ambient_desert_dry_vegetation_block_sounds,mineable/pickaxe}.json`,
`data/minecraft/tags/item/terracotta.json`,
`data/minecraft/tags/item/sulfur_cube_archetype/slow_bouncy.json`,
`data/minecraft/sulfur_cube_archetype/slow_bouncy.json`,
`data/minecraft/loot_table/blocks/terracotta.json`,
`data/minecraft/loot_table/blocks/{white,orange,magenta,light_blue,yellow,lime,pink,gray,light_gray,cyan,purple,blue,brown,green,red,black}_terracotta.json`,
`data/minecraft/recipe/{terracotta,host_armor_trim_smithing_template,wayfinder_armor_trim_smithing_template}.json`,
`data/minecraft/recipe/{white,orange,magenta,light_blue,yellow,lime,pink,gray,light_gray,cyan,purple,blue,brown,green,red,black}_{terracotta,glazed_terracotta}.json`,
`data/minecraft/advancement/recipes/{building_blocks,decorations,misc}/*terracotta*.json`,
`data/minecraft/villager_trade/mason/4/emerald_{white,orange,magenta,light_blue,yellow,lime,pink,gray,light_gray,cyan,purple,blue,brown,green,red,black}_terracotta.json`,
`data/minecraft/tags/villager_trade/mason/level_4.json`,
`data/minecraft/trade_set/mason/level_4.json`,
`data/minecraft/worldgen/processor_list/zombie_{desert,plains,savanna}.json`,
`data/minecraft/worldgen/noise_settings/{overworld,large_biomes,amplified,caves,floating_islands}.json`,
`assets/minecraft/blockstates/terracotta.json`,
`assets/minecraft/blockstates/{white,orange,magenta,light_blue,yellow,lime,pink,gray,light_gray,cyan,purple,blue,brown,green,red,black}_terracotta.json`,
`assets/minecraft/models/block/terracotta.json`,
`assets/minecraft/models/block/{white,orange,magenta,light_blue,yellow,lime,pink,gray,light_gray,cyan,purple,blue,brown,green,red,black}_terracotta.json`,
`assets/minecraft/items/terracotta.json`,
`assets/minecraft/items/{white,orange,magenta,light_blue,yellow,lime,pink,gray,light_gray,cyan,purple,blue,brown,green,red,black}_terracotta.json`.

**Test vectors:**

Run `EXP-BLK-042` across all seventeen identities, direct placement/writes, correct and incorrect
tools, explosion contexts, shape/light/spawn queries, every tag consumer and reload boundary,
plain/dyed/glazed/template recipes, every mason candidate, controlled dry-grass/dead-bush RNG,
badlands/azalea/sculk/carver/surface/processor/structure fixtures, save/reload and every block/item
model. Assert states, map colors, physical predicates, writes/drops, recipe/trade values, consumer
admission, RNG draw order, generated identities and selected models.

**Limits:**

This leaf does not re-specify generic placement/break packets, tool speed, explosion survival,
recipe/furnace allocation, villager lifecycle/pricing, sulfur-cube composition, mob spawning,
plant updates, sculk/carver/surface/structure traversal or model loading. Those remain with
`BLK-002`, `PLY-006`, `ITM-LOOT-001`, recipe/furnace owners, villager owners,
`ENT-KNOCKBACK-001`, `MOB-SPAWN-001`, `WGEN-PIPELINE-001`, jigsaw/structure owners and `CLI-006`.
