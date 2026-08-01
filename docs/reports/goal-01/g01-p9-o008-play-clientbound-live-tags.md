# G01-P9-O008 Play Clientbound Live-Tags Gate Report

## Result

Ferrite explicitly gates the Play `minecraft:update_tags` packet in
`PROTO-PLAY-CLIENTBOUND-LIVE-TAGS-001`. Live data-pack reload publication defaults to disabled. If
the flag is enabled without a separately registered reload service, the packet degrades as
unavailable and the existing configured tag snapshot remains authoritative.

## Verified boundaries

- Play clientbound ID 134 is locked to `minecraft:update_tags`. Its empty generic registry map has
  packet body `860100` and compression-threshold frame `0400860100`; the required Play decoder
  remains fail-closed for this optional family.
- The Play payload is configuration-identical: registry and tag identifiers own signed-VarInt
  member lists. The existing Configuration codec remains the grammar owner instead of introducing a
  second dormant codec for an unregistered optional service.
- The gate defaults to disabled, degrades while no live-reload service is registered, and publishes
  only a committed reload snapshot. In-memory connections retain local bindings rather than
  applying remote network tags.
- Every registry must prepare successfully before any binding replacement. A missing registry
  preserves all previous bindings; negative and out-of-range member IDs are filtered, while valid
  encounter order and duplicates remain exact.
- A successful remote application replaces prepared adapter-local relationships, then recomputes
  fuel values and creative search tag trees. Reload publication order is tags, recipes, then recipe
  book. There is no reload generation, acknowledgement, atomic tags/recipes transaction, or raw-ID
  persistence claim.

The live reload publisher and Play payload codec remain outside Goal 01 and require an explicit
registered child batch before this gate can emit.

## Evidence

- `crates/ferrite-protocol/src/java_26_2/play/clientbound/live_tags/`
- `crates/ferrite-protocol/tests/c4/play_clientbound_live_tags.rs`

Focused validation:

```text
cargo test -p ferrite-protocol --test c4 play_clientbound_live_tags --all-features
8 passed; 0 failed
cargo clippy -p ferrite-protocol --all-targets --all-features -- -D warnings
cargo ferrite source verify
source policy verified: 1172 handwritten Rust files, maximum 1200 physical lines
```

The batch acceptance gate is `cargo ferrite task check` followed by `git diff --check`.
