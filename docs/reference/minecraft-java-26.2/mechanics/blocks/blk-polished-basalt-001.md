# Blocks mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `BLK-POLISHED-BASALT-001` — Polished basalt rotates, converts from basalt and forms Ancient-City and Bastion trim

**Parent:** `BLK-001`, `BLK-002`, `BLK-003`, `BLK-005`, `PLY-005`, `PLY-006`, `ITM-004`,
`ITM-006`, `ENT-001`, `ENT-005`, `WGEN-003`, `CLI-006`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — the locked `RotatedPillarBlock` implementation and registration, reports,
complete loot/recipe/advancement/tag and class-reference searches, all 1,212 decoded templates,
the complete Ancient-City/Bastion owners and exact client assets exhaust this identity. Its only
runtime specialization is generic face-axis placement/rotation; crafting, equipment and structure
joins are fixed data consumed by already-audited algorithms.

**Applies when:**

`minecraft:polished_basalt` is placed against a face, explicitly written, rotated or mirrored,
mined, exploded, crafted or stonecut from Basalt, equipped on a Sulfur Cube, placed from an
Ancient-City or Bastion payload, persisted, mapped or rendered.

**Authoritative state:**

Polished Basalt is a `RotatedPillarBlock` with no block entity. Its sole property is
`axis={x,y,z}`: states `7003/7004/7005` respectively, with Y state `7004` the default. Its locked
block protocol ID is `289`, and its ordinary block-item raw ID is `391`.

Registration independently repeats Basalt's values rather than legacy-copying it: map color
`COLOR_BLACK`, note instrument `BASEDRUM`, correct-tool-required hardness/resistance `1.25/4.2`
and `BASALT` sounds. Every axis state is a full unit selection/collision/visual/occlusion cube with
emission `0`, light dampening `15`, shade brightness `0.2`, friction `0.6`, speed/jump factors `1`,
restitution `0`, solid redstone conduction, normal piston reaction, full sturdy faces and ordinary
spawn support. It adds no random or scheduled tick, use, attack, contact, neighbor, signal,
comparator, fluid or block-event override.

Its sole direct block tag is `mineable/pickaxe`; no minimum-tier tag contains it. Every Pickaxe is
therefore correct, while a hand or other tool can remove it but fails the loot admission gate.
The sound type has volume/pitch `1/1` and exact event IDs break `142`, step `146`, place `145`, hit
`144` and fall `143`.

The ordinary common block item stacks to `64`, has only standard block-item components and directly
belongs to `sulfur_cube_archetype/slow_bouncy`.

**Transition and ordering:**

### Axis placement, transform and self loot

Ordinary placement starts from state `7004` and replaces `axis` with the clicked face's axis:
East/West selects X, Up/Down Y and North/South Z. Explicit component, command, jigsaw-final and
template writes preserve their supplied legal state.

Clockwise or counterclockwise quarter turns exchange X and Z while retaining Y. No rotation, a
half turn and every mirror retain the axis. Structure-template transforms apply the same rule.
Axis changes affect only state/model orientation; physical, map, note, tool, loot and sound
behavior remain identical.

After successful survival removal, any Pickaxe reaches the one-roll loot table. It offers one
Polished Basalt behind `survives_explosion`, using random sequence
`minecraft:blocks/polished_basalt`; Silk Touch and Fortune add no branch. Wrong-tool player removal
emits nothing, and an admitted explosion can suppress the self entry.

### Basalt conversion and recipe knowledge

Two building recipes produce default Polished Basalt:

- a shaped 2-by-2 square consumes four exact Basalt and returns four; and
- the Stonecutter consumes one exact Basalt and returns one.

Both are count-preserving but discard input component patches. Each record has its own advancement:
exact Basalt possession shares one OR requirement with `has_the_recipe`, then grants only the
matching recipe. No recipe consumes Polished Basalt, converts it to Basalt or Smooth Basalt, or
uses either latter identity as its input. Grid offset/mirror, Stonecutter publication, output
capacity, default-result components and inventory consumption remain with generic owners.

### Slow-bouncy Sulfur-Cube equipment

The item directly selects `slow_bouncy`. Its locked record fixes horizontal/vertical knockback
powers `0.4125/0.24`, hit/push sounds, push cooldown `0.5`, impulse threshold `0.05`, additive
knockback and explosion-knockback resistance
`0.4000000059604645/0.4000000059604645`, additive bounciness `0.6000000238418579`,
total-multiplied friction `-0.699999988079071` and total-multiplied air drag
`-0.949999999254942`.

Matching order, equipment replacement, modifier lifecycle, buoyancy, contact, knockback, sound and
entity projection remain with the Sulfur-Cube/entity owners. Reload changes future classification
without mutating placed states.

### Ancient-City raw payload

Twelve reachable Ancient-City templates contain `709` raw Polished-Basalt cells:

| Template | Total | Stored X/Y/Z |
|---|---:|---:|
| `city/entrance/entrance_connector` | 138 | 50/14/74 |
| `city/entrance/entrance_path_1` | 138 | 50/14/74 |
| `city/entrance/entrance_path_2` | 113 | 42/23/48 |
| `city/entrance/entrance_path_3` | 90 | 36/27/27 |
| `city/entrance/entrance_path_4` | 91 | 36/28/27 |
| `city/entrance/entrance_path_5` | 72 | 33/20/19 |
| `structures/ice_box_1` | 4 | 0/4/0 |
| `structures/tall_ruin_1` | 43 | 0/43/0 |
| `walls/intact_corner_wall_1` | 5 | 0/5/0 |
| `walls/intact_intersection_wall_1` | 5 | 0/5/0 |
| `walls/intact_lshape_wall_1` | 5 | 0/5/0 |
| `walls/ruined_corner_wall_1` | 5 | 0/5/0 |

Aggregate X/Y/Z counts are `247/193/269`; structure rotation can exchange X/Z.

The six entrance templates and `tall_ruin_1` use `ancient_city_generic_degradation`; the four wall
templates use `ancient_city_walls_degradation`. Polished Basalt is outside their rottable tag and
none of their substitution rules targets it, so it passes both stages unchanged. Their later
protected-block processor can still reject a cell over a live `features_cannot_replace` target.
`ice_box_1` is the sole one-child list with inline-empty processors, so its four cells skip both
degradation and protection. Transform, water preservation, clip and write admission remain with
`WGEN-JIGSAW-ANCIENT-CITY-001`.

### Bastion raw and connector-final payload

All 31 matching Bastion templates are reachable and contain `196` raw cells, all stored axis Y:

- bridge `bridge_pieces/bridge`, `starting_pieces/entrance_face` and
  `starting_pieces/entrance` contain `45/13/47`, total `105`;
- Hoglin-Stable `stairs_{1,2,3}_{0..4}` each follow counts `3/3/3/2/1`, total `36`;
- Hoglin-Stable `starting_pieces/stairs_{0..4}_mirrored` and
  `starting_stairs_{0..4}` each follow `3/3/3/2/1`, total `24`; and
- Treasure `corners/bottom/corner_1`, `walls/mid/wall_0` and
  `walls/top/main_entrance` contain `1/19/11`, total `31`.

Their `bridge`, `bastion_generic_degradation`, `entrance_replacement`, `stable_degradation` and
`treasure_rooms` rule lists name other input identities only. Polished Basalt therefore passes each
list unchanged, subject to ordinary transform, destructive payload overlap, clip and write gates;
no Bastion list protects live feature blocks.

Separately, Treasure `walls/mid/wall_{0,1,2}` each stores one rollable `center_connector` at local
`[4,0,11]`, targeting/pooling `minecraft:empty`, whose jigsaw final state is default axis-Y
Polished Basalt. Jigsaw replacement runs before `treasure_rooms`, which also leaves that result
unchanged. `wall_0` also owns its 19 raw cells; `wall_1/2` have no raw Polished Basalt. Thus the
Bastion source offers `196` raw cells plus three connector-final cells across a union of 33 files.

The exhaustive template-palette scan finds `905` raw cells in `43` of all `1,212` templates:
Ancient City `12/709` and Bastion `31/196`. Adding the three nonpalette jigsaw finals yields `908`
explicit source cells across a union of `45` templates. These counts are inputs, not guaranteed
final world writes.

The complete direct server-data search has seven JSON files: self loot; two recipes; two recipe
advancements; and the direct block/item tags. Outside registrations, generic publication, data
generators and compatibility tables, the class-reference sweep finds no identity-specific runtime
consumer. No other loot, recipe, advancement, trade, configured feature or optional built-in-pack
record names Polished Basalt.

**Client projection:**

The blockstate maps X to `block/polished_basalt` with model rotations X/Y `90/90`, Y to the
unrotated model, and Z to rotation X `90`. The model inherits `block/cube_column`, using
`block/polished_basalt_top` on axis ends and `block/polished_basalt_side` elsewhere. The item
selects the same vertical block model directly.

English translation is `Polished Basalt`. The Building Blocks tab places it once after Smooth
Basalt and before Blackstone, in the local order Basalt, Smooth Basalt, Polished Basalt,
Blackstone. State updates publish `7003..7005`, inventory paths use item ID `391`, sounds use
`142..146`, and maps use `COLOR_BLACK`. This identity adds no packet field or connection-local
state.

**Branches and aborts:**

Three axes and six clicked faces; ordinary versus explicit/template/jigsaw-final writes;
quarter/half/no rotation and mirror; any Pickaxe versus wrong tool; ordinary/explosion loot; shaped
versus Stonecutter recipe, malformed input, output capacity and two OR unlocks; current/reloaded
slow-bouncy selection; 12 Ancient templates and three processor/protection paths; 31 Bastion raw
templates, five processor lists and three connector finals; every transform/clip/write result;
save/reload and state/item/sound/map/model projection are distinct.

**Constants and randomness:**

X/Y/Z states `7003/7004/7005`; block/item IDs `289/391`; strength `1.25/4.2`; emission `0`,
dampening `15`, shade `0.2`, friction `0.6`, speed/jump `1`, restitution `0`; sound
break/step/place/hit/fall IDs `142/146/145/144/143`, volume/pitch `1/1`; stack `64`; recipe ratios
`4:4` and `1:1`; slow-bouncy values as listed; Ancient raw files/cells `12/709`, axes
`247/193/269`; Bastion raw `31/196`, connector finals `3`; global raw `43/905`, source union
`45/908`. The block consumes no RNG; loot, entity and structure owners retain their streams.

**Side effects:**

Axis-selected full-block placement/removal; correct-tool/explosion-gated self loot; two Basalt
conversion results and knowledge grants; reload-selected slow-bouncy equipment; Ancient-City and
Bastion raw/final palette writes; ordinary persistence, black maps, Basalt sounds and oriented
cube-column projection.

**Gates:**

World-write/transform/break authority; correct Pickaxe and explosion survival; recipe/advancement
snapshot and output admission; live item tag/archetype; Ancient/Bastion pool, processor,
protection, overlap, clip and write admission; valid registry/map/sound/client-resource context.

**Boundary cases and quirks:**

Ordinary placement uses the clicked face rather than player look; quarter turns exchange only X/Z,
and mirrors/half turns retain axis. Both recipes preserve counts but not component patches.
Polished Basalt is neither a geode layer nor interchangeable with Smooth Basalt. Ancient processors
leave its source state intact but may protect the live target; Bastion processors neither alter nor
protect it. The three Bastion connector finals are not palette hits, and two occur in templates
with no raw Polished Basalt, producing the `43`-file raw versus `45`-file source-union distinction.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-REPORT-001`; `OFF-DATA-001`;
`net.minecraft.world.level.block.Blocks`;
`net.minecraft.world.level.block.RotatedPillarBlock`;
`net.minecraft.world.entity.monster.cubemob.SulfurCube#matchingArchetypes`;
`net.minecraft.world.item.CreativeModeTabs`;
`reports/blocks.json#minecraft:polished_basalt`;
`reports/registries.json#minecraft:{block,item}/minecraft:polished_basalt`;
`reports/registries.json#minecraft:sound_event/minecraft:block.basalt.*`;
`reports/minecraft/components/item/polished_basalt.json`;
`data/minecraft/loot_table/blocks/polished_basalt.json`;
`data/minecraft/recipe/polished_basalt*.json`;
`data/minecraft/advancement/recipes/building_blocks/polished_basalt*.json`;
`data/minecraft/tags/block/mineable/pickaxe.json`;
`data/minecraft/tags/item/sulfur_cube_archetype/slow_bouncy.json`;
`data/minecraft/sulfur_cube_archetype/slow_bouncy.json`;
`data/minecraft/worldgen/template_pool/{ancient_city,bastion}/**/*.json`;
`data/minecraft/worldgen/processor_list/{ancient_city_*,bridge,entrance_replacement,bastion_generic_degradation,stable_degradation,treasure_rooms}.json`;
`data/minecraft/structure/{ancient_city,bastion}/**/*.nbt`;
`assets/minecraft/blockstates/polished_basalt.json`;
`assets/minecraft/models/block/polished_basalt.json`;
`assets/minecraft/items/polished_basalt.json`;
`assets/minecraft/lang/en_us.json`.

**Test vectors:**

Run `EXP-BLK-091` across all axis states, faces and transforms; every Pickaxe/wrong-tool and
ordinary/explosion loot path; both Basalt recipes/unlocks; slow-bouncy reload/equipment; all 43 raw
templates and three jigsaw-final sources across every processor/protection/clip/write branch;
persistence, IDs, sounds, map and block/item projection. Assert exact constants, the
`43/905` raw and `45/908` source-union censuses and vanilla-client convergence.

**Limits:**

Generic placement, breaking, loot, crafting, Stonecutting, advancements, Sulfur-Cube behavior,
jigsaw/template processing, packet encoding and rendering remain with `BLK-PLACE-001`,
`PLY-BREAK-001`, `ITM-LOOT-001`, `ITM-RECIPE-001`, `ITM-STONECUTTER-001`,
`ITM-ADVANCEMENT-001`, `ENT-KNOCKBACK-001`, `WGEN-JIGSAW-PROCESSORS-001`,
`WGEN-JIGSAW-ANCIENT-CITY-001`, `WGEN-JIGSAW-BASTION-001`,
`PROTO-PLAY-CLIENTBOUND-BLOCK-001`, `PROTO-PLAY-CLIENTBOUND-SOUND-001` and `CLI-006`. This leaf
fixes exact Polished-Basalt identity, axis specialization, data joins, absences and projection.
