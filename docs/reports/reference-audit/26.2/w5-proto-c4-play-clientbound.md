# Minecraft Java 26.2 Reference Audit — Wave 2, Worker 5: C4 Play Clientbound

## Result

The source-backed audit completed for the scope below. Its findings update reference documentation
only and do not change Ferrite implementation dispositions.

## Scope and evidence

This worker falsified the locked Minecraft Java 26.2 reference for:

- `PROTO-PLAY-CLIENTBOUND-ADMIN-PRESENTATION-001`;
- `PROTO-PLAY-CLIENTBOUND-COMMON-SERVICES-001`;
- `PROTO-PLAY-CLIENTBOUND-DEBUG-PROJECTION-001`;
- `PROTO-PLAY-CLIENTBOUND-LIVE-TAGS-001`;
- `PROTO-PLAY-CLIENTBOUND-RECONFIGURATION-001`.

The audit baseline was `c5675bd7945981cbbfb120146c716abb130edaf8`. Evidence was limited to the
repository-locked official client/server jars, generated packet and registry reports, existing
reference documents and `mc-ref`. No Ferrite runtime code, runtime packet catalog, implementation
disposition or shared conformance document changed.

Locked artifacts:

- server bundler SHA-1 `823e2250d24b3ddac457a60c92a6a941943fcd6a`;
- client SHA-1 `2dc72797acbc1b63fc16a11c4ac393605f453754`;
- protocol version `776` from the locked client `version.json`;
- assigned packet IDs `21/24/26/27/28/29/30/39/40/50/62/80/81/118/120/126/129/134/136/137/139/140`
  from `generated/reports/packets.json`;
- all sixteen debug-subscription raw IDs from `generated/reports/registries.json`.

## Findings

### Common services

The packet identities, field bounds, cookie/resource-pack/report/link/dialog behavior and direct
recipient model remained source-supported. Two observable handler boundaries were missing:

- a decoded `DiscardedPayload` returns before `PacketUtils.ensureRunningOnSameThread`, whereas brand
  and recognized non-discarded payloads take the main-thread path;
- transfer sets `isTransferring` before that thread hop. Singleplayer then throws with the mark
  still set; remote play disconnects, makes the old connection read-only, processes disconnection
  and starts the transfer connection with cookies, seen-player state and insecure-chat-warning
  state.

Primary entry points were
`net.minecraft.client.multiplayer.ClientCommonPacketListenerImpl#handleCustomPayload(net.minecraft.network.protocol.common.ClientboundCustomPayloadPacket)`
and
`net.minecraft.client.multiplayer.ClientCommonPacketListenerImpl#handleTransfer(net.minecraft.network.protocol.common.ClientboundTransferPacket)`.

### Reconfiguration

The terminal directional order remained correct. The audit made the exact client state boundary
explicit. The new configuration listener receives a fresh load tracker and the old profile,
telemetry manager, registry snapshot, feature set, brand, server record, post-disconnect screen,
cookies, stored chat state, report details, validated links, seen-player map and insecure-chat flag.
The old play level and UI projection are cleared. Early serverbound ID 16 faults the old server play
listener; a duplicate is decoded under configuration and is illegal there. Neither packet confirms
registry application or return-to-play completion.

Primary entry points were
`net.minecraft.client.multiplayer.ClientPacketListener#handleConfigurationStart(net.minecraft.network.protocol.game.ClientboundStartConfigurationPacket)`,
`net.minecraft.server.network.ServerGamePacketListenerImpl#switchToConfig()` and
`net.minecraft.server.network.ServerGamePacketListenerImpl#handleConfigurationAcknowledged(net.minecraft.network.protocol.game.ServerboundConfigurationAcknowledgedPacket)`.

### Debug projection

No contradiction was found. The audit rechecked the strict sixteen-entry subscription mapping,
sample-only raw ID zero, subscription-owned value codecs, present/absent update behavior, UUID
entity keys, unrequested/missing-entity ignores, source wake/seed/change/clear flow, tracking
audiences, global tick-time audience, and exact expiry condition `gameTime >= deadline`. These
caches and samples remain transient diagnostic state with no acknowledgement or persistence handoff.

Primary entry points were `net.minecraft.client.multiplayer.ClientDebugSubscriber#tick(long)`,
`net.minecraft.client.multiplayer.ClientDebugSubscriber#updateEntity(long,net.minecraft.world.entity.Entity,net.minecraft.util.debug.DebugSubscription$Update)`,
`net.minecraft.util.debug.ServerDebugSubscribers#tick()` and
`net.minecraft.util.debug.LevelDebugSynchronizers#tick(net.minecraft.util.debug.ServerDebugSubscribers)`.

### Administrative presentation

The gamerule and test-instance screen gates, requester-only publications, exact 64 MiB low-space
threshold and dedicated-administrator audience remained correct. Two presentation transitions were
missing:

- game-test highlights are keyed by absolute position. Receipt stores relative-position text and a
  wall-clock deadline of `now + 10,000 ms`; the same absolute key replaces the marker/deadline,
  different keys coexist, and render extraction removes only at `now > deadline`;
- the low-space packet handler calls `Minecraft#sendLowDiskSpaceWarning` without the ordinary packet
  main-thread assertion, and that method queues `SystemToast.onLowDiskSpace` through the client
  executor. Repeated packets are repeated queued add/update signals.

Primary entry points were
`net.minecraft.client.renderer.debug.GameTestBlockHighlightRenderer#highlightPos(net.minecraft.core.BlockPos,net.minecraft.core.BlockPos)`,
`net.minecraft.client.renderer.debug.GameTestBlockHighlightRenderer#emitGizmos()`,
`net.minecraft.client.multiplayer.ClientPacketListener#handleLowDiskSpaceWarning(net.minecraft.network.protocol.game.ClientboundLowDiskSpaceWarningPacket)`,
`net.minecraft.client.Minecraft#sendLowDiskSpaceWarning()` and
`net.minecraft.world.level.storage.LevelStorageSource$LevelStorageAccess#checkForLowDiskSpace()`.

### Live tags

The prior reference incorrectly treated every invalid member raw ID as a fatal resolution failure.
Official `TagNetworkSerialization` maps each signed raw ID through `Registry#get(int)`, then
flat-maps the returned `Optional`. Negative and out-of-range IDs are therefore silently omitted;
valid IDs preserve encounter order and duplicates. Duplicate registry/tag map keys overwrite during
decode. A missing named registry or another thrown preparation fault still occurs before any pending
set is applied, preserving all previous bindings; invalid member IDs instead yield a filtered set
that is applied normally. In-memory connections skip the binding phase but still prepare and later
recompute fuel/search state.

Primary entry points were
`net.minecraft.tags.TagNetworkSerialization#deserializeTagsFromNetwork(net.minecraft.core.Registry,net.minecraft.tags.TagNetworkSerialization$NetworkPayload)`
and
`net.minecraft.client.multiplayer.ClientPacketListener#handleUpdateTags(net.minecraft.network.protocol.common.ClientboundUpdateTagsPacket)`.

## Reproduction

Round trips are not sufficient for these branches. The corrected completion records now require:

1. Decode a hand-built update-tags frame with duplicate registry/tag keys and a surviving member
   list `[-1, valid-A, out-of-range, valid-A, valid-B]`; assert only `[valid-A, valid-A, valid-B]`
   binds in encounter order.
2. Prepare a multi-registry packet whose later registry key is absent; assert no earlier prepared
   binding applies. Repeat with only invalid member IDs and assert a successful empty/filtered
   replacement rather than rollback.
3. Deliver highlights for the same and different absolute positions at wall-clock offsets
   `9,999/10,000/10,001 ms`; assert overwrite/coexistence and the strict-greater removal edge.
4. Deliver low-space warnings from a non-client thread and assert executor ordering plus repeated
   toast add/update, separately from server threshold/permission publication.
5. Seed a sentinel in every state named by `CommonListenerCookie`, then perform the terminal
   play-to-configuration transition and inject early/duplicate acknowledgements.
6. Exercise discarded versus brand/recognized custom payload dispatch and remote versus singleplayer
   transfer to distinguish pre-thread state mutation from main-thread actions.
7. Re-run unrequested/requested debug updates, missing entity IDs, unsubscribe/resubscribe,
   wake/seed/clear, permission changes, and each `gameTime == deadline` expiry edge.

## Integration notes

The following required corrections were not made because `protocol/conformance.md` is outside this
worker's allowed files:

- `C4-LIVE-TAGS-CODECS` currently says out-of-range raw members are rejected. Its oracle must say
  structurally malformed forms and missing registries fail, while negative/out-of-range member IDs
  are filtered and valid order/duplicates survive.
- `C4-LIVE-TAGS-RESOLUTION` currently groups invalid members with all-or-none lookup failure. It
  must distinguish missing-registry preparation failure from successful filtered member resolution.
- `C4-GAMETEST-PRESENTATION` should add absolute-key replacement and the wall-clock
  `9,999/10,000/10,001 ms` removal boundary.
- `C4-LOW-DISK-WARNING` should assert the client-executor handoff in addition to threshold and
  recipient checks.

These are integration corrections, not unresolved source questions. All five completion families
remain `GatedOptional` with `unknowns = []`; no implementation disposition is claimed Verified.

## Evidence and verification

All requested checks used Azul Java 25:

```text
MC_REF_JAVA="$JAVA_HOME/bin/java"
MC_REF_JAVAP="$JAVA_HOME/bin/javap"

cargo run -q -p mc-reference --bin mc-ref -- protocol inventory
cargo run -q -p mc-reference --bin mc-ref -- protocol coverage
cargo run -q -p mc-reference --bin mc-ref -- protocol readiness
cargo run -q -p mc-reference --bin mc-ref -- protocol verify
cargo run -q -p mc-reference --bin mc-ref -- verify --offline
```

Results:

- inventory verified 256 packets, including 141 play clientbound packets, with digest
  `f34b0956b6399c749d4638cd6d3c9226685f41fa`;
- coverage verified all 256 packets in 58 families with 14 `GatedOptional` families;
- protocol readiness and protocol verification passed, including the unchanged runtime packet
  catalog;
- full offline verification passed 417 documentation IDs, 331 readiness slices, 2,798 locators
  across 952 classes, 9,078 locked registry IDs, 307 experiment definitions, protocol/command/join/
  surface coverage and implementation-manifest consistency;
- `git diff --check` passed;
- locked jar SHA-1 verification returned:

```text
823e2250d24b3ddac457a60c92a6a941943fcd6a  target/mc-reference/26.2/server.jar
2dc72797acbc1b63fc16a11c4ac393605f453754  target/mc-reference/26.2/client.jar
```

No Rust source changed, so the documentation-only exemption in `AGENTS.md` applies to Rust format,
Clippy and crate tests.

## File-size review

The assigned `play-clientbound.md` is a pre-existing consolidated legacy document above the
1,200-line guideline. This audit edits only its assigned terminal C4 sections. Splitting it would
require shared ownership/index changes explicitly prohibited for this worker, so a repository-level
protocol-document split remains the appropriate follow-up.
