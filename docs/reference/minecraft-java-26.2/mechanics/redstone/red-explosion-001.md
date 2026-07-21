# Redstone mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `RED-EXPLOSION-001` — Explosion calculation, entity effects, block effects, and fire are separate phases

**Parent:** `RED-006`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked source fixes every sample, arithmetic step, RNG site and phase order;
`EXP-RED-004` is a regression trace rather than semantic evidence.

**Applies when:**

Gameplay creates an explosion with a level, source, center, radius, damage source, block-interaction
mode, and optional fire flag.

**Authoritative state:**

Explosion parameters, source/owner, affected-block candidate set/order, entities and exposure, block
interaction mode, gamerules, loot contexts, fire flag and RNG.

**Transition and ordering:**

Emit `GameEvent.EXPLODE`, calculate the affected-position list, hurt entities, optionally interact
with blocks, then optionally create fire. The returned count is the sampled unique-position count
even for `KEEP`, not the number actually destroyed.

Affected positions come from the boundary cells of a `16×16×16` direction cube: iterate X, then Y,
then Z from `0..15`, retaining a cell when any coordinate is 0 or 15. Normalize
`(coordinate / 15 * 2 - 1)` and start at the exact center with power
`radius * (0.7 + random.nextFloat() * 0.6)`. Each step examines
`BlockPos.containing(x,y,z)`, stops outside world bounds, subtracts
`(max(blockResistance, fluidResistance) + 0.3) * 0.3` when non-air/nonempty resistance exists,
adds the position when remaining power is positive and the calculator/source admits it, advances
0.3 along the ray, then subtracts `0.22500001F`. A hash set deduplicates positions; its list order is
irrelevant until the later explicit RNG shuffle.

Entity selection uses the integer-floor AABB from `center ± radius*2 ± 1`, excluding the direct
source in the level query. Ignore `ignoreExplosion`; otherwise require center distance divided by
`radius*2 <= 1`. Direction begins at TNT position or every other entity's eye position and is
normalized from center. Exposure samples the entity AABB at increments
`1/(extent*2+1)` on X/Y/Z, with the source-defined centering offsets on X/Z; each ray clips collider
blocks and no fluid, and exposure is misses divided by total samples. Exposure is skipped only when
damage is disabled and knockback multiplier is zero.

Default damage is `((q*q + q)/2 * 7 * radius*2 + 1)` as float, where
`q=(1-normalizedDistance)*exposure`. Damage occurs before knockback. Knockback power is
`(1-normalizedDistance) * exposure * calculatorMultiplier *
(1-living explosion_knockback_resistance)` along the normalized direction. The server pushes every
eligible entity, redirects redirectable projectiles to the damage-source entity, records the vector
for non-spectator players except creative flight, then calls `onExplosionHit` even when damage was
disabled.

For any interaction except `KEEP`, shuffle the sampled list with level RNG, re-read each current
block state and invoke its `onExplosionHit`. Drops are scanned against collectors in insertion order;
merge-compatible stacks merge up to 16 and any remainder continues through later collectors before a
new collector retains the callback position. Only after every callback are collectors popped in
collector order. Fire then scans the same shuffled list and consumes `nextInt(3)` for every sampled
position; on zero it writes the derived base-fire state only when the resulting position is air and
the block below is solid-rendering.

**Branches and aborts:**

Entity work is skipped only when radius is below `1.0E-5F`; ray sampling still runs for any radius.
`KEEP` skips block callbacks but not sampling, entities or optional fire. Resistance/source hooks can
exclude positions. Block callbacks own keep/destroy/trigger behavior and their gamerule/loot gates.
`TRIGGER_BLOCK` reports trigger permission, except breeze wind charge additionally requires
`mobGriefing`. Blocklike-entity effects exclude both wind-charge types and, with mob griefing off,
require the interaction mode to opt in. Fire false or support/RNG failure skips writes.

**Constants and randomness:**

Ray initialization consumes one float per boundary direction (1,352 rays). Block callbacks and loot
may consume more RNG after the one list shuffle; fire consumes one bounded integer per sampled list
entry after all block/drop work. Damage/exposure themselves consume no explosion RNG. Preserve the
float/double promotions shown above and the fixed 0.3/0.22500001F constants.

**Side effects:**

Entity damage/knockback/velocity notification, block callbacks/removal/transformation, item drops,
fire states, game events, sounds, particles and source-specific criteria.

**Gates:**

Block interaction mode, `mobGriefing` or explosion-decay gamerules selected by caller, damage
immunity/tags, exposure/collision, block resistance/hooks, drops/fire flags and chunk writability.

**Boundary cases and quirks:**

Affected block collection is ray sampled, not all blocks inside a sphere. Exposure tests entity
collision rays against the entity's own level. A zero-length direction normalizes according to the
locked vector implementation. Drop collectors keep the first contributing position even after later
merges. Optional fire is post-destruction, tests the resulting world and can act on sampled
positions whose block callback kept or transformed them.

**Evidence:**

`Confirmed`; `OFF-SERVER-001`; `ServerExplosion`, `ExplosionDamageCalculator`,
`EntityBasedExplosionDamageCalculator`, `ServerExplosion$StackCollector`; `EXP-RED-004`.

**Test vectors:**

Radius below/equal/above `1.0E-5`; exact 1,352-ray RNG trace; world-bound exit; block/fluid resistance
and source override; entity at normalized distance 0/1/above 1 with 0/partial/full exposure and each
immunity/knockback/player/projectile branch; each interaction/gamerule; shuffled callbacks that
mutate later positions; compatible drops across the 16 cap; fire on changed air/support states.
