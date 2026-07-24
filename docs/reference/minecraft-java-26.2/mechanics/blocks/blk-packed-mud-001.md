# Blocks mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `BLK-PACKED-MUD-001` — Packed mud joins mud recipes, regular equipment and trail-ruins aging

**Parent:** `BLK-001`, `BLK-002`, `BLK-003`, `BLK-005`, `PLY-005`, `PLY-006`, `ITM-004`,
`ITM-006`, `ENT-001`, `ENT-005`, `ENV-003`, `WGEN-003`, `CLI-006`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — the locked registration, reports, complete loot/recipe/advancement/tag data,
server class-reference search, all 1,212 decoded structure templates, trail-ruins processors and
exact client assets exhaust this property-free identity. The block has no implementation subclass
or live identity-specific callback. Its content joins are tool-independent self loot, two
recipe/unlock records, one regular sulfur-cube membership, 68 raw trail-ruins cells and a
position-random trail-ruins processor output.

**Applies when:**

`minecraft:packed_mud` is placed, written, harvested, exploded, used by crafting, equipped on a
sulfur cube, read from or produced while processing a trail-ruins template, persisted, mapped or
rendered.

**Authoritative state:**

Packed mud is an ordinary property-free `Block` with no block entity and sole state `7758`. Its
locked block protocol ID is `330`, and its block-item raw ID is `407`. Registration legacy-copies
`DIRT`, then replaces hardness/resistance with `1/3` and sound with `PACKED_MUD`. It therefore keeps
map color `DIRT`, default note instrument `HARP` and no correct-tool requirement.

The state is a full unit selection/collision/visual/occlusion cube with emission `0`, light
dampening `15`, shade brightness `0.2`, friction `0.6`, speed/jump factors `1`, restitution `0`,
solid redstone conduction, normal piston reaction, full sturdy faces and ordinary full-face spawn
support. It adds no random or scheduled tick, use, attack, entity-contact, neighbor, signal,
comparator or block-event override. Its only direct block tag is `mineable/pickaxe`; that tag
accelerates suitable mining but does not create a loot admission gate.

The packed-mud sound type has volume/pitch `1/1` and selects sound registry IDs break `1116`, step
`1120`, place `1119`, hit `1118` and fall `1117`. The ordinary block item is common, stacks to
`64`, has standard block-item components and is directly in the reloadable
`sulfur_cube_archetype/regular` item tag.

**Transition and ordering:**

#### Placement, harvest and mud recipes

Ordinary placement and authoritative component/command writes always select state `7758`;
rotation and mirror are identity operations. Hand, wrong-tool and pickaxe removal all admit the
one-roll loot table, which offers one matching item behind `survives_explosion` and uses random
sequence `minecraft:blocks/packed_mud`; an admitted explosion can therefore suppress the entry.
Silk Touch and Fortune do not otherwise change the table.

Two exact recipes consume or produce this identity:

- one mud plus one wheat shapelessly produces one packed mud; and
- a shaped 2-by-2 square converts four packed mud into four mud bricks.

Each record has its own advancement. `has_mud` or `has_packed_mud` shares one OR requirement with
`has_the_recipe` and grants only the matching recipe. Grid transforms, ingredient consumption,
output admission and recipe-book publication remain with their generic owners.

#### Regular sulfur-cube equipment

The block item directly selects the `regular` archetype. That locked record is buoyant and fixes
horizontal/vertical knockback powers `0.4125/0.09`, `regular.hit` and `regular.push` sounds, push
cooldown `0.5`, impulse threshold `0.2`, and five attribute entries: additive knockback and
explosion-knockback resistance `-1/-1`, additive bounciness `0.5`, total-multiplied friction
`-0.699999988079071`, and total-multiplied air drag `-0.8999999985098839`.

Matching order, equipment replacement, transient modifier removal/addition, buoyancy, contact and
knockback math, sounds and entity projection remain with the sulfur-cube and entity owners. Reload
changes future tag/archetype matching without changing a placed or saved packed-mud state.

#### Trail-ruins inputs and processor output

The exhaustive NBT scan finds 68 raw state-`7758` cells in six of 1,212 templates, all selected
through the trail-ruins houses processor:

- `buildings/large_room_2` and `large_room_4` contain `9/6` cells; and
- `tower/hall_3`, `hall_4`, `hall_5` and `tower_1` contain `19/15/15/4` cells.

Those raw cells do not match any aging rule and pass through unchanged, subject to ordinary jigsaw
transform, clipping and write admission. Independently, the houses and roads processor lists test
each mud-bricks input with a position-derived float and replace it with state `7758` when the value
is below `0.1`. Tower-top inputs skip that aging stage. First-match rule order, connector
replacement, position-derived RNG, archaeology caps, overlap, clipping and final writes remain with
`WGEN-JIGSAW-TRAIL-RUINS-001`; raw cells and eligible mud-bricks inputs are not guaranteed final
packed-mud writes.

The server class-reference sweep finds no other direct runtime generator using
`Blocks.PACKED_MUD`; outside registration, item/creative publication and data generation, the
locked runtime inputs are the named templates and processor records.

**Client projection:**

The only blockstate variant unconditionally selects `minecraft:block/packed_mud`. That model
inherits `cube_all` and maps every face to `minecraft:block/packed_mud`; the item selector points
directly to the same model. Authoritative block updates publish state `7758`, inventory projection
uses item ID `407`, material sounds use IDs `1116/1120/1119/1118/1117`, and map projection uses
`DIRT`. This leaf adds no packet field, acknowledgement or connection-local state.

**Branches and aborts:**

Ordinary/component/template/processor placement; hand, wrong tool and pickaxe speed paths; player
versus explosion removal and survived/suppressed loot; both recipe matches, transforms, output
capacities and OR unlocks; current/reloaded regular snapshots; empty/other/packed-mud sulfur body
equipment; six raw templates versus other inputs, houses/roads/tower-top processors, mud-bricks
draw pass/fail, transform, clip and rejected write; ordinary state versus block/item/sound/map
projection; save/reload are distinct branches.

**Constants and randomness:**

State/block/item IDs `7758/330/407`; hardness/resistance `1/3`; sound volume/pitch `1/1`; sound IDs
break/step/place/hit/fall `1116/1120/1119/1118/1117`; emission `0`, dampening `15`, shade `0.2`,
friction `0.6`, speed/jump `1`, restitution `0`, stack `64`; recipe input/output counts as listed
above; regular powers `0.4125/0.09`, cooldown `0.5`, threshold `0.2` and five modifier amounts as
listed above; raw template files/cells `6/68`; processor threshold `0.1`. The block consumes no
RNG; loot, crafting, sulfur-cube and trail-ruins owners retain their selection streams.

**Side effects:**

Ordinary full-block placement/removal and explosion-gated self loot; two recipe results/grants; a
reload-selected buoyant regular equipment profile; raw and processor-produced trail-ruins palette
writes through the owning pipeline; ordinary palette/inventory persistence; packed-mud sounds,
dirt map shading and opaque cube-all projection.

**Gates:**

World-write and break authority; explosion context; active loot, recipe, advancement, item-tag,
archetype, structure, pool and processor snapshots; crafting output admission; sulfur
body-equipment admission; trail-ruins start/transform/random-rule/clip/write admission; valid
registry, map, sound and client-resource context.

**Boundary cases and quirks:**

The pickaxe tag changes mining suitability/speed but does not require a correct tool for loot.
Packed mud is both a raw template state and a possible mud-bricks processor result. The source
recipe is shapeless and yields one, while its 2-by-2 masonry recipe consumes four and yields four
mud bricks. Raw structure-cell counts and eligible processor inputs are not guaranteed final
placed counts.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-REPORT-001`; `OFF-DATA-001`;
`net.minecraft.world.level.block.Blocks`;
`net.minecraft.world.entity.monster.cubemob.SulfurCube#matchingArchetypes`;
`reports/blocks.json#minecraft:packed_mud`;
`reports/registries.json#minecraft:{block,item}/minecraft:packed_mud`;
`reports/registries.json#minecraft:sound_event/minecraft:block.packed_mud.*`;
`reports/minecraft/components/item/packed_mud.json`;
`data/minecraft/loot_table/blocks/packed_mud.json`;
`data/minecraft/recipe/{packed_mud,mud_bricks}.json`;
`data/minecraft/advancement/recipes/building_blocks/{packed_mud,mud_bricks}.json`;
`data/minecraft/tags/block/mineable/pickaxe.json`;
`data/minecraft/tags/item/sulfur_cube_archetype/regular.json`;
`data/minecraft/sulfur_cube_archetype/regular.json`;
`data/minecraft/worldgen/processor_list/trail_ruins_{houses,roads}_archaeology.json`;
`data/minecraft/structure/trail_ruins/**/*.nbt`;
`assets/minecraft/blockstates/packed_mud.json`;
`assets/minecraft/models/block/packed_mud.json`;
`assets/minecraft/items/packed_mud.json`.

**Test vectors:**

Run `EXP-BLK-060` across state/registry identity, ordinary/component/template/processor writes,
hand/wrong-tool/pickaxe mining and ordinary/explosion loot, both recipes and unlock paths, regular
reload/equipment selection, all 1,212 structure inputs and all three trail processor paths,
save/reload, packed-mud sounds, map color and both block/item models. Assert exact constants,
outputs, matching, six-file/68-cell raw census, the `0.1` processor boundary and vanilla-client
convergence.

**Limits:**

Generic placement, breaking, loot, crafting, advancements, sulfur-cube
equipment/contact/knockback, trail-ruins selection/processing, packet encoding and client rendering
remain with `BLK-PLACE-001`, `PLY-BREAK-001`, `ITM-LOOT-001`, `ITM-RECIPE-001`,
`ITM-ADVANCEMENT-001`, `ENT-KNOCKBACK-001`, `WGEN-JIGSAW-TRAIL-RUINS-001`,
`PROTO-PLAY-CLIENTBOUND-BLOCK-001`, `PROTO-PLAY-CLIENTBOUND-SOUND-001` and `CLI-006`.
