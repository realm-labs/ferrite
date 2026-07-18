# World generation and dimension leaf rules

## Leaf rule `WGEN-PIPELINE-001` — Chunk generation advances status-by-status with locked data inputs

**Parent:** `WGEN-001`, `WGEN-002`, `WGEN-003`, `WGEN-007`  
**FidelityClass:** `EquivalentPlayerVisibleBehavior`  <br>
**EvidenceStatus:** `Confirmed`  <br>
**SourceConclusion:** `SourceInconclusive` — pipeline boundaries are located, but quantitative equivalence thresholds and separate terrain/feature/structure slices do not yet exist.  <br>
**Applies when:** A chunk is requested at a generation status beyond its current status.  
**Authoritative state:** World seed/random state, dimension/noise settings, biome source, chunk status and neighboring status dependencies, carvers/features/structures and retrogen state.  
**Transition and ordering:** Resolve required neighbor region; run each missing chunk status in the version-defined order; assign biomes/noise terrain/surface/carvers/features/lighting/spawn/finalization through the status tasks; seed each positional random derivation exactly where the algorithm requests it; persist the completed status before exposing later dependencies. Definitions under `data/minecraft/worldgen` parameterize these algorithms.  
**Branches and aborts:** Already complete; neighbor dependency unavailable; structure placement exclusion/frequency; biome predicate; feature placement modifier fails; carver mask; dimension-specific generator; upgrade/retrogen path. Failed feature attempts do not roll back earlier features.  
**Constants and randomness:** Ferrite's contract is `EquivalentPlayerVisibleBehavior`, not same-seed block identity. Nevertheless, all observable distributions, ordering dependencies, bounds and data values must be source/data-based. RNG streams and exact same-seed parity are not acceptance requirements unless a later rule upgrades fidelity.  
**Side effects:** Chunk sections/heightmaps, biomes, structures/references, block entities, scheduled post-processing, light data, entities for worldgen hooks and later population eligibility.  
**Gates:** Dimension/generator settings, chunk status dependencies, biome/tags, structure sets, feature placement predicates, world seed and enabled data packs/features.  
**Boundary cases and quirks:** Features may read/modify neighboring chunks and earlier features. Parallel execution must preserve status dependency and externally visible determinism. A data JSON is not itself the algorithm.  
**Evidence:** `Confirmed` pipeline/data architecture; `EquivalentPlayerVisibleBehavior`; `OFF-SERVER-001`, `OFF-DATA-001`; catalog snapshot; distribution suite `EXP-WGEN-001`.  
**Test vectors:** Generate boundaries in different request orders; restart between statuses; compare biome/height/structure/feature distributions over fixed sample regions; enable optional pack separately; assert no order-dependent visible seams.

## Leaf rule `WGEN-DIMENSION-001` — Dimension type gates time scale, environment, coordinates, and spawn semantics

**Parent:** `WGEN-004`  
**FidelityClass:** `ExactObservableBehavior`  <br>
**EvidenceStatus:** `Confirmed`  <br>
**SourceConclusion:** `SourceInconclusive` — individual property consumers and coordinate rounding/clamping branches remain unexpanded.  <br>
**Applies when:** A level is created or a mechanic queries dimension-type properties.  
**Authoritative state:** Dimension key, dimension type holder/data, level stem/generator, coordinate scale, logical height/min Y, ultrawarm/natural/skylight/ceiling/bed-anchor rules and fixed time.  
**Transition and ordering:** Load the dimension type and level stem; create the level with its height/environment properties; mechanics query those properties at their branch point. Cross-dimension travel converts horizontal coordinates by source/destination coordinate scale, then clamps/searches according to portal/world-border rules. Fixed-time dimensions expose the configured celestial time rather than advancing visible day cycle.  
**Branches and aborts:** Natural versus fixed time; skylight/no skylight; ultrawarm; bed/respawn-anchor allowed or explosive; raids/piglin safety; coordinate scale; height bounds; missing destination.  
**Constants and randomness:** The four locked dimension-type IDs and all numeric properties are DataOnly inputs via `mc-ref query dimension_type <id>`. Coordinate conversion uses doubles and destination border clamping; portal placement later floors/rounds as its algorithm specifies.  
**Side effects:** Sky/time/weather behavior, fluid evaporation/flow settings, block interactions, spawning/light, portal coordinate and safe-position search, death/respawn behavior and client level transition.  
**Gates:** Dimension type fields/tags, level key, gamerules, world border, portal availability and feature/data-pack definitions.  
**Boundary cases and quirks:** Dimension key and dimension type are separate: custom dimensions can reuse a type. Coordinate scale affects X/Z, not Y. Fixed visual time does not freeze server gameplay.  
**Evidence:** `Confirmed`; `OFF-DATA-001`, `OFF-SERVER-001`; catalog snapshot; coordinate boundary `EXP-WGEN-002`.  
**Test vectors:** Query all four type IDs; scale positive/negative/fractional X/Z near border; fixed time while game time advances; water placement ultrawarm; beds/anchors; custom dimension reusing a vanilla type.

## Leaf rule `WGEN-PORTAL-001` — Portal travel is cooldown, destination transform, search, creation, and safe placement

**Parent:** `WGEN-005`  
**FidelityClass:** `ExactObservableBehavior`  <br>
**EvidenceStatus:** `Cross-checked`  <br>
**SourceConclusion:** `SourceInconclusive` — per-portal wait/cooldown, search order/radii, creation, exit pose, and failure branches remain unexpanded.  <br>
**Applies when:** An eligible entity remains in or contacts a Nether portal, End portal, or gateway and the corresponding transfer path is enabled.  
**Authoritative state:** Portal contact/inside timer and cooldown, source/destination level, entry position/axis/relative coordinates, world border, existing portal POIs/exit records and passenger graph.  
**Transition and ordering:** Record portal contact; advance wait timer where required; when eligible build destination transition; transform/clamp coordinates; search the version-defined radius/order for an existing destination portal or gateway exit; create a portal only on paths that permit it; compute exit position/rotation/velocity and collision-safe placement; transfer entity and set cooldown. Nether, End portal and gateway dispatch different algorithms.  
**Branches and aborts:** Passenger/cannot-change-dimensions; cooldown; insufficient portal time; destination missing; existing portal found; creation allowed/fails; border clamp; no safe exit; End-specific spawn/return route; gateway exact/chorus-style destination logic.  
**Constants and randomness:** Wait/cooldown, coordinate scale, search/create radius and POI ordering are source constants/properties. Placement uses exact floor/clamp rules. Randomness is limited to branches that explicitly choose/search candidates; exact POI tie order is `EXP-WGEN-003`.  
**Side effects:** Portal blocks/POIs if created, chunk tickets, old/new entity tracking, passenger handling, position/rotation/velocity, cooldown, sounds/particles/game events and advancement triggers.  
**Gates:** Entity transfer ability, portal type, dimension destination, cooldown/time, border/build bounds, chunk availability, creation rules and server configuration.  
**Boundary cases and quirks:** Touching and completing portal travel are distinct. Portal search is not simply nearest Euclidean block. Passenger roots and players have special transfer handling.  
**Evidence:** `Confirmed` state-machine split; tie/search constants `Cross-checked`; `OFF-SERVER-001`, `OFF-DATA-001`; `EXP-WGEN-003`.  
**Test vectors:** Enter/leave/reenter around wait boundary; cooldown; coordinate/border extremes; two equidistant portals; blocked destination creation; player with passenger; momentum/orientation; End gateway versus Nether portal.

## Leaf rule `WGEN-BORDER-001` — World border is a tick-interpolated geometry used by independent mechanics

**Parent:** `WGEN-006`
**FidelityClass:** `ExactObservableBehavior`  <br>
**EvidenceStatus:** `Confirmed`  <br>
**SourceConclusion:** `SourceSpecified` — 26.2 stores the configured duration as a tick countdown and decrements it once per `WorldBorder#tick()`.  <br>
**Applies when:** Border state changes or a mechanic checks containment/distance/damage.
**Authoritative state:** Center, previous/current/target size, configured duration, remaining interpolation ticks, absolute max size, warning distance/time, damage safe zone and damage rate.
**Transition and ordering:** Set-center or set-size mutates border state. Each `WorldBorder#tick()` calls the moving extent's update: decrement remaining ticks by one, copy current size to previous size, compute `progress = (duration - remaining) / duration`, linearly interpolate current size, mark the border dirty, and replace the moving extent with a stationary extent once remaining ticks are non-positive. Containment uses current geometry; render-partial queries interpolate previous/current size. Each player tick applies warning and outside damage through independent calculations. Anchors: `net.minecraft.world.level.border.WorldBorder#tick()` and `net.minecraft.world.level.border.WorldBorder$MovingBorderExtent#update()`.
**Branches and aborts:** Stationary/lerping; inside/outside; within safe zone; warning threshold from distance or projected shrink; absolute coordinate clamp. Other entities collide/check only where their mechanic calls border APIs.
**Constants and randomness:** Border geometry and the interpolation fraction use doubles; duration and remaining progress are signed 64-bit tick counts. One border tick consumes exactly one unit regardless of wall-clock delay. Damage amount and warning fields are configured values. No RNG. Exact edge inclusivity/floating-point is `EXP-WGEN-004`.
**Side effects:** Player damage, movement/placement/teleport rejection where checked, warning overlay, border update synchronization and command feedback. The border does not automatically delete every outside entity/block.
**Gates:** Mechanic-specific border check, execution of `WorldBorder#tick()`, safe zone/damage rate, player state/damage immunity and command permission for mutation.
**Boundary cases and quirks:** Server overload stretches a configured duration in wall-clock time because progress advances by executed border ticks. If the owning level does not execute the border tick while frozen, queried current size remains frozen. Different shapes use min/max edge inclusivity.
**Evidence:** `Confirmed`; `OFF-SERVER-001`; bytecode for the two anchors and `lerpProgress`; client partial-tick geometry remains separately observable; `EXP-WGEN-004`.
**Test vectors:** Configure a 20-tick shrink and assert sizes at initial, 1st, 19th, and 20th border updates; insert wall-clock delay without a tick and assert no progress; exact min/max edges; outside-safe-zone damage boundaries; teleport/placement/entity behavior outside; reconnect during lerp.
