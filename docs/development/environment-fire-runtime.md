# Environment Fire Runtime

`G01-P5-S012` implements the `SourceSpecified` `ENV-FIRE-001` slice. The
`ferrite-gameplay::environment::fire` module owns protocol-neutral fire decisions; Region
adapters provide ordered block, weather, player, attribute and RNG observations and commit the
resulting schedules, writes, removals, portal requests, TNT effects and entity-contact effects.

## Closed content and state ownership

The runtime contains the complete 207-entry Java 26.2 `(ignite odds, burn odds)` table as explicit
registrations. Tests prove path uniqueness and exact counts for all eleven odds groups. Unknown
blocks and every waterlogged registered state resolve to `0/0`.

Ordinary Fire owns age `0..15` and its five directional properties. Placement uses an all-false
shape over sturdy or flammable support; otherwise it derives each property from the corresponding
neighbor. Shape updates preserve age, recompute those properties, can switch to default Soul Fire
over Soul Sand/Soil, and return Air when support is lost. Soul Fire has no ordinary scheduled
callback or age/spread table.

## Scheduled callback transaction

Placement and each admitted ordinary callback schedule `30 + nextInt(10)`. A callback schedules
its successor before the strict nearby-nonspectator gate and then executes:

1. survival removal without early return;
2. dimension-selected infiniburn;
3. horizontally short-circuited Rain extinction;
4. captured-state age update;
5. no-fuel/age-15 self-removal;
6. positional increased-burnout resolution;
7. East, West, Below, Above, North and South direct fuel transactions;
8. the 53-position X-outer, Z-middle, Y-inner empty-space scan.

The module exposes branch-local draw-consumption results. Direct checks always consume their first
bounded draw, including waterlogged/unregistered targets. Replacement/removal results are ignored;
captured TNT primes only after that mutation. Empty-space work consumes no draw for occupied,
zero-encouragement or zero-threshold candidates, uses the inclusive comparison, performs Rain
rejection after it, then selects Ordinary/Soul Fire from current support.

The eight increased-burnout biome paths are explicit. Their direct denominators decrease by 50,
while their spatial threshold is integer-divided by two. Infiniburn skips Rain and self-removal but
does not skip aging or outward work.

## Portal, TNT and contact boundaries

Overworld/Nether placement tries portal axis X then Z before survival removal. Placement of
ordinary Fire still schedules after portal creation or removal, leaving the scheduler to reject a
stale current block. `canBePlacedAt` keeps the Air, adjacent-Obsidian, clicked-face preference,
vertical axis draw and fallback-axis gates; frame recognition/construction remains
`WGEN-PORTAL-001`.

TNT rule denial occurs after Fire has already replaced or removed the block. When priming is
enabled, a centered entity admission failure does not suppress the primed sound or `PRIME_FUSE`.
Fuse/explosion behavior remains `RED-EXPLOSION-001`.

Base-fire contact requests `CLEAR_FREEZE`, then `FIRE_IGNITE`, then queues `in_fire` damage.
Negative fire counters increment without RNG; only nonnegative ServerPlayer counters consume the
`nextInt(1,3)` draw. Crossing nonnegative sets eight seconds of ignition. Ordinary/Soul queued
damage is respectively `1.0F`/`2.0F`; later effect merging, immunity and recurring damage remain
`ENT-EFFECT-001`.

## Region determinism

All block queries and writes are Region-local and retain authoritative traversal order. Cross-Region
adapters must pass boundary observations in logical tick/phase order; mailbox arrival order cannot
become Fire ordering. Player proximity uses exact positions against the Fire block's integer
corner. Presentation sound/particles remain `CLI-EFFECT-001`.

## Verification

The committed test owner is
`crates/ferrite-gameplay/tests/slices/environment/env_005.rs`. Its 13 tests cover the 207-ID table,
placement/shape/Soul behavior, portal axes and stale scheduling, radius `-1/0/128`, callback order,
infiniburn/biomes, Rain probe/chance equality, age `3/4/15`, all direct denominators and
post-mutation TNT, all 53 offsets/Y denominators, zero/equality/post-draw spatial branches,
TNT admission failure and every contact-counter class.
