# Minecraft Java 26.2 Reference Audit — Wave 1, Worker 4: Structure Graphs

## Result

The source-backed audit completed for the scope below. Its findings update reference documentation
only and do not change Ferrite implementation dispositions.

## Scope and evidence

This worker audited the recursive graph and room-layout reference for these leaf rules against the
repository-locked official Minecraft Java 26.2 server and client artifacts:

- `WGEN-STRUCTURE-STRONGHOLD-001`;
- `WGEN-STRUCTURE-MINESHAFT-001`;
- `WGEN-STRUCTURE-END-CITY-001`;
- `WGEN-STRUCTURE-FORTRESS-001`;
- `WGEN-STRUCTURE-OCEAN-MONUMENT-001`;
- `WGEN-STRUCTURE-WOODLAND-MANSION-001`.

The audit corrected two source-backed reference errors. Stronghold retries relocate every completed
attempt before testing the portal-room success condition, so a failed attempt can consume a
relocation draw before it is discarded. Woodland-mansion room identification tests all four
rectangle corners exactly once; it does not repeat the initial corner or omit the fourth corner.

The Mineshaft, End City, Fortress, and Ocean Monument leaf documents already matched the reviewed
26.2 bytecode for graph/frontier order, weights and quotas, retry or abort behavior, latches, room
connectivity, live-chunk decisions, RNG order, and saved piece state. They required no edits.

For every assigned leaf, the audit followed its documented generation entry point through child or
room selection, piece construction and chunk placement. It rechecked source locators, locked data
inputs and constants, each RNG gate, abort and rollback edge, mutation versus observable-write
order, transient and persisted state, downstream placement/entity/block-entity handoffs, and the
leaf's executable reproduction vectors.

This is a reference-material audit only. It does not assert or change Ferrite implementation
disposition.

## Locked inputs

- `target/mc-reference/26.2/server.jar`: official server artifact, locked SHA-1
  `823e2250d24b3ddac457a60c92a6a941943fcd6a`.
- `target/mc-reference/26.2/client.jar`: official client artifact, locked SHA-1
  `2dc72797acbc1b63fc16a11c4ac393605f453754`.
- `target/mc-reference/26.2/server-26.2.jar`: server implementation extracted by `mc-ref reports`
  from the locked official server bundle.
- `target/mc-reference/26.2/generated/reports`: official data reports generated with the repository
  mc-ref tool and Java 25.
- Existing leaf documents and their exact `completion.toml` records.

No external source, decompiler mapping, or unpinned game artifact was used.

## Findings

### `WGEN-STRUCTURE-STRONGHOLD-001`

Material correction: `StrongholdStructure#generatePieces` drains the randomly indexed pending
frontier, then calls `StructurePiecesBuilder#moveBelowSeaLevel`, and only afterward evaluates
`builder.isEmpty()` and the start piece's portal-room pointer. Relocation therefore occurs on every
attempt, including a graph that will be discarded and reseeded because it has no portal room. The
next attempt's explicit `setLargeFeatureSeed(worldSeed + attempt, chunkX, chunkZ)` isolates its
graph stream, but the failed attempt still has a source-observable relocation draw in its own trace.

The remaining documented behavior was confirmed: static graph reset, imposed first-five-crossing
selection, weighted quota and repeated-piece eligibility, null-factory fallthrough, uniform pending
removal, depth/range/collision stops, filler overlap, portal guarantee, per-piece persistence,
shared source-global selection state, chest/spawner latches, and the per-intersecting-chunk
portal-eye redraw.

The leaf and its completion selector/reproduction text now make the per-attempt relocation order
explicit.

### `WGEN-STRUCTURE-MINESHAFT-001`

Confirmed without edits: the source-piece child expansion, corridor/crossing/stairs branching, depth
and distance rejection, live collision queries, random rail/web/spawner/chest decisions, per-piece
latches, and save/load fields match the existing leaf. No source-undetermined claim was promoted.

### `WGEN-STRUCTURE-END-CITY-001`

Confirmed without edits: recursive generator calls, depth cap, per-branch collision rollback,
generator-wide ship latch, bridge/tower/fat-tower/house template order, rotation-aware attachment,
marker handling, and persisted template-piece state match the existing leaf.

### `WGEN-STRUCTURE-FORTRESS-001`

Confirmed without edits: separate bridge/castle weight pools and quotas, repeated-piece eligibility,
pending-child uniform removal, source-piece range/depth/collision aborts, filler fallback, process
wide mutable weight entries, piece latches, piece fields, live placement decisions, and save/load
behavior match the existing leaf.

### `WGEN-STRUCTURE-OCEAN-MONUMENT-001`

Confirmed without edits: room-definition graph linking and openings, source-room reservation,
fit-helper ordering, sponge and simple-room decisions, wing/penthouse ordering, elder placement
latches, chunk-clip-sensitive live writes, and persisted piece state match the existing leaf.

### `WGEN-STRUCTURE-WOODLAND-MANSION-001`

Material correction: after two Booleans choose a tentative X/Y endpoint, `MansionGrid#identifyRooms`
checks that corner, the diagonal opposite, the X-opposite/Y-original corner, and the
original-X/Y-opposite corner. All four rectangle corners are tested once in that exact order. If
none touches a corridor, the door flag is cleared while the source origin and size/room identifiers
are still assigned as documented.

The remaining documented behavior was confirmed: the 11-by-11 recursive base graph, live cleaning,
independently shuffled first- and second-floor partitions, optional third-floor stair transaction
and rollback quirk, room/template transform selection, template overwrite and marker order, mob and
chest behavior, save/load state, and live foundation descent.

The leaf and its completion selector/reproduction text now describe four-corner door resolution.

## Placement and protocol handoffs

The recursive builders emit ordered piece lists that later enter the generic structure/chunk
placement path; template structures additionally enter template processing and DATA-marker handling.
Chest loot initialization, spawner configuration, entity creation, and post-placement foundation
writes occur at the exact piece-local gates documented by the leaves. Persistence stores
piece/template fields rather than reconstructing transient frontiers or room-search work.

None of the six graph builders directly selects or emits a network packet. Their block,
block-entity, entity, loot, and saved-piece results enter the ordinary downstream chunk, entity, and
block-entity persistence/synchronization owners. The audit found no missing direct protocol handoff
or ordering claim in the assigned leaves.

## Reproduction

The decisive control-flow findings can be reproduced from the extracted locked server jar:

```text
javap -classpath target/mc-reference/26.2/server-26.2.jar -p -c -constants \
  net.minecraft.world.level.levelgen.structure.structures.StrongholdStructure

javap -classpath target/mc-reference/26.2/server-26.2.jar -p -c -constants \
  'net.minecraft.world.level.levelgen.structure.structures.WoodlandMansionPieces$MansionGrid'
```

The wider audit used the same `javap -p -c -constants` form for the assigned structure classes,
their nested pieces/generators, `StructurePiecesBuilder`, and template-piece placement handoffs.
Locked records and constants were checked against the generated reports and existing data paths
named by each completion slice.

Executable behavioral vectors remain those stated in each leaf and completion record: fixed random
streams spanning every weighted endpoint, quota/repeat boundary, frontier index, retry/rollback,
collision and live-chunk outcome, latch transition, partial clip, and save/reload state. In
particular, the Stronghold vector must trace relocation before a failed portal decision, and the
Woodland Mansion vector must force each ordered corner to be the first corridor-connected endpoint.

## Unresolved items

No new experiment was introduced. `EXP-WGEN-001` remains explicitly required only for the separately
owned placement/distribution calibration already named by these structure references. The official
source determines the graph and state-machine corrections in this report, so neither correction is
deferred to an experiment.

## Evidence and verification

The ignored mc-ref cache was bootstrapped from the locked artifacts by generating official reports
with Java 25 and extracting the client jar's embedded `version.json` into the cache layout expected
by protocol verification.

```text
MC_REF_JAVA="$JAVA_HOME/bin/java" \
  cargo run -p mc-reference --bin mc-ref -- reports

MC_REF_JAVA="$JAVA_HOME/bin/java" \
  cargo run -p mc-reference --bin mc-ref -- verify --offline

cargo test -p mc-reference
git diff --check
shasum target/mc-reference/26.2/server.jar target/mc-reference/26.2/client.jar
```

The complete offline verification passed: 417 documentation IDs, 331 completion slices, 2,789 source
locators across 952 classes, 9,078 locked catalog IDs, 307 experiment definitions, all 256 protocol
packets, all behavior surfaces, and the implementation-manifest consistency check. The mc-reference
test suite passed all 38 tests, `git diff --check` reported no error, and both artifact SHA-1 values
matched the locks above.
