# C1 Clientbound Play Entry

This page source-specifies the clientbound packets used by the locked server to create the first
play-state client level and synchronize its initial connection projection. Chunk, light, block,
entity, inventory, chat, and later gameplay deltas remain in their independently owned C2-C4
families. Every numeric registry value below is a wire projection derived from the configuration
registries; it is not an authoritative Ferrite identifier.

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
