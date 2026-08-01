# G02-P2-B1 — Immutable client observations

## Outcome

The mod now copies Minecraft state at the end of each client tick and publishes one immutable
snapshot through an atomic boundary. HTTP worker threads never retain or read a Minecraft player,
world, screen, menu, ItemStack, entity, hit result, or block state.

The MCP catalog now includes:

| Tool | Copied state |
|---|---|
| `client_status` | client tick, connection phase, game-state availability |
| `player_state` | pose, motion, ground/life flags, health, hunger, mode, dimension, flight |
| `inventory_state` | selected hotbar slot and bounded non-empty item records |
| `crosshair_state` | ordinary miss, block, or entity pick result |
| `screen_state` | screen, title, overlay, dimensions, menu ID/revision/slots/carried item |
| `nearby_blocks` | radius-filtered non-air blocks from a fixed 5×5×5 copied volume |
| `client_errors` | newest bounded redacted observation and connection failures |

Block collection checks loaded client chunks and build height, and reports whether the copied
volume was complete. Non-finite player, motion, health, or hit values fail the collection and enter
the error ring instead of becoming invalid JSON. Lists are defensively copied, and the error ring
keeps at most 64 messages with user-home and credential-like values redacted.

## Actual-client probe

A Fabric development client was launched twice on macOS using Java 25, an isolated test identity,
an ephemeral MCP port, and a temporary owner-only secret. An authenticated MCP session observed:

```json
{
  "state": "Ready",
  "clientTick": 806,
  "connectionState": "DISCONNECTED",
  "gameObservationAvailable": false
}
```

`screen_state` independently returned the real `AccessibilityOnboardingScreen`, its rendered title,
and `427×240` GUI dimensions. `client_errors` was empty. This proves tick-driven data publication
from a running Minecraft 26.2 render client; it does not claim a server gameplay scenario yet.

The first terminal interruption revealed that Fabric's normal stopping callback is not guaranteed
when Gradle terminates the child process. A JVM shutdown hook was added, and the second real launch
proved that interruption closed the listener and removed the ready file. The temporary secret and
probe responses were discarded after the run.

## Verification

Thirteen Java tests now cover the transport plus snapshot immutability, bounded error retention,
redaction, tool schemas, copied observation output, radius filtering, and rejected bounds. Required
commands passed on 2026-08-01:

```text
JAVA_HOME=<local-jdk-25> ./tools/ferrite-client-mcp/gradlew --no-daemon clean check
JAVA_HOME=<local-jdk-25> ./tools/ferrite-client-mcp/gradlew --no-daemon build
JAVA_HOME=<local-jdk-25> ./tools/ferrite-client-mcp/gradlew --no-daemon runClient
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
git diff --check
```

All handwritten Java files remain below 1,200 physical lines and compile with
`-Xlint:all -Werror`.
