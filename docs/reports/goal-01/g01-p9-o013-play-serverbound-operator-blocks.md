# G01-P9-O013 Play Serverbound Operator-Blocks Gate Report

## Result

Ferrite explicitly gates all seven serverbound Play operator-block packets in
`PROTO-PLAY-SERVERBOUND-OPERATOR-BLOCKS-001`. The family defaults to disabled. Enabled decisions
require both instabuild and command-game-master permission, resolve only a handler-time matching
target, and expose typed level-thread effects without treating packet positions, entity IDs,
ordinals, registry IDs, or client presentation values as durable authority.

## Verified boundaries

- IDs 27/54/55/58/59/60/65 are locked to jigsaw generation, command block, command minecart,
  jigsaw edit, structure edit, test block, and test-instance action. The audited catalog resolves
  every identity and the required Play decoder remains fail-closed for the optional family.
- Command modes and structure update/mode ordinals are strict. Test-block modes and test-instance
  actions use zero fallback. Only `rollable` selects that jigsaw joint; every other string selects
  aligned. Structure offsets clamp to `[-48,48]`, sizes to `[0,48]`, integrity to `[0,1]`, and NaN
  survives the comparison clamp.
- Permission requires both instabuild and command-game-master state. Command-tool denials retain
  their explicit denial message; every other denial is silent. Missing or wrong block/entity
  targets no-op after handler-time resolution.
- Command blocks and minecarts still replace their carried state while command blocks are disabled.
  Tracking disablement clears last output; server enablement controls the update hook and selects
  success versus disabled messaging only for nonempty commands.
- Structure fields are written before update/save/load/scan, and every branch marks and publishes
  the target even for update-data or failed operations. Jigsaw generation preserves the signed
  levels value and keep flag without a handler clamp; jigsaw edits set all fields before marking
  and publishing.
- Test blocks retain mode/state/message/mark/publication order. Test-instance query/init reply
  directly without installing carried data; set/reset/save/export/run install first, then operate,
  and publish synthetic AIR to handler-time current state with flags `3`.

The configuration gate does not install full optional codecs or handler wiring. Those service
implementations require an explicit child batch.

## Evidence

- `crates/ferrite-protocol/src/java_26_2/play/serverbound/operator_blocks/`
- `crates/ferrite-protocol/tests/c4/play_serverbound_operator_blocks.rs`

Focused validation:

```text
cargo test -p ferrite-protocol --test c4 play_serverbound_operator_blocks --all-features
9 passed; 0 failed
cargo clippy -p ferrite-protocol --all-targets --all-features -- -D warnings
cargo ferrite source verify
source policy verified: 1194 handwritten Rust files, maximum 1200 physical lines
```

The batch acceptance gate is `cargo ferrite task check` followed by `git diff --check`.
