# G01-P8-S004 — WGEN-005 portals

## Result

Complete. `WGEN-PORTAL-001` has a production owner and deterministic behavioral evidence in
`ferrite-world`.

The batch implements portal processor timing and eligibility, Nether routing/search/creation/exit,
End portal blocks and travel, End gateway state/generation/travel, and same-level or cross-level
passenger-graph transfer.

## Evidence

Production owner:

- `ferrite-world::generation::portal`;
- responsibility modules `processor`, `nether`, `end_portal`, `gateway`, and `transfer`.

Committed test owner:

- `crates/ferrite-world/tests/slices/wgen_005.rs` and its responsibility-specific children.

Design contract:

- [Minecraft 26.2 portal runtime](../../development/worldgen-portal-runtime.md).

Validated commands:

```text
cargo test -p ferrite-world --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo fmt --all -- --check
cargo run -q -p mc-reference --bin mc-ref -- implementation-manifest verify
git diff --check
```

Focused result before the repository gates:

```text
37 WGEN-005 slice tests passed; 0 failed
Nether search radii 16/128, creation radius 16, and maximum rectangle 21 locked
fallback construction emits the source-ordered 24 + 14 + 6 writes with flags 3/3/18
End platform emits all 100 ordered block positions
End gateway 40-tick cooldown, 200-tick spawn age, 2,400-tick attention, and bounded radial walk verified
15/15 End-portal shader layers and all 16 color constants locked
```

## Boundary disposition

The implementation retains portal timing and failure side effects rather than collapsing travel
into a destination calculation. In particular, a ready attempt consumes cooldown before resolution;
new Nether frames ticket the final entity block while existing portals ticket their POI; gateway
contact broadcasts cooldown even if transition creation fails; and cross-level passenger transfer
preserves the specified partial result if root construction fails after passengers move.

The source audit used the SHA-1-locked official 26.2 server jar
`823e2250d24b3ddac457a60c92a6a941943fcd6a` plus the locked client shader. `PortalForcer` bytecode
confirmed the fallback's 3×2×4 support/clearance loop, the separate 4×5 border-only frame loop, and
the final 2×3 portal loop. Those writes remain sequential because duplicate mutations and neighbor
side effects are observable at the caller boundary.

S004 supplies ordered block operations, portal tickets, and transfer operations to Phase 8
integration. World-border mutation and interpolation remain `G01-P8-S005`; Region ownership and
durable application remain `G01-P8-B1`.
