# Blocks mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `BLK-NETHER-FUNGUS-001` — Nether fungi share reloadable support but grow only from color-matched nylium

**Parent:** `SIM-004`, `SIM-005`, `BLK-001`, `BLK-STATE-001`, `BLK-002`,
`BLK-PLACE-001`, `BLK-BREAK-001`, `BLK-BREAK-HOOK-001`, `BLK-BREAK-CONTENT-001`,
`BLK-UPDATE-001`, `PLY-002`, `PLY-005`, `PLY-006`, `PLY-INTERACT-001`,
`PLY-BREAK-001`, `PLY-COLLISION-001`, `PLY-AUTOJUMP-001`, `ITM-003`, `ITM-004`,
`ITM-006`, `ITM-RECIPE-001`, `ITM-LOOT-001`, `ITM-ADVANCEMENT-001`, `MOB-004`,
`MOB-005`, `MOB-AI-001`, `MOB-BREED-001`, `ENV-001`, `ENV-002`, `ENV-003`,
`WGEN-003`, `WGEN-PIPELINE-001`, `WGEN-JIGSAW-BASTION-001`, `CLI-001`, `CLI-006`,
`CLI-UI-001`, `CLI-EFFECT-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — the two locked registrations and reports, complete
`NetherFungusBlock` and inherited `VegetationBlock` control flow, bonemeal caller,
huge-fungus and nether-vegetation joins, exact loot/recipe/advancement/tag/worldgen data,
hard-coded Composter and mob consumers, exhaustive exact-ID and compiled-field-reference
searches, all 1,212 decoded structure templates and exact client resources close the family.

**Applies when:**

`minecraft:crimson_fungus` or `minecraft:warped_fungus` is placed, loses support,
is bonemealed, mined, exploded, composted, potted, selected by chest loot, crafting,
Hoglin/Strider AI, Enderman carriage or world generation, consumed by a huge feature,
persisted, synchronized or rendered.

**Authoritative state:**

Both are property-free `NetherFungusBlock` instances without block entities:

| Identity | Planted feature | Required growth base | Map color | State | Block ID | Item ID |
|---|---|---|---|---:|---:|---:|
| Crimson Fungus | `crimson_fungus_planted` | Crimson Nylium | `NETHER` | `20975` | `876` | `277` |
| Warped Fungus | `warped_fungus_planted` | Warped Nylium | `COLOR_CYAN` | `20958` | `867` | `278` |

Both registrations use the default `HARP` note instrument, zero hardness/resistance,
zero emission, no collision or occlusion, Fungus sounds and piston reaction `DESTROY`.
They do not request random ticks. Their centered selection shape is
`(4,0,4)..(12,9,12)` in sixteenths; collision and occlusion are empty. Friction is
`0.6`, speed/jump factors are `1`, AIR pathfinding is allowed, and an empty fluid state
propagates skylight. Neither state is sturdy, suffocating, view-blocking, a spawn floor,
signal-producing, comparator-readable, waterloggable or backed by a scheduled/block-entity tick.

Fungus sound volume/pitch is `1/1`; break/step/place/hit/fall sound IDs are
`1136/1137/1138/1139/1140`. Each ordinary block item is a common nondamageable
64-stack with the standard block-item components.

Both blocks directly belong to `enderman_holdable`. Warped alone is a direct
`hoglin_repellents` block, Crimson alone is the sole `hoglin_food` item, and Warped
alone is the sole `strider_food` item. Neither has a mining-tool, minimum-tier,
flammability, fuel, slow-sliding or other direct block/item tag. They are not
generically player-replaceable, but all four bundled huge-fungus configurations name
both in their separate 59-block replacement predicate.

**Transition and ordering:**

### Placement, support and support loss

Ordinary placement and every later neighbor-shape update call the same support predicate.
The state directly below must belong to the instance's stored, reloadable support tag;
there is no brightness, solidity or fluid test.

The locked `supports_crimson_fungus` tag contains only
`#minecraft:supports_warped_fungus`. The latter contains
`#minecraft:supports_vegetation`, `#minecraft:nylium`, Mycelium and Soul Soil.
After recursive expansion and deduplication, both current tags therefore admit the same
14 blocks:

- Coarse Dirt, Dirt, Farmland, Grass Block, Moss Block, Mud, Muddy Mangrove Roots,
  Mycelium, Pale Moss Block, Podzol and Rooted Dirt;
- Crimson Nylium, Warped Nylium and Soul Soil.

That equality is data-selected rather than hard-coded: replacing either tag can make the
two support sets diverge without changing the stored growth bases below.

An admitted item placement writes the sole state. Any neighbor-shape notification
re-evaluates support even when the changed neighbor is not below. Failure returns AIR;
ordinary `updateOrDestroy` handling removes the Fungus and evaluates block loot unless
the initiating caller suppresses drops. A forced invalid state can remain until a
qualifying shape update. Rotation and mirror are identity operations.

Pistons destroy rather than move either state, and fluid placement has no retained fluid
state. The blocks are not ordinary placement replacements. Huge-fungus stem generation
can overwrite them through its configured predicate and planted destruction rules.

### Bonemeal target, success and feature dispatch

Bonemeal admission is intentionally narrower than survival:

1. The block immediately below must be the instance's exact stored `requiredBlock`,
   Crimson Nylium for Crimson and Warped Nylium for Warped. Properties are irrelevant.
2. `origin.above()` must be inside build height.

The target check does not consult the support tag or configured-feature registry.
Consequently a tag reload can make a Fungus invalid for ordinary survival while the
same forced state remains a valid target over its exact nylium; the opposite is also
possible on a nonmatching support-tag member.

The common bonemeal item calls this target predicate on both sides. A valid client call
predicts success without consuming an item or RNG. On the server, target admission calls
`nextFloat()` and growth succeeds strictly below `0.4f`. Whether that draw succeeds or
fails, the server then consumes one Bone Meal and reports the admitted interaction.
Only a successful draw calls `performBonemeal`.

Execution looks up the stored configured-feature key in the live configured-feature
registry. Missing lookup does nothing. A present holder is invoked at the Fungus origin
with the same server level, live chunk generator and RNG; neither its feature type nor
configuration is checked, and its Boolean result is ignored. There is no separate
small-Fungus removal or restoration transaction.

With the locked data, the selected feature is the color-matched planted huge fungus.
Its exact-base check is already satisfied, it skips the ordinary generation-depth
ceiling rejection and is always narrow. The delegated `WGEN-PIPELINE-001` transaction
then offers AIR at the origin with flags `260`, builds stem before hat and returns true
after admission regardless of later destroy/write results. It may destroy the origin
with drops through planted-mode rules; failure has no Fungus restoration.

The inherited bonemealable type is `GROWER`, so generic growth particles target
`origin.above()`. Bone Meal consumption, vibration and level event `1505` remain with
the generic item owner.

### Loot, composting and potting

Each unpotted block table has one roll, one matching item and one
`survives_explosion` condition under random sequence
`minecraft:blocks/<fungus>`. Hand, every tool and support-loss destruction therefore
yield one matching Fungus without Silk Touch or Fortune when no explosion radius is
present. An explosion independently applies the generic survival draw.

The hard-coded Composter map admits both items at Java-float chance `0.65f`
(`0.6499999761581421`). Level zero always advances without a draw; levels one through
six advance exactly when `nextDouble() < chance`. Direct/automated consumption,
event `1500`, level-seven scheduling and extraction remain with the Composter owner.

The code-built flower-pot map accepts Crimson into state `21826`/block ID `919` and
Warped into state `21827`/block ID `920`. Empty-hand extraction and clone-pick return
the matching Fungus. Each potted loot table independently offers one Flower Pot and
one matching Fungus through two `survives_explosion` pools. The complete insertion,
occupied-pot, inventory-full, drop and explosion transaction remains with
`BLK-FLOWER-POT-001`.

### Chest, recipe and mob joins

Crimson alone appears in the Bastion Hoglin-Stable chest table. Its second pool takes a
uniform three or four rolls with replacement over 14 equal-weight entries. Selecting
Crimson Fungus applies an inclusive uniform count `2..7`. The other pools, random
sequence, container seed and Bastion placement remain with the loot/jigsaw owners.
No other direct nonblock loot table or merchant record names either item.

Warped alone is consumed by the shaped `warped_fungus_on_a_stick` recipe. Its two-row
pattern is Fishing Rod at upper-left and Warped Fungus at lower-right
(`"# "`, `" X"`), with Rod/Fungus represented by `#`/`X`.
The result is one Warped Fungus on a Stick. Its recipe advancement awards the recipe
when either that recipe is already unlocked or the inventory contains Warped Fungus.
Generic matching, assembly, remaining-item and criterion persistence stay with their
owners. Crimson has no bundled recipe consumer.

The live food/AI joins are asymmetric:

- Crimson's sole `hoglin_food` membership enters generic Animal feeding and breeding.
  A successful Hoglin interaction also marks it persistent; a pacified Hoglin cannot
  enter love. Child creation and parent/XP finalization remain with `MOB-BREED-001`.
- Warped's sole `strider_food` membership enters generic Strider feeding and breeding.
  The composed `strider_tempt_items` tag contains that food tag plus Warped Fungus on
  a Stick. Strider installs the predicate as a priority-three `TemptGoal` at speed
  `1.4`, with the scared-by-movement option false.
- Raw and potted Warped Fungus are two of the four current `hoglin_repellents`.
  The Hoglin-specific sensor scans the closest tagged block within horizontal range
  `8` and vertical range `4` on its default 20-tick cadence and writes/erases
  `NEAREST_REPELLENT`. Idle/fight behavior turns a new memory into `PACIFIED=true`
  for `200` ticks and erases `ATTACK_TARGET`; idle behavior also requests a
  speed-`1.0` walk target at least eight blocks away.

Both raw blocks are Enderman-holdable. The generic take goal removes a selected matching
state without drops and stores its default state. The leave goal first transforms the
carried state through neighbor shapes, then applies its air/full-collision/survival/entity
gates. An unsupported placement target can therefore transform a carried Fungus to AIR
and clear it without placing the Fungus. Gamerule, ray, RNG, scheduling, persistence and
block-event details remain with the Enderman owner.

### Reload-selected world generation

Four `nether_forest_vegetation` configurations can generate the two property-free states.
The ordinary and bonemeal variants retain the same provider weights but use different
spread dimensions:

| Configuration pair | Ordinary width/height | Bonemeal width/height | Weighted provider |
|---|---:|---:|---|
| Crimson Forest | `8/4` | `3/1` | Crimson Roots `87`, Crimson Fungus `11`, Warped Fungus `1` |
| Warped Forest | `8/4` | `3/1` | Warped Roots `85`, Crimson Roots `1`, Warped Fungus `13`, Crimson Fungus `1` |

The feature first requires nylium below and an admitted origin-height window. It performs
`width*width` attempts: `64` ordinary or `9` bonemeal. Every attempt consumes six
triangular offset draws and selects a provider state before candidate air, minimum-Y and
selected-state-survival gates. An admitted state is offered with flags `2`; writes are
ignored, and the feature result tracks offers rather than successful writes. Exact
feature control flow remains with `WGEN-PIPELINE-001`.

Placed `crimson_forest_vegetation` uses count-on-every-layer `6` then biome;
`warped_forest_vegetation` uses `5` then biome. Crimson Forest and Warped Forest place
their matching wrapper at decoration group `9`. Matching Nylium bonemeal invokes the
width-three configuration one block above the Nylium. Warped Nylium subsequently invokes
Nether Sprouts and conditionally Twisting Vines; those extra branches do not change the
Fungus provider weights.

Two other placed profiles select the ordinary huge-fungus configurations:

- `crimson_fungi` uses count-on-every-layer `8`, then biome, at group `9` in Crimson
  Forest after Weeping Vines and before Crimson Forest Vegetation.
- `warped_fungi` uses the same count and modifiers at group `9` in Warped Forest after
  Red Mushroom Normal and before Warped Forest Vegetation.

All four huge-fungus configurations contain both Fungus identities in the same exact
59-block `matching_blocks` replacement predicate. The two stored block keys instead
select the planted matching-color records for bonemeal. Huge topology, probabilities,
destruction and write order remain with `WGEN-PIPELINE-001`.

The exhaustive raw-template census finds zero Crimson or Warped Fungus cells across all
1,212 bundled structure templates. Bastion acquisition comes from the Hoglin-Stable
chest loot table rather than a raw Fungus cell.

**Client projection:**

Each property-free blockstate selects its matching untinted `block/cross` model. The two
crossed planes are full-height, shade-disabled and ambient-occlusion-disabled even though
the server selection shape is only nine sixteenths high. Both textures request
`strict_cutout` mipmapping. The item selector uses a flat generated model whose sole
layer is the same block texture.

English names are exactly `Crimson Fungus` and `Warped Fungus`. Natural Blocks publishes
Crimson then Warped immediately after Red Mushroom and before Short Grass; neither raw
item appears in another ordinary tab.

Block updates publish states `20975/20958`; inventory paths use raw item IDs `277/278`.
Both emit zero light, maps use `NETHER/COLOR_CYAN`, note blocks read `HARP`, and both use
the five Fungus sounds. This family adds no packet field or connection-local state.

**Branches and aborts:**

Both identities; ordinary/forced placement; every direct/nested/reloaded support member;
any neighbor update; hand/tool/explosion/support-loss loot; matched/mismatched nylium;
above-height inside/outside; client/server target; success endpoint; missing/present/rebound
feature and every feature result; Composter/pot/chest/recipe/unlock; Hoglin food,
Strider food/tempting, raw/potted repellent and Enderman paths; four vegetation providers,
ordinary/bonemeal spread dimensions, both count-eight huge profiles, four replacement
predicates, 1,212-template census; persistence and client projection are distinct.

**Constants and randomness:**

States `20975/20958`; block IDs `876/867`; item IDs `277/278`; potted states
`21826/21827` and block IDs `919/920`; shape `(4,0,4)..(12,9,12)`; strength `0/0`;
emission `0`; sounds `1136..1140`; stack `64`; expanded supports `14`; bonemeal
chance `<0.4f`; Composter `0.65f`; Hoglin-Stable rolls `3..4`, equal entries `14`,
Crimson count `2..7`; Strider tempt priority/speed `3/1.4`; repellent scan
`8x4`, cadence `20`, pacification `200`; vegetation dimensions `8x4`/`3x1`,
weights `87:11:1` and `85:1:13:1`; huge wrapper count `8`; raw templates/cells
`1212/0/0`.

**Side effects:**

Placement, support-loss destruction/drop; Bone Meal consumption, particles and optional
configured-feature writes/destruction; loot and Composter mutation/events; pot
state/item/stat effects; chest inventory and recipe/reward changes; Hoglin/Strider
love, age, persistence, navigation and offspring paths; repellent memories/activity;
Enderman carried state/removal/placement; vegetation/huge generation; ordinary
persistence; map, sounds, cross model, sprite, name and tab projection.

**Gates:**

World-write/break authority; active block/loot/item/tag/worldgen snapshots; recursive
support membership; bonemeal base/build-height/server/success; configured-feature
lookup; explosion context; Composter level/draw; pot occupancy; chest selection;
recipe/criterion state; Hoglin/Strider age/love/pacification/goal state; Enderman
gamerule/ray/placement; vegetation origin/provider/candidate, placement modifier,
biome and huge replacement; registry and client-resource context.

**Boundary cases and quirks:**

- Both current support tags expand identically because Crimson delegates to Warped;
  reload can break that equality.
- Survival admits 14 current substrates, while bonemeal growth admits only the
  color-matched nylium and ignores the support tag.
- Any neighbor notification can pop a Fungus because support is rechecked without
  testing the changed direction.
- Valid server bonemeal consumes one item even when the `<0.4f` success draw fails.
- The target predicate does not validate the feature. A successful draw with a missing
  key consumes Bone Meal and does nothing; a rebound key may invoke another feature type.
- Growth has no small-Fungus remove/restore wrapper; planted huge-feature destruction
  and ignored writes are directly observable.
- Vegetation providers select the opposite-color Fungus at weight one in both forests.
- Warped has the Strider and repellent joins; Crimson has the Hoglin-food, chest and
  `warped_fungus_on_a_stick`-absence asymmetries.

**Failure semantics:**

Invalid ordinary placement is rejected. Shape-update failure returns AIR through the
generic update owner. Invalid bonemeal targets do not consume; valid server targets
consume after either success result. Missing configured feature is a no-op after a
successful draw, while present-feature mutations have no rollback and the result is
ignored. Failed loot/Composter/pot/chest/recipe/mob/tag admission commits only the
generic owner's stated effects. Vegetation admission/write results and huge-feature
failure semantics remain exactly as delegated. Client-resource failure affects
projection, not authoritative identity.

**Client/server authority split:**

The server owns identity, support, growth, loot, inventories, recipes, mob state,
world generation, persistence and emitted light. Clients run target prediction and
project authoritative state, particles, maps, sounds, models, sprites, names and tabs.

**Observability:**

Commands/state packets, support-tag reloads, neighbor traces, bonemeal item count and
RNG/feature traces, drops, inventories, Composter/pot events, recipes/advancements,
Hoglin/Strider goals and memories, carried blocks, feature output, maps, sounds,
particles, tabs and rendering expose the listed branches.

**Persistence and reload:**

Placed states persist only identity and have no block entity. Item stacks persist
ordinary components. Loot, block/item tags, recipes, advancement, configured/placed
features, biomes and structure templates are reload-selected at their owners. Direct
registrations, `NetherFungusBlock` control flow, stored key/base/tag identities,
Composter chances, mob consumers and tab order remain code-built. Reload does not
retroactively recheck existing support or resample generated vegetation.

**Evidence:**

`Confirmed`; `OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-DATA-001`;
`OFF-REPORT-001`. Anchors: `net.minecraft.world.level.block.Blocks`;
`NetherFungusBlock#getShape`, `#mayPlaceOn`, `#getFeature`,
`#isValidBonemealTarget`, `#isBonemealSuccess` and `#performBonemeal`;
`VegetationBlock#updateShape`, `#propagatesSkylightDown` and `#isPathfindable`;
`BoneMealItem#growCrop`; `ComposterBlock#bootStrap`; `NyliumBlock#performBonemeal`;
`NetherForestVegetationFeature#place`; `HugeFungusFeature#place`; `Hoglin#isFood`;
`Strider#isFood` and `#registerGoals`; `HoglinSpecificSensor#findNearestRepellent`;
`CreativeModeTabs`; both report/component/block-loot/support/tag/resource sets, four
potted/vegetation and four huge configurations, four placed wrappers, two biomes,
Hoglin-Stable table, Warped-Stick recipe/unlock and all 1,212 NBT templates.
Complete exact-ID and compiled-field-reference searches found no other acquisition,
use, generation or runtime path.

**Test vectors:**

Run `EXP-BLK-101` across both states and IDs; every direct/nested support and update
boundary; matched/mismatched nylium, height, chance and feature-reload outcome;
Composter, pot, chest, recipe, advancement, Hoglin, Strider and Enderman joins; all
four vegetation inputs, both count-eight huge profiles, four huge replacement
predicates and all 1,212 templates; persistence, maps, sounds, particles, tabs and
models. Assert exact constants, conditional draw/read/write order, absence claims
and client convergence.

**Limits:**

Generic placement/update/break/loot/explosion, Bone Meal, Composter and flower-pot
lifecycles, crafting/advancement, Animal breeding and TemptGoal state machines,
Hoglin Brain activity, Enderman goal scheduling, nether-vegetation/huge-feature
algorithms, placement modifiers, Bastion/container loot, packet encoding and rendering
remain with their named owners. This leaf fixes the two Nether Fungus identities,
their custom support/growth dispatch, exact joins, asymmetries, acquisition and
projection.
