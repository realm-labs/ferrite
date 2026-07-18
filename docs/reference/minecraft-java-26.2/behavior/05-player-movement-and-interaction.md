# 05 — Player Movement, Collision, Targeting, and Interaction

This page describes server gameplay results. See `CLI-*` for client input, prediction, and correction. The two cross-reference one another without merging ownership.

## `PLY-001` Input forms movement intent; the server entity owns movement truth

- **FidelityClass:** `ExactObservableBehavior`
- **Evidence status:** `Cross-checked`
- **Primary evidence:** `OFF-CLIENT-001`; `OFF-SERVER-001`; `net.minecraft.client.player.ClientInput#tick()`; `net.minecraft.client.player.LocalPlayer#tick()`; `net.minecraft.world.entity.player.Player#tick()`; `net.minecraft.world.entity.LivingEntity#travel(net.minecraft.world.phys.Vec3)`; `COM-WIKI-PLY-001`
- **Applies when:** A client owns a local player and the user changes directional, jump, sneak, sprint, or flight input.
- **Behavior and timing:** A client tick converts key state into a movement vector and posture intent, then immediately simulates the local player for responsiveness. The server validates received player state and places the server player at the authoritative result. Velocity, pose, grounded state, and collision form one tick transition rather than unrelated network fields that can overwrite one another arbitrarily.
- **Boundaries and quirks:** Packet loss, latency, and client FPS do not change server gameplay tick progression. Server correction can roll back local prediction; input and packet cadence are not guaranteed one-to-one.
- **Verification owner (`PLY-MOVE-001`; `EXP-PLY-*`):** Build protocol-independent black-box semantics for movement-packet selection, accepted error, teleport confirmation, and repeated correction.

## `PLY-002` Collision clips displacement by axis and shape

- **FidelityClass:** `ExactObservableBehavior`
- **Evidence status:** `Confirmed`
- **Primary evidence:** `OFF-SERVER-001`; `net.minecraft.world.entity.Entity#move(net.minecraft.world.entity.MoverType,net.minecraft.world.phys.Vec3)`; `net.minecraft.world.entity.Entity#collide(net.minecraft.world.phys.Vec3)`; `net.minecraft.world.entity.Entity#collideBoundingBox(net.minecraft.world.entity.Entity,net.minecraft.world.phys.Vec3,net.minecraft.world.phys.AABB,net.minecraft.world.level.Level,java.util.List)`
- **Applies when:** A player or other entity requests nonzero displacement outside a branch that bypasses ordinary collision.
- **Behavior and timing:** Movement collects block, world-border, and entity collision shapes across the swept region, then clips the requested displacement in vanilla's axis order to a non-penetrating vector. Actual displacement updates the bounding box, horizontal/vertical collision flags, grounded/fall state, and follow-up step/inside-block effects.
- **Boundaries and quirks:** Tiny displacement error, piston movement, world border, special blocks such as powder snow/scaffolding, and `noPhysics` add branches. Step-up compares an elevated candidate against normal clipping; it is not simply adding Y.
- **Verification owner (`PLY-MOVE-001`; `EXP-PLY-*`):** Lock axis order, epsilon, step-candidate comparison, multiple simultaneous shapes, and moving-platform boundaries.

## `PLY-003` Ground, water, lava, fall flying, and flight share an entry point but not dynamics

- **FidelityClass:** `ExactObservableBehavior`
- **Evidence status:** `Confirmed`
- **Primary evidence:** `OFF-SERVER-001`; `net.minecraft.world.entity.LivingEntity#travel(net.minecraft.world.phys.Vec3)`; `net.minecraft.world.entity.LivingEntity#jumpFromGround()`; `net.minecraft.world.entity.LivingEntity#travelInAir(net.minecraft.world.phys.Vec3)`; `net.minecraft.world.entity.LivingEntity#travelInFluid(net.minecraft.world.phys.Vec3)`; `net.minecraft.world.entity.player.Player#aiStep()`; `COM-WIKI-PLY-001`
- **Applies when:** A living entity advances velocity from input and its current medium.
- **Behavior and timing:** `travel` chooses dynamics for swimming/fluid, flight capability, fall flying, climbing, or ordinary ground/air state, then resolves collision through `Entity#move`. Jump changes velocity only when grounded/fluid/cooldown gates pass. Sprint, sneak, item use, effects, and attributes modify input or velocity.
- **Boundaries and quirks:** Crossing a medium boundary in one tick, eye-in-fluid versus bounding-box-in-fluid, auto-jump, and flight-mode transitions are observable. Extract concrete speed constants from locked methods and attributes.
- **Verification owner (`PLY-MOVE-001`; `EXP-PLY-*`):** Build tick-by-tick trajectory fixtures for medium boundaries, low-ceiling jumps, swimming pose, elytra launch/landing, and creative flight.

## `PLY-004` View targeting compares block and entity hits along the camera ray

- **FidelityClass:** `ExactObservableBehavior`
- **Evidence status:** `Confirmed`
- **Primary evidence:** `OFF-CLIENT-001`; `OFF-SERVER-001`; `net.minecraft.client.Minecraft#pick(float)`; `net.minecraft.world.entity.projectile.ProjectileUtil#getHitResultOnViewVector(net.minecraft.world.entity.Entity,java.util.function.Predicate,double)`; `net.minecraft.world.entity.player.Player#blockInteractionRange()`; `net.minecraft.world.entity.player.Player#entityInteractionRange()`
- **Applies when:** The client refreshes its crosshair target or prepares attack/use interaction.
- **Behavior and timing:** It clips block shapes from the eye/camera along the view vector and ray-tests expanded entity boxes within entity interaction range. The nearest eligible result becomes a miss, block hit, or entity hit. The server still validates against its own position, range, and target state.
- **Boundaries and quirks:** Block outline/collision/interaction shape, fluid mode, entity pick radius, passenger relations, and reach attributes change candidates. Integer-block DDA without entity-distance comparison is insufficient.
- **Verification owner (`PLY-MOVE-001`; `EXP-PLY-*`):** Lock exact ties, the eye starting inside a shape, just-over-reach positions, and moving-target client/server disagreement.

## `PLY-005` Hit type and InteractionResult govern attack/use priority

- **FidelityClass:** `ExactObservableBehavior`
- **Evidence status:** `Cross-checked`
- **Primary evidence:** `OFF-CLIENT-001`; `OFF-SERVER-001`; `net.minecraft.client.Minecraft#startAttack()`; `net.minecraft.client.Minecraft#startUseItem()`; `net.minecraft.server.level.ServerPlayerGameMode#useItemOn(net.minecraft.server.level.ServerPlayer,net.minecraft.world.level.Level,net.minecraft.world.item.ItemStack,net.minecraft.world.InteractionHand,net.minecraft.world.phys.BlockHitResult)`; `net.minecraft.server.level.ServerPlayerGameMode#useItem(net.minecraft.server.level.ServerPlayer,net.minecraft.world.level.Level,net.minecraft.world.item.ItemStack,net.minecraft.world.InteractionHand)`; `net.minecraft.world.entity.player.Player#interactOn(net.minecraft.world.entity.Entity,net.minecraft.world.InteractionHand,net.minecraft.world.phys.Vec3)`
- **Applies when:** The local player presses attack or use while UI, cooldown, spectator, and related gates allow an action.
- **Behavior and timing:** Attack chooses entity attack, block-break start, or miss swing from the crosshair result. Use first attempts the matching entity or block interaction; its `InteractionResult` controls action consumption, swing, and fallback to the item's own use or the other hand. The server reruns the rules and synchronizes final item/world state.
- **Boundaries and quirks:** Main/offhand, sneak bypass of block use, interactable entities, empty items, and “successful without swing” results make a simple “block first” model inaccurate.
- **Verification owner (`PLY-MOVE-001`; `EXP-PLY-*`):** Extract the full decision table for every `InteractionResult` variant and both hands from `26.2` client/server control flow into tests.

## `PLY-006` Continuous breaking has client progress and a server-authoritative session

- **FidelityClass:** `ExactObservableBehavior`
- **Evidence status:** `Confirmed`
- **Primary evidence:** `OFF-CLIENT-001`; `OFF-SERVER-001`; `net.minecraft.client.multiplayer.MultiPlayerGameMode#startDestroyBlock(net.minecraft.core.BlockPos,net.minecraft.core.Direction)`; `net.minecraft.client.multiplayer.MultiPlayerGameMode#continueDestroyBlock(net.minecraft.core.BlockPos,net.minecraft.core.Direction)`; `net.minecraft.client.multiplayer.MultiPlayerGameMode#destroyBlock(net.minecraft.core.BlockPos)`; `net.minecraft.server.level.ServerPlayerGameMode#handleBlockBreakAction(net.minecraft.core.BlockPos,net.minecraft.network.protocol.game.ServerboundPlayerActionPacket$Action,net.minecraft.core.Direction,int,int)`; `COM-WIKI-PLY-001`
- **Applies when:** A survival/adventure player holds attack on a breakable target, or a creative player instant-breaks.
- **Behavior and timing:** Start locks a position/state and opens progress. Each following client tick advances visible crack/prediction from current tool and block destroy progress; the server independently records start tick, target, and permitted progress. Only after the threshold does the server execute the `BLK-002` destruction transaction. Stop, target/tool change, or failed validation cancels or resets the session.
- **Boundaries and quirks:** Creative instant break, swords/restricted items, target mutation, latency making the client remove first, and sequence correction have separate branches.
- **Verification owner (`PLY-MOVE-001`; `EXP-PLY-*`):** Lock tick-by-tick results for tool swapping, haste/fatigue, underwater/airborne penalties, and client completion followed by server rejection.
