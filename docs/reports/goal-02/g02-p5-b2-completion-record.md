# G02-P5-B2 — Client MCP automation completion record

## Result

`Satisfied`. Goal 02 delivers a pure-Java Minecraft Java 26.2 Fabric client MCP, normal-input
control and framebuffer observation, an isolated Quick Play supervisor, deterministic reference
and Ferrite scenarios, bounded fault handling, CI coverage, and an operator workflow. The client
automation remains test infrastructure outside the Ferrite server runtime.

## Clean-source proof

The staged implementation tree `a00f26c76078e6f90d9162b228610c8cae882707` was attached to a
temporary detached worktree at ephemeral commit
`0be310c34bff415ef9a9442871d0dfdba0dc21ba`. Before running any command, that worktree contained
only tracked source plus an ignored symlink to the operator-owned locked reference inputs.
`git status --short` was empty before and after the run. The temporary worktree and its generated
worlds, logs, screenshots, run secrets, and evidence were removed after inspection.

The following passed from that clean worktree:

```text
JAVA_HOME=/Users/mikai/Library/Java/JavaVirtualMachines/azul-25/Contents/Home ./gradlew --no-daemon clean check build
CARGO_TARGET_DIR=/Users/mikai/CLionProjects/ferrite/target/goal02-clean-cargo cargo fmt --all -- --check
CARGO_TARGET_DIR=/Users/mikai/CLionProjects/ferrite/target/goal02-clean-cargo cargo clippy --workspace --all-targets --all-features -- -D warnings
CARGO_TARGET_DIR=/Users/mikai/CLionProjects/ferrite/target/goal02-clean-cargo cargo test --workspace --all-features
CARGO_TARGET_DIR=/Users/mikai/CLionProjects/ferrite/target/goal02-clean-cargo cargo build -p ferrite-server
java -jar tools/ferrite-client-mcp/build/libs/ferrite-client-mcp-0.1.0-SNAPSHOT-acceptance.jar --workspace <clean-worktree> --java-home <jdk-25> --ferrite-bin <isolated-ferrite-binary> --mode all
```

The Gradle run included `verifyDistribution`. The clean all-mode acceptance produced two
`Satisfied` summaries:

| Scenario | Deterministic result | Screenshot |
|---|---|---|
| Reference gameplay | `FerriteMcp` moved from `z=-9.5` to `z=-7.482817997723767`; jump, hotbar, attack, use, inventory open/close, nearby blocks, and client errors completed | 969,489 bytes; SHA-256 `f9900d87f7d2909277d9b83ca67015f4beed2647f1b02a00dd3659529c0e788f` |
| Ferrite visual | Play remained satisfied from tick 77 through tick 118; status recorded `active_sessions=1`, 25 active Region authorities, zero pending commits, and no failure | 791,479 bytes; SHA-256 `df8731f9abf4d0b454b32368a38b8fc8ae947913d4485c3370a18430082370ba` |

The Ferrite screenshot was visually inspected after the sustained wait. It contains the actual
flat stone terrain, sky, clouds, crosshair, HUD, hotbar, and player hand rather than a Mojang or
terrain-loading overlay. No isolated client run remained after either scenario.

## CI and operations

The repository CI now has a `client-mcp-java` job using Java 25. It runs the same clean Gradle
build, tests the fault matrix, builds all three Java distributions, and enforces the embedded
artifact boundary. Full graphical gameplay remains an operator profile because CI must not publish
the locked Mojang artifacts and a real render session is required.

The [operator guide](../../development/client-mcp-automation.md) records prerequisites, build and
scenario profiles, lifecycle ownership, evidence interpretation, failure triage, security limits,
and safe cleanup. It explicitly excludes HMCL state, normal Minecraft directories, accounts,
tokens, server commands, direct world mutation, and hand-built gameplay packets.

## Source, dependency, and license audit

- `git ls-files target` returned zero paths. No tracked path below the client MCP contains `build`,
  `run`, `.gradle`, screenshots, logs, or secret files.
- The committed Gradle lockfile and SHA-256 verification metadata were exercised by the clean
  build. Versions remain Minecraft 26.2, Fabric Loader 0.19.3, Fabric API 0.154.1+26.2, Gson 2.14.0,
  and JUnit 6.0.3.
- The remapped mod JAR does not redistribute dependencies. The acceptance JAR contains no
  `net.minecraft` or `net.fabricmc` classes; it embeds only project launcher/acceptance classes and
  Gson, with `META-INF/THIRD_PARTY_NOTICES.md` and `META-INF/LICENSE-GSON.txt`.
- The only external Mojang inputs are ignored operator-owned files verified by exact size and
  SHA-1 before launch. They were not copied into either Java distribution.
- A production-source search found no bearer, access-token, client-token, or refresh-token value.
  Synthetic test secrets remain confined to test source.
- All handwritten Java files are below 1,200 physical lines; the largest is 356 lines. The change
  adds no `super::super`, visibility widening, lint suppression, or Rust source modification.

## Terminal evidence index

| Requirement | Evidence |
|---|---|
| Reproducible Java 25 build and locked inputs | [G02-P1-B1](g02-p1-b1-client-mod-build.md) and this clean-source proof |
| Authenticated MCP lifecycle and bounded faults | [G02-P1-B2](g02-p1-b2-mcp-transport.md) and [G02-P5-B1](g02-p5-b1-fault-hardening.md) |
| Client observations and framebuffer image content | [G02-P2-B1](g02-p2-b1-client-observations.md) and [G02-P2-B2](g02-p2-b2-framebuffer-screenshot.md) |
| Tick-fenced normal input, interaction, and GUI | [G02-P3-B1](g02-p3-b1-tick-fenced-client-control.md), [G02-P3-B2](g02-p3-b2-client-interactions.md), and [G02-P3-B3](g02-p3-b3-inventory-screen-control.md) |
| Isolated exact-client launch and cleanup | [G02-P4-B1](g02-p4-b1-isolated-quick-play-launcher.md) |
| Reference and Ferrite gameplay | [G02-P4-B2](g02-p4-b2-unattended-gameplay-scenarios.md) and this clean all-mode rerun |
| CI, operator, source, dependency, and license gates | This report and the [operator guide](../../development/client-mcp-automation.md) |

All Goal 02 batches are terminally complete. Future gameplay coverage belongs to Ferrite server
goals and can use this client MCP as the reusable real-client automation boundary.
