# G01-P6-S016 — Player Block Breaking

## Result

Complete for the source-known surface. `PLY-BLOCK-BREAK-001` now has production owners for client
input, progress, predicted local mutation, and acknowledgement convergence. The independent
rendered-frame question remains `DeferredExperiment` under `EXP-PLY-003`.

## Evidence

Production owners:

- `ferrite-gameplay::player::breaking::{input,session,mutation,prediction}`;
- `ferrite-gameplay::block::breaking` for the already-verified authoritative transaction;
- `ferrite-protocol::java_26_2::play::clientbound::block` for wire projection and exact fastutil
  multi-position release order.

Committed test owner:

- `crates/ferrite-gameplay/tests/slices/player/ply_006.rs`.

Validated commands:

```text
cargo test -p ferrite-gameplay --test slices player_ply_006 -- --nocapture
cargo test -p ferrite-gameplay player::breaking -- --nocapture
cargo clippy -p ferrite-gameplay --all-targets --all-features -- -D warnings
cargo ferrite task check
git diff --check
```

Focused result before the universal gate:

```text
8 PLY-006 slice tests passed; 0 failed
1 breaking module test passed; 0 failed
1 source-inconclusive slice source surface implemented
1 rendered-frame observation remains deferred
```

The tests lock same-tick click suppression, stop asymmetry, target/tool replacement, creative and
survival prediction order, delay-before-validation, Java float and NaN behavior, local flags-11
mutation, cumulative ACK, teleport fencing, flags-19 restoration, and collision snap. No test or
documentation promotes ACK-before-update packet order into an unobserved frame guarantee.
