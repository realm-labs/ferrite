# Goal 02 — Minecraft Java 26.2 Client MCP Automation

## 1. Objective

Build a repository-owned, pure-Java test tool that instruments the locked Minecraft Java Edition
26.2 client through Fabric and exposes safe Model Context Protocol tools for real client input,
state observation, screenshots, deterministic waits, and unattended acceptance. This tool exists to
test Ferrite; it is not the future Ferrite-native client and it never becomes a gameplay oracle.

The finished path is:

```text
Codex or CI
  -> Streamable HTTP MCP on loopback
  -> Java 25 Fabric client mod
  -> Minecraft client main-thread input and interaction APIs
  -> ordinary Minecraft 26.2 packets
  -> tested server
  -> ordinary client decode, prediction, GUI, and rendering
  -> structured observations and screenshots
```

## 2. Scope boundary

### In scope

- one standalone Gradle project below `tools/ferrite-client-mcp`;
- Java 25, Minecraft 26.2, Fabric Loader, Fabric API, and Mojang mappings;
- loopback-only Streamable HTTP MCP with bounded request bodies and a per-run bearer secret;
- connection, screen, player, inventory, crosshair, nearby-world, and client-error observations;
- framebuffer screenshots returned as MCP image content;
- tick-fenced movement, look, jump, sneak, sprint, attack, use, hotbar, drop, hand-swap, chat,
  inventory-open/close, cursor, and slot-click operations;
- condition waits using client ticks and structured state instead of wall-clock sleeps;
- guaranteed key release on timeout, disconnect, world change, MCP shutdown, and client shutdown;
- a pure-Java launcher for isolated game directories, Quick Play multiplayer, process cleanup, and
  evidence collection;
- deterministic scenarios against a locked local reference server and Ferrite where the server
  feature exists;
- Linux virtual-display instructions and local macOS execution.

### Out of scope

- server administration through commands, RCON, or direct state mutation;
- Mineflayer or another independent protocol bot as the client-under-test;
- pathfinding, crafting planners, combat AI, natural-language planning, or autonomous survival;
- replacing Minecraft networking, physics, prediction, GUI logic, or rendering;
- reading HMCL account tokens or modifying the user's normal game directory;
- distributing Mojang jars, assets, libraries, mappings payloads, or generated game data;
- claiming that the instrumented client is unmodified.

An unmodified Quick Play smoke remains a separate compatibility gate. Instrumented-client evidence
proves interactive behavior and visual state; it cannot replace the unmodified-client claim.

## 3. Reuse and provenance

Two upstream projects define the reviewed starting boundary:

| Project | Locked revision | License | Reused boundary |
|---|---|---|---|
| `cuspymd/mcp-server-mod` | `43dcec547ad3a5ca6b6e0e2e1b37f5c2a6581cfe` | CC0-1.0 | Minecraft 26.2/Fabric/Java 25 compatibility, HTTP MCP and screenshot feasibility |
| `lucasoyen/MCCTP` | `50ffa27b04a934d105c2ae9b79f10fc50651f20d` | MIT | Main-thread input action vocabulary, tick state publication, and key-release hazards |

Ferrite owns its implementation and schemas. If any substantial MIT-licensed code is copied rather
than independently implemented, its copyright and license notice must accompany the affected
source. The repository does not vendor either upstream repository or its binaries.

The initial toolchain lock is Minecraft `26.2`, Java `25`, Fabric Loader `0.19.3`, Fabric API
`0.154.1+26.2`, and a reviewed Fabric Loom version. Snapshot build plugins are not accepted in the
completion record; the first build batch must resolve and lock a reproducible release.

## 4. Responsibility model

The project is split by responsibility, not by implementation phase:

```text
transport       bounded HTTP, authentication, MCP framing, lifecycle
tools           stable tool schemas and dispatch
control         main-thread action queue, tick fences, held-input ownership
observation     immutable client snapshots, crosshair, screen, inventory, errors
capture         render-thread screenshot requests and bounded image responses
launcher        isolated client/reference-server processes and Quick Play
acceptance      deterministic scenarios, assertions, artifacts, cleanup
```

No handwritten Java file may exceed 1,200 physical lines. Tool descriptions are assembled from
responsibility-local definitions rather than one monolithic protocol file.

## 5. Tool contract

Every mutating call carries a unique action ID and returns one of `Queued`, `Applied`, `Satisfied`,
`TimedOut`, `Cancelled`, or `Rejected`. `Applied` means the action ran on the client thread; it does
not assert that the server accepted the resulting gameplay operation. Scenario assertions must
observe a later client or server state transition.

The minimum MCP surface is:

| Area | Tools |
|---|---|
| Lifecycle | `client_status`, `wait_for_state`, `release_all_inputs` |
| Observation | `player_state`, `inventory_state`, `crosshair_state`, `screen_state`, `nearby_blocks`, `client_errors` |
| Visual | `take_screenshot` |
| Camera/input | `look`, `hold_movement`, `jump`, `set_sneaking`, `set_sprinting` |
| Interaction | `attack`, `use_item`, `select_hotbar`, `drop_item`, `swap_hands` |
| GUI/chat | `open_inventory`, `close_screen`, `click_slot`, `move_cursor`, `send_chat` |

Movement duration and waits are expressed in bounded client ticks. Tools reject missing world/player
state, unsupported screens, invalid slots, non-finite coordinates, excessive durations, concurrent
ownership of the same held input, and requests after shutdown.

## 6. Security and isolation

- Bind only an explicit loopback address; wildcard binding is invalid.
- Require a cryptographically random bearer secret supplied through an ignored per-run file or
  environment variable. Never print it or include it in evidence.
- Reject non-loopback `Origin`, unexpected content types, oversized bodies, batch requests, and
  unsupported MCP protocol versions.
- Use a bounded executor and bounded pending-action queue; never create one thread per request.
- Redact access tokens, client IDs, XUIDs, secrets, and absolute user-profile paths from logs.
- Use an isolated ignored game directory and deterministic offline test identity. Never read HMCL
  account databases or reuse the user's normal saves, options, screenshots, or logs.
- Disable command execution and direct player/world mutation. Tools must travel through ordinary
  client input and interaction APIs.

## 7. Phased batches

### Phase 0 — Freeze the executable contract

| Batch | Outcome |
|---|---|
| `G02-P0-B1` | Record scope, provenance, architecture, security, batches, terminal gates, prompt, and resumable ledger. |

### Phase 1 — Reproducible client mod and MCP foundation

| Batch | Outcome |
|---|---|
| `G02-P1-B1` | Add the standalone Java 25/Fabric 26.2 Gradle project, dependency locks, license inventory, and deterministic build command. |
| `G02-P1-B2` | Implement bounded authenticated loopback HTTP, MCP initialize/ping/tools lifecycle, configuration, and shutdown. |

### Phase 2 — Observation and visual evidence

| Batch | Outcome |
|---|---|
| `G02-P2-B1` | Publish immutable connection/player/inventory/crosshair/screen/nearby-block/error snapshots from the client thread. |
| `G02-P2-B2` | Add render-thread screenshot capture, size bounds, image responses, and visual artifact metadata. |

### Phase 3 — Real client control

| Batch | Outcome |
|---|---|
| `G02-P3-B1` | Add the bounded tick action queue, action receipts, waits, movement/look/jump/sneak/sprint, and unconditional input release. |
| `G02-P3-B2` | Add attack/use/hotbar/drop/swap/chat through normal client interaction paths and prove packet-producing behavior. |
| `G02-P3-B3` | Add inventory screen/cursor/slot operations with screen-handler revision and slot validation. |

### Phase 4 — Unattended launch and scenarios

| Batch | Outcome |
|---|---|
| `G02-P4-B1` | Add a pure-Java launcher for locked artifacts, Fabric profile, isolated directories, Quick Play, readiness, timeouts, and cleanup. |
| `G02-P4-B2` | Run reference-server movement/interaction/GUI scenarios and Ferrite connection/terrain/visual scenarios with evidence bundles. |

### Phase 5 — Hardening and completion

| Batch | Outcome |
|---|---|
| `G02-P5-B1` | Fault authentication, malformed MCP, overload, disconnect, render absence, stuck input, process crash, and artifact mismatch. |
| `G02-P5-B2` | Add CI profiles, operator documentation, clean-checkout acceptance, license/source audit, and completion record. |

## 8. Required verification

Each Java batch runs its focused tests plus:

```text
./tools/ferrite-client-mcp/gradlew --no-daemon check
./tools/ferrite-client-mcp/gradlew --no-daemon build
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
git diff --check
```

Gradle caches and generated run directories remain outside tracked source or under ignored `target`.
Java code changes do not waive Rust repository gates because the tool exists to validate the same
workspace.

## 9. Terminal acceptance

- [x] A clean checkout builds the exact 26.2 Fabric mod with locked dependencies and Java 25. See [completion record](../reports/goal-02/g02-p5-b2-completion-record.md).
- [x] MCP initialize, ping, list, call, errors, authentication, bounds, and shutdown pass. See [transport](../reports/goal-02/g02-p1-b2-mcp-transport.md) and [fault hardening](../reports/goal-02/g02-p5-b1-fault-hardening.md).
- [x] All minimum tools have stable schemas, bounds, and main-thread/tick semantics. See [client control](../reports/goal-02/g02-p3-b1-tick-fenced-client-control.md) and [interactions](../reports/goal-02/g02-p3-b2-client-interactions.md).
- [x] Screenshots come from the actual framebuffer and include dimensions and client-tick metadata. See [framebuffer capture](../reports/goal-02/g02-p2-b2-framebuffer-screenshot.md).
- [x] Every held input is released across success, timeout, disconnect, world switch, and shutdown. See [client control](../reports/goal-02/g02-p3-b1-tick-fenced-client-control.md) and [fault hardening](../reports/goal-02/g02-p5-b1-fault-hardening.md).
- [x] The launcher never reads HMCL credentials and uses only isolated ignored state. See [launcher evidence](../reports/goal-02/g02-p4-b1-isolated-quick-play-launcher.md).
- [x] Quick Play starts the locked exact client and reaches the configured endpoint without clicks. See [launcher evidence](../reports/goal-02/g02-p4-b1-isolated-quick-play-launcher.md).
- [x] A reference-server scenario performs real movement, jump, interaction, hotbar, and GUI input. See [scenario evidence](../reports/goal-02/g02-p4-b2-unattended-gameplay-scenarios.md).
- [x] A Ferrite scenario connects, renders terrain, captures a screenshot, and records client/server state. See [scenario evidence](../reports/goal-02/g02-p4-b2-unattended-gameplay-scenarios.md).
- [x] The instrumented and unmodified-client evidence classes are labeled separately. See [launcher](../reports/goal-02/g02-p4-b1-isolated-quick-play-launcher.md) and [scenario](../reports/goal-02/g02-p4-b2-unattended-gameplay-scenarios.md) evidence.
- [x] Malformed, overload, auth, process, render, and artifact faults fail closed and clean up. See [fault hardening](../reports/goal-02/g02-p5-b1-fault-hardening.md).
- [x] No Mojang artifact, generated game payload, access token, secret, or user game state is committed. See [completion audit](../reports/goal-02/g02-p5-b2-completion-record.md).
- [x] Java/Rust format, lint, test, source-size, dependency, license, and clean-worktree gates pass. See [completion record](../reports/goal-02/g02-p5-b2-completion-record.md).

Goal 02 is complete only when every checkbox links to committed reproducible evidence. A mod that
starts, an MCP tools list, a screenshot, or one successful connection is not completion.
