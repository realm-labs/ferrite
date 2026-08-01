# G02-P4-B1 — Isolated Quick Play launcher

## Outcome

The client MCP now includes a standalone pure-Java launcher for one unattended, instrumented
Minecraft 26.2 client.

- The launcher verifies the external locked client before launch: 39,193,383 bytes and SHA-1
  `2dc72797acbc1b63fc16a11c4ac393605f453754`.
- Every launch receives a random directory below `target/client-mcp-runs`, an owner-only random MCP
  secret, a secret-free atomic readiness file, a dedicated log, and first-run-safe options.
- Fabric Loom receives the isolated game directory through `ferriteGameDir`; the user's normal game
  directory, HMCL installation, saves, accounts, and tokens are never consulted.
- Quick Play targets an explicit loopback host and port. Readiness uses a file-system watch rather
  than a sleep loop, and stdout exposes only the run ID and loopback MCP endpoint.
- The launcher owns the Gradle/client process tree. Normal exit, startup failure, maximum-runtime
  expiry, and JVM shutdown all converge on bounded descendant termination and optional run deletion.
- A classifier JAR contains only the five JDK-only launcher classes, so starting the supervisor does
  not require Minecraft, Fabric, Gson, or another sidecar runtime on its class path.

## Exact-client evidence

The launcher started the locked client with no UI clicks and Quick Played into the locked original
26.2 reference server at `127.0.0.1:25566`.

1. The secret file was mode `0600` and 65 bytes (64 hexadecimal characters plus a newline). The
   readiness record advertised `http://127.0.0.1:61253/mcp` without the secret.
2. MCP `initialize` negotiated `2025-11-25`. A tick-bounded state wait immediately observed
   `connectionState=PLAY`, `screenType=NONE`, and `playerAvailable=true` at client tick 845.
3. `player_state` observed the real reference-world player at `(7.5, -60.0, 6.5)`, health 20,
   creative mode, and `minecraft:overworld`.
4. Framebuffer capture produced a 1708×960 PNG of the flat reference world, 620,816 bytes, with
   SHA-256 `ff41afe700f69991fd5c66a2cf47acc9fb3d5315a61a88aedd6bf099e050b5ef`.
5. At the configured maximum runtime the launcher printed the structured `TimedOut` result, exited
   with code 124, terminated the client process tree, and left no listener on MCP port 61253.

The retained diagnostic run used for this observation was removed after its metadata and hashes
were recorded. No Mojang artifact, game directory, MCP secret, or screenshot payload is committed.

## Verification

The batch's containing commit runs:

```text
JAVA_HOME=<local-jdk-25> ./gradlew --no-daemon clean check build
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
git diff --check
```

Focused tests cover loopback-only endpoints, workspace-owned run roots, timeout bounds, exact-client
artifact rejection, isolated options, secret length, and recursive run deletion. Manual acceptance
also inspects the launcher JAR manifest and verifies that only launcher-package classes are present.
