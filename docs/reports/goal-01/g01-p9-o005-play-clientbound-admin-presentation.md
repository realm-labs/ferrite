# G01-P9-O005 Play Clientbound Admin-Presentation Gate Report

## Result

Ferrite implements the four C4 clientbound administration-presentation packets in
`PROTO-PLAY-CLIENTBOUND-ADMIN-PRESENTATION-001`. Their codecs are isolated from required Play
families, and every publisher is default-closed behind its exact authorization, recipient, server,
and threshold conditions.

## Verified boundaries

- IDs 39, 40, 50, and 126 are locked to game-rule values, game-test highlight position, low-disk
  warning, and test-instance block status. Required and optional family decoders fail closed across
  their ownership boundary.
- Game-rule values use a generic identifier-to-`UTF(32767)` map; duplicate keys overwrite earlier
  values at decode while unknown keys and unparseable strings remain UI-level inputs. Publication
  requires an authorized direct requester.
- Game-test highlight preserves two packed signed block positions and targets only the invoking
  recipient. Test-instance status preserves trusted component NBT and an optional three-signed-
  VarInt Vec3i, and likewise requires an authorized direct requester.
- Low-disk warning is a fieldless unit. It emits only for a dedicated-server administrator when
  usable space is strictly below 67,108,864 bytes; equality, unknown space, non-dedicated servers,
  and non-administrators omit it. Repeated packets remain repeated toast signals.
- All four services default to disabled. Successful decisions are typed presentation effects only;
  normalized gamerule, test, disk, and world authority remains server-owned.

## Evidence

- `crates/ferrite-protocol/src/java_26_2/play/clientbound/admin_presentation/`
- `crates/ferrite-protocol/tests/c4/play_clientbound_admin_presentation.rs`

Focused validation:

```text
cargo test -p ferrite-protocol --test c4 play_clientbound_admin_presentation --all-features
8 passed; 0 failed
cargo clippy -p ferrite-protocol --all-targets --all-features -- -D warnings
cargo ferrite source verify
source policy verified: 1158 handwritten Rust files, maximum 1200 physical lines
```

The batch acceptance gate is `cargo ferrite task check` followed by `git diff --check`.
