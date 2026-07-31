# Minecraft 26.2 portal runtime

Ferrite's `WGEN-PORTAL-001` owner is `ferrite-world::generation::portal`. The module separates the
entity-side processor, Nether lookup and frame construction, End portal behavior, End gateway
state, and passenger-graph transfer. Callers supply level lookup, collision queries, block writes,
entity construction, packet projection, and Region ownership; the portal runtime returns ordered,
deterministic decisions and side effects.

## Processor and admission

`processor` preserves the optional portal entry, elapsed time, and per-tick contact mark. Contact
while cooling down refreshes the full cooldown without advancing portal time. Otherwise contact
creates, replaces, or updates the processor entry and marks it for the entity tick. The entity tick
decrements cooldown first, compares the preincrement wait value, applies cooldown before resolving
a ready transition, and decays an unmarked processor by four until removal.

The admission boundary keeps the base alive/passenger/sleeping checks separate from special entity
overrides and cross-key rules. It includes player and root-passenger wait/cooldown ownership,
Nether gamerule gating, destination capability, End credits, pearl-owner admission, and the
same-level gateway exception. Client confusion progression exposes its one random pitch draw,
screen-close side effect, and clamped rise/decay rates without consuming hidden randomness.

## Nether portals

`nether` selects the destination key before applying the source/destination coordinate scale,
world-border clamp, maximum-edge epsilon, and block floor. POI selection uses the
destination-specific inclusive square,
exact portal-axis state, border admission, 3D squared distance, minimum Y, and stable encounter
order. Rectangle discovery requires exact state identity and caps both axes at 21 blocks.

Frame creation scans columns in the locked east-then-south square spiral and retains the first
candidate at an equal squared distance. It distinguishes the three-plane preferred site from the
center-plane site and the height-clamped fallback. The fallback emits the official 24 support and
clearance writes with update flags `3`, followed by the 14 frame writes with flags `3` and six
portal writes with flags `18` in source order. Lookup plans explicitly require POI loading and
validation before consuming the inclusive square stream. Exit
geometry retains relative entry coordinates, handles oversized entities, limits collision
adjustment to eligible dimensions, rotates yaw by 90 degrees on an axis change, preserves motion
and pitch, and distinguishes existing-POI tickets from newly created exits.

## End portal and gateway

`end_portal` owns the no-collision contact slab, nonreplaceable/invisible block surface, two-draw
smoke particle, no-op block-entity persistence contract, horizontal world faces, all-face special
model, and exact 15-layer shader constants and transforms. End entry returns the ordered 5×5×4
platform replacement plan and entity target. Exit separates player respawn resolution from
nonplayer post-effects, while unseen credits bypass the ordinary processor and emit the win event
once.

`gateway` persists age, configured exit, and exactness while keeping cooldown transient. Its tick
order covers spawning age, 40-tick cooldown broadcasts, 2,400-tick attention, and failed-contact
cooldown. Configured exits apply the End-only surface scan and exact/nonexact rules. Unconfigured
exits expose the bounded radial chunk walk, End-stone anchor selection, reciprocal gateway
placement, fresh feature-random source, and linked writes. Gateway transitions stay in the same
level, ticket the final block, and zero pearl velocity and rotation.

## Passenger and Region boundary

`transfer` guards cycles while traversing the passenger graph. Same-level movement walks passengers
before the root and preserves relative poses. Cross-level movement ejects the source graph,
recursively constructs destination entities, restores state, removes source entities, adds new
entities, and remounts in stable order. A failed root construction deliberately leaves already
transferred passengers committed and the old root ejected, matching the locked partial-failure
surface. Server players retain their instance and synchronize destination state; spectator cameras
follow the transferred target.

The module does not directly mutate Region state. Its ordered write lists, tickets, and transfer
operations are the inputs to the Phase 8 durable-world integration batch, where ownership,
generation fencing, persistence, and bounded projection are applied atomically where the source
surface permits it.
