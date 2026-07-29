# Minecraft 26.2 Required Clientbound Configuration Family

`G01-P3-F001` implements
`PROTO-CONFIGURATION-CLIENTBOUND-REQUIRED-001` in `ferrite-protocol`. It covers the nine
clientbound packets required for the C1 offline configuration path:

| Wire ID | Locked identity | Ferrite responsibility |
|---:|---|---|
| 1 | `minecraft:custom_payload` | Brand and bounded unknown-channel consumption |
| 2 | `minecraft:disconnect` | Trusted context-free component NBT |
| 3 | `minecraft:finish_configuration` | Terminal configuration-to-play handoff |
| 4 | `minecraft:keep_alive` | Exact signed-long echo |
| 5 | `minecraft:ping` | Exact signed-int echo |
| 7 | `minecraft:registry_data` | Ordered dynamic-registry reconstruction |
| 12 | `minecraft:update_enabled_features` | Connection-local feature set |
| 13 | `minecraft:update_tags` | Registry-indexed tag replacement |
| 14 | `minecraft:select_known_packs` | Ordered known-pack offer |

The adapter resolves IDs through the locked state/direction-local packet catalog. An ID absent from
the configuration/clientbound lane fails closed. A catalogued optional packet is reported as outside
this required family instead of being misdecoded as one of these packets.

## Value boundaries

Minecraft's `Identifier` grammar is owned by the versioned adapter. It intentionally is not the
storage-facing `ferrite_foundation::ResourceId`: 26.2 accepts empty and ambiguous path segments that
Ferrite persistence rejects. The wire type implements the exact lowercase namespace/path character
sets, the special `..` namespace rejection, and the default `minecraft` namespace.

Unnamed network NBT is retained byte-for-byte after structural validation. The scanner implements
all tag types, Java modified UTF, the locked depth of 512, and the source accumulator costs.
Registry entry NBT uses the 2,097,152-byte default quota. Trusted disconnect component NBT has no
heap quota but retains the depth bound. Component construction exposes a canonical literal form;
decoded component values must at least have a nonempty string/list/compound component root shape.

Counts have no invented protocol cap below the enclosing packet. Decode nevertheless proves that
each count can fit in the remaining packet bytes before iterating, never reserves from an untrusted
count, and remains bounded by the transport's 8,388,608-byte inflated packet ceiling.

The built-in `minecraft:brand` remainder is `UTF(32767)`. Unknown clientbound custom-payload
remainders are consumed only through the vanilla client's 1,048,576-byte cap and are deliberately
not retained or re-encodable.

## Connection-local projection

`ConfigurationProjection` is a headless-client oracle, not authoritative simulation state. It
enforces this sequence:

1. brand;
2. enabled features;
3. one known-pack offer and its response gate;
4. registry data in the locked 29-registry order, with later packets for the same registry
   appending IDs;
5. one tag update, with dynamic members checked against the matching reconstructed registry;
6. spawn readiness;
7. terminal finish, which installs clientbound play before emitting the configuration ACK.

Known packs, enabled features, brand, reconstructed IDs, and tags live only in this connection
projection. Duplicate registry element keys, backward registry order, duplicate tag keys, negative
or out-of-range dynamic tag members, and illegal stage transitions fail instead of being
normalized. Static registry sizes can be supplied to the projection so static tag members receive
the same exact range validation.

Keepalive and ping packets are legal while configuration work is pending. They produce exact echo
actions without changing task stage. Disconnect makes the projection terminal.

## Evidence

`crates/ferrite-protocol/tests/c1/configuration_clientbound_required.rs` owns the independent
conformance surface. It checks every locked C1 golden, exact packet IDs, compression envelopes,
round trips, NBT quota/depth and component boundaries, unknown custom-payload limits, malformed
counts and identifiers, registry append order, tag-to-registry mapping, feature filtering,
finish-before-ACK order, and liveness echoes.

Primary locked-source anchors are `ConfigurationProtocols`,
`ClientboundRegistryDataPacket`, `ClientboundUpdateEnabledFeaturesPacket`,
`ClientboundSelectKnownPacks`, `ClientboundUpdateTagsPacket`,
`ClientboundCustomPayloadPacket`, `ClientboundDisconnectPacket`,
`RegistrySynchronization.PackedRegistryEntry`, `TagNetworkSerialization.NetworkPayload`,
`FriendlyByteBuf`, `ByteBufCodecs`, `NbtIo`, `NbtAccounter`, and `Identifier`.
