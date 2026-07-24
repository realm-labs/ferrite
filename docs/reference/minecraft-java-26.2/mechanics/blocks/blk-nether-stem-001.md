# Blocks mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `BLK-NETHER-STEM-001` — Nether stems and hyphae preserve axis through placement, stripping and fungus growth

**Parent:** `SIM-003`, `BLK-001`, `BLK-002`, `BLK-003`, `BLK-005`, `PLY-005`, `PLY-006`,
`ITM-004`, `ITM-005`, `ITM-006`, `ITM-007`, `ENT-001`, `MOB-001`, `MOB-004`, `ENV-003`,
`WGEN-002`, `WGEN-003`, `CLI-001`, `CLI-006`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked registration and reports, complete loot/recipe/advancement/tag/worldgen
data, exhaustive server/client class-reference sweeps, all 1,212 decoded structure templates and
exact client assets close the eight Nether stem/hyphae blocks. Their shared rotated-pillar state
machine joins state-preserving axe stripping, nonflammable log semantics, crafting, tree/leaf and
parrot tag consumers, crimson/warped huge-fungus stems and axis-aware client projection.

**Applies when:**

Any of `minecraft:{crimson_stem,warped_stem,stripped_crimson_stem,
stripped_warped_stem,crimson_hyphae,warped_hyphae,stripped_crimson_hyphae,
stripped_warped_hyphae}` is placed, transformed with an axe, harvested, exploded, crafted,
considered as a log by a leaf, parrot, tree, lava-pool, tutorial, fuel or blending consumer,
generated as a huge-fungus stem, persisted, mapped or rendered.

**Authoritative state:**

Each identity is a `RotatedPillarBlock` with `axis=x|y|z`, default `y`, no block entity and the
following locked state, block-protocol and block-item IDs:

| Identity | Axis X/Y/Z state IDs | Block ID | Item ID |
|---|---:|---:|---:|
| `warped_stem` | `20945/20946/20947` | `862` | `173` |
| `stripped_warped_stem` | `20948/20949/20950` | `863` | `185` |
| `warped_hyphae` | `20951/20952/20953` | `864` | `208` |
| `stripped_warped_hyphae` | `20954/20955/20956` | `865` | `196` |
| `crimson_stem` | `20962/20963/20964` | `871` | `172` |
| `stripped_crimson_stem` | `20965/20966/20967` | `872` | `184` |
| `crimson_hyphae` | `20968/20969/20970` | `873` | `207` |
| `stripped_crimson_hyphae` | `20971/20972/20973` | `874` | `195` |

Registration fixes note instrument `BASS`, hardness/resistance `2/2`, `STEM` sounds and no
correct-tool drop gate, random ticking, lava ignition or block entity. Crimson stem variants use
map color `CRIMSON_STEM`, crimson hyphae variants use `CRIMSON_HYPHAE`; warped variants similarly
use `WARPED_STEM` or `WARPED_HYPHAE`. The selected color is constant across axis.

Every state is a full unit selection/collision/visual/occlusion cube with emission `0`, light
dampening `15`, shade brightness `0.2`, friction `0.6`, speed/jump factors `1`, restitution `0`,
solid redstone conduction, normal piston reaction, full sturdy faces and ordinary full-face spawn
support. The blocks add no tick, use, attack, entity-contact, neighbor, signal, comparator or
block-event override.

The Stem sound type has volume/pitch `1/1` and registry IDs break `1121`, step `1122`, place
`1123`, hit `1124` and fall `1125`. Each ordinary block item is common, stacks to `64` and has
standard block-item components. Its direct item tags are its crimson/warped stem family and
`non_flammable_wood`.

**Transition and ordering:**

#### Axis placement, rotation, stripping and loot

Ordinary item placement selects the clicked face's axis; forced state/component writes may select
any legal axis. Quarter-turn rotations around Y exchange X and Z while preserving Y; half turns,
mirrors and rotations around the pillar's axis preserve its axis. Clone/pick returns the matching
unmodified block item.

An axe's ordered block transform maps:

- `crimson_stem -> stripped_crimson_stem`;
- `warped_stem -> stripped_warped_stem`;
- `crimson_hyphae -> stripped_crimson_hyphae`;
- `warped_hyphae -> stripped_warped_hyphae`.

Each mapping copies the source `axis`. It has precedence over copper scraping and unwaxing. On a
server-authoritative match, the axe plays `minecraft:item.axe.strip` (sound ID `88`, `BLOCKS`,
volume/pitch `1/1`), triggers the server-player item-used-on-block criterion before mutation, calls
`setBlock` with flags `11`, emits `BLOCK_CHANGE` with the player and new state, damages the axe by
one when a player exists and returns success. The write result is not consulted before the event,
damage and success result. A main-hand attempt instead returns pass before transformation when the
offhand item has `BLOCKS_ATTACKS` and secondary use is not active. Already stripped identities
have no stripping mapping and, absent a later copper/wax transform, return pass.

All eight block loot tables contain one matching item behind `survives_explosion`, with random
sequence `minecraft:blocks/<identity>`. Tool class, correctness, Silk Touch, Fortune and axis do
not otherwise affect the table.

#### Log tags, fire, fuel, recipes and progression

Block and item tags `crimson_stems` and `warped_stems` each contain their four exact family
identities. `logs` nests both families, but `logs_that_burn` does not; all eight item identities
also occur directly in `non_flammable_wood`. Consequences of that closure are:

- an axe is the accelerated mining tool, without becoming a loot requirement;
- each block is distance zero for nearby leaf-decay propagation;
- a parrot may spawn above it when the independent light gate passes, and the wander goal may
  select an air destination above it as a tree perch when the destination and the block above are
  empty;
- ordinary tree clearance treats it as free, but the later `placeLog` gate still rejects it because
  tagged logs are not thereby `replaceable_by_trees`;
- lava-pool stone replacement is prohibited at it;
- terrain blending excludes it, like leaves and mushroom caps, from an old-chunk surface sample;
- the client find-tree tutorial accepts looking at, obtaining, already holding or previously
  mining it through the completed-tree tag;
- the item can select the `bouncy` sulfur-cube body-equipment archetype.

The `bouncy` record has horizontal/vertical knockback powers `0.4125/0.105` and sound
`bouncy.hit`; matching, equipment, contact and knockback remain with `ENT-KNOCKBACK-001`. The
parrot spawn, wander, leaf-distance, tree, blending, lava-pool and tutorial algorithms retain their
own admission, traversal and update semantics.

No identity is registered in `FireBlock`'s encouragement/flammability table, and registration does
not enable lava ignition, so ordinary fire spread has ignition/burn odds `0/0` and lava does not
ignite it. Vanilla fuel construction initially admits the nested `logs` tag, then removes every
`non_flammable_wood` item; all eight therefore have furnace burn time zero. The charcoal recipe
requires `logs_that_burn`, so none matches it.

The exact recipe joins are:

- four matching unstripped stems in a 2-by-2 square yield three matching hyphae;
- four matching stripped stems similarly yield three matching stripped hyphae;
- one member of the matching stem-family tag shapelessly yields four matching planks;
- six matching stripped stems plus two chains yield six matching hanging signs;
- six matching stripped stems in the top and bottom rows yield six matching shelves;
- any three of the eight, through `logs`, fill the bottom row of the campfire or soul-campfire
  recipe, and any four surround a furnace in the smoker recipe.

The four hyphae, two plank, two hanging-sign and two shelf recipes each have a direct advancement.
Its relevant inventory criterion and `recipe_unlocked` criterion form one OR requirement and the
reward grants that recipe. Campfire, soul-campfire and smoker keep their independent generic
unlock owners. There is no bundled recipe that turns a hyphae block back into stems.

#### Huge-fungus generation and structure absence

`crimson_fungus` and `crimson_fungus_planted` configure default-axis `crimson_stem` as their stem
state and require exact crimson nylium at the origin base. The two warped records use
default-axis `warped_stem` and warped nylium. No configured huge fungus selects hyphae, stripped
material or a horizontal stem.

After the exact-base gate, height is uniform `4..13`, then has an independent `1/12` chance to
double. An unplanted feature aborts when `originY + height + 1` reaches or exceeds generation
depth; only unplanted generation consumes the next float for a `0.06` chance of the huge
three-by-three stem form. The origin is cleared to air with flags `260` before stem then hat
placement.

Ordinary and planted stems offer one vertical Y-axis column. A natural huge stem offers every
non-corner cell in its three-by-three layer and independently offers each of four corners only
when its next float is below `0.1`. A stem cell is considered only when it is air or satisfies the
feature's locked replacement predicate. Planted writes use flags `3` and destroy a nonair
replaceable cell with drops before writing; unplanted writes use the feature setter. The pipeline
owner retains exact candidate order, replacement list, failed writes, hat/decor traversal and RNG.

The exhaustive NBT scan finds zero cells for all eight identities in all 1,212 bundled structure
templates. The four configured huge-fungus records are therefore their only bundled worldgen
sources.

**Client projection:**

Every blockstate file maps axis X to the corresponding block model with rotations `x=90,y=90`,
axis Y to the unrotated model and axis Z to `x=90`. Every model inherits `cube_column`:

- stems use the matching stem side texture and matching `_top` end texture;
- stripped stems use the matching stripped side and stripped `_top` end textures;
- hyphae use the unstripped stem side texture on both side and end;
- stripped hyphae use the stripped stem side texture on both side and end.

Each item selector points directly at the same identity's unrotated block model. The Natural Blocks
tab contains unstripped crimson stem then warped stem; the Building Blocks tab contains crimson
stem, crimson hyphae, stripped crimson stem, stripped crimson hyphae and the analogous four warped
entries in that order within their family groups. Authoritative block updates publish the state
IDs above, inventory projection uses the listed item IDs, material sounds use IDs `1121..1125`,
stripping uses ID `88`, and map projection uses the four registration colors. This leaf adds no
packet field, acknowledgement or connection-local state.

**Branches and aborts:**

Eight identities and three axes; clicked-face versus forced placement; every rotation/mirror;
four strip matches, four stripped misses, offhand blocking and failed authoritative writes; every
tool/explosion survival; direct and nested tag reload; leaf/parrot/tree/lava-pool/blending/tutorial/
archetype positive and negative selection; fire source/lava/fuel/charcoal exclusion; all thirteen
recipe matches, malformed grids, output blocking and ten unlocks; crimson/warped, ordinary/planted,
height/double/huge/bounds/replacement/corner/write outcomes; zero template selection; save/reload;
sound/map/block/item/model projection are distinct branches.

**Constants and randomness:**

State IDs, block/item IDs and axes as tabulated; hardness/resistance `2/2`; sound volume/pitch
`1/1`; Stem sound IDs `1121..1125`; axe-strip sound ID `88`, write flags `11`, durability cost
`1`; emission `0`, dampening `15`, shade `0.2`, friction `0.6`, speed/jump `1`, restitution `0`,
stack `64`; fire encouragement/flammability `0/0`, burn time `0`; recipe outputs
hyphae/planks/hanging-signs/shelves `3/4/6/6`; fungus height `4..13`, double denominator `12`,
unplanted huge chance `0.06`, corner chance `0.1`, origin-clear flags `260`, planted flags `3`;
scanned templates/cells `1212/0`; bouncy powers `0.4125/0.105`. Placement, stripping, loot before
explosion decay and tag consumers add no RNG. Worldgen and owning algorithms retain the exact
streams described above.

**Side effects:**

Axis-sensitive placement and palette writes; state-preserving strip sound, criterion, block write,
game event and durability loss; tool-independent self loot; reload-selected tag membership; leaf,
parrot, tree, lava-pool, blending, tutorial and equipment selection; thirteen recipe results and
ten grants; explicit fire/fuel/charcoal absence; vertical fungus-stem writes; ordinary palette and
inventory persistence; map shading, sounds and cube-column projection.

**Gates:**

World-write and break authority; axe transform and offhand/secondary-use gate; explosion context;
active loot, recipe, advancement, tag, archetype and worldgen snapshots; recipe output admission;
leaf/parrot/tree/blending/tutorial and sulfur-equipment admission; feature base, bounds, planted,
replacement and write gates; valid registry, map, sound, creative-tab and client-resource context.

**Boundary cases and quirks:**

These blocks are nested `logs` but deliberately neither `logs_that_burn` nor furnace fuel. Direct
family tags include both stems and hyphae, stripped and unstripped. Hyphae display the stem side
texture on every face rather than using the stem top texture. Axe stripping preserves horizontal
axis and reports success even if its flags-11 block write fails. Huge fungi write only unstripped
Y-axis stems; planted fungi cannot become the three-by-three huge form. Tree clearance accepts
these logs but the ordinary trunk offer still skips them; lava-pool stone may not replace them.
Zero structure cells does not mean natural fungus generation is absent.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-REPORT-001`; `OFF-DATA-001`;
`net.minecraft.world.level.block.Blocks`;
`net.minecraft.world.level.block.RotatedPillarBlock#getStateForPlacement`;
`net.minecraft.world.level.block.RotatedPillarBlock#rotate`;
`net.minecraft.world.level.block.RotatedPillarBlock#createBlockStateDefinition`;
`net.minecraft.world.item.AxeItem#useOn`;
`net.minecraft.world.item.AxeItem#evaluateNewBlockState`;
`net.minecraft.world.item.AxeItem#getStripped`;
`net.minecraft.world.level.block.FireBlock#bootStrap`;
`net.minecraft.world.level.block.LeavesBlock#getOptionalDistanceAt`;
`net.minecraft.world.entity.animal.parrot.Parrot#checkParrotSpawnRules`;
`net.minecraft.world.entity.animal.parrot.Parrot$ParrotWanderGoal#getTreePos`;
`net.minecraft.world.level.levelgen.feature.trunkplacers.TrunkPlacer#isFree`;
`net.minecraft.world.level.levelgen.feature.trunkplacers.TrunkPlacer#placeLog`;
`net.minecraft.world.level.levelgen.feature.TreeFeature#validTreePos`;
`net.minecraft.world.level.levelgen.blending.BlendingData#isGround`;
`net.minecraft.world.level.block.entity.FuelValues#vanillaBurnTimes`;
`net.minecraft.client.tutorial.FindTreeTutorialStepInstance`;
`net.minecraft.world.entity.monster.cubemob.SulfurCube#matchingArchetypes`;
`net.minecraft.world.level.levelgen.feature.HugeFungusFeature#place`;
`net.minecraft.world.level.levelgen.feature.HugeFungusFeature#placeStem`;
`net.minecraft.world.item.CreativeModeTabs#bootstrap`;
`reports/blocks.json#minecraft:{crimson_stem,warped_stem,stripped_crimson_stem,
stripped_warped_stem,crimson_hyphae,warped_hyphae,stripped_crimson_hyphae,
stripped_warped_hyphae}`;
`reports/registries.json#minecraft:{block,item,sound_event}`;
`reports/minecraft/components/item/{crimson_stem,warped_stem,stripped_crimson_stem,
stripped_warped_stem,crimson_hyphae,warped_hyphae,stripped_crimson_hyphae,
stripped_warped_hyphae}.json`;
`data/minecraft/loot_table/blocks/{crimson_stem,warped_stem,stripped_crimson_stem,
stripped_warped_stem,crimson_hyphae,warped_hyphae,stripped_crimson_hyphae,
stripped_warped_hyphae}.json`;
`data/minecraft/tags/{block,item}/{crimson_stems,warped_stems,logs}.json`;
`data/minecraft/tags/block/{mineable/axe,completes_find_tree_tutorial,
lava_pool_stone_cannot_replace,parrots_spawnable_on,prevents_nearby_leaf_decay}.json`;
`data/minecraft/tags/item/{non_flammable_wood,completes_find_tree_tutorial,
sulfur_cube_archetype/bouncy}.json`;
`data/minecraft/sulfur_cube_archetype/bouncy.json`;
`data/minecraft/recipe/{crimson_hyphae,warped_hyphae,stripped_crimson_hyphae,
stripped_warped_hyphae,crimson_planks,warped_planks,crimson_hanging_sign,
warped_hanging_sign,crimson_shelf,warped_shelf,campfire,soul_campfire,smoker,charcoal}.json`;
`data/minecraft/advancement/recipes/**/*.json`;
`data/minecraft/worldgen/configured_feature/{crimson_fungus,crimson_fungus_planted,
warped_fungus,warped_fungus_planted}.json`;
`data/minecraft/structure/**/*.nbt`;
`assets/minecraft/blockstates/{crimson_stem,warped_stem,stripped_crimson_stem,
stripped_warped_stem,crimson_hyphae,warped_hyphae,stripped_crimson_hyphae,
stripped_warped_hyphae}.json`;
`assets/minecraft/models/block/{crimson_stem,warped_stem,stripped_crimson_stem,
stripped_warped_stem,crimson_hyphae,warped_hyphae,stripped_crimson_hyphae,
stripped_warped_hyphae}.json`;
`assets/minecraft/items/{crimson_stem,warped_stem,stripped_crimson_stem,
stripped_warped_stem,crimson_hyphae,warped_hyphae,stripped_crimson_hyphae,
stripped_warped_hyphae}.json`.

**Test vectors:**

Run `EXP-BLK-069` across every identity/axis, clicked-face and forced placement, transforms and
failed writes, all loot/tag/fire/fuel/recipe/advancement consumers, leaf/parrot/tree/lava-pool/
blending/tutorial/archetype joins, ordinary/planted crimson/warped fungi, all 1,212 templates,
persistence, creative tabs, sounds, maps and models. Assert exact IDs, constants, read/draw/write
order, negative memberships, zero template cells and vanilla-client convergence.

**Limits:**

Generic placement, breaking, axe item dispatch, loot, crafting, advancement, fuel, fire, leaf
updates, mob spawning/AI, sulfur equipment/knockback, tree/fungus generation, tutorial lifecycle,
packet encoding and client rendering remain with `BLK-PLACE-001`, `PLY-BREAK-001`,
`ITM-USE-001`, `ITM-LOOT-001`, `ITM-RECIPE-001`, `ITM-ADVANCEMENT-001`,
`ITM-FURNACE-001`, `ENV-FIRE-001`, `MOB-SPAWN-001`, `MOB-AI-001`,
`ENT-KNOCKBACK-001`, `WGEN-PIPELINE-001`, `CLI-001`,
`PROTO-PLAY-CLIENTBOUND-BLOCK-001`, `PROTO-PLAY-CLIENTBOUND-SOUND-001` and `CLI-006`.
