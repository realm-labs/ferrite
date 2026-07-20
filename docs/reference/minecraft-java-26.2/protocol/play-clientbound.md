# C1-C2 Clientbound Play

This page source-specifies the clientbound packets used by the locked server to create the first
play-state client level and synchronize its initial connection projection, plus the C2 liveness,
disconnect, rotation, vehicle-correction, terrain, and block-convergence families. Entity,
inventory, chat, and later gameplay deltas remain in their independently owned C3-C4 families.
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
