# World-Service Conformance (Historical Goal 01 Phase 8)

> **Historical status:** this suite closed the former player-visible-equivalence contract, which
> has been superseded by the vanilla-exactness contract. Its results remain valid replay and
> source-coverage evidence, but they no longer complete Goal 01 world generation or Goal 04.
> Its `RegionFileStore` cases are also local-adapter evidence; they do not prove the Goal 07
> location-independent distributed storage contract.

Phase 8 conformance combines the source-specific `ferrite-world` slice suite with executable
whole-system checks in `ferrite-testkit::world_service`. Slice tests retain exact Java control flow,
random-call order, numerical boundaries, and locked data decoding. The aggregate suite tests the
Ferrite-owned scheduling, persistence, and lifecycle contracts. Same-seed vanilla terrain identity
requires the separate official-server differential suite.

## Generation and catalog coverage

The conformance fixture requires the generated, schema-validated 26.2 content bundle. It resolves
all 14 runtime worldgen record kinds and their exact 963 records, covering WGEN-001 and WGEN-003
data families. The ordinary `ferrite-world` suite remains the behavioral denominator: 399 tests
exercise the 28 audited world slices, including features, carvers, density functions, biome and
noise records, structures, jigsaw, dimensions, portals, and borders.

Ferrite stage publication is tested over all 12 `ChunkStatus` values. Sixty-four fixed-seed cases
generate four chunks in forward and reverse dispatch order and compare canonical durable records;
the locked golden digest is
`19140a5608d4549ca22a1895d8f83ecfa96cfd216eae22de83089e7829e33fd1` for the current `FWC3`
structure-state format.
The generator used by this architectural harness produces deterministic project-owned mutations
through the production world-service status/fencing path. Source-family output behavior remains
owned by the per-slice tests rather than being collapsed into a synthetic claim of vanilla terrain
parity.

## Boundaries and durable recovery

Eight chunks around negative and positive Region edges validate Euclidean ownership, foreign-owner
rejection, full status/activity progression, and byte-exact recovery. Sixteen independent
save/load cases commit through `RegionFileStore`, select the durable point, restore under a newer
activation generation, and compare lifecycle plus encoded chunk state.

Six fault vectors cover content-manifest mismatch, stale activation generation, wrong Region,
repairable torn journal tail, non-recoverable data corruption, and mismatched commit receipts.
Together with the B1 integration tests, they keep failure atomic and ensure no unload teardown can
occur before the matching durable receipt.

## Root surfaces and joins

Three root surfaces close in this phase:

- `ContentDispatch`: exact catalog inventory, deterministic dispatch, manifest fencing, and
  in-flight-save refusal;
- `PersistenceReload`: chunk, simulation scheduler, and player-service continuity survive one Region
  recovery point;
- `WorldLifecycle`: 64 bootstrap/shutdown cases retain Overworld-first construction, readiness
  waits, ordered shutdown, and per-level failure continuation. Its event golden digest is
  `22081fb59b8f0695261ccb2d0acccbf663f6226ecbf49068fe14e3fa31fae7be`.

The 12 Phase 8 join owners have separate tests under `apps/behavior-runner/tests/joins`. They cover
every required pairing among ContentDispatch, PersistenceReload, WorldLifecycle, NetworkIngress,
PlayerLifecycle, and TickScheduler. Tests use the production semantic command, player continuity,
scheduler continuity, world-service recovery, and lifecycle APIs, so the join evidence checks actual
boundary representations rather than duplicating them in the harness.

This development-document filename and the historical Goal 01 phase terminology remain stable for
existing ledger links. Active conformance modules and test targets use responsibility names.
