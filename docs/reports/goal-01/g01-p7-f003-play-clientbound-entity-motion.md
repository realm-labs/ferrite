# G01-P7-F003 Play Clientbound Entity Motion Report

## Result

Ferrite implements and verifies all nine packets in
`PROTO-PLAY-CLIENTBOUND-ENTITY-MOTION-001`. Authoritative entity motion stays in simulation owners;
the Java 26.2 adapter owns quantized bases, masks, compact vectors, interpolation and tracker
publication choices.

## Verified boundaries

- Nine exact goldens lock packet IDs, fixed/VarInt/short distinctions, absolute records, rotation
  bytes, booleans, minecart lists, compact motion, relative masks and projectile power.
- Signed entity/short/rotation endpoints, high mask bits and raw IEEE absolute/minecart/projectile
  values round-trip; malformed lists, compact vectors, truncation and trailing data fail closed.
- `LpVec3` tests cover zero, continued scale, NaN/infinity sanitization, component clamping and
  accepted noncanonical zero-scale forms while retaining finite decode.
- Relative projection covers exact zero preservation, Java rounding/wrapping, base-before-authority
  ordering, immediate/default interpolation and identical-target suppression.
- Absolute sync covers strict snap distance, nonticking/local gates, ignored encoded velocity,
  ground state and rider repositioning.
- Teleport covers interpolation-target source, all low mask roles, ignored high bits, velocity
  rotation, pitch clamp, direct vehicle echo and persistent former-vehicle player fallback.
- Motion, living head interpolation, projectile runtime gates, old/new minecart behavior, raw
  weight activation and last-step fallback are covered.
- Tracker tests lock threshold/zero velocity, pose selection, passenger/minecart branches,
  motion-projectile-pose-dirty-head-hurt order and riding-player teleport forms.

## Evidence

- `crates/ferrite-protocol/tests/c3/play_clientbound_entity_motion.rs`
- `docs/development/protocol-play-clientbound-entity-motion.md`

The batch acceptance gate is `cargo ferrite task check` followed by `git diff --check`.
