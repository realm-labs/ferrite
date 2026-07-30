# Play Serverbound Container Convergence

`G01-P6-F007` implements all five packets in
`PROTO-PLAY-SERVERBOUND-CONTAINER-CONVERGENCE-001` for Minecraft Java 26.2:

| ID | Identity | Fields |
|---:|---|---|
| 17 | `minecraft:container_button_click` | signed container and button VarInts |
| 18 | `minecraft:container_click` | container/state VarInts, signed slot short/button byte, input, changed hashes and cursor hash |
| 19 | `minecraft:container_close` | signed container VarInt |
| 20 | `minecraft:container_slot_state_changed` | slot/container VarInts and boolean |
| 53 | `minecraft:set_carried_item` | signed selected-slot short |

Input values `0..=6` map to pickup, quick move, swap, clone, throw, quick craft and pickup all.
Every other signed VarInt deliberately falls back to pickup. Click decoding requires the configured
1,537-item and 111-data-component registries. Changed-slot maps are bounded at 128; added and removed
component collections are independently bounded at 256. Duplicate slot/component keys replace or
collapse according to their map/set ownership. Negative/oversized collections, unknown holders,
truncation, malformed VarInts and residual bytes fault.

## Hashed prediction evidence

A present hash carries exact item identity, signed count, a component-to-CRC32C map and a removed
component set. Component values are hashed from their registry-aware encoded payload; the
connection-local loading cache is bounded to 256 complete typed values. Stack matching also
requires exact item, count, added-map cardinality and removed set. The comparison remains 32 bit,
so equal CRC32C evidence—including a collision—matches.

AIR and every other semantically empty authoritative stack generate the false/empty hash. A forged
wire present form still decodes completely and simply cannot match an authoritative empty stack.
Receiving a prediction clears the exact remote snapshot and installs only the hash. A later match
promotes a copy of authoritative state; mismatch emits correction and replaces the snapshot. No
hash ever constructs or mutates authoritative item state.

## Click prediction and server transaction

The client transaction first requires the exact current container ID, checked-casts slot and button,
copies all slots, executes the existing gameplay click operation locally, then hashes only
semantically changed slots and the post-click cursor. Reordering equal component maps is not a
change. The packet uses the current server-projected state ID after local prediction.

The server resets idle before checking container identity. Wrong container, invalid menu and slots
at or above menu size are ignored. Spectator or dead/dying players receive a full snapshot without
execution. The outer slot helper accepts `-1`, `-999` and every value below slot count, including
other negative shorts; the selected gameplay click branch owns those values.

An admitted transaction suppresses remote publication, calls the existing seven-input gameplay
executor, installs only in-range prediction hashes and always installs the cursor hash. The protocol
module deliberately does not duplicate the item/menu mutation state machine from
`ferrite-gameplay`.

- A stale packet state still executes, then increments state and emits full content/cursor followed
  by every data value.
- A matching state compares slot order, then cursor, then data. Matching prediction hashes suppress
  packets; mismatches emit authoritative slot/cursor corrections and data deltas.

This is a tokenless convergence protocol. State ID is not an acknowledgement and no click sequence
exists.

## Independent controls

Button clicks reset idle, require current container, nonspectator state and `stillValid`, then
broadcast slots, cursor and data only if the concrete menu accepts the button.

Crafter slot-state requests do not reset idle or check validity. They require nonspectator state,
the exact current container, a real Crafter block entity, an empty slot `0..=8`, and an actual state
change. Enabled stores zero, disabled stores one, and success dirties the block entity.

Close ignores its wire container ID, transfers matching shared remote slot/hash state to the
inventory menu, removes whichever menu is current at handling time, selects the inventory menu and
sends no response. A delayed old close can therefore close a newer menu.

Carried selection accepts only `0..=8`. Invalid slots do not reset idle. Every valid request resets
idle; changing selection stops active main-hand use before installing the slot and marks equipment
projection dirty. Repeating the current slot is accepted without stopping use.

## Ownership

Packet/container/state/raw registry IDs, signed widths, hashes, component-hash cache and remote
snapshots remain connection-local 26.2 adapter state. Gameplay owns the seven click operations,
menu button implementations, close disposition and authoritative inventory. Region projection owns
committed state and ordinary clientbound convergence.

## Evidence

`crates/ferrite-protocol/tests/c3/play_serverbound_container_convergence.rs` owns five goldens,
malformed and collection bounds, CRC32C/cache/shape matching, client prediction, every click gate,
stale/full and matching/delta order, button/Crafter controls, delayed close, carried selection and
end-to-end prediction suppression.
