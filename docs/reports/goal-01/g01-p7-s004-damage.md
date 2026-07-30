# G01-P7-S004 — ENT-005 Damage

## Result

Complete. Four source-specified damage slices now have production owners for wrapper and cooldown
admission, blocking and retaliation, defense/absorption/health reduction, and common plus
sulfur-cube knockback. The owners expose ordered values and effects for later Region composition
with effects and death.

## Evidence

Production owner:

- `ferrite-gameplay::entity::runtime::ent_005::{admission,blocking,reduction,knockback}`.

Committed test owner:

- `crates/ferrite-gameplay/tests/slices/entities/ent_005.rs`.

Design contract:

- [ENT-005 damage runtime](../../development/entity-damage-runtime.md).

Validated commands:

```text
cargo test -p ferrite-gameplay --test slices entities_ent_005 -- --nocapture
cargo clippy -p ferrite-gameplay --all-targets --all-features -- -D warnings
cargo ferrite task check
git diff --check
```

Focused result before the universal gate:

```text
16 ENT-005 damage tests passed; 0 failed
4 source-specified slices
4 responsibility modules
```

The tests lock immunity and wrapper order; signed zero, NaN and infinity behavior; block/freeze/
helmet and cooldown order; full-block false results; shield use/angle/reduction/durability and
disable behavior; Hoglin/Ravager reactions; armor/Breach/Resistance/protection/witch formulas;
absorption, health, stats and combat; Wolf/Camel/Animal/Armadillo/Copper Golem hooks; common
knockback retries and subtype gates; player indication; and sulfur-cube settings and zero-power
side effects.
