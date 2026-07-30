# G01-P7-S005 — ENT-006 Effects

## Result

Complete. The source-specified `ENT-EFFECTS-001` slice now has production owners for instance
merge/hidden chains, add/force/remove callbacks, server duration and attribute ticking, and every
specialized vanilla effect described by the audited leaf.

## Evidence

Production owner:

- `ferrite-gameplay::entity::runtime::ent_006::{instance,ticking,special}`.

Committed test owner:

- `crates/ferrite-gameplay/tests/slices/entities/ent_006.rs`.

Design contract:

- [ENT-006 effect runtime](../../development/entity-effect-runtime.md).

Validated commands:

```text
cargo test -p ferrite-gameplay --test slices entities_ent_006 -- --nocapture
cargo clippy -p ferrite-gameplay --all-targets --all-features -- -D warnings
cargo ferrite task check
git diff --check
```

Focused result before the universal gate:

```text
14 ENT-006 effect tests passed; 0 failed
1 source-specified slice
40 locked mob-effect registry identities
```

The tests lock stronger/equal/weaker/infinite merge behavior; hidden expiry and promotion;
unchanged-start, force-add and removal contracts; callback false, duration one and 600; concurrent
mutation; attributes; periodic/instant effects; applicability; Bad/Raid Omen; Infested;
Wind Charged, Weaving and Oozing killed-only branches; max-cramming, failed construction and yaw
draw cardinality.
