# G01-P7-S003 — ENT-004 Projectiles

## Result

Complete. The source-specified `ENT-PROJECTILES-001` slice now has production owners for common
launch, owner, sweep, deflection, border and callback rules; the three projectile-breaking block
callbacks; throwable, arrow and hurting-projectile families; and every remaining catalog family
listed by the audited leaf.

## Evidence

Production owner:

- `ferrite-gameplay::entity::runtime::ent_004::{geometry,block,throwable,arrow,hurting,special}`.

Committed test owner:

- `crates/ferrite-gameplay/tests/slices/entities/ent_004.rs`.

Design contract:

- [ENT-004 projectile runtime](../../development/entity-projectile-runtime.md).

Validated commands:

```text
cargo test -p ferrite-gameplay --test slices entities_ent_004 -- --nocapture
cargo clippy -p ferrite-gameplay --all-targets --all-features -- -D warnings
cargo ferrite task check
git diff --check
```

Focused result before the universal gate:

```text
16 ENT-004 projectile tests passed; 0 failed
1 source-specified slice
6 responsibility modules
```

The tests lock explicit RNG cardinality; equal-distance and endpoint ties; owner, deflector,
permission and gamerule filters; callback order; gravity/inertia and hit order; egg, pearl and
potion thresholds; arrow stable sorting, piercing, damage, failure and timers; Trident loyalty;
fireball, skull, cloud and Wind Charge behavior; and Firework, Spit, Shulker, Fishing, Eye and Fang
terminal boundaries.
