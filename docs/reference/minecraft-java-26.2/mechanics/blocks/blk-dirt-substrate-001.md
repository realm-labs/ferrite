# Blocks mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `BLK-DIRT-SUBSTRATE-001` — Overworld dirt substrates couple snow state, spreading, tool conversion and terrain roles

**Parent:** `SIM-004`, `SIM-005`, `SIM-RANDOM-001`, `BLK-001`,
`BLK-STATE-001`, `BLK-002`, `BLK-PLACE-001`, `BLK-BREAK-001`,
`BLK-BREAK-HOOK-001`, `BLK-BREAK-CONTENT-001`, `BLK-UPDATE-001`,
`PLY-002`, `PLY-005`, `PLY-006`, `PLY-INPUT-001`,
`PLY-INTERACT-001`, `PLY-BREAK-001`, `PLY-COLLISION-001`,
`PLY-AUTOJUMP-001`, `ITM-003`, `ITM-004`, `ITM-006`,
`ITM-RECIPE-001`, `ITM-CRAFT-001`, `ITM-LOOT-001`,
`ITM-ADVANCEMENT-001`, `ITM-ANVIL-001`, `ITM-DISPENSER-001`,
`ENT-001`, `MOB-001`, `MOB-AI-001`, `MOB-SPAWN-001`, `ENV-001`,
`ENV-002`, `ENV-003`, `ENV-FLUID-001`, `ENV-FIRE-001`,
`ENV-LIGHT-001`, `WGEN-003`, `WGEN-PIPELINE-001`,
`WGEN-JIGSAW-PROCESSORS-001`, `WGEN-JIGSAW-VILLAGES-001`,
`WGEN-JIGSAW-TRAIL-RUINS-001`, `WGEN-JIGSAW-TRIAL-CHAMBERS-001`,
`WGEN-STRUCTURE-ANCIENT-CITY-001`, `WGEN-STRUCTURE-WOODLAND-MANSION-001`,
`CLI-001`, `CLI-006`, `CLI-UI-001`, `CLI-EFFECT-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — the locked reports and registrations fix six identities,
three property-free states and three two-state snowy blocks. Source inspection
closes snow maintenance, Grass Block/Mycelium decay and spread, both
bonemeal implementations, shovel/hoe/water-bottle transformations, mob and
trade joins, loot, recipe, tag, terrain and client branches. A decoded scan
of all 1,212 structure templates separates 8,461 raw target cells from 526
Jigsaw `final_state` strings and finds no target block-entity payload.

**Applies when:**

`minecraft:grass_block`, `minecraft:dirt`, `minecraft:coarse_dirt`,
`minecraft:podzol`, `minecraft:mycelium` or `minecraft:rooted_dirt` is
placed, updated, random-ticked, bonemealed, transformed by a tool or water
bottle, mined, generated, traded, persisted, synchronized or rendered.

**Authoritative state:**

None of the six blocks has a block entity. Their exact registrations are:

| Identity | Block ID | State ID(s) | Default | Item ID | Implementation | Map color | Strength/resistance | Sound |
| --- | ---: | --- | --- | ---: | --- | --- | --- | --- |
| Grass Block | `8` | `8` true, `9` false | `9` | `54` | `GrassBlock` | Grass | `0.6/0.6` | Grass |
| Dirt | `9` | `10` | `10` | `55` | ordinary `Block` | Dirt | `0.5/0.5` | Gravel |
| Coarse Dirt | `10` | `11` | `11` | `56` | ordinary `Block` | Dirt | `0.5/0.5` | Gravel |
| Podzol | `11` | `12` true, `13` false | `13` | `57` | `SnowyBlock` | Podzol | `0.5/0.5` | Gravel |
| Mycelium | `373` | `8918` true, `8919` false | `8919` | `450` | `MyceliumBlock` | Purple | `0.6/0.6` | Grass |
| Rooted Dirt | `1149` | `30414` | `30414` | `58` | `RootedDirtBlock` | Dirt | `0.5/0.5` | Rooted Dirt |

The Boolean column is the sole `snowy` property. Grass Block and Mycelium
enable random ticks; the other four do not. All six are opaque full cubes
with Harp note instrument, friction `0.6`, speed/jump factors `1`, light `0`,
normal piston reaction, ordinary survival and no scheduled tick. None is
lava-ignitable.

Grass sound-event IDs for break/step/place/hit/fall are
`755/759/758/757/756`; Gravel uses `760/764/763/762/761`; Rooted Dirt uses
`1387/1391/1390/1389/1388`. All three sound profiles use volume/pitch `1/1`.

Every item is a common stack-64 `BlockItem` with empty attribute,
enchantment and lore defaults. Grass Block, Podzol and Mycelium belong to the
direct item tag `grass_blocks`; Dirt, Coarse Dirt and Rooted Dirt belong to
direct `dirt`. Sulfur Cube archetypes are `regular` for every member except
Mycelium, which is `slow_sliding`.

**Transition and ordering:**

### Snow state

`SnowyBlock` initializes `snowy=false`. Placement reads the block above and
sets `snowy` to whether that state belongs to the live `snow` block tag,
whose baseline members are Snow, Snow Block and Powder Snow. Only an
`UP`-direction shape update recomputes the property from the supplied
neighbor state; all other directions retain the incoming value. Rotation
and mirroring do not change it.

That maintenance applies to Grass Block, Podzol and Mycelium. It is a visual
state and does not itself select survival. Save/load and block-state packets
preserve the Boolean exactly, including command-authored combinations.

### Grass Block and Mycelium decay/spread

Both spreading blocks resolve their base-block registry key on each random
tick. The key is exact Dirt for both; a missing registry value aborts before
world mutation.

Survival at the source first reads the block above:

- exact one-layer Snow returns true immediately;
- otherwise a full fluid above returns false;
- otherwise `LightEngine.getLightDampeningInto` must be strictly below `15`.

Failure replaces the source with default Dirt through
`setBlockAndUpdate` and returns. A surviving source reads maximum local raw
brightness above; values below `9` stop spreading.

At brightness at least `9`, exactly four attempts run. Each consumes
`nextInt(3)-1`, `nextInt(5)-3`, `nextInt(3)-1` for candidate offsets
`x=-1..1`, `y=-3..1`, `z=-1..1`. A candidate must be exact Dirt. The
prospective Grass Block or Mycelium state must pass the same survival test
there, and the fluid above the candidate must not belong to live `water`.
An admitted candidate is offered with `setBlockAndUpdate`; its `snowy` value
is recomputed from the candidate's above state and the write result is
ignored. Each implementation spreads only itself; neither converts the
other.

Mycelium additionally performs a client animate tick. It always consumes
`nextInt(10)`; zero then consumes two doubles and emits one Mycelium particle
at `(x+double,y+1.1,z+double)` with zero velocity.

### Bonemeal

Grass Block is a neighbor-spreader bonemeal target only when the cell above
is air and inside build height; its success predicate is unconditional once
valid. It starts at the cell above and performs exactly 128 outer attempts.
For outer index `i`, it walks `i/16` times, each step consuming four
`nextInt(3)` draws and adding
`(a-1, (b-1)*c/2, d-1)`. A walk aborts that outer attempt if the block below
the new cursor is not exact Grass Block or the cursor has a full collision
shape.

At the retained cursor, exact Short Grass consumes `nextInt(10)` and, on
zero, invokes that block's bonemeal action if still a valid target. A
non-air cursor or a cursor outside build height then ends the attempt. For an
air cursor, `nextInt(8)==0` reads that biome's bonemeal configured-feature
list, aborts if it is empty, otherwise chooses one list entry uniformly and
places it directly. The other seven results place the optional
`grass_bonemeal` placed feature when present. Feature results are ignored.

Rooted Dirt is valid only when the cell below is air and inside build
height; success is unconditional. Its bonemeal action offers default Hanging
Roots below through `setBlockAndUpdate` and ignores the result. It reports
that below position for bonemeal particles.

Dirt, Coarse Dirt, Podzol and Mycelium are not bonemealable blocks.

### Shovel, hoe and water bottle

`ShovelItem.FLATTENABLES` maps all six identities to default Dirt Path.
Use returns `PASS` when the clicked face is `DOWN`, the cell above is not
air, or the exact block is absent from the map. Otherwise it plays Shovel
Flatten at the target and returns `SUCCESS`; the server writes the path with
flags `11`, emits `BLOCK_CHANGE`, and damages a non-null player's shovel by
one.

The Hoe map instead has these exact branches:

- Grass Block and Dirt become default Farmland only when face is not `DOWN`
  and above is air;
- Coarse Dirt becomes default Dirt under the same predicate;
- Rooted Dirt uses an unconditional predicate, becomes default Dirt and
  additionally pops one Hanging Roots item from the clicked face;
- Podzol and Mycelium are absent and return `PASS`.

An admitted Hoe path plays Hoe Till and returns `SUCCESS`; the server writes
with flags `11`, emits `BLOCK_CHANGE`, then damages a non-null player's Hoe
by one. Both write consumers ignore the write result; Rooted Dirt's item pop
therefore follows even after a rejected write.

The direct `convertable_to_mud` tag contains exactly Dirt, Coarse Dirt and
Rooted Dirt. Using an exact Water potion on a non-`DOWN` face of any member
plays Generic Splash, passes the held stack and a Glass Bottle through
`ItemUtils.createFilledResult`, emits five server Splash particles, plays
Bottle Empty, emits `FLUID_PLACE`, offers default Mud through
`setBlockAndUpdate`, and returns `SUCCESS`. The generic filled-result helper
preserves an infinite-material player's potion while ensuring one Bottle is
present; otherwise it consumes one potion and returns/adds/drops the Bottle
according to remaining count and inventory capacity. A dispenser with an
exact Water potion performs the same identity conversion in its facing cell,
using the dispenser position for particles/sound/event and consuming with a
Glass Bottle remainder; other contents/targets use default dispensing.

**Breaking, fire and fuel:**

All six are directly `mineable/shovel`. None requires the correct tool, so a
hand or any tool remains harvest-eligible.

Dirt, Coarse Dirt and Rooted Dirt each have one explosion-surviving
count-one self drop. Grass Block, Podzol and Mycelium first test Silk Touch
level at least one: success emits the matching block item without an
explosion condition; failure emits one Dirt guarded by
`survives_explosion`. Each table has random sequence
`minecraft:blocks/<identity>` and no Fortune branch.

None of the six appears in `FireBlock.bootStrap`,
`FuelValues.vanillaBurnTimes` or `ComposterBlock.bootStrap`; direct fire
encouragement/flammability are `0/0`, furnace burn time is `0`, and compost
chance is absent. None sets `ignitedByLava`.

**Tags, mobs and progression joins:**

The two exact substrate partitions are reloadable: block/item `dirt`
contains Dirt, Coarse Dirt and Rooted Dirt; block/item `grass_blocks`
contains Grass Block, Podzol and Mycelium. `substrate_overworld` composes
those two tags with Mud and Moss families. All six directly support Big
Dripleaf and are shovel-mineable. Their remaining direct block tags are:

- Grass Block: `animals_spawnable_on`, `foxes_spawnable_on`,
  `frogs_spawnable_on`, `parrots_spawnable_on`, `rabbits_spawnable_on`,
  `sniffer_diggable_block`, `valid_spawn`, `wolves_spawnable_on`;
- Coarse Dirt: `armadillo_spawnable_on`, `foxes_spawnable_on`,
  `wolves_spawnable_on`;
- Podzol: `cannot_replace_below_tree_trunk`,
  `foxes_spawnable_on`, both Huge-Mushroom placement tags,
  `overrides_mushroom_light_requirement`, `sniffer_diggable_block`,
  `valid_spawn`, `wolves_spawnable_on`;
- Mycelium: both Huge-Mushroom placement tags,
  `mooshrooms_spawnable_on`, `overrides_mushroom_light_requirement`,
  `supports_warped_fungus`;
- Dirt and Rooted Dirt add no direct block tag beyond their partition,
  mud-conversion, shovel and Big-Dripleaf tags.

Spawn, support, mushroom-light, Sniffer and plant semantics remain with the
generic consumers of those live tags.

`EatBlockGoal` admits exact Grass Block immediately below after its
baby/adult `1/adjusted(50 or 1000)` start gate. It starts an adjusted
40-tick animation and, at adjusted tick `4`, `mobGriefing=true` emits level
event `2001` for default Grass Block and writes default Dirt with flags `2`;
the mob's `ate` callback runs whether or not griefing was allowed. Generic
animals give walk target value `10` over exact Grass Block, eligible horses
may enter their eating animation after a `1/300` idle draw, and Ocelot spawn
placement accepts Grass Block or Leaves below once its other gates pass.

Mooshrooms give walk target value `10` over exact Mycelium, and their spawn
rule separately uses the live `mooshrooms_spawnable_on` tag plus the generic
brightness gate.

The only recipe in this slice is shaped Coarse Dirt: `DG/GD`, exact Dirt and
Gravel, yields four Coarse Dirt. Its recipe advancement accepts either exact
Gravel possession or existing recipe knowledge and rewards only that recipe.
The Story root uses Grass Block only as its displayed icon; Crafting Table
possession remains its criterion.

The baseline Wandering Trader common tag includes
`emerald_rooted_dirt`: one Emerald buys two Rooted Dirt, maximum uses `5`,
reputation discount `0.05`. Its uncommon tag includes `emerald_podzol`:
three Emeralds buy three Podzol, maximum uses `6`, discount `0.05`. Generic
trade-set selection, pricing and restocking retain their merchant owners.

**World sources:**

The six identities are first-class inputs/outputs of the already audited
worldgen pipeline:

- Overworld/amplified/large-biomes/caves/floating-islands surface rules name
  Grass Block, Dirt, Coarse Dirt, Podzol and Mycelium; the flat presets
  directly name Grass Block and Dirt;
- disk, ice-patch, berry, melon, pumpkin, leaf-litter and village-street
  records select exact family states or test them;
- tree below providers, root systems and alter-ground decorators produce or
  replace Dirt, Rooted Dirt and Podzol; `azalea_tree` and
  `rooted_azalea_tree` directly name Rooted Dirt;
- Bamboo, giant-conifer and jungle vegetation paths produce/test Podzol;
  ore, spring, carver, blending and surface material paths directly consume
  the listed substrates.

Those feature/provider/modifier/surface algorithms, data-record counts and
write semantics remain owned by `WGEN-PIPELINE-001`; this leaf fixes the
selected identities and their live tag partitions.

An exhaustive decoded scan of all 1,212 bundled templates finds:

| Identity | Files with raw cells | Raw cells | Root groups |
| --- | ---: | ---: | --- |
| Grass Block | `145` | `4,915` | Ancient City `3/132`; Village `142/4,783` |
| Dirt | `158` | `3,473` | Ancient City `3/12`; Trial Chambers `1/1`; Village `150/3,352`; Woodland Mansion `4/108` |
| Coarse Dirt | `7` | `67` | Trail Ruins `6/46`; Woodland Mansion `1/21` |
| Podzol | `3` | `6` | Trial Chambers `3/6` |
| Mycelium | `0` | `0` | none |
| Rooted Dirt | `0` | `0` | none |

No raw target cell has block NBT. A separate decompressed-string census finds
`535/294/7/3/0/0` occurrences in family order. Beyond one palette name per
raw-cell file, all `390` extra Grass Block and `136` extra Dirt strings are
decoded Jigsaw `final_state` values: Grass Block only in Village templates;
Dirt in Village and Trail Ruins. They are executable connector-replacement
sources, not hidden item stacks. Jigsaw selection, transforms, processors,
terrain adaptation, clipping and failed writes retain their named owners.

**Client projection:**

Grass Block, Dirt, Podzol, Mycelium and Rooted Dirt non-snowy blockstates
choose uniformly among Y rotations `0/90/180/270`; Coarse Dirt uses one fixed
cube. Snowy Grass Block, Podzol and Mycelium all select the same
`grass_block_snow` cube-bottom-top model.

- Grass Block uses Dirt below/particles, grass top tinted at index `0`,
  untinted side base plus a tint-index-`0` side overlay. Its world tint is
  averaged biome grass color, its inventory fallback is default grass color,
  and terrain particles deliberately return untinted white.
- Dirt, Coarse Dirt and Rooted Dirt are same-texture cubes.
- Podzol and Mycelium use Dirt below and distinct top/side textures without
  tint.
- The snowy shared model uses Dirt below, Grass top and snowy side; it has no
  tint index.

Each item definition directly selects its block model. Grass Block's item
adds the grass tint source with temperature `0.5` and downfall `1.0`; the
other five add no tint. All eleven dedicated textures are static 16x16 PNGs
without animation metadata. Names are `Grass Block`, `Dirt`, `Coarse Dirt`,
`Podzol`, `Mycelium` and `Rooted Dirt`.

The Natural Blocks tab orders this run exactly as Grass Block, Podzol,
Mycelium, Dirt Path, Dirt, Coarse Dirt, Rooted Dirt, Farmland, Mud. Grass
Block is also the tab icon.

**Branches and aborts:**

- Snowy placement/update reads only the above state; random-tick survival
  separately applies the one-layer-Snow, full-fluid and dampening rules.
- A missing Dirt registry base, failed source survival or brightness below
  `9` prevents all spread attempts.
- Every spread attempt consumes all three offset draws before candidate
  identity/survival checks.
- Grass bonemeal can abort a cursor walk, reject a non-air/out-of-height
  target, choose an empty biome feature list, or ignore a failed feature.
- Rooted Dirt bonemeal requires air below; its accepted write result is
  ignored.
- Shovel conversion requires non-`DOWN` face and air above for all six.
  Hoe predicates differ, and Podzol/Mycelium have no Hoe entry.
- Player water conversion rejects `DOWN`, nonmembers and non-Water potions;
  dispenser conversion tests only its facing target after exact-Water
  admission.
- Explosion survival can suppress every non-Silk item branch; Silk outputs
  are not explosion-decayed by these tables.
- Template cells and Jigsaw final states remain conditional on reachability
  and successful placement.

**Constants and randomness:**

IDs/states as tabulated; physical values `0.5` or `0.6`; stack `64`; spread
brightness `9`, four attempts and offset bounds `3/5/3`; dampening threshold
`15`; Mycelium particle chance `1/10`; Grass bonemeal attempts `128`, walk
depth `i/16`, Short-Grass chance `1/10`, biome-feature chance `1/8`; Hoe and
Shovel durability cost `1`; Water conversion particles `5`; raw structure
cells `4,915/3,473/67/6/0/0`; Jigsaw final-state strings `390/136`.

**Side effects:**

State writes and neighbor/light updates; random-tick decay/spread; bonemeal
consumption, vegetation and Hanging Roots; tool damage, sounds, game events
and Hanging Roots item spawn; potion/container mutation, particles and Mud;
loot, recipe knowledge and merchant offers; mob animation/terrain change;
worldgen/template writes; client tint, particles, models and sounds.

**Gates:**

Server random-tick selection; base registry; exact source/candidate identity;
above fluid/light and candidate water; raw brightness; bonemeal target/build
height/collision/biome features; tool map/face/air/player; exact Water potion
and live conversion tag; Silk Touch/explosion; mob RNG/game rule/spawn tags;
trade-set choice; worldgen/template selection and write; resource/tag reload.

**Boundary cases and quirks:**

One-layer Snow is an explicit survival success even though `snowy` accepts
three tag members. Grass Block and Mycelium spread only onto exact Dirt, not
the reloadable `dirt` tag. Their candidate Y interval is asymmetric
`-3..1`. Hoe conversion is deliberately asymmetric: Rooted Dirt ignores
face/air and drops Hanging Roots, while Podzol/Mycelium do nothing. A failed
Rooted Dirt Hoe write does not suppress its item pop. Grass Block's snowy
model shares an untinted snow texture even though the ordinary model uses
biome tint.

**Failure semantics:**

Every local mutation ignores its Boolean write result and adds no rollback.
A random-tick source decay commits before any spread branch. Hoe, Shovel and
Water-potion side effects follow their documented client/server ordering;
failed low-level writes can leave sounds, events, durability/container
changes or the Rooted Dirt item pop already committed. Worldgen and Jigsaw
owners likewise retain earlier writes after a later failure.

**Client/server authority split:**

The server owns random ticks, bonemeal vegetation, tool/potion/dispenser
mutation, loot, mob terrain changes, trades and generation. Clients return
ordinary predicted tool/item success and render synchronized snowy states,
biome tint, particles, models and sounds. Mycelium's ambient particle is a
client animate-tick effect.

**Observability:**

Observe exact state/registry IDs, snow transitions, decay/spread read/draw/
write order, bonemeal cursors and selected features, tool/potion outcomes,
loot and drops, tags/spawns/trades/recipe, raw cells and Jigsaw final states,
persisted/wire states, particles, tint, model rotations, names and tab order.

**Persistence and reload:**

Snowy states persist their Boolean; property-free members persist identity
only. No member has block-entity data. Stacks use generic components.
Block/item/fluid tags, loot, recipe/advancement, trade sets, worldgen and
client resources retain their independent reload boundaries. Registrations,
state IDs, physical profiles and tool maps are code-built.

**Evidence:**

`net.minecraft.world.level.block.Blocks`;
`net.minecraft.world.level.block.SnowyBlock`;
`net.minecraft.world.level.block.SpreadingSnowyBlock#randomTick`;
`net.minecraft.world.level.block.GrassBlock`;
`net.minecraft.world.level.block.MyceliumBlock#animateTick`;
`net.minecraft.world.level.block.RootedDirtBlock`;
`net.minecraft.world.item.ShovelItem#useOn`;
`net.minecraft.world.item.HoeItem#useOn`;
`net.minecraft.world.item.PotionItem#useOn`;
`net.minecraft.core.dispenser.DispenseItemBehavior$13#execute`;
`net.minecraft.world.entity.ai.goal.EatBlockGoal`;
`net.minecraft.world.entity.animal.Animal#getWalkTargetValue`;
`net.minecraft.world.entity.animal.equine.AbstractHorse#aiStep`;
`net.minecraft.world.entity.animal.feline.Ocelot#checkSpawnObstruction`;
`net.minecraft.world.entity.animal.cow.MushroomCow`;
`net.minecraft.world.level.block.FireBlock#bootStrap`;
`net.minecraft.world.level.block.ComposterBlock#bootStrap`;
`net.minecraft.world.level.block.entity.FuelValues#vanillaBurnTimes`;
`net.minecraft.world.item.CreativeModeTabs#bootstrap(net.minecraft.core.Registry)`;
`net.minecraft.client.color.block.BlockColors`;
block/item/sound reports and item-component reports; all direct and composed
block/item/fluid tags; six loot tables; Coarse Dirt recipe/advancement;
Story root; both Wandering Trader records and trade tags; every direct
worldgen JSON; all 1,212 decoded structures; blockstates, models, item
definitions, textures and language resources. Complete compiled exact-field,
data and decoded-NBT searches found no other identity-specific runtime path.

**Test vectors:**

Run `EXP-BLK-113` across every state, above-block/fluid/light boundary,
controlled spread/particle/bonemeal draw, tool face/air/write result, Water
potion and dispenser path, loot/recipe/trade/mob/tag branch, direct worldgen
record, all 8,461 raw cells and 526 Jigsaw final states, persistence/reload
and exact client projection. Assert IDs, order, constants, absences, census
and vanilla convergence.

**Limits:**

Generic random-tick selection, block/item placement and break, Bone Meal
consumption, tool use, loot, recipe, merchant, mob AI/spawn, feature/surface/
Jigsaw algorithms and rendering remain with their named owners. Dirt Path,
Farmland, Mud, Hanging Roots, vegetation, mobs and structures retain their
own catalog families. This leaf fixes the six substrate identities and every
exact join that selects them.
