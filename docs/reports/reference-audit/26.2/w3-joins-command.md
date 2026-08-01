# Minecraft Java 26.2 Reference Audit — Wave 3, Worker 3: Command Administration Joins

## Result

The source-backed audit completed for the scope below. Its findings update reference documentation
only and do not change Ferrite implementation dispositions.

## Scope and evidence

- Baseline: `1f655268dd0c5ab980b58d4fcfdcd22e8daf84d1`
- Audited joins: `JOIN-16` through `JOIN-21`
- Audited surface: `SURFACE-COMMAND-ADMINISTRATION-001`
- Unique root corrected: `command-administration-roots.md`
- Sources: repository-locked official Minecraft Java 26.2 client/server jars, generated reports,
  existing reference documents and `mc-ref` only

No Ferrite runtime code, implementation disposition, shared matrix, behavior-surface ledger,
completion ledger or implementation goal was changed. This audit makes no claim that Ferrite
implementation is Verified. The four existing `SourceInconclusive` leaves (`SIM-002`, `ENV-004`,
`PLY-003` and `WGEN-PIPELINE-EQUIVALENCE-001`) remain unchanged.

Locked artifacts inspected:

- server SHA-1 `823e2250d24b3ddac457a60c92a6a941943fcd6a`;
- client SHA-1 `2dc72797acbc1b63fc16a11c4ac393605f453754`.

## Findings

### `JOIN-16`: command content is not one blanket snapshot or transaction

Command arguments do not share one capture rule. Some parse to keys, some retain holders and some
resolve world or target objects in their terminal getters. The active argument class and handler
therefore determine whether a value was captured during dispatcher construction/parsing or read from
live server state. The source does not support a blanket "functions/arguments use the snapshot
active when parsed" claim.

Preflight and commit are likewise owner-specific. `SetBlockCommand#setBlock` in destroy mode calls
`destroyBlock` before attempting `BlockInput#place`; a failed replacement leaves the destruction.
`FillCommand#fillBlocks` walks the region and commits each admitted write without a rollback buffer.
The dispatcher only reports the terminal result and runs callbacks; it does not restore content
state, feedback or packets from a completed prefix.

### `JOIN-17`: selection is fixed for a synchronous handler, not across lifecycle tasks

Entity/player selectors resolve their collection before the terminal handler starts. The handler
then walks those object references synchronously on the server thread; it does not repeat selector
resolution between targets. A lifecycle task ordered before selection can remove a target or expose
a replacement, while an ordinary task ordered after handler admission cannot interleave with the
loop.

`KickCommand#kickPlayers` first rejects an unpublished server. It then visits selected players,
skips the single-player owner, calls `connection.disconnect` for each other player, and only then
routes that target's success. An all-owner selection throws after the loop. This is per-target
disconnect/feedback ordering, not one batch commit. `GameModeCommand#setGameMode` changes the
player, optionally sends the target's `gameMode.changed` message, and then routes the source
success.

### `JOIN-18`: chunk admission differs for every world command

- `setblock` uses `BlockPosArgument.getLoadedBlockPos` for its target.
- `fill` uses that getter only for its two endpoints. Its interior `BlockInWorld` objects permit
  loading, so loaded endpoints do not prove a fully loaded area.
- `clone` checks the complete source and destination boxes with `hasChunksAt` before mutation, then
  captures source and destination state before its move/barrier/place/update phases.
- place-feature checks a 3-by-3 chunk window around the origin; place-jigsaw checks only the origin
  chunk; place-structure constructs a candidate start before checking its derived bounding-box span;
  place-template checks from origin through origin plus the unrotated template size.
- `forceload add/remove` validates coordinate bounds and the 256-chunk limit, then changes tickets
  in x-major/z-minor order. It does not require each chunk already loaded. Loading/generation caused
  by those tickets is later work, with no captured generation phase or generation fence.

Consequently a shared "exact chunk preflight" statement must name the command, and neither full-area
admission nor an atomic chunk batch can be inferred from the command family.

### `JOIN-19`: save commands expose ordered prefixes rather than a save transaction

`SaveAllCommand#saveAll` emits `commands.save.saving` before calling
`MinecraftServer#saveEverything`. The server sets `isSaving`, saves all players, then invokes
`saveAllChunks`; level and world-data writes follow in their loop order. Flush mode joins saved-data
work and chunk flushes, while non-flush mode may schedule saved-data writes. A false result throws
after the initial feedback and after any completed durable prefix. Success feedback is emitted only
after the true result.

`save-off` and `save-on` call `MinecraftServer#setAutoSave`, which changes each level's `noSave`
flag in iteration order and reports only after the loop. `save-all` passes the override that saves
levels despite `noSave`. These methods provide no cross-level/cross-file crash atomicity, journal or
rollback. Command frames, callbacks, results and feedback remain transient; persisted carrier state
continues to be owned by the command-block carrier rules.

### `JOIN-20`: feedback has an exact nested order

`CommandSourceStack#sendSuccess` determines direct and administrator routes, evaluates the supplier
once when either route accepts it, sends to the direct source, visits the current player-list order
for eligible operators, then sends to the server log when configured. `sendFailure` sends only the
direct red failure. Silent sources suppress both. The handler decides when this call occurs, so
mutation and target-specific packets can precede it: kick requests disconnect first, gamemode
mutates and optionally informs the target first, and `save-all` deliberately emits one direct
pre-save message before the save. A later failure or result callback cannot retract those prefixes.

### `JOIN-21`: `/reload` failure preserves repository discovery changes, and command return waits

The previous join text had two material overstatements:

1. `/reload` calls `PackRepository#reload` synchronously before admission feedback and candidate
   loading. That operation replaces the repository's available set and rebuilds selection from the
   previous selected IDs. Candidate failure retains the old live `ReloadableServerResources` and
   `WorldDataConfiguration`, but it does not undo this repository discovery/selection prefix.
   `datapack enable/disable` differs: it builds a temporary selected list and does not call
   `setSelected` before successful publication.
2. `MinecraftServer#reloadResources` does asynchronous candidate work, but when invoked by these
   server-thread commands it calls `managedBlock(future::isDone)`. `/reload` and datapack
   enable/disable therefore return only after success publication or failure feedback, not before
   asynchronous completion. Their success/admission message still precedes candidate completion;
   there is no second completion-success message.

On successful publication the server closes the old resources, replaces the live resources pointer
(so `getCommands()` immediately exposes the candidate dispatcher), installs repository selection and
world-data configuration, applies components/tags and recipes, saves/reloads players, then replaces
the function library and later derived structure/fuel managers. Candidate functions were compiled
against the candidate dispatcher's final reference. The old function library remains installed
during the earlier prefix of this one server task. Ordinary server tasks cannot interleave there,
but an injected failure or reentrant observer can expose a post-dispatcher, pre-function-library
prefix; there is no rollback. Normal live reload sends no command-tree packet to existing play
clients, while later execution and later joins use the new dispatcher.

## Official anchors inspected

The principal locked server anchors were:

- `net.minecraft.commands.CommandSourceStack#sendSuccess`, `#sendFailure` and `#broadcastToAdmins`;
- `net.minecraft.server.commands.SetBlockCommand#setBlock`, `FillCommand#fillBlocks`,
  `CloneCommands#clone`, `PlaceCommand#placeFeature`, `#placeJigsaw`, `#placeStructure`,
  `#placeTemplate`, `#checkLoaded`, and `ForceLoadCommand#changeForceLoad`;
- `net.minecraft.commands.arguments.coordinates.BlockPosArgument#getLoadedBlockPos`;
- `net.minecraft.server.commands.KickCommand#kickPlayers`, `GameModeCommand#setMode`, `#setGameMode`
  and `#logGamemodeChange`;
- `net.minecraft.server.commands.SaveAllCommand#saveAll`, `#saveOff` and `#saveOn`, plus
  `net.minecraft.server.MinecraftServer#saveEverything`, `#saveAllChunks` and `#setAutoSave`;
- `net.minecraft.server.commands.ReloadCommand#discoverNewPacks`, `#reloadPacks`,
  `net.minecraft.server.commands.DataPackCommand#createPack`, `#enablePack`, `#disablePack`,
  `net.minecraft.server.packs.repository.PackRepository#reload` and `#setSelected`;
- `net.minecraft.server.MinecraftServer#reloadResources` and `#getCommands`,
  `net.minecraft.server.ReloadableServerResources`, `net.minecraft.server.ServerFunctionLibrary` and
  `net.minecraft.server.ServerFunctionManager#replaceLibrary`.

The command inventory, owners, terminal counts and path digests were checked against locked
`reports/commands.json`, `command-roots.toml` and the generated mc-ref command report.

## Integration changes

These changes were intentionally not applied because the shared files belong to the integration
coordinator.

### `cross-system-ordering.md`

Replace the six assigned matrix descriptions with the following wording (the first two table columns
stay unchanged):

- `JOIN-16`: "Argument capture is type-specific: parsed keys, holders, resolved targets and
  handler-time live lookups must not be treated as one snapshot. The terminal invokes the same
  content owner as gameplay and uses that owner's exact preflight and mutation order.
  `setblock destroy` can retain destruction when replacement fails, and area/target loops retain
  their completed prefix; dispatcher failure and result callbacks never roll it back. Vector: change
  a keyed/holder-backed resource across reload, then inject failure after destruction or one target
  while comparing `/data`, `/item`, `/loot`, `/setblock` and direct gameplay."
- `JOIN-17`: "Selectors resolve a target collection at handler entry and the server-thread handler
  traverses those object references without re-resolution or ordinary task interleaving. Lifecycle
  first changes later selection; handler first completes its per-target prefix. `kick` disconnects
  each non-owner before that target's success route, skips owner entries and fails an all-owner
  selection; gamemode mutates and optionally informs the target before source/admin feedback.
  Vector: fixed mixed-owner selectors for gamemode/effect/kick/teleport around death, respawn, join
  and disconnect on one UUID."
- `JOIN-18`: "Chunk admission is command-specific: setblock checks its position; fill checks only
  endpoints and can load its interior; clone preflights complete source/destination boxes; the four
  place subtypes check 3-by-3, origin, derived structure bounds, or
  origin-to-unrotated-template-size spans respectively. Forceload commits tickets per chunk without
  requiring an existing loaded chunk; resulting activity/generation is later and no fence or atomic
  batch is implied. Vector: compare fill/clone/place/forceload over loaded endpoints with unloaded
  interior and inject a late iteration failure."
- `JOIN-19`: "Durable command writes use their owners' dirty/save paths without a cross-owner
  transaction. `save-all` emits pre-feedback, saves players, then level/saved/world data, and keeps
  completed writes plus feedback on failure; flush joins work while non-flush can schedule it.
  `save-off/on` change per-level flags in order, and save-all overrides those flags. Frames,
  callbacks, results and feedback never persist; command-carrier persistence remains owner-specific.
  Vector: inject at every save substep and restart after ordinary save, flush, no-save, clean close
  and crash."
- `JOIN-20`: "The handler fixes mutation and target projection relative to feedback. Success routing
  evaluates once and sends direct source, current eligible OP list in order, then server log;
  failure is direct-only. Kick disconnects before each success, gamemode mutates and sends its
  target message before source/admin feedback, and save-all has pre-save feedback. Every earlier
  state, packet and feedback prefix survives later failure. Vector: mixed success/failure targets
  with direct, silent, command-block, RCON and console sources under both feedback gamerules."
- `JOIN-21`: "`/reload` refreshes repository discovery/selection before admission feedback;
  candidate failure retains old live resources/configuration but not that repository prefix.
  Datapack enable/disable keeps its proposed list temporary until publication. Server-thread
  `reloadResources` drives candidate completion before the command returns. Success swaps the live
  resources/dispatcher before configuration, tag/recipe/player and function-library steps, with no
  rollback; ordinary tasks do not interleave. Existing play clients get no command-tree resend,
  later execution/joins use the new dispatcher, and argument capture remains type-specific. Vector:
  fail after discovery, before swap and after every publication step while changing a function, tag
  and each argument-capture kind."

### `cross-system-joins.toml`

Replace `shared_domains` for the six ordered `CommandAdministration` pairs with:

```toml
["type-specific command argument capture, live content reads, owner preflight and retained mutation prefixes"]
["handler-entry target snapshots, synchronous per-target lifecycle effects and replacement boundaries"]
["command-specific chunk admission, partial area mutation, forced tickets and later generation/activity"]
["ordered command save prefixes, flush joins, per-level no-save flags and crash/reload durability"]
["handler mutation, target packet, direct feedback, OP fan-out and server-log ordering"]
["pack discovery, candidate failure, blocking command completion and ordered dispatcher/data publication"]
```

Replace their `owners` arrays, in the same order, with:

```toml
["BLK-003", "BLK-007", "ENT-001", "ITM-001", "ITM-006", "ITM-007", "WGEN-003", "BLK-COMMAND-001", "BLK-TEST-BLOCK-001", "BLK-COMMAND-AREA-001", "BLK-TEST-INSTANCE-001"]
["PLY-001", "PLY-005", "ENT-001", "ENT-005", "ENT-006", "ENT-008", "CLI-COMMAND-FEEDBACK-001"]
["BLK-003", "BLK-COMMAND-AREA-001", "WGEN-003", "WGEN-005", "WGEN-006", "WGEN-BORDER-001", "BLK-TEST-INSTANCE-001"]
["SIM-001", "BLK-003", "BLK-007", "ENT-001", "PLY-001", "BLK-COMMAND-001", "BLK-TEST-BLOCK-001", "BLK-COMMAND-AREA-001", "BLK-TEST-INSTANCE-001"]
["CLI-006", "PLY-001", "ENT-001", "BLK-003", "BLK-007", "ITM-001", "ITM-006", "ITM-007", "BLK-COMMAND-001", "BLK-TEST-BLOCK-001", "CLI-COMMAND-FEEDBACK-001", "BLK-COMMAND-AREA-001", "BLK-TEST-INSTANCE-001"]
["SIM-001", "ITM-004", "WGEN-003", "CLI-COMMAND-FEEDBACK-001", "BLK-TEST-INSTANCE-001"]
```

These are responsibility links, not implementation dispositions. Before integration, the coordinator
should mechanically confirm that every proposed owner remains a valid completion ID.

### `behavior-surfaces.toml`

For `SURFACE-COMMAND-ADMINISTRATION-001`, retain the boundary, triggers, inventory sources, protocol
families, status/evidence/unknowns and replace the following fields exactly:

```toml
selectors = [
  "all 92 roots in the locked commands report, partitioned exactly once by command-roots.toml",
  "all 1,290 executable paths and 110 redirects locked by normalized path digest",
  "permission and operator-only branches",
  "argument-type capture versus handler-time live lookup and handler-entry target snapshots",
  "command-specific chunk admission, save barriers and reload publication prefixes",
  "mutation, target projection, direct feedback, operator fan-out, server log and result-callback ordering",
]
owners = [
  "SIM-001", "SIM-006", "SIM-COMMAND-LIMIT-001", "BLK-003", "BLK-007",
  "BLK-COMMAND-001", "BLK-TEST-BLOCK-001", "BLK-COMMAND-AREA-001",
  "BLK-TEST-INSTANCE-001", "CLI-006", "CLI-COMMAND-FEEDBACK-001", "ENT-001",
  "ENT-005", "ENT-006", "ENT-008", "ENV-004", "ITM-001", "ITM-006", "ITM-007",
  "PLY-001", "WGEN-003", "WGEN-006", "WGEN-BORDER-001",
]
state_domains = [
  "server configuration, permissions, pack repository and live reloadable resources",
  "world, chunk ticket, player, entity, item, time, weather and border state",
  "command result, target projection, direct feedback, operator fan-out and server-log channels",
  "snapshotted command-action and redirect budgets plus argument-type-specific captured/live values",
  "bounded area-command block, block-entity, scheduled-tick and biome mutation prefixes",
  "operator test-block and test-instance edit, latch, template-volume and outcome state",
]
persistence = [
  "persistent command effects use each semantic owner's dirty/save boundary without generic rollback",
  "save-all exposes pre-feedback and ordered player/level/saved/world-data prefixes; flush joins work while non-flush can schedule it",
  "save-off/on mutate per-level no-save flags, while save-all explicitly overrides them",
  "command-block carriers save command, result and trigger state through BLK-COMMAND-001",
  "test blocks save mode, message and powered while their trigger latch is transient",
  "test-instance data and markers persist while a successful RUN replaces the edited entity record",
  "parser state, handler target collections, callbacks, results, feedback and packets are transient",
]
client_projection = [
  "success routes direct source before current eligible operators and server log; failure is direct-only",
  "handler-owned target packets or disconnect can precede source/admin feedback",
  "command-block and test carriers use their owner-specific block/entity update hooks",
  "ordinary world/player/entity/item convergence follows the committed owner prefix",
  "successful live reload sends tags/recipes and related data but no command-tree resend to existing play clients",
]
reproduction = [
  """
Run all eleven vectors in command-administration-roots.md. Require the locked root/path/redirect
digests, then sweep source permissions, argument capture kinds, target cardinality, chunk admission,
save injection, reload failure/publication, feedback routes, restart and client convergence without
inferring rollback, crash atomicity, generation fences or implementation verification.
""",
]
```

The owner replacement is the exact union currently declared by `command-roots.toml`; it closes the
surface ledger's omissions for data/entity/item/worldgen/score and command owners without changing
any underlying leaf disposition.

## Reproduction

1. Parse key-, holder- and target-bearing commands, gate a data reload before handler entry, and
   record which value each argument getter supplies. Repeat `setblock ... destroy` with rejected
   replacement and `fill` with an injected late write failure.
2. Select a mixed owner/non-owner kick set and a multi-player gamemode set. Record selector
   resolution, disconnect/state, target message, direct feedback, OP fan-out and server log in exact
   order.
3. Keep fill endpoints loaded around an unloaded interior, run fill and clone over the same box, and
   capture loads, writes, neighbor updates and failure. Exercise all four place subtypes at the
   documented boundary and forceload multiple chunks with a late injected failure.
4. Inject `save-all` failure at player save, each level, saved-data join and world-data write;
   restart after every prefix. Repeat flush/non-flush and save-off/on with multiple levels.
5. For `/reload`, change discovered pack membership and fail candidate construction; for datapack
   enable, fail the same stage. Compare repository available/selected lists, live resources,
   world-data configuration, feedback and command return timing.
6. Instrument every successful publication step. Observe dispatcher and function-library identity,
   player data packets and existing-client command tree; inject after the pointer swap to verify the
   retained prefix.

## Unresolved items and non-inferences

- Cross-file and filesystem crash durability depends on completed operating-system writes and must
  be measured with the declared crash/save vectors; source order is not evidence of atomicity or a
  rollback guarantee.
- Mid-loop and post-swap failures that vanilla cannot normally induce require fault injection to
  observe exact durable/client-visible prefixes. The source fixes operation order but not a
  platform-independent crash persistence result.
- Reentrant observation of the short successful publication prefix requires instrumentation. The
  source proves ordinary server tasks cannot interleave, so no concurrent visibility claim is made.
- Forceload ticket mutation does not prove when a later chunk becomes loaded, ticking or generated;
  those timings remain world-lifecycle observations and no generation fence or same-seed equivalence
  is inferred.
- The four repository `SourceInconclusive` experiments named in Scope remain explicit and were not
  changed, executed or promoted by this audit.

## Evidence and verification

All mc-ref commands used Azul Java/Javap 25:

```text
MC_REF_JAVA="$JAVA_HOME/bin/java"
MC_REF_JAVAP="$JAVA_HOME/bin/javap"
```

- `cargo run -q -p mc-reference --bin mc-ref -- surface coverage` passed: 92 command roots in 12
  mapped families, 36 mapped unordered joins and 10 mapped root surfaces.
- `cargo run -q -p mc-reference --bin mc-ref -- surface readiness` passed with the same inventory.
- `cargo run -q -p mc-reference --bin mc-ref -- surface verify` passed offline with the same
  command-root, join and surface locks.
- `cargo run -q -p mc-reference --bin mc-ref -- coverage` passed: 9,078 locked IDs, zero
  unclassified or ambiguous and zero explicitly unreviewed.
- `cargo run -q -p mc-reference --bin mc-ref -- readiness` passed: 331 slices, comprising 327
  `SourceSpecified` and the unchanged four `SourceInconclusive`; 65 parent rules, 352 leaf rules, 95
  registries and zero unreviewed catalog IDs.
- `cargo run -q -p mc-reference --bin mc-ref -- verify --offline` passed: 417 documentation IDs
  including 352 leaves; 331 completion slices; 2,798 locators across 952 classes with 952 cache hits
  and no misses; all 9,078 locked IDs; 307 experiments; 256 protocol packets in 58 families; the
  command-root/join/surface checks; and the unchanged implementation manifest.
- `git diff --check` passed before full verification and is repeated after this final report edit.

No Rust source changed, so the documentation-only exemption in `AGENTS.md` applies to Rust format,
Clippy and crate tests.
