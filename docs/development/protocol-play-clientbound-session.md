# Required Play Clientbound Session Protocol

Ferrite implements all five packets in
`PROTO-PLAY-CLIENTBOUND-SESSION-001` for Minecraft Java 26.2:

| ID | Identity | Session responsibility |
|---:|---|---|
| 32 | `minecraft:disconnect` | close with a trusted component reason |
| 44 | `minecraft:keep_alive` | schedule an exact signed-long ID-28 echo |
| 57 | `minecraft:move_vehicle` | correct the local authoritative root vehicle |
| 61 | `minecraft:ping` | immediately emit the exact signed-int ID-45 pong |
| 73 | `minecraft:player_rotation` | apply rotation and emit ID-32 movement rotation |

These packets are accepted only after a client level is installed. The version adapter keeps all
wire IDs, echo payloads, root-vehicle identity, interpolation state, and presentation reason data
connection-local.

## Disconnect and liveness

Disconnect uses the common trusted context-free component NBT codec. It retains the 512-depth and
packet-frame bounds but has no default NBT heap quota. Handling closes with the decoded reason and
does not produce an acknowledgement.

Keepalive and ping have disjoint response domains. An unfrozen client maps a signed-long keepalive
to serverbound ID 28 and maps a signed-int ping to serverbound ID 45. A pong cannot clear or
acknowledge a keepalive challenge.

When event polling is frozen, keepalive echoes enter a bounded deferred queue. Each entry expires
at receipt time plus 60,000 ms. Polling checks the send condition first: unfreezing at the expiry
instant still sends; an entry that remains frozen at or after expiry is removed. Ping bypasses this
queue and responds immediately.

## Player rotation

ID 73 carries independent yaw and pitch relativity booleans:

```text
yaw   = relative_yaw   ? current_yaw   + packet_yaw   : packet_yaw
pitch = clamp(relative_pitch ? current_pitch + packet_pitch : packet_pitch, -90, 90)
```

The values install immediately, and both old-render rotations synchronize to the result. The
response is serverbound ID 32 with the resulting floats and both movement flags false. There is no
teleport challenge or interpolation state. All float bit patterns pass the codec: infinities clamp
only when they are pitch, while NaN remains NaN and reaches the server movement validator.

## Vehicle correction

ID 57 applies only when the root vehicle differs from the player and is locally authoritative.
Ferrite's client conformance projection represents that root explicitly. An absent or nonlocal root
ignores the packet without a response.

For a qualifying root, position is compared with the interpolation target when active and otherwise
with the current position. The comparison is Euclidean distance strictly greater than the
double representation of `1e-5f`. Only that branch cancels interpolation and snaps position, yaw,
and pitch. A same-position rotation-only packet leaves rotation unchanged. Every qualifying packet
still emits serverbound ID 34 from the resulting vehicle pose and current on-ground state.

A NaN coordinate makes the comparison false and echoes current state. Infinity exceeds the
threshold, installs, and is then echoed into the server's explicit invalid/clamp boundary.

## Evidence

`crates/ferrite-protocol/tests/c2/play_clientbound_session.rs` owns the five exact goldens,
trusted reason quota, malformed boundaries, freeze scheduling, distinct response IDs, relative
rotation and render state, root-vehicle qualification, interpolation target, distance threshold,
rotation-only behavior, and exceptional floats.
