# Ferrite Client MCP

This client-only Fabric mod instruments the locked Minecraft Java 26.2 client for Ferrite
acceptance testing. It is test infrastructure, not part of the Ferrite server runtime and not an
unmodified-client compatibility claim.

## Build

Use a Java 25 runtime to launch the checked-in Gradle wrapper:

```text
JAVA_HOME=/path/to/jdk-25 ./gradlew --no-daemon check
JAVA_HOME=/path/to/jdk-25 ./gradlew --no-daemon build
```

The remapped mod artifact is written below `build/libs`. Gradle caches, local client state, and run
directories are ignored. Do not place Mojang jars, assets, mappings payloads, access tokens, or a
personal Minecraft game directory in source control.

## MCP startup

The endpoint is disabled unless `FERRITE_CLIENT_MCP_SECRET_FILE` names an absolute or resolvable
owner-only file containing a 32–256 byte bearer secret. The launcher will create that file; do not
pass an account token or reuse a Minecraft credential.

Optional configuration:

| Environment variable | Meaning | Default |
|---|---|---:|
| `FERRITE_CLIENT_MCP_PORT` | Loopback TCP port; `0` requests an ephemeral port | `0` |
| `FERRITE_CLIENT_MCP_READY_FILE` | Atomic secret-free JSON endpoint discovery file | none |

The mod always binds `127.0.0.1` and exposes `/mcp`. It requires `Authorization: Bearer <secret>`,
validates browser origins, bounds requests and worker queues, supports MCP `2025-11-25` and
`2025-06-18`, and permits one control session. GET/SSE is intentionally unavailable.

The observation catalog is `client_status`, `player_state`, `inventory_state`,
`crosshair_state`, `screen_state`, `nearby_blocks`, and `client_errors`. Minecraft objects are read
only at the end of a client tick and copied into immutable records before HTTP workers can observe
them. `take_screenshot` returns a bounded PNG from the real main framebuffer as MCP image content,
with dimensions, client tick, byte length, and SHA-256 metadata. Tick-fenced tools cover state
waits, movement, view, jump, sneak, sprint, attack, use, hotbar, drop, hand swap, chat, inventory
open/close, native cursor movement, and revision-fenced slot clicks.

## Isolated Quick Play launcher

Build the standalone JDK-only supervisor:

```text
JAVA_HOME=/path/to/jdk-25 ./gradlew --no-daemon launcherJar
```

Then start one instrumented client from the repository root:

```text
/path/to/jdk-25/bin/java \
  -jar tools/ferrite-client-mcp/build/libs/ferrite-client-mcp-0.1.0-SNAPSHOT-launcher.jar \
  --workspace /absolute/path/to/ferrite \
  --java-home /path/to/jdk-25 \
  --endpoint 127.0.0.1:25565
```

The launcher verifies the locked external 26.2 client SHA-1 and size, creates a random run below
`target/client-mcp-runs`, generates an owner-only MCP secret, disables first-run screens, and starts
Quick Play without clicks. It prints only a secret-free ready record. The default readiness timeout
is 90 seconds and maximum runtime is 300 seconds; `--ready-timeout-seconds` and
`--max-runtime-seconds` are bounded overrides. On exit, timeout, interruption, or launch failure it
terminates the Gradle/client process tree and deletes the isolated run. `--retain-run` is only for
local diagnosis and leaves the ignored game directory, client log, and secret for manual cleanup.

The run root must remain below the workspace `target` directory. The launcher never reads HMCL,
the normal Minecraft directory, account databases, saves, options, or access tokens.

## Unattended gameplay acceptance

Build the current Ferrite process and both Java supervisor artifacts, then run both scenarios:

```text
cargo build -p ferrite-server
JAVA_HOME=/path/to/jdk-25 ./gradlew --no-daemon clean check build
/path/to/jdk-25/bin/java \
  -jar build/libs/ferrite-client-mcp-0.1.0-SNAPSHOT-acceptance.jar \
  --workspace /absolute/path/to/ferrite \
  --java-home /path/to/jdk-25 \
  --mode all
```

`reference` and `ferrite` are also valid focused modes. The runner verifies both locked Mojang
artifacts, uses the deterministic offline identity `FerriteMcp`, starts isolated servers and
clients on loopback ports, drives normal MCP gameplay tools, and writes secret-free evidence below
`target/client-mcp-evidence`. Reference responses, tick receipts, screenshots, client/server logs,
and Ferrite management snapshots are retained there for local inspection. Generated worlds and
evidence bundles are ignored and must not be committed.

The complete scope, security boundary, and acceptance requirements are defined in
`docs/goals/02-client-mcp-automation.md` at the repository root. The operator workflow and failure
triage guide are in `docs/development/client-mcp-automation.md`.
