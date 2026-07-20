# Blocks mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `BLK-DECORATED-POT-001` — Decorated pots own one-stack storage, four faces, shattering, and wobble

**Parent:** `BLK-001`, `BLK-002`, `BLK-003`, `BLK-005`, `BLK-007`, `PLY-005`, `RED-003`,
`ITM-001`, `ITM-003`, `ITM-004`, `ITM-006`, `ENT-001`, `ENT-004`, `CLI-006`,
`ENV-001`, `ENV-002`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked server/client source, block/item/registry reports, recipes, tags and block
loot fix all 16 states, placement and fluid projection, insertion/failure routing, single-stack
storage, comparator dirties, persistence/components, shattering and projectile branches, four-face
crafting and pattern mapping, drops, wobble events and rendering. Generic interaction precedence,
loot-table evaluation, container automation, block-event admission, entity-item admission and effect
delivery remain under their shared owners; this rule fixes the pot-specific inputs and ordering.

**Applies when:**

A decorated pot is placed, loaded, synchronized, cloned, crafted, rendered, queried by a comparator,
used with an item or empty hand, changed through its one-slot container, hit by a projectile, broken,
removed, or receives a fluid-shape update.

**Authoritative state:**

The block has horizontal `facing` × `cracked` × `waterlogged`, exactly 16 states. Its default is
north/false/false. The block entity owns four ordered optional item identities
`(back,left,right,front)`, one item stack, an optional lazy loot-table key and seed, and transient
`wobbleStartedAtTick` plus nullable `lastWobbleStyle`. New entities start with empty decorations,
empty item, no loot table, zero wobble start and null style. The block has zero destroy/explosion
strength, destroy piston reaction, no occlusion, no pathfinding and a centered 14-wide column shape
from Y 0 through 16.

**Transition and ordering:**

Placement fixes facing/water state and transfers item components through the generic block-item
transaction. Server insertion resolves any lazy loot first, validates stack compatibility/capacity,
queues a positive wobble, awards the used-item stat, consumes one unit, mutates storage, emits sound
and particles, marks the entity changed, updates comparator neighbors and emits `BLOCK_CHANGE`.
Every server rejection then runs empty-hand fallback, which emits failure sound, queues a negative
wobble and emits the same game event without changing storage. Breaking or projectile destruction
may first write `cracked=true`; removal then ejects stored content before the cracked/uncracked block
loot projection is evaluated.

**Placement, state, shape, and fluid:**

Placement selects `facing=context.getHorizontalDirection`, sets `waterlogged` exactly when the
target fluid is water, and always clears `cracked`. Rotation transforms facing; mirror applies its
derived rotation; neither changes the other properties. A waterlogged shape update schedules water
at the fluid's level-specific delay before delegating to the base block update. Its fluid projection
is a nonfalling water source when waterlogged and the base projection otherwise. The outline and
collision-owned block shape is the centered X/Z interval `1..15` for the full `0..16` height.

The cracked property changes only the selected sound type and shattering loot path; it does not
change shape, fluid, facing, storage, comparator output or render textures. Ordinary and cracked
sound types supply their distinct break/step/place/hit/fall resources; insertion and failure use
their dedicated sounds below.

**Item insertion and failure routing:**

`useItemOn` returns `PASS` when the block entity is absent or wrong. With a matching client entity it
returns `SUCCESS` immediately, without checking the hand stack or mutating/sounding locally. The
server first calls `getTheItem`, which unpacks a pending loot table with null player context. A
nonempty hand is admitted only when storage is empty, or its item and all components equal the hand
and its count is strictly below that stack's maximum. Admission then:

1. queues wobble event `(1, POSITIVE.ordinal=0)`;
2. awards `ITEM_USED` for the hand item;
3. obtains one unit through `consumeAndReturn(1,player)`;
4. stores that returned unit if empty, otherwise grows the existing live stack by one;
5. computes fill `storedCount/storedMax` from the resulting stack;
6. plays `block.decorated_pot.insert` in `BLOCKS`, volume 1, pitch `0.7+0.5*fill`;
7. sends seven `DUST_PLUME` particles from `(x+0.5,y+1.2,z+0.5)` with zero spread/speed;
8. calls `setChanged`, then emits source-player `BLOCK_CHANGE`, and returns `SUCCESS`.

`consumeAndReturn` owns survival versus infinite-material hand disposition; the pot always stores
exactly the returned one-unit value on its empty branch. An empty hand, unequal item/component set,
or full stack returns `TRY_WITH_EMPTY_HAND`. The resulting matching-entity `useWithoutItem` always
plays `block.decorated_pot.insert_fail` at volume/pitch 1, queues `(1,NEGATIVE.ordinal=1)`, emits
source-player `BLOCK_CHANGE`, and returns `SUCCESS`, even when the pot itself is empty. Thus a client
with a matching entity predicts success for every item while only the server chooses insert versus
failure; the server's sound/event supplies the visible resolution.

**One-slot container, comparator, persistence, and components:**

The entity is a size-one container. Slot zero reads, splits and writes the owned stack; other slots
read empty and ignore writes/removals. Every read, split or write resolves pending loot first with a
null player. Split normalizes an exhausted live stack back to the canonical empty stack. Generic
container callers own their subsequent dirty notification; the direct interaction explicitly calls
`setChanged`. That call marks the chunk and updates neighboring comparator outputs. The output is
the generic one-slot fullness signal: zero when empty, otherwise
`floor(14*count/maxStackSize)+1`; query direction is ignored. Removal also explicitly refreshes
neighboring output.

Save writes `sherds` only when decorations are nonempty. It writes a loot table/seed instead of an
item while lazy loot is present; otherwise it writes `item` only when nonempty. Load defaults absent
or invalid decorations/item to empty, and a loaded loot table forces the live item empty until
unpacked. The update packet is the ordinary block-entity data packet and its tag is the full custom
save, so decorations plus materialized item or pending loot metadata are synchronized by callers
that request it. Wobble fields are neither saved nor componentized.

The implicit item projection always contains `pot_decorations` and a one-entry `container`
projection. Applying components defaults missing decorations/container to empty and copies one stack
from the container component. Raw `sherds`/`item` fields are removed after component transfer.
Picking the block creates one decorated-pot item with only `pot_decorations`; it never copies stored
content or lazy loot. Uncracked block loot likewise copies only `pot_decorations`, while generic
block-entity pre-removal separately ejects the materialized one-slot content.

**Decorations, recipes, registry mapping, tooltip, and render faces:**

The component codec accepts at most four item IDs in `(back,left,right,front)` order. A missing list
position or `minecraft:brick` becomes an empty optional face; encoding maps every empty face back to
brick and always produces four entries. `PotDecorations.EMPTY` therefore projects as four bricks,
matching the decorated-pot item's default component. Tooltip output is absent only for `EMPTY`;
otherwise it adds an empty line followed by gray item names in the distinct order
`front,left,right,back`, substituting brick for empty faces.

The locked special recipe requires a 3×3 input with exactly four ingredients at top/left/right/
bottom center. These become back/left/right/front respectively and every ingredient must be in
`decorated_pot_ingredients`, exactly brick plus the 23 pottery sherds. The shaped simple recipe uses
brick at those same four cells and inherits the default all-blank component. The special serializer
is protocol ID 5. Recipe matching/consumption/remainders remain with `ITM-CRAFT-001`; this rule fixes
the exact shape, orientation and result component.

The `decorated_pot_pattern` registry has protocol ID 60. IDs 0..22 in order are `angler`, `archer`,
`arms_up`, `blade`, `brewer`, `burn`, `danger`, `explorer`, `flow`, `friend`, `guster`, `heart`,
`heartbreak`, `howl`, `miner`, `mourner`, `plenty`, `prize`, `scrape`, `sheaf`, `shelter`, `skull`,
and `snort`; ID 23 is `blank`. Each named sherd maps to its same-named pattern and asset
`<name>_pottery_pattern`; brick, absent faces and unrecognized item IDs render the blank
`decorated_pot_side`. Block rendering submits base neck/top/bottom then front/back/left/right sprites
from the four stored identities. Facing rotates that model about block center by
`180°-facing.toYRot`. Item special rendering reads only `pot_decorations`, defaults a missing
component to `EMPTY`, and uses the same side submission without world-facing rotation.

**Shattering, projectiles, drops, and content ejection:**

During player destruction, a main-hand item in `breaks_decorated_pots` writes `cracked=true` with
flags 260 before the generic pre-destroy callback unless the tool has an enchantment in
`prevents_decorated_pot_shattering`; that prevention tag contains only `silk_touch`. The breaking
tag expands swords, axes, pickaxes, shovels and hoes plus trident and mace. The hook returns the
possibly cracked state to the generic break transaction.

A projectile hit acts only on a `ServerLevel` and requires both `mayInteract(level,pos)` and
`mayBreak(level)`. It writes cracked with flags 260, then immediately calls
`destroyBlock(pos,true,projectile)`; it does not consult the tool or prevention-enchantment tags and
does not skip the write when already cracked. Failed permissions and every client hit do nothing.

Cracked block loot selects the dynamic `minecraft:sherds` drop and emits four unit stacks in stored
back/left/right/front order, substituting brick for every empty face. Uncracked loot emits one
decorated pot carrying the decoration component. The alternatives select exactly one path and the
locked table has no explosion-survival condition. Independently, block-entity pre-removal resolves
lazy loot if needed and ejects its one stored stack through generic item-entity admission. Loot
evaluation, item-entity offsets/velocity and failed entity admission retain their shared owners.

**Wobble event and locked client rendering quirk:**

`wobble(style)` is server-only and queues block event ID 1 with the style ordinal. Event execution
accepts only ID 1 and parameter 0 or 1 with a nonnull level; it records current game time and the
selected style, then returns true for broadcast. Other values delegate. Positive duration is seven
ticks and negative duration ten. Repeated identical records may deduplicate in the shared block
event set, while each ingress has already committed its item/sound/particle/stat/game-event effects.

The client render extractor copies decorations and facing and computes
`progress=(gameTime-wobbleStartedAtTick+partialTick)/style.duration` when a style exists, otherwise
zero. In the locked 26.2 client it does **not** copy that style into the render state's
`wobbleStyle` field. A fresh render state therefore retains null, so even positive events take the
renderer's non-positive yaw branch rather than its otherwise present X/Z positive branch. For both
styles, while progress is inclusively 0..1, the visible rotation is about the block center around Y:
`sin(-3*PI*progress)*0.125*(1-progress)`. The observable difference is duration 7 versus 10. After
progress exceeds 1 the stored event fields remain but no transform applies; a later accepted event
restarts the clock. This source-fixed quirk is part of exact 26.2 client compatibility.

**Branches and aborts:**

All 16 states; every facing/rotation/mirror and water transition; missing/wrong entity; client and
server use; empty, equal, component-unequal and full stacks; survival/infinite-material disposition;
lazy loot resolution; direct and automated slot access; comparator values 0..15; absent/present raw
and component fields; every four-face brick/sherd/arbitrary-item tuple; both recipes; normal,
silk-touch and tagged-tool breaks; already-cracked state; projectile side/permissions; stored content
and failed ejection; block events with every ID/parameter; rerings and render progress boundaries.

**Constants and randomness:**

Sixteen states; shape 14×16; one slot; event ID 1; style ordinals 0/1 and durations 7/10; insertion
volume 1 and pitch `0.7+0.5*fill`; failure volume/pitch 1; seven dust particles at Y+1.2; shattering
flags 260; four decoration positions; registry protocol ID 60 and pattern IDs 0..23; recipe
serializer ID 5. No pot-owned branch consumes RNG. Lazy loot, generic sound seed, block-event set,
item-entity creation and particle delivery own any downstream randomness/order.

**Side effects:**

Block/fluid writes and schedules; item/stat mutation; lazy loot resolution; entity dirty and
comparator notifications; one of two sounds; dust particles; `BLOCK_CHANGE`; block-event broadcast;
stored-content ejection; normal or sherd loot; update tags; tooltips and block/item render submissions.

**Gates:**

Placement and component admission; logical side; matching block entity; hand stack identity,
components and capacity; container slot; lazy-loot server availability; projectile interaction/break
permission; tool/enchantment tags; cracked state at loot evaluation; block-event activity/current
block; client resource mapping and render progress. Difficulty and game rules do not directly gate
this slice.

**Boundary cases and quirks:**

The client reports success for every item before the server knows whether insertion will fail. Empty
hand use always produces the failure effect, even on an empty pot. Arbitrary items can be stored but
only an identical item/component stack can extend them. Pick block and uncracked loot preserve faces
but not contents; contents eject separately. Cracked loot produces four bricks for an undecorated
pot. Tooltip order differs from encoded/render submission order. Renderer state loses the wobble
style, making both ingress styles use the yaw formula with different durations.

**Evidence:**

`Confirmed`; `OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-DATA-001`; `OFF-REPORT-001`. Anchors:
`net.minecraft.world.level.block.DecoratedPotBlock`,
`net.minecraft.world.level.block.entity.DecoratedPotBlockEntity`,
`net.minecraft.world.level.block.entity.PotDecorations`,
`net.minecraft.world.level.block.entity.DecoratedPotPatterns`,
`net.minecraft.world.item.crafting.DecoratedPotRecipe`,
`net.minecraft.client.renderer.blockentity.DecoratedPotRenderer`,
`net.minecraft.client.renderer.special.DecoratedPotSpecialRenderer`; locked block/item/registry
reports, both decorated-pot recipes, ingredient/break/prevention tags and block loot; `EXP-BLK-014`.

**Test vectors:**

Exhaust all 16 state IDs, facing transforms, shape/fluid behavior and placement components; every
client/server item/empty-hand branch with counts 0/1/max-1/max and component equality; lazy loot and
one-slot operations with comparator output 0..15; raw/component save/load/sync/clone projections;
all 24 ingredient identities in every face, codec lengths 0..5, tooltip/render orders and registry
IDs; exact and malformed recipe grids; tagged/silk/projectile crack paths, both loot alternatives and
stored-content ejection; event IDs/parameters, dedup/rering, partial progress around 0/1, and the
null-style yaw quirk at both durations. Run `EXP-BLK-014` as the executable matrix.
