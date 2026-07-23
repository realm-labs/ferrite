# Blocks mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `BLK-LAVA-CAULDRON-001` — A full lava cauldron joins bucket dispatch, ordered lava contact, comparator output and navigation

**Parent:** `BLK-001`, `BLK-002`, `BLK-003`, `BLK-004`, `BLK-005`, `PLY-002`, `PLY-006`,
`ITM-001`, `ITM-004`, `ITM-006`, `ENT-001`, `MOB-001`, `MOB-004`, `ENV-001`, `ENV-003`,
`RED-001`, `WGEN-003`, `CLI-006`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked registration, the abstract and lava-cauldron hooks, complete lava
interaction dispatcher, inside-effect collector, navigation/POI consumers, loot, reports and client
assets fix the sole state, physical surface, interaction transitions, entity effects and projection.

**Applies when:**

`minecraft:lava_cauldron` is written, broken, queried for shape/light/path/POI/comparator behavior,
intersected by an entity, used with an item, targeted by precipitation or dripstone, serialized or
rendered.

**Authoritative state:**

Lava cauldron is a `LavaCauldronBlock` extending `AbstractCauldronBlock`, with one property-free
default state, ID `9464`, and no block entity. It legacy-copies the ordinary cauldron's stone map
color, correct-tool requirement, strength/resistance `2`, no-occlusion flag and otherwise default
stone sound, friction `0.6`, speed/jump `1`, restitution `0` and piston reaction `NORMAL`, then
adds constant light emission `15`.

The shared outline/movement shape is the cauldron shell: a full cube with the centered `12x12`
column through Y `0..16` removed, plus centered `16x8` and `8x16` columns through Y `0..3`
removed. The interaction interior is X/Z `2..14`, Y `4..16`. Lava content height is `15/16`;
the entity-inside test shape merges the shell with an interior X/Z `2..14`, Y `4..15` column.
Selection is therefore hollow while content contact reaches one pixel below the rim.

No occlusion plus the non-full shape yields skylight propagation, light dampening `0`, shade
brightness `1`, and false ordinary redstone-conductor, suffocation, view-blocking and default
full-top spawn-support results. All path-computation types are rejected by the block hook. The
abstract cauldron advertises analog output, and lava cauldron returns constant signal `3`.

There is no `lava_cauldron` item or direct creative entry. Its generic `asItem` falls back to AIR,
so the generic clone stack is empty. The reloadable block loot table instead returns one ordinary
`minecraft:cauldron` behind `survives_explosion`, with random sequence
`blocks/lava_cauldron`. It is indirectly pickaxe-mineable because `mineable/pickaxe` includes the
`cauldrons` block tag.

**Transition and ordering:**

Item use resolves the active lava `CauldronInteraction.Dispatcher`: tag registrations, if any, are
iterated before exact item registrations; the locked lava dispatcher has no tag entry. Unknown
items return `TRY_WITH_EMPTY_HAND`. Recognized interactions decide their underwater/eligibility
gate first, perform authoritative inventory/stat/state/sound/game-event effects only on the server,
then return the documented success result on both sides.

#### Bucket dispatcher

The lava dispatcher has exactly four exact item paths:

1. An empty bucket always passes the full-state predicate. Client returns `SUCCESS`. Server replaces
   the hand input through `createFilledResult` with one lava bucket, awards `USE_CAULDRON` and
   `ITEM_USED(bucket)`, writes ordinary empty cauldron, plays `BUCKET_FILL_LAVA` at volume/pitch
   `1`, emits `FLUID_PICKUP`, ignores the write result and returns `SUCCESS`.
2. A lava bucket first tests whether the fluid immediately above belongs to the water tag. If so,
   it returns `CONSUME` without inventory, stat, state, sound or game-event mutation. Otherwise the
   generic empty-bucket helper replaces the input with an empty bucket, awards `FILL_CAULDRON` and
   `ITEM_USED(lava_bucket)`, rewrites the same default lava-cauldron state, plays
   `BUCKET_EMPTY_LAVA`, emits `FLUID_PLACE`, ignores the write result and returns `SUCCESS`.
3. A water bucket has no underwater gate. It performs the same empty-bucket transaction but writes
   water cauldron level `3` and plays `BUCKET_EMPTY`.
4. A powder-snow bucket uses the same immediate-above water-tag gate as a lava bucket. When clear,
   it performs the empty-bucket transaction, writes powder-snow cauldron level `3` and plays
   `BUCKET_EMPTY_POWDER_SNOW`.

Every server-side bucket mutation updates the hand and stats before the state write, then plays the
sound and emits the game event regardless of that write's boolean result. Survival/creative stack
replacement details remain with `ItemUtils`; comparator neighbor propagation and client correction
remain with their generic owners.

#### Entity contact

When the movement collector admits intersection with the filled inside shape, lava cauldron submits
`CLEAR_FREEZE`, submits `LAVA_IGNITE`, and registers `Entity#lavaHurt` after `LAVA_IGNITE`. The
step collector orders effect types by enum order, deduplicates each type per step, and places the
registered callback after the selected ignite effect. Applied consequences therefore clear frozen
ticks, skip ignition/damage for fire-immune entities, otherwise ignite for `15` seconds and then
submit `4.0` lava damage on the server. A successful unsilenced hurt may play `GENERIC_BURN` at
volume `0.4` and pitch `2 + nextFloat()*0.4`.

This leaf owns the content shape and submitted effect/callback sequence. Cross-block effect
deduplication, entity-alive aborts, fire timers, damage admission/reduction/death and sound eligibility
remain with the entity, damage and inside-effect owners.

#### Drip, weather, path and POI roles

Lava cauldron is always full. It does not override precipitation handling, so rain and snow do not
change it or draw the empty/layered-cauldron precipitation chances. Its inherited
`canReceiveStalactiteDrip` returns false for every fluid; pointed dripstone therefore does not
select it as a fillable target. Even a manually scheduled abstract-cauldron tick can find a tip and
fluid but fails the receive gate and performs no write/event.

The active `cauldrons` block tag contains all four vanilla cauldron identities. During
`PathNavigation#trimPath`, every path node whose current state is in that tag is replaced at Y+1.
When a following node exists and the original current Y is at least the following Y, that next node
is also replaced using its X/Z and current Y+1. This reloadable navigation selector is separate
from `AbstractCauldronBlock#isPathfindable`, which always returns false.

Leatherworker POI bootstrap independently constructs a hardcoded set from every state of ordinary,
water, lava and powder-snow cauldrons; it does not read the reloadable tag. Lava state `9464` is
therefore a leatherworker job-site state in the POI type registered with max tickets `1` and valid
range `1`. POI claiming, profession assignment and path execution remain with mob owners.

`NodeEvaluator#isBurningBlock` also recognizes exact lava cauldron as a burning/danger input beside
fire, lava, magma and lit campfires. Path type, malus and route choice remain with `MOB-AI-001`.

The block adds no random tick, precipitation callback, concrete drip receiver, use-without-item,
attack, step, fall, neighbor, block-event or block-entity behavior of its own.

**Client projection:**

The empty blockstate variant selects `block/lava_cauldron`. That model inherits
`template_cauldron_full`, uses cauldron top/side/bottom/inner textures and `block/lava_still` for the
content surface. State `9464` emits world light `15`; it has no emissive-rendering predicate or
special renderer. The moving lava sprite follows the shared atlas animation, while the block model
itself has no random or conditional variant.

There is no `items/lava_cauldron` definition. The separately registered cauldron item uses its
ordinary item model and places an empty cauldron; terrain packets can still publish state `9464`
after bucket, command or loaded-palette mutation.

**Branches and aborts:**

Explicit write/break/explosion; shell/content intersection; inside-effect same-step composition and
fire immunity; four exact bucket paths, two immediate-above water gates, client/server split,
inventory placement and ignored state-write result; precipitation/drip nonadmission; tag-reloaded
path nodes versus hardcoded POI states; comparator/light queries; loot and block-model projection
are distinct.

**Constants and randomness:**

State `9464`; strength/resistance `2`; friction `0.6`; emission `15`; dampening `0`; shade `1`;
content height `15/16`; interaction/content columns X/Z `2..14` with Y `4..16`/`4..15`; comparator
`3`; bucket sounds volume/pitch `1`; water/powder levels `3`; ignition `15` seconds; lava damage
`4.0`; burn sound volume `0.4`, pitch `[2,2.4)` with one draw only after successful admitted hurt
and sound gates; POI tickets/range `1`/`1`. No block-owned RNG exists.

**Side effects:**

Generic state writes/removal; ordinary-cauldron loot; bucket hand replacement, stats, state
conversion, sounds and fluid game events; comparator/light updates; freeze clearing, ignition,
lava damage and possible burn sound; reload-selected path-node elevation; POI registration and
ordinary block-model projection.

**Gates:**

Placement/removal authority; correct tool and explosion survival; inside filled-shape intersection;
effect step and entity fire immunity/aliveness; active dispatcher/tag snapshot, exact held item,
server/client side, immediate-above water fluid tag, inventory destination and state-write result;
drip/precipitation admission; path nodes and POI snapshot; comparator/light/model context.

**Boundary cases and quirks:**

The state contains lava behavior without exposing a lava `FluidState`; water above is tested at the
separate cell immediately above. Redundant lava-bucket use outside water consumes the bucket and
rewrites the same state, while water above changes that path to mutation-free `CONSUME`. Water
buckets ignore that underwater gate. Item lookup and clone yield AIR/empty, yet block loot yields a
cauldron. Navigation uses a reloadable tag, but leatherworker POI membership is hardcoded. Constant
emission `15` coexists with a hollow, skylight-propagating, dampening-`0` shape.

**Evidence:**

`OFF-SERVER-001`, `OFF-CLIENT-001`, `OFF-REPORT-001`, `OFF-DATA-001`;
`net.minecraft.world.level.block.Blocks`,
`net.minecraft.world.level.block.AbstractCauldronBlock#useItemOn`,
`net.minecraft.world.level.block.AbstractCauldronBlock#getShape`,
`net.minecraft.world.level.block.AbstractCauldronBlock#getInteractionShape`,
`net.minecraft.world.level.block.AbstractCauldronBlock#hasAnalogOutputSignal`,
`net.minecraft.world.level.block.AbstractCauldronBlock#isPathfindable`,
`net.minecraft.world.level.block.AbstractCauldronBlock#tick`,
`net.minecraft.world.level.block.AbstractCauldronBlock#canReceiveStalactiteDrip`,
`net.minecraft.world.level.block.LavaCauldronBlock#getContentHeight`,
`net.minecraft.world.level.block.LavaCauldronBlock#isFull`,
`net.minecraft.world.level.block.LavaCauldronBlock#getEntityInsideCollisionShape`,
`net.minecraft.world.level.block.LavaCauldronBlock#entityInside`,
`net.minecraft.world.level.block.LavaCauldronBlock#getAnalogOutputSignal`,
`net.minecraft.core.cauldron.CauldronInteraction$Dispatcher#get`,
`net.minecraft.core.cauldron.CauldronInteractions#bootStrap`,
`net.minecraft.core.cauldron.CauldronInteractions#addDefaultInteractions`,
`net.minecraft.core.cauldron.CauldronInteractions#fillBucket`,
`net.minecraft.core.cauldron.CauldronInteractions#emptyBucket`,
`net.minecraft.core.cauldron.CauldronInteractions#fillWaterInteraction`,
`net.minecraft.core.cauldron.CauldronInteractions#fillLavaInteraction`,
`net.minecraft.core.cauldron.CauldronInteractions#fillPowderSnowInteraction`,
`net.minecraft.core.cauldron.CauldronInteractions#isUnderWater`,
`net.minecraft.world.entity.InsideBlockEffectApplier$StepBasedCollector#flushStep`,
`net.minecraft.world.entity.InsideBlockEffectApplier$StepBasedCollector#applyAndClear`,
`net.minecraft.world.entity.Entity#clearFreeze`,
`net.minecraft.world.entity.Entity#lavaIgnite`,
`net.minecraft.world.entity.Entity#lavaHurt`,
`net.minecraft.world.level.block.PointedDripstoneBlock#findFillableCauldronBelowStalactiteTip`,
`net.minecraft.world.entity.ai.navigation.PathNavigation#trimPath`,
`net.minecraft.world.entity.ai.village.poi.PoiTypes#bootstrap`,
`net.minecraft.world.level.pathfinder.NodeEvaluator#isBurningBlock`,
`net.minecraft.world.item.Item#byBlock`;
`reports/blocks.json#minecraft:lava_cauldron`,
`data/minecraft/tags/block/cauldrons.json`,
`data/minecraft/tags/block/mineable/pickaxe.json`,
`data/minecraft/loot_table/blocks/lava_cauldron.json`,
`assets/minecraft/blockstates/lava_cauldron.json`,
`assets/minecraft/models/block/lava_cauldron.json`,
`assets/minecraft/models/block/template_cauldron_full.json`,
`assets/minecraft/items/cauldron.json`.

**Test vectors:**

Run `EXP-BLK-039` across explicit write/break/explosion, all shape/content/light/spawn/path/
comparator queries, inside-effect combinations, every bucket/client/server/water-above/inventory/
write-result branch, precipitation and drip probes, tag reload, POI rebuild, save/reload, clone/loot
and terrain/item models. Assert state, stacks, stats, writes/results, sounds/events, effect order,
damage/RNG, nodes, POI identity, light/comparator values and model selection.

**Limits:**

This leaf does not re-specify generic placement/breaking, item-stack destinations, comparator
propagation, inside-block collection, fire/damage/death, mob navigation/professions, weather,
dripstone scheduling, loot evaluation, terrain packets or model loading. Those remain with
`BLK-002`, `ITM-001`, `RED-COMPARATOR-001`, `ENT-001`, `ENT-DAMAGE-001`, `MOB-AI-001`,
`ENV-WEATHER-001`, dripstone owners, `ITM-LOOT-001`, `CLI-006` and their dedicated leaves.
