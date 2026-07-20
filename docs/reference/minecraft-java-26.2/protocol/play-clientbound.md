# C1-C3 Clientbound Play

This page source-specifies the clientbound packets used by the locked server to create the first
play-state client level and synchronize its initial connection projection, the C2 liveness,
disconnect, rotation, vehicle-correction, terrain, and block-convergence families, and the first C3
entity session and motion families. Entity spawn/state, inventory, chat, and later gameplay deltas
remain in their independently owned C3-C4 families.
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

This first C3 clientbound entity slice specifies six outcome/session packets. The next section
specifies motion and projectile acceleration independently. Entity spawn/removal, metadata,
attributes, equipment, passengers, effects, and explosions remain in separate recoverable families.

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
