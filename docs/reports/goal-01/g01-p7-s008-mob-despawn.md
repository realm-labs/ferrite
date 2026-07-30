# G01-P7-S008 — MOB-003 Despawn

## Result

Complete. The source-specified `MOB-DESPAWN-001` slice now has a production owner for root-loop
invocation, strict hard/soft distance evaluation, inactivity/RNG cadence and every audited custom-
persistence and far-removal subtype policy.

## Evidence

Production owner:

- `ferrite-gameplay::mob::runtime::mob_003`.

Committed test owner:

- `crates/ferrite-gameplay/tests/slices/mobs/mob_003.rs`.

Design contract:

- [MOB-003 despawn runtime](../../development/mob-despawn-runtime.md).

Validated commands:

```text
cargo test -p ferrite-gameplay --test slices mobs_mob_003
cargo clippy -p ferrite-gameplay --all-targets --all-features -- -D warnings
cargo ferrite task check
git diff --check
```

Focused result before the universal gate:

```text
11 MOB-003 despawn tests passed; 0 failed
1 source-specified slice
1 responsibility-owned runtime module
```

The tests lock Peaceful/persistence/no-player ordering, category and soft-distance equality, timer
600/601, draw-before-distance/policy order, post-hard soft continuation, AI increment, passenger/
leash and subtype persistence, and every animal/fish/raid/patrol/Piglin/Zombie Villager removal
override family.
