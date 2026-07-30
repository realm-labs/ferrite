# G01-P6-F008 Play Serverbound Inventory Auxiliary Report

## Result

Ferrite implements and verifies IDs 3, 24 and 50 in
`PROTO-PLAY-SERVERBOUND-INVENTORY-AUXILIARY-001`. Bundle prediction, asynchronous book mutation and
advancement-tab selection remain three independent, tokenless domains.

## Verified boundaries

- Four goldens lock the bundle, empty-book, advancement-open and advancement-close forms.
- Signed VarInts, `-1` bundle clearing, 100-page and UTF bounds, replacement UTF decode, strict
  advancement actions/identifiers, truncation and residual data are covered.
- Bundle prediction mutates before send, uses the padded visible count and permits redundant clear;
  equality and reconstruction exclude transient selection.
- Server bundle admission uses the handler-time current menu with no invented state, validity or
  mode gates; hidden full-list indices are accepted and removal uses selected/zero fallback.
- Book admission accepts only hotbar/offhand slots, starts independent tasks and rereads only
  callback-time writable content.
- Reverse completions, failures, disconnect, slot replacement, filtering preference and
  post-finalization no-op behavior are covered.
- Edit replaces all pages; signing preserves unrelated components and installs written content
  with author, generation zero and resolved true without server-side title trimming.
- Advancement opens distinguish unknown retention from known null normalization; close retains the
  cursor, reload clears it, changed roots request correction and client correction never echoes.
- Ten named C3 vectors pass; the combined C3 suite is 71 tests.

## Evidence

- `crates/ferrite-protocol/tests/c3/play_serverbound_inventory_auxiliary.rs`
- `docs/development/protocol-play-serverbound-inventory-auxiliary.md`

The batch acceptance gate is `cargo ferrite task check` followed by `git diff --check`.
