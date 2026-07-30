# G01-P7-S006 — ENT-007 Death

## Result

Complete. The source-specified `ENT-DEATH-TRANSACTION-001` slice now has production owners for
death protection, ordinary and player death entry, all audited loot/equipment/experience branches,
and common, Creaking and Ender Dragon removal timelines.

## Evidence

Production owner:

- `ferrite-gameplay::entity::runtime::ent_007::{protection,entry,drops,experience,timelines}`.

Committed test owner:

- `crates/ferrite-gameplay/tests/slices/entities/ent_007.rs`.

Design contract:

- [ENT-007 death runtime](../../development/entity-death-runtime.md).

Validated commands:

```text
cargo test -p ferrite-gameplay --test slices entities_ent_007
cargo clippy -p ferrite-gameplay --all-targets --all-features -- -D warnings
cargo ferrite task check
git diff --check
```

Focused result before the universal gate:

```text
15 ENT-007 death tests passed; 0 failed
1 source-specified slice
5 responsibility-owned runtime modules
```

The tests lock hand scans and Totem order; ordinary/player/conversion entry; skull and wither-rose
gates; loot context and construction draws; equipment chance, damage and subtype rules; player
inventory/item drops; XP eligibility, mutation, splitting and merging; common/Creaking boundaries;
Dragon event, particle, periodic-plus-final reward and removal order; and post-death pearl removal.
