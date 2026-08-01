# G01-P9-F006 Player Projection Report

## Result

Ferrite implements and verifies clientbound IDs 3, 22, 103, and 104 in
`PROTO-PLAY-CLIENTBOUND-PLAYER-PROJECTION-001`. Typed stats, cooldown groups, experience, and
vitals are the normalized connection-local boundary; registry raw IDs, dirty/sent markers,
cooldown intervals, hurt/display timers, and screen callbacks remain adapter-local.

## Verified boundaries

- All four official packet bodies are locked exactly. Signed VarInts and raw IEEE-754 values are
  admitted without semantic clamping; malformed/truncated fields, overlong values, invalid
  identifiers, invalid strict registry IDs, impossible counts, and residual bytes fail closed.
- Award-stats dispatches through the exact nine stat types. Block/item/entity backing misses
  resolve to air/air/pig, while stat-type and custom-stat misses fault. Duplicate typed keys replace
  earlier wire values before application, omitted values remain unchanged, and an open stats screen
  receives exactly one callback even for an empty delta.
- Health publication compares full health, food, and only the saturation zero predicate. Positive
  saturation-only changes are suppressed, negative zero is zero, and NaN health republishes every
  tick. Client application locks first-value behavior, hurt/increase timers, finite/infinite/NaN
  health clamping, and direct signed food/saturation replacement.
- Experience publication follows health, uses the total-experience marker and preserves the `-1`
  collision. Explicit respawn projection does not advance that marker. The client replaces all
  three fields directly and resets its display timer only on Java-float progress inequality,
  including repeated NaN.
- Cooldowns are keyed by strict namespaced group. Nonzero durations replace wrapped intervals,
  zero removes even after a newer replacement, negative durations survive until the next ordinary
  tick, and expiry publishes zero before vitals and experience. No generation, request token, or
  acknowledgement protects receive order.
- Statistics assignments mark exact typed keys dirty; positive increment overflow saturates and
  negative underflow narrows with Java wrapping. A request drains exactly one delta response,
  including empty, while dirtiness alone emits nothing. The integrated Play projection requires an
  installed level and applies all four families only to that connection's local player.

## Evidence

- `crates/ferrite-protocol/src/java_26_2/play/clientbound/player_projection/`
- `crates/ferrite-protocol/tests/c3/play_clientbound_player_projection.rs`

Focused validation:

```text
cargo test -p ferrite-protocol --test c3 play_clientbound_player_projection
10 passed; 0 failed
cargo test -p ferrite-protocol --test c3
229 passed; 0 failed
cargo test -p ferrite-protocol --test c1
68 passed; 0 failed
cargo clippy -p ferrite-protocol --all-targets --all-features -- -D warnings
```

The batch acceptance gate is `cargo ferrite task check` followed by `git diff --check`.
