# G01-P9-F002 Chat Presentation Report

## Result

Ferrite implements and verifies IDs 31, 33, 65 and 121 in
`PROTO-PLAY-CLIENTBOUND-CHAT-PRESENTATION-001`. Normalized authored content and audience/filter
intent remain authoritative; packed indices, registry raw IDs, signature caches, validation state,
delay queues and presentation objects stay connection- or client-local.

## Verified boundaries

- Four official packet bodies lock delete, disguised, minimal player and system chat. Full/cache
  signatures, optional fixed-256 signatures, signed body fields, 20-entry last-seen lists, strict
  filter masks, direct/registered bound chat types, fallback parameters, trusted components and
  nonzero booleans round-trip or fail closed at their locked boundary.
- The received-signature cache resolves against the pre-packet 128-entry snapshot, distinguishes
  empty from invalid slots, de-duplicates new signatures and installs the queue tail first. Global
  index mismatch advances only the index; unresolved body references precede cache push, while
  sender and validator failures follow it.
- Session validation evidence drives expiry/signature/chain rejection and permanent validator
  poisoning. Missing-session secure enforcement, non-enforced signature removal, integrated-local
  trust, the seven-minute trust window, modified content and secure-only behavior remain distinct.
- Player presentation covers visibility, receiver, block/friend, full/partial filtering, unsigned
  stripping, out-of-content filter faults, error presentation and processed-signature tracking.
  The 20-entry tracker de-duplicates consecutive signatures and signals standalone acknowledgement
  after more than 64 pending offsets.
- Delay queues drain suppressed entries until one message is shown, pause shifts the timing anchor,
  and zero delay flushes the queue. Delete clears pending/queued state first, delays young HUD-line
  replacement to 60 ticks and affects only the first matching displayed duplicate.
- Disguised chat bypasses player trust/social/filter policy but retains delay and visibility.
  System chat remains immediate; overlay ignores hidden visibility, while non-overlay system chat
  uses persistent discovered-name social policy.
- Publication is per connection: `FULL` visibility, receiver filtering, global indices, signature
  packing/cache push, post-send 4,096 pending overflow, nil-sender disguised selection and
  fully-filtered sender notice are independent. System send failure uses only the visible
  non-overlay 256-character fallback path.
- Delete is non-skippable; disguised, player and system chat are skippable. The family requires an
  installed Ready-for-Terrain Play projection and has no generation or unrelated-family
  acknowledgement.

## Evidence

- `crates/ferrite-protocol/src/java_26_2/play/clientbound/chat_presentation/`
- `crates/ferrite-protocol/tests/c3/play_clientbound_chat_presentation.rs`

Focused validation:

```text
cargo test -p ferrite-protocol --test c3 play_clientbound_chat_presentation
13 passed; 0 failed
cargo test -p ferrite-protocol --test c3
195 passed; 0 failed
cargo test -p ferrite-protocol --test c1
68 passed; 0 failed
cargo clippy -p ferrite-protocol --all-targets --all-features -- -D warnings
```

The batch acceptance gate is `cargo ferrite task check` followed by `git diff --check`.
