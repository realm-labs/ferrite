# G01-P6-S009 — PLY-004 Boat and Harness Runtime

## Result

Complete. Both `SourceSpecified` item slices primarily owned by `PLY-004` map to modular
production boat, harness, and identity-catalog runtimes plus the required behavioral test owner.

## Evidence

Production owner:

- `ferrite-gameplay::item::runtime::transport::{catalog,boat,harness}`.

Committed test owner:

- `crates/ferrite-gameplay/tests/slices/items/ply_004.rs`.

Validated commands:

```text
cargo test -p ferrite-gameplay --test slices items_ply_004 --all-features
cargo ferrite task check
git diff --check
```

Focused result before the universal gate:

```text
12 PLY-004 item tests passed; 0 failed
20 boat/raft identities; 16 harness identities; 10 boat families; 5 trade records
```

The runtime models audited caller-visible transactions without claiming generic vehicle physics,
loot evaluation, item-entity motion, equipment death release, protocol, recipe, advancement,
trade selection, or renderer ownership. Those joins retain their dedicated manifest batches.
