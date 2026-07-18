# Player leaf rules

## Leaf rule `PLY-MOVE-001` — Server movement validates requested position against authoritative collision

**Parent:** `PLY-001`, `PLY-002`, `PLY-003`  
**Applies when:** The server processes player input/movement and advances the player entity.  
**Authoritative state:** Position, rotation, velocity, pose, on-ground/fall state, abilities, effects, attributes, collision box, chunk tickets and pending correction/teleport acknowledgement.  
**Transition and ordering:** Convert current input/abilities/effects into acceleration; apply environment movement mode; integrate through collision clipping on each axis using the authoritative world; update on-ground, fall distance, pose and contacted blocks; validate client-requested displacement against legal movement and pending teleport state; accept or issue correction. Anchor mechanics: `net.minecraft.world.entity.Entity#move(net.minecraft.world.entity.MoverType,net.minecraft.world.phys.Vec3)` and `net.minecraft.world.entity.player.Player#travel(net.minecraft.world.phys.Vec3)`.  
**Branches and aborts:** Pending teleport acknowledgement; sleeping/riding/spectating; flying; swimming/crawling; climbing; in fluid; elytra; collision; unloaded destination; excessive/invalid coordinate request. A correction rejects the unauthorized result but does not rewind unrelated server ticks.  
**Constants and randomness:** Positions/velocity use doubles; collision comparisons use source epsilon rules. Attribute speeds, friction, gravity and effect modifiers are applied in source order. Movement has no ordinary RNG. Exact floating-point/rounding vectors are `EXP-PLY-001`.  
**Side effects:** Chunk tracking/tickets, fall damage, step/landing sounds, block contact callbacks, game events, exhaustion, statistics, vehicle/passenger movement and client correction.  
**Gates:** Abilities/game mode, pose clearance, effects, attributes, collision shapes, world border, chunk availability, flying permission and pending correction state.  
**Boundary cases and quirks:** `onGround` is derived from collision, not trusted from the client. Axis clipping order and epsilon comparisons affect edge sliding. Client prediction can make an illegal step briefly visible before correction.  
**Evidence:** `Confirmed` authority and collision model; numeric parity `Implementation blocker`; `OFF-SERVER-001`, `OFF-CLIENT-001`; locators above; `EXP-PLY-001`.  
**Test vectors:** Walk into a full cube, slab edge and corner; step-height boundary; jump while ceiling-constrained; move across chunk unload boundary; send false on-ground and oversized displacement; compare bit-level position sequences.

## Leaf rule `PLY-INTERACT-001` — Use selects entity, block, and item paths with explicit pass/fail semantics

**Parent:** `PLY-004`, `PLY-005`  
**Applies when:** A player presses use and a server-side hit result is processed for either hand.  
**Authoritative state:** Eye/rotation/reach, hit target, hand stack, cooldown, player mode/permissions, target state/entity, interaction result and inventory.  
**Transition and ordering:** Determine the targeted entity or block from the appropriate reach/clip context; for entity use, invoke interaction-at then general interaction as defined; for block use, apply spectator/container and secondary-use rules, block interaction, then item-on-block if prior result passes; if no target consumes the action, invoke item use in air. Stop at the first result whose semantics consume/definitively fail that path. Anchors: `net.minecraft.server.level.ServerPlayerGameMode#useItemOn(net.minecraft.server.level.ServerPlayer,net.minecraft.world.level.Level,net.minecraft.world.item.ItemStack,net.minecraft.world.InteractionHand,net.minecraft.world.phys.BlockHitResult)` and `net.minecraft.world.item.Item#use(net.minecraft.world.level.Level,net.minecraft.world.entity.player.Player,net.minecraft.world.InteractionHand)`.  
**Branches and aborts:** Out of reach; occluded/changed target; spectator; cooldown; disabled feature; secondary-use bypass; block returns success/consume/fail/pass; stack changes during callback; client target disagrees. Each hand is its own attempt under caller ordering.  
**Constants and randomness:** Reach comes from current attributes/game mode and geometric clipping. Generic dispatch consumes no RNG; selected item/block may. Hit face/vector must be preserved with double precision until block context derives discrete directions.  
**Side effects:** Block/entity/item state, inventory mutation, menu opening, cooldown, statistic/criterion/game event, swing decision, sound/particles and server correction.  
**Gates:** Reach, line of sight where required, game mode, feature flags, permissions/adventure predicates, cooldown, hand, sneaking/secondary-use, target state and interaction result.  
**Boundary cases and quirks:** “Pass” continues dispatch and is not success. Client swing does not prove server acceptance. A callback may replace the held stack, so consumption must use returned/current stack semantics rather than a stale copy.  
**Evidence:** `Confirmed`; `OFF-SERVER-001`; locators above; ambiguous dual-hand/secondary-use sequence `EXP-PLY-002`.  
**Test vectors:** Interactive block while holding a placeable item, sneaking variant, item cooldown, target removed before server receipt, off-hand fallback, spectator container, and callback that replaces its own stack.

## Leaf rule `PLY-BREAK-001` — Survival breaking is a server-validated progress state machine

**Parent:** `BLK-002`, `PLY-006`  
**Applies when:** A non-creative player starts, continues, aborts, or completes block destruction.  
**Authoritative state:** Active destroy position/start tick/progress, target state, tool stack/components, attributes/effects, environment penalties, permissions, game mode and block entity.  
**Transition and ordering:** Start validates reach/permission and calls attack/start callbacks; record target and start tick; derive per-tick destroy progress from state/tool/player conditions; emit staged crack state; on stop/completion revalidate same state and sufficient progress; remove through the game-mode destroy path; invoke player/tool mining callbacks and loot/drop logic in source order. Anchor: `net.minecraft.server.level.ServerPlayerGameMode#handleBlockBreakAction(net.minecraft.core.BlockPos,net.minecraft.network.protocol.game.ServerboundPlayerActionPacket$Action,net.minecraft.core.Direction,int,int)` and `net.minecraft.server.level.ServerPlayerGameMode#destroyBlock(net.minecraft.core.BlockPos)`.  
**Branches and aborts:** Adventure/command restrictions; out of reach/build height; target changed; abort action; instant creative branch; unbreakable state; progress insufficient; tool/item mutation; block removal rejected. Failed completion restores authoritative state and clears/updates crack animation.  
**Constants and randomness:** Progress is floating-point accumulation derived from destroy speed and harvest divisor; haste/fatigue, underwater and airborne penalties apply in source order. Loot RNG is consumed only after successful destruction and loot context construction.  
**Side effects:** Crack animation, state removal, block-entity removal, tool durability/components, exhaustion/statistics/criteria, drops/XP, neighbor updates, game event, sound and correction.  
**Gates:** Game mode, permissions/adventure predicates, reach, block hardness/feature, tool correctness, effects, fluid/ground status, current state identity and gamerules controlling drops.  
**Boundary cases and quirks:** Client crack progress is presentation; server tick/start state decides completion. Switching tools/targets invalidates or changes later validation. Creative destruction does not reuse survival timing/drops.  
**Evidence:** `Confirmed`; `OFF-SERVER-001`; locators above; numeric progress matrix `EXP-PLY-003`.  
**Test vectors:** Exact tick-to-break boundaries across tool/effect/air/water states; swap tool mid-break; target state changes but block ID remains; abort/restart; creative sword-like restrictions; out-of-reach completion packet.
