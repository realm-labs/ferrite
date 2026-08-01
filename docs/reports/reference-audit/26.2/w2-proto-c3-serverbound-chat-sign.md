# Minecraft Java 26.2 Reference Audit — Wave 2, Worker 3: Serverbound Chat and Sign Update

## Result

The source-backed audit completed for the scope below. Its findings update reference documentation
only and do not change Ferrite implementation dispositions.

## Scope and evidence

- Audited families: `PROTO-PLAY-SERVERBOUND-CHAT-001` and `PROTO-PLAY-SERVERBOUND-SIGN-UPDATE-001`
- Baseline: `c5675bd7945981cbbfb120146c716abb130edaf8`
- Sources: repository-locked official Minecraft Java 26.2 client/server jars, generated reports,
  existing documentation and `mc-ref` only
- No Ferrite runtime code, implementation disposition, protocol catalog or shared conformance file
  was changed.

## Findings

### Chat, commands and acknowledgement state

- Corrected chain consumption from a packet-level rule to a verification-level rule. A link advances
  only after a signature verifies. Missing signature and expired-key failures do not consume a link;
  a signed command can nevertheless retain an already-verified prefix before a later entry or
  coverage failure.
- Distinguished command mismatch branches. An unknown transmitted argument name explicitly breaks
  the chain. A missing authoritative name is found after supplied entries were processed, returns
  the same mismatch error without an explicit break, and does not roll back prior link advances.
  Repeated transmitted names each verify and consume a link before the later value replaces the
  earlier map entry.
- Added the timestamp falsifier omitted by the prior prose: ordering compares the full
  epoch-millisecond `Instant`, while the RSA body signs only epoch seconds. A rollback inside one
  second can therefore break the chain although the encoded signed timestamp component is equal.
- Corrected command slash handling. The vanilla UI normally passes a slashless command, but IDs 7
  and 8 accept a default UTF string and the server parses it unchanged. Only suggestion ID 15 strips
  at most one leading slash.
- Narrowed future-chain ordering. Each accepted chat starts its filter and computes decoration in
  the scheduled task; the connection `FutureChain` serializes the completion consumer and broadcast,
  not those starts. Non-cancellation failure skips that broadcast and is logged while later work can
  continue; disconnect closes/cancels the continuation.
- Confirmed that last-seen update mutation precedes later admission and remains nontransactional,
  session installation precedes its queued player-info publication, chat and command throttlers are
  separate, and suggestion IDs have no server outstanding-request table.
- Recorded the persistence boundary: session/decoder, chain/timestamp, last-seen validator,
  signature cache, chat index, future chain, suggestion correlation and throttlers are play-listener
  or client-connection state and are not player/world save data.

### Sign update

- Corrected the authorization model. `SignBlockEntity` stores only the allowed editor's UUID; it
  stores no authorized face. The packet boolean selects the face at async completion, so a
  nonvanilla still-authorized sender can open one face and submit the other.
- Confirmed that editor-open admission checks build permission, chosen-face editability and editor
  ownership, but completion revalidates only loaded chunk/sign identity, wax, level and sender UUID.
  It does not directly recheck range, build permission or the originally opened face.
- Made formatting normalization exact: `ChatFormatting.stripFormatting` removes only
  case-insensitive section-sign pairs in `[0-9A-FK-OR]`; orphan or unrecognized pairs remain.
- Added exceptional async behavior. Failed filtering skips the completion callback; disconnect turns
  the filter into cancellation. Neither path resets idle or reaches world mutation.
- Confirmed completion-time style and player filtering preference determine the new text projection;
  ordinary four-line success constructs new `SignText`, producing the first flags-3 update, then
  clears authorization and produces the unconditional second update.
- Added reload behavior. Front/back text and wax are saved and included in update-tag projection.
  The editor UUID is not serialized, and a pending filter future is runtime state, so reload loses
  authorization while accepted text persists.

## Official anchors inspected

Server anchors included `ServerGamePacketListenerImpl`, `LastSeenMessagesValidator`,
`SignedMessageChain`, `SignedMessageLink`, `SignedMessageBody`, `FutureChain`, `SignBlock`,
`SignBlockEntity`, `SignText`, `StringUtil`, `ChatFormatting`, `ProfilePublicKey`,
`RemoteChatSession`, `ArgumentSignatures` and all six chat-family plus sign-update packet codecs.

Client anchors included `ClientPacketListener`, `LastSeenMessagesTracker`, `LocalChatSession`,
`LocalPlayer`, `AbstractSignEditScreen`, `ClientSuggestionProvider` and the corresponding official
packet codecs. Registry/protocol identity was checked against the locked generated reports.

## Reproduction

- Sign two messages in one epoch second and send the later-millisecond message first, then the
  earlier-millisecond message with otherwise valid chain/body inputs.
- Send a signed command with a valid first entry followed by unknown, missing and duplicate-name
  variants; inspect the next accepted chain index after each branch.
- Send leading-slash strings through IDs 7, 8 and 15 and compare the exact dispatcher input.
- Complete two chat filters out of order, then repeat with exceptional completion and disconnect;
  distinguish filter/decorator start order from serialized consumer/broadcast order.
- Apply an invalid last-seen checksum after legal offset and slot writes, then inspect the mutated
  window before disconnect.
- Open one sign face, delay filtering, submit the opposite packet side, and independently toggle
  wax, build permission, range, editor UUID, block entity, chunk and player level before completion.
- Submit recognized, orphan and unrecognized section-sign pairs; fail/cancel filtering; reload
  before completion and after acceptance; compare persisted text/editor authority and both update
  calls.

## Unresolved items and integration notes

- `protocol/conformance.md` still says the chat future chain serializes filter/decorate/broadcast as
  one unit. The locked handlers show that filters start and decoration is computed before append;
  only completion consumers are serialized. This shared-file correction is intentionally deferred to
  integration because it was outside this worker's allowed edit set.
- The existing conformance vectors should add the within-one-second millisecond rollback, partial
  command-chain consumption, leading-slash ID 7/8 comparison, opposite-face sign submission,
  filter-failure/cancellation and sign reload cases. No source-indeterminate fact remains in the two
  assigned leaf sections; executing those vectors remains an implementation/conformance task, not a
  claim of Ferrite implementation verification.

## Evidence and verification

All commands used Azul Java 25 through `MC_REF_JAVA="$JAVA_HOME/bin/java"` and the matching
`MC_REF_JAVAP="$JAVA_HOME/bin/javap"` executable.

- `cargo run -q -p mc-reference --bin mc-ref -- protocol inventory` — passed: 256 packets; digest
  `f34b0956b6399c749d4638cd6d3c9226685f41fa`.
- `cargo run -q -p mc-reference --bin mc-ref -- protocol coverage` — passed: 256 packets in 58
  families; levels C0 3, C1 6, C2 5, C3 30 and C4 14; 44 specified and 14 gated optional.
- `cargo run -q -p mc-reference --bin mc-ref -- protocol readiness` — passed.
- `cargo run -q -p mc-reference --bin mc-ref -- protocol verify` — passed offline, including the
  runtime packet catalog.
- `cargo run -q -p mc-reference --bin mc-ref -- verify --offline` — passed: 417 documentation IDs
  including 352 leaf rules; 331 completion slices; 2,798 symbol locators across 952 classes with 952
  cache hits and no misses; 9,078 locked IDs with no unclassified, ambiguous or unreviewed IDs; 307
  experiment definitions; protocol inventory/coverage and the unchanged implementation manifest all
  verified.
- `git diff --check` — passed before verification; repeated after final report editing.
