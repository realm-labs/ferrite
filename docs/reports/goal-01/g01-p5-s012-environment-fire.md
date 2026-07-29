# G01-P5-S012 — Environment Fire Runtime

## Result

Complete. The `SourceSpecified` `ENV-FIRE-001` slice now maps to modular production semantics and
committed behavioral tests. `EXP-ENV-003` remains a conformance trace and owns no unresolved
implementation behavior.

## Evidence

Production owner:

- `ferrite-gameplay::environment::fire`.

Committed test owner:

- `crates/ferrite-gameplay/tests/slices/environment/env_005.rs`.

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
195 passed; 0 failed
207 unique registered fire-odds identities
1 SourceSpecified slice verified
```

## Ownership notes

This batch fixes Ordinary/Soul Fire state selection, the complete odds table, portal dispatch,
near-player admission, schedule/age/Rain/infiniburn/self-removal order, direct and spatial spread,
TNT nontransactionality and base-fire contact. Region state and schedules remain authoritative;
portal construction, TNT explosion behavior, entity effect processing and client presentation
remain with their generated owners.
