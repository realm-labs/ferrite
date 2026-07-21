# C0-C3 Ordering and Acknowledgements

This page is the normative index for independent correlation domains through compatibility level
C2, owns the exact block-prediction lifecycle, and records the specified C3 entity and container
order that has no acknowledgement domain. An acknowledgement in one row never satisfies, advances,
or resets another row.

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
| C3 statistics drain | serverbound play ID 12 action `request_stats` | clientbound play ID 3 stat map | exactly one response per request, including empty; no token; request atomically drains current dirty set |

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

## C3 entity session, spawn, motion, and state order

The specified C3 packets add no challenge counter or general entity acknowledgement. Damage event,
hurt yaw, entity motion, health/metadata and death remain separate projections. Camera selection
follows any required same- or cross-dimension relocation. Respawn precedes its position challenge
and the remaining new-level/player projections, and starts a fresh client-loaded interval. The
position challenge still uses only the C1/C2 teleport row above.

An ordinary tracker pairing sends one bundle with ID 1 `add_entity` first, then nondefault metadata,
syncable attributes, nonempty equipment, the entity's passenger list, its vehicle's passenger list,
and leash link when each exists. It calls `startSeenByPlayer` only after sending that bundle. Player
info must already exist before a player add can construct the remote player. Leaving visibility calls
`stopSeenByPlayer` before canonical singleton ID 77 removal. The removal packet's wider list form is
processed in wire order, and removal does not implicitly clear independent player-info state.

Within a pairing bundle, metadata is the complete nondefault snapshot, attributes are the complete
syncable set, equipment contains every nonempty slot, and relationship packets are complete current
lists/links. Runtime ID 99 and ID 131 dirty updates go to tracking players and self; runtime
equipment goes to tracking players, while leash attach/detach follows its explicit mutation send
flag. None has an acknowledgement. A later pairing snapshot is not a replay of old dirty packets.

Passenger-list comparison occurs at the start of `ServerEntity#sendChanges`, before ordinary
motion/state publication. Viewers whose own membership changes are filtered from the tracker
broadcast and receive the full list directly from `ServerPlayer#startRiding`/`removeVehicle` after
the corresponding position/effect work. Other viewers receive the tracker broadcast. Leash packets
are independent mutation publications except for their final position in the pairing bundle.

Within one ordinary `ServerEntity#sendChanges` pass for a regular nonpassenger entity, changed
velocity (and hurting-projectile acceleration) is sent before the chosen absolute/relative
position/rotation packet. Dirty metadata and attributes follow that pose packet; head rotation
follows dirty state; a `hurtMarked` self-inclusive motion packet is last. Any position-bearing
publication advances that viewer's delta base, while rotation-only publication does not. A passenger
instead publishes qualifying rotation, advances its base directly to current position, then
publishes dirty state. Feature-enabled new-behavior minecarts replace the ordinary pose selection
with their ordered step-list packet.

Dirty metadata is ordered before dirty attributes within the single dirty-state call. Equipment
changes are detected during living-entity ticking: old location effects/modifiers are removed, new
ones are installed, an exact hand swap may emit entity event 55, then remaining slot deltas are
published. Resulting syncable attribute dirtiness uses the ordinary ID-131 path rather than being
embedded into ID 102. Ferrite must preserve those separate projections and may not infer one from
the other on the client.

Mob-effect state uses direct audience publication rather than ordinary tracking broadcast. A
living entity first mutates effect attributes and marks particle metadata dirty, then sends ID 132
to direct `ServerPlayer` passengers. A player additionally receives its own ID 132; only a newly
added self effect sets blend, while updates and every replay clear it. Removal sends ID 78 to direct
player passengers after attribute removal, then to the affected player when it is a player. The
later metadata ID 99 and attribute ID 131 remain independent dirty-state projections.

Joining replays the player's active-effect hash-map iteration with blend clear. Successful riding
positions and challenges the player, replays every active effect of a living vehicle with blend
clear, then sends the vehicle's complete passenger list. Dismount removes every such vehicle effect
before sending the complete passenger list. These effect packets carry no acknowledgement and are
not implicit in passenger convergence.

ID 36 is emitted only after the server explosion has produced its game event, damaged entities,
applied configured block interaction, and optionally created fire. Each player at squared distance
strictly below 4096 receives a packet with its own optional hit-map knockback. The packet produces
client sound/particles before adding that optional vector to local velocity; it carries no block
delta, correction counter or response and does not replace authoritative block/entity updates.

ID 125 entity teleport has two narrowly scoped client responses, neither of which acknowledges a
server challenge. A direct result for a locally authoritative vehicle carrying the player produces
serverbound ID 34 `move_vehicle`. A missing entity matching the retained removed-player-vehicle ID
instead applies to the local player and produces ID 31 `move_player_pos_rot` with both flags false.
Interpolated, ordinary remote, unrelated missing and noncarrying branches send neither response.
These movement packets enter their already-specified normal server validation domains and must not
be correlated with player-position teleport IDs.

## C3 container state and prediction order

Container state IDs are version-local convergence versions, not challenge/response tokens. A menu
starts at zero. The server increments `(old + 1) & 32767` before every full-content ID 18 and every
individual slot ID 20. Cursor ID 96 and data ID 19 do not increment or carry that state. The client
assigns received state IDs without range or monotonicity validation and echoes its current value in
serverbound click ID 18.

Opening an ordinary server menu orders:

```text
optional current-menu ID 17 close -> removal and shared-state transfer
new ID 59 open_screen
new ID 18 complete slots + cursor with incremented state
new ID 19 for every property in ascending index order
server selects the new menu as current
```

The client creates/selects the menu on ID 59, then accepts the content/data packets by matching its
new current ID. A missing client screen constructor warns and leaves the old menu, so following
nonzero content/data normally fail the match. Both close handlers ignore the decoded container ID:
the server closes its current menu for any ID-19 request, and the client closes its current menu for
any ID-17 projection. A delayed old close can therefore terminate a newer menu; there is no close
acknowledgement or protected generation.

For a click, the client copies all slots, executes the full click locally, hashes only changed slots
and the resulting cursor with registry-aware CRC32C, then sends container ID, preexisting/current
menu state and hashes. The server validates/gates first, suppresses remote publication, executes the
same click authoritatively, receives the hashes into remote snapshots, and resumes publication.
Hash comparison can suppress a matching correction but never mutates authoritative state.

If the echoed state differed at server admission, the already executed click is followed by one
full content/cursor snapshot and every data property. If it matched, delta publication scans slots
ascending, emitting one state-incrementing ID 20 for each mismatching remote snapshot, then cursor
ID 96, then changed data ID 19 ascending. Thus a click can receive zero packets when prediction and
data all match. Spectator/dead click rejection instead sends a full snapshot immediately; wrong
container, invalid menu/slot and other named rejections send none.

Button clicks publish that same delta order only when the menu accepts the button. Crafter slot
state relies on later menu data/slot dirtiness rather than a direct response. Carried-slot selection
has neither menu state ID nor acknowledgement. These independent paths must not be correlated with
block prediction sequence ACKs, player-position teleport IDs, keepalives or chunk feedback.

Primary anchors are `MultiPlayerGameMode#handleContainerInput`,
`ServerGamePacketListenerImpl#handleContainerClick`, `ServerPlayer#openMenu`, its
`ContainerSynchronizer`, `AbstractContainerMenu#broadcastChanges`, `#broadcastFullState`,
`#incrementStateId`, and `RemoteSlot.Synchronized`.

## C3 player-projection order

The statistics row is request/response correlation without an echoed token. Handling a valid
`request_stats` first resets player idle time, then copies and clears the dirty set, and finally
queues exactly one ID 3. A later request cannot acknowledge, replace or cancel an earlier request;
it independently drains whatever is dirty then and still receives an empty map when nothing is.
No health, experience, cooldown, container, teleport, keepalive or block-prediction value can serve
as that response.

Ordinary server-player ticking advances food and cooldowns before projection. Within
`ServerPlayer#doTick`, canonical per-tick statistics and special-item work precede ID 104 health;
health/food/air/armor/experience scoreboard criteria follow it; ID 103 experience follows those
criteria. Cooldown start is sent immediately by its mutation path, while natural-expiry ID 22 zero
is sent during the cooldown tick. Dirty statistics accumulated by any of these actions remain
unsent until the statistics request path.

For a new connection, constructor sentinels force health and experience projection on its first
ordinary player tick; marking all statistics dirty during placement does not insert ID 3 into the
locked initial queue. Respawn has an explicit ID 103 after position and difficulty and before
active effects and level information. That send does not update the restored `lastSentExp=-1`, so
canonical respawn state repeats the current ID 103 tuple on its first ordinary tick.
Cross-dimension relocation resets health/food/experience sent markers only after position,
abilities, level information, player information and active-effect replay, so their next-tick
projections cannot be treated as part of the position challenge acknowledgement.

These four packets have no acknowledgement state of their own. Ferrite may coalesce internal
immutable snapshots only when the observable per-connection trigger and order remain identical;
it may not collapse the statistics response, suppress zero cooldown removal, or infer vitals from
entity metadata packets.

Primary anchors are `ServerGamePacketListenerImpl#handleClientCommand`,
`ServerStatsCounter#sendStats`, `ServerPlayer#doTick/#changeDimension`, `PlayerList#placeNewPlayer`,
`PlayerList#respawn`, `ItemCooldowns#tick`, and `ServerItemCooldowns`.

## C3 specialized screen activation and sign submission order

Mount activation reuses ordinary container identity and state convergence but replaces generic
`open_screen`. Canonical order is current-menu ID 17 and removal when needed, ID 41 specialized
activation, authoritative menu selection, then ordinary ID 18 full content and ID 19 data. The
tracked mount entity must precede ID 41. Later container clicks, deltas and close use the same
current ID/state rules and create no second mount-screen acknowledgement.

Book activation has no response. When written content resolution changes the held stack, ordinary
menu broadcasting precedes ID 58. The client nevertheless reads whichever stack is current in the
named hand when it handles ID 58, so intervening inventory convergence may select a different book
or the ignore path. Closing the view emits no packet.

Sign editing is the only request-return flow in this family:

```text
server stores allowed-editor UUID
    -> clientbound ID 8 current block-state correction
    -> clientbound ID 60 editor activation
    -> client screen removal sends serverbound ID 61 once
    -> strip four lines and asynchronously filter
    -> server executor checks then-current level/chunk/sign/wax/editor authority
    -> success rebuilds text and calls flags-3 update
    -> clears editor and calls a second unconditional flags-3 update
```

The ID-8 packet does not contain sign text, and ID 60 adds neither text nor a token; prior chunk or
block-entity synchronization supplies the client editor's initial text. ID 61 echoes position and
side but no challenge. Every accepted four-line rebuild creates new sign text, so even semantically
unchanged input takes both flags-3 update-call sites. Rejection has no correction or response.
While filtering is pending, sign ticks, range, level, block replacement, wax and allowed-editor
mutations legitimately change the eventual branch. These values must not be correlated with
block-prediction sequences, container state IDs, player-position challenges, keepalives or
statistics requests.

Primary anchors are `ServerPlayer#openHorseInventory/#openNautilusInventory/#openItemGui/#openTextEdit`,
`AbstractContainerMenu#sendAllDataToRemote`, `AbstractSignEditScreen#removed`,
`ServerGamePacketListenerImpl#handleSignUpdate`, and `SignBlockEntity#updateSignText`.
