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

The locked Java 25 official codecs also encoded the nine clientbound entity-motion fixtures below.
Every entity ID is one, all numeric/vector/rotation fields are positive zero, booleans are false,
the teleport relative mask is empty, and the minecart fixture contains one all-zero step. The
compression threshold is 256, so every frame has `data_length = 0`. ID 83 was constructed by
official decode then re-encoded because its value constructor requires an entity instance.

| Vector | Clientbound fixture | Exact frame bytes |
|---|---|---|
| `C3-GOLD-CB-ENTITY-POSITION-SYNC` | ID 35, absolute zero pose/motion | `3c002301000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000` |
| `C3-GOLD-CB-MOVE-POS` | ID 53, zero relative shorts | `0a00350100000000000000` |
| `C3-GOLD-CB-MOVE-POS-ROT` | ID 54, zero relative shorts/rotations | `0c003601000000000000000000` |
| `C3-GOLD-CB-MINECART` | ID 55, one zero step | `3a00370101000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000` |
| `C3-GOLD-CB-MOVE-ROT` | ID 56, zero rotations | `06003801000000` |
| `C3-GOLD-CB-ROTATE-HEAD` | ID 83, zero head yaw | `0400530100` |
| `C3-GOLD-CB-SET-MOTION` | ID 101, compact zero vector | `0400650100` |
| `C3-GOLD-CB-TELEPORT-ENTITY` | ID 125, absolute zero pose/motion | `40007d0100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000` |
| `C3-GOLD-CB-PROJECTILE-POWER` | ID 135, positive-zero power | `0c008701010000000000000000` |

`C3-GOLD-CLIENTBOUND-ENTITY-MOTION` is the aggregate assertion over those nine rows.

The locked Java 25 official registry-aware codec encoded ID 1 with the built-in static entity-type
registry, whose raw ID zero is `minecraft:acacia_boat`. The fixture uses entity ID one, zero UUID,
zero position/movement/rotations/data. ID 77 contains the singleton entity ID one. Both frames use
compression threshold 256 and therefore `data_length = 0`.

| Vector | Clientbound fixture | Exact frame bytes |
|---|---|---|
| `C3-GOLD-CB-ADD-ENTITY` | ID 1, entity 1, zero UUID, type raw 0, zero projection | `3100010100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000` |
| `C3-GOLD-CB-REMOVE-ENTITY` | ID 77, singleton entity 1 | `04004d0101` |

`C3-GOLD-CLIENTBOUND-ENTITY-SPAWN` is the aggregate assertion over those two rows.

The locked Java 25 official registry-aware codecs encoded ID 99 for entity one, slot zero,
serializer zero byte zero; ID 100 for source one and absent destination zero; ID 102 for entity one,
terminal mainhand and an empty stack; ID 107 for vehicle one and passenger two; and ID 131 for
entity one with an empty attribute list. Every frame uses compression threshold 256 and therefore
`data_length = 0`.

| Vector | Clientbound fixture | Exact frame bytes |
|---|---|---|
| `C3-GOLD-CB-ENTITY-DATA` | ID 99, entity 1, slot 0 byte 0, terminator | `07006301000000ff` |
| `C3-GOLD-CB-ENTITY-LINK` | ID 100, source 1, destination 0 | `0a00640000000100000000` |
| `C3-GOLD-CB-EQUIPMENT` | ID 102, entity 1, terminal mainhand empty | `050066010000` |
| `C3-GOLD-CB-PASSENGERS` | ID 107, vehicle 1, passenger 2 | `05006b010102` |
| `C3-GOLD-CB-ATTRIBUTES` | ID 131, entity 1, no snapshots | `050083010100` |

`C3-GOLD-CLIENTBOUND-ENTITY-STATE` is the aggregate assertion over those five rows.

The locked Java 25 official registry-aware codecs encoded ID 36 with zero center/radius/count,
absent knockback, `minecraft:explosion`, registered `minecraft:entity.generic.explode`, and no
block-particle recipes; ID 78 for entity one removing `minecraft:speed`; and ID 132 for entity one,
speed amplifier zero, duration 20, visible/icon flags and no blend. Every frame uses compression
threshold 256 and therefore `data_length = 0`.

| Vector | Clientbound fixture | Exact frame bytes |
|---|---|---|
| `C3-GOLD-CB-EXPLODE` | ID 36, zero result and default explosion presentation | `2700240000000000000000000000000000000000000000000000000000000000000000001ebc0500` |
| `C3-GOLD-CB-REMOVE-EFFECT` | ID 78, entity 1, speed | `04004e0100` |
| `C3-GOLD-CB-UPDATE-EFFECT` | ID 132, entity 1, speed 0 for 20 ticks, flags 6 | `080084010100001406` |

`C3-GOLD-CLIENTBOUND-ENTITY-EFFECTS` is the aggregate assertion over those three rows.

The locked Java 25 official codecs encoded the five serverbound container fixtures with container
one, state/button/slot zero, pickup input, empty changed-slot map and empty cursor hash. They encoded
the seven clientbound fixtures with container one, state/property/slot zero and empty stack/list;
the open-screen fixture uses raw menu ID zero (`minecraft:generic_9x1`) and an empty component title.
Every frame uses compression threshold 256 and therefore `data_length = 0`.

| Vector | Serverbound fixture | Exact frame bytes |
|---|---|---|
| `C3-GOLD-SB-CONTAINER-BUTTON` | ID 17, container 1, button 0 | `0400110100` |
| `C3-GOLD-SB-CONTAINER-CLICK` | ID 18, container 1/state 0/slot 0/button 0/pickup, no changed slots, empty cursor | `0a00120100000000000000` |
| `C3-GOLD-SB-CONTAINER-CLOSE` | ID 19, container 1 | `03001301` |
| `C3-GOLD-SB-CRAFTER-SLOT` | ID 20, slot 0, container 1, enabled | `050014000101` |
| `C3-GOLD-SB-CARRIED` | ID 53, selected slot 0 | `0400350000` |

`C3-GOLD-SERVERBOUND-CONTAINER` is the aggregate assertion over those five rows.

| Vector | Clientbound fixture | Exact frame bytes |
|---|---|---|
| `C3-GOLD-CB-CONTAINER-CLOSE` | ID 17, container 1 | `03001101` |
| `C3-GOLD-CB-CONTAINER-CONTENT` | ID 18, container 1/state 0, empty list/cursor | `06001201000000` |
| `C3-GOLD-CB-CONTAINER-DATA` | ID 19, container 1, property/value 0 | `0700130100000000` |
| `C3-GOLD-CB-CONTAINER-SLOT` | ID 20, container 1/state 0/slot 0, empty stack | `0700140100000000` |
| `C3-GOLD-CB-OPEN-SCREEN` | ID 59, container 1, generic 9x1, empty title | `07003b0100080000` |
| `C3-GOLD-CB-CURSOR` | ID 96, empty stack | `03006000` |
| `C3-GOLD-CB-PLAYER-INVENTORY` | ID 108, slot 0, empty stack | `04006c0000` |

`C3-GOLD-CLIENTBOUND-CONTAINER` is the aggregate assertion over those seven rows.

The locked Java 25 official codecs encoded ID 3 with one
`minecraft:custom/minecraft:jump=1` entry, ID 22 with group `minecraft:test` and duration zero,
and IDs 103/104 with zero values. The stat fixture therefore exercises stat-type raw ID 8 and
custom-stat raw ID 23 instead of proving only an empty map. Every frame uses compression threshold
256 and therefore `data_length = 0`.

| Vector | Clientbound fixture | Exact frame bytes |
|---|---|---|
| `C3-GOLD-CB-AWARD-STATS` | ID 3, custom jump statistic 1 | `06000301081701` |
| `C3-GOLD-CB-COOLDOWN` | ID 22, `minecraft:test`, remove | `1200160e6d696e6563726166743a7465737400` |
| `C3-GOLD-CB-EXPERIENCE` | ID 103, zero progress/level/total | `080067000000000000` |
| `C3-GOLD-CB-HEALTH` | ID 104, zero health/food/saturation | `0b0068000000000000000000` |

`C3-GOLD-CLIENTBOUND-PLAYER-PROJECTION` is the aggregate assertion over those four rows.

The locked Java 25 official codecs encoded the specialized screen fixtures with container/entity
one and zero mount columns, main hand, zero block position, front side, and four empty submitted
sign lines. Every frame uses compression threshold 256 and therefore `data_length = 0`.

| Vector | Fixture | Exact frame bytes |
|---|---|---|
| `C3-GOLD-CB-MOUNT-SCREEN` | Clientbound ID 41, container 1/columns 0/entity 1 | `080029010000000001` |
| `C3-GOLD-CB-OPEN-BOOK` | Clientbound ID 58, main hand | `03003a00` |
| `C3-GOLD-CB-OPEN-SIGN` | Clientbound ID 60, zero position/front side | `0b003c000000000000000001` |
| `C3-GOLD-SB-SIGN-UPDATE` | Serverbound ID 61, zero position/front side/four empty lines | `0f003d00000000000000000100000000` |

`C3-GOLD-CLIENTBOUND-SPECIAL-SCREENS` is the aggregate assertion over the three clientbound rows;
`C3-GOLD-SERVERBOUND-SIGN-UPDATE` is the assertion over the serverbound row.

The locked Java 25 official codecs encoded the recipe-book fixtures with container/display zero or
one as shown, crafting settings false/false, an empty removal list, and a shapeless display whose
ingredient list, result and crafting-station slots are empty. Every frame uses compression threshold
256 and therefore `data_length = 0`.

| Vector | Fixture | Exact frame bytes |
|---|---|---|
| `C3-GOLD-CB-GHOST-RECIPE` | Clientbound ID 63, container 1, empty shapeless display | `07003f0100000000` |
| `C3-GOLD-CB-RECIPE-REMOVE` | Clientbound ID 75, empty display-ID list | `03004b00` |
| `C3-GOLD-SB-PLACE-RECIPE` | Serverbound ID 39, container 1/display 0/not maximum | `050027010000` |
| `C3-GOLD-SB-RECIPE-SETTINGS` | Serverbound ID 46, crafting/closed/not filtering | `05002e000000` |
| `C3-GOLD-SB-RECIPE-SEEN` | Serverbound ID 47, display 0 | `03002f00` |

`C3-GOLD-CLIENTBOUND-RECIPE-BOOK` is the aggregate assertion over the two clientbound rows;
`C3-GOLD-SERVERBOUND-RECIPE-BOOK` is the aggregate assertion over the three serverbound rows.

The locked Java 25 official codecs encoded ID 52 with container one, an empty offer list, level/XP
zero and both presentation flags false. The empty list is a codec fixture—the canonical publisher
omits ID 52 for an empty authoritative list. ID 51 selects hint zero. Both frames use compression
threshold 256 and therefore `data_length = 0`.

| Vector | Fixture | Exact frame bytes |
|---|---|---|
| `C3-GOLD-CB-MERCHANT-OFFERS` | Clientbound ID 52, container 1, empty offers, level/XP 0, flags false | `080034010000000000` |
| `C3-GOLD-SB-SELECT-TRADE` | Serverbound ID 51, selection hint 0 | `03003300` |

`C3-GOLD-CLIENTBOUND-MERCHANT` is the assertion over the clientbound row;
`C3-GOLD-SERVERBOUND-MERCHANT` is the assertion over the serverbound row.

The locked Java 25 official codecs encoded ID 48 with an empty rename string and serverbound ID 52
with primary `minecraft:speed` (configured raw ID zero) plus absent secondary. Both frames use
compression threshold 256 and therefore `data_length = 0`.

| Vector | Fixture | Exact frame bytes |
|---|---|---|
| `C3-GOLD-SB-RENAME-ITEM` | Serverbound ID 48, empty name | `03003000` |
| `C3-GOLD-SB-SET-BEACON` | Serverbound ID 52, speed primary, absent secondary | `050034010000` |

`C3-GOLD-SERVERBOUND-ANVIL-BEACON` is the aggregate assertion over those two rows.

The locked Java 25 official codecs encoded the six chat-family fixtures with zero transaction and
offset, empty command/message strings, epoch/zero salt, no signatures or acknowledged bits, and
checksum zero. The session fixture uses zero UUID/expiry, one fixed valid 1,024-bit RSA X.509 public
key and an empty services signature. Every frame is below compression threshold 256 and therefore
uses `data_length = 0`.

| Vector | Fixture | Exact frame bytes |
|---|---|---|
| `C3-GOLD-SB-CHAT-ACK` | Serverbound ID 6, offset 0 | `03000600` |
| `C3-GOLD-SB-CHAT-COMMAND` | Serverbound ID 7, empty command | `03000700` |
| `C3-GOLD-SB-CHAT-COMMAND-SIGNED` | Serverbound ID 8, empty command/epoch/zero salt/no arguments/empty update | `1900080000000000000000000000000000000000000000000000` |
| `C3-GOLD-SB-CHAT` | Serverbound ID 9, empty message/epoch/zero salt/no signature/empty update | `1900090000000000000000000000000000000000000000000000` |
| `C3-GOLD-SB-CHAT-SESSION` | Serverbound ID 10, zero session/expiry, fixed RSA key, empty services signature | `bf01000a000000000000000000000000000000000000000000000000a20130819f300d06092a864886f70d010101050003818d0030818902818100b086c5cc34849ab54b67d1707825ea92a0e47a944455096ed060e56dc3eb6be7f29c6e78e0615fcc17587cdf21f918da2ca09f817601d789f4c83120a1b39989d0eb4ccb3f60ada5fe130b02293f2b5d055d6fa5a3d328e9641199ac2c7f95523e8ba6e7dba0f91bcf20d5ca7cf82ef4b30d02439f6a2919e200061c04786287020301000100` |
| `C3-GOLD-SB-COMMAND-SUGGESTION` | Serverbound ID 15, transaction 0/empty input | `04000f0000` |

`C3-GOLD-SERVERBOUND-CHAT` is the aggregate assertion over these six rows.

The locked Java 25 official codecs encoded bundle clear at menu slot zero, an empty unsigned book
edit in hotbar slot zero, and advancement open/close using `minecraft:story/root`. Every frame is
below compression threshold 256 and therefore uses `data_length = 0`.

| Vector | Fixture | Exact frame bytes |
|---|---|---|
| `C3-GOLD-SB-BUNDLE-SELECTION` | Serverbound ID 3, menu slot 0/selection -1 | `08000300ffffffff0f` |
| `C3-GOLD-SB-EDIT-BOOK` | Serverbound ID 24, slot 0/zero pages/no title | `050018000000` |
| `C3-GOLD-SB-ADVANCEMENT-OPEN` | Serverbound ID 50, OPENED_TAB/`minecraft:story/root` | `18003200146d696e6563726166743a73746f72792f726f6f74` |
| `C3-GOLD-SB-ADVANCEMENT-CLOSE` | Serverbound ID 50, CLOSED_SCREEN | `03003201` |

`C3-GOLD-SERVERBOUND-INVENTORY-AUXILIARY` is the aggregate assertion over these four rows.

The locked Java 25 official codecs encoded clientbound map zero with scale zero, unlocked and no
decorations/patch; tag-query transaction zero with null/END NBT; and a nonreset, empty advancement
delta with presentation disabled. Every frame is below compression threshold 256 and therefore
uses `data_length = 0`.

| Vector | Fixture | Exact frame bytes |
|---|---|---|
| `C3-GOLD-CB-MAP-DATA` | Clientbound ID 51, map 0/scale 0/unlocked/no decorations/no patch | `0700330000000000` |
| `C3-GOLD-CB-TAG-QUERY` | Clientbound ID 123, transaction 0/null NBT | `04007b0000` |
| `C3-GOLD-CB-ADVANCEMENTS` | Clientbound ID 130, nonreset/empty add-remove-progress/show false | `080082010000000000` |

`C3-GOLD-CLIENTBOUND-INVENTORY-PROGRESSION` is the aggregate assertion over these three rows.

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

## C3 entity-motion boundaries and traces

| Vector | Stimulus | Required oracle |
|---|---|---|
| `C3-ENTITY-MOTION-CODECS` | Cross signed entity VarInts, all short/rotation/on-ground bytes, absolute and teleport IEEE bit patterns, every relative-mask bit/high bit, canonical/noncanonical `LpVec3`, minecart counts/steps/weights, truncation, overlong VarInts and trailing bytes. | Preserve exact layouts, signed-short and signed-byte rotation rules; ignore mask high bits; accept raw absolute/minecart/projectile IEEE values; retain compact-vector finiteness; fault negative/impossible lists, malformed and trailing data. |
| `C3-RELATIVE-ENTITY-MOVEMENT` | Deliver IDs 53/54/56 to missing, locally authoritative, direct and three-step-interpolated entities with zero/nonzero deltas around rounding boundaries and short endpoints; repeat/interrupt interpolation and vary ground. | Decode zero against the exact base and nonzero against `round(base*4096)`; replace the base only for position forms; make local authority update only its base; apply remote pose/ground through the entity-specific interpolation hook. |
| `C3-ABSOLUTE-ENTITY-SYNC` | Deliver ID 35 at squared distance below/at/above 4096 to ticking/nonticking, local-authoritative and player-carrying targets with distinct encoded velocity and arbitrary pose/ground bits. | Replace every present target's base first; ignore every other field for local authority; otherwise select interpolation only for ticking distance `<=4096`, never apply encoded velocity, reposition a rider after a noninterpolating result, and set ground last. |
| `C3-ENTITY-TELEPORT` | Cross all nine relative bits and high bits, distance 4096 boundary, ticking/local-authoritative combinations, direct/interpolation handlers, indirect local-player passengers, ordinary missing IDs, and the retained removed-player-vehicle ID. | Calculate absolute pose/velocity including rotate-delta and pitch clamp; reproduce interpolation predicate/result and old-pose update; set ground only for present targets; emit ID 34 only for the direct local-authoritative carrying branch and ID 31 with both flags false only for former-vehicle fallback. |
| `C3-MINECART-STEP-QUEUE` | Send empty/multiple step lists with positive, zero, negative, NaN and infinite weights to missing, wrong-type, old-behavior and feature-enabled new-behavior minecarts; append during/between windows. | Ignore all but the new behavior; append without validation; transfer pending steps at window rollover; reproduce double weight sum, positive-weight selection/last fallback, three-tick interpolation and linear position/motion plus shortest-path rotations. |
| `C3-ENTITY-MOTION-PUBLICATION` | Run regular, passenger, abstract-arrow, precise, changed-ground, long-unsynced, short-overflow, fall-flying, hurting-projectile, new-minecart and hurt-marked entities around position/rotation/velocity thresholds. | Select IDs 35/53/54/55/56 exactly; emit motion then projectile power before pose, dirty state after pose, head after dirty state and self-inclusive hurt motion last; reset per-viewer bases and publication state exactly. |
| `C3-ENTITY-MOTION-END-TO-END` | Track entities through spawn-owned initialization, ordinary relative motion, velocity change, head turn, absolute resync, riding teleport, removed-vehicle fallback, minecart feature toggle, projectile acceleration and removal. | Maintain client/server pose convergence with per-viewer delta bases, preserve separate velocity/acceleration and ground projections, emit only branch-specific movement echoes, never invent an acknowledgement, and retain no packet IDs/deltas/weights in authoritative ECS or persistence. |

## C3 entity spawn/removal boundaries and traces

| Vector | Stimulus | Required oracle |
|---|---|---|
| `C3-ENTITY-SPAWN-CODECS` | Cross signed entity/data IDs, UUID bits, all 158 type IDs plus negative/out-of-range IDs, position IEEE bits, canonical/noncanonical compact movement, every rotation byte, removal count/ID VarInt boundaries, truncation, overlong VarInts and trailing bytes. | Preserve exact pitch/yaw/head order and static registry mapping; default unknown type IDs to pig; accept raw positions/data; keep compact movement finite; reject malformed/trailing forms; accept negative removal count only as an empty body and fault impossible positive lists. |
| `C3-ENTITY-TYPE-MAPPING` | Emit every namespaced locked entity type through the 26.2 static adapter and substitute dynamic-registry, block-state, metadata-serializer and connection-local numbers; query selected mapping landmarks/default. | Match all 158 `reports/registries.json` protocol IDs exactly, including player 156/fishing bobber 157; map every negative/out-of-range ID through default pig; never cross-map another numeric domain or persist a raw type ID. |
| `C3-ENTITY-CREATION` | Add player before/after player info; create nonplayers across peaceful/feature checks; use missing factory, same ID, same UUID/different ID, negative IDs, marker and exceptional positions; fail after a matching former-vehicle marker. | Clear matching fallback marker first; apply player/nonplayer construction gates; recreate before same-ID discard; reproduce duplicate-UUID section-only insertion and living/base pose differences; start only the documented sound/seen-player side effects. |
| `C3-ENTITY-SPAWN-DATA` | Cross every signed data value for item/glow frames, paintings, falling blocks, Warden, all projectile owner cases and fishing hooks; spawn dragon, Shulker, llama spit, Shulker bullet and old/new minecart. | Reproduce modulo/absolute direction and painting vertical fault; invalid-state-to-air; exact emerging value; pre-insertion owner lookup including ID zero/old same ID; invalid hook discard/insertion; every non-data recreation side effect and wrapping dragon-part IDs. |
| `C3-ENTITY-REMOVAL` | Send empty, negative-count, missing, negative, duplicate, nested-vehicle and mixed lists in alternate order; remove players/dragons and later send matching/unrelated add and ID-125 teleport. | Process wire order; retain the qualifying vehicle before detach; discard present targets and auxiliary/debug tracking once; keep player info/seen history independent; use/clear the former-vehicle marker only on its specified teleport/add paths; emit no response. |
| `C3-ENTITY-PAIRING-ORDER` | Move self/other viewers around horizontal range and view/chunk/broadcast boundaries with indirect passengers; enter/leave twice; pair entities with every optional metadata/attribute/equipment/passenger/leash projection. | Exclude self; use inclusive squared effective range plus broadcast/chunk gates; send one ordered add-first bundle and call start-seen after it; avoid duplicate pairing; stop-seen before canonical singleton removal. |
| `C3-ENTITY-SPAWN-END-TO-END` | Publish player info, pair player/mob/projectile/hanging/falling/dragon/vehicle entities, apply following state/motion, replace IDs, remove a player vehicle and exercise teleport fallback, then unpair all viewers. | Maintain exact client lookup/auxiliary/relationship convergence, spawn-first and remove-last ordering, type/data mappings and branch side effects; preserve authoritative namespaced IDs/UUID policy without persisting packet IDs, raw registries or client fallback state. |

## C3 entity-state boundaries and traces

| Vector | Stimulus | Required oracle |
|---|---|---|
| `C3-ENTITY-METADATA-CODECS` | Cross slots `0/24/254/255`, all 43 serializers, every primitive endpoint, registry/enum/default mapping, absent/present optional, component/item/particle payload, missing terminator, unknown serializer, truncation and trailing bytes. | Treat 255 only as terminator; select exactly one locked serializer and consume its exact value; preserve documented IEEE/VarInt/optional forms and mapping fallbacks; fault unknown serializers, malformed delegated values and residual data. |
| `C3-ENTITY-METADATA-TABLES` | Re-audit every top-level entity class and inheritance branch, all 221 accessor declarations, default values and callbacks; substitute same-number slots/serializers from unrelated classes and alter registration order. | Match serializer digest `96047ad220ac7064e205594f3222d182c87591d7` and accessor digest `b489eec18fc1981ebfb7ac97c54a4485fe2f938a`; compose each concrete table only from its source hierarchy; reject wrong table/type and never persist or globally interpret a slot number. |
| `C3-ENTITY-METADATA-APPLICATION` | Send empty, ascending, descending and duplicate lists to missing/present entities with valid, absent and wrong-serializer slots; return values to default and dirty multiple slots before pairing/ordinary update. | Ignore a missing target; otherwise apply/callback in wire order and aggregate once, fault absent/type-mismatched slots, leave the last duplicate value, pair only nondefaults, publish all dirty values ascending to tracking plus self, and refresh the pairing snapshot. |
| `C3-ENTITY-ATTRIBUTES` | Cross 0/1/128/129 snapshot counts; all 40 holder IDs and invalid IDs; absent/present/nonliving instances; repeated attributes; raw bases; empty/duplicate modifier identifiers; all/invalid operations and negative/impossible modifier counts. | Enforce the outer 128 cap and strict holder/identifier forms; ignore missing entities/instances but fault present nonliving targets; sanitize base through the instance, replace all modifiers per snapshot in wire order, default invalid operations to add-value, and publish only syncable/dirty sets. |
| `C3-ENTITY-EQUIPMENT` | Cross all eight ordinals, high continuation combinations, invalid low values, missing final entries, positive/zero/negative stack counts, every item/component mapping, duplicate patches/slots, component-count signed sums/overflow, missing/nonliving targets, clears, hand swaps and simultaneous changes. | Require at least one terminal entry; decode the exact trusted item patch including raw signed-count loop/capacity behavior, fault invalid mappings/ordinals, ignore wrong targets, apply slots in order/last-wins, send all nonempty pairing slots, send changed clears, and replace an exact hand swap with event 55 while retaining other deltas. |
| `C3-ENTITY-PASSENGERS` | Send empty, duplicate, missing, negative and cross-vehicle passenger lists to missing/present vehicles, including cycles, local-player add/remove, boats, already-indirect membership and counts around remaining-byte bounds. | Reject negative/impossible arrays; ignore unknown vehicles; otherwise eject then force-start present IDs in order, preserve `startRiding` results, clear the former-vehicle marker on local add, run boat/onboarding only on a new carry, and converge full lists through filtered tracker/direct-player publication. |
| `C3-ENTITY-LEASH` | Send signed-int endpoints, zero, missing/future/current holders and missing/nonleashable/leashable sources before/after holder spawn/removal; attach, reassign, detach and pair with mutation send flags. | Ignore wrong sources, replace delayed ID for leashable sources, keep nonzero unresolved until lookup can bind, make zero no holder, project current IDs only when the server path requests it, and pair the current nonnull holder after passenger lists. |
| `C3-ENTITY-STATE-PUBLICATION` | Pair and tick entities with combinations of default/dirty metadata, syncable/dirty attributes, equipment effects, passenger membership, leash mutations, ordinary/passenger motion, head changes and self viewers. | Preserve spawn→metadata→attributes→equipment→own passengers→vehicle passengers→leash pairing; publish passenger comparison before motion, motion before metadata then attributes then head, preserve self inclusion, and keep equipment/leash mutation sends in their independent paths. |
| `C3-ENTITY-STATE-END-TO-END` | Pair representative base/living/player/mob/vehicle/projectile/display/hanging entities, mutate every owned state domain, reorder/drop/duplicate legal packets, unpair/repair, respawn and re-pair under the same configured registries. | Maintain exact typed client convergence and documented failure/ignore behavior without acknowledgements; recover through authoritative replacements/pairing; retain namespaced typed Ferrite state without raw serializer, slot, ordinal, registry, component, operation or connection-local relationship IDs. |

## C3 entity-effect boundaries and traces

| Vector | Stimulus | Required oracle |
|---|---|---|
| `C3-EXPLOSION-CODECS` | Cross every IEEE center/radius/knockback value, signed block-count endpoint, optional presence, all 125 particle types/options, registered/direct sound holders, zero/negative/huge recipe counts, every recipe weight and malformed/trailing forms. | Preserve the exact double/float/int/optional order; strictly dispatch particle and sound mappings; enforce nonnegative recipe count/weight and total-weight limits; accept raw semantic numeric values and fault malformed, unknown or residual input. |
| `C3-EXPLOSION-SEMANTICS` | Explode before/at/after squared distance 4096 with small/large/non-block interaction, zero/positive/negative/mixed block counts, empty/zero/positive recipes, particle settings, solid/air samples and present/absent knockback. | Complete authoritative explosion first; send only at strict range with per-player hit-map knockback; play sound and primary particle immediately; schedule at most 512 all-particle air samples with the documented raw signed block-count/recipe weighting; add optional vector to local velocity and emit no acknowledgement. |
| `C3-MOB-EFFECT-CODECS` | Cross every signed entity/amplifier/duration value, all 40 effect IDs and invalid IDs, all 256 flag bytes, truncation, overlong VarInts and trailing bytes for IDs 78/132. | Resolve the strict effect holder, clamp amplifier only while constructing the client instance, retain duration including `-1`, read only low flag semantics, and fault malformed/unknown/residual forms. |
| `C3-MOB-EFFECT-APPLICATION` | Add/replace/remove absent and present effects on missing/nonliving/living/immune entities with visible/ambient/icon/blend combinations, duplicate updates, duration `-1/0/negative/positive` and amplifier boundaries. | Ignore wrong targets and ineligible additions; otherwise force-replace without hidden-effect merge, copy prior blend state on replacement, skip blend only when requested, remove silently if present, and let ordinary client ticking/metadata presentation consume the result. |
| `C3-MOB-EFFECT-PUBLICATION` | Add/update/remove player and vehicle effects with direct/indirect/no player passengers; cross the 600-tick refresh; join, start riding and dismount with multiple active effects; trigger attribute and particle-metadata changes. | Send new self additions with blend only, updates/periodic refresh/replays/passenger sends without blend, direct passengers only, complete unsorted active replays before passenger convergence and removals before dismount convergence; keep attribute and metadata packets separate. |
| `C3-ENTITY-EFFECTS-END-TO-END` | Run authoritative explosions and timed/additive/replaced effects across join, riding, dismount, death/removal and re-entry while capturing IDs 36/78/99/107/131/132. | Converge local knockback, effect maps, visuals, attributes and relationships in documented order without response packets or persistence of raw holders, flags, counts, recipes or client blend/tracker state. |

## C3 container-convergence boundaries and traces

| Vector | Stimulus | Required oracle |
|---|---|---|
| `C3-CONTAINER-INGRESS-CODECS` | Cross every signed container/state VarInt, slot short, button byte, input ID, changed-slot count/key and boolean; use truncated/overlong/trailing fields and counts around 128. | Preserve exact field order/widths; map every invalid input to pickup; replace duplicate map keys; enforce the upper bound and fail negative allocation/malformed/residual forms without adding unsigned container semantics. |
| `C3-CONTAINER-EGRESS-CODECS` | Cross container/state/slot/property/value endpoints; empty/positive/negative stack counts and generic list counts; all menu/item/component IDs; trusted title NBT; truncated and trailing fields. | Preserve exact VarInt/short/stack/list/title layouts; strictly resolve all three registries, retain raw positive counts/component patches, fail invalid mappings/allocation/NBT/malformed forms, and keep numeric domains separate. |
| `C3-CONTAINER-MENU-MAPPING` | Enumerate all 25 menu raw IDs and substitute item/component/entity/raw packet numbers at both endpoints and outside range; omit client screen constructors. | Match menu digest `dc1416c68f9fb0efac6c1a3ce39db0d5e2216387`, reject invalid registry IDs, warn/retain current client state for a valid type without a screen, and never cross-map or persist a raw ID. |
| `C3-CONTAINER-HASHED-STACKS` | Hash empty and component-bearing stacks; cross exact/mismatched count/item/added/removed sets, duplicate entries, 256 boundaries, CRC32C collision witnesses and server cache eviction. | Treat false as empty; compare the complete patch shape and registry-aware 32-bit hashes; accept collisions as matches; use the 256-entry typed-component cache only as an optimization; never mutate authoritative stacks from hashes. |
| `C3-CONTAINER-CLICK-PREDICTION` | Execute every source-specified click input/button/sentinel/quick-craft phase through the vanilla client, with zero/one/many slot changes and out-of-short/byte local arguments. | Predict before send, hash only count/item/component changes and cursor, send current state, preserve deterministic click rules, and have checked casts throw before emitting out-of-width values. |
| `C3-CONTAINER-CLICK-ADMISSION` | Click with wrong/current container, invalid/current menu, spectator/dead/alive state, signed slot endpoints and matching/stale/wrapped/future state IDs; include in/out-of-range client hash keys and matching/colliding/mismatching hashes. | Reset idle before container comparison; full-resync spectator/dead; enforce validity/outer slot quirk; execute admitted clicks even when stale; ignore invalid hash keys; choose full snapshot on stale and hash-filtered deltas on match. |
| `C3-CONTAINER-CONTROLS` | Submit every generic menu button boundary and crafter slot-state request across container, spectator, validity, menu/backing, empty/nonempty slot and enabled-state branches. | Apply `ITM-CONTAINER-CONTROL-001`; broadcast button deltas only on true, give crafter no idle/validity gate, mutate only empty slots 0..8 in a real crafter and rely on ordinary later convergence. |
| `C3-CONTAINER-OPEN-CLOSE` | Open from inventory/another menu through counter wrap and failed creation/screen lookup; reorder/duplicate/delay serverbound and clientbound close packets with stale/current/arbitrary IDs. | Order close/removal before open then content/data; cycle canonical IDs 1..100; ignore close IDs on both peers and close current state; preserve no-response serverbound close and no protected generation. |
| `C3-CONTAINER-STATE-LIFECYCLE` | Send full snapshots, multiple slot deltas, cursor-only/data-only changes and state increments around 32767; deliver duplicate/decreasing/future clientbound states and click matching/stale echoes. | Increment only full content and each slot with 15-bit wrap; assign client state without monotonic checks; order slots ascending then cursor/data; execute stale click then full-resync and allow a fully matching click to produce no packet. |
| `C3-CONTAINER-CLIENT-APPLICATION` | Apply zero/nonzero content with short/exact/long lists; slot/data signed boundaries; wrong IDs under normal/creative screens; cursor under creative; player-inventory slots negative, 0..42 and above. | Target inventory/current menus exactly, preserve partial long-list fault order and short-list leftovers, run tutorial/pop-time/creative remote quirks, suppress creative cursor mutation, and reproduce ordinary/equipment inventory mapping and faults. |
| `C3-CONTAINER-REMOTE-HASH` | Install exact then hashed remote snapshots, mutate/revert authoritative stacks, use matching/mismatching hashes and full/delta broadcasts across suppressed updates. | Clear exact state on receive, promote a matching current copy, correct mismatches, force exact snapshots after sends/full state, and never publish during suppression or trust a hash as item state. |
| `C3-CARRIED-SELECTION` | Send every signed short while idle/using main/off hand and selecting same/different slot. | Accept only 0..8; stop use only on a changed selection using main hand; reset idle on every valid request but not invalid ones; emit no direct acknowledgement. |
| `C3-CONTAINER-CLOSE` | Close current/old/new menus with arbitrary packet IDs while cursor/results contain removable items and shared inventory state differs. | Ignore the packet ID, run current-menu removal and inventory-menu state transfer once, send no response for serverbound close, and let server-initiated close send ID 17 before removal. |
| `C3-CONTAINER-CONVERGENCE-END-TO-END` | Open representative menus, exercise every click/control path with correct and stale predictions, mutate slots/cursor/data, switch hotbar, race close/open, and capture IDs 17/18/19/20/53/59/96/108. | Reach identical authoritative/client menu state through exact open/full/delta/hash ordering, preserve all semantic ignore/fault branches, create no cross-domain ACK, and retain no wire IDs, registries, hashes or GUI snapshots in ECS/persistence. |

## C3 player-projection boundaries and traces

| Vector | Stimulus | Required oracle |
|---|---|---|
| `C3-PLAYER-PROJECTION-CODECS` | Cross signed map counts/stat values/durations/levels/totals/food, every IEEE bit class for progress/health/saturation, identifier lengths/forms, truncation, overlong VarInts and trailing bytes. | Preserve exact layouts including experience level-before-total; accept raw semantic numeric values; fault negative map allocation, invalid identifier/nondefaulted mappings, malformed and residual forms while preserving defaulted backing behavior and adding no unsigned/range policy. |
| `C3-STAT-MAPPING` | Enumerate all nine stat types and all 1,196 block, 1,537 item, 158 entity-type and 77 custom-stat backing IDs; substitute same raw numbers across registries and invalid endpoints. | Match the five locked counts/digests, dispatch each type only to its backing registry, reject unknown type/custom-stat IDs, default unknown block/item/entity IDs to air/air/pig, and retain resolved typed namespaced pairs rather than raw IDs. |
| `C3-HEALTH-FOOD-PROJECTION` | Tick around health/food changes and positive/zero/negative-zero/NaN saturation edges; deliver finite endpoint, infinite and NaN packets before/after the first projection and alongside damage/metadata. | Trigger sends on health, food or only saturation-zero predicate; carry latest complete tuple; apply first/later hurt-flash branches, clamp health exactly, assign food/saturation directly, and keep damage/metadata independent. |
| `C3-EXPERIENCE-PROJECTION` | Change total, progress and level independently through canonical and direct paths; cross sentinel/wrap/signed/nonfinite values, join, same/cross-dimension relocation and respawn. | Send on total/marker mismatch, including canonical `-1` forcing except its exact total collision; preserve progress/level/total wire reorder, reset XP display timing on float progress inequality including repeated NaN, and retain explicit respawn plus canonical first-tick repeat without an acknowledgement. |
| `C3-COOLDOWN-PROJECTION` | Start, replace, explicitly remove and naturally expire absent/present groups; vary item fallback/component group, duration zero/negative/positive and tick/end signed wrapping; delay old removals. | Send start immediately and zero for explicit/natural removal; use exact group identity, interpret only zero as client removal, replace nonzero intervals with wrapping arithmetic, expire by the documented tick comparison, and allow delayed zero to clear a newer projection. |
| `C3-STATISTICS-DRAIN` | Dirty zero/one/many/repeated stats with positive/negative/overflowing increments; join-mark all, request repeatedly before/after mutation, duplicate typed keys on decode, and open/close the stats screen. | Compute increments with long addition, upper saturation and signed narrowing wrap below minimum; mark assigned stats dirty, send nothing merely for dirtiness, atomically drain exactly one map per request including empty, replace included client values only, collapse duplicate keys last, and callback an open screen once. |
| `C3-PLAYER-PROJECTION-ORDER` | Capture one player tick with food/cooldown/stat/vital/score/XP changes, fresh placement, relocation, respawn and interleaved stats requests plus container/teleport/block/keepalive traffic. | Preserve cooldown mutation/expiry sites, health-before-score-criteria-before-experience, explicit respawn experience and forced next-tick projections; keep statistics request correlation tokenless and all other acknowledgement domains independent. |
| `C3-PLAYER-PROJECTION-END-TO-END` | Join, take/heal damage, exhaust/eat, gain/spend XP, start/replace/expire cooldowns and request statistics repeatedly across teleport, dimension change, death/respawn and reconnect while capturing IDs 3/22/99/103/104. | Converge local HUD/player/stat/cooldown state with exact triggers and order, tolerate documented delay/replacement behavior, derive no authority from client presentation, and persist no raw IDs, sent markers, tick intervals or UI timers. |

## C3 special-screen boundaries and traces

| Vector | Stimulus | Required oracle |
|---|---|---|
| `C3-SPECIAL-SCREEN-CODECS` | Cross signed mount container/column/entity endpoints, strict hand ordinals, packed coordinate endpoints, every boolean byte, four UTF lines at receive 384-UTF-16-unit/1,152-byte and encode 32,767-unit/98,301-byte boundaries, malformed UTF-8, truncation, overlong VarInts and trailing bytes. | Preserve exact widths and order; expose the sign writer's larger default bound but enforce the smaller server read bound, reject invalid hands and length/truncation/residual faults, replace malformed UTF-8 sequences, and otherwise retain signed mount arithmetic, packed sign extension, nonzero-true booleans and exactly four lines without semantic range checks. |
| `C3-MOUNT-SCREEN-ACTIVATION` | Send ID 41 for missing/wrong/horse/llama/nautilus entities with negative, zero, canonical and wrapping/resource-sized columns, current old menus, entity tracking races and following full/delta/close traffic. | Allocate wrapped `columns*3` before type admission; fault allocation where specified; leave state unchanged for wrong entities; install exact specialized menus/screens for horse/nautilus and converge through ordinary container ordering with the entity already tracked. |
| `C3-BOOK-VIEW-ACTIVATION` | Resolve/mutate written and writable components, filtered/raw pages, wrong items and either hand before publication and between send/handle; close the resulting screen. | Broadcast canonical resolution changes before ID 58; read only the handler-time current hand stack; open a view-only screen for either recognized component, ignore absence, and send no response or editing request. |
| `C3-SIGN-EDITOR-ACTIVATION` | Interact with ordinary/hanging, front/back, waxed, command-bearing, occupied, uneditable and missing/stale signs while reordering ID 8, prior block-entity data and ID 60. | Execute command/wax/edit admission, store one allowed editor, send block correction before activation, require the existing client sign entity, select the exact editor subtype/side/filtering view and add no token or embedded text. |
| `C3-SIGN-EDITOR-SUBMIT` | Exit with Done, Escape, replacement, distance invalidation, removed sign and lost connection after zero/multiple line edits and rendered-width boundaries. | Close through one removal callback, send exactly one current four-line update when connected regardless of exit reason, preserve activation position/side, and send nothing only when connection is absent. |
| `C3-SIGN-UPDATE-CODEC` | Submit all packed coordinates/sides, format-code forms and four UTF line boundaries including official-encode/server-decode asymmetry, absent/extra lines, malformed UTF-8 and trailing bytes. | Let the member writer use default UTF bounds but decode exactly four UTF(384) strings, replace malformed UTF-8, strip formatting before filtering, preserve order/side/position and fault negative/over-limit lengths, incomplete fields, decoded-unit excess and residual data. |
| `C3-SIGN-UPDATE-ADMISSION` | Delay filtering while moving player/level, ticking range authorization, waxing/unwaxing, replacing/removing/loading the sign, changing editor/side/filter setting and submitting accepted/unchanged/unauthorized text. | Reset idle and inspect current state only after filtering; require loaded current authorized unwaxed sign; retain styles, select filtered-only or raw-plus-filtered literals, call flags-3 update while installing rebuilt text, clear authorization, make the unconditional second flags-3 update call even unchanged, and send no direct response on either branch. |
| `C3-SPECIAL-SCREENS-ORDER` | Interleave mount close/open/full state, resolved book menu deltas/open, sign ID-8/ID-60/ID-61/filter completion/block-entity update with container, teleport and prediction traffic. | Preserve every family-local sequence without inventing screen acknowledgements or cross-family generations; allow delayed handler-time book and async sign state to take their documented current-state branches. |
| `C3-SPECIAL-SCREENS-END-TO-END` | Join, track and open horse/llama/nautilus inventories, read both book component forms, edit both sides of ordinary/hanging signs, race close/reopen/filtering and reconnect while capturing IDs 8/17/18/19/20/41/58/60/61. | Reach authoritative menu and sign convergence plus correct local book presentation under all specified ignore/fault/race branches, with no raw packet/container/entity IDs, GUI state or edit authorization persisted as gameplay identity. |

## C3 recipe-book boundaries and traces

| Vector | Stimulus | Required oracle |
|---|---|---|
| `C3-RECIPE-BOOK-CODECS` | Cross signed container/display VarInts, all four/invalid book ordinals, every boolean byte, removal counts empty/positive/negative/impossible, every recipe/slot display dispatch and nested registry endpoint, truncation, overlong VarInts and trailing bytes. | Preserve exact order and signed domains; map only book ordinals 0..3; accept nonzero booleans and semantic negative IDs; strictly resolve ghost display/nested registries; fault invalid enum/registry, allocation, malformed and residual forms. |
| `C3-RECIPE-DISPLAY-ID-MAPPING` | Reload sorted and grouped recipes with zero/one/multiple displays, special/non-special placement information and enabled/disabled feature requirements; resolve negative, endpoint and stale indices. | Assign contiguous enabled display IDs from zero in exact recipe/display iteration order; assign first-seen nonempty groups locally; retain full display/category/optional placement plus namespaced parent; reject out-of-range lookup and never treat display/group integers as durable registry identities. |
| `C3-RECIPE-PLACE-ADMISSION` | Place from spectator/non-spectator players across wrong/current container, valid/invalid menu, negative/missing/valid display, unknown/known parent, recipe/nonrecipe menus and possible/impossible placement data. | Reset idle before every semantic gate; apply gates in documented order; log only invalid-menu/impossible-placement branches; call placement only with current menu, mapped parent, current level/inventory and decoded max flag; send no rejection correction. |
| `C3-RECIPE-PLACE-MUTATION` | Exercise crafting/furnace grids with full/free/partial inventory, creative/survival, craftable/uncraftable ingredients, already matching/nonmatching grids, max/single/increment placement, stack-component clamps and shaped/shapeless mappings. | Reject noncreative insufficient-clear capacity unchanged; never conjure in creative; return/clear and immediately ghost only when aggregate cannot craft; otherwise reproduce amount/clamp/clear/distribution rules and converge mutations only through ordinary later container traffic. |
| `C3-GHOST-RECIPE-APPLICATION` | Deliver every display subtype to matching, wrong, closed and container-ID-reused menus with recipe-aware/nonaware screens, absent local book entries and prior ghost slots. | Require exact current ID and recipe-aware screen only; clear prior ghost and fill from the full supplied display/current level without requiring a known display ID, mutating item state or sending a response; expose ordinary ID-reuse delay behavior. |
| `C3-RECIPE-BOOK-SETTINGS` | Toggle every type/open/filter pair repeatedly with/without a client connection and around initial settings publication, reconnect and persistence packing. | Update local state before send, replace the exact server tuple without idle/menu/mode gates or echo, persist normalized four-type settings, and let later explicit clientbound settings replace local state independently. |
| `C3-RECIPE-BOOK-HIGHLIGHTS` | Add and show parent recipes with one/multiple displays, duplicate/missing IDs and invalid/stale indices; remove known/unknown parents whose current displays are zero/one/many. | Key server known/highlight state by parent recipe; remove exactly one local highlight before ID 47 but the shared parent server highlight after valid mapping; publish all display IDs on parent removal only when nonempty; never persist display IDs. |
| `C3-RECIPE-BOOK-REMOVE` | Apply ID 75 with empty, singleton, duplicate, missing and mixed IDs in varied order while search/UI is closed or recipe-aware. | Remove known entry and exact highlight per ID in wire order; make missing/duplicates no-ops; preserve omitted entries; rebuild collections/search and callback the open listener exactly once after the whole packet, including empty. |
| `C3-RECIPE-BOOK-ORDER` | Capture initial settings/add, later unlock/remove, local settings/seen, craftable and cannot-craft placement interleaved with recipe reload, menu reuse, ordinary slot deltas, teleports, block predictions and liveness. | Preserve ID 76 before initial ID 74, parent add/remove mappings and local-first requests; send conditional ID 63 after clear and before later slot broadcasts; create no recipe generation or cross-family acknowledgement and reproduce stale-index/ID-reuse semantics. |
| `C3-RECIPE-BOOK-END-TO-END` | Join, unlock grouped/multidisplay recipes, toggle all books, mark displays seen, place single/max recipes in crafting/furnace menus with sufficient/insufficient ingredients and remove recipes across reload/reconnect while capturing IDs 18/20/39/46/47/63/74/75/76. | Converge authoritative parent knowledge/settings and client collections/highlights/ghost/menu state under every specified success/no-op/fault/order branch, retaining no raw display/container/type IDs, GUI state or placement caches in ECS/persistence. |

## C3 merchant-offer and selection boundaries and traces

| Vector | Stimulus | Required oracle |
|---|---|---|
| `C3-MERCHANT-CODECS` | Cross signed container/level/XP/selection/count VarInts, empty/positive/negative/impossible offer and predicate lists, every boolean byte, signed offer ints, all IEEE multiplier classes, optional costs, empty/nonempty results, every item/component mapping, truncation, overlong VarInts and trailing bytes. | Preserve exact nested field order and signed/raw domains; require a nonempty result; strictly resolve item/component dispatch; accept nonzero booleans and semantic signed values; fault invalid mapping/allocation/empty-result/malformed/residual forms. |
| `C3-MERCHANT-COST-MAPPING` | Enumerate locked items/components in costs and candidate stacks with absent/equal/different/extra/duplicate predicate components, raw signed counts and optional B present/absent. | Resolve only the 1,537-item/111-component domains; require item identity and every listed component equality while allowing extras; retain duplicate-entry effects; check count separately and require an empty second input only when B is absent. |
| `C3-MERCHANT-OFFER-DECODE` | Decode out-of-stock false/true around uses/max boundaries and mismatches, arbitrary XP/special/demand/multiplier values, then mutate source offers/results after packet construction and re-encode. | Force network reward-experience true; let true replace uses with max before installing special, let false retain uses but derive stock from counts, encode the derived flag, preserve other raw fields and prove copied packet/list/result isolation. |
| `C3-MERCHANT-OFFER-PRICING` | Cross base count, demand and special-price signed overflow, zero/negative/NaN/infinite multipliers, item max-stack boundaries, optional B and swapped payment inputs. | Reproduce Java int wrapping, float conversion/multiplication, `Mth.floor`, nonnegative demand delta and final `1..item-max` clamp for A; keep B at base count; reject/accept payments and copy assembly exactly. |
| `C3-MERCHANT-CLIENT-APPLICATION` | Deliver empty/nonempty ID 52 to wrong/equal and reused container IDs with inventory/nonmerchant/merchant menus, open/closed merchant screens, arbitrary level/XP/flags and prior offers. | Ignore the complete packet unless current ID and menu type both match; otherwise replace offers then XP/level/progress/restock with no merge/version/ack; preserve signed HUD edge behavior and tooltip-only restock semantics. |
| `C3-MERCHANT-SELECTION-ADMISSION` | Send every signed hint while current menu is inventory, valid/invalid merchant, spectator/dead/alive and around menu replacement, with/without loaded state and idle timing. | Require only current `MerchantMenu` and `stillValid`; add no packet ID, spectator/death/load/idle gate; store/recompute every admitted hint before auto-fill range checking and send no direct response. |
| `C3-MERCHANT-SELECTION-LOOKUP` | Select zero, positive in-range, negative and at/beyond-size hints with empty/one/multiple offers, normal/swapped payments and in/out-of-stock candidates. | For positive in-range hints test only that offer; otherwise scan first match from zero; retry swapped only after missing/out-of-stock, install copied result/future XP or clear them for a nonempty list, and preserve both the empty-input no-callback and nonempty-input/empty-list stale-result callback quirks. |
| `C3-MERCHANT-AUTOFILL` | Select valid/invalid indices with zero/full/partially returnable payment slots and matching/nonmatching/exact/extra components across inventory order and source stack sizes. | Let invalid indices retain hint/result mutation but move nothing; return payment zero then one non-atomically through reverse player range, fill only after both empty by ascending exact-cost scan, move up to source max-stack rather than required count and recompute result on writes. |
| `C3-MERCHANT-ORDER` | Capture merchant open with zero/nonzero offers, local selection, server replay, result clicks and ordinary slot/data convergence while delaying/reordering ID 52/51 around ID reuse and unrelated ACK domains. | Preserve close/open/full/data before conditional complete offers, local hint/auto-fill before ID 51 and server hint-before-range behavior; use only ordinary later container deltas, create no merchant token and correlate neither direction as an acknowledgement. |
| `C3-MERCHANT-END-TO-END` | Open representative villager/wandering merchant offers; select, swap and fill with full/partial inventories and component costs; project externally exhausted/restocked snapshots and issue result clicks while capturing IDs 17/18/19/20/51/52/59/96. | Converge copied offer/HUD plus payment/result projection through every specified fault/ignore/prediction branch, route actual trade execution through its owning container/gameplay rules, and retain no packet/container/registry IDs, menu ordering, predicates or GUI snapshots in ECS/persistence identity. |

## C3 anvil-rename and beacon-commit boundaries and traces

| Vector | Stimulus | Required oracle |
|---|---|---|
| `C3-ANVIL-BEACON-CODECS` | Encode rename strings at 32,767/32,768 UTF-16 units and 98,301/98,302 bytes, malformed UTF-8 and trailing forms; cross both beacon optionals, every boolean byte and all 40/invalid effect raw IDs with truncation/overlong/trailing data. | Enforce only the default UTF transport bound before semantic filtering; replacement-decode malformed UTF-8; decode optionals independently with nonzero true and strict configured holders; fault length, missing-present, unknown-registry, malformed and residual forms. |
| `C3-ANVIL-RENAME-PREDICTION` | Type, paste, delete and restore names around 50 UTF-16 units with no input, default/custom hover names, disallowed characters, whitespace and slot refresh/close. | Filter and cap the vanilla editor, normalize an uncustomized exact hover name to empty, run local result/cost recomputation before each effective ID 48, send nothing when local `setItemName` returns false and make close independent rather than a final submission. |
| `C3-ANVIL-RENAME-ADMISSION` | Send wrong/current/invalid anvil menus, filtered strings whose raw/filtered lengths cross 50, same/colliding names, empty/all-whitespace names and result-empty/present states as spectator/dead/alive. | Require only current valid AnvilMenu; remove section/control/DEL before length/equality; ignore over-50/same, otherwise store, remove or install literal custom name, invoke the owned anvil computation and ordinary broadcast without idle/mode/container-ID/echo semantics. |
| `C3-BEACON-EFFECT-MAPPING` | Enumerate every configured mob-effect holder and substitute same raw IDs from item/entity/menu/data domains; vary absent values and menu-data zero/ID-plus-one values. | Resolve only the locked 40-effect packet registry; admit only tiered holder names semantically; keep packet optional raw IDs distinct from built-in-plus-one menu data and retain normalized holder choices rather than either number. |
| `C3-BEACON-SELECTION-ADMISSION` | Submit wrong/current/invalid beacon menus with empty/nonempty/ordinary-invalid payment, levels 0..4, all absent/primary/secondary pairs, every tier relation and a payment/level race after Done. | Require only current valid BeaconMenu then nonempty payment; apply exact tier, level, primary-below-four and low-tier-secondary-equals-primary gates; consume/store/mark on success, controlled-disconnect every false result, and reproduce absent-primary/low-secondary level-four null-equality fault. |
| `C3-ANVIL-BEACON-ORDER` | Interleave repeated rename edits/result/data deltas, beacon Done/Cancel/Escape, payment/level updates, menu replacement/ID reuse and ordinary closes with unrelated ACK domains. | Preserve local rename computation before ID 48 and authoritative broadcast after acceptance; preserve beacon ID 52 before ID 19 with no local mutation; let handler-time current menus and ID-ignoring close take their documented races without cross-family correlation. |
| `C3-ANVIL-BEACON-END-TO-END` | Rename, repair and take representative items; open beacons at every tier, select primary/upgrade/regeneration, commit/cancel under payment and menu races, reconnect and reopen while capturing IDs 18/19/20/48/52/59 plus data. | Converge authoritative result/custom-name/cost and normalized beacon choices/payment/persistence through every specified success/no-op/disconnect/fault branch, delegating owned anvil business and beacon ticking effects to their gameplay rules and retaining no wire IDs, GUI strings/choices or menu snapshots as durable identity. |

## C3 serverbound chat, command and suggestion boundaries and traces

| Vector | Stimulus | Required oracle |
|---|---|---|
| `C3-CHAT-CODECS` | Cross all six IDs; default, 256, 16 and 32,500 UTF unit/byte boundaries; malformed UTF-8; signed long/VarInt endpoints; null/present signatures; argument counts 0/8/9; fixed-field truncation; last-seen high bits; public-key/signature lengths; malformed X.509; trailing data. | Preserve exact field order and signed domains; replacement-decode UTF; use fixed 256-byte signatures and exactly three bitset bytes; cap every declared string/list/array; parse the key; fault ninth/over-limit/truncated/malformed/residual forms before semantic handling. |
| `C3-CHAT-LAST-SEEN` | Project null, unsigned, distinct and repeated signed messages; apply offsets below/at/above `tracked_size-20`, all 20 bit patterns, upper encoded bits, pending/acknowledged/null clear/set combinations, checksum zero/correct/wrong, standalone ACKs at client offsets 64/65 and grow through 4,096/4,097 tracked entries. | Append only distinct consecutive nonnull signatures; reproduce slot mutation and checksum folding; let zero bypass checksum only; disconnect every invalid offset/window/checksum and overgrown server window; emit standalone ACK only after client offset exceeds 64 and keep embedded updates equivalent but not message-specific ACKs. |
| `C3-CHAT-SESSION-SECURITY` | Send first/equal-key/different-session/newer/older/expired key updates with missing services validator, invalid/valid services signatures and offline/online/enforce combinations; interleave prior filter work and later signed payloads. | Ignore equal key data including changed UUID and missing-validator updates; disconnect rollback/invalid validation branches; install a validated decoder immediately, publish player chat initialization behind earlier future-chain work, retain last-seen/cache state, and make offline enforcement false while authenticated enforcement requires all three server gates. |
| `C3-CHAT-SIGNATURE-CHAIN` | Sign chat and command arguments across index zero/maximum, equal/decreasing/future/older-than-five-minute timestamps, null/invalid/valid signatures, wrong sender/session/body/salt/last-seen values and post-break/new-session traffic. | Verify the exact SHA256withRSA byte stream and expected link; accept equal and future timestamps, warn-but-accept old valid messages, distinguish nonbreaking missing/expired failures from breaking order/signature/index exhaustion, and restore only by installing a new validated session. |
| `C3-CHAT-ADMISSION-FILTERING` | Send allowed/disallowed characters with valid/invalid last-seen updates as chat-visible/hidden players; delay, fail and reorder filter futures and decorators around disconnect and session update. | Apply last-seen before the character/visibility gates; disconnect illegal text, return only the hidden-chat system error, reset idle only after those gates, serialize accepted filter/decorate/broadcast results per sender, cancel after disconnect and charge chat spam only after broadcast. |
| `C3-COMMAND-SIGNING-DISPATCH` | Submit authoritative commands with zero/one/eight signable arguments through ID 7/8; vary empty, missing, extra, duplicate, reordered and invalid named signatures, parse differences, permissions and secure enforcement. | Let no-signable client commands use ID 7; refuse secure unsigned signable commands without dispatch; on ID 8 apply last-seen first, validate entries in wire order against authoritative parsed values, require complete name coverage, reproduce duplicate replacement/chain consumption, attach normalized signing context and delegate results to command ownership. |
| `C3-CHAT-SPAM-THROTTLING` | Burst/interleave successful, hidden, illegal, filter-canceled and signature-failed chat plus successful, invalid, refused and decode-failed commands at configured seconds negative/zero/one/default, around ticks, as ordinary player/operator/single-player owner. | Maintain independent counters; add 20 only after chat broadcast but after every scheduled command attempt; decay positive counts by one per completed listener tick; disconnect ordinary players at positive post-increment `>= 20*seconds`, exempt privileged players and disable nonpositive thresholds. |
| `C3-COMMAND-SUGGESTION-CORRELATION` | Send empty/slashed/long inputs with negative, duplicate, stale, wrapping and current transactions; delay/reorder server completions of 0/1/1,000/1,001 suggestions and vary player permissions/context. | Strip at most one slash, parse every request independently with handler-time authoritative source, preserve range/order while truncating after 1,000 and echo the raw transaction; cancel the prior client future and complete only the current matching ID, ignoring stale/duplicate results. |
| `C3-CHAT-ORDER` | Interleave clientbound signed messages, standalone/embedded acknowledgements, chat, unsigned/signed commands, key rotation, asynchronous filters, suggestions, disconnect and unrelated block/container/teleport/keepalive traffic. | Preserve acknowledgement-before-admission, scheduled payload order, per-sender filter-chain serialization, immediate decoder replacement plus delayed session publication and latest-suggestion correlation; create no cross-family acknowledgement or persistent raw protocol state. |
| `C3-CHAT-END-TO-END` | Join an offline server and an authenticated enforcing server; exchange filtered/unfiltered chat, signable/nonsignable commands, delayed acknowledgements, key renewal and live suggestions while capturing IDs 6..10/15 and paired clientbound traffic. | Reach authoritative chat/command effects and client presentation under every specified accept/refuse/disconnect/order branch, keep C3 offline null-signature play usable, require validated C4 security when enabled, and retain no salts, signature bytes, windows, chain indices, packet/transaction IDs or raw command trees in world/ECS persistence. |

## C3 inventory-auxiliary boundaries and traces

| Vector | Stimulus | Required oracle |
|---|---|---|
| `C3-INVENTORY-AUX-CODECS` | Cross signed VarInt endpoints, bundle selection -2/-1/zero/large, book slots, 0/100/101 pages at 1,024-unit/3,072-byte bounds, absent/present 32-unit titles, malformed UTF-8, advancement enum ordinals and default identifier bounds/grammar, truncation and trailing data. | Preserve exact field order and replacement UTF decode; reject bundle values below -1 in buffer decoding, enforce every list/string/enum/identifier bound and conditional OPENED_TAB field, and fault malformed/truncated/residual forms before semantic handling. |
| `C3-BUNDLE-SELECTION-PREDICTION` | Scroll forward/back across displayed subsets for content sizes 0..17, repeat a selection, stop hover, quick-move/swap and reconstruct/save/network-decode selected stacks. | Mutate locally before ID 3; send scroll only when the choice changes plus possibly redundant clear requests; reproduce padded visible counts, same-index toggle and selected/zero-fallback removal, while proving selection is absent from equality, persistence and component stream projection. |
| `C3-BUNDLE-SELECTION-ADMISSION` | Send every current/negative/out-of-range menu slot and -1/in-range/hidden-existing/out-of-content index around current-menu replacement, bundle/nonbundle stack replacement and ordinary clicks. | Resolve only the handler-time current menu slot with no container/state/still-valid/mode gate; no-op invalid slot/missing component, accept full-content hidden indices, clear out-of-content indices and let later owned clicks consume the transient choice without a response token. |
| `C3-BOOK-EDIT-ADMISSION` | Send absent/present-title books to inventory slots below/within/above 0..8 and 40, with empty/max page lists, empty/blank/15/16/32-unit titles, writable/nonwritable stacks and main-hand hotbar changes. | Ignore only invalid inventory slots up front; re-read callback-time occupancy, require only writable content there, replace all pages for absent title, and finalize every present transport-valid title without server trim/blank/15-unit checks. |
| `C3-BOOK-FILTER-ORDER` | Delay, fail and reverse filter completion for edit/edit, edit/sign and sign/edit pairs while moving/replacing writable books, changing filtering preference and disconnecting. | Run independent futures, cancel after disconnect, preserve raw+filtered alternatives when filtering is disabled or filtered-only pass-through when enabled, let completion order and current slot occupant decide mutation, and make post-finalization callbacks no-op unless writable state returns. |
| `C3-BOOK-FINALIZATION` | Use Done, Escape, sign-cancel and finalize with leading/trailing whitespace, trailing empty pages and filtered title/page variants, then inspect stack components and ordinary slot traffic. | Make Escape/cancel send nothing; let Done remove trailing exact-empty pages and send absent title; let finalize keep its current page list and send trimmed present title; transmute to written book with player author, generation zero, literal filtered pages and resolved true, with no direct ACK. |
| `C3-ADVANCEMENT-TAB-CORRELATION` | Initialize/reopen/click same and different roots; send known displayed roots, roots without display, children, unknown IDs and CLOSED_SCREEN around advancement reload and clientbound corrections. | Send OPENED_TAB before local equality notification; retain unknown and close selections, normalize known invalid tab shapes to null, reply only on server cursor identity change, award/revoke no progress, omit cursor from save data and reset it on reload. |
| `C3-INVENTORY-AUX-ORDER` | Interleave ID 3 with current-menu replacement and bundle clicks, multiple ID 24 filter futures with inventory moves, and ID 50 open/close with advancement sync plus unrelated ACK domains. | Preserve client-first bundle prediction, book completion-time mutation and advancement retained-cursor rules; use ordinary item/tab projection only where specified and create no correlation among these requests or with container, chat, block, teleport, recipe and liveness state. |
| `C3-INVENTORY-AUX-END-TO-END` | Select/remove visible and crafted-hidden bundle entries, edit/finalize books under filter and slot races, and open/close/reopen representative advancement roots while capturing IDs 3/24/50 and resulting component/tab traffic across save/reconnect. | Converge owned bundle removal, normalized writable/written components and displayed-root presentation through every specified no-op/race branch; persist book content and advancement progress only, reset transient bundle/tab selections at their documented boundaries, and retain no packet/menu/inventory slot or raw registry IDs as durable identity. |

## C3 clientbound inventory-progression boundaries and traces

| Vector | Stimulus | Required oracle |
|---|---|---|
| `C3-MAP-PROJECTION-CODECS` | Cross signed map IDs/scale and every boolean byte; absent/empty/duplicate/large decoration lists, strict type raw IDs, all X/Y/rotation bytes and optional trusted names; patch width 0/1/255, height/starts 0/127/128/255, short/exact/long color arrays, truncation and trailing data. | Preserve field/sentinel order and rotation mask; reject unknown strict holders, malformed components, impossible allocations and residual forms; admit transport-valid rectangles without inventing size or bounds validation. |
| `C3-MAP-PATCH-APPLICATION` | Apply absent/present-empty/full decoration sets and zero-height, short, exact, overlong and out-of-map patches to missing/existing maps with differing scale/lock/dimension. | Create missing data once, retain existing metadata, replace decorations before patch, write X-major/Y-inner using the flat indexes including out-of-X row aliasing, retain ignored suffixes and exact partial mutation/fault prefixes, and refresh texture only after successful application. |
| `C3-MAP-PUBLICATION-ORDER` | Dirty pixels and decorations separately/together across at least ten holding-player publication opportunities, multiple players, changing bounding boxes, unheld maps and locked/unlocked updates. | Consume each exact dirty pixel rectangle immediately; for dirty decorations reproduce old-value `tick++ % 5 == 0`, including first-opportunity publication and no clean-time increment; omit empty packets, keep holder cadence independent and send no sequence/ACK. |
| `C3-TAG-QUERY-CODECS` | Decode null, empty/nested compounds, noncompound roots, depth 511/512/513, NBT accounting below/at/above 2,097,152, signed transactions, malformed tags, truncation and trailing bytes. | Accept END as null and quota/depth-valid compounds only; preserve transaction; fault wrong roots, quota/depth excess, malformed/truncated/residual forms under the skippable-packet error policy. |
| `C3-TAG-QUERY-CORRELATION` | Issue entity/block requests zero through wrap while replacing callbacks; deliver matching, stale, duplicate, null and reordered responses, including a throwing callback; vary permission and missing/present targets. | Preincrement from -1 with signed wrap, keep one latest callback, invoke and only then clear exact matches, retain it on callback throw, and ignore all other responses; let denied/missing-entity requests time out, return explicit null only for permitted missing block entity, and retain no transaction/NBT as world identity. |
| `C3-ADVANCEMENT-CODECS` | Cross reset/show boolean bytes; zero/duplicate/large add/remove/progress collections; parent order, identifiers, requirement nesting/default strings, trusted display components, item templates, enum ordinals, every flag bit, raw floats and nullable epoch-millisecond endpoints. | Preserve reduced advancement grammar and added order; collapse removed duplicates and overwrite duplicate progress keys; fix omitted rewards/criteria/announce-chat, ignore high flag bits, and fault malformed registry/component/enum/identifier/string/allocation/residual forms. |
| `C3-ADVANCEMENT-TREE-APPLICATION` | Reset/nonreset packets with recursive removal, unknown IDs, child-before-parent, missing/cyclic parents, duplicate added IDs, remove/re-add with and without progress, and retained selected tabs/listeners. | Execute reset, remove, dependency-add, progress in order; recursively remove, discard unresolved additions, reproduce duplicate topology and nonreset stale-progress retention, and leave selected-tab clearing to its separate presentation path. |
| `C3-ADVANCEMENT-PROGRESS-PRESENTATION` | Send progress with extra/missing/duplicate criteria, empty and AND-of-OR requirements, null/past/future timestamps, unknown IDs, incomplete/complete/repeated-complete values under reset/show/display/toast/hidden/telemetry combinations. | Normalize criteria to requirement names; complete only nonempty requirements with one obtained member per group; notify every known update; suppress reset telemetry/toasts, gate later toast by show+display+toast only, and allow repeated complete telemetry/toasts without transition inference. |
| `C3-INVENTORY-PROGRESSION-ORDER` | Interleave map dirtiness/ID 51, latest debug requests/ID 123, advancement visibility/ID 130 and ID-50 tab traffic with unrelated container, recipe, chat, block, teleport and liveness packets. | Preserve per-holder map cadence and nontransactional patch order, sole-latest debug correlation and advancement reset/remove/add/progress order; create no correlation or acknowledgement across these domains. |
| `C3-INVENTORY-PROGRESSION-END-TO-END` | Carry and update representative filled maps, query present/missing entities and block entities with permission branches, and earn/revoke/hide/reveal representative advancement trees across reconnect/reload while capturing IDs 51/123/130. | Converge normalized map pixels/decorations and visible advancement definitions/progress through every specified delta/reset branch, return exact debug snapshots only where admitted, and persist only authoritative map/advancement identity/state—not raw registry/packet/transaction IDs, callbacks, texture/tree/toast/telemetry objects or hash order. |
