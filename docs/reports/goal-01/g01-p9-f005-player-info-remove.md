# G01-P9-F005 Player-Info Removal Report

## Result

Ferrite implements and verifies clientbound ID 69 in
`PROTO-PLAY-CLIENTBOUND-PLAYER-INFO-REMOVE-001`. Normalized profile/session presence remains the
projection boundary; UUID lists, player-info objects, social callbacks and listed-object order stay
adapter/client-local.

## Verified boundaries

- The official singleton zero-UUID body is locked exactly. Empty, duplicate, reordered and all-bit
  UUID lists round-trip without a semantic cap beyond frame feasibility; negative/impossible
  counts, truncation and residual bytes fail closed.
- Every wire UUID invokes the social removal callback in order, including unknown and duplicate
  values. A present info entry is removed once and only its exact listed object token is removed;
  missing repeats are otherwise silent.
- Removal drops online-name and chat-session lookup with the info object while preserving the
  persistent discovered-name map and hide/block/friend relationship state. Independently owned
  entity, chat-history, scoreboard/team and waypoint domains are not part of this mutation.
- Receive order has no generation or acknowledgement: a delayed remove deletes a newly
  reinitialized object under the same UUID, and a later player-info add recreates the projection.
  The integrated Play projection accepts live player-info updates after level installation and
  converges add/remove/re-add accordingly.
- Canonical departure publication saves and removes the player, clears server membership and
  presentation services, publishes ordinary tracker teardown, then sends one singleton ID-69
  packet to each remaining global player in list order. Dimension/range/tracking do not filter this
  audience, and in-session respawn replacement emits no ID 69.
- The family requires an installed Play level and introduces no response, correlation token or
  acknowledgement of entity teardown.

## Evidence

- `crates/ferrite-protocol/src/java_26_2/play/clientbound/player_info_remove/`
- `crates/ferrite-protocol/tests/c3/play_clientbound_player_info_remove.rs`

Focused validation:

```text
cargo test -p ferrite-protocol --test c3 play_clientbound_player_info_remove
7 passed; 0 failed
cargo test -p ferrite-protocol --test c3
219 passed; 0 failed
cargo test -p ferrite-protocol --test c1
68 passed; 0 failed
cargo clippy -p ferrite-protocol --all-targets --all-features -- -D warnings
```

The batch acceptance gate is `cargo ferrite task check` followed by `git diff --check`.
