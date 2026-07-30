# Random Tick Runtime

`G01-P5-S019` implements `SIM-RANDOM-ACTIVITY-001` in
`ferrite-simulation::random_tick`. The runtime separates the level position stream, simulation
ticket storage, fastutil-compatible activity ordering, and chunk/section sampling. Concrete block,
fluid, and precipitation callbacks remain in their content owners.

## Tick admission and ticket activity

Every chunk-cache invocation propagates queued distance updates. Stale tickets purge only while
gameplay runs normally; debug levels still update inhabited time but skip gameplay chunk work.
Normal non-debug work constructs natural-spawn state, reads the spawn and random-tick gamerules,
shuffles spawning chunks, performs spawning and thunder, and only then enters precipitation/random
ticks. Those earlier stages must use the same level gameplay RNG instance.

The nine ticket types retain their exact timeout and flags:

| Type | Timeout | Flags |
|---|---:|---|
| `player_spawn` | 20 | loading |
| `spawn_search` | 1 | loading |
| `dragon` | none | loading, simulation |
| `player_loading` | none | loading |
| `player_simulation` | none | simulation, keep-active |
| `forced` | none | persist, loading, simulation, keep-active |
| `portal` | 300 | persist, loading, simulation, keep-active |
| `ender_pearl` | 40 | loading, simulation, keep-active |
| `unknown` | 1 | loading, expire-if-unloaded |

Simulation selection ignores loading-only tickets and chooses the lowest numeric level. Duplicate
type/level admission resets the existing timeout. An eligible purge decrements before testing and
removes only when the signed 64-bit counter is negative, so timeout one survives at zero. Frozen
ticks do not decrement. Ordinary timed tickets pause while an updating holder is not ready for
saving; `expire-if-unloaded` bypasses that holder gate.

The distance-graph adapter writes every propagated result below 33 into
`SimulationChunkTracker`. Random work uses the entity-ticking predicate `level <= 31`; level 32
remains valid for other block-ticking gates but is excluded here. A visible holder and non-null
ticking chunk are also required.

## Compatibility iteration order

The tracker reproduces the observable `Long2ByteOpenHashMap` 8.5.18 layout rather than sorting
chunks. It uses expected size 16, load factor 0.75, a 32-slot initial table, the locked fastutil
64-bit mix, linear probing, cluster shifts, high-to-low reinsertion during resize, a special packed
key-zero slot, and iterator scanning from high table slots to low. Packed key zero is emitted
first.

Insertion, removal, and resize history can therefore change callback order even for the same final
key set. Tests lock vectors produced directly by the audited fastutil artifact before and after the
25th-entry resize and several cluster-removing updates. Region storage may use its own maps, but the
level activity coordinator must feed graph changes to this compatibility view in source order and
must not sort its output.

## Position and gameplay randomness

`RandomPositionStream` is a separate signed-32-bit wrapping generator:

```text
rand = rand * 3 + 1_013_904_223
q = rand >> 2
x = baseX + (q & 15)
y = baseY + ((q >> 16) & 15)
z = baseZ + ((q >> 8) & 15)
```

Arithmetic is Java-style wrapping. The initial value comes from startup randomness in vanilla and
is not world-seed-derived or part of the ordinary world save. Ferrite must record the chosen
runtime value in recovery/replay state used by a live authority handoff, while retaining that it is
not a same-seed world-generation promise.

The gameplay RNG remains one ordered per-level stream shared with spawning shuffle, thunder,
precipitation selection, block callbacks, fluid callbacks, and later chunks. It must not be
replaced with independently seeded per-Region random streams for this compatibility surface.
Callback-owned draws intentionally shift all later consumers.

## Chunk and section sampling

Each admitted chunk performs exactly `random_tick_speed` precipitation draws with bound 48. Only a
zero advances the position stream and invokes precipitation. Misses do not consume position
samples. The gamerule defaults to three, accepts zero through signed-32 maximum, and has no lower
operational clamp below that maximum.

For positive speed, sections are visited bottom-to-top. `is_randomly_ticking` is read once when a
section is reached. An ineligible section consumes neither stream. Every admitted section then
performs exactly `speed` position samples even if an early callback removes its final eligible
state. A callback can make a later section eligible before that later one-time check.

Each attempt reads one state from the section and retains that captured value. An eligible block
callback runs first. The fluid predicate and callback then use the same captured state rather than
rereading the mutated section or world. Both receive the shared gameplay RNG. Repeated positions
are valid, skipped/inactive time accumulates no samples, and reactivation creates no catch-up work.

`SectionRandomTickCounts` maintains signed-short block and fluid counts with wrapping replacement
updates and full-section recomputation. Eligibility is `block > 0 || fluid > 0`; the counts are an
admission optimization, never a queue of owed callbacks.

## Region integration

This batch provides the exact activity and sampling semantics but does not move mutable chunks
between nodes. `G01-P5-B1` binds ticket-source graph updates, holder readiness, chunk snapshots,
level RNG/position state, callback transactions, and durable Region handoff.

Because callback RNG consumption makes the per-level chunk order observable, concurrently owned
Regions cannot execute this surface independently and later merge results. The integration must
form an ordered consistency island for the admitted random-work sequence, while unrelated levels,
maps, and nonconflicting phases remain parallel. Cross-Region callback effects travel through the
existing bounded semantic-command and commit barriers.

## Verification

The committed owner is `crates/ferrite-simulation/tests/slices/sim_004.rs`. Its tests cover the
position vector, level 31/32 boundary, exact fastutil history vectors, holder gates, all ticket
flags and timeouts, duplicate reset, freeze/save expiry gates, precipitation draws, section
admission, captured block/fluid order, callback RNG consumption, no-catch-up zero speed, unbounded
legal speed planning, signed-short counts, and cache phase gates.
