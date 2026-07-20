# World mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `WGEN-STRUCTURE-IGLOO-001` — Igloos terrain-anchor a top and an optional laboratory shaft

**Parent:** `WGEN-003`

**FidelityClass:** `EquivalentPlayerVisibleBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked 26.2 server source fixes the generation stub, piece inventory/order,
terrain probe, sparse-template transaction, marker loot and entity restoration; all three templates
and both registry records are audited data-only. Shared random-spread admission remains owned by
`WGEN-PIPELINE-001`.

**Applies when:**

The `minecraft:igloo` type has passed caller-owned structure-set placement, or one of its retained
template pieces intersects a placement chunk.

**Authoritative state:**

The structure record selects `snowy_taiga`, `snowy_plains` and `snowy_slopes`, no spawn overrides,
`surface_structures`, and default `none` terrain adaptation. Its set contains only igloo at weight
`1`, with random-spread spacing `32`, separation `8`, salt `14357618`, and default frequency fields.
A piece persists its template ID/position, bounding box and rotation; placement rebuilds settings
from that rotation, the template-specific pivot and offset.

**Transition and ordering:**

The generic top-of-chunk-center stub samples `WORLD_SURFACE_WG` first occupied height at local
`(8,8)` and the valid-biome gate samples that exact 3-D quart position. After admission, the piece
anchor is instead `(chunkMinX,90,chunkMinZ)`. The structure RNG draws rotation from
`[none, clockwise_90, 180, counterclockwise_90]`, then draws a double. A value below `0.5` creates a
basement depth `d=4+nextInt(8)`, adds `bottom` with vertical offset `3d`, adds `middle` for indices
`0..d-2` with offsets `3i`, then adds `top`; otherwise it adds only `top`. Thus the basement branch
retains `5..12` pieces in bottom, shaft-top-to-bottom, top list order, while the top-only branch
retains one.

Every intersecting-piece invocation reconstructs settings, then derives one common horizontal
surface probe from its original Y-90 position. Relative to the chunk minimum, that probe is `(3,0)`,
`(8,5)`, `(3,10)` or `(-2,5)` for the four rotations above. Let `H` be the live `WORLD_SURFACE_WG`
height there. The piece temporarily shifts by `H-91`, so `top` begins at Y `H-1`, `middle(i)` at
`H-4-3i`, and `bottom(d)` at `H-4-3d`; base placement recomputes the piece box at that position, and
the original template position is restored afterward. This live probe is independent of the earlier
center stub and can cross the source chunk boundary.

Settings use mirror none, pivots `top=(3,5,5)`, `middle=(1,3,1)`, `bottom=(3,6,7)`, offsets
`top=(0,0,0)`, `middle=(2,-3,4)`, `bottom=(0,-3,-2)`, `STRUCTURE_BLOCK` ignore processing and
`IGNORE_WATERLOGGING`. Only structure-block entries are removed; listed air is written and unlisted
template cells remain untouched. The one palette is chosen by a position-seeded `nextInt(1)`,
independent of the caller placement RNG. Remaining entries are transformed, clipped to the
processing box and offered with flags `2` in the generic full-block/other/NBT groups, each Y-X-Z
ordered. A listed NBT block is first offered a barrier with flags `820`; only a successful
real-state offer loads its locked NBT. Neighbor-shape repair and block-entity dirtying then follow
the generic template transaction.

**Locked template audit:**

`top` is size `7×5×8`, with 152 listed entries: 94 snow blocks, 38 air, nine white and three
light-gray carpets, two ice and one each crafting table, east unlit empty furnace, closed north/top
oak trapdoor, south red-bed foot/head and lit redstone torch. `middle` is `3×3×3`, with 12 stone
bricks and three north ladders. `bottom` is `7×6×9`, with 244 listed entries: 104 stone bricks, 64
air, 17 mossy and eight cracked stone bricks, seven chiseled stone bricks, nine infested variants,
11 stone, four east-west iron bars, four north ladders, three wall torches, two red carpets, and one
each cobweb, polished andesite, potted cactus, top spruce slab, north/south top stair, south wall
sign, water cauldron level `2`, brewing stand, east chest and `chest` data marker. The locked sign
text points both ways; the brewing stand contains one weakness splash potion; the chest begins
empty. None has a jigsaw.

The bottom template also owns two entity records at local block positions `(2,1,1)` and `(4,1,1)`: a
persistent plains cleric villager with its two locked novice trades, and a persistent plains cleric
zombie villager. For an entity whose transformed integer block position is in the clip, placement
copies its NBT, replaces `Pos`, removes `UUID`, creates it for `STRUCTURE`, rotates its yaw around
the bottom pivot, preserves pitch and remaining payload, sets body/head yaw, skips `finalizeSpawn`,
and offers it with passengers; decode/create/add failure is nonfatal. These entities do not consume
the caller placement RNG.

After a valid base transaction, the transformed bottom marker is processed only when its own
position is in the clip. It offers air at the marker with flags `3`, ignores that result, reads the
block entity one block below, and, only for a chest, consumes `nextLong` and assigns
`minecraft:chests/igloo_chest` with that seed. Independently, a successfully placed bottom chest is
a randomizable container, so generic NBT loading first consumes its own `nextLong`; the marker seed
replaces the effective loot seed. A failed chest offer can therefore skip the first draw yet seed a
preexisting chest through the marker.

After each top base call, regardless of its return value, the invariant transformed local `(3,0,5)`
candidate at `(chunkMinX+3,H-1,chunkMinZ+5)` reads the block below. If that state is neither air nor
ladder, it offers a snow block with flags `3`, without consulting the processing box; the result is
ignored. Each intersecting-chunk invocation can repeat this same repair against the then-current
world.

**Branches and aborts:**

Stub biome accept/reject; four rotations; basement chance below/equal/above `0.5`; all eight depths;
every piece/processing-box intersection; terrain probes inside/outside the source chunk; each sparse
block outside/inside clip and rejected/accepted barrier/real write; existing fluids and
listed/unlisted air; entity outside/inside clip, create/add success/failure; marker outside/inside
clip, air-write result, absent/wrong/chest block entity; top support air/ladder/other and snow-write
result.

**Constants and randomness:**

Generation draws are rotation, basement chance, then conditional depth. Palette/filter draws use
fresh position-seeded sources. Placement RNG advances once only for a successfully installed
randomizable chest block entity, then once for an in-clip marker that observes a chest; entity
restoration and top repair do not advance it. Flags are `2` for template states, `820` for NBT
barriers and `3` for marker air/top snow.

**Side effects:**

One top or a bottom/shaft/top piece list; per-invocation piece-box relocation; sparse block and air
offers, shape/neighbor updates, up to four locked NBT loads plus state-created block entities, two
optional live entities, one optional loot table/seed, and repeated top snow repair. The family has
no spawn override and default `afterPlace` is a no-op.

**Gates:**

Caller-owned structure placement/start/reference lifecycle; center-stub biome; piece intersection
before terrain relocation; live surface height; template availability and clip; processor result;
block write/block-entity type; entity decode; support state.

**Boundary cases and quirks:**

The biome stub `(8,h,8)`, rotation-dependent live surface probe, chunk-min template anchor and
invariant top-repair coordinate are four distinct positions. Counterclockwise rotation probes two
blocks west of the source chunk. Listed air clears blocks even though sparse omissions do nothing.
Base placement reports true for these nonempty valid templates even if every state offer fails, so
in-clip marker/entity work can still run. The top repair is not clip-bounded. Bottom pieces run
before shaft and top pieces in retained-list order.

**Evidence:**

`Confirmed`; `OFF-SERVER-001`; `OFF-DATA-001`; `OFF-REPORT-001`. Anchors:
`net.minecraft.world.level.levelgen.structure.Structure#onTopOfChunkCenter`,
`net.minecraft.world.level.levelgen.structure.Structure#isValidBiome`,
`net.minecraft.world.level.levelgen.structure.structures.IglooStructure#findGenerationPoint`,
`net.minecraft.world.level.levelgen.structure.structures.IglooStructure#generatePieces`,
`net.minecraft.world.level.levelgen.structure.structures.IglooPieces#addPieces`,
`net.minecraft.world.level.levelgen.structure.structures.IglooPieces$IglooPiece#makeSettings`,
`net.minecraft.world.level.levelgen.structure.structures.IglooPieces$IglooPiece#makePosition`,
`net.minecraft.world.level.levelgen.structure.structures.IglooPieces$IglooPiece#postProcess`,
`net.minecraft.world.level.levelgen.structure.structures.IglooPieces$IglooPiece#handleDataMarker`,
`net.minecraft.world.level.levelgen.structure.TemplateStructurePiece#postProcess`,
`net.minecraft.world.level.levelgen.structure.templatesystem.StructureTemplate#placeInWorld`,
`net.minecraft.world.level.levelgen.structure.templatesystem.StructureTemplate#placeEntities`,
`net.minecraft.world.level.levelgen.structure.templatesystem.BlockIgnoreProcessor#processBlock`,
`data/minecraft/worldgen/structure/igloo.json`, `data/minecraft/worldgen/structure_set/igloos.json`,
`data/minecraft/tags/worldgen/biome/has_structure/igloo.json`, and all three
`data/minecraft/structure/igloo/*.nbt` inputs.

**Test vectors:**

Cross negative/positive chunks, four center-versus-probe biome/height combinations, all rotations,
chance endpoint and eight depths; assert bottom/shaft-top-to-bottom/top piece order, origins, boxes
and all audited sparse counts. For each piece cross clip edges, every NBT write outcome, entity
transform/create/add outcome, chest/marker draw combination, top-support class and repeated
intersecting-chunk invocation. Assert exact placement-RNG traces, marker loot key, flags,
preserved/overridden entity fields and lack of family-local placement claim; use `EXP-WGEN-001` only
for separately owned distribution/locate equivalence.
