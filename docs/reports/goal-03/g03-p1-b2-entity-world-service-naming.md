# G03-P1-B2 Entity, World, and Service-Conformance Naming

## Outcome

Active Goal 01 planning-phase ownership names were removed from the remaining runtime and
service-test architecture:

| Previous active name | Responsibility-owned name |
|---|---|
| `ferrite_server_runtime::phase7` | `ferrite_server_runtime::entity_service` |
| `Phase7RegionRuntime` | `EntityServiceRegionRuntime` |
| `Phase7RuntimeLimits` / `Phase7RuntimeError` | `EntityServiceRuntimeLimits` / `EntityServiceRuntimeError` |
| `Phase7ContinuityError` | `EntityServiceContinuityError` |
| `ferrite_server_runtime::phase8` | `ferrite_server_runtime::world_service` |
| `Phase8RegionRuntime` | `WorldServiceRegionRuntime` |
| `Phase8RuntimeConfig` / `Phase8RuntimeError` | `WorldServiceRuntimeConfig` / `WorldServiceRuntimeError` |
| `Phase8ContinuityError` | `WorldServiceContinuityError` |
| `ferrite_testkit::phase7` / `phase8` / `phase9` | `entity_service` / `world_service` / `service_conformance` |
| `Phase9ProtocolAuditReport` / `Phase9JoinReport` / `Phase9Surface` | `ServiceProtocolAuditReport` / `ServiceJoinReport` / `ServiceSurface` |

The active server-runtime test targets are now `entity_service_region_integration` and
`world_service_durable_world`. Behavior-runner entry targets are now
`entity_service_conformance`, `world_service_conformance`, and
`service_integration_conformance`. All client, surface, join, experiment, testkit, and
`world-inspector` consumers use the responsibility-owned paths and diagnostics.

## Compatibility boundary

This batch does not rewrite persistence. The Goal 01 values below remain byte-for-byte stable and
are isolated behind `LEGACY_*_DOMAIN` constants:

- `ferrite:phase7/entity_v1`;
- `ferrite:phase7/applied_transfer_v1`;
- `ferrite:phase8/chunk_v1`;
- `ferrite:phase8/level_v1`.

The renamed integration suites assert those exact emitted domains. `world-inspector` likewise
labels its remaining Phase 8 identity as legacy rather than exposing an active phase-owned
diagnostic. `G03-P1-B3` owns their versioned migration and dual-generation inspection.

Development-document filenames referenced by completed Goal 01 ledgers remain link-stable. Their
active paths and test targets now use responsibility names, and their Goal 01 phase terminology is
explicitly labeled as historical provenance.

## Verification

- `cargo check -p ferrite-server-runtime -p ferrite-testkit -p behavior-runner -p world-inspector --all-targets --all-features`: passed.
- `cargo test -p ferrite-server-runtime --test entity_service_region_integration --test world_service_durable_world`: passed; 16 tests.
- focused behavior-runner entity/world/service, surface, join, client, and experiment targets:
  passed; 81 tests.
- `cargo test -p world-inspector`: passed.
- active Rust symbol/module audit for `Phase7`, `Phase8`, `Phase9`, `::phase7`, `::phase8`, and
  `::phase9`: passed; no matches.
- legacy lowercase `phase7` / `phase8` audit: only the four persisted compatibility domains remain.
- `cargo fmt --all -- --check`: passed.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`: passed.
- `cargo test --workspace --all-features`: passed.
- `cargo ferrite source verify`: passed; 1,250 handwritten Rust files checked against the
  1,200-line limit.
- `git diff --check`: passed.
