# G02-P1-B2 — Authenticated bounded MCP transport

## Outcome

The client mod now embeds a pure-Java Streamable HTTP MCP endpoint. It is disabled unless an
owner-only secret file is explicitly configured, binds only IPv4 loopback, publishes an optional
secret-free ready file, owns one bounded control session, and stops with the Minecraft client.

The protocol follows the current finalized MCP revision `2025-11-25` and retains negotiated
compatibility with `2025-06-18`. The implementation boundary was checked against the official
[transport](https://modelcontextprotocol.io/specification/2025-11-25/basic/transports),
[lifecycle](https://modelcontextprotocol.io/specification/2025-11-25/basic/lifecycle), and
[tool](https://modelcontextprotocol.io/specification/2025-11-25/server/tools) specifications.

## Closed boundaries

- `127.0.0.1` is fixed in configuration; wildcard and external addresses cannot be selected.
- Every request requires a 32–256 byte bearer secret loaded from a regular non-symlink file.
- POSIX group or other access on the secret file is rejected.
- Browser origins must be syntactically valid HTTP(S) loopback origins.
- POST requires the MCP JSON and SSE accept types and an `application/json` request.
- Bodies, worker threads, the executor queue, and active control sessions are bounded.
- Batch JSON-RPC, unknown sessions, mismatched protocol headers, and pre-initialization tool calls
  fail closed.
- GET/SSE is deliberately rejected with `405`; session deletion uses authenticated DELETE.
- Shutdown stops HTTP service, clears session state and secret bytes, and removes the ready file.

The initial `client_status` tool reports transport readiness only and explicitly states that game
observation is not available yet. It cannot send commands, mutate game state, or manufacture
packets.

## Verification

Nine Java tests cover configuration, permission rejection, JSON-RPC/MCP lifecycle, one-session
ownership, authenticated HTTP, origin checks, media types, request bounds, ready-file cleanup, tool
listing, and tool invocation. These commands passed on 2026-08-01:

```text
JAVA_HOME=<local-jdk-25> ./tools/ferrite-client-mcp/gradlew --no-daemon clean check
JAVA_HOME=<local-jdk-25> ./tools/ferrite-client-mcp/gradlew --no-daemon build
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
git diff --check
```

Java compilation uses `-Xlint:all -Werror`. The remapped mod JAR contains only Ferrite MCP classes
and metadata; it embeds no Gson, Fabric, or Mojang class or artifact.
