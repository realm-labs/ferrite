# Minecraft Java 26.2 Reference Audit — Wave 3, Worker 2: Network Ingress Joins

## Result

The source-backed audit completed for the scope below. Its findings update reference documentation
only and do not change Ferrite implementation dispositions.

## Scope and evidence

Baseline: `1f655268dd0c5ab980b58d4fcfdcd22e8daf84d1`

This worker falsified and corrected the locked-source reference for:

- `JOIN-09` NetworkIngress × CommandAdministration
- `JOIN-10` NetworkIngress × ContentDispatch
- `JOIN-12` NetworkIngress × WorldLifecycle
- `JOIN-13` NetworkIngress × PersistenceReload
- `JOIN-14` NetworkIngress × ClientProjection
- `JOIN-15` NetworkIngress × DataReload
- `SURFACE-NETWORK-INGRESS-001`

`JOIN-11` was deliberately excluded. No Ferrite runtime code, completion ledger, implementation
disposition or shared join/surface file was changed.

## Evidence and method

Only repository-locked inputs were used: official 26.2 client/server jars, generated reports,
current reference documents and `mc-ref`. The inspected locked artifacts had these SHA-256 digests:

- `client.jar`: `40896ee9f1e2bec3c934daac7e93d41e9e3d9c2f8ae0ca366d52ffbfd1afa290`
- `server.jar`: `cdacdfb25898de5e4b4b0e5ddcc2722f77067e46605709c2d886c000ebb63ec5`
- `server-26.2.jar`: `183c0499c5f855570ee487dd38e141a53f0121f83a0b07a3bac2d8b6698823e8`

Azul Java/Javap 25 `javap -c -p` traces covered `Connection#channelRead0`, `PacketUtils`,
`PacketProcessor`, `PacketListener`, `MinecraftServer#processPacketsAndTick`,
`MinecraftServer#stopServer/#reloadResources`, `ServerCommonPacketListenerImpl`,
`ServerGamePacketListenerImpl`, `ServerConfigurationPacketListenerImpl`,
`SynchronizeRegistriesTask`, `Level#getBlockEntity`, and the relevant command, chat,
block-interaction, container, operator-block, disconnect and reconfiguration handlers. Existing
protocol, data-reload, persistence and player-lifecycle roots were cross-checked for the downstream
owners.

## Findings

1. **Ingress is not the ordinary main queue.** `PacketProcessor` owns a separate
   `ConcurrentLinkedQueue` of exact listener-and-packet pairs. `processPacketsAndTick` drains it in
   `scheduledPacketProcessing` before `tickServer`. The captured listener is gated both in
   `Connection#channelRead0` and again at queue execution, and is never rebound after a protocol
   transition.
2. **Command packets span two scheduling domains.** Chat/command listener admission occurs during
   the packet drain, signed-command last-seen state can mutate there, and `tryHandleChat` then uses
   `MinecraftServer#execute` for later parsing/dispatch. An operator-block edit normally completes
   in the packet drain. Arrival order or a single “main-queue order” therefore does not determine
   every command/admin race.
3. **Sequence ACK is a deferred high-water mark.** Use/use-on advance the mark before several
   rejection branches; block-action paths advance it after the game-mode handler. Values coalesce by
   `max`, negative input throws, and the listener sends one ACK at the start of a later tick.
   Correction is branch-specific and is not guaranteed for every rejection.
4. **Container state mismatch is convergence selection, not mutation admission.** A valid click is
   executed even when its state ID is stale. Packet hashes update only remote mirrors; mismatch
   selects a full broadcast and match selects incremental broadcast. Invalid menu/slot branches can
   return without an immediate correction.
5. **Packet positions are dimensionless and chunk admission is owner-specific.** Handlers re-read
   `player.level()` at execution. If transfer wins, the same coordinates address the destination
   level rather than necessarily failing as stale. Operator block/entity lookup reaches
   `Level#getChunkAt`, which requests `FULL`, so it can synchronously resolve a chunk instead of
   requiring it to have been loaded/active beforehand.
6. **Queued ingress is not a disconnect or clean-stop durability barrier.** A closed connection or
   switch-to-configuration causes the second listener gate to ignore queued ordinary play packets.
   Clean stop closes `PacketProcessor` before connection shutdown and player/world saves, so its
   remaining queue is abandoned. Only already completed handler prefixes can flow to player/chunk
   persistence; no transport state is replayed.
7. **Configuration does not select one atomic reload snapshot.** At configuration start the server
   captures the known-pack list and `LayeredRegistryAccess` object. Registry/tag serialization waits
   until the select-known-packs response; configuration finish separately binds play codecs from the
   then-current `server.registryAccess()` and uses the latest directly replaced client information.
   Reload publication uses the ordinary server executor, while packet admission uses the earlier
   packet-drain stage.

The unique surface root now records these source-backed boundaries and six independently runnable
race vectors.

## Integration changes: join matrix

The integration coordinator should replace the complete rows in `cross-system-ordering.md` with the
following wording.

### `JOIN-09`

> Command/chat/admin ingress first passes the active listener gate; handlers that require server
> authority run from the dedicated packet drain. Operator edits normally execute there, while
> chat/command admission may commit signature/last-seen state and then enqueue ordinary server work
> for authoritative parse/dispatch. Same-target order follows those actual queue/stage linearization
> points, not wire arrival or one generic main queue. Parse/dispatch failure retains any earlier
> admission prefix, and operator handlers retain their documented mutation/feedback prefix; durable
> effects and projection remain owner-specific. Vector: gate the packet drain and ordinary task
> queue independently while a signed command and operator-block packet edit the same carrier in both
> arrival orders.

### `JOIN-10`

> Content handlers use the current player level, menu, held stack, target entity/block and
> owner-specific sequence state when their packet-drain attempt executes. Completed packet handlers
> are visible to later drained handlers, but client prediction fields are convergence inputs only: a
> container state-ID mismatch still executes the authoritative click and chooses full rather than
> incremental sync, and block sequence updates can survive later semantic rejection. Invalid
> menu/slot/content branches emit only their explicitly mapped correction or ACK, including none.
> Vector: two use/click packets target one slot/block while content identity changes, with matching,
> stale and invalid state/sequence inputs.

### `JOIN-12`

> Movement, interaction and operator handlers re-read `player.level()` at execution and apply each
> owner's bounds, range, chunk/entity and activity rules. Packet positions contain no dimension, so
> transfer first can reinterpret identical coordinates in the destination level; it does not
> generically make the target stale. Some operator level accesses request a `FULL` chunk
> synchronously, while other owners reject unavailable/inactive targets. A completed mutation is
> visible to later unload/transfer, and cross-level membership still follows the mapped transfer
> transaction. Vector: gate transfer versus movement/use/operator packets at equal coordinates in
> two levels, including an initially unloaded destination chunk.

### `JOIN-13`

> Durability includes only handler prefixes completed before the applicable player/chunk save. Queue
> admission alone is not a commit: disconnect, listener transition or processor closure makes a
> queued captured packet fail its execution-time gate or remain undrained. Disconnect removal saves
> the completed player prefix, and chunk saves see completed world writes; clean stop closes the
> packet processor before player/world saves rather than draining it. Reconnect/restart never
> replays packets, listener state, sequences, state IDs or acknowledgements. Vector: gate one
> inventory/block mutation before execution, during its mutation prefix and after completion around
> disconnect, autosave and clean stop.

### `JOIN-14`

> Prediction convergence is packet-family-specific, not a generic accept/reject transaction.
> Block-use sequences update a cumulative high-water mark at owner-specific points, can advance on a
> later-rejected use, coalesce, and emit one ACK at a subsequent listener tick; negative sequences
> fail before update. A valid container click executes despite stale state ID, then stale selects a
> full sync and match an incremental sync. Rejection sends only the correction/ACK explicitly owned
> by that branch, possibly none. Reliable transport preserves actual send order, including deferred
> ACK and separately documented async-filter completion. Vector: record mutation, correction and
> send sites for accepted/rejected/reordered use, block-action and container-click inputs.

### `JOIN-15`

> Packet-drain work and reload publication use distinct scheduling stages: the dedicated packet
> queue drains before that cycle's `tickServer` ordinary work, while a publication task already in
> progress cannot be interleaved. Active content reads use the reload state present at their exact
> read. Configuration capture is field-specific: start captures known packs and the layered registry
> object, the known-pack response later serializes registry/tag data, and finish binds play codecs
> from then-current registry access and uses latest client information. Live reload does not force
> reconfiguration and separately broadcasts play tags/recipes. Waiting play accepts only its
> configuration ACK. Vector: gate publication before/after packet drain, configuration start,
> known-pack response and finish, recording each captured/serialized field.

The compact `cross-system-joins.toml` family records remain correctly mapped; no status,
`remaining_work` or implementation-disposition change is proposed.

## Integration changes: behavior surface

For `SURFACE-NETWORK-INGRESS-001` in `behavior-surfaces.toml`, the coordinator should make these
exact content changes without reformatting other records:

- Append selector:
  `"channel-thread gate, captured-listener packet queue, execution-time recheck and ordinary-server-task handoff"`.
- Append state domain:
  `"packet-processor FIFO, captured listener identity, waiting-for-configuration gate and processor closure"`.
- Append state domain:
  `"block-sequence high-water mark and deferred coalesced ACK; container state-ID full/incremental convergence selection"`.
- Append persistence item:
  `"queued but unexecuted ingress is discarded by disconnect, listener transition or clean-stop processor closure and is never a save barrier"`.
- Replace the first `client_projection` item with:
  `"owner-specific rejection and convergence may emit correction, deferred/coalesced acknowledgement, full or incremental state, or no immediate packet"`.
- Replace `reproduction` with:

  > Regenerate the packet report and exact serverbound-family partition; instrument both listener
  > gates, packet-processor enqueue/drain, ordinary server tasks, listener replacement and outbound
  > sends; then run captured-listener transition/closure, command two-queue, block ACK high-water,
  > container stale-state, cross-dimension position/chunk-resolution and configuration/reload
  > capture vectors from `network-ingress-roots.md`.

## Unresolved items

No new source ambiguity was introduced. The repository's four existing `SourceInconclusive` slices
and their experiments remain untouched:

- `SIM-SCHEDULED-TICKS-001` / `EXP-SIM-002`
- `ENV-LIGHTING-001` / `EXP-ENV-004`
- `PLY-BLOCK-BREAK-001` / `EXP-PLY-003`
- `WGEN-PIPELINE-EQUIVALENCE-001` / `EXP-WGEN-001`

In particular, this audit fixes server ACK send order but does not claim the rendered-frame outcome
owned by `EXP-PLY-003`.

## Evidence and verification

- `MC_REF_JAVA="$JAVA_HOME/bin/java" MC_REF_JAVAP="$JAVA_HOME/bin/javap" cargo run -q -p mc-reference --bin mc-ref -- protocol inventory`
  — passed: 256 packets; digest `f34b0956b6399c749d4638cd6d3c9226685f41fa`.
- The same environment with `protocol coverage` — passed: 256 packets in 58 families; levels
  `{C0: 3, C1: 6, C2: 5, C3: 30, C4: 14}`; statuses `{Specified: 44, GatedOptional: 14}`.
- The same environment with `protocol readiness` — passed.
- The same environment with `protocol verify` — passed offline, including the unchanged runtime
  packet catalog.
- The same environment with `verify --offline` — passed: 417 documentation IDs, 331 completion
  slices with 327 `SourceSpecified` and four `SourceInconclusive`, 2,798 symbol locators, 9,078
  locked IDs with complete coverage, 307 experiment definitions, all 36 mapped cross-system joins,
  all ten mapped behavior surfaces, protocol inventory/coverage and the implementation manifest.
- `git diff --check` — passed.

Rust formatting, Clippy and crate tests were not run because this worker changed reference
documentation and a worker report only.
