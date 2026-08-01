# G03-P3-B1 Explicit Serverbound Dispatch

## Outcome

The formal Play application boundary now classifies every decoded 26.2 serverbound packet into a
responsibility and an explicit `Handled`, `Rejected`, `Gated`, or `Unsupported` disposition. The
exhaustive match covers all 48 enum variants and therefore fails compilation when the protocol
surface grows without a production decision.

Current block interaction, chunk feedback, lifecycle, and movement families continue into their
existing authority paths. Client-load and Region-transfer deferrals are named gates. Invalid
movement and flying are named rejections. Chat/command, entity interaction, inventory/container,
player mode/input, pong, and vehicle families remain assigned to their future Goals but now return
`Unsupported` without producing a success update.

The formal gateway records the latest bounded result, exposed through
`NodeProcess::last_serverbound_dispatch`. This makes the unsupported/default-closed path
inspectable without creating an unbounded packet log.

## Verification

- `cargo test -p ferrite-server-runtime --all-features --lib --test serverbound_dispatch`: passed;
  handled, unsupported, gated, and rejected dispositions are locked.
- `cargo ferrite production verify`: passed; 12 rows still cover all 48 packets exactly once and
  future-family rows now link explicit semantic/focused-test evidence.
- `cargo fmt --all -- --check`: passed.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`: passed.
- `cargo test --workspace --all-features`: passed.
- `cargo ferrite source verify`: passed.
- `git diff --check`: passed.
