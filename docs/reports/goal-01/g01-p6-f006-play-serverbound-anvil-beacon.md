# G01-P6-F006 Play Serverbound Anvil and Beacon Report

## Result

Ferrite implements and verifies IDs 48 and 52 in
`PROTO-PLAY-SERVERBOUND-ANVIL-BEACON-001`. Both are normalized against the handler-time current menu
without inventing container, state, sequence or acknowledgement fields.

## Verified boundaries

- Rename locks default UTF bounds and replacement decoding; beacon locks two independent optional
  strict holders in the configured 40-effect registry.
- The connection driver uses its configured Play registry snapshot for framed ingress, while
  compatibility codec entry points fail explicitly if beacon decoding lacks that context.
- Client rename edits enforce the 50-unit edit-box boundary, normalize an unchanged default hover
  name, predict filtered result presentation before send and suppress equivalent edits.
- Server rename filtering precedes the semantic bound and equality test, reproduces Java blank
  custom-name removal, and records one full recomputation plus ordinary broadcast per change.
- Beacon client selection respects tier activation, clearing/upgrading behavior and emits selection
  before ordinary close without predicting authoritative mutation.
- Server beacon admission distinguishes wrong/invalid menu, missing payment, tier/equality refusal,
  forged absent choices and the locked absent-primary null-equality fault.
- Success orders primary then secondary built-in-plus-one data writes, filters block-entity choices,
  conditionally records sound, consumes exactly one payment and marks the chunk unsaved.
- Eight named C3 vectors pass; the combined C3 suite is 52 tests.

## Evidence

- `crates/ferrite-protocol/tests/c3/play_serverbound_anvil_beacon.rs`
- `docs/development/protocol-play-serverbound-anvil-beacon.md`

The batch acceptance gate is `cargo ferrite task check` followed by `git diff --check`.
