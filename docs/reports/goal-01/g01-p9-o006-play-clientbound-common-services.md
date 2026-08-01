# G01-P9-O006 Play Clientbound Common-Services Gate Report

## Result

Ferrite implements all 11 clientbound Play common-service packets in
`PROTO-PLAY-CLIENTBOUND-COMMON-SERVICES-001`. The isolated codec preserves their exact Play wire
forms, while eight default-closed service gates classify every enabled result as connection-local
state, diagnostic sampling, resource UI, or dialog/report presentation.

## Verified boundaries

- IDs 21/24/62/80/81/120/129/136/137/139/140 are locked to cookie request, custom payload, pong,
  resource pop/push, cookie store, transfer, reports, links, and dialog clear/show. Required Play and
  C4 common-service decoders remain fail-closed across family ownership.
- Brand uses `UTF(32767)`; unknown custom channels retain a remainder capped at 1,048,576 bytes.
  Pong accepts every signed-long token without pending correlation, so stale/duplicate tokens remain
  diagnostic samples only.
- Resource packets preserve nullable UUIDs, URL/hash bounds, required Boolean, and optional trusted
  prompt. Cookie values cap at 5,120 bytes. Transfer preserves unchecked signed ports and explicitly
  refuses singleplayer.
- Report maps cap at 32 entries with 128/4,096-unit keys/values. Server links preserve known/custom
  labels and URLs; known IDs outside 0–9 select type-zero fallback. Dialog holders use strict
  registered raw IDs or trusted direct NBT, rejecting missing registered entries.
- All services default to disabled. Enabled effects request/replace cookies, replace brand/reports/
  validated links, update resource UI, log pong samples, transfer the connection, or clear/show
  dialog presentation; none maps packet/holder IDs or client service state into world authority.

## Evidence

- `crates/ferrite-protocol/src/java_26_2/play/clientbound/common_services/`
- `crates/ferrite-protocol/tests/c4/play_clientbound_common_services.rs`

Focused validation:

```text
cargo test -p ferrite-protocol --test c4 play_clientbound_common_services --all-features
8 passed; 0 failed
cargo clippy -p ferrite-protocol --all-targets --all-features -- -D warnings
cargo ferrite source verify
source policy verified: 1163 handwritten Rust files, maximum 1200 physical lines
```

The batch acceptance gate is `cargo ferrite task check` followed by `git diff --check`.
