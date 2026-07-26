# Blocks mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `BLK-PACKED-ICE-001` — Packed Ice preserves cold terrain and enables sliding equipment

**Parent:** `SIM-004`, `SIM-005`, `SIM-RANDOM-001`, `BLK-001`,
`BLK-STATE-001`, `BLK-002`, `BLK-PLACE-001`, `BLK-BREAK-001`,
`BLK-BREAK-HOOK-001`, `BLK-BREAK-CONTENT-001`, `BLK-UPDATE-001`,
`BLK-SNOW-FAMILY-001`, `PLY-002`, `PLY-005`, `PLY-006`,
`PLY-INPUT-001`, `PLY-INTERACT-001`, `PLY-BREAK-001`, `PLY-MOVE-001`,
`PLY-COLLISION-001`, `PLY-AUTOJUMP-001`, `RED-001`,
`RED-UPDATE-001`, `RED-COMPARATOR-001`, `ITM-003`, `ITM-004`,
`ITM-006`, `ITM-USE-001`, `ITM-RECIPE-001`, `ITM-CRAFT-001`,
`ITM-LOOT-001`, `ITM-ADVANCEMENT-001`, `ITM-ANVIL-001`,
`ITM-DISPENSER-001`, `ENT-001`, `ENT-VEHICLE-001`,
`ENT-KNOCKBACK-001`, `MOB-001`, `MOB-AI-001`, `MOB-SPAWN-001`,
`ENV-001`, `ENV-002`, `ENV-003`, `ENV-FLUID-001`, `ENV-FIRE-001`,
`ENV-LIGHT-001`, `WGEN-003`, `WGEN-PIPELINE-001`,
`WGEN-JIGSAW-ANCIENT-CITY-001`, `WGEN-JIGSAW-TRIAL-CHAMBERS-001`,
`WGEN-JIGSAW-VILLAGES-001`, `WGEN-STRUCTURE-OCEAN-RUIN-001`,
`WGEN-STRUCTURE-OCEAN-MONUMENT-001`, `CLI-001`, `CLI-006`,
`CLI-UI-001`, `CLI-EFFECT-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked registration and implementation bytecode,
reports, complete loot/recipe/advancement/trade/tag data, all worldgen
records, every compiled exact-identity reference, all `1,212` decoded
structure templates and exact client resources close this property-free
block and item. Its identity-specific runtime is Silk-only self loot,
generic ice friction, Snow/Goat/geode/carver/ocean selectors, fast-sliding
Sulfur-Cube equipment, exact Frozen-Peaks/Frozen-Ocean and feature
generation, 268 raw template cells and two executable Jigsaw final states.

**Applies when:**

`minecraft:packed_ice` is placed, moved over, used as support, mined,
exploded, crafted, traded, selected by live block or item tags, generated,
read by a structure, persisted, migrated, synchronized or rendered.

**Authoritative state:**

Packed Ice is an ordinary property-free `Block` with no block entity,
subclass callback or scheduled/random tick. Its sole block-state ID is
`12914`; its block protocol ID is `556`; its ordinary block-item raw ID is
`550`.

Registration selects map color `ICE`, note instrument `CHIME`, friction
`0.98`, hardness/resistance `0.5/0.5`, Glass sounds, emission `0`,
speed/jump factors `1`, light dampening `15`, normal piston reaction and no
correct-tool requirement. Outline, collision, visual, support and occlusion
shapes are full unit cubes. Every face is sturdy; shade brightness is
`0.2`; the state is a redstone conductor, view blocker, suffocation state
and ordinary valid-spawn support.

It has no placement-state, survival, shape-update, neighbor, use, attack,
entity-contact, fall, signal, comparator, fluid, block-event or destruction
override. Land, water and air pathfinding return false through the ordinary
full-solid path. Rotation and mirror preserve its sole state.

The Glass sound profile has volume/pitch `1/1` and event IDs break `720`,
fall `721`, hit `722`, place `723` and step `724`. The common stack-64
`BlockItem` has ordinary generic components.

**Transition and ordering:**

### Placement, friction and self loot

Ordinary item placement, component placement and command writes select
state `12914`; there is no support predicate. Packed Ice does not use
`IceBlock`: it has no random melt path and non-Silk removal never creates
Water.

Generic grounded living movement samples block friction `F=0.98`.
Acceleration uses the high-friction ground branch and final horizontal
damping multiplies X/Z velocity by `F*0.91 = 0.8918`; exact movement,
attribute and input order remains with `PLY-MOVE-001`. Other entities use
their generic friction readers. A boat in land status averages the
friction of intersected supporting states. Its player-controlled result is
halved, then both horizontal velocity and rotation delta are multiplied by
that value as specified by `ENT-VEHICLE-001`.

The direct `mineable/pickaxe` membership enables Pickaxe mining speed but
does not establish loot correctness. The one-roll block table emits one
Packed Ice only when the tool's enchantments contain Silk Touch at level
at least `1`; its random sequence is `minecraft:blocks/packed_ice`.
There is no `survives_explosion`, Fortune or correct-tool condition.
Consequently any Silk-enchanted tool can emit the item, while hand,
non-Silk Pickaxes, other non-Silk tools and tool-less explosion contexts
emit nothing.

Packed Ice has no `FireBlock.bootStrap` row, lava-ignition property, fuel
time or Composter entry: direct fire encouragement/flammability are `0/0`.

### Compression recipes and knowledge

Two Building-category shapeless records form a one-way compression chain:

- nine exact Ice items produce one Packed Ice; and
- nine exact Packed Ice items produce one Blue Ice.

Each output advancement has one OR requirement: possession of its exact
input item or existing knowledge of that recipe. Grid order is irrelevant,
all nine inputs are consumed, and input component patches are discarded.
There is no Packed-Ice-to-Ice decompression recipe.

### Chest and Wandering-Trader acquisition

`chests/ancient_city_ice_box` makes uniformly `4..10` rolls with
replacement across total weight `9`. Packed Ice has weight `2`, so each
roll selects it with probability `2/9`, then emits a uniformly integral
count `2..6`. The table uses random sequence
`minecraft:chests/ancient_city_ice_box`; pool conditions, container seed,
stack splitting and placement retain `ITM-LOOT-001`.

Wandering-Trader record
`villager_trade/wandering_trader/emerald_packed_ice` consumes one Emerald
for one Packed Ice, permits `6` uses, has reputation discount multiplier
`0.05` and inherits merchant XP `1`. It belongs to the `uncommon` trade
tag. The exact uncommon set contains `15` records and selects two distinct
offers through random sequence
`minecraft:trade_set/wandering_trader/uncommon`; selection is not two
independent draws with replacement. Offer creation, pricing, demand,
reputation, stock and transaction state retain the merchant owner.

Complete non-block loot and trade searches find no other Packed-Ice
source. Recipes, the ice-box chest, the uncommon trader, Silk block loot,
creative publication, generation and commands are its baseline acquisition
paths.

### Complete block-tag closure and consumers

Packed Ice belongs directly, and only, to these seven block tags; none has
a locked ancestor that adds another membership:

- `cannot_support_snow_layer`;
- `geode_invalid_blocks`;
- `goats_spawnable_on`;
- `ice`;
- `mineable/pickaxe`;
- `overworld_carver_replaceables`; and
- `snaps_goat_horn`.

Snow survival checks `cannot_support_snow_layer` before support overrides
or geometry, so a Snow layer cannot survive above Packed Ice despite its
full sturdy top face.

A geode distribution point that samples live Packed Ice increments the
invalid-block counter. The vanilla configured threshold is `1`, and a
count strictly greater than that aborts the feature before its layer
writes. Sampling order, distribution points and all other admission remain
with the geode owner.

A Goat natural-spawn candidate can satisfy its below-block selector with
Packed Ice, but the generic Animal brightness and spawn-placement gates
remain required. During `RamTarget`, failure to hit a living target samples
the floor at the horizontally normalized forward position and then the
state above it. Packed Ice in either position selects the horn-snapping
impact path. The goal plays its impact, asks the Goat to drop a horn and
then finishes the ram; the horn-break sound is produced only when an adult
has a horn and the drop succeeds.

The exact `ice` tag members are Ice, Packed Ice, Blue Ice and Frosted Ice.
Ocean-Ruin live-height discovery descends while the sampled cell is Air,
contains Water-tag fluid or belongs to `ice`, stopping at the first other
state or the minimum-Y boundary. This selector changes ruin placement
height but does not itself replace Packed Ice.

`overworld_carver_replaceables` admits Packed Ice to the owning Overworld
carver replacement predicate. The carver retains mask, fluid, surface,
write and postprocessing rules; tag admission does not guarantee removal.

### Fast-sliding Sulfur-Cube equipment

The item directly belongs to `sulfur_cube_archetype/fast_sliding`, which
is nested by `sulfur_cube_swallowable`; this is the complete two-tag item
closure.

The non-buoyant fast-sliding archetype fixes horizontal/vertical knockback
powers `0.6625/0.09`, additive knockback and explosion-knockback
resistance `0.5/0.5`, additive bounciness `0.10000000149011612`,
total-multiplied friction `-0.9499999992549419`, total-multiplied air drag
`-0.9900000002235174`, hit/push sound IDs `1949/1950`, push cooldown `1`
and impulse threshold `0.05`. It supplies no contact damage or explosion.

An accepting adult Sulfur Cube can install one Packed Ice in empty BODY
equipment. Because `sulfur_cube_swallowable` contains the archetype tag, an
otherwise unregistered dispenser behavior searches the front AABB and lets
the first accepting cube consume one; when none accepts, protected default
ejection runs. Matching order, equipment replacement, modifier lifecycle,
contact, knockback, sounds, traversal and dispenser residue retain
`ENT-KNOCKBACK-001` and `ITM-DISPENSER-001`.

### Frozen-Peaks surface rules

Five locked noise settings—`overworld`, `large_biomes`, `amplified`,
`caves` and `floating_islands`—each contain four default-Packed-Ice result
nodes in the Frozen-Peaks branch:

- beneath floor stone depth with no surface-depth contribution and water
  `(offset=-1, addStone=false, multiplier=0)`, `steep` returns Packed Ice;
- under the same shallow gates, `packed_ice` noise in closed interval
  `[0,0.2]` returns Packed Ice;
- beneath water `(offset=-6, addStone=true, multiplier=-1)` and floor
  stone depth with surface depth enabled, `steep` returns Packed Ice; and
- under those deep gates, `packed_ice` noise in closed interval
  `[-0.5,0.2]` returns Packed Ice.

The first three settings gate their shared surface tree with
`above_preliminary_surface`; Caves and Floating Islands leave it ungated.
Only cells identical to the setting's default block are offered, and
first-non-null ordering can select an earlier rule. Thus the five records
contain 20 Packed-Ice nodes but do not guarantee 20 or any writes.
Traversal, cached predicates, noise evaluation and commits retain
`WGEN-PIPELINE-001`.

### Frozen-Ocean surface extension

After generic surface replacement, exact `frozen_ocean` or
`deep_frozen_ocean` invokes the source-built frozen-ocean extension. For
each eligible column it creates the positional stream at `(X,0,Z)`, draws
`snowLimit=2+nextInt(4)` and
`snowFloor=seaLevel+18+nextInt(10)`, then scans downward.

An Air cell strictly below the computed upper iceberg boundary draws a
double and is admitted only when it is strictly greater than `0.01`.
A Water cell additionally requires a nonzero lower boundary, Y strictly
above it and below sea level, then draws a double that must be strictly
greater than `0.15`. Other states are untouched. An admitted cell becomes
Snow while the prior Snow-write count is `<=snowLimit` and Y is strictly
above `snowFloor`; otherwise it becomes Packed Ice. The inclusive counter
allows `3..6` Snow writes, and rejected Air samples may leave gaps.
Boundary-noise construction, exact scan limits, draw order and generation
height handling remain with `WGEN-PIPELINE-001`.

### Disk, spike, pile and iceberg outputs

Configured `ice_patch` is a disk of half-height `1`, uniform radius `2..3`
and a simple default-Packed-Ice provider. Its exact targets are Dirt, Grass
Block, Podzol, Coarse Dirt, Mycelium, Snow Block and Ice. Its placed wrapper
is scheduled only in Ice Spikes biome. Disk traversal, target reads,
provider calls and accepted writes retain the disk-feature owner.

Configured `ice_spike` supplies default Packed Ice to the source-built
spike algorithm. Exact Snow Block supports the origin search, and live
`ice_spike_replaceable` controls replacement. Its placed wrapper applies
count `3`, in-square, `MOTION_BLOCKING` height and biome filtering, again
only in Ice Spikes biome. Tower height/radius, overhangs, buried roots,
random draws and partial writes retain the spike owner.

Configured `pile_ice` uses a weighted provider with Blue Ice weight `1`
and Packed Ice weight `5`, total `6`; its placed wrapper has no modifiers.
It appears as a feature-pool element in snowy-village decor: weight `1` of
total `27` for the normal pool and weight `4` of total `22` for its zombie
variant. Feature-pool selection, pile candidates and provider draws retain
their generic owners.

Configured `iceberg_packed` supplies default Packed Ice to the Iceberg
algorithm. Its placed wrapper applies rarity `16`, in-square and biome
filtering and is scheduled in Frozen Ocean and Deep Frozen Ocean. The
algorithm can place Packed Ice, smooth/remove exposed Packed Ice and carve
it while building its above-water body, underwater body and cutouts.
Geometry, elliptical tests, snow caps, smoothing, carving and write
failure behavior remain with the Iceberg owner.

### Blue-Ice and spring inputs

The Blue-Ice feature requires its origin below `seaLevel-1`, exact Water at
the origin or immediately below, and at least one neighboring Packed Ice
among UP, NORTH, SOUTH, WEST and EAST. Success writes Blue Ice at the
origin, then makes `200` attempts consuming six integer draws each.
Candidate states may be Air, Water, Packed Ice or Ice, so Packed Ice is
both the initial admission neighbor and a replaceable later candidate.
Offsets, distance-dependent ranges, neighbor tests, writes and true-result
semantics retain the Blue-Ice owner.

`spring_lava_frozen` supplies falling Lava and names Snow Block, Powder
Snow and Packed Ice as valid blocks. Its placed wrapper is scheduled in
Frozen Peaks, Snowy Slopes, Grove and Jagged Peaks. `spring_water`
supplies falling Water and includes Packed Ice in its broader valid-block
set and is scheduled across its locked biomes. A spring can therefore
count Packed Ice at the above, below, current or five-neighbor positions
when testing its exact valid/air-hole counts before replacing the origin
with fluid. Scheduling, `count=20` for the frozen Lava wrapper, offsets,
neighbor counts and fluid writes retain the spring owner.

### Monument preservation and structure-template census

Ocean-Monument procedural fill uses a fixed keep set containing Ice,
Packed Ice, Blue Ice and Water. A live Packed-Ice cell in a fill region is
therefore preserved, while a non-kept cell can become Water below sea level
or Air above. Room construction, bounds, local coordinates and fill order
retain `WGEN-STRUCTURE-OCEAN-MONUMENT-001`.

An exhaustive decoded scan of all `1,212` bundled templates finds exactly
268 raw Packed-Ice cells in 11 files, with no target-cell block NBT:

- Ancient City `ice_box_1` contains `62`;
- Trial Chambers `spawner/ranged/stray` and
  `spawner/slow_ranged/stray` contain `4` each;
- normal snowy-village `snowy_medium_house_3`,
  `snowy_small_house_1`, `snowy_meeting_point_1` and
  `snowy_meeting_point_2` contain `30/41/17/11`, total `99`; and
- the four corresponding zombie snowy-village templates contain the same
  `30/41/17/11`, another `99`.

The Ancient-City ice-box element is weight `1` in its 46-weight structures
pool with inline-empty processors. Trial-Chambers ranged alias selection
chooses Skeleton, Stray or Poison Skeleton as paired ranged/slow-ranged
pools; each Stray pool is a sole weight-one rigid template with
inline-empty processors.

Normal snowy houses have total weight `68`, with
`snowy_small_house_1` and `snowy_medium_house_3` weight `2` each. Zombie
snowy houses total `65`, with those templates weight `2/1` and processor
`zombie_snowy`; none of its replacement rules matches Packed Ice. The
combined snowy town-center pool totals `306`: normal meeting points `1/2`
have weights `100/50`, while zombie variants have weights `2/1`; all four
use inline-empty processors.

Exact decompressed-string scanning finds 13 occurrences in those 11
files. The two beyond palette names are Jigsaw block-entity `final_state`
strings in normal and zombie `snowy_medium_house_3`, each at connector
local coordinate `[0,1,2]`. Generic Jigsaw final-state replacement can
therefore offer two additional default-Packed-Ice outputs beyond the 268
raw cells. Pool reachability, alias binding, shuffling, attachment,
rotation, overlap, clipping, processors, connector replacement and writes
retain the Ancient-City, Trial-Chambers and Village Jigsaw owners; neither
raw cells nor connector outputs are guaranteed final-world writes.

### Persistence, legacy migration and client projection

Block persistence and terrain packets preserve only state identity.
Stacks preserve generic components. Legacy numeric block-state inputs
`2784..2799`—block ID `174` times all sixteen metadata values—flatten to
property-free Packed Ice. Legacy numeric item ID `174` maps to
`minecraft:packed_ice`. Generic palette, entity-block-state and saved
feature-pool fixes may transport the name, but complete data-fix search
finds no additional Packed-Ice-specific rewrite.

The sole blockstate variant selects `minecraft:block/packed_ice`. That
`cube_all` model maps every face to the static, untinted, fully opaque
16×16 `minecraft:block/packed_ice` texture. The item definition points
directly to the block model.

Its English name is `Packed Ice`. Natural Blocks publishes it exactly once
between Ice and Blue Ice, in local order Ice, Packed Ice, Blue Ice, Snow
Block, Snow. It appears in no other baseline creative tab.

**Branches and aborts:**

- Placement is unconditional; Packed Ice has neither ordinary-Ice melting
  nor Water-after-break behavior.
- Generic friction readers differ by entity and grounded/vehicle state.
- Only Silk Touch level at least one emits block loot; no other tool,
  Fortune, correctness or explosion-survival branch restores a drop.
- Recipes compress Ice to Packed Ice to Blue Ice without a reverse recipe.
- Chest rolls and uncommon-trade selection may omit Packed Ice entirely.
- Live tag reload can change every Snow, geode, Goat, carver, ruin and
  Sulfur-Cube admission without mutating an existing state or stack.
- Surface first-match, feature predicates/providers, structure reachability
  and write admission can reject every Packed-Ice output.

**Constants and randomness:**

State/block/item IDs `12914/556/550`; legacy state/item
`2784..2799/174`; strength `0.5/0.5`; friction `0.98`, living final
ground damping `0.8918`; sound IDs break/fall/hit/place/step
`720/721/722/723/724`; stack `64`; tag closures `7/2`; recipes
`9 Ice -> 1 Packed Ice -> 1 Blue Ice`; ice-box rolls/weight/count
`4..10`, `2/9`, `2..6`; trader uses/discount/XP `6/0.05/1`;
fast-sliding constants as listed; Frozen-Peaks result nodes `20`;
frozen-ocean integer draws `2`, Snow writes `3..6`, thresholds
`0.01/0.15`; disk half-height/radius `1/2..3`; spike count `3`;
pile weights `1/5`; iceberg rarity `16`; Blue-Ice attempts/draws
`200/1200`; structure files/raw cells/final states `11/268/2`.

**Side effects:**

Block placement/removal, high-friction movement, Silk-only self loot,
crafting/results/knowledge, chest stacks and merchant offers, Snow
rejection, geode/carver/Goat/ruin selectors, Sulfur-Cube equipment and
dispenser consumption, surface/feature/structure writes, monument
preservation, state/stack persistence and migration, sounds, map color,
model, texture, name and creative-tab projection.

**Gates:**

World-write and break authority; grounded/entity/vehicle friction context;
Silk enchantment; recipe grid and knowledge; loot rolls and merchant
selection/economy; live block/item tags; Snow priority, geode threshold,
Goat spawn/ram state, carver and ruin reads, Sulfur-Cube age/equipment;
surface default-state/biome/depth/noise/order; feature scheduling,
predicates/providers/RNG; Jigsaw reachability/alias/transform/process/clip/
write; registry, reload, migration and client-resource validity.

**Boundary cases and quirks:**

Packed Ice is physically an ordinary full cube whose only movement
distinction is high friction. `mineable/pickaxe` improves speed but Silk
Touch alone gates its drop, and the Silk tool need not be a Pickaxe. Its
`cannot_support_snow_layer` tag overrides otherwise complete top support.
It neither melts nor turns into Water because it is not `IceBlock`.
Structure census separates 268 raw cells from two executable connector
final states. Monument fill preserves existing Packed Ice instead of
creating it.

**Failure semantics:**

Generic placement, break, crafting, loot, trade and equipment transactions
retain their owners' commit behavior. Surface, feature and structure
algorithms retain documented rejected-write and partial-commit semantics.
Tag/resource reload affects future reads only. Failed legacy lookup follows
the owning data-fix fallback rather than synthesizing Packed Ice.

**Client/server authority split:**

The server owns collision/support/friction state, placement, break/loot,
crafting/progression, loot/trades, tags, mobs/equipment, generation,
structures, persistence and migration. The client predicts ordinary
placement and movement, consumes synchronized state/sound/inventory/offer
outputs, and renders the model, texture, name and tab entry.

**Observability:**

Observe state/block/item/sound and legacy IDs; shapes, support, light,
path, redstone and friction reads; every Silk/tool/explosion result;
recipes/knowledge, chest/trade RNG and economy; complete tag closures and
consumers; every surface/feature/structure read, draw and write; exact
268-cell plus two-final-state census; save/wire identity and exact client
projection.

**Persistence and reload:**

Packed Ice saves one property-free identity and has no block entity. Its
stack uses generic components. Tags, loot, recipes, advancements, trades,
worldgen and client resources have independent reload boundaries.
Registration, ordinary physical behavior, frozen-ocean source extension,
Goat ram, monument keep set, legacy mapping and creative ordering are
code-built.

**Evidence:**

`net.minecraft.world.level.block.Blocks`;
`net.minecraft.world.entity.vehicle.boat.AbstractBoat`;
`net.minecraft.world.entity.animal.goat.GoatAi`;
`net.minecraft.world.entity.ai.behavior.RamTarget`;
`net.minecraft.world.level.levelgen.SurfaceSystem`;
`net.minecraft.world.level.levelgen.feature.BlueIceFeature`;
`net.minecraft.world.level.levelgen.feature.IcebergFeature`;
`net.minecraft.world.level.levelgen.structure.structures.OceanMonumentPieces`;
`net.minecraft.world.entity.SulfurCubeArchetypes`;
`net.minecraft.util.datafix.fixes.BlockStateData`;
`net.minecraft.util.datafix.fixes.ItemIdFix`;
`net.minecraft.world.item.CreativeModeTabs`; block/item/sound/component
reports; block/chest loot, two recipes/advancements, trader record/tag/set;
complete block/item tags and fast-sliding archetype; all five noise
settings and every relevant configured/placed feature, biome, pool and
processor; all `1,212` decoded templates and decompressed strings; exact
blockstate/model/item/texture/language resources. Complete compiled
exact-field, data, legacy-fix and decoded-NBT searches find no other
identity-specific runtime path.

**Test vectors:**

Run `EXP-BLK-118` across state/registry/legacy identity, every placement,
shape/support/path/redstone/friction/tool/Silk/Fortune/explosion branch,
both compression recipes/unlocks, ice-box rolls and uncommon trader
selection, complete `7/2` tag closures and consumers, fast-sliding
Sulfur-Cube equipment/dispenser paths, all surface/feature/structure and
monument selectors, all 268 raw cells and two final states,
persistence/reload and exact client projection. Assert IDs, ordering,
constants, absences, census and vanilla convergence.

**Limits:**

Generic placement/break/movement/boat, loot, crafting/progression, merchant,
Snow, Goat, geode/carver, Sulfur-Cube, surface/feature/Jigsaw/monument,
packet encoding and rendering algorithms retain their named owners. Ice,
Blue Ice, Frosted Ice, Snow and all structures retain their catalog
families. This leaf fixes exact Packed Ice and every direct join that
selects it.
