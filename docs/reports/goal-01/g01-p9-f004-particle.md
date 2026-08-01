# G01-P9-F004 Particle Projection Report

## Result

Ferrite implements and verifies clientbound ID 47 in
`PROTO-PLAY-CLIENTBOUND-PARTICLE-001`. Normalized particle intent remains the authored input; raw
type IDs, flags, narrowed floats, user settings, RNG draws and engine objects remain adapter/client
presentation state.

## Verified boundaries

- The official zero-valued simple-particle body is locked exactly. Both booleans normalize nonzero
  bytes to true; position, spread, speed and signed count preserve their IEEE/integer domains.
- ID 47 delegates to the already audited strict 125-entry particle option dispatch shared with the
  explosion family. Every simple, block, item, dust, color, entity, vibration, trail and
  registry-aware form round-trips; unknown types, mismatched options, invalid mappings,
  truncation and residual data fail closed.
- Negative count performs no attempt or Gaussian draw. Zero count performs one exact-position
  attempt with float `speed*spread` products widened to double. Positive count consumes three
  position Gaussians followed by three velocity Gaussians for every attempted particle.
- Packet and the exact 32 type-owned overrides are ORed only after particle-setting calculation.
  Distance 32 is inclusive client-side; ALL always admits, DECREASED admits with probability 2/3,
  MINIMAL normally rejects, and MINIMAL plus always-show admits with probability 1/15.
- A missing provider is silent. A provider fault is logged and abandons remaining work while
  retaining prior attempts and RNG consumption; setting/distance rejection does not invoke the
  provider. No protocol response, retry or disconnect is introduced.
- Canonical publication builds one packet, preserves position doubles/count/flags/options, narrows
  spreads and speed once, and filters current-level viewers by strict block-center radius 32 or 512
  for packet override. Always-show has no server-audience effect; aggregate and targeted forms
  expose exact delivery results.
- The packet requires an installed Ready-for-Terrain Play projection and has no sequence,
  generation, acknowledgement or convergence state.

## Evidence

- `crates/ferrite-protocol/src/java_26_2/play/clientbound/particle/`
- `crates/ferrite-protocol/tests/c3/play_clientbound_particle.rs`

Focused validation:

```text
cargo test -p ferrite-protocol --test c3 play_clientbound_particle
8 passed; 0 failed
cargo test -p ferrite-protocol --test c3
212 passed; 0 failed
cargo test -p ferrite-protocol --test c1
68 passed; 0 failed
cargo clippy -p ferrite-protocol --all-targets --all-features -- -D warnings
```

The batch acceptance gate is `cargo ferrite task check` followed by `git diff --check`.
