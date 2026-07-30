# ITM-002 Container Runtime

`G01-P6-S005` implements the seven audited container slices primarily owned by `ITM-002`: generic
container transactions, Barrel, Chest, Ender Chest, Hopper, Dispenser and Dropper.

## Runtime boundary

The runtime is split by responsibility:

- `inventory` owns signed stack counts, slot policies, comparator projection, reservoir selection,
  the merge-then-empty quick-move primitive and rollback-safe one-item transfer;
- `menu_click`, `menu_layout` and `menu_sync` own the seven click variants, all 25 registered menu
  layouts, prediction mirrors, stale-click replay, 15-bit state IDs and delta/full correction;
- `container_lifecycle` owns cursor/transient-input disposition, inventory-menu snapshot transfer
  and server-validated non-click controls;
- `container_storage` owns pending-loot caller boundaries, open/recount state, Barrel storage,
  canonical right-first double Chests, trapped power, player-owned Ender storage, lid state and
  generic removal-drop RNG accounting;
- `hopper` owns cooldown normalization/persistence values, ordered push/pull, sided admission,
  one-item rollback, receiving-Hopper cadence and full/partial loose-item collection;
- `dispenser` owns retained four-tick triggers, nine-slot reservoir choice, all 80 explicit
  behavior entries, dynamic resolution precedence, optional-behavior sticky state, remainder
  insertion/ejection, wrapper event ordering and Dropper target/ejection selection.

Loot-table evaluation, concrete dispenser actions, player/entity drop construction, Region event
delivery, menu wire codecs, crafting outputs, piglin AI and client render/sound execution retain
their dedicated owners. The container runtime exposes the deterministic semantic decisions those
owners consume.

## Validation

`crates/ferrite-gameplay/tests/slices/items/itm_002.rs` verifies transfer order and rollback, all
25 menu layouts, all seven click inputs, stale/full synchronization, close/control behavior,
Barrel/Chest/Ender ownership, drop RNG budgets, Hopper cooldown and partial absorption, the exact
80-entry dispenser partition, dynamic precedence, sticky optional outcomes, remainder event pairs,
Dropper target behavior and retained trigger transitions.
