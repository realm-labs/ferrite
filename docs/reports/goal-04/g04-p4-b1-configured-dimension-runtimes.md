# G04-P4-B1 — Configured dimension runtimes

## Result

The formal server now activates every configured Overworld, Nether, and End level as a real local
Region authority. A configured dimension is no longer only a metadata or lifecycle label: it owns a
dimension-scoped chunk lifecycle, deterministic generator, control Region, level clock/weather and
border state, and contained durable store.

## Production chain

- `WorldMetadata::dimensions` remains the ordered, versioned ingress and rejects unsupported,
  duplicate, or non-Overworld-first catalogs.
- Formal bootstrap includes every dimension control Region, restores its selected checkpoint and
  activation generation, and creates a separate bounded generation worker and ticket book.
- `FormalDimensionKind` fixes the build layout and sea level. The seeded generators produce
  Overworld, Nether, and End-specific authoritative columns through the ordinary fenced generation
  status pipeline.
- Router snapshot lookup and collision capture now include `DimensionId`; equal `(x,z)` chunk
  coordinates cannot resolve authority from another level.
- Each control Region writes exactly one `FWL2` record. Only the Overworld control Region also writes
  `world_v1`, and the existing Overworld-last checkpoint publication keeps the multi-level prefix
  atomic.
- Every enabled level ticks its own environment and border before the composite commit. Player
  tickets are routed to the lifecycle for the player's current dimension.
- Java Play login advertises the configured level set and uses the current dimension type and sea
  level. Terrain registry projection includes default Nether and End block states and biomes.

## Focused evidence

- `world_service::dimension` locks layouts and proves same-seed replay plus dimension-distinct
  Nether/End columns.
- `minecraft::world::formal_bootstrap_activates_every_configured_dimension_control_region` drives
  Nether and End origin chunks to projectable authority through their formal lifecycles.
- `formal_world_persistence::configured_dimensions_commit_independent_control_records_and_restart_together`
  verifies three separate control stores at one checkpoint and a clean joint restart.
- `minecraft::entry::login_advertises_every_enabled_level_and_current_dimension_semantics` verifies
  protocol-visible catalog, dimension type, and sea level projection.
- Focused `ferrite-server-runtime` tests and Clippy pass before the universal batch gates.

## Boundary

This batch activates and persists dimension runtimes. It does not claim portal discovery, portal
creation, cooldown, safe destination placement, cross-dimension player ownership transfer, or the
Java respawn transition; those form the atomic `G04-P4-B2` transaction. Exact-client visual
acceptance remains `G04-P5-B1`.
