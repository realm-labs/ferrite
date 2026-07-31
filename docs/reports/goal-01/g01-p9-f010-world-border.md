# G01-P9-F010 World-Border Report

## Result

Ferrite implements and verifies clientbound IDs 88 through 92 in
`PROTO-PLAY-CLIENTBOUND-WORLD-BORDER-001`. Normalized per-dimension border authority and saved
state remain authoritative; packet encodings, listener fan-out, receive order, and client motion
anchors remain adapter-local.

## Verified boundaries

- All five packet bodies and IDs are locked. Center, old/new size, and immediate size preserve raw
  IEEE bits; duration preserves the complete signed VarLong domain; warning time and distance
  preserve signed VarInts. Truncation, overlong variable integers, and residual bytes fail before
  mutation without adding finite, sign, or gameplay-range checks.
- Center replaces both coordinates while retaining extent and warnings. Immediate size replaces
  either a static or moving extent. Warning time and blocks replace independent signed fields and
  do not alter geometry.
- Lerp uses Java-double endpoint equality: equal endpoints select a static extent at the new value,
  while unequal endpoints install a new motion at handler-time client game time. Negative zero,
  NaN, infinity, zero/negative duration, and signed endpoints remain unnormalized.
- Every authoritative matching setter publishes even when its value equals current state. Delivery
  preserves player-list order and targets only players in the border's dimension; moving ticks,
  damage-per-block, and safe-zone callbacks intentionally publish no delta.
- IDs 89 and 90 replace the same extent, so receive order decides the winner. Center and warning
  deltas remain independent, and a later delta may overwrite its field after a complete ID-43
  snapshot. There is no sequence, revision, response, monotonicity check, or acknowledgement.

## Evidence

- `crates/ferrite-protocol/src/java_26_2/play/clientbound/world_border/`
- `crates/ferrite-protocol/tests/c3/play_clientbound_world_border.rs`

Focused validation:

```text
cargo test -p ferrite-protocol --test c3 play_clientbound_world_border --all-features
12 passed; 0 failed
cargo test -p ferrite-protocol --test c3 --all-features
277 passed; 0 failed
cargo clippy -p ferrite-protocol --all-targets --all-features -- -D warnings
```

The batch acceptance gate is `cargo ferrite task check` followed by `git diff --check`.
