# G03-P2-B2 Simulation and Player Composition

## Outcome

The composite runtime now owns concrete simulation and player-service state under one Region key,
activation generation, tick, and commit boundary. Typed commands route deterministically to:

- player admission with persistent inventory/progression state;
- player/item mutation with Region, generation, session, revision, and sequence fences;
- menu open and close lifecycle;
- scheduled block and fluid work in the bounded simulation queues.

Every typed command also contributes a complete canonical metadata encoding to the coordinator
replay identity. Commands execute in owner and sequence order regardless of admission order.

## Joins and boundaries

The player stage preflights the composite projection budget before mutation, drains the concrete
player-service projections after each command, and keeps them private until composite commit. The
simulation stage retains scheduled work in simulation continuity. The continuity stage captures
simulation and player records together and rejects legacy writes. Commit advances both coordinator
and simulation clocks; only the following projection stage exposes the committed prefix.

The tick report contains the authoritative commit receipt, semantic outcomes, all nine stage
events, and the bounded projection prefix. Service execution faults poison the composite instance
so partial mutation cannot be retried as a clean tick. Entity, world voxel, boundary-transaction,
transfer, and durable-save joins remain assigned to `G03-P2-B3`.

## Verification

- `cargo test -p ferrite-server-runtime --test composite_simulation_player --all-features`:
  passed; four cross-service commit, player mutation, replay-order, and pre-mutation backpressure
  tests.
- `cargo test -p ferrite-server-runtime --test composite_runtime --all-features`: passed.
- `cargo fmt --all -- --check`: passed.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`: passed.
- `cargo test --workspace --all-features`: passed.
- `cargo ferrite source verify`: passed.
- `git diff --check`: passed.
