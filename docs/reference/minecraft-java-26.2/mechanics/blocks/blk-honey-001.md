# Blocks mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `BLK-HONEY-001` — Honey block joins side sliding, piston adhesion and surface carrying

**Parent:** `BLK-001`, `BLK-002`, `BLK-003`, `BLK-004`, `BLK-005`, `PLY-002`, `PLY-006`,
`RED-001`, `RED-004`, `ITM-004`, `ITM-007`, `ENT-001`, `MOB-004`, `MOB-005`, `ENV-003`, `WGEN-003`,
`CLI-006`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked registration, concrete block hooks, piston and AI identity branches,
generated reports/data, advancement and client assets fix the sole state, movement effects,
adhesion, acquisition, AI exclusions, diagnostic generation and projection.

**Applies when:**

`minecraft:honey_block` is placed, broken, fallen onto, slid down, moved by a piston, used by a
piston resolver, queried for support or path type, crafted, serialized, rendered or selected by
the aquifer debug terrain branch.

**Authoritative state:**

Honey block is a `HoneyBlock` extending `HalfTransparentBlock`, with one property-free default
state, ID `21816`, and no block entity. Registration supplies orange map color, speed factor `0.4`,
jump factor `0.5`, honey-block sound and no occlusion. It leaves the property-builder defaults of
destroy speed and explosion resistance `0`, friction `0.6`, restitution `0`, collision enabled and
piston reaction `NORMAL`.

Selection shape is a full cube. Collision and inherited visual/support shapes are the centered
`14 x 15 x 14` column from X/Z `1/16` through `15/16` and Y `0` through `15/16`. The full selection
shape prevents inherited skylight propagation; no occlusion makes the state non-solid-rendering,
so light dampening is `1`. Reduced collision gives shade brightness `1.0` and makes the ordinary
redstone-conductor, suffocating, view-blocking and spawn-support full-shape predicates false.

Its ordinary common-rarity `BlockItem` stacks to `64` with no special components or use gate. It
appears in both building-block and redstone creative tabs. Generic placement writes state `21816`.

**Transition and ordering:**

`fallOn` first plays the honey slide sound at volume and pitch `1`; the server then broadcasts
entity event `54`, and fall damage is attempted with multiplier `0.2`. Only a successful damage
result adds the block sound type's fall sound at `0.5 * volume` and `0.75 * pitch`. Event `54` is
handled by living entities on clients as ten honey-block particles.

`entityInside` tests sliding before delegating the inherited callback. Sliding is rejected when
the entity is on ground, its Y exceeds `blockY + 0.9375 - 1e-7`, its reconstructed pre-physics
vertical speed `oldY = currentY / 0.9800000190734863 + 0.08` is at least `-0.08`, or both horizontal
center offsets plus `1e-7` are at most `0.4375 + entityWidth / 2`. Thus an admitted entity is
falling beside at least one outer side of the inset column.

For an admitted slide, pre-physics `oldY < -0.13` scales X and Z by `-0.05 / oldY`; the other
admitted band preserves X/Z. Both branches write Y as `(-0.05 - 0.08) * 0.9800000190734863`
(approximately `-0.1274`) and reset fall distance. This targets pre-physics Y `-0.05`; the stored
post-physics velocity is intentionally different.

Every 20 game ticks, an admitted sliding `ServerPlayer` tests the live block state through
`HONEY_BLOCK_SLIDE`; the bundled advancement requires exact honey block and sends its telemetry
event. Only living entities, minecarts, primed TNT and boats receive slide effects. Each side makes
one independent one-in-five level-RNG sound draw. After its first draw the client stops; the server
makes a second independent one-in-five draw and broadcasts event `53` on success. Event `53`
creates five honey-block particles on clients. Other entity types still receive slide movement,
fall-distance reset and inherited handling but consume neither effect draw.

The piston resolver treats exact honey and slime as sticky. A honey/slime pair never sticks in
either order; otherwise a pair sticks when either member is honey or slime. Sticky members inspect
all four directions perpendicular to piston motion. Admission, traversal order, reactions and the
total 12-block cap remain with `RED-PISTON-001`.

On each moving-piston block-entity tick, ordinary collided-entity displacement runs first. The
separate top-surface branch then applies only when the carried state is exact honey and movement is
horizontal. It searches from the transformed collision top (`15/16`) through local Y `1.500001`.
An entity qualifies only with `NORMAL` piston reaction, on-ground state, and either support by the
moving position or an X/Z point inclusively inside that search box. Each qualifier is moved along
the piston direction by the progress delta. Vertical honey motion has no surface-carry branch;
honey also lacks slime's movement-axis velocity assignment in ordinary collision displacement.

Walk-node classification maps exact honey to `STICKY_HONEY`, whose default pathfinding malus is
`8.0`; step-up probing does not grant its ordinary extra step allowance when the current node is
sticky honey. Generic long-jump startup rejects exact honey at the mob's current block position and
stores half a sampled ordinary cooldown on that failure. Breeze long jump independently rejects
exact honey before testing the four cells above its current position.

The reloadable `support_override_snow_layer` membership makes a snow layer survive above honey
after the snow block's cannot-support rejection, despite honey's reduced collision top. The direct
`suppresses_bounce` membership prevents the ground-block restitution contribution in generic
collision. Honey's registered restitution is independently zero, so removing only this membership
is outcome-equivalent for the locked honey state unless another restitution input is positive.

The block loot table offers one honey-block item behind `survives_explosion`. A shaped 2x2 grid of
four honey bottles crafts one block; the bottles' `use_remainder` component returns four glass
bottles. The reverse shapeless recipe consumes one honey block plus four glass bottles and returns
four honey bottles. Crafting match, allocation and remainder placement remain generic.

During noise-chunk `doFill`, the private aquifer diagnostic can replace the interpolated state only
when `SharedConstants.DEBUG_AQUIFERS` is enabled, Z is nonnegative and divisible by four, and Y is
exactly `preliminarySurfaceLevel(X, Z) + 8`. It selects honey block at or above sea level and slime
block below sea level. This is diagnostic behavior, not default world generation.

The block adds no scheduled tick, random tick, use, attack, neighbor, comparator or block-event
callback of its own.

**Client projection:**

The sole blockstate variant selects a model with a full outer cube textured from the bottom on all
six cull-faced sides and an inner cube from `1` through `15`: bottom, top and four side textures map
to their respective inner faces. The item definition selects the same model.
`HalfTransparentBlock.skipRendering` omits a face next to exact honey and otherwise delegates.
Terrain updates project state `21816`; events `53` and `54` project five slide and ten fall
particles respectively. No block entity, conditional model, random variant or special renderer is
involved.

**Branches and aborts:**

Fall-damage success; slide ground/height/speed/side gates; fast versus throttled slide; player
20-tick advancement check; eligible versus other effect entity; client/server draws; honey/honey,
honey/ordinary and honey/slime adhesion; horizontal/vertical piston motion and surface admission;
sticky path or long-jump exclusions; tag snapshot membership; player versus explosion loot; both
crafting directions; disabled/enabled debug aquifers; same/other-neighbor rendering are distinct.

**Constants and randomness:**

State ID `21816`; destroy speed/resistance/restitution `0`; friction `0.6`; speed `0.4`; jump `0.5`;
collision X/Z `1/16..15/16`, Y `0..15/16`; dampening `1`; shade `1.0`; fall multiplier `0.2`;
slide thresholds `-0.08` and `-0.13`; target pre-physics Y `-0.05`; drag `0.9800000190734863`,
gravity offset `0.08`; side epsilon `1e-7`; advancement interval `20`; two independent one-in-five
effect draws; events `53`/`54` with `5`/`10` particles; piston search top `1.500001` and cap `12`;
path malus `8.0`; stack `64`; four-to-one and one-plus-four-to-four recipes; debug offset `8` and Z
period `4`. Explosion survival, cooldown sampling and generic transactions retain their randomness.

**Side effects:**

Generic placement/removal and self loot; entity velocity, fall distance, damage, sounds, events and
particles; advancement progress/telemetry; piston plans plus collided and surface entity movement;
AI path/cooldown decisions; snow support and bounce-tag decisions; crafting outputs/remainders;
optional debug terrain writes; ordinary state, item and model projection.

**Gates:**

Placement/removal authority; entity position/type/velocity and side; server/client and game-time
phase; exact block identities; piston direction, progress, pushability/reaction/cap and support;
path/brain caller; current tag and recipe snapshots; loot context; debug flags/coordinates/sea
level; client event/entity/model context.

**Boundary cases and quirks:**

Full selection but reduced collision makes skylight propagation false while ordinary support and
full-cube predicates are false. Slide tests reconstruct velocity from the post-physics value and
use entity position/width rather than AABB overlap. Server effect RNG always performs the second
draw for eligible sliding entities even when the first draw fails. Advancement tests the live state
only on the 20-tick cadence. Moving honey carries qualifying grounded entities only horizontally
and only after ordinary collision displacement. The snow-support tag overrides the geometry; the
bounce tag is redundant with honey's current zero restitution. Debug honey stripes are not normal
terrain.

**Evidence:**

`OFF-SERVER-001`, `OFF-CLIENT-001`, `OFF-REPORT-001`, `OFF-DATA-001`;
`net.minecraft.world.level.block.Blocks`,
`net.minecraft.world.level.block.HoneyBlock#getCollisionShape`,
`net.minecraft.world.level.block.HoneyBlock#fallOn`,
`net.minecraft.world.level.block.HoneyBlock#entityInside`,
`net.minecraft.world.level.block.HoneyBlock#isSlidingDown`,
`net.minecraft.world.level.block.HoneyBlock#doSlideMovement`,
`net.minecraft.world.level.block.HoneyBlock#maybeDoSlideAchievement`,
`net.minecraft.world.level.block.HoneyBlock#maybeDoSlideEffects`,
`net.minecraft.world.level.block.HoneyBlock#showSlideParticles`,
`net.minecraft.world.level.block.HoneyBlock#showJumpParticles`,
`net.minecraft.world.entity.Entity#handleEntityEvent`,
`net.minecraft.world.entity.LivingEntity#handleEntityEvent`,
`net.minecraft.world.level.block.HalfTransparentBlock#skipRendering`,
`net.minecraft.world.level.block.state.BlockBehaviour#getBlockSupportShape`,
`net.minecraft.world.level.block.state.BlockBehaviour#getLightDampening`,
`net.minecraft.world.level.block.state.BlockBehaviour#propagatesSkylightDown`,
`net.minecraft.world.level.block.SnowLayerBlock#canSurvive`,
`net.minecraft.world.entity.Entity#restituteMovementAfterCollisions`,
`net.minecraft.world.level.block.piston.PistonStructureResolver#isSticky`,
`net.minecraft.world.level.block.piston.PistonStructureResolver#canStickToEachOther`,
`net.minecraft.world.level.block.piston.PistonStructureResolver#addBranchingBlocks`,
`net.minecraft.world.level.block.piston.PistonMovingBlockEntity#tick`,
`net.minecraft.world.level.block.piston.PistonMovingBlockEntity#moveStuckEntities`,
`net.minecraft.world.level.pathfinder.WalkNodeEvaluator#getPathTypeFromState`,
`net.minecraft.world.level.pathfinder.PathType`,
`net.minecraft.world.entity.ai.behavior.LongJumpToRandomPos#checkExtraStartConditions`,
`net.minecraft.world.entity.monster.breeze.LongJump#canJumpFromCurrentPosition`,
`net.minecraft.world.level.levelgen.NoiseBasedChunkGenerator#doFill`,
`net.minecraft.world.level.levelgen.NoiseBasedChunkGenerator#debugPreliminarySurfaceLevel`,
`net.minecraft.SharedConstants#debugFlag`,
`net.minecraft.world.item.CreativeModeTabs#bootstrap`;
`reports/blocks.json#minecraft:honey_block`,
`reports/minecraft/components/item/honey_block.json`,
`reports/minecraft/components/item/honey_bottle.json`,
`data/minecraft/tags/block/support_override_snow_layer.json`,
`data/minecraft/tags/block/suppresses_bounce.json`,
`data/minecraft/loot_table/blocks/honey_block.json`,
`data/minecraft/recipe/honey_block.json`,
`data/minecraft/recipe/honey_bottle.json`,
`data/minecraft/advancement/adventure/honey_block_slide.json`,
`assets/minecraft/blockstates/honey_block.json`,
`assets/minecraft/models/block/honey_block.json`,
`assets/minecraft/items/honey_block.json`.

**Test vectors:**

Run `EXP-BLK-036` across placement/break/explosion, shapes/light/support/tags, fall damage and both
events, every slide threshold/side/entity/time/RNG branch, piston structures and horizontal versus
vertical surface carrying, path and both long-jump exclusions, both recipes/remainders,
disabled/enabled aquifer debugging, save/reload and same/other-neighbor model contexts. Assert state,
velocity, damage, RNG count, sounds, events/particles, advancement, piston lists/moves, AI result,
outputs, writes and faces.

**Limits:**

This leaf does not re-specify generic placement/breaking, collision physics, piston transactions,
AI scheduler/path search, crafting allocation, advancement storage, debug-flag setup, worldgen
equivalence, tag publication, light propagation, loot RNG, state packets or model loading. Those
remain with `BLK-002`, `PLY-COLLISION-001`, `RED-PISTON-001`, `MOB-004`, `MOB-005`,
`ITM-RECIPE-001`, `ITM-ADVANCEMENT-001`, `WGEN-PIPELINE-001`, `ENV-LIGHT-001`, `ITM-004`, loot owners
and `CLI-006`.
