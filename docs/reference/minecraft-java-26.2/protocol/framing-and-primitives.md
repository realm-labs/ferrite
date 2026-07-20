# C0 Framing and Primitive Encodings

This page specifies the uncompressed and unencrypted transport used from TCP connection establishment
through status discovery. It is version-locked to Java Edition `26.2`, protocol `776`. Compression
and encryption are not active anywhere in C0; their insertion points and altered frame body are C1
work.

## Frame grammar

Each packet occupies one frame:

```text
frame := frame_length:VarInt packet_body[frame_length]
packet_body := packet_id:VarInt packet_fields
```

`frame_length` counts the packet ID and fields, but not its own prefix. It must be in
`1..=2_097_151`, so its prefix is at most three bytes. A zero length or a prefix whose first three
bytes all carry the continuation bit is a corrupt frame. An incomplete prefix or body waits for
more TCP bytes; end-of-stream before completion closes the connection.

Packet IDs are local to connection state and direction. The decoder reads the ID as a VarInt,
selects the state/direction codec, and requires that codec to consume the complete frame. An ID not
in the selected table, an underflow, or trailing bytes is a packet failure. Encoders emit minimal
VarInts. Decoders intentionally accept non-minimal encodings when they terminate within the width
limit.

These conclusions come from `net.minecraft.network.Varint21FrameDecoder#copyVarint`,
`net.minecraft.network.Varint21FrameDecoder#decode`,
`net.minecraft.network.Varint21LengthFieldPrepender#encode`,
`net.minecraft.network.codec.IdDispatchCodec#decode`,
`net.minecraft.network.codec.IdDispatchCodec#encode`, and
`net.minecraft.network.PacketDecoder#decode` in `OFF-SERVER-001`.

## Primitive table

| Primitive | Locked encoding and bounds |
|---|---|
| `VarInt` | Signed 32-bit value in little-endian base-128 groups, seven payload bits per byte and bit 7 as continuation. Nonnegative values use one through five bytes; negative values use five. More than five consumed bytes is invalid. Decode does not require the shortest representation. |
| `UTF(N)` | A VarInt byte length, followed by that many UTF-8 bytes. The byte length must be nonnegative, no greater than `3 * N`, and no greater than the remaining frame. After UTF-8 decoding, the Java UTF-16 code-unit length must be at most `N`. Encoding checks the same code-unit limit and emits a minimal byte-length VarInt. Java's UTF-8 decoder replaces malformed sequences; higher-level JSON parsing can still reject the resulting text. |
| unsigned short | Two bytes, network byte order, producing `0..=65_535`. |
| signed long | Eight bytes, network byte order, preserving all 64 bits. |

The generic VarInt and string rules are fixed by `net.minecraft.network.VarInt#read`,
`net.minecraft.network.VarInt#write`, `net.minecraft.network.Utf8String#read`, and
`net.minecraft.network.Utf8String#write`. The C0 string instantiations are:

- handshake host: decode limit `N = 255`, hence at most `765` encoded bytes;
- status JSON: encode/decode limit `N = 32_767`, hence at most `98_301` encoded bytes.

The official handshake writer uses the generic `32_767` encoder overload, but the server decoder is
normative for ingress and rejects a decoded host longer than 255 code units. Ferrite must enforce
255 on ingress; independent client-fixture builders must enforce it when constructing test frames
rather than inheriting the official writer's looser construction bound.

## Failure boundary

Framing, dispatch, primitive, JSON, or residual-byte failures enter connection fault handling and
make the channel read-only before closure. Handshake and status have no status-state disconnect
packet with which to report a codec failure. Implementations must therefore close the TCP session;
they must not invent a packet, reinterpret bytes under another state, or continue after the bad
frame. A normal duplicate-request or completed-ping close is specified separately in
[`handshake-and-status.md`](handshake-and-status.md).

Primary fault-path anchors are `net.minecraft.network.Connection#exceptionCaught` and
`net.minecraft.network.Connection#disconnect`. Exact byte and failure oracles are in
[`conformance.md`](conformance.md).
