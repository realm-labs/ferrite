# G01-P6-S005 — ITM-002 Container Runtime

## Result

Complete. All seven `SourceSpecified` slices primarily owned by `ITM-002` map to modular production
container, storage, menu, transfer, Hopper and dispenser/dropper runtimes plus the required
behavioral test owner.

## Evidence

Production owners:

- `ferrite-gameplay::item::runtime::{inventory,menu_click,menu_layout,menu_sync}`;
- `ferrite-gameplay::item::runtime::{container_lifecycle,container_storage}`;
- `ferrite-gameplay::item::runtime::{hopper,dispenser}`.

Committed test owner:

- `crates/ferrite-gameplay/tests/slices/items/itm_002.rs`.

Validated commands:

```text
cargo test -p ferrite-gameplay --test slices items_itm_
cargo ferrite task check
git diff --check
```

Focused result before the universal gate:

```text
23 ITM-001/ITM-002 tests passed; 0 failed
7 slices; 25 registered menu layouts; 80 explicit dispenser entries
```

Loot evaluation, concrete dispenser actions, entity construction, Region event delivery, protocol
codecs, crafting outputs, piglin AI and client presentation retain their later owners. No deferred
experiment or guessed behavior was introduced.
