# G01-P9-S004 — client-observable effects and player rules

## Result

Complete. `CLI-OBSERVABLE-EFFECTS-001` now has an executable Java 26.2 shared-presentation model in
`behavior-runner`. This closes all four audited client gameplay slices and brings gameplay coverage
to 331/331, including 327/327 source-specified slices and the already verified source-known behavior
of all four source-inconclusive slices.

## Effect transport and presentation

The server-side model locks player-only exclusion, same-dimension and strict distance audiences,
fixed/volume-scaled sound range, tracking-plus-self delivery, ordinary radius-64 level events,
global near/far/cross-level position projection, and strict particle ranges of 32 or 512 blocks.
Server game events remain a separate listener-side effect and do not imply presentation packets.

The sound model preserves supplied seeds, suppresses missing entity-bound sounds, rejects unloaded,
disallowed, unknown, intentionally empty and empty resources, and models channel refusal. It clamps
pitch and audible gain while retaining original volume for attenuation, preserves the 20-sound-tick
channel lifetime floor, consumes a client `nextLong` only when no local seed is supplied, and applies
distance delay only above squared distance 100 at 40 blocks per second.

The particle model distinguishes count-zero exact position/velocity from the six-Gaussian positive
count loop, including exception logging and loop termination. It executes MINIMAL/DECREASED option
draws before limiter override, retains the inclusive client camera boundary at squared distance
1024, and proves that `alwaysShow` is probabilistic rather than unconditional.

Entity IDs 21, 35 and 63 retain their special guardian, totem and sniffer paths; other IDs reach the
entity handler. Missing entity and damage targets are ignored, the level-event packet bit selects
exactly one handler, and repeated local call-site effects are retained without synthetic
deduplication. Concrete gameplay leaves continue to own their mutation/effect ordering.

## Player-facing rules

Join snapshots preserve `reduced_debug_info` and invert only `immediate_respawn` into
`show_death_screen`; respawn replacement copies both flags. Live callbacks notify first, then emit
canonical immediate-respawn values or reduced-debug entity-event bytes. Client handling uses exact
zero comparison, preserves unrelated/missing entity events, and repeats death-screen or ordinary
respawn/toggle-reset behavior for every qualifying combat-kill packet. A cross-crate fixture joins
this behavior to the existing `ferrite-protocol` combat projection.

The locator-bar model rejects self and disabled connections, handles absent representations,
retains intact connections, re-evaluates broken ones, removes player/transmitter rows, disconnects
before clearing, and rebuilds only current per-level players on re-enable. Waypoint packet encoding
and representation thresholds remain assigned to `G01-P9-F001`.

## Evidence

Implementation owner:

- `behavior_runner::client::effects`.

Committed test owner:

- `apps/behavior-runner/tests/client/cli_006.rs`.

Focused validation:

```text
cargo test -p behavior-runner --test client cli_006
10 passed; 0 failed
cargo clippy -p behavior-runner -p ferrite-testkit --all-targets --all-features -- -D warnings
cargo fmt --all -- --check
git diff --check
```

Ordinary sound, particle, level-event and waypoint packet families remain in their fixed Phase 9
protocol batches; this batch claims their shared source-specified behavior, not premature wire
completion.
