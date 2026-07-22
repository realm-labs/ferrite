# Client-observable behavior mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `CLI-COMMAND-FEEDBACK-001` — Command success splits into direct, operator, and server-log channels

**Parent:** `BLK-003`, `PLY-001`, `CLI-006`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — lazy success construction, silent/source gates, all three live Boolean rules,
operator fan-out, server logging, player/command-block/RCON/console sources, gamemode's explicit
target message and command-block placement/output joins are explicit in locked source.

**Applies when:**

A command calls `CommandSourceStack#sendSuccess` or `#sendFailure`, a gamemode change reports to its
source/target, a tracked command block receives output, or a freshly placed command block chooses
its tracking default. Individual commands still own whether they request administrator informing
and the component/result they supply.

**Authoritative state:**

The live `send_command_feedback`, `command_block_output` and `log_admin_commands` rules; source
identity and its accepts-success/failure/inform-admins traits; stack `silent`; the caller's
inform-admins Boolean; ordered online players, operator state, dedicated broadcast properties,
server/integrated/RCON identity, command-block tracking/open state and its last output.

**Transition and ordering:**

All three `CHAT` rules default true and have no change callback. Every relevant use reads the live
level rules at its branch; ordinary gamerule persistence owns restart continuity.

**Generic success:** `sendSuccess(componentSupplier, informAdmins)` computes two gates before
evaluating the supplier. Direct output requires source `acceptsSuccess` and a nonsilent stack.
Administrator output requires caller `informAdmins`, source `shouldInformAdmins` and nonsilent. If
neither gate is open, the method returns without evaluating the supplier. Otherwise the component
is evaluated exactly once, sent unchanged to the source when direct is true, then passed to
administrator broadcast when that gate is true. One route does not suppress the other.

Administrator broadcast first constructs gray italic `chat.type.admin(sourceDisplayName,
component)`. If live `send_command_feedback` is true, it traverses the current player list and sends
that system component to every operator except a player whose `commandSource()` is the exact source
object. It then independently sends the same component to `MinecraftServer#sendSystemMessage` when
the source object is not the server and live `log_admin_commands` is true. Thus feedback false does
not suppress server logging; log false does not suppress OPs. Server identity avoids duplicate
console logging. Both rules are downstream of the source/caller/silent administrator gate.

**Failures:** `sendFailure` never takes the administrator path. If the source accepts failure and
the stack is nonsilent, it sends a red-styled copy directly. Neither `send_command_feedback` nor
`log_admin_commands` is read by this method. The source's own failure handler may still store or
project that component.

**Player and gamemode sources:** A server player's command source accepts success exactly when live
`send_command_feedback` is true, always accepts failure and always elects administrator informing.
A normal successful player command that requests informing can therefore lose its direct and OP
messages when feedback is false while still reaching server logging under
`log_admin_commands=true`. Its failures remain visible unless the stack is silent.

After a successful gamemode mutation, a source whose entity is the target reports the self message
through ordinary `sendSuccess(..., true)`. For a different target, live `send_command_feedback`
first directly controls the target-only `gameMode.changed` message, independent of the source
stack's silent flag; the source then reports the other-target success through ordinary routing.
Failed/no-change target mutations report nothing.

**Command blocks:** A tracked, still-open command-block source accepts successful output exactly
when live `send_command_feedback` is true, always accepts failure while open, and elects the
administrator path exactly when live `command_block_output` is true. Each accepted direct component
replaces last output with a wall-clock `HH:mm:ss` prefix and invokes the carrier update hook.

Consequently `command_block_output=false` suppresses the whole administrator route, including both
OP fan-out and `log_admin_commands` logging, but does not suppress direct stored output.
`send_command_feedback=false` suppresses successful stored output and OP fan-out; with
`command_block_output=true`, a successful command can still be server-logged when
`log_admin_commands=true`. Failures can still become stored output but are never broadcast. Closing
the source after dispatch rejects later asynchronous output. Output tracking false uses the null
source, whose three traits are false and whose message handler is a no-op.

On server placement without `BLOCK_ENTITY_DATA`, live `send_command_feedback` becomes the command
block's initial tracking flag. Component-bearing placement retains its applied value. Later rule
changes do not rewrite that flag, although the live rule still controls accepted successes.
`BLK-COMMAND-001` owns the carrier execution, persistence and publication around these routes.

**Console and RCON:** The server source accepts success/failure; a dedicated console elects
administrator output from `broadcast-console-to-ops` (default true), while an integrated server
always does so. Its direct component is logged, OP broadcast remains feedback-gated, and source
identity prevents the later log copy. RCON accepts direct success/failure into its response buffer
and elects administrator output from `broadcast-rcon-to-ops` (default true). RCON OP fan-out and
server log then follow the two live rules independently. The null source accepts nothing.

**Branches and aborts:**

Success/failure; lazy supplier; source traits; silent; caller inform flag; live three-rule matrix;
server/source identity; player-list order, operator and exact-source exclusion; player self/other
gamemode result; command-block tracking/open state; plain/component placement; console integrated/
dedicated property and RCON property.

**Constants and randomness:**

All three rules and both dedicated broadcast properties default true. Admin presentation is gray
and italic with `chat.type.admin`; failure is red; command-block direct output uses `HH:mm:ss`.
There is no RNG. Wall-clock output text is intentionally not game-time deterministic.

**Side effects:**

Lazy component evaluation, player/target/OP system messages, console/logger output, RCON response
buffer, command-block last output/update hook, and placement tracking state. Feedback routing does
not change command success counts or roll back command effects.

**Gates:**

Command-selected reporting call and inform flag; source traits/identity; stack silence; the three
live rules; operator membership; console/RCON properties; command-block tracking/open state and
placement data component.

**Boundary cases and quirks:**

Feedback false is not “no command output”: failures, RCON/console direct results and independent
server logging can remain. Log-admin false does not hide OP broadcasts. Command-block-output false
does not hide the block's direct timestamped output. The success supplier is never evaluated when
both routes are closed. Gamemode's different-target message bypasses source silence but not live
feedback. Exact source-object exclusion prevents a player from receiving the admin copy twice.

**Evidence:**

`OFF-SERVER-001`; `OFF-REPORT-001`; `net.minecraft.world.level.gamerules.GameRules`;
`net.minecraft.commands.CommandSourceStack#sendSuccess`, `#sendFailure`, `#broadcastToAdmins`;
`net.minecraft.server.level.ServerPlayer$3`;
`net.minecraft.server.commands.GameModeCommand#logGamemodeChange`;
`net.minecraft.world.level.BaseCommandBlock$CloseableCommandBlockSource`;
`net.minecraft.world.level.block.CommandBlock#setPlacedBy`;
`net.minecraft.server.MinecraftServer`; `net.minecraft.server.dedicated.DedicatedServer`;
`net.minecraft.client.server.IntegratedServer`; `net.minecraft.server.rcon.RconConsoleSource`;
`BLK-COMMAND-001`; `EXP-CLI-004`.

**Test vectors:**

Cross success/failure, supplier counter, inform flag and silent with player/server/RCON/null and
tracked/untracked/open/closed block sources. Sweep all eight live-rule combinations plus both
dedicated properties; assert direct raw versus formatted admin component, target/source/OP/server
audiences, exact player order/exclusion, logger/RCON/block state and supplier count. Repeat self/
other/no-change gamemode and plain/component-bearing command-block placement, then toggle each rule
without replacing saved tracking state.
