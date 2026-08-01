# G03-P2-B3 Entity, World, Transfer, and Continuity Composition

## Outcome

`CompositeProductionRegionRuntime` now owns all four authoritative Region service states:
simulation, player, entity, and world. Typed commands execute entity insertion/observation/mutation,
chunk demand, revision-fenced world mutation, cross-Region mechanic transactions, and the full
entity-transfer prepare/accept/commit protocol in their fixed production stages.

Simulation boundary transactions mutate the world-owned voxel state, drain every deferred mechanic
effect as a typed result, and publish protocol-neutral authoritative block updates only after
commit. Entity projections follow the same private pre-commit queue and stable observer order as
player projections. Projection capacity is preflighted before applicable authority mutation.

## Transfer and continuity boundary

Entity transfer remains two-phase across independent Region ticks. The source prepares durable
outbound-pending state and emits a typed transfer; the target accepts it once and persists an
idempotence receipt; the source removes its pending entity only after the receipt is routed back.
Focused tests execute all three commands across two composite Region instances.

The continuity stage joins simulation runtime and scheduled queues, player state, entity state,
transfer receipts, world chunks, and auxiliary records into one current-generation candidate.
Commit retains the exact records, canonical hash, count, and tick. A consumer must take this
`CommittedCompositeContinuity` before the next tick begins, preventing silent overwrite or an
unbounded persistence handoff queue.

Replay encoding was split from authority execution into a dedicated codec module. Every typed
command includes its complete bounded semantic identity, including boundary entry counts and
transfer endpoints, generations, role, kind, and state.

## Verification

- `cargo test -p ferrite-server-runtime --test composite_entity_world --all-features`: passed;
  three voxel reconciliation, entity projection/four-service continuity, and two-Region transfer
  tests.
- composite coordinator and simulation/player focused suites: passed; ten tests.
- `cargo fmt --all -- --check`: passed.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`: passed.
- `cargo test --workspace --all-features`: passed.
- `cargo ferrite source verify`: passed; authority and codec files remain below 1,200 lines.
- `git diff --check`: passed.
