# 05 — Player Movement, Collision, Targeting, and Interaction

This page describes server gameplay results. See `CLI-*` for client input, prediction, and correction. The two cross-reference one another without merging ownership.

## `PLY-001` Input forms movement intent; the server entity owns movement truth

- **FidelityClass:** `ExactObservableBehavior`
- **Evidence status:** `Confirmed` for input shaping, automatic-jump detection, ordinary ground/air dynamics and packet validation; special movement modes remain open
- **Primary evidence:** `OFF-CLIENT-001`; `OFF-SERVER-001`; `net.minecraft.client.player.KeyboardInput#tick()`; `net.minecraft.client.player.LocalPlayer#aiStep()`; `net.minecraft.client.player.LocalPlayer#tick()`; `net.minecraft.world.entity.player.Player#tick()`; `net.minecraft.world.entity.LivingEntity#travel(net.minecraft.world.phys.Vec3)`; `COM-WIKI-PLY-001`
- **Applies when:** A client owns a local player and the user changes directional, jump, sneak, sprint, or flight input.
- **Behavior and timing:** `PLY-INPUT-001` maps seven sampled movement booleans through conflict cancellation, float normalization, posture/item modifiers, sprint/flight transitions and independently cadenced intent/action messages. `PLY-MOVE-001` then specifies ordinary dynamics through jump, acceleration, collision, gravity and drag. After actual movement, `PLY-AUTOJUMP-001` probes two look-ahead lines through ordered entity/block collision AABBs and may schedule a synthetic jump for the next input pass. `PLY-MOVE-VALIDATE-001` separately specifies coordinate/status message selection, server collision probing and teleport convergence. OS/focus/event-to-key state belongs to `CLI-PREDICT-001`.
- **Boundaries and quirks:** Shift-derived movement slowdown uses the previous input sample while tail pose selection uses the current sample. Cardinal intent remains `0.98f` before travel while the square remap restores an unmodified diagonal to unit magnitude. Auto-jump does not require a horizontal collision, uses raw input as its slow-motion fallback, retains the last intersecting candidate rather than the highest and rejects exactly half-block rises. Input, sprint, ability and coordinate messages have independent change detectors and cadence. Packet loss, latency and client FPS do not change server gameplay tick progression. More than five coordinate packets changes the anti-cheat multiplier to one; finite vertical moved-wrongly residual is discarded by an OR-condition quirk; pending movement may rotate but not translate the player.
- **Verification owner (`PLY-INPUT-001`, `PLY-AUTOJUMP-001`, `PLY-MOVE-001`, `PLY-MOVE-VALIDATE-001`; `EXP-PLY-001`, `EXP-PLY-005`, `EXP-PLY-006`, `EXP-PLY-007`):** Input shaping, auto-jump scheduling/consumption, ordinary dynamics and coordinate validation/convergence are all source-specified; experiments are regression probes. The client event-to-KeyMapping boundary remains intentionally owned by `CLI-PREDICT-001`.

## `PLY-002` Collision clips displacement by axis and shape

- **FidelityClass:** `ExactObservableBehavior`
- **Evidence status:** `Confirmed`
- **Primary evidence:** `OFF-SERVER-001`; `net.minecraft.world.entity.Entity#move(net.minecraft.world.entity.MoverType,net.minecraft.world.phys.Vec3)`; `net.minecraft.world.entity.Entity#collide(net.minecraft.world.phys.Vec3)`; `net.minecraft.world.entity.Entity#collideBoundingBox(net.minecraft.world.entity.Entity,net.minecraft.world.phys.Vec3,net.minecraft.world.phys.AABB,net.minecraft.world.level.Level,java.util.List)`
- **Applies when:** A player or other entity requests nonzero displacement outside a branch that bypasses ordinary collision.
- **Behavior and timing:** `PLY-COLLISION-001` specifies swept-shape collection, Y-first/dominant-horizontal axis clipping, ascending step-candidate selection, position recording, collision/support flags, restitution and post-move emission/speed effects. Concrete registry content supplies collision shapes and block properties without changing the generic transaction.
- **Boundaries and quirks:** Shape clipping and collision flags intentionally use different epsilons. Equal absolute X/Z selects X before Z; step-up accepts the first ascending height that strictly improves horizontal squared displacement rather than a globally best candidate.
- **Verification owner (`PLY-COLLISION-001`; `EXP-PLY-001`):** The source-specified transaction owns axis order, epsilons, edge backoff, step selection, simultaneous shapes, piston restriction and bounce state.

## `PLY-003` Ground, water, lava, fall flying, and flight share an entry point but not dynamics

- **FidelityClass:** `ExactObservableBehavior`
- **Evidence status:** `Confirmed`
- **Primary evidence:** `OFF-SERVER-001`; `net.minecraft.world.entity.LivingEntity#travel(net.minecraft.world.phys.Vec3)`; `net.minecraft.world.entity.LivingEntity#jumpFromGround()`; `net.minecraft.world.entity.LivingEntity#travelInAir(net.minecraft.world.phys.Vec3)`; `net.minecraft.world.entity.LivingEntity#travelInFluid(net.minecraft.world.phys.Vec3)`; `net.minecraft.world.entity.player.Player#aiStep()`; `COM-WIKI-PLY-001`
- **Applies when:** A living entity advances velocity from input and its current medium.
- **Behavior and timing:** `travel` dispatches ordinary ground/air to `PLY-MOVE-001` and special fluid/swimming/fall-flying/ability-flight modes to `PLY-MOVE-SPECIAL-001`; all colliding modes reuse `PLY-COLLISION-001`. The ordinary leaf fixes jump cooldown, input normalization, friction acceleration, gravity/effects and drag order.
- **Boundaries and quirks:** Crossing a medium boundary in one tick, eye-in-fluid versus bounding-box-in-fluid, swimming pose, elytra launch/landing and ability-flight transitions remain explicit special-mode work and may not inherit ordinary constants.
- **Verification owner (`PLY-MOVE-001`, `PLY-MOVE-SPECIAL-001`; `EXP-PLY-001`, `EXP-PLY-004`):** Ordinary dynamics are source-specified; the special-mode leaf owns the remaining tick-by-tick trajectory and side-effect matrix.

## `PLY-004` View targeting compares block and entity hits along the camera ray

- **FidelityClass:** `ExactObservableBehavior`
- **Evidence status:** `Confirmed`
- **Primary evidence:** `OFF-CLIENT-001`; `OFF-SERVER-001`; `net.minecraft.client.Minecraft#pick(float)`; `net.minecraft.world.entity.projectile.ProjectileUtil#getHitResultOnViewVector(net.minecraft.world.entity.Entity,java.util.function.Predicate,double)`; `net.minecraft.world.entity.player.Player#blockInteractionRange()`; `net.minecraft.world.entity.player.Player#entityInteractionRange()`
- **Applies when:** The client refreshes its crosshair target or prepares attack/use interaction.
- **Behavior and timing:** It clips block shapes from the eye/camera along the view vector and ray-tests expanded entity boxes within entity interaction range. The nearest eligible result becomes a miss, block hit, or entity hit. The server still validates against its own position, range, and target state.
- **Boundaries and quirks:** Block outline/collision/interaction shape, fluid mode, entity pick radius, passenger relations, and reach attributes change candidates. Integer-block DDA without entity-distance comparison is insufficient.
- **Verification owner (`PLY-INTERACT-001`; `EXP-PLY-002`):** Lock exact ties, the eye starting inside a shape, just-over-reach positions, and moving-target client/server disagreement.

## `PLY-005` Hit type and InteractionResult govern attack/use priority

- **FidelityClass:** `ExactObservableBehavior`
- **Evidence status:** `Cross-checked`
- **Primary evidence:** `OFF-CLIENT-001`; `OFF-SERVER-001`; `net.minecraft.client.Minecraft#startAttack()`; `net.minecraft.client.Minecraft#startUseItem()`; `net.minecraft.server.level.ServerPlayerGameMode#useItemOn(net.minecraft.server.level.ServerPlayer,net.minecraft.world.level.Level,net.minecraft.world.item.ItemStack,net.minecraft.world.InteractionHand,net.minecraft.world.phys.BlockHitResult)`; `net.minecraft.server.level.ServerPlayerGameMode#useItem(net.minecraft.server.level.ServerPlayer,net.minecraft.world.level.Level,net.minecraft.world.item.ItemStack,net.minecraft.world.InteractionHand)`; `net.minecraft.world.entity.player.Player#interactOn(net.minecraft.world.entity.Entity,net.minecraft.world.InteractionHand,net.minecraft.world.phys.Vec3)`
- **Applies when:** The local player presses attack or use while UI, cooldown, spectator, and related gates allow an action.
- **Behavior and timing:** Attack chooses entity attack, block-break start, or miss swing from the crosshair result. Use first attempts the matching entity or block interaction; its `InteractionResult` controls action consumption, swing, and fallback to the item's own use or the other hand. The server reruns the rules and synchronizes final item/world state.
- **Boundaries and quirks:** Main/offhand, sneak bypass of block use, interactable entities, empty items, and “successful without swing” results make a simple “block first” model inaccurate.
- **Verification owner (`PLY-INTERACT-001`, `ITM-ENDER-CHEST-001`, `ITM-BARREL-001`, `ITM-BOOKSHELF-001`, `ITM-JUKEBOX-001`, `BLK-COPPER-GOLEM-STATUE-001`, `BLK-BELL-001`, `BLK-ENCHANTING-TABLE-001`, `BLK-LECTERN-001`; `EXP-PLY-002`, `EXP-ITM-008`, `EXP-ITM-009`, `EXP-ITM-010`, `EXP-ITM-011`, `EXP-BLK-008`, `EXP-BLK-009`, `EXP-BLK-010`, `EXP-BLK-011`):** Concrete leaves fix their success/fallback transactions, including statue item precedence and bell/table/lectern main-hand try-empty-hand routing. Extract the remaining full decision table for every `InteractionResult` variant and both hands into tests.

## `PLY-006` Continuous breaking has client progress and a server-authoritative session

- **FidelityClass:** `ExactObservableBehavior`
- **Evidence status:** `Confirmed`
- **Primary evidence:** `OFF-CLIENT-001`; `OFF-SERVER-001`; `net.minecraft.client.multiplayer.MultiPlayerGameMode#startDestroyBlock(net.minecraft.core.BlockPos,net.minecraft.core.Direction)`; `net.minecraft.client.multiplayer.MultiPlayerGameMode#continueDestroyBlock(net.minecraft.core.BlockPos,net.minecraft.core.Direction)`; `net.minecraft.client.multiplayer.MultiPlayerGameMode#destroyBlock(net.minecraft.core.BlockPos)`; `net.minecraft.server.level.ServerPlayerGameMode#handleBlockBreakAction(net.minecraft.core.BlockPos,net.minecraft.network.protocol.game.ServerboundPlayerActionPacket$Action,net.minecraft.core.Direction,int,int)`; `COM-WIKI-PLY-001`
- **Applies when:** A survival/adventure player holds attack on a breakable target, or a creative player instant-breaks.
- **Behavior and timing:** The client accumulates current per-tick progress for a position plus held item/components, predicts eligible removal before sending sequenced start/stop actions, and retains pre-write states until a cumulative acknowledgement. The server independently follows `BLK-BREAK-001`, which recomputes progress from current speed over elapsed ticks and owns the real transaction. Authoritative updates are staged behind predictions and applied at acknowledgement.
- **Boundaries and quirks:** Target state changes do not reset the client when position and item/components still match; mid-session speed changes can diverge because client accumulation and server whole-interval recomputation differ. Creative five-call delay, unsequenced aborts, multi-position callback prediction, teleport-aware collision restoration and ACK/update ordering are separate branches.
- **Verification owner (`PLY-BREAK-001`; `EXP-PLY-003`):** The leaf locks all client input, prediction, sequence and convergence transitions. The experiment owns only whether the source-specified transient ACK restoration reaches a rendered frame before the subsequent authoritative update.
