# G01-P9-F011 World-Effect Report

## Result

Ferrite implements and verifies clientbound ID 46 `minecraft:level_event` in
`PROTO-PLAY-CLIENTBOUND-WORLD-EFFECT-001`. Authoritative producers use normalized namespaced
effects; the 26.2 adapter alone maps them to packet integers, local/global scope, and packed
positions.

## Verified boundaries

- The fixed body preserves the complete signed event/data domains, signed 26/12/26 block-position
  packing, and nonzero-Boolean normalization. Truncation and residual bytes fault before handling.
- The boolean selects disjoint tables: all 80 local identities are audited and reversible at the
  adapter boundary, the global table contains only 1023, 1028, and 1038, and unknown or wrong-table
  IDs are no-ops without fallback.
- Owned data semantics cover extinguish selection, dynamic jukebox-song lookup, composter success,
  bone-meal/growth counts, signed directional smoke, block-state air fallback, potion color,
  dragon-breath sound selection, smash strength, electric-spark axes, sculk count/mask decoding,
  trial/vault flames, signed detection-loop arithmetic, and ominous activation volume.
- Trial flame data two retains the sound prefix for events 3012 and 3021 before the modeled handler
  fault, while 3011 faults without a prefix. Negative values and values above two select ordinary
  flame, and data one selects soul-fire flame.
- Ordinary publication preserves player-list order, excludes only the exact player source, requires
  the same dimension, and uses strict distance below 64 blocks from the integer packet position.
  Global publication either falls back to that ordinary packet or visits every connected player,
  substituting the actual, projected 32-block, or player block position for near, far, and
  cross-dimension recipients respectively.
- Requests remain tokenless and ordered. The adapter adds no acknowledgement, retry, generation,
  convergence, or cross-authority mutation, and client render/cache/RNG state stays nonauthoritative.

## Evidence

- `crates/ferrite-protocol/src/java_26_2/play/clientbound/world_effect/`
- `crates/ferrite-protocol/tests/c3/play_clientbound_world_effect.rs`
- `docs/reports/goal-01/g01-p9-s004-observable-effects.md`

Focused validation:

```text
cargo test -p ferrite-protocol --test c3 play_clientbound_world_effect --all-features
12 passed; 0 failed
cargo test -p ferrite-protocol --test c3 --all-features
289 passed; 0 failed
cargo clippy -p ferrite-protocol --all-targets --all-features -- -D warnings
cargo ferrite source verify
```

The batch acceptance gate is `cargo ferrite task check` followed by `git diff --check`.
