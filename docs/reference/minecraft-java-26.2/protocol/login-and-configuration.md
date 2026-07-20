# C1 Login and Configuration

This page is the normative C1 entry sequence for an unmodified Java Edition `26.2` client. It
source-specifies login, configuration, and their terminal transition. The following
[clientbound](play-clientbound.md) and [serverbound](play-serverbound.md) pages own minimal play
entry; configuration completion does not implicitly specify those packets or any C2 behavior.

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

## Configuration packet inventory

All fields below follow the primitive and compression rules in
[`framing-and-primitives.md`](framing-and-primitives.md). A `component-NBT` or `dialog-NBT` value is
one unnamed network NBT tag decoded through the trusted context-free codec: the tag retains the
512-depth structural limit but has no quota below the enclosing transport limits.

| Direction | ID | Identity | Fields in exact order | C1 role |
|---|---:|---|---|---|
| clientbound | `0` | `minecraft:cookie_request` | identifier key | gated |
| clientbound | `1` | `minecraft:custom_payload` | identifier channel; channel-specific remainder | required brand / gated extensions |
| clientbound | `2` | `minecraft:disconnect` | component-NBT reason | required failure |
| clientbound | `3` | `minecraft:finish_configuration` | none; terminal | required |
| clientbound | `4` | `minecraft:keep_alive` | signed big-endian long token | required liveness |
| clientbound | `5` | `minecraft:ping` | signed big-endian int token | required echo |
| clientbound | `6` | `minecraft:reset_chat` | none | gated reconfiguration |
| clientbound | `7` | `minecraft:registry_data` | registry identifier; entry list | required |
| clientbound | `8` | `minecraft:resource_pack_pop` | presence boolean; optional UUID | gated |
| clientbound | `9` | `minecraft:resource_pack_push` | UUID; URL `UTF(32767)`; hash `UTF(40)`; required boolean; optional component-NBT prompt | gated |
| clientbound | `10` | `minecraft:store_cookie` | identifier; byte array capped at `5_120` | gated |
| clientbound | `11` | `minecraft:transfer` | host `UTF(32767)`; port VarInt | gated |
| clientbound | `12` | `minecraft:update_enabled_features` | identifier-set count VarInt; identifiers | required |
| clientbound | `13` | `minecraft:update_tags` | registry-to-tag-payload map | required |
| clientbound | `14` | `minecraft:select_known_packs` | known-pack list | required |
| clientbound | `15` | `minecraft:custom_report_details` | up to 32 `UTF(128)` to `UTF(4096)` pairs | gated |
| clientbound | `16` | `minecraft:server_links` | untrusted-link list | gated |
| clientbound | `17` | `minecraft:clear_dialog` | none | gated |
| clientbound | `18` | `minecraft:show_dialog` | direct dialog-NBT | gated |
| clientbound | `19` | `minecraft:code_of_conduct` | text `UTF(32767)` | gated |
| serverbound | `0` | `minecraft:client_information` | client-information record | required |
| serverbound | `1` | `minecraft:cookie_response` | identifier; nullable byte array capped at `5_120` | gated |
| serverbound | `2` | `minecraft:custom_payload` | identifier channel; channel-specific remainder | required brand / gated extensions |
| serverbound | `3` | `minecraft:finish_configuration` | none; terminal | required |
| serverbound | `4` | `minecraft:keep_alive` | signed big-endian long token | required acknowledgement |
| serverbound | `5` | `minecraft:pong` | signed big-endian int token | required echo |
| serverbound | `6` | `minecraft:resource_pack` | UUID; action enum VarInt `0..=7` | gated |
| serverbound | `7` | `minecraft:select_known_packs` | at most 64 known packs | required |
| serverbound | `8` | `minecraft:custom_click_action` | identifier; at-most-65,536-byte length-prefixed optional NBT | gated |
| serverbound | `9` | `minecraft:accept_code_of_conduct` | none | gated |

A known pack is three `UTF(32767)` strings: namespace, ID, and version. The server offer's list count
has no explicit cap below the frame; the response is capped at 64. Client information is language
`UTF(16)`, signed view-distance byte, chat-visibility enum VarInt (`full=0`, `system=1`, `hidden=2`),
chat-colors boolean, unsigned model-customization byte, main-hand enum VarInt (`left=0`, `right=1`),
text-filtering boolean, server-listing boolean, and particle-status enum VarInt (`all=0`,
`decreased=1`, `minimal=2`). Unknown enum ordinals fail decode.

The only built-in configuration custom payload is `minecraft:brand`, whose remainder is one
`UTF(32767)` string. An unknown clientbound channel consumes and discards at most `1_048_576`
remainder bytes; an unknown serverbound channel consumes and discards at most `32_767`. The vanilla
client retains a received server brand, while the base server deliberately ignores every received
custom payload, including client brand.

Primary codec anchors are `net.minecraft.network.protocol.configuration.ConfigurationProtocols`,
`net.minecraft.network.protocol.configuration.ClientboundRegistryDataPacket`,
`net.minecraft.network.protocol.configuration.ClientboundUpdateEnabledFeaturesPacket`,
`net.minecraft.network.protocol.configuration.ClientboundSelectKnownPacks`,
`net.minecraft.network.protocol.configuration.ServerboundSelectKnownPacks`,
`net.minecraft.network.protocol.common.ClientboundUpdateTagsPacket`,
`net.minecraft.network.protocol.common.ServerboundClientInformationPacket`,
`net.minecraft.server.level.ClientInformation`,
`net.minecraft.network.protocol.common.ClientboundCustomPayloadPacket`, and
`net.minecraft.network.protocol.common.ServerboundCustomPayloadPacket`.

## Registry, tag, and feature mapping

One registry-data entry is an identifier followed by a presence boolean and, when present, an
unnamed network NBT tag with a `2_097_152`-byte accumulator quota and depth 512. The packet's list
count is a VarInt with no explicit cap below the frame. Arrival order is the registry's numeric wire
order: the first entry is ID zero, and later packets for the same registry append rather than
replace. The tag packet is:

```text
registry_count:VarInt
repeat registry_count {
    registry:identifier
    tag_count:VarInt
    repeat tag_count {
        tag:identifier
        member_count:VarInt
        member_ids[member_count]:VarInt
    }
}
```

Each member ID indexes the matching static or reconstructed dynamic registry. Tag payloads for a
repeated registry key replace the prior payload. Negative counts, truncated values, bad identifiers,
out-of-range tag member IDs, duplicate element keys, or element NBT that fails the registry's locked
element codec must fail no later than final registry collection; they must never be normalized into
Ferrite persistence.

The locked synchronized registry order is exactly:

1. `minecraft:worldgen/biome`, `minecraft:chat_type`, `minecraft:trim_pattern`,
   `minecraft:trim_material`;
2. `minecraft:wolf_variant`, `minecraft:wolf_sound_variant`, `minecraft:pig_variant`,
   `minecraft:pig_sound_variant`, `minecraft:frog_variant`, `minecraft:cat_variant`,
   `minecraft:cat_sound_variant`, `minecraft:cow_sound_variant`, `minecraft:cow_variant`,
   `minecraft:chicken_sound_variant`, `minecraft:chicken_variant`,
   `minecraft:zombie_nautilus_variant`;
3. `minecraft:painting_variant`, `minecraft:sulfur_cube_archetype`,
   `minecraft:dimension_type`, `minecraft:damage_type`, `minecraft:banner_pattern`,
   `minecraft:enchantment`, `minecraft:jukebox_song`, `minecraft:instrument`;
4. `minecraft:test_environment`, `minecraft:test_instance`, `minecraft:dialog`,
   `minecraft:world_clock`, `minecraft:timeline`.

The server offers the known-pack descriptors exposed by its active resource packs. The client
returns the sublist it can supply locally. Only exact list equality is trusted: equality causes
entries originating in those packs to retain their identifiers but omit NBT; any other response
causes the server to ignore the whole selection and send NBT for every entry. An empty offer and
empty response are therefore the interoperable no-cache path. The vanilla core descriptor is
`minecraft:core:26.2`.

Enabled features are a set, not a numeric registry. The locked names are `minecraft:vanilla`,
`minecraft:trade_rebalance`, `minecraft:redstone_experiments`, and
`minecraft:minecart_improvements`; a normal world includes `minecraft:vanilla`. The client ignores
unknown names with a warning and collapses duplicates. Ferrite must keep these connection-local
feature selections separate from authoritative registry identities and from gameplay storage.

Primary mapping anchors are `net.minecraft.core.RegistrySynchronization`,
`net.minecraft.resources.RegistryDataLoader`,
`net.minecraft.tags.TagNetworkSerialization`,
`net.minecraft.client.multiplayer.RegistryDataCollector`,
`net.minecraft.server.network.config.SynchronizeRegistriesTask`,
`net.minecraft.server.packs.repository.KnownPack`, and
`net.minecraft.world.flag.FeatureFlagRegistry`.

## Required configuration state machine

Configuration begins after the login acknowledgement has installed both configuration directions.
Packets in opposite directions may interleave, but each direction preserves its listed order.

1. The vanilla client immediately sends its `minecraft:brand` payload and client information. The
   server accepts later client-information replacements; the most recently received record is used
   to create the play listener cookie.
2. The server sends its brand, optionally server links, then enabled features. It starts the
   `synchronize_registries` task by sending one known-pack offer.
3. A known-pack response is legal only while that task is current. The server applies the exact
   equality rule above, sends one or more registry-data packets in synchronized-registry order, then
   one update-tags packet. A response before negotiation or a duplicate after task advancement is a
   configuration fault.
4. Optional code-of-conduct and resource-pack tasks run next when configured. Without them, the
   server begins `prepare_spawn`: it loads saved player data and waits for the radius-three spawn
   area without sending a task packet. Common keepalive remains active during the wait.
5. Once spawn preparation is ready, the `join_world` task sends terminal clientbound finish. The
   client finalizes dynamic registries/tags and data-component initializers, installs play
   clientbound, sends terminal serverbound finish under configuration, then installs play
   serverbound.
6. The server accepts finish only while `join_world` is current, installs play clientbound,
   rechecks duplicate/admission policy, creates the player, and installs play serverbound before
   emitting the first play packet sequence.

An early finish, wrong task acknowledgement, duplicate known-pack response, or task exception is a
configuration fault; a task exception sends the configuration-error disconnect. The second
admission check may send its policy reason, and spawn/setup failure sends invalid-player-data. A
configuration disconnect carries trusted context-free component NBT and closes the connection.

The common liveness scheduler sends a keepalive after 15,000 ms for a non-singleplayer owner and
uses the current millisecond timestamp as its signed-long challenge. A second 15,000 ms interval
with a challenge pending disconnects for timeout. Only an exact pending echo clears it and updates
latency; a stale, unsolicited, or mismatched keepalive disconnects unless this is the singleplayer
owner. The vanilla client sends the echo immediately unless rendering is frozen at event polling;
in that case it defers the echo until unfrozen and drops that deferred packet after one minute.
Ping is independent: the client immediately returns the identical signed-int pong and the base
server otherwise ignores pong.

Primary state anchors are `net.minecraft.server.network.ServerConfigurationPacketListenerImpl`,
`net.minecraft.server.network.ServerCommonPacketListenerImpl#keepConnectionAlive`,
`net.minecraft.server.network.ServerCommonPacketListenerImpl#handleKeepAlive`,
`net.minecraft.server.network.config.PrepareSpawnTask`,
`net.minecraft.server.network.config.JoinWorldTask`,
`net.minecraft.client.multiplayer.ClientConfigurationPacketListenerImpl`, and
`net.minecraft.server.players.PlayerList#placeNewPlayer`.

## Optional configuration gates

- Cookie request/store/response reuse the login key and 5,120-byte formats. The base configuration
  server sends no request and disconnects every response as unexpected; store is clientbound only.
- Resource-pack pop carries absent UUID to remove all packs or a present UUID to remove one. Push
  actions are `successfully_loaded=0`, `declined=1`, `failed_download=2`, `accepted=3`,
  `downloaded=4`, `invalid_url=5`, `failed_reload=6`, and `discarded=7`; only accepted/downloaded are
  nonterminal. A configured task blocks until a terminal response. Declining a required pack
  disconnects. The locked task does not correlate a terminal response UUID before advancing.
- A code-of-conduct task chooses text by the latest lowercased client language, then `en_us`, then
  the map's first value. A duplicate clientbound document fails on the client. Accept is legal only
  while that task is current; rejection closes client-side without an accept packet.
- Custom click carries an identifier plus a VarInt-length-prefixed optional NBT value. The prefix is
  capped at 65,536 bytes; the NBT accumulator is capped at 32,768 bytes and depth 16. The base server
  dispatches it to the server-owned custom-click handler.
- Report details cap the map at 32 entries, keys at 128 code units, and values at 4,096. Server-link
  entries encode a boolean (`true` for known type ID `0..=9`, `false` for component-NBT label) then
  URL `UTF(32767)`; an out-of-range known-type ID maps to type zero (`bug_report`). The list is
  frame-bounded, and the client drops invalid untrusted URIs.
- Transfer uses an unchecked codec-level VarInt port, closes the current remote connection, and
  carries cookies into a new transfer-intention login. It is invalid from singleplayer. Reset-chat
  clears retained chat state for play-to-configuration re-entry. Clear/show dialog and context-free
  dialog NBT affect only client presentation.

These gates are not part of the minimum fresh offline trace. A Ferrite adapter may support them only
through an explicitly owned connection service; it must preserve the documented refusal, blocking,
or presentation behavior instead of silently treating an unsolicited packet as C1 success.

Optional-path anchors are `net.minecraft.server.network.config.ServerResourcePackConfigurationTask`,
`net.minecraft.server.network.config.ServerCodeOfConductConfigurationTask`,
`net.minecraft.network.protocol.common.ClientboundResourcePackPushPacket`,
`net.minecraft.network.protocol.common.ServerboundResourcePackPacket`,
`net.minecraft.network.protocol.common.ServerboundCustomClickActionPacket`,
`net.minecraft.network.protocol.common.ClientboundCustomReportDetailsPacket`,
`net.minecraft.server.ServerLinks`, `net.minecraft.server.dialog.Dialog`, and
`net.minecraft.client.multiplayer.ClientCommonPacketListenerImpl`.
