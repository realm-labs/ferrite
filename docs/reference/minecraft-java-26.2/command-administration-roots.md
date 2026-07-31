# Command Administration Root Inventory

**Surface:** `SURFACE-COMMAND-ADMINISTRATION-001`
**Status:** `Mapped`
**Primary evidence:** `OFF-REPORT-001`; `OFF-SERVER-001`

This inventory maps the complete locked command grammar to semantic owners. The authoritative
`commands.json` has 92 root children, 1,290 executable paths and 110 redirects. Their sorted
newline-delimited SHA-1 locks are respectively `14bbe081980917f9ce81f5a6dc877823963f0e59`,
`b56959908e8b8ce2996d6a0d8ade26ee8efbe071` and
`3f9020eff0134824dcb4b2d9385ff62784f7a93f`.

Every executable node inherits the contract of its first path segment in the matrix below.
Argument/literal descendants only select a handler branch, target, mode, optional value or default;
they cannot escape that root's owner set. The machine verifier derives and locks every executable
and redirect path, so an added terminal, moved alias or changed root fails coverage.

## Dispatcher contract

- The dispatcher registers the ordinary roots first, conditionally adds `jfr` when the JVM
  profiler is available, adds dedicated-only and integrated-only roots, then installs the result
  consumer. Debug-development roots are absent from the locked 92-root production report.
- Permission predicates run during Brigadier traversal and command-tree projection. `LEVEL_ALL`
  roots are `help`, `list`, `me`, `msg`/`tell`/`w`, `teammsg`/`tm` and `trigger`. `seed` and
  `version` are all-user in integrated selection and gamemaster-only otherwise. `gamemode` uses
  `Permissions.COMMANDS_GAMEMASTER`. The remaining ordinary roots use the named gamemaster, admin
  or owner groups below; an unusable node is neither executable nor sent in that source's tree.
- Dedicated selection alone registers `ban`, `ban-ip`, `banlist`, `deop`, `op`, `pardon`,
  `pardon-ip`, `perf`, `save-all`, `save-off`, `save-on`, `setidletimeout`, `stop`, `transfer` and
  `whitelist`. Integrated selection alone registers `publish` and `unpublish`. `kick` is registered
  in both but rejects an unpublished single-player server and never kicks its owner.
- Parsing strips at most one leading slash. A syntax failure sends its raw red failure, then a
  gray/red clickable context line when input/cursor exist, and queues no execution. An unexpected
  top-level exception sends `command.failed`; debug modes may add detail. Ordinary execution errors
  reach the current source, while already-forked errors are tracer-only.
- Each outer execution snapshots `max_command_sequence_length` and `max_command_forks`, drains one
  synchronous execution context and clears it in `finally`; nested commands reuse it.
  `SIM-COMMAND-LIMIT-001` fixes every debit, fork, queue, exhaustion and return interaction.
- The dispatcher result consumer invokes the terminal source callback with `(success, result)`.
  A parse failure never reaches it; an execution-stage Brigadier failure reports failure/zero,
  while an unexpected outer exception has only the generic failure route. Each handler's integer
  below remains its command result. Redirected sources retain or compose callbacks as specified by
  `execute`, `return` and function execution. Command effects completed before a later
  target/branch failure are not rolled back unless the named semantic owner explicitly preflights
  a transaction.
- `CLI-COMMAND-FEEDBACK-001` fixes lazy success, red failure, silent stacks, direct source output,
  operator fan-out, server logging and command-block/RCON/console behavior. A handler's feedback is
  emitted at its source location in program order; result callbacks do not undo it.
- `CommandSourceStack.sendSuccess` evaluates its supplier once when at least one route accepts it,
  sends to the direct source first, then visits the current player list in order for eligible
  operators, then writes the server log. `sendFailure` is direct-only. Target-specific packets and
  handler mutations can precede either call and remain visible if a later route or target fails.

## Permission groups

| Permission | Locked roots |
|---|---|
| Gamemaster | `advancement`, `attribute`, `bossbar`, `clear`, `clone`, `damage`, `data`, `datapack`, `defaultgamemode`, `dialog`, `difficulty`, `effect`, `enchant`, `execute`, `experience`/`xp`, `fetchprofile`, `fill`, `fillbiome`, `forceload`, `function`, `gamemode`, `gamerule`, `give`, `item`, `kill`, `locate`, `loot`, `particle`, `place`, `playsound`, `random`, `recipe`, `reload`, `return`, `ride`, `rotate`, `say`, `schedule`, `scoreboard`, `setblock`, `setworldspawn`, `spawnpoint`, `spectate`, `spreadplayers`, `stopsound`, `stopwatch`, `summon`, `swing`, `tag`, `team`, `teleport`/`tp`, `tellraw`, `test`, `time`, `title`, `waypoint`, `weather`, `worldborder`; conditionally `seed`, `version` |
| Admin | `ban`, `ban-ip`, `banlist`, `debug`, `deop`, `kick`, `op`, `pardon`, `pardon-ip`, `setidletimeout`, `tick`, `transfer`, `whitelist` |
| Owner | `jfr`, `perf`, `publish`, `save-all`, `save-off`, `save-on`, `stop`, `unpublish` |

## Executable terminal map

Counts are executable terminals beneath the root; `r` is a redirect node. Query branches named
below are proven read-only with respect to authoritative gameplay state, although feedback,
profiling, profile-cache lookup or RNG consumption can still be observable where stated.

| Family | Locked roots and terminal counts | Terminal classification and exact owners |
|---|---|---|
| Delegated execution | `execute` 40 + 105r; `function` 8; `return` 2; `schedule` 4 | `ExecuteCommand` modifiers change source position/rotation/dimension/entity/anchor, conditionally fork, summon, or store downstream success/result in score, boss bar or numeric data; all 105 redirects return to `execute`. `function` invokes one function/tag with optional arguments; `return value/fail/run` terminates the current frame with exact callback propagation; `schedule function ... append/replace` and `schedule clear` mutate the persistent scheduled-event identity. `SIM-COMMAND-LIMIT-001`, `SIM-001`, `BLK-COMMAND-001` and function/test owners fix queue, nesting, partial completion and carrier effects. |
| Server access and lifecycle | `ban` 2; `ban-ip` 2; `banlist` 3; `deop` 1; `fetchprofile` 3; `kick` 2; `list` 2; `op` 1; `pardon` 1; `pardon-ip` 1; `publish` 4; `setidletimeout` 1; `stop` 1; `transfer` 3; `unpublish` 1; `whitelist` 6 | Ban/pardon, OP and whitelist terminals mutate their exact access list, persist it through its list implementation and return changed-entry count; list/show terminals return displayed cardinality without gameplay mutation. Ban/kick disconnect admitted online targets after the list/session decision; transfer sends one transfer packet per selected player and returns that count without disconnecting locally. `fetchprofile` is an asynchronous profile/cache query. Publish/unpublish change the integrated listener; timeout and stop change server lifecycle state. `SIM-001`, `PLY-001` and the mapped World/Player lifecycle surfaces own session, save and stop ordering. |
| Persistence, reload and generic data | `data` 300; `datapack` 10; `reload` 1; `save-all` 2; `save-off` 1; `save-on` 1 | All 300 compositional `data` terminals are the cross-product of block/entity/storage target, `get/merge/remove/modify`, source target/path and insert/prepend/append/set/merge/string modes. `get` only reads and returns the handler's scaled/truncated result; mutation validates target/path/value through `BLK-007` or `ENT-001`, commits that target and then reports. `datapack list` is read-only. Create writes a directory and `pack.mcmeta` without enabling or reloading it; an I/O failure can leave a filesystem prefix. Enable/disable report before candidate reload but do not install their temporary selected list on candidate failure. `/reload` first refreshes and rebuilds `PackRepository` selection, reports admission, then reloads resources. On the server thread `reloadResources` drives its future with `managedBlock`, so the handler returns zero only after publication or failure feedback, not before asynchronous completion. Save roots use the ordering below. PersistenceReload and DataReload fix every commit/failure boundary. |
| World, blocks and generation | `clone` 104; `fill` 12; `fillbiome` 2; `forceload` 7; `locate` 3; `place` 13; `setblock` 5; `setworldspawn` 3; `spawnpoint` 4; `worldborder` 10 | `BLK-COMMAND-AREA-001` exhaustively owns every clone dimension/filter/normal-force-move/strict combination, fill replace/keep/outline/hollow/destroy/strict combination and fill-biome filter branch, including volume precharge, traversal, partial writes, ticks and biome resend. Chunk admission is command-specific: `fill` checks only its endpoints and can load interior chunks while traversing, whereas `clone` checks the complete source and destination boxes before writing. `forceload query` and `locate` are read-only; add/remove/all mutate tickets one chunk at a time without requiring the chunk already loaded. `place` invokes feature, jigsaw, structure or template generation under `WGEN-003` with the distinct checks below; `setblock` uses destroy/keep/replace/strict state writes under `BLK-003`, and destroy can remain committed when replacement fails. Spawn roots write global or per-player spawn state. `worldborder get` is read-only; all setters use `WGEN-BORDER-001`, including interpolation and projection. |
| Clock, environment and rules | `difficulty` 5; `gamerule` 236; `random` 12; `tick` 9; `time` 19; `weather` 6 | Difficulty's root query returns current ID and its four literals set the server value. Each of 59 registered rules has both bare and `minecraft:`-qualified literal paths, and each path has one query plus one typed setter, producing all 236 terminals; callbacks and persistence remain with that rule's semantic owner. Random `value/roll` always advances the named/default random sequence; optional score storage additionally mutates selected scores, so it is not read-only. Tick query is read-only; rate/freeze/step/sprint branches use `SIM-006`. Time query is read-only; set/add and named-clock branches use the active dimension clock. Weather sets clear/rain/thunder and duration. `ENV-004`, `SIM-006` and command feedback own scope, bounds and projection. |
| Players and entities | `attribute` 12; `damage` 5; `defaultgamemode` 1; `effect` 10; `gamemode` 2; `kill` 2; `ride` 2; `rotate` 4; `spectate` 3; `spreadplayers` 2; `summon` 3; `teleport` 8; `tp` 1r | Attribute get/value-get terminals are read-only numeric queries; base/modifier add/remove/reset mutate the entity attribute map. Damage, effect give/clear and kill use the exact damage/effect/removal owners. Selected targets are resolved to object references before the terminal handler loops; they are not re-resolved between targets. Default/player gamemode updates policy or admitted targets; player mode mutates the player, sends that target's special message when enabled, then reports to the source/admin routes. Ride validates passenger/vehicle policy; rotate and teleport use entity/player correction and dimension admission; spectate changes camera target; spread iterates accepted entities after placement search; summon creates, loads optional components and finalizes one entity. `tp` redirects exactly to `teleport`. `PLY-001`, `ENT-001`, `ENT-005`, `ENT-006` and `ENT-008` own state and partial per-target results. |
| Items and progression | `advancement` 12; `clear` 4; `enchant` 2; `experience` 8; `xp` 1r; `give` 2; `item` 14; `loot` 70; `recipe` 4 | Advancement grant/revoke maps only/from/until/through/everything and optional criterion over targets. Clear count-only query is read-only; removal branches mutate inventories. Enchant applies the selected enchantment to admitted held stacks. Experience add/set/query uses points or levels; `xp` redirects to it. Give inserts then drops overflow under inventory ownership. Item replace/modify operates block/entity slots with exact component semantics. Loot's give/insert/replace/spawn targets share table/fish/kill/mine sources and `ITM-006`/`ITM-007` contexts. Recipe give/take mutates recipe books. Each handler returns its affected count or query value; committed earlier targets survive later failure. |
| Score, social and waypoint state | `bossbar` 25; `scoreboard` 30; `tag` 3; `team` 28; `trigger` 3; `waypoint` 6 | Boss-bar list/get and scoreboard objectives/players display/list/get branches are read-only; their add/remove/set/reset/operation/display/option branches mutate the respective manager and synchronously fan out listener packets. Tag list is read-only; add/remove mutates persistent entity tags within the fixed cap. Team list is read-only; join/leave/modify/add/remove updates scoreboard membership/options. Trigger enable is administrative scoreboard state, while all-user add/set requires an enabled trigger then locks it. Waypoint list is read-only; color/style set/reset updates the entity waypoint and tracking projection. `PLY-001`, `CLI-006` and feedback own persistence and recipients. |
| Messaging and text presentation | `dialog` 2; `me` 1; `msg` 1; `tell` 1r; `say` 1; `teammsg` 1; `tm` 1r; `tellraw` 1; `title` 6; `w` 1r | Dialog clear/show, tellraw, and title clear/reset/title/subtitle/actionbar/times resolve components per selected player and send their dedicated packets without durable world mutation. `me`, `msg`, `say` and `teammsg` route signed/unsigned, filtered and attributed messages through the locked chat protocol and visibility rules. `tell`/`w` redirect to `msg`; `tm` redirects to `teammsg`. Results are recipient counts where handlers return them; delivery already completed before any later recipient failure remains delivered. `CLI-006` and `CLI-COMMAND-FEEDBACK-001` own presentation. |
| Ephemeral world presentation | `particle` 7; `playsound` 67; `stopsound` 24; `swing` 4 | Particle terminals select normal/force viewers, optional delta/speed/count and return the actual receiving-player count, failing when none. Every sound-category/default-position/volume/pitch/min-volume branch sends the play packet to distance-admitted targets and returns count; stop-sound's source/category cross-product sends its stop packet. Swing selects hand and whether to broadcast. These roots mutate no durable world state; packet emission, recipient order, seeds, range and empty-selection failure are owned by `CLI-006`. |
| Informational | `help` 2; `seed` 1; `version` 1 | All four terminals are read-only. Help lists the current source's usable dispatcher nodes or one parsed subtree and returns displayed usage count. Seed returns the current level seed subject to selection-dependent permission. Version emits the locked server version text and returns one. Only feedback/log projection occurs. |
| Diagnostics and tests | `debug` 3; `jfr` 2; `perf` 2; `stopwatch` 5; `test` 41 | Debug start/stop/function, JFR start/stop and dedicated perf start/stop own profiler/file lifecycle and diagnostics only; failures do not mutate gameplay. Stopwatch query tests elapsed ranges without mutation, while create/restart/pause/reset change named server stopwatch state used by `execute if/unless stopwatch`. GameTest run/retry/verify/locate/reset/clear/stop/position/create terminals use the test framework; locate/position are read-only, while the rest mutate test runners, structures or operator test records under `BLK-TEST-BLOCK-001` and `BLK-TEST-INSTANCE-001`. Tool output formats are evidence, not Ferrite runtime architecture requirements. |

## Cross-system command joins

- **Content and state (`JOIN-16`).** A parsed command is not a blanket snapshot of every later
  content lookup. Argument implementations may retain a key, holder or already-resolved object, and
  the terminal getter defines whether it consults the active registry or world. Each owner supplies
  its own validation and mutation boundary. For example, `setblock ... destroy` destroys first and
  can then fail to place the requested state; `fill` commits in traversal order. Neither the
  dispatcher nor the command root restores a prior prefix.
- **Targets and lifecycle (`JOIN-17`).** The selected target collection is fixed when the handler
  starts on the server thread, and its object references are traversed synchronously. A lifecycle
  task processed before that point changes selection; ordinary later server tasks cannot interleave
  with the loop. `kick` first rejects an unpublished server, then for each non-owner target requests
  disconnect before sending that target's success feedback; owner entries are skipped and an
  all-owner selection fails. Thus lifecycle and feedback are per-target, not one atomic batch.
- **Chunks and generation (`JOIN-18`).** `setblock` requires its position loaded. `fill` applies the
  loaded-position getter only to its two endpoints, while `clone` requires the complete source and
  destination boxes loaded before writes. Place-feature checks a 3-by-3 chunk window, place-jigsaw
  checks the origin chunk, place-structure constructs its candidate before checking the resulting
  bounding-box span, and place-template checks the origin-through-unrotated-template-size span.
  Forceload changes tickets in x-major/z-minor order without an existing-chunk requirement; later
  loading or generation is not part of that command's committed prefix and no generation fence is
  implied.
- **Save barriers (`JOIN-19`).** `save-all` emits `commands.save.saving` before attempting any save,
  then saves players before iterating levels and world data. Flush mode joins saved-data work and
  chunk flushes; non-flush mode can schedule saved-data writes. The final success occurs only after
  a true result, while failure preserves the initial feedback and every completed save prefix.
  `save-off` and `save-on` change each level's `noSave` flag in iteration order, while `save-all`
  explicitly saves levels despite that flag. None of these operations establishes cross-file crash
  atomicity or rollback.
- **Observable ordering (`JOIN-20`).** A handler owns the position of its feedback call relative to
  mutation and target projection. Success routing is direct source, current eligible operator list,
  then server log; failures are direct-only. `kick` requests disconnect before success routing,
  gamemode mutates and optionally informs its target before source success routing, and `save-all`
  has the pre-save message described above. A result callback observes the terminal outcome but
  cannot retract any earlier state, packet or feedback.
- **Reload and dispatcher publication (`JOIN-21`).** `/reload` synchronously refreshes pack discovery
  and rebuilds repository selection before its admission message and candidate load. Candidate
  failure retains the old live server resources and world-data configuration but does not undo
  those repository discovery/selection changes. Successful publication closes the old resources,
  swaps the live resources pointer (and therefore `getCommands()`), installs the selected packs and
  data configuration, applies tags/components/recipes and player reload, then replaces the function
  library and other derived managers. Function files were compiled against the candidate
  dispatcher, but the old function library remains installed for the earlier prefix of that one
  publication task. There is no ordinary server-task interleaving inside the prefix and no rollback
  if an injected later step fails. Existing play clients receive no ordinary reload command-tree
  resend; later command execution and later joins use the new dispatcher. Argument snapshot/live
  lookup remains argument-type-specific.

## Persistence and projection

Command execution has no separate transaction log. Persistent mutations dirty and save through the
same block, entity, player, scoreboard, rule, world, pack or scheduled-event owner as direct
gameplay. Command source, parse tree, execution frames, callbacks, result integers, feedback,
profile/profiler work and emitted packets are transient. Command blocks alone persist their carrier
command/result/track-output state through `BLK-COMMAND-001`.

World/player/entity mutations project through their normal update paths in handler order. Dedicated
presentation commands send packets directly. Scoreboard, team, boss-bar and waypoint managers own
listener fan-out. A command-thread resource reload internally uses asynchronous candidate work, but
the server drives that future to completion before returning from `reloadResources`; profile and
diagnostic futures instead retain only their explicitly registered later feedback or file/cache
effect.

## Reproduction matrix

1. Regenerate `commands.json`; require all three counts/digests and the exact one-family root
   partition to match before any semantic test runs.
2. For every root, execute every terminal path at its argument minima/maxima/defaults and with each
   selector cardinality; verify handler result, callback and feedback order against the matrix.
3. Sweep all/all-user, gamemaster, admin and owner sources plus dedicated/integrated/JFR-absent
   construction; compare executable admission and projected command trees.
4. Replay syntax, handler, per-target, asynchronous and injected late failures. Verify the
   documented preflight or retained partial prefix and that no generic rollback occurs.
5. Execute each mutation from player, console, RCON, function and command-block sources while
   sweeping silence and feedback rules; verify direct/admin/log/carrier routes.
6. Save/reload every durable result and reconnect affected clients; confirm ordinary owner
   persistence and reprojection, with no persisted parser, callback, execution-frame or packet
   state.
7. Place `fill` endpoints in loaded chunks around an unloaded interior, then compare `clone` over the
   same span; inject a later traversal failure and record chunk loads, write prefix, neighbor updates
   and command feedback. Repeat every `place` subtype at its distinct chunk-check boundary.
8. Run `forceload add` over multiple chunks with injected failure after one ticket and observe later
   chunk activity separately. Run `setblock ... destroy` with a replacement that fails and retain
   the destroyed state as the expected prefix.
9. Run `save-all` with failure injection in player save, successive level saves, saved-data join and
   world-data save; distinguish pre-feedback, durable prefixes, final feedback and post-restart
   visibility. Repeat `save-off`/`save-on` across multiple levels.
10. Delay and fail candidate reload after `/reload` discovery and after `datapack enable`; compare
    repository selection, live resources, world-data configuration, feedback and handler return
    timing. On successful reload, instrument each publication step and verify dispatcher/function
    library prefixes and the absence of an existing-client command-tree resend.
11. Select owner and non-owner players for `kick`, and multiple players for `gamemode`; record target
    mutation or disconnect, target message, direct source message, operator fan-out and server-log
    order under every feedback gamerule/source combination.
