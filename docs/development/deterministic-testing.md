# Deterministic Testing

`ferrite-testkit` owns reusable deterministic test controls. Production crates must not depend on
it. Its current primitives are:

- `clock`: a checked monotonic fake clock with explicit advancement;
- `seed`: stable named seed derivation using the versioned `ferrite:test-seed:v1` domain;
- `snapshot`: bounded byte snapshots, BLAKE3 digests, and first-difference diagnostics;
- `malformed`: bounded authored corpora, strict-prefix generation, and rejection assertions;
- `scenario`: the schema-versioned TOML behavior DSL and target-neutral runner.

## Authored scenarios

A scenario has a stable resource ID, a root seed, and nondecreasing tick steps. An `apply` step sends
a bounded semantic action to a target. An `assert_snapshot` step compares the target's current
snapshot with authored bytes. Unknown TOML fields, unsupported schema versions, oversized inputs,
and backwards ticks fail validation.

Use the runner from the workspace root:

```text
cargo run -q -p behavior-runner -- validate tests/fixtures/scenarios/recording-smoke.toml
cargo run -q -p behavior-runner -- run tests/fixtures/scenarios/recording-smoke.toml
```

The current `behavior-runner` target is deliberately a recording harness used to prove the DSL,
validation, and execution path. It is not evidence that a Minecraft gameplay rule is implemented.
Later batches supply Region, protocol, persistence, and topology targets through the
`ScenarioTarget` boundary, and their scenarios must name the audited rule or protocol family they
verify.

## Repository gates

`cargo ferrite source verify` scans handwritten Rust outside build, generated, and vendor trees. It
enforces the 1,200-physical-line limit and rejects deep parent-relative paths, unreviewed public
re-exports, broad Clippy suppression, and compilation paths that bypass Clippy.

`cargo ferrite-check` performs the source and architecture checks, exercises the smoke scenario,
then runs format, Clippy, workspace tests, and locked offline reference verification. CI runs all
portable gates; the offline reference gate remains a local locked-artifact gate because official
client and server artifacts are intentionally not committed.
