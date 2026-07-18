# Rust Minecraft-Style Behavioral Reimplementation
## Architecture and Implementation Reference

**Status:** Initial architecture baseline<br>
**Language:** Rust<br>
**Simulation ECS:** `bevy_ecs`<br>
**Reference engine version:** Bevy / `bevy_ecs` 0.19<br>
**Reference gameplay version:** Minecraft Java Edition 26.2<br>
**Primary objective:** Reproduce Minecraft-style gameplay behavior while designing an independent, high-performance architecture<br>
**Compatibility objective:** None. The project does not need to load original saves, speak the original protocol, or preserve original implementation details.

Normative behavior entry: [Minecraft Java 26.2 behavior manual](reference/minecraft-java-26.2/README.md). Implementations and compatibility tests must resolve the relevant leaf rule and content-family query before choosing behavior; unresolved branches remain experiment-owned rather than implementation-defined.

---

## Table of Contents

1. [Executive Summary](#1-executive-summary)
2. [Scope](#2-scope)
3. [Architectural Principles](#3-architectural-principles)
4. [System Overview](#4-system-overview)
5. [Workspace Layout](#5-workspace-layout)
6. [Runtime Topology](#6-runtime-topology)
7. [Simulation Core](#7-simulation-core)
8. [Tick Pipeline and Determinism](#8-tick-pipeline-and-determinism)
9. [World Data Model](#9-world-data-model)
10. [Registries and Game Data](#10-registries-and-game-data)
11. [Entity Model with Bevy ECS](#11-entity-model-with-bevy-ecs)
12. [World Mutation Model](#12-world-mutation-model)
13. [Block Behavior Model](#13-block-behavior-model)
14. [Tick Scheduling and Neighbor Updates](#14-tick-scheduling-and-neighbor-updates)
15. [Physics and Collision](#15-physics-and-collision)
16. [Redstone-Like Logic](#16-redstone-like-logic)
17. [Fluid Simulation](#17-fluid-simulation)
18. [Lighting](#18-lighting)
19. [World Generation](#19-world-generation)
20. [Server Runtime](#20-server-runtime)
21. [Protocol and Transport](#21-protocol-and-transport)
22. [Engine-Independent Client Runtime](#22-engine-independent-client-runtime)
23. [Bevy Frontend](#23-bevy-frontend)
24. [Chunk Meshing and Rendering](#24-chunk-meshing-and-rendering)
25. [Persistence](#25-persistence)
26. [Concurrency and Job Scheduling](#26-concurrency-and-job-scheduling)
27. [Replay, Testing, and Behavioral Specifications](#27-replay-testing-and-behavioral-specifications)
28. [Observability and Developer Tooling](#28-observability-and-developer-tooling)
29. [Error Handling and Recovery](#29-error-handling-and-recovery)
30. [Future Modding and Scripting](#30-future-modding-and-scripting)
31. [Security Model](#31-security-model)
32. [Performance Budgets](#32-performance-budgets)
33. [Implementation Roadmap](#33-implementation-roadmap)
34. [Architecture Decision Records](#34-architecture-decision-records)
35. [Known Risks](#35-known-risks)
36. [Initial Definition of Done](#36-initial-definition-of-done)
37. [Reference Dependencies](#37-reference-dependencies)
38. [References](#38-references)

---

# 1. Executive Summary

This project is an independent Rust implementation of a Minecraft-style voxel sandbox. Its goal is to reproduce gameplay behavior as closely as practical while replacing the original implementation architecture with a clean, explicit, testable, and high-performance design.

The project is not a client mod, protocol-compatible server, save-file converter, or source-level port. It is a behavioral reimplementation.

The architecture is server-authoritative from the beginning:

```text
Player Input
    ↓
Client Runtime
    ↓
Local or Network Transport
    ↓
Server Runtime
    ↓
Simulation Core
    ↓
World and Entity State
    ↓
Snapshots, Deltas, and Effects
    ↓
Client Runtime
    ↓
Bevy Frontend
```

The core simulation uses `bevy_ecs` as an independent ECS library. It does not depend on Bevy rendering, windows, input, audio, assets, scenes, or UI. The full Bevy engine is confined to the frontend adapter.

The simulation is driven by a fixed-rate tick loop. All gameplay semantics are expressed through explicit phases, stable identifiers, deterministic queues, and controlled mutation boundaries. Large immutable or sparse voxel data remains in specialized chunk storage instead of becoming ECS entities.

The essential separation is:

```text
Specialized voxel storage  → blocks, light, biomes, heightmaps
bevy_ecs                   → dynamic entities and simulation resources
Server runtime             → sessions, chunk interest, networking, persistence
Client runtime             → replicated state, interpolation, prediction
Bevy frontend              → rendering, input, audio, UI, resource presentation
```

---

# 2. Scope

## 2.1 Primary Goals

The project should eventually support:

- An effectively unbounded chunked voxel world.
- Block placement and destruction.
- Block states and block-specific behavior.
- Items, inventories, crafting, furnaces, containers, and equipment.
- Player movement and collision behavior similar to Minecraft.
- Dynamic entities such as mobs, projectiles, drops, boats, and minecarts.
- Scheduled block ticks and random block ticks.
- Neighbor updates and immediate block reactions.
- Redstone-like circuits with behavior matching the intended reference rules.
- Fluid propagation.
- Skylight and block light.
- Procedural terrain, biomes, caves, structures, and dimensions.
- Single-player through an embedded server.
- Dedicated multiplayer servers.
- Save games with crash recovery and format migration.
- A Bevy-based graphical frontend.
- A frontend-independent protocol and client state layer.
- Behavioral tests that describe and preserve gameplay semantics.

## 2.2 Explicit Non-Goals

The initial architecture does not require:

- Compatibility with original Minecraft clients.
- Compatibility with original Minecraft servers.
- Compatibility with original network packet formats.
- Compatibility with NBT, Anvil, Region, or original save formats.
- Identical world generation for the same seed.
- Identical internal class structure.
- Identical bugs unless they are considered important gameplay behavior.
- Support for several historical game versions.
- A stable public modding API in the first implementation phase.
- Distributed simulation across multiple server nodes.
- Perfect deterministic execution across different CPU architectures in the first release.

## 2.3 Behavioral Fidelity Policy

Behavioral fidelity should be classified instead of treated as a binary property.

```rust
pub enum FidelityClass {
    ExactObservableBehavior,
    EquivalentPlayerVisibleBehavior,
    IntentionallyImprovedBehavior,
    Unimplemented,
}
```

Every complex feature should document its target class. For example:

- Repeater delay: `ExactObservableBehavior`.
- Terrain noise implementation: `EquivalentPlayerVisibleBehavior`.
- Chunk I/O format: `IntentionallyImprovedBehavior`.
- A rare historical piston bug: a deliberate project decision.

This avoids silently mixing compatibility goals with architectural goals.

The version-locked [Minecraft Java Edition 26.2 behavioral reference](reference/minecraft-java-26.2/README.md) is the required entry point before implementing or testing observable gameplay. A rule marked `Provisional` or `Conflict` must gain stronger evidence before it is treated as exact behavior. Later Minecraft versions receive sibling reference directories; they do not silently rewrite the `26.2` baseline.

---

# 3. Architectural Principles

## 3.1 Core Logic Must Not Depend on a Rendering Engine

The simulation core must not reference:

- `bevy_render`
- `bevy_asset`
- `bevy_window`
- `bevy_input`
- `bevy_audio`
- meshes
- materials
- textures
- cameras
- frontend scene entities
- GPU resource handles

The only Bevy crate allowed in the simulation core is `bevy_ecs`, plus carefully selected standalone utility crates if justified.

## 3.2 Server Authority Is the Default

The server owns authoritative state:

- world blocks
- entity state
- inventories
- health
- damage
- item use
- recipes
- redstone results
- fluid results
- spawning
- time and weather
- permissions

The client may predict movement and presentation, but prediction never becomes authoritative state by itself.

## 3.3 Single-Player Is an Embedded Server

Single-player must run the same server and simulation logic as multiplayer.

```text
Single-player:
Bevy Frontend ↔ Client Runtime ↔ Local Transport ↔ Embedded Server

Multiplayer:
Bevy Frontend ↔ Client Runtime ↔ Network Transport ↔ Dedicated Server
```

The local transport may bypass serialization for performance, but it must preserve the same message semantics.

## 3.4 Blocks Are Not ECS Entities

A loaded world can contain hundreds of millions of addressable blocks. Normal blocks belong in dense or palette-compressed section storage.

ECS entities are reserved for objects with independent dynamic identity:

- players
- mobs
- projectiles
- dropped items
- vehicles
- experience orbs
- dynamic block entities when useful
- temporary gameplay objects

## 3.5 Bevy ECS Is a Storage and Scheduling Tool, Not the Gameplay Specification

Gameplay phase ordering must be explicit. The project must not allow accidental system ordering to define game rules.

Use Bevy ECS for:

- component storage
- queries
- resources
- system execution
- conflict-aware parallelism
- commands for ECS structural changes

Do not rely on an opaque automatically parallel schedule for order-sensitive mechanics.

## 3.6 Stable IDs Must Be Independent of Runtime Handles

Never persist or transmit `bevy_ecs::entity::Entity`.

Use separate identity types:

```rust
#[repr(transparent)]
pub struct PersistentEntityId(pub u128);

#[repr(transparent)]
pub struct NetworkEntityId(pub u64);

#[repr(transparent)]
pub struct RuntimeEntityKey(pub u64);
```

A runtime map connects stable IDs to Bevy `Entity` values.

## 3.7 Immediate and Deferred Mutations Must Be Explicit

Some behavior requires immediate visibility during the same tick. Other work can be batched.

Every mutation path must declare whether it is:

- immediate
- deferred until a phase barrier
- deferred until end of tick
- asynchronous and revision-checked

## 3.8 Performance Must Not Destroy Observable Semantics

Parallelize:

- chunk generation
- chunk meshing
- compression
- pathfinding
- read-only AI sensing
- region I/O
- independent dimensions
- immutable snapshot encoding

Be conservative around:

- neighbor updates
- redstone
- pistons
- fluids
- collision resolution
- scheduled ticks
- operations with observable order

A common pattern is:

```text
Parallel read/compute
    ↓
Deterministic merge
    ↓
Ordered commit
```

## 3.9 Every Large Subsystem Needs a Behavioral Specification

Implementation is not the specification. Tests and written rules must define:

- update order
- timing
- visibility
- conflict resolution
- recursion limits
- random number consumption
- persistence guarantees

---

# 4. System Overview

```text
┌─────────────────────────────────────────────────────────────┐
│                        Applications                         │
│                                                             │
│  client-bevy        dedicated-server        world-tools     │
└──────────────┬───────────────┬──────────────────┬───────────┘
               │               │                  │
┌──────────────▼───────────────▼──────────────────▼───────────┐
│                     Runtime Layer                           │
│                                                             │
│ client-runtime   server-runtime   replay-runtime   admin    │
└──────────────┬───────────────┬──────────────────┬───────────┘
               │               │                  │
┌──────────────▼───────────────▼──────────────────▼───────────┐
│                    Simulation Core                          │
│                                                             │
│ fixed tick | bevy_ecs | commands | events | game rules      │
└──────────────┬───────────────────────┬──────────────────────┘
               │                       │
┌──────────────▼──────────────┐ ┌──────▼──────────────────────┐
│         World Model         │ │       Gameplay Model        │
│ chunks, sections, light,    │ │ blocks, items, entities,    │
│ biomes, block entities      │ │ crafting, redstone, fluids  │
└──────────────┬──────────────┘ └──────┬──────────────────────┘
               │                       │
┌──────────────▼───────────────────────▼──────────────────────┐
│                 Foundation and Registries                   │
│ coordinates, IDs, math, schemas, resource identifiers       │
└──────────────┬───────────────────────┬──────────────────────┘
               │                       │
┌──────────────▼──────────────┐ ┌──────▼──────────────────────┐
│        Persistence          │ │          Protocol           │
│ region files, journal,      │ │ semantic messages, codecs,  │
│ migrations, snapshots       │ │ local/network transports    │
└─────────────────────────────┘ └─────────────────────────────┘
```

## 4.1 Dependency Direction

The dependency graph must remain acyclic.

```text
foundation
  ↑
registry
  ↑
world
  ↑
simulation ← gameplay
  ↑
server-runtime ← persistence / protocol
  ↑
client-runtime
  ↑
client-bevy
```

Some practical dependency edges can differ, but the core rule remains:

> Presentation and transport may depend on simulation concepts. Simulation must never depend on presentation or transport implementations.

---

# 5. Workspace Layout

Start with a moderate number of crates. Split further only when boundaries are proven.

```text
workspace/
├── Cargo.toml
├── crates/
│   ├── foundation/
│   ├── registry/
│   ├── world/
│   ├── simulation/
│   ├── gameplay/
│   ├── protocol/
│   ├── persistence/
│   ├── server-runtime/
│   ├── client-runtime/
│   ├── client-bevy/
│   ├── testkit/
│   └── tooling/
├── apps/
│   ├── client/
│   ├── server/
│   ├── world-inspector/
│   └── behavior-runner/
├── assets/
├── data/
│   ├── blocks/
│   ├── items/
│   ├── recipes/
│   ├── loot/
│   ├── biomes/
│   └── dimensions/
├── spec/
│   ├── blocks/
│   ├── redstone/
│   ├── fluids/
│   ├── physics/
│   ├── inventory/
│   └── entities/
└── docs/
    ├── architecture/
    ├── behavior/
    └── adr/
```

## 5.1 Crate Responsibilities

### `foundation`

Owns small, stable value types:

- `BlockPos`
- `ChunkPos`
- `SectionPos`
- `DimensionId`
- `ResourceId`
- stable entity IDs
- fixed-point or floating-point simulation vectors
- direction and axis enums
- bounding boxes
- common error types

It must not contain generic “utility” dumping grounds.

### `registry`

Owns:

- runtime registries
- persistent resource identifiers
- block state schemas
- item definitions
- entity kind definitions
- biome definitions
- runtime ID assignment
- registry snapshots for clients and saves

### `world`

Owns:

- chunks
- sections
- palette containers
- heightmaps
- light arrays
- block entities
- dimension storage
- chunk lifecycle state
- world read/write views

It must not implement specific gameplay behavior beyond storage invariants.

### `simulation`

Owns:

- `bevy_ecs::World`
- schedules
- tick orchestration
- simulation clock
- gameplay command buffers
- event buffers
- entity identity mapping
- phase barriers
- deterministic commit logic

### `gameplay`

Owns concrete mechanics:

- block behaviors
- item use
- crafting
- inventories
- entities and AI
- damage
- redstone
- fluids
- farming
- weather
- dimensions
- portals
- game rules

### `protocol`

Owns semantic messages, not socket APIs:

- client commands
- server snapshots
- chunk deltas
- entity deltas
- registry synchronization
- protocol versioning
- serialization codecs

### `persistence`

Owns:

- save metadata
- region storage
- write-ahead journal
- chunk codecs
- entity codecs
- migrations
- corruption recovery

### `server-runtime`

Owns:

- fixed tick loop
- client sessions
- authentication hooks
- chunk interest management
- player lifecycle
- network ingress and egress
- save scheduling
- server commands
- metrics

### `client-runtime`

Owns engine-independent replicated state:

- chunk cache
- registry mirror
- entity mirror
- interpolation buffers
- prediction state
- reconciliation
- outbound command generation
- frontend-facing events

### `client-bevy`

Owns:

- full Bevy application
- window and input
- cameras
- chunk mesh entities
- model entities
- shaders and materials
- audio
- UI
- visual effects
- frontend resource loading

### `testkit`

Owns:

- small in-memory worlds
- scenario DSL
- deterministic seed helpers
- snapshot comparison
- fake clocks
- local transports
- test entity factories

---

# 6. Runtime Topology

## 6.1 Dedicated Server

```text
Network Threads
    ↓ decoded messages
Ingress Queues
    ↓
Server Tick Thread
    ├── apply player commands
    ├── run simulation tick
    ├── update chunk interest
    ├── build per-client deltas
    └── enqueue persistence work
    ↓
Egress Queues
    ↓ encoded frames
Network Threads
```

The authoritative simulation should initially run on one logical tick coordinator. Internal phases may use worker threads where safe.

## 6.2 Embedded Single-Player Server

```text
Process
├── Bevy main/render thread
│   └── client-runtime
└── embedded server thread
    ├── server-runtime
    ├── simulation
    └── persistence
```

A separate server thread is preferred even for single-player because it:

- preserves client/server separation
- prevents render frame time from directly controlling world ticks
- reveals synchronization assumptions early
- makes dedicated server extraction trivial
- allows pausing policy to remain an explicit runtime feature

A same-thread mode may exist for deterministic tests.

## 6.3 Headless Simulation Test

```text
Scenario Input
    ↓
Simulation Core
    ↓ N ticks
State Snapshot
    ↓
Assertions
```

This mode must not initialize renderer, audio, networking, or filesystem persistence.

---

# 7. Simulation Core

## 7.1 Primary Ownership Model

The simulation runtime owns one Bevy ECS `World` per simulated dimension group or server world.

Recommended initial design:

```rust
pub struct Simulation {
    ecs: bevy_ecs::world::World,
    schedules: SimulationSchedules,
    tick: GameTick,
}
```

World voxel storage is inserted as an ECS resource:

```rust
#[derive(Resource)]
pub struct Worlds {
    dimensions: HashMap<DimensionId, DimensionWorld>,
}
```

Registries, clocks, queues, and runtime services are also resources.

Dynamic actors are ECS entities.

## 7.2 Why One ECS World Initially

One `bevy_ecs::World` for all dimensions simplifies:

- cross-dimensional player identity
- global registries
- shared server resources
- entity transfer
- scheduling
- administration
- metrics

Each entity carries a dimension component:

```rust
#[derive(Component)]
pub struct InDimension(pub DimensionId);
```

If profiling later proves that dimensions require isolation, the runtime may move to one ECS world per dimension. Do not start there unless needed.

## 7.3 Simulation Context

```rust
pub struct SimulationConfig {
    pub ticks_per_second: u32,
    pub max_neighbor_update_depth: u16,
    pub max_scheduled_ticks_per_chunk: usize,
    pub random_tick_speed: u32,
    pub deterministic_mode: bool,
}

#[derive(Resource)]
pub struct SimulationClock {
    pub current_tick: u64,
    pub day_time: u64,
    pub accumulated_real_time: std::time::Duration,
}
```

The default gameplay target should use a 20 Hz logical tick, but the frequency must be configured in one place and not embedded throughout mechanics.

## 7.4 Schedule Ownership

The simulation should own explicit schedules rather than depending on a default Bevy application schedule.

```rust
pub struct SimulationSchedules {
    pub ingress: Schedule,
    pub block_pre: Schedule,
    pub entities: Schedule,
    pub block_post: Schedule,
    pub commit: Schedule,
    pub publish: Schedule,
}
```

Alternatively, use one schedule with strongly ordered system sets. Multiple schedules are easier to inspect and harder to reorder accidentally.

## 7.5 Exclusive Systems as Semantic Barriers

Use normal parallel systems for read-heavy or independent work. Use exclusive systems when a phase requires ordered direct access to the ECS world and voxel state.

Examples:

- deterministic application of block commands
- neighbor update queue drain
- entity spawn/despawn reconciliation
- end-of-tick snapshot publication

An exclusive system is not a failure. It is an explicit semantic barrier.

---

# 8. Tick Pipeline and Determinism

## 8.1 Canonical Tick Pipeline

The first implementation should use this fixed ordering:

```text
0. Begin Tick
1. Drain Server Ingress
2. Validate and Normalize Player Commands
3. Apply Player Intent
4. Process Scheduled Block Ticks
5. Process Random Block Ticks
6. Process Immediate Neighbor Updates
7. Tick Block Entities
8. Tick Fluids
9. Tick Redstone
10. Tick Entity AI and Intent
11. Tick Entity Physics and Collision
12. Resolve Damage, Death, Drops, and Spawns
13. Commit Deferred World Changes
14. Drain Resulting Neighbor Updates
15. Finalize ECS Structural Changes
16. Build Replication Changes
17. Build Effects
18. End Tick
```

This ordering is a project specification. It may be refined as behavior is studied, but changes require an ADR and regression tests.

## 8.2 Phase Contract

Each phase must document:

- mutable resources
- mutable components
- messages consumed
- messages produced
- whether structural ECS commands are applied
- whether world mutations are immediate
- deterministic ordering key
- maximum work budget
- behavior when the budget is exceeded

Example:

```text
Phase: Scheduled Block Ticks

Reads:
- SimulationClock
- BlockRegistry

Writes:
- DimensionWorld
- NeighborUpdateQueue
- DeferredWorldCommands

Ordering:
- due tick ascending
- priority descending
- insertion sequence ascending

Budget:
- configurable maximum per dimension per tick

Overflow:
- remaining entries stay scheduled for the next tick
- lateness is recorded in metrics
```

## 8.3 Deterministic Ordering Keys

Never depend on:

- `HashMap` iteration order
- thread completion order
- Bevy archetype discovery order
- OS scheduling
- pointer addresses
- filesystem enumeration order

Use explicit ordering keys:

```rust
pub struct OrderedWorkKey {
    pub dimension: DimensionId,
    pub chunk: ChunkPos,
    pub local_index: u16,
    pub sequence: u64,
}
```

Use deterministic hashers or ordered maps in systems where iteration order affects gameplay.

## 8.4 Randomness

Use named random streams rather than one global generator.

```rust
pub enum RandomDomain {
    WorldGeneration,
    RandomBlockTick,
    EntityAi,
    Loot,
    Weather,
    ParticlePresentation,
}
```

A stream seed should be derived from stable inputs:

```text
world seed
+ dimension ID
+ tick
+ chunk/entity identifier
+ random domain
```

Gameplay randomness and presentation randomness must never share a stream.

## 8.5 Replayability

A simulation replay consists of:

```rust
pub struct ReplayHeader {
    pub format_version: u32,
    pub game_data_hash: [u8; 32],
    pub world_seed: u64,
    pub initial_snapshot: SnapshotRef,
}

pub struct ReplayTick {
    pub tick: u64,
    pub commands: Vec<ValidatedPlayerCommand>,
    pub admin_commands: Vec<ValidatedAdminCommand>,
}
```

Given the same initial snapshot, game data, and command stream, deterministic mode should produce the same canonical state hash.

---

# 9. World Data Model

## 9.1 Coordinates

Use strongly typed coordinates.

```rust
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

pub struct ChunkPos {
    pub x: i32,
    pub z: i32,
}

pub struct SectionY(pub i16);

pub struct LocalBlockPos {
    pub x: u8,
    pub y: u8,
    pub z: u8,
}
```

Do not pass raw `IVec3` everywhere because the coordinate domain matters.

## 9.2 Chunk and Section Layout

Recommended baseline:

- Chunk footprint: `16 × 16` blocks.
- Section size: `16 × 16 × 16`.
- Vertical dimension bounds are configured per dimension.
- Chunk columns contain optional sections.

```rust
pub struct ChunkColumn {
    pub position: ChunkPos,
    pub sections: Box<[Option<ChunkSection>]>,
    pub heightmaps: Heightmaps,
    pub block_entities: HashMap<BlockPos, BlockEntityData>,
    pub generation_status: GenerationStatus,
    pub revision: ChunkRevision,
    pub dirty: DirtyFlags,
}
```

```rust
pub struct ChunkSection {
    pub blocks: PalettedContainer<BlockStateId, 4096>,
    pub biomes: PalettedContainer<BiomeId, 64>,
    pub sky_light: Option<NibbleArray<2048>>,
    pub block_light: Option<NibbleArray<2048>>,
    pub non_air_count: u16,
    pub random_tickable_count: u16,
    pub revision: SectionRevision,
}
```

The exact biome sampling resolution is a project choice.

## 9.3 Palette Container

A palette container should support:

1. Single-value encoding.
2. Local palette with packed indices.
3. Direct global runtime IDs after a threshold.

```rust
pub enum PaletteMode<T> {
    Single(T),
    Local {
        palette: Vec<T>,
        bits_per_entry: u8,
        packed: Vec<u64>,
    },
    Direct {
        bits_per_entry: u8,
        packed: Vec<u64>,
    },
}
```

Invariants:

- Reads are bounds checked in debug builds.
- Palette compaction is not performed on every write.
- Bulk edits use a builder or mutable transaction.
- Serialization is versioned.
- Runtime IDs are never assumed stable across save migrations unless the registry snapshot is stored.

## 9.4 Loaded Chunk States

```rust
pub enum ChunkLifecycle {
    Absent,
    Requested,
    Loading,
    Generating(GenerationStatus),
    Ready,
    Unloading,
    Failed(ChunkFailure),
}
```

A chunk may be loaded but not simulation-active. Track separate concepts:

```rust
pub struct ChunkActivation {
    pub loaded: bool,
    pub ticking_blocks: bool,
    pub ticking_entities: bool,
    pub visible_to_clients: bool,
    pub eligible_for_save: bool,
}
```

## 9.5 Chunk Tickets

Chunk retention is controlled by tickets:

```rust
pub struct ChunkTicket {
    pub source: TicketSource,
    pub center: ChunkPos,
    pub level: TicketLevel,
    pub expires_at: Option<GameTick>,
}
```

Sources include:

- player view
- entity simulation
- portal
- forced load
- world generation dependency
- pending save
- block scheduled tick
- server administration

The ticket resolver computes effective chunk activation.

## 9.6 World Access Interfaces

Avoid exposing raw storage to all gameplay code.

```rust
pub trait BlockRead {
    fn block_state(&self, dimension: DimensionId, pos: BlockPos)
        -> Result<BlockStateId, WorldAccessError>;
}

pub trait BlockWrite: BlockRead {
    fn set_block(
        &mut self,
        dimension: DimensionId,
        pos: BlockPos,
        state: BlockStateId,
        context: BlockChangeContext,
    ) -> Result<BlockChangeResult, WorldMutationError>;
}
```

For performance-critical internal code, concrete views may expose more efficient batch APIs.

## 9.7 Read Views and Write Transactions

```rust
pub struct WorldReadView<'a> {
    // Immutable access to selected dimensions/chunks.
}

pub struct WorldTransaction<'a> {
    // Controlled mutation plus emitted consequences.
}
```

A transaction collects:

- old and new states
- dirty section revisions
- lighting invalidation
- heightmap invalidation
- neighbor updates
- replication deltas
- persistence dirty flags

No caller should manually remember every side effect of changing a block.

---

# 10. Registries and Game Data

## 10.1 Resource Identifiers

```rust
pub struct ResourceId {
    pub namespace: SmolStr,
    pub path: SmolStr,
}
```

Examples:

```text
core:stone
core:oak_log
core:zombie
core:overworld
```

Names are persistent. Numeric IDs are runtime-local.

## 10.2 Runtime IDs

```rust
#[repr(transparent)]
pub struct BlockId(pub u16);

#[repr(transparent)]
pub struct BlockStateId(pub u32);

#[repr(transparent)]
pub struct ItemId(pub u16);

#[repr(transparent)]
pub struct EntityKindId(pub u16);

#[repr(transparent)]
pub struct BiomeId(pub u16);
```

These types must not be interchangeable.

## 10.3 Block Definitions and States

```rust
pub struct BlockDefinition {
    pub id: ResourceId,
    pub behavior: BlockBehaviorId,
    pub material: MaterialId,
    pub state_schema: BlockStateSchema,
    pub default_state: BlockStateId,
    pub hardness: f32,
    pub blast_resistance: f32,
    pub luminance: u8,
    pub opacity: u8,
    pub collision_shape: VoxelShapeId,
    pub outline_shape: VoxelShapeId,
    pub flags: BlockFlags,
}
```

A block state should be a compact registry entry rather than an owned property map in each chunk cell.

```rust
pub struct BlockStateDefinition {
    pub block: BlockId,
    pub packed_properties: u64,
    pub derived: DerivedBlockProperties,
}
```

Derived properties may include:

- collision shape
- opacity
- luminance
- random-tick flag
- fluid occupancy
- render layer
- occlusion flags

Precomputing common derived values avoids repeated property decoding.

## 10.4 Registry Freeze

Registries have a construction phase and a frozen runtime phase.

```text
Load built-in definitions
    ↓
Load data packs or project data
    ↓
Validate references
    ↓
Assign runtime IDs
    ↓
Build derived lookup tables
    ↓
Freeze
```

Runtime mutation of registries is not allowed initially. A future mod reload system should build a new registry snapshot and migrate deliberately.

## 10.5 Data Hash

Compute a canonical game data hash covering:

- registries
- recipes
- loot tables
- world generation definitions
- behavior configuration
- relevant scripts

The hash is stored in saves, replays, and network handshakes.

---

# 11. Entity Model with Bevy ECS

## 11.1 Dynamic Entity Components

Example component groups:

```rust
#[derive(Component)]
pub struct PersistentId(pub PersistentEntityId);

#[derive(Component)]
pub struct EntityKind(pub EntityKindId);

#[derive(Component)]
pub struct InDimension(pub DimensionId);

#[derive(Component)]
pub struct SimTransform {
    pub position: DVec3,
    pub yaw: f32,
    pub pitch: f32,
}

#[derive(Component)]
pub struct Velocity(pub DVec3);

#[derive(Component)]
pub struct Collider(pub ColliderId);

#[derive(Component)]
pub struct Health {
    pub current: f32,
    pub maximum: f32,
}

#[derive(Component)]
pub struct InventoryComponent(pub InventoryId);

#[derive(Component)]
pub struct Replicated;

#[derive(Component)]
pub struct PlayerControlled {
    pub session: PlayerSessionId,
}
```

## 11.2 Do Not Mirror Every Block Entity into ECS Automatically

Block entities have two possible representations:

### Inline world representation

Best for simple, chunk-owned state:

```rust
pub enum BlockEntityData {
    Furnace(FurnaceData),
    Chest(ChestData),
    Sign(SignData),
}
```

### ECS-backed representation

Best when it needs dynamic ticking, relationships, or shared entity systems.

```rust
#[derive(Component)]
pub struct AnchoredBlockEntity {
    pub dimension: DimensionId,
    pub position: BlockPos,
}
```

Start with inline block entities. Promote individual categories to ECS only when the benefit is proven.

## 11.3 Stable Identity Mapping

```rust
#[derive(Resource, Default)]
pub struct EntityIdentityMap {
    pub persistent_to_ecs: HashMap<PersistentEntityId, Entity>,
    pub ecs_to_persistent: HashMap<Entity, PersistentEntityId>,
}
```

Network IDs are session-scoped or server-runtime scoped:

```rust
pub struct ClientEntityMap {
    pub network_to_ecs: HashMap<NetworkEntityId, Entity>,
    pub ecs_to_network: HashMap<Entity, NetworkEntityId>,
}
```

## 11.4 ECS Structural Mutation Policy

Normal component value changes can occur inside systems according to Bevy ECS borrowing rules.

Structural changes are deferred:

- spawn
- despawn
- add component
- remove component

Use ECS `Commands`, then apply them at explicit phase barriers. Do not let deferred ECS operations cross a phase where their visibility matters.

## 11.5 Archetype Discipline

Avoid highly fragmented component combinations for common mobs. Prefer a small number of stable archetypes and data components.

Do not encode every boolean state as a marker component if it causes constant archetype transitions. Use bitflags or state enums for frequently changing conditions.

Good marker components:

- `Player`
- `Projectile`
- `ItemDrop`
- `HostileMob`
- `Persistent`

Potentially bad marker components when frequently toggled:

- `OnGround`
- `Burning`
- `Sprinting`
- `Swimming`

These may belong in compact state components.

---

# 12. World Mutation Model

## 12.1 Mutation Sources

```rust
pub enum BlockChangeCause {
    PlayerPlacement,
    PlayerBreaking,
    Explosion,
    Piston,
    Fluid,
    Fire,
    Growth,
    Decay,
    EntityInteraction,
    WorldGeneration,
    StructureGeneration,
    Command,
    InternalSimulation,
}
```

## 12.2 Mutation Context

```rust
pub struct BlockChangeContext {
    pub cause: BlockChangeCause,
    pub actor: Option<PersistentEntityId>,
    pub flags: BlockUpdateFlags,
    pub recursion_depth: u16,
    pub sequence: MutationSequence,
}
```

Flags may include:

```rust
bitflags::bitflags! {
    pub struct BlockUpdateFlags: u32 {
        const NOTIFY_NEIGHBORS   = 1 << 0;
        const UPDATE_LIGHT       = 1 << 1;
        const UPDATE_HEIGHTMAP   = 1 << 2;
        const SEND_TO_CLIENTS    = 1 << 3;
        const DROP_ITEMS         = 1 << 4;
        const CALL_HOOKS         = 1 << 5;
        const FORCE_IMMEDIATE    = 1 << 6;
    }
}
```

## 12.3 World Commands

```rust
pub enum WorldCommand {
    SetBlock {
        dimension: DimensionId,
        position: BlockPos,
        state: BlockStateId,
        context: BlockChangeContext,
    },
    SpawnEntity(EntitySpawnRequest),
    RemoveEntity {
        id: PersistentEntityId,
        reason: RemovalReason,
    },
    ScheduleBlockTick(ScheduledBlockTickRequest),
    EmitGameEffect(GameEffect),
    ModifyInventory(InventoryMutation),
}
```

## 12.4 Command Buffers

Use separate buffers by phase or producer:

```rust
#[derive(Resource, Default)]
pub struct DeferredWorldCommands {
    pub commands: Vec<SequencedWorldCommand>,
}
```

Each command receives a stable sequence number when created.

Parallel producers may write into thread-local buffers, followed by deterministic merge.

## 12.5 Immediate Mutation API

Some mechanics require recursive or immediate observation.

```rust
pub enum MutationMode {
    Immediate,
    EndOfPhase,
    EndOfTick,
}
```

Immediate mutation must:

- run under controlled exclusive access
- enforce recursion depth
- append audit records
- trigger required secondary updates
- avoid arbitrary reentrant ECS schedule execution

## 12.6 Mutation Audit Log

In debug builds and test mode, record:

```rust
pub struct MutationRecord {
    pub tick: u64,
    pub sequence: u64,
    pub cause: BlockChangeCause,
    pub position: BlockPos,
    pub old_state: BlockStateId,
    pub new_state: BlockStateId,
    pub actor: Option<PersistentEntityId>,
}
```

This is invaluable for redstone, fluid, and neighbor-update debugging.

---

# 13. Block Behavior Model

## 13.1 Static Behavior Table

Avoid one heap object per block type or state.

```rust
pub struct BlockBehavior {
    pub on_place: Option<OnPlaceFn>,
    pub on_break: Option<OnBreakFn>,
    pub on_use: Option<OnUseFn>,
    pub on_neighbor_update: Option<OnNeighborUpdateFn>,
    pub on_scheduled_tick: Option<OnScheduledTickFn>,
    pub on_random_tick: Option<OnRandomTickFn>,
    pub on_entity_inside: Option<OnEntityInsideFn>,
    pub on_projectile_hit: Option<OnProjectileHitFn>,
}
```

Function signatures should use explicit contexts:

```rust
pub type OnNeighborUpdateFn = fn(
    ctx: &mut BlockBehaviorContext<'_>,
    position: BlockPos,
    state: BlockStateId,
    source_position: BlockPos,
    source_state: BlockStateId,
);
```

## 13.2 Behavior Context

```rust
pub struct BlockBehaviorContext<'a> {
    pub tick: GameTick,
    pub dimension: DimensionId,
    pub world: &'a mut WorldTransaction<'a>,
    pub registries: &'a FrozenRegistries,
    pub random: &'a mut BehaviorRandom,
    pub commands: &'a mut DeferredWorldCommands,
}
```

Do not provide unrestricted access to the entire ECS world unless absolutely necessary.

## 13.3 Data Versus Code

Prefer data for:

- hardness
- resistance
- sound group
- luminance
- shapes
- state properties
- drops
- simple recipes
- tags

Use Rust behavior code for:

- pistons
- redstone components
- fluids
- crops
- fire
- portals
- complex containers
- special item interactions

## 13.4 Tags

Use precomputed tag bitsets or compact membership structures for hot paths:

```text
core:mineable/pickaxe
core:logs
core:replaceable
core:redstone_connectable
core:blocks_motion
```

Do not perform string lookups in per-block tick code.

---

# 14. Tick Scheduling and Neighbor Updates

## 14.1 Scheduled Tick Entry

```rust
pub struct ScheduledBlockTick {
    pub due_tick: u64,
    pub priority: TickPriority,
    pub sequence: u64,
    pub dimension: DimensionId,
    pub position: BlockPos,
    pub expected_block: BlockId,
    pub kind: ScheduledTickKind,
}
```

`expected_block` prevents stale ticks from applying to a replacement block unless the mechanic intentionally allows it.

## 14.2 Scheduler Structure

Use:

- a near-future timing wheel for common short delays
- a heap or ordered map for distant ticks
- per-chunk indexes for unload/reload handling
- deduplication keys where behavior requires it

```rust
pub struct TickScheduler {
    near: TimingWheel<ScheduledBlockTick>,
    far: BinaryHeap<Reverse<ScheduledBlockTick>>,
    by_chunk: HashMap<ChunkKey, SmallVec<[TickHandle; 8]>>,
}
```

## 14.3 Unloaded Chunks

Define one policy per tick category:

- keep the tick serialized with the chunk
- retain a global ticket until execution
- defer until the chunk becomes active
- cancel

The policy must be explicit. Never silently lose scheduled work due to chunk unload.

## 14.4 Neighbor Update Queue

```rust
pub struct NeighborUpdate {
    pub target: BlockPos,
    pub source: BlockPos,
    pub direction: Direction,
    pub sequence: u64,
    pub depth: u16,
}
```

Use a FIFO queue unless a specific behavior requires another order.

## 14.5 Recursion and Work Limits

Protect against pathological update graphs:

```rust
pub struct UpdateBudget {
    pub max_depth: u16,
    pub max_updates_per_tick: usize,
    pub max_updates_per_origin: usize,
}
```

Exceeding a budget should:

- preserve server liveness
- produce structured diagnostics
- optionally pause the affected circuit in debug mode
- not corrupt world state

---

# 15. Physics and Collision

## 15.1 Simulation Representation

Use double-precision world positions in the server core unless profiling proves otherwise.

```rust
pub struct SimTransform {
    pub position: DVec3,
    pub yaw: f32,
    pub pitch: f32,
}
```

Chunk-relative rendering can convert to `f32`.

## 15.2 Collider Model

Use voxel shapes composed of a small number of axis-aligned boxes.

```rust
pub struct VoxelShape {
    pub boxes: SmallVec<[Aabb; 4]>,
}
```

Cache shapes by `VoxelShapeId`.

## 15.3 Movement Pipeline

```text
Read movement intent
    ↓
Apply acceleration, drag, and status effects
    ↓
Compute desired displacement
    ↓
Collect nearby block collision shapes
    ↓
Resolve axis movement in specified order
    ↓
Attempt step-up
    ↓
Resolve entity interactions
    ↓
Update grounded/fluid/climbing state
    ↓
Commit transform and velocity
```

Axis order is observable and must be specified.

## 15.4 Broad Phase

Use a chunk- or section-partitioned spatial index for dynamic entities.

```rust
pub struct EntitySpatialIndex {
    buckets: HashMap<SpatialCell, SmallVec<[Entity; 16]>>,
}
```

Update it after entity movement commit, not during every partial movement calculation.

## 15.5 Client Prediction

Only player movement should be predicted initially.

The client stores:

- input sequence
- predicted position
- server-confirmed position
- unacknowledged inputs

On authoritative correction:

```text
restore confirmed state
    ↓
reapply unacknowledged inputs
    ↓
smooth visual error
```

World interactions such as block placement may be visually predicted but remain server-validated.

---

# 16. Redstone-Like Logic

## 16.1 Goals

The redstone subsystem should reproduce externally visible behavior while using an explicit architecture.

It must model:

- weak and strong power
- direct and indirect connection rules
- repeaters
- comparators
- observers
- torches
- dust connectivity
- pistons
- block events
- delayed transitions
- update ordering quirks selected by the project

## 16.2 Redstone Data

Frequently queried derived state should be encoded in block states.

Examples:

- dust connection directions
- power level
- repeater delay
- repeater facing
- comparator mode
- powered flag
- piston extension state

## 16.3 Dirty Graph Approach

```text
Block mutation
    ↓
Mark redstone positions dirty
    ↓
Expand dependency neighborhood
    ↓
Sort by deterministic update key
    ↓
Evaluate signals
    ↓
Commit state changes
    ↓
Schedule delayed components
    ↓
Generate new neighbor updates
```

Do not rebuild a permanent global graph for the whole world initially. Most circuits are local and sparse.

## 16.4 Piston Transactions

Pistons require an atomic planning stage:

```text
Discover push chain
    ↓
Validate limits and immovable blocks
    ↓
Compute destroy/move order
    ↓
Reserve affected cells
    ↓
Commit movement atomically
    ↓
Emit neighbor and block events
```

Represent the plan:

```rust
pub struct PistonMovePlan {
    pub moved: Vec<(BlockPos, BlockPos, BlockStateId)>,
    pub destroyed: Vec<(BlockPos, BlockStateId)>,
    pub source: BlockPos,
    pub direction: Direction,
}
```

Never mutate the world while still discovering the push chain.

## 16.5 Redstone Test Strategy

Every component should have:

- isolated truth-table tests
- delay tests
- update-order tests
- circuit-level scenario tests
- piston integration tests
- regression tests for discovered edge cases

---

# 17. Fluid Simulation

## 17.1 Representation

Fluid may be represented as block states or a layered block/fluid model. Choose one model and document it.

Recommended initial model:

```rust
pub struct FluidState {
    pub fluid: FluidId,
    pub level: u8,
    pub falling: bool,
}
```

The block state definition may reference a fluid state for waterlogged or occupied blocks.

## 17.2 Fluid Update Pipeline

```text
Block or neighbor changed
    ↓
Schedule fluid tick
    ↓
Read source and neighbors
    ↓
Compute candidate downward flow
    ↓
Compute lateral flow
    ↓
Apply source rules
    ↓
Commit affected cells
    ↓
Schedule follow-up ticks
```

## 17.3 Deterministic Flow

When several directions are equally valid, choose using a specified stable direction order or a stable hash. Do not depend on map iteration order.

## 17.4 Fluid Budget

Large releases of fluid can create update storms. Apply:

- per-dimension tick budget
- deduplication
- delayed continuation
- metrics for queue size and lateness

The result may spread over more ticks under load, but must not become nondeterministic.

---

# 18. Lighting

## 18.1 Channels

Baseline channels:

- skylight: 0–15
- block light: 0–15

Store each as four-bit values in `NibbleArray`.

## 18.2 Lighting Ownership

Lighting is world-derived data. It belongs in the world layer, but update algorithms may live in a dedicated subsystem.

## 18.3 Update Model

Use separate increase and decrease queues:

```rust
pub struct LightUpdateQueues {
    pub sky_increase: VecDeque<LightNode>,
    pub sky_decrease: VecDeque<LightNode>,
    pub block_increase: VecDeque<LightNode>,
    pub block_decrease: VecDeque<LightNode>,
}
```

Removing a source requires decrease propagation followed by re-increase from surviving sources.

## 18.4 Cross-Chunk Boundaries

When a neighboring chunk is unavailable:

- store boundary light obligations
- retain a generation dependency ticket when required
- reconcile boundaries when the neighbor loads
- mark the affected section revision

Do not assume all neighboring chunks are present.

## 18.5 Client Lighting

The server owns authoritative light values if lighting affects gameplay. The client may apply purely visual enhancements, but those must not change visibility-sensitive gameplay rules.

---

# 19. World Generation

## 19.1 Generation Stages

```rust
pub enum GenerationStatus {
    Empty,
    Climate,
    Terrain,
    Carvers,
    Surface,
    Structures,
    Features,
    Lighting,
    Complete,
}
```

Each stage declares dependencies on neighboring chunks.

## 19.2 Pipeline

```text
Seed and dimension settings
    ↓
Climate field sampling
    ↓
Biome selection
    ↓
Density functions
    ↓
Base terrain fill
    ↓
Cave carving
    ↓
Surface rules
    ↓
Structure placement
    ↓
Feature decoration
    ↓
Initial lighting
```

## 19.3 Stateless and Stateful Generation

Prefer stateless generation functions derived from:

- world seed
- stage
- chunk coordinate
- feature identifier

This improves parallelism and reproducibility.

Structure systems that require region coordination should use explicit region-level planning data.

## 19.4 Generation Tasks

World generation runs outside the authoritative tick thread.

A task receives:

```rust
pub struct GenerationRequest {
    pub dimension: DimensionId,
    pub chunk: ChunkPos,
    pub target: GenerationStatus,
    pub input_revisions: NeighborRevisionSet,
}
```

A result is accepted only if:

- the chunk is still requested
- dependency revisions remain valid
- no newer result exists

## 19.5 Fidelity Boundary

World generation targets `EquivalentPlayerVisibleBehavior`, as defined by `WGEN-*` in the version-locked gameplay reference. It must preserve player-visible terrain classes, reachability, biome/structure constraints, resource distributions, and dependent gameplay, but does not promise block-for-block identity for the same seed. Runtime mechanics that consume generated state retain their own fidelity classes.

---

# 20. Server Runtime

## 20.1 Server State

```rust
pub struct ServerRuntime {
    pub simulation: Simulation,
    pub clients: ClientSessions,
    pub chunk_manager: ChunkManager,
    pub persistence: PersistenceCoordinator,
    pub network: Box<dyn ServerTransport>,
    pub config: ServerConfig,
}
```

## 20.2 Tick Loop

```rust
loop {
    let deadline = tick_clock.next_deadline();

    drain_network_ingress();
    validate_commands();
    run_one_simulation_tick();
    update_client_interest();
    build_replication();
    enqueue_network_egress();
    schedule_saves();

    tick_clock.sleep_until(deadline);
}
```

If the server falls behind, define a policy:

- run catch-up ticks up to a limit
- never skip authoritative ticks silently
- expose tick debt
- degrade optional work before gameplay work
- disconnect or throttle abusive clients
- avoid an infinite spiral of death

## 20.3 Chunk Interest Management

Each client has:

```rust
pub struct ClientInterest {
    pub center: ChunkPos,
    pub view_distance: u16,
    pub simulation_distance: u16,
    pub known_chunks: HashMap<ChunkPos, KnownChunkState>,
}
```

Prioritize sending chunks by:

- distance
- camera direction
- player velocity
- spawn safety
- already partially transmitted data

## 20.4 Replication

Maintain change journals during the tick:

```rust
pub struct ReplicationJournal {
    pub block_changes: Vec<BlockDelta>,
    pub entity_spawns: Vec<EntitySpawnDelta>,
    pub entity_updates: Vec<EntityUpdateDelta>,
    pub entity_removals: Vec<EntityRemovalDelta>,
    pub effects: Vec<GameEffect>,
}
```

Filter the journal per client interest instead of rescanning the whole world.

## 20.5 Backpressure

Each client connection has bounded queues.

When a client cannot keep up:

- collapse obsolete entity snapshots
- combine block changes by position
- prioritize essential state over effects
- stop scheduling new chunks
- disconnect after a configured threshold

Never allow one slow client to grow memory without bound.

---

# 21. Protocol and Transport

## 21.1 Semantic Protocol

The protocol represents game meaning rather than mirroring Rust memory layout.

Client messages:

```rust
pub enum ClientMessage {
    Hello(ClientHello),
    PlayerInput(PlayerInputFrame),
    InteractBlock(BlockInteraction),
    InteractEntity(EntityInteraction),
    InventoryAction(InventoryAction),
    Chat(ChatMessage),
    ClientSettings(ClientSettings),
    ChunkAck(ChunkAck),
}
```

Server messages:

```rust
pub enum ServerMessage {
    Welcome(ServerWelcome),
    RegistrySnapshot(RegistrySnapshot),
    PlayerSpawn(PlayerSpawn),
    ChunkSnapshot(ChunkSnapshot),
    ChunkDelta(ChunkDelta),
    EntitySpawn(EntitySpawnSnapshot),
    EntityDelta(EntityDelta),
    EntityRemove(EntityRemove),
    InventorySnapshot(InventorySnapshot),
    InventoryDelta(InventoryDelta),
    GameEffect(GameEffect),
    TimeSync(TimeSync),
    Disconnect(DisconnectReason),
}
```

## 21.2 Protocol Versioning

```rust
pub struct ProtocolVersion {
    pub major: u16,
    pub minor: u16,
}
```

Policy:

- major mismatch may reject connection
- minor versions may negotiate optional features
- every message has bounded decoding
- registry hash is part of handshake
- save format version is independent from network protocol version

## 21.3 Transport Abstraction

```rust
pub trait ServerTransport: Send {
    fn poll_events(&mut self, out: &mut Vec<TransportEvent>);
    fn send(&mut self, client: ConnectionId, channel: Channel, bytes: Bytes);
    fn disconnect(&mut self, client: ConnectionId, reason: DisconnectReason);
}
```

Implementations may include:

- local in-process transport
- QUIC
- UDP reliability layer
- TCP for early development
- replay transport

The simulation core does not depend on this trait.

## 21.4 Channels

Recommended semantic channels:

```text
Control       reliable ordered
Inventory     reliable ordered
WorldEdits    reliable ordered
ChunkData     reliable unordered or independently sequenced
EntityState   unreliable sequenced
PlayerInput   unreliable sequenced with redundancy
Effects       unreliable
Chat          reliable ordered
```

Exact transport mapping is an implementation detail.

## 21.5 Serialization

Requirements:

- explicit integer widths
- bounded lengths
- no direct `bincode` of arbitrary internal structs as the permanent format
- versioned schemas
- fuzz-tested decoders
- zero-copy slices where practical
- compression only above measured thresholds

---

# 22. Engine-Independent Client Runtime

## 22.1 Client Runtime State

```rust
pub struct ClientRuntime {
    pub registries: ClientRegistries,
    pub chunks: ClientChunkCache,
    pub entities: ClientEntityStore,
    pub local_player: LocalPlayerState,
    pub prediction: PredictionState,
    pub connection: Box<dyn ClientTransport>,
    pub frontend_events: VecDeque<FrontendEvent>,
}
```

This crate should not depend on full Bevy.

## 22.2 Frontend Events

```rust
pub enum FrontendEvent {
    ChunkBecameAvailable(ChunkRenderSnapshot),
    ChunkChanged(ChunkSectionDelta),
    ChunkUnloaded(ChunkPos),
    EntitySpawned(ClientEntitySnapshot),
    EntityUpdated(ClientEntityDelta),
    EntityRemoved(NetworkEntityId),
    InventoryChanged(InventoryView),
    PlaySound(SoundPresentationEvent),
    SpawnParticles(ParticlePresentationEvent),
    ShowMessage(TextMessage),
}
```

## 22.3 Frontend Commands

```rust
pub enum FrontendCommand {
    SetMovementInput(MovementInput),
    LookDelta { yaw: f32, pitch: f32 },
    UseSelectedItem,
    Attack,
    Interact,
    SelectHotbarSlot(u8),
    OpenInventory,
    SendChat(String),
}
```

The Bevy adapter translates input events into these commands.

## 22.4 Client Chunk Cache

The client keeps only replicated data needed for:

- rendering
- collision prediction
- block targeting
- local light
- visual effects

It must not receive server-only data such as hidden inventories or AI internals.

---

# 23. Bevy Frontend

## 23.1 Responsibilities

The Bevy frontend owns:

- application lifecycle
- windowing
- input mapping
- rendering
- camera
- chunk mesh entities
- entity visual entities
- materials
- shaders
- textures
- audio
- UI
- debug overlays
- visual interpolation

## 23.2 Non-Responsibilities

The frontend does not decide:

- whether placement is legal
- final player position
- item ownership
- damage
- entity drops
- recipe validity
- redstone state
- fluid state
- save contents

## 23.3 Frontend Entity Mapping

```rust
#[derive(Resource, Default)]
pub struct RenderEntityMap {
    pub network_to_bevy: HashMap<NetworkEntityId, Entity>,
}
```

A Bevy render entity is disposable. If the frontend reloads, it can reconstruct render entities from the client-runtime mirror.

## 23.4 Chunk Render Entity

Recommended granularity:

- one Bevy entity per chunk section and render layer, or
- one parent per chunk with child entities per section/layer

Example:

```text
Chunk (4, -3)
├── Section 0 Opaque
├── Section 0 Cutout
├── Section 0 Translucent
├── Section 1 Opaque
└── ...
```

Do not create one Bevy entity per block.

## 23.5 Render Origin Rebasing

For distant worlds, use a floating render origin.

```text
Authoritative position: f64 world coordinates
Render position: f32 relative to current render origin
```

When the player crosses a threshold, move the render origin and update visible transforms.

---

# 24. Chunk Meshing and Rendering

## 24.1 Mesh Inputs

A meshing task receives an immutable snapshot containing:

- center section block states
- one-block border from neighboring sections
- light values
- biome tint data
- registry-derived model information
- section revision
- resource-pack revision

```rust
pub struct SectionMeshingInput {
    pub position: SectionPos,
    pub revision: SectionRevision,
    pub neighborhood: Arc<SectionNeighborhoodSnapshot>,
    pub render_registry: Arc<RenderRegistry>,
}
```

## 24.2 Render Layers

At minimum:

- opaque
- cutout
- translucent

Potential later layers:

- fluids
- emissive
- animated
- decals

## 24.3 Meshing Algorithm

Initial sequence:

1. Face culling.
2. Per-face ambient occlusion.
3. Texture atlas or texture array addressing.
4. Separate render layers.
5. Greedy meshing for compatible faces.
6. Packed vertex formats.
7. GPU culling and indirect drawing if profiling requires it.

Greedy merge compatibility includes:

- material
- texture
- light values
- tint
- face orientation
- ambient occlusion pattern
- render layer

## 24.4 Asynchronous Revision Safety

```rust
pub struct MeshBuildResult {
    pub section: SectionPos,
    pub input_revision: SectionRevision,
    pub resource_revision: u64,
    pub meshes: SectionMeshes,
}
```

Accept only when both revisions still match. Old tasks are dropped.

## 24.5 Mesh Upload Budget

GPU upload occurs on the render/main thread under a per-frame budget.

Prioritize:

1. nearest missing opaque geometry
2. geometry in camera direction
3. changed nearby sections
4. cutout
5. translucent
6. distant rebuilds

## 24.6 Translucency

Initial approach:

- one translucent mesh per section
- back-to-front section sorting
- accept imperfect per-face order

Later approaches may include order-independent transparency or smaller translucent batches.

---

# 25. Persistence

## 25.1 Save Layout

```text
world/
├── world.meta
├── registries.snapshot
├── dimensions/
│   ├── overworld/
│   │   ├── regions/
│   │   ├── entities/
│   │   └── dimension.meta
│   └── nether/
├── players/
├── journal/
└── backups/
```

## 25.2 Region Storage

A region contains many independently compressed chunk records.

Each record includes:

```rust
pub struct ChunkRecordHeader {
    pub magic: [u8; 4],
    pub format_version: u32,
    pub chunk_x: i32,
    pub chunk_z: i32,
    pub uncompressed_len: u32,
    pub compressed_len: u32,
    pub checksum: u64,
    pub generation: u64,
}
```

The region index uses append-and-repoint semantics rather than in-place chunk overwrite.

## 25.3 Write-Ahead Journal

Save transaction:

```text
encode new records
    ↓
append journal intent
    ↓
fsync journal
    ↓
append region data
    ↓
fsync region data
    ↓
update region index
    ↓
fsync index
    ↓
mark journal transaction committed
```

Batching fsync operations is configurable.

## 25.4 Snapshot Boundary

A chunk save snapshot must be captured at a simulation phase boundary. Serialization and compression occur asynchronously from immutable data.

```rust
pub struct ChunkSaveSnapshot {
    pub position: ChunkPos,
    pub revision: ChunkRevision,
    pub sections: Arc<[SectionSaveSnapshot]>,
    pub block_entities: Arc<[BlockEntitySaveRecord]>,
    pub scheduled_ticks: Arc<[ScheduledTickSaveRecord]>,
}
```

When a save completes, clear dirty state only if the saved revision is still current.

## 25.5 Entity Persistence

Persist:

- persistent ID
- entity kind
- dimension
- transform
- persistent components
- relationships
- inventory references

Do not serialize arbitrary ECS archetypes directly. Use explicit versioned records.

## 25.6 Migrations

Each record has a schema version. Migration functions are pure transforms:

```text
v1 record → v2 record → v3 record
```

Never require loading the entire world to migrate one chunk.

## 25.7 Corruption Handling

A corrupted chunk must not make the entire world unloadable.

Policy options:

- load last valid generation
- restore from journal
- quarantine the record
- regenerate terrain while preserving an audit report
- require operator confirmation for destructive repair

---

# 26. Concurrency and Job Scheduling

## 26.1 Thread Categories

Recommended conceptual pools:

- authoritative tick thread
- general CPU worker pool
- I/O workers
- network runtime
- render/main thread

Physical pools may be shared initially.

## 26.2 Job Types

```rust
pub enum JobKind {
    GenerateChunk,
    LightChunk,
    MeshSection,
    CompressChunk,
    DecompressChunk,
    Pathfind,
    EncodeSnapshot,
}
```

Each job carries:

- priority
- cancellation token
- input revisions
- memory estimate
- owner
- result destination

## 26.3 Bounded Queues

Every queue must have a limit or backpressure policy:

- chunk generation requests
- mesh jobs
- save snapshots
- network output
- pathfinding requests
- light updates
- behavior events

Unbounded queues are prohibited in production paths.

## 26.4 Cancellation

Chunk-related jobs are canceled when:

- the chunk loses all tickets
- a newer revision supersedes the input
- the client disconnects
- the dimension unloads
- shutdown begins

Cancellation must be cooperative.

## 26.5 Memory Accounting

Track approximate memory by subsystem:

- loaded chunk data
- section palettes
- light arrays
- mesh CPU buffers
- GPU meshes
- client snapshots
- save queues
- network queues
- ECS tables

The chunk manager uses budgets, not only distances.

## 26.6 Safe Parallelism Model

For order-sensitive systems:

```text
Snapshot read state
    ↓
Parallel calculation of intents
    ↓
Sort intents by stable key
    ↓
Single ordered commit
```

For independent chunks:

```text
Partition by chunk ownership
    ↓
Parallel mutation of non-overlapping partitions
    ↓
Boundary exchange
    ↓
Deterministic reconciliation
```

Do not implement cross-chunk parallel mutation until single-thread semantics are well tested.

---

# 27. Replay, Testing, and Behavioral Specifications

Every gameplay scenario should cite the rule ID from the [26.2 behavioral reference](reference/minecraft-java-26.2/README.md) that defines its expected result. If the relevant rule has an open verification item, the test should first reproduce vanilla `26.2` against the locked evidence artifacts rather than inventing a result from the architecture alone.

## 27.1 Test Pyramid

### Unit tests

- coordinate conversions
- palette packing
- registry lookup
- shape intersections
- scheduler ordering
- serialization bounds

### Subsystem tests

- block behavior
- fluid propagation
- redstone components
- inventory transactions
- collision
- lighting

### Scenario tests

Small worlds with scripted actions and expected states.

### Long-running simulations

- random player actions
- stress circuits
- fluid floods
- repeated save/load
- chunk churn
- many entities

### Fuzzing

- protocol decoding
- save decoding
- palette mutation
- inventory actions
- malformed registry data

## 27.2 Scenario Format

```yaml
name: sand_falls_after_support_is_removed
seed: 1
dimension: core:overworld

initial:
  blocks:
    - pos: [0, 0, 0]
      state: core:stone
    - pos: [0, 1, 0]
      state: core:sand

actions:
  - at_tick: 0
    break_block:
      player: test:player
      pos: [0, 0, 0]

run_ticks: 20

expect:
  blocks:
    - pos: [0, 0, 0]
      state: core:sand
  no_block:
    - [0, 1, 0]
```

## 27.3 Canonical State Hash

For deterministic tests, hash a canonical ordered representation of:

- blocks
- block entities
- scheduled ticks
- entities by persistent ID
- inventories
- time
- selected queues

Exclude:

- Bevy runtime entity IDs
- pointer addresses
- thread IDs
- presentation-only state
- metrics
- unordered cache contents

## 27.4 Behavioral Reference Process

When studying reference gameplay:

1. Build the smallest possible scenario.
2. Record initial state.
3. Apply one controlled action.
4. Observe state each tick.
5. Record visible and timing behavior.
6. Write a specification.
7. Implement independently.
8. Add regression tests.
9. Mark fidelity class and known deviations.

Do not copy implementation structure merely because a reference implementation uses it.

## 27.5 Golden World Tests

Generate selected chunks from fixed seeds and store hashes for:

- block states
- biome samples
- heightmaps
- structures

Golden data changes only through explicit review.

---

# 28. Observability and Developer Tooling

## 28.1 Metrics

Server metrics:

- tick duration
- phase duration
- tick debt
- loaded chunks
- active chunks
- entity count
- scheduled tick queue size
- neighbor update queue size
- fluid queue size
- light queue size
- chunk generation latency
- save latency
- network bytes by message type
- per-client backlog
- memory by subsystem

## 28.2 Tracing

Use structured spans:

```text
server_tick
├── ingress
├── scheduled_blocks
├── neighbor_updates
├── redstone
├── entities
├── commit
├── replication
└── persistence_snapshot
```

Chunk jobs include chunk coordinates and revision.

## 28.3 Debug Commands

Initial administration commands:

```text
/tick pause
/tick step <count>
/tick rate <hz>
/chunk info <x> <z>
/chunk tickets <x> <z>
/entity inspect <id>
/updates trace <x> <y> <z>
/redstone trace <x> <y> <z>
/save flush
/state hash
```

## 28.4 World Inspector

A separate tool should inspect:

- region files
- chunk sections
- palettes
- block entities
- scheduled ticks
- lighting
- entity records
- registry snapshots
- corruption status

It should not require launching the game client.

## 28.5 Tick Timeline

In debug mode, retain a short rolling timeline:

```rust
pub struct TickDebugFrame {
    pub tick: u64,
    pub phase_timings: Vec<PhaseTiming>,
    pub mutations: Vec<MutationRecord>,
    pub queue_sizes: QueueSizes,
    pub state_hash: Option<StateHash>,
}
```

---

# 29. Error Handling and Recovery

## 29.1 Error Categories

```rust
pub enum ErrorSeverity {
    RecoverableInput,
    RecoverableChunk,
    ClientFatal,
    WorldFatal,
    ProcessFatal,
}
```

Examples:

- invalid inventory click: recoverable input
- corrupt one chunk record: recoverable chunk
- registry mismatch after handshake: client fatal
- missing mandatory registry entry: world fatal
- memory allocator failure: process fatal

## 29.2 No Panics for Untrusted Input

Network packets, save records, data files, and commands must return structured errors.

Panics are acceptable for violated internal invariants in debug builds, but release behavior should favor controlled shutdown or subsystem isolation where possible.

## 29.3 Shutdown Sequence

```text
Stop accepting new clients
    ↓
Notify connected clients
    ↓
Stop generating new asynchronous work
    ↓
Finish current simulation tick
    ↓
Capture dirty snapshots
    ↓
Flush persistence journal and indexes
    ↓
Stop worker pools
    ↓
Close transports
```

---

# 30. Future Modding and Scripting

## 30.1 Do Not Stabilize a Script ABI Too Early

First stabilize:

- registries
- block behavior contexts
- world mutation commands
- entity component access policy
- event model
- persistence ownership
- security boundaries

## 30.2 Future Script Boundary

Scripts should operate through handles and host calls, not Rust references.

```rust
pub trait ScriptHost {
    fn get_block(&self, dimension: DimensionId, pos: BlockPos) -> BlockStateId;
    fn enqueue_world_command(&mut self, command: WorldCommand);
    fn query_entities(&self, query: ScriptEntityQuery) -> ScriptEntityList;
    fn emit_event(&mut self, event: ScriptGameEvent);
}
```

## 30.3 Script Execution Policy

Define:

- instruction budget
- memory budget
- deterministic random API
- allowed host calls
- server-only and client-only modules
- failure handling
- save schema
- versioning

A future Vela or WebAssembly runtime can implement this boundary.

## 30.4 Data Packs Before Scripts

Before arbitrary scripting, support data-driven:

- block definitions
- item definitions
- recipes
- loot tables
- tags
- biome parameters
- dimensions
- simple state machines

This covers much customization with less risk.

---

# 31. Security Model

## 31.1 Trust Boundaries

Untrusted:

- clients
- network frames
- chat text
- names
- imported world data
- scripts
- data packs from external sources

Trusted but fallible:

- built-in game data
- server operator commands
- persistence layer
- internal worker results

## 31.2 Server Validation

The server validates:

- movement limits
- interaction range
- line of sight where required
- block placement legality
- item ownership
- inventory transaction sequence
- attack cooldowns
- permissions
- chunk availability
- message rate

## 31.3 Resource Limits

Per connection:

- maximum packet size
- maximum messages per tick
- chat rate
- inventory action rate
- chunk request behavior
- decode time
- outbound queue bytes

Per world:

- update budgets
- entity limits
- scheduled work limits
- save queue limits
- script budgets

---

# 32. Performance Budgets

Budgets are targets, not guaranteed numbers. Measure on representative hardware.

## 32.1 Server Tick

For a 20 Hz simulation:

```text
Total tick budget: 50 ms
Preferred average: below 20 ms
Preferred p99: below 45 ms
```

Suggested phase targets for an early dedicated server:

```text
Ingress and validation        2 ms
Block systems                 8 ms
Entities and physics         10 ms
Commit and updates            8 ms
Replication                   5 ms
Headroom                     17 ms
```

## 32.2 Client Frame

For 60 FPS:

```text
Frame budget: 16.67 ms
```

Chunk CPU meshing must not execute on the render thread. GPU uploads are budgeted.

## 32.3 Chunk Memory

Track a realistic loaded-section memory model:

```text
block palette
biome palette
light arrays
heightmaps
block entities
scheduler data
temporary snapshots
mesh CPU data
GPU data
```

Do not base view distance only on raw block storage.

## 32.4 Profiling Rules

Optimize only after identifying:

- hot function
- allocation source
- cache behavior
- queue growth
- synchronization wait
- serialization cost
- GPU bottleneck

Retain simple scalar reference implementations for complex optimized algorithms when practical.

---

# 33. Implementation Roadmap

## Phase 0: Foundation

Deliver:

- workspace and CI
- coordinate types
- resource identifiers
- registries
- basic block states
- chunk/section storage
- palette container
- in-memory world
- state hash
- testkit

Exit criteria:

- create, mutate, snapshot, and hash a small world
- palette property tests pass
- no Bevy rendering dependency below `client-bevy`

## Phase 1: Headless Simulation Core

Deliver:

- standalone `bevy_ecs` world
- fixed tick runner
- explicit schedules
- player entity
- block placement/destruction
- immediate neighbor updates
- scheduled ticks
- command and effect output
- scenario runner

Exit criteria:

- headless tests can run 10,000 ticks
- deterministic replay produces the same state hash
- simple falling-block behavior works

## Phase 2: Minimal Bevy Client

Deliver:

- local transport
- embedded server
- client-runtime cache
- camera and input
- naive chunk meshing
- block targeting
- place/break interaction
- basic UI

Exit criteria:

- client never mutates authoritative world directly
- server remains playable without renderer
- client can be restarted and rebuild presentation from snapshots

## Phase 3: World Streaming and Persistence

Deliver:

- chunk tickets
- async generation
- async meshing
- region storage
- journal
- save/load
- revision-safe async results
- world inspector

Exit criteria:

- travel continuously without unbounded queues
- crash test does not corrupt unrelated chunks
- loaded world survives repeated save/load cycles

## Phase 4: Survival Vertical Slice

Deliver:

- inventory
- item drops
- crafting
- health and damage
- tools and block hardness
- furnace
- day/night
- basic mobs
- small set of biomes

Exit criteria:

- complete gather/craft/survive loop
- dedicated server supports at least several clients
- inventory actions are server validated

## Phase 5: Advanced Simulation

Deliver:

- light propagation
- fluids
- redstone foundation
- pistons
- more entity physics
- portals and dimensions
- structures

Exit criteria:

- subsystem-specific scenario suites
- update queues remain bounded under stress
- replay captures advanced mechanics

## Phase 6: Scale and Tooling

Deliver:

- improved meshing
- render origin rebasing
- client prediction
- network delta compression
- profiling dashboards
- behavior comparison tools
- data pack support

---

# 34. Architecture Decision Records

Create ADRs for decisions that affect long-term structure.

Initial ADR set:

```text
ADR-0001 Use bevy_ecs in the simulation core
ADR-0002 Keep voxel blocks outside ECS
ADR-0003 Server-authoritative architecture
ADR-0004 Single-player uses an embedded server
ADR-0005 Stable entity IDs are independent of Bevy Entity
ADR-0006 Fixed explicit simulation phase pipeline
ADR-0007 Immediate versus deferred world mutation
ADR-0008 Custom persistence format with journaling
ADR-0009 Semantic protocol independent of transport
ADR-0010 Engine-independent client runtime
ADR-0011 Runtime registries freeze after initialization
ADR-0012 Deterministic command merge policy
ADR-0013 Chunk ticket lifecycle
ADR-0014 Named random streams
ADR-0015 Behavior specification and replay format
```

ADR template:

```markdown
# ADR-NNNN: Title

## Status
Proposed | Accepted | Superseded

## Context

## Decision

## Consequences

## Alternatives Considered

## Migration or Reversal Plan
```

---

# 35. Known Risks

## 35.1 Scope Explosion

Minecraft-like gameplay contains many interacting systems. The project can remain permanently in infrastructure work.

Mitigation:

- vertical slices
- fixed feature milestones
- behavior fidelity classifications
- explicit non-goals
- playable builds throughout development

## 35.2 Over-Abstraction

Planning for replaceable engines, transports, mod runtimes, and storage backends can create excessive traits.

Mitigation:

- abstract only at actual process or data boundaries
- prefer concrete internal types
- use messages and snapshots between layers
- add traits when a second implementation exists or is imminent

## 35.3 Accidental Nondeterminism

Parallel ECS systems and unordered containers can change behavior.

Mitigation:

- explicit phase schedules
- stable sort keys
- canonical hashes
- deterministic test mode
- controlled command merge
- no gameplay dependence on unordered iteration

## 35.4 ECS Misuse for Voxel Data

Putting blocks into ECS would create severe memory and scheduling overhead.

Mitigation:

- enforce world storage APIs
- prohibit block entities for ordinary blocks
- review all new ECS component categories

## 35.5 Long Tick Stalls

Redstone, fluids, generation, or entity storms can exceed tick budget.

Mitigation:

- bounded queues
- work continuation
- circuit diagnostics
- profiling
- degradation of optional work
- operator controls

## 35.6 Save Corruption

Asynchronous saving can clear dirty flags incorrectly or write inconsistent data.

Mitigation:

- immutable save snapshots
- revision checks
- write-ahead journal
- checksums
- append-and-repoint records
- recovery tests

## 35.7 Bevy API Churn

`bevy_ecs` APIs may change.

Mitigation:

- pin versions
- wrap schedule construction in one module
- isolate Bevy ECS-specific helpers
- use stable project-owned domain types
- perform upgrades at milestone boundaries
- accept rewrites in the experimental project

## 35.8 Frontend and Core State Leakage

Convenience may cause gameplay code to depend on render state.

Mitigation:

- crate dependency checks
- no full Bevy dependency in core crates
- frontend communicates through client-runtime events
- headless CI tests

---

# 36. Initial Definition of Done

The architecture baseline is proven when all of the following are true:

- The simulation runs using standalone `bevy_ecs`.
- A dedicated headless executable starts without graphics dependencies.
- The same simulation runs behind an embedded local server.
- The Bevy client connects only through the client-runtime/transport boundary.
- Blocks are stored in palette-compressed chunk sections.
- Dynamic players and entities live in Bevy ECS.
- Bevy `Entity` values never appear in protocol or persistence data.
- The fixed tick pipeline is explicitly defined in code.
- Block placement, breaking, scheduled ticks, and neighbor updates are tested.
- A state hash is stable across repeated deterministic runs.
- Chunk generation and meshing are asynchronous and revision checked.
- Save snapshots are asynchronous and revision checked.
- A basic region/journal save can recover after simulated interruption.
- The Bevy client renders chunk meshes rather than one entity per block.
- A test scenario can run without launching a client.
- A replay can reproduce a small scenario.
- Metrics expose tick phase duration and queue sizes.

---

# 37. Reference Dependencies

The exact list should remain small and justified.

## Core candidates

```toml
[dependencies]
bevy_ecs = "0.19"
glam = { version = "...", features = ["serde"] }
serde = { version = "...", features = ["derive"] }
thiserror = "..."
bitflags = "..."
smallvec = "..."
smol_str = "..."
slotmap = "..."
bytes = "..."
tracing = "..."
```

## Storage and compression candidates

```toml
crc32fast = "..."
xxhash-rust = "..."
zstd = "..."
lz4_flex = "..."
```

Choose compression through benchmarks. Do not add all algorithms permanently.

## Networking candidates

The semantic protocol must remain transport-independent. A runtime may evaluate:

- QUIC implementations
- a lightweight UDP reliability layer
- TCP during early development

Do not allow networking library types into gameplay crates.

## Client

The frontend may depend on the full Bevy feature set required by rendering, audio, input, and UI. Keep it in `client-bevy`.

---

# 38. References

The architecture targets Bevy 0.19 at the time of writing, while keeping project-owned interfaces independent from Bevy rendering APIs.

- Minecraft Java Edition 26.2 behavioral reference: [README](reference/minecraft-java-26.2/README.md)

- Minecraft Java Edition 26.2 evidence and source lock: [sources](reference/minecraft-java-26.2/sources.md)

- [Bevy 0.19 release notes](https://bevy.org/news/bevy-0-19/)

- [`bevy_ecs` crate documentation](https://docs.rs/bevy_ecs/0.19.0/bevy_ecs/)

- [`bevy_ecs::World`](https://docs.rs/bevy_ecs/0.19.0/bevy_ecs/world/struct.World.html)

- [`bevy_ecs::Schedule`](https://docs.rs/bevy_ecs/0.19.0/bevy_ecs/schedule/struct.Schedule.html)

- [`bevy_ecs::Commands`](https://docs.rs/bevy_ecs/0.19.0/bevy_ecs/system/struct.Commands.html)

- [Bevy ECS quick start](https://bevy.org/learn/quick-start/getting-started/ecs/)

---

## Final Architectural Rule

The central rule of the project is:

> The server simulation owns gameplay truth. Specialized voxel storage owns blocks. Bevy ECS owns dynamic simulation entities. The client runtime owns replicated presentation state. The Bevy frontend only renders and collects input.

If this boundary remains intact, the project can evolve from a local experiment into a dedicated-server voxel sandbox without rewriting its gameplay core.
