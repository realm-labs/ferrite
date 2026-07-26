# Blocks mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `BLK-AGRICULTURAL-GROUND-001` — Dirt Path and Farmland couple reduced-height support to delayed decay, irrigation and trampling

**Parent:** `SIM-004`, `SIM-005`, `SIM-RANDOM-001`, `BLK-001`,
`BLK-STATE-001`, `BLK-002`, `BLK-PLACE-001`, `BLK-BREAK-001`,
`BLK-BREAK-HOOK-001`, `BLK-BREAK-CONTENT-001`, `BLK-UPDATE-001`,
`PLY-002`, `PLY-005`, `PLY-006`, `PLY-INPUT-001`,
`PLY-INTERACT-001`, `PLY-BREAK-001`, `PLY-COLLISION-001`,
`PLY-AUTOJUMP-001`, `RED-001`, `RED-UPDATE-001`,
`RED-COMPARATOR-001`, `ITM-003`, `ITM-004`, `ITM-006`,
`ITM-LOOT-001`, `ITM-ANVIL-001`, `ENT-001`, `MOB-001`,
`MOB-AI-001`, `ENV-001`, `ENV-002`, `ENV-003`, `ENV-FLUID-001`,
`ENV-FIRE-001`, `ENV-LIGHT-001`, `WGEN-003`,
`WGEN-PIPELINE-001`, `WGEN-JIGSAW-PROCESSORS-001`,
`WGEN-JIGSAW-VILLAGES-001`, `WGEN-STRUCTURE-WOODLAND-MANSION-001`,
`CLI-001`, `CLI-006`, `CLI-UI-001`, `CLI-EFFECT-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked registrations and source inspection close both
reduced-height implementations, including placement fallback, above-neighbor
scheduling, the stale-tick asymmetry, Farmland irrigation/drying/trampling,
tool and crop/Villager joins, loot, tags, legacy migration and client
projection. An exhaustive decoded scan of all 1,212 structure templates fixes
8,311 raw cells and 53 executable Dirt Path Jigsaw final states.

**Applies when:**

`minecraft:dirt_path` or `minecraft:farmland` is placed, updated,
scheduled/random-ticked, tilled, fallen on, mined, used by crops or Farmer
Villagers, generated, migrated, persisted, synchronized or rendered.

**Authoritative state:**

Neither block has a block entity. Their exact registrations are:

| Identity | Block ID | State IDs | Default | Item ID | Implementation | Map color | Strength/resistance | Sound |
| --- | ---: | --- | --- | ---: | --- | --- | --- | --- | --- |
| Dirt Path | `666` | `14815` | `14815` | `551` | `DirtPathBlock` | Dirt | `0.65/0.65` | Grass |
| Farmland | `208` | `5319..5326` for moisture `0..7` | `5319` (`0`) | `361` | `FarmlandBlock` | Dirt | `0.6/0.6` | Gravel |

Both expose the same fixed `16×15×16` outline/collision shape, use that shape
for light occlusion and reject every path-computation type. Registration
forces both view-blocking and suffocation predicates true despite the reduced
shape. They retain ordinary Harp instrument, friction
`0.6`, speed/jump factors `1`, light `0`, normal piston reaction, no redstone
signal or comparator output and no scheduled-tick loop. Dirt Path does not
random-tick; Farmland does.

Grass sound-event IDs for break/step/place/hit/fall are
`755/759/758/757/756`; Gravel uses `760/764/763/762/761`. Both profiles use
volume/pitch `1/1`.

Each item is a common stack-64 ordinary `BlockItem` with empty attribute,
enchantment and lore defaults. Neither item has a direct item tag or
Sulfur-Cube archetype.

**Transition and ordering:**

### Placement, support and delayed conversion

Dirt Path survives when the state above is not solid, or when the above
block is any `FenceGateBlock` implementation. Placement first tests the
default path state at the clicked position. Failure returns default Dirt
through `pushEntitiesUp(default path, default dirt, level, position)`;
success delegates to ordinary block placement.

Farmland survives when the state above is not solid, or that above state is
in the live `maintains_farmland` block tag. Placement tests default
moisture-0 Farmland; failure returns default Dirt directly, while success
delegates to ordinary block placement.

For either block, only an `UP`-direction shape update can schedule
invalidation. When the incoming state cannot survive, it schedules that block
at the current position with delay `1`, then returns the ordinary superclass
shape-update result. Other directions do not test support.

The scheduled callbacks deliberately differ:

- Dirt Path calls `FarmlandBlock.turnToDirt(null, state, level, position)`
  unconditionally. Restoring a legal above state before the queued tick does
  not save it.
- Farmland rechecks `canSurvive`; only a still-invalid state calls the same
  conversion helper.

The helper computes default Dirt through
`pushEntitiesUp(current, dirt, level, position)`, offers it through
`setBlockAndUpdate` and ignores the result. It then emits `BLOCK_CHANGE`
unconditionally with the supplied nullable entity and resulting Dirt state.

The baseline `maintains_farmland` tag directly contains Pumpkin Stem,
Attached Pumpkin Stem, Melon Stem, Attached Melon Stem, Beetroots, Carrots,
Potatoes, Torchflower Crop, Torchflower, Pitcher Crop, Wheat and Moving
Piston, plus the nested live `fence_gates` tag. Reloading either tag changes
subsequent survival and dry-retention checks. Dirt Path instead uses the Java
Fence-Gate class test and is not affected by those tag reloads.

### Moisture random ticks

Farmland reads its current `moisture` on each selected server random tick.
`isNearWater` scans every position from `(x-4,y,z-4)` through
`(x+4,y+1,z+4)`, a `9×2×9` volume of 162 fluid states, and succeeds on the
first member of the live `water` fluid tag. If none matches, rain is tested
at the cell immediately above through `ServerLevel.isRainingAt`.

When water or rain is present, any moisture below `7` is set directly to `7`
with flags `2`; state `7` causes no write. Without water or rain:

- moisture `1..7` is decremented by exactly one with flags `2`;
- moisture `0` remains Farmland when the above state belongs to
  `maintains_farmland`;
- moisture `0` without that tag calls `turnToDirt(null,...)`.

Thus fully wet Farmland needs seven admitted dry random ticks to reach `0`
and an eighth to become Dirt, unless a later hydration or retention branch
intervenes. All state-write results are ignored.

### Trampling

`fallOn` attempts conversion only on a server level. It consumes one level
`nextFloat` before testing entity class, game rule or dimensions, and
continues only when that draw is strictly below `fallDistance-0.5`.
The entity must be living. Players bypass the game-rule test; every other
living entity requires `mobGriefing=true`. Finally,
`boundingBoxWidth² × boundingBoxHeight` must be strictly greater than
`0.512`.

An admitted entity is passed to `turnToDirt`, so the block-change event
attributes the conversion to it. The superclass `fallOn` callback always
runs afterward, whether conversion occurred or its low-level write
succeeded; ordinary fall damage is therefore not replaced by trampling.

### Tools

The Hoe map contains exact Dirt Path with the same `onlyIfAirAbove`
predicate as Grass Block and Dirt: the clicked face must not be `DOWN` and
the cell above must be air. Admission plays Hoe Till, predicts `SUCCESS`,
then the server offers default moisture-0 Farmland with flags `11`, emits
`BLOCK_CHANGE` and damages a non-null player's Hoe by one. The write result
is ignored.

Farmland itself is absent from both `HoeItem.TILLABLES` and
`ShovelItem.FLATTENABLES`; either tool returns `PASS` unless another generic
tool branch applies. Dirt Path is likewise absent from the Shovel map. The
six substrate-to-Path and substrate-to-Farmland inputs remain owned by
`BLK-DIRT-SUBSTRATE-001`.

**Breaking, fire and fuel:**

Both blocks are directly `mineable/shovel`. Neither requires a correct tool,
so any tool or hand is harvest-eligible. Each loot table contains one
explosion-surviving count-one Dirt entry, with random sequence
`minecraft:blocks/dirt_path` or `minecraft:blocks/farmland`. Silk Touch and
Fortune do not recover or alter the matching block item.

Neither identity appears in `FireBlock.bootStrap`,
`FuelValues.vanillaBurnTimes` or `ComposterBlock.bootStrap`; direct fire
encouragement/flammability are `0/0`, burn time is `0`, compost chance is
absent and neither registration sets `ignitedByLava`.

**Crop, vegetation and Villager joins:**

Farmland is the sole direct member of both `supports_crops` and
`grows_crops`. Crops therefore use it for survival, and the generic 3×3
growth-speed scan assigns each matching cell contribution `1` at moisture
`0` or `3` at positive moisture before quartering off-center values. Those
growth algorithms and each crop's age transaction retain their crop leaves.

Farmland is also a direct member of `supports_vegetation`,
`supports_big_dripleaf` and `support_override_cactus_flower`. It supports
those generic consumers without becoming a member of `substrate_overworld`.
Dirt Path has no direct tag beyond `mineable/shovel`.

The Farmer Villager profession installs exact Farmland as its sole secondary
POI block. `SecondaryPoiSensor` runs at scan rate `40`, scans offsets
`x,z=-4..4` and `y=-2..2`, and stores every exact Farmland position as a
same-dimension `GlobalPos` list in `SECONDARY_JOB_SITE`; an empty result
erases that memory.

`HarvestFarmland` requires that memory, Farmer profession and
`mobGriefing=true`. Its 3×3×3 work search accepts either a mature
`CropBlock`, or air whose block immediately below is any `FarmlandBlock`
implementation. Harvesting, seed choice, planting, sounds, inventory
consumption and work timing retain `MOB-AI-001` and the crop leaves; this
leaf fixes the exact profession-memory and support identity.

Neither item is an ingredient or result in the locked recipe set, a direct
advancement criterion/reward, a merchant offer or a non-block loot output.
Baseline acquisition of the matching items is therefore creative/command or
generic stack transfer; mining either block yields Dirt instead.

**World sources:**

The generic `BlockPileFeature` has one exact Dirt Path support exception:
when the block below a candidate is Dirt Path it consumes and returns
`nextBoolean`; every other support uses the upward face-sturdy test. The
feature's footprint, radial gates, provider and writes remain owned by
`WGEN-PIPELINE-001`.

Three ordered Village street processor lists select Dirt Path:

- `street_plains` replaces it over Water with Oak Planks, otherwise a
  random-block-match of `0.1` replaces it with non-snowy Grass Block;
- `street_savanna` uses Acacia Planks over Water, then probability `0.2`;
- `street_snowy_or_taiga` uses Spruce Planks over Water or Ice, then
  probability `0.2`.

The Rule Processor, first-match ordering, template transforms and failed
writes retain `WGEN-JIGSAW-PROCESSORS-001`.

An exhaustive decoded scan of all 1,212 bundled templates finds:

| Identity | Files with raw cells | Raw cells | Root groups |
| --- | ---: | ---: | --- |
| Dirt Path | `244` | `7,474` | Village `244/7,474` |
| Farmland | `31` | `837` | Village `29/773`; Woodland Mansion `2/64` |

Farmland cells split as moisture `0/6/7` counts `32/1/804`; no other moisture
appears. No raw target cell has block NBT. A decompressed-string census finds
297 exact Dirt Path strings: 244 palette entries plus 53 Jigsaw
`final_state=minecraft:dirt_path` values, all in Village templates. Farmland
has 41 exact strings, exactly its 41 moisture-distinct palette entries and no
extra final-state/reference string. The 53 connector values are executable
replacement sources, not item stacks.

**Persistence migration:**

Current saves and block-state packets preserve Dirt Path identity or
Farmland moisture exactly. Legacy numeric block-state IDs `960..967` map to
Farmland moisture `0..7`; legacy item ID `60` maps to Farmland. Legacy block
state `3328` and item ID `208` first decode as `minecraft:grass_path`; data
fix schema `2680` then renames both the block and item identity to
`minecraft:dirt_path`. Generic schema/version dispatch retains its data-fix
owner.

**Client projection:**

Dirt Path's sole blockstate chooses uniformly among Y rotations
`0/90/180/270` of one `16×15×16` model. That model uses Dirt below and for
particles, plus dedicated Dirt-Path top and side textures.

Farmland moisture `0..6` all select the same dry `farmland` model; only
moisture `7` selects `farmland_moist`. Both are fixed
`16×15×16` template-Farmland models with Dirt side/bottom/particles and the
selected top texture. Neither block or item has tint.

Each item definition directly selects its corresponding block model; the
Farmland item therefore uses the dry model. The four dedicated textures are
static 16×16 PNGs without animation metadata. Names are `Dirt Path` and
`Farmland`.

The Natural Blocks tab places Dirt Path immediately after Mycelium and
Farmland immediately after Rooted Dirt in the run Grass Block, Podzol,
Mycelium, Dirt Path, Dirt, Coarse Dirt, Rooted Dirt, Farmland, Mud.

**Branches and aborts:**

- Placement can return Dirt before any matching reduced-height state is
  written.
- Only an above-neighbor update schedules support loss; Dirt Path never
  rechecks at its queued tick, while Farmland always does.
- Water scanning stops at the first live-tag fluid; rain is queried only
  after all 162 misses.
- Hydration jumps directly to `7`; drying takes one state step per selected
  random tick and tag-based retention is read only at `0`.
- Trampling stops at server, draw, living-entity, game-rule and strict-volume
  gates in that order.
- Hoe conversion requires non-`DOWN` and air above; using Hoe or Shovel on
  Farmland, or Shovel on Dirt Path, has no map entry.
- Loot explosions can suppress the sole Dirt output.
- Village processors and connectors remain conditional on selected
  structures, ordered rules, transforms, clipping and successful writes.

**Constants and randomness:**

Shape `16×15×16`; moisture `0..7`; water radius `4`, Y span `0..1`, 162
fluid reads; scheduled delay `1`; trampling draw threshold
`fallDistance-0.5`, strict entity-volume threshold `0.512`; Hoe cost `1`;
secondary-POI scan rate `40`, offsets `9×5×9`; Block-Pile Dirt-Path support
chance `1/2`; street degradation `0.1/0.2/0.2`; raw cells `7,474/837`;
Dirt-Path final states `53`.

**Side effects:**

Placement fallback and entity displacement; delayed and random-tick Dirt
conversion; moisture writes; fall conversion plus ordinary fall handling;
Hoe sound/event/durability; Dirt loot; crop support/growth; Farmer memory,
navigation, harvest and planting; feature/template/processor writes; legacy
migration; client models, textures and sounds.

**Gates:**

Above solidity/class/live retention tags; direction and queued tick;
random-tick selection; live Water membership and rain; moisture; server,
trampling RNG, entity class/game rule/volume; tool map/face/air/player;
explosion; crop/vegetation tags; Villager profession/memory/game rule;
feature/template/processor selection; data version; resource/tag reload.

**Boundary cases and quirks:**

Dirt Path's queued tick is intentionally stale: it converts even after its
support becomes legal. Farmland's equivalent tick rechecks. Fence Gates are
accepted by Dirt Path through a class test but by Farmland through nested
live tag membership. A dry moisture-0 crop can retain Farmland without
hydrating it. Positive moisture affects crop speed identically at `1` and
`7`. The trampling random draw is consumed even for a nonliving entity or a
mob later rejected by `mobGriefing`.

**Failure semantics:**

Every direct state write ignores its Boolean result and has no rollback.
`turnToDirt` emits `BLOCK_CHANGE` after a rejected write. Hoe sound,
predicted success, event and durability can likewise survive a failed write.
Farmland's superclass fall handling runs after any conversion outcome.
Worldgen and Jigsaw owners retain earlier writes after later failures.

**Client/server authority split:**

The server owns scheduled/random ticks, irrigation, trampling, tool mutation,
loot, crop/Villager decisions, generation and data migration. Clients return
ordinary predicted Hoe success and render synchronized identity/moisture,
models, textures and sounds.

**Observability:**

Observe exact state/registry IDs, 15/16 geometry, support reads, scheduled
ticks, hydration scan/order, moisture transitions, trampling draw/gates,
conversion event/write order, tool/loot/tag/crop/Villager outcomes, processor
and template census, legacy inputs, persisted/wire states, models, rotations,
textures, names and tab order.

**Persistence and reload:**

Dirt Path persists identity only; Farmland persists moisture. Neither has
block-entity data. Stacks use generic components. Block/fluid tags, loot,
worldgen and client resources retain independent reload boundaries.
Registrations, physical profiles, tool maps and data-fix registrations are
code-built.

**Evidence:**

`net.minecraft.world.level.block.Blocks`;
`net.minecraft.world.level.block.DirtPathBlock`;
`net.minecraft.world.level.block.FarmlandBlock#randomTick`;
`net.minecraft.world.level.block.FarmlandBlock#fallOn`;
`net.minecraft.world.level.block.FarmlandBlock#turnToDirt`;
`net.minecraft.world.item.HoeItem#useOn`;
`net.minecraft.world.item.ShovelItem#useOn`;
`net.minecraft.world.level.levelgen.feature.BlockPileFeature`;
`net.minecraft.world.entity.npc.villager.VillagerProfession`;
`net.minecraft.world.entity.ai.sensing.SecondaryPoiSensor#doTick`;
`net.minecraft.world.entity.ai.behavior.HarvestFarmland`;
`net.minecraft.world.level.block.CropBlock#getGrowthSpeed`;
`net.minecraft.world.level.block.FireBlock#bootStrap`;
`net.minecraft.world.level.block.ComposterBlock#bootStrap`;
`net.minecraft.world.level.block.entity.FuelValues#vanillaBurnTimes`;
`net.minecraft.util.datafix.fixes.BlockStateData`;
`net.minecraft.util.datafix.DataFixers`;
`net.minecraft.world.item.CreativeModeTabs#bootstrap(net.minecraft.core.Registry)`;
block/item/sound reports and item-component reports; all direct/composed
block/fluid tags; both loot tables; all three street processor lists; all
1,212 decoded structures; blockstates, models, item definitions, textures
and language resources. Complete compiled exact-field, data and decoded-NBT
searches found no other identity-specific runtime path.

**Test vectors:**

Run `EXP-BLK-114` across both identities, every state, above
solid/Fence-Gate/retention-tag combination, queued-tick race, all 162
irrigation cells, rain and moisture transitions, trampling gate/draw/write,
Hoe and loot branch, crop/vegetation/Villager/Block-Pile consumer, all three
street processors, 8,311 raw cells and 53 Jigsaw final states, legacy
migration, persistence/reload and client projection. Assert IDs, order,
constants, absences, census and vanilla convergence.

**Limits:**

Generic placement/break, scheduled/random-tick selection, entity collision
and fall damage, Hoe use, loot, crop growth, plant survival, Villager brain,
feature/processor/Jigsaw algorithms, data-fix dispatch and rendering remain
with their named owners. Dirt, crops, vegetation, Villagers and structures
retain their catalog families. This leaf fixes both reduced-height ground
identities and every exact join that selects them.
