# Blocks mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `BLK-SUGAR-CANE-001` — Sugar Cane ages into three-block columns beside water and anchors paper and sugar production

**Parent:** `SIM-004`, `SIM-005`, `SIM-RANDOM-001`, `BLK-001`,
`BLK-STATE-001`, `BLK-002`, `BLK-PLACE-001`, `BLK-BREAK-001`,
`BLK-BREAK-HOOK-001`, `BLK-BREAK-CONTENT-001`, `BLK-UPDATE-001`,
`PLY-002`, `PLY-005`, `PLY-006`, `PLY-INTERACT-001`, `PLY-BREAK-001`,
`PLY-COLLISION-001`, `PLY-AUTOJUMP-001`, `RED-001`, `RED-UPDATE-001`,
`RED-COMPARATOR-001`, `ITM-003`, `ITM-004`, `ITM-006`,
`ITM-RECIPE-001`, `ITM-CRAFT-001`, `ITM-LOOT-001`,
`ITM-ADVANCEMENT-001`, `ITM-ANVIL-001`, `ENT-001`, `MOB-AI-001`,
`ENV-001`, `ENV-002`, `ENV-003`, `ENV-FLUID-001`, `ENV-FIRE-001`,
`ENV-LIGHT-001`, `WGEN-003`, `WGEN-PIPELINE-001`, `CLI-001`,
`CLI-006`, `CLI-UI-001`, `CLI-EFFECT-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked `SugarCaneBlock`, registration, loot, recipes,
advancements, Composter and Wandering-trader bootstraps, tags, four placement
profiles, 43 biome records, huge-fungus configuration, all `1,212` templates,
legacy fixes and exact client assets close every Sugar Cane state and direct
join. Growth consists of two independently accepted writes rather than an
atomic column transaction, and support failure is deliberately deferred to a
scheduled one-tick recheck.

**Applies when:**

`minecraft:sugar_cane` is placed, supported, updated, randomly ticked, mined,
exploded, piston-destroyed, composted, crafted, traded, generated, replaced by
a Huge Fungus, migrated, persisted, synchronized or rendered.

**Authoritative state:**

Sugar Cane is a `SugarCaneBlock` with integer property `age=0..15`, no block
entity and exactly 16 states. State IDs `6947..6962` correspond monotonically
to ages `0..15`; default state `6947` has age `0`. Block, block-type and item
IDs are `282/208/284`. Its ordinary common `BlockItem` stacks to 64 and has
only generic item components, name and model references.

Registration fixes `PLANT` map color, default `HARP`, `noCollision`,
`randomTicks`, instant break, Grass sounds and piston reaction `DESTROY`.
Strength/resistance are `0/0`; friction is `0.6`, speed/jump factors are `1`,
emission is `0`, and no correct tool is required.

The selection outline is the centered full-height column
`[2/16,0,2/16]..[14/16,1,14/16]`. Collision, support and visual-occlusion
shapes are empty under the no-collision/no-occlusion registration. The state
does not form an opaque cube, contains Empty fluid, and supplies no signal or
comparator output. Grass profile volume/pitch is `1/1`; break/fall/hit/place/
step sound IDs are `755/756/757/758/759`.

**Transition and ordering:**

### Placement and live survival

Generic block-item placement supplies default `age=0`; there is no custom
placement-state transform. At position `P`, `canSurvive` evaluates in this
order:

1. if `P.below()` is exact Sugar Cane, return true without reading substrate
   or water;
2. otherwise require the below state in block tag `supports_sugar_cane`; and
3. scan the four horizontal neighbors of that below-support position, accepting
   if any neighbor fluid is in fluid tag `supports_sugar_cane_adjacently` or
   its block is in the same-named block tag.

The substrate tag expands through seven nested tags to exactly 13 blocks:
Coarse Dirt, Dirt, Grass Block, Moss Block, Mud, Muddy Mangrove Roots,
Mycelium, Pale Moss Block, Podzol, Red Sand, Rooted Dirt, Sand and Suspicious
Sand. The adjacent-fluid closure is the one nested `water` tag and exactly
Water plus Flowing Water. The adjacent-block closure contains only Frosted
Ice. Consequently source, flowing and waterlogged Water qualify through live
fluid state, while dry Frosted Ice qualifies through block identity.

An upper segment supported by exact Cane bypasses every substrate/water read.
Removing the base environment makes the base fail but does not directly make
upper segments fail until their immediate lower Cane disappears and they are
updated.

On a neighbor-shape update, an unsupported segment schedules this block at its
own position after one tick, then delegates to the generic update result; it
does not immediately return Air. The scheduled server tick rechecks survival.
If still unsupported it calls `destroyBlock(P,true)` and ignores the result,
requesting ordinary drops; restored support makes the tick a no-op. Neighbor
notifications from successful destruction can schedule later segments, so a
column may dismantle in successive one-tick waves.

### Selected random-tick growth

The random-tick scheduler owns whether this randomly ticking state is selected.
Once `randomTick` runs, Sugar Cane consumes no local random number:

1. require `P.above()` to be empty;
2. count exact consecutive Sugar Cane downward from distance one, starting
   with height `1`, until the first non-Cane state;
3. require the resulting column height to be less than `3`; and
4. read this segment's current age.

An occupied top or height at least three returns without changing age. Because
age belongs to each segment, only the selected top segment with empty space
above normally progresses.

For ages `0..14`, the block offers the same state with `age+1` at `P` using
flags `260` and ignores the Boolean result. At age `15`, ordering is:

1. call `setBlockAndUpdate(P.above(), default age-zero Sugar Cane)` and ignore
   its result; then
2. call `setBlock(P, current state with age=0, 260)` and independently ignore
   its result.

There is no rollback. A rejected upper write can still reset the lower state;
an accepted upper write followed by a rejected reset can leave an age-15 lower
segment with a new age-zero upper segment. Earlier reads and accepted writes
remain committed in either case.

### Fluids, mining, explosion, piston and fire

`FlowingFluid.canHoldAnyFluid` explicitly rejects exact Sugar Cane even though
its no-collision state would otherwise pass the generic non-motion-blocking
test. Flowing-fluid placement therefore cannot retain fluid in or generically
occupy the Cane state through that holder branch. Surrounding fluid changes
still reach the ordinary neighbor/update and survival owners.

Every tool mines instantly and shares the one-roll self loot table: one Sugar
Cane behind `survives_explosion`, using random sequence
`minecraft:blocks/sugar_cane`. Silk Touch and Fortune do not alter the result;
explosion decay can suppress it. Piston reaction `DESTROY` selects destruction
rather than movement. There is no Fire bootstrap row, lava-ignition property
or fuel duration, so encouragement/flammability and burn time are zero.

There is no entity-contact, attack, fall, use, transform, signal, comparator
or block-event override.

### Recipes, advancement, composting and trade

The shaped `paper` recipe contains the single row `###`; three exact Sugar
Cane across one row produce three Paper. The shapeless
`sugar_from_sugar_cane` recipe consumes one exact Cane and produces one Sugar
in group `sugar`. Each has a no-display recipe advancement whose single
requirement is an OR between inventory possession of Sugar Cane and the exact
recipe-unlocked trigger. No other locked recipe names Sugar Cane exactly.

Composter bootstrap assigns exact Sugar Cane chance `0.5f`. Generic Composter
semantics retain the guaranteed level-zero increase and, at levels one through
six, advance only when the widened random double is strictly below `0.5`;
level-seven behavior remains with the Composter owner.

Wandering-trader record `emerald_sugar_cane` wants one Emerald and gives one
Sugar Cane, with maximum uses `8`, merchant XP `1` and reputation discount
multiplier `0.05`. It occurs five times in the 76-entry `common` trade set, so
five independently addressable units select this offer under that owner's
weighted draw. Self loot, this trade, world generation, Creative and commands
are baseline sources; recipes and composting are sinks.

### World generation

Configured feature `sugar_cane` is a vertical `block_column` pointing upward.
Its one layer uses default age-zero Cane, admits only Air and samples a
`biased_to_bottom` height from `2..4`; `prioritize_tip` is false. The exact
sampler is:

`2 + nextInt(nextInt(3) + 1)`.

Therefore heights `2/3/4` have probabilities `11/18`, `5/18` and `1/9`.
The column algorithm offers consecutive blocks while the allowed-placement
predicate and generic writes admit them; obstruction/write semantics and the
false tip-priority branch retain `WGEN-PIPELINE-001`.

Four placed features wrap that configuration. Normal, Badlands and Swamp
variants first pass rarity filters `1/6`, `1/5` and `1/3`; Desert has no rarity
filter. Each then applies square spread, `MOTION_BLOCKING` heightmap, biome
filter, count `20`, trapezoidal X/Z offsets `-4..4` with Y `0`, and an `all_of`
predicate requiring Air, default-Cane survival, and exact Water or Flowing
Water at any of the four below-horizontal offsets.

Exactly 43 biome records schedule a Sugar Cane variant: Desert uses Desert,
Swamp uses Swamp, Badlands/Eroded Badlands/Wooded Badlands use Badlands, and
the other 38 use Normal. Rarity and 20-attempt expansion therefore compose
after each biome's vegetation-decoration scheduling rather than defining a
global density.

The Crimson, Crimson-planted, Warped and Warped-planted Huge-Fungus configured
features also include exact Sugar Cane in their stem-candidate replacement
predicate. Hat/decorator positions still require Air. Planted configurations
destroy a non-Air stem candidate with drops before writing; non-planted
configurations directly offer the replacement. Thus only the stem path
selects Cane, and the planted/non-planted destruction asymmetry remains with
the Huge-Fungus owner.

### Tags, structures and migration

Sugar Cane itself has no direct locked block or item tag, so its membership
closures are `0/0`. The support closures are instead seven nested block tags
to 13 substrates, one nested fluid tag to two fluids, and one adjacent block
tag to Frosted Ice. Reloading these tags changes subsequent placement,
updates and feature survival checks.

Exhaustive decoded and string scans of all `1,212` structure templates find
zero raw Sugar Cane cells and zero palette/final-state/marker/block-entity/
entity-NBT occurrences. Its worldgen is feature-driven.

Legacy flattening preserves four exact paths. `BlockStateData` maps packed
numeric block states `1328..1343` (`83 << 4 | metadata 0..15`) and old
`minecraft:reeds` ages `0..15` to the modern states. `EntityBlockStateFix`
maps the old block name to numeric block ID `83`; `ItemIdFix` maps numeric item
ID `338` to `minecraft:reeds`; and `ItemStackTheFlatteningFix` maps
`minecraft:reeds.0` to `minecraft:sugar_cane`.

Chunk palettes and block packets persist the age state ID. Stacks retain
identity, count and generic component patches; no block entity or Sugar-Cane
component exists.

### Client projection

All 16 age variants select the same unrotated block model. Its
`tinted_cross` parent disables ambient occlusion and renders two crossed,
double-sided, unshaded quads spanning approximately pixel coordinates
`0.8..15.2` horizontally and `0..16` vertically, with tint index `0`.
World-aware block color is the biome-averaged grass color at the position;
the context-free block-color path returns opaque white. Authoritative
selection outline remains the centered 12-pixel column and is not derived
from the crossed render planes.

The item uses a separate generated flat `item/sugar_cane` texture with no
predicate, component branch or biome tint. English name is `Sugar Cane`.
Natural Blocks publishes it once after Bamboo and before Cactus.

**Branches and aborts:**

- Exact lower Cane short-circuits substrate and adjacent-water checks.
- Failed live survival schedules a one-tick recheck; only continued failure
  requests drop-producing destruction.
- Selected growth requires empty top and column height below three.
- Age increment, upper placement and lower reset all ignore write results;
  the two age-15 writes are independent and upper-first.
- Feature rarity, placement predicates, obstruction and writes gate each
  candidate before a segment exists.
- Recipes, Composter and trader selection retain independent admission and
  result gates.

**Constants and randomness:**

States `6947..6962`; block/block-type/item IDs `282/208/284`; outline
`12×16×12` pixels; support closures `7→13`, `1→2`, `1→1`; scheduled support
delay `1`; height cap `3`; reset age `15`; flags `260`; self count `1`;
Composter `0.5f`; Paper `3→3`; Sugar `1→1`; trader `5/76`, `1:1`, max uses
`8`, XP `1`, discount `0.05`; feature heights `2/3/4` at
`11/18,5/18,1/9`; rarity `1/6,1/5,1/3,1`; attempts `20`; biomes `43`;
memberships `0/0`; templates/cells `1212/0`.

**Side effects:**

State placement, aging, upper growth and reset; scheduled ticks and
drop-producing destruction; mining/explosion/piston loot; crafting knowledge
and outputs; Composter levels; merchant offers; feature placement and fungus
replacement; migration, persistence, synchronization and client projection.

**Gates:**

Generic placement/write; live support and horizontal fluid/block tags;
neighbor notification and delayed recheck; random-tick selection, top
occupancy, column height and write admission; tool/explosion/piston/fluid;
recipe grid and knowledge; Composter level/draw; trade-set draw; biome,
rarity, spread, heightmap, placement predicates and feature writes; legacy
schema and resource validity.

**Boundary cases and quirks:**

Upper Cane can stay provisionally valid after base water disappears because
exact lower Cane bypasses the environment scan. Support loss is delayed, not
an immediate update-to-Air. Age-15 growth can leave split partial states when
either write is rejected. Waterlogged neighbors qualify through fluid state,
but the Cane itself cannot hold flowing fluid through the generic holder
path. Every age renders identically despite affecting future growth.

**Failure semantics:**

Placement, scheduled ticks, loot, recipes, composting, trades and worldgen
retain their generic owners. Sugar Cane ignores growth and scheduled-destroy
results; successful earlier writes and scheduler/RNG effects do not roll
back. Planted Huge Fungus can destroy with drops before a rejected replacement
write, while non-planted replacement has no preceding destruction.

**Client/server authority split:**

The server owns survival, ticks, growth, destruction, loot, recipes,
Composter, trades, worldgen, migration and persistence. The client predicts
placement and renders the biome-tinted crossed block, flat item, name and tab
entry. Authoritative states, stacks and effects synchronize.

**Observability:**

Observe IDs/age, shapes/light/path/redstone/piston, ordered support reads,
scheduled tick and destruction, random-tick height scan and independent
writes, fluid exclusion, all loot/recipe/compost/trade results, feature
sampling and predicates, fungus replacement order, complete closures/template
census, migrations, persistence and exact block/item projection.

**Persistence and reload:**

Age persists in chunk palettes and synchronizes by state ID; stacks use
generic components. Tags, loot, recipes, advancements, trades, worldgen and
client resources reload through their owners. Registration, growth, fluid
exclusion, Composter odds, legacy mappings and Creative ordering are
code-built.

**Evidence:**

`net.minecraft.world.level.block.Blocks`;
`net.minecraft.world.level.block.SugarCaneBlock`;
`net.minecraft.world.level.material.FlowingFluid`;
`net.minecraft.world.level.block.ComposterBlock`;
`net.minecraft.world.entity.npc.VillagerTrades`;
`net.minecraft.world.level.levelgen.feature.BlockColumnFeature`;
`net.minecraft.util.valueproviders.BiasedToBottomInt`;
`net.minecraft.data.worldgen.features.VegetationFeatures`;
`net.minecraft.data.worldgen.placement.VegetationPlacements`;
`net.minecraft.data.worldgen.biome.OverworldBiomes`;
`net.minecraft.util.datafix.fixes.BlockStateData`;
`net.minecraft.util.datafix.fixes.EntityBlockStateFix`;
`net.minecraft.util.datafix.fixes.ItemIdFix`;
`net.minecraft.util.datafix.fixes.ItemStackTheFlatteningFix`;
`net.minecraft.client.color.block.BlockColors`;
`net.minecraft.client.data.models.BlockModelGenerators`;
`net.minecraft.world.item.CreativeModeTabs`; reports, tags, loot, both
recipes/advancements, trade records, all feature/biome records, all `1,212`
templates, blockstate/model/item/texture/language resources and complete
compiled/data/fix/NBT identity searches.

**Test vectors:**

Run `EXP-BLK-124` across all 16 states, every substrate/adjacency/update/tick/
height/write-result branch, tools/explosion/piston/fluid holding, recipes/
unlocks/Composter/trade, four patch profiles in all 43 biomes, four fungus
configurations, all closures/templates/fixes, persistence/reload and exact
client projection. Assert IDs, ordering, constants, probabilities, absences
and vanilla convergence.

**Limits:**

Generic placement, random-tick selection, block writes, loot, crafting,
advancements, Composter, merchant, feature pipeline, Huge Fungus, packets and
rendering retain their owners. Substrate blocks, Water, Frosted Ice, Paper and
Sugar retain their catalog families. This leaf fixes exact Sugar Cane and
every direct join selecting it.
