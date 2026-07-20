# World mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `WGEN-STRUCTURE-NETHER-FOSSIL-001` — Nether fossils scan for a cavity, place one bone template, and may add a dried ghast

**Parent:** `WGEN-003`

**FidelityClass:** `EquivalentPlayerVisibleBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked 26.2 server source fixes anchor sampling, the downward cavity/support
scan, rotation/template selection, template processing and the dried-ghast postpass; all 14 locked
template payloads and the structure/structure-set records are audited data-only. Shared
random-spread admission remains owned by `WGEN-PIPELINE-001`.

**Applies when:**

The `minecraft:nether_fossil` type has passed caller-owned structure-set placement, or its retained
one-piece start is placed into an intersecting chunk.

**Authoritative state:**

The structure record selects exactly `soul_sand_valley`, no spawn overrides,
`underground_decoration`, `beard_thin`, and uniform height from absolute `32` through two below
generation top inclusive. Its structure set contains this one structure at weight `1` with
random-spread spacing `2`, separation `1`, salt `14357921`, and default frequency fields. A start
owns one rotated `NetherFossilPiece` at the accepted anchor and one of
`nether_fossils/fossil_1..14`.

**Transition and ordering:**

The anchor phase first draws local X then Z with `nextInt(16)` from the structure RNG. It resolves
the uniform bounds and, when ordered, consumes `nextInt(max-min+1)` to choose Y inclusively; an
inverted hostile range returns the minimum without a draw and warns once per packed bound pair. It
then obtains the base noise column at X/Z. While Y is strictly above generator sea level, it reads
the state at Y, decrements Y, then reads the state at the decremented Y. The scan stops only when
the upper state is air and the lower state is either soul sand or has a sturdy upper face against
`EmptyBlockGetter`; soul sand short-circuits the sturdy-face query. Every other pair continues
downward. A stopped or exhausted Y at or below sea level rejects the start. Otherwise `(X,Y,Z)` is
both generation stub and template anchor; the generic valid-biome gate samples its quart coordinate
before pieces materialize.

**Template transition:**

The deferred piece builder continues the same structure RNG, first choosing rotations
`[none, clockwise_90, 180, counterclockwise_90]` with `nextInt(4)`, then template indices `1..14`
with `nextInt(14)`. Settings use that rotation, no mirror, and the `STRUCTURE_AND_AIR` ignore
processor. The processor returns null for air or structure-block identities and otherwise preserves
the transformed block info. All 14 locked payloads have a three-state palette, a complete
rectangular block list, no entities, no block NBT, no structure blocks and no jigsaws: every nonair
entry is a bone block whose X/Y/Z axis rotates with the template. Consequently the base template
transaction offers only those bone states, with flags `2`, in locked list order; its data-marker
callback is empty.

Before base placement, the piece computes its rotated template box and mutates the caller's
processing box to encapsulate that whole box. It then sets that expanded box as the template clip
and recomputes its piece box, so every invocation can offer the entire template rather than only the
original chunk slice. Marker/jigsaw scans occur only when template placement reports true, but find
nothing in these payloads. The Nether-fossil override runs its dried-ghast postpass after the base
call regardless of that placement result.

**Locked template audit:**

Each tuple is `id: size; Y/X/Z-axis bone counts`; all remaining cells are ignored air:
`1:4×4×5;9/1/0`, `2:5×1×5;0/5/5`, `3:3×4×2;4/2/0`, `4:3×4×1;5/1/0`, `5:2×5×1;4/1/0`,
`6:7×5×5;18/3/0`, `7:4×6×5;13/5/0`, `8:3×5×1;4/2/0`, `9:3×5×5;9/6/0`, `10:3×7×1;6/2/0`,
`11:5×5×7;18/6/0`, `12:4×4×3;7/4/0`, `13:4×5×6;11/6/0`, `14:7×7×6;21/5/0`. This accounts for all
1,194 listed cells: 183 bone and 1,011 air.

**Dried-ghast postpass:**

Every piece-placement invocation creates a fresh random source from world seed, forks its positional
factory, and selects the stream at the rotated template box center. A first float must be strictly
below `0.5`. Admission then draws X uniformly across the box span, fixes Y to box minimum, and draws
Z uniformly across its span. It reads that candidate and requires air, then checks membership in the
already expanded processing box. Only then `nextInt(4)` uniformly rotates the default dried-ghast
state and offers it with flags `2`, ignoring the result. The default keeps hydration `0` and
waterlogged `false`; rotation changes its North facing. Because the stream is recreated from the
same world seed and box center, repeated invocations reproduce the same chance and candidate. A
successful prior offer, or a bone written there by the template, makes the later air gate fail
before the facing draw.

**Branches and aborts:**

Ordered/inverted height range; sampled Y at/below/above sea level; every air/nonair upper and
soul-sand/sturdy/nonsturdy lower pair; biome accept/reject; four rotations and 14 templates;
template placement true/false; repeated/full-box placement; dried-ghast chance below/equal/above
`0.5`, candidate bone/air, inside/outside hostile box and accepted/rejected write.

**Constants and randomness:**

Anchor draws are X, Z, height; accepted piece draws are rotation then template. Base placement does
not advance that caller-supplied structure RNG: its single-palette choice instead creates a
position-seeded random source and calls `nextInt(1)`, the ignore processor has no draw, and these
payloads have no entities, markers or jigsaws. The independent postpass draws chance, X, Z, then
conditional facing.

**Side effects:**

One retained piece and rotated box; bone-block offers; mutation of the invocation's processing box
and place-settings clip; and an optional default dried ghast. No block entity, entity, loot table or
structure spawn override is created.

**Gates:**

Caller-owned structure placement/start/reference lifecycle; generation height/sea level; base-column
pair; exact biome; template availability; base placement result for marker scans only; world-seed
positional chance and post-template air.

**Boundary cases and quirks:**

The accepted anchor is the lower/support cell, not the air cell above. Absolute `32` can be sampled
but cannot pass when sea level is `32`. The full-template box expansion also makes the dried-ghast
membership check true for every ordinary sampled candidate. Dried-ghast selection is independent of
the caller's structure RNG and deterministic per world seed plus rotated box center. Base-placement
failure does not suppress that postpass.

**Evidence:**

`Confirmed`; `OFF-SERVER-001`; `OFF-DATA-001`; `OFF-REPORT-001`. Anchors:
`net.minecraft.world.level.levelgen.structure.structures.NetherFossilStructure#findGenerationPoint`,
`net.minecraft.world.level.levelgen.heightproviders.UniformHeight#sample`,
`net.minecraft.world.level.levelgen.VerticalAnchor$BelowTop#resolveY`,
`net.minecraft.world.level.chunk.ChunkGenerator#getBaseColumn`,
`net.minecraft.world.level.NoiseColumn#getBlock`,
`net.minecraft.world.level.levelgen.structure.structures.NetherFossilPieces#addPieces`,
`net.minecraft.world.level.levelgen.structure.structures.NetherFossilPieces$NetherFossilPiece#postProcess`,
`net.minecraft.world.level.levelgen.structure.structures.NetherFossilPieces$NetherFossilPiece#placeDriedGhast`,
`net.minecraft.world.level.levelgen.structure.TemplateStructurePiece#postProcess`,
`net.minecraft.world.level.levelgen.structure.templatesystem.StructureTemplate#placeInWorld`,
`net.minecraft.world.level.levelgen.structure.templatesystem.StructurePlaceSettings#getRandomPalette`,
`net.minecraft.world.level.levelgen.structure.templatesystem.StructurePlaceSettings#getRandom`,
`net.minecraft.world.level.levelgen.structure.templatesystem.BlockIgnoreProcessor#processBlock`,
`net.minecraft.world.level.levelgen.structure.BoundingBox#encapsulate`,
`data/minecraft/worldgen/structure/nether_fossil.json`,
`data/minecraft/worldgen/structure_set/nether_fossils.json`,
`data/minecraft/tags/worldgen/biome/has_structure/nether_fossil.json`, and all 14
`data/minecraft/structure/nether_fossils/fossil_*.nbt` inputs.

**Test vectors:**

Cross local-coordinate endpoints and negative chunks; all height/sea-level endpoints and inverted
contexts; air/support pair prefixes and short circuits; biome mismatch; every rotation/template;
template availability and placement result; rotated axis/box endpoints; repeated invocations from
every intersecting chunk; and all dried-ghast chance/candidate/write branches. Assert RNG
ownership/order, exact template inventory/counts, full-box mutation, flags `2`, empty
marker/jigsaw/entity work, deterministic positional replay and postpass independence from base
success. Use `EXP-WGEN-001` only for separately owned placement/distribution equivalence.
