# Blocks mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `BLK-MOSSY-COBBLESTONE-001` — Mossy cobblestone joins masonry recipes to rocks, dungeons and structure weathering

**Parent:** `SIM-004`, `SIM-005`, `SIM-RANDOM-001`, `BLK-001`,
`BLK-STATE-001`, `BLK-002`, `BLK-PLACE-001`, `BLK-BREAK-001`,
`BLK-BREAK-HOOK-001`, `BLK-BREAK-CONTENT-001`, `BLK-UPDATE-001`,
`PLY-002`, `PLY-005`, `PLY-006`, `PLY-INTERACT-001`, `PLY-BREAK-001`,
`PLY-COLLISION-001`, `PLY-AUTOJUMP-001`, `ITM-003`, `ITM-004`,
`ITM-006`, `ITM-RECIPE-001`, `ITM-CRAFT-001`,
`ITM-STONECUTTER-001`, `ITM-LOOT-001`, `ITM-ADVANCEMENT-001`,
`ITM-SMITHING-TEMPLATE-001`, `ITM-DISPENSER-001`,
`ENT-KNOCKBACK-001`, `ENV-001`, `ENV-002`, `ENV-003`,
`ENV-FIRE-001`, `ENV-FLUID-001`, `ENV-LIGHT-001`, `WGEN-003`,
`WGEN-PIPELINE-001`, `WGEN-JIGSAW-PROCESSORS-001`,
`WGEN-JIGSAW-VILLAGES-001`, `WGEN-JIGSAW-OUTPOST-001`,
`WGEN-JIGSAW-TRIAL-CHAMBERS-001`, `WGEN-STRUCTURE-OCEAN-RUIN-001`,
`WGEN-STRUCTURE-JUNGLE-TEMPLE-001`,
`WGEN-STRUCTURE-RUINED-PORTAL-001`, `CLI-001`, `CLI-006`,
`CLI-UI-001`, `CLI-EFFECT-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — the locked registration/reports, complete loot/recipe/
advancement/tag/data and compiled-field searches, all five producing processor
lists and their 125 exact pool entries, both procedural structures, the direct
feature, all 1,212 decoded templates and exact client resources exhaust the sole
block state and item.

**Applies when:**

`minecraft:mossy_cobblestone` is placed, mined, exploded, crafted from
Cobblestone, cut or crafted into masonry, consumed by Wild-template duplication,
selected by a Sulfur Cube or dispenser, emitted by a feature or procedural
structure, read or rewritten in a structure template, persisted, synchronized or
rendered.

**Authoritative state:**

Mossy Cobblestone is a property-free ordinary `Block` without a block entity:

| State ID | Block protocol ID | Item raw ID | Map color | Instrument |
|---:|---:|---:|---|---|
| `3368` | `192` | `348` | `STONE` | `BASEDRUM` |

Registration fixes hardness/resistance `2/6`,
`requiresCorrectToolForDrops` and ordinary Stone sounds. The state is a full
`0..16` selection/collision/visual/occlusion cube with emission `0`, light
dampening `15`, shade brightness `0.2`, friction `0.6`, speed/jump factors `1`,
restitution `0`, full sturdy faces, ordinary spawn support, solid redstone
conduction and default `NORMAL` piston reaction. It holds no fluid and produces
no signal or comparator output.

Sound volume/pitch is `1/1`; break/step/place/hit/fall event IDs are
`1596/1604/1601/1600/1599`. Its common nondamageable block item stacks to `64`
and has only standard block-item components.

The block directly belongs only to `mineable/pickaxe`; no minimum-tier tag
contains it, so every Pickaxe is correct. The item directly belongs only to
`sulfur_cube_archetype/slow_bouncy`, which is nested by
`sulfur_cube_swallowable`. There is no FireBlock ignite/burn row, lava-ignition
property, fuel time, Composter entry, repair tag or other direct membership.

**Transition and ordering:**

### Placement, breaking and self loot

Ordinary placement, explicit writes, rotation and mirror retain sole state
`3368`. The block adds no random/scheduled tick, use, attack, contact, neighbor,
signal, comparator, fluid or block-event override.

After successful survival removal, every Pickaxe reaches the one-roll block
table. It offers one Mossy Cobblestone behind `survives_explosion`, using random
sequence `minecraft:blocks/mossy_cobblestone`. Hand and wrong-tool player
removal emits nothing. Silk Touch and Fortune add no branch; an admitted
explosion can independently suppress the entry.

### Exact crafting, cutting and knowledge

Exactly nine bundled recipes name the full block semantically:

- shapeless Cobblestone plus exact Moss Block produces one Mossy Cobblestone;
- shapeless Cobblestone plus exact Vine produces one Mossy Cobblestone;
- one row of three Mossy Cobblestone produces six Mossy Cobblestone Slabs;
- the six-cell stair shape produces four Mossy Cobblestone Stairs;
- two full rows produce six Mossy Cobblestone Walls;
- Stonecutting one block produces respectively two Slabs, one Stair or one Wall;
  and
- seven Diamonds, one Wild Armor Trim Smithing Template and center Mossy
  Cobblestone in `#S#/#C#/###` produce two Wild templates.

All outputs are default stacks and discard arbitrary input component patches.
No recipe converts a shape descendant back into the full block and no cooking
record accepts or emits it.

The Moss Block and Vine producer advancements unlock from possession of their
respective additive, not Cobblestone. Possessing exact Mossy Cobblestone unlocks
the six shape/Stonecutting records independently. Wild-template duplication
unlocks from the Wild template, not its Mossy center material. Prior recipe
knowledge is the other member of every two-criterion OR requirement.

### Slow-bouncy Sulfur-Cube and dispenser joins

The direct item tag selects `slow_bouncy`. Its record fixes horizontal/vertical
knockback powers `0.4125/0.24`; additive knockback and explosion-knockback
resistance `0.4000000059604645/0.4000000059604645`; additive bounciness
`0.6000000238418579`; total-multiplied friction and air drag
`-0.699999988079071/-0.9499999992549419`; hit/push sounds, push cooldown `0.5`
and impulse threshold `0.05`. Matching, equipment mutation, attributes, contact
and sound remain with the Sulfur-Cube owners.

Because `sulfur_cube_swallowable` nests `slow_bouncy`, an otherwise unregistered
dispenser behavior searches the front AABB and lets the first accepting Sulfur
Cube consume one item. When none accepts, the protected default eject path runs.
Traversal, acceptance, shrinking and ejection remain with `ITM-DISPENSER-001`.

### Forest-rock and monster-room production

The fixed `forest_rock` block-blob configuration emits Mossy Cobblestone and
tests the reloadable `forest_rock_can_place_on` tag below its descending center.
Its placed record runs count `2`, in-square, `MOTION_BLOCKING` heightmap and
biome filtering, and is scheduled in Old Growth Pine and Old Growth Spruce
Taiga.

After support admission, each invocation runs three passes. Each samples three
binary extents, offers the resulting 1/1/5/19-cell Euclidean blob with flags
`3`, then samples a nonpositive center shift; the unused third shift is still
drawn. It consumes exactly `18` feature-stream draws, allows overlap, ignores
write results and returns true. Search admission and exact traversal remain with
`WGEN-PIPELINE-001`.

Every admitted Monster Room rebuilds each supported, solid, non-chest floor cell
after consuming `nextInt(4)`: nonzero (`3/4`) requests Mossy Cobblestone and zero
requests Cobblestone. The floor draw occurs before protected safe-write
admission; walls always request Cobblestone. Room geometry, descending traversal,
openings, cave-air writes, chests and spawner initialization remain with the
feature owner. The empty configured record has count-`10` ordinary and count-`4`
deep placed wrappers.

### Procedural Jungle Temple masonry

An admitted Jungle Temple makes exactly `1,522` selector offers in fixed
Y/X/Z traversal. Every offer consumes one caller-stream float before the
processing-box test; `<0.4` selects Cobblestone and every other float selects
Mossy Cobblestone, so the Mossy endpoint has nominal probability `0.6`.

Twelve additional coordinates request Mossy Cobblestone without RNG: the two
trap-wire supports, nine corridor cells and one hidden-mechanism support. Piece
rotation, processing-box clipping, pre-existing target state and ignored flags-2
writes mean neither `1,522` nor the 12 fixed offers is a guaranteed final-world
count. The complete temple transaction remains with
`WGEN-STRUCTURE-JUNGLE-TEMPLE-001`.

### Village mossification and zombie degradation

Each of the five locked lists is one `rule` processor with always-true live
location predicates. It creates a position-seeded RNG per processed world
position, tests rules in data order and returns the first match:

- `mossify_10_percent`, `_20_percent` and `_70_percent` change exact
  Cobblestone to Mossy Cobblestone on strict `<0.1`, `<0.2` and `<0.7`;
- `zombie_plains` first changes Cobblestone to Mossy on `<0.8`. On first-gate
  failure, a later Cobblestone rule can change it to Cobweb on a second `<0.07`
  draw. Raw Mossy skips the first five rules and independently changes to Cobweb
  on `<0.07`;
- `zombie_taiga` first changes Cobblestone to Mossy on `<0.8`. On failure, a
  later Cobblestone rule can change it to Cobweb on a second `<0.08` draw. No
  rule targets raw Mossy, so it passes unchanged.

The exhaustive pool join is:

| Processor | Pools | Entries | Raw Cobblestone candidates | Raw Mossy cells |
|---|---:|---:|---:|---:|
| mossify 10% | `3` | `53` | `3,447` | `29` |
| mossify 20% | `1` | `2` | `107` | `0` |
| mossify 70% | `2` | `2` | `45` | `1` |
| zombie Plains | `3` | `40` | `1,903` | `1` |
| zombie Taiga | `2` | `28` | `1,693` | `29` |

The `125` exact pool elements occur in seven Village pools. Candidate
counts are source cells per entry, not weight-expanded or expected final counts.
Pool selection, terrain matching, legacy air filtering, transforms, clip,
overlap, live-target admission and flags-`18` writes remain with the Village and
processor owners.

### Raw structure payload and processor outcomes

The exhaustive scan finds `369` raw Mossy Cobblestone cells in 20 of all `1,212`
templates:

| Owner/template | Raw cells |
|---|---:|
| `pillager_outpost/watchtower_overgrown` | `207` |
| `trial_chambers/corridor/atrium/bogged_relief` | `14` |
| `trial_chambers/spawner/melee/zombie` | `4` |
| `trial_chambers/spawner/small_melee/baby_zombie` | `2` |
| `underwater_ruin/brick_1` | `17` |
| `underwater_ruin/brick_2` | `8` |
| `underwater_ruin/brick_6` | `3` |
| `underwater_ruin/brick_7` | `6` |
| `underwater_ruin/cracked_1` | `17` |
| `underwater_ruin/cracked_2` | `6` |
| `underwater_ruin/cracked_7` | `6` |
| `underwater_ruin/mossy_1` | `19` |
| `underwater_ruin/mossy_2` | `10` |
| `underwater_ruin/mossy_6` | `3` |
| `underwater_ruin/mossy_7` | `6` |
| `village/plains/houses/plains_meeting_point_4` | `1` |
| `village/plains/zombie/houses/plains_meeting_point_4` | `1` |
| `village/taiga/houses/taiga_small_farm_1` | `19` |
| `village/taiga/town_centers/taiga_meeting_point_2` | `10` |
| `village/taiga/zombie/town_centers/taiga_meeting_point_2` | `10` |

The owner totals are Outpost `207`, Trial Chambers `20`, cold small Ocean Ruins
`101` and Villages `41`.

The Outpost tower list first places `watchtower`, then overlays
`watchtower_overgrown`; its unfiltered `outpost_rot` retains every second-child
cell only on `nextFloat() <= 0.05`, before legacy air filtering. The three Trial
elements are reachable through the Atrium pool and structure-wide melee/
small-melee aliases, use empty processor lists and otherwise retain their raw
Mossy states through generic placement.

Cold Ocean Ruins choose one small suffix and place Brick/Cracked/Mossy overlays
at integrities `0.8/0.7/0.5`; only suffixes `1,2,6,7` contain this identity.
Position-stable block rot can remove every listed source cell before the
archaeology processor and chunk-clipped write. No large cold or warm ruin stores
Mossy Cobblestone.

The five Village files follow the processor rows above: ordinary Mossify lists
leave raw Mossy unchanged, Zombie Taiga leaves it unchanged, and the sole Zombie
Plains cell has the strict `.07` Cobweb branch. Raw source, processor output and
final-world writes remain distinct at every structure boundary.

The NBT UTF census finds exactly the same 20 strings: each is one palette entry.
There is no Jigsaw final-state, item stack or block-entity occurrence.

### Ruined-portal negative replacement

`blackstone_replace` deterministically maps Mossy Cobblestone to default
Blackstone, retaining position and NBT. It runs last only for Nether Ruined
Portals. None of the 13 locked portal templates has a raw Mossy-Cobblestone cell
and the preceding aging processor does not create this exact full block, so the
mapping has zero reachable inputs in the locked vanilla portal corpus. It remains
an exact code-built consumer for custom/reloaded template input.

Complete exact-ID and tag-consumer sweeps find no chest/entity loot, trade,
archaeology, crop/plant, surface/noise or other acquisition path beyond those
listed.

**Client projection:**

The property-free blockstate unconditionally selects
`block/mossy_cobblestone`. That model inherits `block/cube_all` and maps every
face to `block/mossy_cobblestone`; the item directly selects the same model.
There is no tint or randomized variant.

The English name is `Mossy Cobblestone`. Building Blocks publishes it once after
Cobblestone Wall and before its Stair, Slab and Wall descendants, followed by
Smooth Stone. It appears in no other ordinary tab. Updates use state `3368`,
inventory paths use item ID `348`, maps use `STONE`, note blocks read
`BASEDRUM`, and sounds use `1596/1604/1601/1600/1599`. No subtype packet or
connection-local state is added.

**Branches and aborts:**

Sole placement/save state; correct/wrong Pickaxe and ordinary/explosion loot;
two producers, six shape derivatives and Wild-template sink with independent
unlocks; live archetype/swallowability; feature support and every blob extent/
shift/write; Monster-Room floor draw/protection; Jungle-Temple selector/fixed
offers and clipping; all 125 Village entry/rule outcomes; every raw-template
integrity/processor/transform/clip/write path; dormant Nether replacement;
persistence and client projection are distinct.

**Constants and randomness:**

State/block/item IDs `3368/192/348`; strength `2/6`; emission `0`; dampening
`15`; shade `0.2`; friction `0.6`; speed/jump `1`; sound IDs
`1596/1604/1601/1600/1599` at `1/1`; stack `64`; fire odds/fuel
`0/0/0`; recipes producers/consumers `2/7`; derivative ratios
`3:6/6:4/6:6` and cutting `1:2/1:1/1:1`; forest-rock count/draws
`2/18`; Monster-Room Mossy chance `3/4`; Jungle selector offers/Mossy
threshold/fixed `1,522/0.6/12`; pool entries/candidates as tabled;
templates/files/cells `1,212/20/369`.

**Side effects:**

Block placement/removal and self loot; crafting/cutting outputs and recipe
knowledge; Wild-template copies; reload-selected Sulfur-Cube equipment and
swallowing; feature, dungeon, temple, Village, Outpost, Trial and Ocean-Ruin
writes; optional Cobweb/Blackstone replacement; ordinary persistence, maps,
notes, sounds, model, name and tab projection.

**Gates:**

World-write/break authority; correct Pickaxe and explosion survival; active
loot/recipe/advancement/tag/archetype snapshots; crafting/Stonecutter output;
Sulfur-Cube/front-AABB acceptance; feature support/biome/write; procedural
structure validation and processing box; pool selection, aliases, processor RNG,
transform, integrity, clip, live target and write admission; valid registry, map,
sound and client-resource context.

**Boundary cases and quirks:**

- Every Pickaxe is correct without a tier gate; wrong tools still remove but
  fail loot admission.
- Moss Block or Vine unlocks the corresponding producer; holding the output
  unlocks six descendants but not Wild-template duplication.
- A Zombie-Plains Cobblestone that becomes Mossy stops at the first rule and
  cannot become Cobweb in the same processor invocation; a raw Mossy cell can.
- Outpost block-rot equality retains, whereas random rule equality rejects.
- Monster-Room protected floor cells still consume their Mossy draw.
- Jungle Temple offers 60%-Mossy selector output plus 12 fixed Mossy states;
  processing-box clipping prevents a fixed final count.
- The raw `369` census excludes processor-created and procedural states and is
  not a final-world expectation.
- Nether Ruined Portals know the replacement mapping but provide no locked
  source cell that can reach it.

**Failure semantics:**

Failed placement/removal commits only generic earlier work. Wrong-tool or failed
explosion loot emits nothing. Invalid/output-blocked recipes consume nothing.
Feature and structure owners retain their stated nonrollback behavior; their
ignored block-write results do not retract earlier RNG or side effects. A failed
Sulfur-Cube search falls through to default dispenser ejection.

**Client/server authority split:**

The server owns state, placement, loot, recipes/knowledge, tags, Sulfur-Cube
interactions, generation, structures and persistence. Clients project state/item
IDs, map color, note/sounds, name, cube model and Building Blocks order.

**Observability:**

Commands/state packets, shape/light/signal probes, mining/drops, crafting and
Stonecutting screens, recipe book, Sulfur-Cube/dispenser state, controlled
feature/structure traces, template decodes, maps, notes/sounds, tabs and rendering
expose every listed branch.

**Persistence and reload:**

Placed blocks persist only identity; stacks persist ordinary components. Loot,
recipes, advancements, tags, archetypes, features, processors, pools and
templates are reload-selected. Registration, Monster-Room/Jungle-Temple/
Blackstone-replacement control flow and creative order remain code-built. Reload
does not retroactively convert placed states.

**Evidence:**

`Confirmed`; `OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-DATA-001`;
`OFF-REPORT-001`. Anchors:
`net.minecraft.world.level.block.Blocks`;
`net.minecraft.world.level.block.FireBlock#bootStrap`;
`net.minecraft.world.level.block.entity.FuelValues#vanillaBurnTimes`;
`net.minecraft.world.entity.monster.cubemob.SulfurCube#matchingArchetypes`;
`net.minecraft.core.dispenser.DispenseItemBehavior#bootStrap`;
`net.minecraft.world.level.levelgen.feature.BlockBlobFeature#place`;
`net.minecraft.world.level.levelgen.feature.MonsterRoomFeature#place`;
`net.minecraft.world.level.levelgen.structure.structures.JungleTemplePiece#postProcess`;
`net.minecraft.world.level.levelgen.structure.structures.JungleTemplePiece$MossStoneSelector#next`;
`net.minecraft.world.level.levelgen.structure.templatesystem.RuleProcessor#processBlock`;
`net.minecraft.world.level.levelgen.structure.templatesystem.BlackstoneReplaceProcessor#processBlock`;
`net.minecraft.world.item.CreativeModeTabs#bootstrap`;
both reports and item components; self loot; nine recipes and nine recipe
advancements; direct/composed tags and slow-bouncy record; forest-rock, five
processor lists, seven Village pools, all owner pools/structures and all 1,212
templates; exact blockstate/model/item/name resources. Complete compiled exact-
field and data-reference searches found no other runtime path.

**Test vectors:**

Run `EXP-BLK-106` across state/IDs; every tool/explosion, recipe/unlock,
archetype/dispenser and negative fire/fuel path; both forest-rock biomes and all
blob shapes; every Monster-Room floor and Jungle-Temple offer; all 125 Village
entries and ordered processor draws; all 1,212 templates through Outpost, Trial,
Ocean-Ruin, Village and Ruined-Portal boundaries; persistence, maps, notes,
sounds, name, model and tab order. Assert exact constants, `20/369` raw census,
processor candidate table and vanilla convergence.

**Limits:**

Generic placement, breaking, loot/explosion, crafting, Stonecutting,
advancements, Sulfur-Cube/dispenser behavior, feature algorithms, procedural and
template structure placement, packet encoding and rendering remain with their
named owners. Shape-family stairs/slabs/walls remain under their existing leaf.
This leaf fixes the full-block identity, exact acquisition/consumption joins,
generation census, absences and projection.
