# G01-P6-S003 — BLK-003-Owned Items

## Result

Complete. Nine `SourceSpecified` item slices map to production catalog, consumption, Book and food
family profiles plus the required behavioral test owner.

## Evidence

Production owners:

- `ferrite-gameplay::item::runtime::catalog`;
- `ferrite-gameplay::item::runtime::consumption`;
- `ferrite-gameplay::item::runtime::food_family`;
- `ferrite-gameplay::item::runtime::books`.

Committed test owner:

- `crates/ferrite-gameplay/tests/slices/items/blk_003.rs`.

Validated commands:

```text
cargo test -p ferrite-gameplay --test slices
cargo clippy -p ferrite-gameplay --all-targets --all-features -- -D warnings
cargo ferrite content verify
cargo ferrite task check
git diff --check
```

Focused result before the universal gate:

```text
15 item-partition tests passed; 0 failed
9 slices; 9 imported behavior families; 18 item identities
```

Generic loot, crafting, trade, advancement, entity state and client projection remain with their
later owners. No deferred experiment or guessed behavior was introduced.
