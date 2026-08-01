# G01-P9-O014 Play Serverbound Reconfiguration Gate Report

## Result

Ferrite explicitly gates fieldless serverbound Play ID 16 in
`PROTO-PLAY-SERVERBOUND-RECONFIGURATION-001`. Reconfiguration defaults to disabled and requires a
separately registered service. An admitted acknowledgement is legal only under the old Play
listener's waiting state and installs Configuration inbound directly at the terminal network
boundary.

## Verified boundaries

- ID 16 is locked to `minecraft:configuration_acknowledged`; its fieldless compression frame is
  exactly `020010`, and the required Play decoder remains fail-closed for the optional identity.
- An early acknowledgement faults while the server is still in ordinary Play. A valid transition
  moves Play to waiting and then Configuration. A duplicate is no longer a legal Play
  acknowledgement and faults the old state machine.
- The acknowledgement validates the waiting state, captures the replacement common-listener
  cookie, and installs Configuration inbound without a server-thread transfer.
- The replacement cookie contains profile, current latency, latest client information, and the
  transferred flag. It deliberately excludes the client's cookie map and carries no world data.
- Direction-local order remains asymmetric: the server sets waiting, saves/removes, sends terminal
  clientbound ID 118, and only then changes outbound; the client changes inbound before sending ID
  16 under Play and changes outbound afterward; the server changes inbound only after ID 16.
- ID 16 acknowledges no registry, chat, command, container, teleport, block, or gameplay state.
  Ordinary Configuration tasks and finish remain solely responsible for recreating Play.

No optional handler is installed by default. Network-driver wiring for the registered service
requires an explicit child batch.

## Evidence

- `crates/ferrite-protocol/src/java_26_2/play/serverbound/reconfiguration/`
- `crates/ferrite-protocol/tests/c4/play_serverbound_reconfiguration.rs`

Focused validation:

```text
cargo test -p ferrite-protocol --test c4 play_serverbound_reconfiguration --all-features
7 passed; 0 failed
cargo clippy -p ferrite-protocol --all-targets --all-features -- -D warnings
cargo ferrite source verify
source policy verified: 1199 handwritten Rust files, maximum 1200 physical lines
```

The batch acceptance gate is `cargo ferrite task check` followed by `git diff --check`.
