# Wave 3 lifecycle joins and world-lifecycle surface audit

Date: 2026-08-01

Worktree: `/Users/mikai/CLionProjects/ferrite-worktrees/w5-world-runtime`

Branch: `codex/ref-joins-lifecycle`

Base: `1f655268dd0c5ab980b58d4fcfdcd22e8daf84d1`

## Scope and constraints

This worker falsified `JOIN-27`, `JOIN-28`, `JOIN-29`, `JOIN-30` and
`SURFACE-WORLD-LIFECYCLE-001`. Evidence was limited to the repository-locked official Minecraft
Java 26.2 client/server jars, generated reports, current reference documents and `mc-ref`. The
locked artifacts have server SHA-1 `823e2250d24b3ddac457a60c92a6a941943fcd6a` and client SHA-1
`2dc72797acbc1b63fc16a11c4ac393605f453754`; the locked protocol version is 776.

No Ferrite runtime code, completion ledger, implementation disposition, shared join/surface file or
index was changed. The already implementation-Verified PlayerLifecycle surface was not reopened.

## Evidence roots

The audit traced bytecode with Azul Java/Javap 25 at these principal owners:

- configuration and initial spawn: `ServerConfigurationPacketListenerImpl#startConfiguration`,
  `#handleConfigurationFinished`, `PrepareSpawnTask#start`, `#close`,
  `PrepareSpawnTask$Ready#spawn`, `PlayerSpawnFinder#findSpawn` and
  `SynchronizeRegistriesTask#handleResponse`;
- membership and projection: `PlayerList#placeNewPlayer`, `#respawn`, `#remove`,
  `ServerPlayer#teleport`, `ServerLevel#addNewPlayer`, `#addRespawnedPlayer`,
  `#addDuringTeleport`, `PersistentEntitySectionManager#addNewEntity`,
  `ServerLevel$EntityCallbacks#onTrackingStart`, `ChunkMap#addEntity`,
  `#updatePlayerStatus`, `#applyChunkTrackingView` and `Player#isAlwaysTicking`;
- persistence and reload: `PlayerList#save`, `#saveAll`, `#reloadResources`,
  `PlayerDataStorage#save`, `ServerStatsCounter#save`, `PlayerAdvancements#save`, `#reload`,
  `MinecraftServer#reloadResources` and `ServerGamePacketListenerImpl#switchToConfig`.

The locked `player-lifecycle-roots.md`, `world-lifecycle-roots.md`,
`persistence-reload-roots.md`, `data-reload-roots.md` and protocol play documents supplied the
existing handoff claims and packet inventories. No fact below relies on a round trip alone.

## Material findings

### `JOIN-27` — lifecycle membership versus world readiness

Initial configuration spawn, respawn and cross-level teleport are not one transaction with one
chunk-readiness gate:

- `PrepareSpawnTask` first searches under a radius-0 `SPAWN_SEARCH` ticket. Its preparing state
  then adds/refreshes a radius-3 `PLAYER_SPAWN` ticket and its ready state waits for entities before
  constructing and placing the fresh player. Cancellation only cancels the still-preparing spawn
  position future; it does not establish a general rollback for the later load/ticket or placement
  prefix. Player data is read once for preparation and again immediately before placement.
- A portal transition owns the final-position radius-3 ticket documented by `WGEN-PORTAL-001`.
  `PlayerList#respawn` and direct `ServerPlayer#teleport`, however, do not perform an equivalent
  ticket or chunk/entity-readiness wait.
- All three `ServerLevel` player insertion entry points call the same player insertion path. It
  calls `PersistentEntitySectionManager#addNewEntity` and ignores its Boolean result. A player is
  always ticking, so insertion starts destination tracking and ticking immediately, independently
  of ordinary chunk activity publication. A duplicate UUID already in the destination is forcibly
  removed before the replacement is added.
- Cross-level player teleport sends respawn, difficulty and permission packets before removing the
  player from the source. It then removes from the source, clears removed state, changes the level,
  sends position, inserts in the destination, and reprojects abilities, level state, player info
  and effects. There is no local rollback after source removal or a visible prefix.
- Respawn resolves the transition before mutation, then removes the old object from global and
  source-level membership before constructing/projecting the replacement. Destination insertion
  occurs only after respawn, position, spawn, difficulty, experience, effects, level-info and
  permission packets. It also has no local rollback. Failure after old removal can therefore leave
  neither object live; it does not restore the source object.

The join-owned paths add no placement-choice RNG. The only lifecycle-local conditional RNG observed
here is one destination-level `nextLong()` used as the client sound seed when a non-retained
respawn consumes an anchor; portal search/creation RNG and world-spawn selection remain with their
named owners.

### `JOIN-28` — save prefix and rebind boundaries

`PlayerList#remove` calls primary-player save, stats save and advancements save, in that order,
before source-level and global membership removal. The three writes are not one atomic commit:

- primary-player save writes through a temporary/safe-replace path, catches `Exception`, logs it
  and returns, so stats and advancements are still attempted after its failure;
- stats serialize and overwrite their target directly, catch I/O/JSON I/O failures, log and
  return;
- advancement encoding occurs before the write try/catch and can escape, while its direct target
  write catches I/O/JSON I/O failures. An escaping encode failure can leave primary and stats
  changed while aborting disconnect before membership removal.

Consequently, clean disconnect attempts the ordered save prefix, but crash observation cannot be
summarized as “last completed writes”: the direct-overwrite stats/advancement files may be partial,
and there is no cross-file rollback. Reconnect creates a fresh player and rebinds UUID-indexed
stats/advancements. Configuration preparation reads player data for the candidate position, then
the ready spawn path reads it again with the then-current registry access. Death/respawn still
transfers only the fields mapped by PlayerLifecycle; no old connection/menu/callback object is
resumed.

### `JOIN-29` — first visible prefix is flow-specific

The audited flows do not share one universal login-through-chunk sequence:

- fresh join emits the login/difficulty/abilities/slot/recipes, permission/commands, recipe-book,
  scoreboard/join, position/status and existing-player-info prefix before global-list insertion;
  self player-info broadcast and level-info follow, then level insertion starts tracking;
- respawn emits its replacement prefix through level-info and permission before destination
  insertion and global-list publication;
- cross-level teleport emits respawn/difficulty/permission before source removal, position before
  destination insertion, and abilities/level-info/player-info/effects after insertion;
- destination tracking can synchronously emit a chunk-cache-center control packet and marks
  differing chunks pending. Terrain chunk payload still depends on the ready-to-send path and the
  player chunk sender; cache-center control is therefore not interchangeable with the first terrain
  batch.

Replacement resets the mapped experience, health and food sent mirrors. None of the entry points
provides rollback for an already sent prefix. Tests must assert each flow's packet and membership
prefix separately, including failure injection before and after source removal, destination
insertion, cache-center emission and pending terrain marking.

### `JOIN-30` — reload serialization is task-scoped, not session-scoped

The reload candidate is prepared asynchronously and publication runs as one server-executor task.
That task closes old resources, swaps the resource/registry snapshot, updates pack/configuration and
static tag/component state, finalizes recipes, saves every player currently in the global list,
reloads every current advancement tracker from disk, broadcasts tags, sends each current player
recipes then the initial recipe book, and finally updates functions, structures and fuel. It does
not resend command trees and has no rollback after the swap.

Serialization at that server task does not provide a generation fence across an entire join or
reconfiguration session:

- an individual server-thread respawn or ready placement call is wholly before or after reload
  publication; reload cannot interleave inside that call;
- reconfiguration first removes/saves the player from world/global membership and then changes
  protocol state, so reload `saveAll` excludes that player while it is in configuration;
- `startConfiguration` stores the known-pack list and layered registry access for the registry
  synchronization task, but its response is handled by a later server task that resolves and
  serializes the applicable composite/tag view;
- spawn preparation and ready placement are later tasks and perform separate player-data reads;
  ready placement uses the then-current server registry access.

A reload can therefore publish between those tasks. Each individual task observes the state named
by its own captured or live reads; source does not support the current claim that the complete
join/reconfiguration flow uses one snapshot.

## Exact proposed shared replacements

The following text is for the integration coordinator. This worker did not edit the shared files.

### `cross-system-ordering.md`

Replace the complete `JOIN-27` row with:

```markdown
| `JOIN-27` | PlayerLifecycle × WorldLifecycle | Initial configuration spawn, respawn and cross-level travel are distinct membership transactions. Configuration spawn searches under radius-0 `SPAWN_SEARCH`, maintains radius-3 `PLAYER_SPAWN` and waits for entities; a portal transition owns its documented final-position radius-3 ticket. Respawn and direct cross-level teleport have no equivalent chunk-readiness wait, and always-ticking player insertion immediately starts destination tracking/ticking. Cross-level teleport sends respawn/difficulty/permission before source removal, then changes level, sends position and inserts; respawn removes the old object before projecting and inserting its replacement. Neither path locally rolls back a visible or membership prefix. Vector: gate configuration search/load/entity readiness and inject failure at every source removal, level/global/UUID/entity-section insertion and tracking boundary for direct, portal and respawn routes. |
```

Replace the complete `JOIN-28` row with:

```markdown
| `JOIN-28` | PlayerLifecycle × PersistenceReload | Disconnect attempts primary player data, stats and advancements in that order before level/global removal, but they are not one atomic commit: primary safe-replace failure is caught and later saves continue; stats and advancements overwrite directly, their I/O failures are caught, and advancement encoding may escape before its write catch. Crash may expose a partial direct-overwrite file, not merely the last completed file set. Reconnect constructs a fresh player and rebinds UUID state; configuration reads player data for preparation and again for ready placement, while respawn transfers only mapped fields and never resumes old transport/menu/callback objects. Vector: inject primary encode/write/replace, stats encode/write and advancement encode/write failures, then reconnect/reconfigure around death and reload. |
```

Replace the complete `JOIN-29` row with:

```markdown
| `JOIN-29` | PlayerLifecycle × ClientProjection | First projection is flow-specific. Fresh join sends its login-through-existing-player-info prefix before global insertion, then self player-info and level info before level insertion. Respawn sends replacement state through level info/permission before destination insertion. Cross-level teleport sends respawn/difficulty/permission before source removal, position before destination insertion, then abilities/level info/player info/effects. Insertion may synchronously send chunk-cache center and mark chunks pending, while terrain payload waits for ready-to-send/player-chunk-sender work. Replacement resets mapped sent mirrors; no path rolls back an emitted prefix. Vector: capture each flow with failure gates at every packet, membership, cache-center, pending-mark and first-terrain boundary. |
```

Replace the complete `JOIN-30` row with:

```markdown
| `JOIN-30` | PlayerLifecycle × DataReload | Reload publication is one server-executor task: after resource swap and recipe finalization it saves current global players, reloads their advancement trackers from disk, broadcasts tags, sends recipes then recipe books, and does not resend command trees; there is no rollback after swap. An individual server-thread respawn or ready placement call is wholly before or after publication, but this serialization does not fence a whole configuration session. Reconfiguration removes/saves the player before configuration; start-configuration captures known packs and layered registry access, registry response later resolves/serializes its applicable composite/tag view, and spawn preparation/ready placement perform separate data/registry reads. A reload may publish between those tasks, so reproduce each captured/live read rather than assuming one session snapshot. Vector: publish reload before/after every configuration task, registry response and both spawn-data reads, including save/advancement failure and disconnect. |
```

The `cross-system-joins.toml` records need no status or implementation-disposition change. If the
schema duplicates prose reproduction, replace only the reproduction of each exact family with the
corresponding vector above; do not mark any family implementation-Verified.

### `behavior-surfaces.toml` — `SURFACE-WORLD-LIFECYCLE-001`

Replace `boundary` with:

```toml
boundary = "World and dimension bootstrap, player destination preparation/membership, chunk generation/load/activity/unload, border and shutdown."
```

Replace the third `triggers` entry with these two entries:

```toml
  "initial join spawn preparation, respawn, direct dimension transfer and portal transfer",
  "border update, save, reload publication and server shutdown",
```

Replace the third `selectors` entry with these two entries:

```toml
  "configuration spawn tickets/readiness and direct, respawn and portal destination insertion",
  "dimension, level/global/UUID/entity-section/tracking membership, first projection, border, save, unload, reload-publication and shutdown boundaries",
```

Add this `state_domains` entry immediately after dimension registry/level-global state:

```toml
  "configuration spawn-search/player-spawn tickets, current level, level/global/UUID/entity-section/tracking memberships and client projection prefix",
```

Replace the surface reproduction paragraph with:

```toml
reproduction = [
  """
Follow one demanded chunk through storage, generation, activity, tracking, demotion, save and unload,
and trace world bootstrap, border and shutdown as documented by world-lifecycle-roots.md. Separately
drive initial configuration through SPAWN_SEARCH, PLAYER_SPAWN and entity readiness, then replay
fresh join, respawn, direct cross-level teleport and portal-prepared teleport. Inject completion,
failure, cancellation and reload publication at every source removal, level/global/UUID/entity-
section insertion, tracking, cache-center, pending-terrain and first-payload boundary. Distinguish
captured from live reads, assert the exact packet/membership prefix and preserve every documented
no-local-rollback boundary; never infer a universal chunk gate or session-wide reload snapshot.
""",
]
```

No owner or implementation-disposition replacement is proposed. The surface remains `Mapped` as a
reference surface; this audit does not alter the separate PlayerLifecycle implementation status.

## Independent reproduction vectors

1. Gate `PrepareSpawnTask` at spawn search, chunk-load completion, entity readiness and ready spawn;
   disconnect/cancel or fail each future and compare tickets, both player-data reads, connection
   state and every membership index.
2. Run same-level respawn, cross-level respawn, direct cross-level teleport and portal-prepared
   teleport. Inject before/after every packet send, old global/source removal, level change,
   destination entity-section/tracking insertion and new global/UUID insertion. Assert that no
   unobserved rollback or common readiness gate is invented.
3. Capture protocol bytes through cache-center and the first terrain batch. Assert flow-specific
   ordering and distinguish synchronous cache control/pending marks from ready terrain payload.
4. Fail every primary/stats/advancement encode, open, write and replacement point; restart after
   each prefix and record intact, old, partial or malformed files and fresh UUID rebind behavior.
5. Pause reload before/after resource swap, player save, advancement reload, tag broadcast, recipe
   send and recipe-book send while a player is joining, respawning or in each configuration task.
   Record captured/live registry and data observations independently.

## Unresolved experiments

This audit introduced no new source ambiguity and did not run or close an experiment. The four
existing `SourceInconclusive` slices remain exactly unresolved:

- `SIM-SCHEDULED-TICKS-001` / `SIM-SCHEDULE-001` — `EXP-SIM-002`;
- `ENV-LIGHTING-001` / `ENV-LIGHT-001` — `EXP-ENV-004`;
- `PLY-BLOCK-BREAK-001` / `PLY-BREAK-001` — `EXP-PLY-003`;
- `WGEN-PIPELINE-EQUIVALENCE-001` / `WGEN-PIPELINE-001` — `EXP-WGEN-001` (with the existing
  conformance experiments retained and no same-seed identity inference).

## Verification

All commands used Azul Java 25 through
`MC_REF_JAVA=/Users/mikai/Library/Java/JavaVirtualMachines/azul-25/Contents/Home/bin/java`; commands
that inspect bytecode also used the sibling `MC_REF_JAVAP` binary.

- `mc-ref coverage` — passed: 9,078 locked IDs, zero unclassified/ambiguous and zero explicitly
  unreviewed.
- `mc-ref readiness` — passed: 331 slices, 327 `SourceSpecified`, exactly four
  `SourceInconclusive`, zero todo/in-progress and 352 leaf rules; command-root, join and behavior-
  surface readiness also passed.
- `mc-ref surface coverage`, `surface readiness` and `surface verify` — passed: 10 root surfaces,
  36 unordered joins and 92 command roots in 12 recoverable families, all mapped.
- `mc-ref protocol inventory`, `protocol coverage`, `protocol readiness` and `protocol verify` —
  passed: protocol 776 inventory digest `f34b0956b6399c749d4638cd6d3c9226685f41fa`, 256 packets in 58
  families and the runtime packet catalog verified.
- `mc-ref verify --offline` — passed: 417 documentation IDs including 352 leaves, 331 completion
  slices, 2,802 symbol locators across 954 classes, 9,078 locked IDs, 307 experiment definitions,
  protocol/surface/join coverage and implementation manifest SHA-256
  `5e9704478eed9404c1b8fed2e9d4c6f6735423f82a70bedee1dc5112c4ea65b6`.
- `git diff --check` — passed.

Rust format, Clippy and crate tests were not run because this worker changes reference
documentation only, as exempted by `AGENTS.md`.
