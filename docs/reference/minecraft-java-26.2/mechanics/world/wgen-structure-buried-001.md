# World mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `WGEN-STRUCTURE-BURIED-001` — Buried treasure descends one anchored piece and encloses a loot chest

**Parent:** `WGEN-003`

**FidelityClass:** `EquivalentPlayerVisibleBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked 26.2 server source fixes the generation anchor, biome sample, piece
origin, support search, six-neighbor enclosure, chest orientation and loot initialization; locked
data fixes its structure and structure-set records. The shared random-spread placement algorithm
remains owned by `WGEN-PIPELINE-001`.

**Applies when:**

The `minecraft:buried_treasure` structure type has passed the caller-owned structure-set placement
gate, or its retained start is placed into an intersecting chunk.

**Authoritative state:**

The structure record selects beach biomes, no spawn overrides, `underground_structures`, and default
terrain adaptation `none`. A new start owns exactly one single-cell buried-treasure piece. Its
initial block is chunk-local `(9,90,9)`; the generation stub separately records the chunk middle
`(8,h,8)`, where `h` is the generator's first occupied `OCEAN_FLOOR_WG` height. The generic biome
gate samples the configured biome source at the stub's quart X/Y/Z and requires the structure's
biome holder set. The locked `has_structure/buried_treasure` tag resolves through `is_beach` to
exactly `beach` and `snowy_beach`.

**Transition and ordering:**

Piece placement discards the stored Y `90` for terrain search. At the piece X/Z it queries
`OCEAN_FLOOR_WG`, starts at that returned Y, and descends while Y is strictly greater than level
minimum. At each candidate it reads the current state, then the state below. Only below-block
identity `sandstone`, `stone`, `andesite`, `granite`, or `diorite` admits the chest transaction; any
other below state moves one cell down. Reaching minimum Y without an admitted support returns
without mutation or RNG.

On admission, the enclosure state is sand when the current cell is air, water, or lava by block
identity; otherwise it is that exact current state. The piece then visits all six enum directions in
order `DOWN, UP, NORTH, SOUTH, WEST, EAST`. Each neighbor is read. A neighbor that is neither air
nor water/lava is preserved and performs no further read. For an empty/liquid neighbor, the block
below that neighbor is read. If this second state is air or water/lava and the direction is not
`UP`, the neighbor is offered the admitted support state; otherwise it is offered the enclosure
state. Offers use flags `3` and ignore write results. Thus the downward neighbor can be rewritten
with the support state when the block two below is empty/liquid, while an empty/liquid upward
neighbor always receives the enclosure state. Earlier offers are visible to later reads.

The piece replaces its persisted bounding box with the final one-cell candidate, then invokes the
shared chest helper for the current chunk's processing box. A candidate outside that box or already
containing a chest aborts chest creation. Otherwise the default chest is reoriented from horizontal
neighbors: any adjacent chest retains its default orientation; exactly one solid-render neighbor
faces the chest away from that neighbor; multiple solid neighbors fall through to the helper's
deterministic default-facing avoidance sequence. It offers the chest with flags `2` and ignores the
result, then rereads the block entity. A resulting `ChestBlockEntity` alone consumes one `nextLong`
and stores `minecraft:chests/buried_treasure` with that seed. Missing/wrong block entities consume
no RNG. The helper reports success after an admitted offer regardless of write or block-entity
result, and the piece ignores that report.

**Locked placement input:**

`structure_set/buried_treasures` contains this one structure at weight `1` and a `random_spread`
record with spacing `1`, separation `0`, salt `0`, frequency `0.01`, `legacy_type_2` reduction and
locate offset `[9,0,9]`. These values are exact data-only inputs; this leaf does not infer the
shared placement draw, seed or locate semantics from them.

**Branches and aborts:**

Placement admitted/rejected outside this leaf; biome accepted/rejected; support at first/deeper/no
candidate; current air/water/lava/solid; every neighbor solid/air/liquid and below-neighbor
solid/air/liquid; `UP` versus other directions; accepted/rejected enclosure writes; processing-box
miss, preexisting chest, accepted/rejected chest write; chest/wrong/missing block entity.

**Constants and randomness:**

Stub local `(8,h,8)`, piece local `(9,90,9)`, five support blocks, exact six-direction order, flags
`3` for enclosure and `2` for chest. Generation and enclosure consume no family-local RNG. Exactly
one `nextLong` is consumed only after chest admission when the postwrite block entity is a chest.
Placement RNG remains outside this leaf.

**Side effects:**

The piece's final bounding box, up to six enclosure offers, one chest offer, and optional loot-table
key/seed. The family creates no spawn override and its default `afterPlace` is a no-op.

**Gates:**

Caller-owned structure-set admission and start/reference lifecycle; stub-biome membership; current
chunk/piece intersection; build minimum; support whitelist; per-neighbor material; processing
bounding box; block-entity type.

**Boundary cases and quirks:**

The biome sample uses the `(8,h,8)` stub while the piece searches at `(9,*,9)`. Initial Y `90` does
not bound the terrain search. Only exact water/lava blocks count as liquid here; arbitrary nonempty
fluid states do not. Failed enclosure writes can change later reads only if the world nevertheless
mutates. A failed chest offer may still seed a preexisting chest block entity exposed by the world
accessor.

**Evidence:**

`Confirmed`; `OFF-SERVER-001`; `OFF-DATA-001`; `OFF-REPORT-001`. Anchors:
`net.minecraft.world.level.levelgen.structure.Structure#onTopOfChunkCenter`,
`net.minecraft.world.level.levelgen.structure.Structure#findValidGenerationPoint`,
`net.minecraft.world.level.levelgen.structure.structures.BuriedTreasureStructure#findGenerationPoint`,
`net.minecraft.world.level.levelgen.structure.structures.BuriedTreasurePieces$BuriedTreasurePiece#postProcess`,
`net.minecraft.world.level.levelgen.structure.StructurePiece#reorient`,
`net.minecraft.world.level.levelgen.structure.StructurePiece#createChest`,
`net.minecraft.world.level.levelgen.structure.StructureStart#placeInChunk`,
`data/minecraft/worldgen/structure/buried_treasure.json`,
`data/minecraft/worldgen/structure_set/buried_treasures.json`,
`data/minecraft/tags/worldgen/biome/has_structure/buried_treasure.json`, and
`data/minecraft/tags/worldgen/biome/is_beach.json`.

**Test vectors:**

Cross negative/positive chunk coordinates; distinct `(8,h,8)` and `(9,*,9)` biomes; every height and
minimum-Y endpoint; all five support identities plus state-property variants and near misses; every
current/neighbor/below-neighbor material mask; all six direction positions; accepted/rejected
writes; processing-box rejection; preexisting chest; all horizontal solid/chest masks; and absent,
wrong or chest block entities. Assert exact height/read/write order, final one-cell bounding box,
orientation, conditional lone `nextLong`, loot key and no family-local placement claim. Use
`EXP-WGEN-001` only to calibrate the separately owned distribution/locate equivalence.
