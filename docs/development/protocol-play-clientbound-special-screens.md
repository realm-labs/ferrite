# Play Clientbound Special-Screen Protocol

`G01-P6-F005` implements all three packets in
`PROTO-PLAY-CLIENTBOUND-SPECIAL-SCREENS-001` for Minecraft Java 26.2:

| ID | Identity | Fields |
|---:|---|---|
| 41 | `minecraft:mount_screen_open` | signed container/column VarInts and signed entity int |
| 58 | `minecraft:open_book` | strict interaction-hand ordinal VarInt |
| 60 | `minecraft:open_sign_editor` | packed block position and front/back boolean |

The hand mapping is exactly `0=main`, `1=off`; all other signed values fault. Mount identifiers and
columns remain unclamped at codec time. Sign positions reuse the common signed 26/12/26-bit layout,
and nonzero booleans decode true before canonical egress writes one.

## Mount activation

The client projection computes Java wrapping `columns * 3` and validates the resulting allocation
before consulting the tracked entity. Negative results fault; positive allocations above an
explicit adapter limit resource-fault even for a missing or wrong entity. This preserves source
ordering without allowing malformed packets to allocate unbounded memory.

After successful allocation, a tracked horse installs a horse menu with three rows of column cargo
slots. A tracked nautilus installs its specialized menu but ignores those allocated slots for cargo.
Missing and other entity types leave the current menu unchanged.

`MountPublisher` uses the ordinary container publisher's specialized opener: it closes an existing
menu, advances the shared `1..=100` container counter, emits ID 41 instead of `open_screen`, then
emits initial full content/properties. Subsequent convergence uses the ordinary container family.

## Book activation

ID 58 carries no stack snapshot. The handler reads the selected current-hand projection at
execution time. Written content takes precedence over writable content; filtered pages are selected
per page when local filtering is enabled, with raw fallback. A recognized component opens a
view-only projection, while an absent component leaves the current screen unchanged. Delayed
traffic can therefore observe a different hand stack.

Canonical publication is narrower: the server sends ID 58 only for written content. If component
resolution mutates the stack, ordinary menu changes precede the open-book packet. There is no
response or acknowledgement.

## Sign-editor activation

The client resolves a current sign block entity at the packed position, selects ordinary or hanging
screen subtype, and copies exactly four strings from the requested front/back side. Missing or
wrong block entities are semantic no-ops; the packet contains neither text nor sign type.

Canonical server admission rejects command-consumed interactions, waxed signs, a different active
editor, missing build permission, and selected-side messages that are neither empty nor plain.
Success stores the player's editor identity, emits the current block-state correction, then emits
ID 60. Earlier block-entity projection remains responsible for text. Submission and authorization
return through the separately owned serverbound sign-update family.

Raw entity/container IDs, allocation arithmetic, hand ordinals, GUI objects, current-hand timing,
and editor UUIDs remain version-local adapter state.

## Evidence

`crates/ferrite-protocol/tests/c3/play_clientbound_special_screens.rs` owns all three goldens,
signed/malformed codec bounds, allocation-before-entity behavior, mount subtype slots, delayed book
resolution, filtered page selection, sign subtype/side projection, canonical publication order,
and end-to-end decode-to-client activation.
