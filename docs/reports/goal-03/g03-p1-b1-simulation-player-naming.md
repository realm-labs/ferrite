# G03-P1-B1 Simulation and Player-Service Naming

## Outcome

Active Goal 01 planning-phase ownership names were removed from the simulation and player-service
runtime architecture:

| Previous active name | Responsibility-owned name |
|---|---|
| `ferrite_server_runtime::phase5` | `ferrite_server_runtime::simulation` |
| `Phase5RegionRuntime` | `SimulationRegionRuntime` |
| `Phase5RuntimeConfig` / `Phase5RuntimeError` | `SimulationRuntimeConfig` / `SimulationRuntimeError` |
| `Phase5Continuity` | `SimulationContinuity` |
| `Phase5QueueBudget` / `Phase5QueueKind` | `SimulationQueueBudget` / `SimulationQueueKind` |
| `Phase5ProjectionBuffer` / `Phase5ProjectionError` | `SimulationProjectionBuffer` / `SimulationProjectionError` |
| `ferrite_server_runtime::phase6` | `ferrite_server_runtime::player_service` |
| `Phase6RegionRuntime` / `Phase6RuntimeError` | `PlayerServiceRegionRuntime` / `PlayerServiceRuntimeError` |
| `ferrite_testkit::phase5` / `ferrite_testkit::phase6` | `ferrite_testkit::simulation` / `ferrite_testkit::player_service` |

The two active integration-test targets are now `simulation_region_integration` and
`player_service_region_integration`. Behavior-runner surface and join entry points, later world
conformance consumers, module documentation, error diagnostics, and deterministic fixture
identities were updated with the same responsibility vocabulary.

## Compatibility boundary

This batch does not rewrite persistence. The Goal 01 values below remain byte-for-byte stable and
are isolated behind `LEGACY_*_DOMAIN` constants:

- `ferrite:phase5/runtime_v1`;
- `ferrite:phase5/scheduled_block_v1`;
- `ferrite:phase5/scheduled_fluid_v1`;
- `ferrite:phase5/boundary_receipt_v1`;
- `ferrite:phase6/player_v1`.

The renamed integration suites assert these exact emitted domains. `G03-P1-B3` owns the versioned,
fail-closed migration to responsibility-owned persistence identities. Development-document
filenames referenced by completed Goal 01 ledgers are retained for link stability, but their active
module and test references now use responsibility names and explicitly label Goal 01 phase wording
as historical provenance.

## Verification

- `cargo check -p ferrite-server-runtime -p ferrite-testkit -p behavior-runner --all-targets --all-features`: passed.
- `cargo test -p ferrite-server-runtime --test simulation_region_integration --test player_service_region_integration`: passed; 13 tests.
- `cargo test -p behavior-runner --test surfaces --test joins`: passed; 46 tests.
- active Rust symbol/module audit for `Phase5`, `Phase6`, `::phase5`, and `::phase6`: passed; no matches.
- legacy lowercase `phase5` / `phase6` audit: only the five persisted compatibility domains and one
  later world-compatibility fixture remain.
- `cargo fmt --all -- --check`: passed.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`: passed.
- `cargo test --workspace --all-features`: passed.
- `cargo ferrite source verify`: passed; 1,250 handwritten Rust files checked against the
  1,200-line limit.
- `git diff --check`: passed.
