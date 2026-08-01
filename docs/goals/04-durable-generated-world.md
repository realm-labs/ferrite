# Goal 04 — Durable Generated World

## 1. Objective

Replace the formal server's hard-coded `MinimalTerrain` and flat collision plane with a configurable,
generated, loadable, durable Minecraft 26.2 world. Chunk lifecycle, authoritative voxel state,
collision, lighting, environment, dimensions, portals, and client projection must share one
Region-owned production path established by Goal 03.

The result is a world that survives restart and can be explored through ordinary client movement;
it is not merely a precomputed terrain projection or an isolated world-generation equivalence test.

## 2. Production evidence contract

Every world feature must prove the applicable chain:

```text
world configuration / player movement / world event
  -> Region-owned ticket or command
  -> load or deterministic generation
  -> authoritative chunk mutation
  -> durable commit and restart reconstruction
  -> collision/environment/client projection
  -> exact 26.2 MCP observation and screenshot
```

Generated output that is never installed into production chunks, a chunk that cannot be saved and
reloaded, or a client packet assembled from a separate terrain provider does not satisfy the Goal.

## 3. Scope boundary

### In scope

- a versioned Minecraft/world configuration schema for world identity, seed, generator, spawn,
  view distance, simulation distance, dimension set, time, weather, border, and save policy;
- fail-closed migration from server configuration schema 1;
- a Region-owned chunk ticket, load, generation, activation, save, unload, and recovery lifecycle;
- deterministic overworld terrain, biome, feature, structure, and spawn preparation pipelines based
  on the audited Goal 01 world behavior;
- production heightmaps, lighting state, block/fluid state, and client chunk projection from the same
  authoritative chunk data;
- voxel/block-state collision shapes for player movement, stepping, falling, suffocation, fluids,
  and world bounds required by ordinary exploration;
- dimension registry and authoritative travel between enabled overworld, nether, and end worlds;
- portal discovery, creation, cooldown, coordinate scaling, safe placement, and client dimension
  transition;
- day time, weather, world border, scheduled environmental work, and restart continuity;
- bounded generation work, load shedding, corruption handling, and offline inspection;
- exact-client exploration, restart, collision, dimension, and visual scenarios.

### Out of scope

- full player inventory, crafting, survival damage, hunger, and progression owned by Goal 05;
- full entity, mob, vehicle, and multiplayer tracking owned by Goal 06;
- production multi-process ownership and remote generation workers owned by Goal 07;
- byte-for-byte same-seed identity with Mojang where Goal 01 records an explicit inconclusive or
  statistical equivalence boundary;
- importing or redistributing Mojang world saves or generated assets.

## 4. World ownership rules

- `MinimalTerrain` may remain only as a focused fixture; it is not the formal production provider.
- The same authoritative chunk column supplies simulation, collision, persistence, and projection.
- A generated chunk is invisible until its generation result is fenced, validated, and committed.
- Unload cannot discard dirty state, in-flight generation, active tickets, or cross-Region work.
- Recovery validates world identity, dimension, mapping, content manifest, generator version, and
  chunk format before activation.
- Deterministic generation uses named random streams and records generator/content versions.
- Unsupported or corrupt world data fails closed with actionable inspection diagnostics.

## 5. Phased batches

### Phase 0 — Freeze world production truth

| Batch | Outcome |
|---|---|
| `G04-P0-B1` | Commit the world scope, ownership model, versioned formats, configuration migration, production-manifest rows, and acceptance denominator. |

### Phase 1 — Configuration and durable world bootstrap

| Batch | Outcome |
|---|---|
| `G04-P1-B1` | Add versioned world configuration and schema-1 migration with deterministic validation and canonical examples. |
| `G04-P1-B2` | Replace hard-coded world ID/dimension/spawn bootstrap with configured world creation and durable metadata load. |
| `G04-P1-B3` | Integrate chunk/level continuity, recovery selection, autosave, shutdown flush, and offline inspection into the formal server. |

### Phase 2 — Production chunk lifecycle and generation

| Batch | Outcome |
|---|---|
| `G04-P2-B1` | Install bounded player/simulation tickets, asynchronous generation requests, fenced results, activation, save, and unload. |
| `G04-P2-B2` | Integrate deterministic biome, density, terrain, surface, carver, feature, and spawn preparation stages. |
| `G04-P2-B3` | Integrate structure placement/start/reference state and versioned generation continuation. |
| `G04-P2-B4` | Project chunks, heightmaps, biomes, block entities, lighting, and unloads from committed authoritative columns. |

### Phase 3 — Collision and environment

| Batch | Outcome |
|---|---|
| `G04-P3-B1` | Replace `FlatWorldCollision` with block-state shape queries, stepping, falling, world bounds, and correction convergence. |
| `G04-P3-B2` | Integrate fluids, scheduled/random ticks, lighting propagation, fire, weather, and day-time projection. |
| `G04-P3-B3` | Add world border enforcement, spawn selection, respawn placement, and exploration-driven ticket movement. |

### Phase 4 — Dimensions and portals

| Batch | Outcome |
|---|---|
| `G04-P4-B1` | Activate configured overworld, nether, and end dimension runtimes with independent durable state. |
| `G04-P4-B2` | Integrate portal discovery/creation, scaling, cooldown, safe exit placement, Region transfer, and client transition. |
| `G04-P4-B3` | Prove restart and fault continuity across dimensions, portals, generation, and dirty-chunk saves. |

### Phase 5 — Real-client acceptance and completion

| Batch | Outcome |
|---|---|
| `G04-P5-B1` | Add exact-client MCP exploration, nonflat terrain, collision, weather/time, portal, restart, and screenshot scenarios. |
| `G04-P5-B2` | Complete production-manifest, format, migration, performance, source, and clean-checkout audits and publish the Goal 04 completion record. |

## 6. Required verification

Every Rust batch runs focused affected-crate tests plus:

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
git diff --check
```

Durability batches require restart, torn write, corrupt record, stale generation, save/unload race,
and format migration tests. Generation batches require deterministic replay and canonical hash
evidence. Player-visible batches require Goal 02 MCP state assertions and framebuffer screenshots.

## 7. Terminal acceptance

- [ ] Formal server world identity, seed, generator, spawn, distances, dimensions, and save policy are configured and versioned.
- [ ] Formal startup loads or creates durable world metadata instead of hard-coded world ID 1.
- [ ] Production chunks move through bounded ticket, load/generate, activate, save, unload, and recovery states.
- [ ] Terrain projection, simulation, collision, and persistence use the same authoritative chunk state.
- [ ] `MinimalTerrain` and `FlatWorldCollision` are absent from the formal production entry.
- [ ] Nonflat generated terrain, biomes, structures, heightmaps, lighting, fluids, weather, and time are observable in the exact client.
- [ ] Overworld, nether, and end state are durable and enabled according to configuration.
- [ ] Portal travel is authoritative, fenced across Regions, restart-safe, and visually convergent.
- [ ] Corrupt, mixed-version, stale-generation, and overloaded world work fails closed.
- [ ] Exact-client exploration and restart scenarios plus universal gates pass from a clean worktree.

Goal 04 is complete only when the formal server owns a durable generated world rather than a
separate flat projection fixture.
