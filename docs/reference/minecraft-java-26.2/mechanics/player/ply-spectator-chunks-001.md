# Player mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `PLY-SPECTATOR-CHUNKS-001` — A live rule admits spectators to player-distance sources without gating their client view

**Parent:** `PLY-001`, `SIM-005`, `MOB-002`, `CLI-006`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — the rule default and sole reader, join/move/remove reconciliation, loading and
simulation ticket sources, natural-spawn counter effect, delayed mode/rule convergence, entity
tracking order and independent client chunk projection are explicit in locked server/client source.

**Applies when:**

A server player is added, removed, moves or changes the camera position; a spectator or the live
`spectators_generate_chunks` value changes before the next movement reconciliation; player-distance
updates produce loading/simulation/natural-spawn state; or the server projects ready chunks around
any player. The generic ticket thresholds remain owned by `SIM-005`, the natural-spawn transaction
by `MOB-SPAWN-001`, and terrain packet grammar by `PROTO-PLAY-CLIENTBOUND-TERRAIN-001`.

**Authoritative state:**

The live `PLAYER` Boolean rule (default true), `ServerPlayer#isSpectator`, current and last section
positions, the `PlayerMap` membership/ignored bit, `DistanceManager` players-per-chunk sources,
natural-spawn and player-loading distance trackers, `PLAYER_SIMULATION` tickets, server view and
simulation distances, and the player's requested view distance. Client projection additionally
uses the player's current `ChunkTrackingView`, connection-local pending chunk set, batch quota and
the server's currently ready visible chunks.

**Transition and ordering:**

**Rule predicate and initial registration:** `ChunkMap#skipPlayer` returns exactly
`player.isSpectator() && !spectators_generate_chunks`; non-spectators never read the rule. Adding a
player computes this predicate, inserts the player into `PlayerMap` with that ignored bit and saves
the current section. Only a nonignored player is added to `DistanceManager`. Regardless of the bit,
the player's chunk view is first set to empty and then rebuilt around the current chunk. Thus a
spectator joining while the rule is false has a normal client interest view but contributes no
player-distance source.

**Movement reconciliation:** Every admitted `ChunkMap#move` updates entity projection first: the
moving player's tracked entity is re-evaluated against all level players and every other tracked
entity is re-evaluated for that player. It then snapshots the saved section, computes the current
section, old ignored bit and new skip predicate. If neither the section nor ignored state changed,
it returns after entity tracking and performs no distance operation.

Otherwise it stores the current section; removes the player from the old distance source only when
the old bit was nonignored; adds the current source only when the new predicate is false; changes
the `PlayerMap` bit from nonignored to ignored or vice versa; and finally refreshes client chunk
tracking. A section move while admitted is therefore remove-old then add-new. A rule/mode-only
change at one section performs just the applicable remove or add. `ServerChunkCache#move` suppresses
the entire transaction for a removed player, then updates waypoint tracking after a live move.

**Distance-source consequences:** Adding the first admitted player at a chunk creates the
players-per-chunk set and supplies distance zero to both the radius-eight natural-spawn tracker and
the player-loading tracker. The loading tracker propagates distance and asynchronously installs
`PLAYER_LOADING` tickets at the fixed player ticket level for positions within the configured
server view distance. The same add directly installs one `PLAYER_SIMULATION` ticket at the source
chunk with level `max(0, 31 - simulationDistance)`; the simulation tracker then
propagates the activity thresholds specified by `SIM-005`.

Removing the last admitted player from a source chunk deletes that set, changes both distance
sources to infinity and removes the equal `PLAYER_SIMULATION` ticket. Loading-ticket additions and
removals still pass through the throttled distance-update/dispatcher pipeline; this rule does not
promise synchronous generation or unloading at the `move` call. Multiple admitted players sharing
a chunk retain the shared sources until the last leaves.

The natural-spawn tracker's union size becomes `spawnableChunkCount`, so an admitted spectator can
increase `baseMax * spawnableChunkCount / 289` global category caps. Candidate and local-cap checks
nevertheless call `playerIsCloseEnoughForSpawning`, which rejects every spectator independently of
this rule. A spectator therefore cannot itself qualify a spawn position, although its admitted
coarse union may raise the cap available to qualifying non-spectators elsewhere. With the rule
false it affects neither union nor cap.

**Mode, rule and movement cadence:** The gamerule has no change callback and `ServerPlayer#setGameMode`
does not generally reconcile chunk distance. Entering spectator changes mode without calling
`ServerChunkCache#move`. Leaving spectator calls `setCamera(self)`, but that method returns without
moving when the old camera was already self; changing from another camera does call `move` after
any camera-position teleport. Consequently a stationary own-camera player can retain the old
admission bit after either mode or rule changes until the next move call.

Accepted ordinary and vehicle movement handlers call `ServerChunkCache#move`; a player whose
camera is another live entity is snapped to that camera and moved every server player tick. For a
controlled-camera vanilla client, `LocalPlayer#sendPosition` forces a position message when its
unchanged-position reminder reaches 20 client ticks, and that handler reaches the same
reconciliation after its normal movement gates. This ordinary cadence bounds a healthy vanilla
own-camera transition, but the normative boundary is the next admitted move call, not the command
or rule mutation itself.

**Independent client projection:** `PlayerMap` retains ignored players as keys. `ChunkMap#tick`
refreshes chunk tracking for every key, and an initial add or reconciled move does so regardless of
the skip predicate. The view center is the player's current chunk and its distance is
`clamp(requestedViewDistance, 2, serverViewDistance)`; the gamerule is never read here.

A new or changed center sends `set_chunk_cache_center` before applying the old/new view difference.
Newly included positions mark an already ready visible chunk pending; they do not themselves make
one ready. Excluded positions remove a pending entry without a packet or, if already sent and the
player is alive, send `forget_level_chunk`. When any chunk later becomes ready, every player's view
is checked, including ignored spectators, so a false-rule spectator may receive a chunk generated
by another ticket source. Pending ready chunks are subsequently sent through the normal batch
start, full-chunk-with-light and batch-finish flow.

Changing only the rule or admission bit does not change the chunk view. In particular, disabling
generation at a stationary spectator removes its distance contribution at reconciliation but does
not emit cache-center or forget packets for the retained view. Already sent chunks remain on the
client until an ordinary view difference excludes them; positions entered while ignored remain
empty unless some other source makes their chunks ready. Entity tracking likewise precedes and is
not hidden by the admission bit.

**Removal and persistence:** Player removal snapshots the last section, removes `PlayerMap`
membership, removes the distance source only when the old bit was nonignored, then diffs the view
against empty. The gamerule is durable level state. The ignored bit, player-distance sets/tickets,
tracking view and pending send set are session state and are reconstructed on join rather than
persisted as player data.

**Branches and aborts:**

Non-spectator versus spectator; live rule true/false; add/move/remove; same/different section;
unchanged/changed ignored bit; own/remote camera; removed player; first/shared/last source player;
ready/not-ready chunk; pending/already-sent outgoing chunk; alive/dead recipient; and equal/changed
client center or requested distance.

**Constants and randomness:**

Rule default true; natural-spawn tracker radius 8; forced local-client position reminder 20;
requested view clamp minimum 2; simulation ticket level
`max(0, 31 - simulationDistance)`; natural global cap denominator 289. No branch
of this rule consumes gameplay RNG.

**Side effects:**

PlayerMap membership/ignored state, last section, distance sources, asynchronous `PLAYER_LOADING`
tickets, `PLAYER_SIMULATION` tickets, natural-spawn union and cap, chunk activity/generation,
entity/waypoint tracking, client cache-center/full-chunk/forget projections and connection pending
chunk state.

**Gates:**

Spectator status plus live rule for distance admission; player removal; section/ignored changes for
move reconciliation; configured server/simulation/requested view distances; ready visible chunks,
view membership, send quota/acknowledgements and recipient life for client projection. Final
natural-spawn positions independently require a nonspectator player through `MOB-SPAWN-001`.

**Boundary cases and quirks:**

The name describes generation, not visibility: false-rule spectators keep a moving client view and
can see externally generated chunks. A live toggle is delayed until movement reconciliation and
does not unload retained client chunks. True-rule spectators raise the coarse natural-spawn union
and possibly global cap but never satisfy the final nearby-player test. Entity tracking work runs
before admission changes. Requested client view distance controls projection while the shared
player-loading tracker uses configured server view distance.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-REPORT-001`;
`net.minecraft.world.level.gamerules.GameRules`;
`net.minecraft.server.level.ChunkMap#skipPlayer`, `#updatePlayerStatus`, `#move`,
`#updateChunkTracking`, `#applyChunkTrackingView`, `#onChunkReadyToSend`;
`net.minecraft.server.level.PlayerMap`;
`net.minecraft.server.level.DistanceManager#addPlayer`, `#removePlayer`,
`DistanceManager$PlayerTicketTracker`;
`net.minecraft.server.level.ServerChunkCache#move`, `#tickChunks`;
`net.minecraft.server.level.ServerPlayer#setGameMode`, `#setCamera`, `#tick`;
`net.minecraft.server.network.ServerGamePacketListenerImpl#handleMovePlayer`,
`#handleMoveVehicle`; `net.minecraft.server.network.PlayerChunkSender`;
`net.minecraft.client.player.LocalPlayer#sendPosition`; `SIM-005`; `MOB-SPAWN-001`;
`PROTO-PLAY-SERVERBOUND-MOVEMENT-001`; `PROTO-PLAY-CLIENTBOUND-TERRAIN-001`; `EXP-PLY-008`.

**Test vectors:**

Cross spectator/non-spectator with the live rule true/false on join, shared-source join, every
same/cross-section move and removal. Toggle mode and rule while stationary at every
`positionReminder` value, while moving, riding, own-camera and remote-camera; record the exact move
call at which ignored state changes. Trace players-per-chunk, distance levels, loading/simulation
tickets, activity and eventual unload with zero/one/two same-chunk contributors. Hold one ordinary
player elsewhere and assert spectator-dependent natural union/global caps while every
spectator-only final spawn location remains rejected. Finally record entity visibility, tracking
view, cache-center, pending/full/forget packets and retained client chunks while the spectator
enters ready and unready terrain supplied by no source, another player, forced chunks and portals.
