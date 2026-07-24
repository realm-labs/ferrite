# Blocks mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `BLK-BRICKS-001` — Bricks join masonry recipes, slow-bouncy equipment and structure palettes

**Parent:** `BLK-001`, `BLK-002`, `BLK-003`, `BLK-005`, `PLY-005`, `PLY-006`, `ITM-004`,
`ITM-006`, `ENT-001`, `ENT-005`, `ENV-003`, `WGEN-003`, `CLI-006`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — the locked registration, reports, complete loot/recipe/advancement/tag data,
server class-reference search, all 1,212 decoded structure templates and exact client assets
exhaust this property-free identity. The block has no implementation subclass or live
identity-specific callback. Its content joins are correct-tool self loot, eight recipe/unlock
records, one slow-bouncy sulfur-cube membership and 2,558 raw cells in 31 structure templates.

**Applies when:**

`minecraft:bricks` is placed, written, harvested, exploded, used by crafting or stonecutting,
equipped on a sulfur cube, selected from a structure template, persisted, mapped or rendered.

**Authoritative state:**

Bricks is an ordinary property-free `Block` with no block entity and sole state `2340`. Its
locked block protocol ID is `176`, and its block-item raw ID is `332`. Registration selects map
color `COLOR_RED`, note instrument `BASEDRUM`, hardness/resistance `2/6`, the default `STONE`
sound type and `requiresCorrectToolForDrops`.

The state is a full unit selection/collision/visual/occlusion cube with emission `0`, light
dampening `15`, shade brightness `0.2`, friction `0.6`, speed/jump factors `1`, restitution `0`,
solid redstone conduction, normal piston reaction, full sturdy faces and ordinary full-face spawn
support. It adds no random or scheduled tick, use, attack, entity-contact, neighbor, signal,
comparator or block-event override. Its only direct block tag is `mineable/pickaxe`; no tool-tier
tag contains it, so any pickaxe satisfies the correct-tool gate.

The stone sound type selects sound registry IDs break `1596`, step `1604`, place `1601`, hit
`1600` and fall `1599`. The ordinary block item is common, stacks to `64`, has standard block-item
components and is directly in the reloadable `sulfur_cube_archetype/slow_bouncy` item tag.

**Transition and ordering:**

#### Placement, harvest and masonry inputs

Ordinary placement and authoritative component/command writes always select state `2340`;
rotation and mirror are identity operations. Wrong-tool removal produces no block loot. Any
pickaxe admits the one-roll loot table, which offers one matching item behind
`survives_explosion` and uses random sequence `minecraft:blocks/bricks`; an admitted explosion can
therefore suppress the entry. Silk Touch and Fortune do not otherwise change the table.

Eight exact recipes consume or produce this identity:

- a shaped 2-by-2 square converts four `brick` items into one `bricks`;
- three bricks produce six brick slabs, while stonecutting one produces two;
- six bricks in the stair pattern produce four brick stairs, while stonecutting one produces one;
- six bricks in two rows produce six brick walls, while stonecutting one produces one; and
- one paper plus one bricks block shapelessly produces one `field_masoned_banner_pattern`.

Each record has its own advancement with `has_brick` or `has_bricks` and `has_the_recipe` in one
OR requirement and grants only that recipe. The resulting banner-pattern item provides
`#minecraft:pattern_item/field_masoned`, whose sole pattern is `minecraft:bricks`; later loom and
banner-layer behavior stays with `ITM-LOOM-001` and `BLK-BANNER-001`. Grid transforms,
ingredient consumption, stonecutter selection, output admission and recipe-book publication stay
with their generic owners.

At bootstrap only, brick stairs copy the bricks block's legacy properties and brick wall copies
them before forcing solid support. Brick slab declares the same red/base-drum/correct-tool/2/6
profile directly. Those derived blocks retain their own state, shape and placement owners; later
changes to state `2340` do not mutate their registered properties.

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
future tag/archetype matching without changing a placed or saved bricks state.

#### Structure-template inputs

The exhaustive NBT scan finds 2,558 raw state-`2340` cells in 31 of 1,212 templates:

- 27 trail-ruins templates contain 2,526 cells;
- cold ocean-ruin templates `big_brick_3`, `big_cracked_3` and `big_mossy_3` contain ten each; and
- village template `plains_armorer_house_1` contains two.

The 27 trail inputs pass bricks unchanged through the trail aging rules, subject to ordinary
jigsaw transform, clipping and write admission. Each cold ocean-ruin overlay independently gates
its ten raw cells through the owning position-seeded integrity and ordered-overlay transaction.
The plains armorer cells pass through the owning village pool, projection and processor pipeline.
Selection probability, rotation, integrity, processor replacement, overlap, clipping and final
world writes remain with `WGEN-JIGSAW-TRAIL-RUINS-001`,
`WGEN-STRUCTURE-OCEAN-RUIN-001` and `WGEN-JIGSAW-VILLAGES-001`.

The server class-reference sweep finds no direct runtime generator using `Blocks.BRICKS`; outside
those template payloads, registration, item/creative publication and data generation are the only
locked exact references.

**Client projection:**

The only blockstate variant unconditionally selects `minecraft:block/bricks`. That model inherits
`cube_all` and maps every face to `minecraft:block/bricks`; the item selector points directly to
the same model. Authoritative block updates publish state `2340`, inventory projection uses item
ID `332`, material sounds use IDs `1596/1604/1601/1600/1599`, and map projection uses
`COLOR_RED`. This leaf adds no packet field, acknowledgement or connection-local state.

**Branches and aborts:**

Ordinary/component/template placement; wrong tool versus any pickaxe; player versus explosion
removal and survived/suppressed loot; eight recipe matches, transforms, output capacities and OR
unlocks; current/reloaded slow-bouncy snapshots; empty/other/bricks sulfur body equipment; all
three structure families, transforms, processor outcomes and chunk clips; ordinary state versus
block/item/sound/map projection; save/reload are distinct branches.

**Constants and randomness:**

State/block/item IDs `2340/176/332`; hardness/resistance `2/6`; sound IDs
break/step/place/hit/fall `1596/1604/1601/1600/1599`; emission `0`, dampening `15`, shade `0.2`,
friction `0.6`, speed/jump `1`, restitution `0`, stack `64`; recipe input/output counts as listed
above; slow-bouncy powers `0.4125/0.24`, cooldown `0.5`, threshold `0.05` and five modifier
amounts as listed above; template files/cells `31/2558`. The block consumes no RNG; loot,
crafting, sulfur-cube and structure owners retain their selection streams.

**Side effects:**

Ordinary full-block placement/removal and gated self loot; eight recipe results/grants; a
reload-selected slow-bouncy equipment profile; structure-palette writes through three owning
pipelines; ordinary palette/inventory persistence; stone sounds, red map shading and opaque
cube-all projection.

**Gates:**

World-write and break authority; correct pickaxe and explosion context; active loot, recipe,
advancement, item-tag, archetype, structure, pool and processor snapshots; crafting/stonecutter
output admission; sulfur body-equipment admission; structure start/transform/integrity/clip/write
admission; valid registry, map, sound and client-resource context.

**Boundary cases and quirks:**

The block requires a correct tool but no minimum pickaxe tier. The six-input shaped stair recipe
yields four, whereas the one-input stonecutting recipe yields one; slab and wall conversions have
their exact separate ratios. The physical block, the field-masoned pattern item and the banner
pattern all use the identifier word `bricks`, but they are different registry objects. Raw
structure-cell counts are inputs to processor and write gates, not guaranteed final placed counts.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-REPORT-001`; `OFF-DATA-001`;
`net.minecraft.world.level.block.Blocks`;
`net.minecraft.world.entity.monster.cubemob.SulfurCube#matchingArchetypes`;
`reports/blocks.json#minecraft:bricks`;
`reports/registries.json#minecraft:{block,item}/minecraft:bricks`;
`reports/registries.json#minecraft:sound_event/minecraft:block.stone.*`;
`reports/minecraft/components/item/bricks.json`;
`data/minecraft/loot_table/blocks/bricks.json`;
`data/minecraft/recipe/{bricks,brick_slab,brick_slab_from_bricks_stonecutting,brick_stairs,brick_stairs_from_bricks_stonecutting,brick_wall,brick_wall_from_bricks_stonecutting,field_masoned_banner_pattern}.json`;
`data/minecraft/advancement/recipes/{building_blocks,decorations,misc}/*.json`;
`data/minecraft/tags/block/mineable/pickaxe.json`;
`data/minecraft/tags/item/sulfur_cube_archetype/slow_bouncy.json`;
`data/minecraft/sulfur_cube_archetype/slow_bouncy.json`;
`data/minecraft/banner_pattern/bricks.json`;
`data/minecraft/tags/banner_pattern/pattern_item/field_masoned.json`;
`data/minecraft/structure/{trail_ruins,underwater_ruin,village}/**/*.nbt`;
`assets/minecraft/blockstates/bricks.json`;
`assets/minecraft/models/block/bricks.json`;
`assets/minecraft/items/bricks.json`.

**Test vectors:**

Run `EXP-BLK-059` across state/registry identity, ordinary/component/template writes,
wrong-tool/each-pickaxe and ordinary/explosion loot contexts, all eight recipes and unlock paths,
slow-bouncy reload/equipment selection, all 1,212 structure inputs and three owning pipelines,
save/reload, stone sounds, map color and both block/item models. Assert exact constants, outputs,
matching, raw template counts, processor boundaries and vanilla-client convergence.

**Limits:**

Generic placement, breaking, loot, crafting, stonecutting, advancements, loom/banner behavior,
sulfur-cube equipment/contact/knockback, structure selection/processing, packet encoding and client
rendering remain with `BLK-PLACE-001`, `PLY-BREAK-001`, `ITM-LOOT-001`, `ITM-RECIPE-001`,
`ITM-STONECUTTER-001`, `ITM-ADVANCEMENT-001`, `ITM-LOOM-001`, `BLK-BANNER-001`,
`ENT-KNOCKBACK-001`, the three named worldgen leaves, `PROTO-PLAY-CLIENTBOUND-BLOCK-001`,
`PROTO-PLAY-CLIENTBOUND-SOUND-001` and `CLI-006`.
