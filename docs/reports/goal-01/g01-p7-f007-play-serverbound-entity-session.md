# G01-P7-F007 Play Serverbound Entity Session Report

## Result

Ferrite implements and verifies all six packets in
`PROTO-PLAY-SERVERBOUND-ENTITY-SESSION-001`. Entity interaction, selection, spectator relocation,
respawn, statistics, and game-rule requests now enter through a strict 26.2 adapter boundary.

## Verified boundaries

- Six goldens lock IDs and exact field order.
- Signed entity/action/hand/optional identifiers, all boolean bytes, UUID endpoints, wrapping
  optional bias, invalid client commands, malformed/truncated fields, and trailing data are
  covered.
- Canonical and noncanonical `LpVec3` forms cover zero/near-zero/NaN/infinite input, scale
  continuation, saturated fields, zero scale, finite decode, truncation, and overlong VarInts.
- Attack tests lock loaded/spectator admission, idle-reset point, inclusive default/custom reach,
  creative and mob-factor endpoints, piercing order, feature/charge gates, and invalid-target
  disconnect.
- Interact tests lock early idle/shift mutation, strict reach, hand fallback, target-before-item
  precedence, spectator menus, infinite-material restoration, event/criterion stack selection, and
  self-inclusive server swing.
- Pick tests lock its absent loaded/border/idle gates, strict range/removal, exact inventory
  selection, enabled-result convergence, and independent authorized Avatar profile output.
- Camera tests lock absent/present gates, strict reach and relocation-before-publication.
  UUID teleport covers camera reset and same/cross-level ordering with keep mask `3`.
- Client-command tests cover alive/dead/win/hardcore respawn, renewed load grace, repeated
  dirty/empty statistics, permission-gated complete game rules, and ordered respawn projection.
- The aggregate path executes interact, attack, camera, and UUID teleport through the root packet
  dispatcher without promoting raw entity numbers or UUID lookup forms into domain state.

## Evidence

- `crates/ferrite-protocol/tests/c3/play_serverbound_entity_session.rs`
- `docs/development/protocol-play-serverbound-entity-session.md`

The batch acceptance gate is `cargo ferrite task check` followed by `git diff --check`.
