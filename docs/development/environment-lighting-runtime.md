# Environment Lighting Runtime

`G01-P5-S010` implements the source-specified portion of the
`SourceInconclusive` `ENV-LIGHTING-001` slice. The only unresolved observation remains a universal
mutation-to-first-render latency bound under arbitrary executor, network and renderer load; it is
retained as `EXP-ENV-004` and is not claimed as vanilla behavior.

## Owned semantics

`ferrite-gameplay::environment::lighting` owns the protocol-neutral lighting decisions:

- independent nibble-valued Block and Sky channels;
- Block-before-Sky checks and complete decrease-before-increase drains;
- emission, attenuation, dampening and joined-face occlusion gates;
- source removal/recovery with overlapping emitters;
- Sky column thresholds, direct level-15 sources and empty-section lookup/bridging;
- section reference/layer creation, deferred removal, copy-on-write and visible-map publication;
- 1,000-record server task batches and complete selected-engine drains;
- raw brightness, synchronized Sky-light darkening and all audited emitter formulas;
- packet admission/mask clearing and bounded FIFO client imports before renderer update.

Region state and its light storage remain authoritative. The module emits deterministic decisions
and ordering contracts; adapters own chunk/state/shape reads, section arrays, prioritized tasks,
packet delivery and renderer dirtiness.

## Publication boundary

The final server authority is the visible-map swap plus dirty affected-section publication. A
ticking visible holder with tracking players sends nonempty Sky/Block masks and clears them. Client
receipt appends an import task; each client update imports Sky before Block, marks sections and
neighbors dirty, enables the chunk, drains lighting, then permits renderer update.

The client budget is all queued tasks at 1,000 or more, otherwise
`max(10,floor(queueSize/10))`. This is a poll budget, so a queue smaller than ten simply empties.

## Deferred latency policy

The production policy contains no universal tick or frame deadline and explicitly does not claim a
vanilla bound. A replacement requires a committed, load-profile-scoped `EXP-ENV-004` observation.
Server gameplay may observe a published map before a client imports or redraws it.

## Verification

The committed test owner is
`crates/ferrite-gameplay/tests/slices/environment/env_003.rs`. Its 16 tests cover propagation,
source overlap, 999/1,000/1,001 task batches, storage/publication, Sky queries and thresholds,
brightness/darkening, packet gates, client budgets/order, every state-varying emitter family,
representative constant emitters at all audited levels, the 109-ID closure count and the explicit
deferred policy.
