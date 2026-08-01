# Goal 04 Status — Durable Generated World

This ledger is the resumable source of truth for
[Goal 04](04-durable-generated-world.md). Update it in every implementation batch.

## Goal state

| Field | Value |
|---|---|
| State | `InProgress` |
| Active batch | `G04-P4-B1` |
| Next unblocked batch | `G04-P4-B2` |
| Depends on | Goal 03 `Complete` |
| Goal plan | [Goal 04 plan](04-durable-generated-world.md) |
| Launch prompt | [Goal 04 prompt](04-durable-generated-world-prompt.md) |
| Blocker | None |

Allowed states are `Planned`, `Ready`, `InProgress`, `Blocked`, and `Complete`. Only one batch may
be `InProgress`.

## Batch ledger

| Batch | State | Depends on | Evidence | Result |
|---|---|---|---|---|
| `G04-P0-B1` | `Complete` | Goal 03 | [production contract](../development/durable-world-production.md), [batch report](../reports/goal-04/g04-p0-b1-world-production-truth.md), production manifest, `cargo ferrite production verify` | Eight world responsibilities, one authoritative representation, versioned formats, migration/failure rules, and terminal acceptance are frozen |
| `G04-P1-B1` | `Complete` | P0-B1 | [batch report](../reports/goal-04/g04-p1-b1-world-configuration-migration.md), `world_config`, CLI migration, Java 25 build, complete workspace gates | Schema 2 owns bounded world identity/generation/save policy; schema 1 migrates deterministically without overwrite or conflicting durable attachment |
| `G04-P1-B2` | `Complete` | P1-B1 | [batch report](../reports/goal-04/g04-p1-b2-durable-world-bootstrap.md), `world_v1` codec/store bootstrap, formal-route tests, complete workspace gates | Formal startup creates or validates contained durable metadata and routes the configured world identity and spawn |
| `G04-P1-B3` | `Complete` | P1-B2 | [batch report](../reports/goal-04/g04-p1-b3-formal-world-persistence.md), formal restart/corruption tests, bounded recovery selection, world inspector, complete workspace gates | Formal composite continuity autosaves, flushes before authority release, resumes the published control-Region checkpoint, and remains inspectable offline |
| `G04-P2-B1` | `Complete` | P1-B3 | [batch report](../reports/goal-04/g04-p2-b1-production-chunk-lifecycle.md), formal lifecycle and receipt tests, network/restart regressions, complete workspace gates | Player view/simulation tickets now drive bounded formal load, fenced generation, activation, save acknowledgement, and unload |
| `G04-P2-B2` | `Complete` | P2-B1 | [batch report](../reports/goal-04/g04-p2-b2-overworld-generation.md), generator determinism and formal lifecycle tests, complete workspace gates | The configured seed now drives biome, density terrain, surface, carver, feature, and spawn-preparation stages in authoritative chunks |
| `G04-P2-B3` | `Complete` | P2-B2 | [batch report](../reports/goal-04/g04-p2-b3-structures-and-continuation.md), structure/durable/restart tests, complete workspace gates | Versioned starts/references and placed structure blocks persist in FWC2; P8C2 resumes fenced generation after restart |
| `G04-P2-B4` | `Complete` | P2-B3 | [batch report](../reports/goal-04/g04-p2-b4-authoritative-chunk-projection.md), formal snapshot/projection and network regressions, complete workspace gates | Formal Java terrain batches now derive from committed FULL+accessible Region-owned columns; `MinimalTerrain` is fixture-only |
| `G04-P3-B1` | `Complete` | P2-B4 | [batch report](../reports/goal-04/g04-p3-b1-authoritative-voxel-collision.md), collision-scene and formal network regressions, complete workspace gates | Formal movement now clips, steps, falls, and corrects against bounded shapes captured from committed generated columns; missing authority fails closed |
| `G04-P3-B2` | `Complete` | P3-B1 | [batch report](../reports/goal-04/g04-p3-b2-authoritative-environment.md), environment/light durability and formal network tests, complete workspace gates | FWC3 light authority, FWL2 time/weather continuity, bounded scheduled/random fluid/fire work, relighting, and Java clock/weather/block projection now commit through the formal Region path |
| `G04-P3-B3` | `Complete` | P3-B2 | [batch report](../reports/goal-04/g04-p3-b3-border-spawn-exploration.md), spawn/border/ticket/restart tests, complete workspace gates | Formal readiness prepares a safe generated spawn area; durable border state drives Java projection and movement clipping; committed movement atomically recenters view and simulation tickets |
| `G04-P4-B1` | `InProgress` | P3-B3 | — | Activate durable dimensions |
| `G04-P4-B2` | `Pending` | P4-B1 | — | Integrate authoritative portal travel |
| `G04-P4-B3` | `Pending` | P4-B2 | — | Prove dimensional restart and fault continuity |
| `G04-P5-B1` | `Pending` | P4-B3 | — | Run exact-client world scenarios |
| `G04-P5-B2` | `Pending` | P5-B1 | — | Complete audits and completion evidence |

## Decisions and blockers

| Date | ID | State | Decision or blocker | Evidence / follow-up |
|---|---|---|---|---|
| 2026-08-01 | `G04-D001` | `Accepted` | One authoritative chunk representation serves simulation, collision, persistence, and projection. | Server gap audit |
| 2026-08-01 | `G04-D002` | `Accepted` | Goal 01 worldgen equivalence boundaries remain explicit; Goal 04 does not guess byte-identical Mojang output. | Goal 01 deferred experiment policy |
| 2026-08-01 | `G04-D003` | `Accepted` | `MinimalTerrain` remains test-only after formal production replacement. | Goal 04 scope |
| 2026-08-01 | `G04-D004` | `Accepted` | Ferrite recovery points are the production format; Mojang Anvil/NBT compatibility is an optional boundary adapter. | [Production contract](../development/durable-world-production.md) |
| 2026-08-01 | `G04-D005` | `Accepted` | Schema 1 migration uses former formal constants and fails on conflicting durable identity. | `G04-P0-B1` compatibility freeze |
| 2026-08-01 | `G04-D006` | `Accepted` | A synced receipt clears dirty state only for the exact captured revision; stale receipts cannot acknowledge newer authority. | Persistence recovery contract |
| 2026-08-01 | `G04-D007` | `Accepted` | Physical stores are contained and sharded by world, dimension, and `SimulationRegionKey`; active logs are never compacted in place. | `G04-P0-B1` storage layout |
| 2026-08-01 | `G04-D008` | `Accepted` | `world_v1` is current-only continuity: it has no synthetic `phase*` predecessor and cannot be written through the legacy identity path. | `G04-P1-B2` metadata codec and continuity classifier |
| 2026-08-01 | `G04-D009` | `Accepted` | The overworld control Region commits last and publishes the world checkpoint tick; valid unpublished Region successors are bounded recovery inputs, not completed checkpoints. | `G04-P1-B3` formal persistence and prefix-selection tests |
| 2026-08-01 | `G04-D010` | `Accepted` | Until P2-B3 defines versioned generation continuation, a formal generation request and its fenced result must finish before composite continuity commit; an in-flight generation marker fails closed. | `G04-P2-B1` lifecycle and continuity tests |
| 2026-08-01 | `G04-D011` | `Accepted` | `ferrite:overworld_v1` derives independent named noise streams from the configured seed and promises deterministic Ferrite replay plus the audited equivalence class, not Mojang same-seed block identity. | `G04-P2-B2` generator tests and Goal 01 equivalence boundary |
| 2026-08-01 | `G04-D012` | `Accepted` | `FWC2` owns bounded version-1 structure starts/references and `P8C2` owns generation continuation; `FWC1`/`P8C1` remain read-only migration inputs rather than receiving synthetic state. | `G04-P2-B3` codec, restart, and structure-placement tests |
| 2026-08-01 | `G04-D013` | `Accepted` | Formal terrain projection waits for a committed `FULL` and accessible authoritative column; missing work remains pending rather than falling back to flat terrain. Full-sky/empty-block light is the explicit bounded projection until P3-B2 installs propagated light authority. | `G04-P2-B4` snapshot, gateway, and packet tests |
| 2026-08-01 | `G04-D014` | `Accepted` | Each movement packet captures a bounded immutable collision scene from projectable authoritative columns before routing. The current generated state set maps air to empty and stone/grass to full cubes; missing chunks, height escape, oversized queries, and future unknown non-air states fail closed. | `G04-P3-B1` scene, adapter, movement, and network tests |
| 2026-08-01 | `G04-D015` | `Accepted` | `FWC3` owns exact sky/block light layers and `FWL2` owns clock, weather strengths, and weather RNG without introducing a new phase identity. `FWC1`/`FWC2`/`P8L1` remain read-only inputs; recovered lightless advanced chunks resume from `FEATURES`. Scheduled/random fire and fluid work runs only for block-ticking authority and every mutation relights, persists, and projects from that column. | `G04-P3-B2` durability, environment executor, gateway, and restart tests |
| 2026-08-02 | `G04-D016` | `Accepted` | Generated world spawn is selected from fully generated authoritative columns and persisted; the legacy `(8,64,8)` placeholder is accepted only as a generated-world migration input. Bootstrap uses bounded `spawn_search` tickets before readiness, fixed world spawn remains distinct from safe player placement, and the durable level border supplies both Java initialization and server movement clipping. | `G04-P3-B3` spawn, entry, collision, ticket, and restart tests |

## Completion record

| Field | Value |
|---|---|
| Final state | Pending |
| Completion commit | — |
| Remaining required work | `G04-P4-B1` through `G04-P5-B2` |
