# Player mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `PLY-INTERACT-001` — Use selects entity, block, and item paths with explicit pass/fail semantics

**Parent:** `PLY-004`, `PLY-005`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceInconclusive` — hit ties, reach boundaries, both-hand dispatch, and every `InteractionResult`
branch remain unexpanded.

**Applies when:**

A player presses use and a server-side hit result is processed for either hand.

**Authoritative state:**

Eye/rotation/reach, hit target, hand stack, cooldown, player mode/permissions, target state/entity,
interaction result and inventory.

**Transition and ordering:**

Determine the targeted entity or block from the appropriate reach/clip context; for entity use,
invoke interaction-at then general interaction as defined; for block use, apply spectator/container
and secondary-use rules, block interaction, then item-on-block if prior result passes; if no target
consumes the action, invoke item use in air. Stop at the first result whose semantics
consume/definitively fail that path. Anchors:
`net.minecraft.server.level.ServerPlayerGameMode#useItemOn(net.minecraft.server.level.ServerPlayer,net.minecraft.world.level.Level,net.minecraft.world.item.ItemStack,net.minecraft.world.InteractionHand,net.minecraft.world.phys.BlockHitResult)`
and
`net.minecraft.world.item.Item#use(net.minecraft.world.level.Level,net.minecraft.world.entity.player.Player,net.minecraft.world.InteractionHand)`.

**Branches and aborts:**

Out of reach; occluded/changed target; spectator; cooldown; disabled feature; secondary-use bypass;
block returns success/consume/fail/pass; stack changes during callback; client target disagrees.
Each hand is its own attempt under caller ordering.

**Constants and randomness:**

Reach comes from current attributes/game mode and geometric clipping. Generic dispatch consumes no
RNG; selected item/block may. Hit face/vector must be preserved with double precision until block
context derives discrete directions.

**Side effects:**

Block/entity/item state, inventory mutation, menu opening, cooldown, statistic/criterion/game event,
swing decision, sound/particles and server correction.

**Gates:**

Reach, line of sight where required, game mode, feature flags, permissions/adventure predicates,
cooldown, hand, sneaking/secondary-use, target state and interaction result.

**Boundary cases and quirks:**

“Pass” continues dispatch and is not success. Client swing does not prove server acceptance. A
callback may replace the held stack, so consumption must use returned/current stack semantics rather
than a stale copy.

**Evidence:**

`Confirmed`; `OFF-SERVER-001`; locators above; ambiguous dual-hand/secondary-use sequence
`EXP-PLY-002`.

**Test vectors:**

Interactive block while holding a placeable item, sneaking variant, item cooldown, target removed
before server receipt, off-hand fallback, spectator container, and callback that replaces its own
stack.
