# Blocks mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `BLK-LECTERN-001` — Lecterns own book insertion, page menus, two-tick pulses, analog output, removal ejection, and state/content divergence

**Parent:** `SIM-003`, `BLK-001`, `BLK-002`, `BLK-003`, `BLK-005`, `BLK-007`, `PLY-005`, `RED-001`,
`RED-003`, `ITM-001`, `ITM-002`, `ITM-003`, `ENT-001`, `CLI-005`, `CLI-006`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked server source, block/item/registry reports, the exact two-member
lectern-books tag, and locked block loot determine every lectern-owned transition. Written-page
component parsing/rendering, generic interaction routing, item consumption, entity admission, menu
synchronization, neighbor propagation, and loot evaluation remain at their named parent boundaries
rather than being redefined here.

**Applies when:**

A lectern is placed, queried or interacted with; a tagged book is inserted, read, paged or removed;
its scheduled pulse expires; comparator/direct signal is queried; content saves, loads or is
cleared; or the block entity is removed.

**Authoritative state:**

The block has exactly 16 states: four horizontal `facing` values × `has_book` × `powered`,
defaulting north/false/false. It is wood-map-colored, bass-instrumented, lava-ignitable, strength
`2.5`, wood-sounding and nonpathfindable. Collision and occlusion are a full-width Y `0..2` base
plus centered 8-wide Y `2..14` column. The facing outline additionally rotates three full-width Z
bands: Y `10..14`, Z `1..5.333333`; Y `12..16`, Z `5.333333..9.666667`; and Y `14..18`, Z
`9.666667..14`. The collision shape is used for light occlusion. The block entity separately owns
one stack, zero-based signed page and derived page count; `hasBook()` means the stack has writable-
or written-book-content, not that it is nonempty or agrees with block state.

**Transition and ordering:**

Placement first builds the raw facing/state candidate before generic component transfer. Interaction
chooses insertion versus empty-hand/menu fallback from the captured state. Successful server
insertion stores and dirties content before requesting block state/event/update/sound. Menu page
mutation requests power, schedules its falling edge and emits the level event in that order. Removal
side effects inspect captured block state and actual content before generic block loot. The detailed
branches and failure boundaries follow.

**Placement and interaction routing:**

Placement faces opposite the placer. Its raw candidate has `powered=false`; only a server-side
nonnull player allowed to use game-master blocks can make raw `has_book=true`, and only when the
item has block-entity data whose typed payload contains a `Book` field. This test does not validate
that field or inspect the later transferred stack; generic placement component/block-entity
application owns the resulting divergence. Rotate/mirror transform only facing.

With captured `has_book=true`, item-on-block requests empty-hand fallback. With it false, an item in
exact tag `#minecraft:lectern_books` (`minecraft:writable_book`, `minecraft:written_book`) attempts
insertion and returns success on both sides; a main-hand empty stack passes immediately; every other
stack requests empty-hand fallback. The generic outer router alone owns secondary-use bypass and the
fact that only main-hand `TRY_WITH_EMPTY_HAND` reaches `useWithoutItem`. There, state true returns
success and opens on the server only; state false returns consume, so a nonbook main-hand item can
be prevented from reaching item use while an explicit empty main hand already passed. A state-true
matching subtype opens `LecternMenu`, titled `container.lectern`, then awards
`minecraft:interact_with_lectern`; a missing/wrong subtype still leaves the block result successful
but performs neither operation. The state menu provider is null when `has_book=false`, including the
spectator provider path.

**Insertion transaction and book resolution:**

`tryPlaceBook` tests only the captured false state. The client mutates nothing and returns true. The
server returns true even if the block entity is missing/wrong; that mismatch consumes nothing and
emits nothing. With a matching subtype, order is: take a count-one copy through generic
`consumeAndReturn` (an infinite-materials player does not shrink the source), resolve and store it,
reset page to zero/derive page count/mark dirty, offer state `{powered=false,has_book=true}` with
flags 3, emit source-context `BLOCK_CHANGE` carrying that requested new state, update the position
below, then play `BOOK_PUT` in `BLOCKS` at volume/pitch 1. State-write failure rolls back none of
the earlier content/consumption or later event/update/sound.

On a server, an unresolved written book is resolved with a block-center, zero-rotation command
source at game-master permission: inserting player plain/display name and entity when present,
otherwise literal `Lectern` and null entity for load/direct calls. Writable content is unchanged.
The component resolver owns page-component evaluation; its success stores resolved content, while
its caught failure retains content marked resolved. Client-side `setBook` never resolves. Public
`setBook`, including `clearContent`, updates only content/page/count/dirty state and does not repair
block state.

**Container, menu, and page pulse:**

The private container has size/max stack 1, exposes only slot 0, rejects placement, and is valid
only while the generic block-entity validity test and actual component-based `hasBook()` both pass.
`removeItem` splits and resets the block only when the remainder becomes empty; `removeItemNoUpdate`
returns the whole stack and resets even if already empty; container `setItem` and its `clearContent`
are no-ops. Removal resets page/count to zero and calls the state reset with null source and
`has_book=false`; it does not itself call the block entity's dirty method.

The lectern menu has one slot and one data value; quick-move always returns empty. Button IDs
`>=100`, `2`, and `1` request page `id-100`, current+1, and current-1 respectively and return true
even when clamping makes no change. Page is clamped by `min(max(request,0),pageCount-1)`: a
zero-page stack therefore changes page `0` to `-1`. Only a changed value marks dirty, powers the
captured state, schedules a deduplicated lectern tick at delay 2, and emits level event `1043`.
Repeated page changes before the due tick do not extend that scheduled pulse. Button `3` returns
false without `mayBuild`; otherwise it removes without update, marks dirty, adds the returned stack
to inventory or drops it with `throwRandomly=false` on failure, and returns true. Every other button
returns false. `setData` performs generic data application then immediately broadcasts changes; menu
validity follows the private container.

Page change first offers captured `powered=true` with flags 3, updates below using that captured
state, schedules the captured block type, then emits event `1043`. The due callback uses its live
captured state, offers `powered=false` with flags 3 and updates below. Deduplication,
collected-batch behavior, replacement before due time and failed writes remain `SIM-003`/generic
update boundaries. Resetting content also forces requested power false and updates below. Every
below update uses orientation initialized from `facing.opposite` with vertical-up bias and calls
neighbor update at `pos.below` using the lectern block. Removing a powered state performs the same
below update; removing an unpowered one does not.

**Signals and divergence:**

Ordinary signal is 15 on all queried sides exactly while `powered`; direct signal is 15 only for
query direction up; otherwise each is 0. Analog output is zero unless captured state says
`has_book=true` and the current block entity is a lectern. A matching subtype returns
`floor(14 * progress) + (actual hasBook ? 1 : 0)`, where progress is `page/(pageCount-1)` only when
count exceeds one and otherwise exactly 1. Normal one/zero-page component books therefore output 15;
a multipage book maps first/last to 1/15. A deliberately divergent state-true but componentless
subtype outputs 14, while state false suppresses even real content.

**Persistence, block removal, item, and loot:**

Load decodes optional `Book`, resolves it with the null-player server context, derives count, and
applies the same clamp to saved `Page` default 0, including `-1` for zero pages. Save writes both
`Book` and `Page` only when the stack is nonempty; a nonempty componentless stack is therefore
preserved. State/content are never reconciled on load. Before removal, captured `has_book=true` plus
a nonnull level copies the actual stack and offers an item entity at block center shifted `0.25` in
facing, Y+1, with default pickup delay; admission result is ignored and content is not cleared.
State false ejects nothing even when content exists; true may offer an empty/componentless stack.
The default lectern item has max stack 64. Locked block loot independently yields one lectern
through `survives_explosion` and copies no book, page or state, so ordinary removal can eject
content before generic block loot without duplication from the table.

**Branches and aborts:**

All 16 states/facings and shapes; raw operator data gate; client/server; hand, secondary-use and
item tag; captured state versus current subtype/content; infinite-material consumption;
written/writable/resolved/failing resolution; every state/admission write result;
partial/whole/empty removal; all menu IDs, permissions, clamps and inventory admission; scheduled
dedup/replacement; signal direction and all state/content divergence; save/load/clear/removal and
explosion-loot outcomes.

**Constants and randomness:**

Pulse delay 2; event 1043; signal 15; comparator scale 14; insertion sound 1/1; ejection offset 0.25
and Y+1; state flags 3; shapes and permission above. Lectern-owned logic consumes no RNG. Generic
item-entity construction/drop motion, sound seed, component resolution and loot evaluation own their
streams.

**Side effects:**

Stack consumption/storage, dirty marks, block writes, game/level events, scheduled work, oriented
neighbor updates, menu/data synchronization, stat, sound, optional inventory/drop and removal item
entity, save data and block loot.

**Gates:**

Current captured state, logical side, outer interaction priority, exact item tag, player ability,
current subtype, component presence/page count, menu permission/validity, scheduled tick
identity/activity, signal query direction, removal state and explosion survival.

**Boundary cases and quirks:**

Success can be reported without a matching block entity. Captured-state writes and events can
describe a lectern no longer present. State and actual content intentionally diverge through raw
block-entity data, public clear/set, malformed load and failed state writes. Zero pages clamp to
`-1`; componentless state-true content outputs 14. Button success does not imply a page change, and
repeated changes do not restart the two-tick pulse. Removal eligibility is state-based rather than
content-based.

**Evidence:**

`Confirmed`; `OFF-SERVER-001`; `OFF-DATA-001`; `OFF-REPORT-001`. Anchors:
`net.minecraft.world.level.block.LecternBlock`,
`net.minecraft.world.level.block.entity.LecternBlockEntity`,
`net.minecraft.world.inventory.LecternMenu`,
`net.minecraft.world.item.component.WrittenBookContent#resolveForItem`; locked
block/item/block-entity/menu reports, `data/minecraft/tags/item/lectern_books.json`, and
`data/minecraft/loot_table/blocks/lectern.json`; `EXP-BLK-011`.

**Test vectors:**

Exhaust all 16 states, facings/shapes and placement operator/data combinations; every
hand/item/tag/secondary/spectator/client/subtype route; resolution and infinite-material consumption
with failed writes; content/state mismatch matrix; zero/one/many pages and every menu
ID/clamp/permission/inventory result; page changes before/equal/after the two-tick due time with
replacement/dedup; every weak/direct/comparator direction and divergence; persistence,
public/container clear, partial/whole removal, entity admission and explosion loot. Run
`EXP-BLK-011` as the executable matrix.
