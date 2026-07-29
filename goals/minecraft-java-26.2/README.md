# Goal 01 Implementation Manifest

`implementation.toml` is the generated progress and ownership map for the locked Minecraft Java
Edition 26.2 reference. The reference documents remain normative. The manifest does not copy
behavioral conclusions into production code and does not make a `Pending` record complete.

Regenerate it from the workspace root:

```text
cargo run -q -p mc-reference --bin mc-ref -- implementation-manifest render
```

The renderer reads the locked catalog, gameplay completion ledger, behavior surfaces, cross-system
joins, protocol completion ledger, and cached locked packet report. It emits records in a stable
order. A second render with unchanged inputs must leave the file byte-for-byte unchanged.

## Schema

The root fields bind the manifest to Goal 01, reference version 26.2, the immutable baseline path
and SHA-256, the exact generator command, and schema version. `totals` is a generated denominator
summary and is never an implementation-completion claim.

Every implementation record has:

- one concrete `id` or `implementation_batch`;
- one responsibility-scoped `implementation_owner`;
- one future `test_owner`;
- an `evidence` list, empty until committed verification exists;
- one implementation `disposition`;
- its exact reference identity or complete reference partition;
- an owning phase and dependency/closure boundary when the record is executable as a batch.

Generated batch fields have the following meaning:

| Field | Meaning |
|---|---|
| `id` | Concrete batch identity; generated IDs replace `Snn`, `Fnn`, and `Onn` placeholders. |
| `phase` | Goal phase in which the implementation work executes. |
| `responsibility` | One bounded implementation outcome. |
| `depends_on` | Completed predecessor batches required before this batch starts. |
| `closes_in` | Fixed integration or conformance batch that consumes this batch's evidence. |
| `implementation_owner` | Crate or application responsible for the production behavior. |
| `test_owner` | Planned test module responsible for behavioral evidence. |
| `disposition` | Current implementation state from the Goal 01 disposition vocabulary. |
| `evidence` | Committed test/report references required before `Verified`. |

The dependency edge is `depends_on -> id`; the downstream aggregation edge is
`id -> closes_in`. A generated batch must not depend on its own closure batch.

## Record families

`catalog_batch` partitions all 9,078 locked IDs by the 32 catalog kinds. Each row retains the
locked ID count and digest plus the complete sorted catalog-family inventory. Catalog batches run
after deterministic registries and close in the official-data import batch.

`gameplay_batch` assigns all 331 slices exactly once. Slices are grouped by implementation phase,
subsystem, and primary parent rule. Each row retains the full slice list and the union of its parent
and leaf rules. The four `SourceInconclusive` slices still have required source-known
implementation work in these batches.

`deferred_observation` contains only the four exact source-inconclusive observations. Its
`source_part_batch` points to the batch that still implements the source-known behavior.
`DeferredExperiment` must never be broadened to the whole slice.

`surface_owner` and `join_owner` map the 10 root behavior surfaces and 36 unordered cross-system
joins to fixed integration batches and dedicated behavior-runner tests.

`protocol_batch` assigns all 58 protocol families and their 256 packets exactly once. Required
C0-C3 families use concrete `F` batches. Optional C4 families use concrete `O` batches whose
implementation mode is `ConfigurationGate`; that mode requires disabled/refusal/degradation
behavior and does not imply that the optional external service is enabled.

## Dispositions

New generated implementation records start as `Pending`. Later batches may change a record only
to a disposition defined by Goal 01:

- `Pending`
- `InProgress`
- `Implemented`
- `Verified`
- `DeferredExperiment`
- `NotApplicable`
- `Blocked`

The renderer owns reference partitions, batch identities, dependency edges, and test-owner paths.
Phase 0 coverage tooling will own validation of manual progress/evidence fields so regeneration can
preserve implementation state without weakening the locked denominator.
