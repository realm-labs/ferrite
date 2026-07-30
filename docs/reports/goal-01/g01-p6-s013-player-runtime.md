# G01-P6-S013 — PLY-001 Player Runtime

## Result

Complete. Five `SourceSpecified` player slices map to seven modular production owners and one
behavioral test owner. The partition closes ordinary input, collision/travel, movement validation,
automatic jump detection, and spectator chunk admission without absorbing the special-movement,
interaction, or block-breaking batches.

## Evidence

Production owner:

- `ferrite-gameplay::player::{auto_jump,collision,convergence,input,movement,spectator,state,travel}`.

Committed test owner:

- `crates/ferrite-gameplay/tests/slices/player/ply_001.rs`.

Validated commands:

```text
cargo test -p ferrite-gameplay --test slices player_ply_001 -- --nocapture
cargo clippy -p ferrite-gameplay --all-targets --all-features -- -D warnings
cargo ferrite task check
git diff --check
```

Focused result before the universal gate:

```text
8 PLY-001 slice tests passed; 0 failed
30 player module tests passed; 0 failed
5 source-specified slices
```

The implementation preserves Java-float input and jump arithmetic, source-ordered generic shape
clipping and step selection, packet-probe versus authoritative-position separation, strict
teleport resend/acknowledgement convergence, entity-before-block auto-jump traversal, and the
independence of spectator distance admission from client chunk visibility.
