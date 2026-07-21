# C1-C3 Clientbound Play

This page source-specifies the clientbound packets used by the locked server to create the first
play-state client level and synchronize its initial connection projection, the C2 liveness,
disconnect, rotation, vehicle-correction, terrain, and block-convergence families, and the first C3
entity session, motion, spawn, state, effect, container-convergence, local-player projection,
special-screen, and recipe-book projection families. Remaining inventory/progression, chat, and
later gameplay deltas stay in their independently owned C3-C4 families.

Every numeric registry value below is a wire projection derived from the configuration registries;
it is not an authoritative Ferrite identifier.

## Entry packet inventory

| ID | Identity | Fields in exact order |
|---:|---|---|
| `10` | `minecraft:change_difficulty` | difficulty VarInt; locked boolean |
| `16` | `minecraft:commands` | command-node list; root-node VarInt |
| `34` | `minecraft:entity_event` | signed big-endian entity ID int; signed event byte |
| `38` | `minecraft:game_event` | unsigned event byte; float parameter |
| `43` | `minecraft:initialize_border` | center X/Z doubles; old/new size doubles; lerp VarLong; absolute maximum, warning blocks, warning time VarInts |
| `49` | `minecraft:login` | play-login record and common spawn record |
| `64` | `minecraft:player_abilities` | flag byte; flying-speed float; walking-speed float |
| `70` | `minecraft:player_info_update` | fixed action mask; player-entry list |
| `72` | `minecraft:player_position` | teleport VarInt; position/motion/rotation record; relative-flags int |
| `74` | `minecraft:recipe_book_add` | recipe-display entry list; replace boolean |
| `76` | `minecraft:recipe_book_settings` | eight booleans |
| `86` | `minecraft:server_data` | trusted context-free component NBT; nullable byte array |
| `97` | `minecraft:set_default_spawn_position` | dimension identifier; packed block-position long; yaw/pitch floats |
| `105` | `minecraft:set_held_slot` | slot VarInt |
| `113` | `minecraft:set_time` | signed big-endian game-time long; clock-state map |
| `127` | `minecraft:ticking_state` | tick-rate float; frozen boolean |
| `128` | `minecraft:ticking_step` | remaining step count VarInt |
| `133` | `minecraft:update_recipes` | recipe-property map; stonecutter selection list |

All list and map counts are VarInts. Unless a nested codec states a smaller bound, they are bounded
by the enclosing `8_388_608`-byte uncompressed frame and fail on negative, truncated, or impossible
allocation. Booleans and identifiers use the common primitive rules. Server data uses trusted
context-free component NBT; player-info display names use trusted registry-aware component NBT.
Both retain depth 512 and have no quota below transport.

Primary inventory anchors are `net.minecraft.network.protocol.game.GameProtocols`,
`net.minecraft.network.protocol.game.GamePacketTypes`, and the locked `OFF-REPORT-001` packet
report.

## Play login grammar

The ID-49 body is:

```text
player_entity_id:i32
hardcore:boolean
level_count:VarInt
levels[level_count]:dimension_identifier
max_players:VarInt
chunk_radius:VarInt
simulation_distance:VarInt
reduced_debug_info:boolean
show_death_screen:boolean
limited_crafting:boolean
dimension_type_raw_id:VarInt
dimension:dimension_identifier
obfuscated_seed:i64
game_mode:i8
previous_game_mode:i8
is_debug:boolean
is_flat:boolean
last_death_present:boolean
if last_death_present {
    last_death_dimension:dimension_identifier
    last_death_position:packed_block_position_i64
}
portal_cooldown:VarInt
sea_level:VarInt
online_mode:boolean
enforces_secure_chat:boolean
```

The dimension-type raw ID indexes the dynamic `minecraft:dimension_type` registry reconstructed in
configuration. An unknown raw ID fails decode. The level collection becomes a set, so duplicate
identifiers collapse. Current game-mode bytes map `0..=3` to survival, creative, adventure, and
spectator; any other current value maps to survival. Previous mode `-1` means absent and every other
out-of-range value maps to survival. The seed is
`net.minecraft.world.level.biome.BiomeManager#obfuscateSeed` of the authoritative level seed, not
the raw persistence seed.

The client creates its level only after this packet: it installs the entity ID, configured render
and simulation distances, dimension type/key, mode, flags, last-death position, portal cooldown,
sea level, offline/online policy, and secure-chat policy, then opens a level-load tracker. The
informational maximum-player value is decoded but does not become admission policy. A second login
would recreate play client state and is not a legal Ferrite session transition.

Primary anchors are `net.minecraft.network.protocol.game.ClientboundLoginPacket`,
`net.minecraft.network.protocol.game.CommonPlayerSpawnInfo`,
`net.minecraft.server.level.ServerPlayer#createCommonSpawnInfo`, and
`net.minecraft.client.multiplayer.ClientPacketListener#handleLogin`.

## Command-tree grammar

ID 16 encodes a node list followed by its root index. Each node is:

```text
flags:u8
child_count:VarInt
children[child_count]:VarInt
if flags & 0x08 { redirect:VarInt }
switch flags & 0x03 {
    0 => root, no node payload
    1 => literal_name:UTF(32767)
    2 => argument_name:UTF(32767), argument_type_raw_id:VarInt,
         argument_type_specific_payload,
         if flags & 0x10 { suggestion_provider:identifier }
}
root_index:VarInt
```

Bit `0x04` marks executable, `0x08` redirect, `0x10` custom suggestions, and `0x20` a restricted
client node. Argument-type raw IDs index the locked `minecraft:command_argument_type` registry and
select that type's own stream grammar. Literal/root nodes carrying argument-only flags do not gain
argument payload. Type `3` and an unknown argument raw ID produce a null stub, which the client
resolves as a root placeholder; a placeholder reached as a child is skipped. Because an unknown
argument codec consumes no type-specific bytes, any such bytes remain trailing and fault the
packet. Child or redirect cycles fail the decoder's build/resolution validation. A reachable
out-of-range child, redirect, or root fails during dispatcher construction; inert unreachable
out-of-range data can remain unobserved. A conforming server emits neither placeholder nor invalid
reference forms.

The initial server sends its permission-filtered command tree after a permission entity event. An
empty but valid dispatcher is one root node with flags `0`, no children, and root index `0`.

Primary anchors are `net.minecraft.network.protocol.game.ClientboundCommandsPacket`,
`net.minecraft.commands.Commands#sendCommands`,
`net.minecraft.commands.synchronization.ArgumentTypeInfos`, and
`net.minecraft.client.multiplayer.ClientPacketListener#handleCommands`.

## Player-info grammar

ID 70 starts with one fixed eight-bit action mask. Bit order is:

| Bit | Action | Per-entry payload |
|---:|---|---|
| `0` | add player | name `UTF(16)`; at most 16 profile properties |
| `1` | initialize chat | nullable chat-session data |
| `2` | update game mode | mode VarInt |
| `3` | update listed | boolean |
| `4` | update latency | VarInt milliseconds |
| `5` | update display name | nullable trusted registry-aware component NBT |
| `6` | update list order | VarInt |
| `7` | update hat | boolean |

After the mask comes the entry count. Every entry begins with a UUID and then contains selected
payloads in ascending bit order. Profile properties use name `UTF(64)`, value `UTF(32767)`, and a
nullable signature `UTF(1024)`. Nullable chat data is a session UUID, expiry epoch-millisecond long,
VarInt-length-prefixed X.509 public key capped at 512 bytes, and a signature byte array capped at
4,096 bytes. Game-mode VarInts outside `0..=3` map to survival.

An add action must precede updates for a new UUID. The client ignores update-only entries whose UUID
is unknown. A new offline connection receives an all-actions packet for existing players (valid
even with zero entries), then an all-actions packet containing itself. Its chat session is absent,
and secure-chat validation remains a C4 gate.

Primary anchors are `net.minecraft.network.protocol.game.ClientboundPlayerInfoUpdatePacket`,
`net.minecraft.world.entity.player.ProfilePublicKey$Data`, and
`net.minecraft.client.multiplayer.ClientPacketListener#handlePlayerInfoUpdate`.

## Recipe projection grammars

ID 76 contains open/filtering booleans for crafting, furnace, blast furnace, and smoker, in that
order. ID 74 contains a list of entries followed by `replace`. Each entry contains:

1. display index VarInt;
2. recipe-display type raw ID and that registered type's stream payload;
3. optional group encoded as zero for absent or `group + 1` otherwise;
4. recipe-book-category raw ID;
5. a boolean and, when present, a list of item-holder-set crafting requirements;
6. a flags byte where bit `0` requests a toast and bit `1` highlights the entry.

ID 133 first carries a map from recipe-property-set identifier to a list of item raw IDs. It then
carries a stonecutter entry list; each entry is an item holder set followed by a slot-display type
raw ID and its registered payload. Recipe-display, slot-display, category, item, component, trim,
and potion raw IDs resolve through their locked registries. Holder sets use the common
tag-or-direct-set grammar: encoded count zero selects a following tag identifier; a positive value
`n + 1` selects `n` following holder raw IDs.

The locked recipe-display type IDs are `crafting_shapeless=0`, `crafting_shaped=1`, `furnace=2`,
`stonecutter=3`, and `smithing=4`. Slot-display type IDs are `empty=0`, `any_fuel=1`,
`with_any_potion=2`, `only_with_component=3`, `item=4`, `item_stack=5`, `tag=6`, `dyed=7`,
`smithing_trim=8`, `with_remainder=9`, and `composite=10`; each dispatches to its named locked
stream codec. Unknown raw IDs fail. Initial recipe settings replace the client settings, an ID-74
packet with `replace=true` clears the prior book before adding its entries, and ID 133 replaces the
client's property/stonecutter projection without mutating authoritative server recipes.

Primary anchors are `net.minecraft.network.protocol.game.ClientboundUpdateRecipesPacket`,
`net.minecraft.network.protocol.game.ClientboundRecipeBookAddPacket`,
`net.minecraft.network.protocol.game.ClientboundRecipeBookSettingsPacket`,
`net.minecraft.world.item.crafting.RecipePropertySet`,
`net.minecraft.world.item.crafting.SelectableRecipe`,
`net.minecraft.world.item.crafting.display.RecipeDisplayEntry`, and
`net.minecraft.world.item.crafting.display.SlotDisplay`.

## Position and acknowledgement trigger

ID 72 contains teleport challenge VarInt; position X/Y/Z doubles; delta X/Y/Z doubles; yaw/pitch
floats; and a big-endian 32-bit relative mask. Bits `0..=8` mean relative X, Y, Z, yaw, pitch, delta
X, delta Y, delta Z, and rotate-delta. Higher bits are ignored. Absolute components replace the
client value; relative components add to it. Pitch is clamped to `[-90, 90]`; rotate-delta first
rotates the existing velocity by the angle difference. This local-player correction is applied
immediately and is never interpolated.

The local player applies the correction unless riding, sends the identical challenge in
serverbound ID 0, then immediately sends serverbound ID 31 `move_player_pos_rot` containing three
doubles, yaw/pitch floats, and one flags byte (`bit 0 = on ground`, `bit 1 = horizontal collision`).
It finally resets block-state prediction on teleport. The acknowledgement and first movement echo
are distinct packets and may not be collapsed.

Primary anchors are `net.minecraft.network.protocol.game.ClientboundPlayerPositionPacket`,
`net.minecraft.world.entity.PositionMoveRotation`, `net.minecraft.world.entity.Relative`, and
`net.minecraft.client.multiplayer.ClientPacketListener#handleMovePlayer`.

## Remaining entry state

- Difficulty IDs are VarInts mapped modulo the four values `peaceful=0`, `easy=1`, `normal=2`, and
  `hard=3`; the following boolean controls the client difficulty lock.
- Ability flags are `invulnerable=0x01`, `flying=0x02`, `can_fly=0x04`, and
  `instant_build=0x08`; other bits are ignored. Speeds are raw IEEE-754 floats.
- Entity event dispatches its signed byte to the entity selected by the signed int. Initial IDs
  `24..=28` set the local permission tier to no permissions, moderator, gamemaster, admin, and
  owner respectively; an unknown entity is ignored and all other event-byte meanings remain entity
  behavior.
- Game event IDs are `no_respawn=0`, `start_rain=1`, `stop_rain=2`, `game_mode=3`, `win=4`,
  `demo=5`, `arrow_hit=6`, `rain_level=7`, `thunder_level=8`, `puffer_sting=9`,
  `elder_guardian=10`, `immediate_respawn=11`, `limited_crafting=12`, and
  `level_chunks_load_start=13`. An unknown ID becomes no known type and has no handler effect. ID
  13 tells the level-load tracker that terrain packets may now satisfy readiness.
- Border initialization applies center, either timed size lerp or immediate new size, absolute
  maximum, warning blocks, and warning time. A nonpositive lerp duration selects immediate size.
- Server data is presentation-only MOTD plus nullable frame-bounded icon bytes. It is ignored when
  no multiplayer server-list record exists; invalid icon bytes are not installed.
- Default spawn is a global dimension/packed block position plus yaw/pitch. Held slots outside the
  nine-slot hotbar are decoded but ignored.
- Set-time uses a signed long game time, then clock-map entries of world-clock raw ID, total-ticks
  VarLong, partial-tick float, and rate float. It replaces the corresponding client clock states.
- Ticking state installs tick rate/frozen state; ticking step installs the remaining frozen step
  count. Both are ignored before a client level exists.

Primary anchors are the packet classes named in the inventory,
`net.minecraft.client.multiplayer.ClientPacketListener`,
`net.minecraft.server.ServerTickRateManager#updateJoiningPlayer`, and
`net.minecraft.server.players.PlayerList#sendLevelInfo`.

## Locked initial ordering

After configuration finish installs play in both directions, the locked server suspends flushing
and queues this order for a new player:

1. login, difficulty, abilities, held slot, and complete recipe projection;
2. permission entity event and permission-filtered commands;
3. recipe-book settings and a replace-style recipe-book add (valid with no entries);
4. any nonempty scoreboard projection and join messages to already connected players;
5. player position challenge;
6. server data unless this is a transferred connection;
7. all-actions player-info for existing players, then after insertion an all-actions self entry;
8. border, full clock state, default spawn, optional rain events, and
   `level_chunks_load_start`;
9. ticking state and ticking step;
10. asynchronous C2 cache-center, chunk-batch, chunk/light, and terrain-ready flow.

A fresh player with no saved recipes still receives settings and an empty replace packet. Marking
statistics dirty does not itself send an award-stats packet. An empty scoreboard and no prior
players remove those conditional packets without changing the relative order above. Flushing
resumes only after player insertion, active effects, inventory listener setup, and join
notification, preserving the queued order.

Primary ordering anchors are `net.minecraft.server.players.PlayerList#placeNewPlayer`,
`net.minecraft.server.players.PlayerList#sendPlayerPermissionLevel`,
`net.minecraft.stats.ServerRecipeBook#sendInitialRecipeBook`, and
`net.minecraft.server.players.PlayerList#sendLevelInfo`.

## Normalized boundary and failure behavior

The adapter maps play login to a normalized session/world-view creation event, recipe/command/player
information to versioned client projections, and position challenge to connection-local correction
state. Raw packet IDs, entity IDs, teleport IDs, registry raw IDs, command-node indices, recipe
display indices, metadata codecs, and trusted client-presentation NBT must not enter Ferrite ECS or
persistence.

Malformed primitives, unknown required registry raw IDs, impossible command graphs, invalid NBT,
truncation, or trailing bytes fail packet decode/handling and close through play protocol fault
handling. Semantically unknown game/entity events and invalid held-slot values follow the explicit
ignore behavior above; Ferrite must not turn those cases into gameplay mutations.

## C2 session packet inventory

| ID | Identity | Fields in exact order |
|---:|---|---|
| `32` | `minecraft:disconnect` | trusted context-free component NBT reason |
| `44` | `minecraft:keep_alive` | signed big-endian challenge long |
| `57` | `minecraft:move_vehicle` | X/Y/Z doubles; yaw/pitch floats |
| `61` | `minecraft:ping` | signed big-endian payload int |
| `73` | `minecraft:player_rotation` | yaw float; relative-yaw boolean; pitch float; relative-pitch boolean |

These are legal only after play codecs are installed. Disconnect reason uses the common trusted
context-free component codec with NBT depth 512 and no quota below the enclosing uncompressed
frame. The client closes the connection with that reason; there is no acknowledgement packet and
the reason remains presentation/session state rather than a gameplay event.

Keepalive and ping reuse the common codecs but have distinct acknowledgement domains. The client
echoes ID 44's signed long in serverbound ID 28, immediately unless rendering is frozen at event
polling, in which case it defers for at most one minute. It answers ID 61 immediately with the same
signed int in serverbound ID 45. A pong never acknowledges keepalive. The server's exact 15-second
challenge/timeout and latency rules are specified in [serverbound play](play-serverbound.md).

Primary anchors are `net.minecraft.network.protocol.common.ClientboundDisconnectPacket`,
`ClientboundKeepAlivePacket`, `ClientboundPingPacket`, and
`net.minecraft.client.multiplayer.ClientCommonPacketListenerImpl`.

## Player rotation correction

ID 73 carries two independent relativity booleans, not the ID-72 bit mask. The client computes:

```text
new_yaw   = relative_yaw   ? current_yaw   + packet_yaw   : packet_yaw
new_pitch = clamp(relative_pitch ? current_pitch + packet_pitch : packet_pitch, -90, 90)
```

It installs both values immediately, synchronizes the old-render rotation, then sends serverbound
ID 32 `move_player_rot` containing the resulting yaw/pitch and both movement flags false. There is
no challenge ID, interpolation, teleport acknowledgement, or stale-correction state. This packet
is emitted by the locked server's `ServerPlayer#forceSetRotation`, including the `/rotate` path.

The codec accepts every float bit pattern. Infinite pitch clamps to an endpoint, while NaN pitch
and non-finite yaw can remain non-finite; the mandatory ID-32 response is then rejected by the
server's movement finite-value check. Ferrite must preserve this fault boundary rather than
pre-validating the clientbound codec as if it were ID 72.

Primary anchors are `net.minecraft.network.protocol.game.ClientboundPlayerRotationPacket`,
`net.minecraft.world.entity.PositionMoveRotation#calculateAbsolute`,
`net.minecraft.client.multiplayer.ClientPacketListener#handleRotatePlayer`, and
`net.minecraft.server.level.ServerPlayer#forceSetRotation`.

## Vehicle correction

The server sends ID 57 only as a correction from its vehicle-movement validator: either the client
moved more quickly than allowed, or collision/residual validation restored the authoritative
vehicle pose. The packet contains the server vehicle's current absolute pose and no entity ID,
flags, velocity, challenge, or interpolation duration. It therefore applies only to the local
player's current root vehicle.

The client ignores the packet unless that root vehicle differs from the player and is locally
authoritative. For a qualifying vehicle it compares packet position with the current interpolation
target when interpolating, otherwise current position. Only Euclidean distance greater than
`1e-5` causes a correction: active interpolation is cancelled and the vehicle snaps to
the supplied position and rotations. Regardless of whether a snap was necessary, it then sends
serverbound ID 34 built from its resulting position, rotations, and on-ground state. Consequently,
a same-position packet whose only change is rotation does not apply that rotation, but still
elicits an echo.

No client-side finite-value guard exists. A NaN position makes the distance comparison false and
elicits an echo of current state without snapping; an infinite position compares greater and can
be installed. Rotations are installed only along the position-change branch. The echoed
serverbound packet then enters the explicit vehicle invalid-value/clamp/collision validator in
[serverbound play](play-serverbound.md).

Primary anchors are `net.minecraft.network.protocol.game.ClientboundMoveVehiclePacket`,
`net.minecraft.client.multiplayer.ClientPacketListener#handleMoveVehicle`, and
`net.minecraft.server.network.ServerGamePacketListenerImpl#handleMoveVehicle`.

## C2 normalized boundary and failure behavior

Ferrite projects disconnect and liveness as connection-local state, rotation as a normalized local
player correction, and vehicle correction as an absolute session projection derived from the
authoritative vehicle. Packet IDs, echo payloads, local entity identity, and codec component types
remain inside the 26.2 adapter.

Malformed/truncated primitives, malformed or over-deep reason NBT, and trailing bytes fail the
packet. Semantically irrelevant vehicle corrections follow the explicit ignore path; they are not
reinterpreted for another entity. IEEE-754 exceptional values follow the handler-specific behavior
above and must be covered independently from transport-malformation tests.

## C2 terrain packet inventory

| ID | Identity | Fields in exact order |
|---:|---|---|
| `0` | `minecraft:bundle_delimiter` | no fields; pipeline delimiter |
| `11` | `minecraft:chunk_batch_finished` | batch-size VarInt |
| `12` | `minecraft:chunk_batch_start` | no fields |
| `13` | `minecraft:chunks_biomes` | chunk-biome record list |
| `37` | `minecraft:forget_level_chunk` | packed chunk-position long |
| `45` | `minecraft:level_chunk_with_light` | chunk X/Z ints; full chunk data; light data |
| `48` | `minecraft:light_update` | chunk X/Z VarInts; light data |
| `94` | `minecraft:set_chunk_cache_center` | chunk X/Z VarInts |
| `95` | `minecraft:set_chunk_cache_radius` | radius VarInt |
| `111` | `minecraft:set_simulation_distance` | distance VarInt |

Chunk X/Z in ID 45 are fixed signed big-endian ints. ID 48 and all cache controls use signed
VarInts. ID 37 uses `ChunkPos.pack`: X occupies the low 32 bits and Z the high 32 bits of one
big-endian long. The packet report fixes these numeric IDs; the codec classes named by each
identity fix the bodies.

## Bundle pipeline

ID 0 is not delivered to the play listener. The first delimiter opens a pipeline bundle, the next
delimiter closes it, and the client receives one synthetic `ClientboundBundlePacket` whose
subpackets are handled sequentially on the client packet thread. An empty pair is valid. At most
4,096 subpackets may appear between delimiters; the next subpacket faults. A terminal packet inside
a bundle also faults. A delimiter encountered while a bundle is open closes that bundle rather
than nesting another one; until closure, enclosed packets are withheld from the listener.

The outbound adapter represents a bundle as delimiter, each independently framed/compressed
subpacket, delimiter. There is no aggregate bundle body or length. Ferrite may schedule a normalized
atomic projection internally, but must preserve the delimiter count, subpacket order, individual
frames, and 4,096-packet bound at this versioned boundary.

Primary anchors are `net.minecraft.network.protocol.BundlerInfo`, `PacketBundlePacker`,
`PacketBundleUnpacker`, `ClientboundBundleDelimiterPacket`, and
`ClientPacketListener#handleBundlePacket`.

## Cache interest and batch flow

Play login supplies the initial render and simulation distances. ID 94 moves the chunk-cache view
center to its signed coordinates. ID 95 stores the raw server radius, updates the options display,
and reallocates client cache storage only when internal chunk radius `max(2, radius) + 3` changes;
the square array side is twice that radius plus one. Chunks still in range are retained and omitted
old slots are discarded without an explicit per-chunk unload callback. ID 111 stores the raw
simulation distance in client level state. The wire codecs accept every signed VarInt, including
negative values. A normal locked server constrains effective view distance to `2..=32`; client
behavior for adversarial raw values uses unchecked Java int arithmetic in the radius, side, square
allocation, center-delta and absolute-value calculations. Overflow can therefore create degenerate
storage or a handler/allocation fault; it is not saturation or a hidden wire clamp.

When the server tracking view first becomes positioned or changes center, it sends ID 94 before
computing the tracking-view difference. Newly tracked ready chunks enter a connection-local pending
set; chunks leaving the view are removed from that set without an unload packet, or, if already
sent and the player is alive, produce ID 37. Thus a client never needs to unload a chunk it was
never sent.

Each server tick, quota and unacknowledged-batch limits permit this exact sequence:

1. ID 12 starts a batch and records the client's measurement timestamp.
2. One or more ID-45 full chunks are selected nearest the player's chunk from the pending set.
3. ID 11 ends the batch with the number of ID-45 packets actually sent.
4. The client updates its time-per-chunk estimator and returns serverbound ID 11
   `chunk_batch_received` with desired chunks/tick.

ID 11's signed VarInt is informational to the client estimator: only positive values update the
sample, while zero/negative values still produce feedback. An unmatched finish is tolerated and
uses the current timestamp; another start simply replaces that timestamp. The server begins with
one allowed in-flight batch and nine desired chunks/tick; the exact quota, acknowledgement, NaN,
clamp, and ten-batch window rules are in [serverbound play](play-serverbound.md). The client does
not count intervening ID-45 packets or require them to be between markers, so a mismatched declared
size or standalone full chunk is still handled; these are legal codec/handler inputs but not normal
locked-server output.

Primary server anchors are `net.minecraft.server.level.ChunkMap#applyChunkTrackingView` and
`net.minecraft.server.network.PlayerChunkSender`; client anchors are
`ClientPacketListener#handleChunkBatchStart`, `#handleChunkBatchFinished`, and
`ChunkBatchSizeCalculator`.

## Full chunk grammar

ID 45 is:

```text
chunk_x:i32
chunk_z:i32
heightmap_count:VarInt
heightmaps[heightmap_count] {
    heightmap_type:VarInt
    long_count:VarInt
    data[long_count]:i64
}
section_blob_length:VarInt          # 0..=2_097_152
section_blob[section_blob_length]:u8
block_entity_count:VarInt
block_entities[block_entity_count] {
    packed_local_xz:i8
    y:i16
    block_entity_type_raw_id:VarInt
    update_tag:nullable_compound_nbt
}
light_data
```

The section blob contains exactly the bottom-to-top section records described in
[wire registry and palette mappings](registry-and-metadata-mappings.md), with count implied by the
configured dimension. A negative blob length or one above 2,097,152 faults allocation; truncation
faults section parsing. The client does not assert that every isolated blob byte was consumed, so
well-formed required sections followed by extra blob bytes are ignored. Heightmap map and
block-entity counts have no smaller explicit cap than the enclosing 8,388,608-byte uncompressed
packet. A negative heightmap count is observably accepted as an empty map because its EnumMap
constructor ignores capacity and the decode loop runs zero times; a negative block-entity count
faults its list allocation.

The client clears/replaces the addressed chunk's sections, heightmaps, and block entities, or
creates it, only when the coordinates fall inside its current cache range. Existing block entities
are cleared first. Heightmap length mismatch recomputes that map; block-entity NBT applies only to
a block-derived entity of the same mapped type. It then marks the chunk loaded.

Light data from the same packet is queued independently. After applying it, the client enables
light for the chunk; if the chunk exists, it also updates every section's empty status and marks the
chunk plus its one-chunk neighborhood dirty. Therefore an out-of-range full chunk can be ignored by
the chunk cache while its queued light payload still follows the light handler path.

The server constructs ID 45 from a ready `LevelChunk`: it includes only client-use heightmaps,
every dimension section, every current chunk block entity's update tag, and complete sky/block light
data. It is sent only inside the start/finish batch sequence. Packet data is a client projection;
Ferrite must build it from an authoritative immutable chunk snapshot and may not persist palette
indices, heightmap type bytes, or packed block-entity coordinates.

Primary anchors are `ClientboundLevelChunkWithLightPacket`, `ClientboundLevelChunkPacketData`,
`LevelChunkSection`, `PalettedContainer`, `LevelChunk#replaceWithPacketData`, and
`ClientChunkCache#replaceWithPacketData`.

## Light-data grammar and application

Both ID 45 and ID 48 use:

```text
sky_data_mask:bitset
block_data_mask:bitset
empty_sky_mask:bitset
empty_block_mask:bitset
sky_update_count:VarInt
sky_updates[sky_update_count]:byte_array(max=2048)
block_update_count:VarInt
block_updates[block_update_count]:byte_array(max=2048)
```

A bitset is a VarInt long count followed by that many big-endian longs; its length cannot exceed
the longs remaining in the packet. Update list counts are packet-bounded and every byte array has
its own VarInt length capped inclusively at 2,048. Section bit zero corresponds to the light
engine's minimum light section, which includes the engine's extra boundary sections rather than
the dimension's first ordinary chunk section.

For each in-range light section in ascending order, a data-mask bit consumes the next update and
constructs a nibble `DataLayer`; that consumed array must be exactly 2,048 bytes or handling faults.
An empty-mask-only bit installs an all-zero layer. When both masks contain a bit, data wins. A data
bit without another array faults by exhausted iteration. Bits above the configured light-section
count and surplus arrays are ignored by the handler.

ID 48 queues this work and marks each touched section plus neighbors dirty, then enables light for
the chunk. Full-chunk ID 45 queues the same data without per-section rebuild scheduling and follows
with full chunk-light enabling. The server's incremental ID 48 includes changed sky/block sections
for tracked border players; unchanged layers are absent, empty layers use the empty mask, and
nonempty layers use exactly one 2,048-byte update in mask order.

Primary anchors are `ClientboundLightUpdatePacketData`, `DataLayer`,
`ClientPacketListener#applyLightData`, `#readSectionList`, and
`net.minecraft.server.level.ChunkHolder#broadcastChanges`.

## Biome refresh and unload

ID 13 starts with a VarInt record count. Each record is packed chunk-position long followed by a
VarInt-length byte array capped at 2,097,152 bytes. The array contains one biome paletted container
per configured dimension section, bottom-to-top, using the same dynamic biome mapping as full
chunks. Too-short or malformed required containers fault; extra isolated bytes are ignored.

For each record the client replaces biomes only when that exact chunk is present and in cache;
otherwise it warns and ignores replacement. It nevertheless issues the chunk-loaded notification
for every listed coordinate and dirties every render section in the surrounding 3-by-3 chunk area.
The server groups requested biome resends by tracking player and emits one list packet per player.

ID 37 drops the exact in-range cached chunk when present, not an arbitrary slot collision. It also
drops debug state and queues removal of both sky and block light data for every light section,
disables chunk lighting, marks all ordinary sections empty, and unloads the chunk. Out-of-range or
absent chunk cache data is a no-op, while light removal still runs for the named coordinate.

Primary anchors are `ClientboundChunksBiomesPacket`, `ClientboundForgetLevelChunkPacket`,
`ClientChunkCache#replaceBiomes`, `ClientChunkCache#drop`, and
`ClientPacketListener#queueLightRemoval`.

## Terrain-ready relationship and failures

The C1 `level_chunks_load_start` event moves the level-load tracker from waiting-for-server to
waiting-for-player-section. Full chunk installation makes sections available for rendering; the
renderer's compiled-player-section callback opens readiness. Spectator/dead state, player or
camera outside build height, or expiry of the 30-second deadline established when client level load
started also opens it. Waiting-for-server does not test that deadline; load-start carries the same
deadline into waiting-for-player-section. The tracker then honors its configured close delay (zero
for the normal remote connection, 500 ms for the new integrated world path), sends fieldless
serverbound ID 44 `player_loaded`, and clears itself. Batch finish by itself is not terrain
readiness.

Malformed counts other than the explicit negative-heightmap exception, palettes, raw registry IDs,
NBT, fixed storage, light layers, truncation, and trailing outer packet bytes fault as specified.
Isolated section/biome blob trailing bytes and handler-ignored surplus light data are explicit
exceptions. Cache misses and unloads follow their documented ignore paths. All chunk positions,
batch counts, palette IDs, masks, and cache settings remain connection projections; Ferrite maps
their content to authoritative chunk snapshots and namespaced registry values at the adapter
boundary.

Primary readiness anchors are `net.minecraft.client.multiplayer.LevelLoadTracker`,
`ClientPacketListener#notifyPlayerLoaded`, and `ClientPacketListener#handleGameEvent`.

## C2 block-convergence packet inventory

| ID | Identity | Fields in exact order |
|---:|---|---|
| `4` | `minecraft:block_changed_ack` | sequence VarInt |
| `5` | `minecraft:block_destruction` | breaker/entity VarInt; packed block-position long; progress unsigned byte |
| `6` | `minecraft:block_entity_data` | packed block-position long; block-entity-type raw ID VarInt; trusted compound NBT |
| `7` | `minecraft:block_event` | packed block-position long; action unsigned byte; parameter unsigned byte; block raw ID VarInt |
| `8` | `minecraft:block_update` | packed block-position long; global block-state raw ID VarInt |
| `84` | `minecraft:section_blocks_update` | packed section-position long; change count VarInt; packed changes as VarLongs |

All six identities are legal only after clientbound play and a client level are installed. A normal
ID 4 correlates a serverbound predictive request, IDs 8/84/6 project an authoritative loaded-world
change, ID 5 projects an active break session to other players, and ID 7 projects a server block
event that already succeeded. The locked client codecs do not encode those semantic preconditions;
their handler-specific cache, mismatch, stale, and fault behavior below is still observable.

Packed block positions use signed 26-bit X, signed 26-bit Z, and signed 12-bit Y as specified in
[serverbound play](play-serverbound.md). A packed section position uses signed 22-bit section X in
bits `42..=63`, signed 22-bit section Z in bits `20..=41`, and signed 20-bit section Y in bits
`0..=19`. Every ID-84 change is:

```text
packed_change = (global_block_state_raw_id << 12) | relative_position
relative_position = (local_x << 8) | (local_z << 4) | local_y
```

The decoder reads the state portion with unsigned shift and then converts it to a Java int. Change
count zero is valid; negative count faults array allocation. The enclosing uncompressed frame is
the only bound on the bytes of a complete entry list, but the declared positive count itself is not
validated before allocating both arrays. A small adversarial frame can therefore fault allocation
with a huge count before entry truncation is detected. Changes are applied in wire order, so a later
duplicate position wins. The canonical server emits nonnegative state IDs `0..=32_365` and
relative values `0..=4095`.

ID 6 uses the static 49-entry block-entity-type registry and a **non-null** compound tag. Unlike a
full-chunk entry's nullable, default-quota tag, this standalone tag uses the trusted unlimited-heap
accounter: depth 512 and the enclosing 8,388,608-byte packet limit remain, but there is no separate
2,097,152-byte NBT quota. ID 7 uses the distinct 1,196-entry static block registry. IDs 8 and 84 use
the 32,366-entry global block-state table. Exact mappings and their normalized boundary are in
[wire registry and palette mappings](registry-and-metadata-mappings.md).

Primary codec anchors are the six identically named packet classes,
`SectionPos#STREAM_CODEC`, `SectionPos#sectionRelativePos`, and the registry codecs selected by
each class.

## Prediction acknowledgement and authoritative state

ID 4 accepts every signed VarInt at the client handler. ACK `N` visits all retained predicted
positions and removes those whose latest prediction sequence is `<=N`; it then synchronizes each
removed position to its most recently staged server state. A duplicate or smaller ACK simply finds
no already-removed entry, while an adversarial future ACK can release predictions the server has
not processed. Vanilla server output is nonnegative, but it is cumulative only over requests
received since the preceding connection tick, not globally monotonic.

IDs 8 and 84 both call `setServerVerifiedBlockState(pos,state,19)`. With a retained prediction at
that position they update only its saved authoritative state and leave the predicted state visible.
Without one they immediately write the client level. At ACK, a differing state is written with
flags `19`; when it collides with the local player, the client may snap to the position captured by
the first prediction. A later prediction at the same position preserves that original state and
captured position but moves the entry to the later sequence. Teleport handling suppresses captured
position rollback for acknowledgements not newer than the teleport's prediction sequence.

The exact retain/update/ACK algorithm, fastutil iteration order, server coalescing, dropped-request
behavior, and packet-order cases are normative in
[ordering and acknowledgements](ordering-and-acknowledgements.md). The visible-render question
between an ACK and a later authoritative update remains the isolated gameplay experiment in
[`PLY-BREAK-001`](../mechanics/player/ply-break-001.md); it does not make their wire send order
unknown.

Primary anchors are `ClientPacketListener#handleBlockChangedAck`, `#handleBlockUpdate`,
`#handleChunkBlocksUpdate`, `ClientLevel#setServerVerifiedBlockState`, and
`BlockStatePredictionHandler#endPredictionsUpTo`.

## Single and section block deltas

ID 8 resolves its raw ID strictly during decode; an absent global state faults. ID 84 explicitly
uses nullable lookup while decoding each packed change, so an absent or integer-wrapped state ID
survives construction as null. Without a retained prediction it faults on the immediate state
write. With one it can stage null without an immediate exception; ACK release later faults unless a
subsequent valid authoritative update replaced that saved value first. No branch substitutes air.
Truncation and malformed VarLongs fail decode.

When there is no retained prediction, a position outside client world bounds or in an absent/out-of-
range chunk reaches the client's immutable empty chunk and has no state effect. A retained entry can
still stage an update until ACK. Section changes are not required to name the client's configured
dimension sections at codec level; each expanded block position independently follows that same
bounds/cache behavior.

The server's per-tick chunk broadcaster sends changed light first. It then scans dimension sections
bottom-to-top. One changed block produces ID 8 followed, when applicable, by that block entity's
ID 6 update. Two or more changes produce one ID 84 followed by applicable ID-6 updates in the
change set's iteration order. The packet captures each current block state when broadcasting; it
does not carry old states, flags, causes, neighbor work, or block-entity NBT inline.

Ferrite projects authoritative block deltas from an immutable simulation snapshot to namespaced
states before mapping to raw IDs. Raw state numbers, relative packed positions, change-set order,
and update flags remain client projection details and never become persistence or ECS identity.

Primary anchors are `ChunkHolder#broadcastChanges`, `ClientLevel#setServerVerifiedBlockState`, and
`ClientChunkCache#getChunk`.

## Block-entity data

ID 6 looks up a current client block entity at the position whose runtime type exactly matches the
decoded wire type. Only that match loads the tag with components; absent chunks/entities and type
mismatches are ignored. The packet does not create an entity, change the block state, or override a
block-derived type. Semantic tag fields are interpreted by the matched type's loader; unknown or
reported fields follow that loader's problem-reporting/default behavior.

The normal chunk broadcaster sends this packet only when the current state has a block entity, the
entity exists, and its `getUpdatePacket()` is non-null. It follows the corresponding ID-8 or ID-84
state delta on the same connection. Special direct paths may send the same grammar independently,
including command-block editing with a custom-only tag.

Ferrite maps the type and tag to a versioned client projection generated from authoritative block-
entity state. The trusted wire NBT and raw type ID remain adapter values; they are not accepted back
as arbitrary authoritative persistence.

Primary anchors are `ClientboundBlockEntityDataPacket#create`,
`ClientPacketListener#handleBlockEntityData`, `ChunkHolder#broadcastBlockEntityIfNeeded`, and each
block entity's `getUpdatePacket`/`getUpdateTag`.

## Block events

Server block events are queued records `(position, block, action, parameter)`. When the position is
not currently block-ticking they are deferred. Otherwise the server requires the current state to
belong to the queued block and requires its `triggerEvent` to return true; only then does it
broadcast ID 7 within 64 blocks in the same dimension. The two parameters are transmitted modulo
one unsigned byte by the canonical encoder.

The client decodes and validates the packet's block raw ID, but its `Level#blockEvent` implementation
then invokes `triggerEvent` on the **current local state** and does not compare or pass the decoded
block object. A cache miss therefore reaches the empty state and normally has no effect; a changed
local block can interpret the same action/parameter under its own event logic. Unknown parameter
values are not a packet fault and remain block-specific behavior.

ID 7 is a presentation/effect projection of a server event that already succeeded. Ferrite must
not rerun it as an authoritative simulation command when encoding, and the client-supplied values
never flow in the reverse direction.

Primary anchors are `ServerLevel#blockEvent`, `#runBlockEvents`, `#doBlockEvent`,
`ClientPacketListener#handleBlockEvent`, and `Level#blockEvent`.

## Destruction progress

ID 5 keys a client crack record by the signed breaker/entity ID. Progress `0..=9` replaces that
ID's prior record, moving it to the new position when necessary and timestamping it with current
client game time. Progress `10..=255` removes the record; the unsigned codec can never produce a
negative handler value. Reusing an ID therefore removes or relocates its prior crack independently
of other breakers at the same position. At every client game-time multiple of 20, an entry is also
removed when `current_game_time - updated_game_time > 400`; equality at 400 survives that scan.

The locked server sends this packet only to **other** players in the same level whose squared
distance from the integer target coordinates is strictly below `1024.0`. The breaker uses local
prediction instead. The encoder writes the low eight bits of the server int: the client retains a
crack exactly when `(server_stage & 255)` is `0..=9` and removes it otherwise. Thus canonical `-1`
becomes byte `255`, stages `10..=255` remove, but stages `256..=265` wrap and visibly reintroduce
progress `0..=9`. This is byte truncation, not clamping. No chunk-presence test gates the client
crack map or the server broadcast.

Crack entity IDs and stages are ephemeral client projection state. They do not identify a Ferrite
block owner, durable entity, or persistence record.

Primary anchors are `ServerLevel#destroyBlockProgress`, `ClientLevel#destroyBlockProgress`, and
[`BLK-BREAK-001`](../mechanics/blocks/blk-break-001.md).

## Block-family ordering and failures

Immediate use-on corrections and break-denial corrections are queued before their next-tick ACK.
Ordinary successful world changes can instead be acknowledged before their later chunk-broadcast
delta. Block-entity data follows the state delta, while destruction progress and successful block
events are independent presentation streams with no sequence. The complete order matrix is in
[ordering and acknowledgements](ordering-and-acknowledgements.md).

Malformed/truncated fields, trailing bytes, invalid required registry IDs, non-compound or over-
depth ID-6 NBT, negative ID-84 count, invalid VarLongs, and an ID-84 null state reaching a write
fault the play packet.
Unknown event parameters, destruction stages above nine, valid cache misses, type-mismatched block
entities, and duplicate/stale ACKs follow their explicit semantic paths instead. No case may remap a
numeric ID through another registry or substitute a guessed state from another version.

# C3 Entity Feedback, Camera, Pickup, and Respawn

This first C3 clientbound entity slice specifies six outcome/session packets. Later sections specify
motion/projectile acceleration and spawn/removal independently. Metadata, attributes, equipment,
passengers, effects, and explosions remain in separate recoverable families.

| ID | Identity | Fields in exact order |
|---:|---|---|
| `2` | `minecraft:animate` | entity ID VarInt; action unsigned byte |
| `25` | `minecraft:damage_event` | damaged entity ID VarInt; configured damage-type holder raw ID VarInt; cause ID plus one VarInt; direct ID plus one VarInt; position-present boolean; when present X/Y/Z doubles |
| `42` | `minecraft:hurt_animation` | entity ID VarInt; yaw float |
| `82` | `minecraft:respawn` | common player spawn record; signed keep-data byte |
| `93` | `minecraft:set_camera` | entity ID VarInt |
| `124` | `minecraft:take_item_entity` | source entity ID VarInt; collector entity ID VarInt; amount VarInt |

Every entity number is local to the current client level. Missing entities do not resolve through a
UUID or another dimension. The damage-type and respawn dimension-type raw IDs instead resolve
through the frozen registry snapshot established in configuration; those mappings are normative in
[wire registry and palette mappings](registry-and-metadata-mappings.md).

Primary codec anchors are the six identically named packet classes and `CommonPlayerSpawnInfo`.

## Animation and hurt direction

ID 2 recognizes action byte `0` as main-hand swing, `3` as off-hand swing, `2` as player wake,
`4` as critical-hit particles, and `5` as enchanted-hit particles. Any other byte is ignored after
entity lookup. A missing entity is ignored. Actions 0/3 cast the target to `LivingEntity`, and
action 2 casts it to `Player`; a present wrong runtime type therefore faults the client handler
rather than becoming an unknown action. Critical actions accept any entity type.

The server's ordinary `LivingEntity#swing` sends action 0/3 to tracking players, adding self only
when its caller requests self-inclusive publication. Player wake sends action 2 to trackers and
self. ServerPlayer critical and magic-critical callbacks send actions 4/5 to trackers and self.
Swing-time admission happens before packet construction as specified for C2, so this packet is an
accepted animation result, not an instruction to bypass the authoritative swing gate.

ID 42 ignores a missing entity and otherwise calls `animateHurt(yaw)` on any entity; the float is
accepted without finite/range validation. The ordinary server damage path sends it directly only
to the damaged `ServerPlayer` from `indicateDamage`, after horizontal knockback direction is
derived and only when blocking did not suppress indication. It is independent of ID 25 and has no
acknowledgement. Other viewers receive the damage event and entity motion/projection streams, not
this direct player packet.

Primary anchors are `ClientPacketListener#handleAnimate`, `#handleHurtAnimation`,
`LivingEntity#swing`, `ServerPlayer#crit`, `#magicCrit`, and
`ServerPlayer#indicateDamage`.

## Damage event projection

ID 25's cause and direct entity fields encode `server_entity_id + 1`; server absence `-1` therefore
encodes zero. Both addition and decode subtraction use wrapping signed-int arithmetic, and the
codec imposes no semantic range check. The optional source-position flag is followed by three raw
doubles and likewise accepts all IEEE-754 bit patterns at codec level.

The client first resolves the damaged entity. If absent, it ignores the entire event. When source
position is present, it constructs a positional `DamageSource` from the configured damage type and
position and ignores both decoded cause/direct IDs. Otherwise it independently looks up cause and
direct in the current level; either may remain null. A living target's handler sets walk-animation
speed to `1.5`, invulnerability time to `20`, hurt time/duration to `10`, plays its damage-type-
selected hurt sound, and records source plus current client game time. A nonliving entity's base
handler is a no-op.

The server emits ID 25 to trackers and self only on the full-damage branch where item blocking did
not select its own `onBlocked` response. Cooldown-delta hits (`tookFullDamage=false`) and that
blocked branch do not emit it. The packet communicates source/presentation, not amount, health,
absorption, knockback, death, equipment wear, or combat criteria; those authoritative results use
their own metadata/health/motion/event packets and the source-specified ENT-DAMAGE rules. A
configured damage-type raw ID absent from the session registry faults decode before entity lookup.

Primary anchors are `ClientboundDamageEventPacket#getSource`,
`ClientPacketListener#handleDamageEvent`, `LivingEntity#handleDamageEvent`,
`LivingEntity#hurtServer`, and `ServerLevel#broadcastDamageEvent`.

## Camera projection

ID 93 changes the client camera only when its entity ID resolves in the current level. A missing ID
is ignored and leaves the existing camera unchanged. The packet does not itself move the local
player, create a target, or acknowledge the serverbound request.

`ServerPlayer#setCamera` first changes authoritative camera ownership and, for a changed camera,
relocates the player to that camera's level/position. It updates chunk tracking for a nonnull target,
then sends ID 93 and resets the connection's known position. Thus any required same-dimension
position challenge or cross-dimension respawn/reprojection is ordered before the camera packet.
The near spectator-action request can enter this flow only after its mode/load/range/pickable
checks; UUID teleport instead resets camera and moves the player rather than selecting the UUID
target as camera.

Primary anchors are `ClientPacketListener#handleSetCamera`, `ClientboundSetCameraPacket#getEntity`,
`ServerPlayer#setCamera`, and the C3 serverbound spectator handlers.

## Pickup projection

For ID 124 the client resolves the source, then casts the collector lookup result to
`LivingEntity`; an absent collector falls back to the local player, while a present nonliving
collector faults that cast. An absent source makes the packet a no-op after collector resolution.
For a present source it plays the experience-orb or ordinary pickup sound and creates one pickup
particle moving from source render state to collector.

An item source subtracts the signed VarInt amount from its local stack count with wrapping int
arithmetic and removes the entity when the resulting stack is empty (`count <= 0`). A negative
amount therefore grows the local count unless arithmetic wraps it empty. An experience orb is not
removed by this handler. Every other present source, including the canonical abstract-arrow case,
is removed regardless of amount. These are client projection effects only; they do not grant an
item or experience to authoritative inventory state.

The locked server constructs this packet only when `LivingEntity#take` sees a nonremoved item
entity, abstract arrow, or experience orb on the server side. It sends to players tracking the
source, not automatically to a server-player source itself. The amount is the caller-supplied
original count; the packet has no transaction/state ID or acknowledgement.

Primary anchors are `LivingEntity#take`, `ChunkMap$TrackedEntity#sendToTrackingPlayers`,
`ClientPacketListener#handleTakeItemEntity`, and `ItemStack#shrink`.

## Respawn grammar

ID 82 repeats the common spawn record used inside play login, followed by one signed byte:

```text
dimension_type_raw_id:VarInt
dimension:identifier
obfuscated_seed:i64
game_mode:i8
previous_game_mode:i8
is_debug:boolean
is_flat:boolean
last_death_present:boolean
if last_death_present {
    last_death_dimension:identifier
    last_death_position:packed_block_position_i64
}
portal_cooldown:VarInt
sea_level:VarInt
data_to_keep:i8
```

Dimension type is a configured holder ID; dimension and last-death dimension are namespaced level
keys. Current game-mode bytes outside `0..=3` map to survival. Previous `-1` means absent and every
other out-of-range value maps to survival. Keep bit `0x01` means retain attribute modifiers and bit
`0x02` means retain entity data; higher bits are ignored. Tests are independent bit intersections,
so any byte carrying either bit has that effect.

The client compares the new dimension key with its old key. On change it creates a new
`ClientLevel` using the new dimension type/key, debug/flat flags, seed and sea level while retaining
map data, then installs it and drops level-scoped debug subscriptions. In all cases it clears the
camera, closes an open container, creates a replacement `LocalPlayer`, marks client-loaded false,
and restarts level waiting with a reason derived from death/dimension change. It preserves the old
entity ID, stats and recipe book.

With bit `0x02`, the replacement also retains last sent input/sprinting, nondefault synchronized
entity data, velocity and yaw/pitch. Without it, the player resets position and uses yaw `-180`.
With bit `0x01`, it copies all old attribute values/modifiers; without it, only base values. It then
adds the replacement player to the level and makes it the camera. The packet itself does not carry
position, inventory, effects, health, experience, permissions, chunks, or load acknowledgement.

The ordinary death respawn flow sends keep byte `0`; post-win retained-player-data respawn sends
`1`; cross-dimension teleport sends `3`. `PlayerList#respawn` orders respawn before its position
challenge, default spawn, difficulty, experience, active effects, level info and permission
projection. The separate cross-dimension path also sends respawn before difficulty/permission,
level transfer, position, abilities and new-level projection. Both restart or continue the
player-loaded synchronization boundary as specified by their owning flow; accepted ID-12 respawn
explicitly restarts the server's 60-tick client-load grace, while the client later sends
serverbound `player_loaded` through the already specified C2 terrain/readiness path. Duplicate
valid respawn packets independently replace client player/session state; there is no respawn
sequence number.

Primary anchors are `ClientboundRespawnPacket`, `CommonPlayerSpawnInfo`,
`ClientPacketListener#handleRespawn`, `PlayerList#respawn`, `ServerPlayer#teleport`, and
`ServerGamePacketListenerImpl#restartClientLoadTimerAfterRespawn`.

## C3 clientbound fault and ordering boundary

Malformed/truncated primitives, trailing bytes, absent configured holder IDs, and wrong runtime
entity types in the explicit animation/collector casts fault handling. Unknown animation actions,
missing damage/hurt/camera/source entities, arbitrary keep-mask high bits, missing damage causes,
exceptional floats/doubles, and signed pickup amounts follow their documented semantic paths.

No packet in this slice acknowledges another. Damage event, hurt yaw, motion, health/metadata and
death are distinct projections and may not be collapsed. Camera follows relocation. Respawn begins
a new player-loaded interval and precedes correction/level reprojection. Pickup is an unsequenced
visual/cache delta; authoritative inventory convergence remains owned by the C3 inventory family.

# C3 Entity Motion Projection

This family specifies nine clientbound packets that maintain an already-created entity's position,
rotation, velocity, head direction, minecart interpolation queue, and hurting-projectile
acceleration. Entity creation/removal and entity metadata/state are deliberately separate families.
All entity IDs below resolve only in the current client level and never through UUID, registry, or
another dimension.

| ID | Identity | Fields in exact order |
|---:|---|---|
| `35` | `minecraft:entity_position_sync` | entity ID VarInt; position X/Y/Z doubles; velocity X/Y/Z doubles; yaw/pitch floats; on-ground boolean |
| `53` | `minecraft:move_entity_pos` | entity ID VarInt; X/Y/Z signed big-endian shorts; on-ground boolean |
| `54` | `minecraft:move_entity_pos_rot` | entity ID VarInt; X/Y/Z signed big-endian shorts; yaw byte; pitch byte; on-ground boolean |
| `55` | `minecraft:move_minecart_along_track` | entity ID VarInt; minecart-step count VarInt; steps |
| `56` | `minecraft:move_entity_rot` | entity ID VarInt; yaw byte; pitch byte; on-ground boolean |
| `83` | `minecraft:rotate_head` | entity ID VarInt; head-yaw byte |
| `101` | `minecraft:set_entity_motion` | entity ID VarInt; compact `LpVec3` velocity |
| `125` | `minecraft:teleport_entity` | entity ID VarInt; position/velocity/rotation record; relative-flags int; on-ground boolean |
| `135` | `minecraft:projectile_power` | entity ID VarInt; acceleration-power double |

The position/velocity/rotation record is three position doubles, three velocity doubles, then yaw
and pitch floats. The relative mask uses the same fixed big-endian int and bits `0..=8` specified
for ID 72: X, Y, Z, yaw, pitch, velocity X, velocity Y, velocity Z, and rotate-velocity. Higher bits
are ignored. `LpVec3` uses the common scale/header/15-bit grammar and canonical sanitization in
[framing and primitives](framing-and-primitives.md).

Each minecart step is:

```text
position_x:f64, position_y:f64, position_z:f64
movement_x:f64, movement_y:f64, movement_z:f64
yaw:rotation_byte, pitch:rotation_byte
weight:f32
```

A rotation byte decodes as its signed byte value times `360/256` degrees. Canonical encoding floors
`angle * 256 / 360` and writes the low byte. The minecart list has no element cap below the
`8_388_608`-byte uncompressed packet bound. A negative count faults while constructing the list;
zero is valid. Position, velocity, weight, yaw/pitch and acceleration codecs perform no finite or
semantic-range validation. Booleans use the common nonzero-true decoder, and trailing bytes fault
the packet.

Primary codec anchors are the nine named packet classes, `PositionMoveRotation`, `Relative`,
`VecDeltaCodec`, `Vec3#LP_STREAM_CODEC`, and `NewMinecartBehavior$MinecartStep`.

## Relative movement and ordinary interpolation

IDs 53 and 54 decode each signed-short component against the entity's packet-position base. A zero
component preserves the corresponding base double exactly. A nonzero component becomes
`(round(base * 4096) + delta) / 4096`. After decoding a packet with position, the client replaces
the base with that decoded position. ID 56 has no position and does not change the base. This is a
quantized projection state; Ferrite must derive deltas from a per-viewer base and must not store
short deltas as authoritative coordinates. `round` and the intermediate signed-long addition use
Java semantics: a malicious nonfinite prior base converts through `Math.round` and addition can
wrap; a zero delta bypasses that conversion and preserves the nonfinite base exactly.

If the target is absent, all three packets are ignored. If it is locally authoritative, every
variant first decodes its zero-or-present deltas and replaces the base, then returns: rotation and
on-ground are not applied, and a rotation-only packet therefore merely rewrites the same base. For
any other target, a position variant submits its decoded position, optionally with byte-decoded
rotation, to the target's movement/interpolation hook; a rotation-only variant submits only its
rotation. It then installs the packet's on-ground value.

Entities without an interpolation handler apply submitted components immediately; byte rotations
are reduced modulo 360 on that direct path. A living entity and other default users schedule the
target over three client ticks. Each tick uses `1 / remaining_steps`, shortest-path yaw interpolation
and linear pitch/position interpolation, including collision-safe movement or rotation accumulated
since the prior interpolation tick. Repeating the identical active target is a no-op. Entity types
may supply zero steps or specialized handlers, so the three-tick rule is not a universal wire
constant.

Primary handler anchors are `ClientPacketListener#handleMoveEntity`,
`Entity#moveOrInterpolateTo`, and `InterpolationHandler`.

## Absolute sync

ID 35 first replaces a present target's packet-position base with the encoded position. For a
locally authoritative target it then returns immediately: position, encoded velocity, rotation,
and on-ground are all ignored after the base update. For any other target, squared distance from
current position strictly greater than `4096.0` selects an immediate snap. At or below that bound,
a currently ticking entity enters its movement/interpolation hook; a nonticking entity snaps.
After either path, a noninterpolating vehicle carrying the local player immediately repositions
that rider and refreshes the rider's old pose. The client finally installs on-ground.

The encoded velocity in ID 35 is **never applied by this client handler**, including on the snap
path. Velocity convergence therefore remains owned by ID 101 and entity simulation. This is not
permission to omit the six doubles from the wire record. A missing entity ignores the packet.

Primary anchors are `ClientboundEntityPositionSyncPacket#of` and
`ClientPacketListener#handleEntityPositionSync`.

## Teleport application and former-vehicle fallback

For a present ID-125 target, the client first obtains a source record. During active interpolation
this uses the interpolation target position/rotation rather than the entity's currently rendered
pose; otherwise it uses current pose. Velocity is the entity's known movement, which for a live
player-controlled vehicle can come from its controller. A set position/rotation/velocity bit adds
the corresponding source value; an unset bit replaces it. Rotate-velocity rotates source velocity
by the old-to-new pitch and yaw difference before the three velocity-component additions or
replacements. Final pitch is clamped to `[-90, 90]` (infinities reach an endpoint and NaN remains
NaN); yaw is not normalized here.

The client requests the movement/interpolation path when the entity is ticking, when it is not
locally authoritative, or when any of X/Y/Z is relative. If that request is made and the squared
position distance is at most `4096.0`, it submits position/rotation, installs the resulting velocity,
and treats the operation as interpolated even for an entity whose hook applies immediately.
Otherwise it directly sets position, velocity, yaw and pitch. The direct branch also applies the
same change/relative calculation to old position/rotation, with old velocity treated as zero, and
stores those old values. Both branches then install on-ground.

After the direct branch, a target carrying the local player repositions that rider and refreshes
the rider's old pose. If the vehicle is locally authoritative, the client immediately sends
serverbound ID 34 `move_vehicle` with the vehicle's resulting absolute pose. No echo is sent for the
interpolation branch or for a present target that does not indirectly carry the player.

Removal of a vehicle carrying the local player retains that vehicle ID in client session state. If
ID 125 later names that missing retained ID, the client instead applies the change immediately to
the local player, ignores the packet's on-ground value, and sends serverbound ID 31
`move_player_pos_rot` with the resulting absolute pose and both on-ground/horizontal-collision flags
false. It does not clear the retained ID. A different missing ID is ignored. Creation of a new
entity with the retained ID, not this teleport, clears the fallback marker.

Primary anchors are `PositionMoveRotation#calculateAbsolute`,
`ClientPacketListener#handleTeleportEntity`, `#setValuesFromPositionPacket`, and
`#handleRemoveEntities`.

## Velocity, head, projectile, and minecart projections

ID 101 ignores a missing entity and otherwise calls its `lerpMotion` hook with the decoded finite
compact vector. The base entity implementation sets velocity immediately; old minecart behavior
also records its interpolation target, while entity types may specialize the hook. It does not set
position, on-ground, or a position-codec base and has no acknowledgement.

ID 83 ignores a missing entity. The base entity applies decoded head yaw immediately; a living
entity instead sets a three-tick shortest-path head-yaw target, using `1 / remaining_steps` each
tick. It does not alter body yaw or look pitch.

ID 135 changes only a present `AbstractHurtingProjectile`, assigning the raw decoded double to its
acceleration-power field. Missing and wrong-runtime-type entities are ignored; NaN and infinities
are accepted. The ordinary tracker emits ID 101 and ID 135 together in that order inside one bundle
when such a projectile's tracked motion changes, so acceleration accompanies rather than replaces
velocity.

ID 55 is meaningful only for an `AbstractMinecart` created with the enabled
`minecraft:minecart_improvements` feature and therefore using `NewMinecartBehavior`. The handler
appends all decoded steps, including an empty list and arbitrary weights, to its pending queue;
missing entities, wrong types, and old-behavior minecarts ignore the packet. At a client tick whose
current three-tick window expires, pending steps replace the current queue, their float weights are
summed as doubles, and a nonzero total opens a three-tick window. Selection walks only positive
weights against that weighted progress; if none qualifies, it selects the last step. Position and
movement interpolate linearly and rotations use shortest-path interpolation from the preceding
step (or the minecart's captured pre-window state). Zero, negative, NaN and infinite weights follow
those raw arithmetic/comparison rules rather than validation.

Primary anchors are `ClientPacketListener#handleSetEntityMotion`, `#handleRotateMob`,
`#handleProjectilePowerPacket`, `#handleMinecartAlongTrack`, `LivingEntity#lerpHeadTo`, and
`NewMinecartBehavior#lerpClientPositionAndRotation`.

## Locked server publication and ordering

The ordinary tracker initializes its delta base and last-sent motion/body/head rotations from the
entity. On each configured update interval, a nonpassenger regular entity uses rounded 1/4096
deltas. Position changes at squared distance at least `7.62939453125e-6`, or the 60-tick refresh,
are eligible for relative publication. It instead sends ID 35 when precise positioning is required,
any encoded component is outside signed-short range, more than 400 ordinary tracker passes have
elapsed since the last absolute sync, it just stopped riding, or on-ground changed. Otherwise it
chooses ID 54 for position plus rotation (and always for an abstract arrow), ID 53 for position
alone, or ID 56 for rotation alone. Byte-rotation change uses absolute signed-byte subtraction at
least one, not a wrap-aware angular comparison.

When velocity tracking is enabled, the entity needs sync, or a living entity is fall-flying, a
squared velocity difference greater than `1e-7` is sent; transition to exact zero is also sent even
below that threshold. ID 101, and the hurting-projectile ID-135 bundle member, are queued **before**
the selected position/rotation packet. Dirty metadata/attributes follow that packet. A qualifying
head-byte change then sends ID 83. Finally `hurtMarked` emits a separate ID 101 to trackers and self
and clears the marker. Passengers send only qualifying ID 56, reset the delta base to their current
position, and send dirty state. New-behavior minecarts instead send ID 55 from their recorded steps
or a one-step current snapshot and reset their tracker base.

Entity teleport transitions send ID 125 directly to riding players: the controlling player receives
the transition's relative record, while another indirect player passenger receives the entity's
current absolute record with no relative flags. Ordinary trackers subsequently converge other
viewers through their normal absolute/relative stream. None of the nine packets carries a sequence
or acknowledgement; ID-125 echoes above are ordinary movement validation inputs, not teleport
challenge acknowledgements.

Primary server anchors are `ServerEntity#sendChanges`, `#handleMinecartPosRot`,
`Entity#sendTeleportTransitionToRidingPlayers`, and direct ID-101 publication sites in player
knockback and mace handling.

## C3 entity-motion fault boundary

Malformed/truncated primitives, overlong VarInts, a negative minecart count, an impossible
frame-bounded list, and trailing bytes fault the packet. Missing targets and the documented wrong
runtime types are semantic ignores. Signed-short endpoints, all rotation bytes, all relative-mask
high bits, arbitrary on-ground bytes, raw minecart weight floats, raw absolute/teleport IEEE values,
and raw projectile-power doubles follow their documented decode/application branches. Compact
ID-101 velocity remains finite because canonical and accepted `LpVec3` forms decode to finite
bounded components.

# C3 Entity Spawn and Removal

This family specifies creation and destruction of the client-level entity projection. It does not
specify the metadata, attributes, equipment, passenger, or leash packet codecs that may immediately
follow a spawn; those remain an independent entity-state family. Numeric entity IDs are session
projection keys. UUID instead projects the entity's normalized authoritative UUID (and a player's
profile UUID); it is not interchangeable with the numeric ID or raw registry value.

| ID | Identity | Fields in exact order |
|---:|---|---|
| `1` | `minecraft:add_entity` | entity ID VarInt; UUID; static entity-type raw ID VarInt; X/Y/Z doubles; compact `LpVec3` movement; pitch/yaw/head-yaw bytes; data VarInt |
| `77` | `minecraft:remove_entities` | entity-count VarInt; that many entity ID VarInts |

UUID is two big-endian signed longs. The rotation bytes decode as signed byte times `360/256`
degrees; note that pitch precedes yaw on the wire. The entity type resolves through the
158-entry defaulted static mapping in
[wire registry and palette mappings](registry-and-metadata-mappings.md). Negative and out-of-range
raw IDs resolve to the registry default `minecraft:pig`; decode then continues with position.
`LpVec3` uses the common finite compact-vector grammar. The remaining doubles and signed VarInts
have no codec-level semantic range checks.

ID 77's list has no count cap below transport. A negative count runs no loop and produces a valid
empty list if the packet ends there; a positive impossible count eventually faults on truncation.
Duplicate and negative entity IDs are valid list elements. Both packet bodies reject trailing bytes.

Primary codec anchors are `ClientboundAddEntityPacket`, `ClientboundRemoveEntitiesPacket`,
`ByteBufCodecs#registry`, and `FriendlyByteBuf#readIntIdList`.

## Client construction and insertion

The add handler first clears `removedPlayerVehicleId` when its retained value equals the packet ID.
This happens before type construction and remains cleared even if all later creation is skipped or
faults. It then constructs by type:

- `minecraft:player` requires an existing `PlayerInfo` for the packet UUID and creates a
  `RemotePlayer` from that profile. Missing info logs and skips the entity. The registry type's own
  factory intentionally creates nothing and is not used on this branch.
- Every other type calls `EntityType#create(level, LOAD)`. This enforces the type's required feature
  set and its peaceful-difficulty allowance; failure or a null factory result logs and skips the
  entity. It does not apply summon/save eligibility as a substitute check.

A constructed entity runs its `recreateFromPacket` chain **before** level insertion. The base chain
sets packet-position-codec base, packet ID/UUID, pose and compact movement. A nonliving base entity
uses the decoded position and yaw/pitch directly and ignores head yaw. A living entity instead
clamps X/Z to `[-30_000_000,30_000_000]`, clamps pitch to `[-90,90]`, initializes body/head and
their old values from head yaw, and installs yaw/pitch plus movement. Its Y coordinate is not
clamped. Rotation bytes are always finite; raw position doubles retain Java NaN/infinity clamp and
floor behavior. `RemotePlayer` additionally makes current pose its old pose.

After recreation, `ClientLevel#addEntity` first discards any entity currently found under the same
numeric ID, then inserts the new instance. Consequently type-specific owner lookup during recreation
sees the pre-replacement level and can resolve an old same-ID entity; replacement happens only
afterward. A duplicate UUID under a different ID is stranger: `EntityLookup` warns and refuses its
ID/UUID-map insertion, but the section manager still adds it, runs tracking/ticking callbacks, and
the handler continues with sound/seen-player work. A conforming server emits neither collision.
The implicit same-ID discard does not run ID-77's player-vehicle retention test.

On successful construction the handler starts a minecart rolling sound for any abstract minecart,
or queues the aggressive/nonaggressive flying sound selected from a bee's state at that moment.
Pairing metadata has not yet been handled, so canonical bee selection initially observes constructor
state. A created player with corresponding info is added to `seenPlayers`; removal does not by
itself remove player info or that seen-player history.

Primary anchors are `ClientPacketListener#handleAddEntity`, `#createEntityFromPacket`,
`#postAddEntitySoundInstance`, `Entity#recreateFromPacket`, `LivingEntity#recreateFromPacket`,
`ClientLevel#addEntity`, `TransientEntitySectionManager#addEntity`, and `EntityLookup#add`.

## Spawn-data discriminator

The signed `data` VarInt is interpreted only by the following recreate families. All other types
ignore it after decode.

| Runtime family | Canonical server value | Exact client interpretation |
|---|---|---|
| item frame, including glow frame | attachment direction's 3D value | `Direction.from3DDataValue(data)`, where `abs(data % 6)` maps `0=down, 1=up, 2=north, 3=south, 4=west, 5=east`; all six are accepted and recalculate rotation/bounds |
| painting | horizontal attachment direction `2..=5` | same modulo/absolute direction mapping, then horizontal-only validation; a result down/up faults recreation |
| falling block | global block-state raw ID | `Block.stateById`; every absent, negative, or oversized ID becomes the air default state, then start position is the resulting block position |
| Warden | `1` only while spawning in `EMERGING` pose, otherwise `0` | exact value `1` sets emerging; every other int leaves constructor/default pose |
| any `Projectile` | owner entity ID, or `0` for absent | lookup in the pre-insertion current level; a present result becomes owner, while missing leaves owner absent, so malicious ID zero can bind if an entity zero exists |
| fishing bobber | owner ID, or its own ID when server owner is absent | applies the projectile lookup, then requires the resulting owner to be a player; otherwise logs and marks the hook discarded, after which the handler still reaches ordinary insertion |

For item frames, paintings and leash knots the canonical packet constructor uses integer attachment
block coordinates rather than the tracker's floating base. Their block-attached recreation updates
that anchor and recalculates the type-specific bounding box; leash-knot data remains zero and is
ignored. Direction and falling-state numbers are not entity-type IDs.

Primary anchors are `ItemFrame#getAddEntityPacket`, `Painting#getAddEntityPacket`,
`LeashFenceKnotEntity#getAddEntityPacket`, `FallingBlockEntity#getAddEntityPacket`,
`Warden#getAddEntityPacket`, `Projectile#getAddEntityPacket`, and
`FishingHook#getAddEntityPacket` plus their recreate overrides.

## Other recreation specializations

These overrides do not reinterpret `data` but are part of exact spawn projection:

- an ender dragon assigns its eight client-only part IDs to wrapping signed-int values
  `spawn_id + 1` through `spawn_id + 8` and client-level tracking registers those parts separately;
- a Shulker resets body yaw/current-old to zero after living reconstruction;
- llama spit creates seven spit particles with horizontal motion multipliers `0.4` through `1.0`
  and reapplies packet movement; a Shulker bullet likewise reapplies movement;
- an abstract minecart passes initial movement into its selected old/new behavior; and
- every ordinary projectile performs the owner lookup above before insertion.

Spawn does not carry on-ground, metadata, attributes, equipment, passengers, leash, health, effects,
inventory, or removal reason. Those must arrive through their owning packets and may not be guessed
from type or `data`.

Primary anchors are `EnderDragon#recreateFromPacket`, `Shulker#recreateFromPacket`,
`LlamaSpit#recreateFromPacket`, `ShulkerBullet#recreateFromPacket`, and
`AbstractMinecart#recreateFromPacket`.

## Server pairing order and visibility

The ordinary tracker never pairs an entity to itself. A viewer enters the tracking set only when
horizontal squared distance is at most the square of the smaller of effective tracking range and
view distance in blocks, `entity.broadcastToPlayer(viewer)` succeeds, and the entity's chunk is
tracked. Effective range is the server-scaled maximum of the entity type's range and every indirect
passenger type's range.

On the first qualifying transition the server calls `updateDataBeforeSync`, builds one bundle, and
orders:

1. ID 1 add packet;
2. nondefault synchronized metadata, when any;
3. nonempty syncable living attributes;
4. nonempty living equipment slots;
5. this entity's passenger list, when nonempty;
6. its vehicle's passenger list when this entity is a passenger; and
7. its leash link when currently leashed.

Only after sending the bundle does it call `startSeenByPlayer`. The add packet normally takes ID,
UUID, entity type, position base, last-sent pitch/yaw/movement and last-sent head yaw from
`ServerEntity`. Hanging entities use the block-coordinate constructor described above.
Canonical player-info publication precedes player entity pairing because the client cannot construct
a remote player without it.

Leaving visibility or entity removal calls `stopSeenByPlayer` and then sends canonical ID 77 with a
single entity ID. The multi-ID codec is therefore a valid wider client input, not the ordinary locked
server emission shape.

Primary anchors are `ChunkMap$TrackedEntity#updatePlayer`, `#getEffectiveRange`,
`ServerEntity#addPairing`, `#sendPairingData`, and `#removePairing`.

## Removal and former-player-vehicle retention

The client processes ID 77 entries in wire order. A missing ID is ignored. Before removing a present
entity, it tests whether that entity indirectly carries the local player; if so, it replaces
`removedPlayerVehicleId` with this ID. Removal uses reason `DISCARDED`, detaches the entity from its
vehicle and all passengers, runs client tracking/ticking teardown, removes player/dragon auxiliary
tracking, invokes client-removal hooks, and drops its debug-subscriber projection. Those relationship
changes affect later IDs in the same packet, so list order is observable. A duplicate ID removes on
its first occurrence and is missing thereafter.

The retained vehicle ID is not an acknowledgement. It exists only for the already-specified
ID-125 missing-vehicle teleport fallback, survives unrelated removals, and is cleared by a later
ID-1 packet with the same numeric ID even when that add cannot construct an entity. Ordinary entity
removal has no client response, does not remove independent player-info state, and does not infer
authoritative death, drops, inventory, or UUID destruction.

Primary anchors are `ClientPacketListener#handleRemoveEntities`, `ClientLevel#removeEntity`,
`Entity#setRemoved`, and `ClientLevel$EntityCallbacks#onTrackingEnd`.

## C3 entity-spawn fault boundary

Malformed/truncated primitives, impossible positive removal lists, overlong VarInts, trailing bytes,
and type-specific recreation faults (notably a painting direction resolving vertical) fault the
packet/handler. Negative/out-of-range entity types default to pig and negative removal count is a
valid empty list. Missing player info, failed type spawn checks/factories, missing removal IDs,
unknown projectile owners, invalid falling-state IDs, duplicate list IDs and the documented
duplicate ID/UUID cases follow their semantic paths. Add/remove carry no sequence and acknowledge
no request.

# C3 Entity Metadata, Attributes, Equipment, and Relationships

This family projects mutable entity state after the independently specified spawn packet. Numeric
entity IDs are current-level lookup keys. Metadata serializer IDs, metadata slots, equipment
ordinals, registry holder IDs and data-component type IDs are five different wire domains; none is
an authoritative Ferrite identifier.

| ID | Identity | Fields in exact order |
|---:|---|---|
| `99` | `minecraft:set_entity_data` | entity ID VarInt; zero or more entries of slot unsigned byte, serializer ID VarInt, serializer value; `255` byte terminator |
| `100` | `minecraft:set_entity_link` | source entity signed int; destination entity signed int |
| `102` | `minecraft:set_equipment` | entity ID VarInt; one or more entries of slot/continue byte and optional item stack |
| `107` | `minecraft:set_passengers` | vehicle ID VarInt; passenger count VarInt; that many passenger ID VarInts |
| `131` | `minecraft:update_attributes` | entity ID VarInt; attribute count VarInt; that many attribute snapshots |

An ID-99 slot is `0..=254`; `255` ends the list and has no serializer or value. Each serializer
value follows the exact 43-entry table in
[wire registry and metadata mappings](registry-and-metadata-mappings.md). There is no entry count or
duplicate-slot rejection below the frame limit. An unknown serializer ID faults immediately.

ID 102 uses bit `0x80` as “another entry follows” and the low seven bits as the ordinal
`0=mainhand, 1=offhand, 2=feet, 3=legs, 4=chest, 5=head, 6=body, 7=saddle`. Low values `8..=127`
fault by indexing the eight-entry slot list. At least one entry is required: after the entity ID the
decoder always reads a descriptor. Each item stack is:

```text
count:VarInt
if count > 0:
    item:static minecraft:item holder VarInt
    present_component_count:VarInt
    removed_component_count:VarInt
    present entries: component-type VarInt, then that type's trusted stream value
    removed entries: component-type VarInt
```

Any count at most zero denotes the empty stack and consumes no item or component fields. Positive
counts have no packet-specific stack-size clamp. Component entries preserve the patch map, not a
resolved full item prototype. Present entries are decoded before removals; duplicate types replace
earlier map values, so a later removal of the same type wins. The 1,537 item IDs, 111 component-type
IDs, and every component value codec are locked mappings described in the mapping section.

The two component counts are raw signed VarInts rather than a bounded collection helper. If both
are zero, the patch is empty. Otherwise their wrapping signed sum becomes the initial map capacity,
capped only above at 65,536: a negative capacity faults, while a negative individual count runs no
loop when the sum is still nonnegative. This is an observable malformed-input distinction, not a
license for an encoder to emit negative counts.

ID 107 delegates its array bound to `readVarIntArray`: after the vehicle ID, the declared count may
not exceed the remaining byte count. This is sufficient because every passenger VarInt consumes at
least one byte. A negative count faults array allocation. Empty arrays, duplicate IDs and arbitrary
signed passenger IDs otherwise decode.

Each ID-131 snapshot is a registry-aware `minecraft:attribute` holder VarInt, base double,
modifier count VarInt, then that many modifiers. A modifier is identifier `UTF(32767)`, amount
double, and operation VarInt. Attribute count is restricted to `0..=128`; modifier count uses the
generic nonnegative collection bound. Operations `0/1/2` are add-value, add-multiplied-base and
add-multiplied-total. The by-ID policy maps every other signed operation value to operation zero.
All doubles preserve their IEEE bits through the packet codec; attribute instances may sanitize a
base during semantic application.

Primary codec anchors are `ClientboundSetEntityDataPacket`,
`SynchedEntityData$DataValue`, `ClientboundSetEntityLinkPacket`,
`ClientboundSetEquipmentPacket`, `ItemStack#OPTIONAL_STREAM_CODEC`,
`DataComponentPatch#STREAM_CODEC`, `ClientboundSetPassengersPacket`,
`FriendlyByteBuf#readVarIntArray`, `ClientboundUpdateAttributesPacket`, and
`ClientboundUpdateAttributesPacket$AttributeSnapshot`.

## Metadata application and publication

The locked hierarchy declares 221 static metadata accessors. A reproducible jar audit records each
row as `declaring_class#field<TAB>slot<TAB>serializer_id`, sorts lexicographically and hashes every
newline-terminated row to SHA-1 `b489eec18fc1981ebfb7ac97c54a4485fe2f938a`; all top-level
`net.minecraft.world.entity` classes load without failure. The highest declared slot is 24, although
the generic allocator permits through 254. Exact inherited slots, serializers, defaults and update
callbacks come from each class's `defineSynchedData`, static `defineId` calls and superclass chain;
Ferrite must generate its 26.2 type table from that complete inventory rather than maintain a
second guessed numbering scheme.

On ID 99 the client first resolves the entity. A missing entity ignores the already decoded list. A
present entity applies entries in wire order. Each slot must exist in that runtime entity's
`itemsById` array and its declared serializer object must equal the wire serializer. An absent slot
faults by invalid/null array access; a wrong serializer throws an explicit invalid-item-type fault.
Every successful entry replaces the slot value and immediately invokes the accessor-specific
callback. After the list, one aggregate callback receives the entire ordered list. Duplicates
therefore apply and callback more than once; the last successful value remains.

Before pairing or update, the server calls `updateDataBeforeSync`. Pairing includes only nondefault
values, in ascending slot-array order. Runtime dirty packing also scans ascending slots, clears each
dirty flag, refreshes the pairing snapshot of nondefault values, and publishes ID 99 to tracking
players **and the entity itself**. A value returning to its default is still dirty and therefore
sent; it merely disappears from later pairing snapshots. Empty/no-dirty metadata emits no packet.

Primary anchors are `SynchedEntityData#defineId`, `Builder#define`, `#getNonDefaultValues`,
`#packDirty`, `#assignValues`, every entity `defineSynchedData`/`onSyncedDataUpdated`, and
`ServerEntity#sendDirtyEntityData`.

## Attribute replacement

ID 131 first resolves the entity. Missing entities ignore all decoded snapshots. A present
nonliving entity throws. For a living entity, snapshots run in wire order:

1. resolve the attribute instance already present in that entity's `AttributeMap`;
2. if absent, warn and skip that complete snapshot;
3. otherwise set/sanitize its base, remove its complete current modifier set, and add every wire
   modifier as transient in wire order.

Thus the packet is replacement, not an additive patch. Repeated snapshots for one attribute replace
again. Modifier identity is its namespaced identifier; colliding identifiers follow
`AttributeInstance#addTransientModifier` rather than forming duplicate mathematical terms. The
packet cannot create an attribute that the entity type does not own.

At pairing, the server sends every nonempty set of client-syncable attribute instances. During
tracking it sends only `attributesToSync`, to tracking players and self, then clears that set after
publication. Equipment modifiers enter the ordinary `AttributeMap` first and consequently can
cause a following attribute packet; clients must not derive current attribute values merely from
the equipment payload.

Primary anchors are `Attribute#STREAM_CODEC`, `AttributeMap#getSyncableAttributes`,
`#getAttributesToSync`, `AttributeInstance#setBaseValue`, `#removeModifiers`,
`#addTransientModifier`, `ClientPacketListener#handleUpdateAttributes`, and
`ServerEntity#sendDirtyEntityData`.

## Equipment replacement

For ID 102, a missing or nonliving entity ignores every decoded entry. A living target invokes
`setItemSlot` for each pair in wire order, so repeated slots are legal and the last application
wins. The stack includes its exact count and component patch; the client does not synthesize omitted
components from another item or protocol version before constructing the item stack.

Pairing walks all eight slot ordinals and sends only nonempty stacks, copied into one packet in slot
order. Runtime equipment detection compares current and remembered stacks with
`ItemStack#matches`, including count, item and components. It removes old location-based effects,
installs new modifiers/effects, copies every remaining changed stack into ordinal order, updates the
remembered snapshot, and broadcasts one packet to tracking players. Empty changed stacks are
included to clear slots. An exact main/offhand swap instead publishes entity event 55 and removes
both hand entries from the equipment packet; other simultaneous changes still publish normally.

Primary anchors are `EquipmentSlot`, `LivingEntity#collectEquipmentChanges`,
`#handleHandSwap`, `#handleEquipmentChanges`, `#equipmentHasChanged`,
`ClientPacketListener#handleSetEquipment`, and `ServerEntity#sendPairingData`.

## Passenger-list replacement

ID 107 is a complete direct-passenger list for one vehicle. An unknown vehicle warns and does
nothing. For a present vehicle the client records whether it already indirectly carried the local
player, ejects every current passenger, then processes wire IDs in order. Present passengers call
forced `startRiding(vehicle, true, false)`; missing IDs are skipped and the return value is ignored.
This can detach a present passenger from another vehicle. Duplicate, cyclic or rejected requests
therefore follow `Entity#startRiding` sequentially rather than being normalized in advance.

When the local player is successfully encountered, the retained removed-vehicle marker is cleared.
If the vehicle did not already carry that player, a boat also copies its yaw into the player's
current/old/head yaw, and the client displays and narrates the dismount-key onboarding message once
for that transition. Removing the player with a later list does not show that message.

The server compares direct passenger-list equality every tracker tick. On change it broadcasts the
new full list only to tracking players whose own membership is equal in old and new lists. A
`ServerPlayer` whose membership changes instead receives the full list directly from its successful
`startRiding` or `removeVehicle` path; starting also positions the rider, issues the ordinary player
position challenge, and projects a living vehicle's effects first. Pairing sends a nonempty entity
list and, for a passenger entity, its vehicle's full list after equipment.

Primary anchors are `ClientPacketListener#handleSetEntityPassengersPacket`,
`Entity#ejectPassengers`, `#startRiding`, `ServerPlayer#startRiding`, `#removeVehicle`, and
`ServerEntity#sendChanges`/`#sendPairingData`.

## Delayed leash relation

ID 100 uses two fixed big-endian signed ints, unlike the other four packets' entity VarInts. The
canonical constructor encodes source entity ID and destination holder ID, or destination zero for
no holder. The client ignores a missing or non-`Leashable` source. A leashable source replaces its
leash data with the destination as a delayed ID; nonzero destination resolution occurs lazily when
the current level later contains that entity. Zero remains no holder. A missing nonzero destination
is retained for later resolution rather than faulting or binding entity zero.

Canonical attach/reassign broadcasts only when the mutation requests a packet and the source is in
a server level. Canonical detach broadcasts destination zero when its send flag is true. Pairing
always emits the current nonnull holder after passengers. The relation carries no owner UUID,
distance, lead item, acknowledgement or persistence decision; those are authoritative gameplay
state projected through current connection-local entity IDs.

Primary anchors are `ClientPacketListener#handleEntityLinkPacket`, `Leashable#setDelayedLeashHolderId`,
`#getLeashHolder`, `#setLeashedTo`, `#dropLeash`, and `ServerEntity#sendPairingData`.

## C3 entity-state fault and acknowledgement boundary

Malformed/truncated values, residual bytes, metadata without a `255` terminator, unknown metadata
serializer, invalid equipment ordinal, missing equipment entry, negative/impossible bounded
passenger or attribute counts, invalid registry holders/components and strict identifier faults
enter normal packet failure. Component-patch counts retain the distinct raw signed behavior above.
Wrong metadata slot/type and attributes addressed to a present nonliving entity fault in the client
handler. Missing entity targets, missing attribute instances, unknown passenger IDs, unknown leash
destinations and the documented wrong runtime types follow their ignore/deferred paths.

None of these packets has a sequence or response. They acknowledge neither spawn nor interaction.
Convergence comes from ordered authoritative replacement, later dirty projections, tracker pairing
and ordinary movement/teleport protocols. Ferrite may retain typed dirty sets and normalized
relationships internally, but it must never persist packet IDs, serializer IDs, slots, equipment
ordinals, raw registries, component IDs, modifier operation IDs or client delayed-resolution state.

# C3 Explosions and Mob Effects

These three packets project an already authoritative explosion result or a living entity's active
effect state. ID 36 contains presentation inputs and only the receiving player's knockback; it is
not an explosion request or a destroyed-block list. IDs 78/132 target connection-local entity IDs.

| ID | Identity | Fields in exact order |
|---:|---|---|
| `36` | `minecraft:explode` | center X/Y/Z doubles; radius float; calculated block count fixed big-endian signed int; optional knockback boolean and X/Y/Z doubles; particle; sound-event holder; weighted block-particle list |
| `78` | `minecraft:remove_mob_effect` | entity ID VarInt; mob-effect holder VarInt |
| `132` | `minecraft:update_mob_effect` | entity ID VarInt; mob-effect holder VarInt; amplifier VarInt; duration ticks VarInt; flags byte |

ID 36 uses three different particle/sound forms. The primary particle is a strict static
particle-type VarInt followed immediately by that type's options. The sound holder begins with a
VarInt: zero means an inline identifier plus boolean-optional fixed-range float; any nonzero value
minus one is a strict ID in the connection sound-event registry. The final weighted list is a
nonnegative VarInt count. Each entry is particle/options, scaling float, speed float, then weight
VarInt. Negative weights fault construction; zero is legal; the decoded total may not exceed
`2,147,483,647`. The generic list count has no smaller packet cap.

ID 132 flag bits are `0x01=ambient`, `0x02=visible`, `0x04=show icon`, and `0x08=blend`; every
higher bit is ignored. There are no conditional fields. The packet codec itself accepts every
signed amplifier and duration VarInt. Client `MobEffectInstance` construction clamps amplifier to
`0..=255`, retains duration verbatim, treats exactly `-1` as infinite, and gives other nonpositive
durations no remaining tick. A normal server instance already enforces the amplifier range.

Primary codec anchors are `ClientboundExplodePacket#STREAM_CODEC`, `Vec3#STREAM_CODEC`,
`ParticleTypes#STREAM_CODEC`, `SoundEvent#STREAM_CODEC`, `WeightedList#streamCodec`,
`ExplosionParticleInfo#STREAM_CODEC`, `ClientboundRemoveMobEffectPacket#STREAM_CODEC`,
`ClientboundUpdateMobEffectPacket#read/write`, and `MobEffect#STREAM_CODEC`.

## Explosion publication and client presentation

`ServerLevel#explode` resolves block interaction from the explosion interaction/gamerules, creates
one `ServerExplosion`, and completes it before network publication. Completion emits the game
event, calculates affected positions, damages entities, optionally interacts with blocks, optionally
creates fire, and returns the calculated-position list size as ID 36's block count. It selects the
small primary particle when radius is below 2 or blocks are not being interacted with; otherwise it
selects the large primary particle.

Every server player whose squared distance from center is strictly less than 4096 receives one
packet. Center, radius, count, selected primary particle, sound and weighted recipes are common;
optional knockback is independently the receiving player entry in the explosion hit-player map.
Players at exactly 64 blocks do not receive it. No source entity, damage, fire, affected positions,
block interaction, random seed or acknowledgement is carried.

The client first plays the holder sound locally at center in the blocks source, volume 4, no delay,
and pitch `(1 + (random1 - random2) * 0.2) * 0.7`. It then adds the primary particle at center with
velocity `(1,0,0)` and submits center/radius/count/recipes to `ClientExplosionTracker`. Finally, if
knockback is present, it adds the vector to the local player's existing delta movement; it does not
set velocity or emit movement immediately as an acknowledgement.

The tracker queues only a recipe list with positive total weight. On the next tick it discards all
queued explosions unless particle settings are `ALL`. Otherwise it totals queued block counts,
attempts `min(total, 512)` samples, selects an explosion by block-count weight and a recipe by its
weight, samples within the radius, and spawns only when the sampled block is air. It clears the
queue after that tick. A zero or negative total produces no sample. If signed block counts mix to a
positive total, their raw subtraction order distorts selection; a sum above signed-int maximum
faults. Radius, center, count, scaling and speed
have no packet-level semantic clamp and flow into that presentation math.

## Mob-effect replacement, removal, and audience

For ID 132 the client resolves the entity and requires a `LivingEntity`; missing and other runtime
types are ignored. It constructs an effect instance with the decoded holder, raw duration, clamped
amplifier, three presentation flags, no hidden effect and fresh blend state. If blend is clear it
immediately skips blending. `forceAddEffect` then honors `canBeAffected`, replaces any same-holder
instance rather than merging duration/amplifier, copies the prior blend state on replacement, and
runs the ordinary add/update hooks. ID 78 uses the same target/type gate and removes that holder
with `removeEffectNoUpdate`; an absent effect is a silent no-op.

Canonical server effect publication is deliberately not a general tracking broadcast. Adding or
updating a living entity marks particle metadata dirty, performs any required effect-attribute
modifier change, then sends ID 132 with blend clear to direct `ServerPlayer` passengers. A
`ServerPlayer` also receives its own packet: a newly added effect sets blend, while an update clears
it. A remaining finite effect whose duration is divisible by 600 triggers the same update path with
no attribute refresh. Removal removes attributes, sends ID 78 to direct player passengers,
refreshes affected attributes, then sends a player its own removal. Metadata particles and
syncable attributes converge later through their independent ID 99/131 paths.

Initial player entry replays every active self effect with blend clear in current hash-map iteration
order. When a player successfully starts riding a living vehicle, the server positions and issues
the position challenge, replays the vehicle's complete active-effect collection with blend clear,
then sends the complete passenger list. Dismount sends a removal for every active vehicle effect
before the passenger list. Indirect passengers and ordinary tracking viewers receive no active
effect packets merely for viewing; visible aggregate particles remain metadata.

Malformed/truncated values, residual bytes, unknown particle/effect/registered-sound holders,
invalid delegated particle options, negative weighted-list counts/weights, and overflowing total
recipe weight enter normal packet failure. Unknown/nonliving effect targets follow the ignore path.
These packets have no sequence, correction or response. Ferrite retains namespaced effect and
explosion state while raw holders, flags, particle recipes/counts and client blend/tracker state
remain version-local projection details.

# C3 Container Projection and Convergence

Seven clientbound packets open/close an ordinary menu and project its slots, cursor, properties, or
the player's inventory. Container IDs are signed VarInts; the named codec does not constrain them
to a byte or nonnegative range.

| ID | Identity | Fields in exact order |
|---:|---|---|
| `17` | `minecraft:container_close` | container ID VarInt |
| `18` | `minecraft:container_set_content` | container ID VarInt; state ID VarInt; item-stack list; carried item stack |
| `19` | `minecraft:container_set_data` | container ID VarInt; property ID signed big-endian short; value signed big-endian short |
| `20` | `minecraft:container_set_slot` | container ID VarInt; state ID VarInt; slot signed big-endian short; item stack |
| `59` | `minecraft:open_screen` | container ID VarInt; strict `minecraft:menu` raw ID VarInt; trusted registry-aware component NBT title |
| `96` | `minecraft:set_cursor_item` | item stack |
| `108` | `minecraft:set_player_inventory` | inventory slot VarInt; item stack |

The content list starts with a VarInt count and then that many optional stacks; its generic list
codec has no packet-specific maximum below signed-int allocation limits. Every item occurrence uses
the same optional grammar: count VarInt, with any value at most zero denoting empty and consuming no
item/component fields; a positive count is a strict 1,537-entry item holder plus the 111-type
component patch described in the entity-state slice. Positive counts have no packet-specific
stack-size clamp. The open title is trusted component NBT rather than JSON or UTF text. The menu
type resolves through the locked 25-entry static registry and cannot be inline.

Primary codec anchors are `ClientboundContainerClosePacket#STREAM_CODEC`,
`ClientboundContainerSetContentPacket#STREAM_CODEC`,
`ClientboundContainerSetDataPacket#STREAM_CODEC`,
`ClientboundContainerSetSlotPacket#STREAM_CODEC`, `ClientboundOpenScreenPacket#STREAM_CODEC`,
`ClientboundSetCursorItemPacket#STREAM_CODEC`, `ClientboundSetPlayerInventoryPacket#STREAM_CODEC`,
`ItemStack#OPTIONAL_STREAM_CODEC`, and `ItemStack#OPTIONAL_LIST_STREAM_CODEC`.

## Menu creation, state IDs and server publication

Opening an ordinary menu first server-closes any noninventory menu: it sends ID 17, invokes removal
and transfers shared remote state. The per-player container counter then advances
`counter % 100 + 1`, yielding canonical IDs `1..=100`. If menu creation succeeds, the server sends
ID 59 before attaching listeners/synchronizer. Synchronizer attachment immediately sends ID 18 and
then ID 19 once for every data property in ascending index order; only afterward does the server
make the new menu its current menu. A menu type with no registered client screen only warns and
leaves the current client menu/screen unchanged.

The menu state ID starts at zero. Every full-content send and every individual slot send increments
it as `(old + 1) & 32767` before construction. A full snapshot is one ID 18 containing all current
slots and cursor, followed by all data properties as ID 19. Delta broadcast scans slots in ascending
index order and sends each changed authoritative slot separately as ID 20, incrementing for each;
it then compares/sends cursor ID 96 and finally data ID 19 in ascending index order. Cursor and data
packets neither carry nor increment state ID.

The client stores the received ID 18/20 state ID verbatim; it does not require monotonicity or a
one-step increment. Container ID zero targets the persistent inventory menu. Other content/slot/data
packets act only when their ID exactly equals the current menu ID. Wrong IDs are ignored, except
the creative-screen bookkeeping quirk below. A serverbound click echoes the client's current menu
state ID; server behavior for matching/stale IDs and hashed predictions is specified in
[serverbound play](play-serverbound.md).

Primary ordering anchors are `ServerPlayer#openMenu`, `#closeContainer`, `#initMenu`, its
`ContainerSynchronizer`, `AbstractContainerMenu#sendAllDataToRemote`, `#broadcastChanges`,
`#broadcastFullState`, and `#incrementStateId`.

## Client application and edge behavior

ID 59 constructs the selected menu with the decoded ID and player inventory, assigns it as current,
then opens the screen. It does not compare an old container ID or emit a close request. ID 17
ignores its packet container ID, closes whatever menu is current, and clears the GUI. Consequently
delayed close traffic can close a newer screen on both peers; the ID is descriptive rather than a
guard.

ID 18 with container zero always initializes the inventory menu; any other ID initializes only an
exact current match. Initialization writes one slot for every transmitted list element, then cursor,
then state ID. A shorter list leaves trailing slots unchanged. A longer list faults at the first
nonexistent slot, before cursor/state installation. ID 19 exact-matches the current menu and indexes
its data list with the widened signed short; an invalid index faults.

ID 20 always invokes the tutorial item hook first. Container zero targets the inventory menu; a
nonempty hotbar update whose old stack is empty or has a smaller count receives pop time five before
the slot/state write. A nonzero exact current ID writes that menu; other IDs are ignored. Invalid
slot indices fault. If any creative inventory screen is visible, the client then unconditionally
forces the inventory menu's remote slot at the packet slot and broadcasts local changes, even when
the packet ID was otherwise ignored; that remote-slot index can itself fault.

ID 96 also invokes the tutorial hook. It replaces the current menu cursor unless a creative
inventory screen is visible, in which case the mutation is ignored. It has no container/state ID.
ID 108 invokes the tutorial hook then calls player-inventory `setItem`. Negative slots pass the
method's `slot < 36` test and fault list indexing. Slots `0..=35` replace ordinary inventory;
`36..=39` map to feet, legs, chest and head through equipment indices; `40` maps
offhand, `41` body and `42` saddle. Values above 42 are ignored. The same item can therefore be
installed in exactly one ordinary-list or equipment destination for every nonnegative mapped slot.

Malformed/truncated fields, residual bytes, overlong VarInts, negative/impossible list allocation,
invalid item/component/menu IDs, invalid trusted component NBT and truncated stacks fail decoding.
Invalid matched slot/property indices and overlong content lists fault during client application.
Wrong menu IDs, missing screen constructors, creative cursor suppression and the exact close rules
are semantic ignore paths. Ferrite projects normalized menu state through these version-local IDs,
registries, stack/component encodings and state counters; none is persisted as authoritative ECS or
world-storage identity.

# C3 Player Vitals, Cooldowns, and Statistics Projection

These four packets project local-player survival state or the response to the already specified
serverbound statistics request. They have no entity/container ID and apply only to the receiving
connection's local player.

| ID | Identity | Fields in exact order |
|---:|---|---|
| `3` | `minecraft:award_stats` | stat/value map |
| `22` | `minecraft:cooldown` | cooldown-group identifier; duration ticks VarInt |
| `103` | `minecraft:set_experience` | progress float; level VarInt; total experience VarInt |
| `104` | `minecraft:set_health` | health float; food VarInt; saturation float |

The stat map begins with a VarInt count. Every entry is a stat-type registry raw ID VarInt, then a
raw ID from that type's backing registry, then a signed value VarInt. The stat type and custom-stat
registries throw on unknown raw IDs; the defaulted block/item/entity-type backings map unknown IDs
to air/air/pig respectively. Its generic map codec has the signed-int maximum rather than a smaller
packet bound; it caps only the initial allocation hint at 65,536. A negative count reaches the
negative-capacity constructor and faults.
Repeated stat keys replace earlier decoded values before the handler sees the map. Stat types and
their backing registries are locked in [registry mappings](registry-and-metadata-mappings.md).

The cooldown group uses `UTF(32767)` followed by strict namespaced-identifier parsing. Duration is
an unrestricted signed VarInt. Both floats in the two vitals packets and the experience-progress
float preserve their raw IEEE-754 bits. The codec does not range-check health, food, saturation,
progress, level, total experience, or duration. Note that the ID-103 constructor argument order is
`progress,total,level`, but its wire order is deliberately `progress,level,total`.

Primary codec anchors are `ClientboundAwardStatsPacket#STAT_VALUES_STREAM_CODEC`,
`Stat#STREAM_CODEC`, `StatType#streamCodec`, `ClientboundCooldownPacket#STREAM_CODEC`,
`ClientboundSetExperiencePacket#read/write`, and `ClientboundSetHealthPacket#read/write`.

## Health and food publication and application

On the ordinary `doTick` branch, `Player.tick` (including cooldown expiry), food simulation and
per-tick statistic awards precede the special-item scan and vitals comparison. A spectator touching
an unloaded chunk skips that early branch but rejoins at the special-item scan and still reaches
vitals. The server sends ID 104 when any of these comparisons differs from its last sent markers:

1. current health differs by Java float comparison;
2. current food level differs exactly;
3. the predicate `saturation == 0.0F` differs.

It includes current health, food and the complete saturation value, then records health, food and
only that zero/nonzero predicate. Consequently, a positive saturation change alone sends nothing;
the next health/food/zero-edge send carries its latest value. `-0.0F` counts as zero. A NaN health
compares unequal even to itself and therefore sends every tick while it remains NaN. Fresh players
start with impossible sentinels, and cross-dimension/respawn paths reset health/food sent markers,
so the following player tick projects a complete current tuple.

The client routes health through `LocalPlayer#hurtTo`. The first received value sets health and
arms later flash behavior without a hurt flash. Thereafter a decrease records the lost amount,
sets invulnerability to 20 ticks, and sets hurt duration/time to 10; an increase sets
invulnerability to 10; an equal value only sets health. `LivingEntity#setHealth` clamps finite
values to `[0,max_health]`, maps negative infinity and NaN to zero, maps positive infinity to
maximum health, and preserves negative zero. Food and saturation are then assigned directly with
no range clamp. On a later raw NaN packet, `hurtTo`'s comparison also takes its nondamage branch,
sets health to zero and sets invulnerability to 10 rather than producing the ordinary hurt flash.
This packet is independent of living-entity health metadata and damage/hurt/death packets.

Primary anchors are `ServerPlayer#doTick`, its `lastSentHealth`, `lastSentFood`, and
`lastFoodSaturationZero` fields, `ClientPacketListener#handleSetHealth`,
`LocalPlayer#hurtTo`, `LivingEntity#setHealth`, and `FoodData#setFoodLevel/#setSaturation`.

## Experience publication and application

The ordinary server-player tick sends ID 103 after health and the health/food/air/armor/experience
score criteria when `totalExperience != lastSentExp`; it records total before sending current
progress, total and level. Canonical mutations that can change only progress or level explicitly
set `lastSentExp=-1`; with canonical nonnegative totals this forces their next tick even when total
is unchanged. An exceptional authoritative total of exactly `-1` collides with that marker and
suppresses the forced send until the total or marker differs. Fresh and relocated players use
sentinels for the same ordinary projection rule. Respawn additionally sends an explicit experience
packet after the new position challenge and difficulty and before active effects/level information.
That explicit send does not update `lastSentExp`; for a canonical total other than `-1`, the first
ordinary tick of the respawned player therefore sends the same current experience tuple again.

The client assigns progress, total and level directly. Progress inequality by Java float comparison
resets the XP display-start tick, so repeated NaN progress resets it every packet; total and level
changes alone do not. There is no clamping, monotonicity check, derived consistency check, request,
or acknowledgement. Signed and non-finite codec values remain client projection state until a
later packet replaces them.

Primary anchors are `ServerPlayer#doTick`, `#setExperiencePoints`, `#setExperienceLevels`,
`#giveExperiencePoints`, `#giveExperienceLevels`, `PlayerList#respawn`,
`ClientPacketListener#handleSetExperience`, and `LocalPlayer#setExperienceValues`.

## Cooldown replacement and expiry

Server and client key cooldowns by a namespaced group, not by item raw ID. For an item stack the
group is its `use_cooldown` component's optional group when present, otherwise the stack item's
registry identifier. Starting or replacing a server cooldown stores
`(start=tickCount,end=tickCount+duration)` with signed-int wrapping and immediately sends ID 22
with that duration. Explicit removal sends duration zero even when no entry existed. Natural expiry
removes an entry when `endTime <= tickCount` after incrementing the counter and sends zero.

The client interprets exactly zero as removal. Every nonzero value, including a negative one,
replaces the group with its own wrapped start/end interval. Its ordinary cooldown tick then applies
the same expiry rule; a negative duration is therefore normally visible only until that next tick.
Cooldown percentage is the clamped quotient `(end-(tick+partial))/(end-start)`. A packet has no
item list, server tick counter, start time, acknowledgement, or protected generation: a delayed
zero can remove a newer same-group client cooldown.

Primary anchors are `ItemCooldowns#getCooldownGroup`, `#addCooldown`, `#removeCooldown`, `#tick`,
`#getCooldownPercent`, `ServerItemCooldowns#onCooldownStarted/#onCooldownEnded`, and
`ClientPacketListener#handleItemCooldown`.

## Statistics drain and client replacement

Every server statistic assignment stores its signed value and marks that exact typed stat dirty.
Increment computes `(int)min((long)current + delta, 2_147_483_647)`: positive overflow saturates,
while values below signed-int minimum wrap when narrowed back to int. Joining calls `markAllDirty`,
but dirtiness by itself sends no packet. Serverbound
`client_command(request_stats)` resets idle, copies the current dirty set and values into one map,
clears the set, and sends ID 3 even when the map is empty. A second request before another change
therefore receives an empty map. There is no request token or unsolicited periodic stat packet.

The client replaces each decoded stat value in its local `StatsCounter`; omitted stats are
unchanged. After all entries, an open `StatsScreen` receives one `onStatsUpdated` callback even for
an empty map. Map/hash iteration provides no semantic ordering guarantee, and a packet is a delta
over the drained dirty set rather than a complete snapshot of stored counter entries unless
`markAllDirty` preceded it; absent/unmaterialized zero-valued stat keys are not synthesized.

Primary anchors are `StatsCounter#increment/#setValue`, `ServerStatsCounter#setValue`,
`#markAllDirty`, `#getDirty`, `#sendStats`,
`ServerGamePacketListenerImpl#handleClientCommand`,
`ClientPacketListener#handleAwardStats`, and `StatsScreen#onStatsUpdated`.

## Failure, ordering, and Ferrite boundary

Malformed/truncated fields, overlong VarInts, residual bytes, invalid identifiers, invalid stat
type/custom-stat raw IDs and impossible map allocation enter normal packet failure. Invalid
block/item/entity-type backing IDs instead use their documented defaults. Semantic numeric values
otherwise follow the application rules above. The packets carry no response, state ID or
cross-family acknowledgement; only the serverbound stats request has the one-response drain
relationship described above.

Ferrite projects normalized player vitals, experience, cooldown groups and typed namespaced stats
through this connection-local adapter. Packet IDs, raw registry IDs, sent/dirty markers, cooldown
tick intervals, client hurt timers, XP display timers and screen callbacks remain version-local and
must not become authoritative ECS or persistence identities.

# C3 Mount, Book, and Sign-Editor Activation

These packets activate three screens whose data authority lives elsewhere. Mount inventory state
continues through the already specified container family, book contents come from the current hand
stack and its synchronized data components, and sign text comes from the current client block
entity before returning through serverbound `sign_update`.

| ID | Identity | Fields in exact order |
|---:|---|---|
| `41` | `minecraft:mount_screen_open` | container ID signed VarInt; inventory columns signed VarInt; entity ID signed big-endian int |
| `58` | `minecraft:open_book` | interaction-hand ordinal VarInt |
| `60` | `minecraft:open_sign_editor` | packed block-position signed long; front-text boolean |

The hand ordinal is strict indexed-enum decode: `0=main_hand`, `1=off_hand`; every other signed
VarInt faults. Container ID and inventory columns have no codec range check. Packed block position
uses the common signed 26-bit X, signed 26-bit Z and signed 12-bit Y layout. Booleans use the common
zero-false/nonzero-true rule.

Primary codec anchors are `ClientboundMountScreenOpenPacket#STREAM_CODEC`,
`ClientboundOpenBookPacket#STREAM_CODEC`, `FriendlyByteBuf#readEnum`, and
`ClientboundOpenSignEditorPacket#STREAM_CODEC`.

## Mount menu activation and convergence

The client first resolves the fixed-width entity ID, reads the column count and allocates an empty
`SimpleContainer` of Java-int size `columns * 3`, including signed multiplication wrap, before it
tests the entity type. A negative result faults allocation; a sufficiently large positive result
can resource-fault. This occurs even when the entity is missing or has the wrong type. After a
successful allocation:

- an `AbstractHorse` installs `HorseInventoryMenu`, assigns it as the local player's current menu,
  and opens `HorseInventoryScreen`;
- an `AbstractNautilus` analogously installs `NautilusInventoryMenu` and opens its screen;
- any other or missing entity leaves the current menu and screen unchanged.

Both menus contain saddle and body-equipment slots plus the standard player inventory. A horse menu
adds exactly three rows of `columns` cargo slots when columns is positive. A nautilus menu never
adds cargo slots from the column value even though the preceding empty-container allocation still
uses it. Canonical server values are safe: an ordinary horse and nautilus report zero columns, and
a chest-bearing llama reports its strength while a non-chested llama reports zero.

The canonical server opens a living tamed mount only. It first closes/removes a current noninventory
menu, advances the ordinary container counter in `1..=100`, sends ID 41 with the tracked entity ID
and current column count, selects the matching authoritative mount menu, and initializes its menu
synchronizer. The resulting complete content and later close/click/slot/data traffic use the
already specified container convergence rules. The entity must therefore already exist in the
client tracker before ID 41, and ID 41 is a specialized replacement for `open_screen`, not a second
container generation or acknowledgement.

Primary anchors are `ClientPacketListener#handleMountScreenOpen`,
`AbstractMountInventoryMenu#getInventorySize`, `HorseInventoryMenu`, `NautilusInventoryMenu`,
`ServerPlayer#openHorseInventory`, and `ServerPlayer#openNautilusInventory`.

## Book view activation

On ID 58 the client reads the stack currently in the decoded hand at handler execution time; the
packet carries no item, contents, slot revision or snapshot. `BookAccess#fromItem` first accepts a
`written_book_content` component, selecting its filtered or raw pages according to local client
filtering, and otherwise accepts `writable_book_content` pages. A recognized component opens a
`BookViewScreen`; no recognized component silently leaves the current screen unchanged. Both forms
are view-only here. Editing and serverbound `edit_book` are source-specified separately in
`play-serverbound.md`. A delayed ID 58 can consequently display a different current book or no book
after the hand stack changes.

The canonical `ServerPlayer#openItemGui` path sends ID 58 only when the stack passed to it has
`written_book_content`. It first resolves that component against the player's command/registry
context; when resolution mutates the stack it broadcasts ordinary menu changes before ID 58. The
screen closes locally and sends no response or acknowledgement.

Primary anchors are `ClientPacketListener#handleOpenBook`, `BookViewScreen.BookAccess#fromItem`,
`WrittenBookContent`, `WritableBookContent`, and `ServerPlayer#openItemGui`.

## Sign editor activation

The client resolves the current block entity at the packed position. A `SignBlockEntity` opens the
ordinary `SignEditScreen`, or `HangingSignEditScreen` for a hanging-sign entity, using the selected
front/back side and local text-filtering option. A missing or wrong block entity logs and ignores
the packet. ID 60 carries no sign type, text, edit token or player identity, so the correct block
entity projection must already exist.

Canonical sign interaction executes click commands first and does not open an editor when commands
consume the interaction or the sign is waxed. Opening additionally requires no different active
editor, build permission, and four editable selected-side messages—each empty or plain text. The
server stores the player's UUID as the sign's sole allowed editor, then `ServerPlayer#openTextEdit`
sends the current ID-8 `block_update` before ID 60. It does not send block-entity data at this point.
The ID-8 correction therefore orders current block state before activation, while earlier chunk or
block-entity projection supplies the sign text.

The editor copies exactly four selected-side plain strings, limited locally by rendered line width
rather than the packet's character bound. Done, Escape, screen replacement, sign removal, a missing
local player, or the client distance check all close the screen; its single `removed()` callback
sends one serverbound ID 61 containing the current four strings when a connection exists. That
submission transaction and its server authorization are specified in the serverbound page.

Primary anchors are `SignBlock#useWithoutItem`, `ServerPlayer#openTextEdit`,
`ClientPacketListener#handleOpenSignEditor`, `LocalPlayer#openTextEdit`, and
`AbstractSignEditScreen#tick/#onClose/#removed`.

## Special-screen failure and Ferrite boundary

Malformed/truncated fields, invalid hand ordinals, overlong VarInts and residual bytes fault normal
packet decode. Signed mount values otherwise reach the allocation/application behavior above;
missing mount entities, wrong entity types, absent book components and absent/wrong sign entities
are semantic ignore paths. None of the three packets creates a general screen acknowledgement.

Ferrite projects normalized mount-menu authority, resolved book components and sign-edit authority
through this version-local adapter. Raw packet/entity/container IDs, column arithmetic, hand
ordinals, GUI objects, current-hand timing and allowed-editor UUID bookkeeping do not become
authoritative ECS or persistence identities.

# C3 Recipe-Book Ghost and Removal Projection

This slice completes the recipe-book deltas left beside the C1 initial settings, add and recipe-data
packets. Recipe display IDs are not registry IDs and are not namespaced recipe identities: they are
signed integer positions in the server's current feature-filtered display list. The ghost packet
instead carries a complete display payload.

| ID | Identity | Fields in exact order |
|---:|---|---|
| `63` | `minecraft:place_ghost_recipe` | container ID signed VarInt; registry-aware `RecipeDisplay` |
| `75` | `minecraft:recipe_book_remove` | generic list count signed VarInt; that many recipe-display IDs as signed VarInts |

The ghost display begins with the locked recipe-display-type raw VarInt and dispatches to its
registered stream payload. The five raw IDs and every nested slot-display, item, component, trim
and potion mapping are the same mappings specified for clientbound ID 74 under Recipe projection
grammars. An unknown type or nested strict registry ID faults. ID 75's generic list codec has no
smaller packet-specific maximum than signed-int maximum; negative/impossible allocation, malformed
VarInts, truncation and residual bytes still fault normal packet handling. Every display ID,
including a negative one, is otherwise accepted by the codec as a signed VarInt.

Primary codec anchors are `ClientboundPlaceGhostRecipePacket#STREAM_CODEC`,
`RecipeDisplay#STREAM_CODEC`, `ClientboundRecipeBookRemovePacket#STREAM_CODEC`,
`RecipeDisplayId#STREAM_CODEC`, and `ByteBufCodecs#list`.

## Session-local display identity

On recipe reload, `RecipeManager#unpackRecipeInfo` walks the loaded recipes and each recipe's
ordered `display()` list. It skips a display whose required features are disabled and assigns every
retained display the next contiguous integer starting at zero. One namespaced parent recipe may
therefore own multiple display IDs. Each retained entry stores the full display, optional
session-local integer group, category and optional placement ingredients; a special recipe has no
placement-ingredient projection. A nonempty recipe group string receives the next local group
integer on first encounter, while an empty group is absent.

The same index maps server-side to both that display entry and its namespaced parent
`RecipeHolder`. Negative IDs and IDs at or beyond the current display-list size resolve to no
entry. A data-pack or feature-set reload rebuilds the list, so neither display nor group integers
are durable identity. Ferrite must retain normalized parent recipe keys and display semantics and
must never persist or compare these integers across recipe-manager generations or sessions.

Primary anchors are `RecipeManager#unpackRecipeInfo/#getRecipeFromDisplay`,
`RecipeDisplayEntry`, `RecipeManager.ServerDisplayInfo`, and `RecipeHolder#id`.

## Ghost display application

The client first requires the packet container ID to equal the local player's exact current menu
ID. It then requires the current screen to implement `RecipeUpdateListener`. Either failure is a
silent no-op. Success calls `fillGhostRecipe` with the supplied display; no display ID or locally
known recipe-book entry is required because ID 63 carries the full display.

The recipe-book component clears its previous ghost slots, builds a `SlotDisplayContext` from the
current client level, and lets the decoded display populate the display-specific crafting/furnace/
stonecutter/smithing ghost arrangement. This changes GUI guidance only: it does not install item
stacks, mutate the authoritative menu, change recipe knowledge or send a response. A delayed packet
can target a later menu that has reused the same unprotected container ID.

The canonical server emits ID 63 only from admitted serverbound `place_recipe` when the resolved
recipe cannot be crafted from the combined inventory and current menu inputs. That branch first
returns the menu inputs to inventory and clears the crafting content, then sends the current menu
ID and the resolved display payload immediately. Ordinary container broadcasts occur separately
and may follow the ghost packet; ID 63 has no state ID or acknowledgement role.

Primary anchors are `ClientPacketListener#handlePlaceRecipe`,
`RecipeUpdateListener#fillGhostRecipe`, `RecipeBookComponent#fillGhostRecipe`,
`GhostSlots`, and `ServerGamePacketListenerImpl#handlePlaceRecipe`.

## Removal and refresh

For every ID in wire order, the client recipe book removes the matching known display entry and
removes that exact display ID from its highlight set. Missing IDs and duplicates after their first
effective removal are no-ops. After the complete list—including an empty list—the handler performs
one refresh: it rebuilds recipe collections, updates the session recipe search tree from the
current book and level, and calls `recipesUpdated()` once when the current screen implements
`RecipeUpdateListener`. Entries omitted from the packet remain unchanged.

The server's authoritative recipe book keys known and highlighted state by namespaced parent
recipe. Removing a known parent removes it from both sets, resolves all of that parent's current
display entries, and sends ID 75 only when at least one display ID was collected. The list can
therefore remove multiple client display entries for one parent. The server method's return count
is the number of collected display IDs, not necessarily the number of parent recipes. There is no
client acknowledgement or protected recipe-book generation.

Primary anchors are `ClientPacketListener#handleRecipeBookRemove/#refreshRecipeBook`,
`ClientRecipeBook#remove/#removeHighlight`, and `ServerRecipeBook#removeRecipes`.

## Failure, ordering, and Ferrite boundary

Malformed/truncated data, overlong VarInts, negative/impossible list allocation, unknown strict
display or nested registry mappings and residual bytes fault packet handling. Signed container and
display IDs otherwise reach the semantic current-menu/no-entry branches. A valid but absent screen,
stale container, missing removal entry or repeated removal is not a decode fault.

Initial publication sends recipe-book settings before ID 74 with `replace=true`; later additions
use ID 74 with `replace=false`; parent removal uses ID 75; placement failure uses ID 63. Those flows
share session-local display IDs but no sequence or acknowledgement token. Ferrite projects
normalized recipe knowledge, settings and displays through this adapter. Raw display/type/item/
component IDs, container IDs, client highlights, search collections and ghost slots never become
authoritative ECS or persistence state.

# C3 Merchant Offer Projection

Clientbound ID `52`, `minecraft:merchant_offers`, replaces the current merchant menu's complete
client-side offer/HUD projection. Its exact outer field order is:

| Field | Wire form |
|---|---|
| container ID | signed VarInt |
| offers | generic signed-VarInt count, followed by that many `MerchantOffer` values |
| villager level | signed VarInt |
| villager XP | signed VarInt |
| show progress | boolean |
| can restock | boolean |

The collection codec has no smaller merchant-specific count limit than signed-int maximum; its
allocation is only pre-sized up to `65,536`. Negative/impossible allocation, malformed/truncated
VarInts and residual bytes fault normal handling. Level and XP have no codec range check. Each
boolean uses zero-false/nonzero-true.

Primary outer codec anchors are `ClientboundMerchantOffersPacket#STREAM_CODEC`,
`MerchantOffers#STREAM_CODEC`, and `ByteBufCodecs#collection`.

## Offer and cost wire grammar

Each offer contains, in exact order:

1. base `ItemCost` A;
2. a nonempty result `ItemStack`;
3. optional `ItemCost` B as presence boolean then value;
4. out-of-stock boolean;
5. uses as signed big-endian int;
6. maximum uses as signed big-endian int;
7. offer XP as signed big-endian int;
8. special-price difference as signed big-endian int;
9. price multiplier as raw IEEE-754 big-endian float;
10. demand as signed big-endian int.

An `ItemCost` contains a strict item-registry raw VarInt, signed count VarInt, then a generic
VarInt-counted exact-component predicate. Each predicate entry contains a strict data-component-
type raw VarInt followed by that registered type's stream payload. The locked mappings are the same
1,537-item and 111-component maps specified for container stacks. Unknown item/component IDs,
malformed dispatched payloads and impossible predicate allocation fault. The cost count itself has
no positive stream constraint. A local presentation stack reconstructed from item, count and the
predicate patch is not an additional wire field.

The result uses `ItemStack.STREAM_CODEC`, not the optional stack codec: empty results are rejected
by the member encoder/decoder contract. Positive counts and their component patch otherwise follow
the already specified trusted item-stack grammar. Optional cost B uses an ordinary boolean; any
nonzero presence byte requires the complete cost.

Primary nested anchors are `MerchantOffer#STREAM_CODEC`, `ItemCost#STREAM_CODEC`,
`DataComponentExactPredicate#STREAM_CODEC`, `TypedDataComponent#STREAM_CODEC`, and
`ItemStack#STREAM_CODEC`.

## Decode normalization and copied snapshot

Network decode constructs every offer with `rewardExp=true`; reward-experience is not carried on
the wire. It initially uses the wire uses, maximum, XP, multiplier and demand and a zero special
price. If the wire out-of-stock boolean is true, decode calls `setToOutOfStock`, replacing uses with
maximum uses and thereby discarding a different wire uses value. It then installs the wire special-
price difference.

A false out-of-stock byte does not force an in-stock state: the wire uses value is retained, and
the derived predicate is still out of stock whenever `uses >= maxUses`. Encoding writes that
derived predicate, so a normally published packet makes the flag agree with its current counts.
All five signed integer values and the raw multiplier otherwise remain unclamped at codec time.

Packet construction copies the offer list and copies each result stack. The emitted packet is
therefore a snapshot insulated from later mutation of the source merchant offers/results. Client
application installs that decoded/copy-owned list rather than a shared server object.

Primary anchors are `MerchantOffer#createFromStream/#setToOutOfStock/#isOutOfStock`,
`MerchantOffers#copy`, and `ClientboundMerchantOffersPacket`'s public copy constructor.

## Exact predicates and modified first cost

An `ItemCost` matches only the named item holder and requires equality for every component listed
in its exact predicate. Components present on the candidate stack but absent from the predicate are
allowed. Predicate entries retain list order and duplicates; incompatible expectations for a
duplicated type can therefore make a cost impossible to satisfy. Required count is checked
separately after predicate matching.

Cost A's displayed/payable count is calculated with locked Java arithmetic:

```text
product       = Java signed-int wrapping(base_count * demand)
demand_delta  = max(0, floor((float) product * price_multiplier))
modified      = clamp(Java signed-int wrapping(base_count + demand_delta
                                                + special_price_difference),
                      1, base-cost item stack maximum size)
```

The cast, float multiplication, `Mth.floor` and clamp retain Java behavior for NaN, infinities,
negative values and overflow; the codec does not pre-sanitize them. Cost B, when present, uses its
base signed count without demand or special-price modification. Satisfaction requires cost A with
at least the modified count, plus cost B with at least its base count when present; absent cost B
requires the second payment input to be empty. Assembly returns a copy of the result stack.

Primary anchors are `MerchantOffer#getModifiedCostCount/#getCostA/#satisfiedBy/#assemble`,
`ItemCost#test`, and `Mth#floor/#clamp`.

## Client application and presentation

The main-thread handler first requires exact equality between the packet container ID and the local
player's current menu ID, then requires that menu to be a `MerchantMenu`. Either failure silently
ignores the whole packet. No merchant screen need be open. Success replaces the
`ClientSideMerchant` offer list and then sets XP, merchant level, show-progress and can-restock in
that order. There is no merge, monotonic/version check, acknowledgement or menu-generation token;
a delayed packet can affect a later merchant menu that reuses the same container ID.

The merchant screen uses show-progress to gate its level/XP bar. Only levels `1..5` receive a tier
title, and the bar has advancement thresholds only below level five; arbitrary signed levels/XP
remain stored without handler clamping. Can-restock changes the out-of-stock tooltip rather than
making an offer usable. Offer use/result consumption is predicted by the merchant result-slot path
and converged by ordinary authoritative container traffic; selection alone is specified in the
serverbound merchant section.

Primary anchors are `ClientPacketListener#handleMerchantOffers`,
`MerchantMenu#setOffers/#setXp/#setMerchantLevel/#setShowProgressBar/#setCanRestock`,
`ClientSideMerchant#overrideOffers/#overrideXp`, and
`MerchantScreen#extractLabels/#extractProgressBar/#extractContents`.

## Publication order and Ferrite boundary

Canonical `Merchant#openTradingScreen` first opens an ordinary merchant menu. The generic open path
closes/removes any previous menu when needed, sends ID 59 `open_screen`, sends initial ID 18 full
content and ID 19 properties through menu initialization, then selects the new server current menu.
Only after that return does merchant opening send ID 52, and it sends no ID 52 when the authoritative
offer list is empty. The client thus has the matching merchant menu before canonical offer
replacement. A forged/reordered ID 52 follows the handler gates above.

Offer projection has no acknowledgement. The later local-first ID 51 selection path predicts
payment/result changes and ordinary container state convergence corrects them; completing a trade
likewise does not make ID 52 a per-click response. Ferrite projects normalized copied offer costs,
results, uses/limits, XP/level and flags through this adapter. Packet/container IDs, strict registry
numbers, exact-predicate encoding, wire offer order, computed presentation counts and GUI state do
not enter authoritative ECS or persistence identities.

Primary publication anchors are `Merchant#openTradingScreen`, `ServerPlayer#openMenu/#initMenu`,
`ServerPlayer#sendMerchantOffers`, and `AbstractContainerMenu#sendAllDataToRemote`.

# C3 Map, Tag-Query and Advancement Projection

These three clientbound packets project independent inventory/progression-adjacent views. A map ID
is the numeric member of a durable map saved-data key, a tag-query transaction is an ephemeral
debug callback token, and an advancement identifier is namespaced content identity. They are not
interchangeable integer namespaces and none is an ECS entity ID.

| ID | Identity | Fields in exact order |
|---:|---|---|
| `51` | `minecraft:map_item_data` | map ID signed VarInt; scale signed byte; locked boolean; optional decoration list; color-patch sentinel/payload |
| `123` | `minecraft:tag_query` | transaction signed VarInt; raw nullable compound NBT |
| `130` | `minecraft:update_advancements` | reset boolean; added advancement list; removed identifier set; progress map; show-advancements boolean |

Every outer boolean uses zero-false/nonzero-true. Normal packet dispatch moves all three handlers to
the client main thread. Malformed/truncated fields, overlong VarInts and residual body bytes fault
before semantic handling.

## Map wire grammar and identity

The optional decoration field is a presence boolean followed, when present, by a generic signed-
VarInt list count and that many decorations. Each decoration contains a strict configured
`minecraft:map_decoration_type` holder raw VarInt, signed X byte, signed Y byte, signed rotation
byte and optional trusted component name. The constructor retains X/Y but masks rotation with
`0x0f`. The list codec has no family-specific maximum below signed-int/transport feasibility;
negative or impossible allocation, unknown decoration raw IDs, malformed trusted components and
truncation fault.

The color patch is not a normal optional boolean. Its first unsigned byte is width: zero means no
patch and ends the field. A positive width is followed by unsigned height, start X and start Y
bytes, then a signed-VarInt byte-array length and exactly that many color bytes. Decode admits width
and coordinates through `255`, height zero, and any transport-feasible color-array length. It does
not require `colors.length == width * height` or prove that the rectangle fits the 128-by-128 map.

The signed map VarInt constructs `MapId`; it does not index a registry. Canonical map creation
allocates IDs in level saved data and a filled-map stack carries that durable key in its map-ID
component. Ferrite must normalize that key and map content, never persist packet order, decoration
raw IDs or a client texture handle as identity. The locked catalog classifies
`minecraft:filled_map` as the special `cartography-map-items` family.

Primary codec anchors are `ClientboundMapItemDataPacket#STREAM_CODEC`, `MapId#STREAM_CODEC`,
`MapDecoration#STREAM_CODEC`, `MapDecorationType#STREAM_CODEC`, and
`MapItemSavedData.MapPatch#STREAM_CODEC`.

## Map client application and malformed patches

The handler looks up the map ID in the current `ClientLevel`. If absent, it creates client map data
with packet scale/locked, center zero/zero, the current level dimension, tracking false and
unlimited-tracking false, then stores it under that ID. An existing map keeps its original scale,
locked flag and dimension: later ID-51 values do not replace them.

When decorations are present, the client clears the complete prior decoration map and inserts the
decoded list in order under transient keys `icon-0`, `icon-1`, and so on, recomputing the tracked-
decoration count. An absent decoration field retains the previous set; a present empty list clears
it. The handler applies the patch next, then asks `MapTextureManager` to refresh this map.

Patch application loops local X outside local Y and reads color index `x + y * width`; it writes
the flat global index `(startX + x) + (startY + y) * 128`. There is no two-dimensional bounds check,
preflight validation or rollback. An X beyond 127 can therefore alias a later row while that flat
index remains in the array. A short array or eventually oversized flat index can mutate an exact
traversal prefix and then throw an array-bounds fault; an overlong array has ignored suffix bytes;
zero height performs no writes. Decorations were already replaced before such a patch fault, and
texture refresh is not reached. A compatible implementation must reject or reproduce malformed
input at the same boundary rather than treating patch application as an atomic validated
rectangle.

Primary handler anchors are `ClientPacketListener#handleMapItemData`,
`MapItemSavedData#createForClient/#addClientSideDecorations/#setColor`,
`MapItemSavedData.MapPatch#applyToMap`, and `MapTextureManager#update`.

## Canonical map publication

Each server map keeps a per-player holding record. A publication opportunity consumes the complete
current dirty-pixel bounding box immediately into a tightly packed patch, with source/destination
index `x + y * width`. While decorations are dirty, each opportunity evaluates Java
`tick++ % 5 == 0`: it tests the old counter, then increments it. A new holder starts at zero, so its
first dirty opportunity includes the full current decoration collection and clears the dirty flag;
if dirtied again, the next inclusion follows after five dirty opportunities. The counter does not
advance while decorations are clean. ID 51 is omitted when neither a patch nor sampled decorations
exists. Pixel dirtiness and decoration cadence are independent, and a pixel-only packet never
waits for decoration cadence.

The map item registers holders and updates its saved pixels through ordinary inventory ticks; map
locking and held/equipped conditions govern those owned gameplay mutations. `getUpdatePacket`
selects only the requesting player's holding record. ID 51 has no sequence, acknowledgement or
generation token: later projections overwrite decoration state when present and patch pixels in
arrival order.

Primary server anchors are `MapItemSavedData.HoldingPlayer#nextUpdatePacket`,
`MapItemSavedData#createPatch/#getHoldingPlayer`, and `MapItem#getUpdatePacket/#inventoryTick`.

## Tag-query codec and latest-callback correlation

ID 123 writes raw network NBT after its signed transaction. The root type `END` decodes as null;
every non-END root must be a compound or decoding faults. Parsing uses the default 2,097,152-byte
NBT accounting quota and maximum nesting depth 512. These are structured-NBT limits in addition to
the packet transport boundary. This packet is marked skippable by the connection error policy, but
that does not make a malformed body semantically valid.

The client debug-query handler owns exactly one pending callback and one wrapping signed-int
counter initialized to `-1`. Starting an entity or block query replaces the callback, increments
the counter with Java signed wrap, and sends the new value; the initial request therefore uses
transaction zero. A response invokes and then clears the callback only when its transaction
exactly equals the current pending transaction and the callback is nonnull. The clear occurs after
the callback returns, so a throwing callback remains installed. Stale, duplicate and unmatched
responses are logged/ignored. There is no queue and the tag may be null.

Canonical serverbound entity/block query admission is C4-owned, but its locked response shape is
relevant here: both requests require game-master command permission and otherwise receive no
response. A permitted block query always echoes the transaction, using block-entity data saved
without metadata or null when no block entity exists. A permitted entity query sends only when the
entity currently exists and serializes it without its ID. Permission denial and missing entity are
thus timeout/no-response branches rather than null responses; missing block entity is an explicit
null response. This debug NBT is callback data, not an authoritative world mutation or persistence
format.

Primary anchors are `ClientboundTagQueryPacket`, `FriendlyByteBuf#readNbt`, `NbtAccounter`,
`DebugQueryHandler#startTransaction/#handleResponse/#queryEntityTag/#queryBlockEntityTag`, and
`ServerGamePacketListenerImpl#handleEntityTagQuery/#handleBlockEntityTagQuery`.

## Advancement wire grammar

ID 130's added list, removed set and progress map each begin with a signed VarInt count. They have
no packet-specific count ceiling below signed-int/transport feasibility. The added list retains
wire order and duplicates. Removed values decode into a hash set, collapsing duplicates and
discarding semantic wire order. Progress decodes into a hash map from identifiers to values, so a
later duplicate key replaces its earlier decoded value. Negative/impossible allocations,
malformed identifiers and truncated members fault.

Each added holder is an identifier followed by this reduced network `Advancement`:

1. optional parent identifier;
2. optional `DisplayInfo`;
3. requirements as a generic outer list of generic lists of default UTF strings; and
4. sends-telemetry-event boolean.

Rewards and trigger/criterion definitions are absent from the wire; decode fixes rewards empty and
criterion definitions to an empty map. Requirements alone name the progress criteria. A
`DisplayInfo` contains trusted title component, trusted description component, registry-aware
`ItemStackTemplate` icon, strict enum ordinal (`0=task`, `1=challenge`, `2=goal`), big-endian flags
int, optional background identifier when flags bit 0 is set, then raw big-endian X and Y floats.
Flags bit 1 means show toast and bit 2 means hidden; higher bits are ignored. Announce-to-chat is
not transmitted and decodes false. Trusted component, item-template and nested item/component
rules remain those of their shared locked codecs.

Each progress value is a generic map from default UTF criterion name to `CriterionProgress`.
Criterion progress begins with a nullable boolean; present is a signed big-endian long interpreted
as epoch milliseconds. The packet does not carry the advancement requirements again inside the
progress value. Empty and duplicate requirement strings and raw past/future timestamp values pass
this stream grammar; semantic normalization happens during client application.

Primary codec anchors are `ClientboundUpdateAdvancementsPacket`,
`AdvancementHolder#LIST_STREAM_CODEC`, `Advancement#STREAM_CODEC`, `DisplayInfo#STREAM_CODEC`,
`AdvancementRequirements`, `AdvancementProgress#fromNetwork`, and
`CriterionProgress#fromNetwork` plus `FriendlyByteBuf#readInstant`.

## Advancement tree and progress application

The main-thread client performs these steps in fixed order:

1. if reset is true, clear the tree and progress map;
2. remove every decoded ID, recursively removing all known descendants; unknown IDs only warn;
3. add holders, repeatedly inserting roots or entries whose parents are already present until a
   pass makes no progress, then log and discard the unresolved remainder; and
4. process every progress-map entry against the resulting tree.

An add does not validate requirement names against absent criterion definitions. A duplicate added
ID is not pre-removed: insertion replaces the ID lookup with a new node while the prior node can
remain in root/task/parent-child collections. Canonical publication avoids this malformed topology.
Removal does **not** delete entries from the separate client progress map; only reset clears that
map. Canonical re-visibility includes fresh progress, but a crafted remove/re-add without progress
can therefore expose an old ID-equal cached value to later listener initialization. Reset and
removal also do not directly clear the retained selected-tab field; advancement-screen/listener
logic resolves the resulting presentation separately.

For a known progress ID, the client derives the set of requirement names, removes wire criteria not
in that set, inserts every missing named criterion as not obtained, installs the received
requirements and stores the normalized value under the holder. Completion is false for an empty
outer requirements list; otherwise every outer group must contain at least one obtained member.
Unknown progress IDs warn and are ignored. Listener notification follows storage for every known
entry, including unchanged or incomplete values.

If reset is false and the resulting value is complete, the client reports advancement telemetry
when a level exists. It then adds a toast only when packet show-advancements is true and display is
present with show-toast true. There is no old-to-new transition test: repeated complete deltas can
repeat telemetry and toasts. Reset suppresses both for the initial snapshot. Packet
show-advancements gates the toast only, not telemetry; display hidden and announce-chat do not add
another client gate here.

Primary handler anchors are `ClientAdvancements#update`, `AdvancementTree#clear/#remove/#addAll`,
`AdvancementProgress#update/#isDone`, and `AdvancementRequirements#test`.

## Canonical advancement publication and visibility order

Server `PlayerAdvancements` maintains authoritative progress, currently visible holders, dirty
progress, dirty visibility and a first-packet flag. Flush first evaluates visibility changes,
collects newly visible full definitions and newly invisible IDs, and includes dirty progress only
for holders visible after that evaluation. Evaluation walks the complete root subtree postorder.
Each node's rule is HIDE when display is absent, SHOW when displayed and complete, HIDE when
displayed/incomplete/hidden, and NO_CHANGE otherwise. A node is visible immediately when itself or
any descendant is complete. Otherwise it inspects itself and at most two ancestors: the first SHOW
makes it visible, the first HIDE conceals it, and no decisive rule conceals it. Newly visible
holders are also marked for progress projection.

Flush sends ID 130 only when at least one added, removed or progress entry exists. Its reset flag is
the current first-packet value; the canonical player tick passes show-advancements true. The
first-packet flag is cleared after the flush attempt even when no packet was needed. A reset packet
therefore supplies the authoritative visible snapshot and suppresses initial client presentation;
later tokenless deltas apply in receive order. Advancement selection uses the separate ID 50/its
clientbound correction specified in the advancement-tab transaction and is not an acknowledgement
of ID 130.

Advancement data such as locked `minecraft:story/root` supplies normalized parent/display/
requirements/telemetry semantics. The packet is a reduced client projection and cannot replace the
authoritative data-pack definition, rewards, trigger state or save format. Ferrite retains
namespaced advancement identity and authoritative progress, not client tree nodes, timestamps as
ordering tokens, hash iteration, flags integers, toast/telemetry state or GUI selection.

Primary publication anchors are `PlayerAdvancements#flushDirty/#updateTreeVisibility`,
`AdvancementVisibilityEvaluator`, and `ServerPlayer#doTick`.

## C3 inventory-progression fault and order boundary

Map decoration and advancement configured-holder/raw registry failures, malformed trusted
components or item templates, invalid advancement enum ordinals, NBT quota/depth/root violations,
negative/impossible collection allocation, malformed identifiers/UTF, invalid nullable members,
truncation and trailing bytes fault their owning packet. The admitted map-patch rectangle hazards,
unknown/stale semantic identifiers and correlation misses follow the application branches above.

The families share no acknowledgement state. A map patch can interleave with ordinary filled-map
inventory movement; a tag response correlates only with the latest debug callback; an advancement
delta changes only its client tree/progress presentation. None confirms bundle, book, container,
recipe, chat, block, teleport or keepalive work. Ferrite must preserve each independent order while
normalizing durable map/advancement data at the version adapter boundary.

# C3 World-Border Delta Projection

The complete ID-43 border snapshot used at play entry and level transitions is specified in C1.
IDs 88 through 92 are later, dimension-scoped replacements for individual fields of that same
client border. Their exact observable geometry and warning consequences are owned by
[`WGEN-BORDER-001`](../mechanics/world/wgen-border-001.md); this section fixes their wire and
session boundary.

| ID | Identity | Fields in exact order |
|---:|---|---|
| `88` | `minecraft:set_border_center` | center X double; center Z double |
| `89` | `minecraft:set_border_lerp_size` | old size double; new size double; duration signed VarLong |
| `90` | `minecraft:set_border_size` | size double |
| `91` | `minecraft:set_border_warning_delay` | warning time signed VarInt |
| `92` | `minecraft:set_border_warning_distance` | warning blocks signed VarInt |

Doubles are raw big-endian IEEE-754 bits. The codecs add no finite, sign or gameplay-range checks;
they admit negative zero, infinities and NaNs. Warning values retain the full signed-int domain and
duration retains the full signed-long domain through their ordinary VarInt/VarLong encodings.
Overlong or truncated variable integers, truncated fixed-width fields and residual body bytes fault
the owning packet.

Primary codec anchors are `ClientboundSetBorderCenterPacket`,
`ClientboundSetBorderLerpSizePacket`, `ClientboundSetBorderSizePacket`,
`ClientboundSetBorderWarningDelayPacket`, and `ClientboundSetBorderWarningDistancePacket`.

## Client replacement semantics

Every handler first moves to the client main thread and targets the current `ClientLevel` border.
ID 88 calls `setCenter` with both carried values, retaining the current extent and warnings but
recomputing extent coordinates. ID 90 calls `setSize`, immediately replacing any static or moving
extent with a static extent constructed from the carried value.

ID 89 calls `lerpSizeBetween(old,new,duration,currentClientGameTime)`. Equal Java-double endpoints
select a static extent at `new`; unequal endpoints replace the prior extent with a new motion whose
begin metadata is the client level's game time at packet handling. No handler-time positivity or
finite check is added. Consequently zero, negative, NaN and infinite cases retain the precise raw
motion/geometry behavior in `WGEN-BORDER-001`; the protocol layer must not normalize them into a
positive canonical transition.

IDs 91 and 92 replace the independent signed warning-time and warning-block fields. They do not
alter geometry, and center changes do not restart motion. All setters run even for a value equal to
the current one. There is no packet response, revision or monotonicity check.

Primary handler anchors are `ClientPacketListener#handleSetBorderCenter`,
`ClientPacketListener#handleSetBorderLerpSize`, `ClientPacketListener#handleSetBorderSize`,
`ClientPacketListener#handleSetBorderWarningDelay`,
`ClientPacketListener#handleSetBorderWarningDistance`, and `WorldBorder`.

## Authoritative publication and snapshot interaction

`PlayerList#addWorldborderListener` installs a listener on each server level's border. Every call to
`setCenter`, `setSize`, `lerpSizeBetween`, `setWarningTime` or `setWarningBlocks` first mutates the
border and marks its saved state dirty, then synchronously invokes that listener. The listener
constructs the matching packet from the now-current border and broadcasts it only to players whose
dimension key matches that level. There is no equality suppression: an equal setter call still
marks dirty and emits one delta.

Ordinary moving-border ticks send no per-step packet. Damage-per-block and safe-zone listener
callbacks are intentional no-ops and have no clientbound delta identity. A joining, reconnecting or
relocating player instead receives the complete ID-43 snapshot; mid-lerp it carries calculated
current size, target and remaining ticks and therefore restarts client history as already specified
by the gameplay leaf.

IDs 88/91/92 replace independent fields in receive order. IDs 89 and 90 both replace the extent, so
the later received one wins even if it was published earlier. A later ID-43 replaces the complete
border; a delayed delta received after it can then replace its owned field again. Client ticking
advances motion locally and produces no acknowledgement, so packet delay and independent tick
freeze can cause the documented temporary server/client drift.

Ferrite stores normalized per-dimension border authority and saved data. The 26.2 adapter owns the
packet IDs and encodings; client game-time anchors, listener objects, packet order and absent
acknowledgement state are not durable world identity.

Primary publication anchors are `WorldBorder#setCenter`, `WorldBorder#setSize`,
`WorldBorder#lerpSizeBetween`, `WorldBorder#setWarningTime`, `WorldBorder#setWarningBlocks`,
`PlayerList#addWorldborderListener`, `PlayerList$1`, and `PlayerList#sendLevelInfo`.

## C3 world-border delta fault and order boundary

Malformed VarInts/VarLongs, truncated fixed fields and trailing bytes fault before mutation. Raw
but transport-valid numeric values take the source-specified border paths rather than being rejected
by an invented protocol range policy. These deltas acknowledge neither border commands nor any
other gameplay transaction, and ID 43 is an unacknowledged authoritative replacement rather than a
delta-generation barrier.

# C3 Sound Projection and Filtering

IDs 117 and 116 project one sound event at a fixed position or bound to an already tracked entity.
ID 119 filters current client sound instances. These packets carry presentation requests only:
they do not report whether a resource exists, a mixer category is audible or an instance actually
played.

| ID | Identity | Fields in exact order |
|---:|---|---|
| `117` | `minecraft:sound` | sound holder; source enum; X/Y/Z signed ints; volume float; pitch float; seed long |
| `116` | `minecraft:sound_entity` | sound holder; source enum; entity signed VarInt; volume float; pitch float; seed long |
| `119` | `minecraft:stop_sound` | flags byte; optional source enum; optional sound identifier |

The sound holder uses the configured `minecraft:sound_event` registry. A positive encoded value
`n+1` selects registered raw ID `n`; zero selects a direct value containing an identifier and a
boolean-optional fixed-range float. Unknown registered IDs fault. The locked registry has 1,968
entries and digest `174ea5fc5cfc6212cf6a858475811e3d90889734`.

The source is a strict VarInt enum:
`master=0`, `music=1`, `records=2`, `weather=3`, `blocks=4`, `hostile=5`, `neutral=6`,
`players=7`, `ambient=8`, `voice=9`, and `ui=10`. Every other ordinal faults. Volume, pitch and
direct fixed range retain raw big-endian IEEE-754 float bits; seed retains all big-endian signed-long
bits. Malformed identifiers, overlong VarInts, unknown required mappings, truncation and residual
body bytes fault.

Primary codec anchors are `ClientboundSoundPacket`, `ClientboundSoundEntityPacket`,
`ClientboundStopSoundPacket`, `SoundEvent#STREAM_CODEC`, and `SoundSource`.

## Positional coordinate quantization and application

The canonical ID-117 constructor multiplies each source double by eight and applies Java
double-to-int conversion: finite values truncate toward zero, overflow saturates to the signed-int
endpoint and NaN becomes zero. The wire carries those three big-endian signed ints. On receive,
each getter converts int to float, divides by `8.0f`, then widens to double. Coordinates beyond
float's exact integer range therefore lose additional low bits; a compatible decoder must not use
an exact double `wire / 8.0` result.

On the client main thread, ID 117 resolves the holder value and constructs one nonlooping,
nonrelative, linearly attenuated `SimpleSoundInstance` at those decoded coordinates. It uses the
carried seed to initialize resource selection and passes volume/pitch unchanged. This path requests
immediate playback even when the position is far from the camera; ordinary resource lookup,
category volume and sound-engine admission can still make it inaudible.

Primary anchors are `ClientboundSoundPacket#ClientboundSoundPacket`, its coordinate getters,
`ClientPacketListener#handleSoundEvent`, `ClientLevel#playSeededSound`, and
`SimpleSoundInstance`.

## Entity-bound application

ID 116 looks up its signed entity ID in the current client level at handling time. A missing ID is
ignored with no queued retry. A present entity creates one `EntityBoundSoundInstance` whose seeded
resource choice, source, volume and pitch come from the packet. Initial playback is allowed only
while that entity is not silent. The instance narrows each entity coordinate to float and widens it
back to double at construction and every client tick; it stops when that exact entity is removed.

The binding is to the resolved entity object, not a durable numeric identity. A packet received
before spawn is lost; later ID reuse cannot revive it, and removing the bound entity stops its
instance even if another entity later receives the same numeric ID.

Primary anchors are `ClientPacketListener#handleSoundEntityEvent`,
`ClientLevel#playSeededSound`, and `EntityBoundSoundInstance#canPlaySound/#tick`.

## Stop-sound mask and engine filtering

ID 119 reads one signed byte but interprets only low bit `0x01` as source-present and low bit `0x02`
as sound-name-present. Source, when present, precedes the identifier. High bits are ignored. The
four semantic forms are:

| Low bits | Source | Name | Effect |
|---:|---|---|---|
| `0` | absent | absent | stop all current sound instances |
| `1` | present | absent | stop every current instance in that source category |
| `2` | absent | present | stop every current instance with that exact identifier |
| `3` | present | present | stop only current instances matching both |

Identifier matching uses each resolved `SoundInstance` identifier. It is not restricted to sounds
created by IDs 116/117 and can stop matching local, looping or other presentation instances. The
packet installs no persistent suppression rule, reports no stopped count and elicits no response.

Primary anchors are `ClientboundStopSoundPacket`,
`ClientPacketListener#handleStopSoundEvent`, and `SoundEngine#stop`.

## Authoritative publication and ordering

Both canonical `ServerLevel#playSeededSound` overloads derive the audience radius from the event:
an event's fixed range is used unchanged; otherwise range is `16*volume` when `volume>1`, else `16`.
The player-list broadcaster visits players in list order and sends only when the player is not the
exact excluded source player, has the same dimension key and has squared distance strictly less
than squared range. A nonplayer or null source excludes nobody. NaN/infinite/direct-negative range
values retain their Java comparison/squaring behavior rather than receiving an invented protocol
range policy.

The positional overload constructs ID 117 and quantizes coordinates as above. The entity overload
uses the target's current position only for audience selection and carries its current connection
entity ID in ID 116. Canonical tracking should publish spawn before entity sound; receive-order races
still follow the missing-ID rule. `StopSoundCommand` constructs one of the four ID-119 forms and
sends it directly to every selected player, without dimension or distance filtering.

All three packets are unacknowledged. Duplicate sounds create duplicate engine instances; a stop
affects matching current instances only, so stop-before-sound does not suppress a later event.
Ferrite retains namespaced sound/source intent at its gameplay boundary and confines raw registry
IDs, source ordinals, entity IDs, coordinate ints, seeds, mixer objects and masks to the 26.2
adapter/client presentation.

Primary publication anchors are `ServerLevel#playSeededSound`, `SoundEvent#getRange`,
`PlayerList#broadcast`, and `StopSoundCommand#stopSound`.

## C3 sound fault and order boundary

Unknown registered sound IDs, invalid source ordinals, malformed direct identifiers, overlong
entity VarInts, truncation and trailing bytes fault before handling. Transport-valid direct events,
raw numeric values, missing resource definitions, absent entity IDs and unmatched stop filters take
their documented decode/application branches. No sound packet acknowledges another sound, entity
state, chat, container, border, block, teleport or liveness transaction.

# C3 Particle Projection

ID 47 `minecraft:level_particles` carries, in exact order: override-limiter boolean; always-show
boolean; X/Y/Z doubles; X/Y/Z spread floats; max-speed float; count big-endian signed int; then a
strict `minecraft:particle_type` raw VarInt and that type's options payload. Both booleans accept any
nonzero byte as true. Positions and float fields retain every IEEE bit pattern, and count retains the
full signed-int domain.

The configured particle registry has 125 entries and digest
`5dbdae8be2ba868ae33601e37e127d3c9848109a`. The raw ID strictly selects one type and its
`ParticleType#streamCodec`; simple types add no option bytes, while all option-bearing types reuse
the exact block/item/dust/color/entity/vibration/trail/registry-aware codecs already audited by the
C3 explosion family. Unknown type IDs, malformed selected options, truncation and residual bytes
fault before handling.

Primary codec anchors are `ClientboundLevelParticlesPacket`, `ParticleTypes#STREAM_CODEC`, and
every registered `ParticleType#streamCodec`.

## Count forms and client sampling

The main-thread handler has three count forms:

1. `count == 0`: multiply `maxSpeed*xDist`, `maxSpeed*yDist`, and `maxSpeed*zDist` as floats, widen
   each product to double, and attempt one particle at exact X/Y/Z with those velocity components.
   No Gaussian is drawn.
2. `count > 0`: for each attempt, draw three Gaussians from the packet listener's random source and
   multiply by X/Y/Z spread for position offsets, then draw three more and multiply each by
   max-speed for velocity. Position/spread/speed operands widen to double before these products.
3. `count < 0`: the loop executes zero times and no particle or Gaussian draw occurs.

Each attempt calls `ClientLevel#addParticle` immediately. A provider may return null without a
fault. A thrown creation/application error is caught by the packet handler, logged, and abandons the
packet; earlier particles and random consumption remain. The zero form likewise catches/logs its
single failure. No response or semantic retry follows.

## Override, distance and user-setting gates

The effective override is packet override OR the selected particle type's locked override flag.
Exactly these 32 raw-ID/type pairs own that flag:

```text
2 block_marker; 7 geyser; 8 geyser_base; 9 geyser_poof; 10 geyser_plume;
14 damage_indicator; 24 elder_guardian; 29 explosion_emitter; 30 explosion;
31 gust; 33 gust_emitter_large; 34 gust_emitter_small; 35 sonic_boom;
45 sculk_charge; 46 sculk_charge_pop; 55 vibration; 66 poof; 72 spit;
73 squid_ink; 74 sweep_attack; 84 campfire_cosy_smoke; 85 campfire_signal_smoke;
106 glow_squid_ink; 107 glow; 108 wax_on; 109 wax_off; 110 electric_spark;
111 scrape; 115 trial_spawner_detection; 116 trial_spawner_detection_ominous;
117 vault_connection; 119 ominous_spawning
```

The client calculates the user particle level before testing effective override. With always-show
true, MINIMAL first has a `1/10` chance to become DECREASED; every DECREASED value then has a `1/3`
chance to become MINIMAL. Thus a nonoverridden attempt emits with probability `2/3` under DECREASED,
never under MINIMAL without always-show, and `1/15` under MINIMAL with always-show. ALL emits.

Effective override bypasses both the resulting setting and camera distance, but the calculation and
its RNG draws still occurred. Without override, squared camera distance above `1024` rejects the
attempt (exactly 32 blocks is admitted), then a MINIMAL result rejects it. Successful admission asks
the particle engine to create the selected type/options at the sampled state.

Primary anchors are `ClientPacketListener#handleParticleEvent`,
`ClientLevel#addParticle`, `ClientLevel#doAddParticle`, `ClientLevel#calculateParticleLevel`,
`ParticleType#getOverrideLimiter`, and `ParticleEngine#createParticle`.

## Authoritative publication and audience

Canonical `ServerLevel#sendParticles` accepts position/spread/speed as doubles, narrows the four
spread/speed values once to floats in the packet, and preserves position doubles/count/flags. It
builds one packet and iterates the level's player list. A player receives it only when still in that
exact `ServerLevel` and the center of the player's integer block position is strictly closer than
`32` blocks to packet position, or `512` when packet override is true. Always-show does not enlarge
the server audience. The aggregate overload returns the number of recipients; the targeted overload
returns whether its one player passed.

Packet override therefore has coupled server/client meaning: long-distance audience plus forced
client admission. Type-owned override affects only client admission because it is discovered after
decode. Count, option semantics and visibility do not alter authoritative simulation and receive no
acknowledgement. Duplicate packets repeat their full sampling; interleaving is ordinary receive
order with independent sound/level-event/entity/block/chat/container/border/teleport/liveness state.

Ferrite projects normalized particle intent from owned gameplay effects but confines raw type IDs,
packet flags, narrowed spread values, client settings, random streams and engine particle objects to
the 26.2 adapter/client presentation.

Primary publication anchors are both `ServerLevel#sendParticles` overloads and
`BlockPos#closerToCenterThan`.

## C3 particle fault and order boundary

Codec mapping/options failures fault the packet. Transport-valid negative counts, nonfinite values,
limiter omissions, missing providers and caught handler-time creation failures take their exact
branches above. ID 47 has no sequence, response, generation, retry or convergence state.

# C3 Level Event Projection

ID 46 `minecraft:level_event` is the shared presentation envelope used by blocks, items, entities,
world transitions and trial/vault effects. Its body is, in exact order: event type big-endian signed
int; packed block-position long; event data big-endian signed int; global-event boolean. The
position uses the common signed 26/12/26 X/Y/Z packing. The boolean accepts zero as false and every
nonzero byte as true; neither signed int has codec-level range validation.

The client first moves handling to its main thread, then selects exactly one dispatch table from the
boolean. A false packet calls the 80-entry local table below. A true packet calls the global table,
which recognizes only 1023, 1028 and 1038. There is no fall-through between tables: a global-only
ID with false is a no-op, and any local ID with true is a no-op. Every otherwise unknown signed ID
is also a no-op and `data` is ignored unless this section assigns it a meaning.

Primary codec and dispatch anchors are `ClientboundLevelEventPacket`,
`ClientPacketListener#handleLevelEvent`, `ClientLevel#levelEvent/#globalLevelEvent`, and
`LevelEventHandler`.

## Local sound and jukebox events

For the compact sound matrix, `D(b)` means pitch `1+(r1-r2)*b` using two successive level-random
floats, and `U(a,b)` means `a+r*b` using one. Every row is positioned at the packet block position,
uses an immediate local sound (`distanceDelay=false`), and ignores `data`. The sound constructor
then consumes a level-random long as its resource-selection seed. Constant pitch rows draw no
pitch random value.

| Event | Sound; source; volume; pitch |
|---:|---|
| `1000` | `minecraft:block.dispenser.dispense`; blocks; 1; 1 |
| `1001` | `minecraft:block.dispenser.fail`; blocks; 1; 1.2 |
| `1002` | `minecraft:block.dispenser.launch`; blocks; 1; 1.2 |
| `1004` | `minecraft:entity.firework_rocket.shoot`; neutral; 1; 1.2 |
| `1015` | `minecraft:entity.ghast.warn`; hostile; 10; `D(0.2)` |
| `1016` | `minecraft:entity.ghast.shoot`; hostile; 10; `D(0.2)` |
| `1017` | `minecraft:entity.ender_dragon.shoot`; hostile; 10; `D(0.2)` |
| `1018` | `minecraft:entity.blaze.shoot`; hostile; 2; `D(0.2)` |
| `1019` | `minecraft:entity.zombie.attack_wooden_door`; hostile; 2; `D(0.2)` |
| `1020` | `minecraft:entity.zombie.attack_iron_door`; hostile; 2; `D(0.2)` |
| `1021` | `minecraft:entity.zombie.break_wooden_door`; hostile; 2; `D(0.2)` |
| `1022` | `minecraft:entity.wither.break_block`; hostile; 2; `D(0.2)` |
| `1024` | `minecraft:entity.wither.shoot`; hostile; 2; `D(0.2)` |
| `1025` | `minecraft:entity.bat.takeoff`; neutral; 0.05; `D(0.2)` |
| `1026` | `minecraft:entity.zombie.infect`; hostile; 2; `D(0.2)` |
| `1027` | `minecraft:entity.zombie_villager.converted`; hostile; 2; `D(0.2)` |
| `1029` | `minecraft:block.anvil.destroy`; blocks; 1; `U(0.9,0.1)` |
| `1030` | `minecraft:block.anvil.use`; blocks; 1; `U(0.9,0.1)` |
| `1031` | `minecraft:block.anvil.land`; blocks; 0.3; `U(0.9,0.1)` |
| `1033` | `minecraft:block.chorus_flower.grow`; blocks; 1; 1 |
| `1034` | `minecraft:block.chorus_flower.death`; blocks; 1; 1 |
| `1035` | `minecraft:block.brewing_stand.brew`; blocks; 1; 1 |
| `1039` | `minecraft:entity.phantom.bite`; hostile; 0.3; `U(0.9,0.1)` |
| `1040` | `minecraft:entity.zombie.converted_to_drowned`; hostile; 2; `D(0.2)` |
| `1041` | `minecraft:entity.husk.converted_to_zombie`; hostile; 2; `D(0.2)` |
| `1042` | `minecraft:block.grindstone.use`; blocks; 1; `U(0.9,0.1)` |
| `1043` | `minecraft:item.book.page_turn`; blocks; 1; `U(0.9,0.1)` |
| `1044` | `minecraft:block.smithing_table.use`; blocks; 1; `U(0.9,0.1)` |
| `1045` | `minecraft:block.pointed_dripstone.land`; blocks; 2; `U(0.9,0.1)` |
| `1046` | `minecraft:block.pointed_dripstone.drip_lava_into_cauldron`; blocks; 2; `U(0.9,0.1)` |
| `1047` | `minecraft:block.pointed_dripstone.drip_water_into_cauldron`; blocks; 2; `U(0.9,0.1)` |
| `1048` | `minecraft:entity.skeleton.converted_to_stray`; hostile; 2; `D(0.2)` |
| `1049` | `minecraft:block.crafter.craft`; blocks; 1; 1 |
| `1050` | `minecraft:block.crafter.fail`; blocks; 1; 1 |
| `1051` | `minecraft:entity.wind_charge.throw`; blocks; 0.5; `0.4/(0.8+0.4*r)` |
| `1052` | `minecraft:block.sulfur_spike.land`; blocks; 2; `U(0.9,0.1)` |

Event 1009 is data-sensitive: zero plays `minecraft:block.fire.extinguish` in blocks at volume 0.5
and pitch `2.6+(r1-r2)*0.8`; one plays `minecraft:entity.generic.extinguish_fire` at volume 0.7
and pitch `1.6+(r1-r2)*0.4`; every other data value does nothing. Event 1032 ignores position and
data and submits local ambience `minecraft:block.portal.travel` with pitch `0.8+0.4*r`, volume
0.25 and the ambience source/settings path.

Event 1010 interprets `data` as a raw ID in the ordered dynamic `minecraft:jukebox_song` registry
frozen during configuration. A negative, absent or out-of-range ID does nothing and does not stop a
currently tracked song. A present holder first stops the tracked instance at the exact block
position, then creates that song's configured sound at block center, stores it by position, updates
the HUD with its configured description and marks living entities in the position's one-block AABB
inflated by three as record-playing. Event 1011 ignores data, stops/removes the tracked instance if
present, and marks every living entity in that same AABB as not record-playing even when no sound
was tracked. Jukebox raw IDs and client sound instances are presentation state, not durable song or
block identity.

## Local block, item and particle events

The remaining local IDs have the following exact data ownership. Counts are Java signed-int
arithmetic and loops use their literal comparisons, so negative and overflowing inputs are not
silently clamped unless a row says otherwise.

| Event | Data interpretation and ordered presentation effect |
|---:|---|
| `1500` | `data > 0` selects composter-success, otherwise ordinary fill; play the selected blocks sound at 1/1, derive surface height from the current block shape and emit ten composter particles. |
| `1501` | Ignore data; play lava-extinguish at 0.5 and `2.6+(r1-r2)*0.8`, then emit eight zero-velocity large-smoke particles at random X/Z and Y+1.2. |
| `1502` | Ignore data; play redstone-torch-burnout at 0.5 and `2.6+(r1-r2)*0.8`, then emit five zero-velocity smoke particles uniformly in the block's `[0.2,0.8)` cube. |
| `1503` | Ignore data; play end-portal-frame-fill at 1/1, then emit 16 zero-velocity smoke particles at Y+0.8125 with X/Z offsets `(5+6*r)/16`. |
| `1504` | Ignore data; ask pointed-dripstone to emit one drip from the current block state. |
| `1505` | Pass data to bone-meal growth particles, then always play bone-meal-use at 1/1. A neighbor-spreader bonemealable or water requests `data*3` particles; an in-block bonemealable requests `data`; other states request none. |
| `2000` | Treat `abs(data % 6)` as direction `down,up,north,south,west,east` and emit ten directional smoke particles with the locked `shootParticles` position/velocity sampling. |
| `2010` | Same direction and ten-attempt algorithm as 2000, using white smoke. |
| `2001` | Map data through the 32,366-entry global block-state table, falling back to air for every absent ID. If nonair, play its break sound at `(volume+1)/2` and `pitch*0.8`; then invoke the destroy-block effect even for air. |
| `2002` | Splash potion: emit eight `minecraft:splash_potion` item particles, then 100 effect particles colored from data bits `23..16`, `15..8`, `7..0`, then play potion-break at 1 and `U(0.9,0.1)`. |
| `2007` | Identical to 2002 except the 100 colored particles use instant-effect. |
| `2003` | Ignore data; emit eight `minecraft:ender_eye` item particles and, for angles starting at zero below `2*pi` in steps of `pi/20`, two portal particles at radius five with inward speeds five and seven. |
| `2004` | Ignore data; for 20 sampled positions in the two-block cube centered on block center, emit one smoke and one flame. |
| `2006` | Emit 200 radial dragon-breath particles; only data exactly one additionally plays dragon-fireball-explode at 1 and `U(0.9,0.1)`. |
| `2008` | Ignore data; emit one explosion particle at block center. |
| `2009` | Ignore data; emit eight zero-velocity cloud particles at random X/Z and Y+1.2. |
| `2011` | Bee growth: request data happy-villager particles inside the current block shape. |
| `2012` | Turtle-egg placement: the same data/count and current-shape algorithm as 2011. |
| `2013` | Smash attack: use the current block state as dust-pillar options; the center cloud loop runs while `i < data/3.0f`, then the radius-3.5 ring loop while `i < data/1.5f`, with the helper's Gaussian positions and velocities. |
| `3000` | Ignore data; add one always-visible explosion-emitter at block center, then end-gateway-spawn in blocks at volume 10 and pitch `(1+(r1-r2)*0.2)*0.7`. |
| `3001` | Ignore data; play ender-dragon-growl in hostile at volume 64 and pitch `0.8+0.3*r`. |
| `3002` | Data 0/1/2 selects X/Y/Z and emits 10..19 electric sparks along that axis at radius 0.125; every other value emits 3..5 sparks on each of all six faces. |
| `3003` | Ignore data; emit 3..5 wax-on particles on each face, then honeycomb-wax-on in blocks at 1/1. |
| `3004` | Ignore data; emit 3..5 wax-off particles on each face. |
| `3005` | Ignore data; emit 3..5 scrape particles on each face. |
| `3006` | Sculk charge; decode exactly as specified below. |
| `3007` | Ignore data; emit ten shriek particles at block-top center with delays `0,5,...,45`; play sculk-shrieker-shriek at volume 2 and pitch `0.6+0.4*r` unless the current state has waterlogged=true. |
| `3008` | Map data through the global block-state table with absent-to-air fallback; if its block is brushable play that type's completed sound in players at 1/1, then invoke the mapped state's destroy effect. |
| `3009` | Ignore data; emit 3..6 egg-crack particles on each face. |

For 3006, `count = data >> 6` is an arithmetic shift and the low six bits are the face mask in
Direction ordinal order: down, up, north, south, west, east. When count is positive, first draw a
charge-sound admission float and play `minecraft:block.sculk.charge` only when it is below
`0.3+0.1*count`; admitted volume is `0.15+0.02*count*count*r` and pitch is `0.4+0.3*count*r` with
ordinary float/int promotion. A zero mask emits `UniformInt(0,count)` charge particles
on all faces; a nonzero mask emits them only on set faces, with the locked rotations, face offsets
and three `[-0.005,0.005)` velocity draws. When count is zero or negative, ignore the mask, play
sculk-charge at 1/1, inspect whether the current collision shape is a full block, then emit 40 pop
particles with spread 0.45 or 20 with spread 0.25; velocity scale is 0.07.

The directional helper used by 2000/2010 performs ten attempts. Each draws power in `[0.01,0.21)`,
three position uniforms and then three velocity Gaussians. Its position is block center shifted
0.61 along the selected normal, with the source's exact normal-dependent tangential formulas; its
velocity is normal times power plus Gaussian times 0.01. Particle-face and growth helper routines
use the exact draw/order rules owned by `ParticleUtils`, `BoneMealItem#addGrowthParticles`,
`ComposterBlock#handleFill` and `PointedDripstoneBlock#spawnDripParticle`.

## Trial-spawner, vault and cobweb events

Trial flame data decoding is intentionally irregular in the locked source: zero selects flame, one
selects soul-fire-flame, two indexes past the two-entry array and faults during handling, while every
negative value and every value greater than two falls back to flame. This mapping is used by 3011,
3012 and 3021.

| Event | Data interpretation and ordered presentation effect |
|---:|---|
| `3011` | Decode flame data, then emit 20 co-located smoke/flame pairs in the two-block cube around center. |
| `3012` | Play trial-spawner-spawn-mob in blocks at volume 1, `D(0.2)`, distance-delay enabled; then decode data and emit the 20 pairs. |
| `3013` | Play trial-spawner-detect-player with the same sound settings, then emit `30+min(data,10)*5` detected-player particles under literal signed-int loop arithmetic. |
| `3014` | Play trial-spawner-eject-item with the same sound settings, then emit 20 small-flame/smoke pairs in the center `[0.4,0.6)` cube. |
| `3015` | Require a vault block entity at the current position or do nothing. For one, data zero selects small flame and every nonzero value soul-fire flame; emit activation particles from current vault state/shared data, then play vault-activate with the same delayed 1/`D(0.2)` settings. |
| `3016` | Without a block-entity gate, choose small flame for data zero and soul-fire flame otherwise, emit vault deactivation particles, then play vault-deactivate with the same delayed settings. |
| `3017` | Ignore data; emit the same 20 eject-item pairs as 3014 and no sound. |
| `3018` | Ignore data; emit ten poof particles, each drawing three Gaussian velocities before three uniform block-position coordinates; then play cobweb-place with delayed 1/`D(0.2)`. |
| `3019` | As 3013 but use ominous detected-player particles. |
| `3020` | Play trial-spawner-ominous-activate at volume 0.3 for data zero or 1 otherwise, `D(0.2)` and distance-delay enabled; emit the data-zero ominous detection set and then 20 trial-omen/soul-fire pairs. |
| `3021` | Play trial-spawner-spawn-item with delayed 1/`D(0.2)`, then decode flame data and emit the 20 spawn pairs. |

For a distance-delay-enabled local sound farther than ten blocks from the camera, the client delays
playback by Java-int truncation of `distance/40*20` ticks; at or within ten blocks it submits it
immediately. Sound/pitch and particle operations occur in the table order, so their shared client
level RNG consumption and partial visible prefix are observable presentation details.

## Global-only events and authoritative publication

For a true global packet, event 1023 selects wither-spawn in hostile at volume 1, event 1028
selects ender-dragon-death in hostile at volume 5, and event 1038 selects end-portal-spawn in hostile
at volume 1; all have pitch 1 and ignore data. If the main camera is uninitialized, they do nothing.
Otherwise the client normalizes the vector from camera position to packet block center and places
the sound exactly two blocks along that vector from the camera. These are local sound submissions,
not sounds placed at the carried block position.

`ServerLevel#levelEvent` constructs a false packet and asks `PlayerList#broadcast` to visit players
in list order. It excludes the exact source only when that entity is a player, requires the same
dimension key and squared distance strictly below `64^2` from the integer packet position, and
sends no one else. A null or nonplayer source excludes nobody.

`ServerLevel#globalLevelEvent` first reads `globalSoundEvents`. When false it invokes the ordinary
local publisher with null source and the same type/position/data. When true it sends one true packet
to every connected player. A player in the event level and strictly within 32 blocks of event block
center receives the actual position. A farther player in that level receives the floor-packed point
32 blocks from player position toward event center. A player in every other level receives their
own floor-packed position. Those substituted positions preserve direction for the client-side
two-block placement; they do not change which players receive the global event.

All forms are tokenless presentation requests. Receive order directly orders sounds, jukebox map
replacement, shared RNG draws and particles. An unknown ID, invalid jukebox raw ID, absent vault
entity or nonmatching current block state takes its documented no-op/fallback without a response.
Local-event helper failures are wrapped as a `ReportedException` crash report after retaining any
earlier sounds/particles/RNG effects; notably flame data two can take this path. No level event
acknowledges or converges block, entity, item, container, chat, teleport, border, particle, sound or
liveness authority.

Ferrite projects a normalized namespaced effect plus its owned semantic data and publication scope.
It does not persist packet/event integers, dynamic jukebox raw IDs, global/local booleans, client
block/entity caches, RNG streams, HUD state, sound instances or engine particles as authoritative
world identity.

Primary anchors are `ServerLevel#levelEvent/#globalLevelEvent`, `PlayerList#broadcast`,
`LevelEventHandler#levelEvent/#globalLevelEvent`, `TrialSpawner`, `VaultBlockEntity.Client`,
`ParticleUtils`, `Block#stateById`, and `MultifaceBlock#unpack`.

## C3 level-event fault and order boundary

Truncation and trailing bytes fault the fixed packet grammar before handling; every complete bit
pattern is transport-valid. Semantic unknowns generally take the explicit no-op or fallback paths,
while helper-time exceptions retain their already-applied prefix and become a reported client
failure. ID 46 has no sequence, response, generation, retry or convergence state.

# C3 Title, Action-Bar and Tab Presentation

Seven clientbound packets replace independent HUD presentation fields or correct the selected
advancement tab. Their exact bodies are:

| ID | Identity | Fields in exact order |
|---:|---|---|
| `14` | `minecraft:clear_titles` | reset-times boolean |
| `85` | `minecraft:select_advancements_tab` | nullable advancement identifier |
| `87` | `minecraft:set_action_bar_text` | trusted registry-aware component |
| `112` | `minecraft:set_subtitle_text` | trusted registry-aware component |
| `114` | `minecraft:set_title_text` | trusted registry-aware component |
| `115` | `minecraft:set_titles_animation` | fade-in, stay and fade-out big-endian signed ints |
| `122` | `minecraft:tab_list` | trusted registry-aware header component; trusted registry-aware footer component |

The ID-14 boolean accepts every nonzero byte as true. ID 85 writes a presence boolean followed, when
true, by the common default-bounded identifier. The other fields are fixed ints or trusted
registry-aware component NBT, with the shared frame bound and depth 512 but no smaller component
quota. Invalid identifiers, malformed component/NBT/registry references, truncation and residual
bytes fault before handling. Numeric signs and boolean byte values receive no further codec check.

Primary codec anchors are the seven matching `Clientbound*Packet` classes,
`ComponentSerialization#TRUSTED_STREAM_CODEC`, and `FriendlyByteBuf#readNullable/#writeNullable`.

## HUD replacement and timer state

Every handler first moves to the client main thread. ID 87 replaces the action-bar overlay component,
sets its remaining time to 60 client ticks and disables animated overlay color. It does not touch
title/subtitle state. Repeated action-bar packets replace the component and restart the 60 ticks;
an empty component is still stored rather than normalized to null.

ID 112 replaces only the subtitle. It does not activate or restart a title. ID 114 replaces the
title and sets remaining title time to the Java signed-int sum of the current fade-in, stay and
fade-out values. Defaults are 10, 70 and 20. The sum may wrap; a nonpositive result is retained and
does not create visible time. A later subtitle therefore affects the currently active title without
restarting it, while a later title uses whatever subtitle is then stored.

ID 115 examines its three values independently. A nonnegative value replaces that phase duration;
a negative value leaves the current duration unchanged. After those replacements, it recomputes
remaining title time from the three effective durations only when the old remaining time was
positive. Thus timing received before a title changes defaults only, timing during an active title
restarts that title from the effective sum, and timing after clear cannot revive it. The arithmetic
is ordinary wrapping signed-int addition.

ID 14 always clears title and subtitle and sets remaining title time to zero. When reset-times is
true it then restores durations to 10/70/20; false preserves the current durations. It does not
clear or reset action-bar state. All title/action-bar transitions are local presentation with no
response, completion notice or acknowledgement when a timer expires.

Primary handler anchors are `ClientPacketListener#handleTitlesClear/#setActionBarText`,
`#setTitleText/#setSubtitleText/#setTitlesAnimation`, and
`Hud#setOverlayMessage/#setTitle/#setSubtitle/#setTimes/#clearTitles/#resetTitleTimes`.

## Advancement selection and player-list decoration

For ID 85, null selects null. A present identifier is looked up in the current client advancement
manager; an unknown identifier also selects null. `ClientAdvancements#setSelectedTab` is invoked
with `notifyServer=false`, so the correction never emits the serverbound opened-tab packet. It
replaces by holder object identity and notifies the installed client listener only when that
identity changes. Display/root validity was already normalized by the server publisher; a manually
sent known child can still become the client selection because the receive handler performs only
the identifier lookup.

ID 122 replaces header and footer independently. Before each replacement the handler flattens that
component through `Component#getString`: an empty resulting string becomes null, while any nonempty
result retains the original styled component. A structurally nonempty component whose rendered
plain string is empty therefore clears its field. Header/footer do not affect player-info entries,
ordering, visibility or list-open state, and the locked vanilla server has no construction site for
ID 122 outside the protocol class itself.

Primary anchors are `ClientPacketListener#handleSelectAdvancementsTab`,
`ClientAdvancements#get/#setSelectedTab`,
`ClientPacketListener#handleTabListCustomisation`, and `PlayerTabOverlay#setHeader/#setFooter`.

## Authoritative publication and ordering

The gamemaster-only `title` command sends IDs 14/87/112/114/115 directly to its selected players in
target iteration order and without dimension or distance filtering. Clear/reset and times construct
one immutable packet and reuse it for every target. Title/subtitle/action-bar commands instead
resolve the raw component separately for each target with the command source plus that player as
entity override, construct one packet from the resolved component and immediately send it. A
resolution failure after earlier targets therefore retains their already-sent prefix and prevents
later sends. Canonical `title ... times` input is converted to nonnegative tick ints before ID 115,
while direct protocol senders can exercise every signed value.

`PlayerAdvancements#setSelectedTab` retains only a nonnull root advancement with display metadata;
every other requested holder becomes null. It sends ID 85 only when retained holder identity changes,
carrying that root's identifier or null. The receive-side no-echo rule prevents a correction loop.
Advancement definition/progress packets may independently replace the holders, so receive order and
current manager contents decide whether an identifier resolves.

These packets have no common transaction or generation. Delayed title text can use newer timing;
clear-before-title and title-before-clear differ; a delayed tab correction can resolve against a
newer advancement tree; later header/footer/action-bar packets simply win their independent fields.
Ferrite retains normalized presentation intent at its command/gameplay boundary but does not persist
packet IDs, timer counters, advancement holder objects, flattened strings, HUD widgets or absent
acknowledgements as authoritative state.

Primary publication anchors are `TitleCommand#clearTitle/#resetTitle/#showTitle/#setTimes` and
`PlayerAdvancements#setSelectedTab`.

## C3 title/tab fault and order boundary

Malformed trusted components, identifiers, truncation and trailing bytes fault before mutation.
Transport-valid negative times, empty components and missing advancement IDs take the exact handler
branches above. All seven packets are tokenless presentation replacements and acknowledge no chat,
advancement-progress, command, scoreboard, entity, container, teleport or liveness transaction.

# C3 Combat Notice and Look-at Projection

Four packets project combat lifecycle/death presentation and an imperative local-player rotation:

| ID | Identity | Fields in exact order |
|---:|---|---|
| `66` | `minecraft:player_combat_end` | signed duration VarInt |
| `67` | `minecraft:player_combat_enter` | empty body |
| `68` | `minecraft:player_combat_kill` | signed player entity VarInt; trusted registry-aware death-message component |
| `71` | `minecraft:player_look_at` | from-anchor enum; X/Y/Z doubles; at-entity boolean; if true, signed entity VarInt and to-anchor enum |

Anchor ordinals are strict: feet is zero and eyes is one; every other VarInt faults. The look-at
boolean accepts every nonzero byte as true. Doubles retain all big-endian IEEE-754 bit patterns,
entity/duration VarInts retain the complete signed domain, and the death component uses the shared
trusted registry-aware component NBT rules. ID 68 is marked skippable by the packet type; its
decode/application errors use the protocol's skippable-packet error policy. Invalid enums,
malformed components/VarInts, truncation and residual bytes otherwise fault before semantic use.

Primary codec anchors are `ClientboundPlayerCombatEndPacket`,
`ClientboundPlayerCombatEnterPacket`, `ClientboundPlayerCombatKillPacket`,
`ClientboundPlayerLookAtPacket`, and `EntityAnchorArgument.Anchor`.

## Combat lifecycle and death handling

The ID-66 and ID-67 client handlers are intentional empty methods: they do not switch threads,
inspect duration, mutate UI/combat state or reply. Negative, duplicate and reordered end durations
are therefore transport-visible but semantically inert in this client.

ID 68 switches to the client main thread and looks up its signed player ID in the current client
level. It continues only when the resulting entity object is the exact current local-player object;
a missing ID, another entity or a prior/later numeric reuse is ignored. When the local player's
login-derived show-death-screen flag is true, the GUI installs a new `DeathScreen` with the packet
message, the current client level's hardcore flag and the current player. When false, it immediately
calls local respawn: send serverbound ID 11 `PERFORM_RESPAWN`, then reset toggle keys. The message is
unused in that immediate-respawn branch.

There is no death generation or duplicate suppression. Repeated qualifying ID 68 packets replace
the death screen repeatedly or send repeated respawn requests. The packet does not itself mutate
health, death state, entity removal, inventory, statistics or respawn authority; those remain in
their independent families.

Primary handler anchors are `ClientPacketListener#handlePlayerCombatEnd/#handlePlayerCombatEnter`,
`#handlePlayerCombatKill`, `LocalPlayer#shouldShowDeathScreen/#respawn`, and `DeathScreen`.

## Look-at target resolution and rotation

For a coordinate-form ID 71, at-entity is false and the packet doubles are always the target. For
an entity-form packet, X/Y/Z are the selected target anchor at send time. At handling time the
client looks up the signed entity ID in its current level: a present entity replaces those doubles
with its current feet or eyes anchor; a missing entity uses the carried fallback coordinates. Thus
spawn-before-look follows a moving entity, look-before-spawn still rotates once to the send-time
fallback, and there is no queued binding or later retry.

The local player computes its origin from the from-anchor: feet is current position, eyes adds the
current eye-height float widened to double. For differences `dx,dy,dz`, it computes horizontal
distance `sqrt(dx*dx+dz*dz)`, then sets pitch to
`wrapDegrees((float)(-atan2(dy,horizontal)*57.2957763671875))` and yaw to
`wrapDegrees((float)(atan2(dz,dx)*57.2957763671875)-90)`. It copies yaw to head rotation and copies
the resulting pitch/yaw to previous rotations; living-entity handling also aligns current/previous
body and previous head rotation. Raw coincident, NaN and infinite targets follow these exact Java
math/float-narrowing paths without a finite check.

Primary anchors are `ClientboundPlayerLookAtPacket#getPosition`,
`ClientPacketListener#handleLookAt`, `EntityAnchorArgument.Anchor#apply`,
`Entity#lookAt`, and `LivingEntity#lookAt`.

## Authoritative publication and ordering

`ServerPlayer#onEnterCombat` invokes ordinary player combat entry then sends the singleton empty ID
67 directly to that player's connection. `onLeaveCombat` invokes ordinary leave behavior, reads the
combat tracker's current duration into ID 66 and sends it to the same connection. These packets have
no client effect despite preserving lifecycle observability on the wire.

On player death, `ServerPlayer#die` always sends ID 68 to the dying player's connection before its
later death cleanup. With `showDeathMessages=true`, it uses the combat tracker's death component and
attaches an exceptional-send fallback: if sending that component fails, retry with the
`death.attack.even_more_magic` component containing the first 256 flattened message characters in
yellow hover text. Team/global death-message broadcast is a separate ID-121 path. With the gamerule
false, ID 68 instead carries the shared empty component and there is no public death-message
broadcast. The player entity ID is its current connection-local ID in both cases.

Both `ServerPlayer#lookAt` overloads first rotate authoritative player state, then send ID 71 only
to that player's connection. Coordinate form preserves supplied doubles. Entity form carries the
target's current connection-local ID and selected-anchor coordinates as fallback. It applies no
distance/dimension/client-tracking gate, so a cross-level or untracked target predictably takes the
fallback on the client.

All publications are direct and tokenless. Combat enter/end may bracket ID 68 but the client does
not correlate them; look-at resolution and death entity identity use handler-time state. ID 68's
immediate-respawn request is causally triggered but carries no combat-kill token. Ferrite projects
normalized death/look intent and authoritative rotations while keeping raw entity IDs, fallback
coordinates, client screens, prior rotations and combat packet order out of durable identity.

Primary publication anchors are `ServerPlayer#onEnterCombat/#onLeaveCombat/#die` and both
`ServerPlayer#lookAt` overloads.

## C3 combat/look fault and order boundary

Invalid anchor ordinals, malformed trusted death components, malformed VarInts, truncation and
trailing data fault under the packet's applicable error policy. Missing/wrong entities, ignored
durations and raw nonfinite coordinate values take the documented semantic branches. IDs 66/67/71
have no response; ID 68 can produce only the uncorrelated immediate-respawn request described above.

# C3 Boss-Bar and Waypoint Projection

IDs 9 and 138 maintain two UUID/keyed client presentation collections. They do not report boss
health or entity position authority back to the server.

ID 9 `minecraft:boss_event` begins with a 16-byte UUID and a strict operation VarInt:

| Operation | Ordinal | Following fields |
|---|---:|---|
| add | `0` | trusted name component; progress float; color enum; overlay enum; properties byte |
| remove | `1` | none |
| update progress | `2` | progress float |
| update name | `3` | trusted name component |
| update style | `4` | color enum; overlay enum |
| update properties | `5` | properties byte |

Colors are strict ordinals `pink=0`, `blue=1`, `red=2`, `green=3`, `yellow=4`, `purple=5`, and
`white=6`. Overlays are `progress=0`, `notched_6=1`, `notched_10=2`, `notched_12=3`, and
`notched_20=4`. Properties low bits are darken-screen `0x01`, play-music `0x02` and create-fog
`0x04`; high bits are ignored. Floats retain raw IEEE bits and trusted components use the common
registry-aware NBT rules. Unknown operation/color/overlay ordinals fault.

ID 138 `minecraft:waypoint` begins with an operation VarInt, followed by one complete tracked
waypoint. Operation decoding deliberately wraps by positive modulo three: track is residue zero,
untrack residue one and update residue two, including negative and out-of-range signed VarInts. The
tracked waypoint is:

```text
identifier_is_uuid:boolean
if identifier_is_uuid { uuid:128 bits } else { identifier:string }
style:identifier
color_present:boolean
if color_present { red:u8; green:u8; blue:u8 }
type:strict VarInt enum
switch type {
    0 empty    => no fields
    1 position => x:VarInt; y:VarInt; z:VarInt
    2 chunk    => chunk_x:VarInt; chunk_z:VarInt
    3 azimuth  => angle:f32
}
```

Both booleans accept every nonzero byte as true. The alternative string and style use common
default UTF/identifier bounds; style is a waypoint-style resource key but is not resolved through a
wire registry. Optional RGB is exactly three bytes and decodes with opaque alpha. Position/chunk
coordinates retain signed VarInts and azimuth retains raw float bits. The type enum is strict even
though the outer operation wraps. Malformed strings/identifiers/VarInts, invalid types, truncation
and residual data fault.

Primary codec anchors are `ClientboundBossEventPacket` and its operation records,
`ClientboundTrackedWaypointPacket.Operation`, `TrackedWaypoint#STREAM_CODEC` and its four type
classes, and `Waypoint.Icon#STREAM_CODEC`.

## Boss collection, interpolation and aggregate properties

The main-thread boss handler owns a linked UUID map. Add constructs a new `LerpingBossEvent` from
all carried fields and puts it under the UUID; adding an existing UUID replaces its value without
moving its linked-map position. Remove deletes that UUID and silently tolerates absence. Every
update dereferences the existing value without a null check, so update-before-add or update-after-
remove throws during handling rather than being queued or ignored.

Name, style and properties updates replace their exact fields. Progress update first samples the
current interpolated progress, stores it as the new start, stores the raw target and records current
wall-clock milliseconds. Reads linearly interpolate from start to target over 100 milliseconds with
the elapsed fraction clamped to `[0,1]`. A second update during the lerp therefore starts from the
then-visible value. Raw NaN/infinite targets follow float lerp/render arithmetic without protocol
normalization.

Bars render in linked insertion order from the top, stopping once the next vertical offset reaches
one third of GUI height. Each uses the selected 182-by-5 color/overlay sprites, a discrete progress
width derived from raw interpolated progress, and centered name text. Music, screen darkening and
world fog are each enabled when any currently mapped bar owns that property; remove/replacement can
therefore change these aggregate presentation gates immediately.

Primary handler anchors are `ClientPacketListener#handleBossUpdate`, `BossHealthOverlay` and its
packet handler, and `LerpingBossEvent#setProgress/#getProgress`.

## Waypoint collection, update rules and projection

The main-thread waypoint handler owns a concurrent map keyed by the decoded `Either<UUID,String>`.
Track puts/replaces the complete object. Untrack removes by identifier only; its icon, type and
contents are otherwise irrelevant. Update requires an existing key or throws. On an existing key it
mutates only location content when old and new concrete types match: position replaces Vec3i,
chunk replaces ChunkPos and azimuth replaces angle. A mismatched type logs a warning and leaves the
old content. Empty update does nothing. The existing icon is never replaced by update, even when the
packet carries a different one; changing icon or representation requires track replacement.

Position projection normally uses block center. When its identifier is a UUID resolving to a
current client entity whose block position is within Manhattan distance three of the carried
vector, it instead uses that entity's partial-tick eye position. String IDs and absent/far entities
use block center. Chunk projection uses the chunk middle block center at camera Y for yaw and at the
viewer's block Y for squared-distance ordering. Azimuth stores radians, converts to degrees for yaw
difference and has infinite distance; empty has NaN yaw/infinite distance. Position uses projected
point pitch, while chunk/azimuth use projected horizon. Rendering iterates markers in descending
squared distance; equal-key hash/concurrency order is not a stable authority.

Primary anchors are `ClientPacketListener#handleWaypoint`, `ClientWaypointManager`, and
`TrackedWaypoint.EmptyWaypoint/#Vec3iWaypoint/#ChunkWaypoint/#AzimuthWaypoint`.

## Boss publication

`ServerBossEvent` starts visible with an explicit UUID and a hash-set audience. Adding/removing a
player changes membership once and, while visible, directly sends add/remove only to that player.
Visibility changes send one full add or remove to every current member; changes while hidden mutate
server state but emit no deltas, and the next visible add snapshots all current fields.

Each setter suppresses an equal value, otherwise mutates, marks dirty and—only while visible—builds
one delta and sends it to every audience member in set iteration order. Progress uses Java float
comparison (`+0` equals `-0`; NaN never equals, including NaN to NaN). Name uses component equality;
enums use identity. Color or overlay sends their combined current style. Changing any one of the
three booleans sends their combined current property byte. There is no dimension/range/tracking
gate, generation or response.

Primary publication anchor is `ServerBossEvent` and the six
`ClientboundBossEventPacket#create*Packet` factories.

## Waypoint publication and representation changes

Each server level's `ServerWaypointManager` relates tracked living transmitters to registered
players. Self-connections and first-tick sources are absent. Locator-bar gamerule must be enabled.
A spectator receiver is never range-rejected. Otherwise a spectator source or a receiver riding
the source is rejected; all other pairs require source-to-receiver distance strictly below the
minimum of source transmit-range and receiver receive-range attributes.

An admitted pair selects exactly one connection representation:

- distance greater than 332 uses azimuth: angle is `atan2` of the receiver-minus-source direction
  rotated clockwise 90 degrees;
- at distance at most 332 with the source chunk outside receiver chunk view, use chunk X/Z;
- otherwise use integer block position.

Connect sends track with the source UUID and an icon snapshot. Explicit icon style/color are kept;
an absent color is filled from current team color, with black mapped to `0xff303030`. Disconnect
sends canonical UUID/empty untrack. A representation transition sends a new track under the same
UUID, which replaces the client object without a preceding untrack.

A block connection sends update on any block-position change but is remade when Manhattan change
since its last sent position exceeds one. A chunk connection sends update on chunk change while
outside view, and is remade when chessboard change exceeds one or its last chunk becomes visible.
Azimuth sends update only when raw angular difference exceeds `0.008726646` radians and is remade
when distance becomes at most 332 or the source chunk becomes visible. Any pair becoming rejected
disconnects. Team changes remake relevant connections so track replaces the icon; locator-bar
disable disconnects/clears all and re-enable recreates connections for level players.

Track/untrack/update iteration follows server set/table iteration and carries no sequence. A delayed
old update can mutate a newer same-type replacement, a mismatched update only warns, and update
before track fails on the client. Ferrite retains normalized boss and locator intent plus authoritative
UUID/entity state, while operation ordinals, string/Either wrappers, client interpolation anchors,
icons, maps and renderer ordering remain adapter/client-local projection.

Primary publication anchors are `ServerWaypointManager`,
`WaypointTransmitter#doesSourceIgnoreReceiver`, `LivingEntity#makeWaypointConnectionWith`, and the
three entity connection implementations.

## C3 boss/waypoint fault and order boundary

Strict boss/type enum failures and malformed nested values fault decode; waypoint operation residues
always map by modulo three. Missing update targets fail during handling, while unknown removal and
waypoint type mismatch take their documented no-op/warn branches. Neither collection acknowledges
another packet; canonical publishers rely on add/track before deltas but install no generation or
reordering barrier.
