# Framing and Primitive Encodings

This page specifies the transport from TCP connection establishment through the C1 compression
envelope. It is version-locked to Java Edition `26.2`, protocol `776`. Compression and encryption
are not active anywhere in C0; login may install them only at the insertion points specified below
and in [`login-and-configuration.md`](login-and-configuration.md).

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

## C1 compression envelope

When login compression is enabled, the ordinary packet body is wrapped before the outer length
prefix:

```text
frame := frame_length:VarInt compression_body[frame_length]
compression_body := data_length:VarInt (packet_body | zlib(packet_body))
```

`data_length = 0` selects the raw packet body. A nonzero value declares the exact inflated packet
body length and the remaining frame is a zlib stream. The official encoder emits raw form when body
length is below the negotiated threshold and compressed form at or above it. It refuses an input
body above `8_388_608` bytes. The server decoder validates every nonzero declaration as at least the
threshold and at most `8_388_608`, and requires inflation to produce exactly the declaration. It
does not reject `data_length = 0` merely because the raw body meets or exceeds the threshold.

The outer frame remains subject to the three-byte, nonzero `frame_length` rule, so compressed bytes
plus `data_length` must still fit at most `2_097_151` bytes. The login-compression negotiation frame
itself uses the uncompressed C0 grammar. Compression begins only after that frame's send completion
callback; it then persists across login-finished, login acknowledgement, configuration, and play.
A negative configured threshold sends no negotiation packet and leaves the C0 grammar active. A
threshold of zero compresses every subsequently encoded nonempty packet body.

Primary anchors are `net.minecraft.network.CompressionEncoder#encode`,
`net.minecraft.network.CompressionDecoder#decode`,
`net.minecraft.network.CompressionDecoder#inflate`, and
`net.minecraft.network.Connection#setupCompression`.
