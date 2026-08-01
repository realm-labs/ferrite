# G04-P3-B3 — Border, Spawn, and Exploration Authority

## Result

The formal overworld now has one authoritative path for safe player placement, world-border
presentation and enforcement, and exploration-driven chunk interest. The remaining visible gate is
the exact-client acceptance batch in P5.

## Spawn and respawn placement

- Generated spawn uses the configured seed and the audited bounded initial-spawn permutation. Each
  candidate is evaluated against a fully generated Ferrite column, solid support, two-block
  headroom, fluids, and player-width border containment.
- The selected world spawn is stored in world metadata. The former `(8,64,8)` value is accepted only
  as a read-only migration input when the configuration still declares generated spawn; it cannot
  mask a changed fixed spawn.
- Bootstrap installs bounded `ferrite:spawn_search` tickets for the complete ten-block placement
  area and does not finish formal-world construction until those columns are committed,
  authoritative, and projectable.
- Fixed world spawn and safe player placement are distinct. The client receives the configured
  world spawn, while admission and reconnect use a collision-free placement resolved from committed
  snapshots. Survival death and health transitions remain Goal 05; this batch owns the world-side
  placement resolver they consume.

## Border authority

- `FWL2` remains the durable border owner. Moving borders advance on formal server ticks and retain
  their current size, target, and remaining duration across restart.
- Java `InitializeBorder` fields are derived from the live border snapshot instead of constants.
- Authoritative movement collision composes committed voxel shapes with the same border, clipping
  the player's full horizontal bounds before an edge. Missing voxel authority still fails closed.

## Exploration tickets

- The Java login and chunk session use schema-2 view and simulation distances instead of gateway
  constants, including capacity calculations and login projection. The login now advertises the
  generated world as non-flat.
- Once a movement command commits, the chunk stream emits center/unload changes and atomically
  replaces player-view and player-simulation tickets. The formal lifecycle consumes that new set on
  the following tick, so abandoned chunks follow the existing save-receipt/unload fence.
- The formal local topology remains the bounded preactivated Region set centered on world spawn;
  remote or elastic Region placement remains Goal 07 rather than being synthesized by the world
  adapter.

## Verification

Focused coverage exercises seeded spawn selection, obstructed respawn placement, legacy metadata
migration, spawn readiness, configured distances, border projection, border collision, moving-border
restart continuity, ticket recentering, formal persistence restart, and the network listener.

The batch is closed only after the full workspace tests, universal Rust gates, source policy,
production manifest, and diff checks pass.
