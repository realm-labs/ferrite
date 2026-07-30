# Play Serverbound Inventory Auxiliary Requests

`G01-P6-F008` implements the three independent packets in
`PROTO-PLAY-SERVERBOUND-INVENTORY-AUXILIARY-001` for Minecraft Java 26.2:

| ID | Identity | Fields |
|---:|---|---|
| 3 | `minecraft:bundle_item_selected` | signed menu-slot and selected-content VarInts |
| 24 | `minecraft:edit_book` | inventory slot, at most 100 UTF(1,024) pages, optional UTF(32) title |
| 50 | `minecraft:seen_advancements` | strict action and OPENED_TAB-only identifier |

Bundle selection accepts every signed slot but rejects selected indices below `-1`. Positive
indices have no transport cap. Book page count, UTF-16-unit and encoded-byte bounds are enforced
before semantic handling; malformed UTF-8 uses replacement decoding. Advancement actions are
strictly `0=OPENED_TAB` and `1=CLOSED_SCREEN`; opened identifiers use the common default bound and
grammar. Truncation and residual data fault all three packets.

## Bundle prediction and transient selection

The client mutates its local bundle component before sending ID 3. Scroll addresses only the
displayed subset: all entries through 12, then `8..=11` entries according to the padded four-column
grid. Hover exit, quick move and swap may send a redundant `-1` clear.

The server resolves the slot against the handler-time current menu. It has no container/state ID
and no still-valid, spectator, loaded-player, idle, item-tag or displayed-count gate. Invalid slots
and stacks without bundle contents are no-ops. A valid full-list index toggles or replaces the
selection; repeating the selected index toggles it off, and `-1` or an out-of-list index clears it.
Crafted clients may therefore select hidden existing entries.

Selection is excluded from component equality and stream reconstruction. Removing an entry uses a
valid selection or index zero as fallback and then clears the selection. The mutation has neither
an acknowledgement nor an ordinary slot delta of its own.

## Independent book filtering completions

Only hotbar slots `0..=8` and offhand slot `40` are admitted before filtering. Admission does not
read or capture the stack. Each request creates an independent filter task; there is no arrival
queue, request ID on the wire, stack revision or container state.

At completion, the server:

1. drops failed or disconnected tasks;
2. re-reads the packet slot;
3. requires only a current writable-book-content component;
4. converts filter results according to the callback-time filtering preference;
5. updates writable pages or finalizes a written book;
6. relies on ordinary inventory projection for convergence.

Completion order wins. A later request may finish first, an edit can be overwritten by an earlier
completion, slot replacement redirects the mutation, and written-book finalization makes later
callbacks no-op until writable content returns.

An absent title replaces the entire writable page list. A present transport-valid title always
finalizes: item identity becomes `minecraft:written_book`, writable content is removed, other
components are preserved, and written content receives filtered literal title/pages, the player
name, generation zero and `resolved=true`. The server does not repeat client blank, trim or
15-unit checks.

The client Done path removes trailing exactly-empty pages and sends no title. Finalize keeps the
current pages, trims the title and sends it as present. Escape and sign cancel send nothing.

## Advancement presentation cursor

OPENED_TAB looks up the identifier in the current advancement definitions. Unknown identifiers
retain the old cursor. Known children and roots without display normalize it to null; only a
displayed root remains selected. A clientbound selection correction is requested only when this
normalized identity changes.

CLOSED_SCREEN deliberately does nothing, so the server cursor survives screen close. Reload clears
the cursor. Client screen initialization and root clicks send OPENED_TAB before local selection
notification, including identity-equal reopens. Screen removal sends CLOSED_SCREEN while connected;
applying an authoritative correction never echoes it.

## Ownership

Packet IDs, raw signed fields, UTF limits, transient bundle prediction, book filter tasks and the
advancement presentation cursor are protocol/session-local. Authoritative menu contents, inventory
components, advancement definitions and ordinary clientbound projection remain with their
gameplay and Region owners. These three requests share no acknowledgement or persistence domain.

## Evidence

`crates/ferrite-protocol/tests/c3/play_serverbound_inventory_auxiliary.rs` owns four goldens, codec
bounds, visible/hidden bundle selection and reconstruction, callback-time book admission,
out-of-order filtering and finalization, advancement normalization and close retention, client
ordering and end-to-end tokenless dispatch.
