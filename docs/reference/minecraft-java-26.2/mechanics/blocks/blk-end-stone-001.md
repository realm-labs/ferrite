# Blocks mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `BLK-END-STONE-001` — End stone joins End terrain, portal lifecycle, chorus and End-city masonry

**Parent:** `SIM-004`, `SIM-005`, `SIM-RANDOM-001`, `BLK-001`,
`BLK-STATE-001`, `BLK-002`, `BLK-PLACE-001`, `BLK-BREAK-001`,
`BLK-BREAK-HOOK-001`, `PLY-005`, `PLY-006`, `PLY-BREAK-001`, `BLK-003`,
`BLK-005`, `BLK-UPDATE-001`, `ITM-003`, `ITM-004`, `ITM-006`,
`ITM-RECIPE-001`, `ITM-CRAFT-001`, `ITM-STONECUTTER-001`, `ITM-LOOT-001`,
`ITM-ADVANCEMENT-001`, `ENT-001`, `ENT-005`, `ENT-KNOCKBACK-001`,
`MOB-AI-001`, `WGEN-003`, `WGEN-PIPELINE-001`, `WGEN-PORTAL-001`,
`WGEN-STRUCTURE-END-CITY-001`, `BLK-CHORUS-001`, `CLI-001`, `CLI-006`,
`CLI-UI-001`, `CLI-EFFECT-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked registrations and reports, complete loot/recipe/advancement/tag/class-
reference searches, all End terrain and code-built identity consumers, all 1,212 decoded templates,
the complete End-city generator, and exact client assets exhaust both identities. The ordinary
End Stone block has no raw structure-template cell; all 576 family template cells are exact End
Stone Bricks in eight reachable End-city inputs.

**Applies when:**

`minecraft:end_stone` or `minecraft:end_stone_bricks` is placed, mined, exploded, crafted, cut,
consumed by template duplication, selected by a live tag or Sulfur-Cube archetype, used as End
terrain/portal support, written by an End feature or End-city piece, persisted, mapped or rendered.

**Authoritative state:**

Both identities are property-free ordinary `Block` instances without block entities:

| Identity | State | Block protocol ID | Item raw ID |
|---|---:|---:|---:|
| End Stone | `9477` | `393` | `463` |
| End Stone Bricks | `14796` | `661` | `464` |

Both registrations fix `SAND` map color, `BASEDRUM`, `requiresCorrectToolForDrops`,
hardness/resistance `3/9` and ordinary Stone sounds. Each state is a full unit
selection/collision/visual/occlusion cube with emission `0`, light dampening `15`, shade brightness
`0.2`, friction `0.6`, speed/jump factors `1`, restitution `0`, solid redstone conduction, normal
piston reaction, full sturdy faces and ordinary spawn support. Neither adds a random or scheduled
tick, use, attack, contact, neighbor, signal, comparator, fluid or block-event override.

End Stone's direct block tags are `dragon_immune`, `mineable/pickaxe`, `sculk_replaceable`,
`supports_chorus_flower` and `supports_chorus_plant`; End Stone Bricks belongs directly only to
`mineable/pickaxe`. Neither belongs to a minimum-tier tag, so every Pickaxe is correct. Stone sound
volume/pitch is `1/1`, with exact event IDs break `1596`, step `1604`, place `1601`, hit `1600` and
fall `1599`.

Both common block items stack to `64`, have only standard block-item components, and directly
belong to `sulfur_cube_archetype/slow_bouncy`.

**Transition and ordering:**

### Placement, transforms and self loot

Placement, explicit writes, rotation and mirror retain state `9477` or `14796`; no legal
block-state component can add a property.

After successful survival removal, any Pickaxe reaches the identity's one-roll table. It offers one
matching block item behind `survives_explosion`, using random sequence
`minecraft:blocks/end_stone` or `minecraft:blocks/end_stone_bricks`. Silk Touch and Fortune add no
branch. Wrong-tool player removal emits nothing, and an admitted explosion can independently
suppress the entry.

### Masonry and Eye-template recipes

The locked family has twelve exact recipe records:

| Input | Machine/shape | Output |
|---|---|---|
| 4 End Stone | shaped `##/##` | 4 End Stone Bricks |
| 1 End Stone | Stonecutter | 1 End Stone Bricks |
| 1 End Stone | Stonecutter | 2 End Stone Brick Slabs |
| 1 End Stone | Stonecutter | 1 End Stone Brick Stairs |
| 1 End Stone | Stonecutter | 1 End Stone Brick Wall |
| 7 Diamonds + 1 Eye template + 1 End Stone | shaped `#S#/#C#/###` | 2 Eye templates |
| 3 End Stone Bricks | shaped `###` | 6 slabs |
| 6 End Stone Bricks | shaped `#  /## /###` | 4 stairs |
| 6 End Stone Bricks | shaped `###/###` | 6 walls |
| 1 End Stone Bricks | Stonecutter | 2 slabs |
| 1 End Stone Bricks | Stonecutter | 1 stairs |
| 1 End Stone Bricks | Stonecutter | 1 wall |

Possessing End Stone or prior knowledge unlocks each of the first five masonry records separately.
Possessing End Stone Bricks or prior knowledge unlocks each of its six descendant records. The Eye
duplication record instead unlocks from possession of the Eye template or its own prior knowledge;
End Stone possession alone does not reveal it. All shaped and Stonecutter results are default
stacks, so arbitrary input component patches are discarded. Shape offsets, result admission,
Stonecutter publication, consumption and the slab/stair/wall states remain with their generic
owners. No record converts Bricks or a descendant back to End Stone.

### Reloadable selectors and chorus

The `dragon_immune` membership makes an Ender Dragon collision treat End Stone as blocking rather
than remove it even when mob griefing permits ordinary destruction. The direct
`sculk_replaceable` membership admits ordinary Sculk spread and Sculk-vein substrate replacement
and composes into `sculk_replaceable_world_gen`.

End Stone is the sole locked member of both chorus-support tags. A Chorus Plant can survive above
it; a Chorus Flower can survive above it and receives the support-specific upward-growth branch.
The configured Chorus-Plant feature likewise requires the block below its origin to belong to
`supports_chorus_plant` before invoking code-built generation. Exact neighbor scheduling, growth
draws and writes remain with `BLK-CHORUS-001` and `WGEN-PIPELINE-001`.

Both item identities select `slow_bouncy`. Its record fixes horizontal/vertical knockback powers
`0.4125/0.24`, hit/push sounds, push cooldown `0.5`, impulse threshold `0.05`, additive knockback
and explosion-knockback resistance `0.4000000059604645/0.4000000059604645`, additive bounciness
`0.6000000238418579`, total-multiplied friction `-0.699999988079071` and total-multiplied air drag
`-0.9499999992549419`. Matching order, modifier lifecycle, contact and entity projection remain
with the Sulfur-Cube owners.

### End terrain, islands, podium and gateway search

The locked End noise setting uses default state `9477`, default air, noise range `0..127`,
horizontal/vertical cell sizes `2/1`, sea level `0`, disabled aquifers/ore veins, disabled noise-
stage mob generation and legacy RNG. Its surface-rule tree is the same unconditional End Stone
state. The `flat_all_dimensions` preset independently builds literal End from one Bedrock layer
then three End Stone layers. Density, surface and flat-generation algorithms remain with
`WGEN-PIPELINE-001`.

The configured `end_island` feature writes only default End Stone. It draws initial radius
`nextInt(3)+4`, visits inclusive disk layers while radius is greater than `0.5`, uses the enlarged
`X²+Z² <= (radius+1)²` predicate and shrinks by `nextInt(2)+0.5` before descending. Its exact
traversal, flags and decorated wrapper are specified by `WGEN-PIPELINE-001`.

`EndPodiumFeature` visits offsets X/Z `-4..4` and Y `-1..32`. At Y `-1`, the 16 positions inside
distance `3.5` but outside distance `2.5` receive End Stone. The active form first destroys the
previous cell without drops and then writes the default state; the inactive form directly writes
it. The inner disk, rim, portal/air column and Bedrock identities are separate branches.

During dragon respawn, `EnderDragonFight#respawnDragon` repeatedly matches the exit-portal pattern
and replaces every matched Bedrock or End Portal cell with End Stone by `setBlockAndUpdate` before
entering `DragonRespawnStage.START`. Other matched cells are untouched.

For an unconfigured End gateway, the destination-chunk scan considers only End Stone cells from Y
`30` through the highest filled section that have two non-full-collision cells above. It retains
strictly smaller squared distance to world origin, hence the first scan encounter on a tie. An
absent candidate invokes the End-island fallback; later highest-full-block selection and reciprocal
gateway creation remain with `WGEN-PORTAL-001`. End Stone Bricks never qualifies for this exact
terrain search.

### End-city brick payload

The exhaustive scan finds zero End Stone cells and `576` End Stone Bricks cells in eight of all
`1,212` templates:

| Reachable End-city template | Cells | Placement mode |
|---|---:|---|
| `base_floor` | 54 | overwrite |
| `fat_tower_top` | 106 | overwrite |
| `second_floor_1` | 84 | preserve template air |
| `second_floor_2` | 82 | preserve template air |
| `ship` | 36 | overwrite |
| `third_floor_1` | 90 | preserve template air |
| `third_floor_2` | 98 | preserve template air |
| `tower_top` | 26 | overwrite |

All eight names have live source edges in the code-built End-city graph. The four overwrite pieces
ignore structure blocks only; the other four ignore structure blocks and template air. Neither
processor alters a retained End Stone Bricks state, so every raw cell reaches rotation, clip, live-
target and flags-`2` write admission unchanged. Graph selection, recursive collision transaction,
rotation, clipping, overwrite-sensitive air, fluid/shape repair and later mutation remain with
`WGEN-STRUCTURE-END-CITY-001`; `576` is a raw payload census, not a guaranteed final-world count.

The exact direct data search has 22 JSON files naming End Stone and 18 naming End Stone Bricks.
Recipe linkage additionally reaches the Eye-template unlock record even though that advancement
does not name its End Stone ingredient. Outside registrations, data generators and generic
publication, the runtime class sweep finds End Stone only in `TheEndGatewayBlockEntity`,
`EnderDragonFight`, `EndIslandFeature`, `EndPodiumFeature`, `NoiseGeneratorSettings`,
`WorldPresets$Bootstrap` and creative publication. No identity-specific runtime class consumes End
Stone Bricks. No configured/placed feature or code-built structure writes Bricks outside the eight
End-city templates.

**Client projection:**

Each property-free blockstate selects its same-named block model. Both inherit `block/cube_all`
with their same-named texture, and each item directly selects the block model.

English translations are `End Stone` and `End Stone Bricks`. Natural Blocks publishes End Stone
once after Smooth Basalt and before Coal Ore. Building Blocks publishes End Stone, End Stone
Bricks, End Stone Brick Stairs, Slab and Wall, then Purpur Block. End Stone is also the icon for
the `The End?` story advancement and the hidden-toast `The End` advancement root; their dimension-
change criteria do not inspect the block. Updates use states `9477/14796`, inventory paths use item
IDs `463/464`, sounds use IDs `1596/1604/1601/1600/1599`, and maps use `SAND`. Neither identity
adds a packet field or connection-local state.

**Branches and aborts:**

Two sole states; ordinary versus explicit/feature/template write; Pickaxe versus wrong tool;
ordinary/explosion loot; twelve recipe records and their independent unlocks; current/reloaded five
block tags and two item memberships; Dragon destruction, Sculk and chorus outcomes; noise/flat/
island/podium/respawn/gateway paths; eight End-city templates through overwrite/preserve,
selection/rotation/clip/write outcomes; save/reload and block/item/icon projection are distinct.

**Constants and randomness:**

State/block/item IDs `9477/393/463` and `14796/661/464`; strength `3/9`; emission `0`, dampening
`15`, shade `0.2`, friction `0.6`, speed/jump `1`, restitution `0`; sound
break/step/place/hit/fall IDs `1596/1604/1601/1600/1599`, volume/pitch `1/1`; stack `64`;
recipe ratios as tabled; End flat layers `1+3`; podium radii `2.5/3.5`, End-Stone cells `16`;
template files/cells `8/576`; slow-bouncy values as listed. The blocks consume no RNG; loot,
equipment, features and End-city owners retain their streams.

**Side effects:**

Block placement/removal and self loot; masonry and template-duplication results/knowledge;
reload-selected Dragon/Sculk/chorus/equipment decisions; End terrain, island, podium and respawn
writes; gateway destination selection; End-city source writes; advancement icons; ordinary
persistence, Sand maps, Stone sounds and opaque cube projection.

**Gates:**

World-write/break authority; correct Pickaxe and explosion survival; recipe/advancement snapshot;
grid/Stonecutter result admission; live tag/archetype snapshots; End noise/preset/feature,
podium/respawn/gateway state; End-city graph, processor, rotation, clip and write admission; valid
registry/map/sound/client-resource context.

**Boundary cases and quirks:**

Every Pickaxe is correct without a tier gate. Direct Stonecutting can bypass intermediate Bricks
for slabs, stairs or walls. End Stone does not itself unlock Eye-template duplication. End Stone is
Dragon-immune yet is explicitly replaced during the separate dragon-respawn portal teardown.
Gateway destination search requires exact End Stone, not Bricks or another full cube. Natural End
Stone occurs in no structure payload, while all 576 Bricks cells are structure payloads. The
End-city overwrite distinction changes air handling but not any of those nonair brick states.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-REPORT-001`; `OFF-DATA-001`;
`net.minecraft.world.level.block.Blocks`;
`net.minecraft.world.entity.boss.enderdragon.EnderDragon#checkWalls`;
`net.minecraft.world.level.block.entity.TheEndGatewayBlockEntity#findValidSpawnInChunk`;
`net.minecraft.world.level.dimension.end.EnderDragonFight#respawnDragon`;
`net.minecraft.world.level.levelgen.feature.EndIslandFeature#place`;
`net.minecraft.world.level.levelgen.feature.EndPodiumFeature#place`;
`net.minecraft.world.level.levelgen.NoiseGeneratorSettings`;
`net.minecraft.world.level.levelgen.presets.WorldPresets$Bootstrap`;
`net.minecraft.world.level.levelgen.structure.structures.EndCityPieces`;
`net.minecraft.world.item.CreativeModeTabs`;
`reports/blocks.json#minecraft:{end_stone,end_stone_bricks}`;
`reports/registries.json#minecraft:{block,item}/minecraft:{end_stone,end_stone_bricks}`;
`reports/registries.json#minecraft:sound_event/minecraft:block.stone.*`;
`reports/minecraft/components/item/{end_stone,end_stone_bricks}.json`;
`data/minecraft/loot_table/blocks/{end_stone,end_stone_bricks}.json`;
`data/minecraft/recipe/{end_stone_bricks*,end_stone_brick_slab*,end_stone_brick_stairs*,end_stone_brick_wall*,eye_armor_trim_smithing_template}.json`;
`data/minecraft/advancement/{end/root,story/enter_the_end,recipes/{building_blocks/end_stone*,decorations/end_stone*,misc/eye_armor_trim_smithing_template}}.json`;
`data/minecraft/tags/block/{dragon_immune,mineable/pickaxe,sculk_replaceable,supports_chorus_flower,supports_chorus_plant}.json`;
`data/minecraft/tags/item/sulfur_cube_archetype/slow_bouncy.json`;
`data/minecraft/sulfur_cube_archetype/slow_bouncy.json`;
`data/minecraft/worldgen/noise_settings/end.json`;
`data/minecraft/worldgen/world_preset/flat_all_dimensions.json`;
`data/minecraft/structure/end_city/*.nbt`;
`assets/minecraft/blockstates/{end_stone,end_stone_bricks}.json`;
`assets/minecraft/models/block/{end_stone,end_stone_bricks}.json`;
`assets/minecraft/items/{end_stone,end_stone_bricks}.json`;
`assets/minecraft/lang/en_us.json`.

**Test vectors:**

Run `EXP-BLK-094` across placement and every tool/loot branch; all twelve recipes and unlocks; live
Dragon/Sculk/chorus/equipment snapshots; End noise/flat/island/podium/respawn/gateway paths; all
1,212 templates and every reachable End-city graph/processor/rotation/clip/write branch;
persistence, IDs, sounds, maps, advancement icons and exact projection. Assert exact constants,
zero End-Stone template cells, the `8/576` Bricks census and vanilla-client convergence.

**Limits:**

Generic placement, breaking, loot, crafting, Stonecutting, advancements, slab/stair/wall behavior,
Sulfur-Cube behavior, End feature algorithms, portal transfer, End-city construction, packet
encoding and rendering remain with `BLK-PLACE-001`, `PLY-BREAK-001`, `ITM-LOOT-001`,
`ITM-RECIPE-001`, `ITM-STONECUTTER-001`, `ITM-ADVANCEMENT-001`, `shape-family`,
`ENT-KNOCKBACK-001`, `WGEN-PIPELINE-001`, `WGEN-PORTAL-001`,
`WGEN-STRUCTURE-END-CITY-001`, `PROTO-PLAY-CLIENTBOUND-BLOCK-001`,
`PROTO-PLAY-CLIENTBOUND-SOUND-001` and `CLI-006`. This leaf fixes the exact End-Stone family
identity joins, absences and projection.
