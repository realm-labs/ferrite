# G03-P3-B3 Production Path Replay and Faults

## Outcome

`ProductionTickReplayEvidence` now provides one canonical digest over formal ingress command
metadata, composite Region replay identities, continuity hashes/counts, and committed semantic
projections. Capture validates tick agreement, continuity receipt agreement, projection counts,
and the formal projection decoder before producing evidence.

Focused replay starts two independent formal Region routes, admits the same two session joins in
opposite arrival order, executes the composite tick, captures continuity, and routes the targeted
post-commit projection. Both routes produce identical evidence and nonzero ingress, projection,
and end-to-end digests.

The fault proof constrains composite projection capacity below the admitted player-join fanout.
Backpressure is detected before service mutation and composite commit, no commit receipt is
published, and the executor rejects retry as poisoned. The P3-B2 projection suite separately proves
malformed/unknown semantic projections and per-session overflow fail closed without partial queue
admission.

## Verification

- `cargo test -p ferrite-server-runtime --all-features --test production_path_replay --test
  composite_projection --test composite_gateway`: passed; deterministic opposite-order replay,
  ingress/continuity/projection digest capture, pre-commit backpressure, poison, malformed decode,
  and bounded delivery pass.
- `cargo fmt --all -- --check`: passed.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`: passed.
- `cargo test --workspace --all-features`: passed.
- `cargo ferrite production verify`: passed.
- `cargo ferrite source verify`: passed.
- `git diff --check`: passed.
