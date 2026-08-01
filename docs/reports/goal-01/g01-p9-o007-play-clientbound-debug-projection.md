# G01-P9-O007 Play Clientbound Debug-Projection Gate Report

## Result

Ferrite explicitly gates all five clientbound Play diagnostic packets in
`PROTO-PLAY-CLIENTBOUND-DEBUG-PROJECTION-001`. Diagnostics default to disabled. Even after the
configuration flag is enabled, the family degrades as unavailable until a separately registered
optional-service implementation exists, so Goal 01 does not claim or accidentally expose an
enabled diagnostics service.

## Verified boundaries

- IDs 26–30 are locked to block, chunk, and entity values, events, and remote samples. The required
  Play decoder remains fail-closed for this optional family, and the empty remote-sample body locks
  the compression-threshold golden frame `04001e0000`.
- The strict 16-entry debug-subscription registry is represented with raw IDs 0–15. Dedicated
  server tick time is sample-only; entity/block intersection values expire after 100 ticks,
  redstone orientations and neighbor updates after 200, and game events after 60. Other value
  subscriptions persist until replacement, clear, or teardown.
- Disabled diagnostics omit every packet. Enabled diagnostics refuse unauthorized requesters and
  degrade explicitly while no optional service is registered. Requested subscriptions are retained
  outside this stateless gate, while only an authorized and effective subscription can emit.
- Value updates replace or clear connection-local projection entries, events append, and samples
  log immediately. Unrequested and untracked targets are omitted, unresolved entities are ignored,
  and dedicated-server samples are unavailable in unsupported environments.
- Expiry is exact at `game_time >= deadline`. Reconfiguration, reconnect, and disconnect all clear
  the connection projection and requested subscriptions. The effect vocabulary exposes no path to
  authoritative world mutation, sequence numbers, acknowledgements, or reliability claims.

The complex debug value codecs and synchronizer producers remain outside Goal 01. Enabling them
requires an explicit registered child batch, as required by the Goal plan's C4 boundary.

## Evidence

- `crates/ferrite-protocol/src/java_26_2/play/clientbound/debug_projection/`
- `crates/ferrite-protocol/tests/c4/play_clientbound_debug_projection.rs`

Focused validation:

```text
cargo test -p ferrite-protocol --test c4 play_clientbound_debug_projection --all-features
8 passed; 0 failed
cargo clippy -p ferrite-protocol --all-targets --all-features -- -D warnings
cargo ferrite source verify
source policy verified: 1167 handwritten Rust files, maximum 1200 physical lines
```

The batch acceptance gate is `cargo ferrite task check` followed by `git diff --check`.
