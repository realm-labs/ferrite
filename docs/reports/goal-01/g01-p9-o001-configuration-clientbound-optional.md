# G01-P9-O001 Configuration Clientbound Optional-Gate Report

## Result

Ferrite explicitly gates all 11 clientbound optional configuration packets in
`PROTO-CONFIGURATION-CLIENTBOUND-OPTIONAL-001`. A fresh offline connection enables none of the
services, so the minimum configuration trace remains unchanged. Enabling a service admits only its
owned packets and returns a typed connection-local effect instead of claiming authoritative world
mutation.

## Verified boundaries

- The gate inventory locks catalog IDs 0, 6, 8–11, and 15–19 to the exact cookie, reset-chat,
  resource-pack, transfer, report-details, server-links, dialog, and code-of-conduct identities.
- Eight independently owned capabilities cover cookies, reconfiguration, resource packs, transfer,
  report details, server links, dialogs, and code of conduct; all default to disabled and disabled
  packets are explicitly omitted with their owning service.
- Cookie request is classified as request/response, store-cookie as connection-local persistence,
  resource-pack push and code of conduct as blocking tasks, and pop/report/link/dialog packets as
  presentation-only effects.
- Reset-chat is omitted outside play-to-configuration re-entry even when its capability is enabled.
  Transfer is explicitly refused in singleplayer and otherwise classified as a connection transfer.
- The decision vocabulary contains only request, blocking, presentation, retained-chat, and
  connection-state effects. It exposes no authoritative world-state mutation path.

## Evidence

- `crates/ferrite-protocol/src/java_26_2/configuration/clientbound/optional.rs`
- `crates/ferrite-protocol/tests/c4/configuration_clientbound_optional.rs`

Focused validation:

```text
cargo test -p ferrite-protocol --test c4 configuration_clientbound_optional --all-features
6 passed; 0 failed
cargo clippy -p ferrite-protocol --all-targets --all-features -- -D warnings
cargo ferrite source verify
source policy verified: 1147 handwritten Rust files, maximum 1200 physical lines
```

The batch acceptance gate is `cargo ferrite task check` followed by `git diff --check`.
