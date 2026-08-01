# Player-Service Region Integration (Historical Goal 01 Phase 6)

`G01-P6-B1` integrates the audited player and item runtimes with Region authority, persistence,
action replay fencing, menu resynchronization, and per-player client projection.

## Ownership boundary

The active `ferrite-server-runtime::player_service` module contains three responsibility-specific
modules:

- `model` defines project-owned persistent player state, bounded canonical inventory/progression
  payloads, action headers, transient menu leases, and semantic projection events;
- `continuity` encodes one versioned `ferrite:phase6/player_v1` entity record per stable player and
  validates all bounds and gameplay scalar invariants on restore;
- `runtime` owns the Region generation, stable-player map, live session epochs, action sequences,
  inventory revisions, transient menus, and a bounded projection queue for each player.

Packet structs remain in `ferrite-protocol`; gameplay algorithms remain in `ferrite-gameplay`.
Connection adapters lower admitted packets into `PlayerActionHeader` plus a canonical mutation. The
owning Region alone validates and commits that mutation.

## Action admission and atomicity

Every action is fenced by exact Region key, activation generation, stable player identity, fresh
session epoch, and contiguous per-player action sequence. Wrong Region, stale generation, stale
session, unknown player, and sequence gaps fail before state or projection changes. A sequence that
was already committed returns `AlreadyApplied`.

Mutation first compares its expected inventory revision. A mismatch advances the admitted action
sequence and queues a full authoritative resynchronization without changing inventory or
progression. Valid candidate payloads and scalar fields are fully checked before reserving a
projection revision. Projection capacity is also preflighted before commit, so malformed input or
backpressure cannot partially advance player state, action sequence, inventory revision, menu state,
or projection revision.

The canonical payload boundary preserves the actual project-owned inventory and progression bytes,
not only a hash. Each payload is capped at 1 MiB and carries a BLAKE3 digest for deterministic
identity and diagnostics.

## Menu convergence

An open menu is a session-local `(container_id,state_id)` lease. A different container ID is ignored
after recording the admitted action sequence. For the current container, an old state ID does not
discard the click: the Region commits the replay, advances menu and inventory revisions, then queues
a full snapshot. A current state ID queues a delta. Inventory-revision mismatch instead rejects the
mutation and queues full state.

Menu leases, remote mirrors, callbacks, pending item use, and transport state are intentionally
transient. They are neither serialized nor transferred into a replacement session.

## Persistence and reconnect

Continuity records are emitted in stable player-ID order through the repository snapshot format.
They persist canonical inventory and progression payloads, selected slot, experience, hunger,
inventory revision, last committed action sequence, and the last issued session epoch.

Restore validates every record, installs a new activation generation, increments each session
epoch, leaves menus closed, and queues one full `Reload` projection. The new connection cannot reuse
old acknowledgements or menu state. The persisted action sequence prevents a delayed pre-reload
action from committing twice.

## Multiplayer isolation and bounds

Player state and projection queues are keyed independently by `StableEntityId`. A full queue for one
player rejects only that player's next mutation and does not consume capacity for another player.
Region player count and every per-player projection queue are configured with nonzero hard limits.
Projection drains name one stable player and cannot expose another player's state.

## Validation

`crates/ferrite-server-runtime/tests/player_service_region_integration.rs` verifies:

- Region/generation/session/player fencing before mutation;
- contiguous action replay and inventory-revision full resync;
- stale-menu commit-then-full-sync and wrong-container ignore;
- per-player queue atomicity and multiplayer isolation;
- invalid-field rejection without revision consumption;
- canonical payload save/restore with menu and transport reset;
- stable continuity ordering and bounded field validation.

This filename is retained because completed Goal 01 ledgers link to it. The active module, type,
diagnostic, and test-target names are responsibility-owned. The legacy
`ferrite:phase6/player_v1` continuity identity remains byte-stable until the dedicated Goal 03
migration batch.
