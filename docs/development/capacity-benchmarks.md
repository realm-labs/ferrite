# Capacity benchmark profiles

Ferrite's capacity reports are measurements of the synthetic Region topology harness. They are
useful for regression comparisons and architecture sizing, but they are not production player,
chunk, or hardware capacity promises. A result applies only to the recorded revision, runner,
build profile, workload, topology, and sample variance.

The versioned inputs live in [`benchmarks/capacity-profiles.toml`](../../benchmarks/capacity-profiles.toml):

| Profile | Regions | Nodes | Worlds | Timed ticks | Samples | Hotspot share |
|---|---:|---:|---:|---:|---:|---:|
| `development-smoke` | 24 | 3 | 3 | 32 | 5 | 75% |
| `reference-multiverse` | 192 | 6 | 24 | 64 | 7 | 75% |
| `scale-envelope` | 768 | 12 | 96 | 32 | 5 | 80% |

Every profile runs in isolated sample processes. The report records:

- balanced and hotspot wall time per topology tick and per Region;
- balanced and hotspot remote-inbox high-water marks against the declared mailbox bound;
- process resident-set growth after the balanced topology warmup when the host provides `ps`;
- canonical durable recovery-point bytes and encoding time;
- per-tick message count, encoded bytes, cross-node messages, and distinct node pairs;
- hotspot slowdown and the exact minimum Region moves needed to reach at most one Region of node
  skew, including moved durable bytes and repartition time;
- the final canonical digest, runner metadata, revision, dirty state, and min/median/p95/max/mean,
  population standard deviation, and coefficient of variation for timed metrics.

Validate the committed workload definitions without collecting timings:

```text
cargo ferrite capacity verify
```

Record all profiles in the isolated benchmark cache namespace with release optimizations:

```text
cargo ferrite cargo bench run --release -p ferrite-cluster -- \
  capacity benchmark --output target/capacity-report.json
```

Use `--profile <NAME>` to record one profile. The command never turns a result into a capacity
claim; reviewers must retain the report's workload and claim boundary when comparing runs.
