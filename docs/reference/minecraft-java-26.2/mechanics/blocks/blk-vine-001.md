# Blocks mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `BLK-VINE-001` — Vines add supported faces and spread through a density-bounded random walk

**Parent:** `BLK-003`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — the locked `VineBlock` class fixes all 32 states, placement and replacement,
support repair, random-tick admission, local-density scan, every directional growth branch and RNG
draw. The sole locked reader of `spread_vines` is this random-tick method. Generic block mutation,
random-tick dispatch, fire, climbing, breaking and loot execution remain delegated to their
existing owners.

**Applies when:**

`minecraft:vine` is placed into air or an existing vine, receives a neighbor shape update, is
sampled for a server random tick, is transformed, or is broken for its locked loot table.

**Authoritative state:**

The five booleans `up`, `north`, `east`, `south`, `west`; neighbor states and attachment faces;
build-height endpoints; all vine states in the local density box; `spread_vines`; the callback
`RandomSource`; mutation results; tool identity; and the generic tick/update/loot state owned by
the parent mechanics.

**Transition and ordering:**

**State, placement and support:**

The registry contains the Cartesian 32-state schema and an all-false default. The all-false state
is representable but cannot survive. Each true property contributes the corresponding one-block
face sheet to the outline; the otherwise empty union deliberately falls back to a full-block
outline. Vines propagate skylight downward and use no collision volume of their own; climbability,
replaceability and fire behavior are selected by locked tags and `ENV-FIRE-001`.

A face other than `DOWN` has direct support when `MultifaceBlock#canAttachTo` accepts the neighbor
at `pos.relative(face)`. A horizontal face may instead hang from a vine immediately above that has
the same face. `DOWN` can never be a property or support face. Placement starts from the clicked
vine state or the all-false default and visits `BlockPlaceContext#getNearestLookingDirections` in
its supplied order. It skips `DOWN`, already-present faces and unsupported faces, returning after
adding the first eligible face. If none is eligible, placement returns the unchanged existing vine
or `null` for a new vine. An existing vine is replaceable by another vine item only while fewer
than all five faces are set; other replacement decisions delegate to `Block`.

For a neighbor shape update from `DOWN`, VineBlock delegates unchanged to the superclass path.
Every other direction recomputes `up` from direct ceiling support and each currently true
horizontal face from direct support or the matching face of the vine above; false faces never turn
true during repair. Zero surviving faces returns air. Rotation and mirror operations permute the
four horizontal properties through their standard direction mapping and retain `up`.

**Random-tick admission and density:**

`spread_vines` is an `UPDATES` boolean with default `true`. False returns before any RNG. True
consumes `nextInt(4)` and returns unless it is zero; the admitted branch then selects one of the six
directions through `Direction#getRandom` and computes the position above the source.

Every branch that calls `canSpread` scans the inclusive box
`[x-4,x+4] × [y-1,y+1] × [z-4,z+4]` in `BlockPos#betweenClosed` order. A counter starting at five is
decremented for every vine and rejects immediately on the fifth, including the source vine. Thus
growth is allowed with at most four total vines in the 243-cell box. The scan consumes no RNG.

**Horizontal direction:**

If the source already has the selected horizontal face, this tick ends. Otherwise density must
admit growth. Let `target = pos.relative(direction)`, with clockwise and counterclockwise directions
`cw` and `ccw`.

- If target is not air, direct acceptable support at target in the selected direction adds that
  face to the source and writes the source with flags `2`; otherwise nothing changes.
- If target is air, the first matching branch wins: source has `cw` and target's `cw` neighbor is
  acceptable, so place a default vine at target with `cw`; otherwise perform the symmetric `ccw`
  test.
- Failing both, if source has `cw`, the diagonal `target.relative(cw)` is empty and the neighbor at
  `pos.relative(cw)` accepts the selected direction's opposite face, place the diagonal vine with
  that opposite face; otherwise perform the symmetric `ccw` test.
- Failing all four side routes, consume one `nextFloat`; only a value strictly below `0.05` plus
  acceptable support above target places a default vine at target with `up=true`.

Every placement/write uses flags `2`, ignores the returned boolean and ends the tick; no later
branch retries a rejected write.

**Up direction:**

The source Y must be strictly below the level maximum. If the source can attach upward, set its
`up` face with flags `2` and return without a density scan or more RNG. Otherwise the above cell
must be air and density must admit. Starting from the source state, visit the four horizontal plane
directions in locked iteration order and consume one `nextBoolean` for each. A face is retained only
when its draw is false and the corresponding neighbor of the above cell is acceptable; every other
horizontal face is cleared. If any horizontal face remains, write that state above with flags `2`.
No write occurs when all four are cleared.

**Down direction:**

The source Y must be strictly above the level minimum, and the below cell must be air or vine.
Start with the all-false default for air or the existing below-vine state. In locked horizontal
iteration order, consume four `nextBoolean` draws; when a draw is true and the source has that face,
set it on the candidate. Write below with flags `2` only when the candidate differs from its starting
state and has at least one horizontal face. `up` is never copied by this helper.

**Breaking and data:**

The item has the locked ordinary block-item components and stack size `64`. The exact block loot
table has one pool and returns one vine only when `match_tool` identifies `minecraft:shears`;
otherwise it returns nothing. Its random sequence is `minecraft:blocks/vine`. Generic break
admission, tool durability, stack insertion and loot execution remain `BLK-BREAK-001` and
`ITM-LOOT-001`.

**Branches and aborts:**

Existing/new placement; direction `DOWN`; occupied face; direct/inherited/no support; repair leaves
one/zero faces; rule false; three-in-four tick rejection; six chosen directions; horizontal target
air/nonair; density fifth vine; clockwise/counterclockwise/diagonal/upward fallback; build-height
edge; above/below air/vine/other; random face subset empty/nonempty; failed write; shears/other tool.

**Constants and randomness:**

Five boolean properties and 32 states; flags `2`; random-tick admission `1/4`; six-way direction;
density radii `(4,1,4)`, 243 cells and fifth-vine rejection; horizontal ceiling fallback
`nextFloat < 0.05`; four unconditional booleans in each admitted upward-copy or downward-copy
branch. All draws use the callback RNG in the exact branch order above. Disabled, density-rejected,
occupied-face, direct-up-support and most solid-target branches consume no draws after their stated
gate.

**Side effects:**

At most one flags-2 block write per random tick, placement state selection, neighbor-driven face
removal or air replacement, inherited break/loot effects and ordinary client block-state
projection. The growth callback emits no direct sound, particle, game event, item or scheduled tick.

**Gates:**

Placement order and replaceability, face support, neighbor direction, random-tick activity,
`spread_vines`, admission draw, chosen direction, build height, density, target state, branch-local
support/emptiness, retained horizontal face and tool predicate.

**Boundary cases and quirks:**

The invalid all-false registered state renders with a full outline before support repair removes it.
The density box counts the source, so four total vines pass and five fail. Horizontal growth checks
density before reading its target, while direct upward face addition does not check density. The
upward branch clears a face when its coin is true; the downward branch copies a face when its coin
is true. Ignored `setBlock` failure never falls through to another candidate.

**Evidence:**

`OFF-SERVER-001`; `OFF-REPORT-001`; `OFF-DATA-001`; `VineBlock#<init>`, `#getShape`,
`#propagatesSkylightDown`, `#canSurvive`, `#canSupportAtFace`, `#isAcceptableNeighbour`,
`#getUpdatedState`, `#updateShape`, `#randomTick`, `#copyRandomFaces`, `#hasHorizontalConnection`,
`#canSpread`, `#canBeReplaced`, `#getStateForPlacement`, `#rotate`, `#mirror`;
`GameRules#SPREAD_VINES`; `reports/blocks.json#minecraft:vine`;
`reports/minecraft/components/item/vine.json`; `data/minecraft/loot_table/blocks/vine.json`;
`EXP-BLK-015`.

**Test vectors:**

All 32 states and every support/removal/rotation/mirror mapping; new and existing placement with
every ordered direction mask; four versus five vines in the complete density traversal; scripted
RNG for every direction and each horizontal winner/fallback; exact `0.05` boundary; Y at/below max
and at/above min; all 16 horizontal source masks with every four-boolean stream above and below;
air/vine/other targets and failed writes; rule false asserts zero draws; shears and every nonmatching
tool assert exact loot.
