# G01-P7-S007 — MOB-001 Spawning

## Result

Complete. Six source-specified MOB-001 slices now have production owners for hostile policy,
natural spawning, Patrol, Phantom, Wandering Trader and Warden warning/spawn transactions.

## Evidence

Production owner:

- `ferrite-gameplay::mob::runtime::mob_001::{hostile,natural,patrol,phantom,trader,warden}`.

Committed test owner:

- `crates/ferrite-gameplay/tests/slices/mobs/mob_001.rs`.

Design contract:

- [MOB-001 spawning runtime](../../development/mob-spawning-runtime.md).

Validated commands:

```text
cargo test -p ferrite-gameplay --test slices mobs_mob_001
cargo clippy -p ferrite-gameplay --all-targets --all-features -- -D warnings
cargo ferrite task check
git diff --check
```

Focused result before the universal gate:

```text
25 MOB-001 spawning tests passed; 0 failed
6 source-specified slices
6 responsibility-owned runtime modules
```

The tests lock hostile rule/cache propagation and direct consumers; natural cap, player, potential,
position, pack, placement, construction and accounting boundaries; Patrol and Phantom pause/timer/
player/RNG/finalization behavior; Wandering Trader persistence, inclusive chance, meeting,
placement, llama and ignored-insertion quirks; and Warden attribution, tracker, delayed response,
search/finalization and Darkness rules.
