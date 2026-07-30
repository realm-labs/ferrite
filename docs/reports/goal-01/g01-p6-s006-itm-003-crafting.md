# G01-P6-S006 — ITM-003 Crafting and Processing Runtime

## Result

Complete. The `ITM-CRAFT-PROCESS-001` `SourceSpecified` slice maps to modular production recipe,
crafting, cooking, brewing, Campfire, Crafter and workstation runtimes plus its required behavioral
test owner.

## Evidence

Production owners:

- `ferrite-gameplay::item::runtime::{recipe,crafting}`;
- `ferrite-gameplay::item::runtime::{furnace,brewing,campfire}`;
- `ferrite-gameplay::item::runtime::{workstation,item_enchantment,grindstone,anvil}`.

Committed test owner:

- `crates/ferrite-gameplay/tests/slices/items/itm_003.rs`.

Validated commands:

```text
cargo test -p ferrite-gameplay --test slices items_itm_003 --all-features
cargo ferrite task check
git diff --check
```

Focused result before the universal gate:

```text
12 ITM-003 tests passed; 0 failed
7 recipe domains; 21 serializers; 43 Loom patterns
```

Protocol encoding, recipe-display packet projection, Region event delivery, statistics and client
presentation retain their dedicated later owners. Random samples are explicit checked inputs, and
no deferred experiment or guessed behavior was introduced.
