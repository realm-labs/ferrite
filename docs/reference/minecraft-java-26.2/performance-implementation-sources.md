# Minecraft Server Performance Implementation References

This register tracks public implementation sources that may inform Ferrite performance work. It is
non-normative: the locked official Minecraft 26.2 behavior/protocol reference remains the fidelity
authority, and Ferrite's [performance engineering contract](../../development/performance-engineering.md)
owns measurement and acceptance.

No third-party benchmark number is adopted without reproducing the same version, seed, data packs,
coordinates, cache state, hardware allocation, and client workload. Source or algorithm reuse also
requires an explicit license/provenance review; a useful idea is not permission to copy code.

## Curated projects

| Project | Public design signal | Ferrite study questions | Boundary |
|---|---|---|---|
| [C2ME](https://github.com/RelativityMC/C2ME-fabric) and its [releases](https://github.com/RelativityMC/C2ME-fabric/releases) | Parallelizes chunk generation, loading, and I/O; its release history also documents scheduler scalability, serialization-copy, density-function, and chunk-send work. | Model the chunk-status dependency DAG, task affinity, neighbor readiness, I/O overlap, cancellation, worker scaling, and copy/allocation costs. | The current source/release line includes Minecraft 26.2, so it is a candidate same-version comparator after configuration and workload equivalence are frozen. Most code is MIT; the separately licensed OpenCL component is all-rights-reserved. Every inspected path still requires provenance review. |
| [Noisium](https://github.com/Steveplays28/noisium) | Optimizes noise population and related worldgen paths; documented work includes direct palette population instead of repeated general block-state mutation. | Measure generic mutation overhead; consider generation-only section/palette builders, pre-sizing, immutable candidate construction, and required side-effect reconciliation. | Supported-version scope differs. Claimed parity must be independently checked against Ferrite's official 26.2 semantic oracle. LGPL-3.0 code must not be copied casually. |
| [Folia](https://github.com/PaperMC/Folia) and its [region logic](https://docs.papermc.io/folia/reference/region-logic/) | Independent nearby chunk regions own separate tick loops scheduled on a thread pool, with explicit split/merge ownership. | Compare Region locality, pool isolation, fairness, hot-region admission, and generation interference with Ferrite's ownership model. | Folia's README recommends pre-generation and notes that adding many generation workers alone can remain too slow at large player counts; Region parallelism is not proof of fast online generation. GPL-3.0 project. |
| [Paper](https://github.com/PaperMC/Paper) and [Moonrise](https://github.com/Tuinity/Moonrise) | Moonrise documents optimization surfaces for chunk ticking/loading/generation/saving, entity tracking/physics, block/entity lookup, responsiveness, and worker-count trade-offs; the current branch includes Minecraft 26.2. | Inspect bounded admission, priority propagation, load/generate/send separation, cancellation, chunk lifecycle contention, lookup locality, and low-core behavior. | Moonrise is a candidate same-version comparator after exact revision/configuration/workload freeze. Its goal of preserving vanilla behavior is a project claim, not Ferrite oracle evidence. GPL-3.0 project. |

## Review protocol

For each adopted technique, its batch report records:

- exact upstream URL, revision/tag, license, inspected files or documentation, and independent design
  conclusion;
- the measured Ferrite bottleneck that justifies the change;
- a scalar or existing correctness path retained as an oracle when practical;
- before/after raw reports under the same frozen workload;
- normalized official-server semantic differential results;
- failure, cancellation, overload, deterministic replay, and Region-boundary results.

Reject a technique when its gain depends on observable non-vanilla behavior, unbounded work,
pre-generation presented as online performance, skipped durability, or a license/provenance boundary
that has not been resolved.
