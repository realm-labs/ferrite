# G01-P6-S001 — BLK-001-Owned Item Runtime

## Result

Complete. All ten `SourceSpecified` slices primarily owned by `BLK-001` map to production code and
the required behavioral test owner.

The partition closes ten imported item behavior families containing eighteen identities. It keeps
Java raw IDs separate from persistent content ordering and exposes the exact item-owned component,
consumption, material and interaction decisions needed by later generic engines.

## Evidence

Production owners:

- `ferrite-gameplay::item::runtime::catalog` — closed identity, raw-ID, component and family
  ownership;
- `ferrite-gameplay::item::runtime::consumption` — food admission and ordered effects;
- `ferrite-gameplay::item::runtime::materials` — live tag, repair and fuel dispatch;
- `ferrite-gameplay::item::runtime::interaction` — item-owned entity and composter decisions.

Committed test owner:

- `crates/ferrite-gameplay/tests/slices/items/blk_001.rs`.

Validated commands:

```text
cargo test -p ferrite-gameplay --test slices
cargo clippy -p ferrite-gameplay --all-targets --all-features -- -D warnings
cargo ferrite content verify
cargo ferrite task check
git diff --check
```

Focused results before the universal gate:

```text
6 passed; 0 failed
10 slices; 10 imported behavior families; 18 item identities
```

## Ownership notes

Recipe matching/allocation, loot selection, trade lifecycle, advancement publication, world
placement, protocol projection and full entity state machines retain their explicit later owners.
Their locked content records and identity-specific inputs are not reimplemented here. No deferred
experiment or guessed vanilla behavior was introduced.
