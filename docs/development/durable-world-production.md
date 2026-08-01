# Durable world production contract

Goal 04 replaces the formal gateway's fixed world ID, `MinimalTerrain`, and `FlatWorldCollision`
with one configured, generated, durable authority. Goal 01 world behavior remains the algorithm and
continuity reference; isolated conformance implementations do not count as production integration
until this contract's formal-entry chain is complete.

## Production denominator

The machine-checked production manifest owns eight non-overlapping world responsibilities:

| Row | Completion boundary |
|---|---|
| `world/configuration` | Versioned identity, seed, generator, spawn, distances, dimensions, and save policy load and validate before listener readiness |
| `world/chunk-lifecycle` | Bounded tickets drive load/generate, activation, save, unload, and recovery without losing dirty state |
| `world/generation` | Audited biome through structure stages install fenced results into authoritative chunks |
| `world/projection` | Client terrain, light, block entities, and unload packets derive from committed authoritative chunks |
| `world/collision` | Movement queries block-state shapes from those same committed chunks |
| `world/environment` | Fluids, scheduled/random work, lighting, fire, weather, time, and border share authority and continuity |
| `world/dimensions` | Configured overworld, nether, and end levels have independent durable state and lifecycle |
| `world/portals` | Discovery, creation, coordinate scaling, safe placement, cooldown, transfer, and projection form one restart-safe transaction |

`storage/production-continuity` remains the cross-service durable sink. No row is promoted merely
because the corresponding Goal 01 algorithm or codec exists. A player-visible row requires the
exact 26.2 MCP to exercise that responsibility through `ferrite-server`.

## One authoritative representation

`ferrite_world::chunk::ChunkColumn` is the durable voxel authority. A live authoritative column
contains section palettes, biomes, structure starts/references, heightmaps, light, block entities,
revisions, generation status, and post-processing state. Simulation, collision, persistence, and Java projection receive bounded
views of the same committed column; none may retain a parallel flat, packet-shaped, or collision-only
world.

Generation executes against immutable inputs and returns a fenced candidate. Publication verifies
world, dimension, Region, activation generation, chunk position, source revision, generator version,
and content-manifest digest before replacing authority. Failed or stale work has no partial visible
effect. Client projection begins only after the authoritative commit.

## Versioned configuration surface

Goal 04 introduces server configuration schema 2. Its world section has these semantic fields:

- nonzero stable `WorldId`;
- signed 64-bit seed and responsibility-owned generator identifier/version;
- generated or explicit spawn policy;
- server view and simulation distances in `2..=32`;
- a nonempty, duplicate-free ordered dimension list whose first entry is the overworld;
- autosave interval, maximum pending Region saves, journal checkpoint cadence, and shutdown flush
  policy.

Schema 1 migration is deterministic and write-explicit. It assigns the former formal constants:
world ID 1, seed 0, the first Ferrite overworld generator version, generated spawn, distances 10/10,
overworld only, a 6,000-tick autosave interval, 128 pending Region saves, and a 64-commit checkpoint
cadence. Migration is allowed only when the selected storage root has no durable world with a
conflicting identity. The parser never silently treats unknown future fields or schemas as defaults.

Configuration selects identity and policy; durable metadata proves what created the existing world.
Changing seed, generator version, dimension set, Region mapping, chunk format, or content manifest
against an existing store is a migration request and fails closed until a supported migration is
explicitly run.

## Durable layout and identities

The workspace-independent logical layout below is rooted under the configured node storage root:

```text
worlds/<world-id-32hex>/
  dimensions/<namespace>/<resource-path>/
    regions/r.<region-x>.<region-z>/
      region-journal.log
      region-data.log
      region-index.log
```

Every namespace and resource-path component is validated before path construction; absolute paths,
`.` and `..` components, symlinks escaping the storage root, and alternate spellings are rejected.
One directory contains one `SimulationRegionKey`, which bounds recovery scans and future compaction.
The control Region in each dimension owns the versioned level record; the overworld control Region
also owns world metadata and the configured dimension catalog.

Existing canonical formats remain compatibility surfaces:

| Format | Current identity |
|---|---|
| Region snapshot/recovery point | `FRSN` schema 1 plus a contiguous journal tail |
| World chunk payload | `FWC3`; `FWC1`/`FWC2` are read-only migration inputs without authoritative light (`FWC1` also lacks structure state) |
| World-service chunk lifecycle wrapper | `P8C2`, written under `ferrite:world-service/chunk_v1`; `P8C1` remains a read-only migration input |
| World-service level state | `FWL2`, written under `ferrite:world-service/level_v1`; `P8L1` is a read-only border-only migration input |
| Goal 04 world metadata | `ferrite:world-service/world_v1` with its own bounded magic/version |

Legacy `FWC1`, `FWC2`, `P8C1`, `P8L1`, and `ferrite:phase8/*_v1` records remain read-only migration inputs. New commits contain current
responsibility identities only. Unknown versions, mixed generations, duplicate canonical keys,
content mismatches, and complete-frame corruption fail closed and remain inspectable.

`FWC3` stores the sky and block-light layers used by projection. `FULL` authority without light is
never projected; an older recovered column is demoted to `FEATURES` and deterministically resumes
the light stages. Block mutation recomputes the bounded column light state before commit. Light from
an emitting block propagates within the owning column. The bounded reconstruction never treats an
unavailable neighbor as light authority; cross-column convergence is deferred until both columns
are owned and recomputed.

`FWL2` stores game time, day time, the five weather fields, current and previous rain/thunder
strengths, and the deterministic weather random state. The overworld control Region advances and
captures it before every composite commit. Joining and active Java 26.2 sessions receive the same
clock and weather projection.

## Save, acknowledgement, and compaction

At the composite continuity stage, a Region captures immutable canonical records and the exact dirty
revision tokens they represent. A bounded storage worker serializes only that capture. Its writer
lease is fenced by `SimulationRegionKey` and activation generation.

`RegionFileStore` commits in intent, data, index-repoint, commit-marker order with a sync at each
step. Only the final receipt acknowledges persistence. A Region clears a dirty bit only when the
receipt's capture token still equals the live revision; concurrent mutation leaves it dirty. Queue
overload is explicit and prevents unload or clean shutdown from claiming durability.

Journal tails bound write amplification between full snapshots. A full checkpoint is forced at the
configured cadence, before handoff, and before a clean unload. Compaction copies only the latest
validated committed recovery point into a sibling temporary store, syncs it, atomically repoints the
store directory, and retains the previous generation until the replacement is reopened and verified.
Compaction never edits active logs in place.

## Startup, unload, and shutdown

Startup validates configuration, resolves every configured durable path, opens the overworld control
Region, and either restores its world metadata or creates the initial revision. Other dimensions are
then restored in configured order. Listener readiness is published only after spawn tickets reach the
required generated and activity states.

A chunk may unload only when no live ticket or transfer owns it, no generation result is in flight,
its pending-unload identity still matches, and the exact captured revision has a durable receipt.
An in-flight generation marker may be committed as a version-1 continuation containing the request,
source revision, next status, and content manifest. Recovery validates that identity and reissues the
same deterministic work under the new Region activation before unload or visibility can advance.
New demand cancels the pending unload by identity. Missing storage generates only when configuration
permits creation; corrupt or mismatched storage never becomes an empty replacement world.

Clean shutdown stops new admission, completes the current composite prefix, deactivates tickets,
captures every dirty Region, drains the bounded save queue, verifies receipts, checkpoints control
Regions, closes stores independently with complete diagnostics, and only then releases process
resources. Abrupt loss recovers the latest fully committed transaction and never resumes a callback,
packet, render mirror, or incomplete random draw.

## Goal 01 equivalence boundary

Ferrite implements the audited generation stages, gates, distributions, ordering dependencies,
bounded codecs, and deterministic project-owned behavior. It does not claim block-for-block
same-seed identity with Mojang. `EXP-WGEN-001`, `EXP-WGEN-005`, and `EXP-WGEN-006` remain separate
deferred population/equivalence calibration and cannot block truthful Goal 04 production evidence.

## Acceptance matrix

Goal 04 must prove all of the following through focused tests and, where visible, the exact client:

- schema-1 migration, schema-2 round trip, unknown field/schema rejection, and incompatible durable
  metadata rejection;
- new-world creation plus restart reconstruction of metadata, chunks, ticks, environment, and
  dimensions;
- torn intent/data/index/commit writes, checksum corruption, stale generation, duplicate writer,
  save-queue overload, save/unload races, and compaction interruption;
- deterministic generation/replay and canonical state hashes across load, save, unload, and restart;
- nonflat exploration, voxel collision, block mutation persistence, time/weather/light convergence,
  overworld/nether/end activation, portal travel, clean restart, and framebuffer evidence;
- universal format, Clippy, workspace test, source-size, production-manifest, and clean-worktree
  gates.
