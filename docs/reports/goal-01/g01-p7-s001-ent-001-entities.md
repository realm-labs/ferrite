# G01-P7-S001 — ENT-001 Entities

## Result

Complete. Thirty-seven source-specified entity slices now have production owners for lifecycle,
live entity-drop gates, all 37 locked entity identities and construction profiles, and the audited
local subtype transitions. The runtime is protocol-neutral and returns ordered effects for later
Region integration.

## Evidence

Production owner:

- `ferrite-gameplay::entity::runtime::ent_001::{lifecycle,drops,catalog,profiles,aquatic,undead,hostile,raider,passive}`.

Committed test owner:

- `crates/ferrite-gameplay/tests/slices/entities/ent_001.rs`.

Design contract:

- [ENT-001 entity runtime](../../development/entity-runtime-ent-001.md).

Validated commands:

```text
cargo test -p ferrite-gameplay --test slices entities_ent_001 -- --nocapture
cargo clippy -p ferrite-gameplay --all-targets --all-features -- -D warnings
cargo ferrite task check
git diff --check
```

Focused result before the universal gate:

```text
13 ENT-001 slice tests passed; 0 failed
37 source-specified slices
37 locked entity identities across 35 subtype slice owners
158-entry minecraft:entity_type registry matched
```

The tests lock UUID/section/tick/passenger/removal/teleport ordering; independent cramming pushes;
all seven `entity_drops` read locations; aquatic counters and variants; Skeleton-family conversion
and projectiles; hostile and raider state-machine thresholds; Golem, Villager, and Trader
boundaries; and exact construction profiles for every owned entity identity.
