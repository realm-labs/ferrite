# Required Play Clientbound Block Protocol

Ferrite implements all six packets in
`PROTO-PLAY-CLIENTBOUND-BLOCK-001` for Minecraft Java 26.2:

| ID | Identity | Ferrite responsibility |
|---:|---|---|
| 4 | `minecraft:block_changed_ack` | release retained predictions through a signed sequence |
| 5 | `minecraft:block_destruction` | project breaker-keyed crack progress |
| 6 | `minecraft:block_entity_data` | update an existing exactly typed block entity |
| 7 | `minecraft:block_event` | project an already successful server event |
| 8 | `minecraft:block_update` | apply one authoritative state |
| 84 | `minecraft:section_blocks_update` | apply an ordered section-local state list |

Packet dispatch uses the locked Play clientbound catalog. Packed block positions retain signed
26/12/26-bit coordinates. Packed section positions retain signed 22/20/22-bit coordinates, and
section changes retain the source `x/z/y` nibble layout. Encoders accept only the 32,366-state,
49-type, and 1,196-block locked registry ranges.

## Nullable section states and NBT

ID 8 performs a strict state lookup. ID 84 preserves the source client's nullable lookup:
an unknown packed state decodes as `None`, can be staged behind a retained prediction, and faults
only when it reaches an immediate or ACK-time state write. Server encoding rejects `None`, so
Ferrite cannot emit a noncanonical state.

Standalone ID-6 data requires a non-null compound root. It uses the trusted NBT quota rather than
the full-chunk default quota, while retaining the 512-depth and packet-frame limits. The client
projection loads the tag only when an entity already exists at the position and its runtime type
matches. A miss or mismatch is ignored and never creates or retags an entity.

## Prediction projection

The bounded conformance projection retains the original server state and captured player position
on the first prediction at a coordinate. Later predictions at that coordinate update only the
latest sequence. Authoritative ID-8/84 states stage without replacing the visible predicted state.
ACK `N` removes every latest sequence `<= N` and restores the newest staged state.

Release order models locked fastutil 8.5.18 `Long2ObjectOpenHashMap` behavior: packed key zero
first, then occupied slots from highest to lowest, including iterator shift and wrapped-key
handling. A teleport records the current prediction sequence; an ACK at or below it still restores
state but suppresses captured-position rollback. Signed, duplicate, stale, and future ACKs receive
no extra range or monotonic validation.

## Presentation state and ordering

Destruction progress is keyed by signed breaker ID. Values `0..=9` replace and timestamp a record;
`10..=255` remove it. Scans occur only at game-time multiples of 20, and remove records strictly
older than 400 ticks.

Block events validate the packet's block registry ID but invoke the current local block state with
the action and parameter. The decoded packet block is not compared at trigger time. These events
and crack records are presentation-only and never become authoritative Ferrite commands.

The server-side block convergence path established in
[block interaction and convergence](block-interaction-and-convergence.md) emits direct corrections
before the next ACK. Normal committed changes may instead be ACKed before publication. One section
change uses ID 8; multiple changes use ID 84. State packets precede matching block-entity updates,
and destruction/event streams do not acknowledge predictions.

## Evidence

`crates/ferrite-protocol/tests/c2/play_clientbound_block.rs` owns exact zero-body goldens, signed
packing and registry boundaries, trusted compound NBT, malformed inputs, nullable section states,
wire-order duplicates, fastutil prediction release, teleport suppression, exact-type block
entities, current-state events, and crack relocation/removal/expiry.
