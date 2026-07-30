# Play Clientbound Entity Spawn

`G01-P7-F005` implements both packets in
`PROTO-PLAY-CLIENTBOUND-ENTITY-SPAWN-001` for Minecraft Java 26.2:

| ID | Identity | Projection |
|---:|---|---|
| 1 | `minecraft:add_entity` | static type lookup, recreation and level insertion |
| 77 | `minecraft:remove_entities` | ordered relationship-aware level removal |

The add codec preserves signed entity/data values, the 128-bit UUID, three raw IEEE positions,
compact velocity and pitch/yaw/head-yaw bytes in source order. Its version-local registry contains
all 158 locked entity types. Negative and out-of-range type IDs follow the client registry fallback
to pig; encoding an identity outside that registry fails closed. Remove uses a signed count:
negative counts decode as an empty list when the body ends, while impossible positive counts,
truncation and residual bytes fault. Negative and duplicate entity IDs remain valid.

## Recreation and insertion

An add clears a matching former-local-player vehicle marker before construction, including when
construction later skips or faults. Player recreation requires a prior player-info entry for the
packet UUID. Other factories honor required-feature, peaceful-mode and availability admission; a
failed admission leaves any existing same-ID entity untouched.

Recreation finishes before same-ID replacement. This preserves the observed owner lookup behavior
where a projectile may resolve the old same-ID entity. Base recreation installs ID, UUID, position,
packet-position base and compact velocity. Nonliving entities retain raw position and body
rotation behavior. Living entities clamp X and Z to `+-30,000,000`, retain raw Y, clamp pitch to
`+-90`, and initialize body/head rotation from the packet head yaw. A remote player also initializes
its old pose to its current pose.

Insertion then discards the existing same-ID entity and installs the replacement. A duplicate UUID
owned by another ID refuses ID/UUID lookup registration but the entity remains present and tracked.
Player insertion records the seen-player history; removal deliberately does not erase that history
or player-info state.

## Type-specific spawn data

Item and glow frames map `abs(data % 6)` to down, up, north, south, west and east. Paintings use the
same mapping but reject vertical directions. Hanging entities derive their block anchor from the
integer containing position. Falling blocks retain state IDs `0..=32365`, fall back to air outside
that range and record their starting block position. Warden data one selects emerging.

Every projectile performs current-level owner lookup before insertion, including owner ID zero.
Fishing bobbers require that owner to be a player; otherwise they are marked discarded but still
pass through level insertion. Leash knots derive an anchor while their data has no extra meaning.
Dragon part IDs use wrapping increments one through eight. Shulker body rotation starts at zero.
Llama spit records the seven construction particle multipliers `0.4..=1.0`; llama spit and Shulker
bullets reapply packet movement after construction. Minecarts retain initial movement and install
their rolling sound, while bees install their constructor-selected nonaggressive flying sound.

## Removal and pairing

Removal processes IDs in packet order. Missing IDs are ignored and duplicates remove only their
first present occurrence. Before removing an entity that indirectly carries the local player, the
client retains that entity ID as the former vehicle. Each removal detaches vehicle/passenger
relationships, removes UUID and dragon-part lookup state, and tears down debug subscription state.
Those mutations affect every later ID in the same packet.

Pairing rejects self, failed broadcast admission, untracked chunks and horizontal distance beyond
the minimum of effective tracking and view distance. Effective tracking range starts from the
maximum range across the entity and all indirect passengers and then applies the configured scale.
The canonical pairing plan orders data synchronization, optional player-info publication, add,
present metadata/attributes/equipment, own/vehicle passengers and leash before bundle send and
`startSeenByPlayer`. Unpairing calls `stopSeenByPlayer` before a one-ID remove packet.

## Ownership and evidence

`ferrite-protocol::java_26_2::play::clientbound::entity_spawn` separates packet records, strict
codec, locked type registry, client recreation/removal projection and publication ordering.
Registry/type adaptation and client-visible caches stay version-local. Authoritative entity
lifecycle, passenger graphs, player information, tracking admission and Region placement remain in
their gameplay and runtime owners.

`crates/ferrite-protocol/tests/c3/play_clientbound_entity_spawn.rs` owns both goldens, the complete
158-ID registry sweep, fallback/error boundaries, signed and IEEE values, construction admission,
living/nonliving recreation, spawn-data specializations, same-ID/UUID behavior, ordered removal,
former-vehicle retention and pairing/unpairing order.
