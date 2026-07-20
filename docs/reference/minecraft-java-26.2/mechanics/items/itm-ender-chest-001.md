# Items mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ITM-ENDER-CHEST-001` — Ender-chest items belong to the player while the used block owns only open presentation

**Parent:** `ITM-002`, `PLY-005`, `SIM-003`, `BLK-003`, `MOB-004`, `CLI-006`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — the player-owned 27-slot storage, active-block link, use/open/close ordering,
opener recount, sounds/game events, block-event lid animation, ambient particles, piglin anger
ingress, persistence and all ender-chest block states are fixed by locked source.

**Applies when:**

A `minecraft:ender_chest` is placed or updated, its block-use path runs, a three-row ender menu
opens/closes or becomes invalid, its five-tick opener recount becomes due, a lid block event
arrives, the client animates/renders the block, or player data loads/saves/restores.

**Authoritative state:**

The block has horizontal `FACING` and `WATERLOGGED`, default north/false, never combines with a
neighbor, uses a centered 14x14 column from Y 0 through 14, is never pathfindable, emits light 7,
and has hardness 22.5/explosion resistance 600. Its block entity stores only a transient opener
counter, maximum observed interaction range and lid controller; it stores no items and writes no
persistent data. Each player instead owns one 27-slot `PlayerEnderChestContainer`, maximum 99
further limited by each stack maximum, plus a nullable identity link to the currently active
ender-chest block entity. The locked block loot selects the chest item for Silk Touch and otherwise
eight obsidian, subject to the generic loot/explosion transaction.

**Transition and ordering:**

Placement faces opposite the player's horizontal direction and waterlogs exactly when the replaced
fluid is water. Rotation/mirroring transform `FACING`; a waterlogged state exposes still source
water, and every waterlogged neighbor-shape update schedules a water tick at the fluid delay. Block
use first fetches the player's container and live block entity; a missing container or wrong block
entity returns success without side effects. It next rejects opening when the block directly above
is a redstone conductor at that position, again returning success. On the server it then sets the
player's active link to this block entity before calling `openMenu`. Menu opening closes any
existing non-inventory menu, allocates the next ID in 1..100, constructs a `generic_9x3` `ChestMenu`
titled `container.enderchest` (whose constructor calls `startOpen`), sends the open-screen packet,
installs synchronization/current-menu state, and returns. Only afterward block use awards
`OPEN_ENDERCHEST` and invokes visible-piglin anger. The client-side block use performs none of these
mutations and still returns success.

**Open/close counter:**

`startOpen` and `stopOpen` do nothing when the block entity is removed or the user is a spectator.
Otherwise first open (`0 -> 1`) plays `ENDER_CHEST_OPEN` at block center in `BLOCKS`, volume 0.5 and
pitch `0.9 + 0.1*nextFloat`, emits `CONTAINER_OPEN` with the opener as source, and schedules this
block after five ticks at normal priority/next sub-tick order. Every increment sends block event
`(1,currentCount)` and raises the stored maximum interaction range. Final close (`1 -> 0`)
analogously plays the close sound with one float draw, emits `CONTAINER_CLOSE`, clears maximum
range, and every decrement sends the count event. Counts are ordinary signed integers with no clamp.

**Recount and block events:**

A due block tick asks the live ender-chest block entity to recount. It searches a block AABB
inflated by `maxInteractionRange+4` for nonspectator `ContainerUser`s reporting this
counter/position open; a player reports true exactly when its ender inventory's active link is this
block entity. The pass resets/recomputes maximum range and replaces the count with list size. A
zero/nonzero boundary discovered here plays the corresponding sound and emits a source-less
container game event; nonzero-to-nonzero changes do neither. It sends block event `(1,newCount)`
even when unchanged and reschedules after five ticks only while the result is positive. The server's
block-event drain applies the event only if the same block remains at a tickable position; accepted
events broadcast within 64 blocks in the dimension. Event 1 makes the lid target open exactly when
its parameter is positive.

**Menu validity and close:**

While the active link is nonnull, menu validity requires the same block-entity object still
installed and strict squared eye-to-block-AABB distance less than `(blockInteractionRange+4)^2`; a
null link falls through to the always-valid simple container. Menu close calls `stopOpen` through
the shared player container and then clears the link. Ordinary click/quick-move/state-ID behavior
and the server's broadcast-before-validity-close order remain `ITM-CONTAINER-*`.

**Player persistence:**

Load first clears all 27 slots, then accepts valid `EnderItems` slot records in list order, so a
later duplicate slot replaces an earlier one; `setItem` enforces its effective maximum. Save emits
nonempty slots only, ascending 0..26. Player slot addresses 200..226 expose these entries.
Server-player restoration assigns the old `PlayerEnderChestContainer` object itself regardless of
the ordinary inventory/XP retention branch, so contents survive player recreation and are never
dropped by breaking an ender-chest block.

**Piglin side effect:**

After every unobstructed server use reaching the open call—including spectator use—the server
queries `Piglin` (not brutes or zombified piglins) in the player's bounding box inflated by 16,
preserves entity-list order, and retains only brains currently in `IDLE` that can see the opener.
With `universal_anger=false`, each attackable survivor clears `CANT_REACH_WALK_TARGET_SINCE` and
stores the opener UUID as `ANGRY_AT` for 600 ticks. With it true, the piglin instead prefers its
`NEAREST_VISIBLE_ATTACKABLE_PLAYER` memory and falls back to the opener, then the same setter also
stores `UNIVERSAL_ANGER=true` for 600 ticks when the target is a player. This ingress consumes no
RNG; subsequent Brain behavior remains `MOB-AI-001`.

**Client presentation:**

Only the client ticks the lid controller. Each tick copies current to previous and moves openness by
exactly 0.1 toward the event-selected target, clamped to [0,1]; rendering linearly interpolates by
partial tick and transforms it to `1-(1-open)^3` before driving the single ender-chest lid model in
block facing. Independently of open state, every client animate call creates three portal particles.
Each particle consumes two `nextInt(2)` draws for X/Z signs and four floats for Y position and X/Y/Z
velocity: position is block center plus 0.25 times each horizontal sign and Y in `[0,1)`, while
velocity is `(signX*f, (f-0.5)*0.125, signZ*f)`.

**Branches and aborts:**

Obstruction, wrong/missing block entity and client-side use all return the same success result as a
normal open. A spectator receives the screen, stat and piglin side effect but contributes no opener
count, sound, game event or lid-open event. Server block events are deferred while the position is
not tickable and discarded if the block identity changed before application. Recount has source-less
game events; direct open/close uses the container user.

**Constants and randomness:**

27 slots, three rows, menu IDs 1..100, validity buffer 4, recount delay 5, block-event radius 64,
lid step 0.1, three ambient particles, particle offsets 0.25/0.125, piglin box inflation 16 and
anger duration 600 are fixed above. A normal first/final opener transition consumes one server float
for sound pitch; each ambient client call consumes six bounded integer draws and 12 floats. Menu,
recount, block event, persistence and anger selection consume no RNG.

**Side effects:**

Active-link changes; old-menu close; open-screen/menu synchronization; stat award; 27-slot
mutations; opener count/range; scheduled block/fluid ticks; sounds; game events; block
events/packets; lid model state; portal particles; piglin Brain memories; player data writes.
Breaking the block has no access to or effect on player-owned ender items.

**Gates:**

Live subtype, conductor above, server side, removed/spectator opener state, current menu,
block-entity identity, strict interaction range, tickable block-event position, positive event
count, valid persisted slot index, idle/visible/attackable piglin and `universal_anger`.

**Boundary cases and quirks:**

The active link is assigned before `openMenu` closes an old ender menu, while both menus share the
same player container. A forced same-chest reopen therefore closes/counts down that chest and clears
the just-assigned link before constructing the new menu; a forced cross-chest reopen counts down the
newly targeted chest (possibly `0 -> -1`) while leaving the old chest to repair itself at recount.
The new menu then starts with no active link, does not increment, and becomes distance-independent;
a negative target counter may remain unscheduled and requires later opens/another user's transition
to recover. Recount sends periodic positive block events even without a count change, allowing a
newly observing client to converge within the five-tick cycle. The above-block test is
redstone-conductor semantics, not a generic nonair or collision-shape test.

**Evidence:**

`OFF-SERVER-001`, `OFF-CLIENT-001`, `OFF-DATA-001`, `OFF-REPORT-001`;
`net.minecraft.world.level.block.EnderChestBlock#getStateForPlacement(net.minecraft.world.item.context.BlockPlaceContext)`,
`net.minecraft.world.level.block.EnderChestBlock#getTicker(net.minecraft.world.level.Level,net.minecraft.world.level.block.state.BlockState,net.minecraft.world.level.block.entity.BlockEntityType)`,
`net.minecraft.world.level.block.EnderChestBlock#useWithoutItem(net.minecraft.world.level.block.state.BlockState,net.minecraft.world.level.Level,net.minecraft.core.BlockPos,net.minecraft.world.entity.player.Player,net.minecraft.world.phys.BlockHitResult)`,
`net.minecraft.world.level.block.EnderChestBlock#animateTick(net.minecraft.world.level.block.state.BlockState,net.minecraft.world.level.Level,net.minecraft.core.BlockPos,net.minecraft.util.RandomSource)`,
`net.minecraft.world.level.block.EnderChestBlock#updateShape(net.minecraft.world.level.block.state.BlockState,net.minecraft.world.level.LevelReader,net.minecraft.world.level.ScheduledTickAccess,net.minecraft.core.BlockPos,net.minecraft.core.Direction,net.minecraft.core.BlockPos,net.minecraft.world.level.block.state.BlockState,net.minecraft.util.RandomSource)`,
`net.minecraft.world.level.block.EnderChestBlock#tick(net.minecraft.world.level.block.state.BlockState,net.minecraft.server.level.ServerLevel,net.minecraft.core.BlockPos,net.minecraft.util.RandomSource)`,
`net.minecraft.world.level.block.entity.EnderChestBlockEntity#startOpen(net.minecraft.world.entity.ContainerUser)`,
`net.minecraft.world.level.block.entity.EnderChestBlockEntity#stopOpen(net.minecraft.world.entity.ContainerUser)`,
`net.minecraft.world.level.block.entity.EnderChestBlockEntity#recheckOpen()`,
`net.minecraft.world.level.block.entity.EnderChestBlockEntity#triggerEvent(int,int)`,
`net.minecraft.world.level.block.entity.EnderChestBlockEntity#lidAnimateTick(net.minecraft.world.level.Level,net.minecraft.core.BlockPos,net.minecraft.world.level.block.state.BlockState,net.minecraft.world.level.block.entity.EnderChestBlockEntity)`,
`net.minecraft.world.level.block.entity.ContainerOpenersCounter#incrementOpeners(net.minecraft.world.entity.LivingEntity,net.minecraft.world.level.Level,net.minecraft.core.BlockPos,net.minecraft.world.level.block.state.BlockState,double)`,
`net.minecraft.world.level.block.entity.ContainerOpenersCounter#recheckOpeners(net.minecraft.world.level.Level,net.minecraft.core.BlockPos,net.minecraft.world.level.block.state.BlockState)`,
`net.minecraft.world.inventory.PlayerEnderChestContainer#fromSlots(net.minecraft.world.level.storage.ValueInput$TypedInputList)`,
`net.minecraft.world.inventory.PlayerEnderChestContainer#storeAsSlots(net.minecraft.world.level.storage.ValueOutput$TypedOutputList)`,
`net.minecraft.world.inventory.PlayerEnderChestContainer#stopOpen(net.minecraft.world.entity.ContainerUser)`,
`net.minecraft.server.level.ServerPlayer#openMenu(net.minecraft.world.MenuProvider)`,
`net.minecraft.world.entity.monster.piglin.PiglinAi#angerNearbyPiglins(net.minecraft.server.level.ServerLevel,net.minecraft.world.entity.player.Player,boolean)`,
`net.minecraft.client.renderer.blockentity.ChestRenderer#extractRenderState(net.minecraft.world.level.block.entity.BlockEntity,net.minecraft.client.renderer.blockentity.state.ChestRenderState,float,net.minecraft.world.phys.Vec3,net.minecraft.client.renderer.feature.ModelFeatureRenderer$CrumblingOverlay)`;
`data/minecraft/loot_table/blocks/ender_chest.json`, registry/state membership from
`reports/blocks.json` and `reports/registries.json`; `EXP-ITM-008`.

**Test vectors:**

All eight facing/water states and water-neighbor scheduling; each conductor/nonconductor above;
client/server, spectator and wrong block entity; first/second/final opener with distinct reach
attributes; strict range boundary, block replacement and unloaded recount; unchanged and changed
five-tick recount; block-event counts negative/zero/positive and a late-joining client; 10 lid ticks
each direction plus partial frames; exact three-particle draws; two players with different 27-slot
contents; invalid/duplicate persisted slots and player recreation; ordinary open, forced same-chest
reopen, cross-chest reopen and negative-counter recovery; visible/hidden/nonidle piglin with
universal anger off/on and alternate nearest target.
