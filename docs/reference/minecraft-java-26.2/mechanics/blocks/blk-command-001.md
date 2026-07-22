# Blocks mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `BLK-COMMAND-001` — Command blocks retain trigger and chain state behind a live dispatch gate

**Parent:** `SIM-003`, `BLK-003`, `BLK-007`, `PLY-005`, `RED-001`, `RED-003`, `CLI-006`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — the three block modes, block and minecart trigger paths, live
`command_blocks_work` reads, same-tick guard, chain traversal, persistence, operator edits and
projection hooks are explicit in locked server source. Individual dispatched commands, command
fork/sequence limits and feedback routing retain their separate owners.

**Applies when:**

An impulse, repeating or chain command block receives redstone, scheduled work or an operator edit;
an activator rail triggers a command-block minecart; or either carrier saves, loads or projects its
command state. This leaf owns the carrier transaction and the `command_blocks_work` gate, not the
semantics of the command string after dispatcher admission.

**Authoritative state:**

The server's live Boolean `command_blocks_work` rule; each carrier's command, success count, custom
name, output tracking, last output, last-execution policy/time; a block entity's powered,
condition-met and automatic flags plus block mode/facing/conditional state; a minecart's entity age,
activation throttle and synchronized command/output fields; scheduled block ticks, neighboring
command blocks, operator permission, command results and ordinary block/entity publication.

**Transition and ordering:**

**Defaults, mode and persistence:** The `MISC` rule defaults true. `BaseCommandBlock` starts with an
empty command, success zero, null output/name, output tracking and last-execution updates enabled,
and last execution `-1`. A block entity also starts unpowered, nonautomatic and condition-false.
The live block type selects impulse/`REDSTONE`, repeating/`AUTO`, or chain/`SEQUENCE`; an unexpected
type falls back to `REDSTONE`. `CONDITIONAL` is read from the live state and otherwise false.

Base save always writes command, success, nullable custom name, tracking and last-execution-policy.
It writes nullable last output only while tracking and writes last execution only when the policy is
enabled and its value is not `-1`. Load defaults those fields to the constructor values; disabling
tracking clears output and disabling timestamp updates forces `-1`. Block entities additionally
save/load powered, condition-met and automatic, all defaulting false. The minecart embeds the same
base record in entity persistence; its transient activation throttle is not saved. Changing a
command immediately resets success to zero; the block subtype also marks its block entity changed.

**Placement:** Placement faces opposite the context's nearest looking direction and starts
unconditional. On the server, a stack without a `BLOCK_ENTITY_DATA` component initializes output
tracking from the live `send_command_feedback` rule and automatic from the registered block:
true only for the chain subtype and false for impulse/repeating. A stack carrying block-entity data
skips both defaults and retains its applied data instead. Both paths then sample current neighbor
power through the ordinary edge transaction. Chain automatic state neither schedules standalone
work nor turns that subtype into a repeating block. Generic item placement, game-master item
admission/components and the feedback rule's other consumers retain their separate owners.

**Block trigger and one-tick scheduling:** Server-side neighbor changes compare the current direct
neighbor-signal aggregate with saved `powered`. Only a changed edge writes that flag. A rising edge
schedules delay one after snapshotting the conditional predecessor result, unless the block is
automatic or is a chain block; a falling edge only clears power. Changing automatic from false to
true while unpowered, attached to a level and not a chain block similarly snapshots the condition
and schedules delay one. Switching into repeating mode schedules under current power or automatic
state. Turning automatic off does not cancel queued work.

Condition snapshot starts true. A conditional block instead reads one position opposite its own
facing and is true only when that position currently holds any command-block subtype with a command
block entity whose success count is greater than zero. This captured Boolean is persisted.

At a due impulse tick, the previously captured condition decides execution; a false conditional
condition clears success. At a due repeating tick, the old capture makes that same decision, then a
fresh predecessor sample is already stored for the next tick. A powered or automatic repeating
block schedules itself again at delay one. A scheduled chain block has no standalone execution
branch. Every compatible due block finally requests comparator-neighbor output updating, including
condition failure and disabled command dispatch.

**Base execution and live rule gate:** `performCommand` first compares level game time with saved
last execution. Equality returns false without clearing success/output, reading the rule or changing
the timestamp. Otherwise the case-insensitive command `Searge` is a pre-rule special case: it sets
literal output `#itzlipofutzli`, sets success one and returns true without updating last execution.
It also bypasses the tracked-output source and therefore invokes no carrier update hook itself.

Every other command first clears success. Only a true live rule and nonempty command continue to
normal dispatch. That branch clears prior output, builds the carrier's game-master-permission source
and calls `Commands#performPrefixedCommand`. Each result callback whose success Boolean is true adds
one to success, ignoring its integer result. Output tracking selects a closeable source; every
accepted message replaces last output with a wall-clock `HH:mm:ss` prefix and invokes the carrier's
update hook. `send_command_feedback` owns successful-message acceptance and
`command_block_output` owns administrator informing; failures are accepted while that source is
open. With tracking disabled, the null command source is used instead.

A false rule or empty ordinary command therefore clears success but preserves any old output. It
still reaches the common tail, returns true and either stores current game time or resets it to
`-1` according to the timestamp policy. A dispatch exception instead becomes an `Executing command
block` reported crash with command/name details and does not reach that tail. Exact parsing,
permissions, fork/context limits, target mutations and feedback recipients after dispatcher entry
belong to the command and remaining game-rule owners.

**Root execution and chain scan:** A due impulse/repeating block with a nonempty command calls the
base transaction and ignores its Boolean result; an empty root only clears success. Both paths then
scan from the root facing. The scan's independent bound is the live
`max_command_sequence_length` rule. Each visited position must be a chain-command-block state with a
chain-mode command block entity; another state/type terminates the scan. An unpowered and
nonautomatic chain block skips its command but continues in that block's facing.

An admitted chain block resamples its condition. A true condition calls `performCommand`; a false
return from that call terminates the whole scan and occurs only at the same-game-time guard. All
false-rule, empty or ordinary completed executions return true and update comparator neighbors. A
failed conditional chain clears success and continues. Each accepted block supplies the next
facing, so chains can turn. The precise
counter decrement/warning boundary remains owned by `max_command_sequence_length`; disabling
`command_blocks_work` does not stop traversal or timestamp updates and makes ordinary conditions
downstream observe zero success.

**Minecart activation:** A powered activator call executes only when entity `tickCount-lastActivated`
is at least four. It ignores the base Boolean result and then stores the current tick count even when
the rule is false, the command is empty or same-game-time suppression occurred. An unpowered or
throttled call changes nothing. A newly constructed/reloaded minecart consequently begins from its
transient zero-valued throttle fields. Its command source uses entity position/rotation and the
minecart as source entity; validity lasts until removal.

**Interaction, edit ingress and projection:** Opening either command-block UI requires
`canUseGameMasterBlocks` but never reads `command_blocks_work`; block use succeeds after opening,
and minecart use opens client-side while both sides return success. The operator-packet family owns
wire fields and admission. After permission and target validation, a block edit chooses the packet's
mode block type, preserves facing, installs its conditional bit with a flags-2 state write, reuses
the existing block entity, then writes command/tracking/automatic. Turning tracking off clears last
output; a mode change runs the scheduling hook. A minecart edit analogously writes command/tracking
and optionally clears output.

Those mutations occur even while the rule is false. True invokes the carrier update hook; false
suppresses it. The block hook publishes its current same-state block update with flags 3. The
minecart hook copies command and last output into synchronized entity data. A nonempty edit sends
the operator either the success or disabled translatable message; an empty edit sends neither.
Thus false means “do not normally execute or invoke this edit update hook,” not “reject editing.” A
mode flags-2 state publication can still occur, later UI opening reads server state, and normal
persistence retains all edits.

The block's analog-output method exposes current success count, with zero for a missing block
entity. Minecart synchronized-data import replaces its local command (thereby resetting local
success) or last output; malformed output import is swallowed. No client reads the gamerule to
simulate execution.

**Branches and aborts:**

Client-side neighbor callbacks; unchanged power; automatic/chain rising edges; absent/wrong block
entity; old versus new conditional snapshots; impulse/repeating/chain mode; empty command; same-time
execution; `Searge`; live rule false/true; tracking and both feedback rules; command success/failure/
exception; every chain state/mode/power/condition/result/bound branch; minecart power/throttle; UI
permission; edit permission/target/mode/tracking/empty-command/rule and carrier update subtype.

**Constants and randomness:**

Rule default true; scheduled delay one; minecart throttle four entity ticks; last-execution sentinel
`-1`; default name `@`; special input/output `Searge`/`#itzlipofutzli`; output timestamp format
`HH:mm:ss`; block edit state flags 2 and update flags 3. The carrier state machine consumes no RNG.
The selected command, its targets and effects may do so under their own owners.

**Side effects:**

Scheduled ticks, powered/automatic/condition/block-mode state, command persistence, success/output/
timestamp state, dirty block entities, comparator-neighbor work, command dispatch and its separately
owned mutations, crash propagation, operator messages, block updates and synchronized minecart
metadata.

**Gates:**

Server side and compatible live carrier; power/automatic/mode/condition; scheduled admission or
minecart throttle; same-game-time guard; special versus ordinary command; live
`command_blocks_work`; nonempty command; operator permission and valid edit/UI target; tracking and
separately owned feedback/limit rules.

**Boundary cases and quirks:**

The rule is a dispatch gate, not a scheduler gate. Disabled executions zero success, retain old
output, advance ordinary timestamps and continue chains. `Searge` bypasses the rule and timestamp
tail, so it can execute repeatedly in one game tick unless a prior timestamp already equals that
tick. Empty roots still scan chains but do not update their own timestamp; empty chain commands do.
Repeating conditional execution uses the previous condition sample while storing the next one.
Editing while disabled persists silently except for the disabled operator message and any independent
mode state publication.

**Evidence:**

`OFF-SERVER-001`; `OFF-REPORT-001`;
`net.minecraft.world.level.gamerules.GameRules`;
`net.minecraft.server.level.ServerLevel#isCommandBlockEnabled`;
`net.minecraft.world.level.BaseCommandBlock#performCommand`, `#save`, `#load`;
`net.minecraft.world.level.BaseCommandBlock$CloseableCommandBlockSource`;
`net.minecraft.world.level.block.CommandBlock#neighborChanged`, `#tick`, `#executeChain`,
`#setPlacedBy`, `#useWithoutItem`, `#getAnalogOutputSignal`;
`net.minecraft.world.level.block.entity.CommandBlockEntity`;
`net.minecraft.world.entity.vehicle.minecart.MinecartCommandBlock`;
`net.minecraft.server.network.ServerGamePacketListenerImpl#handleSetCommandBlock`,
`#handleSetCommandMinecart`; `PROTO-PLAY-SERVERBOUND-OPERATOR-BLOCKS-001`; `EXP-BLK-017`.

**Test vectors:**

Cross all three modes/facings/conditional bits with rising/falling/unchanged power, automatic
transitions, plain/component-bearing placement, both tracking defaults, captured predecessor counts,
accepted/duplicate scheduled ticks and save/load. For base
execution cross rule true/false, empty/ordinary/case variants of `Searge`, tracking, timestamp policy,
same/different game time, zero/one/multiple successful callbacks, failure/output/exception and exact
state/publication results. Build straight/turning/broken chains with every power/automatic/condition
and duplicate-time branch around the independent limit. Trigger/reload minecarts at throttle
differences 3/4 and rule branches. Finally replay both edit packets and both UI interactions across
permission, target, mode, tracking, empty command and live rule, asserting persisted state, dirty
state, messages, flags-2/3 publication and synchronized metadata.
