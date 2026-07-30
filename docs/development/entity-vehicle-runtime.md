# ENT-002 Vehicle Runtime

`G01-P7-S002` implements the protocol-neutral `ENT-VEHICLES-001` transition layer. The owning
Region supplies authoritative observations and commits returned values or effects; this module
does not mutate world state, emit packets, or introduce a second entity owner.

## Responsibility split

`ferrite-gameplay::entity::runtime::ent_002` is divided by vehicle responsibility:

- `damage` owns common vehicle hurt admission, strict destruction thresholds, creative removal,
  live `entity_drops` itemization, custom-name copying, and per-tick hurt decay;
- `boat` owns water-status buoyancy and friction, input/paddles, underwater ejection, bubble-column
  expiry, collision auto-mounting, passenger yaw, and ordered dismount selection;
- `minecart` owns the feature-selected old and improved engines, all ten rail shapes, off-rail
  motion, slowdown, slopes, rail projection, powered/unpowered behavior, opposing-V stops,
  rotation, collision impulses, and ordered dismount selection;
- `subtypes` owns rideable activation, furnace fuel and propulsion, TNT priming/explosion inputs,
  hopper and command activation, and explicit hopper/spawner/command/container integration hooks.

The small vector and outcome types are local transition vocabulary. They intentionally do not
expose ECS, protocol, registry, or Lattice types.

## Source boundaries

Vehicle damage rejects removed or invulnerable vehicles and rejects mob explosions when
`mob_griefing` is disabled. An admitted hit flips the hurt direction, sets ten hurt ticks, and adds
`amount * 10`. Destruction remains strictly greater than 40 damage unless the source forces it.
Creative removal and forced destruction retain their distinct discard/destroy branches, while
itemization reads the live `entity_drops` value only after destruction.

Boat transitions preserve status-specific buoyancy/friction, the 60-tick underwater ejection
boundary, exact player/non-player bubble launch values, two-pass collision policy, two-seat mount
limit, passenger yaw clamp, animal seat orientation, and pose-first dismount search.

Minecart transitions retain the experiment feature flag instead of blending the old and improved
engines. The runtime preserves their different slowdown, slope, speed, conductor-push, substep,
rotation, collision, and opposing-slope behavior. Rail shape handling is closed over all ten
source shapes. Furnace carts retain collision priority; rideable carts retain the source's literal
double `startRiding` result behavior.

Subtype helpers expose ordered facts for later integration rather than performing inventory,
spawner, command-block, or explosion side effects. Random TNT fuse and explosion draws are explicit
inputs, preserving deterministic named-stream ownership at the Region boundary.

## Validation

`crates/ferrite-gameplay/tests/slices/entities/ent_002.rs` owns the source-specified vehicle slice.
Its eleven tests lock rejection and threshold boundaries, every boat status, paddle and bubble
values, mount/dismount order, both minecart engines, all rail shapes, powered and opposing-V
thresholds, collision priority, and each subtype hook. `G01-P7-B1` remains responsible for
installing these pure transitions into the authoritative Region tick and projection pipeline.
