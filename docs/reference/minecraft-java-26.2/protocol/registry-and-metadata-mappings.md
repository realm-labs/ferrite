# 26.2 Wire Registry and Palette Mappings

This page owns numeric mappings that are visible in multiple play packet families. They are
versioned wire projections only. Ferrite's authoritative block, biome, block-entity, entity, item,
component, and metadata identities remain namespaced/domain types and may not persist these raw
numbers.

## C2 terrain registries

The locked global block-state table has exactly 32,366 entries with contiguous raw IDs
`0..=32_365`. `reports/blocks.json` records the ID beside every exact property combination; for
example air is `0` and stone is `1`. The table is `Block.BLOCK_STATE_REGISTRY`, distinct from the
1,196-entry block registry. Chunk palettes and later block-update packets resolve raw state IDs
against this exact state table. An absent single/local-palette raw ID faults during palette decode;
an absent global-storage index faults when that state is later resolved.

Biome raw IDs are not a fixed table copied from another session. They index the ordered
`minecraft:worldgen/biome` registry reconstructed from configuration `registry_data`. Chunk and
biome-refresh palettes must bind to the same frozen registry snapshot used to bind the play codec;
an absent single/local raw ID faults palette decode and an absent global index faults on use.

Block-entity types use the locked static `minecraft:block_entity_type` registry: 49 entries with
raw IDs `0..=48` from `reports/registries.json`. A full-chunk block-entity entry resolves its type
through that registry before its NBT can be applied. Ferrite maps the result back to a namespaced
type at the adapter boundary.

Primary anchors are `net.minecraft.world.level.block.Block#BLOCK_STATE_REGISTRY`,
`net.minecraft.world.level.chunk.PalettedContainerFactory`,
`net.minecraft.core.registries.Registries`, the locked `OFF-REPORT-001` reports, and the dynamic
registry reconstruction in [login and configuration](login-and-configuration.md).

## Section palette stream

A full chunk contains one section record for every section implied by the configured dimension
height, in bottom-to-top order. Section count and minimum Y are not repeated in the packet. Each
record is:

```text
non_empty_block_count:i16
fluid_count:i16
block_states:paletted_container(4096, global_block_state_table)
biomes:paletted_container(64, configured_biome_registry)
```

Both counts are stored as signed shorts without codec range validation. Each paletted container
starts with a signed byte selecting its storage configuration, followed by a palette and the exact
fixed number of big-endian longs required by that configuration. Values are indexed X-fastest as
`(y << axis_bits | z) << axis_bits | x`. Packed values never straddle a long: a long stores
`floor(64 / bits)` values, and the array length is the ceiling of entry count divided by that
quantity.

Block-state configurations are:

| Selector byte | Palette | In-memory/storage bits | Palette payload |
|---:|---|---:|---|
| `0` | single value | `0` | one global block-state VarInt; no longs |
| `1..=4` | linear local | `4` | count VarInt then that many global state VarInts |
| `5..=8` | hash local | selector value | count VarInt then that many global state VarInts |
| every other signed byte | global | `15` for the locked 32,366 states | no palette payload |

The canonical encoder emits selector `0`, `4..=8`, or `15`; the decoder's wider selector behavior
above is observable. A negative local palette count runs no entry loop: linear palettes retain a
negative size and hash palettes remain empty, so decode can finish but any stored index faults when
resolved. Linear local counts above 16 fault while filling fixed palette storage. Hash-palette
counts are transport-bounded; any stored local index that has no palette entry faults when
resolved.

Biome configurations are selector `0` single value; `1..=3` linear local with that many storage
bits; and every other signed selector global. Global biome storage uses
`ceil(log2(configured_biome_count))` bits regardless of the noncanonical selector byte. Canonical
encoding uses the actual selected local/global bit count. The same negative-count,
missing-entry, and registry-ID rules apply.

`PalettedContainer#read`, `Strategy#createForBlockStates`, `Strategy#createForBiomes`, the four
palette implementations, and `SimpleBitStorage` are the primary anchors.

## Heightmap and block-entity mapping

Full-chunk heightmaps are a VarInt-counted map from heightmap type to VarInt-counted long array.
Wire type IDs are `world_surface_wg=0`, `world_surface=1`, `ocean_floor_wg=2`,
`ocean_floor=3`, `motion_blocking=4`, and `motion_blocking_no_leaves=5`. The official server emits
only client-use types 1, 4, and 5. The locked decoder maps every out-of-range type ID to type 0;
duplicate mapped keys overwrite earlier arrays. A negative outer map count is accepted as empty;
a negative long-array length faults, and a length exceeding the remaining body fails. When an array
length differs from the dimension-derived expected heightmap storage, the client warns, ignores it,
and recomputes that type from decoded block states.

Each full-chunk block-entity entry is one packed X/Z byte (high nibble local X, low nibble local Z),
signed Y short, block-entity-type registry VarInt, and nullable compound NBT. The NBT reader has a
2,097,152-byte accounting quota and depth 512. The client creates the block entity implied by the
decoded block state, then applies non-null NBT only when that entity exists and its type equals the
wire type; a mismatch or null tag is ignored rather than replacing the block-derived entity type.

Primary anchors are `net.minecraft.world.level.levelgen.Heightmap$Types`,
`net.minecraft.network.protocol.game.ClientboundLevelChunkPacketData`,
`net.minecraft.world.level.chunk.LevelChunk#replaceWithPacketData`, and
`net.minecraft.nbt.NbtAccounter#defaultQuota`.
