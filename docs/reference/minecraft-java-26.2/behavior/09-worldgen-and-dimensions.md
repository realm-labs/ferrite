# 09 — World Generation, Dimensions, Portals, and Border

Ferrite uses `EquivalentPlayerVisibleBehavior` for world generation: reproduce terrain classes, reachability, structure/resource distributions, and dimension gameplay without promising block-for-block identity from the same seed. Runtime dimension and portal observations still target exact behavior.

## `WGEN-001` Chunk generation advances monotonically through dependent ChunkStatus stages

- **FidelityClass:** `EquivalentPlayerVisibleBehavior`
- **Evidence status:** `Confirmed`
- **Primary evidence:** `OFF-SERVER-001`; `net.minecraft.world.level.chunk.status.ChunkStatus`; `net.minecraft.world.level.chunk.status.ChunkPyramid`; `net.minecraft.world.level.chunk.ChunkGenerator#fillFromNoise(net.minecraft.world.level.levelgen.blending.Blender,net.minecraft.world.level.levelgen.RandomState,net.minecraft.world.level.StructureManager,net.minecraft.world.level.chunk.ChunkAccess)`; `net.minecraft.world.level.chunk.ChunkGenerator#buildSurface(net.minecraft.server.level.WorldGenRegion,net.minecraft.world.level.StructureManager,net.minecraft.world.level.levelgen.RandomState,net.minecraft.world.level.chunk.ChunkAccess)`; `net.minecraft.world.level.chunk.ChunkGenerator#applyCarvers(net.minecraft.server.level.WorldGenRegion,long,net.minecraft.world.level.levelgen.RandomState,net.minecraft.world.level.biome.BiomeManager,net.minecraft.world.level.StructureManager,net.minecraft.world.level.chunk.ChunkAccess)`; `COM-WIKI-WGEN-001`
- **Applies when:** A new/incomplete chunk is requested at a higher generation status.
- **Behavior and timing:** The `26.2` chain is `EMPTY → STRUCTURE_STARTS → STRUCTURE_REFERENCES → BIOMES → NOISE → SURFACE → CARVERS → FEATURES → INITIALIZE_LIGHT → LIGHT → SPAWN → FULL`. A stage runs only after its required neighborhood is ready and publishes a higher status. Asynchronous completion must not let later work observe half-written earlier work.
- **Boundaries and quirks:** Stages read neighboring chunks at different radii; structures and features may write outside the center chunk. Retry/loading an existing status must be idempotent and not place content twice.
- **Verification owner (`WGEN-PIPELINE-001`; `EXP-WGEN-*`):** Ferrite may use a different task graph, but needs integration tests for phase visibility, cancellation/retry, and cross-chunk revisions.

## `WGEN-002` Registry-parameterized noise produces biomes and base terrain

- **FidelityClass:** `EquivalentPlayerVisibleBehavior`
- **Evidence status:** `Confirmed`
- **Primary evidence:** `OFF-DATA-001`; `OFF-REPORT-001`; `OFF-SERVER-001`; `net.minecraft.world.level.biome.BiomeSource#getNoiseBiome(int,int,int,net.minecraft.world.level.biome.Climate$Sampler)`; `net.minecraft.world.level.levelgen.NoiseBasedChunkGenerator#fillFromNoise(net.minecraft.world.level.levelgen.blending.Blender,net.minecraft.world.level.levelgen.RandomState,net.minecraft.world.level.StructureManager,net.minecraft.world.level.chunk.ChunkAccess)`; `net.minecraft.world.level.levelgen.NoiseBasedChunkGenerator#buildSurface(net.minecraft.server.level.WorldGenRegion,net.minecraft.world.level.StructureManager,net.minecraft.world.level.levelgen.RandomState,net.minecraft.world.level.chunk.ChunkAccess)`; `COM-WIKI-WGEN-001`
- **Applies when:** BIOMES, NOISE, or SURFACE selects environment and blocks for a chunk.
- **Behavior and timing:** A biome source selects three-dimensional biomes from a climate/noise sampler. A noise generator uses the dimension's noise settings for base density/aquifers and terrain, then surface rules replace top layers. Registry data determines noise parameters, default block/fluid, sea level, surface rules, and biome effects.
- **Boundaries and quirks:** Cave biomes are three-dimensional; old/new chunk boundaries may use blending. A Data Pack may replace worldgen registries, so default Overworld constants cannot be embedded in a generic algorithm.
- **Verification owner (`WGEN-PIPELINE-001`; `EXP-WGEN-*`):** Equivalence acceptance uses distributions, elevation profiles, connectivity, and player paths rather than vanilla seed hashes; thresholds need a separate statistical baseline.

## `WGEN-003` Features and structures select, place, and reference across chunks separately

- **FidelityClass:** `EquivalentPlayerVisibleBehavior`
- **Evidence status:** `Confirmed`
- **Primary evidence:** `OFF-DATA-001`; `OFF-SERVER-001`; `net.minecraft.world.level.chunk.ChunkGenerator#createStructures(net.minecraft.core.RegistryAccess,net.minecraft.world.level.chunk.ChunkGeneratorStructureState,net.minecraft.world.level.StructureManager,net.minecraft.world.level.chunk.ChunkAccess,net.minecraft.world.level.levelgen.structure.templatesystem.StructureTemplateManager,net.minecraft.resources.ResourceKey)`; `net.minecraft.world.level.chunk.ChunkGenerator#createReferences(net.minecraft.world.level.WorldGenLevel,net.minecraft.world.level.StructureManager,net.minecraft.world.level.chunk.ChunkAccess)`; `net.minecraft.world.level.chunk.ChunkGenerator#applyBiomeDecoration(net.minecraft.world.level.WorldGenLevel,net.minecraft.world.level.chunk.ChunkAccess,net.minecraft.world.level.StructureManager)`; `net.minecraft.world.level.levelgen.structure.Structure#generate(net.minecraft.core.Holder,net.minecraft.resources.ResourceKey,net.minecraft.core.RegistryAccess,net.minecraft.world.level.chunk.ChunkGenerator,net.minecraft.world.level.biome.BiomeSource,net.minecraft.world.level.levelgen.RandomState,net.minecraft.world.level.levelgen.structure.templatesystem.StructureTemplateManager,long,net.minecraft.world.level.ChunkPos,int,net.minecraft.world.level.LevelHeightAccessor,java.util.function.Predicate)`; `net.minecraft.world.level.levelgen.structure.Structure#afterPlace(net.minecraft.world.level.WorldGenLevel,net.minecraft.world.level.StructureManager,net.minecraft.world.level.chunk.ChunkGenerator,net.minecraft.util.RandomSource,net.minecraft.world.level.levelgen.structure.BoundingBox,net.minecraft.world.level.ChunkPos,net.minecraft.world.level.levelgen.structure.pieces.PiecesContainer)`
- **Applies when:** STRUCTURE_STARTS/REFERENCES or FEATURES runs.
- **Behavior and timing:** Structure sets/placements first use salt, spacing/rules, biome, and generation point to create a start. Pieces may cross chunks; affected chunks retain references, and decoration places bounding-box slices before `afterPlace`. Placed features are ordered by biome generation step, select positions through placement modifiers, and invoke configured features.
- **Boundaries and quirks:** Structure spawn overrides, locate commands, and loot depend on starts/references rather than only visible blocks. Feature order can overwrite earlier results and cannot become unordered concurrent writes to one region.
- **Verification owner (`WGEN-PIPELINE-001`; `EXP-WGEN-*`):** Ferrite may use different seed mapping, but must statistically lock rarity, minimum spacing, biome constraints, cross-chunk completeness, and the allowed locate-result divergence.

## `WGEN-004` DimensionType locks height, sky, time, and coordinate-scale semantics

- **FidelityClass:** `ExactObservableBehavior`
- **Evidence status:** `Confirmed`
- **Primary evidence:** `OFF-DATA-001`; `OFF-SERVER-001`; `net.minecraft.world.level.dimension.DimensionType#coordinateScale()`; `net.minecraft.world.level.dimension.DimensionType#minY()`; `net.minecraft.world.level.dimension.DimensionType#height()`; `net.minecraft.world.level.dimension.DimensionType#logicalHeight()`; `net.minecraft.world.level.dimension.DimensionType#hasSkyLight()`; `COM-WIKI-WGEN-001`
- **Applies when:** A dimension is created or gameplay queries height, light, day cycle, beds/anchors, environmental effects, or cross-dimension coordinates.
- **Behavior and timing:** A dimension stem combines dimension type and chunk generator. Type data selects build height, logical height, coordinate scale, sky light, ceiling, ultrawarm/natural, fixed time, and bed/respawn-anchor rules. Each dimension owns chunks, entities, scheduled ticks, weather applicability, and day time separately.
- **Boundaries and quirks:** Logical height differs from storage height; fixed time does not stop game time. Data Pack custom dimensions must pass the same data validation.
- **Verification owner (`WGEN-PIPELINE-001`; `EXP-WGEN-*`):** Generate all property/boundary tests from the `dimension_type` registry, especially minY, highest legal block, and cross-dimension scale rounding.

## `WGEN-005` A portal accumulates eligibility, then searches for or creates an exit

- **FidelityClass:** `ExactObservableBehavior`
- **Evidence status:** `Cross-checked`
- **Primary evidence:** `OFF-SERVER-001`; `OFF-DATA-001`; `net.minecraft.world.entity.Entity#handlePortal()`; `net.minecraft.world.level.portal.PortalForcer#findClosestPortalPosition(net.minecraft.core.BlockPos,boolean,net.minecraft.world.level.border.WorldBorder)`; `net.minecraft.world.level.portal.PortalForcer#createPortal(net.minecraft.core.BlockPos,net.minecraft.core.Direction$Axis)`; `net.minecraft.world.level.dimension.DimensionType#getTeleportationScale(net.minecraft.world.level.dimension.DimensionType,net.minecraft.world.level.dimension.DimensionType)`; `COM-WIKI-WGEN-001`
- **Applies when:** An entity remains in a teleporting portal until wait time and is not on cooldown, or an End portal submits its transition.
- **Behavior and timing:** A portal processor records contact and progress, then creates a `TeleportTransition` at threshold. Nether travel scales coordinates between source/destination dimensions and clamps a candidate to the border, queries portal POIs within the rule radius, and if absent tries to create a rectangle under axis, space, and border rules. The entity is placed from entry-relative position and exit shape with rotation/velocity/cooldown policy.
- **Boundaries and quirks:** Player/non-player wait time, existing-portal search radius, Nether ceiling, blocked exit, creation failure, passengers, and End portals use different policies. A visible portal block does not imply its POI is searchable yet.
- **Verification owner (`WGEN-PIPELINE-001`; `EXP-WGEN-*`):** Exact wait/cooldown, search radius/order, coordinate rounding, exit pose, and failure fallback need vanilla GameTests, hence `Cross-checked`.

## `WGEN-006` The world border interpolates over game time and participates in collision, spawn, and teleport

- **FidelityClass:** `ExactObservableBehavior`
- **Evidence status:** `Confirmed`
- **Primary evidence:** `OFF-SERVER-001`; `net.minecraft.world.level.border.WorldBorder#lerpSizeBetween(double,double,long,long)`; `net.minecraft.world.level.border.WorldBorder#tick()`; `net.minecraft.world.level.border.WorldBorder#isWithinBounds(net.minecraft.world.phys.AABB)`; `net.minecraft.world.level.border.WorldBorder#clampToBounds(net.minecraft.world.phys.Vec3)`; `net.minecraft.world.level.border.WorldBorder#getDistanceToBorder(net.minecraft.world.entity.Entity)`; `COM-WIKI-WGEN-001`
- **Applies when:** Gameplay queries the horizontal world boundary, especially for movement, spawning, teleport, and explosions.
- **Behavior and timing:** The border stores center, current/target size, interpolation game times, warning, and damage parameters. `tick` updates a moving extent and replaces it with a static extent at completion. Queries test points, chunks, and AABBs against the current extent or clamp a target inside. Players outside take damage according to buffer and rate.
- **Boundaries and quirks:** Maximum block coordinates and configurable world border are distinct limits. A moving border can sweep over stationary entities. Client animation presents server parameters and does not own collision truth.
- **Verification owner (`WGEN-PIPELINE-001`; `EXP-WGEN-*`):** Lock AABB exact-touch inclusion, interpolation rounding, teleport clamp epsilon, and first damage tick when the border moves past an entity.

## `WGEN-007` World-generation compatibility is player-visible equivalence, not same-seed identity

- **FidelityClass:** `EquivalentPlayerVisibleBehavior`
- **Evidence status:** `Confirmed`
- **Primary evidence:** `OFF-DATA-001`; `OFF-REPORT-001`; `COM-WIKI-WGEN-001`; [docs/architecture.md#19-world-generation](../../../architecture.md#19-world-generation)
- **Applies when:** Ferrite implements or tests terrain, biomes, features, structures, and resource distributions.
- **Behavior and timing:** It must retain dimension identity, traversable terrain, surface/cave relationships, biome and structure constraints, resource-rarity order of magnitude, and gameplay dependencies such as spawning, locate, loot, and mob spawning. Different RNG, seed mixing, parallel generation tasks, and concrete block placement are allowed when defined statistical and scenario acceptance passes.
- **Boundaries and quirks:** Discrete runtime results such as mutations, structure trades, portal exit, and safe-spawn search are not automatically relaxed by this rule; their own `ExactObservableBehavior` rules still apply.
- **Verification owner (`WGEN-PIPELINE-001`; `EXP-WGEN-*`):** Define a quantitative threshold, sample-seed set, and failure diagnostic for every equivalence claim before implementation. “Looks similar” is not acceptance when thresholds are absent.
