# Blocks mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `BLK-TUFF-001` — Tuff masonry joins deep ore, sulfur springs and Trial Chambers

**Parent:** `SIM-004`, `SIM-005`, `BLK-001`, `BLK-STATE-001`, `BLK-002`,
`BLK-PLACE-001`, `BLK-BREAK-001`, `BLK-BREAK-HOOK-001`, `PLY-005`, `PLY-006`,
`PLY-BREAK-001`, `BLK-003`, `BLK-005`, `BLK-UPDATE-001`, `ITM-003`, `ITM-004`,
`ITM-006`, `ITM-RECIPE-001`, `ITM-CRAFT-001`, `ITM-STONECUTTER-001`,
`ITM-LOOT-001`, `ITM-ADVANCEMENT-001`, `ENT-001`, `ENT-005`,
`ENT-KNOCKBACK-001`, `MOB-AI-001`, `MOB-SPAWN-001`, `WGEN-003`,
`WGEN-PIPELINE-001`, `WGEN-JIGSAW-PROCESSORS-001`,
`WGEN-JIGSAW-TRIAL-CHAMBERS-001`, `CLI-001`, `CLI-006`, `CLI-UI-001`,
`CLI-EFFECT-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked registrations and reports, exhaustive recipe, advancement, loot, tag,
worldgen, class-reference and client-resource searches, all 1,212 decoded structure templates,
the complete sulfur-spring input set and the complete reachable Trial Chambers graph exhaust the
five property-free full-block identities. Tuff, Polished-Tuff and Tuff-Brick stairs, slabs and
walls retain their separately audited shape-family implementations.

**Applies when:**

`minecraft:tuff`, `minecraft:polished_tuff`, `minecraft:tuff_bricks`,
`minecraft:chiseled_tuff` or `minecraft:chiseled_tuff_bricks` is placed, mined, exploded, crafted,
cut, selected by a reloadable tag, used as feature support, replacement terrain or an iron-vein
filler, written by a sulfur-spring or Trial Chambers template, persisted, mapped, sounded or
rendered.

**Authoritative state:**

All five are ordinary `Block` registrations with one property-free state and no block entity:

| Identity | State | Block protocol ID | Item raw ID | Map/instrument | Sound |
|---|---:|---:|---:|---|---|
| Tuff | `23452` | `984` | `12` | `TERRACOTTA_GRAY` / `BASEDRUM` | Tuff |
| Polished Tuff | `23863` | `988` | `17` | copied | Polished Tuff |
| Tuff Bricks | `24275` | `993` | `21` | copied | Tuff Bricks |
| Chiseled Tuff | `24274` | `992` | `16` | copied | Tuff |
| Chiseled Tuff Bricks | `24686` | `997` | `25` | copied | Tuff Bricks |

Tuff fixes `requiresCorrectToolForDrops`, strength `1.5/6`, Tuff sound and the map/instrument
values above. Polished copies Tuff and changes only its sound. Chiseled Tuff copies Tuff. Tuff
Bricks copies Tuff and changes only its sound; Chiseled Tuff Bricks copies Tuff Bricks.

Every state is a full unit selection/collision/visual/occlusion cube with emission `0`, light
dampening `15`, shade brightness `0.2`, friction `0.6`, speed/jump factors `1`, restitution `0`,
solid redstone conduction, normal piston reaction and full sturdy faces. None adds a random or
scheduled tick, use, attack, contact, neighbor, signal, comparator, fluid or block-event hook.

All five are directly `mineable/pickaxe` and belong to no minimum-tier tag, so every Pickaxe is
correct. Their sound types have volume/pitch `1/1` and break/step/place/hit/fall IDs:

| Profile | Break | Step | Place | Hit | Fall |
|---|---:|---:|---:|---:|---:|
| Tuff | `1641` | `1642` | `1643` | `1644` | `1645` |
| Tuff Bricks | `1646` | `1650` | `1649` | `1648` | `1647` |
| Polished Tuff | `1651` | `1655` | `1654` | `1653` | `1652` |

Their ordinary block items are common 64-stacks with the matching translation/model key and all
five directly select `sulfur_cube_archetype/slow_bouncy`.

**Transition and ordering:**

### Placement, harvest and loot

Placement, explicit writes, rotation and mirror retain the sole selected state. A legal
`minecraft:block_state` component cannot add a property.

Each block has one one-roll self-item entry behind `survives_explosion`, with random sequence
`minecraft:blocks/<identity>`. Correct-tool admission occurs before the loot table; a wrong-tool
player removal emits nothing. Silk Touch and Fortune do not change any table, and there is no
alternate drop, count function, XP or block-specific break hook.

### Exact masonry recipes

Exactly 38 bundled recipe records name a family identity: 13 shaped crafting records and 25
Stonecutting records.

- Four Tuff in a `2x2` square produce four Polished Tuff; four Polished Tuff likewise produce four
  Tuff Bricks.
- Two vertically stacked Tuff Slabs produce one Chiseled Tuff; two vertically stacked Tuff-Brick
  Slabs produce one Chiseled Tuff Bricks.
- Tuff, Polished Tuff and Tuff Bricks each use the conventional three-in-a-row slab recipe to six,
  six-input stair recipe to four and six-input wall recipe to six.
- Tuff Stonecuts to 13 outputs: its three shapes; Polished Tuff plus its three shapes; Tuff Bricks
  plus their three shapes; and both Chiseled blocks.
- Polished Tuff Stonecuts to eight outputs: its three shapes; Tuff Bricks plus their three shapes;
  and Chiseled Tuff Bricks.
- Tuff Bricks Stonecuts to its three shapes and Chiseled Tuff Bricks.

Every Stonecutter result is one except slabs, which return two. Each recipe has one paired
advancement with a two-criterion OR requirement: prior recipe knowledge or possession of its exact
source. The two Chiseled crafting records instead use possession of the matching slab. Result
stacks are default stacks, so arbitrary input components are discarded. No cooking, shapeless,
smithing or special-crafting record names a family identity.

### Reloadable terrain, support and ore selectors

Tuff directly belongs to `base_stone_overworld` and `deepslate_ore_replaceables`. The first
membership composes into `azalea_root_replaceable`, `bats_spawnable_on`,
`dripstone_replaceable_blocks`, `forest_rock_can_place_on`, `moss_replaceable`,
`nether_carver_replaceables`, `overworld_carver_replaceables` and `sculk_replaceable`.
`moss_replaceable` further composes into `lush_ground_replaceable`, while `sculk_replaceable`
further composes into `sculk_replaceable_world_gen`. Consequently root systems, Bat support,
dripstone, forest-rock support, moss/lush patches, both configured carvers and ordinary/worldgen
Sculk consumers see Tuff through their live snapshots. This is tag composition, not copied
hard-coded behavior.

Seven size-`64/33` zero-air-discard ore configurations target `base_stone_overworld`, so an
existing Tuff cell can be replaced by Andesite, Diorite, Granite, Clay, Dirt, Gravel or Tuff.
Seventeen ordered two-target ore configurations target its direct deepslate-replaceable membership:
Coal and buried Coal; large/small Copper; four Diamond forms; Emerald; Gold and buried Gold;
Infested; Iron and small Iron; Lapis and buried Lapis; and Redstone. Their second targets produce
the corresponding Deepslate ore, `infested_deepslate[axis=y]`, or unlit Deepslate Redstone Ore.
Sizes, discard chances, placement counts/heights/biome lists, geometry and write order remain with
`WGEN-PIPELINE-001`.

The dedicated `ore_tuff` record has size `64`, air-exposure discard `0` and maps
`base_stone_overworld` to Tuff. Its placed wrapper performs count `2`, in-square, uniform
above-bottom `0` through absolute Y `0`, then biome filtering. It is scheduled by all 55 locked
Overworld biomes, including the cave biomes. Tuff may therefore both replace another base stone
and later be replaced by another base-stone or deepslate-target ore.

Overworld-family noise settings also use Tuff as the iron-vein filler. Within the iron band
`-60..-8`, after toggle, solidity and ridged admission, failure of the richness or gap path returns
Tuff rather than null; successful paths instead choose Iron Ore or Raw Iron Block. Exact
position-seeded draws and float thresholds remain with the ore-vein resolver.

Tuff is named directly as support by `glow_lichen` and `sculk_vein`, and as a valid block by
`spring_water` and `spring_lava_overworld`. Thus it can support the corresponding multiface block
or satisfy the spring's above/below/origin and exact rock/hole census. Polished and decorated
variants do not enter any of these selectors. The configured parameters and wrappers are the exact
records already locked by `WGEN-PIPELINE-001`: Glow Lichen count `104..157`, Sculk Vein count
`204..250`, Water Spring count `25`, and Overworld Lava Spring count `20`.

### Sulfur-spring terrain and templates

Sulfur Caves schedule `rooted_sulfur_spring` with uniform count `1..2`, in-square, uniform
above-bottom `0` through absolute `256`, an upward air-to-solid scan of at most `12`, fixed Y
offset `-1`, then biome filtering. Its child root-system eventually invokes the inline
`sulfur_spring` selector.

That selector chooses small/medium/large/extra-large sequence branches with weights
`200/90/20/5`. The first child offers Tuff `64/80/96/128` times using X/Z spreads
`[-7,7]/[-8,8]/[-9,9]/[-10,10]`, Y spread `[-3,3]`, downward solid scan at most four and a solid
filter. The second child applies fixed Y `-7`, selects an equal-weight template and one of four
rotations, then places with no processors. Generic selector, sequence, simple-block, root-system,
placement and template order remain with `WGEN-PIPELINE-001`.

The ten exact spring templates contain `2,177` raw Tuff cells:

| Template suffix | Tuff cells |
|---|---:|
| `small_1`, `small_2`, `small_3`, `small_4` | `133`, `128`, `126`, `145` |
| `medium_1`, `medium_2`, `medium_3` | `186`, `282`, `265` |
| `large_1`, `large_2` | `266`, `308` |
| `extra_large_1` | `338` |

These raw cells are separate from the first child's additional Tuff offers. Rotation, clipping,
live-target admission and write results mean neither number is a guaranteed final-world count.

### Trial Chambers payload

The exhaustive all-template scan finds no Tuff outside the ten sulfur-spring inputs and no
decorated family cell outside Trial Chambers. The 191 distinct Trial Chambers templates are all
reachable through the 47-pool graph and contain:

| Identity | Templates with raw cells | Raw cells | Connector final states |
|---|---:|---:|---:|
| Polished Tuff | `110` | `13,655` | `11` |
| Tuff Bricks | `121` | `106,683` | `170` |
| Chiseled Tuff | `30` | `203` | `0` |
| Chiseled Tuff Bricks | `88` | `7,131` | `1` |

The Trial subtotal is `127,672` raw cells; adding sulfur-spring Tuff gives `129,849` raw family
cells across the complete 1,212-template corpus. Connector final states replace jigsaws and are not
raw palette cells.

Of the 207 Trial ordinary-single entries, 50 references use the named copper-bulb-degradation list
and 157 use inline empty processors. Neither mode rewrites a family state. The named list can still
suppress its write when the current live block is in `features_cannot_replace`; the inline mode
has no such protection. Selection, aliasing, repetition, connector replacement, template
transform, destructive air, chunk clipping, live-target checks and flags-`18` writes remain with
`WGEN-JIGSAW-TRIAL-CHAMBERS-001`. The raw census is therefore not a promised generated count.

### Trial loot, advancement icon and equipment

Two Trial Chambers chest tables can acquire Tuff. Each selected `corridor` pool roll chooses Tuff
at weight `3/19`, then emits uniform integer `8..20`; the pool rolls uniformly `1..3` times. Each
selected `supply` pool roll chooses Tuff at weight `1/18`, then emits uniform integer `5..10`; that
pool rolls uniformly `3..5` times. Trial templates bind corridor loot to four barrels and supply
loot to one chest. Container seed assignment, table expansion, repeated selection, slot filling and
overflow remain with the Trial/container/loot owners.

Chiseled Tuff is the icon for `adventure/minecraft_trials_edition`. Its location criterion requires
the player to be inside the `trial_chambers` structure and does not inspect the held block. No
trade, barter, other non-block loot table, other non-recipe advancement or hard-coded mob path
names a family item.

All five items select slow-bouncy. Its record fixes horizontal/vertical knockback `0.4125/0.24`,
push cooldown `0.5`, impulse threshold `0.05`, additive knockback and explosion-knockback
resistance `0.4000000059604645/0.4000000059604645`, additive bounciness
`0.6000000238418579`, total-multiplied friction `-0.699999988079071` and air drag
`-0.9499999992549419`, plus its hit/push sounds. Matching and modifier lifecycle remain with the
Sulfur-Cube owners.

**Client projection:**

Each property-free blockstate has one unconditional model and every item directly selects it.
Tuff, Polished Tuff and Tuff Bricks inherit `cube_all` with their matching texture. Both Chiseled
models inherit `cube_column`, using their `_top` texture on the two ends and the base matching
texture on the four sides.

English names are exactly the five display names in the state table. Natural Blocks publishes Tuff
once between Calcite and Dripstone Block. Building Blocks publishes the complete family after
Reinforced Deepslate in this order: Tuff, its stairs/slab/wall, Chiseled Tuff, Polished Tuff, its
stairs/slab/wall, Tuff Bricks, their stairs/slab/wall, Chiseled Tuff Bricks, then Bricks.

State updates use the five state IDs, inventory paths use the five item IDs, maps use
`TERRACOTTA_GRAY`, and sounds use the three profiles above. No identity adds a packet field or
connection-local state.

**Branches and aborts:**

Five sole states; correct/wrong tool and explosion survival; 13 crafting and 25 cutting records
with independent unlocks; two Trial chest rows and one location icon; direct and composed tag
snapshots; seven base-stone and seventeen deepslate-target ore branches; Tuff output, support,
spring and iron-vein-filler paths; four sulfur-spring size branches and ten template choices;
every Trial pool, processor, connector, transform, clip, protection and write outcome;
slow-bouncy selection; persistence and five exact client projections are distinct.

**Constants and randomness:**

States, block/item IDs and sound IDs as tabulated; strength `1.5/6`; stack `64`; recipes
`13/25/38`; Trial loot `3/19 x 1..3 x 8..20` and `1/18 x 3..5 x 5..10`;
base/deepslate ore configurations `7/17`; ore-Tuff size/count/height
`64/2/above-bottom 0..absolute 0`; iron band `-60..-8`; sulfur outer weights `200/90/20/5`,
simple counts `64/80/96/128`, ten templates and `2,177` raw Tuff cells; Trial templates `191`,
raw Trial family cells `127,672`; total raw family cells `129,849`. The blocks consume no RNG
directly; generic loot, recipe, archetype, feature and structure owners retain their streams.

**Side effects:**

Full-block placement/removal and conditional self loot; crafting, cutting and recipe knowledge;
Trial chest output and location advancement/icon;
tag-selected support, spawn, carving, moss/root/dripstone/Sculk and ore replacement; iron-vein,
ore-Tuff and sulfur-spring terrain writes; sulfur-spring and Trial template/connector writes;
slow-bouncy equipment modifiers; ordinary persistence, map color, sounds and opaque block/item
projection.

**Gates:**

Write and break authority; correct Pickaxe and explosion context; active recipe, advancement, loot,
tag and archetype snapshots; ore target/exposure/biome/write admission; spring/root/support and
iron-vein thresholds; selector/template availability; Trial graph, alias, connector, processor,
transform, clip, protected-target and write admission; valid registry/map/sound/client-resource
context.

**Boundary cases and quirks:**

- Tuff is simultaneously a base-stone target result and a member of that target tag; a same-state
  ore write is still owned by generic ore success semantics.
- Its deepslate-replaceable membership causes ordinary Tuff—not only Deepslate—to produce
  Deepslate ore variants and axis-Y Infested Deepslate.
- `nether_carver_replaceables` composes the Overworld base-stone tag, so a reload/custom world may
  expose Tuff to the Nether carver even though locked Nether terrain does not normally create it.
- The sulfur template total excludes the simple-block Tuff offers that precede template placement.
- The Trial connector counts are replacements of jigsaw cells, not additions to the raw census.
- Named Trial processing does not transform Tuff masonry, but its protected-live-state gate can
  suppress a write; inline empty processing cannot.
- Chiseled Tuff and Chiseled Tuff Bricks are property-free despite their column-textured models.
- Shape descendants are recipe results and structure neighbors here, not members of this five-ID
  catalog family.

**Failure semantics:**

Illegal state patches are rejected by the shared component/state owner. Wrong tools fail the
correct-tool loot gate; explosion failure emits nothing. Recipe mismatch, insufficient capacity
or inactive snapshot aborts through the owning recipe path. Feature target/support/height/biome,
scan, clip, protection or write failure preserves the live state according to the owning kernel.
Missing template or registry resources fail at those owners' documented lookup boundaries. Client
resource failure affects projection, not authoritative identity.

**Client/server authority split:**

The server owns registry identity, state, harvest, loot, recipes, reloadable tags, feature and
structure writes, archetype selection and persistence. The client owns models, textures, English
projection, tab presentation and playback/rendering of authoritative state and sound events.

**Observability:**

Commands, debug reports, inventory/loot/recipe state, ore and feature traces, decoded NBT/pool data,
world blocks, equipment modifiers, packets, sounds, maps and rendering expose the listed branches.

**Persistence and reload:**

Placed states persist only the block ID because there is no property or block entity. Item stacks
persist ordinary components. Recipe, advancement, loot, tag, archetype, biome-feature, configured
feature, placed feature, pool and processor snapshots are reloadable where their owners specify;
existing block states do not retroactively change when those snapshots reload.

**Evidence:**

`Confirmed`; `OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-DATA-001`; `OFF-REPORT-001`. Anchors:
`net.minecraft.world.level.block.Blocks`, `net.minecraft.world.level.block.SoundType`,
`net.minecraft.world.level.levelgen.OreVeinifier$VeinType`,
`net.minecraft.world.level.levelgen.feature.OreFeature#place`,
`net.minecraft.world.level.levelgen.feature.MultifaceGrowthFeature#place`,
`net.minecraft.world.level.levelgen.feature.SpringFeature#place`,
`net.minecraft.world.level.levelgen.feature.TemplateFeature#place`,
`net.minecraft.world.level.levelgen.structure.templatesystem.StructureTemplate#placeInWorld`,
`net.minecraft.world.entity.monster.cubemob.SulfurCube#matchingArchetypes` and
`net.minecraft.world.item.CreativeModeTabs`; the five block/item/component/loot/asset reports; all
38 recipe/advancement pairs, both Trial chest tables and the Trial location advancement; every
direct/composed tag and worldgen record; all 1,212 NBT
templates; all 47 Trial pools and the degradation list. Complete exact-ID searches found no other
loot, trade, advancement, data or runtime path.

**Test vectors:**

Run `EXP-BLK-096` across all five states and IDs; physical/tool/explosion/loot behavior; every
crafting, Stonecutting and unlock record; both Trial chest rows and the location icon; all
direct/composed tags, slow-bouncy and ore/support/spring/iron-vein paths; every sulfur
branch/template; all 1,212 templates and reachable Trial
pool/processor/connector/rotation/clip/write outcomes; persistence, sounds, maps, tabs and
models. Assert the exact constants, record graph, raw counts, absence boundaries and client
convergence.

**Limits:**

Generic placement, breaking, loot, crafting, Stonecutting, advancements, Sulfur-Cube behavior,
terrain/features, template processing, packet encoding and rendering remain with their named
generic owners. Shape-family stairs, slabs and walls remain under their existing leaves. This leaf
fixes the exact five full-block identities, joins, absences and projection.
