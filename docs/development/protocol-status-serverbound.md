# Required Status Serverbound Protocol

Ferrite implements both packets in the locked C0 status serverbound lane:

| Identity | ID | Payload |
|---|---:|---|
| `minecraft:status_request` | `0` | fieldless |
| `minecraft:ping_request` | `1` | signed big-endian 64-bit opaque token |

The generated status/serverbound catalog owns dispatch. Unknown IDs, truncated ping tokens, and
trailing bytes fail closed. Ping encoding and decoding preserve all 64 bits for every signed value.

## Cached snapshot lifecycle

`StatusServerSession` is created with the immutable normalized snapshot selected at handshake time.
It owns one request-handled flag and has no gameplay, ECS, Region, replay, or persistence mutation.

The first status request sets the flag before returning exactly one clientbound status response
containing that cached snapshot. The session remains open for ping. A second request sends no
response and closes with the internal request-handled outcome.

A ping is legal both before and after the status request. It returns one clientbound pong carrying
the exact signed-long token. The session then enters `PongPending`; it cannot close or process
another packet until the pong send-completion callback is received. That callback produces the
close action. This makes pong-before-close ordering an API invariant instead of a caller
convention.

The status request and ping have no shared correlation ID. A ping before status sends only pong,
leaves the request flag false, and follows the same send-then-close sequence.

## Conformance evidence

`crates/ferrite-protocol/tests/c0/status_serverbound.rs` locks:

- official empty-request and recognizable signed-long ping frames;
- signed-long endpoints, nonminimal packet ID, truncation, trailing bytes, and unknown-ID refusal;
- cached-snapshot identity, request flag timing, one-response behavior, and duplicate closure;
- ping-before-status behavior and send-completion-gated closure;
- the full response, exact pong echo, then close happy trace using both direction codecs.
