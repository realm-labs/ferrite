# C0 Handshake, Status Discovery, and Ping

This is the normative C0 server contract for an unmodified Java Edition `26.2` client. Packet
identity and numeric ID come only from the locked `OFF-REPORT-001` packet report; fields and state
consequences come independently from `OFF-SERVER-001` and `OFF-CLIENT-001`.

## Packet inventory

| State | Direction | Numeric ID | Identity | Fields in order |
|---|---|---:|---|---|
| handshake | serverbound | `0` | `minecraft:intention` | protocol version VarInt; host `UTF(255)`; port unsigned short; intent VarInt |
| status | serverbound | `0` | `minecraft:status_request` | none |
| status | serverbound | `1` | `minecraft:ping_request` | signed long token |
| status | clientbound | `0` | `minecraft:status_response` | `UTF(32767)` containing status JSON |
| status | clientbound | `1` | `minecraft:pong_response` | signed long token |

All five packets use the frame contract in [`framing-and-primitives.md`](framing-and-primitives.md).
No registry ID, entity metadata index, data-component ID, acknowledgement number, compression flag,
or conditional wire field occurs in C0.

## Handshake intention

`minecraft:intention` is the only legal handshake packet and is terminal for that state. Its intent
mapping is exact:

| Wire value | Intent | Server transition |
|---:|---|---|
| `1` | status | Configure status clientbound first. If status replies are enabled and a cached status snapshot exists, install the status serverbound listener. Otherwise close without a status response. The protocol-version field is not compared on this branch. |
| `2` | login | Configure login clientbound. Protocol `776` installs the login serverbound listener with `transferred = false`; a mismatch sends a login disconnect and closes. The full login exchange belongs to C1. |
| `3` | transfer | If transfers are disabled, configure login clientbound, send the transfers-disabled login disconnect, and close. If enabled, take the login path with `transferred = true`, including the exact-version check. |

Every other intent value fails enum decode and closes. Login mismatch classification is also locked:
values below `754` use the outdated-client reason; every other value unequal to `776` uses the
incompatible-version reason. That distinction is recorded here because it is a handshake
consequence, though its wire disconnect packet is specified with C1.

The host may be empty at codec level and the port may be `0` or `65_535`; this listener applies no
additional syntax, DNS, or routing validation. Ferrite may retain host and port only as untrusted
connection-local routing context. They are never authoritative world or account state. The vanilla
status client sends its parsed server host and port, protocol `776`, and intent `1`.

Primary anchors are `net.minecraft.network.protocol.handshake.ClientIntentionPacket#write`,
`net.minecraft.network.protocol.handshake.ClientIntent#byId`,
`net.minecraft.network.protocol.handshake.ClientIntent#id`,
`net.minecraft.server.network.ServerHandshakePacketListenerImpl#handleIntention`,
`net.minecraft.server.network.ServerHandshakePacketListenerImpl#beginLogin`, and
`net.minecraft.network.Connection#initiateServerboundStatusConnection`.

## Status response schema

The status response is one compact JSON value encoded as `UTF(32767)`. The root must decode as an
object. The official encoder writes known fields in the order below and omits absent or defaulted
fields:

| JSON field | Presence and value |
|---|---|
| `description` | Lenient optional component, default empty. A simple literal can be a JSON string; structured components use the locked component codec. |
| `players` | Lenient optional object. If present and valid, required signed JSON integers `max` and `online`, plus lenient optional `sample` defaulting to `[]`. Each sample entry is `{ "id": <UUID string>, "name": <string> }`. The codec adds no independent numeric or list bound beyond the total status string bound. |
| `version` | Lenient optional object with required string `name` and signed JSON integer `protocol`. A compatible Ferrite status advertises name `26.2` and protocol `776`. |
| `favicon` | Lenient optional string `data:image/png;base64,` followed by standard Base64. Decode removes line-feed characters from the Base64 portion. The vanilla client retains only a decodable PNG whose width and height are each at most `1024`; an invalid or oversized icon becomes absent presentation data. |
| `enforcesSecureChat` | Lenient optional JSON boolean, default `false`. |

Unknown object fields are ignored. A malformed lenient optional field is treated as absent rather
than invalidating the whole response; for example, `{ "players": {} }` produces no players value.
A malformed JSON text or non-object root fails the packet. The minimal valid status value is `{}`.

Ferrite's normalized status snapshot supplies description, optional player counts/sample, locked
version, optional icon bytes, and the secure-chat policy. The adapter serializes that immutable
snapshot; it must not expose JSON or component codec objects to simulation or persistence. The
locked server sends the snapshot cached at handshake time rather than constructing a different
answer for repeated requests.

Schema and presentation anchors are
`net.minecraft.network.protocol.status.ClientboundStatusResponsePacket`,
`net.minecraft.network.codec.ByteBufCodecs#lenientJson`,
`net.minecraft.util.LenientJsonParser#parse`,
`net.minecraft.network.protocol.status.ServerStatus`,
`net.minecraft.network.protocol.status.ServerStatus$Players`,
`net.minecraft.network.protocol.status.ServerStatus$Version`,
`net.minecraft.network.protocol.status.ServerStatus$Favicon`,
`net.minecraft.server.players.NameAndId`, and
`net.minecraft.client.multiplayer.ServerData#validateIcon`.

## Server state machine

The status listener has one local boolean, initially false, recording whether a status request was
handled. Its transitions are:

1. A first `minecraft:status_request` sets the boolean before sending exactly one
   `minecraft:status_response` from the cached snapshot. It does not close the connection.
2. A later status request on the same connection sends no second response and closes with the
   internal request-handled reason.
3. A `minecraft:ping_request` is legal in status state whether or not step 1 occurred. The server
   sends one pong containing the identical 64 token bits, then closes with the same internal
   request-handled reason.

There is no acknowledgement field. The pong itself is the response/correlation operation, and its
token must be an opaque bit-for-bit echo. A second ping cannot begin a new lifecycle after the first
has closed the session. Packet IDs from handshake, login, configuration, or play are illegal under
the installed status dispatch table and cause codec failure/closure.

The normalized ingress/egress mapping is deliberately small:

- intention `status` selects a connection-local discovery session;
- status request reads a server-status snapshot and emits its projection without mutating gameplay;
- ping carries an opaque signed-64 token to pong, then terminates the discovery session.

Primary server anchors are
`net.minecraft.server.network.ServerStatusPacketListenerImpl#handleStatusRequest` and
`net.minecraft.server.network.ServerStatusPacketListenerImpl#handlePingRequest`.

## Vanilla client ordering

The `26.2` client opens a connection, sends the terminal status intention, changes its outbound
codec to status, and sends one status request. Its first valid status response populates the server
list entry, records the local millisecond start time, and sends that value as the ping token. A
second status response is unsolicited and makes the client close. On pong, the client computes
latency from its stored local start time (it does not compare the returned token), closes, and runs
the completion callback.

The server must nevertheless echo the token exactly because that is the official server contract
and is observable to other conforming `26.2` clients. Primary client anchors are
`net.minecraft.client.multiplayer.ServerStatusPinger#pingServer`,
`net.minecraft.client.multiplayer.ServerStatusPinger$1#handleStatusResponse`, and
`net.minecraft.client.multiplayer.ServerStatusPinger$1#handlePongResponse`.

## Completion boundary

The C0 reference conclusion is source-specified. A server implementation is C0-conformant only when
all `C0-*` vectors in [`conformance.md`](conformance.md) pass. C0 does not establish login,
compression, configuration, play, legacy-ping support, proxy forwarding, or compatibility with any
protocol version other than `776`.
