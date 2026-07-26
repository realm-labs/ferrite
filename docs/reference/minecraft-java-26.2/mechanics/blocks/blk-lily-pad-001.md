# Blocks mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `BLK-LILY-PAD-001` — Lily Pads place on source water, break under boats and remain frog-preferred

**Parent:** `SIM-004`, `SIM-005`, `SIM-RANDOM-001`, `BLK-001`,
`BLK-STATE-001`, `BLK-002`, `BLK-PLACE-001`, `BLK-BREAK-001`,
`BLK-BREAK-HOOK-001`, `BLK-BREAK-CONTENT-001`, `BLK-UPDATE-001`,
`PLY-002`, `PLY-005`, `PLY-006`, `PLY-INPUT-001`,
`PLY-INTERACT-001`, `PLY-BREAK-001`, `PLY-COLLISION-001`,
`PLY-AUTOJUMP-001`, `RED-001`, `RED-UPDATE-001`,
`RED-COMPARATOR-001`, `ITM-003`, `ITM-004`, `ITM-006`,
`ITM-USE-001`, `ITM-LOOT-001`, `ITM-ANVIL-001`, `ENT-001`,
`ENT-VEHICLE-001`, `MOB-001`, `MOB-AI-001`, `ENV-001`,
`ENV-002`, `ENV-003`, `ENV-FLUID-001`, `ENV-FIRE-001`,
`ENV-LIGHT-001`, `WGEN-003`, `WGEN-PIPELINE-001`,
`WGEN-STRUCTURE-WOODLAND-MANSION-001`, `CLI-001`, `CLI-006`,
`CLI-UI-001`, `CLI-EFFECT-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked `LilyPadBlock`, inherited vegetation and
`PlaceOnWaterBlockItem` bytecode, registration, loot, trade, tag and
worldgen data, every exact compiled identity consumer, all `1,212` decoded
templates, four legacy fixes and exact client resources close the
property-free Lily Pad block and item. Its special runtime is source-water
placement and survival, server-side Boat destruction, fishing-open-water
admission, frog landing preference, two direct tag joins, fishing/trader/
feature/template acquisition, two-biome generation and one Mansion room
payload.

**Applies when:**

`minecraft:lily_pad` is placed, supported, neighbor-updated, collided with
by a Boat, walked over, path-classified, mined, exploded, piston-destroyed,
fished, composted, traded, selected by frog AI, generated, replaced by a
huge fungus, migrated, persisted, synchronized or rendered.

**Authoritative state:**

Lily Pad is a property-free `LilyPadBlock` extending `VegetationBlock`,
has no block entity and has sole block-state ID `8920`. Its block protocol
ID is `374`, block-type protocol ID is `241`, and its specialized
stack-64 `PlaceOnWaterBlockItem` has raw item ID `451`.

Registration fixes map color `PLANT`, default note instrument `HARP`,
zero hardness/resistance through `instabreak`, Lily-Pad sounds,
friction `0.6`, speed/jump factors `1`, emission `0`, no occlusion and
piston reaction `DESTROY`. It does not require a correct tool.

Selection, collision, visual and block-support shape are the centered box
`[1/16,0,1/16]..[15/16,1.5/16,15/16]`. The interaction shape is empty and
the no-occlusion cache is empty. The state is not a full collision or
occlusion cube, propagates skylight, has light dampening `0` and shade
brightness `1`. It supplies no ordinary redstone signal or comparator
output and is not an ordinary full-face conductor, view blocker,
suffocation state or spawn-support cube.

The Lily-Pad sound profile has volume/pitch `1/1`. It reuses Big-Dripleaf
break, step, hit and fall event IDs `169/173/171/170`, while its dedicated
place event has ID `1724`.

**Transition and ordering:**

### Air-use placement and support

`PlaceOnWaterBlockItem.useOn` always returns `PASS`; directly clicking a
block never invokes its generic placement path. `use` instead performs the
shared player-view raycast with fluid mode `SOURCE_ONLY`, copies that hit
with only its block position moved one block upward, constructs a
`UseOnContext`, and invokes the ordinary `BlockItem.useOn` path. The copied
hit retains its direction, precise location and inside/world-border
metadata.

Generic placement must still admit the target, world border, collision,
permissions and component patch before Lily Pad's survival predicate.
For target position `P`, the predicate reads `B=P.below()` and accepts
exactly when:

- the fluid state at `B` belongs to fluid tag `supports_lily_pad`, whose
  sole direct fluid is `minecraft:water`, or the block state at `B`
  belongs to block tag `supports_lily_pad`, whose direct values are Ice
  and Frosted Ice; and
- the fluid state at `B.above()` is exact Empty.

The fluid tag names the source `water` fluid, not `flowing_water`; a source
fluid contained by a waterlogged block can satisfy the first test.
The placement item's source-only raycast naturally reaches the water
branch. Ice/Frosted-Ice survival remains reachable through commands,
worldgen or another state-placement owner because direct item use-on-block
is deliberately `PASS`.

Every inherited vegetation shape update re-runs `canSurvive`; failure
returns Air, while success delegates to the ordinary block update. The
check is not limited to downward-neighbor notifications. Commands can
initially write unsupported state `8920`, but a later shape update can
remove it. Lily Pad has no random tick, scheduled tick, fluid-state,
placement-state, rotation, mirror, attack, fall, signal, comparator or
block-event override.

### Mining, explosion and piston behavior

Lily Pad belongs to no mining/tool tag and has no correct-tool requirement.
Zero hardness makes ordinary player breaking immediate; hand, every tool,
Silk Touch and Fortune reach the same one-roll self table. It emits exactly
one Lily Pad behind `survives_explosion` and uses random sequence
`minecraft:blocks/lily_pad`. Tool and enchantment do not alter count.
Explosion decay can suppress the result.

A piston resolves `DESTROY` rather than moving the state. Generic support
loss and piston/explosion transactions retain their owners' update and
drop rules. Lily Pad has no `FireBlock` bootstrap row, lava-ignition
property or fuel time: direct encouragement/flammability are `0/0`.

### Boat contact

`entityInside` first runs the inherited vegetation callback. It then tests
the logical level and entity independently:

- a non-server level or non-`AbstractBoat` entity makes no Lily-Pad
  mutation;
- a Boat on a `ServerLevel` calls
  `destroyBlock(new BlockPos(position), true, boat)` and ignores the
  Boolean result.

The copied immutable position prevents a mutable caller position from
changing during destruction. `true` requests ordinary block drops and the
Boat is supplied as the breaking entity. A rejected destruction does not
retry or cause another Lily-Pad-specific effect. Client contact predicts
no removal; authoritative block/loot synchronization supplies the result.

### Step sound and general path classification

Lily Pad belongs directly to `inside_step_sound_blocks`. When entity step
sound selection examines the block immediately above its initial
underfoot position and finds Lily Pad, that upper position becomes the
primary step-sound state. The ordinary step transaction therefore uses
the Lily-Pad profile and its Big-Dripleaf step event rather than silently
falling through to the support beneath it.

`WalkNodeEvaluator.getPathTypeFromState` explicitly classifies exact Lily
Pad as `TRAPDOOR`, alongside the `trapdoors` tag and Big Dripleaf. It does
so before powder-snow, damage, rail, door and ordinary passability tests.
Ordinary walking path expansion treats that type as a special
non-step-up target. Water and air path computation otherwise retain the
vegetation/default owners.

### Frog preference and landing exceptions

The complete `frog_prefer_jump_to` block tag contains Lily Pad and Big
Dripleaf, with no parent tag. It affects frogs in three exact places:

1. Frog long-jump activity constructs `LongJumpToPreferredBlock` with
   horizontal/vertical ranges `4/2`, maximum-velocity multiplier
   `3.5714288f`, the tag and preference chance `0.5f`.
2. At activity start, strict `nextFloat() < 0.5f` decides whether preferred
   candidates are wanted. In that mode, randomized candidates whose block
   immediately below is tagged are accepted first; nonpreferred candidates
   are retained and one becomes fallback only if no preferred candidate
   remains.
3. The frog landing predicate first requires empty fluid at the candidate,
   below it and above it. A tagged candidate state or tagged state directly
   below then accepts immediately; otherwise the two `TRAPDOOR` path-type
   cases precede the generic landing test.

The frog's amphibious node evaluator separately returns `OPEN` for a path
cell whose block immediately below belongs to the preference tag. Thus
Lily Pad is globally classified as `TRAPDOOR`, while a frog can treat the
space over it as open and can preferentially select it as a long-jump
landing surface. Tag reload changes future AI/path reads without mutating
state `8920`.

### Fishing loot and open-water admission

The `gameplay/fishing/junk` table has one unconditional Lily-Pad entry of
weight `17`, count one and no functions. Eligible junk weight is `100`
outside Jungle, Sparse Jungle and Bamboo Jungle; the conditional
weight-`10` Bamboo entry raises it to `110` inside those biomes. Lily Pad
therefore occupies `17/100` or `17/110` of a selected junk pool.

The root fishing table selects junk at weight `10`, quality `-2`; treasure
at weight `5`, quality `2`, only for an open-water hook; and fish at weight
`85`, quality `-1`. At zero luck, an open-water catch is consequently one
Lily Pad with probability `17/1000` outside the three Jungle biomes or
`17/1100` inside them. With treasure ineligible, those probabilities are
`17/950` and `17/1045`. Nonzero hook-plus-player luck changes the root
entry weights through their qualities but not the Lily-Pad entry's local
weight.

Exact Lily Pad also has a separate fishing-environment exception.
`FishingHook.getOpenWaterTypeForBlock` classifies Air and exact Lily Pad
as `ABOVE_WATER`; other nonair states proceed to the source-water,
empty-collision `INSIDE_WATER` test or become `INVALID`. The open-water
scan checks four `5×5` horizontal layers from Y offsets `-1..2`, requires
each layer to reduce to one type, forbids an `INSIDE_WATER` layer after an
`ABOVE_WATER` layer and rejects `INVALID`. A Lily Pad in a legitimate
above-water layer therefore does not invalidate treasure eligibility.
It is not treated as source water and cannot repair a mixed or otherwise
invalid layer.

Fishing retrieval, item entities, experience, durability, statistics and
criteria retain the fishing owner. The junk table uses random sequence
`minecraft:gameplay/fishing/junk`.

### Composter and Wandering-Trader paths

Composter bootstrap registers exact Lily Pad at Java float chance `0.65f`.
At level zero an admitted player or automated insertion succeeds without
RNG; levels `1..6` test strict `nextDouble() < 0.65f` widened to double.
Level-seven extraction, delayed conversion, item/stat/event order and
failed-attempt behavior remain with the Composter owner.

Wandering-Trader record `emerald_lily_pad` consumes one Emerald and gives
five Lily Pads, permits `2` uses, inherits XP `1` and uses reputation
discount `0.05`. It is one of `76` members of the common trade tag; the
common set selects five distinct records through random sequence
`minecraft:trade_set/wandering_trader/common`. Its exact inclusion
probability is `5/76`.

There is no locked recipe or advancement mentioning Lily Pad, no other
merchant record, and no ordinary chest or entity-death table that emits
it. Self/Boat loot, fishing, the Wandering Trader, feature/Mansion
generation, creative publication and commands are its baseline
acquisition paths. Composting is a sink.

### Complete block/item tag closures

The only direct block tags are `frog_prefer_jump_to` and
`inside_step_sound_blocks`; neither is nested by another locked block tag.
The complete block-tag closure is therefore exactly those two tags and
their consumers are the frog/path and step-sound paths above.

The Lily-Pad item belongs to no locked item tag. Live tag reload can alter
the two block-selected paths but does not change placement support:
`supports_lily_pad` contains the supporting fluid/blocks, not Lily Pad
itself.

### Natural feature generation

Configured feature `waterlily` is `simple_block` with a simple provider
for property-free state `8920`. Its `patch_waterlily` placed wrapper runs
these modifiers in exact order:

1. count `4`;
2. in-square X/Z;
3. `WORLD_SURFACE_WG` heightmap;
4. biome filter;
5. count `10`;
6. trapezoid random offset X/Z `-7..7`, Y `-3..3`; and
7. a matching-block-tag predicate requiring the final target in `air`.

The two count stages expand to at most `40` final candidates per admitted
placed-feature invocation. Each surviving candidate calls the provider,
then generic simple-block placement rechecks Lily Pad's live survival
predicate before offering state `8920`. Thus the Air filter alone does not
permit flowing-water, occupied-fluid or unsupported targets. Provider,
survival and world-write failure can independently leave no block.

Exactly two biome records schedule `patch_waterlily`, both in generation
step index `9`:

- Swamp lists it at index `5` of `15`, after `patch_dead_bush` and before
  `brown_mushroom_swamp`;
- Mangrove Swamp lists it at index `4` of `7`, after
  `patch_dead_bush` and before `seagrass_swamp`.

Modifier streams, biome scheduling, simple-block flags and partial writes
retain `WGEN-PIPELINE-001`; scheduling does not guarantee any Lily Pad.

### Huge-fungus replacement input

Each of `crimson_fungus`, `crimson_fungus_planted`, `warped_fungus` and
`warped_fungus_planted` includes exact Lily Pad in its explicit
`matching_blocks` replacement predicate. `HugeFungusFeature` enables that
predicate only for stem candidates; hat/decor candidates pass
`allowConfiguredReplacement=false` and therefore admit only Air through
the earlier common gate.

An otherwise admitted stem candidate can consequently replace Lily Pad.
In a planted configuration, a nonair state below the candidate causes
`destroyBlock(candidate,true)` before the ignored flags-`3` stem write;
every valid supported Lily Pad has such a nonair support and therefore
requests its drop. A non-planted configuration performs the feature write
without that destruction/drop call, and an expanded-stem corner still
needs strict `nextFloat() < 0.1f`. Base block, height/shape draws, candidate
kind, remaining writes and failures retain the huge-fungus feature owner.
This is a stem-replacement classification, not a normal Overworld
generation path for Lily Pad.

### Woodland Mansion payload

An exhaustive decoded scan of all `1,212` bundled structure templates
finds exactly eight raw state-`8920` cells in one file:
`woodland_mansion/1x2_a2`. They occupy template coordinates
`(3,2,6)..(3,2,13)`, have no block NBT, and share one palette entry.
Decompressed-string scanning finds exactly that one palette identity and
no additional final-state, marker, block-entity or entity-NBT occurrence.

`FirstFloorRoomCollection.get1x2SideEntrance` selects names
`1x2_a1..1x2_a9` by `nextInt(9)+1`, so `1x2_a2` has conditional
probability `1/9` whenever Mansion layout reaches that first-floor
side-entrance room selection. Room layout, transforms, clipping,
processor application, overlap and write failures remain with
`WGEN-STRUCTURE-WOODLAND-MANSION-001`; eight raw cells do not guarantee
eight final-world pads.

### Persistence and legacy migration

Ordinary chunk palettes and block-update packets preserve only state
`8920`; stacks preserve identity, count and generic component patches.
The block has no block-entity payload and the item has no Lily-Pad-specific
component.

Four contextual data-fix tables preserve the old identity:

- `BlockStateData` maps packed numeric state `1776` (`111 << 4`) and old
  name `minecraft:waterlily` to property-free
  `minecraft:lily_pad`;
- `EntityBlockStateFix` maps old entity block name
  `minecraft:waterlily` to numeric block ID `111`;
- `ItemIdFix` maps numeric item ID `111` to
  `minecraft:waterlily`; and
- `ItemStackTheFlatteningFix` maps damage-qualified
  `minecraft:waterlily.0` to `minecraft:lily_pad`.

Complete compiled exact-name search finds no other Lily-Pad-specific
migration. These fixes apply only at their owning schema boundary and do
not rerun on already-modern ordinary state or stack persistence.

### Client projection

The property-free blockstate selects the same
`minecraft:block/lily_pad` model in four equal default-weight variants
rotated around Y by `0/90/180/270` degrees. The model disables ambient
occlusion and draws a two-sided horizontal plane from
`[0,0.25/16,0]` to `[1,0.25/16,1]`; only its up/down faces exist, both use
tint index `0`, and the particle/face texture is the same transparent
16×16 `block/lily_pad` image. The render plane is lower and wider than
the authoritative collision/selection box.

Block tint index zero is constant `0xFF208030` in world and defaults to
`0xFF71C35C` without a world/position. The generated flat item model uses
the block texture and constant tint `0xFF71C35C`. It has no predicate or
component-selected item-model branch.

The English name is `Lily Pad`. Natural Blocks publishes it exactly once,
after Nether Wart and before Seagrass. It appears in no other baseline
creative tab.

**Branches and aborts:**

- Direct block use returns `PASS`; air use must source-raycast and then
  pass generic placement plus the two-part support predicate.
- Source water or tagged Ice/Frosted Ice can support the block only while
  the pad cell's fluid is Empty; a failing shape update returns Air.
- Every tool reaches the same self table; only explosion survival can
  suppress its one item.
- Only a server-side `AbstractBoat` selects the ignored-result
  destruction callback.
- Step sound, generic walking path type and frog path/landing preference
  are three independent exact/tag consumers.
- Fishing acquisition and fishing open-water tolerance are separate
  branches with separate probability and spatial gates.
- Composter, trade, placed feature, huge-fungus replacement and Mansion
  selection can each reject or omit the state.

**Constants and randomness:**

State/block/block-type/item IDs `8920/374/241/451`; zero strength;
shape inset/height `1/16,1.5/16`; sound IDs break/step/place/hit/fall
`169/173/1724/171/170`; stack `64`; tag closures `2/0`; frog range/
velocity-multiplier/preference `4×2/3.5714288f/0.5f`; fishing local weights
`17/100` or `17/110`, root zero-luck weights `10/5/85`; Composter
`0.65f`; trader output/uses/inclusion `5/2/5 of 76`; patch counts
`4×10`, offsets `±7/±3`, biomes `2`; fungus configurations `4`;
structure files/cells `1/8`; model rotations `4`; tints
`0xFF208030/0xFF71C35C`.

**Side effects:**

Placement and support removal; immediate mining, piston/explosion/Boat
destruction and loot; step sounds and navigation decisions; frog
long-jump choice; fishing item/experience/stat effects; Composter and
merchant state; feature/fungus/structure writes; state/stack migration
and persistence; exact randomized model, texture, tint, name and tab
projection.

**Gates:**

Use route, raycast and generic placement; supporting fluid/block plus
empty target fluid; world/update authority; tool/explosion and Boat
identity/logical side; direct tags and frog activity/RNG/landing fluids;
fishing open-water layers, biome condition, luck and loot draws;
Composter level/draw; trader common-set/economy; feature modifier,
Air/survival/provider/write; huge-fungus predicate; Mansion layout/
room draw/transform/processor/clip/write; registry, reload, migration and
client-resource validity.

**Boundary cases and quirks:**

The item deliberately cannot place by directly clicking Ice even though
Ice and Frosted Ice can support an already-written pad. Source water
inside a waterlogged support can pass the fluid branch. The thin collision
box is narrower than the full-width render plane. Boat destruction is
server-only and requests drops. Lily Pad counts as an above-water
open-water-fishing cell, while its own junk-loot entry remains independent.
General pathfinding calls it `TRAPDOOR`, but frogs override the cell above
a preferred pad to `OPEN`.

**Failure semantics:**

Generic placement, state update, break/loot, fishing, Composter, merchant,
feature, huge-fungus and Mansion transactions retain their owners' commit
semantics. The Boat callback ignores `destroyBlock` failure. Feature and
structure writes can partially commit. A failed frog-preference search
falls back to a retained nonpreferred candidate rather than failing solely
because no Lily Pad exists. Reload changes future tag/data/resource reads
only; migration applies only while its owning fix is active.

**Client/server authority split:**

The client performs the item-use raycast and placement prediction and
renders the randomized plane/tints/item/name. The server owns placement
admission, support updates, Boat destruction, mining/loot, path/AI,
fishing, composting, trades, worldgen/structures and persistence/
migration, then synchronizes authoritative blocks, entities, stacks,
offers, sounds and statistics.

**Observability:**

Observe state/registry/sound IDs; exact shapes, light, path and redstone
reads; use-on versus air-use hit transformation; support-fluid/block and
target-fluid checks; update/Boat result; mining/loot/explosion; step-sound
source; frog preference draw/candidate order/path override; fishing pool
and four-layer classification; Composter/trader results; complete tag
closures; every feature/fungus/structure read, draw and write; exact
eight-cell census; legacy conversion, durable/wire identity and client
rotation/tint/tab projection.

**Persistence and reload:**

Lily Pad saves one property-free identity and has no block entity. Its
stack uses generic components. Tags, loot, merchant, worldgen and client
resources have independent reload boundaries. Registration,
`PlaceOnWaterBlockItem`, Boat contact, Walk-node identity checks,
Composter entry, frog AI setup, data fixes and creative ordering are
code-built.

**Evidence:**

`net.minecraft.world.level.block.Blocks`;
`net.minecraft.world.level.block.LilyPadBlock`;
`net.minecraft.world.level.block.VegetationBlock`;
`net.minecraft.world.item.PlaceOnWaterBlockItem`;
`net.minecraft.world.level.block.SoundType`;
`net.minecraft.world.entity.Entity#getPrimaryStepSoundBlockPos`;
`net.minecraft.world.level.pathfinder.WalkNodeEvaluator`;
`net.minecraft.world.entity.animal.frog.FrogAi`;
`net.minecraft.world.entity.animal.frog.Frog$FrogNodeEvaluator`;
`net.minecraft.world.entity.ai.behavior.LongJumpToPreferredBlock`;
`net.minecraft.world.entity.projectile.FishingHook`;
`net.minecraft.world.level.block.ComposterBlock`;
`net.minecraft.world.level.levelgen.structure.structures.WoodlandMansionPieces$FirstFloorRoomCollection`;
`net.minecraft.util.datafix.fixes.BlockStateData`;
`net.minecraft.util.datafix.fixes.EntityBlockStateFix`;
`net.minecraft.util.datafix.fixes.ItemIdFix`;
`net.minecraft.util.datafix.fixes.ItemStackTheFlatteningFix`;
`net.minecraft.client.color.block.BlockColors`;
`net.minecraft.client.data.models.BlockModelGenerators`;
`net.minecraft.world.item.CreativeModeTabs`; block/item/sound/component
reports; self/fishing loot; support/direct block/fluid tags; Composter and
common-trader records; waterlily configured/placed feature, both biome
records and four huge-fungus configurations; all `1,212` decoded
templates and decompressed strings; exact blockstate/model/item/texture/
language resources. Complete compiled exact-field, data, legacy-fix and
decoded-NBT searches find no other identity-specific runtime path.

**Test vectors:**

Run `EXP-BLK-121` across state/registry identity, every shape/light/path/
redstone/piston/tool/explosion branch, use-on and source-raycast placement
over every source/flowing/waterlogged/Ice/Frosted-Ice/target-fluid case,
support updates, client/server Boat contact, step-sound and frog
preference/path boundaries, fishing root/junk weights and every four-layer
open-water layout, all Composter/trader branches, complete `2/0` tag
closures, both biome schedules and all `40` feature candidates, all four
fungus replacement configurations, the exact eight Mansion cells, every
legacy fix, persistence/reload and exact client projection. Assert IDs,
ordering, constants, absences, census and vanilla convergence.

**Limits:**

Generic placement/update/mining, loot/fishing, Composter, merchant, frog
brain/pathfinding, feature/huge-fungus/Mansion, packet and rendering
algorithms retain their named owners. Ice, Frosted Ice, water, Boats,
frogs, fishing rods and Woodland Mansions retain their catalog families.
This leaf fixes exact Lily Pad and every direct join that selects it.
