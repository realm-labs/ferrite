# G01-P4-F001 Play Clientbound Block Report

## Result

Ferrite implements and verifies all six required packets in
`PROTO-PLAY-CLIENTBOUND-BLOCK-001`. The version adapter owns wire IDs, packed coordinates, raw
registries, prediction records, crack progress, and presentation events; authoritative world state
continues to use namespaced registry projection and committed Region snapshots.

## Verified boundaries

- IDs 4, 5, 6, 7, 8, and 84 have exact golden bodies and round trips.
- Block and section signed-coordinate extrema and all three locked registry maxima pass.
- Standalone block-entity data rejects null/non-compound tags and accepts a compound larger than
  the default 2,097,152-byte NBT quota through the locked trusted quota.
- ID 8 rejects unknown states during decode; ID 84 retains nullable lookup results until the
  source-observed immediate or ACK-time state write.
- Prediction state retains first-capture data, stages later authority, advances same-position
  sequences, follows locked fastutil removal order, and suppresses rollback after teleport.
- Section duplicates apply in wire order with `x/z/y` relative packing.
- Block-entity cache misses and type mismatches are ignored; exact matches load the tag.
- Block events use the current local block, and destruction records relocate, remove, retain at
  age 400, and expire only on the scheduled scan after age 400.

## Evidence

- `crates/ferrite-protocol/tests/c2/play_clientbound_block.rs`
- `docs/development/protocol-play-clientbound-block.md`
- `docs/development/block-interaction-and-convergence.md`

The batch acceptance gate is `cargo ferrite task check` followed by `git diff --check`.
