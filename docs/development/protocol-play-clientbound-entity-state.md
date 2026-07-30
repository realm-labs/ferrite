# Play Clientbound Entity State

`G01-P7-F006` implements the five packets in
`PROTO-PLAY-CLIENTBOUND-ENTITY-STATE-001` for Minecraft Java 26.2:

| ID | Identity | Projection |
|---:|---|---|
| 99 | `minecraft:set_entity_data` | ordered metadata values and callbacks |
| 100 | `minecraft:set_entity_link` | delayed leash-holder relation |
| 102 | `minecraft:set_equipment` | ordered living equipment replacement |
| 107 | `minecraft:set_passengers` | complete sequential direct-passenger replacement |
| 131 | `minecraft:update_attributes` | living attribute snapshot replacement |

All packet entity numbers remain current-level lookup keys. Metadata serializers, accessor slots,
equipment ordinals, item/component registries, attribute holders and modifier operations remain
separate version-local wire domains.

## Metadata wire and schema

ID 99 reads slot bytes until the required `255` terminator. Slots `0..=254` carry a serializer
VarInt and that serializer's unframed value. The codec implements all 43 locked serializers:
primitives, components, item stacks, rotations/positions/directions, references and block states,
particles, villager data, optionals, poses and configured variants, source enums, vectors,
quaternions, resolvable profiles and humanoid arms. Unknown serializers, missing terminators,
invalid holders and strict nested values fail closed. Direction, pose, source-enum, weathering and
arm by-ID policies preserve their observed default/wrap/clamp behavior.

The reviewed accessor lock contains all 221 top-level declarations, exact slots and serializer IDs.
The build script verifies its newline-terminated SHA-1
`b489eec18fc1981ebfb7ac97c54a4485fe2f938a` and generates immutable Rust declarations into
`OUT_DIR`. Runtime schemas compose only the selected source class hierarchy. A collision,
missing/extra default or serializer mismatch fails schema installation, so same-number accessors
from unrelated branches cannot be treated as global metadata.

A missing target ignores a decoded list. A present target applies entries in wire order. Every
successful entry replaces its slot and records the accessor callback before the aggregate ordered
callback. Duplicate slots therefore callback repeatedly and retain the last value. Missing slots
and wrong serializers fault at the point encountered without rolling back earlier entries.

Server-side dirty packing scans ascending slots, clears dirty flags and refreshes the nondefault
pairing snapshot. Returning to a default is still emitted once and then disappears from later
pairing. Metadata updates target tracking players and self; empty dirty sets emit nothing.

## Attributes and equipment

ID 131 allows `0..=128` snapshots. Each resolves an attribute holder, raw IEEE base, generic
nonnegative modifier list, strict modifier identifier and by-ID operation. Operations outside
`0..=2` become add-value. A missing entity ignores the packet and a present nonliving target
faults. Missing attribute instances skip complete snapshots. Present instances sanitize base to
their declared range, clear all old modifiers and install transient modifiers in wire order;
colliding modifier identities fault while preserving already-installed entries.

Pairing returns every client-syncable attribute instance. Runtime publication drains only the typed
dirty set to tracking players and self after equipment effects have entered the authoritative
attribute map.

ID 102 requires at least one descriptor. Bit `0x80` continues the list and ordinals `0..=7` map to
mainhand, offhand, feet, legs, chest, head, body and saddle. Every stack preserves count and its
component patch. Nonpositive counts are empty; positive air normalizes to empty and canonical
encoding writes count zero. Missing and nonliving targets ignore entries. Living targets apply in
wire order, including repeated slots.

Pairing emits only nonempty stacks in ordinal order. Runtime comparison includes count, item and
patch; empty changes remain in update packets to clear slots. An exact hand swap emits event 55 and
removes both hand entries while retaining other simultaneous changes. Equipment updates target
tracking players, not implicit self.

## Passengers and leash

ID 107 validates a nonnegative passenger count against remaining bytes, then retains arbitrary
signed and duplicate IDs. A missing vehicle ignores the list. A present vehicle records prior
local-player carriage, ejects current passengers and processes new IDs sequentially. Missing,
duplicate, cyclic and rejected rides follow the ordered `startRiding` result rather than
pre-normalization; a successful ride detaches the passenger from its previous vehicle.

Encountering the local player clears the former-vehicle marker. On a new boat transition, boat yaw
becomes the player's current/old/head yaw and the dismount-key onboarding presentation occurs once.
Tracker broadcasts exclude players whose own membership changes; their successful riding/removal
path sends the full list directly. Rider start orders positioning, position challenge, living
vehicle effects and full passenger list.

ID 100 alone uses two fixed big-endian signed ints. A missing or nonleashable source ignores it.
Zero clears the holder; every nonzero destination is retained as a delayed ID and resolves only
when that entity later exists. Canonical leash mutation precedes optional broadcast, and pairing
emits a current nonnull holder after passenger state.

## Ownership and evidence

`ferrite-protocol::java_26_2::play::clientbound::entity_state` separates the generated accessor
registry, 43-value metadata model, packet records, strict codec, client replacement projection and
publication policy. Authoritative attributes, equipment effects, passenger/leash state and entity
lifecycle remain in gameplay owners; raw slots, serializers, registry IDs, ordinals and delayed
client resolution never cross that boundary.

`crates/ferrite-protocol/tests/c3/play_clientbound_entity_state.rs` owns all five goldens, the
43-serializer round trip, 221-row accessor composition, malformed boundaries, item normalization,
ordered metadata callbacks, dirty/default behavior, attribute/equipment replacement,
passenger/leash transitions and publication audiences/order.
