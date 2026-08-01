# Minecraft Java 26.2 Reference Audit — Wave 3, Worker 6: Persistence and Reload Joins

## Result

The source-backed audit completed for the scope below. Its findings update reference documentation
only and do not change Ferrite implementation dispositions.

## Scope and evidence

- Integration baseline: `1f655268dd0c5ab980b58d4fcfdcd22e8daf84d1`
- Joins: `JOIN-31` through `JOIN-36`
- Surfaces: `SURFACE-PERSISTENCE-RELOAD-001`, `SURFACE-CLIENT-PROJECTION-001`, and
  `SURFACE-DATA-RELOAD-001`
- Allowed root documents: `persistence-reload-roots.md` and `data-reload-roots.md`

The official client and server jars match the repository locks: client SHA-1
`2dc72797acbc1b63fc16a11c4ac393605f453754` and server SHA-1
`823e2250d24b3ddac457a60c92a6a941943fcd6a`. The audit used only those jars, the locked generated
reports and current reference documents. Class inspection and verification use Azul Java 25 through
both `MC_REF_JAVA` and `MC_REF_JAVAP`.

Primary source anchors were `ChunkMap#processUnloads`, `ChunkMap#scheduleUnload`, `ChunkMap#save`,
`ChunkMap#scheduleChunkLoad`, `ChunkMap#applyChunkTrackingView`, `ServerLevel#unload`,
`PersistentEntitySectionManager#processChunkUnload`,
`PersistentEntitySectionManager#processPendingLoads`, `PersistentEntitySectionManager#saveAll`,
`PlayerChunkSender#dropChunk/#sendNextChunks`,
`ClientPacketListener#handleForgetLevelChunk/#handleConfigurationStart`,
`MinecraftServer#reloadResources` and its publication continuation, `PackRepository#setSelected`,
`PrimaryLevelData#setDataConfiguration`, `PlayerList#reloadResources`, `PlayerAdvancements#reload`,
and `StructureTemplateManager#onResourceManagerReload`.

## Findings

### JOIN-31 and persistence/reload readiness

- Holder unload waits for a stable pre-existing save-sync future and rechecks pending-holder
  identity. It does **not** wait for the unload-triggered chunk write to become durable.
- The admitted path marks a `LevelChunk` unloaded, flushes POI, clears dirty with `tryMarkSaved`,
  copies `SerializableChunkData` synchronously, admits asynchronous encode/storage, and only then
  clears block entities, tick registration and debug/light state. This is an observable prefix, not
  a chunk/entity/block-entity atomic transaction.
- A synchronous snapshot exception or asynchronous write failure is reported, but neither path
  restores dirty state, keeps the chunk live nor schedules an unload-specific retry. A crash can
  therefore expose only the prior durable version even though live teardown completed.
- Persistent entities have a separate storage state machine. A failed load remains `PENDING` and
  blocks its entity-section store/unload completion; it does not roll back or fence the already
  separate chunk-data snapshot.
- Load still reconstructs chunk data before accessible/ticking/ready/send publication. Missing or
  reported non-`Error` input creates the documented empty proto chunk; an `Error` reports a crash.

The unique persistence root now records the exact mark-unloaded/snapshot/teardown sequence, failure
nonretry behavior and a fault-injection vector.

### JOIN-32 and client projection lifetime

- A player leaving a tracking view can receive `forget_level_chunk` immediately while the same
  server chunk remains live because of another player or ticket. The old matrix wording that placed
  forget after live teardown is false.
- If the chunk was pending but never sent, `PlayerChunkSender#dropChunk` only removes it from the
  pending set and sends no forget packet. If it was already sent and the player is alive, forget is
  queued. The client drops that chunk/debug/light projection only.
- A chunk forget does not reconstruct the connection and does not reset batch ACK counters,
  prediction state, open menus or unrelated sent caches. Disconnect/rejoin and reconfiguration
  rebuild player/play-listener/chunk-sender state and have the broader reset boundary.
- A later full-chunk packet is built from the ready `LevelChunk` when the sender selects it, not
  from the prior ready-time packet or serialized client mirror. Mutations between pending admission
  and actual send are therefore included in that fresh packet snapshot.

The unique persistence root now scopes resets to their actual owner and adds reconfiguration to the
continuity vector.

### JOIN-33 and reloadable versus bootstrap data

- Worldgen/dimension registry values, generator instances and enabled feature flags are bootstrap
  state. Ordinary live reload does not rebuild them.
- Live reload can rebind static tags on existing holders and separately replaces reloadable loot
  registries, recipes, functions, advancements, data-component initializers, structure-template
  resources and fuel values. These owners publish at different post-swap steps.
- Existing chunks/entities are not rewritten. Later operations observe the manager or holder view
  read by their exact owner; no same-seed generation equivalence or one globally atomic data view is
  implied.

### JOIN-34 and boundary-specific mirror reset

- The prior matrix grouped runtime IDs, acknowledgements, interpolation, menus and sent caches under
  one unconditional reset. That is too broad for a single chunk unload/reload.
- Chunk projection teardown is coordinate-local: pending removal can be silent, while a sent forget
  drops chunk/debug/light data. Player reconstruction, reconnect and reconfiguration own the wider
  runtime-ID, ACK, interpolation, menu and sender-cache resets.
- First post-boundary publication is derived from reconstructed authoritative state, but packet
  identity/history need not match uninterrupted execution. Clean/crash differences remain limited to
  writes that actually completed.

### JOIN-35 and pack-selection durability

- Live candidate construction opens only supplied pack IDs that are currently present, in supplied
  order. After the resource pointer swap, `PackRepository#setSelected` processes the original
  request and inserts required packs. A direct adversarial request that omitted a required pack can
  therefore make the repository/world configuration list differ from the packs opened into that
  candidate.
- Publication updates `WorldDataConfiguration` in memory with the old feature set. It performs no
  immediate level-data write; only a later world save makes that selection durable.
- The post-swap task is ordered but not atomic: close old resources, install candidate, update pack
  repository and world config, publish tags/components and recipes, save/reload player state, queue
  live tag/recipe/book packets, replace functions, retarget/clear structures, then rebuild fuel.
  Injected failure retains the completed hybrid prefix without rollback.

The data reload root now distinguishes candidate packs, repository selection, in-memory selection
and later-save durability.

### JOIN-36 and active/joining/reconfiguring publication

- Only players still retained by `PlayerList` participate in the live refresh. Their server
  advancement trackers reload before one all-player tag broadcast, then each player receives the
  shared synchronized-recipe packet followed by its own initial recipe book.
- Advancement reload marks a fresh server-side projection but does not flush an advancement packet
  inside the reload task. Its ordinary later flush occurs after the reload task, unless another
  owner removes the player first.
- Function replacement, structure-template source replacement/cache clear and fuel rebuild occur
  after the tag/recipe/book sends in the same noninterleaved server task.
- A player already removed for reconfiguration is skipped by live refresh and converges through the
  configuration registry/tag pipeline. A joining or re-entering player uses whichever manager prefix
  is current when its later task runs. After a post-swap failure that can be a retained hybrid
  prefix, not an inferred complete new snapshot.
- `/reload` still sends no command tree to existing play clients. Server execution uses the newly
  installed dispatcher after the pointer swap; new/re-entering play entry sends the current tree.
  Client resource-pack assets remain independent.

## Integration changes: joins

Do not edit `cross-system-ordering.md` in this worker branch. The integration coordinator should
replace the complete cells for the assigned rows with the following wording.

### `JOIN-31`

> A dropped holder waits for a stable save-sync dependency and pending-holder identity. It then
> marks a LevelChunk unloaded, synchronously captures a dirty chunk snapshot and admits asynchronous
> storage before block-entity/tick/debug/light teardown; it does not await write durability.
> Snapshot or write failure reports without re-dirty, rollback or unload-path retry. Persistent
> entity storage is separate, and a failed/PENDING entity load blocks that manager's store/unload.
> Load reconstruction still precedes accessible/ticking/ready/send publication. Vector: cancel and
> re-admit one holder, pause after snapshot, fail encode/write/entity load, crash, then reload
> across every holder and entity-load state.

### `JOIN-32`

> Ready admission only marks a chunk pending for each current watcher. Leaving the tracking view
> silently removes an unsent pending chunk or sends forget for an already-sent chunk even while the
> server chunk remains live; the client then drops chunk/debug/light projection. A later send builds
> a fresh packet from the ready LevelChunk at sender selection time. Dimension travel retains its
> owner-defined login/position/state-before-destination-chunk order. Vector: mutate between pending
> and send, move one watcher out while another ticket keeps the chunk live, then forget/retrack/send
> through transfer and unload.

### `JOIN-33`

> Bootstrap worldgen/dimension registry values, generator instances and feature flags are fixed
> before level construction. Live reload rebinds static tags and separately replaces reloadable
> loot, recipes, functions, advancements, components, structure resources and fuel; it does not
> rewrite existing chunks/entities or bootstrap holder values. Each later consumer reads its owning
> published view, with no global atomic-snapshot or same-seed equivalence inference. Vector: gate
> generation/load/unload around tag, structure and manager publication, including an overlapping
> background generation task, and record each read point.

### `JOIN-34`

> First post-boundary projection derives from reconstructed authoritative state, but reset scope is
> owner-specific. Chunk forget drops only sent chunk/debug/light state and can leave connection ACK,
> prediction and menu state intact; reconnect and reconfiguration recreate player/play-listener/
> chunk-sender mirrors, runtime IDs, ACK windows, interpolation, menus and sent caches. Clean/crash
> differences expose only completed durable writes. Vector: compare uninterrupted, pending-unsent
> removal, sent forget/retrack, reconnect, reconfiguration, clean restart and crash through each
> owner's first authoritative packet.

### `JOIN-35`

> Persisted world configuration selects bootstrap packs/features. Live candidate construction opens
> supplied present packs, while post-swap repository selection can insert required packs and updates
> only in-memory WorldDataConfiguration with unchanged features; a later world save makes it
> durable. Publication then exposes ordered tags/components, recipes, player refresh, functions,
> structures and fuel without rewriting saved gameplay state. Pre-swap failure retains old
> resources; post-swap failure leaves its exact hybrid prefix without rollback. Vector: omit
> required/missing IDs, fail each publication step, save or crash before/after config mutation, then
> restart.

### `JOIN-36`

> Successful live publication reloads retained players' server advancement trackers, broadcasts
> tags, then sends each retained player synchronized recipes and its initial recipe book;
> advancement packets wait for ordinary later flush. Functions, structure-template source/cache and
> fuel publish afterward in the same server task. Removed/reconfiguring players are skipped and
> converge through configuration; joining/re-entering clients use the manager prefix current at
> their later task. `/reload` sends no live command tree and resource-pack assets are independent.
> Vector: capture retained, joining and removed/reconfiguring clients across success and every
> post-swap failure, including the later advancement flush.

## Integration changes: behavior surfaces

Do not edit `behavior-surfaces.toml` in this worker branch. Apply these replacements during
integration.

### `SURFACE-PERSISTENCE-RELOAD-001`

1. Replace client-projection entry
   `a reloaded state must converge to the same client-observable result as uninterrupted state`
   with:

   `after each boundary, reconstructed authoritative state converges at the owner's first publication; packet history, runtime IDs and transient interpolation need not match uninterrupted execution`

2. Add this persistence entry immediately after the original-file-format entry:

   `reset scope is owner-specific: chunk forget does not reset connection-wide ACK, prediction or menu state, while disconnect/rejoin and reconfiguration recreate player, play-listener and chunk-sender mirrors`

3. Replace the reproduction text with:

   `Run the eight vectors in persistence-reload-roots.md across clean unload/rejoin/reconfiguration/ restart, crash, missing/malformed/read/write failure, duplicate/interleaved entity load, scheduled-tick and block-entity cases; assert owner-scoped reset, durable completion and each first authoritative publication.`

### `SURFACE-CLIENT-PROJECTION-001`

1. Add this state-domain entry:

   `boundary-scoped projection lifetime: pending unsent chunks are removed silently; sent chunks forget chunk, debug and light state only; player/play-listener/chunk-sender reconstruction resets the broader runtime mirrors`

2. Replace its reproduction entry with:

   `Run the CLI experiment matrix and protocol vectors after projection changes; additionally trace pending-unsent removal, sent forget/retrack, reconnect and reconfiguration through the first fresh authoritative packets without assuming connection-wide reset at a chunk boundary.`

### `SURFACE-DATA-RELOAD-001`

1. Replace selector `reload failure, atomic publication and active-session convergence` with:

   `candidate-isolation failure, ordered post-swap publication prefixes and active-session convergence`

2. Add these persistence entries before the leaf-specific entries:
   - `selected packs update in-memory world configuration during publication and become durable only at a later world save`
   - `bootstrap registry values, generator instances and feature flags remain fixed across live reload; static tags may rebind their holders while reloadable managers publish separately`

3. In the twenty leaf-specific persistence entries currently using atomic language, replace the
   exact substring `replace atomically with the active reload snapshot` and every exact substring
   `replace atomically` with:

   `remain split between fixed bootstrap holders and their distinct ordered reloadable owner steps`

   This retains each entry's existing statement that committed palette/entity/item/progress state is
   not rewritten while removing the unsupported cross-owner atomicity claim.

4. Replace its reproduction text with:

   `Run the eight vectors in data-reload-roots.md across pack/feature selection, registry and listener loading, every pre/post-swap failure prefix and retained/joining/reconfiguring clients; compare candidate isolation, in-memory versus durable pack selection, bootstrap versus reloadable owners and exact convergence order without assuming one atomic snapshot.`

## Unresolved items and nonclaims

The audit did not edit completion ledgers and preserves all four existing `SourceInconclusive`
slices and their experiments:

- `SIM-SCHEDULED-TICKS-001` / `EXP-SIM-002`: equal restored cross-chunk queue heads.
- `ENV-LIGHTING-001` / `EXP-ENV-004`: no source-derived wall-time or rendered-frame latency bound.
- `PLY-BLOCK-BREAK-001` / `EXP-PLY-003`: possible rendered retained-state frame between ACK and
  authoritative block update.
- `WGEN-PIPELINE-EQUIVALENCE-001` / `EXP-WGEN-001`, `EXP-WGEN-005`, `EXP-WGEN-006`: quantitative
  equivalence population/thresholds and permitted divergence, with no same-seed identity inference.

One additional join-level scheduling probe remains necessary if an implementation claims a single
old/new snapshot for already-running background generation: pause a generation consumer around the
server-thread static-tag publication step, run both executor orders, and record every holder/tag/
template read. Source proves there is no global generation/reload barrier, but does not select one
runtime interleaving or justify a same-seed result.

No implementation disposition is changed or marked Verified. No generation fence, rollback,
cross-file atomicity, immediate pack-selection durability or serialized client mirror is inferred.

## Evidence and verification

Every `mc-ref` invocation used Azul Java/Javap 25 through `MC_REF_JAVA` and `MC_REF_JAVAP`.

- `cargo run -p mc-reference --bin mc-ref -- surface coverage` passed: 92 command roots in 12 mapped
  families, 36 mapped joins and 10 mapped behavior surfaces.
- `cargo run -p mc-reference --bin mc-ref -- surface readiness` passed: behavior-surface readiness
  is complete.
- `cargo run -p mc-reference --bin mc-ref -- surface verify` passed offline.
- `cargo run -p mc-reference --bin mc-ref -- coverage` passed: 9,078 locked IDs, zero unclassified
  or ambiguous IDs and zero explicitly unreviewed IDs.
- `cargo run -p mc-reference --bin mc-ref -- readiness` passed: 331 slices are classified as 327
  `SourceSpecified` and the four preserved `SourceInconclusive` slices, with no todo, in-progress or
  explicitly unreviewed slice.
- `cargo run -p mc-reference --bin mc-ref -- verify --offline` passed: documentation schema (417 IDs
  / 352 leaves), completion and symbol locators, 9,078 registry IDs, 307 experiment definitions, 256
  protocol packets (digest `f34b0956b6399c749d4638cd6d3c9226685f41fa`), 92 command roots, 36 joins,
  10 behavior surfaces and the implementation manifest all verified offline.
- `git diff --check` passed.

The changes are documentation-only; the repository's Rust formatting and Clippy checks are not
applicable under `AGENTS.md` section 6.
