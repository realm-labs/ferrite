# G02-P3-B1 — Tick-fenced client control

## Outcome

The instrumented Java 26.2 client now owns a bounded client-thread action queue and exposes real
movement, look, jump, sneak, sprint, state-wait, action-status, and emergency-release MCP tools.
Held input uses `KeyMapping.setDown`; view rotation uses the local player's normal rotation API.
The implementation does not mutate position, teleport, issue server commands, construct packets,
or require a mixin.

## Control contract

- The pending queue is bounded at 64 actions and retained receipt history at 256 entries.
- Mutating tools require a 1–64 character action ID with a restricted portable alphabet.
- Receipt states are `Queued`, `Applied`, `Satisfied`, `TimedOut`, `Cancelled`, and `Rejected`.
  `Applied` only means execution on the Minecraft client thread.
- Forward/backward and left/right conflicts are rejected. An MCP-owned held key cannot be acquired
  concurrently by a second action.
- Movement, sneak, and sprint holds are limited to 1–200 client ticks; jump is limited to 1–20.
- A priority `release_all_inputs` request can enter even when the ordinary queue is full. It
  cancels queued and active work and releases all seven MCP-owned keys.
- Active keys release on normal expiry, explicit release, disconnect, world replacement, missing
  local player, MCP shutdown, and client shutdown.
- Gameplay actions require a player, world, and no open screen. This preserves normal focus and
  prevents a background GUI from being treated as successful gameplay input.
- `wait_for_state` waits on observation publication rather than polling sleeps. It supports a
  strict `afterClientTick` fence plus connection, screen, player-presence, and on-ground conditions;
  client-tick limits and a 30-second wall guard bound paused-client failure.

`action_status` is included in addition to the Goal minimum so callers can distinguish queue
acceptance, client-thread application, natural completion, rejection, and cancellation without
guessing from request latency.

## Exact-client evidence

The live test used the repository-locked official 26.2 server jar, SHA-1
`823e2250d24b3ddac457a60c92a6a941943fcd6a`, on loopback in offline creative mode and the actual
Fabric/Java 25 graphical client through Quick Play. No server command or direct state mutation was
used.

1. MCP waited for `PLAY`, `screenType=NONE`, and an available player at client tick 960.
2. Absolute look set yaw and pitch to zero through the client-thread action queue.
3. A 20-tick forward hold was accepted at tick 961, applied at 962, and satisfied at 982.
4. The observed player moved along the selected +Z view direction from
   `(2.5, -60.0, -1.5)` to `(2.5, -60.0, 2.7749259045575054)` by tick 984.
5. A three-tick jump was applied at tick 985 and released at 988; the observed Y coordinate rose
   from `-60.0` to `-59.2468000194788`.
6. An actual 1708×960 framebuffer screenshot after movement was 744,850 bytes and had SHA-256
   `54e5fceff0b671af706d20989bf074524ae090890e6fb6a33928d97808c287e2`. Visual inspection showed
   the rendered flat world, HUD, hand, entities, and movement tutorial.
7. A separate 200-tick forward action reached `Applied` at tick 1703. Stopping the reference
   server caused a world/connection transition; the action became `Cancelled` at tick 1788 and
   its key was released rather than remaining held through the disconnect screen.

At the disconnected screen, a queued look action was also proven to change from `Queued` to
`Rejected` on the next client tick, while the priority release action changed to `Satisfied` with
both applied and completed tick evidence.

## Ferrite observation

The same Quick Play client attempted the current `ferrite-server` process on `127.0.0.1:25565`.
The client started connecting but reached `DisconnectedScreen` and the server returned to zero
active sessions. This does not invalidate the independently proven client-control batch; it is an
open server-side acceptance finding for the later Ferrite scenario rather than something hidden by
the MCP implementation.

## Verification

The batch's containing commit runs:

```text
JAVA_HOME=<local-jdk-25> ./gradlew --no-daemon check build
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
git diff --check
```

Focused Java coverage verifies queue bounds, priority release, receipt transitions, identifier
validation, duplicate rejection, held-input ownership, close cancellation, finite camera values,
opposing-direction rejection, mandatory hold bounds, tick-fenced state waits, and tool discovery.
