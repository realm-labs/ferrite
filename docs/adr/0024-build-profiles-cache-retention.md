# ADR-0024: Separate Routine and Full-Symbol Builds and Bound Cache Retention

## Status

Accepted

## Context

This workspace will contain many crates and large dependencies. Unbounded debug artifacts can fill
developer disks, while hidden or broad cleanup can destroy useful caches and the expensive locked
Minecraft reference extraction.

## Decision

The workspace root defines:

```toml
[profile.dev]
debug = "line-tables-only"

[profile.dev.package."*"]
debug = false

[profile.debugging]
inherits = "dev"
debug = true
```

Routine builds use `dev`. Human debugging explicitly selects `--profile debugging`. Incompatible
coverage, fuzz, benchmark, debugging, and CI tasks use isolated `CARGO_TARGET_DIR` namespaces keyed
by toolchain, target, profile, lockfile, and relevant flags.

Repository task entrypoints perform at most one rate-limited maintenance check per day. Inspection
and pruning are explicit tooling operations with:

- a workspace-scoped lock;
- resolved, canonical target paths and containment checks;
- dry-run output before deletion;
- active-build detection;
- itemized bytes, age, namespace, and reason;
- no shell glob or unresolved environment-variable deletion.

Inactive auxiliary namespaces become eligible after seven days. Routine `dev` artifacts are
eligible only when the workspace target footprint exceeds a configurable 40 GiB high-water mark and
the candidate has been inactive for fourteen days. Thresholds are policy defaults exposed by the
tool, not hidden constants.

Global Cargo registries/git caches and `target/mc-reference/26.2` are always protected. Unscoped
`cargo clean`, home-directory deletion, and cleanup outside resolved workspace targets are forbidden.

## Consequences

- Routine compilation consumes materially less debug-symbol space.
- Full symbols remain one explicit profile away.
- Auxiliary workloads do not collide with normal debug artifacts.
- Maintenance adds a small amount of repository tooling and metadata.

## Alternatives Considered

- Full symbols in all dev dependencies: rejected due to disk and link-time cost.
- Scheduled `cargo clean`: rejected because it is broad, opaque, and destroys reusable artifacts.
- Never prune: rejected because the expected workspace scale makes disk exhaustion predictable.

## Migration or Reversal Plan

Profile or retention changes require measured build/debug data and cache-tool path-safety tests.
Protected cache roots cannot be relaxed without a superseding ADR.
