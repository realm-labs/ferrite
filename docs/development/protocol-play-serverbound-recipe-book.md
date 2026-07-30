# Play Serverbound Recipe-Book Requests

`G01-P6-F010` implements all three packets in
`PROTO-PLAY-SERVERBOUND-RECIPE-BOOK-001` for Minecraft Java 26.2:

| ID | Identity | Fields |
|---:|---|---|
| 39 | `minecraft:place_recipe` | signed container/display VarInts and maximum-items boolean |
| 46 | `minecraft:recipe_book_change_settings` | strict book type and open/filtering booleans |
| 47 | `minecraft:recipe_book_seen_recipe` | signed display VarInt |

Book types are exactly crafting, furnace, blast furnace and smoker at ordinals `0..=3`. Other
ordinals fault. Container and display IDs retain the full signed domain, and booleans normalize
every nonzero byte to true. Malformed, overlong, truncated and residual forms fault.

## Display mapping and durable identity

The adapter rebuilds contiguous display IDs from enabled displays in recipe/display iteration
order. Each entry retains its complete display payload, namespaced parent recipe and optional
resolved placement. Disabled entries consume no ID. Negative, stale and out-of-range IDs resolve
to no mapping.

Display IDs are reload-local. The server recipe book stores only known and highlighted parent keys
plus four setting tuples. Multiple displays may map to the same parent.

## Placement admission

ID 39 resets idle before every semantic gate, then requires in source order:

1. nonspectator state and exact current container ID;
2. a still-valid current menu;
3. a current display mapping;
4. a known parent recipe;
5. a recipe-book-aware menu; and
6. well-formed placement information compatible with the current target grid.

Only invalid-menu and impossible-placement branches increment their diagnostic counters. Display
variant tabs are not authority; the mapped parent and current menu own admission. Crafting/furnace
placement is bracketed by begin/finish markers, including every early result.

## Placement mutation

Noncreative placement first proves every grid/result clear target can return to compatible or free
inventory space. Failure leaves all state unchanged and sends no ghost. Creative mode skips this
capacity proof but still cannot create missing ingredients.

Available ingredients aggregate player inventory and current input grid. If the recipe cannot be
crafted, the menu returns/clears targets, marks inventory changed and immediately emits the mapped
ghost display. This is the sole direct response.

Craftable placement:

- aborts an already matching grid when incrementing any input would exceed the lesser of biggest
  craftable count and its maximum stack;
- selects biggest craftable for maximum placement, minimum current input plus one for a matching
  grid, or one for a nonmatching grid;
- clamps to the smallest resolved holder maximum;
- clears targets, removes matching inventory inputs in ascending order and distributes the
  resolved shaped/shapeless/furnace slot map;
- marks inventory changed even if an inner removal guard stops a partial placement.

Successful/no-change placement sends no explicit packet. Ordinary container change detection owns
later slot/cursor/data convergence; a ghost response precedes those later deltas.

## Settings and highlights

ID 46 directly replaces only the decoded book type's open/filtering tuple. ID 47 resolves the
display ID and removes its shared parent key from server highlights. Neither resets idle or checks
menu, mode or loaded state, and neither receives an echo.

The client updates settings before sending. It removes only the exact local display highlight
before sending seen; the server then removes the parent highlight shared by all current displays.
Disconnected local changes produce no packet.

## Ownership

Raw display/container IDs, local UI settings/highlights and placement working state remain adapter
or menu-local. Namespaced parent knowledge/settings are normalized player state. Authoritative
inventory/grid mutation remains Region-owned, and direct ghost payloads reuse the clientbound
recipe display family.

## Evidence

`crates/ferrite-protocol/tests/c3/play_serverbound_recipe_book.rs` owns three goldens, codec bounds,
feature-filtered/reload-local mapping, every ordered admission gate, capacity/no-op/ghost and
single/increment/maximum/guard mutations, all settings, shared-parent highlights, local-first order
and end-to-end tokenless convergence.
