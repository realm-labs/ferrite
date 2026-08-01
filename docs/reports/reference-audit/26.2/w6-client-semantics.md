# Minecraft Java 26.2 Reference Audit — Wave 1, Worker 6: Client Semantics

## Result

The source-backed audit completed for the scope below. Its findings update reference documentation
only and do not change Ferrite implementation dispositions.

## Scope and evidence

- Audited baseline: `feba9fac70272c8eaa4a87ea10aacb430b34294b`
- Assigned rules: `CLI-PREDICT-001`, `CLI-UI-001`, `CLI-EFFECT-001`, `CLI-PLAYER-RULE-001`, and
  `CLI-COMMAND-FEEDBACK-001`
- Assigned unresolved experiments: `EXP-PLY-003` and `EXP-ENV-004`
- Sources: repository-locked official Minecraft Java 26.2 client/server jars, generated reports,
  repository documentation, and `mc-ref` only

The locked client jar SHA-1 is `2dc72797acbc1b63fc16a11c4ac393605f453754`. The repository-locked
server distribution and its internal version jar were used without modifying the extracted reference
tree. Generated reports were refreshed with the repository-required Java 25 runtime before registry
queries.

## Findings

### `CLI-PREDICT-001`

- Entry points and handoffs: client callback/tick/frame admission, `MultiPlayerGameMode` prediction,
  client block retention/ACK handling, server player-action/use handlers, connection watermark tick,
  movement publication, teleport acceptance, and correction were followed end to end.
- Constants and data: signed prediction sequence, movement epsilon/heartbeat, input delays, and
  packet booleans remain source-specified. No registry input or RNG participates.
- Branch and ordering correction: break processing advances the ACK watermark after authoritative
  block-break handling, whereas use-on and air-use advance it before authoritative validation and
  mutation. The connection coalesces the greatest nonnegative sequence into at most one cumulative
  ACK per connection tick.
- Abort correction: a negative request sequence throws `IllegalArgumentException`. Client signed-int
  wrap therefore does not establish a wrap-aware ACK epoch.
- Persistence/reload: prediction retention and ACK watermarks are connection/runtime state; TCP
  ordering and later authoritative updates remain the convergence mechanism.
- Reproduction: test coalescing, action/use order, first negative sequence rejection, same-position
  retention, teleport gating, all movement packet forms, and correction-relative flags.

### `CLI-UI-001`

- Entry points and handoffs: screen press/drag/release/key translation, local menu replay, semantic
  click packet construction, dedicated controls, clientbound
  slot/content/cursor/inventory/data/close handlers, and all four dialog control codecs were
  reviewed.
- Constants and data: hover bounds, outside slot, hotbar/offhand buttons, quick-craft masks, checked
  packet narrowing, widget defaults, and registry input remain source-specified. No RNG
  participates.
- Ordering correction: quick-craft add order is set iteration, not a separate stable menu order.
- Identity correction: a wrong nonzero slot-update container ID cannot write the new open menu, but
  an open creative inventory screen still runs the remote-slot mirror/broadcast postlude. Dedicated
  cursor overwrite is suppressed while that creative screen is open; the player-inventory packet
  remains a separate addressed update.
- Persistence/reload: gestures and local predictions belong to the screen/menu instance; server
  packets remain authoritative and close abandons in-progress gesture state.
- Reproduction: include creative/noncreative wrong-ID slot packets and cursor packets in addition to
  the full gesture, delay, close, dedicated-control, and dialog matrices.

### `CLI-EFFECT-001`

- Entry points and handoffs: server audience computation, packet payloads, client packet dispatch,
  sound resolution, particle creation/filtering, entity/damage/level events, and gameplay-leaf
  effect ordering were audited.
- Constants and data: particle range is measured from the center of the recipient's integer block
  position; strict server radii, client distance, sound attenuation/gain, and option probabilities
  remain source-specified.
- RNG correction: positive packet count consumes six Gaussians per attempted particle from the
  packet listener's thread-safe RNG. Particle-option decisions and seedless local-sound `nextLong`
  use the distinct client-level RNG. Count zero consumes no distribution Gaussians but can still
  consume option draws before override filtering.
- Branch and ordering: fixed packet sound seeds do not consume a seed-creation draw; override still
  occurs after option sampling; exceptions preserve the documented single versus loop aborts.
- Persistence/reload: presentation channels and particles are client runtime state; authoritative
  gameplay persistence belongs to the concrete emitting leaf.
- Reproduction: instrument the two RNG cursors independently while covering all audience, option,
  resource, missing-entity, and special/default event branches.

### `CLI-PLAYER-RULE-001`

- Entry points and handoffs: play-login snapshots, rule-change notification/callback, client flags,
  combat-kill presentation, ordinary respawn request, and waypoint manager lifecycle were reviewed.
- Constants and data: the three game-rule IDs and protocol IDs were confirmed through generated
  registry reports; join inversion, event values/bytes, and defaults remain source-specified. No RNG
  participates.
- Ordering correction: new waypoint connections are stored before `connect`; broken replacements
  call `connect` before table replacement. Absent creation removes before old disconnect, while
  absent broken update disconnects before removal. Player removal and break-all orders are recorded
  separately; set traversal order remains unspecified.
- Persistence/reload: the game rules persist normally; local flags are resnapshotted/copied at the
  stated boundaries; waypoint connection objects are runtime state rebuilt from current eligibility.
- Reproduction: verify notification-before-projection, repeated kill packets, respawn authority, all
  per-entry connection mutation/send orders, clear, reload, and re-enable rebuild.

### `CLI-COMMAND-FEEDBACK-001`

- Entry points and handoffs: generic success/failure, player, command block, gamemode, server,
  integrated/dedicated console, RCON, placement, operator fan-out, and logging routes were audited.
- Constants and data: the three live game-rule IDs/protocol IDs were confirmed through generated
  reports. Formatting, property defaults, timestamp source, and no-RNG disposition remain exact.
- Branch and ordering clarification: both success gates are computed before one lazy supplier
  evaluation; direct delivery precedes admin routing. Admin component construction precedes the live
  feedback read and ordered OP traversal; the independent live log-rule read/server log follows that
  traversal.
- Persistence/reload: game-rule and command-block carrier persistence retain their existing owners.
  RCON buffers, silence, source identity, and delivery decisions are runtime state and are not
  replayed.
- Reproduction: cover the complete rule/source matrix and assert direct-before-admin,
  OP-traversal-before-log, exact source exclusion, and reload non-replay.

## Unresolved items

- `EXP-PLY-003` remains planned and `SourceInconclusive`. Source fixes the logical handler and
  packet order, including cumulative ACK before the later authoritative air update for the owning
  break flow, but cannot determine whether scheduler/network/render batching exposes the retained
  restored state in any rendered frame. The experiment must capture packet handling and extracted
  frames without reordering TCP bytes.
- `EXP-ENV-004` remains planned and `SourceInconclusive`. Source fixes propagation, publication, and
  client-import order but cannot provide a universal server-tick, wall-time, or rendered-frame bound
  under arbitrary executor, network, chunk-dispatcher, and renderer load. Results must stay scoped
  to named load profiles.

Neither experiment was promoted, guessed, or used to mark an implementation disposition Verified.

## Evidence and verification

- `cargo run -p mc-reference --bin mc-ref -- symbols` — passed; 2,789 locators across 952 classes.
- `cargo run -p mc-reference --bin mc-ref -- coverage` — passed; 9,078 locked IDs, zero unclassified
  or ambiguous.
- `cargo run -p mc-reference --bin mc-ref -- readiness` — passed; 331 slices, including the existing
  four `SourceInconclusive` slices, and zero unreviewed catalog IDs.
- `cargo run -p mc-reference --bin mc-ref -- experiment verify` — passed; 307 definitions.
- `cargo run -p mc-reference --bin mc-ref -- verify --offline` — passed; documentation, completion,
  symbols, coverage, experiments, all 256 protocol packets, behavior surfaces, joins, and the
  unchanged implementation manifest verified offline.
- `git diff --check` — passed.

This is documentation-only work, so the Rust `cargo fmt` and Clippy checks required for Rust code
changes were not applicable.
