# Entities mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ENT-LIFECYCLE-001` — Entity ownership is a UUID, section-visibility and passenger-tree transaction

**Parent:** `ENT-001`, `ENT-002`, `ENT-008`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — insertion, UUID rejection, section/tracking/ticking callbacks, root/passenger tick
traversal, all removal reasons, chunk unload and same/cross-dimension teleport order are explicit in
locked source.

**Applies when:**

An entity is added, crosses a section/status boundary, ticks, mounts/dismounts, unloads, is removed,
or teleports.

**Authoritative state:**

UUID and numeric ID, owning level callback, section key/status, known/visible/ticking collections,
removal reason, vehicle/passenger immutable lists, chunk load status and teleport transition.

**Transition and ordering:**

Insertion first adds the UUID to `knownUuids`; collision warns and returns false without section or
callback mutation. Success computes the block section, inserts there, installs its callback, invokes
`onCreated` only for a new/worldgen entity, then starts tracking if effective status is accessible
and starts ticking if it is ticking. Always-ticking entities force effective `TICKING`. A removed
entity is rejected before manager insertion. Player insertion is exceptional: a duplicate UUID
causes the old player to unride and be discarded immediately, then the new player is added.

Tracking start adds the chunk tracker before player/waypoint/navigation/dragon-part/game-event
memberships; tracking end removes those in the locked reverse-domain sequence and drops debug state.
On section movement, remove from the old section, remove it if empty, add to the new section, then
compare effective visibility: tracking stop/start precedes ticking stop/start in that method, and an
accessible destination finally moves its dynamic game-event listener. Same-status accessible moves
only invoke the section-change callback.

The server root loop skips removed/frozen entities, calls `checkDespawn`, then admits a player or an
entity whose current chunk is entity-ticking. A valid passenger is not root-ticked; a stale vehicle
link is stopped first. `tickNonPassenger` copies old position/rotation, increments `tickCount`, calls
`tick`, then traverses the current passenger list. Each valid passenger that is a player or remains
in the tick list receives the same old-state/increment sequence followed by `rideTick`, recursively;
removed/mismatched passengers stop riding and unlisted non-player passengers receive no ride tick.

Mount rejects same vehicle, a vehicle that cannot accept, a nonserializable server vehicle, graph
cycles, ride/capacity failure, shift or positive boarding cooldown unless forced. It stops an old
ride, sets standing pose, links both sides, emits mount and fires the indirect-player criterion.
Server insertion puts a player first when the old first passenger is not a player; otherwise append.
Ejection iterates passengers from last to first. Dismount clears the passenger's vehicle before
removing it from the vehicle list, sets boarding cooldown `60`, and emits dismount only if the
passenger has no removal reason or that reason destroys.

Removal keeps the first reason, but every call still runs cleanup: destroying reasons stop its ride;
all reasons stop every passenger, invoke level-callback removal and `onRemoval`. Manager removal
deletes the section entry, then ticking, then tracking, invokes `onDestroyed` only for destroying
reasons, removes UUID/callback and removes an empty section. Reasons are `KILLED` and `DISCARDED`
(destroy, do not save), `UNLOADED_TO_CHUNK` (do not destroy, save), and
`UNLOADED_WITH_PLAYER`/`CHANGED_DIMENSION` (neither).

Chunk unload first persists savable root entities; pending/fresh loads defer it. It then visits each
saved root's passengers-and-self, assigns `UNLOADED_TO_CHUNK`, and replaces the callback with null.
Pending loads are drained before unloads during manager tick.

Same-dimension teleport recursively teleports passengers first, places the same root, sends riding-
player correction when appropriate, then runs the post-transition callback. Cross-dimension
teleport snapshots/ejects passengers, teleports each first, then creates a destination root. On
success it restores state, removes the old root as changed-dimension, places/adds the new root,
force-remounts successfully transferred passengers without events, resets destination activity,
runs post-transition and transfers spectators. Ordinary entity identity therefore changes; players
use their override.

**Branches and aborts:**

Duplicate UUID, removed insertion, hidden/non-ticking sections, invalid ride graph/capacity, pending
chunk I/O, removed source, non-server source level and destination type creation failure. Teleport
stops an existing ride before choosing same/cross path unless `asPassenger` is set.

**Constants and randomness:**

Boarding cooldown is `60`; portal tickets use radius `3`; the level skips ordinary entity processing
after `300` empty ticks. Generic ownership consumes no RNG; entity construction may assign a UUID.

**Side effects:**

Section/UUID/tick/visible membership, chunk tracking, player/sleep/navigation/waypoint/dragon-part/
game-event/debug state, mount graph/events/criteria, save I/O, destination creation and client
tracking/corrections.

**Gates:**

Removal state, UUID uniqueness, effective chunk visibility, frozen state, entity-ticking range,
passenger graph and serialization/capacity rules, load status and teleport transition/type creation.

**Boundary cases and quirks:**

Removal cleanup is not guarded after the first reason assignment, so repeated `setRemoved` calls
re-enter callbacks with the new call's reason while the stored reason stays first. Cross-dimension
creation failure has no rollback: passengers have already been ejected/transferred, while the old
root has not yet been removed. Loaded/accessibly tracked is distinct from ticking. Saved passenger
trees are persisted under their root, never as independent roots.

**Evidence:**

`OFF-SERVER-001`;
`net.minecraft.world.level.entity.PersistentEntitySectionManager#addEntity`,
`net.minecraft.world.level.entity.PersistentEntitySectionManager$Callback#onMove`,
`net.minecraft.world.level.entity.PersistentEntitySectionManager$Callback#onRemove`,
`net.minecraft.server.level.ServerLevel#tickNonPassenger`,
`net.minecraft.server.level.ServerLevel#tickPassenger`,
`net.minecraft.server.level.ServerLevel$EntityCallbacks`,
`net.minecraft.world.entity.Entity#startRiding`,
`net.minecraft.world.entity.Entity#setRemoved`,
`net.minecraft.world.entity.Entity#teleportSameDimension`,
`net.minecraft.world.entity.Entity#teleportCrossDimension`; `EXP-ENT-001`.

**Test vectors:**

Duplicate ordinary/player UUID; removed add; accessible↔ticking section moves; remove and mount
mutation during root/passenger tick; three-level tree; every removal reason/repeated removal; pending
unload; same-dimension relative transition; cross-dimension success and destination-create failure.
