# Blocks mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `BLK-PRISMARINE-001` — Prismarine masonry joins monuments, ocean ruins and Conduit frames

**Parent:** `SIM-004`, `SIM-005`, `BLK-001`, `BLK-STATE-001`, `BLK-002`,
`BLK-PLACE-001`, `BLK-BREAK-001`, `BLK-BREAK-HOOK-001`, `PLY-005`, `PLY-006`,
`PLY-BREAK-001`, `BLK-003`, `BLK-005`, `BLK-UPDATE-001`, `BLK-CONDUIT-001`,
`ITM-003`, `ITM-004`, `ITM-006`, `ITM-RECIPE-001`, `ITM-CRAFT-001`,
`ITM-STONECUTTER-001`, `ITM-LOOT-001`, `ITM-ADVANCEMENT-001`,
`ITM-PRISMARINE-MATERIAL-001`, `ITM-SMITHING-TEMPLATE-001`, `ENT-001`,
`ENT-005`, `ENT-KNOCKBACK-001`, `MOB-AI-001`, `WGEN-003`,
`WGEN-STRUCTURE-OCEAN-MONUMENT-001`, `WGEN-STRUCTURE-OCEAN-RUIN-001`,
`CLI-001`, `CLI-006`, `CLI-UI-001`, `CLI-EFFECT-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked registrations and reports, exhaustive recipe, advancement, loot, tag,
class-reference and client-resource searches, and all 1,212 decoded structure templates exhaust
the three property-free full-block identities. Prismarine, Prismarine-Brick and Dark-Prismarine
stairs, slabs and the Prismarine Wall retain their separately audited shape-family
implementations.

**Applies when:**

`minecraft:prismarine`, `minecraft:prismarine_bricks` or `minecraft:dark_prismarine` is placed,
mined, exploded, crafted, cut, used to duplicate a Tide Armor Trim Smithing Template, tested as a
Conduit frame, written by an Ocean Monument or Ocean Ruin, persisted, mapped, sounded or rendered.

**Authoritative state:**

All three are ordinary `Block` registrations with one property-free state and no block entity:

| Identity | State | Block protocol ID | Item raw ID | Map/instrument |
|---|---:|---:|---:|---|
| Prismarine | `12631` | `527` | `590` | `COLOR_CYAN` / `BASEDRUM` |
| Prismarine Bricks | `12632` | `528` | `591` | `DIAMOND` / `BASEDRUM` |
| Dark Prismarine | `12633` | `529` | `592` | `DIAMOND` / `BASEDRUM` |

Each registration fixes `requiresCorrectToolForDrops`, hardness/resistance `1.5/6` and otherwise
retains the ordinary Block defaults. Every state is a full unit
selection/collision/visual/occlusion cube with emission `0`, light dampening `15`, shade
brightness `0.2`, friction `0.6`, speed/jump factors `1`, restitution `0`, solid redstone
conduction, normal piston reaction and full sturdy faces. None adds a random or scheduled tick,
use, attack, contact, neighbor, signal, comparator, fluid or block-event hook.

All three are directly `mineable/pickaxe` and belong to no minimum-tier tag, so every Pickaxe is
correct. They retain Stone sound volume/pitch `1/1` and break/step/place/hit/fall IDs
`1596/1604/1601/1600/1599`.

Their ordinary block items are common 64-stacks with the matching translation/model key. All three
directly select `sulfur_cube_archetype/slow_bouncy`.

**Transition and ordering:**

### Placement, harvest and loot

Placement, explicit writes, rotation and mirror retain the sole selected state. A legal
`minecraft:block_state` component cannot add a property.

Each block has one one-roll self-item entry behind `survives_explosion`, with random sequence
`minecraft:blocks/<identity>`. Correct-tool admission occurs before the loot table; a wrong-tool
player removal emits nothing. Silk Touch and Fortune do not change any table, and there is no
alternate drop, count function, XP or block-specific break hook.

### Exact processing and progression

Exactly 18 bundled recipe records name an exact family identity: ten shaped crafting records, one
shapeless crafting record and seven Stonecutting records.

- Four Prismarine Shards in a `2x2` square produce one Prismarine. Nine separate Shard ingredients
  shapelessly produce one Prismarine Bricks. Eight Shards around one Black Dye produce one Dark
  Prismarine.
- Prismarine uses the conventional slab/stairs/wall recipes, returning `6/4/6`; Prismarine Bricks
  and Dark Prismarine each use the slab/stairs recipes, returning `6/4`.
- Each base block Stonecuts only to its own shape descendants. Prismarine cuts to slab, stairs or
  wall; Prismarine Bricks to brick slab or stairs; Dark Prismarine to dark slab or stairs. Slabs
  return two and every other cut returns one.
- Prismarine is the core in the Tide Armor Trim Smithing Template duplication layout `#S#/#C#/###`:
  seven Diamonds, one Tide template and one Prismarine produce two default Tide templates.

The three material recipes unlock from possession of a Prismarine Shard or prior knowledge of the
exact recipe. Each of the fourteen shape crafting/cutting records unlocks from possession of its
exact base block or prior recipe knowledge. Tide duplication instead unlocks from possession of
the Tide template or prior knowledge. Every recipe has its paired advancement and each OR
requirement rewards only that recipe.

Result stacks are default stacks, so arbitrary input components are discarded, including the
source Tide template's patch. No cooking, smithing-transform, smithing-trim, special-crafting or
other shapeless record names a family identity.

### Conduit frame admission

The Conduit refresh scans its exact 42 candidate positions and admits a position when its block is
Prismarine, Prismarine Bricks, Sea Lantern or Dark Prismarine. Thus all three identities count
equally toward activation at 16 positions, radius tiers, hunting at all 42 positions and the
client's ordered particle-source list. Their shapes, item state and provenance are irrelevant.

Water-volume admission, the 40-tick refresh, effect radius, target state machine, sounds,
particles, persistence and update tags remain with `BLK-CONDUIT-001`. Reloadable block tags do not
control this four-block test.

### Procedural Ocean Monuments

Ocean Monument code fixes Prismarine, Prismarine Bricks and Dark Prismarine as its `G/L/D`
construction states. Prismarine supplies the bulk floors, walls, shelves and bands; Prismarine
Bricks supplies the ribs, frames, supports, roof courses, posts and other light masonry; Dark
Prismarine supplies the entrance accents, sealed-core shell, room dividers, perimeter accents and
wing/penthouse details.

The exact shell cuboids, room-graph pieces, overwrite order, water replacement, fill-only
admission, chunk clipping and direct foundation columns remain with
`WGEN-STRUCTURE-OCEAN-MONUMENT-001`. Because the generated room graph, piece designs, processing
chunks, live water/ice and overlapping writes vary, these procedural offers have no single raw or
promised final block count.

### Ocean-Ruin template payload

The exhaustive 1,212-template scan finds Prismarine in exactly six cold large-ocean-ruin inputs
and finds no raw Prismarine Bricks or Dark Prismarine anywhere:

| Template | Raw Prismarine cells |
|---|---:|
| `underwater_ruin/big_brick_1` | `4` |
| `underwater_ruin/big_brick_2` | `6` |
| `underwater_ruin/big_cracked_1` | `5` |
| `underwater_ruin/big_cracked_2` | `5` |
| `underwater_ruin/big_mossy_1` | `4` |
| `underwater_ruin/big_mossy_2` | `7` |

The total is 31 raw Prismarine cells. A cold large ruin selects one suffix from `1,2,3,8` and
stacks its brick, cracked and mossy overlays; only suffixes `1` and `2` therefore include this
family, with raw triplet subtotals `13` and `18`. The words `brick`, `cracked` and `mossy` are
overlay/template names and do not denote Prismarine Bricks.

Each overlay independently applies its locked integrity (`.9/.7/.5` for large
brick/cracked/mossy), structure/air ignore, global archaeology cap, live-height restacking,
rotation, clip and write transaction. Later overlays can overwrite earlier live results. The raw
31-cell census is therefore neither an offered-write count nor a generated-world guarantee; exact
selection, marker, processor, order and placement semantics remain with
`WGEN-STRUCTURE-OCEAN-RUIN-001`.

### Equipment selection and absence boundary

All three items select slow-bouncy. Its record fixes horizontal/vertical knockback `0.4125/0.24`,
push cooldown `0.5`, impulse threshold `0.05`, additive knockback and explosion-knockback
resistance `0.4000000059604645/0.4000000059604645`, additive bounciness
`0.6000000238418579`, total-multiplied friction `-0.699999988079071` and air drag
`-0.9499999992549419`, plus its hit/push sounds. Matching and modifier lifecycle remain with the
Sulfur-Cube owners.

No non-block loot table, trade, barter, other advancement, configured/placed feature, processor
rule, template-pool connector or hard-coded mob path names a family item. Prismarine Shards and
Crystals remain separate plain-item identities under `ITM-PRISMARINE-MATERIAL-001`.

**Client projection:**

Each property-free blockstate has one unconditional model and every item directly selects it. All
three models inherit `cube_all` with their matching texture.

Prismarine's texture is `16x64`, giving four `16x16` frames. Its metadata uses frame time `300`,
interpolation enabled and this exact 22-entry sequence:
`0,1,0,2,0,3,0,1,2,1,3,1,0,2,1,2,3,2,0,3,1,3`. Prismarine Bricks and Dark Prismarine each use
one static `16x16` texture.

English names are exactly `Prismarine`, `Prismarine Bricks` and `Dark Prismarine`. Natural Blocks
publishes only Prismarine, between Pointed Dripstone and Cinnabar. Building Blocks publishes the
family after Sea Lantern in this order: Prismarine, its stairs/slab/wall, Prismarine Bricks, their
stairs/slab, Dark Prismarine, its stairs/slab, then Netherrack.

State updates use IDs `12631..12633`, inventory paths use item IDs `590..592`, maps use
`COLOR_CYAN` or `DIAMOND`, and sounds use the Stone profile above. No identity adds a packet field
or connection-local state.

**Branches and aborts:**

Three sole states; correct/wrong tool and explosion survival; ten shaped, one shapeless and seven
cutting records with independent unlocks; four Conduit-frame identities and every 16..42-frame
tier; every Ocean Monument graph/piece/clip/live/write path; large/small, cold/warm, suffix,
overlay/integrity/restack/processor/clip/write Ocean-Ruin outcomes; slow-bouncy selection;
persistence and three exact client projections are distinct.

**Constants and randomness:**

States/block/item IDs and map colors as tabulated; strength `1.5/6`; Stone sounds
`1596/1604/1601/1600/1599`; stack `64`; recipes `10/1/7/18`; Conduit candidates/activation/hunting
`42/16/42`; six Prismarine-bearing templates and `31` raw cells; texture frames/sequence/frame time
`4/22/300`. The blocks consume no RNG directly; generic loot, recipe, archetype and structure
owners retain their streams.

**Side effects:**

Full-block placement/removal and conditional self loot; crafting, cutting, template duplication
and recipe knowledge; Conduit activation/effect/hunting/particle admission; procedural Monument
and template-based Ocean-Ruin writes; slow-bouncy equipment modifiers; ordinary persistence, map
color, sounds and opaque block/item projection.

**Gates:**

Write and break authority; correct Pickaxe and explosion context; active recipe, advancement, loot
and archetype snapshots; Conduit water/frame/refresh thresholds; Monument structure graph,
intersection, live-fluid and write admission; Ocean-Ruin structure selection, integrity,
processor, restack, transform, clip and write admission; valid registry/map/sound/client-resource
context.

**Boundary cases and quirks:**

- Conduit admission is a hard-coded four-block identity test, not a live tag; all three scoped
  blocks count equally despite their different map colors and textures.
- The Ocean Monument is procedural and therefore absent from the NBT census even though it is the
  largest baseline source of all three states.
- The 31 raw Ocean-Ruin cells are ordinary Prismarine only; similarly named cold `brick` overlays
  do not create Prismarine Bricks.
- Ocean-Ruin integrity and later overlays can suppress or replace a raw Prismarine cell, while
  chunk/live-height reevaluation can change the offered transaction.
- Tide duplication consumes a placeable Prismarine block item as its core, not a Prismarine Shard
  or Prismarine Bricks.
- Prismarine is animated even though its sole authoritative block state never changes.
- Shape descendants are recipe results and tab neighbors here, not members of this three-ID
  catalog family.

**Failure semantics:**

Illegal state patches are rejected by the shared component/state owner. Wrong tools fail the
correct-tool loot gate; explosion failure emits nothing. Recipe mismatch, insufficient capacity
or inactive snapshot aborts through the owning recipe path. A nonfamily Conduit frame cell does
not enter the remembered frame list. Monument/Ocean-Ruin intersection, integrity, processor,
height, clip or write failure preserves the live state according to the owning kernel. Missing
structure or registry resources fail at those owners' documented boundaries. Client resource
failure affects projection, not authoritative identity.

**Client/server authority split:**

The server owns registry identity, state, harvest, loot, recipes, Conduit admission, structure
writes, archetype selection and persistence. The client owns texture animation, models, English
projection, tab presentation and playback/rendering of authoritative state and sound events.

**Observability:**

Commands, debug reports, inventory/loot/recipe state, Conduit effects/targets/particles, structure
traces, decoded NBT data, world blocks, equipment modifiers, packets, sounds, maps, tabs and
rendering expose the listed branches.

**Persistence and reload:**

Placed states persist only the block ID because there is no property or block entity. Item stacks
persist ordinary components. Recipe, advancement, loot and archetype snapshots are reloadable
where their owners specify; Ocean Monument construction and Conduit frame identities are
hard-coded, while Ocean-Ruin templates/processors are loaded through the owning structure system.
Existing block states do not retroactively change when any reloadable snapshot changes.

**Evidence:**

`Confirmed`; `OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-DATA-001`; `OFF-REPORT-001`. Anchors:
`net.minecraft.world.level.block.Blocks`,
`net.minecraft.world.level.block.entity.ConduitBlockEntity`,
`net.minecraft.world.level.levelgen.structure.structures.OceanMonumentPieces$OceanMonumentPiece`,
`net.minecraft.world.level.levelgen.structure.structures.OceanRuinPieces$OceanRuinPiece`,
`net.minecraft.world.item.CreativeModeTabs`; the three block/item/component/loot/asset reports;
all 18 recipe records and paired advancements; the direct block/item tags; all 1,212 NBT
templates; both structure owners and exact client resources. Complete exact-ID data and
class-reference searches found no other acquisition, advancement, trade, worldgen or runtime path.

**Test vectors:**

Run `EXP-BLK-097` across all three states and IDs; physical/tool/explosion/loot behavior; every
crafting, Stonecutting and unlock record including Tide duplication; all Conduit frame counts and
refresh boundaries; every Ocean Monument piece/design/clip/live/write path; all 1,212 templates
and every Ocean-Ruin selection/integrity/restack/processor/overlay/clip/write outcome;
slow-bouncy, persistence, sounds, maps, tabs, models and Prismarine animation. Assert the exact
constants, absence boundaries and client convergence.

**Limits:**

Generic placement, breaking, loot, crafting, Stonecutting, advancements, Sulfur-Cube behavior,
Conduit ticking, procedural Monument generation, Ocean-Ruin template processing, packet encoding
and rendering remain with their named owners. Shape-family stairs, slabs and wall remain under
their existing leaves. This leaf fixes the exact three full-block identities, joins, absences and
projection.
