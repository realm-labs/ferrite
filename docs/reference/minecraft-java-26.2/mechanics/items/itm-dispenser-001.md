# Items mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ITM-DISPENSER-001` — A dispenser selects one slot, dispatches by stack semantics, then publishes behavior events

**Parent:** `ITM-002`, `ITM-006`, `RED-001`, `SIM-003`, `SIM-004`, `BLK-003`,
`PLY-006`, `ENT-001`, `CLI-006`, `ENV-005`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked server source fixes the dispenser's twelve block states, retained
four-tick trigger, nine-slot reservoir selection, code-built 80-item behavior registry, dynamic
component/tag/spawn-egg fallbacks, every behavior wrapper and remainder branch, menu, persistence,
removal and level-event projection. This rule owns dispatcher admission and wrapper ordering;
called block, entity, projectile, equipment, loot and item mechanics retain their own leaf rules.
Generic pending-loot evaluation remains unfinished under `ITM-LOOT-001`.

**Applies when:**

A `minecraft:dispenser` is placed, powered, scheduled, used, comparator-read, loaded, saved,
componentized, removed or projected, or when its due scheduled tick selects and dispatches a stack.

**Authoritative state:**

The block has six-way `FACING` and boolean `TRIGGERED`, default north/false: twelve report states
566..577. Placement uses the nearest look direction's opposite, including vertical directions;
rotation and mirror transform facing. It has stone map color, bass-drum instrument, correct-tool
requirement and strength 3.5. Its full-cube ordinary block geometry is independent of state.

The exact `DISPENSER` block entity owns nine slots, optional loot-table key/seed, lock and custom
name. Its default title is `container.dispenser`, block-entity protocol ID is 5, and its shared
`generic_3x3` menu has protocol ID 6. Generic container capacity is 99, while an individual stack
is capped at the lesser of 99 and that stack's maximum.

**Transition and ordering:**

Every neighbor callback evaluates `hasNeighborSignal(pos)` and then, only if false,
`hasNeighborSignal(pos.above())`. Powered plus untriggered schedules this block after four ticks,
then offers `TRIGGERED=true` with flags 2; neither result is inspected and there is no local
already-scheduled test. Unpowered plus triggered offers false with flags 2 but does not cancel
pending work. A due server block tick dispatches without rechecking power or the latch, so a short
pulse is retained. Generic scheduled-tick deduplication, persistence and activity admission remain
`SIM-SCHEDULE-001`/`SIM-003`.

Dispatch first requires the exact `DISPENSER` block-entity type. A mismatch logs a warning and
returns before RNG or events. `BlockSource` then captures the due tick's block state and that block
entity before deferred loot is opened. `getRandomSlot` calls `unpackLootTable(null)`, giving pending
loot the stored seed and a `CHEST` context with only block-center `ORIGIN`; it scans slots 0..8 and
for the kth nonempty slot consumes `nextInt(k)`, replacing the candidate on zero. No occupied slot
therefore yields -1 with no selection draws, emits event 1001/data 0, then emits
`BLOCK_ACTIVATE` at the source using the block entity's current block state as context; it emits no
2000 animation event.

For a selected stack, behavior resolution is ordered:

1. A stack disabled by the level's feature set uses default ejection immediately.
2. Otherwise the code-built identity registry wins.
3. An unregistered stack with `EQUIPPABLE` uses the shared equipment behavior.
4. Otherwise membership in `minecraft:sulfur_cube_swallowable` uses the shared sulfur-cube
   equipment behavior.
5. Otherwise a `SpawnEggItem` carrying `ENTITY_DATA` uses the shared spawn-egg behavior.
6. Everything else uses default ejection.

The registry has 80 item identities: 13 projectiles; armor stand and chest; 20 boat/raft variants;
ten filled buckets; empty bucket, flint and steel, bone meal, TNT, wither-skeleton skull and carved
pumpkin; all 17 shulker-box colors; glass bottle, glowstone, shears, brush, honeycomb and potion;
and six minecart variants. Explicit entries win over components and tags, notably for TNT,
glowstone and carved pumpkin. The sulfur tag delegates to all twelve locked
`sulfur_cube_archetype/*` item tags. No locked vanilla entry uses `NOOP`; if that exact behavior is
externally registered, dispatch skips both the call and source-slot write.

**Behavior wrapper and remainder transaction:**

Except for `NOOP`, the selected behavior receives the live stack, then its returned stack is
written to the selected slot and the source is dirtied. `DefaultDispenseItemBehavior.dispense` is
final: it executes the action, publishes its sound event, publishes event 2000 with captured-facing
data, then returns. Default sound is event 1000. `OptionalDispenseItemBehavior` instead chooses
1000 or 1001 from its mutable `success` field but still always publishes 2000. Projectile behavior
uses its configured launch event instead of 1000. All geometry and animation use the captured
state, even if a called action rotates/replaces the live source.

Default ejection splits one item at source center plus `0.7*facing`, subtracts 0.125 Y for vertical
or 0.15625 otherwise, creates an item entity and overwrites motion using one speed double plus
three triangular samples at accuracy 6; entity-add success is ignored and the item remains
consumed. This is the same geometry/RNG transaction fixed for dropper by `ITM-DROPPER-001`.

Actions producing a remainder first shrink one source item. If that empties the selected stack,
the remainder becomes the returned stack. Otherwise `DispenserBlockEntity.insertItem` scans 0..8,
without placement predicates, merging same item/components or filling empty slots up to the
remainder's effective maximum. Any residue is default-ejected and immediately publishes an extra
1000 then 2000 before the outer behavior publishes its own pair. Empty-slot insertion calls
`setItem`; direct merge grows the resident stack, and the later selected-slot write provides the
source dirty call.

Some failed special actions call a nested default behavior's public `dispense`, not its protected
`execute`: failed filled-bucket emptying, invalid boat/raft water placement, invalid minecart rail
placement and a non-water/non-mud potion each eject one item and publish inner 1000/2000 followed
by outer 1000/2000. Other stated fallback-ejection paths call protected `execute` and publish only
the outer pair.

**Registered and dynamic behavior matrix:**

| Match | Action and exact failure boundary |
|---|---|
| Arrow, tipped/spectral arrow, egg/blue egg/brown egg, snowball | Spawn the item's projectile from default position with power 1.1, uncertainty 6 and event 1002; shrink one after the spawn helper. |
| Experience bottle, splash potion, lingering potion | Projectile as above but power 1.375 and uncertainty 3; event 1002. |
| Firework rocket | Position just outside the face at center plus `0.5000099999997474*facing`, power 0.5, uncertainty 1, event 1004. |
| Fire charge / wind charge | Position at center plus `1.0*facing`, power 1, uncertainty 6.6666665; events 1018 / 1051. |
| Armor stand | Spawn at the adjacent block with `DISPENSER` reason, stack configuration and facing yaw. Null spawn keeps the stack, but the wrapper still reports success events. |
| Chest | Iterate alive, unchested `AbstractChestedHorse`s in the adjacent block AABB; the first tamed horse whose slot 499 accepts the stack gains a chest and consumes one. Otherwise protected-default eject. |
| Every boat/raft and chest variant | Front water gives vertical offset 1; front air over water gives 0. Spawn at center plus facing times `0.5625 + type.width/2`, with Y also shifted by `1.125*facingY`, stack config and facing yaw. Other terrain uses nested-default eject. Null creation keeps the stack; entity-add failure is ignored after shrink. |
| Ten filled buckets | Call `emptyContents(null, level, front, null)` and then `checkExtraContent`; success produces an empty bucket through the remainder transaction. Failure uses nested-default eject. |
| Empty bucket | If the front block implements `BucketPickup` and returns a nonempty stack, emit `FLUID_PICKUP` at the target and produce that filled bucket through the remainder transaction. Otherwise protected-default eject. |
| Flint and steel | In order: prime the first eligible sulfur cube in the front AABB, place fire, light a campfire/candle/candle cake, or prime/remove TNT. Fire emits `BLOCK_PLACE`; lighting emits `BLOCK_CHANGE`. Success damages the tool by one. A nonapplicable non-TNT target nevertheless leaves success true and damages the tool; only a TNT whose `prime` returns false produces failure/no damage. |
| Bone meal | Try crop growth, then water growth. Success emits event 1505/data 15 at the target; failure keeps the stack and reports optional failure. No fallback ejection. |
| TNT | `tnt_explodes=false` keeps the stack and reports failure. Otherwise first offer TNT to a sulfur cube in the front AABB; if none accepts, create centered `PrimedTnt`, ignore entity-add success, play `TNT_PRIMED`, emit `ENTITY_PLACE`, shrink one and report success. |
| Wither-skeleton skull | If front is empty and `canSpawnMob`, place a flags-3 floor skull with facing-derived rotation, emit `BLOCK_PLACE`, call `checkSpawn` and shrink. Otherwise try equipment; failure keeps the stack and reports 1001. |
| Carved pumpkin | If front is empty and a golem pattern can spawn, flags-3 place, emit `BLOCK_PLACE` and shrink; otherwise try equipment, with 1001 on failure. |
| All shulker boxes | Reset failure, then call `BlockItem.place` at the front using captured facing and a placement direction of captured facing when the block below is empty, otherwise up. A consuming result is success; rejection or caught exception keeps the stack and reports 1001. No ejection fallback. |
| Glass bottle | Reset failure. A full beehive is released/reset and produces honey bottle; otherwise any front water fluid produces a water potion without removing that water. Both emit `FLUID_PICKUP` at the dispenser position and use the remainder transaction. Otherwise protected-default ejects the bottle but reports 1001. |
| Glowstone | A nonfull respawn anchor is charged and consumes one; a full anchor keeps it and reports failure. Any non-anchor target protected-default ejects with success. |
| Shears | Try a full beehive first; otherwise scan front nonspectator entities, first removing all leash connections when possible, else shearing the first alive ready `Shearable`. Success emits `SHEAR` and damages one; failure keeps the tool and reports 1001. |
| Brush | Scan front nonspectator armadillos in encounter order; first successful `brushOffScute(null, stack)` damages 16 and returns. Empty/no-success keeps the tool and records failure. |
| Honeycomb | Wax a convertible front state, update it, emit event 3003 and shrink one; otherwise protected-default eject. Its optional state never records failure, so both paths use event 1000. |
| Potion item | Only exact water potion against `convertable_to_mud` is special: consume ten server doubles for five splash particles above the dispenser, play bottle-empty, emit `FLUID_PLACE` at the dispenser, replace the target with mud and produce a glass bottle. All other potion contents/targets use nested-default eject. |
| Six minecarts | A front rail uses Y offset 0.6 when sloped, else 0.1. Front air over a rail uses -0.4 only when facing is not down and that rail slopes, else -0.9. Spawn at center X/Z plus `1.125*facing`, floor(centerY)+facingY+offset; invalid terrain uses nested-default eject. Null creation keeps the stack; entity-add failure is ignored after shrink. |
| Unregistered `EQUIPPABLE` | Select the first front-AABB living entity satisfying `canEquipWithDispenser`, split one into its resolved equipment slot, and for a mob guarantee that drop and require persistence. No candidate protected-default ejects. |
| Unregistered sulfur-swallowable | Iterate front-AABB sulfur cubes and let the first accepting `equipItem` consume one; otherwise protected-default eject. |
| Spawn egg with `ENTITY_DATA` | Resolve the encoded entity type and spawn at the adjacent block with `DISPENSER` reason; Y adjustment is enabled except while facing up. Success shrinks one and emits `ENTITY_PLACE` at the dispenser. Null type/spawn keeps the stack but still reports success events. A caught exception logs and returns empty, clearing the selected slot, then reports success events. |
| Everything else or feature-disabled stack | Default one-item ejection. |

Projectile creation, collision and payload semantics; entity spawn admission; equipment predicates;
bucket/block placement; fire/TNT/golem/wither/sulfur mechanics; shearing; waxing; growth; and mud
replacement effects beyond these dispatcher calls remain with their named subsystem leaves.

**Persistent optional-success state:**

Behavior instances live in the static identity registry and are shared across dispensers/worlds.
Most optional behaviors reset or overwrite `success` on every relevant path. Two locked exceptions
are observable. Brush starts true, records false after any miss, and never sets true on a later
successful brush, so after the first miss all later brush successes still use event 1001 while
performing the action. TNT records false when `tnt_explodes=false`; its later ordinary primed-TNT
branch resets true, but a successful sulfur-cube acceptance returns without resetting, so that
success can inherit event 1001 from an earlier disabled-rule attempt.

**Player, comparator, persistence and removal:**

Empty-hand block use always returns success. Client use does nothing else. Server use accepts any
`DispenserBlockEntity`, calls `openMenu`, ignores the result, then awards `INSPECT_DROPPER` only for
a dropper subtype and `INSPECT_DISPENSER` otherwise; a wrong/missing entity gets neither. Lock and
spectator-pending-loot rejection can therefore still consume the generic menu transition and stat
ordering described by `ITM-CONTAINER-001`.

The 3x3 slots are row-major from `(62,17)` and player inventory begins `(8,84)`. Quick move sends
dispenser slots 0..8 to player slots in reverse and player slots to 0..8 forward. Start/stop open
are default no-ops. Validity requires the identical block entity and the generic strict
`(blockInteractionRange+4)^2` distance boundary.

Comparator output public-reads all nine slots, materializing pending loot with a null-player
context, then uses the generic fullness formula. Load initializes nine empty slots and reads a
pending table/seed instead of `Items`; save writes one representation or the other. Custom name,
nonempty lock, contents and pending `CONTAINER_LOOT` participate in components. The common stack-64
dispenser item begins with an empty container component.

Block loot survives explosion and yields a dispenser copying only custom name. Unless flag 256
suppresses block-entity pre-removal effects, contents are separately public-read and dropped;
nine slots consume 27 position doubles even when empty before ordinary split/velocity/entity
admission. `block_drops=false` does not suppress that transaction. Inventory, lock and pending loot
are not copied into the block-loot item. Removal also requests comparator-neighbor updates.

**Client projection:**

Flags-2 latch writes synchronize `TRIGGERED`, but blockstate JSON selects only by facing, so true
and false look identical. North/east/south/west rotate `block/dispenser`; up/down use
`block/dispenser_vertical`; the item uses the horizontal north model. There is no block-entity
renderer or update packet, so inventory reaches clients only through menu synchronization.

Event 1000 plays `DISPENSER_DISPENSE` in `BLOCKS`, volume/pitch 1/1; 1001 plays
`DISPENSER_FAIL`, 1/1.2; 1002 plays `DISPENSER_LAUNCH`, 1/1.2; 1004 plays
`FIREWORK_ROCKET_SHOOT` in `NEUTRAL`, 1/1.2. Event 1018 plays `BLAZE_SHOOT` in `HOSTILE`, volume 2
and pitch `1+0.2*(r1-r2)`, consuming two client floats. Event 1051 plays `WIND_CHARGE_THROW` in
`BLOCKS`, volume 0.5 and pitch `0.4/(0.8+0.4r)`, consuming one client float. Event 2000 creates the
same ten facing smoke particles specified by `ITM-DROPPER-001`. Nested/default-remainder fallbacks
publish duplicate sound/animation pairs in the server order above. Events 1505 and 3003 additionally
project the generic bone-meal and wax effects at their target positions.

**Branches and aborts:**

Wrong typed block entity, `NOOP`, no occupied slot, feature-disabled stack, null special spawn,
caught spawn-egg/shulker exception, optional failure, nested fallback, remainder insertion/ejection
and uncaught delegated exception are distinct boundaries. `NOOP` performs no source write or
event; empty inventory performs fail sound plus game event but no animation; optional failure keeps
or ejects exactly as its matrix row states and still animates. An uncaught delegated exception
skips outer sound, animation and final source write after any earlier mutation.

**Constants and randomness:**

Delay 4, flags 2, twelve states, nine slots, protocol IDs 5/6, 80 explicit entries, one selected
item, capacity 99, entity/target AABBs of one block, default offsets/accuracy and vehicle/rail
offsets above are fixed. Server RNG includes occupied-slot reservoir draws, default-ejection seven
doubles, projectile shoot randomness, water-potion ten doubles, delegated entity construction and
the separately owned loot evaluator. Encounter-order entity choices consume no selection RNG.
Client RNG includes event-1018/1051 floats, ten event-2000 particle samples and generic 1505/3003
effects.

**Side effects:**

Scheduled tick and latch writes; warning/error logs; deferred loot; source/remainder inventory
mutation and dirty hooks; target block/fluid/equipment/entity/tool mutations; spawned entities;
sounds, particles, level/game events and comparator updates; menu/stat/protocol state; persistence,
components and removal entities; client model/menu/event projection.

**Gates:**

Self/above power, latch and due scheduled tick; exact block-entity type; occupied selection;
feature flags; identity registry then component/tag/class resolution; captured facing; every
behavior-specific block/fluid/entity/item/game-rule predicate; stack capacity and remainder space;
server entity admission; menu spectator/lock/range; persistence representation; flag 256 and
removal entity admission.

**Boundary cases and quirks:**

All dispatch geometry uses the captured due-tick state, not a later live state. An empty dispatch
is the only dispatcher-owned `BLOCK_ACTIVATE` game event. Special null spawns commonly keep the
stack while still sounding/animating; spawn-egg exceptions instead clear the entire selected slot.
Failed filled buckets, boats, minecarts and ordinary potions produce duplicate event pairs. A full
source with no room for a produced bucket/bottle adds another default pair while ejecting the
remainder. Glass-bottle fallback ejects successfully while intentionally playing fail. Flint and
steel can damage/sound success against a nonapplicable target. Brush and sulfur-accepted TNT can
perform success while retaining a process-global failure sound state.

**Evidence:**

`OFF-SERVER-001`, `OFF-CLIENT-001`, `OFF-DATA-001`, `OFF-REPORT-001`;
`net.minecraft.world.level.block.DispenserBlock#dispenseFrom(net.minecraft.server.level.ServerLevel,net.minecraft.world.level.block.state.BlockState,net.minecraft.core.BlockPos)`,
`net.minecraft.world.level.block.DispenserBlock#getDispenseMethod(net.minecraft.world.level.Level,net.minecraft.world.item.ItemStack)`,
`net.minecraft.world.level.block.DispenserBlock#getDefaultDispenseMethod(net.minecraft.world.item.ItemStack)`,
`net.minecraft.world.level.block.DispenserBlock#neighborChanged(net.minecraft.world.level.block.state.BlockState,net.minecraft.world.level.Level,net.minecraft.core.BlockPos,net.minecraft.world.level.block.Block,net.minecraft.world.level.redstone.Orientation,boolean)`,
`net.minecraft.core.dispenser.DispenseItemBehavior#bootStrap()`,
`net.minecraft.core.dispenser.DefaultDispenseItemBehavior#dispense(net.minecraft.core.dispenser.BlockSource,net.minecraft.world.item.ItemStack)`,
`net.minecraft.core.dispenser.DefaultDispenseItemBehavior#consumeWithRemainder(net.minecraft.core.dispenser.BlockSource,net.minecraft.world.item.ItemStack,net.minecraft.world.item.ItemStack)`,
`net.minecraft.core.dispenser.OptionalDispenseItemBehavior#playSound(net.minecraft.core.dispenser.BlockSource)`,
`net.minecraft.core.dispenser.ProjectileDispenseBehavior#execute(net.minecraft.core.dispenser.BlockSource,net.minecraft.world.item.ItemStack)`,
`net.minecraft.core.dispenser.EquipmentDispenseItemBehavior#dispenseEquipment(net.minecraft.core.dispenser.BlockSource,net.minecraft.world.item.ItemStack)`,
`net.minecraft.core.dispenser.BoatDispenseItemBehavior#execute(net.minecraft.core.dispenser.BlockSource,net.minecraft.world.item.ItemStack)`,
`net.minecraft.core.dispenser.MinecartDispenseItemBehavior#execute(net.minecraft.core.dispenser.BlockSource,net.minecraft.world.item.ItemStack)`,
`net.minecraft.core.dispenser.SpawnEggItemBehavior#execute(net.minecraft.core.dispenser.BlockSource,net.minecraft.world.item.ItemStack)`,
`net.minecraft.core.dispenser.ShulkerBoxDispenseBehavior#execute(net.minecraft.core.dispenser.BlockSource,net.minecraft.world.item.ItemStack)`,
`net.minecraft.core.dispenser.ShearsDispenseItemBehavior#execute(net.minecraft.core.dispenser.BlockSource,net.minecraft.world.item.ItemStack)`,
`net.minecraft.core.dispenser.FlintAndSteelDispenseItemBehavior#execute(net.minecraft.core.dispenser.BlockSource,net.minecraft.world.item.ItemStack)`,
`net.minecraft.core.dispenser.SulfurCubeBlockDispenseItemBehavior#dispenseBlock(net.minecraft.server.level.ServerLevel,net.minecraft.core.BlockPos,net.minecraft.world.item.ItemStack)`,
`net.minecraft.world.level.block.entity.DispenserBlockEntity#getRandomSlot(net.minecraft.util.RandomSource)`,
`net.minecraft.world.level.block.entity.DispenserBlockEntity#insertItem(net.minecraft.world.item.ItemStack)`,
`net.minecraft.world.inventory.DispenserMenu#quickMoveStack(net.minecraft.world.entity.player.Player,int)`,
`net.minecraft.client.renderer.LevelEventHandler#levelEvent(int,net.minecraft.core.BlockPos,int)`;
`reports/blocks.json#minecraft:dispenser`,
`reports/registries.json#minecraft:block_entity_type/minecraft:dispenser`,
`reports/minecraft/components/item/dispenser.json`, bundled dispenser block/item models,
`data/minecraft/loot_table/blocks/dispenser.json`,
`data/minecraft/tags/item/sulfur_cube_swallowable.json` and its twelve nested archetype tags;
`EXP-ITM-015`.

**Test vectors:**

All twelve states; power at self/above, retained one-tick pulse, repeated callbacks and latch-write
failure; wrong/missing entity; zero/one/nine occupied slots and pending loot; all 80 explicit
entries plus feature-disabled, equippable, sulfur-tagged, encoded/unencoded spawn egg and default;
every matrix success/failure/fallback, null/failed entity admission and captured-state replacement;
remainder with selected count 1/2, empty/merge/full nine slots and duplicate event trace; brush and
TNT cross-dispenser sticky-failure sequences; menu unlocked/locked/spectator/wrong subtype;
comparator, items/loot/components/removal; every client event ID, exact ordering and RNG trace.
