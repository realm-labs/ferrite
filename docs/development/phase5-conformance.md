# Tick-Scheduler Conformance (Historical Goal 01 Phase 5)

`G01-P5-B2` closes the TickScheduler behavior surface and its NetworkIngress join after all Phase 5
slice and Region-integration owners are complete. The executable harness lives in
`ferrite-testkit::simulation`; the implementation-manifest test owners under `apps/behavior-runner`
invoke it without introducing test orchestration into production crates.

This filename and the Goal 01 phase language below are retained as historical conformance
provenance. Active testkit ownership uses the `simulation` responsibility name.

## TickScheduler surface

The locked golden trace contains all 20 Region phases, the scheduled per-chunk/due-head merge,
level-31 activity and holder filtering, and the signed random-position stream. Its BLAKE3 digest is:

```text
eadb63bd70aec010d8d5854b817ec04172320a823e419093e744c08a579cc501
```

The property sweep runs 128 fixed seeds. Each seed creates 32 uniquely keyed scheduled entries
across four chunks with generated trigger, priority, and sub-tick values, drains them twice, and
requires identical complete output. It also compares 64 signed position samples from independently
restored streams. This tests reproducibility without imposing a false sort on the audited
fastutil-compatible activity history.

Four fail-closed vectors cover unregistered scheduling, inactive-chunk retention, frozen gameplay
retention, and bounded partial draining. Five interior-versus-boundary cases span negative, zero,
and positive Region coordinates. Each compares final voxel state, Java section projection, and due
scheduled identity after applying the same two-block mutation locally or through a
generation-fenced boundary transaction.

Four replay frames carry deterministic scheduler seeds through `ferrite-replay`. A matching target
converges on every Region/world hash; a one-byte semantic perturbation produces a first divergence.
This verifies that the explicit scheduler order is replay input rather than incidental container
order.

## TickScheduler × NetworkIngress

The join harness routes semantic player commands through
`ferrite-server-runtime::session::router::RegionCommandRouter` into the production
`LocalRegionRunner`. Its Region logic owns one due scheduled callback at the same block.

Two task-order cases lock the capture boundary:

- ingress admitted for tick one commits in `Ingress`, then the due `ScheduledBlocks` callback sees
  that committed state;
- ingress admitted only for tick two cannot affect the already committed tick-one callback.

The first journal is exactly `Ingress` then `ScheduledBlocks`; the second case's first journal
contains only `ScheduledBlocks`. A 64-value sweep requires the same result for every replacement
state. Exact duplicate, stale-tick, full-inbox, and excessive-future admission all reject without
an extra mutation. Callback-created zero-delay scheduled work is retained and executes on tick two,
never in the already collected tick-one batch.

This implements `JOIN-01`: decoded or normalized ingress has no authority until its semantic command
passes bounded server-task admission, commands do not interleave within callbacks, and only
committed handler/callback prefixes reach journals and projection owners.

## Deferred observations

`EXP-SIM-002` remains the sole source-inconclusive restored cross-chunk equal-head tie. This suite
does not invent a tie breaker. `EXP-SIM-003` remains a locked activity regression vector; the
implemented fastutil history, ticket thresholds, holder gates, and no-catch-up policy are tested
without relabeling the experiment as new source evidence.
