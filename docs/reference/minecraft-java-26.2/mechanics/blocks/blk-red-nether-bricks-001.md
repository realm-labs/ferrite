# Blocks mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `BLK-RED-NETHER-BRICKS-001` — Red Nether bricks join masonry, slow-bouncy equipment and the Nether display

**Parent:** `BLK-001`, `BLK-002`, `BLK-003`, `BLK-005`, `PLY-005`, `PLY-006`, `ITM-004`,
`ITM-006`, `ENT-001`, `ENT-005`, `ENV-003`, `CLI-006`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked registration, reports, complete loot/recipe/advancement/tag data, the
server class-reference and worldgen-data sweeps, all 1,212 decoded structure templates and exact
client assets exhaust this property-free ordinary block. Its joins are correct-tool self loot,
seven recipe/unlock records, one advancement display icon, slow-bouncy equipment and no generation
or structure-template producer.

**Applies when:**

`minecraft:red_nether_bricks` is placed, written, harvested, exploded, crafted or stonecut,
equipped on a sulfur cube, displayed by the Nether-root advancement, persisted, mapped or rendered.

**Authoritative state:**

Red Nether bricks is an ordinary property-free `Block` with no block entity and sole state
`14847`. Its locked block protocol ID is `673`, and its block-item raw ID is `606`. Registration
selects map color `NETHER`, note instrument `BASEDRUM`, hardness/resistance `2/6`,
`NETHER_BRICKS` sounds and `requiresCorrectToolForDrops`.

The state is a full unit selection/collision/visual/occlusion cube with emission `0`, light
dampening `15`, shade brightness `0.2`, friction `0.6`, speed/jump factors `1`, restitution `0`,
solid redstone conduction, normal piston reaction, full sturdy faces and ordinary full-face spawn
support. It adds no random or scheduled tick, use, attack, entity-contact, neighbor, signal,
comparator or block-event override. Its only direct block tag is `mineable/pickaxe`; no tool-tier
tag contains it, so any pickaxe satisfies the correct-tool gate.

The Nether-bricks sound type has volume/pitch `1/1` and selects sound registry IDs break `1093`,
step `1094`, place `1095`, hit `1096` and fall `1097`. The ordinary block item is common, stacks
to `64`, has standard block-item components and is directly in the reloadable
`sulfur_cube_archetype/slow_bouncy` item tag.

**Transition and ordering:**

#### Placement, harvest and masonry inputs

Ordinary placement and authoritative component/command writes always select state `14847`;
rotation and mirror are identity operations. Wrong-tool removal produces no block loot. Any
pickaxe admits the one-roll loot table, which offers one matching item behind
`survives_explosion` and uses random sequence `minecraft:blocks/red_nether_bricks`. Silk Touch and
Fortune do not otherwise alter the table.

Seven exact recipes consume or produce this identity:

- a shaped 2-by-2 checkerboard converts two `nether_brick` items and two `nether_wart` items into
  one red-Nether-bricks block;
- three blocks produce six red-Nether-brick slabs, while stonecutting one produces two;
- six blocks in the stair pattern produce four red-Nether-brick stairs, while stonecutting one
  produces one; and
- six blocks in two rows produce six red-Nether-brick walls, while stonecutting one produces one.

Each record has a separate advancement. The source record puts `has_nether_wart` and
`has_the_recipe` in one OR requirement; each derived record instead puts
`has_red_nether_bricks` and `has_the_recipe` in one OR requirement. Completion grants only the
corresponding recipe. Generic grid transforms, ingredient consumption, stonecutter selection,
output admission and recipe-book publication remain with their owners.

At bootstrap only, red-Nether-brick stairs copy this block's legacy properties, the slab copies
them before constructing its slab behavior, and the wall copies them before forcing solid
support. Those derived blocks retain their own states, shapes and placement owners; later changes
to state `14847` do not mutate their registered properties.

#### Slow-bouncy sulfur-cube equipment

The block item directly selects the `slow_bouncy` archetype. That locked record fixes
horizontal/vertical knockback powers `0.4125/0.24`, `slow_bouncy.hit` and `slow_bouncy.push`
sounds, push cooldown `0.5`, impulse threshold `0.05`, and five attribute entries: additive
knockback and explosion-knockback resistance
`0.4000000059604645/0.4000000059604645`, additive bounciness `0.6000000238418579`,
total-multiplied friction `-0.699999988079071`, and total-multiplied air drag
`-0.949999999254942`.

Matching order, equipment replacement, transient modifier removal/addition, contact and knockback
math, sounds and entity projection remain with the sulfur-cube and entity owners. Reload changes
future tag/archetype matching without changing placed or saved block state.

#### Advancement display and generation absence

`nether/root` uses the ordinary red-Nether-bricks item only as its display icon. Completion tests
the independent `entered_nether` changed-dimension criterion, sends its telemetry event and does
not inspect this item or block.

The exhaustive NBT scan finds zero matching cells in all 1,212 bundled structure templates. The
bundled worldgen-data sweep has no red-Nether-bricks reference, and the server class-reference
sweep finds no runtime generator. Outside registration and the derived-property copies, exact
class references are item/creative publication, data generation and historical data fixes.

**Client projection:**

The only blockstate variant unconditionally selects `minecraft:block/red_nether_bricks`. That
model inherits `cube_all` and maps every face to `minecraft:block/red_nether_bricks`; the item
selector points directly to the same model. Authoritative block updates publish state `14847`,
inventory projection uses item ID `606`, material sounds use IDs `1093/1094/1095/1096/1097`, map
projection uses `NETHER`, and the Nether-root icon projects the ordinary item stack. This leaf adds
no packet field, acknowledgement or connection-local state.

**Branches and aborts:**

Ordinary/component placement; wrong tool versus any pickaxe; player versus explosion removal and
survived/suppressed loot; seven recipe matches, transforms, output capacities and OR unlocks;
current/reloaded slow-bouncy snapshots; empty/other/red-Nether-bricks sulfur body equipment;
advancement display versus criterion completion; zero generation/template selection; ordinary
state versus block/item/sound/map projection; save/reload are distinct branches.

**Constants and randomness:**

State/block/item IDs `14847/673/606`; hardness/resistance `2/6`; sound volume/pitch `1/1`; sound
IDs break/step/place/hit/fall `1093/1094/1095/1096/1097`; emission `0`, dampening `15`, shade
`0.2`, friction `0.6`, speed/jump `1`, restitution `0`, stack `64`; recipe input/output counts as
listed above; slow-bouncy powers `0.4125/0.24`, cooldown `0.5`, threshold `0.05` and five modifier
amounts as listed above; scanned templates/cells `1212/0`. The block consumes no RNG; loot,
crafting and sulfur-cube owners retain their streams.

**Side effects:**

Ordinary full-block placement/removal and gated self loot; seven recipe results/grants; a
reload-selected slow-bouncy equipment profile; one advancement display icon; ordinary
palette/inventory persistence; Nether-bricks sounds, Nether map shading and opaque cube-all
projection.

**Gates:**

World-write and break authority; correct pickaxe and explosion context; active loot, recipe,
advancement, item-tag and archetype snapshots; crafting/stonecutter output admission; sulfur
body-equipment admission; valid registry, map, sound and client-resource context.

**Boundary cases and quirks:**

The block requires a correct tool but no minimum pickaxe tier. Its source recipe is a checkerboard,
not four identical inputs; possession of Nether wart alone can unlock it. The Nether-root icon is
presentation and does not participate in the entered-Nether criterion. Zero template cells and
worldgen references are an audited absence, not a claim that players or commands cannot place it.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-REPORT-001`; `OFF-DATA-001`;
`net.minecraft.world.level.block.Blocks`;
`net.minecraft.world.level.block.SoundType`;
`net.minecraft.world.entity.monster.cubemob.SulfurCube#matchingArchetypes`;
`reports/blocks.json#minecraft:red_nether_bricks`;
`reports/registries.json#minecraft:{block,item}/minecraft:red_nether_bricks`;
`reports/registries.json#minecraft:sound_event/minecraft:block.nether_bricks.*`;
`reports/minecraft/components/item/red_nether_bricks.json`;
`data/minecraft/loot_table/blocks/red_nether_bricks.json`;
`data/minecraft/recipe/{red_nether_bricks,red_nether_brick_slab*,red_nether_brick_stairs*,red_nether_brick_wall*}.json`;
`data/minecraft/advancement/recipes/{building_blocks,decorations}/red_nether_brick*.json`;
`data/minecraft/advancement/nether/root.json`;
`data/minecraft/tags/block/mineable/pickaxe.json`;
`data/minecraft/tags/item/sulfur_cube_archetype/slow_bouncy.json`;
`data/minecraft/sulfur_cube_archetype/slow_bouncy.json`;
`data/minecraft/{worldgen,structure}/**`;
`assets/minecraft/blockstates/red_nether_bricks.json`;
`assets/minecraft/models/block/red_nether_bricks.json`;
`assets/minecraft/items/red_nether_bricks.json`.

**Test vectors:**

Run `EXP-BLK-063` across state/registry identity, ordinary/component writes,
wrong-tool/each-pickaxe and ordinary/explosion loot, all seven recipes/unlock alternatives,
slow-bouncy reload/equipment selection, the Nether-root display/criterion split, all 1,212
structure inputs, persistence, sounds, map color and both models. Assert exact constants, outputs,
zero generation/template joins and vanilla-client convergence.

**Limits:**

Generic placement, breaking, loot, crafting, stonecutting, advancements, sulfur-cube
equipment/contact/knockback, packet encoding and client rendering remain with `BLK-PLACE-001`,
`PLY-BREAK-001`, `ITM-LOOT-001`, `ITM-RECIPE-001`, `ITM-STONECUTTER-001`,
`ITM-ADVANCEMENT-001`, `ENT-KNOCKBACK-001`, `PROTO-PLAY-CLIENTBOUND-BLOCK-001`,
`PROTO-PLAY-CLIENTBOUND-SOUND-001` and `CLI-006`.
