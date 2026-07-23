# 03 — Fluids, Lighting, Weather, and Fire

Read content-specific flow delays, light values, burn probabilities, and dimension properties from
`OFF-DATA-001` / `OFF-REPORT-001`. This page specifies only generic ordering and gates.

## `ENV-001` Fluid state advances through a separate scheduled-tick queue

- **FidelityClass:** `ExactObservableBehavior`
- **Evidence status:** `Confirmed`

### SourceConclusion

`SourceSpecified` in `ENV-FLUID-001` and the potent-sulfur consumer `ENV-GEYSER-001`; their
experiments are conformance traces.

### Primary evidence

`OFF-SERVER-001`;
`net.minecraft.world.level.material.FlowingFluid#tick(net.minecraft.server.level.ServerLevel,net.minecraft.core.BlockPos,net.minecraft.world.level.block.state.BlockState,net.minecraft.world.level.material.FluidState)`;
`net.minecraft.world.level.material.FlowingFluid#spread(net.minecraft.server.level.ServerLevel,net.minecraft.core.BlockPos,net.minecraft.world.level.block.state.BlockState,net.minecraft.world.level.material.FluidState)`;
`net.minecraft.world.level.block.LiquidBlock#onPlace(net.minecraft.world.level.block.state.BlockState,net.minecraft.world.level.Level,net.minecraft.core.BlockPos,net.minecraft.world.level.block.state.BlockState,boolean)`;
`net.minecraft.world.level.block.LiquidBlock#updateShape(net.minecraft.world.level.block.state.BlockState,net.minecraft.world.level.LevelReader,net.minecraft.world.level.ScheduledTickAccess,net.minecraft.core.BlockPos,net.minecraft.core.Direction,net.minecraft.core.BlockPos,net.minecraft.world.level.block.state.BlockState,net.minecraft.util.RandomSource)`

### Applies when

A liquid block is placed, an adjacent state changes, or a fluid scheduled tick becomes due.

### Behavior and timing

Placement and shape changes schedule a fluid tick at the position. When due, the tick first computes
the position's new fluid state and then spreads downward and sideways when allowed. Writes generate
follow-up schedules through block-update rules. Scheduled block ticks drain before scheduled fluid
ticks in each server dimension.

### Boundaries and quirks

Fluids are not scanned across the whole world each tick. Their schedules suspend while a chunk is
inactive and resume through the queue after activation, without wall-time catch-up.

### Verification

**Owners:** `ENV-FLUID-001`, `ENV-GEYSER-001`, `BLK-SHELF-001`, `BLK-DECORATED-POT-001`,
`BLK-CONDUIT-001`, `BLK-STRUCTURE-VOID-001`, `BLK-AIR-001`; `EXP-ENV-001`, `EXP-ENV-005`, `EXP-BLK-013`,
`EXP-BLK-014`, `EXP-BLK-023`, `EXP-BLK-029`, `EXP-BLK-030`

Regress the specified block-before-fluid queue/live-state order, the geyser gates, and shelf/pot
waterlogged source and schedule projections.
The conduit leaf fixes its source-water state, shape-update schedule, dry-placement override and
27-position activation water gate without changing generic fluid queue semantics.
The structure-void leaf fixes the hard `canHoldAnyFluid` rejection for its otherwise noncolliding
state; replacement or removal exposes a later ordinary fluid update rather than waterlogging it.
The air leaf fixes empty fluid state for all three air identities and ordinary air as removal's
empty-fluid legacy-block result; it adds no waterlogging or fluid scheduling path.

## `ENV-002` Level, obstruction, source rules, and mixing hooks jointly select flow

- **FidelityClass:** `ExactObservableBehavior`
- **Evidence status:** `Confirmed`

### SourceConclusion

`SourceSpecified` in `ENV-FLUID-001`, including subtype constants, tie/write order, containers,
reactions and RNG; `ENV-GEYSER-001` owns the separate potent-sulfur source/obstruction consumer.

### Primary evidence

`OFF-SERVER-001`;
`net.minecraft.world.level.material.FlowingFluid#getNewLiquid(net.minecraft.server.level.ServerLevel,net.minecraft.core.BlockPos,net.minecraft.world.level.block.state.BlockState)`;
`net.minecraft.world.level.material.FlowingFluid#getSpread(net.minecraft.server.level.ServerLevel,net.minecraft.core.BlockPos,net.minecraft.world.level.block.state.BlockState)`;
`net.minecraft.world.level.material.FlowingFluid#getSpreadDelay(net.minecraft.world.level.Level,net.minecraft.core.BlockPos,net.minecraft.world.level.material.FluidState,net.minecraft.world.level.material.FluidState)`;
`net.minecraft.world.level.block.LiquidBlock#shouldSpreadLiquid(net.minecraft.world.level.Level,net.minecraft.core.BlockPos,net.minecraft.world.level.block.state.BlockState)`;
`COM-WIKI-ENV-001`

### Applies when

`FlowingFluid` computes a candidate for the current or neighboring position.

### Behavior and timing

The algorithm recomputes a nonsource locally, tries downward spread first, and otherwise selects
every horizontal candidate with the shortest reachable downward hole. Water uses drop 1/range
4/delay 5. Lava uses drop 2/range 2/delay 30 normally and drop 1/range 4/delay 10 when
`gameplay/fast_lava`; a rising nonfalling lava level has a 3/4 chance to multiply that delay by
four.

### Boundaries and quirks

Two admitted horizontal sources convert only when the matching source-conversion rule is enabled and
support below is solid or a same-family source. Waterlogging is exact interface dispatch. Lava-block
neighbor reactions, downward lava into water, and water replacing sufficiently deep lava are
distinct transactions.

### Verification

**Owners:** `ENV-FLUID-001`, `ENV-GEYSER-001`, `BLK-SHELF-001`, `BLK-DECORATED-POT-001`,
`BLK-CONDUIT-001`, `BLK-STRUCTURE-VOID-001`, `BLK-AIR-001`; `EXP-ENV-001`, `EXP-ENV-005`, `EXP-BLK-013`,
`EXP-BLK-014`, `EXP-BLK-023`, `EXP-BLK-029`, `EXP-BLK-030`

Regress exact flow candidates/reactions, geyser boundaries, and shelf/pot simple-waterlogged
interface dispatch.
`BLK-CONDUIT-001` additionally requires `isWaterAt` at the waterlogged conduit and every other cell
of its centered `3×3×3` activation volume; it does not invoke a distinct mixing algorithm.

## `ENV-003` Lighting propagates sky and block channels separately

- **FidelityClass:** `EquivalentPlayerVisibleBehavior`
- **Evidence status:** `Confirmed`

### SourceConclusion

`SourceInconclusive` only for the finite end-to-end render-latency bound under arbitrary load; all
propagation, section, publication, packet and client-import branches are source-specified in
`ENV-LIGHT-001`.

### Primary evidence

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-REPORT-001`;
`net.minecraft.world.level.lighting.LevelLightEngine#checkBlock(net.minecraft.core.BlockPos)`;
`net.minecraft.world.level.lighting.LightEngine#runLightUpdates()`;
`net.minecraft.server.level.ThreadedLevelLightEngine#runUpdate()`;
`net.minecraft.client.multiplayer.ClientLevel#pollLightUpdates()`

### Applies when

Block emission, opacity, sky visibility, or chunk light data changes.

### Behavior and timing

Each channel converges through complete decrease-before-increase waves using state emission,
at-least-one attenuation and joint face occlusion; block work precedes sky work, then changed
section layers publish atomically as a visible-map snapshot. Raw brightness is
`max(block, sky - (int)(15 - sky_light_level))`. Server work is chunk-prioritized/batched; packets
become bounded FIFO imports on the client before its local light run and render update.

### Boundaries and quirks

Sky uses direct level-15 column sources and empty-section border bridging; block light does not.
Missing/disabled layers have channel-specific values. Ferrite may use a different internal solver
only if converged values, server query/publication order, packet state and client-visible
equivalence match.

### Verification

**Owners:** `ENV-LIGHT-001`, `BLK-CONDUIT-001`, `BLK-BEACON-001`, `BLK-BEDROCK-001`,
`BLK-TINTED-GLASS-001`; `EXP-ENV-004`, `EXP-BLK-023`, `EXP-BLK-024`, `EXP-BLK-031`,
`EXP-BLK-033`

Measure mutation-to-first-rebuilt-frame latency under a named dispatcher/network/render load
profile; do not invent a universal one-tick/one-frame deadline.
`BLK-CONDUIT-001` fixes emission 15 for both waterlogged states; propagation and publication remain
owned by `ENV-LIGHT-001`.
`BLK-BEACON-001` fixes emission 15 plus the distinct vertical beam-obstruction test: dampening 15
blocks a scan except for bedrock, while colored beam blocks take their dedicated branch.
`BLK-BEDROCK-001` fixes the exact beacon exception and the light engine's bedrock-state return for
a missing lighting chunk without treating that sentinel as a world write.
`BLK-TINTED-GLASS-001` fixes false skylight propagation and dampening 15 despite its transparent
model, so beacon scanning terminates through the existing non-bedrock obstruction branch.

## `ENV-004` Weather targets are server-wide; strengths and local effects are per level

- **FidelityClass:** `ExactObservableBehavior`
- **Evidence status:** `Confirmed`

### Primary evidence

`OFF-SERVER-001`; `net.minecraft.server.MinecraftServer#getWeatherData()`;
`net.minecraft.server.MinecraftServer#setWeatherParameters(int,int,boolean,boolean)`;
`net.minecraft.server.level.ServerLevel#advanceWeatherCycle()`;
`net.minecraft.server.level.ServerLevel#tickThunder(net.minecraft.world.level.chunk.LevelChunk)`;
`net.minecraft.server.level.ServerLevel#tickPrecipitation(net.minecraft.core.BlockPos)`;
`net.minecraft.world.level.Level#precipitationAt(net.minecraft.core.BlockPos)`;
`net.minecraft.server.players.PlayerList#sendLevelInfo(net.minecraft.server.level.ServerPlayer,net.minecraft.server.level.ServerLevel)`

### Applies when

A normally running server level reaches its early weather phase or later processes eligible
chunk-local weather work.

### Behavior and timing

`WeatherData` is one server-owned saved record shared by all levels. Every weather-capable level
that ticks mutates those timers/target booleans in its early phase, but owns separate rain/thunder
strengths that approach the shared targets by `0.01F` per admitted level tick. Rain is active only
when the level's strength is greater than `0.2`; thunder is active only when
`thunderStrength * rainStrength` is greater than `0.9`. Chunk-local ice, snow, cauldron, and
lightning branches use those active predicates plus their own biome, exposure, activity, gamerule,
and random gates.

### Boundaries and quirks

Weather capability is exactly `has_skylight && !has_ceiling && dimension != minecraft:the_end`. A
custom server with multiple capable dimensions advances the shared timers once per capable level
while their strengths can differ. Commands mutate the shared target record without immediately
changing strengths; sleep resets it after that level's early weather phase. Target rain does not
imply active or local precipitation.

### Verification

**Owners:** `ENV-WEATHER-001`; `EXP-ENV-002`

The leaf fixes timer distributions, local predicates, lightning selection, packet scope,
command/sleep order, and chunk phase; the experiment is a regression trace.

## `ENV-005` Ordinary fire is a self-scheduled state machine gated by nearby nonspectators

- **FidelityClass:** `ExactObservableBehavior`
- **Evidence status:** `Confirmed`

### Primary evidence

`OFF-SERVER-001`;
`net.minecraft.world.level.block.BaseFireBlock#getState(net.minecraft.world.level.BlockGetter,net.minecraft.core.BlockPos)`;
`net.minecraft.world.level.block.BaseFireBlock#onPlace(net.minecraft.world.level.block.state.BlockState,net.minecraft.world.level.Level,net.minecraft.core.BlockPos,net.minecraft.world.level.block.state.BlockState,boolean)`;
`net.minecraft.world.level.block.FireBlock#onPlace(net.minecraft.world.level.block.state.BlockState,net.minecraft.world.level.Level,net.minecraft.core.BlockPos,net.minecraft.world.level.block.state.BlockState,boolean)`;
`net.minecraft.world.level.block.FireBlock#tick(net.minecraft.world.level.block.state.BlockState,net.minecraft.server.level.ServerLevel,net.minecraft.core.BlockPos,net.minecraft.util.RandomSource)`;
`net.minecraft.world.level.block.FireBlock#canSurvive(net.minecraft.world.level.block.state.BlockState,net.minecraft.world.level.LevelReader,net.minecraft.core.BlockPos)`;
`net.minecraft.world.level.block.FireBlock#isNearRain(net.minecraft.world.level.Level,net.minecraft.core.BlockPos)`;
`net.minecraft.server.level.ServerLevel#canSpreadFireAround(net.minecraft.core.BlockPos)`;
`net.minecraft.server.level.ChunkMap#anyPlayerCloseEnoughTo(net.minecraft.core.BlockPos,int)`

### Applies when

Ordinary `fire` is placed, receives a shape update, or reaches its scheduled callback; base-fire
placement/contact dispatch also selects ordinary versus soul fire and portal creation.

### Behavior and timing

Placement schedules ordinary fire after `30 + nextInt(10)` game ticks. Every due callback first
schedules its successor, then `fire_spread_radius_around_player` admits the remaining survival,
rain, age, direct fuel-burn, and empty-space spread branches. Concrete locked ignite/burn values,
the positional `gameplay/increased_fire_burnout` attribute, difficulty, rain exposure, and the
dimension type's infiniburn set determine those ordered branches. There is no `doFireTick` game rule
in Java 26.2.

### Boundaries and quirks

The radius rule defaults to `128`, has minimum `-1`, and uses strict Euclidean distance from a
nonspectator's position to the fire block's integer corner; `-1` bypasses the search. A denied
callback still reschedules. Soul fire has support/contact behavior but no ordinary fire age or
scheduled-spread callback. Fire-started portal construction is dispatched to `WGEN-PORTAL-001`.

### Verification

**Owners:** `ENV-FIRE-001`, `BLK-SHELF-001`, `BLK-BEDROCK-001`; `EXP-ENV-003`,
`EXP-BLK-013`, `EXP-BLK-031`

The fire leaf fixes every callback branch and fuel table; the shelf leaf audits its ten `(30,20)`
fuel registrations and exact crimson/warped exclusion. The bedrock leaf fixes its added
`infiniburn_end` membership; fire scheduling, neighboring burn/spread and RNG remain here.

## `ENV-006` Chunk environment work and natural spawning share activity constraints, not a phase

- **FidelityClass:** `ExactObservableBehavior`
- **Evidence status:** `Confirmed`

### Primary evidence

`OFF-SERVER-001`;
`net.minecraft.server.level.ServerChunkCache#tickChunks(net.minecraft.util.profiling.ProfilerFiller,long)`;
`net.minecraft.server.level.ServerChunkCache#tickSpawningChunk(net.minecraft.world.level.chunk.LevelChunk,long,java.util.List,net.minecraft.world.level.NaturalSpawner$SpawnState)`;
`net.minecraft.server.level.ServerLevel#tickChunk(net.minecraft.world.level.chunk.LevelChunk,int)`;
`net.minecraft.server.level.ServerLevel#tickThunder(net.minecraft.world.level.chunk.LevelChunk)`;
`net.minecraft.server.level.ServerLevel#tickPrecipitation(net.minecraft.core.BlockPos)`

### Applies when

The server chunk source selects a chunk for active-chunk work in the current tick.

### Behavior and timing

The chunk source first builds and shuffles its spawning-chunk list. For each such chunk it
increments inhabited time, attempts thunder when the chunk is in entity-ticking range, then performs
admitted natural spawning. Only after every spawning chunk does it traverse block-ticking chunks;
each `tickChunk` performs precipitation/freezing attempts before bottom-to-top random block/fluid
ticks. Custom spawners run last when `spawn_mobs` is true.

### Boundaries and quirks

Thunder still runs when `spawn_mobs` is false; that rule suppresses its skeleton-trap roll and
natural/custom spawning, not the lightning attempt. Precipitation attempts are controlled by
`random_tick_speed`: each block-ticking chunk consumes that many `nextInt(48)` draws, and only zero
advances the separate block-position stream and calls the precipitation branch. Speed zero therefore
suppresses freezing, snow, and cauldron callbacks as well as random block/fluid ticks.

### Verification

**Owners:** `ENV-WEATHER-001`; `EXP-ENV-002`

The experiment locks the already source-specified list/phase ordering and distinct RNG streams as a
regression trace.
