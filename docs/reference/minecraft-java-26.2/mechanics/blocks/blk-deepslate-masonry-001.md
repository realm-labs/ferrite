# Blocks mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `BLK-DEEPSLATE-MASONRY-001` — Deepslate masonry joins crafting, sculk replacement and ancient-city degradation

**Parent:** `BLK-001`, `BLK-002`, `BLK-003`, `BLK-004`, `BLK-005`, `PLY-005`, `PLY-006`,
`ITM-004`, `ITM-006`, `ENT-001`, `ENV-003`, `WGEN-003`, `CLI-006`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — the locked registrations, reports, complete loot/recipe/advancement/tag data,
all 1,212 decoded structure templates, processor lists, pool references and exact client assets
exhaust the seven property-free full-block identities. Their stair, slab and wall state machines
remain with the already audited `shape-family`; base deepslate remains `BLK-DEEPSLATE-001`.

**Applies when:**

`cobbled_deepslate`, `polished_deepslate`, `deepslate_bricks`, `cracked_deepslate_bricks`,
`deepslate_tiles`, `cracked_deepslate_tiles` or `chiseled_deepslate` is placed, mined, exploded,
crafted, cooked, cut, selected through a reloadable tag, persisted, written by an ancient-city or
trial-chambers template, degraded by a structure processor, or projected to a vanilla client.

**Authoritative state:**

Each identity is an ordinary property-free `Block`, has no block entity, and has exactly one state:

| Identity | State | Block protocol ID | Item raw ID | Sound profile |
|---|---:|---:|---:|---|
| `cobbled_deepslate` | `30419` | `1152` | `9` | deepslate `506..510` |
| `polished_deepslate` | `30830` | `1156` | `10` | polished `1329..1333` |
| `deepslate_tiles` | `31241` | `1160` | `411` | tiles `511..515` |
| `deepslate_bricks` | `31652` | `1164` | `409` | bricks `501..505` |
| `chiseled_deepslate` | `32063` | `1168` | `413` | bricks `501..505` |
| `cracked_deepslate_bricks` | `32064` | `1169` | `410` | bricks `501..505` |
| `cracked_deepslate_tiles` | `32065` | `1170` | `412` | tiles `511..515` |

The sound ranges are ordered break, fall, hit, place, step and all four sound types use volume and
pitch multipliers `1/1`. Cobbled deepslate copies base deepslate's map color, `BASEDRUM`, correct-
tool gate and other properties, overrides strength to `3.5/6.0`, and retains the deepslate sound.
Polished, brick and tile blocks copy that result and replace only the named sound type; chiseled
copies cobbled and selects brick sound; cracked variants copy their uncracked counterpart exactly.

Every state is a full unit selection/collision/visual/occlusion cube with emission `0`, light
dampening `15`, shade brightness `0.2`, friction `0.6`, speed/jump factors `1`, restitution `0`,
solid redstone conduction, normal piston reaction and all faces sturdy. None adds random/scheduled
ticks, use, attack, entity-contact, neighbor, signal, comparator or block-event behavior. All seven
are directly `mineable/pickaxe`; no incorrect-tier tag names them, so every pickaxe tier is correct
and non-pickaxe tools are not. Their common block items stack to `64` and directly select
`sulfur_cube_archetype/slow_bouncy`. Cobbled alone is also in both three-member item tags
`stone_crafting_materials` and `stone_tool_materials`.

**Transition and ordering:**

#### Placement, breaking and acquisition

Ordinary placement, command/component writes and structure palettes always select the one listed
state. Rotations and mirrors cannot change a property-free state. Each one-roll loot table offers
one matching block item behind `survives_explosion` and uses the corresponding
`minecraft:blocks/<id>` random sequence. The generic correct-tool harvest gate runs before table
evaluation; Silk Touch and Fortune do not alter these self-drop tables.

Exactly 63 bundled recipes contain at least one of the seven identities as an exact ingredient or
result: 18 shaped recipes, 42 stonecutting recipes and three smelting recipes. Every one has a
paired advancement with exactly two criteria in one OR requirement and grants only its own recipe.
The exact graph is:

- four cobbled deepslate in a 2-by-2 yields four polished deepslate; four polished yields four
  bricks; four bricks yields four tiles; two vertically stacked cobbled slabs yield one chiseled;
- each of cobbled, polished, brick and tile blocks has the conventional three-input slab-to-six,
  six-input stair-to-four and six-input wall-to-six shaped record;
- seven diamonds, one matching template and one cobbled deepslate duplicate either a silence or
  ward armor-trim template to two;
- exact cobbled stonecutting has sixteen outputs: chiseled, the three cobbled shapes, polished plus
  its three shapes, bricks plus their three shapes, and tiles plus their three shapes; slabs return
  two and every other cut returns one;
- exact polished stonecutting has eleven outputs: its three shapes, bricks plus three shapes, and
  tiles plus three shapes; exact bricks have seven outputs: their three shapes and tiles plus three
  shapes; exact tiles have their three shapes; and exact base deepslate has five full-block outputs
  in this family, one each of cobbled, polished, bricks, tiles and chiseled; and
- cobbled smelts to base deepslate, bricks to cracked bricks and tiles to cracked tiles. All three
  award `0.1` experience, return one block and omit `cookingtime`, selecting the `200`-tick default.

The cobbled item tags add eight shaped matches outside that exact-ID census: three cobbled material
slots can make a brewing stand, eight a furnace, and the six stone axe/hoe/pickaxe/shovel/spear/
sword patterns consume respectively `3/2/3/1/1/2` tag members. Cobbled possession satisfies the
tag-based unlock criterion for the furnace and six stone tools; brewing-stand acquisition instead
uses blaze-rod possession. Crafting orientation/reflection, grid consumption, remainders, furnace
progress, stonecutter selection and recipe-book publication remain with the generic owners.

#### Reloadable block tags

Cobbled, bricks, cracked bricks, tiles and cracked tiles are direct members of
`ancient_city_replaceable`; those five plus polished are direct members of
`sculk_replaceable_world_gen`; all seven are directly `mineable/pickaxe`. Neither block tag is
included by another locked block tag. Thus worldgen sculk may replace the six named masonry states
through its existing tag gate, and ancient-city block rot admits exactly the five replaceable
states. Chiseled is excluded from both replacement tags and polished is excluded from ancient-city
rot. Reload can change these memberships only through the ordinary tag-publication boundary.

#### Structure templates and degradation

The exhaustive scan finds `44,739` raw cells across the seven identities. The sole unreferenced
ancient-city input, `city_center/walls/bottom_right_corner`, contributes `299` cells (`222` bricks,
`14` tiles and `63` chiseled), so reachable inputs contribute `44,440`. Two trial-chambers hallway
templates contribute `12` cobbled cells each; both are referenced by the main and fallback hallway
pools. Their empty or copper-bulb-degradation processor choices do not target cobbled deepslate.

The remaining `44,416` reachable cells are ancient-city inputs. The three center templates use
start degradation and contain `16,197` family cells: `1,053` cobbled, `9,887` bricks, `18` cracked
bricks, `5,206` tiles, `12` cracked tiles and `21` chiseled. Start degradation performs no rot; it
independently tests each brick/tile cell with probability `0.3` and substitutes the matching
property-free cracked state on success. Other family identities pass through unchanged.

The generic-degradation inputs contain the other `28,219` reachable ancient-city cells:

| Raw identity | Cells | Generic result boundary |
|---|---:|---|
| cobbled | `1,520` | retain with integrity `0.95`, otherwise omit |
| polished | `2,547` | unchanged by both processors |
| bricks | `9,249` | integrity `0.95`, then `0.3` crack test if retained |
| cracked bricks | `198` | retain with integrity `0.95`, otherwise omit |
| tiles | `13,367` | integrity `0.95`, then `0.3` crack test if retained |
| cracked tiles | `45` | retain with integrity `0.95`, otherwise omit |
| chiseled | `1,293` | unchanged by both processors |

The rule processor follows block rot, so a removed brick/tile cell consumes no cracking test.
Protected-live-target rejection follows both processors. Pool choice, RNG, transforms, clipping,
target protection and writes remain with the jigsaw owners; raw counts are not unconditional final-
world counts.

**Client projection:**

Every state has one unconditional blockstate variant selecting the like-named block model. Each
model inherits `cube_all` with one like-named texture, and each item selector points to that same
block model. There is no weighted or property branch. Authoritative block updates publish the seven
listed state IDs; inventory projection uses the seven listed item raw IDs. This family adds no
packet field, ordering rule or connection-local state beyond the already audited registry mapping.

**Branches and aborts:**

Seven identities; correct/incorrect tool and explosion survival; 18 shaped, 42 cut and three cook
records plus eight tag-keyed shaped matches; exact versus tag ingredient, reflected pattern, output
capacity and unlock criterion; three direct block-tag membership sets; reachable/unreferenced,
start/generic/trial processor, rot, crack, transform, clip, protected target and client identity are
distinct branches.

**Constants and randomness:**

States and registry IDs as tabulated; strength `3.5/6`; sound IDs `501..515`, `1329..1333` and base
deepslate `506..510`; emission `0`, dampening `15`, shade `0.2`, friction `0.6`, factors `1`,
restitution `0`, stack `64`; recipe counts `18/42/3/63` and eight tag-keyed records; cook
`200/0.1/1`; raw/reachable/unreferenced structure cells `44,739/44,440/299`; trial cells `24`;
ancient-city start/generic cells `16,197/28,219`; generic integrity `0.95`; crack probability
`0.3`. Blocks consume no RNG themselves; loot, processor and client owners retain their streams.

**Side effects:**

Ordinary full-block placement and self loot; shaped, furnace and stonecutter outputs plus recipe
unlocks; tag-selected crafting/sculk/processor behavior; ancient-city and trial-chambers structure
writes; ordinary palette/inventory persistence; sound and opaque cube-all projection.

**Gates:**

Write authority; correct-tool harvest and explosion context; active recipe/advancement/loot/tag/
archetype snapshots; crafting/furnace/stonecutter admission and output capacity; jigsaw reachability,
processor RNG, clip/protected target; valid registry mapping and client resource context.

**Boundary cases and quirks:**

Polished deepslate is sculk-replaceable but not ancient-city-rot-eligible; chiseled is neither.
Start degradation cracks bricks/tiles without first applying integrity, while generic degradation
rots first and only then tests surviving bricks/tiles. Raw cracked cells can rot but are not tested
again for cracking. The two trial templates have identical 12-cell cobbled payloads but can be
selected from different pool/processor paths. Cobbled can craft a brewing stand without being the
criterion that unlocks that recipe.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-REPORT-001`; `OFF-DATA-001`;
`net.minecraft.world.level.block.Blocks`;
`net.minecraft.world.level.block.SoundType`;
`net.minecraft.world.item.crafting.ShapedRecipePattern#matches`;
`net.minecraft.world.item.crafting.AbstractCookingRecipe#cookingMapCodec`;
`net.minecraft.world.level.levelgen.structure.pools.SinglePoolElement#place`;
`net.minecraft.world.level.levelgen.structure.templatesystem.StructureTemplate#placeInWorld`;
`net.minecraft.world.level.levelgen.structure.templatesystem.BlockRotProcessor#processBlock`;
`net.minecraft.world.level.levelgen.structure.templatesystem.RuleProcessor#processBlock`;
`net.minecraft.world.entity.monster.cubemob.SulfurCube#matchingArchetypes`;
`reports/blocks.json#minecraft:{cobbled_deepslate,polished_deepslate,deepslate_bricks,cracked_deepslate_bricks,deepslate_tiles,cracked_deepslate_tiles,chiseled_deepslate}`;
`reports/registries.json#minecraft:{block,item,sound_event}`;
`reports/minecraft/components/item/{cobbled_deepslate,polished_deepslate,deepslate_bricks,cracked_deepslate_bricks,deepslate_tiles,cracked_deepslate_tiles,chiseled_deepslate}.json`;
`data/minecraft/{loot_table/blocks,recipe,advancement/recipes}/**/*deepslate*.json`;
`data/minecraft/recipe/{brewing_stand,furnace,stone_axe,stone_hoe,stone_pickaxe,stone_shovel,stone_spear,stone_sword}.json`;
`data/minecraft/tags/block/{ancient_city_replaceable,mineable/pickaxe,sculk_replaceable_world_gen}.json`;
`data/minecraft/tags/item/{stone_crafting_materials,stone_tool_materials,sulfur_cube_archetype/slow_bouncy}.json`;
`data/minecraft/worldgen/template_pool/{ancient_city,trial_chambers}/**/*.json`;
`data/minecraft/worldgen/processor_list/{ancient_city_start_degradation,ancient_city_generic_degradation,trial_chambers_copper_bulb_degradation}.json`;
`data/minecraft/structure/**/*.nbt`;
`assets/minecraft/{blockstates,models/block,items}/{cobbled_deepslate,polished_deepslate,deepslate_bricks,cracked_deepslate_bricks,deepslate_tiles,cracked_deepslate_tiles,chiseled_deepslate}.json`.

**Test vectors:**

Run `EXP-BLK-056` across all seven states and IDs, physical/tool/loot behavior, all 63 exact-ID and
eight tag-keyed recipes with paired unlocks, reloadable tag consumers, all containing and all 1,212
structure templates, start/generic/trial processors, transforms/clips/protected targets,
persistence, sounds and block/item models. Assert exact counts, match/result identity, processor
order and RNG ownership, raw/reachable/final structure outcomes and client convergence.

**Limits:**

Generic placement, breaking, loot, crafting, cooking, stonecutting, advancements, sulfur-cube
movement, sculk generation, jigsaw selection/processing, packet encoding and rendering remain with
`BLK-PLACE-001`, `PLY-BREAK-001`, `ITM-LOOT-001`, `ITM-RECIPE-001`, `ITM-FURNACE-001`,
`ITM-STONECUTTER-001`, `ITM-ADVANCEMENT-001`, `WGEN-PIPELINE-001`,
`WGEN-JIGSAW-PROCESSORS-001`, `WGEN-JIGSAW-ANCIENT-CITY-001`,
`PROTO-PLAY-CLIENTBOUND-TERRAIN-001`, `PROTO-PLAY-CLIENTBOUND-BLOCK-001`,
`PROTO-PLAY-CLIENTBOUND-SOUND-001` and `CLI-006`.
