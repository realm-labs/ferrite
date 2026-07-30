# Redstone Explosion Runtime

`G01-P5-S016` implements the `SourceSpecified` `RED-EXPLOSION-001` slice. The
`ferrite-gameplay::redstone::explosion` module separates ray sampling, entity geometry/effects,
block interaction and drop collection, fire admission, and top-level transaction order. Region
adapters own live world reads and writes, entity storage, damage dispatch, named RNG streams,
callback execution, loot, presentation, and packet projection.

## Ray sampling

Every explosion walks the boundary of a `16×16×16` direction cube in X/Y/Z order. This is exactly
1,352 directions and therefore exactly 1,352 float draws, including zero or negative radius cases
where no ray body runs. Direction components use the locked float operations before promotion to
double, rays start at the exact center, and `BlockPos` conversion floors each double coordinate.

Each admitted step reads the current block/fluid observation and aborts that ray immediately when
the position is outside world bounds. Optional source-adjusted resistance subtracts
`(resistance + 0.3F) * 0.3F`. The calculator's admission hook then receives the resulting remaining
power. An admitted ray advances by promoted `0.3F` components and loses `0.22500001F` at the loop
edge.

The runtime returns a deduplicated affected-position set and the exact draw/inspection counts.
Region integration materializes that set once in deterministic order. The same list is then used
for the optional Fisher–Yates block shuffle and the later fire pass; no mailbox receiver may
reconstruct or reorder it.

## Entity geometry and effects

Entity work is absent only below radius `1.0E-5F`; ray draws remain independent of that gate. Query
bounds floor `center ± radius*2 ± 1` and exclude the direct source. Per-entity admission rejects
`ignoreExplosion` and normalized center distances above one. TNT supplies its position as the
effect origin; callers supply eye position for every other entity.

Exposure uses the locked `1/(extent*2+1)` increments and X/Z centering offsets. Every point clips
collider blocks with no fluid in the entity's own level. The pure runtime exposes the point stream
through a miss callback and returns misses/sample count. Invalid negative steps return zero without
clipping.

Damage uses `((q*q+q)/2*7*radius*2+1)` after the caller's damage gate. Exposure is skipped only
when damage is disabled and knockback multiplier is zero. Knockback uses the source multiplier and
unclamped living explosion resistance, then every admitted entity is pushed even for a zero
vector. Redirectable projectiles take the damage-source entity as owner before the player branch;
eligible non-spectator, non-creative-flight players record their vector. `onExplosionHit` remains
last and is present when damage is disabled.

## Block callbacks, drops, and fire

`KEEP` alone skips the block phase. Every other interaction consumes a reverse Fisher–Yates draw
for each bound from list length through two, re-reads current block state in shuffled order, and
invokes its explosion callback. `TRIGGER_BLOCK` alone reports trigger permission; breeze wind
charges additionally require `mobGriefing`. Both wind-charge kinds are excluded from blocklike
entity effects. With griefing off, other sources require `DESTROY` or `DESTROY_WITH_DECAY`.

Drop collectors preserve insertion order and their first contributing position. A compatible
stack first satisfies the item's intrinsic merge predicate, then applies the source's signed
`min(maxStackSize, 16) - collectedCount` transfer arithmetic. Ordinary inputs therefore merge only
up to count 16; an oversized first stack is normalized to 16 and moves its excess into the incoming
remainder. Any remainder scans later collectors and finally creates a collector at the current
callback position. Drops pop only after every callback and in collector order.

Optional fire runs after block callbacks and drop collection over the same target list. It consumes
one `nextInt(3)` result for every entry before testing state. A zero result writes derived base fire
only when the resulting position is Air and the current block below is solid-rendering.

## Transaction and Region determinism

The fixed phase order is explode game event, affected-position calculation, entity effects,
optional profiled block callbacks and drop pops, optional fire, then return of the sampled unique
count. The result is not the number of blocks destroyed and remains populated for `KEEP`.

The Region coordinator reserves the explosion's named RNG stream and affected ownership before
execution. A cross-Region explosion carries the immutable center/parameters, ordered target list,
entity effect plans, RNG cursor, and generation fence. Boundary Regions return current observations
and staged effects to the coordinator; the coordinator commits in source order or rejects the
whole attempt before callbacks. Callback and loot RNG stays on the same stream between shuffle and
fire, so retries restore the pre-explosion cursor instead of consuming a second trace.

## Verification

The committed owner is `crates/ferrite-gameplay/tests/slices/redstone/red_006.rs`. Its 17 tests lock
ray/draw cardinality, post-resistance admission power, bound aborts, entity query and exposure
geometry, damage/knockback/routing order, interaction gates, shuffle draws, collector merging, fire
draws against resulting state, and the complete transaction order.
