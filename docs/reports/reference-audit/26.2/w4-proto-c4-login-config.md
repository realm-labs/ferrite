# Minecraft Java 26.2 optional login/configuration protocol audit

## Scope and result

This Wave 2 worker falsified the four optional C4 login/configuration protocol families against the
repository-locked official Minecraft Java 26.2 client and server:

- `PROTO-LOGIN-CLIENTBOUND-OPTIONAL-001`;
- `PROTO-LOGIN-SERVERBOUND-OPTIONAL-001`;
- `PROTO-CONFIGURATION-CLIENTBOUND-OPTIONAL-001`;
- `PROTO-CONFIGURATION-SERVERBOUND-OPTIONAL-001`.

The audit corrected the primary reference and the four exact completion records. It changes no
Ferrite runtime code or implementation disposition.

The principal falsification is resource-pack acknowledgement admission. Configuration accepts
`accepted` and `downloaded` in any current task without validating UUID or task type and without
advancing. Terminal actions run the common response hook and then require only the
`server_resource_pack` task type; their UUID is ignored. The prior serverbound family record
incorrectly said every response required its owned task and that every unsolicited response was a
fault.

The audit also made previously implicit source behavior explicit: client key-send/cipher callback
order and LAN authentication fallback, server asynchronous authentication outcomes, the absence of
a login query/cookie pending ledger, debug-only base custom-click handling, transfer port validation
after remote closure, code-of-conduct auto-accept, cookie replacement/transfer, and atomic
report/server-link replacement.

## Locked evidence

- `target/mc-reference/26.2/server.jar`: SHA-1
  `823e2250d24b3ddac457a60c92a6a941943fcd6a`.
- `target/mc-reference/26.2/client.jar`: SHA-1
  `2dc72797acbc1b63fc16a11c4ac393605f453754`.
- `target/mc-reference/26.2/server-26.2.jar`: implementation jar extracted from the locked server
  bundle by mc-ref.
- `target/mc-reference/26.2/generated/reports/packets.json` and the locked runtime packet mappings.
- Azul Java/Javap 25 at
  `/Users/mikai/Library/Java/JavaVirtualMachines/azul-25/Contents/Home/bin/`.

No wiki, third-party protocol description, live server, or unpinned artifact was used.

## Family findings

### `PROTO-LOGIN-CLIENTBOUND-OPTIONAL-001`

`ClientboundHelloPacket` retains the documented empty server ID, public-key/challenge arrays and
authentication boolean. The base online server emits a 1024-bit RSA key, four-byte random challenge
and `authenticate=true`. The client creates a 128-bit AES key and signed SHA-1 digest, then:

1. changes `CONNECTING -> AUTHORIZING`;
2. performs `joinServer` only when authentication is requested;
3. normally disconnects on proof failure before sending key, but logs and continues for server data
   marked LAN;
4. sends key after moving to `ENCRYPTING`;
5. installs AES/CFB8 ciphers only in the key send-completion callback.

Login finished is accepted by the client from `CONNECTING` or `ENCRYPTING`; it is illegal from
`AUTHORIZING`. Unknown query and cookie requests are answered directly from the received transaction
or key and current transfer-cookie map. There is no separate pending-request ledger or additional
login-substate validation on those client handlers.

### `PROTO-LOGIN-SERVERBOUND-OPTIONAL-001`

Key remains legal only in server state `KEY`. The handler validates the RSA-decrypted challenge,
derives the secret and signed digest, changes to `AUTHENTICATING`, and installs encryption before
starting its session-service thread. A successful returned profile enters `VERIFYING`. A null result
or unavailable authentication service falls back to an offline profile only for an integrated
singleplayer server; a dedicated server disconnects with the respective reason. Remote IP is passed
only when proxy prevention is enabled.

The custom-query answer decoder reads the transaction then treats the entire remaining frame,
including the writer's nullable marker, as a discarded payload capped at 1,048,576 bytes. Base
custom-query and cookie response handlers inspect no transaction, key, body, or login substate and
unconditionally send the unexpected-query disconnect.

### `PROTO-CONFIGURATION-SERVERBOUND-OPTIONAL-001`

The resource-pack action IDs and terminal classification were confirmed. The exact handler order is:

- every response first reaches the common handler;
- a globally required `declined` response disconnects there, regardless of UUID;
- `accepted` and `downloaded` return without task or UUID validation and do not advance;
- every other action calls `finishCurrentTask(server_resource_pack)`;
- a matching task advances even for the wrong UUID, while a different or absent task throws.

Code-of-conduct accept similarly advances only an exact current conduct task. Cookie response always
disconnects as unexpected. Custom click is legal during any configuration task after its codec
bounds; the common handler calls `MinecraftServer#handleCustomClickAction`, whose locked base
implementation only emits a debug log and returns. It neither advances configuration nor mutates
world, player, or persistent state.

### `PROTO-CONFIGURATION-CLIENTBOUND-OPTIONAL-001`

Cookie store overwrites the current key; request returns the current value or null; a transfer
snapshots the cookie map into the next login. Resource-pack URL admission accepts only HTTP/HTTPS;
invalid URLs immediately report `invalid_url`. Pop-one/pop-all and the eight response states were
confirmed.

Code-of-conduct selection uses the latest lowercased client language, then `en_us`, then the first
map value. An exact previously accepted document is auto-accepted; otherwise the client presents a
screen. Rejection disconnects without acknowledgement, and a second document throws.

For a transfer, the client marks itself transferring before validation. Singleplayer faults before
closing. A remote path closes and makes the old connection read-only before `HostAndPort.fromParts`
validates port `0..=65535`; an invalid signed VarInt port therefore leaves the old connection closed
and starts no replacement connection. Later report-details and server-links packets replace their
complete prior collections. Invalid untrusted URIs are omitted from the replacement server-link
list. Reset/chat and clear/show dialog operations remain client presentation state only.

## Persistence and semantic handoffs

Encryption keys, challenges, query transactions, cookies, resource-pack state, conduct text,
transfer state, report details, server links and dialogs are connection/client-service state. None
is written into authoritative world or ECS persistence. The authenticated profile crosses into the
common login admission path only after the session-service result. A successful transfer carries a
cookie/seen-player/insecure-warning snapshot to a new login; it does not reuse the old protocol
listener.

Resource-pack responses hand off to the common response hook before task admission. Custom click
hands off to the server processor and locked no-op/logging hook. Dialog and report data hand off only
to client UI/reporting services. No assigned packet introduces a gameplay registry, entity metadata,
or data-component numeric mapping.

## Independent reproduction vectors

The following packet-body goldens are independent of an encode/decode round trip. Packet IDs and
outer framing are intentionally excluded so each body can be embedded in raw or compressed login
and configuration frames:

```text
login custom-query answer, transaction 0, null writer value:
00 00

configuration cookie response, key minecraft:test, absent value:
0e 6d 69 6e 65 63 72 61 66 74 3a 74 65 73 74 00

configuration resource-pack accepted, zero UUID:
00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 03

configuration resource-pack successfully_loaded, zero UUID:
00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00

configuration transfer host "a", invalid port -1:
01 61 ff ff ff ff 0f

configuration accept-code-of-conduct:
<empty body>
```

The resource-pack vectors must be injected with no task, each wrong task, and the correct task while
varying UUID. The transfer vector must run on remote and singleplayer listeners and assert whether
closure happens before the exception. Authentication fixtures must use a deterministic RSA pair,
AES key, challenge, signed digest and stubbed `joinServer`/`hasJoinedServer` outcomes; a mere packet
round trip does not test send callbacks, asynchronous state, fallback or admission ordering.

Primary bytecode reproduction uses:

```text
MC_REF_JAVAP=/Users/mikai/Library/Java/JavaVirtualMachines/azul-25/Contents/Home/bin/javap

$MC_REF_JAVAP -classpath target/mc-reference/26.2/server-26.2.jar -p -c -constants -s \
  net.minecraft.server.network.ServerLoginPacketListenerImpl \
  net.minecraft.server.network.ServerConfigurationPacketListenerImpl \
  net.minecraft.server.network.ServerCommonPacketListenerImpl

$MC_REF_JAVAP -classpath target/mc-reference/26.2/client.jar -p -c -constants -s \
  net.minecraft.client.multiplayer.ClientHandshakePacketListenerImpl \
  net.minecraft.client.multiplayer.ClientConfigurationPacketListenerImpl \
  net.minecraft.client.multiplayer.ClientCommonPacketListenerImpl
```

Packet codecs, configuration tasks, `Crypt`, `ServerLinks`, dialog codecs, `ServerAddress`, and the
four packet-protocol registration tables were inspected with the same flags.

## Integration-only follow-ups

The assigned files now contain the source corrections. The shared
`protocol/conformance.md` was deliberately not edited. Its `C1-CONFIG-RESOURCE-PACK-GATE` already
states the correct nonterminal/terminal behavior. Integration should strengthen these existing
vectors without changing their ownership:

- `C1-ONLINE-GATE`: add authenticate-false, LAN proof failure, key-send callback and client illegal
  state cases;
- `C1-CONFIG-CUSTOM-CLICK-GATE`: assert that the locked base hook only debug-logs;
- `C1-CONFIG-TRANSFER-GATE`: distinguish unchecked wire VarInt from client semantic port validation
  after remote closure;
- login/config cookie gates: assert direct client reply without a pending ledger and atomic cookie
  overwrite/transfer snapshot.

No runtime packet catalog, shared conformance document, or cross-family record was changed here.

## Unresolved gates

No source fact required guessing and no new unknown was added. These four families remain C4
`GatedOptional`: online session-service availability/results and user choices for resource packs or
conduct require deterministic service/UI fixtures for execution, not live external services.

## Verification

All required protocol and full-reference checks passed with the repository cache and Azul Java 25:

```text
MC_REF_JAVA=/Users/mikai/Library/Java/JavaVirtualMachines/azul-25/Contents/Home/bin/java \
MC_REF_JAVAP=/Users/mikai/Library/Java/JavaVirtualMachines/azul-25/Contents/Home/bin/javap \
  cargo run -p mc-reference --bin mc-ref -- protocol inventory

MC_REF_JAVA=/Users/mikai/Library/Java/JavaVirtualMachines/azul-25/Contents/Home/bin/java \
MC_REF_JAVAP=/Users/mikai/Library/Java/JavaVirtualMachines/azul-25/Contents/Home/bin/javap \
  cargo run -p mc-reference --bin mc-ref -- protocol coverage

MC_REF_JAVA=/Users/mikai/Library/Java/JavaVirtualMachines/azul-25/Contents/Home/bin/java \
MC_REF_JAVAP=/Users/mikai/Library/Java/JavaVirtualMachines/azul-25/Contents/Home/bin/javap \
  cargo run -p mc-reference --bin mc-ref -- protocol readiness

MC_REF_JAVA=/Users/mikai/Library/Java/JavaVirtualMachines/azul-25/Contents/Home/bin/java \
MC_REF_JAVAP=/Users/mikai/Library/Java/JavaVirtualMachines/azul-25/Contents/Home/bin/javap \
  cargo run -p mc-reference --bin mc-ref -- protocol verify

MC_REF_JAVA=/Users/mikai/Library/Java/JavaVirtualMachines/azul-25/Contents/Home/bin/java \
MC_REF_JAVAP=/Users/mikai/Library/Java/JavaVirtualMachines/azul-25/Contents/Home/bin/javap \
  cargo run -p mc-reference --bin mc-ref -- verify --offline

git diff --check
```

The protocol inventory contained 256 packets with digest
`f34b0956b6399c749d4638cd6d3c9226685f41fa`; coverage assigned all packets to 58 families with 44
`Specified` and 14 `GatedOptional`, readiness completed, and protocol verification matched the
unchanged runtime packet catalog. Full offline verification passed 417 documentation IDs, 331
completion slices, 2,798 locators across 952 official classes, 9,078 locked IDs, 307 experiment
definitions, all protocol/surface ledgers, and the unchanged implementation-manifest consistency
check. `git diff --check` reported no error.

Rust formatting and Clippy were not run because this change is documentation-only and contains no
Rust or runtime implementation changes.
