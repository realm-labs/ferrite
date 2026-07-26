# Blocks mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `BLK-BONE-BLOCK-001` — Bone blocks rotate on placement, compact Bone Meal and form three fossil payloads

**Parent:** `BLK-001`, `BLK-002`, `BLK-003`, `BLK-005`, `PLY-005`, `PLY-006`, `ITM-004`,
`ITM-006`, `ENT-001`, `ENT-005`, `WGEN-003`, `CLI-006`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — the locked `RotatedPillarBlock` implementation and registration, reports,
complete loot/recipe/advancement/tag search, class-reference sweep, all 1,212 decoded templates,
three owning generation leaves and exact client assets exhaust this identity. Its only runtime
specialization is the generic axis-placement/rotation implementation; all acquisition,
Sulfur-Cube and world-generation joins are fixed data consumed by already-audited algorithms.

**Applies when:**

`minecraft:bone_block` is placed against a face, rotated or mirrored, explicitly written, mined,
exploded, compacted from or decomposed to Bone Meal, equipped on a Sulfur Cube, selected by Bastion
loot, generated in a fossil or Trial Chamber, persisted, mapped or rendered.

**Authoritative state:**

Bone Block is a `RotatedPillarBlock` with no block entity. Its sole property is
`axis={x,y,z}`: states `14848/14849/14850` respectively, with Y state `14849` the default. The
locked block protocol ID is `674` and its ordinary block-item raw ID is `607`.

Registration supplies map color `SAND`, note instrument `XYLOPHONE`, correct-tool-required
hardness/resistance `2/2` and `BONE_BLOCK` sounds. The state is a full unit
selection/collision/visual/occlusion cube with emission `0`, light dampening `15`, shade brightness
`0.2`, friction `0.6`, speed/jump factors `1`, restitution `0`, normal piston reaction, solid
redstone conduction, full sturdy faces and ordinary full-face spawn support. It adds no random or
scheduled tick, use, attack, contact, step, fall, neighbor, signal, comparator, fluid or block-event
override.

Its sole direct block tag is `mineable/pickaxe`; no required-tier tag contains it. Every Pickaxe is
therefore a correct harvest tool, while a hand and every other tool can remove it but fail the
correct-tool loot gate. The sound type has volume/pitch `1/1` and selects exact registry IDs break
`186`, step `190`, place `189`, hit `188` and fall `187`.

The ordinary common-rarity block item stacks to `64`, has the standard block-item
name/model/placement components and is directly in `sulfur_cube_archetype/regular`.

**Transition and ordering:**

### Axis placement, transform and harvest

Ordinary placement starts from default state `14849` and replaces `axis` with the clicked face's
axis. Clicking an East/West face yields X, Up/Down yields Y and North/South yields Z. Explicit
component, command and template writes retain their supplied legal state.

Clockwise or counterclockwise quarter turns exchange X and Z while retaining Y. A half turn,
no rotation and every mirror retain the axis. Structure-template transforms apply that same rule to
each stored state. Axis changes affect only state/model orientation: all three states keep the same
physical, note, map, loot and tag behavior.

After a successful survival removal, only a Pickaxe passes the correct-tool admission and reaches
the one-roll loot table. It offers one Bone Block behind `survives_explosion`, using random sequence
`minecraft:blocks/bone_block`; Silk Touch and Fortune add no branch. An admitted explosion can
suppress the entry. Wrong-tool player removal emits nothing even though the table itself has no
tool predicate. Generic Pickaxe durability, speed, stats, exhaustion, game-mode and explosion
sequencing remain with the breaking and loot owners.

### Lossless Bone-Meal compacting and recipe knowledge

The shaped `minecraft:bone_block` recipe requires a filled 3-by-3 grid of nine exact Bone Meal
ingredients and returns one default Bone Block. The shapeless
`minecraft:bone_meal_from_bone_block` recipe consumes one exact Bone Block and returns nine default
Bone Meal, in group `bonemeal`. Together they are component-discarding and count-lossless but do not
preserve a patched input stack.

Each recipe has one advancement whose single requirement is an OR: already having the recipe, or
obtaining Bone Meal for compacting and Bone Block for decompression. Its reward grants only the
matching recipe. Grid offset/mirror, shapeless matching, component normalization, inventory
consumption, capacity and recipe-book publication remain with the generic crafting/progression
owners.

### Regular Sulfur-Cube equipment

The block item directly selects the reloadable `regular` archetype. That record is buoyant and
fixes horizontal/vertical knockback powers `0.4125/0.09`, `regular.hit` and `regular.push` sounds,
push cooldown `0.5`, impulse threshold `0.2`, and five modifiers: additive knockback resistance
`-1`, additive explosion-knockback resistance `-1`, additive bounciness `0.5`, total-multiplied
friction `-0.699999988079071`, and total-multiplied air drag `-0.8999999985098839`.

Matching order, equipment replacement, transient modifier lifecycle, buoyancy, collision,
knockback, sounds and entity projection remain with the Sulfur-Cube/entity owners. Reload changes
future item-tag/archetype selection without mutating placed Bone Blocks.

### Bastion acquisition

`chests/bastion_other` pool three makes uniformly `3..4` rolls with replacement across total weight
`13`. Bone Block has weight `1` and, when selected, emits a uniformly integral count `3..6`.
Consequently an evaluated chest has exact probability `6961/28561` (about `24.3724%`) of at least
one Bone-Block stack from this pool and expected Bone-Block count `63/52` (about `1.21154`).
The other four pools, container seed, jigsaw selection and table evaluation remain with
`WGEN-JIGSAW-BASTION-001` and `ITM-LOOT-001`.

### Overworld fossils

The upper and lower fossil configured features share eight ordered primary templates:
`spine_1..4`, then `skull_1..4`. Their raw Bone-Block totals and X/Y/Z axis counts are:

| Template | Total | X | Y | Z |
|---|---:|---:|---:|---:|
| `spine_1` | 37 | 0 | 24 | 13 |
| `spine_2` | 61 | 24 | 24 | 13 |
| `spine_3` | 97 | 48 | 36 | 13 |
| `spine_4` | 121 | 72 | 36 | 13 |
| `skull_1` | 86 | 44 | 24 | 18 |
| `skull_2` | 75 | 44 | 25 | 6 |
| `skull_3` | 58 | 35 | 11 | 12 |
| `skull_4` | 32 | 14 | 18 | 0 |

This is `567` raw cells: X/Y/Z totals `281/198/88`. A placement selects only one paired index. The
primary `fossil_rot` processor independently retains candidate blocks at integrity `0.9` before
the protected-live-state gate, so these are source payload counts rather than guaranteed writes.
The paired `_coal` overlay contains no Bone Blocks and later offers Coal Ore or, for the lower
variant, processed Deepslate Diamond Ore. Feature rotation can exchange the raw X/Z counts.
Selection, burial, corner admission, processor randomness, clipping and writes remain exactly with
`WGEN-PIPELINE-001`.

### Nether fossils and Trial Chambers

All 14 Nether-fossil templates are complete air/Bone-Block payloads and contain `183` Bone Blocks.
Per-template totals for `fossil_1..14` are
`10,10,6,6,5,21,18,6,15,8,24,11,17,26`; aggregate X/Y/Z counts are `49/129/5`.
The selected template's rotation exchanges X/Z as owned by
`WGEN-STRUCTURE-NETHER-FOSSIL-001`; template air is ignored and every retained bone cell is offered
without a block-rot processor.

Four reachable Trial-Chamber spawner templates contribute another `12` raw cells, all axis Y:
`spawner/ranged/{poison_skeleton,skeleton}` contain `2/4`, and
`spawner/slow_ranged/{poison_skeleton,skeleton}` contain `2/4`. Alias selection, jigsaw transform,
clip/protection/write admission and trial-spawner initialization remain with
`WGEN-JIGSAW-TRIAL-CHAMBERS-001`.

The exhaustive scan therefore finds exact Bone Block in `26` of `1,212` templates and `762` raw
cells: Overworld fossils `8/567`, Nether fossils `14/183`, Trial Chambers `4/12`. No other template
palette contains it.

The complete direct server-data search has exactly eight JSON files: block loot, Bastion-other
loot, two recipes, two recipe advancements and the two direct tags already stated. Outside
registration, publication, data generators and compatibility tables, the class-reference sweep
finds no identity-specific runtime consumer. There is no other loot table, recipe, advancement,
villager trade, configured-feature record, optional built-in-pack record or runtime callback naming
Bone Block.

**Client projection:**

The blockstate maps X to `block/bone_block` with rotations X/Y `90/90`, Y to the unrotated model,
and Z to model rotation X `90`. The model inherits `block/cube_column`, using
`block/bone_block_top` on axis ends and `block/bone_block_side` elsewhere. The item definition
selects that same block model directly. English translation is `Bone Block`; the Natural Blocks tab
places it once, after Soul Soil and before Blackstone.

Block updates publish states `14848..14850`, inventory paths publish item ID `607`, material sounds
use IDs `186..190`, and map projection uses `SAND`. This identity adds no packet field,
acknowledgement, particle or connection-local state.

**Branches and aborts:**

Six clicked faces and three axes; ordinary versus explicit/template state; quarter/half/no
rotation and mirror; Pickaxe versus wrong-tool/player versus explosion removal and
survives-explosion result; both recipe shapes, near misses, output capacity and two OR unlocks;
current/reloaded regular archetype; every Bastion pool roll and count endpoint; eight Overworld
templates, upper/lower processor paths, 14 Nether templates and four Trial templates; every
transform, clip, protection and write result; three durable states and block/item/sound/map/model
projection are distinct.

**Constants and randomness:**

States X/Y/Z `14848/14849/14850`; block/item IDs `674/607`; hardness/resistance `2/2`; full-cube
physical constants as above; sound volume/pitch `1/1`, break/step/place/hit/fall IDs
`186/190/189/188/187`; stack `64`; recipe counts `9:1` and `1:9`; regular powers `0.4125/0.09`,
cooldown `0.5`, threshold `0.2` and five modifiers as listed; Bastion rolls `3..4`, total/entry
weight `13/1`, count `3..6`, probability `6961/28561`, expectation `63/52`; raw template census
`26/762`, split `8/567`, `14/183`, `4/12`; Overworld block integrity `0.9`. The block itself
consumes no RNG; loot and each generation owner retain their named streams.

**Side effects:**

Axis-selected full-block placement/removal; correct-tool and explosion-gated self loot; reversible
Bone-Meal recipe results and knowledge grants; reload-selected regular Sulfur-Cube equipment;
Bastion inventory entries; fossil and Trial-Chamber template writes; ordinary persistence, map
shading, sounds and oriented cube-column projection.

**Gates:**

World-write and transform authority; successful removal/correct Pickaxe/explosion survival; live
recipe/advancement and output admission; live item tag/archetype; Bastion table evaluation;
feature/structure/jigsaw selection, processor/protection/clip/write admission; valid registry,
sound, map and client-resource snapshots.

**Boundary cases and quirks:**

The default is vertical but ordinary placement uses the clicked face rather than player look.
Quarter turns swap only horizontal axes, while mirrors and half turns are state identities.
Pickaxe membership is both the speed and no-tier correct-drop route; wrong-tool removal reaches no
loot. Compacting is exactly reversible in count but strips input component patches. The 762
template cells are mutually exclusive source alternatives, and Overworld fossil rot can remove
primary cells before any overlay. Bastion probability is for at least one pool-three stack, not
the per-roll `1/13` chance.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-REPORT-001`; `OFF-DATA-001`;
`net.minecraft.world.level.block.Blocks`;
`net.minecraft.world.level.block.RotatedPillarBlock`;
`net.minecraft.world.level.block.SoundType`;
`net.minecraft.world.item.CreativeModeTabs`;
`net.minecraft.world.entity.monster.cubemob.SulfurCube#matchingArchetypes`;
`reports/blocks.json#minecraft:bone_block`;
`reports/registries.json#minecraft:{block,item}/minecraft:bone_block`;
`reports/registries.json#minecraft:sound_event/minecraft:block.bone_block.*`;
`reports/minecraft/components/item/bone_block.json`;
`data/minecraft/loot_table/{blocks/bone_block,chests/bastion_other}.json`;
`data/minecraft/recipe/{bone_block,bone_meal_from_bone_block}.json`;
`data/minecraft/advancement/recipes/{building_blocks/bone_block,misc/bone_meal_from_bone_block}.json`;
`data/minecraft/tags/block/mineable/pickaxe.json`;
`data/minecraft/tags/item/sulfur_cube_archetype/regular.json`;
`data/minecraft/sulfur_cube_archetype/regular.json`;
`data/minecraft/structure/{fossil,nether_fossils,trial_chambers}/**/*.nbt`;
`assets/minecraft/blockstates/bone_block.json`;
`assets/minecraft/models/block/bone_block.json`;
`assets/minecraft/items/bone_block.json`;
`assets/minecraft/lang/en_us.json`.

**Test vectors:**

Run `EXP-BLK-090` across all axis states, face placements and transforms; every Pickaxe/wrong-tool,
ordinary/explosion loot branch; both recipes and unlock paths; regular-archetype reload/equipment;
Bastion weight/count boundaries; every fossil and Trial template, processor/transform/clip/write
branch; persistence, wire IDs, sounds, map color and block/item projection. Assert exact constants,
the `26/762` raw census, all owning-algorithm joins and vanilla-client convergence.

**Limits:**

Generic placement, breaking, loot, crafting, advancements, Sulfur-Cube
equipment/contact/knockback, Bastion/fossil/jigsaw/template placement, packet encoding and rendering
remain with `BLK-PLACE-001`, `PLY-BREAK-001`, `ITM-LOOT-001`, `ITM-RECIPE-001`,
`ITM-ADVANCEMENT-001`, `ENT-KNOCKBACK-001`, `WGEN-JIGSAW-BASTION-001`,
`WGEN-PIPELINE-001`, `WGEN-STRUCTURE-NETHER-FOSSIL-001`,
`WGEN-JIGSAW-TRIAL-CHAMBERS-001`, `PROTO-PLAY-CLIENTBOUND-BLOCK-001`,
`PROTO-PLAY-CLIENTBOUND-SOUND-001` and `CLI-006`. This leaf fixes exact Bone-Block identity, axis
specialization, data joins, absences and projection.
