# G01-P10-B4 — Named capacity profiles and benchmark evidence

Ferrite now has three committed synthetic Region capacity profiles and a release-mode report that
covers tick cost, queue pressure, resident memory, durable storage, network fan-out, hotspot skew,
many-world layouts, and rebalance objectives. These measurements are regression and sizing evidence,
not a production player-capacity promise.

## Evidence boundary

- Runner revision: `c476a7b75fc04507dd4a8aacc36a737978ce7e82` from a clean worktree.
- Runner: Apple M1 Max, macOS/aarch64, 10 logical workers, Rust 1.97.1.
- Build: release optimized through the isolated `target/bench` namespace.
- Raw report: [capacity benchmark JSON](g01-p10-b4-capacity-benchmarks.json).
- Workloads: [capacity profile definitions](../../../benchmarks/capacity-profiles.toml).
- Timings include the synthetic topology's complete emit, Lattice-frame route/admission, preflight,
  commit, and digest-retained state transition. Each timing sample runs in a fresh process.

## Recorded profiles

| Profile | Topology | Balanced Region tick mean / p95 | Hotspot Region tick mean / CV | RSS delta mean | Durable bytes | Network bytes/tick | Queue balanced → hotspot | Rebalance mean / moves |
|---|---|---:|---:|---:|---:|---:|---:|---:|
| `development-smoke` | 24 Regions / 3 nodes / 3 worlds | 2,787 / 3,210 ns | 2,634 ns / 2.26% | 671,744 B | 4,608 | 3,480 | 8 → 18 of 24 | 46,950 ns / 10 |
| `reference-multiverse` | 192 / 6 / 24 | 2,931 / 3,093 ns | 2,987 ns / 2.07% | 1,528,393 B | 36,864 | 27,840 | 32 → 144 of 192 | 539,208 ns / 112 |
| `scale-envelope` | 768 / 12 / 96 | 3,289 / 3,476 ns | 3,357 ns / 2.34% | 4,250,010 B | 147,456 | 111,360 | 64 → 615 of 768 | 2,226,650 ns / 551 |

Every Region emitted one bounded message per tick. All messages in the three balanced layouts
crossed a node boundary: 24, 192, and 768 messages across 6, 12, and 24 distinct directed node
pairs respectively. The hotspot profiles exercised 75%, 75%, and 80.07% mailbox utilization.

All rebalance runs moved exactly the over-target Region set, advanced generations for moved
authority, repartitioned durable recovery points, preserved the canonical committed digest, and
finished at zero Region skew (within the declared at-most-one objective). Moved durable footprints
were 1,920, 21,504, and 105,792 bytes.

The balanced Region-tick coefficients of variation are 7.78%, 2.36%, and 3.46%; hotspot values are
2.26%, 2.07%, and 2.34%. The raw report retains all min/median/p95/max and standard-deviation values.
Ratios between balanced and hotspot samples remain descriptive observations, not optimization or
capacity conclusions.

## Verification

```text
cargo ferrite capacity verify
cargo ferrite cargo bench run --release -p ferrite-cluster -- \
  capacity benchmark --output docs/reports/goal-01/g01-p10-b4-capacity-benchmarks.json
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test -p ferrite-region-runtime -p ferrite-cluster -p ferrite-tooling --all-features
```

The repository verification command also validates that the committed raw report uses the exact
named workloads, a clean release runner, ordered finite variance metrics, bounded queues, consistent
traffic totals, a met rebalance objective, canonical digests, and the explicit claim boundary.
