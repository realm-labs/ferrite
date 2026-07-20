# C1 Login and Configuration

This page is the normative C1 entry sequence for an unmodified Java Edition `26.2` client. The login
state is source-specified below. Configuration packet schemas and the minimum configuration task
sequence remain open and are still `Todo` in the protocol ledger; nothing in this page promotes
them implicitly.

## Login packet inventory

| Direction | ID | Identity | Fields in exact order |
|---|---:|---|---|
| clientbound | `0` | `minecraft:login_disconnect` | component as lenient JSON `UTF(262144)` |
| clientbound | `1` | `minecraft:hello` | server ID `UTF(20)`; public-key byte array; challenge byte array; authenticate boolean |
| clientbound | `2` | `minecraft:login_finished` | game profile; server-session UUID |
| clientbound | `3` | `minecraft:login_compression` | threshold VarInt |
| clientbound | `4` | `minecraft:custom_query` | transaction VarInt; identifier; raw remainder payload |
| clientbound | `5` | `minecraft:cookie_request` | identifier |
| serverbound | `0` | `minecraft:hello` | player name `UTF(16)`; supplied profile UUID |
| serverbound | `1` | `minecraft:key` | encrypted secret byte array; encrypted challenge byte array |
| serverbound | `2` | `minecraft:custom_query_answer` | transaction VarInt; nullable-answer marker and raw answer bytes |
| serverbound | `3` | `minecraft:login_acknowledged` | none |
| serverbound | `4` | `minecraft:cookie_response` | identifier; nullable cookie byte array |

UUIDs are 16 bytes: most-significant signed long then least-significant signed long, both network
byte order. A boolean encoder emits `00`/`01`; decode treats zero as false and any nonzero byte as
true. An identifier is `UTF(32767)` followed by the locked namespaced-identifier parse.

The game-profile layout is:

1. profile UUID;
2. name `UTF(16)`;
3. property count VarInt, bounded to `0..=16`;
4. each property: name `UTF(64)`, value `UTF(32767)`, signature-present boolean, and when present a
   signature `UTF(1024)`;
5. the separate server-session UUID.

The two unbounded-looking encryption byte arrays are each VarInt-length-prefixed and in practice
bounded by the remaining packet frame. The custom-query request payload is the raw frame remainder,
at most `1_048_576` bytes, after transaction and identifier. The answer decoder likewise caps and
discards the entire remainder at `1_048_576` bytes; the official writer represents null with one
false marker byte. Cookie payload presence is one boolean and a present byte array is capped at
`5_120` bytes.

Primary codec anchors are `net.minecraft.network.protocol.login.ServerboundHelloPacket`,
`net.minecraft.network.protocol.login.ServerboundKeyPacket`,
`net.minecraft.network.protocol.login.ServerboundCustomQueryAnswerPacket`,
`net.minecraft.network.protocol.login.ServerboundLoginAcknowledgedPacket`,
`net.minecraft.network.protocol.login.ClientboundHelloPacket`,
`net.minecraft.network.protocol.login.ClientboundLoginFinishedPacket`,
`net.minecraft.network.protocol.login.ClientboundLoginCompressionPacket`,
`net.minecraft.network.protocol.login.ClientboundLoginDisconnectPacket`,
`net.minecraft.network.protocol.login.ClientboundCustomQueryPacket`,
`net.minecraft.network.protocol.cookie.ClientboundCookieRequestPacket`, and
`net.minecraft.network.protocol.cookie.ServerboundCookieResponsePacket`.

## Required offline-mode state machine

C1 begins after a protocol-`776` login intention has installed the login codecs. The server listener
starts in `HELLO` and has a 600-tick login timeout counter.

1. The client sends one login hello. The packet codec permits 0 through 16 name code units. The
   handler additionally requires every code unit to be strictly greater than ASCII space and less
   than DEL; therefore empty is accepted, but spaces, control characters, DEL, and non-ASCII are
   rejected. A hello outside `HELLO` is a protocol fault.
2. When the authentication branch is not taken—authentication is disabled or the connection is
   in-memory—the supplied profile UUID is ignored. The server derives a version-3 UUID exactly as
   `UUID.nameUUIDFromBytes(UTF8("OfflinePlayer:" + name))`, creates an empty-property profile, and
   moves to `VERIFYING`.
3. On a listener tick in `VERIFYING`, admission policy checks the remote address and normalized
   name/UUID. A policy reason sends login disconnect and closes. A connection-local intended UUID,
   when present, must equal the normalized UUID. Existing players with the same normalized UUID are
   disconnected first; login waits until the old player leaves.
4. If the configured compression threshold is nonnegative and this is not a memory connection, the
   server sends login compression uncompressed. Only its send-completion callback installs the C1
   compression envelope in both directions. The dedicated-server default is `256`; a negative value
   skips this packet and compression.
5. The server enters `PROTOCOL_SWITCHING` before sending terminal login finished. It contains the
   normalized profile and a random server-session UUID shared by concurrent connections and reset
   after the connection list becomes empty.
6. The client accepts login finished directly from its initial connecting state, installs the
   configuration clientbound codec/listener, sends terminal login acknowledged under the
   serverbound login codec, then installs the configuration serverbound codec. It immediately sends
   configuration brand custom payload and client information.
7. The server accepts the acknowledgement only in `PROTOCOL_SWITCHING`, installs configuration
   clientbound, builds the normalized connection cookie, installs configuration serverbound, starts
   configuration tasks, and marks login `ACCEPTED`.

The counter comparison disconnects a login with the slow-login reason when the listener's prior
tick value is `600`. Duplicate hello, early/duplicate acknowledgement, a key packet outside `KEY`,
malformed fields, or packet IDs illegal for the current state enter login fault handling. Normal
login disconnect is sent before TCP closure whenever the login codec remains usable.

Primary state anchors are `net.minecraft.server.network.ServerLoginPacketListenerImpl#handleHello`,
`net.minecraft.server.network.ServerLoginPacketListenerImpl#startClientVerification`,
`net.minecraft.server.network.ServerLoginPacketListenerImpl#verifyLoginAndFinishConnectionSetup`,
`net.minecraft.server.network.ServerLoginPacketListenerImpl#finishLoginAndWaitForClient`,
`net.minecraft.server.network.ServerLoginPacketListenerImpl#handleLoginAcknowledgement`,
`net.minecraft.server.network.ServerLoginPacketListenerImpl#tick`,
`net.minecraft.core.UUIDUtil#createOfflinePlayerUUID`,
`net.minecraft.util.StringUtil#isValidPlayerName`, and
`net.minecraft.client.multiplayer.ClientHandshakePacketListenerImpl#handleLoginFinished`.

## Normalized boundary

The login adapter maps hello to a connection-local login request, then maps the selected
authentication policy to one normalized identity: UUID, name, and bounded profile properties. The
client-supplied UUID is evidence to an authentication path, never the offline authoritative UUID.
Compression threshold and server-session UUID are connection/session projection state. None of the
login JSON, packet structs, property codecs, supplied UUID, or packet IDs enter world persistence or
ECS state. There are no gameplay registry, entity-metadata, or data-component numeric mappings in
login.

## Explicit optional gates

Online mode is a C4 gate. After a valid hello, a server that uses authentication sends encryption
hello and moves to `KEY`. The locked server uses an empty server ID, a 1024-bit RSA public key, a
random four-byte challenge, and `authenticate = true`. The client creates a 128-bit AES secret,
RSA-encrypts both secret and challenge, optionally proves the SHA-1 session digest to the session
service, sends key, then installs `AES/CFB8/NoPadding` in both directions using the secret bytes as
the IV. The server requires `KEY`, decrypts and compares the challenge, installs encryption only
after the key packet, authenticates the name/digest (optionally binding remote IP), and then rejoins
the common `VERIFYING` path. A bad challenge or cryptographic decode is a protocol fault.

The base `26.2` login listener never sends custom query or cookie request. The vanilla client would
answer an unknown custom query with the same transaction ID and a null answer, and would answer a
cookie request with the same key plus a nullable value from transfer cookie state. The base server
disconnects on every custom-query answer or cookie response with its unexpected-query reason. A
Ferrite implementation must either retain that exact closed gate or define a separately owned
extension listener; it must not silently accept unsolicited answers.

Optional-path anchors are `net.minecraft.server.network.ServerLoginPacketListenerImpl#handleKey`,
`net.minecraft.server.network.ServerLoginPacketListenerImpl#handleCustomQueryPacket`,
`net.minecraft.server.network.ServerLoginPacketListenerImpl#handleCookieResponse`,
`net.minecraft.util.Crypt#generateSecretKey`, `net.minecraft.util.Crypt#generateKeyPair`,
`net.minecraft.util.Crypt#getCipher`,
`net.minecraft.client.multiplayer.ClientHandshakePacketListenerImpl#handleHello`,
`net.minecraft.client.multiplayer.ClientHandshakePacketListenerImpl#handleCustomQuery`, and
`net.minecraft.client.multiplayer.ClientHandshakePacketListenerImpl#handleRequestCookie`.

## Configuration boundary still open

Login success is not play entry. The configuration listener must still complete known-pack
selection, registries, tags, enabled features, required client information, and finish
acknowledgement, while explicitly gating its optional packet families. Those exact schemas and
ordering remain the next C1 batch.
