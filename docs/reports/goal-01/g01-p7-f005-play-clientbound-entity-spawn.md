# G01-P7-F005 Play Clientbound Entity Spawn Report

## Result

Ferrite implements and verifies both packets in
`PROTO-PLAY-CLIENTBOUND-ENTITY-SPAWN-001`. Entity construction, type-specific spawn data,
same-ID replacement, ordered removal and pairing are explicit version-local projections.

## Verified boundaries

- Two goldens lock add and remove packet IDs and their primitive field order. A complete sweep
  round-trips all 158 static entity-type IDs and locks the pig fallback outside the registry.
- Signed entity/data values, UUID, raw IEEE positions, compact movement and rotation bytes are
  covered. Unknown encoding identities, impossible/truncated counts and trailing data fail closed;
  negative remove counts and duplicate/signed IDs preserve the client behavior.
- Construction tests cover player-info and factory admission, living coordinate/pitch clamping,
  nonliving raw placement, head/body initialization and seen-player history.
- Spawn-data tests cover hanging directions and anchors, vertical painting faults, falling-block
  air fallback, emerging Wardens, projectile owner lookup and discarded ownerless fishing bobbers.
- Replacement tests lock recreation before same-ID discard and duplicate-UUID insertion without
  lookup registration. Specialized tests cover dragon parts, Shulker rotation, llama-spit
  multipliers and minecart/bee sound state.
- Removal tests lock packet order, relationship mutation, missing/duplicate IDs, indirect local
  player carriage, former-vehicle clearing and debug teardown.
- Pairing tests lock self/audience/chunk/distance gates, passenger-expanded tracking range, bundle
  order and stop-seen-before-remove unpairing.

## Evidence

- `crates/ferrite-protocol/tests/c3/play_clientbound_entity_spawn.rs`
- `docs/development/protocol-play-clientbound-entity-spawn.md`

The batch acceptance gate is `cargo ferrite task check` followed by `git diff --check`.
