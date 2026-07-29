# G01-P4-F003 Play Clientbound Terrain Report

## Result

Ferrite implements and verifies all ten required packets in
`PROTO-PLAY-CLIENTBOUND-TERRAIN-001`. The existing terrain codec now has the client-observed
compound-NBT and light-mask boundaries, and the former chunk-only cache model is split into
bounded bundle, batch timing, terrain/light projection, and level-readiness responsibilities.

## Verified boundaries

- IDs 0, 11–13, 37, 45, 48, 94, 95, and 111 have exact default goldens and structured round trips.
- Bundles preserve order, allow an empty pair, hold at most 4,096 subpackets, and reject terminal
  packets while open.
- Batch timing preserves constructor and repeated-start behavior, positive-only sample updates,
  one-third/three-times clamping, weight saturation at 49, and feedback for nonpositive counts.
- Full chunks cover single/local/global block and biome palettes, dynamic biome registry bounds,
  heightmap fallback and repair, nullable compound block-entity tags, type matching, negative-count
  asymmetry, isolated trailing bytes, dimension section count, and malformed truncation.
- Light data enforces exact arrays only for in-range consumed bits, gives data precedence over
  empty, ignores high bits and surplus arrays, applies independently of cache presence, merges
  changed layers, and is fully removed on unload.
- Biome refresh replaces only present chunks while notifying every coordinate and dirtying its 3×3
  neighborhood.
- Cache radius, coordinate deltas, neighborhood coordinates, and deadlines preserve locked Java
  signed-overflow behavior; all retained collections have explicit bounds.
- Level readiness preserves waiting-for-server, strict timeout, compilation and player-state
  exemptions, zero/500-ms close delays, one-shot acknowledgement, and independence from batch
  finish.

## Evidence

- `crates/ferrite-protocol/tests/c2/play_clientbound_terrain.rs`
- `docs/development/protocol-play-clientbound-terrain.md`

The batch acceptance gate is `cargo ferrite task check` followed by `git diff --check`.
