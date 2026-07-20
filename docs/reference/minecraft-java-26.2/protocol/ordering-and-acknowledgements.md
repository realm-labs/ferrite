# C0-C2 Ordering and Acknowledgements

This page is the normative index for independent correlation domains through compatibility level
C2 and owns the exact block-prediction lifecycle. An acknowledgement in one row never satisfies,
advances, or resets another row.

| Domain | Challenge/request | Response | Correlation and terminal rule |
|---|---|---|---|
| C0 status ping | serverbound status ID 1 signed long | clientbound status ID 1 signed long | exact opaque echo, then server close |
| C1 login transition | clientbound login-finished ID 2 | serverbound login-acknowledged ID 3 | unit terminal packet, legal only in `PROTOCOL_SWITCHING` |
| C1 configuration transition | clientbound finish ID 3 | serverbound finish ID 3 | unit terminal packet, legal only for the current join-world task |
| C1/C2 teleport | clientbound play ID 72 challenge VarInt | serverbound play ID 0 matching VarInt | exact current ID; match without pending position faults |
| C2 chunk flow | clientbound IDs 12, one or more 45, then 11 | serverbound ID 11 float advice | one feedback decrements the outstanding-batch count; no per-batch token |
| C1/C2 keepalive | clientbound common ID 4/44 signed long | serverbound common ID 4/28 signed long | exact one outstanding token; invalid remote echo times out |
| C1/C2 diagnostic ping | clientbound common ID 5/61 signed int | serverbound common ID 5/45 signed int | exact echo; never clears keepalive |
| C2 block prediction | serverbound play IDs 41/66/67 sequence VarInt | clientbound play ID 4 VarInt | releases retained positions with latest sequence `<=ACK` |

The state-specific IDs in the table are intentionally not interchangeable even where common
packet classes share a body. Full transition and liveness rules remain in
[handshake/status](handshake-and-status.md),
[login/configuration](login-and-configuration.md),
[serverbound play](play-serverbound.md), and
[clientbound play](play-clientbound.md).

## Client block-prediction transaction

`BlockStatePredictionHandler` begins with sequence zero, no retained positions, prediction false,
and last teleport sequence `-1`. `startPredicting()` increments the signed int with Java wrapping,
sets prediction true, and returns the same handler as an `AutoCloseable`. The client performs the
predictive action before constructing/sending its concluding packet, and `close()` clears the flag.
The first ordinary prediction therefore uses sequence `1`.

While prediction is true, every successful client `setBlock` first reads the old state, performs
the write, and retains the old state plus the local player's current position. Retention is keyed by
packed block position:

- the first successful write stores `(current_sequence, old_state, player_position)`;
- a later prediction at that same position changes only the stored sequence;
- it deliberately preserves the first old state and first player position;
- unsuccessful writes retain nothing;
- one predictive callback can retain several positions independently.

The prediction scope itself does not buffer packets, world callbacks, inventory changes, sounds,
or particles. Those local effects occur immediately. Its retained map protects only block-state
convergence.

Vanilla opens this transaction for creative destroy starts, every new survival destroy start,
completed destroy stops, use-on-block, and use-in-air. A slow survival start normally retains no
block state because it performs no successful write, but still consumes a prediction sequence.
Explicit abort packets are outside the scope and use sequence zero. Pick-block and swing are
unsequenced.

Primary anchors are `MultiPlayerGameMode#startPrediction`, `#startDestroyBlock`,
`#continueDestroyBlock`, `#useItemOn`, `#useItem`, `ClientLevel#setBlock`, and
`BlockStatePredictionHandler#retainKnownServerState`.

## Server sequence accumulator

`ServerGamePacketListenerImpl` starts `ackBlockChangesUpTo` at `-1`. A registration requires
`sequence>=0` and stores `max(sequence,current_accumulator)`. This is cumulative only within the
interval before the next listener tick. The first statement of `tick()` sends ID 4 when the value
is greater than `-1`, then resets it to `-1`; the server retains no greatest-ever ACK floor.

Registration occurs at different points:

1. `use_item_on` and `use_item` pass the loaded gate, then register before all later validation or
   gameplay work.
2. The three destroy actions pass the loaded gate, run the entire authoritative break handler, and
   register after it returns, including its ordinary rejection branches.
3. Other player actions carry a sequence field but never register it. Pick-block and swing carry
   none.
4. Any predictive request dropped by the client-loaded gate receives no ACK. A later registered
   larger sequence can release its retained states because the response compares by `<=`.

Negative registration throws a handler fault. Use-on/use-in-air reach registration before their
action and therefore fault without that action; a destroy request reaches registration after the
break handler and can mutate or publish corrections before the fault. Client signed overflow from
`2_147_483_647` to `-2_147_483_648` is unguarded and therefore has no safe continuation. Sequence
zero is valid and normally appears on vanilla aborts. After ACK 10 is sent and the accumulator
resets, an adversarial later sequence 1 can produce ACK 1; monotonicity is a vanilla-sender
property, not a server codec or retained-state invariant.

Primary anchors are `ServerGamePacketListenerImpl#handlePlayerAction`, `#handleUseItemOn`,
`#handleUseItem`, `#ackBlockChangesUpTo`, and `#tick`.

## Authoritative updates before acknowledgement

Clientbound ID 8 and every entry in ID 84 call
`ClientLevel#setServerVerifiedBlockState(pos,state,19)`. If the position has no retained entry, the
state is written immediately. If it does, the packet only replaces that entry's saved server state;
the predicted local state remains visible and all later predictions at that position retain this
latest authoritative value.

This produces the normal use-on order:

```text
client: begin sequence N -> predict one or more writes -> send use_item_on(N)
server: register N -> validate/apply -> send hit-position ID 8
                                 -> send adjacent-position ID 8
next server listener tick: send ACK N
client: stage each retained-position update -> ACK applies staged state
```

Break and use-on denial corrections generated inside their handlers are likewise queued before the
next-tick ACK. For use-on early exits before the common correction tail, no ID-8 pair is sent even
though a post-load sequence was registered.

## Acknowledgement before authoritative update

Successful block writes normally enter the chunk holder's changed-position set. The next server
connection tick emits the block ACK before the later per-player chunk-change publication phase.
Consequently the normal successful break/use mutation can order as:

```text
client: begin sequence N -> predict state -> send request(N)
server: register N -> change authoritative world
next server listener tick: send ACK N
later chunk publication: send ID 8 or ID 84 -> optional ID 6
client: ACK restores saved pre-prediction state -> later update installs authoritative state
```

This ordering is protocol-valid and the client does not defer ACK waiting for a matching update.
Whether a renderer presents a frame between the two client tasks is the sole experiment-owned
question in [`PLY-BREAK-001`](../mechanics/player/ply-break-001.md); the packet/task order itself is
source-specified. Implementations must not reverse or suppress the ACK to hide that possible
intermediate state.

## Client ACK application

On ACK `N`, the client scans the retained map and removes every entry whose **latest** sequence is
`<=N`. It processes entries in locked fastutil `8.5.18`
`Long2ObjectOpenHashMap` iterator order: packed key zero first when present, then occupied table
slots from highest index downward, with the iterator's wrapped-key behavior. This is not insertion
or coordinate order.

For each removed entry it compares the current local state by reference with the saved authoritative
state. A difference calls `syncBlockState(pos,state,captured_position_or_null)`, which writes flags
`19`. The captured player position is supplied only when `lastTeleportSequence<N`. When supplied,
and only when the local player remains in that level and collides with the restored state, the
client snaps exactly to the captured position. An unchanged state causes neither write nor snap.

Handling clientbound ID 72 records `lastTeleportSequence=currentSequence` after applying the
teleport and sending its two responses. Thus an ACK at or below that sequence cannot use a captured
position to undo the teleport; its block state still synchronizes. The comparison uses ACK `N`, not
the individual retained entry sequence.

The client validates no ACK range or monotonicity:

- a duplicate or stale smaller ACK rescans and normally removes nothing;
- a negative ACK can remove only entries whose wrapped sequence is at most that negative value;
- a future ACK removes every current entry at or below it, even without proof that the server saw
  those requests;
- an entry advanced by a later same-position prediction survives an earlier ACK;
- no removed prediction is replayed after correction.

Primary anchors are `ClientPacketListener#handleBlockChangedAck`,
`ClientLevel#handleBlockChangedAck`, `ClientLevel#syncBlockState`,
`BlockStatePredictionHandler#endPredictionsUpTo`, `#onTeleport`, and the locked fastutil iterator.

## Delta, block-entity, event, and crack order

The ordinary chunk broadcaster sends changed light before block deltas. It scans sections
bottom-to-top. A one-position section sends ID 8 and then a matching ID 6 when that block entity has
an update packet. A multi-position section sends one ID 84 and then matching ID-6 packets in the
change set's iteration order. The client applies changes within ID 84 in their wire order. Block
entity data never substitutes for the preceding state and is ignored unless the current client
entity has the decoded type.

ID 5 destruction progress is an independent, unacknowledged presentation stream sent immediately
to nearby nonbreaking players. ID 7 block events are independent queued presentation records sent
only after the server successfully triggers the matching current block. Neither advances block
prediction sequence state. A block ACK acknowledges request processing, not block-entity NBT,
crack visibility, event delivery, a particular coordinate, or completion of chunk publication.

Ferrite may use internal transaction IDs and immutable snapshots, but the 26.2 adapter must project
these independent domains and their exact order without persisting raw counters, packet IDs,
registry IDs, packed coordinates, or client-retention records.
