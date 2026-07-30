# G01-P7-S002 — ENT-002 Vehicles

## Result

Complete. The source-specified `ENT-VEHICLES-001` slice now has production owners for common
vehicle damage, boat motion and contacts, feature-selected minecart physics, collision and
dismount, and all audited minecart subtype hooks. The runtime remains protocol-neutral and returns
deterministic values for later Region integration.

## Evidence

Production owner:

- `ferrite-gameplay::entity::runtime::ent_002::{damage,boat,minecart,subtypes}`.

Committed test owner:

- `crates/ferrite-gameplay/tests/slices/entities/ent_002.rs`.

Design contract:

- [ENT-002 vehicle runtime](../../development/entity-vehicle-runtime.md).

Validated commands:

```text
cargo test -p ferrite-gameplay --test slices entities_ent_002 -- --nocapture
cargo clippy -p ferrite-gameplay --all-targets --all-features -- -D warnings
cargo ferrite task check
git diff --check
```

Focused result before the universal gate:

```text
11 ENT-002 vehicle tests passed; 0 failed
1 source-specified slice
10 rail shapes and both minecart engines covered
```

The tests lock common damage admission and strict destruction thresholds; every boat float status,
input, underwater, bubble, contact, attachment, and dismount boundary; old/improved minecart
selection, off-rail and rail motion, power and opposing-V gates, collision and furnace priority;
and rideable, furnace, TNT, hopper, spawner, command, and container subtype behavior.
