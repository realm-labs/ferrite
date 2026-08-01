# G03-P1-B3 Continuity Identity Migration

## Outcome

All active continuity writers now emit responsibility-owned versioned identities:

| Responsibility | Current identities | Read-only legacy identities |
|---|---|---|
| Simulation | `ferrite:simulation/runtime_v1`, `scheduled_block_v1`, `scheduled_fluid_v1`, `boundary_receipt_v1` | matching `ferrite:phase5/*_v1` identities |
| Player service | `ferrite:player-service/player_v1` | `ferrite:phase6/player_v1` |
| Entity service | `ferrite:entity-service/entity_v1`, `applied_transfer_v1` | matching `ferrite:phase7/*_v1` identities |
| World service | `ferrite:world-service/chunk_v1`, `level_v1` | matching `ferrite:phase8/*_v1` identities |

The server-runtime continuity boundary owns the fixed identity table, expected record kinds,
generation classification, canonical record hash, in-memory normalization, and durable-store
migration. Service restore entry points normalize valid legacy records before decoding. New saves
and the guarded store commit path write only current identities.

## Migration safety

Migration validates the complete snapshot and journal-tail generation before transforming any
record. It rejects legacy/current mixtures, unknown versions under reserved responsibility paths,
wrong record kinds, duplicate canonical identities, and invalid snapshot hashes. Keys, values, and
record kinds remain byte-for-byte unchanged; only the domain identity changes. Because the domain
participates in the canonical hash, the target snapshot explicitly recomputes its state hash.

Durable migration is a prepare/commit operation. Preparation reads and digests the selected legacy
commit, creates a current-identity candidate at the next persistence revision, and does not touch
the store. Commit reloads and verifies the source digest before using the append-and-repoint store
transaction. Dropping an interrupted plan leaves the prior commit selected. Re-running on current
state is idempotent, and guarded writes reject rollback to a legacy identity.

`world-inspector` understands legacy and current world chunk identities, emits
`continuity_generation` as `legacy`, `current`, or `none`, and fails closed on invalid mixtures or
unsupported world-service identity versions.

## Verification

- `cargo test -p ferrite-server-runtime --test continuity_migration --all-features`: passed; six
  clean-old, clean-new, interrupted, mixed/unsupported, duplicate, and rollback-denied tests.
- `cargo test -p ferrite-server-runtime --all-features`: passed.
- `cargo test -p world-inspector --all-features`: passed; two dual-generation compatibility tests
  plus the stable usage-contract test.
- `cargo fmt --all -- --check`: passed.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`: passed.
- `cargo test --workspace --all-features`: passed.
- `cargo ferrite source verify`: passed.
- `git diff --check`: passed.
