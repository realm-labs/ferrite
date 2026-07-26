# Blocks mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `BLK-MOSS-001` — Moss Block launches vegetation patches while Moss Carpet follows thin support

**Parent:** `SIM-004`, `SIM-005`, `SIM-RANDOM-001`, `BLK-001`,
`BLK-STATE-001`, `BLK-002`, `BLK-PLACE-001`, `BLK-BREAK-001`,
`BLK-BREAK-HOOK-001`, `BLK-BREAK-CONTENT-001`, `BLK-UPDATE-001`,
`PLY-002`, `PLY-005`, `PLY-006`, `PLY-INPUT-001`,
`PLY-INTERACT-001`, `PLY-BREAK-001`, `PLY-COLLISION-001`,
`PLY-AUTOJUMP-001`, `RED-001`, `RED-UPDATE-001`,
`RED-COMPARATOR-001`, `ITM-003`, `ITM-004`, `ITM-006`,
`ITM-USE-001`, `ITM-RECIPE-001`, `ITM-CRAFT-001`,
`ITM-LOOT-001`, `ITM-ADVANCEMENT-001`, `ITM-ANVIL-001`,
`ITM-DISPENSER-001`, `ENT-001`, `ENT-KNOCKBACK-001`,
`MOB-001`, `MOB-AI-001`, `ENV-001`, `ENV-002`, `ENV-003`,
`ENV-FLUID-001`, `ENV-FIRE-001`, `ENV-LIGHT-001`,
`WGEN-003`, `WGEN-PIPELINE-001`,
`WGEN-JIGSAW-TRIAL-CHAMBERS-001`, `CLI-001`, `CLI-006`,
`CLI-UI-001`, `CLI-EFFECT-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked registrations, source, data and client inspection
close both property-free blocks and items: full-cube versus one-model-unit
geometry, Carpet support loss, Bone-Meal feature dispatch, loot/crafting/
compost/trade/chest/archetype joins, the complete 38-tag Moss-Block closure,
worldgen selection, five raw template cells, persistence and projection.

**Applies when:**

`minecraft:moss_block` or `minecraft:moss_carpet` is placed, supported,
updated, path-tested, stepped on, mined, exploded, composted, crafted,
traded, selected by a mob or world generator, persisted, synchronized or
rendered; it also fixes Bone Meal use on exact Moss Block.

**Authoritative state:**

Neither block has properties or a block entity.

| Identity | Block ID/state ID | Item ID | Implementation/type | Strength/resistance | Map/sound |
| --- | --- | ---: | --- | --- | --- |
| Moss Block | `1144/30355` | `290` | `BonemealableFeaturePlacerBlock`; `minecraft:bonemealable_feature_placer`, key `minecraft:moss_patch_bonemeal` | `0.1/0.1` | Green/Moss |
| Moss Carpet | `1140/30306` | `289` | `CarpetBlock`; `minecraft:carpet` | `0.1/0.1` | Green/Moss Carpet |

Both item forms are stack-64 `BlockItem`s with common rarity, empty
attributes/enchantments/lore and ordinary generic components. Both blocks
have Harp instrument, friction `0.6`, speed/jump factors `1`, light `0`,
no signal or comparator output and piston reaction `DESTROY`. Neither
requires a correct tool for loot.

Moss break/fall/hit/place/step sound-event IDs are
`1000/1001/1002/1003/1004`; Moss Carpet uses
`990/991/992/993/994`. Both profiles have volume/pitch `1/1`.

**Transition and ordering:**

### Shape, placement and support

Moss Block is an ordinary full collision, outline, visual and support cube.
It is not pathfindable for land, water or air and uses the default opaque
full-block conductor/support behavior.

Moss Carpet returns the full-X/Z column from Y `0` through `1/16` for its
outline, collision and support shape. Because that collision is not a full
cube, inherited land and air pathfinding return true; water pathfinding is
false because it has no Water fluid.

Carpet survival rejects exactly when `isEmptyBlock(position.below())` is
true. This is an Air-identity test, not a sturdy-face or collision test:
any nonair block, including another thin/noncolliding state, supports the
Carpet. Fresh placement must pass that predicate. Every shape update
rechecks it; failure immediately returns Air without loot or a scheduled
tick, while success returns the superclass update unchanged.

Both ordinary item placements write their sole default state and consume
one item after the generic placement transaction succeeds. Rotation and
mirror are identity operations. Moss Block has no neighbor, scheduled or
random-tick transition.

### Bone Meal and vegetation-patch dispatch

Exact Moss Block is a valid Bone-Meal target precisely when the state above
reports Air. Success is then unconditional and consumes no success draw.
The generic server use calls `performBonemeal`, then consumes one Bone Meal,
emits the finished-interaction game event and level event `1505`, and
returns success; client target validation predicts success without
mutation. Its `NEIGHBOR_SPREADER` type projects the growth particles around
the position above the clicked block.

Execution obtains the live configured-feature registry, then the exact
`moss_patch_bonemeal` key. If both are present it calls that feature at
`origin.above()` with the server chunk generator and the supplied RNG.
Registry/key absence skips the call; the feature's Boolean result is
ignored. No Moss-owned draw or write precedes the feature, so an absent,
failed or fully rejected feature still follows the generic admitted
Bone-Meal consumption/event path.

The keyed floor vegetation-patch record fixes:

- default Moss Block ground provider and live `moss_replaceable`;
- depth `1`, extra-bottom chance `0`, vertical range `5`;
- independent uniform X/Z radii `1..2` and extra-edge chance `0.75`;
- vegetation chance `0.6` and child `moss_vegetation`.

The child is a weighted simple-block provider of total weight `96`:
Flowering Azalea `4`, Azalea `7`, default Moss Carpet `25`, Short Grass
`50`, and lower-half Tall Grass `10`. Patch scanning, independent draws,
ground writes with flags `2`, child placement and return aggregation retain
`WGEN-PIPELINE-001`.

### Mining, crafting, composting and acquisition

Both blocks are direct `mineable/hoe` members, which changes suitable Hoe
mining speed but is not a loot gate. Their one-pool block loot tables each
yield one matching item when `survives_explosion` passes, with no
Silk-Touch, Fortune, entity or tool condition. Consequently hand mining can
drop the item, while Carpet support loss returns Air without consulting its
loot table.

Three recipes consume Moss Block:

- one horizontal pair produces three Moss Carpets, group `carpet`;
- shapeless exact Cobblestone plus Moss Block produces one Mossy
  Cobblestone;
- shapeless exact Stone Bricks plus Moss Block produces one Mossy Stone
  Bricks.

Each recipe advancement accepts either possession of exact Moss Block or
existing recipe knowledge in one OR requirement group and rewards its one
recipe. No recipe produces Moss Block, and no recipe consumes Moss Carpet.

The hard-coded Composter map gives Moss Carpet chance `0.3` and Moss Block
chance `0.65`. Player and automation insertion, first-level certainty,
random draw, level/event/consumption order and full-Composter extraction
retain the generic Composter owner. Neither item is fuel; neither block is
registered in `FireBlock.bootStrap`, so direct fire encouragement/
flammability are `0/0`.

Moss Block has exactly two non-block loot sources:

| Table/pool | Rolls | Selection weight | Count |
| --- | --- | --- | --- |
| `chests/shipwreck_supply`, first pool | uniform `3..10` | `7/84 = 1/12` | uniform `1..4` |
| `chests/trial_chambers/supply` | uniform `3..5` | `1/18` | uniform `2..5` |

The baseline Wandering Trader common tag includes
`emerald_moss_block`: one Emerald buys two Moss Blocks, maximum uses `5`,
inherited XP `1`, reputation discount `0.05`, and no second cost or
modifier. The common set chooses five distinct offers from `76` with random
sequence `minecraft:trade_set/wandering_trader/common`. Moss Carpet has no
loot or merchant source beyond its own block loot and recipe.

### Tags, mobs and cross-system joins

Moss Block's five direct block tags are `mineable/hoe`, `moss_blocks`,
`sniffer_egg_hatch_boost`, `supports_big_dripleaf` and
`supports_small_dripleaf`. The complete baseline composition closure is
exactly `38` tags. In addition to those five it gains:

- `cannot_replace_below_tree_trunk`, `enderman_holdable`,
  `moss_replaceable`, `sniffer_diggable_block` and
  `substrate_overworld`;
- `azalea_grows_on`, `azalea_root_replaceable`,
  `beneath_bamboo_podzol_replaceable`,
  `beneath_tree_podzol_replaceable`, `forest_rock_can_place_on`,
  both Huge-Mushroom support tags, `ice_spike_replaceable`,
  `lush_ground_replaceable`, both carver-replaceable tags,
  `sculk_replaceable` and `sculk_replaceable_world_gen`;
- `supports_azalea`, `supports_bamboo`, `supports_crimson_fungus`,
  `supports_crimson_roots`, `supports_dry_vegetation`,
  `supports_mangrove_propagule`, both stem-fruit identities plus their
  shared tag, `supports_nether_sprouts`, `supports_sugar_cane`,
  `supports_vegetation`, `supports_warped_fungus`,
  `supports_warped_roots` and `supports_wither_rose`.

Reload changes subsequent membership tests. These joins admit the default
Moss state to the generic Enderman take/carry/place path, Sniffer dig
substrate check, plant and fungus support paths, tree below-provider
exception, and feature/carver/Sculk replacement predicates. Those
consumers' remaining AI, environment, draw and write gates retain their
named owners.

Moss Block is the sole baseline `sniffer_egg_hatch_boost` member. Whenever
a Sniffer Egg's `onPlace` runs, the live state below selects the interval
for that hatch stage. Moss admission emits server level event `3009` and
schedules after `12000/3 + nextInt(300)`, hence `4000..4299` ticks; the
nonboosted branch uses `24000/3 + nextInt(300)`, hence `8000..8299`.
Later egg-state replacements re-enter this test, so changing the substrate
can affect the next stage but does not rewrite an already queued delay.
Crack/hatch state, sounds, entity creation and scheduling ownership remain
with the Sniffer Egg subtype.

Moss Carpet's exact four-tag closure is `mineable/hoe`,
`combination_step_sound_blocks`, `mangrove_logs_can_grow_through` and
`mangrove_roots_can_grow_through`. The latter two admit feature traversal.
The step tag makes a walking player inside the Carpet use it as the primary
step block: outside water handling, sound `994` plays at volume `0.15` and
pitch `1`, then the support's step sound plays at
`supportVolume*0.05` and `supportPitch*0.8`.

The Moss Block item directly belongs to item `moss_blocks`, which composes
into `sulfur_cube_archetype/fast_flat` and
`sulfur_cube_swallowable`; Moss Carpet has no item tag. An accepting adult
Sulfur Cube can therefore install one Moss Block in empty BODY equipment.
The archetype fixes horizontal/vertical knockback `0.9125/0.09`; additive
knockback and explosion-knockback resistance `-1/-1`; additive bounciness
`0.5`; total-multiplied friction and air drag
`-0.7999999970197678/-0.9900000002235174`; hit/push sounds `1945/1946`,
push cooldown `0.9` and impulse threshold `0.03`. Admission, pickup/
dispenser equipment mutation and contact math retain `ENT-KNOCKBACK-001`.

### World generation and structure census

The other direct Moss records retain `WGEN-PIPELINE-001` algorithms:

- floor `moss_patch` uses Moss Block ground, `moss_replaceable`, depth `1`,
  extra-bottom `0`, vertical range `5`, independent X/Z `4..7`,
  edge chance `0.3`, vegetation chance `0.8` and `moss_vegetation`;
- ceiling `moss_patch_ceiling` uses the same ground/tag/range/radii/edge
  values, uniform depth `1..2`, vegetation chance `0.08` and child
  `cave_vine_in_moss`;
- ordinary and tall Mangrove records use Moss Carpet as the above-root
  provider at strict chance `0.5`, while both grow-through tags also admit
  an existing Carpet;
- all four planted/unplanted Crimson/Warped Huge-Fungus configurations
  include Moss Carpet in their explicit replaceable-block predicate.

An exhaustive decoded scan of all `1,212` bundled templates finds exactly
five raw cells:

- `trial_chambers/spawner/small_melee/slime` has four Moss Blocks at
  `[0,0,0]`, `[0,0,2]`, `[2,0,0]`, `[2,0,2]`; the structure's
  small-melee alias chooses this target at conditional weight `1/4`, and
  its rigid one-element pool then selects the template;
- `trial_chambers/corridor/addon/display_2` has one Moss Carpet at
  `[2,3,3]`; it is one of three equal-weight rigid entrance elements.

None of the five cells has block NBT. Exact decompressed-string scans find
only those two palette strings: there is no extra Jigsaw `final_state`,
processor payload or entity-data occurrence. Reachability, pool aliasing,
rotation, attachment/collision, clipping and write failure remain with
`WGEN-JIGSAW-TRIAL-CHAMBERS-001`.

### Persistence and client projection

Current block persistence and terrain packets preserve identity only.
Stacks preserve generic components. Neither identity has a pre-flattening
numeric state/item mapping, old alias or identity-specific data-fix path.

Each property-free blockstate selects one model. Moss Block uses opaque
`cube_all`; Moss Carpet uses the standard one-model-unit-high Carpet parent.
Both block and item definitions ultimately sample the single static
16×16 `textures/block/moss_block.png`; there is no separate Moss-Carpet
texture or tint. Item definitions directly select the matching block model.
Names are `Moss Block` and `Moss Carpet`.

Natural Blocks orders Snow Block, Snow, Moss Block, Moss Carpet, Pale Moss
Block, Pale Moss Carpet, Pale Hanging Moss, then Stone. Neither item appears
in another baseline creative tab.

**Branches and aborts:**

- Carpet placement/support rejects only Air below; a failed shape update
  becomes Air immediately.
- Bone Meal rejects nonair above. After admission, lookup absence and
  feature failure do not undo generic consumption/events.
- Block loot has no tool gate but explosion survival can reject.
- Sniffer Egg boost reads the live substrate when each stage is scheduled,
  not continuously while the delay runs.
- Moss Carpet is not itself replaceable, but named feature predicates may
  still overwrite it.
- Worldgen/template selection preserves every parent draw, placement and
  clipping abort.

**Constants and randomness:**

Strength/resistance `0.1/0.1`; Carpet height `1/16`; bonemeal patch depth
`1`, range `5`, X/Z `1..2`, edge `0.75`, vegetation `0.6`; child weights
`4/7/25/50/10` of `96`; compost `0.65/0.3`; egg delays
`4000..4299/8000..8299`; trader five of `76`; chest weights/counts as
tabulated; Mangrove chance `0.5`; raw cells `4/1`.

**Side effects:**

Block/item writes and consumption; Carpet removal; Bone-Meal feature writes,
particles and game/level events; loot, crafting, advancement and chest
stacks; Composter state; merchant offers; Enderman/Sniffer/Sulfur-Cube
selection; feature, tree, fungus and structure writes; client models,
textures, names and tabs.

**Gates:**

Air/nonair support; placement and write result; side; live block/item tags;
Bone-Meal target and configured-feature lookup; Hoe/tool speed and explosion
survival; recipe/advancement ingredients; Composter level/draw; loot-table
draw; trader set selection; mob/griefing/path/explored/equipment gates;
Sniffer Egg substrate and queued stage; feature/provider/predicate/draw;
Jigsaw reachability/rotation/attachment/clipping; data/resource reload.

**Boundary cases and quirks:**

Moss Carpet needs no sturdy support face and may form a vertical column
because another Carpet is nonair. Support loss bypasses otherwise valid
self loot. Bone Meal can be consumed and emit its success effects even when
the configured key is missing or every feature write fails. Moss Block is
simultaneously its patch's ground output and a member of the live
replacement tag. Sniffer Egg acceleration is re-evaluated between hatch
stages, not retroactively within a queued stage. Moss Carpet has no distinct
texture.

**Failure semantics:**

Carpet invalidation returns Air without loot or rollback. Bone-Meal
execution ignores configured-feature absence/result after generic admission.
Feature, tree and template writes retain their owners' partial-commit rules.
Loot, recipe, Composter, merchant and Sulfur-Cube transactions retain their
generic atomicity/remainder behavior.

**Client/server authority split:**

The server owns support validation, Bone-Meal execution/consumption,
loot/crafting/progression, Composter/trade/mob selection, Sniffer Egg
scheduling, generation and persistence. The client predicts item placement
and Bone-Meal target success, plays/render synchronized step and block
sounds, and renders states, models, texture, names and tab contents.

**Observability:**

Observe registry/state/item/sound IDs, shapes and path types, nonair support
and update loss, Bone-Meal interaction/result/event plus every feature
read/draw/write, loot/recipe/advancement/compost/chest/trade outputs, complete
block/item tag closure and mob effects, exact five-cell structure census,
persisted/wire identity and client projection.

**Persistence and reload:**

Both blocks persist identity only and have no block entity. Stacks use
generic components. Tags, loot, recipes, advancements, trades, worldgen and
client resources retain independent reload boundaries. Registrations,
Carpet support/shape, Composter probabilities, Bone-Meal dispatch and
creative ordering are code-built.

**Evidence:**

`net.minecraft.world.level.block.Blocks`;
`net.minecraft.world.level.block.CarpetBlock`;
`net.minecraft.world.level.block.BonemealableFeaturePlacerBlock`;
`net.minecraft.world.item.BoneMealItem`;
`net.minecraft.world.level.block.ComposterBlock`;
`net.minecraft.world.level.block.FireBlock`;
`net.minecraft.world.level.block.SnifferEggBlock`;
`net.minecraft.world.entity.animal.sniffer.Sniffer`;
`net.minecraft.world.entity.monster.EnderMan$EndermanTakeBlockGoal`;
`net.minecraft.world.entity.SulfurCubeArchetypes`;
`net.minecraft.world.item.CreativeModeTabs`; block/item/sound/component
reports; complete block/item tag closure; both block loot tables, three
recipes and recipe advancements, two chest tables, Wandering-Trader record/
tag/set and `fast_flat`; all direct/composed worldgen records; all `1,212`
decoded structures and decompressed strings; blockstates, models, item
definitions, texture and language resources. Complete compiled exact-field,
data and decoded-NBT searches found no other identity-specific runtime path.

**Test vectors:**

Run `EXP-BLK-116` across both blocks, every support/shape/path/placement/
tool/explosion/Bone-Meal/registry/feature branch, complete tag closure,
Composter boundary, recipes/advancements/chests/trade/archetype/mob joins,
every worldgen selector, all five raw cells, persistence/reload and exact
client projection. Assert IDs, order, constants, absences and vanilla
convergence.

**Limits:**

Generic placement/break, pathfinding, Bone-Meal item use, loot, recipes/
progression, Composter, merchant, Enderman/Sniffer/Sulfur-Cube AI,
vegetation/tree/fungus/Jigsaw generation and rendering retain their named
owners. Pale Moss, Mossy masonry, plants, Sniffer Egg and mobs retain their
catalog families. This leaf fixes the two Moss identities and every exact
join that selects them.
