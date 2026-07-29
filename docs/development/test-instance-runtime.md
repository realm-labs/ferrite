# Test Instance Runtime

`G01-P5-S005` implements `BLK-TEST-INSTANCE-RUNTIME-001` and its normative
`BLK-TEST-INSTANCE-001` leaf in `ferrite-gameplay::block::test_instance`. The implementation is
split by responsibility:

- `data` owns the six-field record, rich text components, markers, status/action wire fallbacks,
  persistence, and block-entity dirtiness/publication;
- `geometry` owns signed rotations, normalized boxes, permanent forced-chunk ranges, boundary
  shells, and the ordered template-placement transaction;
- `operations` owns level-thread action admission, query/mutation ordering, RESET/SAVE/EXPORT/RUN,
  runner replacement, and result convergence;
- `client` owns empty-hand use, the local non-menu editor, status-response races, beams, bounds,
  markers, and combined render admission.

## Durable entity and publication

The property-free state is locked at runtime ID `21742`, block-entity protocol ID `46`, and POI
range/ticket values `1`/`0`. Block and item properties preserve the full collider, non-occluding
and non-view-blocking projection, no loot, `-1`/`3,600,000` strength, immunity tags, epic stack-64
GameMaster item, and common cube-all texture.

`TestInstanceData` preserves the optional test key, raw signed size, extra rotation,
`ignoreEntities`, status, and optional complete text component. `TestComponent` retains literal,
composed, or opaque adapter payloads so styled trusted components are not flattened at the
gameplay boundary. Status replacement clears only the top-level error; error replacement forces
finished; neither touches positional markers.

Every ordinary data/status/marker mutation emits chunk dirtiness followed by an AIR-to-current
flags-`3` update when attached to a server level. Clearing an empty marker list is the sole marker
no-op. Save always contains data and omits an empty marker field; load retains current data after
record decode failure, always replaces markers with decoded-or-empty state, and routes valid data
through the ordinary setter. The update payload is the same complete record.

## Actions and structure transaction

The server action planner moves to the level thread, checks game-master permission, then looks up
the matching entity. It has no block-identity, reach, or size input. IDs `0..6` map to
INIT/QUERY/SET/RESET/SAVE/EXPORT/RUN and every other signed value maps to INIT.

INIT and QUERY ignore the supplied record and emit one positionless requester response. A resolved
test supplies its rich description; only QUERY with a resolved template supplies size. Missing
registry and missing-template outcomes remain distinct. Mutations install the entire unbounded
packet record first, generating the ordinary dirty/update pair, and finish with the handler's
second AIR-to-current flags-`3` update unless pre-catch export path validation propagates.

Geometry composes intrinsic and extra quarter rotations, swaps X/Z for quarter turns, offsets the
structure by `(p,p+1,p+1)`, normalizes signed/overflowed endpoints, and inflates the test box by
padding. Placement start offsets deliberately use the untransformed stored size. One placement:

1. permanently force-loads the structure-box chunk rectangle;
2. clears the test box to AIR with flags `818` and an explicit neighbor update per cell;
3. clears scheduled block ticks and block events;
4. performs the clear transaction's non-player discard and the entity's second discard query;
5. places the resolved template with effective rotation, raw `ignoreEntities`, common origin/pivot,
   known-shape, live level RNG, and flags `818`.

The plan represents forged unbounded ranges without eagerly allocating every cell or chunk.
Region integration remains responsible for executing that semantic range under the source-defined
transaction rather than introducing a handler clamp.

## RESET, SAVE, EXPORT, and RUN

RESET removes only barrier blocks from the one-block walls/floor/optional-ceiling shell, clears
markers, attempts placement, sends a success message only when the template resolves, and always
forces cleared/no-error status. SAVE selects the holder's structure key before falling back to the
packet test key, reports coordinates only when neither exists, and otherwise captures raw size and
entity inclusion with empty author, disk save, and AIR/STRUCTURE_VOID omission while ignoring the
capture result.

EXPORT always performs SAVE first. Disabled export, missing cache, and caught file failure report
red and return `true`; success reports the absolute path and returns `false`; both returns are
ignored. Pre-catch path validation is an explicit propagating branch.

Successful RUN performs the packet-sized placement, clears markers and global GameTest/failed-test
state, announces the registered key, and creates one no-retry test. Its in-place spawner replaces
the entity at the same position with actual template size or `(1,1,1)`, packet extra rotation,
entity inclusion, and cleared status. It places again, installs the barrier shell except existing
test-instance cells, and begins after setup delay. Missing holder/template paths retain packet data
and markers. Start, pass, failure, absolute marker, passing cleanup, and all-player broadcast
effects preserve their independent update order.

## Local editor and rendering

Empty-hand admitted use succeeds on both sides and opens only the client-local screen. Opening
sends INIT. Identifier edits always send QUERY, with invalid text first installing a local error;
positionless/unsequenced responses update whichever screen is current and optional size replaces
all three fields. The editor enforces 128/15 UTF-16-unit fields, size parse/default/clamp
`1..48`, inverse include/ignore state, effective-rotation initialization, UI-only Save/Export
gates, IDE-only Export visibility, cleared/no-error outgoing records, and packet-free cancel,
escape, or ordinary close.

Beam projection is absent while cleared, gray while running, green on success, red on required or
missing failure, and orange on optional failure. It is opaque, permission-independent, uses
ordinary beacon animation/distance scaling, and terminates at height `2048`. Bounds are always a
light-gray BOX with no invisible cells and require game-master ability or spectator plus positive
transformed axes. Markers are copied without that permission gate as red alpha-`0.375` cubes
inflated `0.02` plus centered white always-on-top text at height `1.2` and scale `0.16`.

`G01-P5-B1` remains the single integration owner for applying these semantic operations to Region
voxel state, scheduled queues, private ECS entities, persistence, registry snapshots, and client
projection.
