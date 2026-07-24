# Blocks mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `BLK-MUD-BRICKS-001` — Mud bricks join masonry recipes, slow-bouncy equipment and trail aging

**Parent:** `BLK-001`, `BLK-002`, `BLK-003`, `BLK-005`, `PLY-005`, `PLY-006`, `ITM-004`,
`ITM-006`, `ENT-001`, `ENT-005`, `ENV-003`, `WGEN-003`, `CLI-006`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — the locked registration, reports, complete loot/recipe/advancement/tag data,
server class-reference search, all 1,212 decoded structure templates, trail processors and exact
client assets exhaust this property-free identity. The block has no implementation subclass or
live identity-specific callback. Its joins are correct-tool self loot, seven recipe/unlock
records, slow-bouncy sulfur equipment, 3,870 raw trail cells, 19 connector final states and the
position-random aging input.

**Applies when:**

`minecraft:mud_bricks` is placed, written, harvested, exploded, crafted or stonecut, equipped on a
sulfur cube, read from or produced by a trail-ruins connector and processed by trail aging,
persisted, mapped or rendered.

**Authoritative state:**

Mud bricks is an ordinary property-free `Block` with no block entity and sole state `7759`. Its
locked block protocol ID is `331`, and its block-item raw ID is `408`. Registration selects map
color `TERRACOTTA_LIGHT_GRAY`, note instrument `BASEDRUM`, hardness/resistance `1.5/3`,
`requiresCorrectToolForDrops` and sound type `MUD_BRICKS`.

The state is a full unit selection/collision/visual/occlusion cube with emission `0`, light
dampening `15`, shade brightness `0.2`, friction `0.6`, speed/jump factors `1`, restitution `0`,
solid redstone conduction, normal piston reaction, full sturdy faces and ordinary full-face spawn
support. It adds no random or scheduled tick, use, attack, entity-contact, neighbor, signal,
comparator or block-event override. Its only direct block tag is `mineable/pickaxe`; no tool-tier
tag contains it, so any pickaxe satisfies the correct-tool gate.

The mud-bricks sound type has volume/pitch `1/1` and selects sound registry IDs break `1010`, step
`1014`, place `1013`, hit `1012` and fall `1011`. The ordinary block item is common, stacks to
`64`, has standard block-item components and is directly in the reloadable
`sulfur_cube_archetype/slow_bouncy` item tag.

**Transition and ordering:**

#### Placement, harvest and masonry recipes

Ordinary placement and authoritative component/command writes always select state `7759`;
rotation and mirror are identity operations. Wrong-tool removal produces no block loot. Any
pickaxe admits the one-roll loot table, which offers one matching item behind
`survives_explosion` and uses random sequence `minecraft:blocks/mud_bricks`; an admitted explosion
can therefore suppress the entry. Silk Touch and Fortune do not otherwise change the table.

Seven exact recipes consume or produce this identity:

- a shaped 2-by-2 square converts four packed mud into four mud bricks;
- three mud bricks produce six slabs, while stonecutting one produces two;
- six mud bricks in the stair pattern produce four stairs, while stonecutting one produces one;
- six mud bricks in two rows produce six walls, while stonecutting one produces one.

Each record has its own advancement. `has_packed_mud` or `has_mud_bricks` shares one OR requirement
with `has_the_recipe` and grants only that recipe. Grid transforms, ingredient consumption,
stonecutter selection, output admission and recipe-book publication remain with their generic
owners.

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
future tag/archetype matching without changing a placed or saved mud-bricks state.

#### Trail-ruins inputs, connector results and aging

The exhaustive NBT scan finds 3,870 raw state-`7759` cells in 40 of 1,212 templates. Thirty-nine
houses-processed templates contain 3,867 cells across the `buildings`, tower additions and tower
base directories. `tower/tower_top_1` contains the other three and selects the tower-top processor.

Jigsaw replacement can additionally produce 19 mud-bricks cells from connector final states in
aging-enabled inputs. The houses and roads processor lists then test every mud-bricks input with a
position-derived float and replace it with packed-mud state `7758` when the value is below `0.1`;
the state passes unchanged otherwise. Tower-top processing skips aging, so its three raw cells
remain mud bricks subject to ordinary clip/write admission.

Connector replacement, first-match rule order, position-derived RNG, archaeology caps, transform,
overlap, clipping and final writes remain with `WGEN-JIGSAW-TRAIL-RUINS-001`. Raw cells, connector
results and aging survivors are inputs to those gates, not guaranteed final mud-bricks writes.

The server class-reference sweep finds no other direct runtime generator using
`Blocks.MUD_BRICKS`; outside registration, item/creative publication and data generation, the
locked runtime inputs are the named templates, connectors and processor records.

**Client projection:**

The only blockstate variant selects `minecraft:block/mud_bricks_north_west_mirrored`. Its
full-cube parent maps the mud-bricks texture to every face and reverses the U coordinates on the
north and west faces. The item selector instead points to `minecraft:block/mud_bricks`, an ordinary
`cube_all` model using the same texture. Authoritative updates publish state `7759`, inventory
projection uses item ID `408`, sounds use IDs `1010/1014/1013/1012/1011`, and map projection uses
`TERRACOTTA_LIGHT_GRAY`. This leaf adds no packet field, acknowledgement or connection-local state.

**Branches and aborts:**

Ordinary/component/template/connector placement; wrong tool versus any pickaxe; player versus
explosion removal and survived/suppressed loot; seven recipe matches, transforms, output
capacities and OR unlocks; current/reloaded slow-bouncy snapshots; empty/other/mud-bricks sulfur
body equipment; 40 raw templates versus other inputs, houses/roads/tower-top processors, aging
draw pass/fail, transform, clip and rejected write; mirrored world model versus ordinary item
model; save/reload are distinct branches.

**Constants and randomness:**

State/block/item IDs `7759/331/408`; hardness/resistance `1.5/3`; sound volume/pitch `1/1`; sound
IDs break/step/place/hit/fall `1010/1014/1013/1012/1011`; emission `0`, dampening `15`, shade
`0.2`, friction `0.6`, speed/jump `1`, restitution `0`, stack `64`; recipe counts as listed above;
slow-bouncy powers `0.4125/0.24`, cooldown `0.5`, threshold `0.05` and five modifier amounts as
listed above; raw template files/cells `40/3870`, aging-enabled raw cells `3867`, tower-top raw
cells `3`, connector final states `19`, processor threshold `0.1`. The block consumes no RNG;
loot, crafting, sulfur-cube and trail-ruins owners retain their selection streams.

**Side effects:**

Ordinary full-block placement/removal and correct-tool/explosion-gated self loot; seven recipe
results/grants; reload-selected slow-bouncy equipment; raw, connector-produced and aging-surviving
trail palette writes through the owning pipeline; ordinary persistence; mud-bricks sounds,
light-gray terracotta map shading, mirrored world faces and ordinary cube-all item projection.

**Gates:**

World-write and break authority; correct pickaxe and explosion context; active loot, recipe,
advancement, item-tag, archetype, structure, pool and processor snapshots; crafting/stonecutter
output admission; sulfur body-equipment admission; trail start/connector/transform/random-rule/
clip/write admission; valid registry, map, sound and client-resource context.

**Boundary cases and quirks:**

Any pickaxe is correct because no minimum-tier tag contains the block. Aging can remove raw or
connector-produced mud bricks, but tower tops skip that rule. The shaped source recipe preserves
the four-to-four count. The world model mirrors north/west UVs while the inventory model does not.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-REPORT-001`; `OFF-DATA-001`;
`net.minecraft.world.level.block.Blocks`;
`net.minecraft.world.entity.monster.cubemob.SulfurCube#matchingArchetypes`;
`reports/blocks.json#minecraft:mud_bricks`;
`reports/registries.json#minecraft:{block,item}/minecraft:mud_bricks`;
`reports/registries.json#minecraft:sound_event/minecraft:block.mud_bricks.*`;
`reports/minecraft/components/item/mud_bricks.json`;
`data/minecraft/loot_table/blocks/mud_bricks.json`;
`data/minecraft/recipe/{mud_bricks,mud_brick_*}.json`;
`data/minecraft/advancement/recipes/{building_blocks,decorations}/mud_brick*.json`;
`data/minecraft/tags/block/mineable/pickaxe.json`;
`data/minecraft/tags/item/sulfur_cube_archetype/slow_bouncy.json`;
`data/minecraft/sulfur_cube_archetype/slow_bouncy.json`;
`data/minecraft/worldgen/processor_list/trail_ruins_{houses,roads,tower_top}_archaeology.json`;
`data/minecraft/structure/trail_ruins/**/*.nbt`;
`assets/minecraft/blockstates/mud_bricks.json`;
`assets/minecraft/models/block/{mud_bricks,mud_bricks_north_west_mirrored}.json`;
`assets/minecraft/items/mud_bricks.json`.

**Test vectors:**

Run `EXP-BLK-061` across identity, ordinary/component/template/connector writes, wrong-tool/each
pickaxe and ordinary/explosion loot, all seven recipes/unlocks, slow-bouncy reload/equipment
selection, all 1,212 structure inputs and all three trail processor paths, save/reload, sounds,
map color and distinct world/item models. Assert exact constants, counts, 40-file/3,870-cell raw
census, 19 connector finals, the `0.1` aging boundary and vanilla-client convergence.

**Limits:**

Generic placement, breaking, loot, crafting, stonecutting, advancements, sulfur-cube
equipment/contact/knockback, trail processing, packet encoding and client rendering remain with
`BLK-PLACE-001`, `PLY-BREAK-001`, `ITM-LOOT-001`, `ITM-RECIPE-001`,
`ITM-STONECUTTER-001`, `ITM-ADVANCEMENT-001`, `ENT-KNOCKBACK-001`,
`WGEN-JIGSAW-TRAIL-RUINS-001`, `PROTO-PLAY-CLIENTBOUND-BLOCK-001`,
`PROTO-PLAY-CLIENTBOUND-SOUND-001` and `CLI-006`.
