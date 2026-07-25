# Blocks mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `BLK-RESIN-MATERIAL-001` — Resin Clumps compact and smelt into fast-flat masonry and orange armor trim

**Parent:** `SIM-004`, `SIM-005`, `BLK-001`, `BLK-002`, `BLK-003`, `BLK-005`,
`BLK-PLACE-001`, `BLK-BREAK-001`, `BLK-BREAK-HOOK-001`, `PLY-002`, `PLY-005`,
`PLY-006`, `PLY-INPUT-001`, `PLY-INTERACT-001`, `PLY-BREAK-001`, `ITM-001`,
`ITM-002`, `ITM-003`, `ITM-004`, `ITM-005`, `ITM-006`, `ITM-007`,
`ITM-USE-001`, `ITM-CONTAINER-001`, `ITM-RECIPE-001`, `ITM-CRAFT-001`,
`ITM-FURNACE-001`, `ITM-STONECUTTER-001`, `ITM-SMITHING-001`, `ITM-LOOT-001`,
`ITM-ADVANCEMENT-001`, `ITM-ANVIL-001`, `ENT-001`, `ENT-KNOCKBACK-001`,
`MOB-001`, `MOB-004`, `MOB-AI-001`, `ENV-001`, `ENV-002`, `ENV-003`,
`ENV-FIRE-001`, `WGEN-PIPELINE-001`, `WGEN-STRUCTURE-WOODLAND-MANSION-001`,
`CLI-001`, `CLI-006`, `CLI-UI-001`, `CLI-EFFECT-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — registrations/components, three block reports, the complete recipe and
advancement graph, trim material, block and chest loot, Creaking-heart bytecode, all 1,212
templates and exact resources determine every loose-Brick and three full-block branch. Generic
multiface, processing, loot, Smithing, structure, stack and rendering algorithms retain their
cited owners.

**Applies when:**

`minecraft:resin_brick`, `minecraft:resin_block`, `minecraft:resin_bricks` or
`minecraft:chiseled_resin_bricks` is acquired, processed, placed, mined, exploded, crafted,
stonecut, used as a trim material or Creaking-Heart input, equipped on a Sulfur Cube, persisted,
synchronized or rendered before and after recipe, advancement, loot, tag, trim or resource reload.

**Authoritative state:**

| Identity | Block ID | Item ID | Sole/default state |
|---|---:|---:|---:|
| Block of Resin | `375` | `441` | `8921` |
| Resin Bricks | `376` | `442` | `8922` |
| Chiseled Resin Bricks | `380` | `446` | `9333` |

All three are property-free orange-terracotta-map-color, `BASEDRUM` full cubes without block
entities, ticks or identity-specific contact/use/signal hooks. Block of Resin has default
hardness/resistance `0/0`, ordinary-tool removal, `RESIN` sounds and no direct block tag. The two
masonry cubes have hardness/resistance `1.5/6`, `RESIN_BRICKS` sounds,
`requiresCorrectToolForDrops` and direct `mineable/pickaxe` membership. No tier tag contains them,
so any pickaxe is correct.

Their ordinary common block items stack to `64` and all three are direct
`sulfur_cube_archetype/fast_flat` members. Loose Resin Brick is raw item ID `1276`, a common
nondamageable plain 64-stack with default `provides_trim_material=minecraft:resin` and direct
`trim_materials` membership. It has no food, consumable, remainder, fuel, compost, durability,
repair, projectile, cooldown, inventory-tick or identity-specific use branch.

**Transition and ordering:**

### Clump acquisition and material conversion

The already generic multiface Resin Clump is the only ordinary input. Nine exact Clumps in a full
grid produce one default Block of Resin; one Block shapelessly returns nine default Clumps.
Exact Clump smelting is Furnace-only, takes the omitted default `200` ticks, awards recipe XP
`0.1` and emits one default Resin Brick. Blast Furnace, Smoker and Campfire reject it.

Two external paths create Clumps:

- each admitted hit on an awake Creaking-heart protector, while its 100-tick emitter cooldown is
  zero, draws `2..3` spread attempts. Each breadth-first traversal starts at the heart, has depth
  `2` and visit limit `64`, follows shuffled adjacent Pale-Oak logs and shuffles six faces at each
  log. Its first air/source-water/existing-Clump neighbor missing the inward face becomes or gains
  one Resin-Clump face (waterlogged for source water) with flags `3`, then plays Resin place sound
  and emits `BLOCK_PLACE`;
- non-Silk Creaking-Heart loot emits inclusive `1..3`, adds uniform `0..Fortune`, caps at `9` and
  then applies explosion decay. Silk Touch emits the Heart instead. Woodland-Mansion chest pool
  two rolls uniform `1..4`; its total weight is `175`, the Clump row has weight `50` and selected
  count `2..4`.

The multiface placement/survival/waterlogging and six-face-count loot algorithms remain with their
generic owners; the Creaking and mansion transactions remain with their cited owners. No other
chest, archaeology, fishing, barter, gift, entity or merchant table directly emits Resin Brick or
the three full blocks.

### Thirteen recipe and unlock records

Four Resin Bricks in a `2×2` grid make one Resin Bricks block. The base masonry then supplies:

| Output | Grid path | Stonecutter path |
|---|---|---|
| Chiseled Resin Bricks | two Resin-Brick Slabs vertically to one | base to one |
| Resin-Brick Slab | three base blocks in a row to six | base to two |
| Resin-Brick Stairs | six base blocks in a stair to four | base to one |
| Resin-Brick Wall | six base blocks in two rows to six | base to one |

The thirteenth family recipe is Creaking Heart: Pale-Oak Log above and below one Block of Resin
produces one Heart. Together with Clump compression/decompression, Brick smelting, base masonry
and the eight shape/Stonecutter records, all thirteen have distinct recipe advancements. Each
uses one OR requirement pairing prior knowledge with exact inventory possession: Clump for the
Block and Brick smelt, Block for Clump decompression and Creaking Heart, Brick for base masonry,
Slab for shaped Chiseled, and base Resin Bricks for the other seven masonry records.

Inputs match identities without copying component patches; results are default stacks. Grid
offset/mirror, machine capacity/progress, Stonecutter selection, atomic consumption and recipe
publication remain generic.

### Loot, archetype and trim material

Block of Resin always admits its one matching self entry; Resin Bricks and Chiseled Resin Bricks
admit theirs only with any pickaxe. Each table gates the item with `survives_explosion`, uses its
matching `minecraft:blocks/<identity>` random sequence and has no Silk/Fortune conversion.

All three block items select locked `fast_flat`: horizontal/vertical knockback powers
`0.9125/0.09`, hit/push sounds, cooldown `0.9`, impulse threshold `0.03` and its five attribute
modifiers. Loose Brick does not match.

Resin Brick's default component resolves trim material `minecraft:resin`, whose asset name is
`resin` and translated description color is `#FC7812`. As a live `trim_materials` member it fills
the addition slot of each of the 18 generic armor-trim Smithing recipes, is consumed once and
writes the Resin material holder into the copied armor result. Removing the tag rejects it;
patching/removing `provides_trim_material` changes or invalidates the material selected after
recipe admission. Template, trimmable-armor, existing-trim, capacity and consumption behavior
remain `ITM-SMITHING-001`.

### Absences, persistence and projection

None of the four identities is furnace fuel or compostable; the three blocks have FireBlock odds
`0/0` and no lava-ignitable property. No configured/placed feature directly emits a full block.
An exhaustive decode of all 1,212 structure templates finds zero full-block family cells and zero
loose-Brick strings; mansion acquisition is loot-table driven.

Chunk palettes persist only the property-free state; stacks persist identity, count and component
patches. Recipe, loot, tag and trim reload affects only future evaluation. Existing chunks,
completed crafts and trimmed equipment are not rewritten; resource reload independently changes
projection.

Generic publication uses state IDs `8921/8922/9333` and item IDs `441/442/446/1276`. English names
are `Block of Resin`, `Resin Bricks`, `Chiseled Resin Bricks` and `Resin Brick`. Each full block
and block item selects its same-named opaque `cube_all` model; the Brick selects an untinted
same-named generated flat. Resin sounds are break/fall/place/step IDs `1374/1375/1376/1377` with
empty hit; Resin-Bricks sounds are break/fall/hit/place/step IDs `1378/1379/1380/1381/1382`.

Natural Blocks places Block of Resin between Honey Block and Ochre Froglight. Building Blocks
places Resin Bricks, Stairs, Slab, Wall and Chiseled after Mud-Brick Wall and before Sandstone.
Ingredients places Resin Brick between Nether Brick and Paper. Resin palette projection covers
the 29 compatible armor item-model overlays plus atlas-driven equipped trim.

**Branches and aborts:**

Identity/components; Clump source/spread; compression/smelting/masonry/Heart recipes and thirteen
unlocks; hand versus correct pickaxe and explosion survival; three fast-flat selectors; live
trim tag/component and 18 Smithing records; zero direct feature/template paths; persistence,
reload, wire and client projection are distinct.

**Constants and randomness:**

IDs/state as tabled plus Brick `1276`; strengths `0/0` and `1.5/6`; stacks `64`; smelting
`200/0.1`; compression `9:1`; Creaking spread `2..3`, depth/visits `2/64`, cooldown `100`;
Creaking loot `1..3+uniform(0..Fortune)`, cap `9`; mansion weight/total `50/175`, rolls/count
`1..4/2..4`; trim color `#FC7812`; templates/cells `1212/0`.

**Side effects:**

Clump faces, sound and game event; machine result/XP and recipe knowledge; compacting, masonry,
Heart and Smithing results; placement/break/self loot; Sulfur-Cube selection; stack/chunk/equipment
persistence, synchronization and exact client projection.

**Gates:**

Exact ingredient/result capacity; Creaking protector/state/cooldown/traversal/target; mansion and
block-loot selection; grid/Stonecutter/knowledge; world-write/tool/explosion; live archetype and
trim tags/provider; registry/chunk/stack/equipment decode and client resources.

**State read/written:**

Reads all gates above and writes only the Clump, processing, knowledge, block, loot, archetype,
trimmed-equipment, durable, wire and projection state listed above.

**Failure behavior:**

Wrong machine/input/capacity commits no transform. Failed Creaking gates or traversal place no
face. Wrong grid or unavailable recipe emits no result. Hand-mined masonry drops nothing; failed
explosion survival suppresses self loot. Missing trim tag/provider rejects or invalidates the
trim result. Reload affects future evaluation only.

**Boundary cases and quirks:**

Block of Resin is a full cube yet has default zero hardness/resistance and needs no tool. Resin
Bricks and Chiseled Resin Bricks instead require any pickaxe. Clump compression is reversible
9:1, while Brick-to-Bricks compression has no reverse recipe. The same loose Brick is both
masonry feedstock and the live orange trim material.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-REPORT-001`; `OFF-DATA-001`;
`net.minecraft.world.level.block.Blocks`; `net.minecraft.world.item.Items`;
`net.minecraft.world.level.block.entity.CreakingHeartBlockEntity#creakingHurt`;
`net.minecraft.world.level.block.entity.CreakingHeartBlockEntity#spreadResin`;
`net.minecraft.world.item.CreativeModeTabs`;
`reports/blocks.json#minecraft:{resin_block,resin_bricks,chiseled_resin_bricks}`;
`reports/minecraft/components/item/{resin_brick,resin_block,resin_bricks,chiseled_resin_bricks}.json`;
`data/minecraft/recipe/{resin_block,resin_clump,resin_brick,resin_bricks,chiseled_resin_bricks*,resin_brick_{slab*,stairs*,wall*},creaking_heart}.json`;
`data/minecraft/advancement/recipes/**/*resin*.json`;
`data/minecraft/loot_table/{blocks/{resin_block,resin_bricks,chiseled_resin_bricks,creaking_heart},chests/woodland_mansion}.json`;
`data/minecraft/{trim_material/resin,tags/item/{trim_materials,sulfur_cube_archetype/fast_flat},tags/block/mineable/pickaxe}.json`;
`data/minecraft/structure/**/*.nbt`; `assets/minecraft/**/*resin*`;
`WGEN-STRUCTURE-WOODLAND-MANSION-001`; `EXP-BLK-088`.

**Test vectors:**

Run `EXP-BLK-088` across default/patched Brick and three full blocks, every Clump spread/loot
source, all thirteen recipes/unlocks, correct-tool/explosion/fast-flat boundaries and all 18 trim
recipes under tag/component reload. Scan every template, persist/reload/synchronize all owners and
assert exact IDs, names, sounds, models, textures, palette and tab order.

**Limits:**

Generic multiface, processing, crafting, Stonecutter, Smithing, loot, Creaking, structure, block,
packet and renderer control flow remains with cited owners. This leaf fixes exact Resin material
identities, transforms, hard-coded joins, absences and projection.
