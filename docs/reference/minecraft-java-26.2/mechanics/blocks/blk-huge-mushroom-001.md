# Blocks mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `BLK-HUGE-MUSHROOM-001` — Huge-mushroom faces join growth, composting and terrain blending

**Parent:** `SIM-004`, `SIM-005`, `SIM-RANDOM-001`, `BLK-001`, `BLK-STATE-001`,
`BLK-002`, `BLK-PLACE-001`, `BLK-BREAK-001`, `BLK-BREAK-HOOK-001`,
`BLK-BREAK-CONTENT-001`, `BLK-UPDATE-001`, `PLY-002`, `PLY-005`, `PLY-006`,
`PLY-BREAK-001`, `PLY-COLLISION-001`, `PLY-AUTOJUMP-001`, `BLK-003`, `BLK-004`,
`BLK-005`, `BLK-007`, `ITM-003`, `ITM-004`, `ITM-006`, `ITM-LOOT-001`,
`ENV-001`, `ENV-002`, `ENV-003`, `ENV-FIRE-001`, `ENV-LIGHT-001`,
`WGEN-003`, `WGEN-PIPELINE-001`, `CLI-001`, `CLI-006`, `CLI-UI-001`,
`CLI-EFFECT-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — the three locked registrations and reports, complete `HugeMushroomBlock`
control flow, exact block loot and hard-coded Composter consumers, both configured huge-mushroom
records and their feature kernels/selectors, terrain-blending exclusion, exhaustive exact-ID and
class-reference searches, all 1,212 decoded structure templates, and exact client resources close
the family.

**Applies when:**

`minecraft:brown_mushroom_block`, `minecraft:red_mushroom_block` or
`minecraft:mushroom_stem` is placed, receives a shape update, transformed, mined, exploded,
composted, produced by huge-mushroom growth or biome decoration, read during old/new chunk
blending, equipped on a Sulfur Cube, persisted, synchronized or rendered.

**Authoritative state:**

All three are `HugeMushroomBlock` full cubes without block entities. Each owns every combination
of `down,east,north,south,up,west` Boolean properties:

| Identity | Map color | State range | Block protocol ID | Item raw ID |
|---|---|---:|---:|---:|
| Brown Mushroom Block | `DIRT` | `7766..7829` | `338` | `415` |
| Red Mushroom Block | `COLOR_RED` | `7830..7893` | `339` | `416` |
| Mushroom Stem | `WOOL` | `7894..7957` | `340` | `417` |

The default state has all six properties true. Within each 64-state range, the offset is
`32*!down + 16*!east + 8*!north + 4*!south + 2*!up + !west`; therefore the all-false states are
`7829/7893/7957`. The properties affect face projection only. Every combination retains a full
unit selection/collision/visual/occlusion cube, emission `0`, light dampening `15`, shade
brightness `0.2`, friction `0.6`, speed/jump factors `1`, restitution `0`, solid redstone
conduction, normal piston reaction and full sturdy faces.

All registrations use `BASS`, hardness/resistance `0.2/0.2`, Wood sounds and
`ignitedByLava`. They directly belong to `mineable/axe`, require no correct tool and have no
minimum-tier tag. Axe membership accelerates mining, while hand and wrong-tool breaks still
evaluate loot. Brown and Red Mushroom Block additionally belong directly to
`replaceable_by_mushrooms`; Mushroom Stem does not. That tag is an input to the feature write
kernel, not generic player-placement replaceability.

Wood sound volume/pitch is `1/1`; break/step/place/hit/fall IDs are
`1853/1857/1856/1855/1854`. The ordinary block items are common 64-stacks with matching
translation/model keys. None has a recipe, advancement-specific grant, fuel value, food,
consumable, use action or other direct item tag beyond the slow-sliding equipment tag below.

**Transition and ordering:**

### Placement and sticky face state

Item placement begins from the all-true default and reads adjacent positions in exact property
order Down, Up, North, East, South, West. A property becomes false only when that neighbor is the
same exact block identity as the block being placed. Brown/Red/Stem adjacency does not cross-match.
Clicked face, player direction and RNG do not affect the result.

On a later shape update, if the new neighbor state is the same exact identity, the property for
the notified direction is set false. Any other neighbor delegates to the ordinary block hook and
does not restore a false property. Face hiding is therefore monotonic under neighbor updates:
removing a matching neighbor leaves the old face property false until some separate command,
placement or state transformation changes it.

Rotation remaps all six directional property values through the requested rotation; Up and Down
map to themselves. Mirror does the same through the mirror direction map. Identity, physical
shape and all non-face state remain unchanged.

### Harvest and loot

Each table has one roll and an identity-specific random sequence. Tool correctness and Fortune
do not gate any branch.

- Brown or Red Mushroom Block first tests tool Silk Touch level at least one. Success emits exactly
  one matching cap-block item with no explosion-decay function. Otherwise it samples one inclusive
  uniform integer from `-6..2`, clamps the count to at least zero, emits the matching small
  Mushroom item, then applies explosion decay. Before explosion decay the distribution is zero
  with probability `7/9`, one with `1/9`, and two with `1/9`.
- Mushroom Stem emits exactly one Stem item only when the tool has Silk Touch level at least one.
  Without Silk Touch its pool condition fails and the result is empty. It has no alternate
  small-mushroom drop and no explosion-decay function.

Thus ordinary non-Silk cap mining can yield at most two small Mushrooms and Stem mining yields
nothing. Silk Touch is checked by enchantment predicate rather than Axe/correct-tool admission.
Generic break context, empty-stack suppression and explosion context remain with the loot owners.

### Composter and equipment joins

The code-built Composter map admits Brown and Red Mushroom Block at Java-float chance `0.85f`
(`0.8500000238418579`) and Mushroom Stem at `0.65f`
(`0.6499999761581421`). A positive chance always advances an empty level-zero Composter without a
draw; levels one through six use `nextDouble() < chance`. Direct player and automated insertion,
consumption, event `1500`, level-seven scheduling and extraction remain with the Composter owner.

All three items directly select `sulfur_cube_archetype/slow_sliding`. Its record fixes
horizontal/vertical knockback `0.4125/0.09`, push cooldown `1`, impulse threshold `0.02`,
additive knockback and explosion-knockback resistance
`0.800000011920929/0.800000011920929`, additive bounciness `0.10000000149011612`,
total-multiplied friction `-0.9499999992549419` and air drag
`-0.9900000002235174`. Hit/push sounds are IDs `1951/1952`.

### Huge-mushroom configured features

The locked `huge_brown_mushroom` and `huge_red_mushroom` configurations use simple providers, so
provider sampling consumes no RNG. Both start with cap state
`down=false,east=true,north=true,south=true,up=true,west=true` and Stem state
`down=false,east=true,north=true,south=true,up=false,west=true`. Brown uses foliage radius `3`;
Red omits the field and therefore uses codec default `2`. Both floor predicates select the
respective, currently identical, `huge_*_mushroom_can_place_on` tag: composed
`substrate_overworld` plus Mycelium, Podzol, Crimson Nylium and Warped Nylium.

Every attempt first consumes `nextInt(3)+4` and then `nextInt(12)`. Base heights `4/5/6` each have
probability `11/36`; when the second draw is zero the selected height doubles to `8/10/12`, each
with probability `1/36`. The origin must be at least `minY+1`, `originY+height+1` must not exceed
`maxY`, and the floor must pass the configured predicate.

Clearance scans Y `0..height` in ascending order, then X and Z ascending:

- Brown requires only the origin column to be air or Leaves at Y `0..3`, and the full radius-three
  square at every higher scanned Y.
- Red requires the origin column below `height-3`, then the full radius-two square from
  `height-3` through `height`.

The first state that is neither air nor in `leaves` aborts with no feature writes. A valid attempt
writes the cap before the trunk. Every offered cell is written with flags `3` only when its live
state is air or in `replaceable_by_mushrooms`; write results are ignored.

Brown offers one radius-three cap layer at Y `height`, omitting the four corners: exactly 45 cap
cells. Up remains true and Down false. Its horizontal properties mark the outer edge and the
corner-adjacent continuation cells so the six multipart faces reproduce the rounded cap.

Red offers three radius-two rings at Y `height-3..height-1`, twelve cells per ring, then the full
radius-one 3-by-3 top: also exactly 45 cap cells. Up is false on the lower two rings and true on
the upper ring and top. West/East/North/South are selected by negative/positive X/Z relative to
the inner threshold zero; Down remains false.

Finally the trunk offers the configured Stem at origin X/Z for Y `0..height-1`, in ascending Y.
Successful ordinary placement therefore offers 45 cap cells plus `4/5/6/8/10/12` Stem cells,
for totals `49/50/51/53/55/57`. Neighbor-shape callbacks may additionally hide newly adjacent
same-identity faces; generic write/notification ownership remains separate.

### Growth callers and biome selectors

The corresponding small Mushroom stores the configured-feature key. Its server-side bonemeal
target check requires that the key resolve to an `AbstractHugeMushroomFeature` with a huge-mushroom
configuration and that `origin.above(4 + foliageRadius)` be inside build height. Success then
requires `nextFloat() < 0.4`. Growth removes the small Mushroom without drops, calls the configured
feature with the same RNG and origin, and restores the original state with flags `3` if placement
returns false. The preliminary height check does not account for doubled height, so a near-ceiling
attempt can pass target admission and then be restored after exact feature validation fails.

Dark Forest lists `dark_forest_vegetation` in decoration group `9`. Its placed feature performs
16 attempts, then in-square, maximum surface-water-depth zero, `OCEAN_FLOOR` heightmap and biome
filters. The ordered random selector tests Brown at `0.025f`, then Red at `0.05f` only after the
Brown draw fails. A selected feature's Boolean result returns immediately—failure does not
continue to a later selector or the default.

Mushroom Fields lists `mushroom_island_vegetation` in decoration group `9`. Each input passes
in-square, `MOTION_BLOCKING` heightmap and biome filters, then one `nextBoolean`: true selects Red
and false Brown. Exact placement-modifier iteration, feature seeding, chunk ordering and biome
decoration ownership remain with `WGEN-PIPELINE-001`.

The exhaustive 1,212-template scan finds zero Brown Mushroom Block, zero Red Mushroom Block and
zero Mushroom Stem cells. Their locked natural acquisition is procedural rather than raw NBT.

### Terrain-blending exclusion

During legacy/new-chunk blending height sampling, `BlendingData.isGround` explicitly rejects
Brown and Red Mushroom Block after rejecting air, Leaves and Logs, even though both cap blocks
have full collision. Mushroom Stem has no such exact exclusion and can pass the remaining
nonempty-collision ground test. This affects blending height evidence only; it does not alter the
blocks in the chunk.

**Client projection:**

Each blockstate file has twelve multipart clauses, one for each face/property-value pair. A true
property selects the identity's exterior single-face texture, while false selects the common
`mushroom_block_inside` single-face texture; fixed X/Y rotations orient the North source plane to
the other five faces. Exactly six planes are selected for every legal state. Brown, Red and Stem
inventory models independently use `cube_all` with their exterior texture, so item appearance
does not encode the placed face bits.

English names are exactly `Brown Mushroom Block`, `Red Mushroom Block` and `Mushroom Stem`.
Natural Blocks publishes Mushroom Stem after Pale Oak Log and before Crimson Stem; later it
publishes Brown then Red Mushroom Block after Flowering Azalea Leaves and before Nether Wart
Block. None appears in another ordinary tab.

State packets use the ranges and bit formula above; inventory paths use item IDs `415..417`.
Maps use the three fixed map colors, note blocks use `BASS`, and block sounds use the Wood profile.
No identity adds a packet field or connection-local state.

**Branches and aborts:**

All 192 states; same/different-identity placement on every face; sticky update, removal, explicit
patch, rotation and mirror; hand/Axe/Silk/Fortune/explosion loot; all Composter levels/draws and
slow-sliding replacement; configured-feature presence/type, bonemeal height/chance/remove/restore;
six height outcomes, floor/build-height/clearance failures, Brown/Red cap geometry, live
replaceability and write failures; Dark-Forest ordered selector and Mushroom-Fields Boolean
selector; blending scan; persistence and all client multipart states are distinct.

**Constants and randomness:**

Block/item IDs `338..340/415..417`; state bases `7766/7830/7894`; 64 states each; strength
`0.2/0.2`; stack `64`; Wood sounds `1853/1857/1856/1855/1854`; cap loot sample `-6..2`;
Composter chances `0.85f/0.65f`; slow-sliding values as listed; height bounds `3/12`, base `4`,
outcomes `4/5/6/8/10/12`; Brown/Red radii `3/2`; 45 cap cells; Dark-Forest count/chances
`16/0.025f/0.05f`; bonemeal chance `0.4f`. Placement/state updates consume no RNG; loot,
Composter, growth, worldgen selection and height retain their specified streams.

**Side effects:**

Block placement and sticky state writes; block/small-Mushroom loot; Composter mutation, event,
consumption and maturation schedule; equipment modifiers/sounds; small-Mushroom removal/restore;
procedural cap/Stem flags-3 writes and neighbor effects; blending height classification; ordinary
state/item persistence, maps, sounds, tabs and rendering.

**Gates:**

Legal registry/property value; placement and notification authority; exact same-block neighbor
identity; break/tool/enchantment/explosion context; active loot/tag/archetype snapshots;
Composter level/item/chance; configured-feature lookup/type, bonemeal height and chance; build
height, floor tag, air/Leaves clearance, live replacement and world write; placed-feature/biome
filters; old-chunk blending path; valid client resources.

**Boundary cases and quirks:**

- Face bits do not affect collision, light blocking or solidity, and false bits do not recover
  automatically when a matching neighbor disappears.
- Only equal block identities hide one another. Adjacent Brown, Red and Stem states retain their
  cross-identity face bits.
- Non-Silk cap loot intentionally samples seven nonpositive outcomes out of nine before clamping;
  Mushroom Stem has no non-Silk fallback.
- The cap configurations expose Down=false even on the top surface, while feature geometry and
  neighbor updates determine the other visible/inside faces.
- Both features consume their two height draws before any build-height, floor or clearance abort.
- Successful Brown and Red features each offer exactly 45 cap cells despite their different
  geometry.
- Bonemeal's preliminary ceiling check uses `4 + foliageRadius`, while final validation uses the
  sampled, possibly doubled, height.
- Cap blocks are explicit non-ground values for terrain blending; equally solid Stem is not.

**Failure semantics:**

Illegal property patches are rejected by the shared state owner. Nonmatching neighbor updates
preserve the state. Failed loot/Composter/tag/archetype admission commits only the generic owner's
documented effects. Missing/non-huge configured features reject bonemeal targeting; failed growth
restores the small Mushroom. Feature validation fails before writes, while admitted write Booleans
are ignored. Selector failure propagates without fallback after a feature is selected. Client
resource failure affects projection, not authoritative identity.

**Client/server authority split:**

The server owns registry/state identity, placement face calculation, updates, loot, Composter
selection, growth, biome features, blending and persistence. Clients project multipart faces,
inventory models, names, tabs, maps and playback/rendering of authoritative states and sounds.

**Observability:**

Commands/state packets, collision/light/signal probes, block updates, drops, inventories,
Composter events, equipment attributes/sounds, feature traces, chunk/blending output, maps, tabs
and rendering expose the listed branches.

**Persistence and reload:**

Placed blocks persist identity plus six Boolean properties and no block entity. Item stacks
persist ordinary components. Loot tables, block/item tags, archetypes, configured/placed features
and biome records are reloadable at their owners; direct registrations, class control flow,
Composter identity chances and blending exclusions remain code-built. Reload does not recompute
existing face bits or retroactively rewrite generated blocks.

**Evidence:**

`Confirmed`; `OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-DATA-001`; `OFF-REPORT-001`. Anchors:
`net.minecraft.world.level.block.Blocks`; `HugeMushroomBlock#getStateForPlacement`,
`#updateShape`, `#rotate` and `#mirror`; `ComposterBlock#bootStrap` and `#addItem`;
`AbstractHugeMushroomFeature`; `HugeBrownMushroomFeature`; `HugeRedMushroomFeature`;
`MushroomBlock#isValidBonemealTarget`, `#isBonemealSuccess`, `#growMushroom` and
`#performBonemeal`; `RandomSelectorFeature`; `RandomBooleanSelectorFeature`;
`BlendingData#isGround`; `CreativeModeTabs`; the three reports/component/loot/tag/resource sets,
two huge-mushroom configurations, both selector and placed-feature records, two biome records and
all 1,212 NBT templates. Complete exact-ID and compiled-field-reference searches found no other
recipe, advancement, trade, fuel, raw-template, acquisition or runtime path.

**Test vectors:**

Run `EXP-BLK-099` across all 192 states and IDs; same/cross-identity neighbors, every placement,
update/removal/patch/rotate/mirror path; hand/Axe/Silk/Fortune/explosion loot; all Composter and
slow-sliding branches; configured-feature reload/type/bonemeal boundaries; every height,
floor/ceiling/clearance/replace/write outcome and both cap geometries; both biome selectors and
all RNG boundaries; blending columns, all templates, persistence, maps, sounds, tabs and models.
Assert the exact constants, absence boundaries and client convergence.

**Limits:**

Generic placement/update propagation, breaking, loot evaluation, Composter lifecycle,
Sulfur-Cube behavior, feature seeding/placement modifiers, chunk generation/blending publication,
packet encoding and rendering remain with their named owners. The separate small Brown/Red
Mushroom spreading and survival family remains independently unreviewed. This leaf fixes the
three huge-mushroom block identities, their stateful hooks, exact joins, absences and projection.
