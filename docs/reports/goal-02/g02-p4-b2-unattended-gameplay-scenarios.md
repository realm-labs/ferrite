# G02-P4-B2 — Unattended gameplay scenarios

## Outcome

The client project now produces a pure-Java acceptance runner in addition to the JDK-only launcher.
One command owns the locked reference server or current Ferrite process, starts the exact 26.2
Fabric client through Quick Play, initializes MCP, applies tick-fenced tools, records assertions,
and cleans every client/server process.

Each scenario receives a random ignored evidence directory containing numbered JSON-RPC responses,
screenshots, process logs, management snapshots, a terminal summary, and only secret-free endpoint
metadata. The bearer secret remains in the temporary launcher run and that run is deleted after log
collection. The generated server world stays inside the ignored evidence bundle and no Mojang
payload, secret, account token, or normal user game state is committed.

The client identity is the fixed offline name `FerriteMcp`. The reference server uses the locked
60,894,273-byte 26.2 server, fixed `FerriteMcp26.2` seed, flat survival world, peaceful difficulty,
disabled structures and mobs, and no command or RCON surface. Ferrite uses a generated schema-1
single-node configuration with the locked registry report and three isolated loopback ports.

## Reference-server gameplay

The `reference-gameplay` scenario passed every assertion:

1. Quick Play reached `PLAY`, `screenType=NONE`, and an available survival player.
2. Absolute view application completed through the normal client-thread action queue.
3. Ten forward-key ticks moved the player from Z `-1.5` to `0.5171820022762333`; the final normal
   movement velocity and on-ground state were observed after the action's tick fence.
4. Jump, hotbar slot 1, attack, and use-item actions each reached terminal `Satisfied` receipts.
5. The ordinary inventory key opened `InventoryScreen`, screen state was observed, and the ordinary
   close path returned to `screenType=NONE` while the connection remained Play.
6. Nearby blocks and client errors were recorded, and the actual framebuffer produced a 1708×960,
   964,994-byte PNG with SHA-256
   `0cae98a06acb605a5b19e311c2f1d8644170c9c991836e2bf850be6c3127cb69`.

No step used a server command, direct world mutation, direct player mutation, or a hand-built
Minecraft packet.

## Ferrite terrain and visual state

The `ferrite-visual` scenario explicitly rejects transient Play or a loading overlay. It passed:

1. The client first reached Play, then independently reached `screenType=NONE` with a live player.
2. The player was at `(8.5, 65.0, 8.5)` in `minecraft:overworld`; the radius-2 observation was
   complete and contained the expected solid `minecraft:stone` surface.
3. A second wait required more than 40 additional client ticks while Play, no screen, and player
   availability all remained true; the final player was on ground.
4. At the same boundary Ferrite reported ready and healthy, exactly one active session, 25 active
   Region authorities, zero pending commits, and no lifecycle failure.
5. Screenshot capture occurred only after the sustained-Play fence. The 1708×960, 789,644-byte
   terrain PNG has SHA-256
   `dd40f050c8a8b9f38c5ae45ac12aa0e35efe685e3e65e0a41bd7f3f59a1fbfd6`.

This is instrumented-client evidence. The separate unmodified-client compatibility evidence from
Goal 01 remains the authority for an unmodified-client claim.

## Reproduction and verification

```text
cargo build -p ferrite-server
cd tools/ferrite-client-mcp
JAVA_HOME=<local-jdk-25> ./gradlew --no-daemon clean check build
<local-jdk-25>/bin/java \
  -jar build/libs/ferrite-client-mcp-0.1.0-SNAPSHOT-acceptance.jar \
  --workspace <workspace> \
  --java-home <local-jdk-25> \
  --mode all
cd ../..
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
git diff --check
```

Focused Java tests cover acceptance argument closure, workspace-owned evidence roots, launcher
argument bounds, client/server artifact mismatches, deterministic client options, and isolated run
deletion. The exact run produced terminal `Satisfied` summaries for both scenarios, left no client
run directory or child process, and visually confirmed that the Ferrite image contains rendered
flat terrain rather than the Mojang or terrain-loading overlay.
