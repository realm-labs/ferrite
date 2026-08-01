# G01-P9-B1 Phase 9 Conformance Report

## Result

Phase 9 is complete. Ferrite's locked Java 26.2 packet catalog contains exactly 256 packets in 58
verified families: 44 required C0–C3 families and 14 optional C4 configuration gates. All ten root
behavior surfaces and all 36 cross-system joins now have executable test owners and committed
evidence.

## Protocol closure

- The runtime catalog retains the frozen 256-packet digest and all nine state/direction lanes.
- Every required protocol family has a verified implementation and present test owner.
- All 14 optional gate types construct default-closed. Enabled service implementations remain
  separate registration work and are not implied by gate coverage.
- Play-to-Configuration traces preserve half-duplex terminal ordering, reject early and duplicate
  acknowledgements, carry only the audited state, and recreate Play only through Configuration
  finish.

## Surface and join closure

- The five previously open surfaces—ClientProjection, CommandAdministration,
  CrossSystemOrdering, DataReload, and NetworkIngress—now have executable behavior-runner tests.
- The full surface suite passes 10/10 roots. It covers client prediction/menu/lifecycle projection,
  typed administration permission effects, reload publication and failure isolation, captured
  ingress boundaries, and aggregate cross-system order.
- The full join suite passes 36/36 pairs. The 21 Phase 9 additions lock tick/command/reload,
  ingress/command/reload/projection, command/content/lifecycle/world/persistence/projection,
  content projection/reload, lifecycle projection/reload, world projection/reload, persistence
  projection/reload, and client projection/reload boundaries.
- Join traces distinguish captured inputs from live revalidation, authoritative commit from
  projection, durable reconstruction from transient state, and reload publication from active
  consumer convergence.

## Evidence

- `crates/ferrite-testkit/src/phase9/conformance.rs`
- `crates/ferrite-testkit/src/phase9/surfaces.rs`
- `crates/ferrite-testkit/src/phase9/joins.rs`
- `apps/behavior-runner/tests/phase9_conformance.rs`
- `apps/behavior-runner/tests/surfaces/`
- `apps/behavior-runner/tests/joins/`

Focused validation:

```text
cargo test -p behavior-runner --test surfaces --all-features
10 passed; 0 failed
cargo test -p behavior-runner --test joins --all-features
36 passed; 0 failed
cargo test -p behavior-runner --test phase9_conformance --all-features
1 passed; 0 failed
cargo clippy -p ferrite-testkit -p behavior-runner --all-targets --all-features -- -D warnings
cargo ferrite source verify
source policy verified: 1229 handwritten Rust files, maximum 1200 physical lines
```

The batch acceptance gate is `cargo ferrite task check`, implementation-manifest verification,
offline reference verification, and `git diff --check`.
