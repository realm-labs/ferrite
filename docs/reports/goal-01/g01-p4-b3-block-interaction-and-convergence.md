# G01-P4-B3 Block Interaction and Convergence Report

## Result

`G01-P4-B3` implements the fixed C2 block transaction spine: five serverbound interaction codecs,
three clientbound convergence codecs, connection-local cumulative prediction ACKs, strict
targeting, Region-routed placement and breaking, committed rejection correction, and deterministic
replication aggregation.

The ownership and ordering contract is recorded in
[Block Interaction and Convergence](../../development/block-interaction-and-convergence.md).

## Source-locked behavior

The batch preserves these reviewed boundaries:

- packed block and section positions use their exact signed bit fields;
- hands and block-hit directions are strict, while the player-action direction maps every byte by
  modulo six;
- block-hit floats and both booleans survive decoding and command normalization;
- reach is strict squared eye-to-unit-AABB distance below `(range + 1)^2`;
- reconstructed hit components use the strict `1.0000001` boundary and reject non-finite values
  semantically rather than in the codec;
- the client-loaded gate precedes sequence registration;
- use-on/use-in-air register before later action work, while destroy registers only after routing;
- the listener ACK accumulator is a tick-local maximum, resets after output, and may regress in a
  later interval;
- use-on corrections retain hit-then-adjacent order and committed voxel changes use the later
  replication stream;
- one committed section change projects as a single update and multiple changes aggregate into a
  section update using `x/z/y` relative packing.

## Automated evidence

Focused tests cover signed packed-coordinate boundaries, exceptional hit/look floats, all five
serverbound round trips, strict and modulo enum behavior, ACK/single/section clientbound round
trips, global state bounds, tick-local cumulative ACK reset/regression, strict reach and hit
boundaries, matching break start/stop, committed-only mutation visibility, two-position use-on
correction, and multi-change section aggregation.

The acceptance commands are:

```text
cargo test -p ferrite-gameplay
cargo test -p ferrite-protocol
cargo test -p ferrite-server-runtime
cargo ferrite task check
git diff --check
```

## Coverage boundary

This fixed batch does not advance generated gameplay-slice or protocol-family counters. Complete
block/item content dispatch, inventories, permissions, hardness/progress, loot, multi-position and
cross-Region placement, crack/event presentation, auxiliary actions, pick-block, and the deferred
client prediction rendering observation remain owned by their generated Phase 5/6 batches.

Local/Lattice playable trace equivalence and the unmodified-client C2 acceptance trace remain
`G01-P4-B4` and `G01-P4-B5`.
