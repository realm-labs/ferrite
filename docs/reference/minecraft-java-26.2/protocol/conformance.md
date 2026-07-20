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

## C1 play-entry golden frames

Every frame uses threshold 256. Each body is smaller than the threshold, so the compression
envelope contains `data_length = 0`. The locked Java 25 official packet codecs generated every
body; fixtures deliberately use empty/default nested projections where a complete server normally
sends nonempty registry-derived content.

| Vector | Direction / fixture | Exact frame bytes |
|---|---|---|
| `C1-GOLD-PLAY-LOGIN-MINIMAL` | Clientbound entity 1; overworld raw dimension type 0; max 20; distances 2; survival; no death; sea 63; offline | `480031000000010001136d696e6563726166743a6f766572776f726c6414020200010000136d696e6563726166743a6f766572776f726c64000000000000000000ff000000003f0000` |
| `C1-GOLD-PLAY-DIFFICULTY` | Clientbound normal, unlocked | `04000a0200` |
| `C1-GOLD-PLAY-ABILITIES` | Clientbound default survival flags, fly speed `0.05`, walk speed `0.1` | `0b0040003d4ccccd3dcccccd` |
| `C1-GOLD-PLAY-HELD` | Clientbound hotbar slot zero | `03006900` |
| `C1-GOLD-PLAY-COMMANDS-EMPTY` | Clientbound one-node empty command root | `06001001000000` |
| `C1-GOLD-PLAY-RECIPE-SETTINGS` | Clientbound all four books closed and unfiltered | `0a004c0000000000000000` |
| `C1-GOLD-PLAY-RECIPE-ADD-EMPTY` | Clientbound empty recipe book, replace true | `04004a0001` |
| `C1-GOLD-PLAY-RECIPES-EMPTY` | Clientbound empty property map and stonecutter list | `050085010000` |
| `C1-GOLD-PLAY-PLAYER-INFO-EMPTY` | Clientbound all-action mask and zero entries | `040046ff00` |
| `C1-GOLD-PLAY-POSITION-ZERO` | Clientbound challenge 1; zero position, motion, rotation and relative mask | `3f004801000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000` |
| `C1-GOLD-PLAY-GAME-LOAD-START` | Clientbound event 13, parameter zero | `0700260d00000000` |
| `C1-GOLD-PLAY-BORDER-DEFAULT` | Clientbound new default `WorldBorder` | `29002b00000000000000000000000000000000418c9c3700000000418c9c370000000000f086a70e050f` |
| `C1-GOLD-PLAY-SPAWN` | Clientbound `minecraft:overworld`, `(0,64,0)`, yaw/pitch zero | `260061136d696e6563726166743a6f766572776f726c6400000000000000400000000000000000` |
| `C1-GOLD-PLAY-TIME-EMPTY` | Clientbound game time zero and empty clock map | `0b0071000000000000000000` |
| `C1-GOLD-PLAY-TICKING` | Clientbound 20 ticks/s and unfrozen | `07007f41a0000000` |
| `C1-GOLD-PLAY-TICK-STEP` | Clientbound zero remaining frozen steps | `0400800100` |
| `C1-GOLD-PLAY-ACCEPT` | Serverbound teleport challenge 1 | `03000001` |
| `C1-GOLD-PLAY-POSROT-ECHO` | Serverbound zero position/rotation and clear flags | `23001f000000000000000000000000000000000000000000000000000000000000000000` |

The empty command and recipe fixtures prove their outer grammars, not the locked server's full
command or recipe content. A real session must project its selected registries, permissions,
recipes, player identity, clocks, dimension, and authoritative spawn values.

## C1 play-entry boundaries and traces

| Vector | Stimulus | Required oracle |
|---|---|---|
| `C1-PLAY-ENTRY-TRACE` | Continue the valid configuration finish for a new offline player with empty saved recipes, scoreboard, effects and prior-player list; capture through the first C2 chunk-batch start. | Decode play first; receive login before every level-dependent packet; preserve the locked core/permission/recipe/position/player-info/level-info order; acknowledge position; finish state/tick projection before chunks. |
| `C1-PLAY-ENTRY-CONDITIONALS` | Repeat with saved recipes, scoreboard entries, another player, rain, server icon/MOTD and a transferred cookie. | Insert only the documented conditional projections in their locked positions; omit server data on transfer; do not reorder position, self player-info, level info or ticking packets. |
| `C1-PLAY-LOGIN-BOUNDARIES` | Exercise level counts, duplicate level keys, dimension-type raw-ID endpoints, signed VarInts, all game-mode bytes, optional death position, portal cooldown and sea-level endpoints. | Collapse duplicate level keys; reject unknown dimension-type raw IDs/truncation; map modes and optional values exactly; create client level only from a fully decoded record. |
| `C1-PLAY-COMMAND-TREE` | Encode an empty root, literals, every locked argument type, redirects, suggestions, restricted/executable bits, reachable/unreachable out-of-range indices, cycles, type 3, unknown type IDs and truncated type payloads. | Reconstruct valid trees; map type 3/unknown types to inert root placeholders when their lack of payload leaves a valid frame; reject cycles, reachable bad indices, trailing unknown payload and malformed known payload without installing a partial reachable tree. |
| `C1-PLAY-PLAYER-INFO` | Exercise all 256 action masks, zero/multiple entries, add-before-update, unknown update UUID, profile/property bounds, every game mode, nullable display/chat data and invalid chat signatures. | Read selected fields strictly in action-bit order; add before update; ignore update-only unknown UUIDs; validate or clear chat session according to the secure-profile gate; reject codec bounds. |
| `C1-PLAY-RECIPE-STATE` | Send empty and nonempty settings/add/update packets, replace false/true, every display/slot type, direct/tag ingredient sets, optional groups/requirements, unknown raw IDs and malformed nested payloads. | Replace or extend recipe book exactly as flagged; replace synchronized property/stonecutter state; resolve every raw ID through the locked registries and reject unknown/malformed dispatch. |
| `C1-PLAY-REGISTRY-MAPPINGS` | Vary valid and invalid dimension-type, world-clock, item, recipe-category, recipe-display, slot-display, component, trim, potion and command-argument raw IDs. | Resolve against the current configuration registry snapshot only; reject unknown required IDs, apply the explicit command placeholder rule, and never persist or reinterpret a raw ID through another version. |
| `C1-PLAY-POSITION-RELATIVE` | Exercise every relative bit independently and together, higher bits, finite/non-finite components, pitch beyond bounds, velocity rotation, and riding. | Apply the correction immediately without interpolation, use exact absolute/relative math, clamp resulting pitch, ignore higher mask bits, preserve riding behavior, send the two response packets, and reset prediction; malformed transport still faults. |
| `C1-PLAY-TELEPORT-ORDER` | Deliver the initial ID-72 correction, then valid ID-0 acknowledgement and ID-31 echo; independently reverse the two serverbound packets and wait through resend tick 21. | In order, clear pending state before validating the echo; reversed, ignore the echo while pending; after more than 20 ticks issue a fresh incremented challenge and accept only that current ID. |
| `C1-PLAY-TELEPORT-STALE-DUPLICATE` | With challenge 1 pending send 0 and 2, then 1 twice; separately send initial challenge 0 when no correction has ever been pending. | Ignore nonmatching values, accept the first matching response, and disconnect on the second matching response or matching-without-pending state as invalid movement. |
| `C1-PLAY-SIMPLE-STATE` | Exercise difficulty wrap, all ability/event flags, unknown entity/event IDs, invalid held slots, border lerp signs, nullable/invalid icon, clock maps, tick-rate floats and step VarInts. | Apply each documented projection or ignore branch exactly; reject malformed nested values; keep presentation and session projections out of authoritative simulation state. |

## C2 movement and session golden frames

Every frame uses compression threshold 256 and therefore has `data_length = 0`. The locked Java
25 official packet codecs decoded and re-encoded every body before the framing/compression envelope
was applied. Numeric fixtures use zero state except for recognizable echo tokens and one chunk-rate
sample.

| Vector | Serverbound fixture | Exact frame bytes |
|---|---|---|
| `C2-GOLD-SB-CHUNK-BATCH` | ID 11, desired rate `4.0` | `06000b40800000` |
| `C2-GOLD-SB-TICK-END` | ID 13, fieldless | `02000d` |
| `C2-GOLD-SB-CLIENT-INFO` | ID 14, default `en_us`, view 2 | `10000e05656e5f75730200010001000000` |
| `C2-GOLD-SB-KEEPALIVE` | ID 28, token `0x0102030405060708` | `0a001c0102030405060708` |
| `C2-GOLD-SB-POS` | ID 30, zero position and flags | `1b001e00000000000000000000000000000000000000000000000000` |
| `C2-GOLD-SB-POSROT` | ID 31, zero position, rotation and flags | `23001f000000000000000000000000000000000000000000000000000000000000000000` |
| `C2-GOLD-SB-ROT` | ID 32, zero rotation and flags | `0b0020000000000000000000` |
| `C2-GOLD-SB-STATUS` | ID 33, clear flags | `03002100` |
| `C2-GOLD-SB-VEHICLE` | ID 34, zero pose and on-ground false | `230022000000000000000000000000000000000000000000000000000000000000000000` |
| `C2-GOLD-SB-PADDLE` | ID 35, both false | `0400230000` |
| `C2-GOLD-SB-ABILITIES` | ID 40, not flying | `03002800` |
| `C2-GOLD-SB-COMMAND` | ID 42, entity 1, start sprinting, data 0 | `05002a010100` |
| `C2-GOLD-SB-INPUT` | ID 43, all input false | `03002b00` |
| `C2-GOLD-SB-LOADED` | ID 44, fieldless | `02002c` |
| `C2-GOLD-SB-PONG` | ID 45, token `0x01020304` | `06002d01020304` |

`C2-GOLD-SERVERBOUND-MOVEMENT` is the aggregate assertion over all 15 rows above.

| Vector | Clientbound fixture | Exact frame bytes |
|---|---|---|
| `C2-GOLD-CB-KEEPALIVE` | ID 44, token `0x0102030405060708` | `0a002c0102030405060708` |
| `C2-GOLD-CB-VEHICLE` | ID 57, zero pose | `2200390000000000000000000000000000000000000000000000000000000000000000` |
| `C2-GOLD-CB-PING` | ID 61, token `0x01020304` | `06003d01020304` |
| `C2-GOLD-CB-ROTATION` | ID 73, zero absolute yaw/pitch | `0c004900000000000000000000` |

`C2-GOLD-CLIENTBOUND-SESSION` is the aggregate assertion over these four rows. Clientbound ID 32
disconnect is covered with structural NBT boundary vectors instead of a context-free default
component fixture.

## C2 movement and session boundaries and traces

| Vector | Stimulus | Required oracle |
|---|---|---|
| `C2-MOVEMENT-FORMS-CADENCE` | Drive sub-threshold/threshold position deltas, changed rotations, ground/collision transitions, 20 unchanged ticks, riding and an unpaused/paused client tick. | Select at most one appropriate player form per active nonriding tick and none when unchanged before the reminder; send position at squared delta above `(2e-4)^2` or reminder 20; riding sends rotation plus locally authoritative vehicle state; send tick-end only while unpaused. |
| `C2-MOVEMENT-VALIDATION` | For every player form exercise normal movement, NaN/infinite coordinates/rotations, horizontal/vertical clamp endpoints, packet counts 5/6, speed thresholds, collision residual around `0.0625`, exemptions and positive takeoff. | Disconnect NaN position/non-finite rotation; clamp position infinities/endpoints; enforce exact speed/collision branches; preserve the always-zero residual-Y `||` defect; issue ID-72 correction without committing rejected position. |
| `C2-MOVEMENT-TELEPORT-LOAD-GATES` | Send valid/invalid movement before load, during pending teleport, after ID 44, after 60 ticks without ID 44, while passenger/sleeping/won-game, and with rotation during a pending challenge. | Invalid values fault before gates; otherwise suppress as specified; open initial/respawn gates only by ID 44 or post-restart expiry; retain passenger/server position; apply only pending-time rotation; preserve C1 acknowledgement/resend state. |
| `C2-MOVEMENT-FLOATING` | Sustain requested Y deltas around `-0.03125` with/without support, nearby blocks and every exemption; vary gravity below/at/above `1e-5`; repeat for controlled vehicles. | Enter/reset floating exactly by predicate; disconnect only after more than the gravity-scaled limit; disable the limit below gravity threshold; reset on player/vehicle lifecycle exclusions. |
| `C2-VEHICLE-CORRECTION` | Send vehicle poses for no vehicle, wrong/noncontrolled/root-changed vehicle, valid control, speed above 100, residual around `0.0625`, new collision, NaN/infinity and singleplayer owner. | Ignore identity/control mismatches; reject invalid values; clamp accepted infinity; send ID 57 on speed/collision rejection; update authoritative pose, ground/fall/known movement and chunk tracking only on success. |
| `C2-INPUT-COMMAND-ABILITIES` | Toggle every input/ability bit including high bits; send every command action with wrong entity ID, data endpoints, unloaded state, incapable/capable vehicles, sleep and fall-flight branches; use invalid enum ordinals. | Decode documented bits only; retain pre-load input but defer shift/idle side effects; gate flying by may-fly; ignore command entity ID and unused data; apply each loaded command branch; reject invalid enums. |
| `C2-CLIENT-TICK-END` | End intervals with zero, player, vehicle and multiple movements; repeat tick-end; use movement squared length around `1e-5`. | Preserve the most recent known movement when any known-movement path ran; otherwise set zero; clear the interval every time; reset idle only above the exact movement threshold; never advance the server tick from this packet. |
| `C2-TERRAIN-READY` | Observe initial, death, respawn and duplicate ID-44 flows; omit ID 44 through timer expiry; send it before actual chunks in an adversarial client. | Start/restart 60-tick grace as specified, keep death gate closed until respawn, open idempotently on ID 44 or expiry, and never treat it as proof of a named chunk/batch. |
| `C2-CHUNK-BATCH-FEEDBACK` | Acknowledge no/one/multiple outstanding batches with NaN, infinities and values around `0.01`/`64`; trace the client estimator with zero and positive batch sizes. | Floor outstanding count at zero; map NaN to minimum and clamp all other values; restore quota when empty and expand in-flight cap to ten; reproduce clamped weighted estimator and batch-finish ordering. |
| `C2-PLAY-LIVENESS` | At 15-second boundaries exercise exact, stale, mismatched, unsolicited and missing signed-long echoes for remote/owner sessions; freeze/unfreeze client rendering past one minute; independently use signed-int ping endpoints. | Maintain one keepalive challenge; match exact bits and latency formula; timeout invalid remote flows; exempt owner; defer/drop frozen echo at one minute; always echo ping in pong and never clear keepalive with pong. |
| `C2-PLAY-DISCONNECT` | Send valid literal/translatable/nested reasons, malformed NBT, depth 512/513, trailing data and a reason at the transport bound. | Close with the fully decoded trusted context-free reason; reject malformed, over-deep, trailing or over-frame data without creating a gameplay event. |
| `C2-PLAYER-ROTATION` | Exercise all four relativity combinations, pitch endpoint/overflow, yaw wrap-sized values, repeated packets and every non-finite float. | Apply exact relative math and pitch clamp immediately; synchronize old rotation; send one ID-32 response with false movement flags; preserve codec acceptance and server-side rejection of any non-finite response. |
| `C2-VEHICLE-CORRECTION-CLIENT` | Deliver ID 57 with no/current/nonlocal vehicle, position distance around `1e-5`, active interpolation, rotation-only changes, and exceptional floats. | Ignore nonqualifying vehicle; compare against interpolation target; cancel/snap only above the position threshold; ignore rotation-only correction but still echo; preserve documented NaN/infinity branch behavior. |
| `C2-SESSION-EXCEPTIONAL-FLOATS` | Cross-product finite, NaN and infinities through both correction codecs and their mandatory echoes. | Accept all codec bit patterns, then follow handler-specific install/ignore/clamp and response behavior; do not replace semantic validation with transport rejection. |

## C2 terrain golden frames

These threshold-256 frames use `data_length = 0`. The locked Java 25 official codecs decoded and
re-encoded the eight ordinary packet bodies; the play protocol's official unit delimiter codec
produced the fieldless ID-0 body.

| Vector | Clientbound fixture | Exact frame bytes |
|---|---|---|
| `C2-GOLD-CB-BUNDLE-DELIMITER` | ID 0, fieldless | `020000` |
| `C2-GOLD-CB-BATCH-FINISHED` | ID 11, zero-size informational fixture | `03000b00` |
| `C2-GOLD-CB-BATCH-START` | ID 12, fieldless | `02000c` |
| `C2-GOLD-CB-BIOMES-EMPTY` | ID 13, empty record list | `03000d00` |
| `C2-GOLD-CB-FORGET-ZERO` | ID 37, chunk `(0,0)` | `0a00250000000000000000` |
| `C2-GOLD-CB-LIGHT-EMPTY` | ID 48, chunk `(0,0)`, empty masks/lists | `0a00300000000000000000` |
| `C2-GOLD-CB-CACHE-CENTER` | ID 94, center `(0,0)` | `04005e0000` |
| `C2-GOLD-CB-CACHE-RADIUS` | ID 95, radius 2 | `03005f02` |
| `C2-GOLD-CB-SIM-DISTANCE` | ID 111, distance 2 | `03006f02` |

`C2-GOLD-CLIENTBOUND-TERRAIN` is the aggregate assertion over these nine rows. Full chunk ID 45
requires the session's dimension and registry context and is tested as a locked-server trace plus
directed structure vectors rather than a misleading empty-section fixture.

## C2 terrain boundaries and traces

| Vector | Stimulus | Required oracle |
|---|---|---|
| `C2-BUNDLE-BOUNDARIES` | Send empty, one-packet, 4,096- and 4,097-subpacket bundles, terminal packets, a closing delimiter, and leave a bundle open. | Withhold until close; deliver one synthetic bundle in exact order; accept empty/4,096; fault the next subpacket or any terminal member; treat the next delimiter as close, never nested content. |
| `C2-CACHE-INTEREST` | Move centers across signed VarInt endpoints; change radius through negative, 2, 32, arithmetic-overflow boundaries and huge values; change simulation distance independently; shrink/expand around present chunks. | Apply raw center/distance; use wrapping int formulas `max(2,radius)+3`, side `2*internal_radius+1` and squared allocation; preserve resulting degenerate/fault branches; retain only exact chunks in a valid new range without per-chunk unload callbacks for omitted old slots; keep simulation distance separate; emit normal server center before tracking difference. |
| `C2-CHUNK-BATCH-TRACE` | Trace initial and moving-player pending sets, nearest selection, quota below/above one, memory connection, unacknowledged cap, zero/negative/mismatched finish values, repeated/unmatched markers, standalone chunks and chunks leaving before/after send. | Normal server emits only nonempty start/chunks/finish batches and counts actual chunks; bounded remote selection sorts by squared distance; client handles standalone/mismatched content without cross-counting and sends feedback after every finish; server removes unsent pending chunks silently and forgets sent live-player chunks. |
| `C2-FULL-CHUNK-PALETTES` | For every configured dimension section exercise block selectors 0, 1..8, 15 and noncanonical signed values; biome selectors 0..3/global; negative/zero/capacity-crossing palette sizes, IDs/indices, exact/short/extra fixed longs, section blob 2,097,152/2,097,153 and trailing isolated bytes. | Read exactly the dimension section count bottom-to-top; apply canonical/local/global mappings and fixed nonstraddling storage; preserve negative local-count deferred missing-entry behavior; reject missing global IDs, short storage, invalid positive capacities and exceeded blob; explicitly ignore extra bytes after all required sections inside the isolated blob. |
| `C2-CHUNK-HEIGHTMAP-BLOCK-ENTITIES` | Exercise negative/zero/multiple heightmap counts, all type IDs plus out-of-range/duplicates, exact/wrong/negative long lengths, negative block-entity count, every 49 type IDs, packed nibble/Y endpoints, null/matching/mismatched tags, NBT quota/depth and unknown type IDs. | Accept negative heightmap count as empty; map unknown type to 0 and later duplicates win; copy exact arrays and recompute wrong lengths; fault negative nested/list lengths; locate block entities by chunk+nibbles/Y; apply only non-null matching-type tag; reject unknown registry type and NBT boundary violations. |
| `C2-LIGHT-MASKS` | Cross all four masks at every ordinary/boundary/out-of-range section; use 0/2,047/2,048/2,049-byte arrays, missing/surplus updates, overlapping data/empty bits, truncated/oversized bitsets, full-chunk and incremental paths. | Consume one exact 2,048-byte layer per in-range data bit in ascending order; data wins overlap; install empty layer for empty-only; fault missing/wrong used data; ignore out-of-range bits/surplus arrays; schedule rebuild only for incremental ID 48 and enable chunk light. |
| `C2-BIOME-REFRESH` | Send zero/multiple records, duplicate coordinates, present/missing/out-of-range chunks, every palette selector/registry boundary, 2,097,152/2,097,153 arrays, short and trailing data. | Replace present exact chunks in list order; warn/skip cache misses but still notify/dirty each coordinate's 3-by-3 render area; enforce dynamic biome mapping and cap; fault short required sections and ignore isolated trailing bytes. |
| `C2-CHUNK-UNLOAD` | Forget present, absent, out-of-range and hash-slot-colliding coordinates with populated sky/block/debug state. | Drop only an exact present cached chunk; always clear named debug/light state, disable its light and mark ordinary sections empty; never unload a colliding different chunk. |
| `C2-TERRAIN-READY-TRACE` | Continue the C1 load-start through cache center, full batches, renderer compilation and ID 44; separately use spectator/dead/outside-height, the client-load-start 30-second deadline before/after load-start, and 0/500-ms close delays; finish a batch without readiness. | Remain waiting-for-server until load-start even after the deadline; then enter waiting-for-player-section carrying that deadline; open on compiled player section or exact exemptions/timeout; honor close delay and send one player_loaded; never equate batch finish with readiness. |

## C2 block-interaction and convergence golden frames

Every frame uses compression threshold 256 and therefore has `data_length = 0`. The locked Java
25 official packet codecs produced every body. Positions are `(0,64,0)`. The use-on fixture hits
the upper face at offsets `(0.5,1.0,0.5)`; clientbound registry fixtures use block stone, block
state stone-default, block-entity type furnace, and an empty compound tag.

| Vector | Serverbound fixture | Exact frame bytes |
|---|---|---|
| `C2-GOLD-SB-PICK-BLOCK` | ID 36, include data false | `0b0024000000000000004000` |
| `C2-GOLD-SB-PLAYER-ACTION` | ID 41, start destroy, up, sequence 1 | `0d00290000000000000000400101` |
| `C2-GOLD-SB-SWING` | ID 63, main hand | `03003f00` |
| `C2-GOLD-SB-USE-ON` | ID 66, main hand, upper-face hit, sequence 2 | `1b0042000000000000000040013f0000003f8000003f000000000002` |
| `C2-GOLD-SB-USE` | ID 67, main hand, sequence 3, zero rotation | `0c004300030000000000000000` |

`C2-GOLD-SERVERBOUND-BLOCK` is the aggregate assertion over those five rows.

| Vector | Clientbound fixture | Exact frame bytes |
|---|---|---|
| `C2-GOLD-CB-BLOCK-ACK` | ID 4, sequence 3 | `03000403` |
| `C2-GOLD-CB-BLOCK-DESTRUCTION` | ID 5, breaker 1, progress 5 | `0c000501000000000000004005` |
| `C2-GOLD-CB-BLOCK-ENTITY` | ID 6, furnace type 0, empty compound | `0d00060000000000000040000a00` |
| `C2-GOLD-CB-BLOCK-EVENT` | ID 7, action 1, parameter 2, stone block ID 1 | `0d00070000000000000040010201` |
| `C2-GOLD-CB-BLOCK-UPDATE` | ID 8, stone-default state ID 1 | `0b0008000000000000004001` |
| `C2-GOLD-CB-SECTION-UPDATE-EMPTY` | ID 84, section `(0,0,0)`, zero changes | `0b0054000000000000000000` |

`C2-GOLD-CLIENTBOUND-BLOCK` is the aggregate assertion over those six rows.

## C2 block-interaction and convergence boundaries and traces

| Vector | Stimulus | Required oracle |
|---|---|---|
| `C2-BLOCK-CODEC-BOUNDARIES` | Cross packed block/section coordinate bit endpoints, both hands, all action ordinals, every player-action direction byte, block-hit directions `0..6`, booleans, signed sequences, float bit patterns, truncated fields and trailing bytes. | Preserve exact field order and packed sign extension; map action direction modulo six but reject strict hand/action/hit-direction ordinals; accept float bits at codec level; reject malformed/trailing data. |
| `C2-BLOCK-SEQUENCE-LIFECYCLE` | Send destroy/use-on/use sequences zero, increasing, duplicate, decreasing, negative and near signed overflow before/after listener ticks; interleave unsequenced actions and requests dropped by the load gate. | Register only the three predictive paths; fault negative use before action but negative destroy after its handler; coalesce maximum only until next tick; ACK at tick start then reset; permit a later smaller ACK; send no ACK for unloaded drops; allow later cumulative release. |
| `C2-BLOCK-PREDICTION-ORDER` | Predict one/multiple positions and repeated writes at one position; deliver single/section updates before ACK, ACK before update, duplicate/stale/future ACKs, teleport before ACK, collision restoration, and absent chunks. | Preserve first old state/player position and latest sequence/server state; stage pre-ACK updates; release entries `<=ACK` in locked fastutil order; apply post-ACK updates immediately; gate position snap by teleport sequence and collision; never replay removed predictions. |
| `C2-BLOCK-USE-ON-ADMISSION` | Exercise unloaded client, negative sequence, current reach attribute/creative modifier with AABB distance immediately below/at its padded square, per-axis hit offsets immediately inside/at `1.0000001`, NaN/infinity, min/max Y, spawn protection, pending teleport, `mayInteract`, spectator, secondary use, cooldown, consuming/nonconsuming results and redirected placement. | Register at the documented point; enforce strict padded AABB reach and each early/no-correction branch; use block/empty-hand/item precedence; emit messages exactly; send hit then adjacent corrections on every common-tail path; leave other changed positions to ordinary deltas. |
| `C2-BLOCK-SERVER-SWING-QUIRK` | Return client-, server-, and no-swing results from use-on/use-in-air while idle and before/at half swing duration; attempt nonconsuming upper/lower-boundary placement; deliver ID 63 before/after load and as spectator. | For use-on, publish two self-inclusive animations when the first swing qualifies and zero when an early active swing suppresses it; duplicate the upper build-limit message but not the lower; make one qualifying server swing for use-in-air; use tracker-only ID-63 animation and reset attack strength even when that animation is suppressed. |
| `C2-BLOCK-BREAK-ACTIONS` | Run start/stop/abort through range, height, protection, adventure, creative, instant, active, delayed, mismatch, commit denial and successful mutation; vary the ignored direction and sequences. | Match `BLK-BREAK-001`; send direct denial correction only on its named paths; register after handler return; publish cracks to other players only; converge accepted writes through authoritative deltas. |
| `C2-BLOCK-AUXILIARY-ACTIONS` | Send swap, both drops, release and stab in loaded/unloaded and spectator modes with arbitrary position/direction/sequence; vary item attack gate and piercing component. | Apply only the documented loaded branch, ignore unused wire fields, reset idle at admission, never create a block ACK, and preserve the stab/component and spectator gates. |
| `C2-BLOCK-PICK` | Pick immediately below/at padded AABB reach, unloaded position, empty/disabled clone, matching hotbar/main inventory, no match in survival/creative, and include-data false/true with a component-bearing block entity. | Enforce strict range/load without client-loaded gate; authorize data only for infinite materials; preserve typed custom data/component order; select/add exactly; send held-slot/menu projection only after an enabled nonempty result. |
| `C2-BLOCK-USE-IN-AIR` | Use empty/disabled/cooldown, same/different object, changed count/damage, positive/nonpositive duration, failed and continuing stacks in every game mode; use finite wrap/clamp boundaries and NaN/infinite rotations during/outside pending teleport. | ACK before validation; reset idle; pass spectator/cooldown; install valid wrapped yaw/clamped pitch, discard non-finite setters without movement disconnect; reproduce the exact fast-return/hand/full-menu-resync predicates; send only the result-selected swing and ordinary world deltas. |
| `C2-BLOCK-DELTA-MAPPINGS` | Send IDs 8/84 with every state-table endpoint, duplicate section positions, negative/huge counts, signed/oversized VarLong states, missing chunks and retained predictions; replace a staged invalid state before/not before ACK; independently substitute block and block-entity raw IDs. | Resolve only the 32,366-state table; apply wire order and later duplicates; fault invalid ID-8 at decode and invalid ID-84 on immediate or deferred write unless replaced before release; preserve exact count/cache/prediction behavior; never cross-map the three registries. |
| `C2-BLOCK-ENTITY-DATA` | Send all 49 type IDs to absent, matching and mismatched entities; exercise empty/unknown/type-valid tags, non-compound NBT, depth 512/513, packet bound, and state-delta then data order. | Load only a matching current typed entity; ignore absence/mismatch; use trusted non-null compound grammar with no 2-MiB subquota; reject invalid registry/NBT; preserve state-before-data ordering. |
| `C2-BLOCK-EVENTS` | Queue events for ticking/nonticking positions, changed block types and trigger false/true; send every byte parameter, invalid block IDs, cache miss and a client state different from the packet block. | Defer server event until tickable; broadcast only matching successful trigger within radius; validate raw block ID but invoke the client's current state without comparing the packet block; keep parameters block-specific. |
| `C2-BLOCK-DESTRUCTION-PROGRESS` | Publish server stages around `-1`, `0..10`, `255..266` and int endpoints; use multiple breaker IDs/positions, same-ID relocation, breaker/nonbreaker viewers, distance squared around 1024, absent chunks, and expiry scans at age 400/401. | Encode the low byte; retain exactly when `(stage & 255)` is `0..9` and clear otherwise, including wrapped reappearance at 256; key/relocate by breaker ID, exclude breaker and require strict range, expire only on 20-tick scans after age 400, and keep cracks unacknowledged/chunk-independent. |
| `C2-BLOCK-END-TO-END` | Continue a loaded C2 session through rejected placement, accepted multi-position placement, rejected break, instant break and delayed break while capturing IDs 4/5/6/7/8/84 and inventory/animation side effects. | Reproduce each immediate-update-before-ACK and ACK-before-chunk-delta branch, cumulative sequence release, state-before-block-entity ordering, other-player cracks, successful events, final authoritative world/client equality, and no raw wire identity in simulation persistence. |

## C3 entity interaction and session golden frames

Every frame uses compression threshold 256 and therefore has `data_length = 0`. The locked Java
25 official codecs encoded the six serverbound packets. The interact fixture uses main hand, an
entity-origin-relative zero `LpVec3`, and no secondary action; the spectator fixture uses the absent
optional target; UUID is all zero.

| Vector | Serverbound fixture | Exact frame bytes |
|---|---|---|
| `C3-GOLD-SB-ATTACK` | ID 1, target entity 1 | `03000101` |
| `C3-GOLD-SB-CLIENT-COMMAND` | ID 12, perform respawn | `03000c00` |
| `C3-GOLD-SB-INTERACT` | ID 26, entity 1, main hand, zero vector, false | `06001a01000000` |
| `C3-GOLD-SB-PICK-ENTITY` | ID 37, entity 1, include data false | `0400250100` |
| `C3-GOLD-SB-SPECTATOR-ACTION` | ID 62, absent target | `03003e00` |
| `C3-GOLD-SB-TELEPORT-ENTITY` | ID 64, zero UUID | `12004000000000000000000000000000000000` |

`C3-GOLD-SERVERBOUND-ENTITY-SESSION` is the aggregate assertion over those six rows.

The clientbound codec fixture binds `minecraft:player_attack` damage type and
`minecraft:overworld` dimension type to raw ID zero in an explicit frozen registry snapshot. The
respawn fixture uses overworld level key, seed zero, survival, absent prior mode/death location,
portal cooldown zero, sea level 63 and keep byte zero. The animate/camera private field constructors
were exercised by official decode then re-encode before framing.

| Vector | Clientbound fixture | Exact frame bytes |
|---|---|---|
| `C3-GOLD-CB-ANIMATE` | ID 2, entity 1, main-hand swing | `0400020100` |
| `C3-GOLD-CB-DAMAGE` | ID 25, entity 1, configured damage type 0, no cause/direct/position | `0700190100000000` |
| `C3-GOLD-CB-HURT` | ID 42, entity 1, yaw positive zero | `07002a0100000000` |
| `C3-GOLD-CB-RESPAWN` | ID 82, minimal overworld common spawn, keep 0 | `27005200136d696e6563726166743a6f766572776f726c64000000000000000000ff000000003f00` |
| `C3-GOLD-CB-CAMERA` | ID 93, entity 1 | `03005d01` |
| `C3-GOLD-CB-TAKE` | ID 124, source 1, collector 2, amount 3 | `05007c010203` |

`C3-GOLD-CLIENTBOUND-ENTITY-SESSION` is the aggregate assertion over those six rows.

## C3 entity interaction and session boundaries and traces

| Vector | Stimulus | Required oracle |
|---|---|---|
| `C3-ENTITY-INGRESS-CODECS` | Cross signed entity/action/hand/optional IDs, every boolean byte, UUID bit endpoints, canonical/noncanonical `LpVec3` scales and 15-bit fields, zero/near-zero/NaN/infinite canonical inputs, truncation, overlong VarInts and trailing bytes. | Reject invalid client-command ordinals but map every invalid interact hand to main; preserve optional zero/bias wrapping; sanitize/clamp only on canonical LP encode and decode all accepted forms finite; reject malformed/trailing data. |
| `C3-ATTACK-ADMISSION` | Attack missing, border-excluded, valid and invalid-type targets before/after load and as spectator; vary default/custom attack ranges at both closed endpoints, creative/mob factor, piercing component, feature flag and minimum charge around the five-tick tolerance. | Reset idle at the documented point; ignore ordinary gate failures; disconnect only a reached nonpiercing invalid target; apply inclusive component reach and downstream source-specified attack exactly; emit no acknowledgement. |
| `C3-INTERACT-ADMISSION` | Interact with absent, border-excluded, present and part targets around the strict padded AABB boundary using both/invalid hands, every secondary flag, feature-disabled stacks, spectator menu/nonmenu targets and every target/item `InteractionResult`. | Retain idle/shift mutation before target rejection; map hand zero fallback; run target then item precedence, creative restoration, criteria and selected swing exactly; rely only on ordinary authoritative deltas for convergence. |
| `C3-PICK-ENTITY` | Pick absent/removed/near/boundary/far targets with empty/disabled/matching/unmatched results in survival/creative; cross include-data, command permission, Avatar/non-Avatar and no-pick-result branches. | Use no client-loaded/world-border/idle gate; enforce strict padded range; converge inventory like block pick; never attach target state to the item; print Avatar profile only on the independent authorized include-data branch. |
| `C3-SPECTATOR-CAMERA` | Send absent/present optional camera targets across mode/load/range/border/pickable and wrapped IDs; send UUID teleport for missing/current/cross-level entities while self-camera or another camera is active. | Apply ID-62 gates and idle behavior, relocate before ID 93, and ignore absent; for ID 64 require only spectator, scan levels by UUID, reset camera as needed, then use ordinary same/cross-dimension teleport ordering without a wire ACK. |
| `C3-CLIENT-COMMAND` | Send every action/invalid ordinal while alive, dead, post-win and hardcore; repeat stats requests around dirty-set changes; request gamerules with/without permission and with no UI waiting. | Reset idle for every valid action; ignore alive respawn, replace/restart load on accepted branches, drain dirty stats including empty responses, gate the complete gamerule map, and fault invalid ordinals. |
| `C3-ENTITY-FEEDBACK` | Send all 256 animation actions to absent/present correct/wrong entity types; cross hurt yaw finite/nonfinite and missing targets; trigger ordinary/server/self-inclusive swing, wake, critical and damage-indication sends. | Ignore missing/unknown actions, fault only documented present wrong-type casts, preserve action routing and sender inclusion, apply hurt yaw without validation, and keep animation/hurt unacknowledged. |
| `C3-DAMAGE-PROJECTION` | Cross configured damage-type IDs, cause/direct biased int boundaries and missing entities, absent/present finite/nonfinite positions, living/nonliving/missing targets, full/cooldown/blocked server damage branches. | Resolve the configured holder strictly; make position override both entity references; update only living presentation; emit only the documented full unblocked event branch and keep health/motion/death separate. |
| `C3-RESPAWN-SESSION` | Send same/cross-dimension respawns with every mode byte, optional death position, signed portal/sea values and all 256 keep masks after ordinary death, win and dimension change; repeat before/after player-loaded. | Resolve dimension holder strictly; replace level only on key change and player every time; retain entity data/attributes by independent low bits; reopen load tracking; preserve respawn-before-position/state ordering and canonical masks 0/1/3. |
| `C3-TAKE-ITEM` | Use absent and item/orb/arrow/other sources, absent/living/nonliving collectors, signed amount endpoints and count arithmetic around zero/overflow; vary source tracking/self relationships. | Apply collector cast/fallback and source no-op exactly; play/particle once; shrink/remove item, retain orb, remove other source as specified; never grant authoritative inventory/XP or invent an acknowledgement. |
| `C3-ENTITY-SESSION-END-TO-END` | Continue a loaded player through interaction, successful attack/damage, pickup, spectator camera, same/cross-dimension teleport, death and respawn while capturing the specified packets plus C1/C2 corrections and load flow. | Preserve carried-slot-before-input, damage/hurt/motion separation, tracker/self inclusion, relocation-before-camera, respawn-before-position/reprojection, renewed player-loaded gate, and final normalized state with no raw IDs in ECS/persistence. |
