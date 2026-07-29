# G01-P4-F004 Play Serverbound Block Report

## Result

Ferrite implements and verifies the five required packets in
`PROTO-PLAY-SERVERBOUND-BLOCK-001`. The existing codecs and Phase-4 Region integration are now
joined by an explicit dispatcher that locks the loaded gate and path-specific prediction
registration order without importing item or block mechanics into the protocol crate.

## Verified boundaries

- IDs 36, 41, 63, 66, and 67 have exact default goldens, signed packed-position endpoints,
  sequence endpoints, both hands, all eight actions, and exceptional float-bit round trips.
- All 256 player-action direction bytes use modulo six; strict hands, block-hit directions, and
  action ordinals reject out-of-range values.
- Truncation at every use-on byte, trailing bytes, invalid VarInts, and malformed enums fail closed;
  ordinary booleans accept every nonzero byte.
- Destroy, use-on, and use-in-air drop before registration while client loading is closed.
- Destroy registers after its authoritative handler returns. Use-on and use-in-air register before
  handler work. Handler and negative-sequence faults retain that observable order.
- Pick, swing, and auxiliary actions ignore the loaded gate. Auxiliary sequences never register.
- Existing gameplay and runtime tests retain strict reach/hit validation, Region-only mutation,
  pending-teleport behavior, correction order, and committed convergence.

## Evidence

- `crates/ferrite-protocol/tests/c2/play_serverbound_block.rs`
- `crates/ferrite-gameplay/src/block/targeting.rs`
- `crates/ferrite-server-runtime/tests/block_interaction.rs`
- `docs/development/protocol-play-serverbound-block.md`

The batch acceptance gate is `cargo ferrite task check` followed by `git diff --check`.
