# Blocks mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `BLK-TEST-BLOCK-001` — Test blocks turn redstone edges into ordered block-based test outcomes

**Parent:** `SIM-003`, `BLK-001`, `BLK-002`, `BLK-003`, `BLK-007`, `PLY-005`,
`RED-001`, `CLI-005`, `CLI-006`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked server and client classes, reports, protocol owners and bundled assets
fix the test block's four modes, redstone edge state, transient trigger latch, block-based test scan,
operator edit transaction, persistence and client projection. Packet framing and decode faults remain
owned by `PROTO-PLAY-SERVERBOUND-OPERATOR-BLOCKS-001`; generic game-master placement and breaking
remain owned by `BLK-PLACE-001` and `BLK-BREAK-001`.

**Applies when:**

A test block is placed, cloned, used, edited, powered, triggered, reset, ticked, saved, loaded or
synchronized; or a `BlockBasedTestInstance` starts and scans test blocks inside its structure.

**Authoritative state:**

The block has one `mode` property and four locked states:

| State ID | Mode | Mode wire ID | Purpose |
|---:|---|---:|---|
| 21738 | `start` (default) | 0 | Emits the test-start redstone signal |
| 21739 | `log` | 1 | Records a message without ending the test |
| 21740 | `fail` | 2 | Fails an active block-based test |
| 21741 | `accept` | 3 | Succeeds an active block-based test |

The entity has four runtime fields. `mode` begins from the constructed block state, `message` is
empty, `powered=false`, and the Java-default `triggered=false`. The first three fields are saved and
sent in the update tag; `triggered` is transient and never saved or synchronized. The block-state
mode and entity mode are separate values and malformed data or failed state writes can make them
diverge.

The block is a full cube with light-gray map color, hardness `-1`, blast resistance `3,600,000` and
no loot table. It belongs to `dragon_immune` and `wither_immune`. Its entity protocol ID is `45`.
The ordinary item is epic-rarity `GameMasterBlockItem` with `block_state.mode=start`.

**Transition and ordering:**

Placement selects the item component's valid mode, then constructs the entity from that state.
Neighbor callbacks sample redstone only for non-start entity modes. A rising edge stores powered,
then triggers; a falling edge only clears powered. Direct test-framework triggering bypasses that
edge detector. A start trigger changes output and notifies neighbors before logging. A block-based
test starts that block once, then scans accept before fail before log on every registered test tick.
Operator edits install entity mode, attempt its flags-2 state projection, install message, dirty,
then send the direct block update. Save/load and client update tags operate independently of these
live transitions.

**Placement, cloning and game-master gates:**

`getStateForPlacement` starts from state ID 21738 and reads only the `mode` entry from the stack's
block-state component. A missing component, missing mode or unparsable property retains start. The
game-master item supplies a candidate only for a null player or a player for whom
`canUseGameMasterBlocks` is true; the normal nonnull requirement is both `instabuild` and
command-game-master permission. Generic block placement owns the later survival, collision, write
and stack transaction.

Clone/pick creates the ordinary item stack and overwrites its block-state component with the live
state mode. It does not copy entity message, powered or triggered data. The generic game-master
break rule rejects missing permission before ordinary unbreakable handling; the block has no loot.

**Use and local edit screen:**

`useWithoutItem` first requires a matching test-block entity and then
`player.canUseGameMasterBlocks`. Either failure returns `PASS`. Admission returns `SUCCESS` on both
sides; only the client side calls `openTestBlock`. The base/server player method is inert, while
`LocalPlayer` installs `TestBlockEditScreen` directly from the synchronized client entity. No menu
or clientbound open-screen packet exists. Generic interaction still owns reach, hit, hand, sneak
and item-precedence admission.

The screen copies position, entity mode and message when constructed. Its cycle control contains
start, log, fail and accept in wire-ID order. The message box accepts at most 128 UTF-16 code units
and is visible for every mode except start; hiding it does not erase the retained text. Done reads
the box, sends exactly one set-test-block packet and closes. Cancel, ordinary close and Escape close
without sending. The screen does not pause the game and identifies itself as in-game UI.

The canonical UI therefore emits at most 128 Java UTF-16 code units, but the packet's default UTF-8
codec accepts up to 32,767 code units. Mode uses a VarInt ID mapper with zero fallback, so every
forged invalid signed value becomes start before handler dispatch.

**Server edit transaction:**

After same-thread dispatch, the handler requires `canUseGameMasterBlocks`; denial silently returns.
It captures the live state and continues only when the position contains a `TestBlockEntity`. It
does not separately require the captured block to be `minecraft:test_block`, nor does it impose a
reach check.

The handler calls `setMode` first. That setter replaces the entity field, then, when the level is
present and the live block is test block, attempts `setBlock(position, liveState[mode=newMode], 2)`.
The Boolean result is ignored. Flags 2 publish the state but do not notify redstone neighbors. A
non-test live block skips this state attempt while retaining the new entity mode.

Next the handler replaces message, calls `setChanged`, and calls
`sendBlockUpdated(position,capturedState,entity.getBlockState(),3)`. Every field mutation precedes
dirtiness and the direct update. As in the locked server method, that final integer is unused: a
visible chunk queues the position and invalidates its path-type cache; navigation recomputation is
skipped when old and new collision shapes are equal. Same-mode or duplicate state publication may
coalesce in the chunk change set. Wrong/missing entities leave no residue.

Editing does not clear powered or triggered and does not call `updateNeighborsAt`. Switching away
from a powered start state makes its ordinary signal read zero immediately from the block mode, but
neighbor devices need some separately admitted update to recompute. Switching back to start can
re-expose the retained powered value. A rejected flags-2 write leaves entity mode/message saved and
synchronized even though the live state mode did not converge.

**Redstone edge and reset state machine:**

`neighborChanged` operates only in a `ServerLevel` with a matching entity. If entity mode is start,
it returns without sampling input. Otherwise it reads `hasNeighborSignal(position)` and compares it
with entity powered:

- `true/false` stores `powered=true`, then calls `trigger`;
- `true/true` and `false/false` do nothing;
- `false/true` stores `powered=false` without triggering.

These powered mutations do not dirty or synchronize the entity. They are nevertheless part of a
later save if another admitted mutation makes the chunk durable.

Triggering start stores powered true, calls `updateNeighborsAt` with the current block type, queries
`blockTicks.willTickThisTick(position, block)` and discards the result, then logs its nonblank
message. It returns without setting triggered. The query may materialize the scheduler's current
run-set cache, but it neither creates nor deduplicates a scheduled tick. Consequently a start block
does not schedule its own reset. Its ordinary signal is 15 in every queried direction exactly while
the live block-state mode is start, a matching entity exists and entity powered is true; otherwise
it is zero. Direct signal remains the inherited zero and there is no comparator output.

Triggering log writes one info-log record for a nonblank message, then sets triggered true.
Triggering fail or accept only sets triggered true. `reset` first clears triggered for every mode.
For start with a level it additionally clears powered and calls `updateNeighborsAt`; other modes
retain powered. An externally scheduled tick invokes reset, but trigger itself never schedules it.

**Block-based test transaction:**

`BlockBasedTestInstance.run` scans the contracted interior of the rotated test-structure bounds.
It selects blocks by their live block-state mode, then requires a matching entity when consumed.
Exactly one start block is required: zero or more than one throws the corresponding test assertion.
The sole start entity is triggered, after which `onEachTick` registers the following ordered scan
from the current test tick through the timeout range:

1. Rescan accept blocks. Having none is an assertion failure. If any accept entity has triggered,
   succeed immediately; fail and log are not processed that tick, and the accepting latch is not
   reset by this path.
2. Otherwise rescan fail blocks in structure traversal order. The first triggered one throws a test
   assertion whose literal component is its message. The throw prevents that entity's following
   reset and aborts later fail/log processing.
3. If no failure throws, rescan log blocks. For each triggered entity, log its nonblank message and
   then reset it. Because the rising-edge `trigger` already logged a log-mode message, an admitted
   log edge during an active block-based test produces the immediate record and a second record
   during this scan before the latch clears.

Accept success takes precedence over an already-triggered fail in the same scan. No fail or log
block is required. Repeated scans use current block states and entities, so mode edits, replacement
and missing entities affect the next callback rather than a cached inventory.

**Persistence and synchronization:**

Full save and update tag write enum `mode`, string `message` and Boolean `powered`. Generic full
metadata separately owns type, position and components. Load independently replaces each field:
missing or invalid mode becomes fail, missing/wrong message becomes empty, and missing/wrong powered
becomes false. Triggered is not read and remains its newly constructed false value.

Load does not call `updateBlockState`, dirty, notify neighbors or schedule work. Therefore a loaded
entity may say fail while the separately loaded block state remains another mode. Redstone edge
handling uses entity mode, ordinary output uses block-state mode plus entity powered, and the local
screen displays entity mode. The ordinary block-entity-data packet carries this same custom tag to
the client. There is no dedicated block-entity renderer; the four cube models select solely from
block state. The item model selects from its block-state component and falls back to start.

**Branches and aborts:**

All four state/wire modes; every valid/missing/invalid item mode; null and every permission/ability
combination; matching/wrong/missing entity; client/server side; message code-unit lengths
0/128/129/32767 and decode overflow; Done/Cancel/close; canonical and invalid mode IDs;
same/different/non-test live state; accepted/rejected flags-2 writes; all four input/powered edge
pairs; direct triggers; present
and absent pending ticks; scheduled reset; zero/one/many start and accept blocks; simultaneous
accept/fail/log latches; blank/nonblank messages; full save/update tag and every missing/wrong field.

**Constants and randomness:**

State IDs `21738..21741`; wire mode IDs `0..3`; entity protocol ID `45`; UI message maximum `128`
UTF-16 code units; packet message maximum `32,767` code units; signal `15`; edit state flags `2`;
direct update argument `3`; hardness `-1`; blast resistance `3,600,000`. Block, entity, handler, UI
and block-based test scans consume no RNG.

**Side effects:**

Local screen installation/closure; one serverbound edit packet; block-state and entity-data
projection; chunk dirtiness; path-cache invalidation; powered/trigger latches; neighbor updates from
start trigger/reset; info-log records; GameTest success or assertion failure. This subtype emits no
sound, particle, game event, inventory mutation, comparator output, direct signal or self-created
scheduled tick.

**Gates:**

Game-master placement/use/break/edit permission; generic interaction and packet-phase admission;
matching entity; live test block for mode-state convergence; non-start entity mode for redstone
input; matching mode/state/entity within the active test bounds. Difficulty and game rules do not
gate these transitions.

**Boundary cases and quirks:**

Mode exists twice and can diverge. Missing persisted mode defaults to fail rather than the block's
start default. Powered persists but its redstone setter does not dirty; triggered is transient.
Start output is ordinary-only and remains on until an external reset. Trigger performs a discarded
same-tick schedule-membership query rather than scheduling. Log edges are logged twice inside an
active block-based test. Accept wins over fail in the same callback; a thrown fail remains latched.
Edits neither clear latches nor notify neighbors. The edit screen is local rather than menu-opened.

**Evidence:**

`Confirmed`; `OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-REPORT-001`. Anchors:
`net.minecraft.world.level.block.TestBlock`,
`net.minecraft.world.level.block.entity.TestBlockEntity`,
`net.minecraft.world.level.block.state.properties.TestBlockMode`,
`net.minecraft.gametest.framework.BlockBasedTestInstance`,
`net.minecraft.gametest.framework.GameTestHelper#forEveryBlockInStructure`,
`net.minecraft.gametest.framework.GameTestHelper#onEachTick`,
`net.minecraft.world.ticks.LevelTicks#willTickThisTick`,
`net.minecraft.client.gui.screens.inventory.TestBlockEditScreen`,
`net.minecraft.client.player.LocalPlayer#openTestBlock`,
`net.minecraft.server.network.ServerGamePacketListenerImpl#handleSetTestBlock`; locked block,
block-entity and item reports; bundled test-block blockstate/model/item assets; `EXP-BLK-022`.

**Test vectors:**

Exhaust the four state and wire IDs, item placement/clone, permission/side/entity gates, every UI and
forged message/mode boundary, exact setter/write/dirty/update order and failed-write divergence.
Drive every redstone edge, direct trigger and reset path with scheduler membership capture. Run
zero/one/many start/accept plus simultaneous accept/fail/log latches in rotating test structures,
then save/load/update all field/default combinations and compare block model, UI, signal, logs and
test result. Run `EXP-BLK-022` as the executable matrix.
