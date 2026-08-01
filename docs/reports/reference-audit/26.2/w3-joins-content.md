# Minecraft Java 26.2 Reference Audit — Wave 3, Worker 4: Content Dispatch Joins

## Result

The source-backed audit completed for the scope below. Its findings update reference documentation
only and do not change Ferrite implementation dispositions.

## Scope and evidence

This worker falsified and corrected `JOIN-22` through `JOIN-26` and `SURFACE-CONTENT-DISPATCH-001`
against the repository-locked Minecraft Java 26.2 artifacts. It edited only
`content-dispatch-roots.md` and this report. The shared ordering matrix, join ledger, surface
ledger, completion ledgers, implementation manifest, runtime code and indexes were read only.

The locked official wrapper/server and client hashes are:

```text
server.jar       823e2250d24b3ddac457a60c92a6a941943fcd6a
server-26.2.jar  86765a5899bd9c96461036a628796b4245715058
client.jar       2dc72797acbc1b63fc16a11c4ac393605f453754
```

`server-26.2.jar` is the locked extracted version payload used for named official-class inspection;
`server.jar` is the locked official distribution wrapper. All `javap` inspection used Azul 25.

## Findings

### `JOIN-22` — content dispatch versus player lifecycle

The existing row is directionally correct but its blanket live-read wording hides caller captures.
`ServerGamePacketListenerImpl#handleUseItemOn` first enters the current player's level executor,
advances the block-change sequence watermark, and then reads the current level, hand stack,
feature/range/hit/build/protection/teleport state before invoking `ServerPlayerGameMode#useItemOn`.
Once that content call returns, the listener emits authoritative updates for the hit position and
adjacent face. A lifecycle task cannot interleave inside the call.

Entity ticking has a different boundary. `EntityTickList#forEach` fixes the iteration map, using a
copy-on-write active map if content removes or adds an entity during iteration. Before each captured
entity reaches `Entity#tick`, `ServerLevel` rechecks `isRemoved`; nonplayers also recheck the live
entity-ticking range. Captured membership is therefore not unconditional callback execution.

Disconnect calls `ServerPlayer#disconnect` and then `PlayerList#remove`. Removal awards the leave
stat and saves player, stats and advancements before vehicle/pearl/level/list/UUID cleanup. A
content mutation committed before removal can enter the saved player payload. Removal first prevents
later packet admission; respawn constructs a new player and copies only the explicit `restoreFrom`
ledger. Bundle selection and active-use progress demonstrate transient content state:
`BundleContents` codecs/equality include the ordered list but not the selected index, and decode
reconstructs selection as `-1`.

### `JOIN-23` — content dispatch versus world lifecycle

There is no universal callback-level loaded-chunk check. Admission belongs to the scheduled, random,
entity, block-entity, interaction or generation caller. `FrogspawnBlock#tick`, for example, receives
an already selected block state and directly performs survival/destruction/hatch behavior; it does
not query chunk-holder visibility. Entity callbacks combine captured iteration membership with the
live removed/ticking-range checks above.

`ChunkMap#scheduleUnload` captures the holder's save-sync future. If its identity changes, the
completion task reschedules from the new future. Otherwise the task proceeds only when the exact
holder is still the pending-unload value. It then obtains the latest chunk, marks a `LevelChunk`
unloaded, invokes `save` and ignores its Boolean result, calls `ServerLevel#unload`, updates light,
and clears the next-save time. Re-added demand cancels removal by pending-holder identity. Thus a
callback ordered first completes its leaf prefix before unload; unload/demotion ordered first
changes the caller-specific admission or revalidation. The callback body must not be credited with a
generic fence it does not contain.

### `JOIN-24` — content dispatch versus persistence/reload

Persistence is a per-field and per-queue boundary:

- Frogspawn has only palette identity. Its separately owned scheduled tick persists, but the hatch
  callback frame, count loop, construction progress and RNG cursor do not.
- Bundle codecs persist ordered nested stack templates, while selection, current use duration,
  client scroll accumulation and sound RNG do not. A reconstructed component selects `-1`.
- Snow Golem persists generic entity state and the Boolean pumpkin projection. Targets, navigation,
  trail candidates, callback position and AI progress are derived or transient.

There is no generic callback rollback. Save after a content prefix observes only fields the leaf
made durable; save before it does not. Restart reconstructs queues and first callbacks through their
owners rather than resuming Java frames, iterators or RNG state. Missing/malformed data remains a
subtype/persistence-owner decision, not a cross-system default.

### `JOIN-25` — content dispatch versus client projection

The existing phrase “owner commit order is projection order” is too strong. Projection follows owner
call-site order, including effect calls made after a rejected write:

- `TryLaySpawnOnFluidNearLand` discards the Frogspawn `setBlock` result, then publishes
  `BLOCK_PLACE`, plays the lay sound, erases pregnancy and returns success.
- `FrogspawnBlock#hatchFrogspawn` discards `destroyBlock`'s result before hatch sound and Tadpole
  attempts. Each Tadpole is marked persistent before `addFreshEntity`, whose result is also ignored.
- `SnowGolem#aiStep` discards each Snow `setBlockAndUpdate` result before publishing that
  candidate's `BLOCK_PLACE` game event. A duplicate candidate can retry when the failed write leaves
  Air.

These effects are observable prefixes, not evidence of a successful authoritative block/entity
commit. Later block/entity update, prediction correction and tracking packets remain separately
ordered owner actions; none rolls back an already published sound or game event.

### `JOIN-26` — content dispatch versus data reload

Candidate construction is isolated. `MinecraftServer#reloadResources` builds it asynchronously and
uses `thenAcceptAsync(..., server)` for publication. The publication task closes old resources,
installs the candidate pointer, persists selected packs, applies pending static tags and then new
components, finalizes recipes, saves/reloads players, replaces functions, reloads structures and
rebuilds fuel values. An exception after the pointer assignment has no rollback path.

`MappedRegistry$3#apply` first binds every pending named set, replaces the registry's tag set, and
calls `refreshTagsInHolders`. A retained `Holder.Reference` can therefore answer later `Holder#is`
queries from the new tag membership without object replacement. In contrast, a recipe, loot table,
configuration or callback argument captured before publication remains that object; only a later
lookup through the installed resource managers selects the replacement. The shared row must not
claim that every read automatically uses whichever snapshot is currently active.

## Integration changes

### `cross-system-ordering.md`

Replace the five assigned rows with:

```markdown
| `JOIN-22` | ContentDispatch × PlayerLifecycle | Packet-driven content reads the handler-time
admitted player, level, hand, menu and entity state, while phase callers may retain an explicit
captured entity/input list and revalidate removal or activity at each element. Lifecycle cannot
interleave inside one callback. Content first contributes only its committed durable fields to later
death, disconnect-save or explicit respawn transfer; lifecycle first rejects later ingress or makes
captured removed entities skip. Menu, active-use, AI and iterator state transfer only when a named
lifecycle owner lists it. Vector: enqueue use/equip/container/entity callbacks and
disconnect/death/respawn in both task orders, including captured-list removal and save inspection. |
| `JOIN-23` | ContentDispatch × WorldLifecycle | Callback admission belongs to its scheduled,
random, entity, block-entity, interaction or generation caller; there is no generic loaded-chunk
check inside every content leaf. An admitted callback completes its owner prefix before later
server-task unload. Accepted unload waits the current save-sync future, revalidates the exact
pending holder, marks a LevelChunk unloaded, attempts save, tears down live chunk callbacks and then
updates light; re-added demand cancels by holder identity. Generation captures only its owner-listed
context and reaches accessible/ticking/send publication through distinct readiness gates. Vector:
run one scheduled, entity and interaction leaf before/after demotion, pending-unload cancellation,
save failure, teardown and fresh reload. | | `JOIN-24` | ContentDispatch × PersistenceReload | Each
leaf classifies durable fields, separately serialized queues, reconstructed caches and transient
callback state. A save after a committed prefix retains only the classified fields/queues; a save
before it does not, and no generic rollback expands that prefix. Reload reconstructs callbacks from
saved owners and never resumes a Java frame, iterator or RNG cursor, but persisted scheduled work
may later invoke a fresh callback. Missing/malformed subtype inputs retain their exact
persistence-owner behavior. Vector: save/unload/restart Frogspawn, Bundle and Snow Golem immediately
before/after each write, queue insertion and effect call, then assert first reconstructed selection,
AI and scheduled callback state. | | `JOIN-25` | ContentDispatch × ClientProjection | Projection
follows each leaf's mutation/effect/packet call sites, not successful-write order. A leaf may
discard a failed write or insertion result and still emit a game event, sound, particle, memory
change or later spawn attempt; these prefixes remain visible without creating the rejected
block/entity commit. Prediction and later authoritative updates converge through their protocol
owners and do not retract earlier effects. Vector: force false Frogspawn
destruction/placement/Tadpole insertion and Snow-Golem trail writes, recording state, event, sound,
memory and correction order. | | `JOIN-26` | ContentDispatch × DataReload | Publication cannot
interleave inside a synchronous content callback. Already captured recipe, loot, configuration and
callback objects remain old; later manager lookups use the installed candidate. Pending static-tag
application deliberately rebinds named sets and refreshes membership on existing holder references,
so the same holder can answer later tag queries from the new binding. Prior commits are not
reinterpreted; pre-swap failure retains the old snapshot, while failure after resource-pointer
replacement exposes the completed publication prefix without rollback. Vector: retain
holder/recipe/loot references, gate callbacks immediately before/after publication, and inject
failure at pointer, tag, component, recipe, player, function, structure and fuel steps. |
```

### `cross-system-joins.toml`

Keep the current owners and `Mapped` statuses. Replace only `shared_domains` for the assigned rows:

```toml
shared_domains = ["caller-captured and live-revalidated player/entity/content state across save, removal and replacement"]
shared_domains = ["caller-specific callback admission and commit prefixes across chunk demotion, save, unload and reload"]
shared_domains = ["content-owned durable fields and persisted queues versus reconstructed and transient callback state"]
shared_domains = ["owner call-site effects and corrections versus accepted, rejected and failed content writes"]
shared_domains = ["captured content objects and live holder bindings across isolated candidate construction and prefix-exposing publication"]
```

These lines correspond in order to `JOIN-22` through `JOIN-26`.

### `behavior-surfaces.toml`

Keep the `SURFACE-CONTENT-DISPATCH-001` boundary, selectors, owners, protocol families, `Mapped`
status, evidence and empty unknown list. Replace its reproduction text with:

```toml
reproduction = [
  """
Re-run catalog coverage, exact/pattern overlap and zero-match checks, query one ID from every
content family, and replay each codec/type/tag dispatch through its named owner. Then execute
JOIN-22 through JOIN-26 in both task orders plus rejected-write, save/reload and reload-publication
failure injection: distinguish captured inputs from live revalidation, durable fields and queues
from transient callback state, effect call sites from successful writes, and retained objects from
post-publication manager lookups. Require all 9,078 locked IDs to retain one audited family, zero
Unreviewed fallbacks, and all four existing SourceInconclusive experiments unchanged.
""",
]
```

No implementation disposition should change to `Verified` from this mapping evidence.

## Reproduction

Primary server bytecode entry points:

- `ServerGamePacketListenerImpl#handleUseItemOn`, `#handleUseItem`, `#onDisconnect`,
  `#removePlayerFromWorld`;
- `PlayerList#remove`, `#respawn`;
- `EntityTickList#forEach`, `#add`, `#remove` and `ServerLevel#tickNonPassenger` plus the
  entity-tick lambda;
- `ChunkMap#processUnloads`, `#scheduleUnload` and both schedule-unload lambdas;
- `FrogspawnBlock#tick`, `#hatchFrogspawn`, `#destroyBlock`, `#spawnTadpoles`;
- `TryLaySpawnOnFluidNearLand`'s trigger;
- `SnowGolem#aiStep`;
- `BundleContents` constructors, codecs, equality and `BundleItem#onUseTick`, `#onDestroyed`;
- `MinecraftServer#reloadResources` and its publication lambda;
- `ReloadableServerResources#updateComponentsAndStaticRegistryTags`;
- `MappedRegistry#prepareTagReload`, `MappedRegistry$3#apply`.

Run the following deterministic vectors without live external services:

1. Queue use-before-disconnect and disconnect-before-use on one player. Capture current stack/menu,
   save payload, world/list membership and the two authoritative block updates.
2. During `EntityTickList#forEach`, let entity A remove B and add C. Assert copy-on-write iteration,
   B's removed recheck, C's next-pass admission and nonplayer ticking-range loss.
3. Gate a Frogspawn scheduled tick around pending unload. Change the save-sync future and re-add the
   holder in separate runs; inject false save/destroy/insert results and capture the retained
   prefix.
4. Save/reload Bundle, Frogspawn and Snow Golem before/after each durable write and queue insertion.
   Assert Bundle selection `-1`, fresh use/AI frames, retained ordered contents/pumpkin and fresh
   scheduled-hatch RNG rather than cursor continuation.
5. Inject false Frogspawn placement/destruction/Tadpole insertion and Snow trail writes. Capture
   authoritative state plus event, sound, pregnancy, persistent-bit and later correction order.
6. Retain one built-in holder, recipe and loot table across reload. Flip tag/recipe/loot data and
   compare retained-object reads with new manager lookups; inject failure before pointer replacement
   and after every publication step.

Each trace must record server task/callback entry, captured inputs, live rechecks, every mutation or
ignored Boolean result, save admission/completion, packet/event/sound emission and first
post-boundary read. Round trips or final-state equality alone are insufficient.

## Unresolved items

No new source-unknown fact was introduced by this audit. The repository's four existing
`SourceInconclusive` slices and experiments remain unchanged:

- `SIM-SCHEDULED-TICKS-001` / `EXP-SIM-002`: restored cross-chunk equal-head ordering;
- `ENV-LIGHTING-001` / `EXP-ENV-004`: no universal loaded-frame latency bound;
- `PLY-BLOCK-BREAK-001` / `EXP-PLY-003`: possible rendered ACK/update transient;
- `WGEN-PIPELINE-001` / `EXP-WGEN-001`: quantitative player-visible equivalence thresholds and no
  inferred same-seed identity requirement.

The deterministic vectors above are executable conformance work, not source gaps, and do not promote
any implementation disposition.

## Evidence and verification

All reference commands used Azul Java/Javap 25 through `MC_REF_JAVA` and `MC_REF_JAVAP`.

```text
target/debug/mc-ref surface coverage
target/debug/mc-ref surface readiness
target/debug/mc-ref surface verify
target/debug/mc-ref symbols
target/debug/mc-ref coverage
target/debug/mc-ref readiness
target/debug/mc-ref experiment verify
target/debug/mc-ref verify --offline
git diff --check
```

All commands passed. The independent checks reported 92 command roots in 12 mapped families, 36
mapped joins, 10 mapped surfaces, 2,798 symbol locators across 952 classes, 9,078 locked IDs with
zero unclassified or ambiguous IDs, 331 readiness slices with the four existing `SourceInconclusive`
slices, and 307 verified experiment definitions. Full offline verification also reported all 256
protocol packets covered in 58 families and verified the implementation manifest without changing
it. `git diff --check` passed. Rust formatting, Clippy and tests were not run because this audit
changes reference documentation only.
