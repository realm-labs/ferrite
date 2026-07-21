# Player mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `PLY-INTERACT-001` — Use selects entity, block, and item paths with explicit pass/fail semantics

**Parent:** `PLY-004`, `PLY-005`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — client pick/attack/use order, both-hand fallthrough, result algebra, prediction
and server admission/callback order are fixed below; concrete target callbacks remain in their
catalog leaf owners.

**Applies when:**

A player presses use and a server-side hit result is processed for either hand.

**Authoritative state:**

Eye/rotation/reach, hit target, hand stack, cooldown, player mode/permissions, target state/entity,
interaction result and inventory.

**Transition and ordering:**

The ordinary client pick takes `max(blockInteractionRange, entityInteractionRange)`, clips blocks
from interpolated eye along view, and if a block is hit shortens the entity ray and maximum squared
distance to that exact block hit. It searches pickable entities in the swept camera AABB inflated by
one. An entity wins only when its hit squared distance is strictly less than the block distance, so
an exact tie keeps the block. The selected result is converted to miss unless its location is
strictly closer than its own block/entity range. An active item's `ATTACK_RANGE` component gets the
first custom hit attempt; its block result is still filtered by ordinary block range, and only a
null/miss falls back to ordinary pick.

Attack uses main hand only. Positive miss delay, null hit, hands-busy, disabled item or
`cannotAttackWithItem(...,0)` abort. Spectator entity hit sends spectate; other hits send spectator
no-action. A piercing-weapon component takes its dedicated attack and swing path. Ordinary entity
hit attacks only when absent custom range or that component admits the exact hit; block hit starts
break if the current block is nonair; block becoming air during start returns the instant-attack
flag. Block-air falls through to miss. Miss installs delay 10 when the game mode uses miss time and
resets attack strength. Every admitted ordinary branch swings main hand, including a custom-range
rejection; server attack independently resolves the current entity, border, item attack range with
buffer 3, excludes item/xp/self and nonattackable arrow targets (invalid target disconnects), then
checks feature and `cannotAttackWithItem(...,5)` before authoritative attack.

Use is suppressed while destroying or hands-busy and loads right-click delay 4. Iterate
`InteractionHand.values()` in main/off order. A feature-disabled held stack returns from the entire
operation, not merely that hand. For an entity hit, being outside the world border returns; when in
strict client entity range, send the entity/hand/relative-hit/shift packet before local
`Player#interactOn`. Only `Success` ends the use loop, swinging locally only for client swing source;
non-success continues into air-item use for that same hand. Server entity admission requires loaded,
current entity, border and strict bounding-box range with buffer 3, installs packet secondary-action
state, checks current hand feature, then invokes the same interaction. A success triggers the entity
criterion with the pre-callback stack only when its item context says item interaction, and server
swing source broadcasts the swing.

`Player#interactOn` in spectator mode opens an entity menu when available but returns pass. Otherwise
copy the held stack, invoke `entity.interact(player,hand,relativeLocation)`, and return immediately on
any consuming success, restoring count for infinite-material players only when the same stack object
remains held and its count fell. A nonconsuming entity result continues only when the stack is
nonempty and target is living: use a copy for infinite materials, invoke
`stack.interactLivingEntity`, emit entity-interact game event and clear an emptied noncreative hand
on consuming success. Otherwise return pass.

For a block hit, the client always enters a sequenced block prediction and sends use-on. Outside
client world border returns fail before prediction. Spectator prediction returns consume. Otherwise,
unless secondary use is active while either hand is nonempty, require the block feature, call
`state.useItemOn(currentHandStack,...)`, and return if it consumes. Exactly
`TryEmptyHandInteraction` from that callback and main hand then invokes `useWithoutItem`, returning
if it consumes. A remaining nonempty, noncooldown stack calls `useOn`; infinite materials restores
only its count. Empty/cooldown returns pass. Server block callback order is identical, except
spectator opens the current state's menu and consumes or passes, and consuming block/item callbacks
fire their respective criteria with the pre-callback stack.

Server use-on first cumulatively acknowledges the packet sequence, then requires loaded, current
hand feature, block-range with buffer 1, each hit coordinate strictly within 1.0000001 of block
center, build-height, no spawn protection, no pending teleport and `mayInteract`. It supplies build-
limit/protection refusal messages by the locked branch, invokes the transaction, performs server-
swing success, then always sends current block updates for target and hit-face neighbor after an
in-range processed-height branch. Failed geometric admission sends no callback.

After target handling, a nonempty held stack runs use-in-air through another sequenced prediction.
The client sends even when cooldown makes its local result pass. Nonspectator/noncooldown invokes
`stack.use`; success may carry a transformed stack, otherwise current hand is retained, and a
different result object replaces the hand. Server first acknowledges, applies carried yaw/pitch
before use, then performs the analogous current-stack call. It avoids inventory resync only when the
same object/count/damage remains and use duration is nonpositive; a fail whose result has positive
duration while the player did not begin using also returns without replacement. Otherwise it
installs/clears transformed stack and sends full inventory data when not actively using.

`InteractionResult.Success` alone consumes; its swing source is none/client/server and its item
context records item participation plus an optional transformed stack. `Fail`, `Pass` and
`TryEmptyHandInteraction` do not consume generically. Block-use treats fail as terminal in the
outer client loop, entity-use does not; `TryEmptyHandInteraction` has its special meaning only in
the main-hand block callback position described above. Anchors:
`net.minecraft.server.level.ServerPlayerGameMode#useItemOn(net.minecraft.server.level.ServerPlayer,net.minecraft.world.level.Level,net.minecraft.world.item.ItemStack,net.minecraft.world.InteractionHand,net.minecraft.world.phys.BlockHitResult)`
and
`net.minecraft.world.item.Item#use(net.minecraft.world.level.Level,net.minecraft.world.entity.player.Player,net.minecraft.world.InteractionHand)`.

**Branches and aborts:**

Null/miss/air target; attack delay/hands busy/destroying; spectator; border/range/geometric/build/
spawn/teleport/adventure admission; cooldown/feature; secondary-use bypass; every result subtype and
swing source; current stack/target replacement; callback transforms/empties/begins use. Each hand is
its own attempt, but a disabled stack, target border failure, block fail or success terminates the
outer client operation as stated.

**Constants and randomness:**

Ordinary client range comparisons are strict; server entity/attack admission adds 3 and block
admission adds 1 before its strict squared-distance comparison. Hit-relative block component limit
is strict 1.0000001 on each axis. Generic dispatch consumes no RNG; selected callbacks may. The
protocol's entity relative vector is low precision, while block hit location remains double on wire.

**Side effects:**

Block/entity/item state, inventory mutation, menu opening, cooldown, statistic/criterion/game event,
swing decision, sound/particles and server correction.

**Gates:**

Reach, line of sight where required, game mode, feature flags, permissions/adventure predicates,
cooldown, hand, sneaking/secondary-use, target state and interaction result.

**Boundary cases and quirks:**

Exact entity/block hit ties select block; exact range equality filters to miss. Entity non-success
can fall through to item use, while block fail cannot. `TRY_WITH_EMPTY_HAND` does not require an
actually empty main hand. Client swing never proves server acceptance. Entity packet secondary
action replaces server shift state before callback. A callback may replace the held stack, so
consumption uses returned/current object semantics rather than a stale copy.

**Evidence:**

`Confirmed`; `OFF-SERVER-001`, `OFF-CLIENT-001`; `LocalPlayer#getHitResult/#pick`,
`Minecraft#startAttack/#startUseItem`, `MultiPlayerGameMode`, `ServerGamePacketListenerImpl`
attack/interact/use handlers, `ServerPlayerGameMode`, `Player#interactOn`, `InteractionResult`;
`EXP-PLY-002`.

**Test vectors:**

Entity/block distance tie and exact range; custom attack range hit/miss; every attack abort/piercing/
invalid target; main-hand entity pass/fail/success into air use and offhand; block every result,
empty-hand marker, secondary-use and feature early return; spectator entity/block; server distance
buffers and hit-coordinate boundaries; prediction ACK/correction; cooldown; callback same/new/
empty/transformed stack, count/damage/use-duration and each swing/item-context combination.
