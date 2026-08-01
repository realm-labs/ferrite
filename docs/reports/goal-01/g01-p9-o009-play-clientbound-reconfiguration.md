# G01-P9-O009 Play Clientbound Reconfiguration Gate Report

## Result

Ferrite explicitly gates the terminal Play `minecraft:start_configuration` packet in
`PROTO-PLAY-CLIENTBOUND-RECONFIGURATION-001`. Reconfiguration defaults to disabled and degrades as
unavailable until a separately registered service exists. An enabled path is administrator-only
and cannot publish the terminal packet before ordinary save/removal has committed.

## Verified boundaries

- Play clientbound ID 118 is locked to the fieldless terminal packet. Its packet body is `76` and
  its compression-threshold frame is `020076`; the required Play decoder remains fail-closed for
  this optional family.
- The server order sets the acknowledgement wait state, saves and removes the player through the
  ordinary Play removal path, sends ID 118, then installs Configuration outbound. Publishing before
  committed removal is explicitly refused.
- The client flushes delayed chat, sends a pending last-seen acknowledgement, stores retained chat
  and common state, clears the old level/UI, creates a Configuration listener with a fresh load
  tracker, installs Configuration inbound, sends terminal Play acknowledgement ID 16, and only then
  installs Configuration outbound.
- Profile, telemetry, registries, features, brand, server record, post-disconnect screen, cookies,
  chat state, reports, validated links, seen players, and the insecure-chat-warning flag are carried.
  The load tracker is fresh and the old Play level/UI projection is not carried.
- A second start is illegal after the inbound switch. Ordinary Configuration finish, not ID 118 or
  ID 16 alone, creates a fresh Play projection. Packet objects and old UI/level objects never become
  persistence authority.

The paired serverbound acknowledgement state machine remains assigned to `G01-P9-O014`; O009 locks
its position in the client transition without claiming that separate family complete.

## Evidence

- `crates/ferrite-protocol/src/java_26_2/play/clientbound/reconfiguration/`
- `crates/ferrite-protocol/tests/c4/play_clientbound_reconfiguration.rs`

Focused validation:

```text
cargo test -p ferrite-protocol --test c4 play_clientbound_reconfiguration --all-features
7 passed; 0 failed
cargo clippy -p ferrite-protocol --all-targets --all-features -- -D warnings
cargo ferrite source verify
source policy verified: 1177 handwritten Rust files, maximum 1200 physical lines
```

The batch acceptance gate is `cargo ferrite task check` followed by `git diff --check`.
