# Block leaf rules

## Leaf rule `BLK-PLACE-001` — Block placement is a validate–derive–commit transaction

**Parent:** `BLK-001`, `BLK-002`  
**FidelityClass:** `ExactObservableBehavior`  <br>
**EvidenceStatus:** `Confirmed`  <br>
**SourceConclusion:** `SourceInconclusive` — placement families with multi-position commits, replacement callbacks, and per-block failure rollback still require separate source-specified slices.  <br>
**Applies when:** A player or dispenser-like source uses a block item against a target context.  
**Authoritative state:** Server inventory, placement context, target/adjacent block and fluid states, build-height/world-border rules, collision entities, permissions, and the resulting block state.  
**Transition and ordering:** Select the replace target from the hit context; reject if use/feature/permission/world bounds fail; ask the block for its placement state; validate survival and unobstructed collision; commit with the placement update flags; invoke placement callbacks and block-entity component transfer; consume one item unless an exemption applies; emit advancement/stat/game-event/sound results. `net.minecraft.world.item.BlockItem#useOn(net.minecraft.world.item.context.UseOnContext)`, `net.minecraft.world.item.BlockItem#place(net.minecraft.world.item.context.BlockPlaceContext)`, and `net.minecraft.world.level.block.Block#getStateForPlacement(net.minecraft.world.item.context.BlockPlaceContext)` are the control-flow anchors.  
**Branches and aborts:** Replaceable target versus adjacent face; no placement state; state cannot survive; collision obstruction; height/border denial; feature disabled; commit returns false; or post-commit state differs. An abort does not consume the item or emit a successful placement result. Waterlogging and orientation are state derivation, not a later cosmetic patch.  
**Constants and randomness:** Coordinates are integer block positions; hit vectors and look direction feed state derivation. Generic placement consumes no RNG. Per-block data values come from `mc-ref query block <id>` and item components.  
**Side effects:** State write, neighbor/shape notifications according to flags, block-entity creation/data transfer, inventory decrement, placement sound, game event, statistic and criteria triggers. Client prediction may briefly display placement but server state wins.  
**Gates:** Adventure/build permissions, world border, build height, feature flags, spectator/creative semantics, collision checks, chunk writability, and interaction cancellation.  
**Boundary cases and quirks:** Multi-block placements must roll back or avoid partial commit according to their special rule. A block state returned for one position must be revalidated against the actual committed neighborhood. Block items can be special-use items and need not reach generic placement.  
**Evidence:** `Confirmed`; `OFF-SERVER-001`; source locators above; family mapping in [catalog](../catalog/README.md). Multi-block rollback is `EXP-BLK-001`.  
**Test vectors:** Place into replaceable grass versus its adjacent face; place at max build height; obstruct with an entity; waterlog a supported block; race a neighbor change between prediction and commit and assert correction plus no duplicate consumption.

## Leaf rule `BLK-UPDATE-001` — State writes and neighbor/shape propagation are distinct operations

**Parent:** `BLK-003`, `BLK-004`, `BLK-005`, `BLK-007`  
**FidelityClass:** `ExactObservableBehavior`  <br>
**EvidenceStatus:** `Confirmed`  <br>
**SourceConclusion:** `SourceInconclusive` — flag combinations, nested collector insertion, and block-event replacement branches remain unexpanded.  <br>
**Applies when:** Gameplay attempts to replace a block state or explicitly notify neighbors.  
**Authoritative state:** Level block-state storage, update flags/recursion budget, six-neighbor states, block entities, scheduled tick queues, and block-event queue.  
**Transition and ordering:** Compare old and new state; write storage when different; remove/create or retain block entity as appropriate; perform client notification, neighbor notification, comparator output update, known-shape handling, and indirect shape propagation according to flags. Neighbor callbacks observe the state installed before their callback. Shape propagation can derive a replacement state and recurse under the update budget. Anchors: `net.minecraft.world.level.Level#setBlock(net.minecraft.core.BlockPos,net.minecraft.world.level.block.state.BlockState,int,int)`, `net.minecraft.world.level.Level#neighborChanged(net.minecraft.core.BlockPos,net.minecraft.world.level.block.Block,net.minecraft.world.level.redstone.Orientation)`, and `net.minecraft.world.level.block.state.BlockBehaviour$BlockStateBase#updateShape(net.minecraft.world.level.LevelReader,net.minecraft.world.level.ScheduledTickAccess,net.minecraft.core.BlockPos,net.minecraft.core.Direction,net.minecraft.core.BlockPos,net.minecraft.world.level.block.state.BlockState,net.minecraft.util.RandomSource)`.  
**Branches and aborts:** Identical-state writes may avoid most work. Flags suppress selected channels; chunk/debug/world bounds may reject storage. A shape result equal to current state stops that branch. Recursion budget exhaustion stops further propagation rather than overflowing the call stack.  
**Constants and randomness:** Update flags and maximum recursion are integer API inputs; callers must preserve their exact combinations. Generic propagation is ordered, not randomized, although a block's shape callback receives a random source and may consume it.  
**Side effects:** Neighbor callbacks, scheduled block/fluid work, comparator recalculation, block-entity lifecycle, drops, game events, client state packets, and secondary state writes. These are not interchangeable and must not be collapsed into a single unordered event bus.  
**Gates:** Update flags, chunk availability, recursion budget, state survival, and block-specific direction rules. `doTileDrops` gates drops, not the underlying state notification.  
**Boundary cases and quirks:** “Neighbor update” and “neighbor shape update” have different receivers and parameters. Update suppression intentionally creates states unobtainable through ordinary play and must remain possible for commands/worldgen callers using those flags.  
**Evidence:** `Confirmed`; `OFF-SERVER-001`; locators above. Direction order and nested replacement trace: `EXP-BLK-002`.  
**Test vectors:** Write the same state twice; compare callbacks. Toggle client-update and neighbor-update flags independently. Remove support from a six-neighbor structure and record callback direction and replacement sequence. Force recursion budget exhaustion and assert bounded work.

## Leaf rule `BLK-FALL-001` — A falling block transfers block state into an entity and back

**Parent:** `BLK-006`  
**FidelityClass:** `ExactObservableBehavior`  <br>
**EvidenceStatus:** `Cross-checked`  <br>
**SourceConclusion:** `SourceInconclusive` — subtype constants and several landing/unload branches remain owned by `EXP-BLK-003`.  <br>
**Applies when:** A falling-block family state is scheduled/ticked and the block below satisfies that block's free-fall predicate.  
**Authoritative state:** Origin block state and block entity data, falling entity position/velocity/time, hurt/drop flags, destination replaceability/support, and gamerules.  
**Transition and ordering:** On the block tick, validate fall space; spawn a `net.minecraft.world.entity.item.FallingBlockEntity` carrying the state; replace the origin according to the block's start-falling behavior; tick gravity and collision; at landing, choose placement, special transformation, or item drop; restore supported block state and transferable block-entity data only on successful placement. The block schedule anchor is `net.minecraft.world.level.block.FallingBlock#tick(net.minecraft.world.level.block.state.BlockState,net.minecraft.server.level.ServerLevel,net.minecraft.core.BlockPos,net.minecraft.util.RandomSource)`.  
**Branches and aborts:** Below not free; spawn rejected; entity removed; exceeds world/time limits; destination cannot be replaced/survived; concrete powder contacts water; anvil damage branch; dragon egg/scaffolding special conditions; drop disabled. A failed landing never overwrites a nonreplaceable destination.  
**Constants and randomness:** Entity motion uses per-tick gravity and drag from the entity implementation. Anvil degradation uses RNG at its impact branch. Exact values are subtype-owned and must be symbol/data checked before implementation; `EXP-BLK-003` locks landing timing and anvil RNG consumption.  
**Side effects:** Origin removal, falling entity spawn/movement/removal, landing state placement or item entity, block-entity restoration, impact damage for configured types, sounds/events/particles and neighbor updates at origin/destination.  
**Gates:** Origin/destination chunk ticking, build bounds, entity spawning, `doEntityDrops`, block subtype, and destination survival.  
**Boundary cases and quirks:** The falling entity carries the original state rather than repeatedly querying origin. Moving through an unloaded boundary must not duplicate both block and entity. Piston movement and instant-fall worldgen paths are distinct callers.  
**Evidence:** `Confirmed` state-machine shape; `Cross-checked` subtype constants; `OFF-SERVER-001`; `FallingBlock#tick`; `FallingBlockEntity`; `EXP-BLK-003`.  
**Test vectors:** Remove support and observe origin-to-entity transition; land on replaceable and solid targets; unload during transit and reload; drop with entity drops disabled; place a block entity-bearing falling state and verify data transfer exactly once.
