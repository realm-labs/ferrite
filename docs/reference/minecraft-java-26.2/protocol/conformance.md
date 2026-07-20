# Protocol Conformance Vectors

Vectors are versioned behavioral oracles, not implementation round trips. Hex strings contain one
complete uncompressed frame unless a vector says otherwise. Implementations must preserve the IDs
when automating these cases because the protocol completion ledger refers to them directly.

## C0 golden frames

| Vector | Direction and meaning | Exact frame bytes |
|---|---|---|
| `C0-GOLD-INTENTION-STATUS` | serverbound handshake: protocol `776`, host `localhost`, port `25565`, intent status | `10008806096c6f63616c686f737463dd01` |
| `C0-GOLD-STATUS-REQUEST` | serverbound empty status request | `0100` |
| `C0-GOLD-STATUS-MINIMAL` | clientbound response containing the canonical minimal JSON `{}` | `0400027b7d` |
| `C0-GOLD-STATUS-POPULATED` | clientbound response containing the exact JSON shown below | `6400627b226465736372697074696f6e223a2246657272697465222c22706c6179657273223a7b226d6178223a32302c226f6e6c696e65223a307d2c2276657273696f6e223a7b226e616d65223a2232362e32222c2270726f746f636f6c223a3737367d7d` |
| `C0-GOLD-PING` | serverbound ping token `0x0102030405060708` | `09010102030405060708` |
| `C0-GOLD-PONG` | clientbound bit-identical pong | `09010102030405060708` |

The populated JSON is exactly:

```json
{"description":"Ferrite","players":{"max":20,"online":0},"version":{"name":"26.2","protocol":776}}
```

The locked official codecs produced both status-response byte sequences. The full happy trace is:

1. client sends `C0-GOLD-INTENTION-STATUS`;
2. client sends `C0-GOLD-STATUS-REQUEST` in a separate frame;
3. server sends either valid status golden response;
4. client sends `C0-GOLD-PING`;
5. server sends `C0-GOLD-PONG` and closes after scheduling/sending it.

## C0 framing and dispatch cases

| Vector | Stimulus | Required oracle |
|---|---|---|
| `C0-FRAME-ZERO` | `00` | Reject corrupt zero-length frame and close. |
| `C0-FRAME-WIDE` | `80808000` | Reject as a length prefix wider than 21 bits without treating the fourth byte as packet data. |
| `C0-FRAME-PARTIAL` | Deliver `09 01 01`, pause, then the remaining seven token bytes. | Emit nothing while incomplete; after completion decode one ping, echo it, then close. EOF during the pause closes without a pong. |
| `C0-FRAME-NONMINIMAL-LENGTH` | `810000` (length one encoded in two bytes, then status-request ID zero). | Accept one status request. |
| `C0-ID-NONMINIMAL` | `028000` (two-byte body containing non-minimal ID zero). | Accept one status request. |
| `C0-ID-UNKNOWN` | status-state `0102` | Reject unknown ID `2` and close. |
| `C0-PACKET-TRAILING` | status-state `020000` | Reject the otherwise valid status request because one byte remains in its frame; close. |
| `C0-PING-TRUNCATED` | `080100000000000000` | Frame completes with only seven token bytes; reject the long underflow and close without pong. |

## C0 handshake boundaries

| Vector | Stimulus | Required oracle |
|---|---|---|
| `C0-HOST-BOUNDARY` | Encode hosts of 255 and 256 ASCII code units. | The 255-unit value reaches the selected intent; the 256-unit ingress value fails string decode and closes. |
| `C0-HOST-MULTIBYTE` | Encode 255 three-byte BMP code points (765 UTF-8 bytes), then 256. | Accept the first and reject the second, proving both byte and decoded-code-unit gates. |
| `C0-HOST-MALFORMED-UTF8` | Declare a one-byte host and supply `ff`. | Primitive decode replaces the malformed sequence with one replacement character, so the host bound passes and the selected intent runs. Do not reject this case earlier than the locked decoder. |
| `C0-PORT-BOUNDARY` | Status intentions with ports `0` and `65_535`. | Decode those exact unsigned values and enter status when replies are available. |
| `C0-INTENT-ILLEGAL` | Intent VarInt `-1`, `0`, then independently `4`. | Reject each before listener transition and close. |
| `C0-STATUS-PROTOCOL-OPAQUE` | Status intention with protocol versions `-1`, `775`, `776`, and `777`. | Enter status for all four when replies are available; the status branch does not compare this field. |
| `C0-STATUS-DISABLED` | Valid status intention while replies are disabled or the cached snapshot is absent. | Close without status response. |
| `C0-TRANSFER-GATE` | Transfer intention with transfers disabled. | Change clientbound protocol to login, emit the transfers-disabled login disconnect, and close; do not install status or ordinary login. |

The host generators must use a minimal VarInt byte length and the field order in
[`handshake-and-status.md`](handshake-and-status.md). They are intentionally generators rather than
committed megabyte-scale fixtures.

## C0 status JSON and lifecycle cases

| Vector | Stimulus | Required oracle |
|---|---|---|
| `C0-JSON-MINIMAL` | Decode `C0-GOLD-STATUS-MINIMAL`. | Empty description, absent players/version/favicon, and `enforcesSecureChat = false`. |
| `C0-JSON-LENIENT-OPTIONAL` | Status JSON `{"players":{},"version":{"name":1,"protocol":"x"},"favicon":"bad"}`. | Decode the response with all three malformed optional values absent. |
| `C0-JSON-ROOT` | Independently send `[]` and syntactically malformed `{bad` as the JSON value. | Reject each response; the first fails the status object codec and the second fails JSON parsing. |
| `C0-JSON-LENGTH` | Generate valid object texts of exactly 32,767 and 32,768 Java code units. | Accept the first if the object codec is valid; reject the second at the UTF bound. Also reject an encoded byte-length prefix above 98,301 before JSON parsing. |
| `C0-FAVICON-PRESENTATION` | Valid Base64 PNGs at `1024x1024`, `1025x1`, and `1x1025`, plus invalid PNG bytes. | Vanilla client may retain the first; it converts each other icon to absent presentation data without invalidating the status response. |
| `C0-STATUS-DUPLICATE` | After intention, send two status-request frames. | Send one response for the first, no response for the second, then close. |
| `C0-PING-BEFORE-STATUS` | After intention, send ping token `-1` before any status request. | Send pong with eight `ff` token bytes, then close; no status response. |
| `C0-PING-SIGNED-BOUNDARIES` | Independently ping with `Long.MIN_VALUE`, `-1`, `0`, and `Long.MAX_VALUE`. | Echo all 64 bits exactly in each pong and close each session. |
| `C0-HAPPY-TRACE` | Execute the five-frame trace above against a TCP endpoint. | One response, exact pong echo, server close after pong, and an unmodified `26.2` client records a completed ping rather than falling back to legacy discovery. |

## Evidence and reproduction

Packet identities/IDs are regenerated with:

```sh
cargo run -p mc-reference --bin mc-ref -- reports
cargo run -p mc-reference --bin mc-ref -- protocol inventory
```

Field and state oracles are rechecked with `javap -p -s -c` against the locked jars, focusing on the
symbols listed in [`framing-and-primitives.md`](framing-and-primitives.md) and
[`handshake-and-status.md`](handshake-and-status.md). Golden status bytes must be produced through
`net.minecraft.network.protocol.status.ClientboundStatusResponsePacket`'s locked stream codec, not
through a Ferrite encoder. A Ferrite codec test then consumes those independent bytes, and a raw TCP
session test plus unmodified-client smoke run establishes the end-to-end result.
