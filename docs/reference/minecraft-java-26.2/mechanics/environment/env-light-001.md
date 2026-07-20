# Environment mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ENV-LIGHT-001` — Sky and block light propagate as separate bounded channels

**Parent:** `ENV-003`

**FidelityClass:** `EquivalentPlayerVisibleBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceInconclusive` — the locked server and client sources specify channel construction, every
source/attenuation/occlusion branch, removal-before-increase convergence, section lifecycle,
visible-map publication, packet production and client queue budgets. They do not impose a finite
server-tick, wall-time or render-frame bound from a world mutation to the first rebuilt client frame
under arbitrary chunk-dispatcher, executor, network and renderer load. The last authoritative
boundary is the server's visible light-map swap and dirty-section packet, followed on receipt by the
client's bounded FIFO import and light-engine run; `EXP-ENV-004` measures the remaining end-to-end
latency for a declared runtime/load profile.

**Applies when:**

A block state changes emission, dampening or light-occlusion shape; a column's lowest sky-blocking
edge changes; a section is initialized/emptied; chunk light data is queued; or a chunk column
enables/disables light sources.

**Authoritative state:**

Two independent nibble-valued maps, pending changed positions, decrease and increase FIFOs,
per-section storage/reference state, queued section data, changed/affected sections, enabled source
columns, each block state's emission/dampening/occlusion shapes, each sky column's lowest source
height, the dimension type's `has_skylight`, and the dimension value of
`minecraft:gameplay/sky_light_level`. The catalog assigns `ENV-003`/this leaf to every block family,
the 109 block IDs with at least one emitting state, all four dimension types and that environment
attribute.

**Transition and ordering:**

`LevelLightEngine#checkBlock` submits the position to block then sky. A synchronous engine
deduplicates changed positions, evaluates them, drains the entire decrease FIFO before the entire
increase FIFO, reconciles section inconsistencies, then copies changed updating layers to the
volatile visible map. The combined engine drains block work before sky work. The server wrapper
turns block/section/source changes into chunk-prioritized `PRE_UPDATE` tasks; one update consumes at
most 1,000 queued task records, runs their pre-actions, drains both engines completely, runs/removes
selected `POST_UPDATE` records, and later scheduling handles any remainder.

- **Block source and propagation:** A source exists only in a source-enabled column/section and
  starts at the current state's emission. On source loss, stored value becomes zero and a decrease
  entry carries the old value; an alternative neighbor brighter than that invalidation is
  re-enqueued as an increase. On source gain, the source value is enqueued directly. For a permitted
  neighbor, candidate `c = source - max(1, target.light_dampening)`; write/enqueue only when `c`
  exceeds the stored target value and the two face occlusion shapes do not jointly cover the crossed
  face. Missing lighting chunks behave as bedrock for state lookup.
- **Sky sources and propagation:** The sky engine exists only when the dimension type has skylight.
  For each local `x,z`, `ChunkSkyLightSources` scans down from the highest non-air section to the
  first vertically occluded edge; any below-state dampening other than zero blocks immediately,
  otherwise the above `DOWN` and below `UP` shapes jointly decide. Positions at or above that lowest
  edge are direct level-15 sources. Sky propagation uses the same attenuation/face test and
  decrease-before-increase recovery as block light. At horizontal section borders it bridges
  consecutive absent sections vertically; newly enabled sky columns prefill 15 down to each column
  threshold rather than requiring a one-block wave through empty sections.
- **Section storage and publication:** Light storage covers the world's section range plus one
  section below and above. A nonempty section and its 26 neighboring sections hold storage
  references; zero-to-nonzero creates a layer, nonzero-to-zero defers removal, and queued data is
  installed only where storage exists. First write copies a section layer. Publication copies the
  changed updating map to the visible snapshot, then reports the changed section and its neighbors.
  Block queries return zero for a missing layer. Sky queries return 15 above the column's top data
  section, climb to the next stored layer when their own layer is absent, and return zero for an
  internally queried disabled column.
- **Brightness and environmental darkening:** `getRawBrightness(pos,darken)` is
  `max(block, sky - darken)`; a missing channel contributes zero. `Level#updateSkyBrightness` sets
  `darken = (int)(15.0F - dimensionSkyLightLevel)`. `minecraft:gameplay/sky_light_level` is a
  synchronized, non-positional float constrained to `[0,15]` with default `15`; the resulting
  integer conversion truncates toward zero. Light-sensitive mechanics call their own query in their
  own phase; reconciliation does not itself trigger spawning, growth or melting.
- **Emitter audit:** Direct evaluation of all states in the locked 1,196-ID block registry finds
  exactly 109 IDs whose maximum emission is nonzero, and the catalog owns every one. State-varying
  rules are: `lit ? candles*3 : 0` for every candle, `lit ? 3 : 0` for candle cakes,
  `lit ? {15,12,8,4} : 0` for copper bulbs by oxidation, `lit ? {15,10} : 0` for ordinary/soul
  campfires, `lit ? 13 : 0` for furnace/smoker/blast furnace, `lit ? 7 : 0` for redstone torches,
  `lit ? 9 : 0` for redstone ores, `berries ? 14 : 0` for cave vines,
  `waterlogged ? 3*(pickles+1) : 0` for sea pickles, `charges == 0 ? 0 : 4*charges-1` for respawn
  anchors, and the `light` block's `level` value. Glow lichen emits 7 iff any face is present;
  redstone lamps emit 15 iff lit. Trial spawners emit 0 inactive/cooldown, 4 waiting-for-players and
  8 otherwise; vaults emit 6 inactive and 12 otherwise. The remaining state-constant emitters are:
  level 1 `brewing_stand`, `brown_mushroom`, both sculk sensors, `dragon_egg`, `end_portal_frame`,
  `small_amethyst_bud`; level 2 `firefly_bush`, `medium_amethyst_bud`; level 3 `magma_block`; level
  4 `large_amethyst_bud`; level 5 `amethyst_cluster`; level 6 `sculk_catalyst`; level 7
  `enchanting_table`, `ender_chest`; level 10 `crying_obsidian`, `soul_fire`, `soul_lantern`,
  `soul_torch`, `soul_wall_torch`; level 11 `nether_portal`; level 14 `copper_torch`,
  `copper_wall_torch`, `end_rod`, `torch`, `wall_torch`; and level 15 `beacon`, `conduit`, all eight
  copper-lantern oxidation/wax combinations, `end_gateway`, `end_portal`, `fire`, `glowstone`,
  `jack_o_lantern`, `lantern`, `lava`, `lava_cauldron`, all three froglights, `sea_lantern` and
  `shroomlight`.

**Branches and aborts:**

Disabled channel; section not storing light; source column disabled; candidate not greater than
stored; full face occlusion; absent neighbor section; unchanged state whose dampening/emission and
shape-use flags cannot alter lighting; deferred section removal; no visible/ticking chunk holder; no
tracking players; empty packet mask. `hasDifferentLightProperties` treats different states as
relevant when dampening or emission differs, or either state uses shape-dependent light occlusion.

**Constants and randomness:**

Values are integers 0–15 in 2,048-byte section nibble arrays. Direct sky is 15, every propagation
step loses at least one, the server light-task batch is 1,000, and propagation consumes no RNG.
Direction/queue representation is not required if the same converged maps and source-specified
publication/query boundaries result.

**Side effects:**

Publication marks the chunk unsaved and records one sky/block bit per affected light section. A
ticking visible holder sends one `ClientboundLightUpdatePacket` containing changed or explicitly
empty layers to players tracking the chunk, then clears its masks. The client receives on its packet
thread boundary, appends an import task, and on `ClientLevel#update` polls a snapshot budget: all
tasks when at least 1,000 are queued, otherwise `max(10,floor(size/10))`; it imports sky before
block, marks updated sections and neighbors dirty, enables the chunk, then drains client lighting
before renderer update/extraction.

**Gates:**

Dimension skylight, source-enabled column, section reference/storage state, chunk status and
priority, server/client work queues, ticking/visible chunk holder, tracking players, network
delivery, client game-load state and render rebuild.

**Boundary cases and quirks:**

Sky and block values may differ at one position. Removing one of overlapping sources produces a
decrease wave before the surviving source restores cells. Empty vertical sections have special sky
bridging but no corresponding block-light shortcut. Server gameplay can observe its newly published
light before a client imports or redraws it. No source constant licenses “one tick” or “one frame”
as the universal visibility deadline.

**Evidence:**

`Confirmed`; `OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-REPORT-001`;
`net.minecraft.world.level.lighting.LevelLightEngine#checkBlock(net.minecraft.core.BlockPos)`,
`LevelLightEngine#runLightUpdates()`,
`LevelLightEngine#getRawBrightness(net.minecraft.core.BlockPos,int)`,
`net.minecraft.world.level.lighting.LightEngine#runLightUpdates()`,
`LightEngine#getOpacity(net.minecraft.world.level.block.state.BlockState)`,
`net.minecraft.world.level.lighting.BlockLightEngine#checkNode(long)`,
`net.minecraft.world.level.lighting.SkyLightEngine#checkNode(long)`,
`net.minecraft.world.level.lighting.LayerLightSectionStorage#swapSectionMap()`,
`net.minecraft.world.level.lighting.ChunkSkyLightSources#fillFrom(net.minecraft.world.level.chunk.ChunkAccess)`,
`net.minecraft.server.level.ThreadedLevelLightEngine#runUpdate()`,
`net.minecraft.server.level.ServerChunkCache#onLightUpdate(net.minecraft.world.level.LightLayer,net.minecraft.core.SectionPos)`,
`net.minecraft.server.level.ChunkHolder#broadcastChanges(net.minecraft.world.level.chunk.LevelChunk)`,
`net.minecraft.world.level.Level#updateSkyBrightness()`,
`net.minecraft.client.multiplayer.ClientPacketListener#handleLightUpdatePacket(net.minecraft.network.protocol.game.ClientboundLightUpdatePacket)`,
`ClientPacketListener#applyLightData(int,int,net.minecraft.network.protocol.game.ClientboundLightUpdatePacketData,boolean)`,
and `net.minecraft.client.multiplayer.ClientLevel#pollLightUpdates()`. `EXP-ENV-004` owns only the
load-profile latency unknown.

**Test vectors:**

Add/remove one emitter; overlap two unequal sources; toggle every state-dependent emitter property;
change dampening and a shape-only occluder; open/close sky at a column threshold; cross
section/chunk borders with empty vertical sections; initialize/remove/reload layers; use
skylight-disabled and custom sky-level dimensions; force server batches of 999/1,000/1,001 and
client queues of 9/10/999/1,000; compare server publication, packet, client import and first rebuilt
frame.
