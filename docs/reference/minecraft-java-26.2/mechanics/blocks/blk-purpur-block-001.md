# Blocks mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `BLK-PURPUR-BLOCK-001` — Purpur blocks join End-city palettes, masonry and advancement display

**Parent:** `BLK-001`, `BLK-002`, `BLK-003`, `BLK-005`, `PLY-005`, `PLY-006`, `ITM-004`,
`ITM-006`, `ENT-001`, `ENT-005`, `ENV-003`, `WGEN-003`, `CLI-006`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked registration, reports, loot/recipe/advancement/tag data, exhaustive
class-reference and 1,212-template scans, the complete End-city owner and exact client assets
exhaust this property-free ordinary block. Its joins are correct-tool self loot, eight recipe
records, seven recipe unlocks, one advancement icon, slow-bouncy equipment and 2,233 raw cells
across the 20 End-city templates.

**Applies when:**

`minecraft:purpur_block` is placed, harvested, exploded, crafted or stonecut, used to duplicate a
spire trim template, equipped on a sulfur cube, selected from an End-city template, displayed by
the End-city advancement, persisted, mapped or rendered.

**Authoritative state:**

Purpur block is an ordinary property-free `Block` with no block entity and sole state `14712`.
Its locked block protocol ID is `658`, and its block-item raw ID is `354`. Registration selects
map color `COLOR_MAGENTA`, note instrument `BASEDRUM`, hardness/resistance `1.5/6`, the default
`STONE` sound type and `requiresCorrectToolForDrops`.

The state is a full unit selection/collision/visual/occlusion cube with emission `0`, light
dampening `15`, shade brightness `0.2`, friction `0.6`, speed/jump factors `1`, restitution `0`,
solid redstone conduction, normal piston reaction, full sturdy faces and ordinary spawn support.
It adds no tick, interaction, contact, neighbor, signal, comparator or block-event override. Its
only direct block tag is `mineable/pickaxe`; no tool-tier tag contains it.

Stone material sounds use registry IDs break `1596`, step `1604`, place `1601`, hit `1600` and
fall `1599`. The common block item stacks to `64`, has standard block-item components and is
directly in `sulfur_cube_archetype/slow_bouncy`.

**Transition and ordering:**

Ordinary placement and component/command writes select state `14712`; rotation and mirror are
identity operations. Wrong-tool removal produces no loot. Any pickaxe admits the one-roll
`survives_explosion` self entry using random sequence `minecraft:blocks/purpur_block`; Silk Touch
and Fortune do not otherwise alter it.

Eight recipe records join this identity:

- four popped chorus fruit shape to four purpur blocks;
- three purpur blocks or pillars shape to six slabs, and one block stonecuts to two;
- six purpur blocks or pillars shape to four stairs, and one block stonecuts to one;
- two purpur slabs shape to one pillar, while one block stonecuts to one pillar; and
- seven diamonds, one spire template and one purpur block duplicate the template to two.

The first seven records have separate advancements: `has_popped_chorus_fruit` or
`has_purpur_block` shares an OR requirement with `has_the_recipe` and grants only its recipe. The
pillar crafting record does not consume the block, but its unlock does. The spire duplication
record has no recipe advancement. Generic crafting, stonecutting, recipe publication and smithing
template semantics retain their owners.

The item selects `slow_bouncy`, with powers `0.4125/0.24`, hit/push sounds, cooldown `0.5`,
threshold `0.05`, additive knockback/explosion resistance
`0.4000000059604645/0.4000000059604645`, bounciness `0.6000000238418579`, total-multiplied
friction `-0.699999988079071` and air drag `-0.949999999254942`. Matching, replacement,
knockback/contact and projection remain entity-owned.

All 20 End-city templates contain 2,233 raw state-`14712` cells. Nineteen reachable templates
contain 2,212; the source-unreferenced `tower_floor` contains the remaining 21 and is audited dead
data. Reachable cells pass unchanged through each owning transform, overwrite mode, clip and write
gate. Graph selection, recursion, collision, ship latch, markers and final writes remain with
`WGEN-STRUCTURE-END-CITY-001`. The class-reference sweep finds no direct runtime producer outside
those templates, registration and generic item/creative publication.

`end/find_end_city` uses the purpur-block item only as its display icon. Its location criterion,
telemetry and completion remain with the advancement and End-city owners.

**Client projection:**

The sole blockstate variant selects `minecraft:block/purpur_block`, an opaque `cube_all` model
mapping every face to the matching texture; the item selects the same model. Updates publish state
`14712`, inventory uses item ID `354`, sounds use `1596/1604/1601/1600/1599`, maps use
`COLOR_MAGENTA`, and the End-city advancement icon projects the ordinary item stack. No new packet
field or connection-local state exists.

**Branches and aborts:**

Ordinary/component/template writes; wrong tool versus any pickaxe; ordinary/explosion loot;
alternative block/pillar shaped ingredients, stonecutting and output capacity; seven OR unlocks;
spire duplication; current/reloaded equipment; 19 reachable versus dead `tower_floor` payload;
End-city selection/transform/clip/write; advancement display; save/reload and block/item projection.

**Constants and randomness:**

State/block/item IDs `14712/658/354`; strength `1.5/6`; sound IDs
`1596/1604/1601/1600/1599`; emission `0`, dampening `15`, shade `0.2`, friction `0.6`,
speed/jump `1`, restitution `0`, stack `64`; recipe ratios above; slow-bouncy constants above;
template files/cells `20/2233`, reachable files/cells `19/2212`, dead cells `21`. The block itself
consumes no RNG; loot, structure and entity owners retain their streams.

**Side effects:**

Correct-tool/explosion-gated self loot; eight recipe results and seven grants; slow-bouncy
equipment selection; End-city palette writes; one advancement icon; ordinary persistence, stone
sounds, magenta maps and cube-all projection.

**Gates:**

World-write/break authority; correct pickaxe and explosion context; active loot, recipe,
advancement, tag, archetype, structure and client-resource snapshots; crafting/stonecutter output;
sulfur equipment; End-city graph/transform/clip/write; valid registry/map/sound context.

**Boundary cases and quirks:**

Any pickaxe is correct despite the absence of a minimum-tier tag. Shaped slab/stair recipes accept
either purpur blocks or pillars; stonecutting accepts only blocks. `tower_floor` contributes raw
evidence but no ordinary generation writes. The End-city advancement icon is presentation, not a
criterion test for the block.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-REPORT-001`; `OFF-DATA-001`;
`net.minecraft.world.level.block.Blocks`;
`net.minecraft.world.entity.monster.cubemob.SulfurCube#matchingArchetypes`;
`reports/blocks.json#minecraft:purpur_block`;
`reports/registries.json#minecraft:{block,item}/minecraft:purpur_block`;
`reports/minecraft/components/item/purpur_block.json`;
`data/minecraft/loot_table/blocks/purpur_block.json`;
`data/minecraft/recipe/{purpur_block,purpur_slab*,purpur_stairs*,purpur_pillar*,spire_armor_trim_smithing_template}.json`;
`data/minecraft/advancement/recipes/building_blocks/purpur*.json`;
`data/minecraft/advancement/end/find_end_city.json`;
`data/minecraft/tags/block/mineable/pickaxe.json`;
`data/minecraft/tags/item/sulfur_cube_archetype/slow_bouncy.json`;
`data/minecraft/sulfur_cube_archetype/slow_bouncy.json`;
`data/minecraft/structure/end_city/*.nbt`;
`assets/minecraft/{blockstates,models/block,items}/purpur_block.json`.

**Test vectors:**

Run `EXP-BLK-062` across identity, writes, wrong-tool/each-pickaxe and explosion loot, eight
recipes/seven unlocks, slow-bouncy reload/equipment, all 20 End-city inputs including dead
`tower_floor`, save/reload, sounds, map, icon and both models. Assert exact constants, ratios,
20-file/2,233-cell raw and 19-file/2,212-cell reachable censuses and vanilla-client convergence.

**Limits:**

Generic placement, breaking, loot, crafting, stonecutting, advancement, sulfur-cube behavior,
End-city generation, packet encoding and rendering remain with `BLK-PLACE-001`, `PLY-BREAK-001`,
`ITM-LOOT-001`, `ITM-RECIPE-001`, `ITM-STONECUTTER-001`, `ITM-ADVANCEMENT-001`,
`ENT-KNOCKBACK-001`, `WGEN-STRUCTURE-END-CITY-001`,
`PROTO-PLAY-CLIENTBOUND-BLOCK-001`, `PROTO-PLAY-CLIENTBOUND-SOUND-001` and `CLI-006`.
