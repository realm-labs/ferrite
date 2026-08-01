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

The current observation catalog is `client_status`, `player_state`, `inventory_state`,
`crosshair_state`, `screen_state`, `nearby_blocks`, and `client_errors`. Minecraft objects are read
only at the end of a client tick and copied into immutable records before HTTP workers can observe
them. Game control actions arrive in later Goal 02 batches.

The complete scope, security boundary, and acceptance requirements are defined in
`docs/goals/02-client-mcp-automation.md` at the repository root.
