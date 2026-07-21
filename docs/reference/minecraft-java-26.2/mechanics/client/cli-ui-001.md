# Client-observable behavior mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `CLI-UI-001` — Container gestures predict one semantic click stream; server packets overwrite it

**Parent:** `CLI-005`, `ITM-002`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — mouse/keyboard gesture mapping, double-click and drag state, predicted packet
construction, serverbound dedicated controls, clientbound overwrite rules and all four dialog input
controls are explicit in locked source. Server click replay and menu-specific controls remain owned
by the completed `ITM-CONTAINER-*` leaves.

**Applies when:**

An `AbstractContainerScreen` receives a mouse or key gesture, game mode predicts a menu click, the
server updates/closes the menu, or a data-driven dialog constructs/submits an input control.

**Authoritative state:**

Hovered active slot, screen rectangle, carried stack, last clicked slot/time/button, last
quick-moved stack, quick-craft active/button/type/slot set/remainder and skip-release bits; player
menu/container/state ID; server slots/carried/data; decoded input-control record and widget value.

**Transition and ordering:**

**Mouse press and release mapping:**

The first active slot whose expanded hover box contains the pointer wins; each box spans
`[x-1,x+17)` and `[y-1,y+17)`. Outside the screen rectangle maps to slot `-999`; inside with no slot
is `-1`. A click is double only when the mouse handler reports same-screen/same-button elapsed
strictly below `250 ms` and this screen's previous slot is the same slot.

After superclass handling, left/right or creative pick presses enter the container machine. With
empty carried stack and no active quick craft, creative pick sends `CLONE`; Shift on a real slot
sends `QUICK_MOVE` and snapshots its stack; outside sends `THROW`; otherwise it sends `PICKUP`.
That press sets `skipNextRelease`. With a carried stack, the press instead starts quick craft,
records the button, clears the slot set and selects type `0` for left, `1` for right, `2` for
creative pick. Other mouse buttons only test offhand (`button 40`) then hotbar swaps (`0..8`) when a
slot is hovered and carried is empty.

Drag adds a slot at most once only while quick crafting, carried is nonempty, count exceeds selected
slot count unless type `2`, and menu/slot replacement, placement and drag gates pass. It recomputes
the preview remainder after each addition. On release, double-left over an eligible slot wins:
Shift iterates menu slots and sends `QUICK_MOVE` for matching, pickup-allowed slots in the same
container; otherwise it sends one `PICKUP_ALL`. A mismatched quick-craft release cancels and clears
the set. `skipNextRelease` then consumes exactly one release. A nonempty quick set sends
`QUICK_CRAFT` start at `-999`, one add for every set member, then end at `-999`, using
`getQuickcraftMask(phase,type)`. Otherwise a carried stack sends clone, Shift quick-move or pickup.
The ordinary tail clears quick-craft active; closing/removal abandons client gesture state with the
screen instance.

**Keyboard mapping:**

Superclass keys win first. Inventory closes the menu. With an item in the hovered slot, pick sends
`CLONE` button `0`; drop sends `THROW`, button `1` with Control and `0` otherwise. Independently,
when carried is empty and a slot is hovered, offhand sends `SWAP/40`; otherwise the first matching
hotbar key in `0..8` sends `SWAP/index`. This screen contains no separate touchscreen inventory
algorithm in 26.2; pointer gestures enter the same mouse-button state machine.

**Prediction and packet construction:**

`slotClicked` normalizes a nonnull slot to its runtime index, notifies matching client slot-action
hooks, then calls game mode. A mismatched container ID is logged and produces no mutation/packet.
Otherwise game mode copies every slot, invokes the same menu `clicked(slot,button,input,player)`
locally, compares each slot with `ItemStack.matches`, hashes only changed after-stacks, hashes the
resulting carried stack, and sends `ServerboundContainerClick` with container ID, the menu's current
state ID, checked signed-short slot, checked signed-byte button, `ContainerInput`,
changed map and carried hash. Overflow of either checked narrowing throws rather than truncating.

The server transaction, 15-bit state-ID arithmetic, validation, remote mirrors, stale-state full
resync and exact `ContainerInput` semantics are `ITM-CONTAINER-CLICK-001`. Recipe placement,
container buttons, slot-state change, rename, beacon, trade, bundle/item actions and close use their
dedicated packets and completed control/close owners; they are never encoded as synthetic clicks.

**Clientbound overwrite:**

All handlers first transfer to the client packet processor. Slot update container `0` targets the
inventory menu and gives a growing nonempty hotbar stack pop time `5`; otherwise only the currently
open matching container is written. A matching content packet atomically initializes state ID,
all items and carried stack. Cursor and player-inventory packets update those dedicated locations;
data writes only a matching open container. Creative-screen remote-slot mirroring and broadcast run
after slot handling as declared. Server close calls `clientSideCloseContainer` regardless of the
screen's in-progress drag. Wrong nonzero container IDs are ignored, so delayed packets cannot write
a newly opened different menu.

**Dialog input controls:**

The registry is a four-entry codec-to-handler dispatch. `boolean` defaults `initial=false`,
`on_true="true"`, `on_false="false"`; its checkbox submits a byte tag and the selected configured
string. `number_range` defaults width `200` and label format `options.generic_value`; start/end may
descend, initial must lie within their unordered bounds, and positive optional step rounds relative
to the initial value (or midpoint), backing off one step if the rounded result lies outside. Equal
endpoints map to slider `0.5`; submission uses an integer string only when float-to-int round trip is
exact and a float tag otherwise.

`single_option` requires a nonempty encoded-order list and at most one initial; otherwise the first
entry is initial. It cycles entries, displays configured text or literal ID, and submits the ID.
`text` defaults width `200`, visible label, empty initial and max length `32`; max length is positive
and initial length may not exceed it. Single line has height `20`. Multiline accepts positive
`max_lines` and height `1..512`; absent height is `min(9*(max_lines or 4)+8,512)`. Text submission is
a string tag and uses escaped-without-quotes template substitution. An unrecognized control logs
and adds no widget/value getter.

**Branches and aborts:**

Superclass consumes; inactive/no hovered slot; outside `-999`; unsupported button; noncreative
clone; carried empty/nonempty; Shift/control; double eligible/ineligible; quick slot gate,
duplicate, mismatched release or skipped release; mismatched menu ID; narrowing overflow; delayed
wrong-container packet; invalid control decode or unregistered handler.

**Constants and randomness:**

Double click `<250 ms`; hover expansion one pixel; outside `-999`; offhand `40`; hotbar `0..8`;
quick types `0/1/2` and phases `0/1/2`; hotbar pop `5`; widget widths default `200`, control height
`20`, text max `32`, multiline default four lines and maximum height `512`. No RNG.

**Side effects:**

Gesture flags/previews, local menu mutation, slot-action hooks, semantic request packets, server
menu mutation/resync, slot/carried/data presentation, menu close, widget layout/value tags and
action template substitutions.

**Gates:**

Screen/superclass, active hover geometry, player creative/modifiers/bindings, carried stack and
slot/menu policies, quick-craft state, container identity/state ID, packet identity, control codec
validation and handler registration.

**Boundary cases and quirks:**

Empty-carried actions commit on press and deliberately suppress release; carried-stack actions
normally commit on release. Double Shift may emit several quick-move clicks in menu order. Quick
craft is exactly start/add/end, and a wrong release button cancels it. Client changed hashes describe
its prediction but never authorize server stacks. Delayed packets are identity-gated rather than
replayed into another menu.

**Evidence:**

`OFF-CLIENT-001`; `OFF-SERVER-001`; `MouseHandler#onButton`;
`AbstractContainerScreen#mouseClicked`, `#mouseDragged`, `#mouseReleased`, `#keyPressed`,
`#quickCraftToSlots`, `#slotClicked`; `MultiPlayerGameMode#handleContainerInput`;
`ClientPacketListener#handleContainerSetSlot`, `#handleContainerContent`,
`#handleContainerSetData`, `#handleContainerClose`; `InputControlTypes`; `InputControlHandlers`;
`BooleanInput`, `NumberRangeInput`, `SingleOptionInput`, `TextInput`; `ITM-CONTAINER-*`;
`EXP-CLI-002`.

**Test vectors:**

All inside/outside/no-slot press/release combinations; `249/250 ms`, same/different slot, screen and
button; empty/carried with Shift/Control/pick/hotbar/offhand; quick-craft zero/one/many slots,
duplicates and wrong-button cancellation; menu ID/state delay and close during drag; every control
default, invalid codec bound, descending/equal range, step ties/endpoints, missing/multiple initial,
single/multiline length/height/line limits and exact submitted tag/string.
