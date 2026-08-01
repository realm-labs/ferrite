# G01-P10-B5 — Full Goal 01 acceptance

Ferrite's clean-checkout Goal 01 acceptance gate passed at revision
`0f706fbc332385798b596010ebce18d3606dad3d` on macOS/AArch64. The gate began and ended with an
empty Git worktree and retained implementation-manifest SHA-256
`6a516ae87b7a1504f490a2ec31f0a2c085ed28d53ab0b5fffd406a6b25e2daf3`.

## Deterministic and topology evidence

| Gate | Result |
|---|---|
| Portable canonical vector | 167 bytes, BLAKE3 `11d18ab3881d50117cab7211fd9bd41355a4b7009843a908520e3ba6e4b4d1ba` |
| Playable topology equivalence | 7 ticks, 16 packets, 3 processes, state `1e7c50dbf4463c858fcd779f4db59a08418e54cab7ae0e502821bba95ad0a858` |
| Canonical replay equivalence | 1 frame, 2,586 bytes, 3 processes, log `a000f4dc4182c89ed2410827f4e971a30dd3a00eabffae0d61150b83b71ab7cd` |
| Long topology equivalence | 10,000 ticks, 12 Regions, 3 nodes, digest `02ae8ad8bb897c569339b725bc3f44ed8ea49db653a25adf8d244ca68bf27685` |
| Multi-node fault campaign | 64 ticks, 12 Regions, 3 nodes, 8 fault classes, digest `f4b11710e88c6d7aabed45a9fae23b0c9418904177c83424e537a9b7fe7b9acd` |

The same portable vector also passed the `ubuntu-latest`, `macos-latest`, and `windows-latest`
jobs in [CI run 30685537732](https://github.com/realm-labs/ferrite/actions/runs/30685537732).

## Client and coverage boundary

The unattended client boundary verified the exact 39,193,383-byte Minecraft Java 26.2 client JAR
with SHA-1 `2dc72797acbc1b63fc16a11c4ac393605f453754`, reconstructed the complete registry/tag fixture, and
reran independent C0-C3 session semantics. It does not claim a new graphical-client observation;
the committed operator-assisted observation remains the
[C2 acceptance report](g01-p4-b5-c2-acceptance-and-adversity.md).

The complete reference and implementation pass verified:

- 327 source-specified gameplay slices and the source-known portions of four inconclusive slices;
- the same four unresolved observations as explicit `DeferredExperiment` records;
- 9,078 catalog IDs with zero unclassified, ambiguous, or unreviewed entries;
- 256 packets in 58 families: 44 required C0-C3 families and 14 optional C4 gates;
- ten behavior surfaces and all 36 unordered cross-system joins;
- 65 reachable parent rules, 352 reachable leaf rules, and all terminal implementation
  dispositions.

## Repository acceptance

The single command below passed from the clean checkout:

```text
cargo ferrite acceptance verify
```

It exercised the portable vector, local/in-process/three-process playable and replay equivalence,
the locked client fixture, architecture and content audits, deployment contracts, topology and fault
verification, capacity-report verification, behavior scenarios, protocol conformance and loopback
smokes, formatting, strict all-target/all-feature Clippy, all-feature workspace tests, offline
reference verification, `git diff --check`, and final worktree cleanliness.
