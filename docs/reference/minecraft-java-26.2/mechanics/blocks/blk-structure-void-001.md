# Blocks mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `BLK-STRUCTURE-VOID-001` — Structure void is an invisible replaceable template sentinel

**Parent:** `BLK-001`, `BLK-002`, `BLK-003`, `BLK-005`, `PLY-006`, `RED-004`, `ENV-001`,
`WGEN-003`, `CLI-006`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — the locked block/item registration and class fix its one state, replacement,
collision, selection, particle and item presentation. Three exact consumers give the ID additional
meaning: flowing fluid rejects it, structure capture always omits it, and jigsaw final-state
replacement treats it as a skipped cell. The structure-block renderer owns its only world-visible
debug projection.

**Applies when:**

`minecraft:structure_void` is placed, replaced, broken, pushed, targeted, encountered by flowing
fluid, captured into a named structure, parsed as a jigsaw connector's final state or scanned by an
operator structure block's invisible-cell renderer.

**Authoritative state:**

The block has one property-free state, ID 14851, and no block entity. Registration starts from
default properties and adds replaceable, no collision, no loot table, no terrain particles and
`PushReaction.DESTROY`. The locked block tag also includes it in `minecraft:replaceable`.

Its ordinary epic-rarity block item stacks to 64 and has no special data component. It is a normal
`BlockItem`, not a `GameMasterBlockItem`: possession and the generic placement transaction, rather
than `canUseGameMasterBlocks`, determine whether a player can place it. The operator creative tab is
an acquisition/UI boundary, not a placement permission check.

**Transition and ordering:**

Placement uses the generic block-item transaction. A replaceable target can be overwritten without
first producing drops; structure void itself is likewise an eligible replacement target. Breaking
or replacement produces no block loot. A piston resolver classifies its explicit DESTROY reaction
for the destruction list rather than the movement list, and that destruction also has no block
drop. The block adds no scheduled tick, random tick, use, neighbor, entity-contact, redstone or
comparator callback of its own.

The block has an empty collision shape. Its selection/raycast shape is a centered 6-by-6-by-6 voxel
cube from `(5,5,5)` through `(11,11,11)` in sixteenths, so a ray may target only that small center
despite the block occupying a world cell. Shade brightness is always 1.0. Its render shape is
INVISIBLE and terrain-particle generation is disabled.

Flowing fluid's hard-coded `canHoldAnyFluid` exclusion returns false for structure void before the
generic motion-blocking and liquid-container checks. Water or lava therefore cannot select an
existing structure-void cell as a holdable flow destination. Replacement/removal can expose the
cell to a later ordinary fluid update, but the state itself has no waterlogged/fluid property.

Every `StructureBlockEntity.saveStructure` call appends STRUCTURE_VOID to the caller's ignored-block
list before `StructureTemplate.fillFromWorld`. A captured structure-void cell is therefore absent
from the template, not encoded as air or as a structure-void block. This applies equally to the
structure-block and test-instance save paths. The release placement path does not run the disabled
debug prefill branch described by `BLK-STRUCTURE-001`.

`JigsawReplacementProcessor` gives the ID a separate output-sentinel role. For a jigsaw input cell,
it parses `final_state` (defaulting to air); a valid structure-void result returns no output block
info, so the connector cell is skipped and the existing world cell remains. Parse failure also
skips after logging. Any other valid final state returns that state with no NBT. This sentinel check
does not make arbitrary raw structure-void entries magical during generic template placement; the
exact jigsaw and structure-template owners retain their other processor/write gates.

**Client projection:**

The blockstate points at a model containing only the structure-void particle texture, while the
block's INVISIBLE render shape submits no ordinary world model. The item uses the generated
`item/structure_void` texture.

Its special world projection is delegated to an admitted structure-block renderer. Under the
permission, SAVE/show-invisible, positive-size and 96-block boundaries fixed by
`BLK-STRUCTURE-001`, a scanned structure-void cell becomes an opaque pale-red outline around the
center box from 0.45 through 0.55 on each axis. Outside that renderer it remains invisible; merely
holding the item does not replace the class's INVISIBLE render shape.

**Branches and aborts:**

Generic place/break admission; replaceable versus occupied target; piston move/destroy rejection;
fluid target versus later exposed cell; captured versus raw template input; jigsaw/non-jigsaw and
valid/invalid/structure-void final state; ordinary versus admitted structure-block rendering are
distinct observable branches.

**Constants and randomness:**

State ID 14851; selection cube size 6 at coordinates 5..11; shade brightness 1.0; item stack 64 and
epic rarity; structure-render center box 0.45..0.55, pale-red `(1.0,0.75,0.75)` and view distance 96.
This leaf consumes no RNG.

**Side effects:**

Generic block replacement/break/piston destruction; no loot or terrain particles; fluid-flow
rejection; omitted structure-capture coordinate; skipped jigsaw connector output; item/world model
selection and conditional structure-block debug geometry.

**Gates:**

Generic placement and break rules; replaceability; piston resolver; fluid holdability; structure
capture ignore list; jigsaw input and parsed final state; structure-block render permission, mode,
show-invisible flag, size and distance.

**Boundary cases and quirks:**

The item is not operator-gated after acquisition. Empty collision does not mean an empty selection
shape. Replaceable/no-loot and piston-destroy compose without a drop. Capture omission means
“preserve whatever is at this coordinate on later placement,” whereas captured air is an explicit
air write. Jigsaw final-state structure void reaches the same no-output result by a different
processor branch. Ordinary rendering remains invisible even though the structure-block renderer
can visualize the cell.

**Evidence:**

`OFF-SERVER-001`, `OFF-CLIENT-001`, `OFF-REPORT-001`, `OFF-DATA-001`;
`net.minecraft.world.level.block.StructureVoidBlock#getRenderShape`,
`net.minecraft.world.level.block.StructureVoidBlock#getShape`,
`net.minecraft.world.level.block.StructureVoidBlock#getShadeBrightness`,
`net.minecraft.world.item.Items#registerBlock`,
`net.minecraft.world.level.material.FlowingFluid#canHoldAnyFluid`,
`net.minecraft.world.level.block.entity.StructureBlockEntity#saveStructure`,
`net.minecraft.world.level.levelgen.structure.templatesystem.JigsawReplacementProcessor#processBlock`,
`net.minecraft.client.renderer.blockentity.BlockEntityWithBoundingBoxRenderer#extractRenderState`,
`net.minecraft.client.renderer.blockentity.BlockEntityWithBoundingBoxRenderer#renderInvisibleBlocks`;
`reports/blocks.json#minecraft:structure_void`,
`reports/minecraft/components/item/structure_void.json`,
`data/minecraft/tags/block/replaceable.json`,
`assets/minecraft/{blockstates,models/block,models/item,items}/structure_void.json`,
`assets/minecraft/textures/item/structure_void.png`.

**Test vectors:**

Run `EXP-BLK-029` across generic placement/break, every replacement and piston outcome, fluid flow,
structure capture/reload, raw template input, jigsaw final-state parsing and structure-block render
admission. Assert exact state/shape/model values and distinguish absent template coordinates from
explicit air.
