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

## C1 login golden frames

These vectors use name `Player`, a zero supplied UUID, the derived offline UUID
`a01e3843-e521-3998-958a-f459800e4d11`, an empty property map, and a zero server-session UUID.
The locked official packet codecs produced every packet body.

| Vector | Envelope and exact bytes |
|---|---|
| `C1-GOLD-LOGIN-HELLO` | Uncompressed serverbound frame: `180006506c6179657200000000000000000000000000000000` |
| `C1-GOLD-COMPRESSION-256` | Negotiation remains uncompressed: `03038002` |
| `C1-GOLD-LOGIN-FINISHED-RAW` | Compression disabled: `2902a01e3843e5213998958af459800e4d1106506c617965720000000000000000000000000000000000` |
| `C1-GOLD-LOGIN-FINISHED-C256` | Threshold 256, raw form inside compression envelope: `2a0002a01e3843e5213998958af459800e4d1106506c617965720000000000000000000000000000000000` |
| `C1-GOLD-LOGIN-ACK-RAW` | Compression disabled: `0103` |
| `C1-GOLD-LOGIN-ACK-C256` | Threshold 256: `020003` |

## C1 login boundaries and traces

| Vector | Stimulus | Required oracle |
|---|---|---|
| `C1-LOGIN-OFFLINE-TRACE` | Login intention `776`; `C1-GOLD-LOGIN-HELLO`; default threshold negotiation; compressed-envelope finished and acknowledgement. | Ignore supplied zero UUID, normalize to the derived UUID, preserve empty properties, switch each direction only at its terminal packet, then start configuration. |
| `C1-NAME-BOUNDARIES` | Independently send empty, 16 printable ASCII, 17 ASCII, a space, DEL, and a non-ASCII code unit. | Empty and 16 printable units pass; 17 fails codec; space, DEL, and non-ASCII fail handler validation. |
| `C1-SUPPLIED-UUID-IGNORED` | Repeat the same offline name with zero, random, and the derived UUID in hello. | Produce the same derived authoritative UUID in all three sessions. |
| `C1-OFFLINE-UUID-CASE` | Independently log in as `Player` and `player`. | Derive distinct UUIDs because the exact name bytes, including case, feed `OfflinePlayer:` MD5 input. |
| `C1-HELLO-DUPLICATE` | Send two login hello packets before success. | First leaves `HELLO`; second is unexpected and the connection fails without a second identity transition. |
| `C1-ACK-EARLY-DUPLICATE` | Send acknowledgement before finished, then independently send a second after transition. | Reject each under its then-current packet state; only one acknowledgement in `PROTOCOL_SWITCHING` starts configuration. |
| `C1-LOGIN-TIMEOUT` | Keep a login connection incomplete through the tick whose prior counter value is 600. | Send slow-login disconnect and close. |
| `C1-ADMISSION-DENIED` | Make admission policy return a component after offline normalization. | Send ID-0 login disconnect containing that component, then close without finished. |
| `C1-COMPRESSION-DISABLED` | Configure threshold `-1`. | Send no compression packet; finished and acknowledgement match raw goldens. |
| `C1-COMPRESSION-ZERO` | Configure threshold `0`. | Negotiation is uncompressed; every later packet body is zlib form with nonzero declared length. |
| `C1-COMPRESSION-BOUNDARY` | For threshold 256, encode bodies of 255 and 256 bytes. | Encoder uses `data_length = 0` for 255 and zlib with declaration 256 for 256. |
| `C1-COMPRESSION-DECLARATION` | Serverbound nonzero declarations below threshold, above 8,388,608, wrong inflated size, and invalid zlib. | Reject each and close. A zero declaration remains accepted even when its raw body length is at least the threshold. |
| `C1-PROFILE-PROPERTIES` | Generate 16 and 17 properties; boundary property strings; nullable signatures; malformed counts. | Accept the complete 16-entry bounded profile, reject 17 and every over-limit/truncated field without entering configuration. |
| `C1-SESSION-ID` | Complete two overlapping logins, then drain all connections and complete another. | The overlapping finished packets share one non-profile session UUID; after the list empties, the later session lazily receives a new random UUID. |
| `C1-ONLINE-GATE` | Enable authentication and complete hello/key with a controlled RSA pair, challenge, AES secret, and session-service result. | Require exact challenge echo, enable stream encryption after key, use authenticated profile on success, and disconnect on bad challenge, invalid session, or unavailable authentication according to the locked dedicated/integrated branch. |
| `C1-CUSTOM-QUERY-GATE` | Inject a clientbound query up to 1,048,576 raw bytes, observe vanilla null answer, then send any answer to the base server. | Client correlates transaction ID; base server sends unexpected-query disconnect. Reject oversized payload/remainder. |
| `C1-COOKIE-GATE` | Inject request key `minecraft:test`; test absent, 0-byte, 5,120-byte and 5,121-byte values; send unsolicited response to base server. | Client echoes key with nullable bounded value; reject 5,121 bytes; base server disconnects on every response. |

## C1 configuration golden frames

Every vector in this table is a complete post-negotiation frame using threshold 256. The packet
bodies are below threshold, so each envelope contains `data_length = 0`. The locked official packet
codecs produced every body; framing uses the independently specified compression grammar.

| Vector | Direction / fixture | Exact frame bytes |
|---|---|---|
| `C1-GOLD-CONFIG-BRAND-CB` | Clientbound brand `vanilla` | `1a00010f6d696e6563726166743a6272616e640776616e696c6c61` |
| `C1-GOLD-CONFIG-BRAND-SB` | Serverbound brand `vanilla` | `1a00020f6d696e6563726166743a6272616e640776616e696c6c61` |
| `C1-GOLD-CONFIG-CLIENT-INFO` | Serverbound `en_us`, view 2, full chat, colors, model 0, right hand, no filtering/listing, all particles | `10000005656e5f75730200010001000000` |
| `C1-GOLD-CONFIG-FEATURES` | Clientbound singleton `minecraft:vanilla` | `15000c01116d696e6563726166743a76616e696c6c61` |
| `C1-GOLD-CONFIG-KNOWN-OFFER` | Clientbound empty known-pack offer | `03000e00` |
| `C1-GOLD-CONFIG-KNOWN-RESPONSE` | Serverbound empty selection | `03000700` |
| `C1-GOLD-CONFIG-REGISTRY` | Clientbound empty `minecraft:timeline` registry codec fixture | `160007126d696e6563726166743a74696d656c696e6500` |
| `C1-GOLD-CONFIG-TAGS` | Clientbound empty registry/tag map | `03000d00` |
| `C1-GOLD-CONFIG-FINISH` | Clientbound or serverbound terminal finish | `020003` |
| `C1-GOLD-CONFIG-KEEPALIVE` | Either direction token `0x0102030405060708` | `0a00040102030405060708` |
| `C1-GOLD-CONFIG-PING` | Clientbound token `0x01020304` | `06000501020304` |
| `C1-GOLD-CONFIG-PONG` | Serverbound token `0x01020304` | `06000501020304` |

The empty registry vector is a codec fixture, not a complete registry set. A happy session must
provide all content not covered by an exactly accepted known pack before finish.

## C1 configuration boundaries and traces

| Vector | Stimulus | Required oracle |
|---|---|---|
| `C1-CONFIG-HAPPY-TRACE` | Continue `C1-LOGIN-OFFLINE-TRACE`; exchange both brand packets and client information; send vanilla feature, empty offer/response, complete NBT for all 29 synchronized registries, tags, finish and acknowledgement. | Preserve direction-local order and compression, construct identical client registry IDs/tags, wait for spawn readiness, switch each direction only at its terminal packet, then decode the next packets as play. |
| `C1-CONFIG-KNOWN-ORDER` | Test empty lists; exact `minecraft:core:26.2`; subset, reordered, unknown and 65-entry responses; early and duplicate responses. | Empty/empty sends all data; exact equality may omit matching entry NBT; every non-equal list sends all data; reject 65 at codec and wrong-task responses without advancing twice. |
| `C1-CONFIG-REGISTRY-BOUNDARIES` | Split one registry across packets; repeat/duplicate element keys; absent/present NBT; all 29 keys; unknown key; malformed tag, depth 512/513 and quota boundaries. | Concatenate split entries in arrival order and make that order numeric IDs; accept valid optional data; reject invalid/duplicate/unknown content no later than finish; enforce default NBT quota/depth and transport limits. |
| `C1-CONFIG-TAG-MAPPINGS` | Encode empty/multiple registry maps, repeated registry keys, empty tags, boundary member VarInts and IDs outside the reconstructed registry. | Resolve each member against its matching registry numeric order; later registry payload replaces earlier; reject invalid IDs/truncation instead of remapping them. |
| `C1-CONFIG-FEATURE-NAMES` | Send every locked name, duplicates, empty and an unknown identifier. | Build the named set, collapse duplicates, permit empty at codec level, and ignore unknown names with a warning; a normal minimum trace includes `minecraft:vanilla`. |
| `C1-CONFIG-CLIENT-INFO-BOUNDARIES` | Exercise language 16/17, signed view byte endpoints, model byte endpoints, all enum ordinals, out-of-range ordinals, and repeated records before finish. | Accept bounded values exactly, reject 17 and invalid enums, and use the latest valid record in the play cookie. |
| `C1-CONFIG-CUSTOM-PAYLOAD-BOUNDS` | Send brand boundary strings and unknown channel remainders at 32,767/32,768 serverbound and 1,048,576/1,048,577 clientbound. | Retain clientbound brand; base server ignores serverbound brand; discard unknown in-bound payloads through each inclusive cap and reject the next byte. |
| `C1-CONFIG-FINISH-ORDER` | Send finish while synchronization, optional task, or spawn preparation is current; then valid finish and a duplicate after transition. | Reject every wrong-task finish; only join-world finish commits the transition; decode the duplicate under play rather than accepting a second configuration transition. |
| `C1-CONFIG-KEEPALIVE-STATE` | At the 15,000 ms boundaries send exact, stale, unsolicited and mismatched long echoes; independently send signed-int ping endpoints. | Exact pending echo clears and updates latency; non-owner invalid echo or a second pending interval disconnects; pong echoes ping bits and otherwise changes no task state. |
| `C1-CONFIG-COOKIE-GATE` | Request/store absent, 0, 5,120 and 5,121-byte cookie values; send a response to the base server. | Client correlates/stores bounded key values, rejects 5,121, and base server disconnects every response as unexpected. |
| `C1-CONFIG-RESOURCE-PACK-GATE` | Push valid/invalid URLs and hash 40/41; exercise all eight actions, required decline, wrong UUID, pop-one/pop-all and unsolicited terminal responses. | Reject hash 41; return invalid-url for bad URL; only action IDs 3/4 are nonterminal; required decline disconnects; current task advances on any terminal UUID, while wrong-task terminal response faults. |
| `C1-CONFIG-CODE-OF-CONDUCT-GATE` | Select exact locale, case variant, `en_us` fallback and first fallback; accept, reject, duplicate document and unsolicited accept. | Choose text in locked fallback order; acceptance advances only the matching task; rejection closes client-side; duplicate/unsolicited cases fault. |
| `C1-CONFIG-CUSTOM-CLICK-GATE` | Send absent/present NBT at 32,768-byte/depth-16 accumulator bounds and 65,536-byte prefix bound, then exceed each. | Dispatch only valid identifier/payload to the owned server handler and reject every exceeded length, quota, depth or malformed tag. |
| `C1-CONFIG-TRANSFER-GATE` | Store cookies then transfer with host/port VarInt endpoints on remote and memory/singleplayer paths. | Remote client closes read-only and starts transfer-intention login carrying cookies; singleplayer transfer faults; codec preserves the unchecked signed port for the resolver. |
| `C1-CONFIG-PRESENTATION-GATES` | Send report maps at 32/33, server links with known/custom labels and valid/invalid URIs, reset chat, clear dialog and valid/invalid dialog NBT. | Enforce report bounds, retain only valid links, and confine reset/dialog/report effects to client state; malformed NBT fails the packet without gameplay mutation. |
