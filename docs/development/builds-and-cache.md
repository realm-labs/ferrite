# Builds and Workspace Cache

Ferrite keeps routine development builds small and isolates incompatible tooling outputs.

## Profiles

Routine commands use Cargo's `dev` profile. It retains line tables for Ferrite crates and disables
debug information in dependencies. Use the explicit `debugging` profile when a debugger needs full
symbols:

```text
cargo ferrite-debug
```

That alias runs the build through `ferrite-tooling` with
`CARGO_TARGET_DIR=target/debugging`, so its artifacts do not collide with routine `target/debug`
artifacts.

The same wrapper accepts the versioned auxiliary namespaces `coverage`, `fuzz`, `bench`, and `ci`:

```text
cargo ferrite cargo coverage test --workspace
cargo ferrite cargo ci test --workspace --all-features
```

The wrapper creates an activity marker for the command duration. Cache maintenance will not touch an
active namespace.

## Repository Checks

Use the repository entrypoint:

```text
cargo ferrite-check
```

It performs the rate-limited cache-policy check, verifies workspace dependency direction and build
profiles, then runs format, Clippy with warnings denied, workspace tests, and the offline Minecraft
reference/implementation verifier. Direct Cargo commands remain available and never hide a cache
deletion.

## Cache Inspection and Pruning

The policy is committed at `.ferrite/cache-policy.toml`. Inspection and explicit pruning are:

```text
cargo ferrite cache inspect
cargo ferrite cache prune
cargo ferrite cache prune --apply
cargo ferrite cache maintain --apply
```

`inspect`, `prune`, and `maintain` are dry-run unless `--apply` is present. The repository check task
uses apply mode, but the maintenance timestamp makes it run at most once per 24 hours.

Auxiliary namespaces become eligible after seven inactive days. Routine `dev` becomes eligible only
when the workspace target footprint exceeds 40 GiB and `target/debug` has been inactive for fourteen
days. Before removal the tool:

- acquires the workspace maintenance lock;
- resolves and rechecks the exact path under this workspace's `target/`;
- checks its activity marker and Cargo lock files;
- rejects any namespace intersecting a protected path;
- prints path, size, age, eligibility, and reason.

`target/mc-reference/26.2` is protected. Global Cargo registry/git caches and paths outside the
workspace target are not candidates.
