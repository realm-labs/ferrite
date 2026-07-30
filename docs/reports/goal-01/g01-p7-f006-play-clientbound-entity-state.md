# G01-P7-F006 Play Clientbound Entity State Report

## Result

Ferrite implements and verifies all five packets in
`PROTO-PLAY-CLIENTBOUND-ENTITY-STATE-001`. Mutable metadata, attributes, equipment, passengers and
leash state are ordered projections over current-level entities.

## Verified boundaries

- Five goldens lock IDs and the distinct VarInt/fixed-int/list/terminator forms.
- All 43 metadata serializers round-trip. By-ID fallback policies, nested item/particle/component/
  holder/profile forms, missing terminators, unknown serializers and trailing data are covered.
- The committed 221-row accessor lock has the exact audited SHA-1 and is build-validated before
  generating Rust declarations. Tests compose the Entity/Living/Mob/Ageable hierarchy and reject
  unrelated slot collisions.
- Metadata tests lock missing-target ignore, ordered duplicate application, per-accessor and
  aggregate callback order, partial fault state, nondefault pairing and default-return dirty sends.
- Attribute tests cover registry/IEEE/operation boundaries, missing instances, nonliving faults,
  base sanitization, complete modifier replacement, duplicate identity faults and dirty draining.
- Equipment tests cover required entries, all ordinals, exact count/component patch retention, air
  normalization, repeated-slot replacement, nonliving ignore, ordinal pairing and hand-swap event
  suppression.
- Passenger tests cover bounded arrays, complete ejection, sequential missing/duplicate/cycle
  behavior, old-vehicle detach, former-marker clearing, boat rotation and once-only onboarding.
- Leash tests cover fixed signed IDs, wrong-source ignore, zero detach and lazy nonzero resolution.
  Publication tests lock metadata/attribute/equipment audiences and pairing/rider/leash order.

## Evidence

- `crates/ferrite-protocol/reference/minecraft-java-26.2-entity-metadata-accessors.tsv`
- `crates/ferrite-protocol/tests/c3/play_clientbound_entity_state.rs`
- `docs/development/protocol-play-clientbound-entity-state.md`

The batch acceptance gate is `cargo ferrite task check` followed by `git diff --check`.
