# G01-P6-F007 Play Serverbound Container Convergence Report

## Result

Ferrite implements and verifies IDs 17, 18, 19, 20 and 53 in
`PROTO-PLAY-SERVERBOUND-CONTAINER-CONVERGENCE-001`. The adapter owns prediction evidence and remote
snapshots while authoritative menu mutations stay with the existing gameplay executor.

## Verified boundaries

- Five goldens lock signed VarInts/shorts/bytes, zero-fallback input and exact hash-map field order.
- Strict item/component holders, 128 changed slots, two 256-component bounds, duplicate
  normalization, noncanonical booleans and malformed/residual inputs are covered.
- CRC32C typed-component hashes use a 256-entry bounded cache; item/count/component/removal shape
  must match and AIR produces the empty form.
- Client prediction executes before emission, compares component maps semantically and hashes only
  post-click differences plus cursor.
- Wrong container resets idle then ignores; spectator/dead forces full state; invalid menu/slot
  ignores; other negative shorts reach the gameplay executor.
- Stale state executes then emits content/cursor and all data. Matching state emits slot, cursor and
  data corrections in order while matching hashes suppress traffic.
- Button and Crafter controls retain different validity/idle/backing/slot gates.
- Delayed close ignores its ID and transfers shared remote state; valid carried selection resets
  idle and stops active main-hand use only on change.
- Nine named C3 vectors pass; the combined C3 suite is 61 tests.

## Evidence

- `crates/ferrite-protocol/tests/c3/play_serverbound_container_convergence.rs`
- `docs/development/protocol-play-serverbound-container-convergence.md`

The batch acceptance gate is `cargo ferrite task check` followed by `git diff --check`.
