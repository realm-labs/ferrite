# Minecraft Java 26.2 Reference Audit — Wave 2, Worker 6: C4 Play Serverbound

## Result

The source-backed audit completed for the scope below. Its findings update reference documentation
only and do not change Ferrite implementation dispositions.

## Scope and evidence

- Integration baseline: `c5675bd7945981cbbfb120146c716abb130edaf8`
- Families: `PROTO-PLAY-SERVERBOUND-ADMIN-STATE-001`, `PROTO-PLAY-SERVERBOUND-COMMON-SERVICES-001`,
  `PROTO-PLAY-SERVERBOUND-DEBUG-SUBSCRIPTION-001`, `PROTO-PLAY-SERVERBOUND-OPERATOR-BLOCKS-001`, and
  `PROTO-PLAY-SERVERBOUND-RECONFIGURATION-001`
- Evidence: locked official Minecraft Java 26.2 client/server jars, generated packet and registry
  reports, existing reference documents, and repository `mc-ref` tooling only

The audited jars matched the source locks: client SHA-1 `2dc72797acbc1b63fc16a11c4ac393605f453754`
and server SHA-1 `823e2250d24b3ddac457a60c92a6a941943fcd6a`. Inspection and verification used the
configured Azul Java 25 `java` and `javap` binaries through `MC_REF_JAVA` and `MC_REF_JAVAP`.

## Findings

### Administration and creative ingress

- All assigned administrative and debug-subscription handlers transfer to the level/server thread.
  The prior text incorrectly exempted debug-subscription assignment.
- An authorized difficulty request remains a no-op while difficulty is locked. Otherwise hardcore
  coerces hard and the resulting difficulty/lock pair is broadcast. The singleplayer owner's game
  mode request also updates the default game mode through the ordinary command path.
- Creative slots `1..=45` update the inventory-menu slot, remote mirror, then broadcast changes.
- The negative-slot drop throttler has increment `20`, threshold `1480`, and
  one-point-per-server-tick decay. An empty or positive-count AIR request creates no item entity but
  still consumes throttle budget while under threshold; this was the most important missing
  abort-side effect.
- The game-rule list has no family-specific count ceiling below the default `Integer.MAX_VALUE`
  collection bound; practical cardinality is constrained by the enclosing packet-size boundary.
  Entries still deserialize, mutate/callback, and announce to operators in wire order.

### Common services and liveness handoff

- Cookie refusal, custom-payload ignore, and ping-request echo are direct receiving-thread paths.
  Ping uses `Connection#send` and has no token table, permission, timeout, rate, or gameplay ACK
  handoff.
- Resource-pack responses, not only custom clicks, transfer to the server packet processor. Play
  retains no UUID/task correlation and disconnects only `DECLINED` when the pack-required flag is
  true.
- The locked base custom-click callback only emits a debug log for the identifier/optional tag. The
  prior wording implied a configured semantic dispatcher that does not exist in the locked base
  server.
- No change is required in the separately owned C2 keepalive/pong section: neither common-service
  ping nor any assigned packet satisfies keepalive or another gameplay acknowledgement.

### Operator blocks

- Existing permission, target-type, enum/fallback, clamp, field-write, operation and no-ACK claims
  survived falsification.
- Test-instance query/init resolve the packet data's test key without installing the record and send
  status directly. Mutation actions install data before their operation and publish a flags-`3`
  update using synthetic old state AIR and handler-time current state; the prior text omitted that
  observable publication payload.
- Command, structure, jigsaw and test state persist only through their owning block/entity stores;
  packet positions, ordinals, priorities and UI error/status values are not independent durable
  protocol identities.

### Debug subscriptions

- The codec counts encoded elements before set deduplication, so encoded element 33 faults even if
  it duplicates an earlier raw ID. Unknown configured raw IDs fault.
- The level-thread handler copies and replaces the requested set. Permission is re-evaluated during
  each effective-subscriber rebuild, so retained unauthorized requests can become effective later.
- Requested/effective sets are runtime-only. Disconnect or reconfiguration removes the old player
  object and loses the request; no reload or persistence handoff restores it.

### Reconfiguration acknowledgement

- `switchToConfig` marks waiting, removes the player, then sends the terminal clientbound packet and
  installs configuration outbound. `PlayerList#remove` awards leave-game and saves the player before
  entity/membership removal, so this is a real persistence boundary rather than an in-place refresh.
- The client installs configuration inbound, sends the fieldless play ID 16, then installs
  configuration outbound. The server valid-ACK handler performs no server-thread transfer and
  replaces inbound listener/protocol at the terminal network boundary.
- The replacement server `CommonListenerCookie` contains profile, current latency, latest client
  information and transferred flag. It does not carry the client's cookie map.
- Early/unsolicited ID 16 throws. A duplicate is decoded under configuration, where the play packet
  identity is illegal. Neither branch acknowledges registry, world, chat, container, teleport or
  simulation state.

## Unresolved items and integration notes

All five families remain `GatedOptional` because they are optional operator/debug/common transition
surfaces, not because of a source unknown. Their `unknowns` arrays remain empty. No implementation
disposition was changed or marked Verified.

No shared-file or cross-family correction is required. The worker changed only the assigned sections
of `play-serverbound.md`, the exact five family records in protocol `completion.toml`, and this
report. The existing runtime packet catalog, conformance document, implementation manifest and other
family records were not edited.

## Reproduction

- Encode/decode minimum, maximum, invalid and residual-byte cases through the official codecs, then
  invoke the official handler independently; a round trip alone is insufficient.
- Administration: cross permission/owner/locked/hardcore/default-mode state; duplicate/invalid game
  rules; query target presence; creative slot, remote mirror, feature, count and throttle cursor.
- Common services: separate receiving-thread cookie/payload/ping order from server-processor
  resource/custom-click order; test every strict resource action and required-pack branch.
- Operator blocks: cross permission, feature enablement and target type; record field writes,
  operation result/messages, changed/update calls and test AIR-to-current publication.
- Debug: encode duplicate positions around 32/33, unknown IDs, permission changes, tracked/global
  delivery, synchronizer sleep/wake and disconnect/reconfiguration loss.
- Reconfiguration: inject early and duplicate ACKs around both directional codec installations;
  assert save/remove-before-start and replacement-listener cookie contents before ordinary return.

## Evidence and verification

Every `mc-ref` invocation used Azul Java 25 through both `MC_REF_JAVA` and `MC_REF_JAVAP`.

- `cargo run -p mc-reference --bin mc-ref -- protocol inventory` — passed: 256 packets, digest
  `f34b0956b6399c749d4638cd6d3c9226685f41fa`.
- `cargo run -p mc-reference --bin mc-ref -- protocol coverage` — passed: 256 packets in 58
  families; levels `{C0: 3, C1: 6, C2: 5, C3: 30, C4: 14}`; statuses
  `{Specified: 44, GatedOptional: 14}`.
- `cargo run -p mc-reference --bin mc-ref -- protocol readiness` — passed.
- `cargo run -p mc-reference --bin mc-ref -- protocol verify` — passed offline, including runtime
  packet-catalog verification.
- `cargo run -p mc-reference --bin mc-ref -- verify --offline` — passed: 9078 locked IDs with zero
  unclassified or ambiguous entries, 307 experiment definitions, all protocol checks above, and the
  implementation/readiness ledgers verified without modification.
- `git diff --check` — passed.

This is a reference-documentation-only change, so the repository's Rust formatting and Clippy checks
are not applicable.
