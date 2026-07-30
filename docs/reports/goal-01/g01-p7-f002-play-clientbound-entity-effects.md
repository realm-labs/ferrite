# G01-P7-F002 Play Clientbound Entity Effects Report

## Result

Ferrite implements and verifies IDs 36, 78, and 132 in
`PROTO-PLAY-CLIENTBOUND-ENTITY-EFFECTS-001`. The Java 26.2 adapter owns raw particle, holder, flags,
blend and tracker details while authoritative explosions and effect state remain in gameplay
owners.

## Verified boundaries

- Three goldens lock fixed/VarInt integer distinctions, field order, optional knockback, direct
  sound holders and the two effect packets.
- The exact 125-entry particle table and every option-bearing codec shape round-trip, including
  block states, geysers, colors, dust, item templates, vibration sources, trails and shrieks.
- Registered/direct sounds, signed entity/amplifier/duration domains and IEEE values round-trip;
  unknown holders/types, option mismatches, invalid block states, malformed frames and trailing
  bytes fail closed.
- Weighted recipes reject negative weights and signed-total overflow. Tracker queues require
  positive recipe weight, clear at the next tick, honor `ALL`, cap attempts at 512 and fault on
  signed block-count overflow.
- Effect handling enforces current living/can-accept gates, amplifier clamp, exact infinite duration,
  low flag bits, replacement blend copying, removal and silent missing-target paths.
- Explosion selection locks the strict 64-block audience, small/large particle branch, sound math,
  primary velocity, tracker-before-knockback presentation and additive player motion.
- Publication plans lock direct-passenger/self audience, add/update/removal attribute ordering,
  periodic refresh, initial replay, and mount/dismount packet order.

## Evidence

- `crates/ferrite-protocol/tests/c3/play_clientbound_entity_effects.rs`
- `docs/development/protocol-play-clientbound-entity-effects.md`

The batch acceptance gate is `cargo ferrite task check` followed by `git diff --check`.
