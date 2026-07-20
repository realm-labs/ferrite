# 04 — Redstone, Pistons, and Explosions

The default baseline does not enable the optional bundled `data/minecraft/datapacks/redstone_experiments`. This page specifies default `26.2` gameplay; experimental-pack behavior requires separate rules with explicit enabling conditions.

## `RED-001` Redstone signals are directional 0–15 levels with direct/ordinary semantics

- **FidelityClass:** `ExactObservableBehavior`
- **Evidence status:** `Confirmed`
- **Primary evidence:** `OFF-SERVER-001`; `net.minecraft.world.level.SignalGetter#getDirectSignal(net.minecraft.core.BlockPos,net.minecraft.core.Direction)`; `net.minecraft.world.level.SignalGetter#getSignal(net.minecraft.core.BlockPos,net.minecraft.core.Direction)`; `net.minecraft.world.level.SignalGetter#getBestNeighborSignal(net.minecraft.core.BlockPos)`; `net.minecraft.world.level.block.state.BlockBehaviour$BlockStateBase#getSignal(net.minecraft.world.level.BlockGetter,net.minecraft.core.BlockPos,net.minecraft.core.Direction)`; `net.minecraft.world.level.block.state.BlockBehaviour$BlockStateBase#getDirectSignal(net.minecraft.world.level.BlockGetter,net.minecraft.core.BlockPos,net.minecraft.core.Direction)`; `COM-WIKI-RED-001`
- **Applies when:** A block queries an input on one face or exposes output to an adjacent position.
- **Behavior and timing:** Signal strength is clamped to `0..15`. Queries carry a direction and distinguish ordinary signal from direct signal; world queries combine adjacent output with conductor propagation. `getBestNeighborSignal` returns the maximum of six neighbor candidates and may stop at 15.
- **Boundaries and quirks:** “Powered block,” “strongly powered,” and “wire visually connected” are not one Boolean. Comparator analog output is another distinct interface.
- **Verification owners:** `RED-UPDATE-001` and `EXP-RED-*` retain the generic source/conductor/face matrix. `RED-DAYLIGHT-DETECTOR-001`/`EXP-RED-005` owns the daylight detector source transaction; `BLK-BELL-001`/`EXP-BLK-009` owns the bell's captured neighbor-signal rising/falling edge and nontransactional ring-before-POWERED write.

## `RED-002` Dust recomputation immediately creates an ordered neighbor-update chain

- **FidelityClass:** `ExactObservableBehavior`
- **Evidence status:** `Cross-checked`
- **Primary evidence:** `OFF-SERVER-001`; `net.minecraft.world.level.block.RedStoneWireBlock#updatePowerStrength(net.minecraft.world.level.Level,net.minecraft.core.BlockPos,net.minecraft.world.level.block.state.BlockState,net.minecraft.world.level.redstone.Orientation,boolean)`; `net.minecraft.world.level.block.RedStoneWireBlock#neighborChanged(net.minecraft.world.level.block.state.BlockState,net.minecraft.world.level.Level,net.minecraft.core.BlockPos,net.minecraft.world.level.block.Block,net.minecraft.world.level.redstone.Orientation,boolean)`; `net.minecraft.world.level.block.RedStoneWireBlock#updateNeighborsOfNeighboringWires(net.minecraft.world.level.Level,net.minecraft.core.BlockPos)`; `net.minecraft.world.level.redstone.CollectingNeighborUpdater#runUpdates()`; `COM-WIKI-RED-001`
- **Applies when:** Dust or a neighbor it can read changes state.
- **Behavior and timing:** Dust recomputes strength and connection shape from surrounding input. A changed write immediately adds more neighbor/wire work. Updates use an `Orientation`-aware neighbor system, so direction and nested enqueue order can affect short pulses, multiple stable solutions, and piston timing.
- **Boundaries and quirks:** Do not solve the whole redstone graph once per tick as an unordered steady state; that erases player-observable update order. The default evaluator must not be mixed with the optional redstone experiments.
- **Verification owner (`RED-UPDATE-001`; `EXP-RED-*`):** GameTests must lock direction sequence, decay chains, dot/cross toggles, and simultaneous source removal under the default pack. This remains `Cross-checked`.

## `RED-003` Repeaters, comparators, and observers express delay with scheduled ticks

- **FidelityClass:** `ExactObservableBehavior`
- **Evidence status:** `Confirmed`
- **Primary evidence:** `OFF-SERVER-001`; `net.minecraft.world.level.block.DiodeBlock#checkTickOnNeighbor(net.minecraft.world.level.Level,net.minecraft.core.BlockPos,net.minecraft.world.level.block.state.BlockState)`; `net.minecraft.world.level.block.DiodeBlock#tick(net.minecraft.world.level.block.state.BlockState,net.minecraft.server.level.ServerLevel,net.minecraft.core.BlockPos,net.minecraft.util.RandomSource)`; `net.minecraft.world.level.block.RepeaterBlock#getDelay(net.minecraft.world.level.block.state.BlockState)`; `net.minecraft.world.level.block.RepeaterBlock#isLocked(net.minecraft.world.level.LevelReader,net.minecraft.core.BlockPos,net.minecraft.world.level.block.state.BlockState)`; `net.minecraft.world.level.block.ComparatorBlock#calculateOutputSignal(net.minecraft.world.level.Level,net.minecraft.core.BlockPos,net.minecraft.world.level.block.state.BlockState)`; `net.minecraft.world.level.block.ComparatorBlock#refreshOutputState(net.minecraft.world.level.Level,net.minecraft.core.BlockPos,net.minecraft.world.level.block.state.BlockState)`; `net.minecraft.world.level.block.ObserverBlock#startSignal(net.minecraft.world.level.LevelReader,net.minecraft.world.level.ScheduledTickAccess,net.minecraft.core.BlockPos)`; `net.minecraft.world.level.block.ObserverBlock#tick(net.minecraft.world.level.block.state.BlockState,net.minecraft.server.level.ServerLevel,net.minecraft.core.BlockPos,net.minecraft.util.RandomSource)`
- **Applies when:** An input or observed state change requires a delayed output transition.
- **Behavior and timing:** The diode base schedules a tick from neighbor checks and switches powered state only when due. A repeater converts its `DELAY` property to property value × `2` game ticks and can be side-locked. A comparator recomputes main input, side input, and container analog output, then applies compare/subtract mode. An observer detects a change, schedules a `2`-tick edge, and uses follow-up scheduled work to end the pulse.
- **Boundaries and quirks:** Tick priority, pulses shorter than the delay, a pre-existing schedule while locked, and comparator block-entity caching can change the result.
- **Verification owners:** `RED-COMPARATOR-001`, `ITM-BARREL-001`, `ITM-BOOKSHELF-001`, `ITM-JUKEBOX-001`, `BLK-COPPER-GOLEM-STATUE-001`, `EXP-RED-006`, `EXP-ITM-009`, `EXP-ITM-010`, `EXP-ITM-011` and `EXP-BLK-008` source-specify the comparator transaction and four concrete projections, including statue pose outputs 1..4 and the jukebox's item-defined output versus playing-defined source signal. `RED-DELAY-001` and `EXP-RED-002` retain repeater, observer and torch waveform work.

## `RED-004` A piston queues a block event, then executes an ordered movement transaction

- **FidelityClass:** `ExactObservableBehavior`
- **Evidence status:** `Confirmed`
- **Primary evidence:** `OFF-SERVER-001`; `net.minecraft.world.level.block.piston.PistonBaseBlock#checkIfExtend(net.minecraft.world.level.Level,net.minecraft.core.BlockPos,net.minecraft.world.level.block.state.BlockState)`; `net.minecraft.world.level.block.piston.PistonBaseBlock#triggerEvent(net.minecraft.world.level.block.state.BlockState,net.minecraft.world.level.Level,net.minecraft.core.BlockPos,int,int)`; `net.minecraft.world.level.block.piston.PistonBaseBlock#moveBlocks(net.minecraft.world.level.Level,net.minecraft.core.BlockPos,net.minecraft.core.Direction,boolean)`; `net.minecraft.world.level.block.piston.PistonStructureResolver#resolve()`; `net.minecraft.world.level.block.piston.PistonStructureResolver#getToPush()`; `net.minecraft.world.level.block.piston.PistonStructureResolver#getToDestroy()`; `COM-WIKI-RED-001`
- **Applies when:** Piston input changes and extension/retraction may change.
- **Behavior and timing:** A neighbor check only decides whether to queue a piston block event. In the event phase conditions are checked again, then the resolver builds `toPush`/`toDestroy` using adhesion, direction, push reaction, world bounds, and the push limit. Execution moves/destroys in overwrite-safe order, creates moving-piston states and block entities, and sends follow-up updates.
- **Boundaries and quirks:** Input may reverse between event enqueue and execution; resolver failure must leave the structure unmoved. Block-entity mobility and concrete `PushReaction` values are content exceptions.
- **Verification owner (`RED-PISTON-001`; `EXP-RED-003`):** Lock exact update order of movement/destruction lists, entity movement, slime/honey branches, and same-tick opposing pistons.

## `RED-005` Pistons have above-adjacent quasi-connectivity behavior

- **FidelityClass:** `ExactObservableBehavior`
- **Evidence status:** `Confirmed`
- **Primary evidence:** `OFF-SERVER-001`; `OFF-BUG-001`; `net.minecraft.world.level.block.piston.PistonBaseBlock#getNeighborSignal(net.minecraft.world.level.SignalGetter,net.minecraft.core.BlockPos,net.minecraft.core.Direction)`; `net.minecraft.world.level.block.piston.PistonBaseBlock#neighborChanged(net.minecraft.world.level.block.state.BlockState,net.minecraft.world.level.Level,net.minecraft.core.BlockPos,net.minecraft.world.level.block.Block,net.minecraft.world.level.redstone.Orientation,boolean)`
- **Applies when:** A piston tests for power, especially power around the position above it.
- **Behavior and timing:** In addition to ordinary adjacent signals, `getNeighborSignal` checks inputs around the block above the piston. A signal not directly connected to a piston face can therefore satisfy the power condition. Immediate action still depends on the piston receiving an update that invokes `checkIfExtend`.
- **Boundaries and quirks:** The phenomenon is commonly tracked as [MC-108](https://bugs.mojang.com/browse/MC-108). This page uses that number only to identify the quirk; source establishes current `26.2` behavior without inferring the ticket's current disposition. **Replication decision: Undecided.**
- **Verification owner (`RED-PISTON-001`; `EXP-RED-003`):** Build a source-derived matrix for “powered without update,” an above-neighbor update, and each signal direction. A later architecture decision must choose whether exact quirk compatibility remains required.

## `RED-006` Explosions separate sampling, entity effects, block effects, and optional fire

- **FidelityClass:** `ExactObservableBehavior`
- **Evidence status:** `Confirmed`
- **Primary evidence:** `OFF-SERVER-001`; `net.minecraft.world.level.ServerExplosion#calculateExplodedPositions()`; `net.minecraft.world.level.ServerExplosion#hurtEntities()`; `net.minecraft.world.level.ServerExplosion#interactWithBlocks(java.util.List)`; `net.minecraft.world.level.ServerExplosion#createFire(java.util.List)`; `net.minecraft.world.level.ServerExplosion#explode()`; `COM-WIKI-RED-001`
- **Applies when:** The server executes an explosion with a center, radius, damage source, block-interaction mode, and fire flag.
- **Behavior and timing:** It ray-samples an affected-block set, computes exposure, damage, and knockback for entities in range, processes block callbacks/destruction/drops according to interaction mode, then optionally attempts fire, and sends observable results to clients.
- **Boundaries and quirks:** Block resistance, fluids, occlusion, damage immunity, game rules, drop merging, and TNT chains alter results. Explosions created during an explosion must not collapse into one unordered set.
- **Verification owner (`RED-EXPLOSION-001`; `EXP-RED-004`):** Exact ray sampling, block traversal order, drop-merge thresholds, and entity-exposure samples need deterministic source vectors or black-box fixtures.
