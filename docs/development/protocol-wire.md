# Minecraft 26.2 wire foundation

`ferrite-protocol::java_26_2::wire` owns bytes between TCP and a complete version-local packet body.
It does not dispatch packet IDs, change connection states, or expose wire types to simulation.

The receive path is:

```text
bounded TCP chunks
    -> FrameDecoder (VarInt21 outer length)
    -> compression envelope selected by connection-local state
    -> complete packet body (packet ID plus fields)
```

`FrameDecoder` accepts arbitrary TCP fragmentation and more than one frame per chunk. Its configured
buffer must hold at least one maximum-sized frame, and a push that would exceed that budget faults
before changing buffered data. Frame length is always nonzero, no greater than `2_097_151`, and no
wider than three bytes. Generic VarInt and VarLong decoders accept non-minimal encodings within their
five- and ten-byte widths; encoders always emit the shortest form.

`WireReader` and `WireWriter` provide bounded primitive operations. Strings enforce both the
`3 * N` encoded-byte bound and Java UTF-16 code-unit bound, while ingress uses replacement
characters for malformed UTF-8 as the locked Java decoder does. Boolean ingress treats any nonzero
byte as true. Counted values, byte arrays, remainders, writer output, and fixed-width values all
require an explicit packet-owned bound or output budget.

Compression is installed only by later connection-state code after the negotiation frame's send
completion. Enabled compression emits a raw envelope below the threshold and a zlib envelope at or
above it. Decode accepts `data_length = 0` regardless of threshold, but nonzero declarations must be
at least the threshold, no greater than `8_388_608`, inflate exactly, consume one complete zlib
stream, and contain no trailing compressed bytes. The outer frame remains capped at `2_097_151`.

Every inbound framing, primitive, compression, or residual-data failure uses
`MALFORMED_INPUT_POLICY`: make the connection read-only and close without resynchronizing or
inventing a response. `PacketStreamDecoder` makes this concrete by becoming permanently faulted
after any malformed frame or compression envelope.

The ordinary workspace tests cover locked boundary and golden behavior. Two standalone cargo-fuzz
targets live under `fuzz/`:

```text
CARGO_TARGET_DIR=target/fuzz cargo +nightly fuzz run frame_stream
CARGO_TARGET_DIR=target/fuzz cargo +nightly fuzz run wire_primitives
```

CI formats and compiles both targets with Clippy in the isolated `target/fuzz-check` build class.
