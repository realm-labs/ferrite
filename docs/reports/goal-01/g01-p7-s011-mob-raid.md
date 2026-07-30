# G01-P7-S011 — MOB-RAID-001

## Result

Complete. The source-specified raid slice now has production owners for omen conversion and
absorption, create/reuse admission, manager retirement, ongoing/cooldown/post-raid state, wave
construction, member cleanup, rewards, projection selection and persistence reconstruction.

## Evidence

Production owner:

- `ferrite-gameplay::mob::runtime::sim_002::{omen,manager,raid,waves}`.

Committed test owner:

- `crates/ferrite-gameplay/tests/slices/mobs/sim_002.rs`.

Design contract:

- [MOB-RAID-001 raid runtime](../../development/mob-raid-runtime.md).

Validated commands:

```text
cargo test -p ferrite-gameplay --test slices mobs_sim_002
cargo clippy -p ferrite-gameplay --all-targets --all-features -- -D warnings
cargo ferrite task check
git diff --check
```

Focused result before the universal gate:

```text
14 MOB-RAID-001 tests passed; 0 failed
1 source-specified slice
4 responsibility-owned runtime modules
```

The tests lock exact threshold and order behavior for omen, manager, raid and wave state, including
strict reuse distance, negative center flooring, preincrement IDs, live gamerule retirement,
cooldown recomputation, cleanup boundaries, fixed group tables, spawn probing, riders, horn
recipients and nonduplicating persistence reconstruction.
