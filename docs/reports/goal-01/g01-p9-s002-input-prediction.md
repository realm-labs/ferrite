# G01-P9-S002 — client input and prediction

## Result

Complete. `CLI-INPUT-PREDICTION-001` now has an executable client-front-end model in
`behavior-runner`, joined to the existing Ferrite block-prediction, movement-message, and correction
implementations by its assigned test owner.

## Observable behavior

The input model locks keyboard and mouse press/repeat/release flow through window identity, screen
consumption, overlays, debug consumption, focus release, and physical-key resampling. Hold mappings
follow physical state. Toggle mappings invert only on a down transition, ignore release, remember a
screen-forced release, optionally restore it after screen close, and are not overwritten by focus
resampling while toggle policy is active. Click counts are queued, consumed one at a time, and
cleared on focus/screen release.

Mouse motion retains absolute callback positions, ignores the first grabbed move, accumulates only
while the window is active, and is consumed once per render frame. Render frames update partial
time without advancing client-tick gameplay cooldowns. Client ticks drain attack, use, and pick
edges in source order, implement the using-item drain/release branch, then run held use and the
continue-attack gate.

The integration tests additionally lock:

- two predictions at one block position retain the original authoritative state while advancing
  the position's sequence, stage a reordered server update, ignore an older cumulative ACK, and
  restore the staged state on the covering ACK;
- position-only, rotation-only, position+rotation, and status-only movement forms plus the exact
  twentieth-tick position heartbeat;
- absolute/relative correction of current and old pose followed by teleport acknowledgement,
  immediate position+rotation echo, and the prediction barrier in that order.

The acknowledged-before-later-block-update rendered transient remains outside this claim and stays
explicitly owned by the existing `PLY-BLOCK-BREAK-001` deferred observation.

## Evidence

Implementation owner:

- `behavior_runner::client::input_prediction`;
- existing `ferrite_gameplay::player::{breaking::prediction,convergence,input}` behavior;
- existing `ferrite_protocol::java_26_2::play::clientbound::block` projection.

Committed test owner:

- `apps/behavior-runner/tests/client/cli_001.rs`.

Focused validation:

```text
cargo test -p behavior-runner --test client --all-features
13 passed; 0 failed
cargo fmt --all -- --check
git diff --check
```

Phase 9 client projection and cross-system integration remain owned by `G01-P9-B1`.
