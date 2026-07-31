# G01-P9-F007 Scoreboard Report

## Result

Ferrite implements and verifies clientbound IDs 79, 98, 106, 109, and 110 in
`PROTO-PLAY-CLIENTBOUND-SCOREBOARD-001`. Normalized objectives, scores, display slots, teams, and
membership form the authoritative boundary; raw methods/registry IDs, client maps, formatting
results, sort state, and HUD animation remain adapter-local.

## Verified boundaries

- All five official empty bodies are locked exactly. Default-bounded strings, signed methods and
  scores, nullable fields, trusted components/styles, player counts, strict objective-render and
  number-format registries, truncation, and residual bytes follow the locked failure policy.
- Display, visibility, collision, and team-color IDs deliberately fall back to their zero values;
  optional booleans normalize and high option bits disappear. Number formats strictly dispatch
  blank, styled, and fixed payloads with entry-over-objective-over-context precedence.
- Objective add/change/remove, score set/reset, and display assignment resolve names at handler
  time. Duplicate objective add faults, missing branches warn or no-op, unknown methods are complete
  no-field operations, and objective removal clears all referencing slots and scores.
- Team operations replace complete parameters and maintain exactly one team per member. Adding a
  member removes its prior team mapping; duplicate removals apply their valid prefix and then fault,
  preserving the locked partial mutation.
- Sidebar selection prefers the local team-color slot, excludes `#` owners, sorts signed scores
  descending then owner case-insensitively, and caps at 15. Player-list projection caps its input at
  80, below-name respects its distance gate, and hearts presentation bypasses number formats while
  retaining raw signed values. Carried display names and team prefix/suffix/color inputs remain
  explicit presentation state.
- Team and membership publication broadcasts globally and records affected waypoint remakes.
  Objective tracking emits add, enum-ordered slots, then backing scores per recipient; final-slot
  removal emits objective removal, live tracked changes emit full packets, owner-wide reset is
  unconditional, and joining projection orders all team snapshots before first-slot distinct
  objective batches.
- The family is connection-local after Play level installation and has no sequence, generation,
  response, or acknowledgement; receive order alone resolves reuse and delayed traffic.

## Evidence

- `crates/ferrite-protocol/src/java_26_2/play/clientbound/scoreboard/`
- `crates/ferrite-protocol/tests/c3/play_clientbound_scoreboard.rs`

Focused validation:

```text
cargo test -p ferrite-protocol --test c3 play_clientbound_scoreboard
12 passed; 0 failed
cargo test -p ferrite-protocol --test c3
241 passed; 0 failed
cargo test -p ferrite-protocol --test c1
68 passed; 0 failed
cargo clippy -p ferrite-protocol --all-targets --all-features -- -D warnings
```

The batch acceptance gate is `cargo ferrite task check` followed by `git diff --check`.
