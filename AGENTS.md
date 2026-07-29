# Ferrite Agent Guidelines

This file applies to the entire repository. An `AGENTS.md` in a subdirectory may add more specific requirements,
but it must not weaken the constraints in this file.

## 1. Core Principles

- Prioritize correctness, readability, maintainability, and testability. Do not sacrifice structure merely to reduce
  the number of files or shorten implementation time.
- Keep responsibility boundaries explicit. When adding code, consider module ownership, dependency direction, error
  boundaries, and test placement.
- Respect existing uncommitted work. Do not overwrite, revert, or opportunistically clean up changes unrelated to the
  current task.
- Do not introduce elaborate abstractions for hypothetical requirements, but establish appropriate boundaries for
  responsibilities that are already known.

## 2. File Size and Modularity

- A handwritten source file must not exceed 1,200 physical lines without a sound and documented reason.
- The 1,200-line limit includes inline tests and comments. Generated code, vendored code, and data snapshots are
  exempt, but their nature must be obvious from the path or file header.
- When a file approaches 1,200 lines, split it by responsibility before adding more functionality. Do not evade the
  limit by compressing formatting, removing useful comments, or combining statements.
- A legacy file already over the limit is not permission to keep growing it. Avoid increasing its net complexity and
  split it when the task scope reasonably permits.
- An exception is acceptable only when splitting would materially damage cohesion, an external format or generator
  imposes the layout, or a short-lived compatibility layer is required for a migration. Document the reason, affected
  scope, and follow-up plan in the change description.
- Organize modules by responsibility. Protocol adaptation, domain logic, runtime behavior, persistence, and operations
  interfaces should have clear boundaries.
- Do not accumulate unrelated responsibilities in one file or flatten a large number of modules into one directory
  without hierarchy.
- Module splits should produce a clear directory structure and one-way dependencies. Avoid dependency cycles and
  catch-all modules such as `common`, `utils`, or `misc` with unclear ownership.
- Keep tests close to the responsibility they verify. Large integration tests may live in a dedicated `tests`
  directory.

## 3. Rust Paths and Imports

- Do not use `super::super` or any deeper parent-relative path.
- When crossing two or more module boundaries, import through a stable path beginning with `crate::`.
- When names do not conflict and readability is preserved, prefer a `use` declaration near the top of the file and
  refer to the imported item by its short name.
- Do not repeatedly use long fully qualified paths in function bodies, type signatures, or expressions when a clear
  import would suffice.
- When names conflict, prefer an explicit and descriptive `as` alias. Retain a qualified path only when an alias would
  still be misleading.
- Use fully qualified syntax only for trait-method disambiguation, macro requirements, same-name item disambiguation,
  or another concrete necessity.
- Keep imports near the top of the module and let `rustfmt` order them. Use local imports only when conditional
  compilation or a very small local scope genuinely improves clarity.

## 4. Visibility and Re-exports

- Use the narrowest visibility that satisfies the responsibility boundary. Prefer private visibility, then
  `pub(crate)`, and reserve `pub` for genuine public APIs.
- Do not use `pub use` unless it is necessary.
- A `pub use` is appropriate only for a deliberately designed stable facade, a public API that must hide internal
  directory layout, or a domain entry point that materially improves the caller experience.
- Do not use `pub use` merely to shorten an import path, create a global prelude, conceal a confused module structure,
  or expose internal types in bulk.
- When adding or widening a public API, identify its callers, stability expectations, and ownership boundary.

## 5. Clippy and Lints

- Do not bypass Clippy with macros, conditional compilation, wrapper modules, or similar techniques.
- Do not add broad crate-level or module-level `#[allow(...)]` or `#[expect(...)]` attributes, or lower lint levels to
  hide findings.
- Prefer fixing the design or implementation issue identified by a lint.
- A local suppression is permitted only when the lint is demonstrably a false positive, conflicts with a protocol or
  external API constraint, or when the alternative would materially reduce correctness or readability.
- Limit a suppression to the smallest possible item and add an adjacent comment explaining the concrete reason.
  Statements such as "Clippy false positive" or "temporarily allowed" are not sufficient.
- If generated code requires lint exceptions, contain them at the generation boundary rather than leaking them into
  handwritten business logic.

## 6. Required Checks Before Completion

Run the following commands before delivering any Rust code change:

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

- Both checks must pass. Do not obtain a superficial pass by skipping workspace members, targets, or features.
- If a repository or dependency constraint objectively prevents a command from running, report the exact command,
  complete blocking reason, and any substitute verification performed. Never describe an unexecuted check as passed.
- Behavioral changes also require tests proportional to their impact. At minimum, run the tests for every affected
  crate. For cross-crate changes, prefer `cargo test --workspace --all-features`.
- Documentation-only changes do not require Rust checks, but still require any applicable link, formatting, or
  documentation validation.

## 7. Review Checklist

Before delivery, confirm that:

- Every source file is below 1,200 lines, or its exception has a specific and reviewable justification.
- Modules are separated by responsibility and the directory hierarchy reflects the actual architecture.
- The change contains no `super::super`, unnecessary fully qualified calls, or unnecessary `pub use`.
- Visibility is minimal and every public API has a clear caller.
- Clippy findings were fixed rather than hidden by macros or broad lint configuration.
- The reported `rustfmt`, Clippy, and applicable test results reflect commands that were actually run.

## 8. Commit Messages

- Every commit message must follow the Conventional Commits specification.
- Use the form `<type>(<optional-scope>): <description>`, for example:
  `feat(region-runtime): add generation-fenced placement claims`.
- Use a short, imperative, lowercase description without a trailing period.
- Use the type that reflects the actual change. Common types include `feat`, `fix`, `docs`, `refactor`, `test`,
  `build`, `ci`, `perf`, `style`, `chore`, and `revert`.
- Add `!` before the colon and include a `BREAKING CHANGE:` footer when a commit introduces a breaking change.
- Keep each commit focused on one coherent purpose. Do not mix unrelated formatting, refactoring, documentation, or
  behavioral changes into the same commit.
- Use a commit body when the motivation, design trade-off, migration path, or non-obvious behavior needs explanation.

## 9. Build Profiles and Cache Safety

- When `G01-P1-B1` replaces the current placeholder package, that workspace skeleton must define
  the repository's routine and full-symbol profiles at the workspace root:

  ```toml
  [profile.dev]
  debug = "line-tables-only"

  [profile.dev.package."*"]
  debug = false

  [profile.debugging]
  inherits = "dev"
  debug = true
  ```

- Use the ordinary `dev` profile for routine builds, tests, and Clippy. Use `--profile debugging`
  only when full Ferrite debug symbols are required.
- Do not change committed profile settings or use ad hoc `RUSTFLAGS` merely to work around local
  cache or debugging needs.
- CI, fuzzing, coverage, benchmarks, and other incompatible build classes must use isolated
  workspace-owned target directories and collision-resistant cache keys.
- Periodic cleanup must use the repository's guarded cache-maintenance command and versioned policy.
  Do not use an unscoped `cargo clean` as routine maintenance.
- Cache deletion must resolve and verify every target inside the declared workspace cache root,
  honor active-build locks, and preserve the most recent ordinary development artifacts.
- Never let generic build-cache maintenance delete the global Cargo registry or Git cache, a user
  home directory, an arbitrary `CARGO_TARGET_DIR`, or `target/mc-reference/26.2/`.
