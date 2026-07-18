# Environment leaf rules

## Leaf rule `ENV-LIGHT-001` — Sky and block light propagate as separate bounded channels

**Parent:** `ENV-003`  
**Applies when:** A block state, section status, sky exposure, or light-emission value changes and the light engine is asked to reconcile affected positions.  
**Authoritative state:** Per-position sky/block light levels, block opacity and shape-dependent occlusion, emission, section readiness, queued increase/decrease work and dimension skylight capability.  
**Transition and ordering:** Enqueue the changed position/channel; process removal/decrease work so values unsupported by their old source are invalidated, then propagate surviving/new increases through neighboring positions using channel-specific source and attenuation rules; retain queued work for unavailable sections; publish section light only after its queued changes reach the engine's completion boundary. `net.minecraft.world.level.lighting.LevelLightEngine#checkBlock(net.minecraft.core.BlockPos)` and `net.minecraft.world.level.lighting.LevelLightEngine#runLightUpdates()` anchor the public work boundary.  
**Branches and aborts:** Dimension lacks skylight; section not ready; unchanged source/occlusion; neighbor fully blocks the path; candidate level is not greater than stored level; old source removal requires recomputation; chunk unload cancels publication but must not leak stale cross-section values.  
**Constants and randomness:** Both channels use integer level 0–15. Propagation consumes no RNG. Attenuation/emission and occlusion depend on the current block state and shapes; exact queue/direction tie order is not a gameplay contract unless it changes the final published light or same-tick observable spawn/melt decision.  
**Side effects:** Stored light arrays, dirty section/chunk tracking, client light updates, and later mechanics that query brightness for spawning, growth, melting, visibility or rendering. Light reconciliation does not itself invoke every light-sensitive mechanic; those query on their own phase.  
**Gates:** Dimension skylight, chunk/section light status, block emission/occlusion shape, work budget/completion, chunk loading and client tracking.  
**Boundary cases and quirks:** Sky light and block light may differ at one position. Removing a source requires a decrease wave before alternative sources restore values. A client may briefly render stale light while authoritative server queries wait on or use its light-engine state.  
**Evidence:** `Confirmed`; `OFF-SERVER-001`; locators above; section-boundary and same-tick query ordering are owned by `EXP-ENV-004`.  
**Test vectors:** Add/remove one emitter; overlap two sources; change a shape without changing block ID; open/close sky exposure; cross a section/chunk boundary; query spawn/melt brightness before and after the engine completion boundary.

## Leaf rule `ENV-FLUID-001` — Fluid propagation recomputes local state through scheduled ticks

**Parent:** `ENV-001`, `ENV-002`  
**Applies when:** A water/lava fluid state receives a scheduled tick or a neighboring change requests reevaluation.  
**Authoritative state:** Fluid type/amount/falling state at each position, containing block state, neighbor fluid states, dimension, gamerules, and the fluid tick queue.  
**Transition and ordering:** A neighbor/state change schedules the fluid using its type-specific delay. On execution, recompute the locally correct source/flowing state from surrounding sources, downward path and horizontal spread; replace the current fluid/block if changed; schedule or notify affected neighbors. Source conversion and lava/water reactions are evaluated at their defined branch points rather than as a global flood fill. Anchors: `net.minecraft.world.level.material.FlowingFluid#tick(net.minecraft.server.level.ServerLevel,net.minecraft.core.BlockPos,net.minecraft.world.level.block.state.BlockState,net.minecraft.world.level.material.FluidState)` and `net.minecraft.world.level.material.FlowingFluid#getNewLiquid(net.minecraft.server.level.ServerLevel,net.minecraft.core.BlockPos,net.minecraft.world.level.block.state.BlockState)`.  
**Branches and aborts:** Stable state; blocked face; nonreplaceable destination; downward flow available; horizontal slope choice; infinite-source condition; waterlogged container; lava/water reaction; ultrawarm evaporation on placement. Each destination is validated at commit time.  
**Constants and randomness:** Amount/level is discrete and subtype-defined. Tick delay, slope-search distance, drop-off, source conversion and dimension effects come from the locked fluid type and level/dimension data; query them or inspect the listed methods. Direction tie behavior and random consumption are owned by `EXP-ENV-001`.  
**Side effects:** Block/fluid replacement, scheduled fluid ticks, neighbor and shape updates, stone/cobblestone/basalt-style reactions where applicable, fizz/sound/particles and bucket interaction results.  
**Gates:** Chunk ticking, fluid type, dimension ultrawarm property, `waterSourceConversion`/`lavaSourceConversion` gamerules where read, block replaceability and waterlogging capability.  
**Boundary cases and quirks:** Fluid states coexist with waterlogged blocks; “empty block” and “empty fluid” are not synonyms. Missed unloaded ticks do not simulate every intermediate spread step on reload. Horizontal candidates must use vanilla slope selection, not breadth-first distance alone.  
**Evidence:** `Confirmed` algorithm boundary; subtype ordering `Cross-checked`; `OFF-SERVER-001`; locators above; `EXP-ENV-001`.  
**Test vectors:** Source over open air, source in a basin, competing equal slopes, waterlogged block, source conversion enabled/disabled, lava beside/above water in each dimension, unload before scheduled execution.

## Leaf rule `ENV-WEATHER-001` — Weather timers and exposed local effects are separate layers

**Parent:** `ENV-004`, `ENV-006`  
**Applies when:** A server level advances weather and later processes chunk-local precipitation/lightning effects.  
**Authoritative state:** Clear/rain/thunder timers and booleans, interpolation levels, dimension weather capability, biome precipitation/temperature, heightmap exposure and random source.  
**Transition and ordering:** Advance level-wide clear/rain/thunder timers and toggle target states when timers expire; move rain/thunder levels toward targets; during eligible chunk environment work, sample exposed positions, apply precipitation fill/extinguish/freezing/snow rules and possibly select lightning. `net.minecraft.server.level.ServerLevel#advanceWeatherCycle()` controls the global cycle; local callbacks consume the current state.  
**Branches and aborts:** Dimension has no weather; clear-weather override; gamerule denial; biome has no precipitation; position is covered or temperature-ineligible; lightning chance misses; no valid lightning target. Sleeping/time skipping may clear weather through its own branch.  
**Constants and randomness:** Timers are integer ticks. Transition rates, random duration ranges and lightning probability are source constants; do not infer from UI. RNG is consumed only by branches reached during eligible chunk processing. Exact ranges are assigned to `EXP-ENV-002` until symbol/control-flow extraction records them.  
**Side effects:** Rain/thunder state packets, sky/light consequences, cauldron fill, fire extinguish, freezing/snow placement, lightning entity spawn, sounds and advancement/game-event consequences.  
**Gates:** Dimension type, `doWeatherCycle`, chunk block-ticking status, biome, heightmap/exposure, difficulty or gamerules for particular lightning consequences.  
**Boundary cases and quirks:** Visual rain at a client is derived from global level plus local biome/exposure and is not proof that precipitation callbacks can occur at that position. Thunder requires rain state but its timer remains distinct.  
**Evidence:** `Confirmed` layering; exact distributions `Implementation blocker` via `EXP-ENV-002`; `OFF-SERVER-001`; `ServerLevel#advanceWeatherCycle()`.  
**Test vectors:** Disable cycle during active rain; assert state freezes. Compare covered/exposed positions in rainy and dry biomes. Run deterministic chunk ticks until lightning and record RNG consumption and target selection.

## Leaf rule `ENV-FIRE-001` — Fire aging, extinguishing, spread, and fuel destruction are ordered random-tick branches

**Parent:** `ENV-005`  
**Applies when:** A fire block receives its scheduled/random tick and remains in a loaded, eligible position.  
**Authoritative state:** Fire age/state, supporting/neighbor blocks and flammability tables, rain exposure, difficulty, gamerules, dimension/portal special cases and RNG.  
**Transition and ordering:** Validate survival and rain extinguish conditions; update/schedule fire age; attempt burning of directionally adjacent fuel with direction-specific odds; attempt spatial spread candidates using local encouragement, age, humidity and difficulty; replace/destroy targets and notify neighbors. Soul fire and portal ignition dispatch through their special state rules rather than ordinary fuel spread. Anchor: `net.minecraft.world.level.block.FireBlock#tick(net.minecraft.world.level.block.state.BlockState,net.minecraft.server.level.ServerLevel,net.minecraft.core.BlockPos,net.minecraft.util.RandomSource)`.  
**Branches and aborts:** `doFireTick` false; cannot survive; rain extinguish succeeds; eternal support; target immune/nonflammable; spread roll misses; target placement invalid; TNT-like target invokes its ignition behavior.  
**Constants and randomness:** Age is bounded discrete state; scheduled delay and every ignition/burn roll consume RNG at their branch sites. Flammability/encouragement are block-family values, not a universal hardness-derived formula. Exact roll ordering is `EXP-ENV-003`.  
**Side effects:** Fire state aging/removal/placement, fuel block removal or transformed state, TNT ignition, scheduled ticks, sounds/particles/game events and neighbor updates. Item drops follow the burn path and gamerules rather than ordinary player destruction.  
**Gates:** `doFireTick`, chunk activity, rain/exposure, difficulty, biome humidity, block face flammability, eternal support and dimension portal rules.  
**Boundary cases and quirks:** Fire may persist without spreading. Rain checks neighboring exposure as defined by source, not merely precipitation at the fire coordinate. Directional flammability matters.  
**Evidence:** `Confirmed`; `OFF-SERVER-001`; `FireBlock#tick`; data/source family values; RNG trace `EXP-ENV-003`.  
**Test vectors:** Fire on eternal and ordinary support; disable fire tick; expose to rain from each side; surround with fuels of different face flammability; ignite TNT; replay fixed RNG and compare every consumed roll.
