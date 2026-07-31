# G01-P9-F013 Sign-Update Report

## Result

Ferrite implements and verifies serverbound ID 61 `minecraft:sign_update` in
`PROTO-PLAY-SERVERBOUND-SIGN-UPDATE-001`. The adapter decodes packed coordinates and side choice,
while accepted text becomes normalized literal world state and converges through ordinary
block-entity publication.

## Verified boundaries

- The body contains one packed signed block position, one nonzero-normalizing side Boolean, and
  exactly four strings. The private server decoder enforces 384 UTF-16 units/1,152 bytes per line,
  while the member encoder deliberately retains its asymmetric default 32,767-unit bound.
- Malformed UTF-8 replacement-decodes. Negative/over-limit lengths, decoded-unit overflow,
  truncation, and residual bytes fault before semantic handling; no sign, edit, range, wax, width,
  or line validation leaks into the codec.
- The editor's removal callback snapshots its activation position/side and current four lines once
  on every connected normal exit; a missing connection emits nothing and repeated removal does not
  duplicate the submission.
- Receipt strips only recognized legacy formatting codes in line order before filtering. Async
  completion then uses current loaded/entity state and accepts only an unwaxed sign with a level and
  the exact sender stored as allowed editor. Rejections neither mutate text nor clear authorization.
- Success replaces exactly the selected front/back side, retains each prior line style, stores
  filtered-only or raw-plus-filtered literals according to player filtering, marks the sign changed,
  emits flags-3 block updates before and after clearing the allowed editor, and repeats both updates
  even for semantically equal text.
- The sign tick independently clears a stored editor whose player is absent or outside the padded
  interaction range. The submission adds no token, menu state, replay guard, direct response, or
  corrective packet.

## Evidence

- `crates/ferrite-protocol/src/java_26_2/play/serverbound/sign_update/`
- `crates/ferrite-protocol/tests/c3/play_serverbound_sign_update.rs`

Focused validation:

```text
cargo test -p ferrite-protocol --test c3 play_serverbound_sign_update --all-features
11 passed; 0 failed
cargo test -p ferrite-protocol --test c3 --all-features
318 passed; 0 failed
cargo clippy -p ferrite-protocol --all-targets --all-features -- -D warnings
cargo ferrite source verify
```

The batch acceptance gate is `cargo ferrite task check` followed by `git diff --check`.
