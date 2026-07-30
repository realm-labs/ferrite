# Play Clientbound Container Convergence Protocol

Ferrite implements all seven packets in
`PROTO-PLAY-CLIENTBOUND-CONTAINER-CONVERGENCE-001` for Minecraft Java 26.2:

| ID | Identity | Projection |
|---:|---|---|
| 17 | `minecraft:container_close` | close the current client menu regardless of the carried ID |
| 18 | `minecraft:container_set_content` | replace a slot prefix, cursor, and state ID |
| 19 | `minecraft:container_set_data` | replace one signed-short indexed property |
| 20 | `minecraft:container_set_slot` | replace one slot and state ID |
| 59 | `minecraft:open_screen` | construct a strict registered menu with a trusted NBT title |
| 96 | `minecraft:set_cursor_item` | replace the current cursor outside the creative screen |
| 108 | `minecraft:set_player_inventory` | replace one ordinary or equipment inventory destination |

The main Play clientbound dispatcher resolves these identities through the locked packet catalog.
Container and state IDs retain their signed VarInt values. Slot, property, and property-value
fields retain signed big-endian short values. Menu types resolve through `minecraft:menu`; titles
use trusted component NBT and reject invalid component root shapes.

## Shared item codec

The Play-level item module owns the optional stack and component-patch grammar shared by containers
and recipe displays. A nonpositive count consumes no further fields and becomes empty. A positive
count resolves a strict item holder, then decodes added and removed component types through the
connection registries. Component values remain delegated to the version-locked typed decoder
because their payloads are not self-delimiting.

Present entries decode before removals; duplicate identities replace earlier values and a later
removal wins. The raw signed count sum preserves the locked wrapping-capacity fault, while a
negative individual count performs no loop when the combined capacity is nonnegative. Canonical
egress requires disjoint unique entries and positive counts; allocations remain packet-bounded.
Positive stack counts are not size-clamped. A positive `minecraft:air` form is fully consumed and
then normalized to empty; all empty, nonpositive, and AIR egress uses the one-VarInt zero form.

## Client application

The bounded client projection reproduces the observed handler edges:

- open replaces the current menu only when a client screen constructor exists; a missing screen
  reports the warning path without changing the current menu;
- close ignores its packet ID and returns to the inventory menu;
- container zero targets the persistent inventory for content and slot traffic, while nonzero IDs
  require an exact current-menu match;
- a full content list writes in order and may fault after an exact prefix; cursor and state install
  only after every transmitted slot succeeds;
- matched data and slot indices preserve signed widening and fault when invalid; received state IDs
  are stored verbatim;
- slot, cursor, and player-inventory packets invoke the tutorial observation before later faults or
  creative suppression;
- creative slot bookkeeping forces the inventory remote slot and broadcasts local changes even for
  an otherwise ignored container ID; creative cursor replacement is suppressed;
- player slots `0..=35`, `36..=39`, `40`, `41`, and `42` map to ordinary, feet-to-head armor,
  offhand, body, and saddle state. Negative values fault and values above 42 are ignored.

Inventory-menu hotbar growth from empty or a smaller count sets pop time five before the slot/state
write. Projection capacity is explicit, so malformed traffic cannot cause an unbounded semantic
model.

## Server publication

`ContainerPublisher` keeps connection-local container IDs, state IDs, and remote snapshots out of
authoritative Region state. Opening a menu closes an existing noninventory menu, cycles the
container counter through `1..=100`, emits open, increments and emits full content/cursor, emits
properties in ascending index order, and only then installs the current publication record.

A full send increments `(state + 1) & 32767`, emits content, then every property. Delta publication
scans slots in ascending order and increments once per emitted slot, then emits cursor and finally
changed properties in ascending order. Cursor and property packets do not increment state. Shape
changes and indices that cannot be represented by the locked signed-short fields fail before
publication.

## Evidence

`crates/ferrite-protocol/tests/c3/play_clientbound_container_convergence.rs` owns all seven golden
bodies, signed and structured codec bounds, AIR canonicalization, malformed input, menu mapping,
open/close races, partial client mutations, creative behavior, inventory destinations, canonical
publication order, ID/state wrap, and encode/decode-to-client convergence.
