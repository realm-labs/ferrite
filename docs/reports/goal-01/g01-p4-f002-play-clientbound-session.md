# G01-P4-F002 Play Clientbound Session Report

## Result

Ferrite implements and verifies the five required packets in
`PROTO-PLAY-CLIENTBOUND-SESSION-001`. Clientbound ID 61 and its ID-45 pong were added; the existing
session projection was extended from a riding boolean to an explicit bounded liveness queue and
locally authoritative root-vehicle state.

## Verified boundaries

- IDs 32, 44, 57, 61, and 73 have exact zero-body goldens and signed endpoint round trips.
- Disconnect rejects malformed component roots and accepts compound NBT beyond the default
  2,097,152-byte quota through the trusted codec.
- Keepalive emits ID 28, ping emits ID 45, and their signed payload domains remain independent.
- Frozen keepalive echoes send on unfreeze, remain pending before 60,000 ms, and drop when still
  frozen at the exact expiry boundary; the queue and deadline arithmetic fail closed.
- Relative yaw/pitch apply independently, pitch clamps, old-render rotation synchronizes, and ID 32
  carries both movement flags false.
- Vehicle correction checks root identity and local authority, compares the interpolation target,
  uses the exact `1e-5f` Euclidean threshold, ignores rotation-only changes, cancels only on snap,
  and always echoes qualifying resulting state through ID 34.
- NaN and infinity retain their distinct source-observed vehicle and rotation paths.

## Evidence

- `crates/ferrite-protocol/tests/c2/play_clientbound_session.rs`
- `docs/development/protocol-play-clientbound-session.md`

The batch acceptance gate is `cargo ferrite task check` followed by `git diff --check`.
