# G01-P8-S005 — WGEN-006 world border

## Result

Complete. `WGEN-BORDER-001` has a production owner and deterministic behavioral evidence in
`ferrite-world`.

The batch implements tick-counted static and moving extents, save/reconnect history reset,
dimension-scoped mutation projection, partial-tick geometry, containment and clamping, collision
walls and ray replacement, outside-border damage, HUD and force-field formulas, and command time
conversion.

## Evidence

Production owner:

- `ferrite-world::generation::border`;
- responsibility modules `state`, `geometry`, `collision`, `effects`, and `command`.

Committed test owner:

- `crates/ferrite-world/tests/slices/wgen_006.rs` and its responsibility-specific children.

Design contract:

- [Minecraft 26.2 world-border runtime](../../development/worldgen-border-runtime.md).

Validated commands:

```text
cargo test -p ferrite-world --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo fmt --all -- --check
cargo run -q -p mc-reference --bin mc-ref -- implementation-manifest verify
git diff --check
```

Focused result before the repository gates:

```text
23 WGEN-006 slice tests passed; 0 failed
D=1/2/20 ordering, freeze gates, save/reload, and reconnect history reset locked
minimum-inclusive/maximum-exclusive geometry and exact AABB epsilon locked
near/far collision, unclamped float-direction ray face, damage floors, HUD, and render branches locked
none/t=1, s=20, d=24,000 float multiplication and Java rounding locked
```

## Boundary disposition

The implementation preserves the unusual intermediate state where calculated size advances while
ordinary authoritative geometry reads the previous sample, followed by an immediate static-target
jump on completion. Saved and reconnect state restart from calculated size and discard that lag
history. Nonpositive direct durations, equal endpoint notifications, Java wrapping chunk origins,
NaN propagation, signed comparison order, and approximate-direction tie order remain explicit
rather than normalized away.

The source audit used the SHA-1-locked official 26.2 server jar
`823e2250d24b3ddac457a60c92a6a941943fcd6a`. Bytecode checks additionally fixed
`WorldBorder#applyInitialSettings`, `WorldBorder#getDistanceToBorder`, `Mth#clamp`,
`CollisionGetter#clipIncludingBorder`, `Direction#getApproximateNearest`, and the moving extent's
status/update behavior.

S005 supplies level mutations, snapshots, geometry, damage decisions, and presentation frames to
Phase 8 integration. Region ownership, durable journaling, packet fan-out, cross-system callers,
and topology conformance remain `G01-P8-B1` and `G01-P8-B2`.
