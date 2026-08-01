# G01-P10-B3 Multi-Node Fault Injection

## Result

The three-process Region topology now has a deterministic 64-tick fault campaign covering all
Phase 10 distributed failure classes. After every injected fault, the process-isolated run
converges with uninterrupted in-process execution at digest
`f4b11710e88c6d7aabed45a9fae23b0c9418904177c83424e537a9b7fe7b9acd`.

## Fault matrix

| Fault | Injection | Verified outcome |
|---|---|---|
| Message duplication | Repeat three exact Lattice envelopes | Duplicate admission is idempotent and capacity is unchanged |
| Message reordering | Reverse the full tick delivery order | Canonical commit state matches uninterrupted execution |
| Network partition / message loss | Withhold one boundary message from a node | Global preflight rejects before any worker commits; delivery retry completes the same tick |
| Control-plane outage | Disable layout reconfiguration for four ticks | Reconfiguration fails closed while the established data plane continues under existing claims |
| Owner crash | Kill one topology-worker process after durable snapshot capture | Regions recover on a survivor at a newer generation; an old-generation envelope is rejected |
| Handoff and drain | Reconcile `BeginHandoff` with target/move identity, then request drain | Authority fences new admission and emits the Region drain action before durable reassignment |
| Restart | Stop and recreate a state-owning worker from its partition snapshot | The replacement resumes at the next logical tick without state drift |
| Rolling upgrade | Replace workers one at a time from runtime version 1 to 2 | Mixed-version workers continue committing; all replacements preserve the final digest |

## Barrier and recovery hardening

The multi-process coordinator previously sent commit requests directly after delivery. It now sends
a read-only preflight request to every worker and issues no commit until all partitions confirm the
complete tick. This makes message loss and network isolation globally atomic instead of permitting a
prefix of worker processes to advance.

Snapshot repartitioning is now a shared Region-runtime operation. Every moved Region is encoded and
decoded as a bounded `RegionRecoveryPoint`, prepared through `RegionHandoffState`, digest-verified,
and installed only under the strictly newer layout generation. Process restore replaces a worker's
partition atomically, clearing transient inbox and authority state rather than copying live actor
memory.

Graceful movement follows the Lattice state machine rather than forcing a drain from `Running`:
the source first reconciles an explicit `BeginHandoff` observation containing the target node and
move ID. Only then may the authority fence admission and request Region drain.

## Commands

```text
cargo run -q -p ferrite-cluster -- verify-faults
cargo ferrite topology verify
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo ferrite task check
git diff --check
```

`cargo ferrite topology verify` and the repository task gate now run both the locked 10,000-tick
local/in-process/three-process equivalence proof and this eight-category multi-node fault campaign.
