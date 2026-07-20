# Blocks mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `BLK-BREAK-001` — Player breaking has a separate authoritative progress and harvest transaction

**Parent:** `BLK-002`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — the locked source fixes packet acknowledgement, the complete start/stop/abort
and delayed-progress state machine, mining-speed formula and Java numeric operations, generic
tool/debug-stick/shears dispatch, crack publication, commit eligibility,
callback/removal/tool/loot-hook order and every generic correction branch below. Concrete block hook
and loot-table algorithms are explicitly separated into `BLK-BREAK-HOOK-001`, so they are not
falsely claimed by this generic rule.

**Applies when:**

A loaded player's `START_DESTROY_BLOCK`, `STOP_DESTROY_BLOCK` or `ABORT_DESTROY_BLOCK` packet
reaches the level thread, while an active or delayed server progress record ticks, or an eligible
destroy commit is attempted.

**Authoritative state:**

Each `ServerPlayerGameMode` owns signed-int `gameTicks`, `isDestroyingBlock`,
`destroyProgressStart`, `destroyPos`, `hasDelayedDestroy`, `delayedDestroyPos`, `delayedTickStart`
and one shared `lastSentState` initialized to `-1`. The server additionally owns the current target
state/block entity/fluid, game mode/abilities/permissions, selected stack and components,
attributes/effects, gamerules, loot state and all world writes. Progress is recomputed from current
state/player inputs; it is not an accumulated client value.

**Transition and ordering:**

Packet admission begins in `ServerGamePacketListenerImpl#handlePlayerAction`, which moves the packet
to the level thread, drops it when the client is not loaded, records player activity, and routes the
three destroy actions to `handleBlockBreakAction(pos,action,direction,level.maxY,sequence)`. The
handler acknowledges `sequence` **after** that call returns, including its ordinary rejection
branches. At the game-mode layer, `direction` has no effect. Every action first requires
`isWithinBlockInteractionRange(pos,1.0)`; failure has no state correction. `pos.y>maxY` sends one
current-state block update to the actor and stops; no explicit minimum-Y test exists here.

Only `START` then checks, in order: spawn protection (message, no state update),
`ServerLevel#mayInteract` (current-state update), `abilities.instabuild`, and
`Player#blockActionRestricted`. The latter is false when the game type does not restrict block
placing or `mayBuild` is true; spectator is always restricted; otherwise a nonempty main hand must
satisfy its `can_break` adventure predicate. Restriction sends a current-state update. `STOP` and
`ABORT` do not rerun those four start-only gates.

**Transition and ordering — start:**

An instabuild start immediately calls `destroyAndAck`. Otherwise:

1. Set `destroyProgressStart=gameTicks`, read the target state and initialize local progress to
   `1.0`. For non-air, run `EnchantmentHelper#onHitBlock` on the current main hand at block center,
   with its item-broken callback; then call current-state `attack`; then compute one-tick destroy
   progress using the possibly mutated current player/tool state.
2. A non-air result `>=1.0` immediately calls `destroyAndAck` and does not modify/clear an older
   active or delayed progress record.
3. Otherwise, if `isDestroyingBlock` was already true, send the actor a current-state correction for
   the **old** `destroyPos`; do not send crack stage `-1` for that old position. Set
   `isDestroyingBlock=true`, replace `destroyPos` with an immutable copy of the new position,
   broadcast initial stage `(int)(progress*10)`, and store it in `lastSentState`. Starting on air
   therefore briefly publishes stage `10`; the next active tick clears it.

**Transition and ordering — tick and actions:**

`ServerPlayerGameMode#tick` pre-increments `gameTicks`. If `hasDelayedDestroy`, it exclusively
processes that record; an active record can coexist but receives no work. Air clears only
`hasDelayedDestroy`. Otherwise compute
`progress=currentState.getDestroyProgress(currentPlayerInputs)*(gameTicks-delayedTickStart+1)`,
publishing a changed stage first; at `progress>=1.0`, clear the delayed flag and call `destroyBlock`
directly, ignoring its result. If no delayed record and `isDestroyingBlock`, air broadcasts `-1`,
sets `lastSentState=-1` and clears the active flag; non-air only recomputes/publishes progress.
Active progress never auto-commits, even at or above `1.0`; it waits for `STOP`.

`STOP` is keyed only by equality with the stored `destroyPos`, not by `isDestroyingBlock`. For a
matching non-air target, compute `(gameTicks-destroyProgressStart+1)*current per-tick progress`. At
`>=0.7`, clear the active flag, broadcast `-1`, and call `destroyAndAck`. Below `0.7`, when no
delayed record already exists, clear active, set delayed, copy the packet position, and set
`delayedTickStart=destroyProgressStart`; it does not clear the crack. If a delayed record already
exists, even the active flag is left unchanged. A mismatched or air `STOP` performs no state/crack
correction. `ABORT` always clears only the active flag. If packet position differs from stored
`destroyPos`, log the mismatch and broadcast `-1` for the old position; then always broadcast `-1`
for the packet position. It does not cancel delayed destruction.

**Mining-speed formula:**

Let hardness be `state.getDestroySpeed(level,pos)`. Hardness exactly `-1.0f` yields per-tick
progress `0`. Otherwise:

1. Raw selected-item speed is the first ordered `minecraft:tool` rule with a present speed whose
   block set matches, or `default_mining_speed` (default `1.0f`); a stack without `tool` also yields
   `1.0f`. If raw speed is strictly greater than `1.0f`, add the player's `MINING_EFFICIENCY`
   attribute converted from double to float.
2. A dig-speed effect multiplies by `1 + (amplifier+1)*0.2f`. Mining Fatigue amplifiers `0,1,2,>=3`
   multiply by `0.3f,0.09f,0.0027f,0.00081f` respectively. Multiply by the `BLOCK_BREAK_SPEED`
   attribute. If eyes are in the water tag, multiply by `SUBMERGED_MINING_SPEED`; if not on ground,
   divide by `5.0f`.
3. `correctTool = !state.requiresCorrectToolForDrops || first matching tool rule with a present correct-for-drops value is true`;
   no matching/tool component means false for a state that requires one. Per-tick progress is
   `playerSpeed / hardness / (correctTool ? 30.0f : 100.0f)`.

All operations are Java `float` in the stated order after attribute double-to-float conversion.
Elapsed ticks and stages use signed-int arithmetic; stage is Java float-to-int conversion of
`progress*10.0f` (truncate toward zero, with Java saturation/NaN semantics). A zero hardness can
therefore yield infinity and instant completion. Changing state, tool, effects or attributes
retroactively rescales the entire `(elapsed+1)` interval because no past increments are accumulated.
Tool rules for speed and correct-drops are searched independently and first-match only. Tool codec
defaults are speed `1.0`, damage per block `1`, and creative destruction allowed.

**Crack publication:**

`ServerLevel#destroyBlockProgress(breakerId,pos,stage)` sends `ClientboundBlockDestructionPacket` to
every other player in the same level whose squared distance from the integer block coordinates is
strictly `<1024.0` (32 blocks). Start publishes unconditionally; tick publication occurs only when
the integer differs from shared `lastSentState`. Stage is not clamped to `0..9`, so an active record
can publish `10` or more. No chunk-activity gate appears in this broadcaster.

**Destroy commit and ordering:**

`destroyAndAck` calls `destroyBlock`; only a `false` result sends the actor a current-state
correction. `destroyBlock(pos)` performs:

1. Read `originalState`, call current-main-stack `canDestroyBlock`, and return false immediately on
   denial. Base item logic denies only an instabuild player holding a `tool` whose
   `can_destroy_blocks_in_creative` is false. `debug_stick` is the sole item override: for a server
   player it performs its left-click property-selection interaction, then always denies destruction.
2. Capture the block entity and original block. A `GameMasterBlock` without `canUseGameMasterBlocks`
   publishes old-to-old with flags `3` and returns false. Rerun `blockActionRestricted`; denial
   returns false.
3. Call original-block `playerWillDestroy`, retaining its returned state as `destroyedState`. The
   base hook emits level event `2001`, angers nearby piglins for the guarded tag, and emits
   `BLOCK_DESTROY`, all before removal. Then `Level#removeBlock(pos,false)` rereads the fluid
   **after** that hook and writes its legacy block with flags `3`. If the write succeeds, call
   original-block `destroy(level,pos,destroyedState)`.
4. If `player.preventsBlockDrops` (`abilities.instabuild`) return true now, even if removal returned
   false. Otherwise read the current main hand after callbacks, copy it, compute correct-tool
   against `destroyedState`, and call `mineBlock` on the live stack. Base item logic returns false
   with no `tool`; with one it returns true and, server-side when destroyed hardness is nonzero and
   `damage_per_block>0`, damages the main-hand stack by that amount. Shears instead skip durability
   only for the `fire` block tag. A true item callback awards that item's `ITEM_USED` stat.
5. Only when removal succeeded **and** correct-tool is true, call original-block `playerDestroy`
   with the already captured block entity and the tool copy from before mining damage. The base hook
   awards `BLOCK_MINED`, adds `0.005f` exhaustion, evaluates drops with origin=center, optional
   player/BE and the pre-damage tool, attempts each returned item spawn in list order, then calls
   `spawnAfterBreak(server,pos,tool,true)`. Item/XP spawn helpers are gated by `doBlockDrops`, but
   loot evaluation and the after-break hook still run when it is false. Return true after this
   survival path even when removal failed or the tool was wrong.

Concrete overrides can alter the returned destroy state, counterpart removal, effects, drops and
after-break XP; their exact catalog mapping is `BLK-BREAK-HOOK-001`. The generic caller never
rereads the original block type after `playerWillDestroy`, and `removeBlock` can overwrite a
replacement installed by that callback.

**Branches and aborts:**

Unloaded client; range; high Y; start-only protection/interaction/restriction; instabuild; air;
instant progress; active/delayed/mismatched actions; unbreakable hardness; item denial; game-master
denial; commit-time adventure denial; failed removal; creative drop prevention; wrong tool; absent
tool component; disabled block drops. A false eligibility result is the only `destroyAndAck`
correction condition; removal failure still reports generic commit success. Delayed completion
ignores false with no direct correction.

**Constants and randomness:**

Range padding `1.0`; hardness sentinel `-1.0`; divisors `30`/`100`; stop threshold `0.7`; completion
`1.0`; crack factor `10`; crack radius squared `1024`; speed/effect constants above; level event
`2001`; exhaustion `0.005`; update flags `3`. Admission, progress and generic commit consume no RNG
except data-driven `onHitBlock` effects. Loot-table selection and per-stack item-entity placement
consume RNG only after the stat/exhaustion step and are specified by `ITM-LOOT-001`; subtype
`spawnAfterBreak` consumption is owned by `BLK-BREAK-HOOK-001`.

**Side effects:**

Activity reset and sequence ACK; messages/corrections; enchantment-on-hit and item break callback;
block attack; crack packets; destroy particles/event, piglin anger and game event; fluid-restoring
state mutation and all `BLK-UPDATE-001` consequences; block-entity invalidation with captured
reference retained for loot; block destroy hook; tool damage/break and item-used stat; block-mined
stat/exhaustion; loot evaluation/item entities; after-break/XP effects. Order is exactly as above.

**Gates:**

Client loaded, interaction range, upper build bound, start-only spawn/mayInteract/adventure,
instabuild, current non-air state, numeric thresholds, item creative-destroy property/debug-stick
behavior, game-master permission, repeated adventure check, removal success, creative drop
prevention, correct tool and `doBlockDrops`. Difficulty does not alter the generic state machine.

**Boundary cases and quirks:**

Active and delayed records may coexist; delayed always starves active work. A new instant start
leaves an old active record intact. A slow replacement start corrects but does not clear the old
crack. `STOP` can operate on a stale stored position without an active flag; `ABORT` never cancels
delayed work. Replacement states inherit the original start time and can be destroyed. Speed changes
apply retroactively to all elapsed ticks. Active progress can exceed stage 9 indefinitely. Air start
publishes stage 10. Callback replacement can be overwritten by fluid restoration. Creative and
removal-failed paths can return success without drops, and survival removal failure can still damage
the tool. The pre-damage tool copy and pre-removal block entity feed loot after both have changed in
live world state.

**Evidence:**

`Confirmed`; `OFF-SERVER-001`. Anchors:
`net.minecraft.server.network.ServerGamePacketListenerImpl#handlePlayerAction(net.minecraft.network.protocol.game.ServerboundPlayerActionPacket)`,
`net.minecraft.server.level.ServerPlayerGameMode#tick()`,
`net.minecraft.server.level.ServerPlayerGameMode#incrementDestroyProgress(net.minecraft.world.level.block.state.BlockState,net.minecraft.core.BlockPos,int)`,
`net.minecraft.server.level.ServerPlayerGameMode#handleBlockBreakAction(net.minecraft.core.BlockPos,net.minecraft.network.protocol.game.ServerboundPlayerActionPacket$Action,net.minecraft.core.Direction,int,int)`,
`net.minecraft.server.level.ServerPlayerGameMode#destroyAndAck(net.minecraft.core.BlockPos,int,java.lang.String)`,
`net.minecraft.server.level.ServerPlayerGameMode#destroyBlock(net.minecraft.core.BlockPos)`,
`net.minecraft.world.level.block.state.BlockBehaviour#getDestroyProgress(net.minecraft.world.level.block.state.BlockState,net.minecraft.world.entity.player.Player,net.minecraft.world.level.BlockGetter,net.minecraft.core.BlockPos)`,
`net.minecraft.world.entity.player.Player#getDestroySpeed(net.minecraft.world.level.block.state.BlockState)`,
`net.minecraft.world.item.component.Tool#getMiningSpeed(net.minecraft.world.level.block.state.BlockState)`,
`net.minecraft.world.item.component.Tool#isCorrectForDrops(net.minecraft.world.level.block.state.BlockState)`,
`net.minecraft.server.level.ServerLevel#destroyBlockProgress(int,net.minecraft.core.BlockPos,int)`,
`net.minecraft.world.level.block.Block#playerWillDestroy(net.minecraft.world.level.Level,net.minecraft.core.BlockPos,net.minecraft.world.level.block.state.BlockState,net.minecraft.world.entity.player.Player)`,
`net.minecraft.world.level.Level#removeBlock(net.minecraft.core.BlockPos,boolean)`, and
`net.minecraft.world.level.block.Block#playerDestroy(net.minecraft.world.level.Level,net.minecraft.world.entity.player.Player,net.minecraft.core.BlockPos,net.minecraft.world.level.block.state.BlockState,net.minecraft.world.level.block.entity.BlockEntity,net.minecraft.world.item.ItemStack)`.

**Test vectors:**

(1) Assert ACK and correction differences for unloaded, range, high-Y, spawn, mayInteract,
spectator/adventure and commit-time denial. (2) Trace start/stop/abort at exact `0.7`/`1.0`, air,
mismatch and stale stored positions; create coexisting active/delayed records and a new instant
start. (3) Change tool/effects/state after several ticks and prove whole-interval recomputation,
float stage conversion, stages above 9 and replacement destruction. (4) Exercise every mining-speed
factor, hardness `-1`/`0`, ordered tool rules, correct/wrong/no tool, debug stick and
shears-on-fire. (5) Force `playerWillDestroy` replacement and removal failure in survival/creative;
assert fluid overwrite, return/correction, tool damage and callback conditions. (6) Record BE
capture, base event/removal/destroy/tool/stat/exhaustion/loot/after-break order with `doBlockDrops`
both ways. `EXP-BLK-004` is the generic conformance matrix.
