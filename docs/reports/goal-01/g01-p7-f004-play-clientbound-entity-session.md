# G01-P7-F004 Play Clientbound Entity Session Report

## Result

Ferrite implements and verifies all six packets in
`PROTO-PLAY-CLIENTBOUND-ENTITY-SESSION-001`. Feedback, camera, pickup and respawn are explicit
version-local projections of already-authoritative outcomes.

## Verified boundaries

- Six goldens lock packet IDs, unsigned animation action, biased wrapping cause/direct IDs,
  optional source position, hurt yaw, common respawn record, camera ID and signed pickup amount.
- Signed entity/amount extremes and raw IEEE values round-trip. Unknown registry holders,
  malformed/truncated packets, noncanonical booleans and trailing data follow the strict boundary.
- Animation tests cover all five recognized actions, missing/unknown ignores and the living/player
  cast faults. Hurt yaw applies to any present runtime type.
- Damage tests cover missing/nonliving targets, positional-source precedence, independent
  cause/direct lookup, living timers and current-game-time recording.
- Camera tests lock present-only replacement. Pickup tests lock collector-first casting, local
  fallback, item wrapping subtraction/removal, retained experience orbs and unconditional other
  source removal.
- Respawn tests cross independent entity-data and attribute bits, ignored high bits, same/new
  dimension replacement, player/stat/recipe retention, reset values, level/debug lifecycle,
  camera/container/load state and duplicate application.
- Publication tests lock animation audience, full/unblocked damage and hurt gates, relocation-before-
  camera order, tracking-only pickup and both respawn sequences.

## Evidence

- `crates/ferrite-protocol/tests/c3/play_clientbound_entity_session.rs`
- `docs/development/protocol-play-clientbound-entity-session.md`

The batch acceptance gate is `cargo ferrite task check` followed by `git diff --check`.
