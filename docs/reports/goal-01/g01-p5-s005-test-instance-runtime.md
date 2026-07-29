# G01-P5-S005 — Test Instance Runtime

## Result

Complete. The one `SourceSpecified` block slice primarily owned by `SIM-002`,
`BLK-TEST-INSTANCE-RUNTIME-001`, now maps to production code and committed behavioral tests for
the normative `BLK-TEST-INSTANCE-001` leaf.

## Evidence

Production owners:

- `ferrite-gameplay::block::test_instance::data` — complete record/component/marker state,
  status/action fallbacks, persistence, update payload, and setter publication;
- `ferrite-gameplay::block::test_instance::geometry` — effective rotations, raw-size geometry,
  permanent chunk forcing, boundary shells, clearing, entity discards, and placement settings;
- `ferrite-gameplay::block::test_instance::operations` — permission/entity admission, all seven
  actions, duplicate updates, save/export quirks, double-placement replacement, and results;
- `ferrite-gameplay::block::test_instance::client` — local editor, UI-only bounds, response races,
  beam/bounds/marker projection, and render admission.

Committed test owner:

- `crates/ferrite-gameplay/tests/slices/blocks/sim_002.rs`.

Validated commands:

```text
cargo test -p ferrite-gameplay --test slices
cargo clippy -p ferrite-gameplay --all-targets --all-features -- -D warnings
cargo run -q -p mc-reference --bin mc-ref -- implementation-manifest verify
cargo ferrite content verify
cargo ferrite task check
git diff --check
```

Focused result before the universal gate:

```text
45 passed; 0 failed
1 SourceSpecified slice verified
```

## Ownership notes

This batch fixes the block-owned deterministic state and ordered semantic transaction without
embedding protocol packet structs, a second world, or a second entity authority. Exact wire bytes
remain with the generated operator-block families; template processors/test bodies, general
collision/entity/command systems, and visible packet lowering retain their generated owners. The
later Phase 5 integration batch binds this runtime to Region state, ECS, queues, persistence,
registry snapshots, and projection.
