# G01-P9-F001 Boss and Waypoint Projection Report

## Result

Ferrite implements and verifies IDs 9 and 138 in
`PROTO-PLAY-CLIENTBOUND-BOSS-WAYPOINT-001`. The adapter retains normalized boss and locator intent
while the Java 26.2 boundary owns operation ordinals, UUID/string keys, interpolation anchors,
icons, presentation collections and renderer ordering.

## Verified boundaries

- Six official packet bodies lock boss add/remove and waypoint untrack, position, chunk and
  azimuth shapes. All boss operations, strict color/overlay ordinals, waypoint location forms,
  modulo-three signed operations, nonzero booleans, opaque optional RGB and raw IEEE values are
  covered; malformed, truncated and residual bodies fail closed.
- Boss projection preserves linked insertion order on replacement, missing-update faults,
  tolerated missing removal, low property bits, any-bar aggregate gates, 100-ms chained progress
  interpolation and the one-third-height rendering cutoff.
- Boss publication starts visible, keeps idempotent membership, snapshots on show, suppresses equal
  setters with Java float equality, marks every changed field dirty and emits no hidden deltas.
- Waypoint projection replaces complete track state, removes by key, mutates only matching location
  content, preserves icons on update, warns on type mismatch and fails missing updates. Position,
  nearby UUID eye, chunk, azimuth, empty and descending-distance marker projection are locked.
- Waypoint publication covers self/first-tick/gamerule, spectator, riding and strict range admission;
  the 332-block representation boundary; team/explicit icon color; block, chunk and azimuth update
  thresholds; replacement tracks; and canonical untrack.
- End-to-end codec registration uses the catalog identities and Ready-for-Terrain projection gate.
  Neither family introduces an acknowledgement, generation fence or reordering barrier.

## Evidence

- `crates/ferrite-protocol/src/java_26_2/play/clientbound/boss_waypoint/`
- `crates/ferrite-protocol/tests/c3/play_clientbound_boss_waypoint.rs`

Focused validation:

```text
cargo test -p ferrite-protocol --test c3 play_clientbound_boss_waypoint -- --nocapture
10 passed; 0 failed
cargo test -p ferrite-protocol --test c3
182 passed; 0 failed
cargo clippy -p ferrite-protocol --all-targets --all-features -- -D warnings
```

The batch acceptance gate is `cargo ferrite task check` followed by `git diff --check`.
