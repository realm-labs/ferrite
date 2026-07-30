# G01-P6-S015 — Player Target Interaction

## Result

Complete. `PLY-TARGET-INTERACTION-001` maps to three responsibility-specific production modules and
one behavioral test owner. Concrete content callbacks remain separate while their shared target,
result, prediction, admission, and stack transactions are source-ordered.

## Evidence

Production owner:

- `ferrite-gameplay::player::interaction::{targeting,attack,use_action}`.

Committed test owner:

- `crates/ferrite-gameplay/tests/slices/player/ply_004.rs`.

Validated commands:

```text
cargo test -p ferrite-gameplay --test slices player_ply_004 -- --nocapture
cargo clippy -p ferrite-gameplay --all-targets --all-features -- -D warnings
cargo ferrite task check
git diff --check
```

Focused result before the universal gate:

```text
8 PLY-004 slice tests passed; 0 failed
5 interaction module tests passed; 0 failed
1 source-specified slice
```

The implementation preserves strict entity/block ties and ranges, main/offhand fallthrough,
block-fail asymmetry, the main-hand empty-interaction marker, client/server swing ownership,
cumulative prediction acknowledgement, server distance and geometry buffers, and object-sensitive
callback stack replacement.
