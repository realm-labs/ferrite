# G01-P9-S001 — command feedback routing

## Result

Complete. `CLI-COMMAND-FEEDBACK-001` is implemented as an executable Java 26.2 behavior model in
`behavior-runner` and verified by its assigned test owner.

The model preserves structured literal, translatable, appended, and styled components. Successful
feedback evaluates its supplier exactly once only when at least one route is open, sends direct
output before administrative copies, retains player-list order, excludes the source player by
source identity, and applies the gray italic `chat.type.admin` wrapper. Failure is direct-only and
uses the red appended-component form.

## Routing coverage

The conformance suite locks these independent gates:

- `send_command_feedback` controls player success, command-block success, and OP fan-out;
- `command_block_output` controls whether a live command-block source reaches the administrative
  fan-out and therefore whether `log_admin_commands` can log that copy;
- `log_admin_commands` controls the server-log copy only after the source's inform-admin gate;
- dedicated `broadcast-console-to-ops` and `broadcast-rcon-to-ops` properties independently control
  console and RCON administrative fan-out;
- silence suppresses direct success/failure and all administrative copies without evaluating a
  success supplier;
- server-console direct output is logged but does not log its own administrative copy, while RCON
  direct output concatenates into its response buffer.

The exhaustive success table covers 288 combinations: player, console with both property values,
RCON with both property values, tracked/open, untracked and closed command-block sources, and the
null source, crossed with all eight gamerule values, silence, and broadcast intent.

## Command-block and gamemode boundaries

Tracked command-block output receives an injected deterministic `HH:mm:ss` prefix, invokes one
update hook per accepted message, accepts failures independently of `send_command_feedback`, and
rejects all late output after source close. A plain placement snapshots `send_command_feedback`
into `track_output` and applies the block subtype's automatic flag. Placement carrying block-entity
data preserves both loaded fields; both forms still refresh powered state.

Gamemode changes distinguish self feedback from the source/other-target split. An actual other
target receives `gameMode.changed` only while `send_command_feedback` is enabled, including the
source-silent exception in the locked control flow. A no-op mode request emits nothing.

## Evidence

Implementation owner:

- `behavior_runner::client::feedback`.

Committed test owner:

- `apps/behavior-runner/tests/client/blk_003.rs`.

Locked official source checks:

- `CommandSourceStack#sendSuccess`, `#sendFailure`, and `#broadcastToAdmins`;
- `ServerPlayer$3`, `MinecraftServer`, `DedicatedServer`, and `RconConsoleSource` command-source
  traits;
- `BaseCommandBlock$CloseableCommandBlockSource` and `CommandBlock#setPlacedBy`;
- `GameModeCommand#logGamemodeChange`.

Focused validation:

```text
cargo test -p behavior-runner --test client --all-features
8 passed; 0 failed; 288 routing combinations checked
cargo fmt --all -- --check
git diff --check
```

Phase 9 integration, client-projection packets, command-administration surfaces, and cross-system
joins remain owned by `G01-P9-B1`.
