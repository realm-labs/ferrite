# Blocks mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `BLK-MELON-001` — Melon blocks turn stem, terrain and structure fruit into edible slices, seeds and Glistering Melon

**Parent:** `SIM-004`, `SIM-005`, `BLK-001`, `BLK-002`, `BLK-003`, `BLK-005`,
`BLK-PLACE-001`, `BLK-BREAK-001`, `BLK-BREAK-HOOK-001`, `BLK-STEM-CROP-001`,
`PLY-002`, `PLY-005`, `PLY-006`, `PLY-INPUT-001`, `PLY-INTERACT-001`,
`PLY-BREAK-001`, `ITM-001`, `ITM-002`, `ITM-003`, `ITM-004`, `ITM-005`,
`ITM-006`, `ITM-007`, `ITM-USE-001`, `ITM-CONTAINER-001`, `ITM-RECIPE-001`,
`ITM-CRAFT-001`, `ITM-LOOT-001`, `ITM-ADVANCEMENT-001`, `ITM-HUNGER-001`,
`ITM-GLISTERING-MELON-SLICE-001`, `ENT-001`, `ENT-KNOCKBACK-001`, `MOB-AI-001`,
`ENV-001`, `ENV-002`, `ENV-003`, `ENV-FIRE-001`, `WGEN-PIPELINE-001`,
`WGEN-JIGSAW-VILLAGES-001`, `WGEN-STRUCTURE-WOODLAND-MANSION-001`, `CLI-001`,
`CLI-006`, `CLI-UI-001`, `CLI-EFFECT-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked block/item registrations and components, the exact block table,
three recipes and unlocks, Balanced Diet, code-built Composter entries, Farmer records, fruit-stem
owner, natural and village feature records, all 1,212 structure templates and exact client
resources determine every Melon/Melon-Slice-specific branch. Generic food use, block lifecycle,
loot, crafting, merchant, Enderman, Sulfur Cube, feature, structure, persistence and rendering
algorithms remain with the cited owners.

**Applies when:**

A `minecraft:melon` block or block item is generated, grown, placed, carried, broken, exploded,
composted, crafted, traded, persisted, synchronized or rendered; or a
`minecraft:melon_slice` stack is emitted, eaten, composted, crafted into a Melon, Melon Seeds or
Glistering Melon Slice, moved, renamed, persisted, synchronized or rendered before and after
component, tag, loot, recipe, advancement, trade, worldgen or resource reload.

**Authoritative state:**

The property-free Melon is block protocol ID `361`, sole/default state `8333` and raw block-item
ID `437`. It is an ordinary full opaque cube with map color `COLOR_LIGHT_GREEN`, default Harp
instrument, hardness/resistance `1/1`, Wood sounds and piston reaction `DESTROY`. It has no block
entity, random tick, fluid property, emission, correct-tool requirement, fire odds or
lava-ignitable property.

Default full-block geometry supplies full collision, selection, occlusion, sturdy faces, redstone
conduction and ordinary suffocation/view-blocking behavior. Friction is `0.6`, speed and jump
factors are `1`, and light emission is zero. Its direct block tags are `enderman_holdable`,
`mineable/axe` and `sword_efficient`; neither tool tag restricts drops. Its common,
nondamageable 64-stack block item has ordinary components and is a direct
`sulfur_cube_archetype/fast_flat` member.

`minecraft:melon_slice` is raw item ID `1135`. It is a common, nondamageable plain `Item` with
maximum stack `64`, no direct tags and these operational defaults:

- `minecraft:food={nutrition:2,saturation:1.2}` with omitted/default
  `can_always_eat=false`;
- the default food consumable: `consume_seconds=1.6` (`32` ticks), `EAT` animation, generic-eat
  sound, consume particles and no consume effects.

Both items otherwise have the ordinary empty modifiers, enchantments and lore, item-break sound,
translated name, direct item-model key, repair cost, swing animation, tooltip display and use
effects. Neither has durability, repairable, equipment, tool, projectile, cooldown, remainder,
inventory-tick or identity-specific glint state.

**Transition and ordering:**

### Slice consumption and progression

In-air Melon-Slice use enters the consumable path only when the food listener admits it. Ordinary
survival at food level `20` returns `FAIL`; lower hunger admits use. Block interaction remains
block-first. Removing only food leaves the consumable and admits use at full hunger but applies no
nutrition. Removing only consumable makes in-air use pass. Patched food/consumable values control
later uses.

Interruption, release or live-hand/component replacement before completion commits no statistic,
criterion, food, event or shrink. Successful server completion applies the generic eat
transaction: final consume effects, used-item statistic, pre-shrink `consume_item` criterion,
food, `EAT`, then one-item consumption unless the user has infinite materials. The default adds
nutrition `2` and saturation `1.2`, clamped by the hunger owner, with no probability draw, status
effect or remainder.

Melon Slice is one of the 40 independent AND requirements in the telemetry-enabled
`husbandry/balanced_diet` challenge. Its pre-shrink consume criterion advances that row; all 40
rows are required for the `100`-XP reward.

### Melon break and exact loot transform

The one-roll `minecraft:blocks/melon` table evaluates an alternatives entry:

1. a tool with Silk Touch level at least one emits one default Melon block item and stops the
   alternatives branch;
2. otherwise it creates Melon Slices, replaces count with an inclusive uniform integer `3..7`,
   adds `nextInt(FortuneLevel+1)` through `uniform_bonus_count` multiplier one when a tool context
   is present, caps the count at `9`, then applies `explosion_decay`.

The Silk branch has no explosion-decay function. The non-Silk ordering is therefore base draw,
Fortune addition, cap and only then explosion decay. Hand, Axe and Sword all reach the same
non-Silk branch unless the actual tool carries Silk Touch; the two efficiency tags change mining
speed, not loot admission. Arbitrary effective Fortune levels cannot raise the pre-explosion
count above nine. The named random sequence is `minecraft:blocks/melon`.

Ordinary player break removes the state before loot delivery under the block-break owner. Piston
movement destroys rather than moves the state. A live `enderman_holdable` membership can admit
generic Enderman pickup and later placement, preserving block identity but creating no item.
Tag reload changes later speed, carry and Sulfur-Cube tests without revisiting existing blocks or
stacks.

### Three slice recipes and their unlocks

Exactly three recipes directly consume Melon Slice:

- `melon` is shapeless and consumes nine exact slices to emit one default Melon block item;
- `melon_seeds` is shapeless and consumes one exact slice to emit one default Melon Seeds;
- `glistering_melon_slice` is a full `3×3` shaped recipe with the slice at center and eight Gold
  Nuggets around it, emitting one default Glistering Melon Slice.

The two shapeless recipes reject extra occupied slots. The shaped recipe is mirror- and
rotation-invariant but requires the centered slice and all eight nuggets. Exact ingredients ignore
component patches and no input components are copied. None returns a remainder.

Each recipe has one no-display recipe advancement with one OR requirement containing exact Melon
Slice possession and prior unlock of that same recipe. Either criterion grants only its matching
recipe. Thus one observed slice can unlock all three listeners, while possessing Melon, Melon
Seeds, Gold Nuggets or Glistering Melon Slice does not satisfy their inventory criterion.
Glistering-Melon brewing, portal/Farmer acquisition and Piglin-loved behavior remain
`ITM-GLISTERING-MELON-SLICE-001`; seed planting, animal food and seed acquisition remain
`BLK-STEM-CROP-001`.

### Composter, trade and Sulfur-Cube boundaries

Composter admission is code-built by exact item identity. Melon Slice has Java float chance
`0.5f`; the Melon block item has `0.65f`. An admitted direct or automated attempt at level zero
succeeds without RNG. At levels `1..6`, success is strict `nextDouble() < chance`; level-seven
extraction and delayed conversion remain generic. The values do not follow the nine-slice recipe
ratio. Melon Seeds separately remain `0.3f` under the stem owner, and Glistering Melon Slice is
not registered.

Neither Melon Slice nor Melon is furnace fuel. The placed block has FireBlock odds `0/0` and lacks
the lava-ignitable property. The Melon block item selects `fast_flat`, whose locked archetype
supplies horizontal/vertical knockback powers `0.9125/0.09`, hit/push sounds, push cooldown `0.9`,
impulse threshold `0.03` and five attribute modifiers. Loose slices do not match that archetype.

Baseline Farmer level three contains exactly `emerald_cookie` and `melon_emerald`, and its
amount-two no-duplicate set therefore guarantees both while randomizing order. The Melon offer
wants four matching Melon block items, gives one default Emerald, has maximum uses `12`, grants
`20` Villager XP and uses reputation discount `0.05`. It consumes blocks rather than slices:
when all four sale blocks are crafted, they cost exactly 36 slices. The optional Trade Rebalance
overlay has no replacement for this set.

### Growth, natural terrain and structure acquisition

Mature Melon Stem growth is the renewable block source. Its admitted fruit transaction writes
default state `8333` at the selected side before replacing the source stem with an attached stem;
brightness, crop speed, direction, target/support and ignored-write boundaries remain
`BLK-STEM-CROP-001`. Looted or traded Melon Seeds are therefore indirect slice sources only after
planting, growth and block break.

Configured feature `melon` is `simple_block` with a simple provider for state `8333`. Its two
placed records share in-square, `MOTION_BLOCKING` heightmap, biome, count `64`, trapezoid random
offsets `x/z=-7..7` and `y=-3..3`, then require a replaceable empty-fluid target over Grass Block.
`patch_melon` first passes rarity `1/6` and is scheduled in Jungle and Bamboo Jungle;
`patch_melon_sparse` first passes rarity `1/64` and is scheduled in Sparse Jungle. Each surviving
candidate offers one Melon state through the generic simple-block transaction.

Configured/placed feature `pile_melon` uses the audited `block_pile` algorithm, a simple Melon
provider and no placement modifiers. It appears as one weight-one feature element in each ordinary
and zombie Savanna village decor pool; the combined village feature inventory expands to two Melon
pile entries. Radius, traversal, air/support and write draws remain `WGEN-PIPELINE-001`.

An exhaustive decode of all 1,212 locked structure templates finds 17 raw Melon cells:
16 in woodland-mansion `1x2_a8` and one in
`village/savanna/houses/savanna_small_farm`. No template contains a Melon Slice, Melon Seeds or
Glistering Melon Slice string. Template selection, transforms, processors, clipping and accepted
writes remain with the mansion/village owners; raw cells are not unconditional placements.

There is no direct chest, archaeology, fishing, gift, entity-drop, villager, wandering-trader,
brewing or dispenser source for ordinary Melon Slice. Those searches do find seed inputs,
Glistering-Melon acquisition and the block-buying Farmer sink already assigned above, but none
emits the loose edible identity.

**Persistence and reload boundary:**

Chunk palettes persist only Melon state `8333`; there is no block entity, pending stem link,
feature attempt, loot draw, Enderman intent or recipe ratio. Stacks persist identity, count and
component patches. Active use, hunger, knowledge, Composter, merchant, entity and worldgen state
belongs to those owners.

Loot reload changes later break results; recipe/advancement reload changes future matching and
listeners; tag reload changes future tool, Enderman and archetype tests; trade reload changes
future Farmer sets; worldgen reload changes future features/biomes/pools. Existing blocks, stacks,
offers and chunks are not replayed. Code-built food/consumable and Composter entries remain fixed
until code/registry reconstruction. Resource reload independently controls language and models.

**Wire and client projection:**

Generic block-state publication uses state `8333`; stack codecs use raw item IDs `437` and `1135`,
counts and component patches. No Melon-specific packet exists.

English names are `Melon` and `Melon Slice`. The property-free blockstate and block item both
select one opaque `cube_column` model with `melon_top` on up/down and `melon_side` on four sides.
Melon Slice selects its same-named `item/generated` flat model and texture. No tint, conditional
model or special renderer applies.

Natural Blocks places Melon after Wet Sponge and before Pumpkin. Food & Drinks places Melon Slice
after Enchanted Golden Apple and before Sweet Berries. Neither appears in another ordinary tab.
Wood break/step/place/hit/fall sound IDs are `1853/1857/1856/1855/1854`.

**Branches and aborts:**

Food/consumable component combinations; hunger/full/infinite/interrupted completion; Balanced
Diet; Silk versus slice loot, tool absence, Fortune level, cap and explosion decay; three recipe
shapes and unlock alternatives; loose/block/seed/glistering Composter distinctions; nonfuel/fire
boundaries; Enderman/archetype tags; Farmer offer; stem growth; common/sparse patches, pile and
17 template cells; persistence/reload/wire/client paths are distinct.

**Constants and randomness:**

Block/item/state `361/437/8333`; slice ID `1135`; stack `64`; block strength `1/1`; food
`2/1.2`; use `1.6` seconds/`32` ticks; Balanced Diet `40` and `100` XP; loot base `3..7`,
Fortune addition `0..L`, cap `9`; recipes `9:1`, `1:1` seed and `8+1:1` glistering; Composter
`0.5f/0.65f`; fast-flat `0.9125/0.09/0.9/0.03`; Farmer `4:1`, uses `12`, XP `20`;
patch rarity `6/64`, count `64`, offsets above; structure cells `16+1`.

**Side effects:**

Food use and advancement; block placement/break/loot/piston and Enderman relocation;
crafting/knowledge; Composter level/item/effects; Sulfur-Cube selection; Farmer offer/trade;
stem/feature/structure block writes; stack/chunk persistence and synchronization; exact client
projection.

**Gates:**

Identity/components/hunger/use; block/tool/enchantment/explosion; active recipe/grid/knowledge;
Composter level/draw; live block/item tags; profession/trade inputs; stem and worldgen
admission/write; registry/stack/chunk decode; language/model/tab bootstrap.

**State read/written:**

Reads all gates above and writes only the active-use, hunger, advancement, block, loot, crafting,
knowledge, Composter, archetype, offer, generated-Melon, durable, wire and projection state listed
above.

**Failure behavior:**

Full-hunger ordinary use fails; interruption commits nothing. Failed loot alternatives or
explosion decay emit less or nothing only as specified. Wrong grid, extra input or unavailable
recipe emits no result. Failed Composter probability leaves level/count unchanged. Neither item
ignites a furnace; Melon does not ignite/spread through FireBlock. Failed trade, stem, feature or
structure gates consume/write nothing. Reload affects future evaluation only.

**Boundary cases and quirks:**

Nine slices compact to one Melon, but ordinary non-Silk break starts at only `3..7`; Fortune can
recover at most nine before explosion decay. Silk Touch bypasses both the slice functions and
explosion decay. The block has Axe/Sword efficiency tags but no correct-tool gate. Slice and block
Composter probabilities are `0.5` and `0.65`, not scaled by `9:1`. The Farmer buys blocks, while
natural, stem and structure sources create blocks; only block loot creates ordinary loose slices.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-DATA-001`; `OFF-REPORT-001`;
`net.minecraft.world.level.block.Blocks`;
`net.minecraft.world.item.Items`;
`net.minecraft.world.food.Foods`;
`net.minecraft.world.item.component.Consumables`;
`net.minecraft.world.level.block.ComposterBlock#bootStrap`;
`net.minecraft.world.level.storage.loot.functions.ApplyBonusCount#run`;
`net.minecraft.world.level.storage.loot.functions.ApplyBonusCount$UniformBonusCount#calculateNewCount`;
`net.minecraft.world.item.CreativeModeTabs#bootstrap`;
`reports/registries.json#minecraft:{block,item,recipe,recipe_serializer,loot_table,advancement,villager_trade,trade_set,worldgen}`;
`reports/blocks.json#minecraft:melon`;
`reports/minecraft/components/item/{melon,melon_slice}.json`;
`data/minecraft/tags/{block/{enderman_holdable,mineable/axe,sword_efficient},item/sulfur_cube_archetype/fast_flat}.json`;
`data/minecraft/sulfur_cube_archetype/fast_flat.json`;
`data/minecraft/loot_table/blocks/melon.json`;
`data/minecraft/recipe/{melon,melon_seeds,glistering_melon_slice}.json`;
`data/minecraft/advancement/{husbandry/balanced_diet,recipes/{building_blocks/melon,misc/melon_seeds,brewing/glistering_melon_slice}}.json`;
`data/minecraft/{villager_trade/farmer/3/melon_emerald,tags/villager_trade/farmer/level_3,trade_set/farmer/level_3}.json`;
`data/minecraft/worldgen/{configured_feature/{melon,pile_melon},placed_feature/{patch_melon,patch_melon_sparse,pile_melon},biome/{jungle,bamboo_jungle,sparse_jungle},template_pool/village/savanna/{decor,zombie/decor}}.json`;
`data/minecraft/structure/**/*.nbt`;
`assets/minecraft/{blockstates/melon,items/{melon,melon_slice},models/{block/melon,item/melon_slice}}.json`;
`assets/minecraft/textures/{block/{melon_top,melon_side},item/melon_slice}.png`;
`BLK-STEM-CROP-001`; `ITM-GLISTERING-MELON-SLICE-001`; `WGEN-PIPELINE-001`;
`WGEN-JIGSAW-VILLAGES-001`; `WGEN-STRUCTURE-WOODLAND-MANSION-001`;
`EXP-BLK-086`.

**Test vectors:**

Run `EXP-BLK-086` across default and patched slice stacks at every hunger/count/use boundary and
complete its Balanced-Diet row. Place and break Melon by hand, Axe and Sword with absent/present
Silk Touch, every Fortune level, explosion contexts and exact RNG endpoints. Exercise all three
recipes, malformed grids, patched ingredients, output capacity and both unlock routes.

Compare Slice, Melon, Melon Seeds and Glistering Melon Slice in every Composter level/draw; assert
fuel/fire negatives, Enderman membership, fast-flat selection and both level-three Farmer offer
orders. Run stem fruiting, both patch schedules, both village piles and every decoded template.
Persist/reload/synchronize each owner and assert IDs, names, sounds, models, textures and tab
positions.

**Limits:**

Generic block lifecycle/mining, loot execution, active food use, hunger, crafting, advancement,
Composter, Enderman AI, Sulfur-Cube behavior, merchant economy, stem growth, feature/structure
execution, packet encoding and rendering remain with their cited owners. This leaf fixes the
Melon block and Melon Slice selectors, constants, data joins, negative joins and exact projection.
