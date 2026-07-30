# G01-P6-F002 Play Clientbound Inventory and Progression Report

## Result

Ferrite implements and verifies all three packets in
`PROTO-PLAY-CLIENTBOUND-INVENTORY-PROGRESSION-001`. Durable map and advancement identities remain
normalized while raw registry IDs, map texture state, debug transactions, client tree nodes,
presentation effects, and flush bookkeeping remain bounded adapter state.

## Verified boundaries

- IDs 51, 123, and 130 have exact empty goldens and structured round trips.
- Map decoration registry failures, signed fields, rotation masking, optional names/lists, width
  sentinel, arbitrary patch dimensions, malformed arrays, and residual bytes are covered.
- Client map creation retains first scale/lock/dimension, replaces decorations before pixels,
  reproduces X-major prefix mutation and flat-index aliasing, and refreshes only after success.
- Map publication consumes exact dirty bounds and independently samples decorations on old-counter
  modulo-five opportunities.
- Tag-query NBT enforces nullable compound/default quota semantics; only the latest exact
  transaction invokes, success clears, and callback failure retains pending state.
- Advancement codecs preserve added duplicates, collapse removed duplicates, replace progress
  duplicates, enforce strict frame/item/component/identifier/NBT forms, and retain raw timestamps.
- Client advancement application covers reset, recursive removal, stale duplicate nodes,
  dependency retries, unresolved parents, retained progress/tab state, criterion normalization,
  repeated telemetry, and toast gates.
- Canonical publication covers descendant/ancestor visibility, first-empty-flush reset clearing,
  visible-only dirty progress, add/remove deltas, and parent-cycle failure.
- A complete advancement publisher-to-codec-to-client trace converges definition and progress while
  suppressing reset-time presentation.

## Evidence

- `crates/ferrite-protocol/tests/c3/play_clientbound_inventory_progression.rs`
- `docs/development/protocol-play-clientbound-inventory-progression.md`

The batch acceptance gate is `cargo ferrite task check` followed by `git diff --check`.
