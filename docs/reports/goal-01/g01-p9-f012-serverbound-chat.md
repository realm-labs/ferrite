# G01-P9-F012 Serverbound-Chat Report

## Result

Ferrite implements and verifies the six packets in `PROTO-PLAY-SERVERBOUND-CHAT-001`: chat
acknowledgement, unsigned and signed command, chat message, chat-session update, and command
suggestion. Durable message/player identity remains normalized; wire proof material and correlation
state are connection-local.

## Verified boundaries

- IDs 6–10 and 15 preserve the exact string, signed integer, timestamp, salt, fixed signature,
  nullable signature, counted argument-signature, last-seen bitset, UUID, X.509 RSA key, services
  signature, and transaction grammars. All limits, malformed counts/VarInts, truncation, invalid
  public-key DER, and residual bytes fail closed.
- The last-seen validator starts with 20 null slots, suppresses only consecutive duplicate pending
  signatures, retains the append that grows the tracked list to 4,097, applies offsets and bit
  mutations nontransactionally, rejects upper bits and illegal clear/set operations, and computes
  the signed-byte Java checksum exactly. The client tracker advances its 20-slot ring and emits the
  standalone acknowledgement only after offset 64.
- Signed payloads lock format version, sender/session UUIDs, signed chain index, salt, epoch seconds,
  UTF-8 content bytes, and ordered last-seen signatures. Missing/expired proof does not break the
  chain; decreasing timestamps, invalid signatures, and unknown signed-argument names do. Equal
  timestamps remain legal, repeated argument names consume links in wire order and replace their
  map entry, and missing authoritative arguments fail coverage.
- Chat-session updates compare only profile-key data, prevent expiry rollback, skip validation when
  data is equal, distinguish absent and invalid services validators, and install a decoder rooted at
  the supplied session UUID only after SHA256withRSA validation. First already-expired keys are not
  pre-rejected by the update handler.
- Chat and signed-command admission apply last-seen state before illegal-character and visibility
  exits. Filter completions drain in sender order and cancel after disconnect. Unsigned secure
  commands reject only when authoritative parsing finds signable arguments; command/chat spam
  counters are independent, charge by 20, tick by one, preserve exempt counters, and use the
  configured positive threshold.
- Suggestions strip at most one slash, preserve every signed transaction ID, cap only the first
  1,000 result entries without changing the range, and keep no server-side outstanding table.

## Evidence

- `crates/ferrite-protocol/src/java_26_2/play/serverbound/chat/`
- `crates/ferrite-protocol/tests/c3/play_serverbound_chat.rs`

Focused validation:

```text
cargo test -p ferrite-protocol --test c3 play_serverbound_chat --all-features
18 passed; 0 failed
cargo test -p ferrite-protocol --test c3 --all-features
307 passed; 0 failed
cargo clippy -p ferrite-protocol --all-targets --all-features -- -D warnings
cargo ferrite source verify
```

The batch acceptance gate is `cargo ferrite task check` followed by `git diff --check`.
