# Items mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ITM-BARREL-001` — A barrel owns 27 slots, materializes loot by caller, and exposes open state

**Parent:** `ITM-002`, `ITM-006`, `PLY-005`, `SIM-003`, `BLK-003`, `RED-003`, `MOB-004`, `ENT-001`,
`CLI-006`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — the barrel-owned state, menu admission, 27-slot storage, exact pending-loot
caller contexts, opener recount, comparator projection, persistence/components, removal drops,
sounds/game events and piglin ingress are fixed by locked source. Generic loot-table evaluation
remains owned by unfinished `ITM-LOOT-001`.

**Applies when:**

A `minecraft:barrel` is placed/rotated, used, opened/closed, read or mutated as a container, sampled
by a comparator, saved/loaded/componentized, replaced, or reached by its five-tick opener recount.

**Authoritative state:**

The block has six-way `FACING` and boolean `OPEN`, default north/false; placement faces opposite the
nearest look direction including vertical, and rotation/mirroring transform facing. It is
wood-map-color, bass-instrument, wood-sound, lava-ignitable, hardness/explosion resistance 2.5. The
matching block entity owns exactly 27 slots, default title `container.barrel`, optional custom
name/lock and optional loot-table key/seed. Generic capacity is 99 capped by the stack maximum. Its
menu is `generic_9x3`; when the generic explosion condition admits its block loot, that loot yields
one barrel carrying only custom name, while stored contents are a separate pre-removal transaction.

**Transition and ordering:**

Block use always returns success. Only on the server and with a live `BarrelBlockEntity` does it
call `openMenu`, then unconditionally award `OPEN_BARREL` and invoke visible-piglin anger, even if
menu admission failed. `openMenu` first closes any current non-inventory menu, advances the ID in
1..100, then asks the barrel to create a menu. Admission requires either no pending loot or a
nonspectator, plus spectator status or a main-hand stack matching the lock predicate. Failure for a
nonspectator sends `container.isLocked` with the display name then `CHEST_LOCKED` at block center,
volume/pitch 1; pending-loot spectator failure instead lets `openMenu` send
`container.spectatorCantOpen`. Either failure returns without screen or opener increment but still
consumes the ID and still reaches the stat/anger calls. Success materializes pending loot before
constructing the chest menu; its constructor calls `startOpen` before the open-screen packet and
current-menu installation.

**Loot caller boundary:**

Successful player opening resolves the table, triggers `GENERATE_LOOT` for the server player, clears
the table key before fill, and supplies stored seed plus `CHEST` context with block-center `ORIGIN`,
player `THIS_ENTITY`, and player luck. Public `isEmpty`, `getItem`, `removeItem`,
`removeItemNoUpdate`, and `setItem` instead first materialize with a null player: origin only, no
luck/entity/criterion. Therefore comparator reads, hopper-style access and removal use the
null-player context. The first such access performs fill; later accesses are idempotent. Raw
persistence/component helpers and `clearContent` do not themselves materialize. `ITM-LOOT-001` owns
evaluation/RNG/emitted stack order after these exact inputs.

**Open counter and state:**

Removed block entities and spectator users do not increment/decrement. First open plays
`BARREL_OPEN` at block center plus half the captured facing unit vector, volume 0.5/pitch
`0.9+0.1*nextFloat`, then offers captured state with `OPEN=true` and flags 3; only afterward the
common counter emits sourced `CONTAINER_OPEN` and schedules this block after five ticks at normal
priority/next sub-tick order. Final close analogously plays close sound, offers false with flags 3,
then emits sourced `CONTAINER_CLOSE`. Write results are ignored. Additional openers change only
signed count/range. A due recount searches the block AABB inflated by `maxInteractionRange+4` for
nonspectator `ContainerUser`s that report this position/counter open; a player matches exactly when
its current menu is a `ChestMenu` backed by this block-entity object. It recomputes count/range,
performs sound/state/game-event only on zero/nonzero boundaries (source null), and reschedules while
positive.

**Validity, comparator and persistence:**

Menu validity requires the identical block entity still installed and strict eye-to-block-AABB
squared distance below `(blockInteractionRange+4)^2`. Comparator output is zero for a wrong/missing
container; otherwise it sums `count/min(99,stackMax)` over nonempty slots, divides by 27, and
returns zero for an empty fraction or `floor(fraction*14)+1`. Removal requests an output-neighbor
refresh. Load initializes 27 empty slots and loads a pending table/seed instead of `Items`; save
writes the pending table and only a nonzero seed instead of `Items`. Lock/custom name persist
independently. Implicit block-entity components carry custom name, nonempty lock and container
contents, plus `CONTAINER_LOOT` when pending; removal from legacy tag discards duplicate
name/lock/items/table fields.

**Removal drops:**

When a state replacement removes the block entity and flag 256 does not suppress block-entity side
effects, the generic pre-removal hook iterates slots 0..26 through public `getItem`, so slot 0 first
materializes pending loot with the null-player context. For every slot, including empty ones, it
consumes three level doubles and fixes one spawn position: X/Z are integer block coordinate plus
`0.125+0.75d`, Y is integer Y plus `0.75d`. Each nonempty stack is then destructively split into
`10+nextInt(21)` chunks; per chunk, item-entity construction precedes an overwritten velocity of
three triangular samples with means `(0,0.2,0)` and deviation `0.11485000171139836`, consuming six
level doubles, then entity admission is attempted and ignored. All chunks from one slot share its
position. Thus empty removal still consumes exactly 81 position doubles, `block_drops=false` does
not suppress contents, and admission failure loses the already split items; flag 256 is the explicit
suppression boundary. Entity-private construction state remains `ENT-LIFECYCLE-001`.

**Piglin and client-visible effects:**

After every server use with the right block-entity subtype, the same no-RNG guarded-container
ingress as `ITM-ENDER-CHEST-001` considers idle visible `Piglin`s in the opener AABB inflated 16 and
writes 600-tick anger/universal-anger memories subject to attackability. Sounds are
server-positioned broadcasts; `OPEN` itself is ordinary block state synchronized by the flags-3
write, with no lid controller or block event.

**Branches and aborts:**

Client use and wrong/missing block entity return success without menu/stat/anger. Wrong lock and
pending-loot spectator attempts close an old menu and consume a new ID before failing. A spectator
with no pending table bypasses locks and receives a read-only menu/stat/anger but no open
count/state/sound/game event. A failed open-state write does not cancel the game event or recount
schedule. Replacing an open barrel makes later menu close a no-op because the block entity is
removed; menu validity closes it through the ordinary server path.

**Constants and randomness:**

27 slots/three rows, ID range 1..100, validity/recount buffer 4, recount delay 5, flags 3, sound
volume 0.5/pitch range `[0.9,1.0)`, piglin range 16/duration 600, comparator scale 14+1, item width
0.25, per-slot position range 0.75/half-width 0.125, split 10..30 and velocity deviation above are
fixed. Each first/final open consumes one server float; removal consumes 81 position doubles plus
one bounded integer and six level doubles per emitted chunk. Loot evaluator/private entity RNG are
separately owned.

**Side effects:**

Old-menu close; menu ID/screen/synchronization; lock overlays/sound; loot criterion/fill; stat;
piglin memories; inventory/dirty changes; `OPEN` writes and resulting neighbors/client publication;
scheduled recount; sounds/game events; comparator output refresh; persistence/components; item
entities and destructive slot emptying.

**Gates:**

Side and subtype, pending loot/spectator, main-hand lock predicate, removed/spectator opener,
current-menu backing identity, strict validity range, scheduled tick/live subtype, public versus raw
container access, block-entity-side-effect flag 256, item-entity admission, idle/visible/attackable
piglin and universal anger.

**Boundary cases and quirks:**

Lock failure still closes the old menu, consumes a container ID, increments the barrel statistic and
angers piglins. Sound uses the captured facing and precedes a flags-3 write of that captured barrel
state, so a callback mutation can be overwritten; failure still leaves count/game event advanced.
Comparator inspection can be the first action that permanently chooses a playerless loot context.
Empty-slot removal consumes RNG. The contents drop independently of the barrel-item loot and copies
neither inventory nor lock into that item; only custom name is copied by the locked block table.

**Evidence:**

`OFF-SERVER-001`, `OFF-DATA-001`, `OFF-REPORT-001`;
`net.minecraft.world.level.block.BarrelBlock#useWithoutItem(net.minecraft.world.level.block.state.BlockState,net.minecraft.world.level.Level,net.minecraft.core.BlockPos,net.minecraft.world.entity.player.Player,net.minecraft.world.phys.BlockHitResult)`,
`net.minecraft.world.level.block.BarrelBlock#tick(net.minecraft.world.level.block.state.BlockState,net.minecraft.server.level.ServerLevel,net.minecraft.core.BlockPos,net.minecraft.util.RandomSource)`,
`net.minecraft.world.level.block.BarrelBlock#getAnalogOutputSignal(net.minecraft.world.level.block.state.BlockState,net.minecraft.world.level.Level,net.minecraft.core.BlockPos,net.minecraft.core.Direction)`,
`net.minecraft.world.level.block.BarrelBlock#getStateForPlacement(net.minecraft.world.item.context.BlockPlaceContext)`,
`net.minecraft.world.level.block.entity.BarrelBlockEntity#startOpen(net.minecraft.world.entity.ContainerUser)`,
`net.minecraft.world.level.block.entity.BarrelBlockEntity#stopOpen(net.minecraft.world.entity.ContainerUser)`,
`net.minecraft.world.level.block.entity.BarrelBlockEntity#recheckOpen()`,
`net.minecraft.world.level.block.entity.BarrelBlockEntity#loadAdditional(net.minecraft.world.level.storage.ValueInput)`,
`net.minecraft.world.level.block.entity.BarrelBlockEntity#saveAdditional(net.minecraft.world.level.storage.ValueOutput)`,
`net.minecraft.world.level.block.entity.RandomizableContainerBlockEntity#createMenu(int,net.minecraft.world.entity.player.Inventory,net.minecraft.world.entity.player.Player)`,
`net.minecraft.world.RandomizableContainer#unpackLootTable(net.minecraft.world.entity.player.Player)`,
`net.minecraft.world.level.block.entity.ContainerOpenersCounter#recheckOpeners(net.minecraft.world.level.Level,net.minecraft.core.BlockPos,net.minecraft.world.level.block.state.BlockState)`,
`net.minecraft.world.inventory.AbstractContainerMenu#getRedstoneSignalFromContainer(net.minecraft.world.Container)`,
`net.minecraft.world.level.block.entity.BlockEntity#preRemoveSideEffects(net.minecraft.core.BlockPos,net.minecraft.world.level.block.state.BlockState)`,
`net.minecraft.world.Containers#dropItemStack(net.minecraft.world.level.Level,double,double,double,net.minecraft.world.item.ItemStack)`,
`net.minecraft.server.level.ServerPlayer#openMenu(net.minecraft.world.MenuProvider)`,
`net.minecraft.world.entity.monster.piglin.PiglinAi#angerNearbyPiglins(net.minecraft.server.level.ServerLevel,net.minecraft.world.entity.player.Player,boolean)`;
`data/minecraft/loot_table/blocks/barrel.json`, registry/state membership from `reports/blocks.json`
and `reports/registries.json`; `EXP-ITM-009`.

**Test vectors:**

All 12 facing/open states; client/wrong subtype; ordinary/custom-name/locked barrels with
correct/wrong hand; spectator with pending/no pending loot; old menu on every failed/successful
attempt; player luck/criterion versus comparator/hopper/removal null-player materialization; one/two
viewers with distinct reach and forced recount; captured-facing mutation/write failure; exact
comparator empty/partial/full values; item/table/seed/name/lock/component round trips; empty removal
RNG, counts 1/10/30/31/99 and rejected entity admission; `block_drops=false` and flag-256
replacement; visible/hidden/nonidle piglin with universal anger.
