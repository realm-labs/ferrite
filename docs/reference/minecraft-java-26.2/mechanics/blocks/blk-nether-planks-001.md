# Blocks mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `BLK-NETHER-PLANKS-001` — Nether planks retain wood crafting and tool repair while rejecting fire and fuel

**Parent:** `SIM-004`, `SIM-005`, `BLK-001`, `BLK-STATE-001`,
`BLK-002`, `BLK-PLACE-001`, `BLK-BREAK-001`, `BLK-BREAK-HOOK-001`,
`BLK-BREAK-CONTENT-001`, `BLK-UPDATE-001`, `PLY-002`, `PLY-005`,
`PLY-006`, `PLY-INTERACT-001`, `PLY-BREAK-001`, `PLY-COLLISION-001`,
`PLY-AUTOJUMP-001`, `ITM-003`, `ITM-004`, `ITM-006`,
`ITM-RECIPE-001`, `ITM-CRAFT-001`, `ITM-LOOT-001`,
`ITM-ADVANCEMENT-001`, `ITM-ANVIL-001`, `ITM-DISPENSER-001`,
`ENT-KNOCKBACK-001`, `ENV-001`, `ENV-002`, `ENV-003`,
`ENV-FIRE-001`, `ENV-FLUID-001`, `ENV-LIGHT-001`, `WGEN-003`,
`WGEN-PIPELINE-001`, `CLI-001`, `CLI-006`, `CLI-UI-001`,
`CLI-EFFECT-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked registration/reports, complete loot/recipe/
advancement/tag/archetype data, FuelValues and tool-repair consumers, exhaustive
server/client exact-field searches, all 1,212 decoded templates and exact client
resources close both property-free states and items. The search distinguishes
their generic plank crafting/mining behavior from their explicit fire and fuel
exclusions.

**Applies when:**

`minecraft:crimson_planks` or `minecraft:warped_planks` is placed, mined,
exploded, used in any matching recipe, offered to repair Wood-material equipment,
selected as Sulfur-Cube equipment or dispenser food, tested by fire/fuel,
persisted, synchronized or rendered.

**Authoritative state:**

Both identities are ordinary property-free `Block` instances without block
entities:

| Identity | State ID | Block protocol ID | Item raw ID | Map color |
|---|---:|---:|---:|---|
| Crimson Planks | `21032` | `883` | `73` | `CRIMSON_STEM` |
| Warped Planks | `21033` | `884` | `74` | `WARPED_STEM` |

Registration fixes `BASS`, hardness/resistance `2/3` and `NETHER_WOOD` sounds.
Every state is a full `0..16` selection/collision/visual/occlusion cube with
emission `0`, light dampening `15`, shade brightness `0.2`, friction `0.6`,
speed/jump factors `1`, restitution `0`, full sturdy faces, ordinary spawn
support, solid redstone conduction and `NORMAL` piston reaction. Neither state
holds fluid or produces signal/comparator output.

Sound volume/pitch is `1/1`; break/step/place/hit/fall event IDs are
`1100/1104/1103/1102/1101`. Each common nondamageable block item stacks to `64`
with only standard block-item components.

Both blocks directly belong only to `planks`; that tag is nested by
`mineable/axe`. Both items directly belong to `planks` and
`non_flammable_wood`. Item `planks` is nested by `wooden_tool_materials` and
`sulfur_cube_archetype/bouncy`; the latter is nested by
`sulfur_cube_swallowable`. These compositions are reload-selected.

**Transition and ordering:**

### Placement, breaking and loot

Ordinary placement, explicit writes, rotation and mirror retain the matching
sole state. The blocks add no random/scheduled tick, use, attack, contact,
neighbor, signal, comparator, fluid or block-event override.

Axes are the accelerated mining tool through composed tag membership, but no
correct-tool requirement exists. Hand, wrong tools and every Axe can therefore
reach the matching one-roll self table after successful removal. Each entry is
behind `survives_explosion` and uses random sequence
`minecraft:blocks/<identity>`. Silk Touch and Fortune add no branch; an admitted
explosion can independently suppress the one item.

### Fire and Furnace fuel exclusion

Neither identity is installed in `FireBlock.bootStrap`, and registration does
not enable lava ignition. Their encouragement/flammability odds are therefore
`0/0`: nearby fire cannot ignite or consume them, and adjacent lava does not
schedule fire from them. The broad `planks` block tag does not supply FireBlock
odds.

Fuel construction initially grants item `planks` burn time `300`, then removes
every member of `non_flammable_wood`. Both items consequently finish with burn
time zero and cannot start or extend Furnace, Blast-Furnace or Smoker fuel.
This removal occurs after the broad addition; reloading either tag can change
the result without changing states `21032/21033`.

### Crafting graph and recipe knowledge

One member of the matching four-entry stem tag shapelessly produces four
matching Planks. The record is in the `planks` group and `building` category.
Its advancement uses one OR requirement: matching stem possession or prior
knowledge of that recipe.

Each Plank identity is an exact ingredient in nine matching family recipes:

| Result | Planks / other material | Output |
|---|---:|---:|
| Button | `1 / 0` | `1` |
| Door | `6 / 0` | `3` |
| Fence | `4 / 2` Sticks | `3` |
| Fence Gate | `2 / 4` Sticks | `1` |
| Pressure Plate | `2 / 0` | `1` |
| Sign | `6 / 1` Stick | `3` |
| Slab | `3 / 0` | `6` |
| Stairs | `6 / 0` | `4` |
| Trapdoor | `6 / 0` | `2` |

Every one has a direct advancement whose single OR requirement accepts exact
matching Planks possession or prior recipe knowledge. Crimson cannot fill a
Warped-family exact slot and vice versa.

Both identities also match each independent `#minecraft:planks` or
`#minecraft:wooden_tool_materials` slot in 40 common shaped records:

- 16 color-specific Beds;
- Wooden Axe, Hoe, Pickaxe, Shovel, Spear and Sword;
- Barrel, Beehive, Bookshelf, Bowl, Cartography Table, Chest, Chiseled
  Bookshelf, Crafting Table, Fletching Table, Grindstone, Jukebox, Loom,
  Note Block, Piston, Shield, Smithing Table, Stick and Tripwire Hook.

Tag ingredients are evaluated per slot, so these common recipes may mix
Crimson, Warped and other locked plank identities. Outputs are one except Bowl
and Stick `4`, and Tripwire Hook `2`. Only the Barrel and Stick advancements
directly accept broad Planks possession; the other 38 common records retain
their independent auxiliary/state criteria or prior-recipe branch.

Thus each identity participates in one production record and 49 consumption
records; the pair spans 60 unique recipe records. All results are default
stacks, all consumed Plank components are discarded, and no cooking or
Stonecutting record accepts either identity.

### Wooden-equipment repair and Sulfur-Cube joins

`wooden_tool_materials` is the repair-item tag stored in `ToolMaterial.WOOD`.
Both Plank items can therefore repair the Wooden Sword, Shovel, Pickaxe, Axe,
Hoe and Spear. Shield registration independently uses the same tag in its
repairable component. Anvil/material consumption, damage restoration, cost,
rename and rejection semantics remain with `ITM-ANVIL-001`.

The composed `bouncy` tag selects the locked Sulfur-Cube archetype. It fixes
horizontal/vertical knockback powers `0.4125/0.105`; additive knockback and
explosion-knockback resistance `-2/-2`; additive bounciness
`0.8999999761581421`; total-multiplied friction and air drag
`-0.699999988079071/-0.9900000002235174`; buoyancy; bouncy hit/push sounds,
cooldown `0.7` and impulse threshold `0.3`. Matching, equipment and contact math
remain with `ENT-KNOCKBACK-001`.

Because `sulfur_cube_swallowable` nests that archetype tag, an otherwise
unregistered dispenser behavior searches the front AABB and lets the first
accepting Sulfur Cube consume one Plank; when none accepts, the protected
default eject path runs. Exact dispenser traversal and consumption remain with
`ITM-DISPENSER-001`.

### Generation and structure absence

The exhaustive NBT scan finds zero Crimson-Plank and zero Warped-Plank cells in
all `1,212` bundled structure templates. Exact-ID, tag and JSON consumer sweeps
find no configured/placed feature, noise/surface rule, structure processor,
non-block loot source, trade or other acquisition record beyond the recipes above.
Bundled Huge Fungi produce matching stems, not Planks; crafting is the sole
bundled source of either Plank item apart from its own block loot.

**Client projection:**

Each property-free blockstate has one unrotated variant selecting the matching
block model. Each model inherits `block/cube_all` and supplies its matching
Planks texture on all faces. The item directly selects the same unrotated block
model; neither identity is tinted.

The English names are `Crimson Planks` and `Warped Planks`. Building Blocks
publishes Crimson after Stripped Crimson Hyphae and before Crimson Stairs, then
publishes Warped after Stripped Warped Hyphae and before Warped Stairs. Neither
appears in another ordinary tab. Updates use states `21032/21033`, inventory
paths use item IDs `73/74`, maps use their respective stem colors, note blocks
read `BASS`, and material sounds use IDs `1100..1104`. No subtype packet or
connection-local state is added.

**Branches and aborts:**

Two identities and sole states; placement/persistence; hand/tool/Axe and
ordinary/explosion loot; direct/composed tag reload; FireBlock and lava
exclusion; fuel broad-add then removal; one production, nine exact-family and
40 common recipe records plus every malformed/mixed grid and unlock branch;
six Wood tools and Shield repair; Sulfur-Cube match/swallow/eject paths; zero
worldgen/template joins; both client assets and tab positions are distinct.

**Constants and randomness:**

State/block/item IDs `21032/883/73` and `21033/884/74`; strength `2/3`;
emission `0`; dampening `15`; shade `0.2`; friction `0.6`; speed/jump `1`;
sound IDs `1100/1104/1103/1102/1101` at `1/1`; stack `64`; fire odds `0/0`;
final fuel time `0`; recipe records per identity production/consumption
`1/49`, family-unique `60`; derivative outputs `1/3/3/1/1/3/6/4/2`;
templates/cells `1212/0/0`; bouncy powers `0.4125/0.105`, cooldown `0.7` and
threshold `0.3`. The blocks consume no RNG directly; explosion loot and owning
generic systems retain their stated streams.

**Side effects:**

Block placement/removal and self loot; recipe outputs and knowledge;
Wood-equipment repair material consumption; reload-selected fuel rejection and
Sulfur-Cube equipment/swallowing; ordinary persistence, map shading, notes,
sounds, models, names and tab projection.

**Gates:**

World-write/break authority; explosion survival; active block/item tags,
loot/recipe/advancement/archetype snapshots; grid/output and recipe-knowledge
admission; repairable equipment/damage/anvil state; dispenser front-AABB and
Sulfur-Cube acceptance; valid registry, map, sound and client-resource context.

**Boundary cases and quirks:**

- The block `planks` tag accelerates Axe mining but does not make these blocks
  flammable.
- Fuel construction first adds all item Planks at `300`, then removes these two
  through `non_flammable_wood`, leaving zero.
- Common recipe slots may mix Plank colors; matching family-derivative slots
  cannot.
- Planks repair all six Wood-material tools and Shields even though they cannot
  fuel a Furnace.
- Their bouncy archetype is also nested into dispenser swallowability.
- Absence from structures and worldgen does not remove their stem-to-Planks
  crafting source.

**Failure semantics:**

Failed placement/removal commits only generic earlier work. A failed
explosion-survival condition emits nothing. Invalid or output-blocked recipes,
repair attempts and Sulfur-Cube admission consume nothing beyond their generic
owners' stated earlier effects. Fuel lookup returns no burn entry. Reload
removal from ingredient/repair/archetype tags prevents later matches without
retroactively changing placed blocks or existing recipe knowledge.

**Client/server authority split:**

The server owns states, placement, breaking, loot, recipes/knowledge, repair,
fuel/fire selection, Sulfur-Cube interactions and persistence. Clients project
state/item IDs, map colors, notes, sounds, names, cube models and Building
Blocks order.

**Observability:**

Commands/state packets, shape/light/signal probes, mining speed and drops,
fire/lava/fuel tests, crafting books/grids, Anvil output, dispenser/Sulfur-Cube
state, template scans, maps, note/sounds, tabs and rendering expose every listed
branch.

**Persistence and reload:**

Placed blocks persist only identity; item stacks persist ordinary components.
Loot, recipes, advancements, block/item tags and Sulfur-Cube archetypes are
reload-selected. Registration, FireBlock omissions, FuelValues ordering,
ToolMaterial wiring, dispenser code and creative-tab order remain code-built.

**Evidence:**

`Confirmed`; `OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-DATA-001`;
`OFF-REPORT-001`. Anchors:
`net.minecraft.world.level.block.Blocks`;
`net.minecraft.world.level.block.FireBlock#bootStrap`;
`net.minecraft.world.level.block.entity.FuelValues#vanillaBurnTimes`;
`net.minecraft.world.item.ToolMaterial`;
`net.minecraft.world.item.Items`;
`net.minecraft.world.entity.monster.cubemob.SulfurCube#matchingArchetypes`;
`net.minecraft.core.dispenser.DispenseItemBehavior#bootStrap`;
`net.minecraft.world.item.CreativeModeTabs#bootstrap`;
both block/registry reports and item components; matching loot, 60 recipe
records and their advancements; block/item/composed tags and bouncy archetype; all 1,212
templates; exact blockstate/model/item/name resources. Complete compiled
exact-field and data-reference searches found no other runtime path.

**Test vectors:**

Run `EXP-BLK-105` across both states/IDs; every placement/tool/explosion path;
Fire/lava and fuel broad-add/removal snapshots; one production, nine matching
family and 40 common recipes including mixed colors and every unlock; all six
Wood tools and Shield repair; bouncy equipment and dispenser swallowing; every
tag reload; all 1,212 templates and worldgen resources; persistence, maps,
notes, sounds, names, sole models and exact Building-Blocks positions. Assert
all constants, outputs, negative joins and vanilla convergence.

**Limits:**

Generic placement, breaking, loot/explosion, crafting/knowledge, Anvil repair,
fuel/fire scheduling, Sulfur-Cube behavior, dispenser traversal, packet encoding
and rendering remain with their named owners. This leaf fixes the two sole
states/items, exact crafting/repair/tag joins, nonflammability/fuel removal,
absence census and projection.
