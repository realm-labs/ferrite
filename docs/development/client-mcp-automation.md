# Client MCP automation operations

The Ferrite client MCP is a test-only, pure-Java Fabric mod for unattended Minecraft Java 26.2
acceptance. It controls normal client input and reads normal client state; it is not part of
`ferrite-server`, an alternative game client, or an unmodified-client compatibility proof.

## Profiles

| Profile | Runs where | Purpose | Command |
|---|---|---|---|
| Java CI | GitHub Actions or any clean source tree | Compile the mod, run unit/fault tests, build the launcher and acceptance JARs, and verify distribution contents | `./gradlew --no-daemon clean check build` |
| Reference gameplay | Graphical operator host | Prove movement, jump, interaction, hotbar, and inventory behavior against the locked reference server | acceptance JAR with `--mode reference` |
| Ferrite visual | Graphical operator host | Prove sustained Play, terrain rendering, screenshot capture, and one active Ferrite session | acceptance JAR with `--mode ferrite` |
| Full acceptance | Graphical operator host | Run both gameplay profiles with one lifecycle and separate evidence bundles | acceptance JAR with `--mode all` |

The CI profile intentionally does not download or publish the locked client/server JARs as workflow
artifacts. Full gameplay acceptance requires a graphical user session and the operator's local,
license-compliant Mojang artifacts.

## Prerequisites

- Java 25, with `JAVA_HOME` pointing at that installation;
- the exact client at `target/mc-reference/26.2/client.jar` with SHA-1
  `2dc72797acbc1b63fc16a11c4ac393605f453754`;
- the exact server at `target/mc-reference/26.2/server.jar` with SHA-1
  `823e2250d24b3ddac457a60c92a6a941943fcd6a`;
- the generated 26.2 registry report at
  `target/mc-reference/26.2/generated/reports/registries.json`;
- a built `target/debug/ferrite-server` for the Ferrite profile; and
- an unlocked graphical desktop. The isolated options disable pause-on-focus-loss, but the client
  still needs a real renderer.

Do not copy an HMCL profile, `.minecraft` directory, account database, access token, or save into
the repository. The launcher creates a separate ignored game directory and deterministic offline
identity named `FerriteMcp`.

## Build and run

From `tools/ferrite-client-mcp`:

```text
JAVA_HOME=/path/to/jdk-25 ./gradlew --no-daemon clean check build
```

For both unattended scenarios, first build Ferrite from the repository root, then run:

```text
cargo build -p ferrite-server
/path/to/jdk-25/bin/java \
  -jar tools/ferrite-client-mcp/build/libs/ferrite-client-mcp-0.1.0-SNAPSHOT-acceptance.jar \
  --workspace /absolute/path/to/ferrite \
  --java-home /path/to/jdk-25 \
  --mode all
```

Each scenario prints its evidence directory. A satisfied run has `summary.json` state `Satisfied`.
The reference bundle must contain positional delta plus terminal action receipts. The Ferrite bundle
must contain sustained Play state, `active_sessions=1`, a terrain screenshot, and a server status
snapshot. Screenshots are evidence only when they visibly contain the rendered world rather than a
loading overlay.

## Lifecycle and cleanup

The launcher owns the Gradle/client process tree, random loopback MCP port, bearer secret, and run
directory below `target/client-mcp-runs`. The acceptance runner additionally owns the reference or
Ferrite server and writes secret-free copies under `target/client-mcp-evidence`.

Normal exit, timeout, interruption, malformed readiness, and startup failure terminate owned child
processes. The launcher deletes its run by default. `--retain-run` is a diagnostic-only launcher
option; after diagnosis, remove only the printed `target/client-mcp-runs/run-<uuid>` directory.
Never use a broad recursive cleanup command against the repository, user home, HMCL, or normal
Minecraft directory.

## Failure triage

1. Read the scenario `summary.json`; it records the first bounded assertion failure.
2. Inspect `launcher-output.log`, `client-process.log`, and `minecraft-latest.log` in the same bundle.
3. For Ferrite, compare `ferrite-server-process.log` with the captured management status. A healthy
   renderer is insufficient if `active_sessions` never becomes one.
4. Confirm the JDK is Java 25 and the three locked reference inputs are present. Size or digest
   mismatch is terminal and must not be bypassed.
5. If a screenshot shows a loading overlay, rerun only after diagnosing the Play/session failure;
   do not relabel it as terrain evidence.
6. Confirm no process from the printed run remains before retrying. Retain a run only when its local
   secret and game directory are required for immediate diagnosis.

MCP requests fail closed for wrong authentication, hostile origins, oversized bodies, malformed
JSON-RPC, concurrent control sessions, full action queues, missing renderers, and tool exceptions.
A generic tool error intentionally omits exception details; use the redacted client error stream and
isolated logs for diagnosis.

## Security and publication boundary

Only loopback endpoints are accepted. The MCP bearer secret belongs to one isolated run, is stored
in an owner-only file, is never printed, and is erased with the run. Evidence must not contain that
secret or any Minecraft access token. Before committing, use `git status --ignored` and verify that
`target`, `tools/ferrite-client-mcp/build`, `.gradle`, and `run` remain ignored.

The mod JAR does not redistribute dependencies. The standalone acceptance JAR embeds only its
launcher/acceptance classes and unmodified Gson, plus the third-party notice and Apache-2.0 license.
The Gradle `verifyDistribution` task enforces the absence of Minecraft and Fabric classes from that
JAR. Mojang artifacts, generated worlds, screenshots, logs, and evidence bundles are never source
or release artifacts.
