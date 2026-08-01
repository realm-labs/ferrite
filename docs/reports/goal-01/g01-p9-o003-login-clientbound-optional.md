# G01-P9-O003 Login Clientbound Optional-Gate Report

## Result

Ferrite implements an isolated C4 adapter for the three optional clientbound login packets in
`PROTO-LOGIN-CLIENTBOUND-OPTIONAL-001`. The offline baseline continues to omit encryption hello,
custom query, and cookie request, while explicitly enabled services receive typed transition or
response-correlation effects.

## Verified boundaries

- IDs 1, 4, and 5 are locked to encryption hello, custom query, and cookie request. The required C1
  decoder still rejects all three identities, and the optional decoder rejects required identities.
- Encryption hello preserves the `UTF(20)` server ID, frame-bounded public-key and challenge arrays,
  and nonzero-normalizing authenticate Boolean. Its gate requires a prior valid login hello, omits
  encryption for in-memory connections, and otherwise explicitly enters the KEY stage.
- Custom query preserves its signed transaction VarInt, identifier, and raw remainder capped at
  1,048,576 bytes. Enabling its service returns only a correlated-query registration effect.
- Cookie request preserves its identifier and returns only a cookie-response registration effect.
  Neither request is silently emitted from the base listener.
- All three capabilities default to disabled. Their effects are connection-local authentication or
  response correlation and expose no gameplay, persistence, ECS, or authoritative world mapping.

## Evidence

- `crates/ferrite-protocol/src/java_26_2/login/clientbound/optional.rs`
- `crates/ferrite-protocol/tests/c4/login_clientbound_optional.rs`

Focused validation:

```text
cargo test -p ferrite-protocol --test c4 login_clientbound_optional --all-features
7 passed; 0 failed
cargo clippy -p ferrite-protocol --all-targets --all-features -- -D warnings
cargo ferrite source verify
source policy verified: 1151 handwritten Rust files, maximum 1200 physical lines
```

The batch acceptance gate is `cargo ferrite task check` followed by `git diff --check`.
