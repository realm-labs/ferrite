# Minecraft 26.2 Required Serverbound Configuration Family

`G01-P3-F002` implements
`PROTO-CONFIGURATION-SERVERBOUND-REQUIRED-001` in `ferrite-protocol`. Its six packet identities are:

| Wire ID | Locked identity | Ferrite responsibility |
|---:|---|---|
| 0 | `minecraft:client_information` | Latest bounded play-cookie preferences |
| 2 | `minecraft:custom_payload` | Ignored brand and bounded unknown payloads |
| 3 | `minecraft:finish_configuration` | `join_world` terminal acknowledgement |
| 4 | `minecraft:keep_alive` | Exact outstanding signed-long challenge |
| 5 | `minecraft:pong` | Connection-local ignored signed-int response |
| 7 | `minecraft:select_known_packs` | At-most-64 ordered pack selection |

Dispatch is state- and direction-local through the locked packet catalog. Unknown IDs fail closed.
Catalogued C4 packet identities remain outside this required codec and cannot accidentally advance a
C1 task.

## Codec contract

Client information encodes language as `UTF(16)`, signed view distance, chat visibility ordinal
`0..=2`, chat-colors Boolean, unsigned model byte, main-hand ordinal `0..=1`, text-filtering and
server-listing Booleans, and particle-status ordinal `0..=2`. Decode rejects every unknown enum
ordinal. Repeated valid records replace the connection's prior record; only the latest record enters
the play listener cookie.

The built-in `minecraft:brand` body is `UTF(32767)`. The base server ignores it and every other
serverbound custom payload. Unknown channel remainders are nevertheless consumed only through the
inclusive vanilla limit of 32,767 bytes, then discarded. A discarded payload cannot be re-encoded
because its bytes deliberately were not retained.

Known-pack entries reuse the shared connection-local three-`UTF(32767)` value. Responses contain at
most 64 entries and preserve list order. Only exact list equality with the offer enables NBT
elision for entries sourced from offered packs. Empty/empty is an exact match but names no data to
elide; a subset, reordering, unknown entry, or any other inequality selects full registry data.

## Task and liveness state

`ConfigurationServerSession` begins with the synchronize-registries task current. A known-pack
response is accepted once, computes the exact-equality branch, and advances to spawn preparation.
After the runtime reports spawn readiness and sends clientbound finish, only serverbound finish is
terminally legal. Its action freezes the latest client-information cookie and requires the runtime
to execute this order:

1. finish `join_world`;
2. install clientbound play;
3. repeat admission and duplicate-player checks;
4. create the player with the latest cookie;
5. install serverbound play;
6. admit or emit ordinary play packets.

The session remains in an explicit installing-play state until the runtime confirms all those
steps. Early, duplicate, or wrong-task known-pack/finish packets terminally fault configuration.
Packet structs, IDs, selections, and liveness state never enter simulation or persistence.

For a remote connection, the scheduler creates no challenge before 15,000 milliseconds. At the
boundary it uses the current signed millisecond timestamp as both challenge and send time. An exact
echo clears the challenge and updates latency with Java's integer formula
`(old_latency * 3 + elapsed) / 4`. A stale, unsolicited, or mismatched echo disconnects, as does the
next 15,000-millisecond boundary with a challenge pending. The singleplayer owner receives no
scheduled challenges and ignores invalid echoes. Pong is independent and changes neither task nor
keepalive state.

## Evidence

`crates/ferrite-protocol/tests/c1/configuration_serverbound_required.rs` checks all six independent
C1 goldens, field and enum boundaries, 16 UTF-16-unit language limit, signed/unsigned byte
endpoints, 32,767-byte custom payload boundary, 64-pack boundary, exact equality branches, latest
client information, task faults, finish installation order, 15-second liveness boundaries, Java
latency arithmetic, owner exemption, pong independence, and optional/unknown packet refusal.

Primary locked-source anchors are `ConfigurationProtocols`, `ServerboundClientInformationPacket`,
`ClientInformation`, `ServerboundCustomPayloadPacket`, `ServerboundSelectKnownPacks`,
`SynchronizeRegistriesTask`, `ServerboundFinishConfigurationPacket`,
`ServerConfigurationPacketListenerImpl`, and `ServerCommonPacketListenerImpl`.
