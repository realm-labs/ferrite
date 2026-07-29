# BLK-001 Block Runtime

`G01-P5-S001` installs the first audited gameplay behavior partition. It covers the 41
`SourceSpecified` slices whose primary owner is `BLK-001`.

## Runtime boundary

Ferrite lowers the ignored, locally imported `minecraft:block` registry into a project-owned
`MinecraftBlockCatalog`. Lowering rejects a missing or repeated default state, incomplete
cartesian state schemas, duplicate raw IDs, duplicate canonical tuples, malformed property values,
and non-block registries. Runtime state identity remains the persistent block ID plus a
schema-local index; the Java raw state ID is adapter lookup data.

Direct state mutation is strict. A missing property or illegal value fails. The
`minecraft:block_state` component path is intentionally lenient: it applies valid entries in
deterministic key order and skips unknown names and unparseable values.

`ferrite-gameplay::block::runtime` separates five responsibilities:

- `catalog` closes the 40 audited block behavior families and their exact block counts;
- `geometry` owns axis transforms, banner rotation, dye/beacon colors, and exceptional physical
  profiles;
- `contact` owns slime/honey movement, sticky pairing, magma admission, and lava-cauldron
  dispatch/effect order;
- `storage` owns banner layer/map limits, shelf selection/chains/swaps/comparator output, and
  decorated-pot faces/insertion/comparator/wobble;
- `operator` owns jigsaw orientation/defaults and structure-block edit, scan, and redstone-edge
  ordering.

No official report, asset, jar, or generated entry is committed. `cargo ferrite content verify`
revalidates the ignored content bundle and now also lowers all block definitions and states.

## Cross-owner boundary

This batch implements the block-owned inputs and transitions named by the 41 leaf rules. Generic
placement/breaking, item allocation, loot evaluation, redstone propagation, entity damage,
world-generation placement, rendering-resource admission, and protocol projection stay with their
explicit later owners. Their eventual integration cannot invent or replace these block-owned
profiles; Phase 5 and later closure batches exercise the joins.

## Validation

The batch test owner is
`crates/ferrite-gameplay/tests/slices/blocks/blk_001.rs`. It checks:

- exact ownership for all 41 slice IDs;
- all 1,196 locally imported definitions and 32,366 contiguous canonical raw states;
- 40 owned families, 178 block IDs, and 1,309 states;
- strict versus lenient state transitions and malformed report refusal;
- the locked orientation, color, physical, contact, cauldron, banner, shelf, pot, jigsaw, and
  structure-block boundaries.

The local-artifact matrix is conditional only on the legal ignored bundle being present. The
separate `cargo ferrite content verify` command is the authoritative artifact-presence and digest
gate.
