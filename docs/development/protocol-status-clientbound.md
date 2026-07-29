# Required Status Clientbound Protocol

Ferrite implements both packets in the locked C0 status clientbound lane:

| Identity | ID | Payload |
|---|---:|---|
| `minecraft:status_response` | `0` | compact status JSON in `UTF(32767)` |
| `minecraft:pong_response` | `1` | signed big-endian 64-bit opaque token |

Dispatch uses the generated status/clientbound catalog. Unknown IDs, truncated fields, residual
bytes, encoded strings above 98,301 bytes, and decoded strings above 32,767 Java code units fail
closed.

## Normalized status snapshot

The status adapter owns a typed immutable projection rather than exposing authored JSON:

- description component, defaulting to the empty component;
- optional signed player maximum, online count, and UUID/name sample;
- optional version name and signed protocol;
- optional decoded favicon bytes;
- default-false secure-chat enforcement.

Encoding emits compact JSON in the official field order and omits absent/default fields. Empty
status therefore encodes exactly as `{}`. Player samples use canonical lowercase hyphenated UUIDs.
Favicon bytes use standard Base64 after the exact `data:image/png;base64,` prefix.

The decoder requires a syntactically valid object root. Each known field is independently lenient:
an invalid description becomes empty; invalid players, version, or favicon become absent; an
invalid secure-chat value becomes false. Invalid optional sample data becomes an empty sample
without discarding otherwise valid player counts. Unknown fields are ignored. Favicon decoding
removes line-feed characters from the Base64 suffix, matching the locked codec.

## Client presentation and lifecycle

The client projection accepts one status response. It replaces description, player, and version
presentation, records the current signed millisecond value, and requests a ping carrying those
exact 64 bits. A second response closes as unrequested.

An absent favicon leaves the existing saved icon untouched. Present bytes equal to the saved icon
also do nothing. Changed bytes are validated with the locked `PngInfo` header checks: the first
chunk must be a 13-byte `IHDR`, and signed width and height must each be at most 1,024. Invalid or
oversized changed bytes clear the icon and still report a persistent-data change. This deliberately
preserves the official signed-dimension behavior rather than adding a positive-size rule.

Pong handling ignores the returned token when computing presentation latency. It subtracts the
stored local ping start from the current signed millisecond value with Java-compatible wrapping,
then closes and runs completion. A pong before any response therefore uses the zero-initialized
start and retains the client's legacy-fallback-on-disconnect state.

The server-side cached-snapshot request and exact pong echo lifecycle are owned by the following
status serverbound family. Neither status response content nor timing is authoritative gameplay,
Region, ECS, replay, or persistence state.

## Conformance evidence

`crates/ferrite-protocol/tests/c0/status_clientbound.rs` locks:

- official minimal, populated, and pong frames;
- compact field order, default omission, signed integer boundaries, UUID samples, structured
  descriptions, secure-chat, favicon Base64, and round trips;
- malformed optional degradation, malformed/non-object roots, exact UTF limits, truncation,
  trailing bytes, and unknown IDs;
- valid, oversized, negative-dimension, malformed, absent, and retained favicon presentation;
- first-response ping, duplicate-response closure, token-independent latency, pong completion, and
  the pre-response pong fallback branch.
