# Blocks mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `BLK-SLIME-001` — Slime block joins restitution, surface drag and piston adhesion

**Parent:** `BLK-001`, `BLK-002`, `BLK-003`, `BLK-004`, `BLK-005`, `PLY-002`, `PLY-006`,
`RED-001`, `RED-004`, `ITM-005`, `ENT-001`, `ENV-003`, `WGEN-003`, `CLI-006`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked registration, the concrete block hooks, piston identity branches,
brewing registration, generated report/data and client assets fix the sole state, physical effects,
adhesion, acquisition, debug-generation boundary and projection.

**Applies when:**

`minecraft:slime_block` is placed, broken, collided with, stepped on, moved by a piston, used by a
piston resolver, crafted, brewed, serialized, rendered or selected by the aquifer debug terrain
branch.

**Authoritative state:**

Slime block is a `SlimeBlock` extending `HalfTransparentBlock`, with one property-free default
state, ID `12532`, and no block entity. Registration supplies grass map color, friction `0.8`, bounce
restitution `1.0`, slime-block sound and no occlusion. It leaves the property-builder defaults of
destroy speed `0`, explosion resistance `0`, speed/jump factors `1`, collision enabled and piston
reaction `NORMAL`.

Selection, collision and visual shapes are full cubes. The full selection shape prevents inherited
skylight propagation; because no occlusion makes the state non-solid-rendering, the base light rule
caches dampening `1`. Full collision gives shade brightness `0.2`. The default predicates admit
ordinary spawn support and make the state a redstone conductor, suffocating and view blocking when
their generic full-cube tests are queried; no-occlusion is not an empty-shape predicate.

Its ordinary common-rarity `BlockItem` stacks to `64` and has no special components or use gate. It
appears in both building-block and redstone creative tabs. Generic placement writes state `12532`.

**Transition and ordering:**

Downward collision first remains in the generic `PLY-COLLISION-001` restitution transaction. The
registered block restitution is `1.0`; the generic owner retains the effective-gravity threshold,
bounce suppression, living/nonliving interpolation, `suppresses_bounce` tag check, `BOUNCE` event,
position synchronization and all velocity formulas. Slime block is not a direct member of that
tag. Its `fallOn` hook separately calls fall damage with multiplier `0` when bounce is not
suppressed, and skips the call when it is suppressed, so this hook deals no self fall damage in
either branch.

After movement reaches `stepOn`, let `d = abs(deltaMovement.y)`. Only when `d < 0.1` and the entity
is not stepping carefully, multiply both horizontal components by `0.4 + d * 0.2`, preserve Y, then
delegate the inherited step callback. Equality at `0.1` and careful stepping leave velocity
unchanged. Friction `0.8` remains a separate input to generic ground movement.

The piston resolver treats exact slime and honey states as sticky. A slime/honey pair never sticks
in either order; otherwise a pair sticks when at least one member is slime or honey. Sticky members
recursively inspect all four directions whose axis is perpendicular to piston motion. Admission,
list ordering, reaction handling and the total 12-block push cap remain with `RED-PISTON-001`.

While a moving piston block entity carries exact slime, collided entities with `IGNORE` reaction
and every `ServerPlayer` are skipped by the slime branch. For every other collided entity, preserve
the two off-axis velocity components and replace the movement-axis component with the movement
direction step (`-1` or `1`), then run the generic overlap-distance piston move and retraction
cleanup. Slime does not take honey's separate top-surface entity-drag branch.

The block loot table offers one slime-block item behind `survives_explosion`. Ordinary player
breaking therefore returns itself; explosion retention and RNG remain generic loot-condition
behavior. A shaped 3x3 grid of nine slime balls crafts one block, and one block shapelessly crafts
nine slime balls. Brewing registration adds two feature-filtered potion edges for slime block:
water becomes mundane and awkward becomes oozing. Slot timing, ingredient consumption and the
400-tick transaction remain with `ITM-BREW-001`.

During noise-chunk `doFill`, the private aquifer diagnostic can replace the interpolated state only
when `SharedConstants.DEBUG_AQUIFERS` is enabled, Z is nonnegative and divisible by four, and Y is
exactly `preliminarySurfaceLevel(X, Z) + 8`. It selects slime block below sea level and honey block
at or above sea level. With the debug flag off, or at every other coordinate, it returns the prior
interpolated state unchanged. This diagnostic branch is not normal world generation.

The block adds no scheduled tick, random tick, use, attack, neighbor, comparator or block-event
callback of its own.

**Client projection:**

The sole blockstate variant selects one model composed of a textured inner cube from `3` through
`13` on every axis plus a textured full outer cube whose six faces carry matching cull-face hints.
The item definition selects the same model. `HalfTransparentBlock.skipRendering` omits a face next
to the exact same block and otherwise delegates ordinary face culling. Terrain and block updates
project state `12532`; no block entity, conditional model, random variant or special renderer is
involved.

**Branches and aborts:**

Suppressed versus admitted bounce; fall hook versus restitution; careful, fast-vertical and
slow-noncareful steps; slime/slime, slime/ordinary and slime/honey adhesion; push-cap or pushability
failure; ignored entity, server player and ordinary entity collision; player versus explosion
loot; two crafting directions; water/awkward/other potion inputs; disabled versus enabled debug
aquifers; same-block versus other-neighbor rendering are distinct branches.

**Constants and randomness:**

State ID `12532`; destroy speed/resistance `0`; friction `0.8`; restitution `1.0`; light dampening
`1`; shade brightness `0.2`; slow-step threshold `0.1`; horizontal multiplier `0.4 + d * 0.2`;
piston-axis velocity step magnitude `1`; generic piston cap `12`; stack `64`; nine-to-one and
one-to-nine recipes; water-to-mundane and awkward-to-oozing brewing edges; debug offset `8` and
Z period `4`. The block hooks and piston identity branches consume no RNG; explosion survival and
generic transactions retain their owning randomness.

**Side effects:**

Generic placement/removal and self loot; entity velocity, fall-damage and bounce/event effects;
piston movement plans plus collided-entity movement; crafting and brewing outputs; optional debug
terrain writes; ordinary state, item and model projection.

**Gates:**

Placement/removal authority; collision direction and effective-gravity threshold; bounce/careful
flags; velocity threshold; exact block identities; piston pushability/reaction/cap and entity type;
loot context; recipe or brewing match and feature set; debug flags/coordinates/sea level; client
neighbor/model context.

**Boundary cases and quirks:**

No occlusion does not make the full shape propagate skylight. Bounce suppression removes both the
registered restitution path and the zero-multiplier `fallOn` call. The surface slowdown uses
vertical speed at `stepOn`, not horizontal speed. Slime sticks to ordinary blocks but never honey.
A moving slime block skips server players rather than assigning their movement-axis velocity. The
two-start-mix brewing edges are code-built, while both crafting recipes are reloadable data. Debug
aquifer stripes are source-locked diagnostics, not a default terrain feature.

**Evidence:**

`OFF-SERVER-001`, `OFF-CLIENT-001`, `OFF-REPORT-001`, `OFF-DATA-001`;
`net.minecraft.world.level.block.Blocks`,
`net.minecraft.world.level.block.SlimeBlock#fallOn`,
`net.minecraft.world.level.block.SlimeBlock#stepOn`,
`net.minecraft.world.level.block.HalfTransparentBlock#skipRendering`,
`net.minecraft.world.level.block.state.BlockBehaviour#getLightDampening`,
`net.minecraft.world.level.block.state.BlockBehaviour#propagatesSkylightDown`,
`net.minecraft.world.level.block.piston.PistonStructureResolver#isSticky`,
`net.minecraft.world.level.block.piston.PistonStructureResolver#canStickToEachOther`,
`net.minecraft.world.level.block.piston.PistonStructureResolver#addBranchingBlocks`,
`net.minecraft.world.level.block.piston.PistonMovingBlockEntity#moveCollidedEntities`,
`net.minecraft.world.item.alchemy.PotionBrewing#addVanillaMixes`,
`net.minecraft.world.item.alchemy.PotionBrewing$Builder#addStartMix`,
`net.minecraft.world.level.levelgen.NoiseBasedChunkGenerator#doFill`,
`net.minecraft.world.level.levelgen.NoiseBasedChunkGenerator#debugPreliminarySurfaceLevel`,
`net.minecraft.SharedConstants#debugFlag`,
`net.minecraft.world.item.CreativeModeTabs#bootstrap`;
`reports/blocks.json#minecraft:slime_block`,
`reports/minecraft/components/item/slime_block.json`,
`data/minecraft/loot_table/blocks/slime_block.json`,
`data/minecraft/recipe/slime_block.json`,
`data/minecraft/recipe/slime_ball.json`,
`assets/minecraft/blockstates/slime_block.json`,
`assets/minecraft/models/block/slime_block.json`,
`assets/minecraft/items/slime_block.json`.

**Test vectors:**

Run `EXP-BLK-035` across placement/break/explosion, light and spawn predicates, downward collision
at gravity thresholds with bounce suppression, fall distance, slow/fast/careful stepping, every
piston direction and 12/13-block slime/ordinary/honey structures, ignored/player/nonplayer entity
collisions, both recipes, every brewing base, disabled/enabled aquifer debugging around Z/Y/sea
boundaries, save/reload and same/other-neighbor rendering. Assert state, shapes, velocity, damage,
events, piston lists/moves, outputs, writes and model faces.

**Limits:**

This leaf does not re-specify generic placement/breaking, collision/restitution formulas, ground
movement, piston transaction ordering, crafting allocation, brewing timing, debug-flag setup,
worldgen equivalence, light propagation, loot-condition RNG, state packets or model loading. Those
remain with `BLK-002`, `PLY-COLLISION-001`, player movement owners, `RED-PISTON-001`,
`ITM-RECIPE-001`, `ITM-BREW-001`, `WGEN-PIPELINE-001`, `ENV-LIGHT-001`, loot owners and `CLI-006`.
