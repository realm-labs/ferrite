# G01-P7-F001 Play Clientbound Combat and Look Report

## Result

Ferrite implements and verifies IDs 66, 67, 68, and 71 in
`PROTO-PLAY-CLIENTBOUND-COMBAT-LOOK-001`. Combat notices and look commands remain tokenless
version-local projections; gameplay death and authoritative player rotation stay in their existing
owners.

## Verified boundaries

- Four goldens lock signed duration/entity VarInts, the empty combat-enter body, trusted component
  NBT, strict anchors, doubles, boolean and optional entity field order.
- Full signed and IEEE domains round-trip; invalid anchors, malformed components, truncation and
  residual bytes fail closed, while nonzero booleans canonicalize.
- Combat enter/end are inert. Combat kill requires the current local-player object and repeatedly
  installs death screens or emits respawn plus toggle-key reset according to login state.
- Coordinate look uses packet coordinates; entity look resolves current handler-time feet/eyes and
  falls back once when absent.
- Java angle constants, float narrowing, wrapping, current/previous head/body alignment and raw
  coincident/nonfinite paths are covered.
- Canonical publication preserves direct tokenless combat order, death empty/fallback/broadcast
  branches and coordinate/entity look forms.
- End-to-end encoding and decoding drives the same handler-time client projection without an
  acknowledgement.

## Evidence

- `crates/ferrite-protocol/tests/c3/play_clientbound_combat_look.rs`
- `docs/development/protocol-play-clientbound-combat-look.md`

The batch acceptance gate is `cargo ferrite task check` followed by `git diff --check`.
