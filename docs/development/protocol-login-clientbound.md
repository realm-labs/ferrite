# Required Login Clientbound Protocol

Ferrite implements the three required Minecraft Java `26.2` clientbound login packets as a closed,
catalog-dispatched family. Packet IDs are resolved from the locked catalog rather than duplicated
in the codec:

| Identity | Locked ID | Implemented payload |
|---|---:|---|
| `minecraft:login_disconnect` | `0` | lenient component JSON in `UTF(262144)` |
| `minecraft:login_finished` | `2` | bounded game profile followed by the server-session UUID |
| `minecraft:login_compression` | `3` | signed VarInt threshold |

IDs `1`, `4`, and `5` belong to optional online-mode, custom-query, and cookie behavior. The
required-family decoder recognizes those catalog entries but fails closed until their separately
owned families are implemented. IDs absent from the locked lane also fail closed.

## Profile and component boundaries

The profile UUID and the server-session UUID are distinct network-order 128-bit values. A profile
name is limited to 16 UTF-16 code units and its property count to 16. Property names, values, and
optional signatures are respectively limited to 64, 32,767, and 1,024 UTF-16 code units. The codec
preserves property order and the nullable signature marker.

Login disconnect reasons retain their original JSON representation after parsing. A valid root is a
string, a nonempty component array, or a nonempty object. Malformed JSON and scalar roots that
cannot represent a component are rejected before projection.

## Transition ownership

The compression packet itself is always decoded with the pre-negotiation envelope. A nonnegative
threshold produces an explicit `InstallCompressionAfterCurrentPacket` action; the projection does
not expose the new envelope until its callback is acknowledged. A negative threshold is invalid on
the wire-facing client path—the server represents disabled compression by omitting the packet.
Threshold zero is valid and compresses every later nonempty packet.

Login finished starts a deliberately split transition:

1. install the configuration clientbound codec and listener;
2. send login acknowledged while the serverbound login codec is still installed;
3. install the configuration serverbound codec.

Each boundary has its own projection callback, so a caller cannot accidentally collapse or reorder
the codec changes around the acknowledgement. Disconnect is terminal from any live login stage.

## Conformance evidence

`crates/ferrite-protocol/tests/c1/login_clientbound_required.rs` locks:

- the threshold-256 uncompressed negotiation frame;
- login-finished framing before and after compression is installed;
- component, profile-property, UTF-16, count, UUID, and nullable-signature boundaries;
- optional and unknown packet fail-closed behavior;
- compression callback ordering, threshold `0`, disabled-by-omission semantics, and the terminal
  configuration transition.
