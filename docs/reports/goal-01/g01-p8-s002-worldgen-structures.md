# G01-P8-S002 — WGEN-003 Structures and Jigsaw

## Result

Complete. All 24 `SourceSpecified` slices owned primarily by `WGEN-003` have production owners and
committed behavioral evidence in `ferrite-world`.

The batch implements the shared saved-template, processor, piece, jigsaw graph, pool-placement, and
registry-record path; the six locked jigsaw families; and all 15 source-procedural structure
families. Runtime records cover all 40 processor lists, 34 structures, 20 structure sets, and 188
template pools in the locked content bundle.

## Evidence

Production owner:

- `ferrite-world::generation::structure`;
- `ferrite-world::generation::worldgen_catalog`.

Committed test owner:

- `crates/ferrite-world/tests/slices/wgen_003.rs` and its responsibility-specific children.

Design contract:

- [Minecraft 26.2 structure runtime](../../development/worldgen-structure-runtime.md).

Validated commands:

```text
cargo test -p ferrite-world --all-features
cargo clippy -p ferrite-world --all-targets --all-features -- -D warnings
cargo fmt --all -- --check
cargo run -q -p mc-reference --bin mc-ref -- implementation-manifest verify
cargo ferrite content verify
cargo ferrite task check
git diff --check
```

Focused result before the universal gate:

```text
15 ferrite-world unit tests passed; 0 failed
108 WGEN-003 slice tests passed; 0 failed
320 ferrite-world slice tests passed; 0 failed
24/24 SourceSpecified WGEN-003 leaves verified
40/40 processor lists, 34/34 structures, 20/20 structure sets, and 188/188 pools available
73/73 woodland-mansion templates decoded with exact dimensions/cell/NBT counts
38/38 woodland-mansion DATA markers matched their locked identities
```

## Slice disposition

The nine jigsaw slices are verified through the shared connector/processor/record implementation
and the Ancient City, Bastion, Outpost, Trail Ruins, Trial Chambers, and Village payload families.
The remaining 15 slices are verified by dedicated buried-treasure, desert-pyramid, End-city,
fortress, igloo, jungle-temple, mineshaft, Nether-fossil, ocean-monument, ocean-ruin,
ruined-portal, shipwreck, stronghold, swamp-hut, and woodland-mansion runtimes.

Tests cover admission and abort gates, exact RNG ordering, graph expansion, collision and clipping,
piece persistence geometry, template decoding/transforms, explicit-air behavior, block entities,
loot latches, marker/entity transactions, spawn overrides, and post-placement effects.

## Source audit notes

The implementation was checked against the SHA-1-locked official 26.2 server jar
`823e2250d24b3ddac457a60c92a6a941943fcd6a` and the local locked data corpus. Two prose-count
issues were resolved from those primary inputs:

- `stronghold_biased_to` contains 38 entries, matching its official JSON and enumeration;
- the mansion room-origin fallback checks all four rectangle corners in bytecode order. The leaf
  prose says the final check repeats the initial corner, but the executable source toggles both
  coordinates for the fourth distinct corner. Runtime and deterministic tests follow the official
  executable behavior.
