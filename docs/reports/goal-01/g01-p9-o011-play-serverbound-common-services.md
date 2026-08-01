# G01-P9-O011 Play Serverbound Common-Services Gate Report

## Result

Ferrite locks the five serverbound Play common-service packets in
`PROTO-PLAY-SERVERBOUND-COMMON-SERVICES-001`. Cookie refusal and custom-payload discard preserve
the fixed base behavior. Ping, resource-pack tracking, and custom-click dispatch are explicit
capabilities; all service-backed paths default to disabled or the audited log-only degradation.

## Verified boundaries

- IDs 21/22/38/49/68 are locked to cookie response, custom payload, ping request, resource-pack
  response, and custom-click action. The packet inventory resolves through the audited catalog and
  the required Play decoder remains fail-closed for every identity.
- Every cookie response is rejected and every custom payload is ignored without becoming world or
  gameplay authority. These fixed base outcomes are not incorrectly hidden behind a feature flag.
- Ping is default-disabled. Once explicitly enabled it echoes the exact signed 64-bit token through
  the direct lane, without correlation state, permission checks, timeout state, or gameplay
  acknowledgement.
- Resource-pack processing is server-processor ordered and default-disabled. All eight strict
  action values are represented; only `Declined` for a required pack disconnects, and no UUID or
  configuration-task correlation is invented for Play.
- Custom click preserves the locked base log-only behavior. Optional dispatch requires both the
  capability and a separately registered handler; enabling the capability alone degrades without
  mutation.
- Cookie refusal, payload discard, and ping echo remain receiving-thread direct. Resource-pack and
  custom-click decisions remain server-processor ordered, with no cross-family acknowledgement.

No optional capability receives default network wiring. Full codecs or registered service
implementations require an explicit child batch.

## Evidence

- `crates/ferrite-protocol/src/java_26_2/play/serverbound/common_services/`
- `crates/ferrite-protocol/tests/c4/play_serverbound_common_services.rs`

Focused validation:

```text
cargo test -p ferrite-protocol --test c4 play_serverbound_common_services --all-features
8 passed; 0 failed
cargo clippy -p ferrite-protocol --all-targets --all-features -- -D warnings
cargo ferrite source verify
```

The batch acceptance gate is `cargo ferrite task check` followed by `git diff --check`.
