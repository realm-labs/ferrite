# G04-P4-B3 — Dimensional portal continuity

## Result

Portal-created chunks and the transferred player now survive a published world checkpoint as one
cross-Region state. If the Overworld control Region is not published, recovery selects the prior
prefix for every dimension and Region even when destination stores contain valid newer commits.

## Continuity changes

- Player-service continuity advances from `F6P1` to `F6P2`. The current record includes a bounded,
  validated `PlayerSessionState` payload containing the authoritative pose and movement state.
  `F6P1` remains readable and restores only its historical inventory/progression fields.
- Reconciliation captures the current Region ECS player state on join, movement, and transfer. A
  restored formal world reconstructs ECS players from `F6P2` before any recovery catch-up tick, so
  synchronization cannot mistake durable players for disconnected sessions.
- Formal flush retains successful per-Region points and receipts after a later store fails. Retry
  skips those Regions, completes the remaining stores, publishes the control Region last, and
  returns the full receipt set without duplicate durable revisions.
- Recovery continues to use the control Region's checkpoint tick as the global selection boundary.
  Complete later points in other dimensions are inspectable but do not become partial authority.

## Focused evidence

- `cross_region_end_platform_and_player_transfer_survive_one_published_checkpoint` creates the
  audited 100-block End platform across End Region z = -1/0, transfers the player, flushes all
  Regions, restarts, and verifies every platform block plus target ownership.
- `unpublished_cross_region_portal_successor_rolls_back_to_the_control_checkpoint` truncates only
  the final control publication while retaining newer dimension points, then proves the player and
  platform both resolve from the earlier checkpoint.
- `partial_flush_resumes_without_recommitting_regions_that_are_already_durable` fails the control
  store after a non-control commit and proves retry does not append the successful Region twice.
- Player continuity tests lock `F6P2` round trip and `F6P1` read-only migration without a synthetic
  session pose.
- Focused server-runtime tests and Clippy pass before the universal workspace gates.

## Boundary

This closes the production `Continuity` stage for `world/portals`. Exact Java 26.2 exploration,
framebuffer evidence, visible restart convergence, and client-observed portal travel remain
`G04-P5-B1`.
