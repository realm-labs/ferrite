# Blocks mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `BLK-PLACE-001` — Block-item placement is an ordered, non-atomic commit pipeline

**Parent:** `BLK-002`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — packet admission, interaction precedence, generic `BlockItem` transaction
control flow, every concrete `BlockItem` subclass dispatch family, state-component and block-entity
transfer order, bed/door/double-plant multi-position writes, flags, branch results and observable
side effects are fixed below. Concrete blocks' candidate-state formulas remain attached to their
audited catalog families rather than being treated as part of the generic transaction.

**Applies when:**

A player use-on-block reaches an item registered as `BlockItem`, including its bed, double-high,
standing/wall, sign, water-surface, scaffolding, game-master and solid-bucket variants. Dispenser
placement uses its own item/content rule and is not silently equated with this player path.

**Authoritative state:**

The server owns the held stack, feature flags, player permissions/cooldown/mode, hit
block/face/vector/sequence, target and adjacent state/fluid, collision entities, world/build border,
block entities and all resulting world writes. The client may run the same item method as
prediction; the two unconditional server correction states described below win.

**Transition and ordering:**

The closed pipeline is packet admission and sequence acknowledgement; hit-block/item interaction
precedence; placement-context target selection; candidate derivation and validation; an initial
state write; conditional state/BE/callback work after a type-only reread; unconditional success
sound, game event, consumption and result; then handler correction publication. The following
sections define every branch in that order.

**Admission and interaction precedence:**

`ServerGamePacketListenerImpl#handleUseItemOn` first moves the packet to the level thread and
rejects an unloaded client, acknowledging its block-change sequence before item validation. It
rejects a disabled held item, a target outside `isWithinBlockInteractionRange(pos, 1.0)`, or a hit
vector whose offset from the target center has any absolute component `>=1.0000001`. It resets the
action timer only after those checks. Target Y greater than `maxY` or below `minY` produces the
matching build-limit message and stops. Spawn protection stops with its message; an outstanding
teleport or `ServerLevel#mayInteract` rejection skips the action. Regardless of success, rejection
at this late gate, or item failure, the handler finally sends authoritative block states for the hit
position and `hit.relative(face)`.

`ServerPlayerGameMode#useItemOn` rejects a disabled hit block. Spectators only open its menu
provider (`CONSUME`) or return `PASS`. Otherwise, unless secondary-use is active while either hand
is nonempty, the hit state receives item interaction first and may consume the action; the main hand
may then receive empty-hand block interaction. Only if neither consumes does the nonempty,
non-cooldown held stack receive `ItemStack#useOn`. For an infinite-materials player, the game-mode
wrapper restores the pre-call stack count; other component and world effects remain. A consuming
block/item result triggers the corresponding use-on criteria from the pre-call stack snapshot.
`ItemStack#useOn` returns `PASS` before item dispatch when a player without `mayBuild` lacks a
matching `can_place_on` predicate; a successful item-interaction result awards `Stats.ITEM_USED`
once.

**Target and candidate derivation:**

Constructing `BlockPlaceContext` records `relativePos=hitPos.relative(face)` and computes
`replaceClicked=state(hitPos).canBeReplaced(context)`. Its placement target is the hit position when
true, otherwise `relativePos`; `canPlace()` accepts the former immediately and otherwise rechecks
replacement at the relative target. Generic `BlockItem#place` then runs, in order:

1. Reject `FAIL` if the placed block's required feature is disabled or the original context cannot
   place. Call `updatePlacementContext`; null rejects. The base implementation returns the same
   context.
2. Ask the selected block for `getStateForPlacement`. The base block returns its default state;
   content families derive facing, clicked sub-shape, fluids and neighbors. Null rejects. Generic
   `BlockItem#canPlace` then requires `candidate.canSurvive(level,target)` and
   `level.isUnobstructed(candidate,target,CollisionContext.placementContext(player))`; either false
   rejects. The standing/wall variant instead searches as specified below.
3. Call `placeBlock`. Generic placement writes the candidate with flags `11`
   (`NEIGHBORS|CLIENTS|IMMEDIATE`) and rejects if `setBlock` returns false. This boolean means the
   initial mutation was accepted, not that the candidate survived its callbacks (`BLK-UPDATE-001`).
4. Reread target as `current`. Only if `current` has the candidate's block type: apply the
   block-state component as in `BLK-STATE-001`; attempt typed custom block-entity NBT on the server
   (type must match, and an `onlyOpCanSetNbt` type additionally requires `canUseGameMasterBlocks`);
   apply item-stack components to any current block entity and mark it changed; call
   `currentBlock.setPlacedBy(level,target,derivedState,player,stack)`; then trigger `PLACED_BLOCK`
   for a `ServerPlayer`. Custom NBT's boolean is otherwise ignored. Sign items open the front-side
   editor only on the server when custom NBT was not applied, a player exists, and the placed
   state/entity are still a sign.
5. Whether or not step 4 ran, use the local `derivedState` variable's sound type, play its place
   sound in `BLOCKS` at volume `(sound.volume+1)/2` and pitch `sound.pitch*0.8`, emit `BLOCK_PLACE`
   at target with `(player,derivedState)`, consume one item unless the living placer has infinite
   materials, and return shared `SUCCESS` (client swing source). If the initial reread had a
   different block type, `derivedState` is that reread state: the action still sounds, emits,
   consumes and succeeds, but performs no component/BE/`setPlacedBy`/`PLACED_BLOCK` work. If the
   flag-`2` state-component write was changed or rejected by a callback, later work still uses the
   requested patched state without rereading.

`BlockItem#useOn` returns this result. Only when placement did not consume and the stack has a
`consumable` component does it fall through to ordinary `Item#use`; a placement failure can
therefore begin consumption rather than returning the placement `FAIL`.

**Concrete item dispatch families:**

The catalog selectors are part of this rule and must not fall through to generic `block-item`:

- `DoubleHighBlockItem` (all `*_door`, `small_dripleaf`, and sunflower/lilac/rose bush/peony/tall
  grass/large fern) runs normal candidate validation first. Its `placeBlock` then replaces
  `target.above()` with source water when `isWaterAt`, otherwise air, using flags `27`
  (`NEIGHBORS|CLIENTS|IMMEDIATE|KNOWN_SHAPE`) and ignoring the result; only then does it write the
  lower state with generic flags `11`. Thus an upper replaceable block can be cleared even when the
  lower write later fails. Door and ordinary double-plant candidates require `target.y<maxY` and a
  replaceable upper state; upper collision is not separately tested.
- `BedItem` writes the foot using flags `26` (`CLIENTS|IMMEDIATE|KNOWN_SHAPE`), deliberately
  suppressing ordinary neighbors before the head exists. `BedBlock` requires the head at
  `target.relative(facing)` to be replaceable and within the world border, then `setPlacedBy` writes
  the head (`part=head`) with `setBlockAndUpdate`, flags `3`. Only the foot candidate receives the
  generic survival/collision test; the head position has no separate entity-collision test.
- Door state uses player horizontal direction, `half=lower`, and sets both `powered` and `open` when
  either half position has a neighbor signal. For hinge selection, let left/right be
  counterclockwise/clockwise of facing and
  `score=-full(leftLower)-full(leftUpper)+full(rightLower)+full(rightUpper)`. Return `RIGHT` when a
  lone lower door is on the left or `score>0`; return `LEFT` when a lone lower door is on the right
  or `score<0`. Otherwise use local click `(x,z)`: `RIGHT` iff
  `(stepX<0 && z<0.5) || (stepX>0 && z>0.5) || (stepZ<0 && x>0.5) || (stepZ>0 && x<0.5)`; equality
  falls to `LEFT`. `setPlacedBy` writes the copied upper state with flags `3`.
- Ordinary `DoublePlantBlock` requires a replaceable upper position below `maxY`; after the lower
  succeeds, it writes its default `half=upper` state with upper-position waterlogging copied and
  flags `3`. `SmallDripleafBlock` derives facing opposite the player's horizontal direction and
  lower waterlogging; server-side only, it writes an upper state with copied facing/waterlogging and
  flags `3`. `pitcher_plant` uses ordinary `BlockItem` (no pre-clear) but the same inherited
  two-half `setPlacedBy`. Neither `pitcher_crop` nor `tall_seagrass` has a corresponding block item;
  their multi-half creation belongs to crop/bonemeal behavior.
- `StandingAndWallBlockItem` precomputes the wall candidate, then scans
  `context.getNearestLookingDirections()` in order, skipping the direction opposite its attachment
  direction. The attachment direction chooses standing versus wall candidate; the first non-null
  candidate that survives wins, followed by one `isUnobstructed(...,CollisionContext.empty())` test.
  This bypasses generic `BlockItem#canPlace`. Signs/hanging signs, banners, torches/wall torches,
  coral fans/wall fans, skulls/heads and the copper torch use this family. Hanging wall signs add
  their own attachment test.
- `PlaceOnWaterBlockItem` (`lily_pad`, `frogspawn`) always returns `PASS` from use-on-block.
  Use-in-air raycasts source fluids, changes the hit block position to one block above while
  retaining the hit data, then invokes the generic block-item use path.
- `ScaffoldingBlockItem` redirects placement when the selected target is scaffolding. Secondary use
  follows the clicked face (opposite when the hit is inside); otherwise an upward face extends
  horizontally in player-facing direction and any other face extends upward. It walks through
  scaffolding until the first replaceable block. A horizontal walk stops after seven scaffolding
  steps; a vertical walk continues to the first non-scaffolding or bounds. Server out-of-bounds
  above `maxY` sends a build-limit message. Starting on non-scaffolding rejects when
  `ScaffoldingBlock#getDistance(level,target)==7`. Its `mustSurvive=false`; the block's later
  physics owns unsupported falling.
- `GameMasterBlockItem` returns no candidate when a non-null player lacks `canUseGameMasterBlocks`.
  `SolidBucketItem` (`powder_snow_bucket`) uses the generic transaction with its configured sound,
  then on any consuming result replaces the survival player's hand with one empty bucket;
  infinite-material players retain the original stack. Its dispenser-facing `emptyContents` is a
  separate path.

**Branches and aborts:**

Every pre-write rejection above returns without the generic success sound/event/consumption. A false
initial `setBlock` does the same. Post-write block replacement, ignored flag-`2` component writes,
ignored upper/head writes, custom-NBT denial and block-entity absence are deliberately not
transaction rollbacks. Block interaction can consume before item placement; secondary use changes
that precedence. A null player bypasses adventure and game-master player checks but still undergoes
state/collision checks.

**Constants and randomness:**

Flags are `11`, `26`, `27`, `3` and `2` in the positions above; generic shape depth is 512.
Interaction range receives padding `1.0`; hit-vector bound is strict `<1.0000001` per axis. Sound
constants are `+1`, `/2`, and `*0.8`. Scaffolding horizontal extension limit is seven. Door click
threshold is exact double `0.5`. This transaction consumes no RNG; any candidate-state RNG must be
declared by its content rule.

**Side effects:**

Packet sequence acknowledgement; action timer reset; messages; hit-block interaction; one or more
ordered state writes and all `BLK-UPDATE-001` consequences; BE NBT/components/editor; `setPlacedBy`;
`PLACED_BLOCK`, `ITEM_USED_ON_BLOCK`, `ANY_BLOCK_USE` and item-used stat triggers at their stated
layers; sound; `BLOCK_PLACE`; stack decrement/transformation; swing selection; and final
two-position correction packets.

**Gates:**

Client loaded, enabled held item and hit block, distance/hit-vector validity, build range, spawn
protection, teleport correction, `mayInteract`, spectator/menu path, secondary-use precedence,
cooldown, adventure `can_place_on`, placed-block feature, replaceability, candidate existence,
survival, collision and subtype permissions. Difficulty and gamerules do not change generic
placement admission.

**Boundary cases and quirks:**

Placement is not atomic. Double-high pre-clear can survive a failed lower write; all second-half
write results are ignored; a successful initial write whose callbacks replace it still consumes and
reports success. Upper/head collision is not generically checked. The two final correction packets
cover only hit and face-adjacent positions, not an arbitrarily redirected scaffolding target or a
bed head. Infinite-material restoration occurs around item dispatch in addition to
`ItemStack.consume` skipping shrink. State-component application occurs after placement validation
and before `setPlacedBy`.

**Evidence:**

`Confirmed`; `OFF-SERVER-001`. Anchors:
`net.minecraft.server.network.ServerGamePacketListenerImpl#handleUseItemOn(net.minecraft.network.protocol.game.ServerboundUseItemOnPacket)`,
`net.minecraft.server.level.ServerPlayerGameMode#useItemOn(net.minecraft.server.level.ServerPlayer,net.minecraft.world.level.Level,net.minecraft.world.item.ItemStack,net.minecraft.world.InteractionHand,net.minecraft.world.phys.BlockHitResult)`,
`net.minecraft.world.item.ItemStack#useOn(net.minecraft.world.item.context.UseOnContext)`,
`net.minecraft.world.item.context.BlockPlaceContext#getClickedPos()`,
`net.minecraft.world.item.context.BlockPlaceContext#canPlace()`,
`net.minecraft.world.item.BlockItem#place(net.minecraft.world.item.context.BlockPlaceContext)`,
`net.minecraft.world.item.BlockItem#placeBlock(net.minecraft.world.item.context.BlockPlaceContext,net.minecraft.world.level.block.state.BlockState)`,
`net.minecraft.world.item.DoubleHighBlockItem#placeBlock(net.minecraft.world.item.context.BlockPlaceContext,net.minecraft.world.level.block.state.BlockState)`,
`net.minecraft.world.item.BedItem#placeBlock(net.minecraft.world.item.context.BlockPlaceContext,net.minecraft.world.level.block.state.BlockState)`,
`net.minecraft.world.item.StandingAndWallBlockItem#getPlacementState(net.minecraft.world.item.context.BlockPlaceContext)`,
`net.minecraft.world.item.ScaffoldingBlockItem#updatePlacementContext(net.minecraft.world.item.context.BlockPlaceContext)`,
`net.minecraft.world.level.block.BedBlock#getStateForPlacement(net.minecraft.world.item.context.BlockPlaceContext)`,
`net.minecraft.world.level.block.DoorBlock#getStateForPlacement(net.minecraft.world.item.context.BlockPlaceContext)`,
`net.minecraft.world.level.block.DoorBlock#getHinge(net.minecraft.world.item.context.BlockPlaceContext)`,
and
`net.minecraft.world.level.block.DoublePlantBlock#setPlacedBy(net.minecraft.world.level.Level,net.minecraft.core.BlockPos,net.minecraft.world.level.block.state.BlockState,net.minecraft.world.entity.LivingEntity,net.minecraft.world.item.ItemStack)`.

**Test vectors:**

(1) Exercise every admission gate and both interaction-precedence paths; assert sequence
acknowledgement and final hit/adjacent corrections. (2) Place into replaceable hit versus adjacent
target, with survival failure, lower collision, upper-only collision, disabled features and
adventure predicates. (3) Instrument initial `onPlace` to replace the candidate; assert success
sound/event/consumption but no components/BE callback/placed criterion. Separately replace the
flag-`2` patched state and assert later callbacks retain the requested local state. (4) For
double-high items, force upper pre-clear then lower failure and force second-half failure; assert
the exact partial states and success branch. (5) Place beds at border, height and head-entity
boundaries; compare flags `26` then `3`. (6) Cover all door score/adjacent-door/click quadrants
including exact `0.5`, initial power and upper-write ordering. (7) Cover waterlogged ordinary
plants, small-dripleaf client/server upper prediction, and plain-item pitcher plant. (8) Query and
place one normal/special/boundary ID from every explicit item catalog family, including sign editor
suppression by custom NBT, water-surface hit redirection, horizontal scaffolding distances 6/7,
vertical extension, game-master denial and powder-snow bucket transformation. `EXP-BLK-001` is the
executable conformance matrix for these cases.
