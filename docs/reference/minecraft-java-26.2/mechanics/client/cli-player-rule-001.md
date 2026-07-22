# Client-observable behavior mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `CLI-PLAYER-RULE-001` — Player-facing game rules snapshot on join and project live changes

**Parent:** `CLI-006`

**FidelityClass:** `EquivalentPlayerVisibleBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — the locked server has a complete direct-reader set for `immediate_respawn`,
`reduced_debug_info` and `locator_bar`. Their join snapshots, live callbacks, client flag updates,
death-screen choice and waypoint connection rebuilds are explicit in locked source and in the
specified play protocol families. This leaf owns those projections, not authoritative respawn
admission or the waypoint wire format.

**Applies when:**

A player enters play, one of the three rules changes, the local player receives its own combat-kill
packet, or a level's locator-bar manager creates, updates, removes or rebuilds connections.

**Authoritative state:**

The level's boolean game-rule values; player/level membership; local-player
`reducedDebugInfo` and `showDeathScreen` flags; the level waypoint manager's player, transmitter and
connection sets; transmitter visibility/team/range result; and the independently authoritative
death/respawn and waypoint states owned by their gameplay/protocol families.

**Transition and ordering:**

**Initial player projection:**

`PlayerList#placeNewPlayer` reads `immediate_respawn` and `reduced_debug_info` from the player's
destination level. The play login packet carries `reduced_debug_info` unchanged and carries
`show_death_screen` as the inverse of `immediate_respawn`. `ClientPacketListener#handleLogin`
assigns both fields to the new local player. These values are a snapshot at placement; they are not
inferred from later death or UI state. Client respawn replacement copies both fields from the old
local player before continuing normal respawn projection. A player entering a server level also
enters that level's waypoint manager; connection creation consults `locator_bar` independently, so
the locator state is not a third play-login field.

The rules are registered as `immediate_respawn` in category `PLAYER` with default `false`,
`locator_bar` in category `PLAYER` with default `true`, and `reduced_debug_info` in category `MISC`
with default `false`.

**Live immediate-respawn and debug projection:**

`MinecraftServer#onGameRuleChanged` first notifies `NotificationManager`, then broadcasts the
rule-specific projection to every current server player. `immediate_respawn` sends the
`IMMEDIATE_RESPAWN` game event with float value `1.0` for true and `0.0` for false. The client sets
`showDeathScreen` exactly when that value equals `0.0`; other finite or nonfinite values therefore
select false, although the canonical publisher emits only the two stated values.

`reduced_debug_info` sends one player-scoped entity event for every player: byte `22` for true or
byte `23` for false. `Player#handleEntityEvent` sets the receiving player object's flag true or
false respectively; other event bytes retain their normal dispatch. The canonical packet targets
each recipient's own entity, so the local debug presentation follows the server rule without
changing authoritative gameplay state.

When a combat-kill packet identifies exactly the current local player, the client either opens a
death screen using the packet message and current hardcore flag when `showDeathScreen` is true, or
immediately calls `LocalPlayer#respawn` when it is false. That call sends the ordinary
`PERFORM_RESPAWN` request and resets toggle keys; it does not bypass server-side dead/win,
connection or lifecycle admission. A packet for any other or missing player ID does nothing.
There is no generation or deduplication token, so repeated qualifying kill packets repeat the
selected presentation/request branch.

**Live locator-bar projection:**

For a `locator_bar` change, `MinecraftServer#onGameRuleChanged` iterates every server level. Enabling
iterates that level's current players and calls `ServerWaypointManager#addPlayer`; disabling calls
`breakAllConnections`. The callback runs after notification and does not establish a stable
cross-level or set-iteration order.

`addPlayer` retains the player, attempts a connection to every tracked transmitter, and also tracks
the player as a transmitter when that player exposes a waypoint. Connection creation rejects self,
rejects a player whose current level rule is false, then asks the transmitter for a representation:
a present representation is sent and stored; an absent one removes any existing connection.
Updates retain a nonbroken connection, while a broken connection is re-evaluated and replaced or
removed. Removing a player disconnects that player's row, removes the player from transmitter and
player sets, and updates the other rows through the ordinary transmitter-removal path.
`breakAllConnections` disconnects every stored connection and then clears the table. Re-enabling
therefore reconstructs current eligibility rather than resurrecting old connection objects.

Waypoint packet identity, block/chunk/azimuth representation thresholds, icon/team remakes and
client collection mutation are fixed by `PROTO-PLAY-CLIENTBOUND-BOSS-WAYPOINT-001`. The rule only
gates connection membership; it does not change authoritative entity location or team state.

**Branches and aborts:**

Join versus live callback; rule true/false; packet target is/is not the local player; death screen
versus immediate request; missing combat entity; waypoint self; player no longer in the level;
locator disabled; transmitter returns no representation; connection intact/broken/missing; player
is/is not itself a transmitter.

**Constants and randomness:**

Game-event values `1.0` and `0.0`; entity-event bytes `22` and `23`; boolean defaults
false/true/false as stated. No RNG. Waypoint range and representation constants remain owned by the
specified protocol family.

**Side effects:**

Play-login fields, local player presentation flags, death-screen construction, a standard respawn
request, toggle-key reset, entity/game-event packets, waypoint track/update/untrack packets and
server waypoint connection tables. No rule branch directly mutates death, respawn, entity position,
team or debug-authoritative gameplay state.

**Gates:**

Destination/current level, player membership and connection, exact callback identity, local-player
combat identity, current presentation flag, locator rule, nonself transmitter eligibility and
transmitter representation.

**Boundary cases and quirks:**

The login field is inverted only for immediate respawn. Live immediate-respawn handling compares
the float to exact positive zero, while the publisher emits only `0.0` or `1.0`. Reduced-debug live
updates are entity events, not a repeat login packet. Locator disable disconnects and clears all
connections in every level; enable re-adds only each level's current players. These client-facing
rules do not make the client authoritative over respawn or waypoint truth.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-REPORT-001`;
`GameRules#IMMEDIATE_RESPAWN`, `#LOCATOR_BAR`, `#REDUCED_DEBUG_INFO`;
`PlayerList#placeNewPlayer`; `MinecraftServer#onGameRuleChanged`;
`ServerWaypointManager#isLocatorBarEnabledFor`, `#addPlayer`, `#createConnection`,
`#updateConnection`, `#removePlayer`, `#remakeConnections`, `#breakAllConnections`;
`ClientPacketListener#handleLogin`, `#handleGameEvent`, `#handlePlayerCombatKill`;
`Player#handleEntityEvent`; `LocalPlayer#respawn`;
`PROTO-PLAY-CLIENTBOUND-ENTRY-001`, `PROTO-PLAY-CLIENTBOUND-COMBAT-LOOK-001`,
`PROTO-PLAY-CLIENTBOUND-BOSS-WAYPOINT-001`.

**Test vectors:**

Join with every boolean combination; toggle each rule both ways with zero/one/multiple players and
multiple levels; assert notification precedes projection; respawn replacement retains both flags;
combat kill for local/other/missing IDs and repeated delivery; death screen versus exactly one
ordinary request per packet; locator enable/disable with self, absent representation, team/range
change, broken connection, player removal and a player transmitter; assert disable sends every
disconnect before clear and re-enable rebuilds only current eligible connections.
