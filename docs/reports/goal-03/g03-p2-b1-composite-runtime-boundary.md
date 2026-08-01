# G03-P2-B1 Composite Runtime Boundary

## Outcome

The server runtime now has one responsibility-owned production Region coordinator with:

- an exact nine-stage service order from ingress through post-commit projection;
- typed command, projection, event, owner, commit-receipt, and capacity identities;
- separate bounded command, event, projection, continuity-record, payload-byte, and tick-horizon
  budgets;
- canonical command and projection ordering independent of admission order;
- current-generation-only continuity preparation;
- an explicit authoritative commit that publishes projections only after the committed tick
  advances; and
- a canonical replay identity spanning Region identity, generation, tick, command order, stage
  order, continuity, and projected effects.

This batch establishes coordination state and invariants only. Concrete simulation/player service
execution joins are assigned to `G03-P2-B2`; entity/world/transfer/continuity joins are assigned to
`G03-P2-B3`; the formal gateway replacement remains `G03-P2-B4`.

## Failure behavior

Out-of-order stages, overlapping ticks or stages, stale and too-future commands, duplicate command
identities, zero sequences, oversized payloads, legacy continuity writes, missing continuity,
post-commit projection production, and every capacity overflow are explicit errors. Event and
continuity backpressure leave the active stage retryable. Projection records remain invisible until
the commit stage succeeds.

## Verification

- `cargo test -p ferrite-server-runtime --test composite_runtime --all-features`: passed; six
  deterministic order, capacity, replay, continuity, commit, and projection-boundary tests.
- `cargo fmt --all -- --check`: passed.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`: passed.
- `cargo test --workspace --all-features`: passed.
- `cargo ferrite source verify`: passed.
- `git diff --check`: passed.
