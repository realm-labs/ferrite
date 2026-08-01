# G01-P9-O010 Play Serverbound Admin-State Gate Report

## Result

Ferrite explicitly gates all seven serverbound Play admin-state packets in
`PROTO-PLAY-SERVERBOUND-ADMIN-STATE-001`. Tag queries, difficulty, game mode, creative inventory,
and game-rule capabilities all default to disabled. Their enabled decisions remain typed level-
thread effects; no packet identity, transaction, slot, or raw registry ID becomes world authority.

## Verified boundaries

- IDs 2/4/5/25/29/56/57 are locked to block tag query, difficulty, game mode, entity tag query,
  difficulty lock, creative slot, and game-rule update. Zero-valued outer-field goldens lock all
  seven schemas, while the required Play decoder remains fail-closed for the family.
- Difficulty raw IDs wrap modulo four and game mode uses survival fallback. Difficulty and lock
  admit a command game master or singleplayer owner; game mode, rules, and tag queries require a
  command game master. Difficulty/game-mode denials warn, while lock/query/rule denials no-op.
- Locked difficulty is an admitted no-op, hardcore coerces the requested value to hard, and an
  admitted lock broadcasts the resulting pair. Owner game-mode changes also update the server
  default. Game-rule entries remain an ordered list, so valid duplicate keys apply callbacks and
  announcements sequentially.
- An admitted block query replies with the same transaction and nullable data even when no block
  entity exists. An entity query replies only when the current-level entity resolves. Transactions
  stay query-local and acknowledge no other family.
- Creative mutation requires infinite materials, enabled item features, and a valid count. Slots
  1–45 update inventory and remote mirror; zero and nonnegative values above 45 no-op. Negative
  slots consume 20 drop-throttle points strictly below 1,480. AIR/empty clears an admitted inventory
  slot and consumes negative-slot throttle without creating an entity.

The capability flags have no default network enablement. Full optional codecs or handler wiring
require an explicit registered child batch.

## Evidence

- `crates/ferrite-protocol/src/java_26_2/play/serverbound/admin_state/`
- `crates/ferrite-protocol/tests/c4/play_serverbound_admin_state.rs`

Focused validation:

```text
cargo test -p ferrite-protocol --test c4 play_serverbound_admin_state --all-features
8 passed; 0 failed
cargo clippy -p ferrite-protocol --all-targets --all-features -- -D warnings
cargo ferrite source verify
source policy verified: 1181 handwritten Rust files, maximum 1200 physical lines
```

The batch acceptance gate is `cargo ferrite task check` followed by `git diff --check`.
