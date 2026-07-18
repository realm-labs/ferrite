# 03 — Fluids, Lighting, Weather, and Fire

Read content-specific flow delays, light values, burn probabilities, and dimension properties from `OFF-DATA-001` / `OFF-REPORT-001`. This page specifies only generic ordering and gates.

## `ENV-001` Fluid state advances through a separate scheduled-tick queue

- **FidelityClass:** `ExactObservableBehavior`
- **Evidence status:** `Confirmed`
- **Primary evidence:** `OFF-SERVER-001`; `net.minecraft.world.level.material.FlowingFluid#tick(net.minecraft.server.level.ServerLevel,net.minecraft.core.BlockPos,net.minecraft.world.level.block.state.BlockState,net.minecraft.world.level.material.FluidState)`; `net.minecraft.world.level.material.FlowingFluid#spread(net.minecraft.server.level.ServerLevel,net.minecraft.core.BlockPos,net.minecraft.world.level.block.state.BlockState,net.minecraft.world.level.material.FluidState)`; `net.minecraft.world.level.block.LiquidBlock#onPlace(net.minecraft.world.level.block.state.BlockState,net.minecraft.world.level.Level,net.minecraft.core.BlockPos,net.minecraft.world.level.block.state.BlockState,boolean)`; `net.minecraft.world.level.block.LiquidBlock#updateShape(net.minecraft.world.level.block.state.BlockState,net.minecraft.world.level.LevelReader,net.minecraft.world.level.ScheduledTickAccess,net.minecraft.core.BlockPos,net.minecraft.core.Direction,net.minecraft.core.BlockPos,net.minecraft.world.level.block.state.BlockState,net.minecraft.util.RandomSource)`
- **Applies when:** A liquid block is placed, an adjacent state changes, or a fluid scheduled tick becomes due.
- **Behavior and timing:** Placement and shape changes schedule a fluid tick at the position. When due, the tick first computes the position's new fluid state and then spreads downward and sideways when allowed. Writes generate follow-up schedules through block-update rules. Scheduled block ticks drain before scheduled fluid ticks in each server dimension.
- **Boundaries and quirks:** Fluids are not scanned across the whole world each tick. Their schedules suspend while a chunk is inactive and resume through the queue after activation, without wall-time catch-up.
- **Verification owner (`ENV-FLUID-001`; `EXP-ENV-*`):** Lock neighbor-queue ordering when support blocks, container blocks, and fluid all change in one tick.

## `ENV-002` Level, obstruction, source rules, and mixing hooks jointly select flow

- **FidelityClass:** `ExactObservableBehavior`
- **Evidence status:** `Confirmed`
- **Primary evidence:** `OFF-SERVER-001`; `net.minecraft.world.level.material.FlowingFluid#getNewLiquid(net.minecraft.server.level.ServerLevel,net.minecraft.core.BlockPos,net.minecraft.world.level.block.state.BlockState)`; `net.minecraft.world.level.material.FlowingFluid#getSpread(net.minecraft.server.level.ServerLevel,net.minecraft.core.BlockPos,net.minecraft.world.level.block.state.BlockState)`; `net.minecraft.world.level.material.FlowingFluid#getSpreadDelay(net.minecraft.world.level.Level,net.minecraft.core.BlockPos,net.minecraft.world.level.material.FluidState,net.minecraft.world.level.material.FluidState)`; `net.minecraft.world.level.block.LiquidBlock#shouldSpreadLiquid(net.minecraft.world.level.Level,net.minecraft.core.BlockPos,net.minecraft.world.level.block.state.BlockState)`; `COM-WIKI-ENV-001`
- **Applies when:** `FlowingFluid` computes a candidate for the current or neighboring position.
- **Behavior and timing:** The algorithm combines fluid above and beside, horizontal distance, downward exits, face occlusion, replaceability, and the concrete fluid's source-conversion and slope parameters. Contact between fluid types may invoke a `LiquidBlock` hook that creates a solid before ordinary spread. Delay may change with old and new state.
- **Boundaries and quirks:** Waterlogging, source conversion, lava's dimension differences, and concrete mixing products are content rules, not consequences of one generic flood fill.
- **Verification owner (`ENV-FLUID-001`; `EXP-ENV-*`):** Build data-driven golden cases for water/lava source conversion, edge flow, waterlogging, and mixing. The generic rule is confirmed; data/subclasses lock the numbers.

## `ENV-003` Lighting propagates sky and block channels separately

- **FidelityClass:** `EquivalentPlayerVisibleBehavior`
- **Evidence status:** `Confirmed`
- **Primary evidence:** `OFF-SERVER-001`; `net.minecraft.world.level.lighting.LevelLightEngine#checkBlock(net.minecraft.core.BlockPos)`; `net.minecraft.world.level.lighting.LevelLightEngine#runLightUpdates()`; `net.minecraft.world.level.lighting.LevelLightEngine#propagateLightSources(net.minecraft.world.level.ChunkPos)`; `net.minecraft.world.level.lighting.LevelLightEngine#getRawBrightness(net.minecraft.core.BlockPos,int)`; `COM-WIKI-ENV-001`
- **Applies when:** Block emission, opacity, sky visibility, or chunk light data changes.
- **Behavior and timing:** Vanilla maintains separate sky-light and block-light propagation. A block change queues light-engine work, which may complete asynchronously across section/chunk boundaries. Gameplay brightness queries combine the channels with environmental darkening.
- **Boundaries and quirks:** Old light can be briefly visible between mutation and propagation completion; chunks with incomplete light have readiness gates. Ferrite may use a different propagation algorithm, but gameplay queries, chunk-visible results, and convergence boundaries must be equivalent.
- **Verification owner (`ENV-FLUID-001`; `EXP-ENV-*`):** Observe the permitted tick/frame latency under generation and network concurrency before turning this into an exact-timing requirement.

## `ENV-004` Weather advances per dimension; local effects test the position

- **FidelityClass:** `ExactObservableBehavior`
- **Evidence status:** `Confirmed`
- **Primary evidence:** `OFF-SERVER-001`; `net.minecraft.server.level.ServerLevel#advanceWeatherCycle()`; `net.minecraft.server.level.ServerLevel#tickThunder(net.minecraft.world.level.chunk.LevelChunk)`; `net.minecraft.server.level.ServerLevel#tickPrecipitation(net.minecraft.core.BlockPos)`; `COM-WIKI-ENV-001`
- **Applies when:** The dimension permits weather and weather timers, precipitation, or thunder logic runs.
- **Behavior and timing:** The server owns rain/thunder state and interpolation strengths, advancing them early in the level tick. Actual precipitation, lightning, ice/snow, and rain collection additionally test biome, sky visibility, height, temperature, chunk activity, and random conditions.
- **Boundaries and quirks:** Global rain does not imply precipitation at every position; a dimension type or biome may suppress the local result. Sleep time-skipping has a separate weather-reset path.
- **Verification owner (`ENV-FLUID-001`; `EXP-ENV-*`):** Minimal-world experiments must lock timer distributions, lightning target choice, and same-tick sleep/command ordering.

## `ENV-005` Fire uses scheduled ticks, survival tests, and a near-player spread gate

- **FidelityClass:** `ExactObservableBehavior`
- **Evidence status:** `Confirmed`
- **Primary evidence:** `OFF-SERVER-001`; `net.minecraft.world.level.block.FireBlock#onPlace(net.minecraft.world.level.block.state.BlockState,net.minecraft.world.level.Level,net.minecraft.core.BlockPos,net.minecraft.world.level.block.state.BlockState,boolean)`; `net.minecraft.world.level.block.FireBlock#tick(net.minecraft.world.level.block.state.BlockState,net.minecraft.server.level.ServerLevel,net.minecraft.core.BlockPos,net.minecraft.util.RandomSource)`; `net.minecraft.world.level.block.FireBlock#canSurvive(net.minecraft.world.level.block.state.BlockState,net.minecraft.world.level.LevelReader,net.minecraft.core.BlockPos)`; `net.minecraft.world.level.block.FireBlock#isNearRain(net.minecraft.world.level.Level,net.minecraft.core.BlockPos)`; `net.minecraft.server.level.ServerLevel#canSpreadFireAround(net.minecraft.core.BlockPos)`; `COM-WIKI-ENV-001`
- **Applies when:** A fire block is placed or its scheduled tick becomes due, and the relevant fire game rules permit spreading.
- **Behavior and timing:** Fire schedules itself after placement. When due, it checks support/flammable neighbors, survival, rain, and humidity, then uses concrete blocks' ignite/burn probabilities to consume blocks or create fire. `canSpreadFireAround` also gates near-player spreading with `FIRE_SPREAD_RADIUS_AROUND_PLAYER`.
- **Boundaries and quirks:** Permanent supports, portal ignition, rain shelter, dimension, and block-specific burn values alter branches. Fire survival and outward spread are not one test.
- **Verification owner (`ENV-FLUID-001`; `EXP-ENV-*`):** Lock the default game-rule value, no-player boundary, fire age, and schedule-delay distribution from reports/experiments rather than guessing in this generic rule.

## `ENV-006` Chunk environment work and natural spawning share activity constraints, not a phase

- **FidelityClass:** `ExactObservableBehavior`
- **Evidence status:** `Cross-checked`
- **Primary evidence:** `OFF-SERVER-001`; `net.minecraft.server.level.ServerLevel#tickChunk(net.minecraft.world.level.chunk.LevelChunk,int)`; `net.minecraft.server.level.ServerLevel#tickThunder(net.minecraft.world.level.chunk.LevelChunk)`; `net.minecraft.server.level.ServerLevel#tickPrecipitation(net.minecraft.core.BlockPos)`; `net.minecraft.server.level.ServerChunkCache#tickChunks()`; `COM-WIKI-SIM-001`
- **Applies when:** The server chunk source selects a chunk for active-chunk work in the current tick.
- **Behavior and timing:** Thunder candidates, precipitation effects, random block/fluid ticks, and natural spawning are constrained by active chunks and player distance, but use distinct calls and random samples. Disabling one game rule must not incidentally disable unrelated environment systems.
- **Boundaries and quirks:** Simulation distance, spectator players, forced chunks, and `spawnChunkRadius` may affect different sets. “Chunk ticking” cannot be one indistinguishable on/off switch.
- **Verification owner (`ENV-FLUID-001`; `EXP-ENV-*`):** The traversal order between natural spawning and per-chunk environment ticking inside `ServerChunkCache#tickChunks()` still needs GameTest or instrumentation confirmation.
