# G01-P7-S009 — MOB-004 AI and Anger

## Result

Complete. The two source-specified MOB-004 slices now have production owners for classic goal
arbitration, Brain memory/behavior/activity/sensing, path/navigation/control transitions, generic
and classic universal anger, and Piglin universal-memory targeting.

## Evidence

Production owner:

- `ferrite-gameplay::mob::runtime::mob_004::{selector,brain,navigation,controls,anger}`.

Committed test owner:

- `crates/ferrite-gameplay/tests/slices/mobs/mob_004.rs`.

Design contract:

- [MOB-004 AI and anger runtime](../../development/mob-ai-and-anger-runtime.md).

Validated commands:

```text
cargo test -p ferrite-gameplay --test slices mobs_mob_004
cargo clippy -p ferrite-gameplay --all-targets --all-features -- -D warnings
cargo ferrite task check
git diff --check
```

Focused result before the universal gate:

```text
17 MOB-004 AI/anger tests passed; 0 failed
2 source-specified slices
5 responsibility-owned runtime modules
```

The tests lock selector phase/priority/flag behavior; memory TTL and write behavior; inclusive
behaviors, activities, schedules, sensors and sight caching; path visit/length/reach/alternative,
recompute, waypoint, stuck and timeout boundaries; base controls; retained revenge events; classic
neutral matching/reset/registration/group facts; and Piglin target, TTL and precedence rules.
