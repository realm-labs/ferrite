# G04-P5-B1 — Exact-client durable world acceptance

## Outcome

`Satisfied`. The repository-owned pure-Java MCP launched the locked Minecraft Java 26.2 client on
Java 25 and exercised the immutable `ferrite-server` listener in two unattended profiles. The
generated-world profile proves formal generation, complete chunk streaming, authoritative
collision, movement, jump/landing, time/weather observation, framebuffer convergence, required
shutdown flush, restart, and identical spawn terrain. The portal profile proves normal client input
entering a generated source fixture, committed Overworld-to-Nether travel, complete destination
streaming, client dimension convergence, and Nether framebuffer output.

The ignored evidence bundles are:

- `target/client-mcp-evidence/ferrite-visual-a8ee8c36-67b7-4b70-90e7-d7da66356703`;
- `target/client-mcp-evidence/ferrite-portal-774d24f0-5d3e-456a-b9da-e6571a118c1e`.

Both `summary.json` files are `Satisfied`. The bundles, Mojang artifacts, isolated game state,
secrets, logs, and framebuffer bytes remain uncommitted; this record retains only stable assertions
and image digests.

## Generated world, collision, environment, and restart

The client observed complete radius-2 block snapshots containing authoritative stone subsurface and
grass surface state. The server completed the full 25-chunk view (`pending_chunks=0`,
`sent_chunks=25`) before visual capture. Normal forward-key input moved the authoritative player
from spawn to `(-0.5, 69.0, 9.932029621392598)` with `on_ground=true`; a later normal jump returned
to the grounded state without a correction loop or session error.

The new read-only `world_state` MCP observation copied the dimension clock and weather on the client
thread. In this deterministic run the clear-weather projection remained valid at rain/thunder
strength `0.0`, while both the Overworld and default clocks advanced from tick 279 to 331. The tool
does not mutate time or weather and is not a server authority oracle.

The first server then drained normally. A second server reused the exact same state root with new
loopback ports. The exact client rejoined, observed the same ordered spawn block signature, a
non-regressing durable clock, one healthy formal session, and another complete 25-chunk view. This
proves visible restart convergence rather than a second fresh-world launch.

| Framebuffer | Dimensions | Bytes | SHA-256 |
|---|---:|---:|---|
| Generated Overworld | `1708×960` | `737369` | `c5abd359b63417ad1e12bf7a9a7c7e9dd238c01de266b6e4255ba28beeaaca8a` |
| Restarted Overworld | `1708×960` | `825779` | `36a27dfed874e49d1482bcfac9d13de899afa6bb6ab4bbd09c86194ed1e5e452` |

## Authoritative portal travel

The separate portal profile selects the explicit fixture identity
`ferrite:portal_acceptance_fixture_v1`. The fixture delegates to the ordinary Overworld generator
and adds only a three-block source portal column at a fixed local coordinate through the same
bounded asynchronous generation result, Region commit, persistence, projection, and collision
path. Production defaults and migrated configurations remain `ferrite:overworld_v1`. The fixture
does not directly move the client or seed a destination.

The client completed the source view, used ordinary look and forward-key actions, and independently
observed `minecraft:nether_portal` around its feet. After the normal 80-tick contact delay the
production portal runtime ticketed and generated destination columns, resolved/created a safe exit,
committed the dimension transfer, emitted Java Respawn/load-start/correction packets, and restarted
the chunk stream. Final management state recorded:

- dimension `minecraft:the_nether`, Region `(0,0)`;
- `region_transfers=1`;
- position `(0.0, 128.0, 1.2451441124880738)`;
- 25 view chunks, zero pending chunks, and 25 sent chunks;
- no session or lifecycle failure.

The client then observed both player and world dimension as `minecraft:the_nether`, left the loading
screen by sending the normal `player_loaded` packet, and captured the active destination portal
effect. The `1708×960`, 559062-byte PNG has SHA-256
`65abf6304a56b1b505fd50d7daa1af905ca6d89b4aefae062376e97016e88e8c`.

## Defects exposed by the exact client

The acceptance loop found and fixed production defects rather than extending timeouts around them:

- Java 26.2 time projection used the nonexistent `minecraft:day_time` clock identity. Overworld and
  Nether now use the Overworld clock, while End uses the End clock.
- One large socket read could decode more events than the former 64-event queue and later deliver a
  client movement burst into one server tick. Ingress now uses bounded 1 KiB reads and a shared
  32-event session-poll budget; the protocol queue retains bounded burst capacity.
- Synchronous one-worker generation and repeated snapshot reconstruction stalled the gateway.
  Formal runtime generation now uses at most four workers, nonblocking result collection, ordered
  publication, revision-cached immutable snapshots, cached durable records, and one collision view
  per session poll.
- Respawn transition omitted Java 26.2 GameEvent 13 (`level_chunks_load_start`). The client could
  install the Nether and all chunks but remained in `LevelLoadingScreen` and never sent
  `player_loaded`. Transition ordering now emits the load-start event immediately after Respawn.

Management status now exposes bounded session position, dimension, collision grounding, chunk-view
progress, outbound backlog, and last close/error information. It remains read-only.

## Reproduction

```text
cargo build -p ferrite-server
cd tools/ferrite-client-mcp
JAVA_HOME=<jdk-25> ./gradlew --no-daemon check build
<jdk-25>/bin/java -jar build/libs/ferrite-client-mcp-0.1.0-SNAPSHOT-acceptance.jar \
  --workspace <workspace> --java-home <jdk-25> --mode ferrite
<jdk-25>/bin/java -jar build/libs/ferrite-client-mcp-0.1.0-SNAPSHOT-acceptance.jar \
  --workspace <workspace> --java-home <jdk-25> --mode ferrite_portal
```

This is instrumented exact-client evidence. It does not broaden the separate unmodified-client
compatibility claim or claim Mojang same-seed block identity.
