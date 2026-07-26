# Blocks mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `BLK-PURPUR-BLOCK-001` — Purpur blocks and pillars join oriented End-city palettes, masonry and advancement display

**Parent:** `BLK-001`, `BLK-002`, `BLK-003`, `BLK-005`, `PLY-005`, `PLY-006`, `ITM-004`,
`ITM-006`, `ENT-001`, `ENT-005`, `ENV-003`, `WGEN-003`, `CLI-006`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked `Block`/`RotatedPillarBlock` registrations and concrete transform code,
reports, complete loot/recipe/advancement/tag searches, class-reference and all-1,212-template
scans, the complete End-city owner and exact client assets exhaust both identities. Their joins are
correct-tool self loot, eight coupled recipes, seven recipe unlocks, one advancement icon,
slow-bouncy equipment and 2,794 raw cells across all 20 End-city templates.

**Applies when:**

`minecraft:purpur_block` or `minecraft:purpur_pillar` is placed, explicitly written, transformed,
harvested, exploded, crafted or stonecut, used to duplicate a Spire trim template, equipped on a
Sulfur Cube, selected from an End-city template, displayed by the End-city advancement, persisted,
mapped or rendered.

**Authoritative state:**

| Identity | Implementation | States | Default | Block/item IDs |
|---|---|---|---|---|
| Purpur Block | ordinary `Block` | `14712` | `14712` | `658/354` |
| Purpur Pillar | `RotatedPillarBlock`, `axis={x,y,z}` | `14713/14714/14715` for X/Y/Z | Y `14714` | `659/355` |

Neither identity has a block entity. Both registrations independently select map color
`COLOR_MAGENTA`, note instrument `BASEDRUM`, correct-tool-required hardness/resistance `1.5/6`
and default `STONE` sounds.

Every state is a full unit selection/collision/visual/occlusion cube with emission `0`, light
dampening `15`, shade brightness `0.2`, friction `0.6`, speed/jump factors `1`, restitution `0`,
solid redstone conduction, normal piston reaction, full sturdy faces and ordinary spawn support.
Neither adds a tick, interaction, contact, neighbor, signal, comparator, fluid or block-event
override. Each identity's only direct block tag is `mineable/pickaxe`; no tool-tier tag contains
either.

Stone sounds use exact registry IDs break `1596`, step `1604`, place `1601`, hit `1600` and fall
`1599`, at volume/pitch `1/1`. Both ordinary common block items stack to `64`, have standard
block-item components and directly belong to `sulfur_cube_archetype/slow_bouncy`.

**Transition and ordering:**

### Placement, transform and correct-tool loot

Purpur Block placement and every rotation/mirror retain state `14712`. Ordinary Pillar placement
starts at state `14714` and replaces `axis` with the clicked face's axis: East/West selects X,
Up/Down Y and North/South Z. Explicit component, command and template writes preserve their
supplied legal state.

Clockwise or counterclockwise quarter turns exchange Pillar X and Z while retaining Y. No
rotation, a half turn and every mirror retain its axis. End-city template transforms apply the
same rule to stored pillar states. Axis affects only state/model orientation; all three states keep
the same physical, map, note, tool, loot and sound behavior.

After successful survival removal, any Pickaxe passes each identity's correct-tool gate. Its
distinct one-roll table offers one matching item behind `survives_explosion`, using random sequence
`minecraft:blocks/purpur_block` or `minecraft:blocks/purpur_pillar`. A hand or non-Pickaxe removes
the state without loot; Silk Touch and Fortune add no branch. An admitted explosion can suppress
the matching self entry.

### Coupled masonry and recipe knowledge

Eight recipes join the family:

- four Popped Chorus Fruit in a 2-by-2 square produce four Purpur Blocks;
- a three-wide row whose cells each accept Block or Pillar produces six Purpur Slabs, while one
  Block stonecuts to two Slabs;
- the six-cell stair pattern whose cells each accept Block or Pillar produces four Purpur Stairs,
  while one Block stonecuts to one Stair;
- two vertical Purpur Slabs produce one Pillar, while one Block stonecuts to one Pillar; and
- seven Diamonds, one Spire template and one Purpur Block duplicate the template to two.

The shaped Slab and Stair inputs can mix Blocks and Pillars cell by cell. No recipe decomposes a
Pillar to a Block, and the three Stonecutter recipes accept Block but not Pillar.

The first seven records have separate advancements. Each record's `has_the_recipe` criterion
shares one OR requirement with `has_popped_chorus_fruit` for the base recipe or
`has_purpur_block` for the other six, then grants only the matching recipe. Thus both Pillar
recipes unlock from obtaining a Block even though the shaped one consumes only Slabs. The Spire
duplication record has no recipe advancement. Generic grid transforms, matching, default-result
components, capacity, consumption, Stonecutter publication and template duplication remain with
their generic owners.

### Slow-bouncy Sulfur-Cube equipment

Either block item directly selects `slow_bouncy`. Its locked record fixes horizontal/vertical
knockback powers `0.4125/0.24`, hit/push sounds, push cooldown `0.5`, impulse threshold `0.05`,
additive knockback and explosion-knockback resistance
`0.4000000059604645/0.4000000059604645`, additive bounciness `0.6000000238418579`,
total-multiplied friction `-0.699999988079071` and total-multiplied air drag
`-0.949999999254942`.

Matching order, equipment replacement, modifier lifecycle, buoyancy, contact, knockback, sound and
entity projection remain with the Sulfur-Cube/entity owners. Reload changes future item
classification without mutating either placed state family.

### End-city payload

All 20 locked End-city templates contain Purpur Block: 2,233 raw state-`14712` cells. Nineteen
reachable templates contain 2,212; source-unreferenced `tower_floor` contains the remaining 21.

Seventeen templates additionally contain 561 raw Pillars with aggregate X/Y/Z axes `11/471/79`:

| Template | Pillars | Stored axes |
|---|---:|---|
| `base_floor` | 12 | Y 12 |
| `bridge_end` | 1 | Z 1 |
| `bridge_gentle_stairs` | 2 | Z 2 |
| `bridge_piece` | 4 | Z 4 |
| `bridge_steep_stairs` | 2 | Z 2 |
| `fat_tower_base` | 84 | X/Y/Z 2/80/2 |
| `fat_tower_middle` | 164 | X/Y/Z 5/154/5 |
| `fat_tower_top` | 18 | Y 18 |
| `second_floor_1`, `second_floor_2` | 12 each | Y 12 each |
| `ship` | 97 | Y/Z 38/59 |
| `third_floor_1`, `third_floor_2` | 18/12 | Y 18 / Y 12 |
| `tower_base` | 39 | X/Y/Z 2/35/2 |
| `tower_piece` | 36 | X/Y/Z 2/32/2 |
| `tower_top` | 12 | Y 12 |
| dead `tower_floor` | 36 | Y 36 |

The 16 reachable Pillar-bearing templates therefore contain `525` cells; dead `tower_floor`
contains `36`. Across both identities, all 20 templates contain `2,794` raw cells, the 19 reachable
inputs contain `2,737`, and dead `tower_floor` contains `57`.

Reachable Block and Pillar cells pass unchanged through the owning overwrite mode, clip and write
gates; structure rotation can exchange Pillar X/Z. Graph selection, recursion, collision, ship
latch, markers and final writes remain with `WGEN-STRUCTURE-END-CITY-001`. Raw counts are source
payload, not guaranteed placed counts.

The class-reference sweep finds no other direct runtime producer outside those templates,
registrations, block-family metadata and generic item/creative publication. The complete
Pillar-specific data search is nine JSON files: self loot; two producing and two consuming recipes;
two recipe advancements; and its block/item tags. No other loot, recipe, advancement, trade,
configured feature or optional built-in-pack record names the Pillar.

`end/find_end_city` uses only the Purpur-Block item as its display icon. Its location criterion,
telemetry and completion neither test nor emit either block and remain with the advancement and
End-city owners.

**Client projection:**

Purpur Block state `14712` selects the opaque `block/purpur_block` `cube_all` model and matching
texture; its item selects the same model. Pillar Y selects `block/purpur_pillar`, a `cube_column`
with `purpur_pillar_top` ends and `purpur_pillar_side` sides. X/Z select
`block/purpur_pillar_horizontal`, a `cube_column_horizontal` with the same textures, using model
rotations X/Y `90/90` for X and X `90` for Z. The Pillar item selects the vertical block model.

English names are `Purpur Block` and `Purpur Pillar`. The Building Blocks tab places them once in
the order End Stone Brick Wall, Purpur Block, Purpur Pillar, Purpur Stairs, Purpur Slab. Updates use
states `14712..14715`, inventory paths use IDs `354/355`, sounds use
`1596/1604/1601/1600/1599`, maps use `COLOR_MAGENTA`, and the End-city advancement projects only
the Block item. Neither identity adds a packet field or connection-local state.

**Branches and aborts:**

Block versus three Pillar axes; six clicked faces; ordinary/component/template writes;
quarter/half/no rotation and mirror; wrong tool versus any Pickaxe; ordinary/explosion loot and
survival; eight recipe matches, mixed inputs, Stonecutter admission, output capacity and seven OR
unlocks; Spire duplication; current/reloaded equipment; 19 reachable versus dead `tower_floor`
payloads; End-city selection/transform/clip/write; advancement display; save/reload and both
block/item projections are distinct.

**Constants and randomness:**

Block state/block/item `14712/658/354`; Pillar X/Y/Z states `14713/14714/14715`, block/item
`659/355`; strength `1.5/6`; emission `0`, dampening `15`, shade `0.2`, friction `0.6`, speed/jump
`1`, restitution `0`; sound IDs `1596/1604/1601/1600/1599`, volume/pitch `1/1`; stacks `64`;
recipe ratios and slow-bouncy values as listed; Block templates raw/reachable/dead
`2233/2212/21`; Pillar `561/525/36` and X/Y/Z `11/471/79`; combined
`2794/2737/57`. The identities consume no RNG; loot, structure and entity owners retain their
streams.

**Side effects:**

Property-free or axis-selected full-block writes; correct-tool/explosion-gated matching self loot;
eight recipe results and seven knowledge grants; slow-bouncy equipment selection; oriented
End-city palette writes; one Block-only advancement icon; ordinary persistence, stone sounds,
magenta maps and cube/cube-column projection.

**Gates:**

World-write/transform/break authority; correct Pickaxe and explosion context; active loot, recipe,
advancement, tag, archetype, structure and client-resource snapshots; crafting/Stonecutter output;
Sulfur-Cube equipment; End-city graph/transform/clip/write; valid registry/map/sound context.

**Boundary cases and quirks:**

Any Pickaxe is correct despite no minimum-tier tag. Pillar placement uses the clicked face, not
player look; mirrors and half turns do not change axis. Shaped Slab/Stair recipes accept mixed
Block/Pillar cells, but Stonecutting accepts only Block. Obtaining Block unlocks the shaped Pillar
recipe even though it consumes Slabs. `tower_floor` contributes both identities' raw evidence but
no ordinary generation writes. The End-city advancement icon is presentation, not a criterion
test.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-REPORT-001`; `OFF-DATA-001`;
`net.minecraft.world.level.block.Blocks`;
`net.minecraft.world.level.block.RotatedPillarBlock`;
`net.minecraft.world.entity.monster.cubemob.SulfurCube#matchingArchetypes`;
`net.minecraft.world.item.CreativeModeTabs`;
`reports/blocks.json#minecraft:{purpur_block,purpur_pillar}`;
`reports/registries.json#minecraft:{block,item}/{purpur_block,purpur_pillar}`;
`reports/registries.json#minecraft:sound_event/minecraft:block.stone.*`;
`reports/minecraft/components/item/{purpur_block,purpur_pillar}.json`;
`data/minecraft/loot_table/blocks/{purpur_block,purpur_pillar}.json`;
`data/minecraft/recipe/{purpur_block,purpur_slab*,purpur_stairs*,purpur_pillar*,spire_armor_trim_smithing_template}.json`;
`data/minecraft/advancement/recipes/building_blocks/purpur*.json`;
`data/minecraft/advancement/end/find_end_city.json`;
`data/minecraft/tags/block/mineable/pickaxe.json`;
`data/minecraft/tags/item/sulfur_cube_archetype/slow_bouncy.json`;
`data/minecraft/sulfur_cube_archetype/slow_bouncy.json`;
`data/minecraft/structure/end_city/*.nbt`;
`assets/minecraft/blockstates/{purpur_block,purpur_pillar}.json`;
`assets/minecraft/models/block/{purpur_block,purpur_pillar,purpur_pillar_horizontal}.json`;
`assets/minecraft/items/{purpur_block,purpur_pillar}.json`;
`assets/minecraft/lang/en_us.json`.

**Test vectors:**

Run `EXP-BLK-062` across both identities, three Pillar axes, all face placements/transforms,
wrong-tool/every-Pickaxe and explosion loot, eight recipes/seven unlocks, slow-bouncy
reload/equipment, all 20 End-city inputs including dead `tower_floor`, persistence, sounds, map,
icon and every model. Assert exact constants, ratios, per-identity and combined raw/reachable/dead
censuses and vanilla-client convergence.

**Limits:**

Generic placement, transforms, breaking, loot, crafting, Stonecutting, advancement, Sulfur-Cube
behavior, End-city generation, packet encoding and rendering remain with `BLK-PLACE-001`,
`PLY-BREAK-001`, `ITM-LOOT-001`, `ITM-RECIPE-001`, `ITM-STONECUTTER-001`,
`ITM-ADVANCEMENT-001`, `ENT-KNOCKBACK-001`, `WGEN-STRUCTURE-END-CITY-001`,
`PROTO-PLAY-CLIENTBOUND-BLOCK-001`, `PROTO-PLAY-CLIENTBOUND-SOUND-001` and `CLI-006`. This leaf
fixes both Purpur full-block identities, their coupled data joins, absences and projection.
