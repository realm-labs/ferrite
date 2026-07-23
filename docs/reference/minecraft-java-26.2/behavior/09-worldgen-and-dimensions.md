# 09 — World Generation, Dimensions, Portals, and Border

Ferrite uses `EquivalentPlayerVisibleBehavior` for world generation: reproduce terrain classes,
reachability, structure/resource distributions, and dimension gameplay without promising
block-for-block identity from the same seed. Runtime dimension and portal observations still target
exact behavior.

## `WGEN-001` Chunk generation advances monotonically through dependent ChunkStatus stages

- **FidelityClass:** `EquivalentPlayerVisibleBehavior`
- **Evidence status:** `Confirmed`

### Primary evidence

`OFF-SERVER-001`; `net.minecraft.world.level.chunk.status.ChunkStatus`;
`net.minecraft.world.level.chunk.status.ChunkPyramid`;
`net.minecraft.world.level.chunk.status.ChunkStep`;
`net.minecraft.world.level.chunk.status.ChunkStatusTasks`;
`net.minecraft.server.level.WorldGenRegion`;
`net.minecraft.world.level.chunk.ChunkGenerator#fillFromNoise(net.minecraft.world.level.levelgen.blending.Blender,net.minecraft.world.level.levelgen.RandomState,net.minecraft.world.level.StructureManager,net.minecraft.world.level.chunk.ChunkAccess)`;
`net.minecraft.world.level.chunk.ChunkGenerator#buildSurface(net.minecraft.server.level.WorldGenRegion,net.minecraft.world.level.StructureManager,net.minecraft.world.level.levelgen.RandomState,net.minecraft.world.level.chunk.ChunkAccess)`;
`net.minecraft.world.level.chunk.ChunkGenerator#applyCarvers(net.minecraft.server.level.WorldGenRegion,long,net.minecraft.world.level.levelgen.RandomState,net.minecraft.world.level.biome.BiomeManager,net.minecraft.world.level.StructureManager,net.minecraft.world.level.chunk.ChunkAccess)`

### Applies when

A new/incomplete chunk is requested at a higher generation status.

### Behavior and timing

The `26.2` chain is
`EMPTY → STRUCTURE_STARTS → STRUCTURE_REFERENCES → BIOMES → NOISE → SURFACE → CARVERS → FEATURES → INITIALIZE_LIGHT → LIGHT → SPAWN → FULL`.
The generation pyramid fixes each step's Chebyshev neighbor statuses: structure and carver-source
access extend to radius `8`, light and spawn use radius `1`, and accumulated dependencies compose
across steps. CARVERS visits the `17×17` source-chunk square in X-major order, selects each source's
cached owner biome at quart Y `0`, and reseeds every ordered configured-carver attempt from world
seed, per-source list index and source coordinates. A future publishes the higher protochunk status
only after successful completion.

### Boundaries and quirks

`NOISE`, `SURFACE`, and `CARVERS` can mutate only the center; `FEATURES` can mutate the 3×3 chunk
square; other region tasks have write radius `-1`. CARVERS shares one target carving mask and
aquifer across all source attempts, and its optional old-generation mask filter is installed before
the generator's disable gate. Its common volume kernel marks admitted geometry before material
admission, clips seven blocks below the generation ceiling except during upgrading, and restores
surface dirt only after a grass/mycelium encounter in the same descending column. Low-level step
application still invokes its task when the chunk is already at/above target, so generation
scheduling and loading-task replay—not a blanket idempotence claim—own repeat behavior.

### Verification

**Owners:** `WGEN-PIPELINE-001`, `BLK-COMMAND-AREA-001`; `EXP-WGEN-*`, `EXP-BLK-018`

The leaf locks the status/dependency/write/task graph and CARVERS source, owner, seed, mask and
admission dispatch. Ferrite may use a different scheduler or seed mapping only if dependency
visibility, exceptional-completion publication, cross-chunk isolation and the eventual quantitative
equivalence suite agree.

## `WGEN-002` Registry-parameterized noise produces biomes and base terrain

- **FidelityClass:** `EquivalentPlayerVisibleBehavior`
- **Evidence status:** `Confirmed`

### Primary evidence

`OFF-DATA-001`; `OFF-REPORT-001`; `OFF-SERVER-001`;
`net.minecraft.world.level.biome.BiomeSource#getNoiseBiome(int,int,int,net.minecraft.world.level.biome.Climate$Sampler)`;
`net.minecraft.world.level.levelgen.NoiseBasedChunkGenerator#fillFromNoise(net.minecraft.world.level.levelgen.blending.Blender,net.minecraft.world.level.levelgen.RandomState,net.minecraft.world.level.StructureManager,net.minecraft.world.level.chunk.ChunkAccess)`;
`net.minecraft.world.level.levelgen.NoiseBasedChunkGenerator#buildSurface(net.minecraft.server.level.WorldGenRegion,net.minecraft.world.level.StructureManager,net.minecraft.world.level.levelgen.RandomState,net.minecraft.world.level.chunk.ChunkAccess)`;
`net.minecraft.world.level.levelgen.NoiseBasedChunkGenerator#applyCarvers(net.minecraft.server.level.WorldGenRegion,long,net.minecraft.world.level.levelgen.RandomState,net.minecraft.world.level.biome.BiomeManager,net.minecraft.world.level.StructureManager,net.minecraft.world.level.chunk.ChunkAccess)`

### Applies when

BIOMES, NOISE, SURFACE, or CARVERS selects environment and blocks for a chunk.

### Behavior and timing

BIOMES fills every section's `4×4×4` quart palette on the background executor. Generic generators
use their source plus the random-state sampler; noise generators use a cached noise-chunk sampler
and resolver order `retrogen(blender(source))`. The four source codecs are fixed, checkerboard,
multi-noise nearest climate region, and End central-radius/erosion thresholds. The Overworld preset
fixes five temperature, five humidity and seven erosion bands, seven continentalness regions, two
ocean rows, five-by-five base/variant matrices, an ordered 13-span surface lattice, four underground
regions and two spawn-target regions. Biome queries separately use inclusive quart cuboids, uniform
reservoir selection over horizontal matches, outward square perimeters, or horizontal-spiral-first
closest-3D traversal. NOISE clamps one of seven settings to the generation accessor, locks
intersecting sections, traverses interpolated cells top-down, resolves aquifer then ore then default
block, updates two worldgen heightmaps and schedules generated fluid updates. Surface rules replace
top layers afterward; CARVERS then dispatches source-biome-owned configured carvers through a shared
aquifer and mask. Its common kernel clips and traverses ellipsoids exactly, applies family shape
skips, replaceability and lava/aquifer material selection, maps debug markers, schedules fluids and
restores exposed surface dirt; cave paths add optional rooms, damped random-walk tunnels and a
one-time two-way fork, Nether changes count/thickness/vertical scale/material, and canyon uses a
single width-table-weighted path with per-step horizontal and vertical radii.

### Boundaries and quirks

Checkerboard's encoded `scale+2` is a Java integer shift and wraps modulo `32` at high legal scales.
Multi-noise comparison uses quantized seven-coordinate squared interval distance; its
strict-improvement R-tree retains an equally good thread-local previous leaf. End's central biome
radius uses chunk-section coordinates and inclusive squared radius `4096`. Old/new chunk boundaries
wrap the source through blender then retrogen.

### Verification

**Owners:** `WGEN-PIPELINE-001`; `EXP-WGEN-*`

The leaf source-specifies biome selection, the NOISE task's settings, clipping, concurrency,
traversal, material precedence, aquifer/ore resolution, every density type and all 35 locked density
records, complete SURFACE traversal, all 15 material dispatches, all seven locked rule trees and all
three extension-biome selectors, plus the complete dispatcher, shared kernel and all three carver
algorithms. Feature/structure algorithms and quantitative distribution acceptance remain open;
equivalence uses distributions, elevation profiles, connectivity, and player paths rather than
vanilla seed hashes.
`BLK-COMMAND-AREA-001` separately owns post-generation `/fillbiome` quart quantization, loaded-chunk
admission, predicate evaluation, palette replacement, dirty marking and biome resend behavior.

## `WGEN-003` Features and structures select, place, and reference across chunks separately

- **FidelityClass:** `EquivalentPlayerVisibleBehavior`
- **Evidence status:** `Confirmed`

### Primary evidence

`OFF-DATA-001`; `OFF-SERVER-001`;
`net.minecraft.world.level.chunk.ChunkGenerator#createStructures(net.minecraft.core.RegistryAccess,net.minecraft.world.level.chunk.ChunkGeneratorStructureState,net.minecraft.world.level.StructureManager,net.minecraft.world.level.chunk.ChunkAccess,net.minecraft.world.level.levelgen.structure.templatesystem.StructureTemplateManager,net.minecraft.resources.ResourceKey)`;
`net.minecraft.world.level.chunk.ChunkGenerator#createReferences(net.minecraft.world.level.WorldGenLevel,net.minecraft.world.level.StructureManager,net.minecraft.world.level.chunk.ChunkAccess)`;
`net.minecraft.world.level.chunk.ChunkGenerator#applyBiomeDecoration(net.minecraft.world.level.WorldGenLevel,net.minecraft.world.level.chunk.ChunkAccess,net.minecraft.world.level.StructureManager)`;
`net.minecraft.world.level.levelgen.feature.ConfiguredFeature#place(net.minecraft.world.level.WorldGenLevel,net.minecraft.world.level.chunk.ChunkGenerator,net.minecraft.util.RandomSource,net.minecraft.core.BlockPos)`;
`net.minecraft.world.level.levelgen.feature.Feature#place(net.minecraft.world.level.levelgen.feature.configurations.FeatureConfiguration,net.minecraft.world.level.WorldGenLevel,net.minecraft.world.level.chunk.ChunkGenerator,net.minecraft.util.RandomSource,net.minecraft.core.BlockPos)`;
`net.minecraft.world.level.levelgen.feature.NoOpFeature#place(net.minecraft.world.level.levelgen.feature.FeaturePlaceContext)`;
`net.minecraft.world.level.levelgen.structure.Structure#generate(net.minecraft.core.Holder,net.minecraft.resources.ResourceKey,net.minecraft.core.RegistryAccess,net.minecraft.world.level.chunk.ChunkGenerator,net.minecraft.world.level.biome.BiomeSource,net.minecraft.world.level.levelgen.RandomState,net.minecraft.world.level.levelgen.structure.templatesystem.StructureTemplateManager,long,net.minecraft.world.level.ChunkPos,int,net.minecraft.world.level.LevelHeightAccessor,java.util.function.Predicate)`;
`net.minecraft.world.level.levelgen.structure.Structure#afterPlace(net.minecraft.world.level.WorldGenLevel,net.minecraft.world.level.StructureManager,net.minecraft.world.level.chunk.ChunkGenerator,net.minecraft.util.RandomSource,net.minecraft.world.level.levelgen.structure.BoundingBox,net.minecraft.world.level.ChunkPos,net.minecraft.world.level.levelgen.structure.pieces.PiecesContainer)`

### Applies when

STRUCTURE_STARTS/REFERENCES or FEATURES runs.

### Behavior and timing

Structure sets/placements first use salt, spacing/rules, biome, and generation point to create a
start. Pieces may cross chunks; affected chunks retain references, and decoration places
bounding-box slices before `afterPlace`. Placed features are ordered by biome generation step,
transform an origin through ordered placement modifiers, and invoke every resulting configured
feature while OR-aggregating success. Configured-feature dispatch checks origin write admission
before constructing the family context. The `no_op` type returns success without reads, writes, or
RNG use; five composite selectors implement ordered independent float gates, integer-weight choice,
uniform index choice, boolean choice, or all-must-succeed sequence. Direct block-write types either
replace the first matching origin rule or fill air across a positive-offset 16×16 layer.
`simple_block` obtains one provider state and requires survival, then performs an ignored-result
ordinary write, ordered double-plant lower/upper offers, or pale-moss face/topper construction using
level RNG; its optional delay-one tick targets the postwrite origin block. Platform features either
write one chunk's intersection with the fixed 33×33 void-start square or normalize the End spawn's
5×5 obsidian floor and three air layers. `vines` writes one face toward the first attachable
neighbor in fixed vertical/horizontal order. `sea_pickle` makes a configured number of
triangular-offset ocean-floor attempts, drawing cluster size before water/support admission.
`blue_ice` seeds an admitted origin, then makes 200 bounded propagation attempts into air, water,
packed ice or ice adjacent to blue ice. `kelp` grows ocean-floor body cells upward and offers one
aged head either at the sampled endpoint or as a fallback below the first failed continuation.
`block_pile` traverses a randomized two-level footprint and conditionally asks its provider for
states only at empty, supported, radially admitted cells. `freeze_top_layer` scans a positive-offset
16×16 `MOTION_BLOCKING` surface, freezes cold source water without an edge requirement, then places
snow and marks supporting snowy blocks. `end_island` writes shrinking end-stone disks downward from
a sampled radius. `glowstone_blob` seeds an admitted ceiling cell, then grows only into sampled air
cells having exactly one glowstone neighbor. `block_blob` finds the first admitted support downward,
then writes three offset compact blob passes. `seagrass` samples one ocean-floor water cell and
conditionally offers a short plant or ordered tall halves. `nether_forest_vegetation` samples
provider states across a triangular-offset volume above nylium and counts survivable empty-cell
offers. `spring_feature` replaces an admitted shell center only when its five-neighbor valid-block
and empty-hole counts exactly match configuration, then schedules its fluid immediately.
`bonus_chest` shuffles a chunk's columns, installs a loot-seeded chest at the first collision-empty
surface and offers four surviving torches. `disk` samples one radius and scans descending target
columns inside that circle, offering provider states and marking exposed run tops for
postprocessing. `basalt_pillar` descends from an admitted ceiling, adding cardinal hangoffs and a
randomized supported root skirt. `delta_feature` walks a sampled horizontal Manhattan footprint,
applying strict cavity gates before optional rim and offset contents offers.
`netherrack_replace_blobs` searches downward for its target and replaces a sampled clipped Manhattan
volume around the first match. `underwater_magma` scans a water column for its floor, then
probabilistically replaces fully enclosed cells in a centered cube. `spike` builds a tapered,
optionally elevated packed-ice body with mirrored lower layers and segmented roots. `desert_well`
validates sand support, writes a fixed well in source order, and installs two independently selected
archaeology blocks. `bamboo` optionally converts a sampled surface disk to podzol, grows an
obstruction-limited trunk, and overwrites its last three cells with a fixed crown. `chorus_plant`
depth-first grows connected vertical segments and bounded branches, terminating unbranched tops with
dead flowers. `twisting_vines` searches randomized ground columns upward, while `weeping_vines`
spreads a wart roof and hangs downward columns. `basalt_columns` samples clustered candidate
squares, searches local surfaces, and grows height-tapered basalt columns. `end_gateway` writes its
fixed gateway/bedrock/air matrix and optionally configures the center block entity. The three coral
types share strict water admission plus coral, pickle and ordered wall-fan decoration; claw builds
shuffled asymmetric arms, mushroom scans randomized face interiors, and tree grows a trunk with
shuffled gated branches. The paired huge-mushroom types share pre-validation height draws, exact
bounds/support/clearance admission and cap-before-trunk replaceable writes; brown uses one rounded
square cap, while red uses three hollow perimeter layers plus a filled top. `block_column` samples
all layer heights, performs a direction-sensitive one-cell lookahead, truncates from base or tip
priority, then emits retained provider states. `large_dripstone` scans for a bounded cave, derives
paired profile shapes from cave-capped radius and sampled scale/bluntness, conditionally bends both
with one wind vector, retreats each base into embedded stone, then writes stalactite before
stalagmite. `speleothem` chooses an upper/lower support, spreads a replaceable base patch, then
emits a one- or two-cell pointed column with target-fluid waterlogging. `speleothem_cluster` scans
an independently sized rectangle of cave columns, optionally inserts floor pools, biases paired
heights by edge/center distance, and repartitions colliding tips. `end_spike` selects cached or
explicit spikes by origin chunk, rebuilds each obsidian cylinder/air-clear volume and optional iron
cage, then spawns and caps its End crystal. `scattered_ore` samples a zero-inclusive attempt count,
spreads rounded triangular offsets up to radius seven, and replaces the first ordered ore target
that passes its exposure gate. `ore` builds and containment-prunes a randomized ellipsoid chain,
deduplicates overlapping cells before admission, and writes admitted targets directly into acquired
chunk sections. `multiface_growth` shuffles eligible support faces, retries one-cell travel
candidates, places a water-aware face, and may run the block-specific same-plane/wrap spreader.
`lake` unions four through seven sampled ellipsoids inside a fixed cavity, fail-fast audits their
axial boundary, fills upper cave air and lower provider fluid, probabilistically coats the solid
shell and runs a water-only fixed-plane freeze pass. `monster_room` validates a randomized solid
floor/ceiling and bounded opening count, carves a protected cave-air room with a stochastic floor,
then makes bounded chest/loot attempts and initializes an origin spawner. `fossil` selects one
aligned primary/overlay template pair and rotation, derives burial from the primary footprint,
rejects excessive empty/liquid bounding corners, then swaps ordered processors across two
ignored-result placements. `template` draws a weighted structure and allowed rotation, offsets the
unrotated X/Z half-sizes along rotated negative axes, and directly returns one no-processor flags-3
template placement. `vegetation_patch` scans a chance-edged rectangular cave-surface footprint,
writes provider-depth ground columns and distributes nested vegetation over successful ground cells;
`waterlogged_vegetation_patch` filters that set by ordered face exposure, fills the enclosed subset
with water, then waterlogs successful child states. `sculk_patch` runs bounded worldgen charge
cursors through default, sculk and vein behaviors before catalyst and shrieker postpasses.
`fallen_tree` unconditionally offers a decorated stump, validates a direction-sampled horizontal log
against tree-replaceability and consecutive support gaps, then applies attached-log decoration.
`root_system` searches upward for an admissible child-feature origin, grows tag-filtered root layers
below a successful child, then makes bounded hanging-root attempts around the original feature
origin. `huge_fungus` builds a narrow or rare 3×3 stem and a layer-randomized wart hat with
mode-specific replacement, destruction, shroomlight and hanging-vine behavior. `geode` constructs a
world-seed-local noise-distorted inverse-distance field from sampled positive-offset centers, carves
optional fluid-ticking cracks, writes four protected layers and grows direction-priority amethyst
placements. `iceberg` builds snow-biased round or elliptical ice masses around sea level, then
smooths and optionally cuts a mutation-sensitive cavity. The shared `tree` pipeline samples and
clips height, places roots/trunk/foliage/decorators, and repairs leaf distance; all nine straight,
giant, mega-jungle, forking, bending, upward-branching, dark-oak, fancy and cherry trunk placers now
have exact validation, target, RNG, axis and attachment semantics, as do all 11 foliage placers: the
blob, bush, fancy, mega-jungle, pine, spruce, acacia and dark-oak row-based families, cherry
hanging-leaf family, mega-pine crown family and random-spread attempt family. The sole mangrove
root-placer family now also has exact offset, column admission, depth-first preflight, length/width
budget, material, waterlogging and above-root semantics across both locked mangrove records.
Trunk-vine, leaf-vine and pale-moss decorators have exact list, direction, gate, hanging-chain and
nested ground-feature semantics; cocoa, creaking-heart and beehive decorators now also own exact
Y-band, direction, shuffle, state, typed-block-entity and occupant initialization behavior.
Attached-to-leaves and attached-to-logs own exact shuffle, direction, strict-versus-inclusive
probability, air-chain, exclusion-reservation, provider and ignored-result timing across all seven
locked users. Alter-ground and place-on-ground own exact lowest-list, footprint, column, heightmap,
provider and mutation timing across both mega-tree and all 14 leaf-litter decorator records. All 39
locked tree configurations are data-only audited across 19 canonical nondecorator signatures and
exact ordered decorator chains.

### Boundaries and quirks

Structure spawn overrides, locate commands, and loot depend on starts/references rather than only
visible blocks. Feature order can overwrite earlier results and cannot become unordered concurrent
writes to one region. An out-of-range no-op returns false at the common wrapper; an admitted one
returns true despite making no block change. Selected composite children propagate false rather than
falling through, while sequence preserves earlier mutations when a later child fails. Both
direct-write algorithms return true after admission even when no block changes, and ignore
individual write failures. Geode protection also ignores write outcomes: a selected crack still
schedules neighboring fluid ticks, and a protected first-valid bud target still suppresses later
direction fallbacks.

### Verification

**Owners:** `WGEN-PIPELINE-001`, `WGEN-STRUCTURE-BURIED-001`, `WGEN-STRUCTURE-NETHER-FOSSIL-001`,
`WGEN-STRUCTURE-IGLOO-001`, `WGEN-STRUCTURE-SWAMP-HUT-001`, `WGEN-STRUCTURE-DESERT-PYRAMID-001`,
`WGEN-STRUCTURE-JUNGLE-TEMPLE-001`, `WGEN-STRUCTURE-SHIPWRECK-001`,
`WGEN-STRUCTURE-RUINED-PORTAL-001`, `WGEN-JIGSAW-CORE-001`, `BLK-JIGSAW-001`,
`BLK-STRUCTURE-001`, `BLK-STRUCTURE-VOID-001`, `BLK-AIR-001`, `BLK-BEDROCK-001`,
`BLK-REINFORCED-DEEPSLATE-001`, `BLK-TERRACOTTA-001`,
`BLK-GLAZED-TERRACOTTA-001`, `BLK-QUARTZ-001`, `BLK-SANDSTONE-001`,
`BLK-STONE-VARIANT-001`, `BLK-SLIME-001`, `BLK-HONEY-001`,
`BLK-SOUL-SAND-001`,
`BLK-MAGMA-001`;
`EXP-WGEN-*`, `EXP-BLK-021`,
`EXP-BLK-027`, `EXP-BLK-029`, `EXP-BLK-030`, `EXP-BLK-031`, `EXP-BLK-032`, `EXP-BLK-035`,
`EXP-BLK-036`, `EXP-BLK-037`, `EXP-BLK-038`, `EXP-BLK-042`, `EXP-BLK-043`, `EXP-BLK-044`,
`EXP-BLK-045`, `EXP-BLK-046`

`BLK-TERRACOTTA-001` owns the 192-state badlands clay-band palette and lookup identities, exact
plain/orange/white surface outputs, terracotta-composed azalea/sculk/carver tags, five
zombie-village replacement inputs, desert-pyramid orange/blue writes and trail-ruins template
states. Surface, feature, carver, processor, structure and template traversal/RNG/write behavior
remain with their dedicated worldgen owners.
`BLK-GLAZED-TERRACOTTA-001` owns the resulting color/facing semantics for 36 locked templates:
24 trail-ruins, one trial-chambers ominous vault, three large underwater ruins and eight desert or
savanna village templates. Their named structure/jigsaw owners retain pool selection, processors,
palette traversal, rotation/mirror choice and write order.
`BLK-QUARTZ-001` owns four quartz-block and two smooth-quartz cells in the rigid, weight-one
`bastion/bridge/bridge_pieces/bridge` element. Two additional smooth-quartz slabs remain with
`shape-family`; `WGEN-JIGSAW-BASTION-001` retains pool selection, `bridge` processor traversal,
template transforms and write order.
`BLK-SANDSTONE-001` owns the full-block identities selected by shared surface rules, sand disks,
desert wells, two flat presets, buried-treasure support and desert-pyramid layout. It also owns
5,833 live cells across 83 underwater-ruin, desert-village and trial-chamber templates, plus
smooth/cut sandstone inputs to the zombie-desert cobweb rules. The named pipeline, structure,
jigsaw and processor owners retain selection, RNG, traversal, transforms and write order; shape
blocks remain with `shape-family`.
`BLK-STONE-VARIANT-001` owns raw granite/diorite/andesite identity in base-stone, stone-ore,
attachment, spring, copper-vein-filler and buried-treasure joins, plus their three size-64 lower/
upper ore pairs. It also owns 1,797 live full-block cells across 42 sulfur-spring, underwater-ruin,
village, mansion, igloo and trial-chamber templates. The named pipeline, structure and jigsaw
owners retain selection, RNG, traversal, transforms and write order; shape blocks remain with
`shape-family`.
`BLK-SLIME-001` owns the diagnostic exception inside noise-chunk fill: only enabled
`DEBUG_AQUIFERS` stripes at nonnegative Z divisible by four replace the Y=preliminary-surface+8
state with slime below sea level or honey otherwise. Normal generation returns the interpolated
state unchanged; pipeline ordering and equivalence remain here.
`BLK-HONEY-001` owns the at-or-above-sea half of that same diagnostic selector without broadening it
into normal terrain generation.
`BLK-SOUL-SAND-001` owns normal Nether identities: soul-sand-valley ceiling/floor and Nether-wastes
surface-rule branches, size/count-12 ore through Y 0..31, lava-spring valid support, Nether-carver
replacement, fortress wart-bed boxes, Nether-fossil anchor short-circuit and basalt-column support/
scan rejection. Generation-region writes additionally mark the cell above unless flags contain bit
16; traversal, RNG, clipping, write failures and pipeline order remain with their worldgen owners.
`BLK-MAGMA-001` owns normal magma identities: five-biome ore, 55-biome underwater feature,
Basalt-Deltas delta rim, non-cold ruined-portal spread/drips, basalt-column exclusion, Nether
lava-spring support and the bastion bottom-rampart processor. Its underwater feature consumes the
0.5 chance before validity and can report success after a failed flags-2 write; generation-region
writes also mark the cell above unless flags contain bit 16. Pipeline, structure and processor
ordering remains with their dedicated owners.

Configured/placed-feature dispatch, all 63 feature algorithms, all 30 locked top-level selector
records, all 32 locked top-level simple-block records, both platform configured/placed record pairs,
all three vines records, both sea-pickle records, both blue-ice records, all three kelp records, all
ten block-pile records, both freeze-top-layer records, both end-island records, all three
glowstone-blob records, both block-blob records, all 12 seagrass records, all nine
Nether-forest-vegetation records, all 13 spring records, the bonus-chest record, all ten disk
records, both basalt-pillar records, both delta records, all four replacement-blob records, both
underwater-magma records, both spike records, both desert-well records, five bamboo-named records,
both chorus-plant records, all five Nether-vines records, all four basalt-columns records, all three
End-gateway records, the warm-ocean placed wrapper, both huge-mushroom configured records, all three
block-column records, both large-dripstone records, the pointed-dripstone placed wrapper, both
dripstone-cluster records, both End-spike records, all four scattered-ore records, all 68 ore
records, all four multiface-growth records, all four lake-family records, all three monster-room
records, all seven fossil-family records, and the template algorithm with all ten sulfur-spring NBT
size/count inputs, plus both vegetation-patch algorithms and all ten direct patch records,
sculk-patch with both configurations and wrappers, fallen-tree with five configurations and five
wrappers, root-system with two configurations and two wrappers, huge-fungus with four configurations
and two wrappers, and geode with its configured/placed record pair, plus iceberg with both
configured states and both placed wrappers, are source-specified/data-only audited; the tree feature
shared orchestration, clearance, clipping, common placement primitives and leaf-distance repair plus
all nine trunk families, all 11 blob, bush, fancy, mega-jungle, pine, spruce, acacia, dark-oak,
cherry, mega-pine and random-spread foliage families, the sole mangrove root-placer family, and the
trunk-vine, leaf-vine, pale-moss, cocoa, creaking-heart, beehive, attached-to-leaves,
attached-to-logs, alter-ground and place-on-ground decorator families are also source-specified,
Buried treasure owns its exact start anchor, biome gate, one-piece support/enclosure/chest algorithm
and both locked records; Nether fossil owns its uniform-height cavity scan, one-of-14 rotated
bone-template transaction, full-box expansion, deterministic dried-ghast postpass and both locked
records; igloo owns its center-biome/live-surface split, optional laboratory shaft, three sparse
template payloads, marker loot, two template entities and top repair; swamp hut owns its
full-footprint terrain mean, exact cabin and supports, latched witch/cat occupants and black-cat
finalization; desert pyramid owns its four-corner gate, four-chunk fixed piece, trap, chests, cellar
collapse and globally stable archaeology selection; jungle temple owns its terrain-averaged
randomized masonry, exact redstone mechanisms and four container latches; shipwreck owns its
ocean/beached height split, all 20 eight-palette templates and marker-loot seed transaction; ruined
portal owns its weighted setup, six height policies, 13 processed templates and unclipped
apron/drip/vine/leaf postpasses. The later structure and jigsaw leaves named by the completion
ledger own the remaining locked structure types and payload families. Ferrite may use different
seed mapping, but must statistically lock rarity, minimum spacing, biome constraints, cross-chunk
completeness, and the allowed locate-result divergence.

`BLK-JIGSAW-001` fixes the operator block's current pool/target/orientation lookup and raw
level/keep handoff; `WGEN-JIGSAW-CORE-001` retains all subsequent selection, collision, RNG and
world-write behavior.
`BLK-STRUCTURE-001` fixes named template capture, manager cache/disk choice, size-prepared versus
direct placement settings and two RNG construction points; `WGEN-PIPELINE-001` retains delegated
template processor, block/entity write and neighbor consequences.
`BLK-STRUCTURE-VOID-001` fixes capture-time coordinate omission and the jigsaw final-state no-output
sentinel while distinguishing both from generic raw-template structure-void writes.
`BLK-AIR-001` fixes the Nether carver's lava-through-`minGenY + 31` then cave-air choice and the
lake/monster-room cave-air empty writes, while other carver aquifer and structure air choices stay
with their existing owners.
`BLK-BEDROCK-001` fixes the exact state selected by surface/flat/End generation, retrogen,
hard-coded feature exclusions, count-on-every-layer, `features_cannot_replace` and
`geode_invalid_blocks`; traversal, RNG, geometry, flags and return values stay with
`WGEN-PIPELINE-001`.
`BLK-REINFORCED-DEEPSLATE-001` fixes the state selected by ancient-city templates and connector
final states plus its `features_cannot_replace` target protection; jigsaw transforms, processors,
feature traversal, RNG, flags and return values stay with their existing worldgen owners.
`BLK-TEST-INSTANCE-001` fixes configured-test lookup, effective rotation/padding geometry, permanent
chunk forcing, exhaustive AIR clearing, tick/event/entity removal, flags-818 template placement,
capture/export, barrier boundary and the successful RUN's second in-place placement; the configured
test body remains delegated GameTest validation infrastructure.

## `WGEN-004` DimensionType locks height, sky, time, and coordinate-scale semantics

- **FidelityClass:** `ExactObservableBehavior`
- **Evidence status:** `Confirmed`

### Primary evidence

`OFF-DATA-001`; `OFF-SERVER-001`; `OFF-CLIENT-001`;
`net.minecraft.world.level.dimension.DimensionType`;
`net.minecraft.world.attribute.EnvironmentAttributeSystem`;
`net.minecraft.world.clock.ServerClockManager`; `net.minecraft.server.level.PlayerSpawnFinder`

### Applies when

A dimension/type is created or a mechanic queries vertical limits, light/weather, clocks/timelines,
typed environment attributes, beds/anchors, spawn conditions, presentation inputs, or
cross-dimension coordinates.

### Behavior and timing

The type record independently selects storage/logical height, coordinate scale, sky/ceiling/dragon
gates, ambient/monster light, render modes, attribute constants, timelines and an optional default
clock. Attribute resolution applies dimension, biome, timeline and eligible-weather layers in order
and sanitizes the typed result. Clocks advance globally under `advance_time`; fixed-time only
disables outside bright/dark predicates. Cross-level conversion multiplies X/Z by source scale
divided by destination scale.

### Boundaries and quirks

Nether logical height does not truncate its storage height. Weather also excludes the literal End
key, portal routing uses keys, but dragon-fight initialization and scale use types. A custom
dimension can therefore reuse a type without inheriting every vanilla-key behavior. Beds have
independent set-spawn/sleep/explode fields; anchors use a separate positional boolean.

### Verification

**Owners:** `WGEN-DIMENSION-001`, `BLK-BANNER-001`, `MOB-RAID-001`; `EXP-WGEN-002`,
`EXP-BLK-012`, `EXP-MOB-011`

The dimension leaf source-specifies all four records, all 48 attribute declarations/layers, height
endpoints, key/type splits, clocks, scale callers, monster and initial-player spawn gates,
beds/anchors and client sampling. The banner leaf fixes its forced-respawn free-cell override. The
raid leaf fixes positional `CAN_START_RAID`, including the default-true value and locked Nether
override. The experiments are regression-only.

## `WGEN-005` A portal accumulates eligibility, then searches for or creates an exit

- **FidelityClass:** `ExactObservableBehavior`
- **Evidence status:** `Confirmed`

### Primary evidence

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-DATA-001`; `net.minecraft.world.entity.PortalProcessor`;
`net.minecraft.world.entity.Entity#handlePortal()`;
`net.minecraft.world.level.block.NetherPortalBlock`;
`net.minecraft.world.level.portal.PortalForcer`; `net.minecraft.world.level.block.EndPortalBlock`;
`net.minecraft.world.level.block.entity.TheEndPortalBlockEntity`;
`net.minecraft.client.renderer.blockentity.TheEndPortalRenderer`;
`net.minecraft.world.level.block.entity.TheEndGatewayBlockEntity`

### Applies when

An eligible entity contacts a Nether portal, End portal or End gateway, or the resulting transition
moves an entity/passenger graph.

### Behavior and timing

Contact marks a processor; its old accumulated time is compared before increment, absence decays
progress by four, and a ready attempt starts cooldown before destination validation. Nether travel
performs key-based routing, type-scale conversion, border clamp, exact POI ranking, deterministic
site creation and entry-relative exit placement. End portals dispatch to End spawn or saved
respawn/credits; their subtype supplies the exact contact slab, display-particle draws and two-face
15-layer client surface. Gateways dispatch to a persisted exact or terrain-adjusted same-level exit
and can generate a reciprocal gateway.

### Boundaries and quirks

Portal families share the processor but not wait, local effect, destination, post-effect or motion
policy. Equal Nether POI distance/Y keeps hash-stream encounter order. Failure still consumes entity
cooldown; gateway contact also consumes its block-entity cooldown. Cross-level ordinary entities are
recreated and passenger transfer can partially succeed if root construction fails.

### Verification

**Owners:** `WGEN-PORTAL-001`; `EXP-WGEN-003`

The leaf source-specifies wait/cooldown, eligibility overrides, all three routes, End-portal
physical/render surface, search/create/tie rules, exact exit pose, gateway terrain selection and
passenger/player transfer. The experiment is regression-only.

## `WGEN-006` The world border advances before entities and supplies shared horizontal geometry

- **FidelityClass:** `ExactObservableBehavior`
- **Evidence status:** `Confirmed`

### Primary evidence

`OFF-SERVER-001`; `OFF-CLIENT-001`; `net.minecraft.world.level.border.WorldBorder`;
`net.minecraft.server.level.ServerLevel#tick(java.util.function.BooleanSupplier)`;
`net.minecraft.world.entity.LivingEntity#baseTick()`;
`net.minecraft.world.entity.Entity#collectCollidersIgnoringWorldBorder`;
`net.minecraft.client.gui.Hud#extractVignette`;
`net.minecraft.client.renderer.WorldBorderRenderer#extract`

### Applies when

A dimension's border is mutated or queried for point/AABB/chunk containment, collision, ray
clipping, interaction, spawn, respawn or portal targeting, player damage, HUD warning, or
force-field rendering.

### Behavior and timing

A positive-duration resize stores a remaining-tick countdown. Each normally running level tick
decrements it first and advances the stored size before weather and entities. During intermediate
steps, however, every ordinary geometry getter requests partial `0` and therefore exposes the copied
pre-update size to collision and player damage. The completion update replaces the moving extent
with a static target, so that final entity phase sees the target immediately. Client sync sends one
resize snapshot and the client independently advances its copy; force-field geometry interpolates
previous/current client sizes.

### Boundaries and quirks

The intermediate geometry lag and final target jump are distinct from `getSize()`, which exposes the
newly calculated size. Minimum edges are inclusive and maximum edges exclusive for points; AABB
maximum faces receive a `0.00001` inward tolerance. Clamp targets stop at `max-0.00001`. Collision
uses integer-rounded walls outside the interior. Save/restart and reconnect preserve remaining ticks
rather than consuming wall-clock time. The fixed absolute coordinate ceiling and configurable border
are distinct.

### Verification

**Owners:** `WGEN-BORDER-001`; `EXP-WGEN-004`

The leaf locks countdown order, intermediate geometry lag, completion-phase damage, all tolerances,
warning projection, client partial interpolation and synchronization; the experiment is
regression-only.

## `WGEN-007` World-generation compatibility is player-visible equivalence, not same-seed identity

- **FidelityClass:** `EquivalentPlayerVisibleBehavior`
- **Evidence status:** `Confirmed`

### Primary evidence

`OFF-DATA-001`; `OFF-REPORT-001`; `OFF-SERVER-001`;
[docs/architecture.md#19-world-generation](../../../architecture.md#19-world-generation)

### Applies when

Ferrite implements or tests terrain, biomes, features, structures, and resource distributions.

### Behavior and timing

It must retain dimension identity, traversable terrain, surface/cave relationships, biome and
structure constraints, resource-rarity order of magnitude, and gameplay dependencies such as
spawning, locate, loot, and mob spawning. Different RNG, seed mixing, parallel generation tasks, and
concrete block placement are allowed when defined statistical and scenario acceptance passes.

### Boundaries and quirks

Discrete runtime results such as mutations, structure trades, portal exit, and safe-spawn search are
not automatically relaxed by this rule; their own `ExactObservableBehavior` rules still apply.

### Verification

**Owners:** `WGEN-PIPELINE-001`; `EXP-WGEN-*`

Define a quantitative threshold, sample-seed set, and failure diagnostic for every equivalence claim
before implementation. “Looks similar” is not acceptance when thresholds are absent.
