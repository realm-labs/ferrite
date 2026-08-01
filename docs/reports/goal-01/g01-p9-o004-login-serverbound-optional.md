# G01-P9-O004 Login Serverbound Optional-Gate Report

## Result

Ferrite implements the three optional serverbound login packets in
`PROTO-LOGIN-SERVERBOUND-OPTIONAL-001` behind isolated, default-closed C4 services. Required C1
login input continues to reject key, custom-query answer, and cookie response rather than silently
accepting traffic that the base listener never requested.

## Verified boundaries

- IDs 1, 2, and 4 are locked to key, custom-query answer, and cookie response. Required and optional
  decoders reject identities owned by the other family.
- Key preserves both frame-bounded encrypted byte arrays and is legal only for the exact KEY task.
  Receipt moves to a pending crypto-verification state and exposes the expected challenge; only a
  later successful verification callback permits encryption installation followed by authentication.
- Custom-query answer preserves the signed transaction VarInt and caps its discarded raw remainder
  at 1,048,576 bytes. The standard null response remains one false marker byte. Handling requires
  the exact explicitly owned transaction and consumes it once.
- Cookie response preserves its identifier and nullable byte array capped at 5,120 bytes. Handling
  requires the exact requested key and consumes it once.
- All three services default to disabled. Disabled, unsolicited, mismatched, duplicate, and
  wrong-stage traffic returns a typed login fault; none can degrade into ordinary C1 success.
- Outputs are limited to crypto/authentication sequencing and connection-local response values;
  there is no world, gameplay, ECS, registry, or persistence mapping.

## Evidence

- `crates/ferrite-protocol/src/java_26_2/login/serverbound/optional.rs`
- `crates/ferrite-protocol/tests/c4/login_serverbound_optional.rs`

Focused validation:

```text
cargo test -p ferrite-protocol --test c4 login_serverbound_optional --all-features
8 passed; 0 failed
cargo clippy -p ferrite-protocol --all-targets --all-features -- -D warnings
cargo ferrite source verify
source policy verified: 1153 handwritten Rust files, maximum 1200 physical lines
```

The batch acceptance gate is `cargo ferrite task check` followed by `git diff --check`.
