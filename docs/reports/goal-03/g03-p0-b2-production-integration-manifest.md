# G03-P0-B2 Production Integration Manifest

## Outcome

Goal 03 now has a production denominator independent from the completed Goal 01 reference and
conformance denominator. The manifest starts at the formal
`apps/ferrite-server -> NodeProcess -> MinecraftGateway` entry and classifies:

- 11 nonpacket formal-entry services;
- all 48 current `PlayServerboundEntryPacket` variants, exactly once, in 12 responsibility rows;
- every row across `Ingress`, `Semantic`, `Authority`, `Continuity`, `Projection`, `FocusedTest`,
  and `ClientAcceptance`;
- 7 `Integrated`, 8 `Partial`, 1 `Unsupported`, and 7 `Planned` rows.

The initial classification is deliberately conservative. Goal 01 codec and conformance evidence is
provenance, not automatic production completion. Player-visible rows claim client acceptance only
when they link committed Goal 02 exact-client evidence.

## Machine checks

`cargo ferrite production verify` fails closed on:

- a stale formal entry, protocol source, service inventory, stage vocabulary, or counter;
- missing, duplicated, dead, or unsorted serverbound packet ownership;
- incomplete or overlapping integration-stage classification;
- invalid status semantics, unknown target Goals, or missing evidence owners;
- unsafe, absent, or workspace-escaping evidence paths;
- a focused-test claim without tests or a client-acceptance claim without Goal 02 evidence.

The verifier is part of `cargo ferrite-check`, so the production denominator is checked by routine
repository verification rather than relying on a manually maintained report.

## Evidence

- [production integration manifest](../../../goals/minecraft-java-26.2/production-integration.toml)
- [manifest contract](../../development/production-integration-manifest.md)
- `crates/ferrite-tooling/src/production.rs`
- `cargo ferrite production verify`
- `cargo test -p ferrite-tooling`

## Verification

- `cargo ferrite production verify`: passed; 11 service rows, 12 serverbound rows, 48 packets.
- `cargo test -p ferrite-tooling`: passed; 26 tests.
- `cargo fmt --all -- --check`: passed.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`: passed.
- `cargo test --workspace --all-features`: passed.
- `cargo ferrite source verify`: passed; 1,250 handwritten Rust files checked against the
  1,200-line limit.
- `git diff --check`: passed.
