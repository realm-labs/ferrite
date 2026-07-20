# Redstone mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `RED-EXPLOSION-001` — Explosion calculation, entity effects, block effects, and fire are separate phases

**Parent:** `RED-006`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Cross-checked`

**SourceConclusion:**

`SourceInconclusive` — sampling vectors, traversal order, drop merging, and exposure-point
arithmetic remain unexpanded.

**Applies when:**

Gameplay creates an explosion with a level, source, center, radius, damage source, block-interaction
mode, and optional fire flag.

**Authoritative state:**

Explosion parameters, source/owner, affected-block candidate set/order, entities and exposure, block
interaction mode, gamerules, loot contexts, fire flag and RNG.

**Transition and ordering:**

Construct the explosion; sample outward rays through block/fluid resistance to collect unique
affected positions; find entities in the radius AABB and for each eligible entity derive normalized
distance and line-of-sight exposure, then apply damage and knockback; if block interaction is
enabled, randomize/process affected positions, invoke block explosion hooks and drop merging through
explosion loot context; finally attempt fire placement only at eligible affected air positions.
Anchor: `net.minecraft.world.level.ServerExplosion#explode()` and
`net.minecraft.world.level.ServerExplosion#interactWithBlocks(java.util.List)`.

**Branches and aborts:**

Radius/noninteraction produces no affected blocks; source immunity; entity outside normalized radius
or zero exposure; block/fluid resistance exhausts ray power; block interaction mode
keeps/destroys/triggers; drops disabled; fire false or support/roll fails. Entity effects and block
effects must not be skipped merely because the other phase has no targets.

**Constants and randomness:**

Radius and damage/knockback calculations use float/double source arithmetic; ray grid, resistance
attenuation, affected-list shuffle, drop survival/merging and fire placement consume explosion RNG
in their phase order. Exact numeric and RNG sequence are owned by `EXP-RED-004`.

**Side effects:**

Entity damage/knockback/velocity notification, block callbacks/removal/transformation, item drops,
fire states, game events, sounds, particles and source-specific criteria.

**Gates:**

Block interaction mode, `mobGriefing` or explosion-decay gamerules selected by caller, damage
immunity/tags, exposure/collision, block resistance/hooks, drops/fire flags and chunk writability.

**Boundary cases and quirks:**

Affected block collection is ray sampled, not all blocks inside a sphere. Exposure uses collision
geometry. Multiple destroyed stacks may merge with an explosion-specific cap/order. Optional fire is
post-destruction and therefore tests the resulting world.

**Evidence:**

`Confirmed` phase structure; numeric/RNG parity `Implementation blocker`; `OFF-SERVER-001`; locators
above; `EXP-RED-004`.

**Test vectors:**

Radius zero; entity with 0/partial/full exposure; high-resistance fluid/block; each interaction
mode/gamerule; overlapping drops; fire enabled with valid/invalid support; fixed RNG trace of ray,
shuffle, drops and fire.
