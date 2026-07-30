# G01-P6-S012 — SIM-004 Material Item Runtime

## Result

Complete. All 15 `SourceSpecified` item slices primarily owned by `SIM-004` map to eight modular
production owners, 15 closed imported families, and one behavioral test owner. Goal 01 now has
all 95 item slices verified.

## Evidence

Production owner:

- `ferrite-gameplay::item::runtime::sim_004::{brewing,catalog,dried_kelp,firework,joins,loot,materials,turtle}`.

Committed test owner:

- `crates/ferrite-gameplay/tests/slices/items/sim_004.rs`.

Validated commands:

```text
cargo test -p ferrite-gameplay --test slices items_sim_004 -- --nocapture
cargo clippy -p ferrite-gameplay --all-targets --all-features -- -D warnings
cargo ferrite task check
git diff --check
```

Focused result before the universal gate:

```text
17 SIM-004 material tests passed; 0 failed
15 slices; 15 imported families; 15 item identities
```

The implementation preserves the distinct Fortune algorithms, Silk/explosion ordering,
identity-specific death and food gates, complete Firework component transitions, exact brewing
edges, Dried Kelp's `4001`-tick fuel, Lapis consumption, live repair/trim roles, and Turtle's
one-shot adulthood boundary. Generic execution engines and Region delivery remain outside this
batch.
