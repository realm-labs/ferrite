# G01-P9-F003 Completion Projection Report

## Result

Ferrite implements and verifies clientbound IDs 15 and 23 in
`PROTO-PLAY-CLIENTBOUND-COMPLETIONS-001`. Command transactions, ranges, tooltips, pending futures
and custom candidate sets remain connection/client presentation state and are not world identity.

## Verified boundaries

- The two official empty packet bodies lock command suggestions and custom chat completion add.
  Signed transaction/start/length values, wrapping range endpoints, ordered default-UTF entries,
  nullable trusted tooltips and all three strict custom actions round-trip exactly.
- Negative/impossible counts, invalid action ordinals, over-limit strings, malformed tooltips,
  malformed VarInts, truncation and residual bytes fail closed. Nonzero tooltip booleans normalize
  to true, while signed range values and wrapping overflow remain accepted transport values.
- Every command result is converted before correlation. Only the latest exact transaction completes
  and clears its pending future; canceled/stale/duplicate IDs are ignored, while idle transaction
  `-1` reproduces the missing-future fault. A live counter that wraps to `-1` still completes.
- Range validity is deferred until UI application and uses Java UTF-16 input length. Invalid,
  negative, reversed or out-of-input endpoints do not become transport failures.
- Canonical publication strips at most one leading slash at parse time, retains handler-time parsed
  state without a server outstanding-request table, preserves raw transaction/range/entry order
  and tooltip values, and truncates only entries after the first 1,000. Independent parsed requests
  may publish in completion order rather than request order.
- Custom add/remove/set operations mutate a set in receive order. Empty custom state yields current
  player names; nonempty state yields the player/custom union. Duplicates and input list order are
  not retained, and custom mutation cannot complete or cancel a command future.
- The locked base server has no ID-23 publisher; the packet remains an adapter-controlled facility.
  Both packets require an installed Ready-for-Terrain Play projection and own no cross-family
  generation, signature acknowledgement or command result.

## Evidence

- `crates/ferrite-protocol/src/java_26_2/play/clientbound/completion/`
- `crates/ferrite-protocol/tests/c3/play_clientbound_completions.rs`

Focused validation:

```text
cargo test -p ferrite-protocol --test c3 play_clientbound_completions
9 passed; 0 failed
cargo test -p ferrite-protocol --test c3
204 passed; 0 failed
cargo test -p ferrite-protocol --test c1
68 passed; 0 failed
cargo clippy -p ferrite-protocol --all-targets --all-features -- -D warnings
```

The batch acceptance gate is `cargo ferrite task check` followed by `git diff --check`.
