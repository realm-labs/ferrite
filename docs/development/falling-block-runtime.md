# Falling Block Runtime

`G01-P5-S004` implements the `BLK-FALLING-001` slice and its normative `BLK-FALL-001` leaf in
`ferrite-gameplay::block::falling`. The module is protocol-neutral: it owns the source-specified
state transitions and ordered semantic effects while the Phase 5 integration batch binds those
effects to Region voxel storage, the Region ECS, persistence, boundary delivery, and client
projection.

## Generic transaction

The runtime recognizes exactly the 26 audited block IDs and assigns their `1`, `2`, or `5` tick
delay. Its start plan locks the center coordinate, minimum-height/free-below gate, waterlogged
clearing, start-position record, flags-`3` origin-fluid replacement, and admission ordering.
Origin removal deliberately precedes an admission request whose result cannot restore the block;
ordinary construction carries no implicit block-entity data.

The entity kernel fixes the source defaults and the active-tick sequence: discard an air carried
state, increment the signed time, apply `-0.04` gravity, move and process block/portal effects,
resolve server landing or timeout, then apply `0.98` drag. Landing first multiplies velocity by
`(0.7,-0.5,0.7)`. It then preserves the moving-piston pause and separates:

- cancel-and-broken-hook without an item;
- replacement, survival, and still-falling eligibility;
- destination-waterlogged copy and flags-`3` placement;
- successful tracking update, discard, `onLand`, then explicit serialized block-entity overlay;
- ineligible discard/drop;
- the failed eligible write that remains active only when its drop gate is closed.

Timeouts use strict `time>100` outside the vertical bounds and strict unconditional `time>600`.
Their item gate intentionally ignores `cancelDrop`. Unload records store-before-callback-removal,
reload uses the persistent UUID guard without advancing wall time, and an End transfer can allow
the removed original to finish its current cleanup branch once. Hurt records the hit but rejects
damage.

## Subtype kernels

The same module keeps subtype behavior beside the generic transaction without importing later
entity, loot, or presentation ownership:

- anvils load their damage-enabled default from the carried tag, configure `2`/`40` on start,
  filter valid victims, use `ceil(fallDistance-1)` and `min(floor(amount*i),maximum)`, expose a
  degradation draw only for positive damage, apply the strict degradation threshold, and select
  level events `1031`/`1029` unless silent;
- concrete powder admits only a source-fluid ray hit at strict squared speed `>1`, preserves the
  water-contact landing bypass, and uses the position-or-nonsturdy-neighbor solidification gate;
- suspicious sand and gravel reset before generic creation, set cancel, create no implicit carried
  block-entity data, and select event `2001` plus `BLOCK_DESTROY` from their broken hook;
- scaffolding computes vertical distance without increment, horizontal minimum plus one capped at
  `7`, exact `bottom`, and distinct newly-unsupported destroy versus already-unsupported fall;
- dragon egg candidates expose the air, nonair-below, build-height, and border gates, cap attempts
  at `1,000`, account for six draws per attempt, and preserve flags-`2` server movement versus 128
  client particles;
- ambient dust accounts for one unconditional `nextInt(16)` and emits only on zero.

## Validation

`crates/ferrite-gameplay/tests/slices/blocks/blk_006.rs` locks the 26-ID catalog, effect ordering,
all strict boundaries and failure branches, persistence behavior, subtype formulas, RNG
cardinality, flags, events, and particles. `G01-P5-B1` remains the sole owner of binding these
semantic decisions into the Region tick pipeline, so this slice does not create a second world or
entity authority.
