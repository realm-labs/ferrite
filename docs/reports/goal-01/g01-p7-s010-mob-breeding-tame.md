# G01-P7-S010 — MOB-006 Breeding and Tame

## Result

Complete. The source-specified `MOB-BREED-TAME-001` slice now has production owners for age/love
clocks, mate approach, generic and special child commits, inheritance families, tame/trust authority,
horse temper and owner teleport.

## Evidence

Production owner:

- `ferrite-gameplay::mob::runtime::mob_006::{age,breeding,families,tame}`.

Committed test owner:

- `crates/ferrite-gameplay/tests/slices/mobs/mob_006.rs`.

Design contract:

- [MOB-006 breeding and tame runtime](../../development/mob-breeding-and-tame-runtime.md).

Validated commands:

```text
cargo test -p ferrite-gameplay --test slices mobs_mob_006
cargo clippy -p ferrite-gameplay --all-targets --all-features -- -D warnings
cargo ferrite task check
git diff --check
```

Focused result before the universal gate:

```text
17 MOB-006 breeding/tame tests passed; 0 failed
1 source-specified slice
4 responsibility-owned runtime modules
```

The tests lock signed/forced/locked age, love and feeding; mate selection and adjusted timing; null,
generic and special producer order; variant and equine inheritance; tame and trust odds; horse
temper; and owner-teleport distance, sample and collision boundaries.
