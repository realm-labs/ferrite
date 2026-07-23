# Blocks mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `BLK-STAINED-GLASS-001` — Stained glass propagates skylight while recoloring beacon sections

**Parent:** `BLK-001`, `BLK-002`, `BLK-003`, `BLK-005`, `PLY-005`, `PLY-006`, `ENV-003`,
`ENT-001`, `MOB-001`, `CLI-006`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — the locked color-collection registration, stained/transparent inheritance,
beacon scan, golem-support strategy, reports, loot, recipes, tag data and client assets exhaust the
sixteen identities, physical/light behavior, color transitions, acquisition and projection.

**Applies when:**

Any `minecraft:<color>_stained_glass` identity is placed, mined, queried for shape/light/spawn
support, scanned above a beacon, serialized, rendered, crafted or considered by the bundled
`impermeable` tag consumer.

**Authoritative state:**

`Blocks.STAINED_GLASS` registers one `StainedGlassBlock` for each `DyeColor` in white, orange,
magenta, light blue, yellow, lime, pink, gray, light gray, cyan, purple, blue, brown, green, red and
black order. Every identity has one property-free state and no block entity:

| Color | State | Opaque beam ARGB |
|---|---:|---:|
| white | 7098 | `FFF9FFFE` |
| orange | 7099 | `FFF9801D` |
| magenta | 7100 | `FFC74EBD` |
| light blue | 7101 | `FF3AB3DA` |
| yellow | 7102 | `FFFED83D` |
| lime | 7103 | `FF80C71F` |
| pink | 7104 | `FFF38BAA` |
| gray | 7105 | `FF474F52` |
| light gray | 7106 | `FF9D9D97` |
| cyan | 7107 | `FF169C9C` |
| purple | 7108 | `FF8932B8` |
| blue | 7109 | `FF3C44AA` |
| brown | 7110 | `FF835432` |
| green | 7111 | `FF5E7C16` |
| red | 7112 | `FFB02E26` |
| black | 7113 | `FF1D1D21` |

Registration selects the matching dye map color and supplies hat note instrument, destroy speed and
explosion resistance `0.3`, glass sound, no occlusion, a never-valid entity-spawn predicate and
false redstone-conductor, suffocating and view-blocking predicates. Collision and selection remain
a full cube. `TransparentBlock` supplies an empty visual shape, shade brightness `1`, true skylight
propagation and therefore light dampening `0`. Default friction is `0.6`, speed/jump factors are
`1`, restitution is `0`, and piston reaction is `NORMAL`.

Each registered ordinary `BlockItem` has common rarity, maximum stack size `64`, its corresponding
block item name/model and no special use component or placement gate. Placement writes the state
paired with the item color.

**Transition and ordering:**

#### Lighting and beacon color

Stained glass remains a full collider but does not obstruct the world light engine: inherited
skylight propagation is true and cached dampening is `0`. Lighting scheduling, propagation,
section dirtiness and terrain publication remain with `ENV-LIGHT-001`.

Every stained-glass block implements `BeaconBeamBlock`. `getColor` returns the identity's fixed
`DyeColor`, and beacon scanning converts its 24-bit texture diffuse color to opaque ARGB before
section selection. The beacon ticker scans at most ten vertical cells per invocation and resumes
from `lastCheckY`; the following rules apply in scan order:

1. The beacon block itself opens the initial white section with height `1`.
2. While the checking list has at most that one section, the first stained-glass cell appends a new
   height-`1` section with its raw opaque color. It does not blend with or merge into the beacon's
   initial white section, even when that first stained glass is white.
3. After a colored section exists, another stained-glass cell whose raw color equals the current
   section color increases that section's height by one.
4. A different raw color appends a height-`1` section whose color is
   `ARGB.average(currentSectionColor, rawColor)`. The helper averages alpha, red, green and blue
   independently with integer division by two. Because later transitions average against the
   already averaged current section, a sequence of three or more different colors is recursive,
   not a global arithmetic mean.
5. A non-beam block encountered after a section exists increases its height only when its light
   dampening is below `15` or it is bedrock. Stained glass never takes this obstruction branch
   because the beam-block branch is tested first.

When the scan reaches the current world-surface height, the checking list replaces the active beam
list. The same scan runs in client and server levels; server beacon activation/effects and client
beam rendering consume their respective completed sections under `BLK-BEACON-001`. Stained glass
therefore both passes the beam and changes its later visible color, unlike plain glass, which only
extends the current section, and tinted glass, which takes the obstruction branch.

#### Break, crafting, spawn and tag consumers

Each of the sixteen block loot tables has one roll that returns its own color only when the tool's
Silk Touch level is at least `1`. No explosion-survival condition exists; ordinary player breaks
without the predicate and ordinary explosion loot return no stained glass.

Each color's shaped crafting recipe surrounds its matching dye with eight ordinary glass blocks in
a `3x3` pattern and returns eight same-color stained-glass blocks in the `stained_glass` group. Its
recipe advancement unlocks from possession of ordinary glass or the recipe-unlocked trigger. A
separate two-row recipe consumes six same-color stained-glass blocks and returns sixteen matching
stained-glass panes. Matching, allocation, unlock publication and pane behavior remain with their
generic or pane owners.

The registered never-spawn predicate rejects every entity type on all sixteen states.
`SpawnUtil.Strategy.LEGACY_IRON_GOLEM` independently rejects any block whose runtime class is
`StainedGlassBlock` before testing the above cell or floor solidity, so villager golem-summon
searches cannot use any color as their candidate floor.

All sixteen identities are direct members of the reloadable `impermeable` block tag. Its only
locked runtime consumer remains `BeehiveBlock.trySpawnDripParticles`, whose vanilla caller passes
the beehive's own state rather than the state below it. Consequently stained-glass membership is
not tested by the current call path and produces no color-specific honey-drip effect.

The family adds no scheduled tick, random tick, use, attack, entity-contact, neighbor, redstone,
comparator or block-event callback.

**Client projection:**

Every property-free blockstate selects its matching `block/<color>_stained_glass` model. Each model
inherits `cube_all`, selects the same-color texture and marks that texture `force_translucent`.
The item definition selects the same block model.

`HalfTransparentBlock.skipRendering` omits a shared face only when the neighbor is the same block
identity. Two adjacent blocks of the same color therefore omit their internal face; different
stained-glass colors, plain glass and tinted glass do not satisfy that identity check and delegate
ordinary face culling. Terrain packets publish states `7098..7113`; beacon geometry and the
completed section colors remain with the beacon renderer.

**Branches and aborts:**

Sixteen identity/color pairs; Silk Touch match versus empty loot; full collision versus empty
visual shape; skylight propagation versus model translucency; first colored section, same-color
extension, recursive different-color average and ordinary transparent continuation; scan pause and
completion; generic spawn rejection versus class-wide legacy-golem rejection; present tag
membership versus unreachable tested state; same-identity versus different-color face rendering
are distinct.

**Constants and randomness:**

States `7098..7113` in dye order; hardness/resistance `0.3`; dampening `0`; shade brightness `1`;
friction `0.6`; common stack `64`; full collision/selection cube and empty visual shape; scan budget
`10` cells per ticker invocation; new section height `1`; component-wise average divisor `2`; Silk
Touch minimum `1`; stained-glass recipe output `8`; pane recipe input/output `6`/`16`. The family
consumes no RNG.

**Side effects:**

Generic placement/removal; conditional same-color drop; zero-dampening light propagation; beacon
section creation, extension and color replacement; rejected entity and legacy iron-golem support;
recipe outputs/unlocks; ordinary state, item, map-color and model projection. The locked
`impermeable` membership has no stained-glass runtime side effect in the current consumer path.

**Gates:**

Placement/removal authority; selected color identity; Silk Touch predicate; lighting query; beacon
scan position/list size/current color/world-surface boundary; entity-spawn predicate; legacy golem
strategy and above cell; active loot/recipe/tag snapshot; beehive caller state; client
neighbor/model context.

**Boundary cases and quirks:**

The full collision cube propagates skylight and has dampening `0`. The first colored block above a
beacon starts a raw-colored section rather than averaging with the initial white section. A later
raw color is compared against the current section's possibly averaged color, so returning to a
source dye can create another average instead of merging. Adjacent different colors render their
shared faces. The same reloadable tag membership remains non-interacting because the sole current
caller never supplies a stained-glass state.

**Evidence:**

`OFF-SERVER-001`, `OFF-CLIENT-001`, `OFF-REPORT-001`, `OFF-DATA-001`;
`net.minecraft.world.level.block.Blocks`,
`net.minecraft.world.level.block.ColorCollection#registerBlocks`,
`net.minecraft.world.level.block.StainedGlassBlock#getColor`,
`net.minecraft.world.item.DyeColor#getMapColor`,
`net.minecraft.world.item.DyeColor#getTextureDiffuseColor`,
`net.minecraft.world.level.block.TransparentBlock#propagatesSkylightDown`,
`net.minecraft.world.level.block.TransparentBlock#getVisualShape`,
`net.minecraft.world.level.block.TransparentBlock#getShadeBrightness`,
`net.minecraft.world.level.block.HalfTransparentBlock#skipRendering`,
`net.minecraft.world.level.block.state.BlockBehaviour#getLightDampening`,
`net.minecraft.world.level.block.entity.BeaconBlockEntity#tick`,
`net.minecraft.world.level.block.entity.BeaconBeamOwner$Section`,
`net.minecraft.util.ARGB#average`,
`net.minecraft.util.SpawnUtil$Strategy`,
`net.minecraft.world.entity.npc.villager.Villager#spawnGolemIfNeeded`,
`net.minecraft.world.level.block.BeehiveBlock#animateTick`,
`net.minecraft.world.level.block.BeehiveBlock#trySpawnDripParticles`;
`reports/blocks.json#*_stained_glass`,
`reports/minecraft/components/item/*_stained_glass.json`,
`data/minecraft/tags/block/impermeable.json`,
`data/minecraft/loot_table/blocks/*_stained_glass.json`,
`data/minecraft/recipe/*_stained_glass.json`,
`data/minecraft/recipe/*_stained_glass_pane.json`,
`data/minecraft/advancement/recipes/building_blocks/*_stained_glass.json`,
`assets/minecraft/blockstates/*_stained_glass.json`,
`assets/minecraft/models/block/*_stained_glass.json`,
`assets/minecraft/items/*_stained_glass.json`.

**Test vectors:**

Run `EXP-BLK-040` across all sixteen identities, placement, Silk Touch levels `0/1`, explosion
loot, shape/light/map-color queries, beacon columns containing first/repeated/alternating colors
across ten-cell and surface boundaries, generic and legacy-golem support, beehive/tag reload,
crafting, save/reload and same/different-color model neighbors. Assert states, colors, sections,
component averages, heights, writes/drops, predicates, recipes and faces/models.

**Limits:**

This leaf does not re-specify generic placement/break packets, light propagation, beacon
activation/effects/render geometry, entity spawning, villager summon search, beehive particle
geometry, crafting allocation, tag publication, map shading or model loading. Those remain with
`BLK-002`, `PLY-006`, `ENV-LIGHT-001`, `BLK-BEACON-001`, entity/mob owners, DataReload,
`ITM-RECIPE-001` and `CLI-006`.
