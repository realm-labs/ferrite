# 04 — Redstone, Pistons, and Explosions

The default baseline does not enable the optional bundled
`data/minecraft/datapacks/redstone_experiments`. This page specifies default `26.2` gameplay;
experimental-pack behavior requires separate rules with explicit enabling conditions.

## `RED-001` Redstone signals are directional 0–15 levels with direct/ordinary semantics

- **FidelityClass:** `ExactObservableBehavior`
- **Evidence status:** `Confirmed`

### Primary evidence

`OFF-SERVER-001`;
`net.minecraft.world.level.SignalGetter#getDirectSignal(net.minecraft.core.BlockPos,net.minecraft.core.Direction)`;
`net.minecraft.world.level.SignalGetter#getSignal(net.minecraft.core.BlockPos,net.minecraft.core.Direction)`;
`net.minecraft.world.level.SignalGetter#getBestNeighborSignal(net.minecraft.core.BlockPos)`;
`net.minecraft.world.level.block.state.BlockBehaviour$BlockStateBase#getSignal(net.minecraft.world.level.BlockGetter,net.minecraft.core.BlockPos,net.minecraft.core.Direction)`;
`net.minecraft.world.level.block.state.BlockBehaviour$BlockStateBase#getDirectSignal(net.minecraft.world.level.BlockGetter,net.minecraft.core.BlockPos,net.minecraft.core.Direction)`;
`COM-WIKI-RED-001`

### Applies when

A block queries an input on one face or exposes output to an adjacent position.

### Behavior and timing

Signal strength is clamped to `0..15`. Queries carry a direction and distinguish ordinary signal
from direct signal; world queries combine adjacent output with conductor propagation.
`getBestNeighborSignal` returns the maximum of six neighbor candidates and may stop at 15.

### Boundaries and quirks

“Powered block,” “strongly powered,” and “wire visually connected” are not one Boolean. Comparator
analog output is another distinct interface.

### Verification owners

`RED-UPDATE-001` and `EXP-RED-*` retain the generic source/conductor/face matrix.
`RED-DAYLIGHT-DETECTOR-001`/`EXP-RED-005` owns the daylight detector source transaction;
`BLK-COMMAND-001`/`EXP-BLK-017` owns command-block power edges, conditional predecessor reads and
success-count analog output;
`BLK-BELL-001`/`EXP-BLK-009` owns the bell's captured neighbor-signal edge;
`BLK-LECTERN-001`/`EXP-BLK-011` owns lectern weak/direct output; `BLK-SHELF-001`/`EXP-BLK-013` owns
shelf power edges and maximum-three side-chain reconstruction. `BLK-SCULK-SENSOR-001`/
`EXP-BLK-020` owns distance strength, calibrated face suppression, upward direct output and
frequency-filter input. `BLK-TEST-BLOCK-001`/`EXP-BLK-022` owns the start block's direction-neutral
15-level ordinary-only output, non-start rising/falling edge latch and explicit neighbor updates on
start trigger/reset. `BLK-STRUCTURE-001`/`EXP-BLK-027` owns its non-directional neighbor-signal
rising/falling latch and mode-selected memory save, immediate load, cache removal or no-op action.
`BLK-REDSTONE-BLOCK-001`/`EXP-BLK-051` owns state 11311's direction-neutral ordinary/own signal
`15`, inherited direct signal `0`, explicit nonconductor status and exact control-input shortcut.
`BLK-COPPER-FULL-001`/`EXP-BLK-073` owns the opposite ordinary-solid boundary for all 24 copper
states: each is a full redstone conductor but emits no weak or direct signal and has no comparator
output. Weather age, wax and collection add no signal branch; the four age-selected trumpet
instruments belong to note-block/client projection rather than redstone strength.
`BLK-SAPLING-001`/`EXP-BLK-074` fixes the no-collision opposite: all sixteen states are
nonconductors, emit no weak/direct signal and have no comparator output. Their self loot remains a
single matching item behind `survives_explosion`; stage adds no explosion-table branch.
`BLK-BAMBOO-001`/`EXP-BLK-075` fixes the same zero-signal/comparator boundary for sapling and
stalk states. Each self table emits one bamboo item behind `survives_explosion`; age, leaves and
stage add no loot branch.
`BLK-ANCIENT-DEBRIS-001`/`EXP-BLK-076` fixes the ordinary full-solid conductor with zero
weak/direct signal and no comparator output. Correct-tool self loot remains behind
`survives_explosion`; registered resistance 1200.0 feeds generic explosion math rather than hard
immunity.
`BLK-STEM-CROP-001`/`EXP-BLK-077` fixes all four forms as nonconductors with zero weak/direct
signal and no comparator output. Each stem table applies explosion decay after its age-selected
binomial seed count; each attached table applies the fixed age-seven binomial, with no tool,
Silk Touch or Fortune branch.
`BLK-OVERWORLD-CROP-001`/`EXP-BLK-078` fixes all 28 states as nonconductors with zero weak/direct
signal and no comparator output. Each crop table applies explosion decay after its age-selected
produce/seed pools; tool, Silk Touch and Fortune add no family-specific branch.
`BLK-TORCHFLOWER-CROP-001`/`EXP-BLK-079` fixes both crop ages and the mature flower as
nonconductors with zero weak/direct signal and no comparator output. The crop table always emits
one seed before explosion decay; the mature flower emits itself only when `survives_explosion`.
`BLK-PITCHER-CROP-001`/`EXP-BLK-080` fixes all twelve pitcher states as nonconductors with zero
weak/direct signal and no comparator output. Crop lower ages zero through three select a pod,
lower age four and mature lower select a plant, and every upper state selects nothing before the
table-level explosion-decay function.
`BLK-SWEET-BERRY-BUSH-001`/`EXP-BLK-081` fixes all four bush states as nonconductors with zero
weak/direct signal and no comparator output. Ages zero/one have empty break loot; ages two/three
emit uniform 1..2/2..3 berries plus a `0..fortuneLevel` bonus before table-level explosion decay.
`BLK-CAVE-VINES-001`/`EXP-BLK-082` fixes every head/body state as nonconducting with zero
weak/direct signal and no comparator output. Either lit identity emits exactly one glow berry with
no tool, Fortune, Silk Touch or explosion-decay gate; unlit states emit nothing.
`BLK-CHORUS-001`/`EXP-BLK-083` fixes every plant/flower state as nonconducting with zero weak/direct
signal and no comparator output. Plant loot is uniform zero or one chorus fruit before explosion
decay. Flower loot is one flower behind both `survives_explosion` and a present `this` entity,
without age, tool, Fortune or Silk Touch branches; entity-less support destruction therefore drops
none. An admitted tagged impact projectile destroys a flower with drops under the projectile break
game rule and the owner/mob-griefing interaction gate.

## `RED-002` Dust recomputation immediately creates an ordered neighbor-update chain

- **FidelityClass:** `ExactObservableBehavior`
- **Evidence status:** `Cross-checked`

### Primary evidence

`OFF-SERVER-001`;
`net.minecraft.world.level.block.RedStoneWireBlock#updatePowerStrength(net.minecraft.world.level.Level,net.minecraft.core.BlockPos,net.minecraft.world.level.block.state.BlockState,net.minecraft.world.level.redstone.Orientation,boolean)`;
`net.minecraft.world.level.block.RedStoneWireBlock#neighborChanged(net.minecraft.world.level.block.state.BlockState,net.minecraft.world.level.Level,net.minecraft.core.BlockPos,net.minecraft.world.level.block.Block,net.minecraft.world.level.redstone.Orientation,boolean)`;
`net.minecraft.world.level.block.RedStoneWireBlock#updateNeighborsOfNeighboringWires(net.minecraft.world.level.Level,net.minecraft.core.BlockPos)`;
`net.minecraft.world.level.redstone.CollectingNeighborUpdater#runUpdates()`; `COM-WIKI-RED-001`

### Applies when

Dust or a neighbor it can read changes state.

### Behavior and timing

Dust recomputes strength and connection shape from surrounding input. A changed write immediately
adds more neighbor/wire work. Updates use an `Orientation`-aware neighbor system, so direction and
nested enqueue order can affect short pulses, multiple stable solutions, and piston timing.

### Boundaries and quirks

Do not solve the whole redstone graph once per tick as an unordered steady state; that erases
player-observable update order. The default evaluator must not be mixed with the optional redstone
experiments.

### Verification

**Owners:** `RED-UPDATE-001`, `BLK-REDSTONE-BLOCK-001`; `EXP-RED-*`, `EXP-BLK-051`

GameTests must lock direction sequence, decay chains, dot/cross toggles, and simultaneous source
removal under the default pack. This remains `Cross-checked`.
The redstone-block leaf supplies the constant source/nonconductor identity; this rule retains the
placement/removal notification chain and every receiver's nested recomputation.

## `RED-003` Repeaters, comparators, and observers express delay with scheduled ticks

- **FidelityClass:** `ExactObservableBehavior`
- **Evidence status:** `Confirmed`

### Primary evidence

`OFF-SERVER-001`;
`net.minecraft.world.level.block.DiodeBlock#checkTickOnNeighbor(net.minecraft.world.level.Level,net.minecraft.core.BlockPos,net.minecraft.world.level.block.state.BlockState)`;
`net.minecraft.world.level.block.DiodeBlock#tick(net.minecraft.world.level.block.state.BlockState,net.minecraft.server.level.ServerLevel,net.minecraft.core.BlockPos,net.minecraft.util.RandomSource)`;
`net.minecraft.world.level.block.RepeaterBlock#getDelay(net.minecraft.world.level.block.state.BlockState)`;
`net.minecraft.world.level.block.RepeaterBlock#isLocked(net.minecraft.world.level.LevelReader,net.minecraft.core.BlockPos,net.minecraft.world.level.block.state.BlockState)`;
`net.minecraft.world.level.block.ComparatorBlock#calculateOutputSignal(net.minecraft.world.level.Level,net.minecraft.core.BlockPos,net.minecraft.world.level.block.state.BlockState)`;
`net.minecraft.world.level.block.ComparatorBlock#refreshOutputState(net.minecraft.world.level.Level,net.minecraft.core.BlockPos,net.minecraft.world.level.block.state.BlockState)`;
`net.minecraft.world.level.block.ObserverBlock#startSignal(net.minecraft.world.level.LevelReader,net.minecraft.world.level.ScheduledTickAccess,net.minecraft.core.BlockPos)`;
`net.minecraft.world.level.block.ObserverBlock#tick(net.minecraft.world.level.block.state.BlockState,net.minecraft.server.level.ServerLevel,net.minecraft.core.BlockPos,net.minecraft.util.RandomSource)`

### Applies when

An input or observed state change requires a delayed output transition.

### Behavior and timing

The diode base schedules a tick from neighbor checks and switches powered state only when due. A
repeater converts its `DELAY` property to property value × `2` game ticks and can be side-locked. A
comparator recomputes main input, side input, and container analog output, then applies
compare/subtract mode. An observer detects a change, schedules a `2`-tick edge, and uses follow-up
scheduled work to end the pulse.

### Boundaries and quirks

Tick priority, pulses shorter than the delay, a pre-existing schedule while locked, and comparator
block-entity caching can change the result.

### Verification owners

`RED-COMPARATOR-001`, `ITM-BARREL-001`, `ITM-BOOKSHELF-001`, `ITM-JUKEBOX-001`,
`BLK-COPPER-GOLEM-STATUE-001`, `BLK-LECTERN-001`, `BLK-SHELF-001`,
`BLK-DECORATED-POT-001`, `BLK-LAVA-CAULDRON-001`, `EXP-RED-006`, `EXP-ITM-009`,
`EXP-ITM-010`, `EXP-ITM-011`, `EXP-BLK-008`, `EXP-BLK-011`, `EXP-BLK-013`, `EXP-BLK-014`
and `EXP-BLK-039` source-specify the
comparator transaction and concrete projections, including statue poses, jukebox playback, lectern
page/content divergence, shelf's back-face occupancy, pot one-stack fullness and lava cauldron's
constant analog value `3`. `RED-DELAY-001` and
`EXP-RED-002` retain repeater, observer and torch waveform work. `BLK-COMMAND-001`/`EXP-BLK-017`
owns the separate one-tick command-block schedule and repeating resubmission transaction.
`BLK-SCULK-SENSOR-001`/`EXP-BLK-020` owns active-only frequency analog output, including persisted
frequency with missing, wrong or nonactive live state.
`BLK-REDSTONE-BLOCK-001`/`EXP-BLK-051` owns the exact-identity side-control result `15` when the
diode-only flag is false and `0` for the same non-diode identity when it is true; the comparator
owner retains side choice, compare/subtract arithmetic, scheduling and output publication.
`BLK-AMETHYST-BLOCK-001`/`EXP-BLK-052` owns state 23402's sole
`vibration_resonators` membership. On sensor activation, `BLK-SCULK-SENSOR-001` retains the
six-direction loop, frequency-to-resonance event, note-derived pitch table, scheduled phase and
sound publication; zero through six adjacent amethyst blocks independently pass its tag gate.

## `RED-004` A piston queues a block event, then executes an ordered movement transaction

- **FidelityClass:** `ExactObservableBehavior`
- **Evidence status:** `Confirmed`

### Primary evidence

`OFF-SERVER-001`;
`net.minecraft.world.level.block.piston.PistonBaseBlock#checkIfExtend(net.minecraft.world.level.Level,net.minecraft.core.BlockPos,net.minecraft.world.level.block.state.BlockState)`;
`net.minecraft.world.level.block.piston.PistonBaseBlock#triggerEvent(net.minecraft.world.level.block.state.BlockState,net.minecraft.world.level.Level,net.minecraft.core.BlockPos,int,int)`;
`net.minecraft.world.level.block.piston.PistonBaseBlock#moveBlocks(net.minecraft.world.level.Level,net.minecraft.core.BlockPos,net.minecraft.core.Direction,boolean)`;
`net.minecraft.world.level.block.piston.PistonStructureResolver#resolve()`;
`net.minecraft.world.level.block.piston.PistonStructureResolver#getToPush()`;
`net.minecraft.world.level.block.piston.PistonStructureResolver#getToDestroy()`; `COM-WIKI-RED-001`

### Applies when

Piston input changes and extension/retraction may change.

### Behavior and timing

A neighbor check only decides whether to queue a piston block event. In the event phase conditions
are checked again, then the resolver builds `toPush`/`toDestroy` using adhesion, direction, push
reaction, world bounds, and the push limit. Execution moves/destroys in overwrite-safe order,
creates moving-piston states and block entities, and sends follow-up updates.

### Boundaries and quirks

Input may reverse between event enqueue and execution; resolver failure must leave the structure
unmoved. Block-entity mobility and concrete `PushReaction` values are content exceptions.

### Verification

**Owners:** `RED-PISTON-001`, `BLK-STRUCTURE-VOID-001`, `BLK-BEDROCK-001`,
`BLK-REINFORCED-DEEPSLATE-001`, `BLK-GLAZED-TERRACOTTA-001`, `BLK-SLIME-001`,
`BLK-HONEY-001`; `EXP-RED-003`, `EXP-BLK-029`,
`EXP-BLK-031`, `EXP-BLK-032`, `EXP-BLK-035`, `EXP-BLK-036`, `EXP-BLK-043`

Lock exact update order of movement/destruction lists, entity movement, slime/honey branches, and
same-tick opposing pistons.
The structure-void leaf fixes its explicit DESTROY reaction and no-loot result; the piston owner
retains resolver admission and destruction-list order.
The bedrock leaf fixes destroy speed -1 and rejection before its inherited NORMAL reaction; it is
therefore absent from both movement and destruction lists.
The reinforced-deepslate leaf fixes positive hardness 55 and the same earlier exact-identity
rejection despite inherited NORMAL reaction.
The glazed-terracotta leaf fixes `PUSH_ONLY`: forward extension admits movement, while sticky
retraction and slime/honey backward or perpendicular resolver edges reject it. The piston owner
retains traversal, cap, moving-state, neighbor-update and correction order.
The slime leaf fixes exact slime/honey adhesion, perpendicular branching and the moving-slime
nonplayer axis-velocity assignment; the piston owner retains traversal, cap, list and move order.
The honey leaf fixes the reciprocal adhesion boundary and the separate horizontal top-surface
carry after ordinary collided-entity displacement; the piston owner retains progress and movement.

## `RED-005` Pistons have above-adjacent quasi-connectivity behavior

- **FidelityClass:** `ExactObservableBehavior`
- **Evidence status:** `Confirmed`

### Primary evidence

`OFF-SERVER-001`; `OFF-BUG-001`;
`net.minecraft.world.level.block.piston.PistonBaseBlock#getNeighborSignal(net.minecraft.world.level.SignalGetter,net.minecraft.core.BlockPos,net.minecraft.core.Direction)`;
`net.minecraft.world.level.block.piston.PistonBaseBlock#neighborChanged(net.minecraft.world.level.block.state.BlockState,net.minecraft.world.level.Level,net.minecraft.core.BlockPos,net.minecraft.world.level.block.Block,net.minecraft.world.level.redstone.Orientation,boolean)`

### Applies when

A piston tests for power, especially power around the position above it.

### Behavior and timing

In addition to ordinary adjacent signals, `getNeighborSignal` checks inputs around the block above
the piston. A signal not directly connected to a piston face can therefore satisfy the power
condition. Immediate action still depends on the piston receiving an update that invokes
`checkIfExtend`.

### Boundaries and quirks

The phenomenon is commonly tracked as [MC-108](https://bugs.mojang.com/browse/MC-108). This page
uses that number only to identify the quirk; source establishes current `26.2` behavior without
inferring the ticket's current disposition. **Replication decision: Undecided.**

### Verification

**Owners:** `RED-PISTON-001`; `EXP-RED-003`

Build a source-derived matrix for “powered without update,” an above-neighbor update, and each
signal direction. A later architecture decision must choose whether exact quirk compatibility
remains required.

## `RED-006` Explosions separate sampling, entity effects, block effects, and optional fire

- **FidelityClass:** `ExactObservableBehavior`
- **Evidence status:** `Confirmed`

### Primary evidence

`OFF-SERVER-001`; `net.minecraft.world.level.ServerExplosion#calculateExplodedPositions()`;
`net.minecraft.world.level.ServerExplosion#hurtEntities()`;
`net.minecraft.world.level.ServerExplosion#interactWithBlocks(java.util.List)`;
`net.minecraft.world.level.ServerExplosion#createFire(java.util.List)`;
`net.minecraft.world.level.ServerExplosion#explode()`; `COM-WIKI-RED-001`

### Applies when

The server executes an explosion with a center, radius, damage source, block-interaction mode, and
fire flag.

### Behavior and timing

It ray-samples an affected-block set, computes exposure, damage, and knockback for entities in
range, processes block callbacks/destruction/drops according to interaction mode, then optionally
attempts fire, and sends observable results to clients.

### Boundaries and quirks

Block resistance, fluids, occlusion, damage immunity, game rules, drop merging, and TNT chains alter
results. Explosions created during an explosion must not collapse into one unordered set.

### Verification

**Owners:** `RED-EXPLOSION-001`, `BLK-BEDROCK-001`, `BLK-REINFORCED-DEEPSLATE-001`,
`BLK-COPPER-FULL-001`, `BLK-SAPLING-001`, `BLK-BAMBOO-001`, `BLK-ANCIENT-DEBRIS-001`,
`BLK-STEM-CROP-001`, `BLK-OVERWORLD-CROP-001`, `BLK-TORCHFLOWER-CROP-001`,
`BLK-PITCHER-CROP-001`, `BLK-SWEET-BERRY-BUSH-001`, `BLK-CAVE-VINES-001`;
`BLK-CHORUS-001`;
`EXP-RED-004`, `EXP-BLK-031`, `EXP-BLK-032`, `EXP-BLK-073`, `EXP-BLK-074`, `EXP-BLK-075`,
`EXP-BLK-076`, `EXP-BLK-077`, `EXP-BLK-078`, `EXP-BLK-079`, `EXP-BLK-080`, `EXP-BLK-081`,
`EXP-BLK-082`, `EXP-BLK-083`

Exact ray sampling, block traversal order, drop-merge thresholds, and entity-exposure samples need
deterministic source vectors or black-box fixtures.
The bedrock leaf fixes registered resistance `3,600,000` plus the wind-charge holder-set lookup;
ordinary explosion traversal and the distinction between finite resistance and hard tag immunity
remain here.
The reinforced-deepslate leaf fixes ordinary resistance `1200` and the opposite wind-charge branch:
nonmembership returns no block resistance while block explosion remains enabled.
The full-copper leaf fixes resistance `6` for all 24 states and one self-item
`survives_explosion` loot pool per identity. Sampling, destruction admission, survival probability,
drop merging and client explosion publication remain with the explosion and loot owners.
The torchflower leaf fixes the crop's unconditional seed pool followed by explosion decay and the
mature flower's single self entry behind explosion survival; generic ray traversal and drop
publication remain with the explosion and loot owners.
