# Blocks mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `BLK-SMALL-MUSHROOM-001` — Small mushrooms spread under a local density cap and grow into huge features

**Parent:** `SIM-004`, `SIM-005`, `SIM-RANDOM-001`, `BLK-001`, `BLK-STATE-001`,
`BLK-002`, `BLK-PLACE-001`, `BLK-BREAK-001`, `BLK-BREAK-HOOK-001`,
`BLK-BREAK-CONTENT-001`, `BLK-UPDATE-001`, `PLY-002`, `PLY-005`, `PLY-006`,
`PLY-INTERACT-001`, `PLY-BREAK-001`, `PLY-COLLISION-001`, `PLY-AUTOJUMP-001`,
`ITM-003`, `ITM-004`, `ITM-006`, `ITM-RECIPE-001`, `ITM-LOOT-001`,
`ITM-ADVANCEMENT-001`, `MOB-004`, `MOB-005`, `MOB-AI-001`, `ENV-001`, `ENV-002`,
`ENV-003`, `ENV-LIGHT-001`, `WGEN-003`, `WGEN-PIPELINE-001`, `CLI-001`, `CLI-006`,
`CLI-UI-001`, `CLI-EFFECT-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — the two locked registrations and reports, complete `MushroomBlock` and
inherited `VegetationBlock` control flow, simple-block and attached-log generation callers, exact
loot/recipe/advancement/tag/trade/worldgen data, hard-coded Composter, horse, Mooshroom and
structure consumers, exhaustive exact-ID and class-reference searches, all 1,212 decoded
structure templates and exact client resources close the family.

**Applies when:**

`minecraft:brown_mushroom` or `minecraft:red_mushroom` is placed, loses survival, receives a
random tick, is bonemealed, mined, exploded, composted, carried by an Enderman, potted, selected
by crafting, trade, entity loot/shearing or world generation, consumed by a huge feature,
persisted, synchronized or rendered.

**Authoritative state:**

Both are property-free `MushroomBlock` instances without block entities:

| Identity | Huge-feature key | Map color | Emission | State | Block ID | Item ID |
|---|---|---|---:|---:|---:|---:|
| Brown Mushroom | `huge_brown_mushroom` | `COLOR_BROWN` | `1` | `2336` | `172` | `275` |
| Red Mushroom | `huge_red_mushroom` | `COLOR_RED` | `0` | `2337` | `173` | `276` |

Both registrations use the default `HARP` note instrument, zero hardness/resistance, random
ticks, no collision or occlusion, Grass sounds, a self-position postprocess callback and piston
reaction `DESTROY`. Their centered selection shape is
`(5,0,5)..(11,6,11)` in sixteenths; collision and occlusion are empty. Friction is `0.6`,
speed/jump factors are `1`, AIR pathfinding is allowed, and an empty fluid state propagates
skylight. Neither state is sturdy, suffocating, view-blocking, a spawn floor, signal-producing,
comparator-readable, waterloggable or backed by a scheduled/block-entity tick.

Grass sound volume/pitch is `1/1`; break/step/place/hit/fall sound IDs are
`755/759/758/757/756`. Each ordinary block item is a common nondamageable 64-stack with the
standard block-item components. Brown has no direct item tag; Red alone is the sole direct member
of `zombie_horse_food`.

Both blocks directly belong to `enderman_holdable` and `replaceable_by_mushrooms`. The latter
admits huge-mushroom feature replacement; it does not set the generic player-placement
`replaceable` property. Both are also explicit values in the four Crimson/Warped, natural/planted
huge-fungus `replaceable_blocks` predicates. They have no mining-tool, minimum-tier, flammability,
fuel, slow-sliding or other direct block/item tag.

**Transition and ordering:**

### Placement, survival and support loss

Ordinary placement and every later neighbor-shape update use the same survival predicate. Let
`below` be the state directly under the Mushroom:

1. If `below` is in `overrides_mushroom_light_requirement`, survival returns true immediately.
   The locked tag is exactly Mycelium, Podzol, Crimson Nylium and Warped Nylium.
2. Otherwise the raw brightness at the Mushroom position must be strictly below `13`, and
   `below.isSolidRender()` must be true.

Thus ordinary solid-render support admits light levels `0..12`, rejects `13..15`, and the four
override substrates ignore both light and the later solid-render test. Brown's own emission one
does not create an exception to this shared predicate.

An admitted item placement writes the sole state. Any neighbor-shape notification re-evaluates
survival even when the changed neighbor is not below. Failure returns AIR; ordinary
`updateOrDestroy` handling removes the Mushroom and evaluates its block loot unless the initiating
caller suppresses drops. A forced state write can leave an invalid Mushroom until a qualifying
shape update. Rotation and mirror are identity operations.

Neither state is generically player-replaceable. Pistons destroy rather than move it, and fluid
placement has no retained fluid state. Huge-mushroom writes may overwrite either identity through
`replaceable_by_mushrooms`; all four huge-fungus configurations may overwrite them through their
separate explicit predicate.

### Density-bounded random spreading

Every selected server random tick first consumes `nextInt(25)`. A nonzero result returns. On zero,
the block sets a counter to five and scans the inclusive box
`origin + (-4,-1,-4)..(+4,+1,+4)`, containing `9*3*9 = 243` positions. Every state whose block is
the same exact `MushroomBlock` identity decrements the counter; reaching zero returns immediately.
The origin counts, so spreading proceeds only with at most four same-color Mushrooms in the box
and can create the fifth. Brown never counts Red and vice versa.

When under the cap, the method performs a five-candidate random walk. Each candidate consumes, in
order:

```text
dx = nextInt(3) - 1
dy = nextInt(2) - nextInt(2)
dz = nextInt(3) - 1
```

`dx` and `dz` are uniform over `-1,0,1`; `dy` is `-1/0/+1` with probabilities
`1/4,1/2,1/4`. Candidate one is relative to the original anchor. For candidates one through four,
an empty candidate where the original state can survive becomes the new anchor; regardless of
that result, the next candidate is sampled relative to the current anchor. Candidate five is
tested but never promoted. If it is empty and survivable, the method offers the original state
with flags `2`; the write result is ignored.

An admitted under-cap tick therefore consumes exactly 20 bounded position draws after the
admission draw, even when no candidate is valid or the final write fails. A density-cap abort
consumes none of those draws. Spread requires actual air through `isEmptyBlock`; merely
replaceable nonair states are not candidates.

### Bonemeal and huge growth

Bonemeal target admission is server-only. The stored configured-feature key must resolve, its
feature must be an `AbstractHugeMushroomFeature`, and its configuration must be a
`HugeMushroomFeatureConfiguration`. The preliminary height probe is
`origin.above(4 + foliageRadius)`: Brown therefore probes seven blocks above and Red six. That
probe must be inside build height.

After target admission, success consumes one `nextFloat()` and requires `< 0.4f`. Performing
growth removes the small Mushroom without drops, invokes the resolved configured feature at the
same origin with the same RNG and live chunk generator, and returns on feature success. Feature
failure restores the original small-Mushroom state with flags `3`; missing lookup returns false
without removal. The public bonemeal callback ignores that Boolean result.

`BLK-HUGE-MUSHROOM-001` owns the subsequent height draws, floor/clearance checks, exact Brown/Red
cap geometry, cap-before-Stem writes and restoration-visible result. In particular, the
preliminary target probe does not account for a doubled feature height, so an admitted
near-ceiling attempt can remove and then restore the Mushroom after feature validation fails.

### Loot, composting and potting

Each unpotted block table has one roll, one matching item and one `survives_explosion` condition
under random sequence `minecraft:blocks/<mushroom>`. Hand, every tool and support-loss destruction
therefore yield one matching Mushroom without Silk Touch or Fortune when no explosion radius is
present. An explosion independently applies the generic survival draw.

The hard-coded Composter map admits both items at Java-float chance `0.65f`
(`0.6499999761581421`). Level zero always advances without a draw; levels one through six advance
exactly when `nextDouble() < chance`. Direct/automated consumption, event `1500`, level-seven
scheduling and extraction remain with the Composter owner.

The code-built flower-pot map accepts each item into its corresponding potted state:
Red `10655`/block ID `437`, Brown `10656`/block ID `438`. Empty-hand extraction and clone-pick
return the matching Mushroom. Each potted loot table independently offers one Flower Pot and one
matching Mushroom through two `survives_explosion` pools. The complete insertion, occupied-pot,
inventory-full, drop and explosion transaction remains with `BLK-FLOWER-POT-001`.

### Recipes, entity acquisition and trade

The bundled shapeless consumers are exact:

- Mushroom Stew consumes one Bowl, Brown Mushroom and Red Mushroom.
- Brown alone joins Spider Eye and Sugar for one Fermented Spider Eye.
- Rabbit Stew has separate Brown and Red recipes; each adds the selected Mushroom to Bowl, Baked
  Potato, Carrot and Cooked Rabbit.
- Seventeen Suspicious Stew recipes each consume Bowl, Brown, Red and one flower. Their result
  payloads are Fire Resistance `60` from Allium; Blindness `220` from Azure Bluet/Open Eyeblossom;
  Saturation `7` from Blue Orchid/Dandelion/Golden Dandelion; Nausea `140` from Closed
  Eyeblossom; Jump Boost `100` from Cornflower; Poison `220` from Lily of the Valley; Weakness
  `140` from all four ordinary Tulips; Regeneration `140` from Oxeye Daisy; Night Vision `100`
  from Poppy/Torchflower; and Wither `140` from Wither Rose.

The Mushroom-Stew recipe advancement awards that recipe when any one of recipe-unlocked, held
Mushroom Stew, Bowl, Brown or Red criteria passes. The Bowl recipe advancement likewise has one
OR group containing recipe-unlocked, Brown, Red and Mushroom Stew. Each Rabbit-Stew variant is
awarded by its recipe-unlocked criterion or possession of Cooked Rabbit. The seventeen flower
recipes and their unlock records remain independently keyed. Generic matching, assembly,
criterion persistence and stew consumption remain with their item/progression owners.

Entity and merchant acquisition is also finite:

- The Bogged shearing table performs two rolls over equal-weight Brown/Red entries, so both rolls
  choose independently. A ready, unsheared Bogged plays its shear sound, emits the selected item
  entities and becomes sheared.
- The Mooshroom shearing dispatcher selects its Red/Brown variant table through the
  `mooshroom/variant` component. Each selected table rolls five matching Mushrooms. An adult
  Mooshroom plays its shear sound and converts to a Cow before that table emits the items.
  Variant IDs `0/1` store the Red/Brown default block states for entity projection.
- A Zombie riding a Zombie Horse has an additional Red-Mushroom loot pool: inclusive uniform
  count `0..1`, followed by an inclusive uniform `0..1` Looting increase per applicable level.
- Red Mushroom is the only `zombie_horse_food` value. The shared horse food kernel assigns it
  healing `3`, baby growth `0` and temper increase `3`; it consumes the item and emits the eating
  effects only when healing or temper actually changes state.
- Both Wandering-Trader common-pool records exchange one Emerald for three matching Mushrooms
  with reputation discount `0.05`. Pool selection and offer lifecycle remain with the trade owner.

Both blocks are Enderman-holdable. The generic take goal can remove a selected matching state with
drops disabled and retain its default state as carried state. The leave goal first transforms the
carried state through neighbor shapes, then requires target air, a nonair/non-Bedrock full
collision block below, state survival and an entity-free unit box. Consequently a bright ordinary
solid substrate can transform a carried Mushroom to AIR and clear it without placing the
Mushroom, while an override substrate retains it. Gamerule, RNG, ray, scheduling, carried-state
persistence and block-event details remain with the Enderman owner.

### Reload-selected and structure generation

The two simple configured features use deterministic simple-state providers for states `2336` and
`2337`; `schedule_tick` is omitted and false. `SimpleBlockFeature` samples the provider, rejects a
null or nonsurviving state, otherwise offers it with flags `2`, ignores the write result and
returns true. The family has five placed profiles per color. After the prefix shown below, every
chain ends with biome, count `96`, offsets `7/3` and air:

| Profile | Brown prefix | Red prefix | Shared position source |
|---|---|---|---|
| Nether | rarity `2` | rarity `2` | in-square, uniform full height |
| Normal | rarity `256` | rarity `512` | in-square, `MOTION_BLOCKING` |
| Old Growth | count `3`, rarity `4` | rarity `171` | in-square, `MOTION_BLOCKING` |
| Swamp | count `2` | rarity `64` | in-square, `MOTION_BLOCKING` |
| Taiga | rarity `4` | rarity `256` | in-square, `MOTION_BLOCKING` |

Each `offsets 7/3` entry uses independent trapezoid X/Z `[-7,7]` and Y `[-3,3]`, followed by an
exact `air` block-tag predicate. The Nether pair occurs in Basalt Deltas and Nether Wastes at
decoration group `7`. Both Normal profiles occur at group `9` in the same 44 biomes: Badlands,
Bamboo Jungle, Beach, Birch Forest, Cold Ocean, Crimson Forest, Dark Forest, Deep Cold Ocean,
Deep Dark, Deep Frozen Ocean, Deep Lukewarm Ocean, Deep Ocean, Desert, Dripstone Caves, Eroded
Badlands, Flower Forest, Forest, Frozen Ocean, Frozen River, Ice Spikes, Jungle, Lukewarm Ocean,
Nether Wastes, Ocean, all three Old Growth forest/taigas, Plains, River, Savanna, Savanna Plateau,
Snowy Beach, Snowy Plains, Sparse Jungle, Stony Shore, Sunflower Plains, Swamp, Warm Ocean, Warped
Forest, Windswept Forest, Windswept Gravelly Hills, Windswept Hills, Windswept Savanna and Wooded
Badlands. Old-Growth profiles additionally occur in both Old Growth Taigas; Swamp profiles in
Swamp; and Taiga profiles in Mushroom Fields, Snowy Taiga and Taiga, all at group `9`.

Five Fallen Tree configurations—Birch, Jungle, Oak, Spruce and Super Birch—also attach Mushrooms
above shuffled logs. Each uses direction Up, probability `0.1f` and a weighted provider with Red
weight two and Brown weight one. For every log, the decorator consumes its probability draw before
the air test; `nextFloat() <= 0.1f` admits the candidate, then provider selection occurs. This path
does not call `MushroomBlock.canSurvive`, so it can write a bright Mushroom that later pops on a
neighbor update.

The raw-template census finds six Brown cells, all in
`woodland_mansion/1x2_a7.nbt`, and ten Red cells: four in that same room plus two each in the
Trial-Chambers `ranged/poison_skeleton`, `slow_ranged/poison_skeleton` and
`small_melee/cave_spider` spawner templates. No cell has block NBT. Separately,
`SwampHutPiece` procedurally places one Potted Red Mushroom at local `(1,3,5)` when that cell is
inside its live bounding box. Structure transforms, clipping, replacement and placement flags
remain with the corresponding structure owners.

**Client projection:**

Each property-free blockstate selects its matching untinted `block/cross` model. The two crossed
planes are full-height, shade-disabled and ambient-occlusion-disabled even though the server
selection shape is only six sixteenths high. Both textures request `strict_cutout` mipmapping.
The item selector uses a flat generated model whose sole layer is that same block texture.

English names are exactly `Brown Mushroom` and `Red Mushroom`. Natural Blocks publishes Brown
then Red after Flowering Azalea and before Crimson Fungus; neither appears in another ordinary
tab. Potted models remain with the flower-pot projection owner.

Block updates publish states `2336/2337`; inventory paths use raw item IDs `275/276`. Brown alone
emits light one, maps use `COLOR_BROWN/COLOR_RED`, note blocks read `HARP`, and both use the five
Grass sounds. Mooshroom projection separately uses the same default block states through its
variant metadata. This family adds no packet field or connection-local state.

**Branches and aborts:**

Both identities; ordinary/forced placement; four override versus ordinary solid/nonsolid
substrates; raw brightness `12/13`; every neighbor update; hand/tool/explosion/support-loss loot;
same/cross-color density counts `4/5`; admission and every five-step walk shape; empty/nonempty and
survival/write outcomes; configured-feature missing/type/height/chance/remove/success/restore;
Composter and potted paths; every recipe, advancement, shearing, mounted-Zombie, horse-food,
trade and Enderman selector; all ten placed profiles, five fallen-tree providers, procedural hut
and 1,212-template census; persistence and client projection are distinct.

**Constants and randomness:**

States `2336/2337`; block IDs `172/173`; item IDs `275/276`; potted states
`10655/10656` and block IDs `437/438`; shape `(5,0,5)..(11,6,11)`; strength `0/0`;
emission `1/0`; sounds `755/759/758/757/756`; stack `64`; survival threshold `<13`; spread
admission `1/25`, box `9*3*9`, cap `5`, five candidates and 20 position draws; bonemeal chance
`0.4f`, preliminary heights `7/6`; Composter `0.65f`; shearing rolls `2/5`; trade
`1 -> 3`, discount `0.05`; placed-profile constants as tabulated; fallen-tree chance/weights
`0.1f/2:1`; raw templates/cells `1212/6/10`.

**Side effects:**

Placement, support-loss destruction/drop and spread writes; bonemeal consumption,
small-Mushroom removal/restore and huge-feature writes; loot and Composter mutation/events;
pot state/item/stat effects; recipe consumption/rewards; shearing conversion/state/items;
horse healing/temper/eating event; merchant offers; Enderman carried state/removal/placement;
simple/attached-log/template/hut generation; ordinary persistence; light, map, sounds, entity
variant, cross model, sprite and tab projection.

**Gates:**

World-write/break authority; active block/loot/item/trade/worldgen snapshots; substrate tag,
solid-render and raw-light reads; random-tick selection and same-identity density; candidate air
and survival; bonemeal server/feature/type/height/chance; explosion context; Composter level and
draw; pot occupancy; shearing/entity/vehicle/horse state; Enderman gamerule/ray/placement; biome,
placement modifier, tree/template/structure bounds; registry and client-resource context.

**Boundary cases and quirks:**

- Override substrates bypass both brightness and the ordinary solid-render check.
- Any neighbor notification can pop a Mushroom because survival is rechecked without testing the
  changed direction.
- The density cap is color-local, includes the origin and admits at most four before a spread can
  create the fifth.
- All five candidates are sampled on an under-cap admitted tick; only the first four may move the
  walk anchor, and only the fifth may be written.
- Spread requires air, while huge mushrooms and huge fungi use two different replacement
  predicates that both include the small Mushrooms.
- Bonemeal's preliminary ceiling probe is shorter than some sampled huge features; failed final
  placement restores the removed Mushroom.
- Fallen-tree attachment bypasses Mushroom survival, unlike the simple-block profile.
- Brown's emission one was formerly sufficient for broad structural catalog ownership but did not
  cover any of this shared `MushroomBlock` dispatch; this exact family replaces that
  misclassification.
- Red has the only horse-food/mounted-Zombie/hut asymmetries; Brown has the only light and
  Fermented-Spider-Eye asymmetries.

**Failure semantics:**

Invalid ordinary placement is rejected. Shape-update failure returns AIR through the generic
update owner. Spread density/admission/candidate failures preserve the original world; an admitted
write result is ignored. Missing/invalid huge-feature targeting rejects bonemeal, while failed
post-removal placement restores the state. Failed loot/Composter/pot/recipe/entity/trade/tag
admission commits only the generic owner's stated effects. Simple-block survival failure returns
false; its admitted write result is ignored. Client-resource failure affects projection, not
authoritative identity.

**Client/server authority split:**

The server owns identity, survival, random ticks, growth, loot, inventories, recipes, entities,
trades, world generation, persistence and emitted light. Clients project the authoritative state,
entity variant, sounds, map color, models, sprites, names and tabs.

**Observability:**

Commands/state packets, raw-light and support probes, scheduled random-tick traces, drops,
inventories, recipes/advancements, Composter and pot events, entity loot/shearing/feeding,
merchant offers, carried blocks, feature/template output, maps, sounds, tabs and rendering expose
the listed branches.

**Persistence and reload:**

Placed states persist only identity and have no block entity. Item stacks persist ordinary
components. Loot, block/item/trade tags, recipes, advancements, shearing/entity tables,
configured/placed features, biomes and structure templates are reload-selected at their owners.
Direct registrations, `MushroomBlock` control flow, Composter chances, horse constants,
Mooshroom variant mapping and procedural Swamp-Hut placement remain code-built. Reload does not
retroactively recheck existing support/light or resample existing generation.

**Evidence:**

`Confirmed`; `OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-DATA-001`; `OFF-REPORT-001`. Anchors:
`net.minecraft.world.level.block.Blocks`; `MushroomBlock#getShape`, `#randomTick`, `#mayPlaceOn`,
`#canSurvive`, `#growMushroom`, `#isValidBonemealTarget`, `#isBonemealSuccess` and
`#performBonemeal`; `VegetationBlock#updateShape`, `#propagatesSkylightDown` and
`#isPathfindable`; `SimpleBlockFeature#place`; `AttachedToLogsDecorator#place`;
`ComposterBlock#bootStrap`; `MushroomCow$Variant`; `MushroomCow#shear`; `Bogged#shear`;
`AbstractHorse#handleEating`; `ZombieHorse#isFood`; `SwampHutPiece#postProcess`;
`CreativeModeTabs`; the two reports/component/block-loot/tag/resource sets, four potted/shearing
tables, Zombie table, 21 recipe records and their advancements, two trade records/common tag,
both simple configurations, ten placed profiles, every referencing biome, five Fallen Tree
configurations, four huge-fungus configurations and all 1,212 NBT templates. Complete exact-ID
and compiled-field-reference searches found no other acquisition, use, generation or runtime path.

**Test vectors:**

Run `EXP-BLK-100` across both states and IDs; every support/light/update/loot boundary; same- and
cross-color density boxes plus every five-candidate walk; feature reload/type/bonemeal
remove/restore; Composter, pot, recipe, advancement, entity, horse, trade and Enderman joins; all
ten placed profiles, five Fallen Tree providers, four huge-fungus predicates, hut placement and
all 1,212 templates; persistence, light, maps, sounds, tabs and models. Assert exact constants,
conditional draw/read/write order, absence claims and client convergence.

**Limits:**

Generic placement/update/break/loot/explosion, Composter and flower-pot lifecycles, crafting and
advancement dispatch, entity/shearing/horse/trade state machines, Enderman goal scheduling,
huge-feature geometry, placement modifiers, Fallen Tree orchestration, structure placement,
packet encoding and rendering remain with their named owners. This leaf fixes the two small
Mushroom identities, their custom survival/spread/growth dispatch, exact joins, asymmetries,
acquisition and projection.
