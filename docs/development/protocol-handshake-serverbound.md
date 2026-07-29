# Minecraft 26.2 Serverbound Handshake

`G01-P3-F003` implements `PROTO-HANDSHAKE-SERVERBOUND-001`. The handshake lane contains exactly
one terminal packet, `minecraft:intention` at ID zero:

```text
protocol_version: VarInt
host: UTF(255)
port: unsigned big-endian short
intention: VarInt (status=1, login=2, transfer=3)
```

The codec resolves ID zero through the locked handshake/serverbound packet catalog, preserves the
signed protocol version and unsigned port, applies Minecraft's lossy UTF and UTF-16-unit rules, and
rejects every other intention ordinal. Empty hosts and ports `0..=65535` remain legal. Host, port,
protocol, and intent form untrusted connection-local routing context; none enters account, world,
simulation, or persistence state.

## Ordered transition plans

`HandshakeSession` is one-shot. It converts the terminal packet into an ordered list of protocol
installation, refusal, and close steps:

| Intent | Condition | Ordered result |
|---|---|---|
| status | replies enabled and cached snapshot present | install status clientbound, then status serverbound |
| status | disabled or snapshot absent | install status clientbound, then close |
| login | protocol `776` | install login clientbound, then login serverbound with `transferred=false` |
| login | protocol mismatch | install login clientbound, send classified disconnect, close |
| transfer | disabled | install login clientbound, send transfers-disabled disconnect, close |
| transfer | enabled and protocol `776` | install login clientbound, then login serverbound with `transferred=true` |
| transfer | enabled and protocol mismatch | use the ordinary classified login refusal |

Status deliberately does not compare the protocol field. Login values below `754` select the
outdated-client refusal; every other value unequal to `776` selects incompatible-version. A second
handshake intention cannot be routed because the first complete frame has already selected or
closed the connection protocol.

The transition is a plan rather than several booleans so a gateway/runtime adapter must preserve
direction installation order. In particular, a login disconnect is encoded only after login
clientbound has been installed, and the next inbound frame is decoded only after the selected
serverbound protocol has been installed.

## Evidence

`crates/ferrite-protocol/tests/c0/handshake_serverbound.rs` owns the family conformance. It verifies
the independent intention golden, ASCII and three-byte BMP host limits, malformed UTF replacement,
empty host, both port endpoints, illegal intents, unknown IDs, trailing data, all status protocol
versions, both status-availability failures, login cutoff classification, transfer gating,
directional step order, and terminal one-shot behavior.

Primary locked-source anchors are `ClientIntentionPacket`, `ClientIntent`,
`ServerHandshakePacketListenerImpl`, `Connection`, and the generated handshake packet report lane.
