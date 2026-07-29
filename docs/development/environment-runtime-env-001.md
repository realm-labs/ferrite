# ENV-001 Fluid and Geyser Runtime

`G01-P5-S009` implements the two `SourceSpecified` environment slices primarily owned by
`ENV-001`. The protocol-neutral kernels live under `ferrite-gameplay::environment`; Region
integration supplies ordered local observations and commits the resulting writes, schedules,
events, effects and entity changes.

## Module ownership

| Module | Audited responsibility |
|---|---|
| `environment::fluid` | Five fluid identities, legacy levels/heights, water/fast-lava constants, local recomputation, source conversion, downward/tied side spread, replacement/container gates, tick writes, lava mixing/fire, waterlogging and evaporation |
| `environment::geyser` | Five Potent Sulfur states, support derivation, bounded source-column scan, periodic countdown, gas effects, client plume/display cadence, launch admission, transient epoch, persisted countdown and loot |

World collision, tag and block queries are explicit observations. The fluid kernel fixes the
`NORTH,EAST,SOUTH,WEST` candidate order and `NORTH,SOUTH,WEST,EAST` tied commit order without
creating a second world representation. Likewise, the geyser kernel accepts the bounded
source-column and entity-query results in their authoritative Region order.

## Determinism and transaction boundaries

- Due block ticks remain ahead of due fluid ticks, with the source cap of 65,536 per queue.
  Source fluids skip recomputation; nonsources write first and spread the resulting state even
  when the low-level write Boolean is ignored.
- Fluid propagation is deterministic after ordered collision/holdability observations. The only
  propagation draw is Lava's conditional `nextInt(4)` slowdown; fire and evaporation expose their
  exact branch-dependent draw counts.
- Lava mixing scans `UP,NORTH,SOUTH,WEST,EAST`, writes the first Obsidian, Cobblestone or Basalt
  result, emits event 1501 and aborts scheduling. Downward Lava-to-Water always fizzes, while Stone
  requires the target block itself to be a liquid block.
- Potent Sulfur countdown receives a caller-created Minecraft positional Xoroshiro stream derived
  from `worldSeed XOR -904011478` and block position. The kernel owns draw/discard order and
  countdown arithmetic; Region runtime owns stream construction.
- Failed state writes do not roll back an already-mutated countdown, fluid callback or earlier
  fire placement. Adapters must preserve that non-atomic ordering.

## State and persistence

Dense block/fluid/entity/particle/sound IDs are Minecraft Java 26.2 projection identities. Durable
world state continues to use stable resource identities. Potent Sulfur persists any signed
`waitingCountdown`; its `eruptionTick` is client transient, initializes on first level attachment
and resets on any admitted block event.

Waterlogged containers retain their block state and schedule Water with delay 5. Intrinsically
water-filled aquatic blocks reject generic liquid placement. Water evaporation occurs only after
bucket target/recursive admission and suppresses the ordinary write, sound and `FLUID_PLACE`
event.

## Verification

The committed test owner is
`crates/ferrite-gameplay/tests/slices/environment/env_001.rs`. Its 29 tests cover fluid identity
and level closure, conversion/replacement/spread boundaries, tied order, write plans, all mixing
products, fire branches, evaporation draw count, Potent Sulfur state derivation, 0–5-cell source
columns, countdown endpoints, gas visibility, launch rejection order, plume epochs, display RNG,
persistence and loot.
