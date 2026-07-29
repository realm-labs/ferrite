# Source Policy Exceptions

Exceptions are temporary, narrow, and tracked to a concrete removal batch. They are not permission
to add unrelated responsibilities.

## `tools/mc-reference/src/lib.rs`

- Current scope: legacy Minecraft reference acquisition, catalog, source-symbol, experiment,
  protocol, surface, and documentation verification.
- Current size: 3,538 physical lines.
- Reason: this file predates `AGENTS.md` and is the verifier that froze Goal 01's audited
  denominator. Splitting it during the Phase 0 baseline capture would have mixed a large mechanical
  migration with evidence freezing and risked changing the reference result.
- Constraint: new implementation-manifest functionality already lives in its own module; do not add
  another responsibility to the legacy file.
- Removal owner: `G01-P1-B6`, which introduces repository-wide source-size gates.
- Exit condition: split the legacy responsibilities into named modules, each at or below 1,200
  physical lines, with unchanged offline verification output and passing regression tests.

No production Ferrite crate has a source-size exception.
