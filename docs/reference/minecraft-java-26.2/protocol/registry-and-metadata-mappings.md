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

## C2 block-delta registry distinctions

Three different numeric spaces appear in adjacent block-convergence packets and may not be
substituted for one another:

| Packet field | Registry/table | Locked range | Example raw ID `1` |
|---|---|---:|---|
| ID 7 `block_event.block` | static `minecraft:block` registry | `0..=1_195` | `minecraft:stone` |
| IDs 8/84 block state | global exact-state table | `0..=32_365` | stone default state |
| ID 6 block-entity type | static `minecraft:block_entity_type` registry | `0..=48` | `minecraft:chest` |

Block raw IDs name only a registered block type; they carry no property values. Global state IDs
name one exact property tuple from `reports/blocks.json`; air is state `0` and stone's default is
state `1`. Block-entity type ID `0` is `minecraft:furnace`, while ID `1` is
`minecraft:chest`. Coincidental numbers across the three columns have no semantic relationship.

`ClientboundBlockUpdatePacket` resolves the state ID with a throwing mapper during decode.
`ClientboundSectionBlocksUpdatePacket` instead extracts the upper bits of each VarLong, converts
them to int, and uses nullable `Block.BLOCK_STATE_REGISTRY.byId`; an absent value therefore faults
on an immediate write, or can stage null behind prediction until ACK, rather than becoming air.
Block and block-entity registry codecs resolve with throwing registry maps. All mappings bind to
the exact registries bootstrapped for 26.2 and are not affected by the ordered dynamic configuration
registries.

Standalone block-entity ID 6 uses trusted non-null compound NBT, whereas full-chunk block-entity
entries use nullable default-quota NBT. Both decoded type IDs are checked against the client block
entity implied by the current block state before tag application. Ferrite emits each form from a
namespaced authoritative type and version-specific serializer; it never persists or accepts the raw
ID as domain identity.

Primary anchors are `ClientboundBlockEventPacket`, `ClientboundBlockUpdatePacket`,
`ClientboundSectionBlocksUpdatePacket`, `ClientboundBlockEntityDataPacket`,
`BuiltInRegistries.BLOCK`, `BuiltInRegistries.BLOCK_ENTITY_TYPE`, and
`Block.BLOCK_STATE_REGISTRY`.

## C3 entity mappings

The first entity-session packets use three distinct identity domains:

| Field | Domain | Resolution/failure |
|---|---|---|
| damage event source type | ordered dynamic `minecraft:damage_type` registry frozen after configuration | unknown raw ID faults decode |
| respawn dimension type | ordered dynamic `minecraft:dimension_type` registry frozen after configuration | unknown raw ID faults decode |
| attack/interact/animation/damage/camera/pickup entity numbers | current server or client level's connection-local entity table | missing ID follows the owning handler's ignore/fallback path |

The vanilla bootstrap contains 51 damage types and four dimension types, but those counts do not
make their raw IDs global constants. Configuration transmits the selected ordered registries, and
the bound play codec resolves the holder VarInt against that exact snapshot. A data-pack-selected
entry or ordering change must therefore be projected by namespaced key through the session mapping,
not guessed from baseline declaration order. The C3 golden registry fixture deliberately maps
`minecraft:player_attack` damage type and `minecraft:overworld` dimension type to raw ID zero to
prove codec dependence on the supplied snapshot.

Respawn's dimension field and optional last-death dimension are identifiers for level keys, not
dimension-type raw IDs. Damage cause/direct entity IDs use the packet-local `entity_id + 1` bias;
zero represents server absence, decode subtracts one with wrapping signed-int arithmetic, and
missing nonnegative or wrapped results remain unresolved. When a damage source position is present,
the client intentionally ignores both entity references. None of these numeric forms is an entity
type, entity metadata serializer, attribute, item/component, or durable Ferrite entity identity.

ID 1 `add_entity` uses the locked static `minecraft:entity_type` registry from
`reports/registries.json`: exactly 158 contiguous raw IDs `0..=157`. The complete mapping is the
report's `entries[*].protocol_id`; this command emits it in normative wire order without copying a
generated table into Git:

```sh
jq -r '.["minecraft:entity_type"].entries | to_entries
  | sort_by(.value.protocol_id)[]
  | "\(.value.protocol_id)\t\(.key)"' \
  target/mc-reference/26.2/generated/reports/registries.json
```

Locked landmarks are `acacia_boat=0`, `pig=100`, `warden=143`, `player=156`, and
`fishing_bobber=157`. The report's `default=minecraft:pig` belongs to registry/data semantics and is
also the packet fallback. Although the stream codec calls `byIdOrThrow`, the underlying
`DefaultedMappedRegistry#byId` returns pig instead of null for every negative or out-of-range raw ID,
so those values decode as pig. The mapping is static bootstrap order, not the dynamic configuration
registry and not the connection-local entity-number table. Ferrite resolves a namespaced
authoritative type through this exact 26.2 adapter mapping on encode.

Falling-block spawn data instead resolves through the distinct 32,366-entry global block-state
table. An absent state ID becomes air in this one handler; it does not use the throwing block-delta
decoder. Hanging spawn direction is an enum projection, and projectile owner data is a current-level
entity ID. Coincidentally equal numbers across these spaces have no relationship.

The entity-state family adds four more independent numeric domains. ID 99 metadata serializer IDs
are a fixed 43-entry registration table. Recording each row as `id<TAB>static_field`, sorting by ID
and hashing newline-terminated rows gives locked SHA-1
`96047ad220ac7064e205594f3222d182c87591d7`:

| ID | Serializer | Exact value codec |
|---:|---|---|
| `0` | `BYTE` | signed byte |
| `1` | `INT` | VarInt |
| `2` | `LONG` | VarLong |
| `3` | `FLOAT` | big-endian IEEE float |
| `4` | `STRING` | `UTF(32767)` |
| `5` | `COMPONENT` | trusted registry-aware component NBT |
| `6` | `OPTIONAL_COMPONENT` | boolean presence, then serializer 5's value |
| `7` | `ITEM_STACK` | optional item stack and trusted component patch specified by ID 102 |
| `8` | `BOOLEAN` | one boolean byte |
| `9` | `ROTATIONS` | X, Y and Z big-endian IEEE floats |
| `10` | `BLOCK_POS` | packed block-position signed long |
| `11` | `OPTIONAL_BLOCK_POS` | boolean presence, then serializer 10's value |
| `12` | `DIRECTION` | VarInt data ID; out-of-range values wrap across down/up/north/south/west/east |
| `13` | `OPTIONAL_LIVING_ENTITY_REFERENCE` | boolean presence, then UUID as two signed longs |
| `14` | `BLOCK_STATE` | global block-state VarInt ID; absent IDs decode null |
| `15` | `OPTIONAL_BLOCK_STATE` | VarInt zero is absent; any nonzero value resolves through `Block.stateById`, whose absent fallback is air |
| `16` | `PARTICLE` | static particle-type VarInt followed by that type's exact registry-aware options codec |
| `17` | `PARTICLES` | nonnegative VarInt count followed by serializer-16 values |
| `18` | `VILLAGER_DATA` | ordered dynamic villager-type holder, profession holder and level VarInt |
| `19` | `OPTIONAL_UNSIGNED_INT` | zero absent; otherwise decoded VarInt minus one with signed wrapping semantics |
| `20` | `POSE` | VarInt IDs `0..=17` in declared order; every other value maps to standing |
| `21` | `CAT_VARIANT` | ordered dynamic `minecraft:cat_variant` holder VarInt |
| `22` | `CAT_SOUND_VARIANT` | ordered dynamic `minecraft:cat_sound_variant` holder VarInt |
| `23` | `COW_VARIANT` | ordered dynamic `minecraft:cow_variant` holder VarInt |
| `24` | `COW_SOUND_VARIANT` | ordered dynamic `minecraft:cow_sound_variant` holder VarInt |
| `25` | `WOLF_VARIANT` | ordered dynamic `minecraft:wolf_variant` holder VarInt |
| `26` | `WOLF_SOUND_VARIANT` | ordered dynamic `minecraft:wolf_sound_variant` holder VarInt |
| `27` | `FROG_VARIANT` | ordered dynamic `minecraft:frog_variant` holder VarInt |
| `28` | `PIG_VARIANT` | ordered dynamic `minecraft:pig_variant` holder VarInt |
| `29` | `PIG_SOUND_VARIANT` | ordered dynamic `minecraft:pig_sound_variant` holder VarInt |
| `30` | `CHICKEN_VARIANT` | ordered dynamic `minecraft:chicken_variant` holder VarInt |
| `31` | `CHICKEN_SOUND_VARIANT` | ordered dynamic `minecraft:chicken_sound_variant` holder VarInt |
| `32` | `ZOMBIE_NAUTILUS_VARIANT` | ordered dynamic `minecraft:zombie_nautilus_variant` holder VarInt |
| `33` | `OPTIONAL_GLOBAL_POS` | boolean presence, then dimension key identifier and packed block position |
| `34` | `PAINTING_VARIANT` | ordered dynamic `minecraft:painting_variant` holder VarInt |
| `35` | `SNIFFER_STATE` | source-order enum VarInt; every out-of-range value maps to ID zero |
| `36` | `ARMADILLO_STATE` | source-order enum VarInt; every out-of-range value maps to ID zero |
| `37` | `COPPER_GOLEM_STATE` | source-order enum VarInt; every out-of-range value maps to ID zero |
| `38` | `WEATHERING_COPPER_STATE` | source-order enum VarInt; out-of-range values clamp to the first/last state |
| `39` | `VECTOR3` | X, Y and Z big-endian IEEE floats |
| `40` | `QUATERNION` | X, Y, Z and W big-endian IEEE floats |
| `41` | `RESOLVABLE_PROFILE` | boolean (`true` resolved, `false` partial), selected profile value, then player-skin patch |
| `42` | `HUMANOID_ARM` | VarInt `0=left, 1=right`; every other value maps to left |

The serializer ID is followed by the selected value with no generic length wrapper. Serializer 16
therefore delegates remaining fields to the selected particle type, and serializers 7/41 delegate
to their component/profile codecs. Unknown serializer IDs return null from the identity table and
fault ID-99 decoding; they do not default to serializer zero.

Serializer 41's resolved branch is UUID, player name `UTF(16)`, then at most 16 properties. Its
partial branch is boolean-present player name `UTF(16)`, boolean-present UUID, then the same
properties. Each property is name `UTF(64)`, value `UTF(32767)`, and nullable signature
`UTF(1024)`. The following skin patch contains, in order, optional body, cape and elytra resource
textures and an optional model; each texture is one identifier, and the model is one boolean
(`true=slim`, `false=wide`). Each optional uses its own boolean presence byte.

Metadata slots are allocated per declaring class after its superclass slots. The locked source has
221 static accessor declarations; sorting
`declaring_class#field<TAB>slot<TAB>serializer_id` and hashing newline-terminated rows yields
`b489eec18fc1981ebfb7ac97c54a4485fe2f938a`. The base `Entity` owns slots `0..=7` and
`LivingEntity` owns `8..=14`; subclasses continue from their exact superclass. The largest locked
declaration is slot 24. Every concrete type's table is the inherited union plus its declarations,
with exact defaults and callbacks from `defineSynchedData`. Slot coincidence across unrelated class
branches has no shared meaning. The audit is reproduced by bootstrapping the locked Java 25 client,
loading every top-level `net.minecraft.world.entity` class, reflecting static
`EntityDataAccessor` fields and resolving each serializer with
`EntityDataSerializers#getSerializedId`; all classes load without failure.

ID 131 attributes resolve through the locked 40-entry `minecraft:attribute` registry. ID 102 item
stacks use the distinct 1,537-entry `minecraft:item` registry and 111-entry
`minecraft:data_component_type` registry; each present patch entry then dispatches the selected
component type's trusted stream codec. Their complete static raw-ID maps are the corresponding
`reports/registries.json` `protocol_id` fields and can be emitted without copying generated tables:

```sh
for kind in minecraft:attribute minecraft:item minecraft:data_component_type; do
  jq -r --arg kind "$kind" '.[$kind].entries | to_entries
    | sort_by(.value.protocol_id)[]
    | "\(.value.protocol_id)\t\(.key)"' \
    target/mc-reference/26.2/generated/reports/registries.json
done
```

The variant, villager and painting holders above instead use the connection's ordered dynamic
registries established during configuration. Invalid static item/component/attribute IDs and
invalid dynamic holder IDs use throwing maps. Equipment ordinals are the separate eight-value
table in `play-clientbound.md`; passenger, leash and packet entity integers are current-level
entity IDs. Metadata block states use the 32,366-entry global state table. Equal integers across any
of these domains are unrelated.

The entity-effects family adds three locked registries. Recording each static raw-ID row as
`id<TAB>identifier`, sorting by ID and hashing newline-terminated rows gives the following 26.2
inventory:

| Registry | Entries | SHA-1 | Wire form |
|---|---:|---|---|
| `minecraft:mob_effect` | `40` | `cdb16be72f5c822c55158caabc8c537fa1d012cc` | strict holder raw ID |
| `minecraft:particle_type` | `125` | `5dbdae8be2ba868ae33601e37e127d3c9848109a` | strict type raw ID, then selected options codec |
| `minecraft:sound_event` | `1,968` | `174ea5fc5cfc6212cf6a858475811e3d90889734` | zero/direct value or registered raw ID plus one |

```sh
for kind in minecraft:mob_effect minecraft:particle_type minecraft:sound_event; do
  jq -r --arg kind "$kind" '.[$kind].entries | to_entries
    | sort_by(.value.protocol_id)[]
    | "\(.value.protocol_id)\t\(.key)"' \
    target/mc-reference/26.2/generated/reports/registries.json | shasum
done
```

Mob-effect and particle invalid IDs throw. Effect flags, amplifier and duration do not select those
maps. Each particle occurrence dispatches the exact selected type codec, including both ID 36's
primary particle and every weighted recipe; this is the same dispatcher used by metadata serializer
16. Sound-event zero is followed by identifier and optional fixed-range float. A registered sound
ID `n` is encoded as VarInt `n+1`; invalid nonzero values after subtracting one throw. Equal numeric
values across effect, particle, sound, metadata and entity domains remain unrelated. Projectile
power is a raw double rather than a registry identity.

The container-convergence slice additionally resolves ID-59 menu types through the static
`minecraft:menu` registry. Recording `protocol_id<TAB>identifier` in raw-ID order and hashing the
newline-terminated rows gives 25 entries and SHA-1
`dc1416c68f9fb0efac6c1a3ce39db0d5e2216387`. IDs `0..=24` are:

| IDs | Menu identities in order |
|---|---|
| `0..=7` | `generic_9x1`, `generic_9x2`, `generic_9x3`, `generic_9x4`, `generic_9x5`, `generic_9x6`, `generic_3x3`, `crafter_3x3` |
| `8..=15` | `anvil`, `beacon`, `blast_furnace`, `brewing_stand`, `crafting`, `enchantment`, `furnace`, `grindstone` |
| `16..=24` | `hopper`, `lectern`, `loom`, `merchant`, `shulker_box`, `smithing`, `smoker`, `cartography_table`, `stonecutter` |

Every identity has the `minecraft:` namespace. The packet uses the strict registry object codec,
not a holder/direct form; invalid IDs throw and no namespaced identifier appears on the wire. The
title immediately following it is trusted registry-aware component NBT and belongs to neither this
menu map nor the item-component-type map.

Container payloads reuse the 1,537-entry `minecraft:item` and 111-entry
`minecraft:data_component_type` maps above for complete stacks. Serverbound prediction hashes carry
the same strict item/component identities but replace each present component's encoded value with a
fixed signed 32-bit CRC32C. Item count, menu/container/state/property/slot numbers, click input IDs,
hash values and packet IDs are distinct numeric domains even when equal.

```sh
jq -r '.["minecraft:menu"].entries | to_entries
  | sort_by(.value.protocol_id)[]
  | "\(.value.protocol_id)\t\(.key)"' \
  target/mc-reference/26.2/generated/reports/registries.json | shasum
```

ID 3 `award_stats` uses a two-stage registry dispatch. The first VarInt selects one of the
nine `minecraft:stat_type` entries; the following VarInt selects a value from that type's backing
registry:

| Type raw ID | Identity | Backing registry |
|---:|---|---|
| `0` | `minecraft:mined` | `minecraft:block` |
| `1` | `minecraft:crafted` | `minecraft:item` |
| `2` | `minecraft:used` | `minecraft:item` |
| `3` | `minecraft:broken` | `minecraft:item` |
| `4` | `minecraft:picked_up` | `minecraft:item` |
| `5` | `minecraft:dropped` | `minecraft:item` |
| `6` | `minecraft:killed` | `minecraft:entity_type` |
| `7` | `minecraft:killed_by` | `minecraft:entity_type` |
| `8` | `minecraft:custom` | `minecraft:custom_stat` |

Recording every involved registry row as `protocol_id<TAB>identifier` in raw-ID order and hashing
newline-terminated rows gives this locked inventory:

| Registry | Entries | SHA-1 |
|---|---:|---|
| `minecraft:stat_type` | `9` | `9fd1bf3943cf604d62c3b948dad78fe23111601f` |
| `minecraft:block` | `1,196` | `849c5b7d2941ace59b94eecd3eecd77cc17abfd4` |
| `minecraft:item` | `1,537` | `5df4809be85980f5f0c3d7ca373c94244c75ba84` |
| `minecraft:entity_type` | `158` | `b61e6fad1d0e96a884b4d32dbf26a061aef96ae5` |
| `minecraft:custom_stat` | `77` | `3b680e29e4831eb47d51785dba27be9963aa4849` |

```sh
for kind in minecraft:stat_type minecraft:block minecraft:item minecraft:entity_type minecraft:custom_stat; do
  jq -r --arg kind "$kind" '.[$kind].entries | to_entries
    | sort_by(.value.protocol_id)[]
    | "\(.value.protocol_id)\t\(.key)"' \
    target/mc-reference/26.2/generated/reports/registries.json | shasum
done
```

The stat-type registry is nondefaulted, so an invalid first ID cannot select a backing codec. The
custom-stat backing is also nondefaulted and throws on an invalid second ID. Block, item and entity
type instead use `DefaultedMappedRegistry`: any negative or out-of-range backing ID becomes
`minecraft:air`, `minecraft:air`, or `minecraft:pig` respectively before the stat object is built.
The following signed statistic value is not a registry identity. For example, the golden
`minecraft:custom/minecraft:jump` key is type raw ID `8` followed by custom-stat raw ID `23`.
Equal values from the item, entity, block, custom-stat, packet-ID or statistic-value domains are
unrelated. Ferrite retains the resolved typed pair of namespaced identities rather than either raw
integer.

Primary anchors are `DamageType#STREAM_CODEC`, `DimensionType#STREAM_CODEC`,
`ByteBufCodecs#holderRegistry`, `ClientboundDamageEventPacket`, `CommonPlayerSpawnInfo`, and the
dynamic registry reconstruction in [login and configuration](login-and-configuration.md). Static
spawn anchors are `ByteBufCodecs#registry`, `BuiltInRegistries#ENTITY_TYPE`,
`DefaultedMappedRegistry#byId`, `ClientboundAddEntityPacket`, and locked
`reports/registries.json`. Entity-state anchors are `EntityDataSerializers`,
`SynchedEntityData#defineId`, every entity `defineSynchedData`, `Attribute#STREAM_CODEC`,
`EquipmentSlot`, `ItemStack#OPTIONAL_STREAM_CODEC`, `DataComponentPatch#STREAM_CODEC`, and the
configuration registry snapshot. Entity-effect anchors are `MobEffect#STREAM_CODEC`,
`ParticleTypes#STREAM_CODEC`, every `ParticleType#streamCodec`, `SoundEvent#STREAM_CODEC`, and
`reports/registries.json`. Player-projection anchors are `Stat#STREAM_CODEC`,
`StatType#streamCodec`, `BuiltInRegistries#STAT_TYPE`, the four backing registries above, and the
same locked registry report.
