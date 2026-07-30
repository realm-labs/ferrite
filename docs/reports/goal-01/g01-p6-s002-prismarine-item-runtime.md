# G01-P6-S002 — Prismarine Item Runtime

## Result

Complete. `ITM-PRISMARINE-MATERIAL-RUNTIME-001` maps to production catalog and prismarine-profile
code plus its required behavioral test owner.

## Evidence

Production owners:

- `ferrite-gameplay::item::runtime::catalog` — two identities, raw IDs, defaults and exact family;
- `ferrite-gameplay::item::runtime::prismarine` — Guardian, Elder Guardian, Buried Treasure,
  Sea Lantern and recipe/unlock profiles.

Committed test owner:

- `crates/ferrite-gameplay/tests/slices/items/blk_002.rs`.

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
10 item-partition tests passed; 0 failed
1 slice; 1 imported behavior family; 2 item identities
```

Generic loot evaluation, crafting, advancement, persistence and client projection remain pending
under their explicit shared owners. No deferred experiment or guessed vanilla behavior was added.
