# World mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `WGEN-STRUCTURE-SWAMP-HUT-001` — Swamp huts terrain-average a fixed cabin and latch two occupants

**Parent:** `WGEN-003`

**FidelityClass:** `EquivalentPlayerVisibleBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked 26.2 server source fixes the start stub, one-piece geometry, terrain
average, ordered block/support writes, direct witch/cat attempts and finalization; the structure,
set, cat-variant and sound-variant inputs are audited data-only. Shared random-spread admission and
generic spawn-override selection remain owned by `WGEN-PIPELINE-001`.

**Applies when:**

The `minecraft:swamp_hut` type has passed caller-owned structure-set placement, or its retained
`minecraft:swamp_hut` piece intersects a placement chunk.

**Authoritative state:**

The structure record selects only `minecraft:swamp`, uses `surface_structures`, default `none`
terrain adaptation, and advertises two piece-box spawn overrides: creature cat and monster witch,
each weight `1` and count exactly `1`. Its set contains only swamp hut at weight `1`, with
random-spread spacing `32`, separation `8`, salt `14357620`, and default frequency fields. The piece
persists its bounding box, horizontal orientation, width `7`, height `7`, depth `9`, cached `HPos`,
and independent `Witch`/`Cat` booleans.

**Transition and ordering:**

The generic top-of-chunk-center stub samples `WORLD_SURFACE_WG` first occupied height at local
`(8,8)` and the valid-biome gate samples that exact 3-D quart position. Deferred generation adds one
piece anchored at `(chunkMinX,64,chunkMinZ)` and consumes `nextInt(4)` to select
`[north,east,south,west]`. North/south produce a `7×7×9` box and east/west a `9×7×7` box; because
either footprint begins at the source chunk minimum and spans at most nine cells, ordinary placement
keeps it wholly inside that chunk.

When `HPos<0`, a placement invocation visits its current world box in Z-major then X order at probe
Y `64`, retaining only probe positions inside the processing box. It reads each retained
`MOTION_BLOCKING_NO_LEAVES` height, stores the truncating integer mean `H`, and moves the box so its
minimum Y becomes `H`. Having no retained probes aborts the entire invocation without storing or
writing. Once `HPos>=0`, later invocations never resample. A negative mean moves the box but remains
sentinel-negative, so a later invocation samples and moves it again; a serialized piece missing
`HPos` instead defaults to cached `0`. Ordinary chunk placement includes all 63 footprint columns; a
hostile partial or Y-excluding box can abort, cache a nonnegative subset mean, or repeatedly
resample a negative mean.

After alignment, these inclusive cuboids are offered in listed order, each with Y outermost, X
middle and Z innermost, the same shell/interior state, and no air skip: spruce planks
`(1,1,1)..(5,1,7)`, `(1,4,2)..(5,4,7)`, `(2,1,0)..(4,1,0)`, `(2,2,2)..(3,3,2)`, `(1,2,3)..(1,3,6)`,
`(5,2,3)..(5,3,6)`, `(2,2,7)..(4,3,7)`; then oak logs `(1,0,2)..(1,3,2)`, `(5,0,2)..(5,3,2)`,
`(1,0,7)..(1,3,7)`, `(5,0,7)..(5,3,7)`. These are 94 plank and 16 log offers before overlap. Singles
follow: oak fences `(2,3,2)`, `(3,3,7)`; air `(1,3,4)`, `(5,3,4)`, `(5,3,5)`; potted red mushroom
`(1,3,5)`; crafting table `(3,2,6)`; empty cauldron `(4,2,6)`; and oak fences `(1,2,1)`, `(5,2,1)`.

The roof then offers north-facing spruce stairs across `(0,4,1)..(6,4,1)`, east-facing across
`(0,4,2)..(0,4,7)`, west-facing across `(6,4,2)..(6,4,7)`, and south-facing across
`(0,4,8)..(6,4,8)`. Four later corner overrides set north outer-right at `(0,4,1)`, north outer-left
at `(6,4,1)`, south outer-left at `(0,4,8)` and south outer-right at `(6,4,8)`. All direct states
are mirror/rotation transformed, clipped, offered with flags `2`, and ignore the write result. After
each in-clip offer, the resulting fluid is read and any nonempty fluid receives a delay-`0` tick
even after a rejected write; each oak-fence offer also marks its position for shape postprocessing.

Next, in Z order `2,7` and nested X order `1,5`, each transformed local `(X,-1,Z)` that is inside
the processing box starts a downward oak-log column. The loop offers flags-`2` logs while the
current state is air, liquid, glow lichen, seagrass or tall seagrass and Y is strictly above
`level.minY+1`; it stops at the first other state or the floor guard. Only the starting position is
clip-tested, so accepted columns continue below the processing box without further membership
checks. These writes do not use the direct-state fluid/shape follow-up.

Finally the piece attempts a witch and then a cat at the same transformed local `(2,2,5)`. Each
attempt runs only while its persisted flag is false and that position is in the processing box, then
sets the flag *before* entity creation. A null creation or failed/nonretained add therefore never
retries. A created occupant is persistent, snapped to block-center X/Z and integer Y with yaw/pitch
`0`, finalized for `STRUCTURE` at current local difficulty, and added with passengers. `STRUCTURE`
makes the witch raid-joinable but neither a patrol leader nor patrolling; it receives no ominous
banner. Both mob base finalizers add the absent follow-range random-spawn modifier from
`triangle(0,0.11485)` and set left-handed from a `<0.05` float using the level RNG.

The cat then builds a spawn context at its snapped block. In ordinary placement, the valid swamp-hut
start contains that position, so the locked `#minecraft:cats_spawn_as_black` structure condition
matches at priority `1`, removes every priority-`0` candidate, and selection consumes `nextInt(1)`
to install `all_black`. If a hostile structure manager cannot return that valid start, the
priority-`0` pool instead contains the ten unconditional variants and also `all_black` only when
moon brightness is at least `0.9`; selection is uniform over the resulting ten or eleven entries. It
finally consumes a uniform registry draw across locked cat sound variants `classic` and `royal`.
Both share the nine baby sounds; their nine adult sounds use the classic and royal sound families
respectively.

**Branches and aborts:**

Stub biome accept/reject; four directions; uncached/cached and negative/nonnegative height, missing
persisted `HPos`, zero/full/partial probe membership; every direct cell outside/inside clip and
rejected/accepted write; postwrite empty/nonempty fluid; four support starts outside/inside clip,
every replaceable/terminal state and minimum-Y edge; each persisted spawn flag, spawn position
outside/inside clip, entity null/non-null and add outcome; valid/missing structure membership, moon
brightness below/equal/above `0.9`, and both sound variants.

**Constants and randomness:**

Structure generation consumes only the direction `nextInt(4)`; `postProcess` does not use its
caller-supplied placement RNG. Occupant creation/finalization uses entity/level randomness: witch
follow-range triangle then handedness float; cat follow-range triangle, handedness float, variant
selection and sound selection. Direct occupant attempts are distinct from the structure record's
later generic piece-box spawn overrides.

**Side effects:**

One retained/cached piece; 150 ordered fixed-layout offers before four variable-depth supports;
fluid ticks and fence postprocessing; persisted height and occupant latches; up to one persistent
witch and one persistent cat with finalized attributes/variant/sounds. No loot table, block NBT or
family-specific `afterPlace` work exists.

**Gates:**

Caller-owned structure placement/start/reference lifecycle; center-stub biome; processing-box
membership at Y `64`, each block, each support start and each occupant; live heightmap; replaceable
support material; entity creation; structure-manager lookup and dynamic cat inputs.

**Boundary cases and quirks:**

The center stub, 63-column terrain average and rotated occupant position are distinct observations.
The footprint never crosses its source chunk in ordinary generation, but the cached-height helper
still exposes hostile partial-box and negative-sentinel behavior. Downward supports are
start-clipped rather than cell-clipped. A rejected fixed-layout write may still schedule the fluid
found afterward. Spawn flags latch before creation rather than after successful addition. Normal hut
cats are deterministically black in identity while still consuming a singleton-selection draw; their
sound family remains random.

**Evidence:**

`Confirmed`; `OFF-SERVER-001`; `OFF-DATA-001`; `OFF-REPORT-001`. Anchors:
`net.minecraft.world.level.levelgen.structure.Structure#onTopOfChunkCenter`,
`net.minecraft.world.level.levelgen.structure.structures.SwampHutStructure#findGenerationPoint`,
`net.minecraft.world.level.levelgen.structure.structures.SwampHutPiece#postProcess`,
`net.minecraft.world.level.levelgen.structure.structures.SwampHutPiece#spawnCat`,
`net.minecraft.world.level.levelgen.structure.ScatteredFeaturePiece#updateAverageGroundHeight`,
`net.minecraft.world.level.levelgen.structure.StructurePiece#placeBlock`,
`net.minecraft.world.level.levelgen.structure.StructurePiece#fillColumnDown`,
`net.minecraft.world.entity.raid.Raider#finalizeSpawn`,
`net.minecraft.world.entity.monster.PatrollingMonster#finalizeSpawn`,
`net.minecraft.world.entity.Mob#finalizeSpawn`,
`net.minecraft.world.entity.animal.feline.Cat#finalizeSpawn`,
`net.minecraft.world.entity.variant.VariantUtils#selectVariantToSpawn`,
`net.minecraft.world.entity.variant.PriorityProvider#pick`,
`net.minecraft.world.entity.variant.StructureCheck#test`,
`net.minecraft.world.entity.animal.feline.CatSoundVariants#pickRandomSoundVariant`,
`data/minecraft/worldgen/structure/swamp_hut.json`,
`data/minecraft/worldgen/structure_set/swamp_huts.json`,
`data/minecraft/tags/worldgen/biome/has_structure/swamp_hut.json`,
`data/minecraft/tags/worldgen/structure/cats_spawn_as_black.json`, all 11
`data/minecraft/cat_variant/*.json` inputs and both `data/minecraft/cat_sound_variant/*.json`
inputs.

**Test vectors:**

Cross negative/positive chunks, center-stub biome and live height differences, all four directions,
all 63 height samples, truncation, empty/full/adversarial partial boxes and cached replay. Assert
every ordered cuboid/single/roof/corner offer, transform, overlap, flags, fluid and shape side
effect; cross all four support columns against clip, state and minimum-Y boundaries. Exhaust both
spawn latches, creation/add failures, finalizer RNG order, raid/patrol flags, structure/moon variant
priorities and both sound IDs; use `EXP-WGEN-001` only for separately owned distribution/locate
equivalence.
