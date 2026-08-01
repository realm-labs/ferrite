# G02-P2-B2 — Render-thread framebuffer screenshot

## Outcome

`take_screenshot` now schedules a single-flight request onto Minecraft's render thread, reads the
main render target through the ordinary 26.2 screenshot path, encodes a temporary PNG, deletes the
temporary file, and returns both MCP image content and structured evidence.

The boundary enforces:

- at most one pending capture;
- a 10-second MCP wait timeout with cancellation;
- render-thread execution and a live positive framebuffer geometry;
- at most 16,777,216 pixels and 16 MiB of encoded PNG;
- PNG signature, defensive byte ownership, lowercase SHA-256 shape, and digest/content equality;
- cancellation of pending work on client shutdown;
- no caller-selected path, camera mutation, player teleport, or permanent screenshot write.

Success returns `state: Satisfied`, MIME type, width, height, copied client tick, byte length, and
SHA-256. The MCP content array contains a text block followed by a standard base64 `image/png`
block. Busy, absent-render, timeout, interruption, encoding, and shutdown failures remain tool
results rather than crashing the protocol connection.

## Actual-client visual proof

A real Minecraft 26.2 Fabric client was started with the temporary isolated MCP configuration.
`take_screenshot` returned:

```json
{
  "state": "Satisfied",
  "mimeType": "image/png",
  "width": 1708,
  "height": 960,
  "clientTick": 801,
  "byteLength": 461388,
  "sha256": "20ab6e743fad6e3971097e8c5f75fcf329ffa3828e41e2c2f38b6890a175abf2"
}
```

The image block decoded to a non-interlaced 8-bit RGBA PNG. Its independently computed digest
matched the metadata exactly. Visual inspection showed an upright, non-black Minecraft Java
Edition accessibility welcome screen at full framebuffer resolution, matching the separately
observed `AccessibilityOnboardingScreen`. The image, response, session ID, and temporary secret
were discarded after verification and are not tracked artifacts.

## Verification

Seventeen Java tests now include defensive PNG ownership and validation, MCP image framing,
failed/busy capture reporting, timeout cancellation, and forbidden path arguments. These commands
passed on 2026-08-01:

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
