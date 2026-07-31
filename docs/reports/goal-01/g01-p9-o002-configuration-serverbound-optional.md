# G01-P9-O002 Configuration Serverbound Optional-Gate Report

## Result

Ferrite implements a fail-closed C4 adapter for all four serverbound optional configuration
packets in `PROTO-CONFIGURATION-SERVERBOUND-OPTIONAL-001`. The required C1 decoder continues to
reject every optional identity; an explicitly owned service must select the isolated optional
decoder and stateful gate before any response can be handled.

## Verified boundaries

- IDs 1, 6, 8, and 9 are locked to cookie response, resource-pack response, custom-click action,
  and code-of-conduct acceptance. Required identities and unknown IDs fail at the optional-family
  boundary.
- Cookie responses preserve nullable values capped at 5,120 bytes and require both an enabled
  cookie service and the exact pending request key. A matching response consumes the request once;
  mismatched or duplicate responses are configuration faults.
- Resource-pack responses preserve the UUID and reject action ordinals outside 0–7. Accepted and
  downloaded are nonterminal; all other actions advance the current task. The locked behavior does
  not correlate a terminal UUID, and declining a required pack returns an explicit disconnect
  decision.
- Custom-click actions preserve their identifier and length-prefixed nullable NBT. The prefix is
  capped at 65,536 bytes, with an independent 32,768-byte NBT accumulator and depth 16. Dispatch is
  possible only through the enabled server-owned handler and never advances the current task.
- Code-of-conduct acceptance advances only the matching current task. Disabled services and
  unsolicited/wrong-task packets produce typed faults instead of silent C1 success.
- Every successful result is connection task advancement or server-owned custom action dispatch;
  no optional packet is mapped to authoritative world state.

## Evidence

- `crates/ferrite-protocol/src/java_26_2/configuration/serverbound/optional.rs`
- `crates/ferrite-protocol/tests/c4/configuration_serverbound_optional.rs`

Focused validation:

```text
cargo test -p ferrite-protocol --test c4 configuration_serverbound_optional --all-features
10 passed; 0 failed
cargo clippy -p ferrite-protocol --all-targets --all-features -- -D warnings
cargo ferrite source verify
source policy verified: 1149 handwritten Rust files, maximum 1200 physical lines
```

The batch acceptance gate is `cargo ferrite task check` followed by `git diff --check`.
