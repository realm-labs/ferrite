# G01-P6-S007 — ITM-006 Enchantment and Loot Runtime

## Result

Complete. Both `SourceSpecified` slices primarily owned by `ITM-006` map to modular production
enchantment and loot runtimes plus the required behavioral test owner.

## Evidence

Production owners:

- `ferrite-gameplay::item::runtime::{enchantment,random}`;
- `ferrite-gameplay::item::runtime::loot::{context,model,evaluator,fill}`.

Committed test owner:

- `crates/ferrite-gameplay/tests/slices/items/itm_006.rs`.

Validated commands:

```text
cargo test -p ferrite-gameplay --test slices items_itm_006 --all-features
cargo ferrite task check
git diff --check
```

Focused result before the universal gate:

```text
11 ITM-006 tests passed; 0 failed
26 loot context sets; 3 loot data kinds; 8 equipment slots
```

The runtime dispatches versioned data by resource identity and does not duplicate vanilla JSON
tables or effects in handwritten switches. Concrete caller contexts, destination refusal policy,
Region stream binding, registry publication and client/menu wire projection retain their dedicated
owners. No deferred experiment or guessed behavior was introduced.
