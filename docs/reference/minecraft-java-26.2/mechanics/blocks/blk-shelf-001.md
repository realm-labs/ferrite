# Blocks mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `BLK-SHELF-001` — Shelves own powered side chains, direct stack swaps, directional occupancy output, and displayed contents

**Parent:** `BLK-001`, `BLK-002`, `BLK-003`, `BLK-007`, `PLY-005`, `RED-001`, `RED-003`, `ITM-001`,
`ITM-003`, `ENV-001`, `ENV-002`, `ENV-005`, `CLI-006`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked server/client source, all 12 block/item reports, the shared block-entity
registry entry, exact wooden-shelves tags, loot tables, blockstates and models determine every
shelf-owned branch. Generic placement/breaking, mutation flags, container removal drops, comparator
propagation, fire scheduling, fluid flow and resource admission retain their named owners.

**Applies when:**

Any acacia, bamboo, birch, cherry, crimson, dark-oak, jungle, mangrove, oak, pale-oak, spruce or
warped shelf is placed, powered, connected, clicked, automated, compared, saved, synchronized,
broken or rendered.

**Authoritative state:**

Every variant has horizontal `facing`, Boolean `powered`, `side_chain` in
`unconnected/right/center/left`, and Boolean `waterlogged`: 64 states each and 768 total. Default is
north, false, unconnected, false. All have destroy time 2, explosion resistance 3, bass instrument,
shelf sound, lava ignition and their source wood's map color; the ten non-Nether woods are fire fuel
with ignite 30/burn 20, while crimson and warped are not registered as fire fuel. Each matching item
has max stack 64 and default empty `CONTAINER`. The shared entity has exactly three item slots and
one NBT-only `align_items_to_bottom` Boolean, with no menu or ticker.

**Transition and ordering:**

Placement establishes facing/power/water before its `onPlace` side-chain transaction, then loads
permitted typed raw entity data and applies item components. Power writes synchronously rebuild or
disconnect parts before the outer sound/game event. Unpowered use swaps one selected stack; powered
use maps the stored connected list to the final `3N` hotbar slots. Each content change dirties
comparator state before its game event and update packet; removal drops entity contents before
deletion, then performs block loot and neighbor follow-ups through the generic owners.

**Shape, placement, water, and paths:**

North shape is the union of boxes `(0,12,11)..(16,16,13)`, `(0,0,13)..(16,16,16)` and
`(0,0,11)..(16,4,13)`, horizontally rotated for facing; it is also used for light occlusion.
Placement faces opposite the player's horizontal direction, samples current neighbor power and water
fluid, and starts unconnected. Rotate/mirror transform only facing. A waterlogged shelf exposes a
nonfalling water source, schedules the normal water delay on every shape update, and is
water-pathfindable; an unwaterlogged shelf is not pathfindable by any computation type.

**Power and canonical side chains:**

Server neighbor change samples `hasNeighborSignal`; equality is a no-op. A rising edge writes
powered true while preserving the existing part, then normal `onPlace` connects only if that part
and the prior state do not mark an in-progress neighbor update. A falling edge writes powered false
and forcibly unconnected. Each flags-3 write runs `onPlace` before the caller emits its sound/event:
powered states connect, unpowered states disconnect their immediate lateral neighbors. The outer
caller then broadcasts `block.shelf.activate` or `block.shelf.deactivate` at volume/pitch 1 and
emits `BLOCK_ACTIVATE`/`BLOCK_DEACTIVATE` with the new state. Client neighbor callbacks do nothing.

Connectability requires a powered member of exact block tag `#minecraft:wooden_shelves` with
identical facing; variants may mix. Relative left is `facing.clockWise`, right is
`facing.counterClockWise`. Power-up measures each adjacent stored chain, admits left first if its
size plus the current size is at most three, then admits right under the remaining capacity. Normal
one/two/three-block results are respectively unconnected; left/right endpoints; or left/center/right
in physical list order. Connection/disconnection changes only the affected parts through nested
flags-3 writes; already-connected new state or previously connectable+connected old state suppresses
recursive rebuilding. Stored noncanonical parts are authoritative: traversal includes at most two
lateral steps, admits a neighbor only when its part points toward the chain, and stops at an
unconnectable cell or endpoint. Removal always asks both immediate neighbors to disconnect using the
removed facing/part.

**Hit selection and unpowered swap:**

Only main-hand use, a matching shelf entity and a hit on the exact facing side are eligible. The one
row has three equal horizontal sections: north uses `1-relativeX`, south uses `relativeX`, west uses
`relativeZ`, and east uses `1-relativeZ`; section division floors and clamps to slots 0..2. Offhand,
wrong face or missing/wrong entity returns pass. The client predicts pass when its selected hotbar
stack is empty and success otherwise, without inspecting shelf contents or powered state.

On the server, an unpowered shelf swaps the entire selected stack with the hit slot; the shelf
setter limits ordinary input to `min(99,item max)`. If the player has infinite materials and the
shelf slot was empty, the selected stack is copied back, duplicating the placed stack; otherwise the
removed shelf stack becomes the selected stack. Inventory dirtying precedes shelf dirtying. Shelf
dirtying marks chunk/comparator state, optionally emits `ITEM_INTERACT_FINISH`, then sends a
same-state flags-3 block update. The event is suppressed only when the stack returned to the player
has `USE_EFFECTS` with `interact_vibrations=false`. Taking plays `block.shelf.take_item`; replacing
nonempty with nonempty plays `block.shelf.single_swap`; filling empty plays
`block.shelf.place_item`, all at volume/pitch 1. Empty-with-empty still dirties, updates and
normally emits the game event, but returns pass and plays no sound. Other completed cases return
success whose held-item-transform field is the original input object, even though the selected slot
was replaced manually.

**Powered hotbar swap:**

A powered use ignores hit slot after eligibility and traverses the stored connected list from
physical left to right. For chain length `N=1..3`, its three-slot shelves swap whole stacks with
hotbar indices `9-3N .. 8`: one shelf maps 6..8, two map 3..8, three map 0..8. Each shelf is
processed slot 0..2; empty/empty pairs do not count as swaps. A missing entity at a listed shelf
skips that entire triplet without changing `N` or the later index mapping. After every present
shelf, even one with no changed pair, inventory is dirtied and that shelf emits `ENTITY_INTERACT`
plus its same-state update. If no pair changed, the result is consume with no sound. Otherwise the
initiating position plays `block.shelf.multi_swap`; result is plain success when the selected-stack
object is unchanged, or success transformed to its new object. Infinite-material ability does not
alter this powered exchange.

**Container, components, synchronization, and removal:**

The container accepts every item type; ordinary `setItem`/nonempty `removeItem` call the no-argument
dirty path, which emits `BLOCK_ACTIVATE` and an update. Still-valid requires the same live entity
and normal block-interaction range plus four. Load clears all three slots and accepts only encoded
indices 0..2. Save always writes `Items` and `align_items_to_bottom` and includes generic residual
components; update packet/tag sends only those two shelf fields. Item component application copies
the first three `CONTAINER` positions or empties them by default and ignores later positions;
positive nonimplicit components remain residual. Collection exposes residual components plus the
three-slot container, and legacy `Items` is discarded after component migration. Generic item
placement applies the default empty container after typed raw data, so raw `Items` alone is erased
while raw alignment remains. Pick/clone is a plain variant item and does not collect contents.
Server removal with side effects drops each slot through generic randomized container splitting
before the entity disappears; each of the 12 loot tables independently drops only its matching shelf
through `survives_explosion`, never container/alignment data.

**Comparator output and rendering:**

The shelf always advertises analog output, but returns zero on the client, for a missing/wrong
entity, or unless the query direction equals `facing.opposite`. Otherwise slot occupancy is the
three-bit value `slot0 | slot1<<1 | slot2<<2`, yielding 0..7 independently of counts/components.
Dirtying and removal request generic comparator-neighbor updates.

Locked blockstates render a facing-rotated body plus exactly one overlay: unpowered ignores stored
part, while powered selects unconnected/left/center/right; waterlogging is not a model selector. The
entity renderer creates one `ON_SHELF` item state for each nonempty slot using seed
`long2int(pos.asLong())+slot`, the entity as item owner and normal resource admission. It translates
to block center, rotates Y by negative facing yaw, offsets X by `(slot-1)*0.3125`, Z by `-0.25`,
optionally Y by `-0.25`, and scales by `0.25`. Bottom alignment then raises by `-modelBounds.minY`;
normal alignment instead centers the model bounds vertically. Submissions use local light, no
overlay and layer 0.

**Branches and aborts:**

All 12 IDs/768 states, normal and authored part chains, mixed variants/facings and power edge order;
water/shape/path cases; hand/face/side/entity/client gates; every selected/slot count, empty pair,
infinite-material and vibration-suppression route; 1/2/3-chain hotbar maps and selected slot
positions; automation, malformed persistence/component positions, clone/removal/explosion;
comparator directions/occupancies and render bounds/alignment/resources.

**Constants and randomness:**

State, shape, chain, slot, signal, sound and render constants are above. Shelf logic consumes no
RNG. Generic fire, breaking loot, removal drops and item-model resource selection own their streams;
the renderer seed is deterministic.

**Side effects:**

Nested state/part writes; water scheduling; sounds and game events; selected/hotbar/container stacks
and dirty state; comparator notifications; block updates/packets; randomized removal drops and block
loot; client model and stored-item submissions.

**Gates:**

Exact shelf/tag membership, logical side, neighbor signal, facing and stored part/capacity, main
hand and hit face/section, current entity subtype, player ability, returned-stack use effects,
container validity, query direction, component codecs, mutation side-effect flags, explosion
survival and client resources.

**Boundary cases and quirks:**

Rising power preserves authored part and may skip rebuilding; normal falling power resets it. Left
admission has priority when both existing chains cannot fit. Empty/empty unpowered use returns pass
after observable dirty/event/update work, while an all-empty powered use returns consume after one
event/update per connected shelf. Client empty-hand prediction does not inspect a removable shelf
stack. Creative duplication exists only when placing into an empty unpowered slot. One powered shelf
controls hotbar 6..8 rather than the selected slot, and the directional comparator is a bitmask
rather than fullness. Raw items lose to the default empty item component, alignment is raw-only,
clone/loot omit contents, and removal drops them separately.

**Evidence:**

`Confirmed`; `OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-DATA-001`; `OFF-REPORT-001`. Anchors:
`net.minecraft.world.level.block.ShelfBlock`, `net.minecraft.world.level.block.SideChainPartBlock`,
`net.minecraft.world.level.block.SelectableSlotContainer`,
`net.minecraft.world.level.block.entity.ShelfBlockEntity`,
`net.minecraft.world.level.block.state.properties.SideChainPart`,
`net.minecraft.client.renderer.blockentity.ShelfRenderer`; locked reports, block/item wooden-shelves
tags, 12 loot tables, blockstates and models; `EXP-BLK-013`.

**Test vectors:**

Exhaust 12 IDs/768 states and shape rotations; canonical/noncanonical lateral layouts across power
edges/removal; exact hit-coordinate boundaries and every single-swap result/event/sound including
client disagreement; all connected-list lengths/parts and hotbar mappings;
save/update/component/automation/clone/drop/loot matrices; eight occupancy/direction outputs; item
renderer positions, bounds, seeds and alignment. Run `EXP-BLK-013` as the executable matrix.
