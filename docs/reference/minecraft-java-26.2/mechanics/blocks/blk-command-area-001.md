# Block mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `BLK-COMMAND-AREA-001` — Area commands precharge the whole inclusive box

**Parent:** `BLK-003`, `WGEN-002`, `CLI-006`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — the integer rule, long volume calculations, validation order, clone/fill modes,
strict flags, block-entity and scheduled-tick handling, biome quart quantization, chunk preflight,
partial-failure behavior and feedback counts are explicit in locked source.

**Applies when:**

`/clone`, `/fill` or `/fillbiome` has parsed its positions and is about to validate or mutate the
requested area, including the public programmatic `FillBiomeCommand#fill` overloads.

**Authoritative state:**

The live source-level `max_block_modifications` rule; inclusive source/destination/fill boxes;
selected dimensions, loaded/debug state, clone overlap/mode/filter/strict state; ordered source and
destination block states, block entities, components and scheduled ticks; fill mode/predicate/input;
quart-quantized biome box, FULL chunks, old biome holders, replacement predicate, dirty state and
connected-client biome projection.

**Transition and ordering:**

The `MISC` integer rule defaults to `32768`, accepts `1..Integer.MAX_VALUE`, and has no change
callback. Every command reads it once from the command source's current level at preflight. Clone
does so even when source and destination are other dimensions. Equality is admitted; only
`inclusiveXSpan * inclusiveYSpan * inclusiveZSpan > rule` fails. Multiplication is widened to
`long`, and the error component receives configured maximum then requested volume. Filters, modes,
air, unchanged state and actual affected count never discount this precharge.

**Clone preflight:** Clone constructs the inclusive source box and a same-sized destination box
whose far corner is destination plus the source box length vector. NORMAL rejects intersection only
when both boxes are in the identical level; FORCE and MOVE allow it. Overlap rejection precedes the
volume rule. After the rule, both complete boxes must have chunks loaded, then a debug destination
fails. These branches mutate nothing.

Clone scans source positions in increasing Z, then Y, then X. The chosen replace predicate sees a
nonloading `BlockInWorld`: default/`replace` accepts all, `masked` rejects air, and `filtered` uses
the parsed block predicate. Each accepted state and its destination offset are snapshotted before
any write. Block entities save custom NBT plus components. Entries partition into block entities,
solid-render/full-collision blocks, and remaining nonfull blocks; the source-clear deque puts
nonfull positions at the front and the other two at the back.

The base destination flag is `2` normally and `818` under `strict`. MOVE first replaces selected
source positions with barrier using flags `818`, then replaces them with air in deque order using
flags `3` normally or `818` under strict. It ignores every source write result. NORMAL/FORCE retain
the source.

Destination work concatenates solid, block-entity, then nonfull entries. It first installs barriers
in reverse concatenated order with flags `818`, ignoring results, then places the saved states in
forward order with the base flag. Only successful final state placements increment the result.
Every block-entity entry then looks up the new destination entity; when both snapshot and entity
exist it loads custom NBT, replaces components and marks changed. It calls `setBlock` again with the
saved state and ignores that result.

Nonstrict clone next invokes `updateNeighboursOnBlockSet` in reverse concatenated order using each
destination's pre-scan state; strict clone skips this pass. Finally destination block ticks copy
from the entire source box with the source-to-destination offset, independent of the clone filter,
placement success or MOVE. If the successful-placement count is zero, this tick copy and any
earlier source/destination/entity writes remain committed before `commands.clone.failed` is thrown.
Otherwise `commands.clone.success` reports and returns that count. No later branch rolls back.

**Fill preflight and traversal:** Command syntax resolves both endpoints through
`BlockPosArgument#getLoadedBlockPos`, so each endpoint's chunk must already be loaded and each point
must be within hard world bounds. `fillBlocks` then applies the full inclusive-box rule and rejects
a debug level, but performs no all-intermediate-chunks preflight. It visits the closed box in X-fast,
then Y, then Z order. An optional `replace` predicate is tested first with a chunk-loading
`BlockInWorld`; `keep` is the empty-block predicate. Predicate failure skips all work at that point.

REPLACE supplies the requested block everywhere. OUTLINE supplies it only on any box face and skips
the interior. HOLLOW supplies it on faces and an air `BlockInput` in the interior. DESTROY first
calls `destroyBlock(pos, true)` and then still supplies the requested block. The destroy result is
remembered before mode filtering/placement. A position increments the count once when destroy or
placement succeeds, never twice; DESTROY can therefore count a destroyed position even if later
placement fails. Mode/filter/placement exceptions retain all earlier writes.

Placement uses flags `258` normally and `818` under strict. Only a successful nonstrict placement
records its immutable position and original pre-destroy state; after traversal those entries call
`updateNeighboursOnBlockSet` in visit order. Strict skips that list. Destroy-only successes rely on
`destroyBlock` follow-up behavior. Count zero throws `commands.fill.failed`; a positive count sends
`commands.fill.success`, informs administrators, and is returned. There is no rollback at the
zero-count or later-exception boundary.

**Fill-biome quantization and commit:** Each endpoint coordinate is independently converted block →
quart → block, rounding down to a multiple of four, and the inclusive box is built from those two
origins. The rule charges that quantized block-coordinate box, not the number of biome cells. The
routine then enumerates intersecting chunk X/Z coordinates in Z-major order and requests FULL with
generation disabled. Any missing chunk returns the unloaded error before biome mutation.

Every collected chunk runs `fillBiomesFromNoise` with a resolver that converts each quart coordinate
to its block origin. Outside the quantized box or when the old biome fails the optional resource/tag
predicate, it returns the old holder. Otherwise it increments the result and returns the target;
default accepts every old biome, including one already identical to the target. Thus the result is
matching quart cells, not changed blocks or distinct biome values.

After each chunk pass it is marked unsaved even when the result stayed zero. After all chunks, the
entire collected list is sent through `resendBiomesForChunks`, then the supplied feedback consumer
is called with `commands.fillbiome.success.count` and quantized bounds. The command wrapper sends
that success with administrator informing and returns the count; zero is successful. The public
overload returns `Either.left(count)` and may use a no-op consumer, while size/unloaded failures are
`Either.right` and have no mutations or success callback.

**Branches and aborts:**

Rule below/equal/above long volume; endpoint/intermediate loading; hard bounds and debug worlds;
same/cross-level overlap; clone NORMAL/FORCE/MOVE, replace/masked/filtered and strict; source block
category and write results; fill replace/keep/outline/hollow/destroy/strict, predicate and placement;
biome quantization, missing FULL chunk, old-biome filter, identical target and zero/nonzero count.

**Constants and randomness:**

Default `32768`, minimum `1`, maximum `Integer.MAX_VALUE`; biome quart width `4`; clone/fill flags
`2`, `3`, `258`, `818`; no RNG is consumed by these transactions. Traversal and feedback order are
deterministic for a fixed world and predicate.

**Side effects:**

Block destruction/drop behavior; block state and block-entity NBT/component writes; neighbor and
comparator consequences through update flags; copied scheduled block ticks; chunk loading reachable
from fill predicates/writes; biome container replacement, chunk dirtiness and biome resend packets;
success/failure/admin feedback and server problem logs. Already completed effects are never rolled
back by a later failure.

**Gates:**

Command permission/parsing; loaded endpoints/boxes; hard bounds/debug/overlap; the one live rule
read; clone/fill/biome predicates and modes; strict; individual write results; target block-entity
existence and FULL chunk availability.

**Boundary cases and quirks:**

The rule measures requested volume before filters. Clone uses the command source level's rule across
dimensions and can copy scheduled ticks then report failure. Fill lacks clone's complete box load
preflight, and DESTROY may count without a successful replacement. Fillbiome rounds both endpoints
down, counts matching quart cells even when already target, dirties/resends every collected chunk,
and treats zero matches as success. Strict changes flags and explicit neighbor follow-up but not the
volume or result-count definitions.

**Evidence:**

`OFF-SERVER-001`; `OFF-REPORT-001`; `net.minecraft.world.level.gamerules.GameRules`;
`net.minecraft.server.commands.CloneCommands#clone`;
`net.minecraft.server.commands.CloneCommands$Mode`;
`net.minecraft.server.commands.FillCommand#fillBlocks`;
`net.minecraft.server.commands.FillCommand$Mode`;
`net.minecraft.server.commands.FillBiomeCommand#quantize`;
`net.minecraft.server.commands.FillBiomeCommand#makeResolver`;
`net.minecraft.server.commands.FillBiomeCommand#fill`;
`net.minecraft.commands.arguments.coordinates.BlockPosArgument#getLoadedBlockPos`;
`net.minecraft.world.ticks.LevelTicks#copyAreaFrom`; `BLK-UPDATE-001`; `EXP-BLK-018`.

**Test vectors:**

For each command, sweep volumes `limit-1/limit/limit+1`, extreme legal coordinates and a rule
change immediately before the call. Cross every mode/filter/strict option, write success/failure,
loaded/debug/dimension/overlap branch and zero/partial/full count; capture exact write, BE, tick,
neighbor, drop, feedback and nonrollback order. For fillbiome, use positive/negative nonaligned
endpoints, missing interior chunks, identical targets and zero/mixed/all predicates; assert charged
quantized volume, quart count, dirty chunks and resend audience.
