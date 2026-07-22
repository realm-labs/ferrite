# Blocks mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `BLK-AIR-001` — Three air states share empty mechanics but retain distinct source roles

**Parent:** `BLK-001`, `BLK-002`, `BLK-003`, `BLK-005`, `PLY-006`, `ITM-001`, `WGEN-003`,
`ENV-001`, `CLI-006`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — the locked registrations, `AirBlock`, chunk access/storage paths, generation
callers and client assets fix the three block identities and their shared empty behavior. The item
registration and optional stack codec independently fix `minecraft:air` as a plain-item empty-stack
sentinel rather than a placeable block item.

**Applies when:**

`minecraft:air`, `minecraft:cave_air` or `minecraft:void_air` is read, written, counted, broken,
removed, generated, saved, loaded or projected, and whenever the `minecraft:air` item identity is
decoded, tested for emptiness or encoded by the optional item-stack codec.

**Authoritative state:**

The three blocks each have one property-free state and no block entity: ordinary air is state 0,
void air is 15292 and cave air is 15293. All three use `AirBlock` and properties marked
replaceable, no collision, no loot table and air. Their cached `BlockState.isAir()` value is true,
their fluid state is empty, `LevelReader.isEmptyBlock` accepts them, their collision and selection
shapes are empty, and their render shape is `INVISIBLE`.

All three are direct members of `minecraft:air` and `minecraft:replaceable`. Only ordinary air is
also a direct member of `minecraft:parrots_spawnable_on` in the locked data. These tag differences
do not alter the code-level `isAir` flag.

`Items.AIR` is registered as a plain common-rarity `Item` with maximum stack size 64 and the block
description prefix; it is not a `BlockItem` and has no placement transaction. `ItemStack.isEmpty()`
returns true for that identity regardless of a positive count or component patch, in addition to
the singleton-empty and nonpositive-count cases.

**Transition and ordering:**

Ordinary air is the canonical empty replacement. `Level.removeBlock` writes the current fluid's
legacy block, which is ordinary air for an empty fluid cell. `Level.destroyBlock` reads the current
state and returns false immediately for any of the three air states, before a level event, loot or
replacement. The states add no scheduled tick, random tick, use, neighbor, entity-contact,
redstone or comparator callback.

Every `LevelChunkSection` block counter ignores states whose `isAir()` is true. A section containing
only any mixture of the three therefore has nonempty count zero and `hasOnlyAir()` true. Normal
`LevelChunk#getBlockState` short-circuits such a section to ordinary air without reading its palette;
`ProtoChunk` does the same for an in-range all-air section. A mixed section reads the exact palette
state, so cave and void air remain distinguishable beside a non-air cell. Chunk snapshot/storage
copies every in-range section and its palette rather than dropping a zero-count section, preserving
those exact encoded identities even when the live all-air read shortcut projects ordinary air.

`Level#getBlockState` returns void air outside valid build/horizontal bounds. `EmptyLevelChunk`
always returns void air, and `ProtoChunk` returns void air outside build height. In-range empty or
all-air section shortcuts return ordinary air. Void air is also a real registry state, so an
explicit state write can store it; the synthetic read role does not make its identity unwriteable.

Cave air is the explicit cavity state in three locked generation paths. `NetherWorldCarver` writes
lava at or below `minGenY + 31` and cave air above that boundary when its replacement gate admits
the cell. `LakeFeature` and `MonsterRoomFeature` use cave air for their owned empty-volume writes.
Other carvers and structures retain their own air/aquifer choices; the shared `AirBlock` class does
not globally convert generation output to cave air.

The optional item-stack decoder first reads a count. Nonpositive values produce the singleton empty
stack without an item or patch; a positive value reads the item holder and component patch, so a
forged positive-count AIR value can be constructed. Every subsequent `isEmpty()` gate nevertheless
treats it as empty. The matching encoder calls `isEmpty()` first and writes only count zero for AIR,
discarding its positive count, item identity and patch from that outbound representation. This
normalization applies wherever the locked optional stack codec is used, including container,
creative-slot, inventory, metadata and equipment traffic.

**Client projection:**

All three blockstate assets select `minecraft:block/air`. That model contains only the missing-texture
particle reference, and `INVISIBLE` plus empty shape submits no ordinary world geometry. The client
may still receive an exact cave-air or void-air state ID from a mixed section or block update; its
render result remains empty.

The air item has an item definition and particle-only `minecraft:item/air` model, but an ordinary
inventory, cursor, equipment or held-stack path first treats `Items.AIR` as empty. The optional
outbound codec consequently projects it as the same zero-count absence as `ItemStack.EMPTY`, not as
a visible item model.

**Branches and aborts:**

In-bounds versus out-of-bounds reads; empty/all-air versus mixed sections; exact palette storage
versus ordinary-air read collapse; explicit state writes versus removal; destroy versus replace;
the Nether lava boundary versus cave air; AIR identity versus other item identities; nonpositive
versus positive counts; and inbound construction versus outbound optional normalization are
separate observable branches.

**Constants and randomness:**

State IDs are 0, 15292 and 15293. Nether cave air begins strictly above `minGenY + 31`. AIR's item
maximum is 64; `ItemStack.isEmpty()` has an AIR-identity branch independent of count, and the
optional encoder writes VarInt zero for every empty stack. This leaf consumes no RNG; generation
owners retain any draws that select positions before these fixed state choices.

**Side effects:**

Empty-state reads and counts; ordinary-air removal writes; destroy short-circuit; exact mixed-section
palette continuity; cave-air generation writes; void-air synthetic reads; no block loot, collision,
selection or model; AIR item empty-stack normalization and omission of its positive item/patch data
from optional outbound encoding.

**Gates:**

World bounds; section index and nonempty count; palette membership; generic state-write admission;
generation replacement and height gates; item identity and signed count; strict item/component
holder decode; and every caller's ordinary empty-stack admission rule.

**Boundary cases and quirks:**

An all-cave-air or all-void-air section can preserve an exact palette in storage while ordinary
chunk reads return ordinary air; adding one non-air state makes the exact air variants readable.
Out-of-bounds void air and explicitly stored void air have the same state identity but different
origins. Empty collision and selection do not erase the registry/state distinction. A positive
AIR stack is syntactically decodable but semantically empty, and optional re-encoding is lossy by
design. AIR's generated asset does not make it holdable or placeable through normal stack paths.

**Evidence:**

`OFF-SERVER-001`, `OFF-CLIENT-001`, `OFF-REPORT-001`, `OFF-DATA-001`;
`net.minecraft.world.level.block.Blocks`,
`net.minecraft.world.level.block.AirBlock#getRenderShape`,
`net.minecraft.world.level.block.AirBlock#getShape`,
`net.minecraft.world.item.Items`,
`net.minecraft.world.item.ItemStack#isEmpty`,
`net.minecraft.world.item.ItemStack$1#decode`,
`net.minecraft.world.item.ItemStack$1#encode`,
`net.minecraft.world.level.Level#getBlockState`,
`net.minecraft.world.level.Level#removeBlock`,
`net.minecraft.world.level.Level#destroyBlock`,
`net.minecraft.world.level.LevelReader#isEmptyBlock`,
`net.minecraft.world.level.chunk.LevelChunk#getBlockState`,
`net.minecraft.world.level.chunk.ProtoChunk#getBlockState`,
`net.minecraft.world.level.chunk.EmptyLevelChunk#getBlockState`,
`net.minecraft.world.level.chunk.LevelChunkSection$1BlockCounter#accept`,
`net.minecraft.world.level.chunk.storage.SerializableChunkData#copyOf`,
`net.minecraft.world.level.levelgen.carver.NetherWorldCarver#carveBlock`,
`net.minecraft.world.level.levelgen.feature.LakeFeature#place`,
`net.minecraft.world.level.levelgen.feature.MonsterRoomFeature#place`;
`reports/blocks.json#{minecraft:air,minecraft:cave_air,minecraft:void_air}`,
`reports/minecraft/components/item/air.json`,
`data/minecraft/tags/block/{air,replaceable,parrots_spawnable_on}.json`,
`assets/minecraft/blockstates/{air,cave_air,void_air}.json`,
`assets/minecraft/models/{block,item}/air.json`,
`assets/minecraft/items/air.json`.

**Test vectors:**

Run `EXP-BLK-030` across all three states in all-air and mixed sections, in/out-of-bounds reads,
destroy/remove/explicit writes, save/reload, the three cave-air generators and client rendering.
Round-trip counts -1, 0, 1 and 64 for AIR and an ordinary item, with and without component patches,
through optional container, creative-slot and equipment stack paths; require positive AIR input to
behave empty and every outbound AIR value to encode as the zero-count form.
