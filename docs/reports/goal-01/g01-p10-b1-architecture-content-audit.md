# G01-P10-B1 Architecture and Content Audit

## Result

The repository now has one repeatable `cargo ferrite audit verify` gate for Goal 01 architecture
and content completion. It fails closed on incomplete implementation dispositions, stale rule
reachability, catalog lowering drift, public API compilation warnings, dependency or source-policy
violations, and committed generated artifacts.

## Manifest and content closure

| Audit | Verified result |
|---|---:|
| Reachable parent rules | 65 |
| Reachable leaf rules | 352 |
| Gameplay slices | 331 |
| Catalog IDs | 9,078 |
| Protocol packets / families | 256 / 58 |
| Required / optional protocol families | 44 / 14 |
| Behavior surfaces | 10 |
| Cross-system joins | 36 |
| Explicit deferred observations | 4 |

The implementation-manifest verifier compares every generated record with the reference-derived
manifest, rejects missing, duplicate, dead, and stale mappings, checks evidence and test owners,
and now independently proves all parent and leaf rules remain reachable. The terminal audit then
requires every catalog, gameplay, protocol, surface, and join record to be `Verified`; only the
four exact unresolved observations may remain `DeferredExperiment`.

The locally generated content bundle lowers all 32 registries and 9,078 entries. The block catalog
contains 1,196 validated definitions and 32,366 canonical states. Re-import kept the locked client
and server artifact identities and the entry-level content-manifest digest unchanged. It corrected
the reproducible aggregate bundle digest to
`887fb5b7be081828a492bcc41c08b880a8fc40648322b5630628be057d7391e2`; no generated bundle or
official payload was committed.

## Architecture and repository boundaries

- Cargo metadata verifies all 18 workspace packages, 53 permitted workspace edges, an acyclic
  dependency graph, and exclusive Lattice dependency ownership in `ferrite-region-runtime`.
- Source policy audits all 1,230 handwritten Rust files for the 1,200-line limit, deep
  parent-relative paths, public re-exports, and broad Clippy bypasses.
- All 20 library and binary crate roots explicitly forbid unsafe code, and workspace rustdoc builds
  every public API with warnings denied.
- The tracked-file audit rejects `target`/`generated` trees, official binary artifacts, partial
  outputs, and generated content bundles. Protocol Rust generation remains confined to Cargo
  `OUT_DIR`, sourced from committed compact locks.

## Commands

```text
cargo ferrite content import
cargo ferrite audit verify
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo ferrite task check
git diff --check
```

`cargo ferrite task check` now invokes the unified audit before deployment, topology, behavior,
protocol, format, Clippy, workspace-test, and offline-reference gates.
