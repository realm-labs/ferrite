# World-generation equivalence boundary

Ferrite verifies the complete source-derivable Minecraft 26.2 world-generation control surface but
does not claim block-for-block same-seed identity with the official server. Those statements are
separate by design: algorithms, state transitions, data records, gates, ordering, numerical
boundaries, and random-call order can be established from locked source and data, while a
player-visible statistical equivalence population and its acceptance thresholds require an
external experimental choice.

## Verified contract

All 27 source-specified world slices are `Verified`. The implementation side of
`WGEN-PIPELINE-EQUIVALENCE-001` is also `Verified`: Ferrite uses a version-locked content manifest,
deterministic project-owned seed mapping, all 12 ordered chunk statuses, generation/revision/content
fences, canonical Region records, and dispatch-order-invariant recovery. The Phase 8 conformance
golden proves that an identical Ferrite input repeats exactly, while a distinct Ferrite seed is
allowed to produce different state.

This is a non-identical-seed architecture contract. No conversion from a vanilla seed to a Ferrite
seed is asserted, and no matching block layout, structure location, resource distribution, or
locate result is inferred merely because both systems implement the same source-derived branch
semantics.

## Deferred observation

The implementation manifest retains one `DeferredExperiment` observation with three named plans:

- `EXP-WGEN-001`: eight dedicated runs over the fixed generation/status and broad behavior
  population, including calibration versus held-out partitions;
- `EXP-WGEN-005`: 4,096 dedicated placed-feature/modifier/catalog runs;
- `EXP-WGEN-006`: 4,096 dedicated flat-generator/preset runs.

Their definitions remain `planned`. In particular, no calibration population, multiple-test
correction, per-family threshold, diagnostic tolerance, or allowed locate/resource divergence has
been selected and committed. Source-specific slice tests are evidence for control flow, not a
substitute for those measurements.

The deferral may be replaced only by committed, named statistical thresholds and the corresponding
experiment evidence. Any future replacement must keep calibration separate from the untouched
held-out population and must not retroactively weaken the already verified source-specified
branches.
