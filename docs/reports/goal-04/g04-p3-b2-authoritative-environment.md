# G04-P3-B2 — Authoritative Environment and Lighting

## Result

The formal world now advances light and environmental state through the same Region-owned chunk
and composite continuity path used by generation, collision, persistence, and projection. The
former projection-only full-sky placeholder is gone.

## Implemented boundary

- `ChunkColumn` owns validated sky and block-light layers. Generation installs deterministic direct
  sky light and bounded block-light propagation at `INITIALIZE_LIGHT`/`LIGHT`; emitting lava and
  fire seed the block-light frontier.
- `FWC3` persists exact light layers. `FWC1` and `FWC2` remain read-only inputs; an advanced legacy
  column without light is demoted to `FEATURES` so it resumes generation instead of synthesizing
  light or reaching projection.
- Authoritative block mutations recompute light before the next commit. Water, lava, and fire have
  stable internal identities, exact registry-report mappings, empty collision, and appropriate
  heightmap/light behavior.
- The Simulation stage registers loaded chunks, drains at most 64 due block ticks and 64 due fluid
  ticks per Region tick, and admits work only at `BlockTicking` activity. Audited Goal 01 fluid
  delays and fire schedule bounds drive deterministic water/lava spread and fire continuation.
  Random sampling uses the already durable position and gameplay RNG streams.
- `FWL2` durably stores game/day time, weather targets/timers, current and previous strengths, and
  weather RNG state. `P8L1` remains a read-only border-only migration input.
- The formal gateway advances and attaches the level record before each composite commit. Java
  26.2 clients receive the authoritative day-time clock and weather game events on join and tick;
  environment block mutations use the normal committed block projection.

The remaining `world/environment` manifest gaps are intentionally retained for the P3-B3
authoritative border/spawn ingress and projection plus exact-client acceptance in P5.

## Verification

- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --workspace --all-features`
- `cargo ferrite source verify`
- `cargo ferrite production verify`
- `git diff --check`

Focused coverage includes FWC1/FWC2 migration, FWC3 light round trips, generation light gates,
fluid mutation plus relight/projection, FWL2 restart continuity, formal network entry, and shutdown
capture.
