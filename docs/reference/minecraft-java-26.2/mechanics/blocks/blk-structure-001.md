# Blocks mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `BLK-STRUCTURE-001` — Structure blocks edit, cache, persist and project bounded template operations

**Parent:** `BLK-001`, `BLK-002`, `BLK-003`, `BLK-007`, `PLY-005`, `PLY-006`,
`RED-001`, `WGEN-003`, `WGEN-005`, `CLI-001`, `CLI-005`, `CLI-006`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked server source fixes the four block states, operator admission, complete
editable record, packet mutation/action order, corner detection, template-manager save/load/remove,
redstone edge actions and persistence. Locked client source and assets fix the edit screen,
mode-specific models and permission-gated boundary/invisible-block rendering. Template capture and
placement internals remain delegated to the existing structure-template/worldgen owners.

**Applies when:**

A `minecraft:structure_block` is placed, used, edited through its operator packet, redstone-powered,
saved/loaded or rendered; it scans matching corner blocks, captures/places/removes a named structure
template, or synchronizes block-entity protocol ID 21.

**Authoritative state:**

The block owns `MODE={save,load,corner,data}`, states 21722..21725, with load/state 21723 default.
It is a full collision/occlusion cube with light-gray map color, default HARP/stone properties,
correct-tool requirement, destroy time -1, explosion resistance 3,600,000 and no loot table. Its
epic, stack-64 `GameMasterBlockItem` retains generic placement/break permission semantics.

The exact structure-block entity owns nullable parsed `structureName`; strings `author` and
`metaData`; offset `structurePos`; nonnegative `structureSize`; mirror, rotation and mode;
`ignoreEntities`, `strict`, `powered`, `showAir`, `showBoundingBox`; float `integrity`; and long
`seed`. Fresh defaults are null name, empty strings, offset (0,1,0), zero size, no mirror/rotation,
mode from the block state, true ignore-entities/show-box, false strict/powered/show-air, integrity
1 and seed 0.

**Transition and ordering:**

Empty-hand use requires the exact block entity and `canUseGameMasterBlocks`. Rejection returns
PASS. Admission returns SUCCESS on both sides, but only the client opens the local structure screen.
Server placement with a nonnull living placer copies that entity's plain-text name to `author`;
the result is not independently dirtied or projected.

The serverbound operator handler first enforces main-thread execution and the same game-master
permission, then requires the block entity at the supplied position. It snapshots the pre-edit
block state and mutates, in order: mode, name, offset, size, mirror, rotation, metadata,
ignore-entities, strict, show-air, show-box, integrity and seed. Setting mode immediately offers a
flags-2 state write if the live block is still a structure block; its result is ignored. Other
setters are plain field writes.

The packet clamps each signed-byte offset to -48..48, each signed-byte size to 0..48 and integrity
to 0..1 on decode; metadata is UTF with a 128-character bound, while the ordinary name UTF bound
is the protocol default. Its final flag byte maps ignore-entities/show-air/show-box/strict to bits
1/2/4/8. Enum wire order is UPDATE_DATA, SAVE_AREA, LOAD_AREA, SCAN_AREA; modes are save, load,
corner, data; mirrors are none, left-right, front-back; rotations are 0, 90, 180, -90.

An empty or invalid identifier becomes null. A nonnull name then selects one action:

- UPDATE_DATA performs no template action.
- SAVE_AREA calls the disk-saving SAVE-mode path and reports save success/failure.
- LOAD_AREA first tests LOAD-mode availability. Missing reports not-found. Equal template/current
  size places and reports success; unequal size copies template author/size, dirties, performs no
  placement and reports prepare, so a later equal-size request can place.
- SCAN_AREA runs the SAVE-mode corner scan and reports size success/failure.

A null name skips action dispatch and reports invalid-name with the raw packet name. Regardless of
action outcome, the accepted exact-entity path then dirties the entity and calls flags-3
`sendBlockUpdated` with the same captured old state as both old and new arguments. Thus mode may
already differ in the live world, and successful size scan additionally performed its own dirty
plus flags-3 update before this final duplicate publication.

**Corner scan:**

Detection requires SAVE mode. It scans every position from X/Z block position minus 80 and level
minimum Y through X/Z plus 80 and level maximum Y, retaining exact structure blocks whose exact
entity is CORNER mode and whose nullable parsed name equals the SAVE block's name. No corner fails.
One corner is enclosed together with the SAVE block; two or more enclose only the corners.

Let the inclusive corner-box coordinate deltas be dx/dy/dz. All three must exceed 1. Success sets
offset to `(min - savePos + 1)` and size to `(dx-1,dy-1,dz-1)`, dirties, then sends a same-state
flags-3 update. There is no explicit 48-axis clamp here; later packet round trips and persisted
loads clamp, but the direct detected live record can exceed that bound.

**Template save, load and redstone actions:**

SAVE requires SAVE mode, nonnull name and a server level. Its capture origin is block position plus
offset. The template manager `getOrCreate` failure returns false. Capture uses the stored size,
includes entities exactly when `ignoreEntities=false`, and ignores `structure_void` plus any
explicit caller ignore list; it then writes author. The UI action requests manager disk save and
returns its result, treating identifier/path failure as false. The rising-edge redstone SAVE action
uses `saveStructure(false)`: it refreshes the manager's in-memory template but performs no disk save.

LOAD resolves the named template. `loadStructureInfo` replaces author with the nonempty template
author or empty string, replaces size and dirties. Placement settings apply mirror, rotation,
ignore-entities and `knownShape=strict`. Integrity below 1 installs one `BlockRotProcessor` with
integrity clamped to 0..1 and its own RNG. Template placement starts at block position plus offset,
uses that same position as pivot, then receives another independently created RNG and flags 2 or
818 when strict. Seed 0 initializes each RNG from current milliseconds; nonzero seed initializes
both independently to the same seed. The release debug-structure-void prefill branch is disabled.
The delegated template owner controls processed block/entity order, neighbor consequences and
partial write failure.

CORNER rising-edge redstone removes the named template from the server manager cache. DATA does
nothing. LOAD rising-edge redstone resolves and places immediately without the UI's availability/
same-size prepare gate. Neighbor callbacks are server-only, read `hasNeighborSignal(pos)` only and
compare it with the entity's persisted latch. Rising writes true before dispatch; falling writes
false; neither latch write dirties, updates block state nor schedules work. Repeated powered
callbacks therefore do not retrigger.

**Persistence and synchronization:**

Save always writes name (empty for null), author, metadata, three offset ints, three size ints,
rotation, mirror, mode, five booleans, integrity and seed. Load defaults as for a fresh entity,
except invalid/missing mode explicitly becomes DATA; it clamps offsets to -48..48 and sizes to
0..48, accepts integrity/seed without clamping, then offers a flags-2 live state-mode write when
the level and exact block exist. The block-entity update packet is always created and its update
tag is the complete custom-only record.

The saved template manager and this block-entity record are distinct continuity domains: SAVE can
mutate only the memory cache, disk save can fail after capture, CORNER can evict the cache without
changing the stored template resource, and chunk/entity saving follows ordinary dirty admission.

**Client UI and projection:**

Opening snapshots mirror, rotation, mode and four booleans. Cancel/close restores only those local
fields, including a local flags-2 mode write, and sends no packet; typed name, offsets, size,
integrity, seed and metadata were never applied locally. Done/Enter sends UPDATE_DATA and closes.
Mode-appropriate Save/Load/Detect buttons send their action and close. The non-pausing in-game UI
shows SAVE fields for name/offset/size/entities/air/save/detect; LOAD for name/offset/integrity/
seed/entities/strict/mirror/rotation/box/load; CORNER for name; DATA for metadata. The mode selector
normally omits DATA but exposes all modes when the client permits alternate values. Invalid local
coordinates parse as 0, integrity as 1 and seed as 0; name/data max length is 128, coordinate/
integrity 15 and seed 31.

World block models are full cubes selected solely by MODE with save/load/corner/data textures.
The item uses the neutral structure-block cube. The block-entity renderer is visible only to a
game-master-capable local player or spectator, renders offscreen through distance 96 and requires
all rendered size axes at least 1. SAVE plus show-air selects box-and-invisible-blocks; SAVE without
show-air and LOAD with show-box select box; LOAD without show-box and CORNER/DATA select none.
The outer box uses the mirrored/rotated exact corners from the entity and an opaque light-gray
stroke. Invisible scanning visits every cell in the rendered volume and marks air, structure void,
barrier and light as blue, pale-red, red and yellow center boxes respectively.

**Branches and aborts:**

Wrong permission/entity/block, invalid name, action/mode mismatch, scan corner count/extent,
manager lookup/create/save failure, missing template, unequal size, integrity threshold, redstone
edge/mode and client render permission/mode/size are distinct boundaries. The accepted packet
commits every field before action validation and never rolls those edits back after action failure.

**Constants and randomness:**

Four states, protocol ID 21, offsets/sizes 48, scan radius 80, name/data UI bound 128, integrity
0..1 on wire, redstone save-to-disk false, state flags 2, final updates 3, placement flags 2/818,
render distance 96 and minimum visible axes 1 are fixed. Only integrity processing/template
placement use RNG; seed 0 is millisecond-derived and creates two potentially distinct streams.

**Side effects:**

Mode block writes; author and editable-record mutation; dirty/update-tag publication; local UI
mutation and one operator packet; private system messages; exhaustive corner reads; template cache,
disk capture/save/load/remove; delegated world/entity placement; redstone latch; client cube,
boundary and invisible-block render submissions.

**Gates:**

Game-master permission; exact block/entity; side; parsed name; update type and mode; bounded decoded
fields; matching corners and extents; server/template manager/resource availability; same-size UI
load; integrity/strict/ignore settings; neighbor edge; client game-master-or-spectator visibility.

**Boundary cases and quirks:**

Packet fields commit before invalid-name/action failure. Successful scan publishes twice. Detected
size is not immediately clamped. Redstone SAVE is memory-only, LOAD skips UI preparation, CORNER
evicts cache and DATA is inert. The power latch is persisted but redstone changes do not dirty it.
Load-time invalid mode becomes DATA and may rewrite the live block state. Seed 0 can yield different
processor and placement RNG seeds. Spectators can see bounds despite lacking edit permission.

**Evidence:**

`OFF-SERVER-001`, `OFF-CLIENT-001`, `OFF-REPORT-001`;
`net.minecraft.world.level.block.StructureBlock#useWithoutItem`,
`net.minecraft.world.level.block.StructureBlock#setPlacedBy`,
`net.minecraft.world.level.block.StructureBlock#neighborChanged`,
`net.minecraft.world.level.block.StructureBlock#trigger`,
`net.minecraft.world.level.block.entity.StructureBlockEntity#saveAdditional`,
`net.minecraft.world.level.block.entity.StructureBlockEntity#loadAdditional`,
`net.minecraft.world.level.block.entity.StructureBlockEntity#usedBy`,
`net.minecraft.world.level.block.entity.StructureBlockEntity#detectSize`,
`net.minecraft.world.level.block.entity.StructureBlockEntity#saveStructure`,
`net.minecraft.world.level.block.entity.StructureBlockEntity#placeStructureIfSameSize`,
`net.minecraft.world.level.block.entity.StructureBlockEntity#loadStructureInfo`,
`net.minecraft.world.level.block.entity.StructureBlockEntity#placeStructure`,
`net.minecraft.world.level.block.entity.StructureBlockEntity#unloadStructure`,
`net.minecraft.world.level.block.entity.StructureBlockEntity#renderMode`,
`net.minecraft.world.level.block.entity.StructureBlockEntity#getRenderableBox`,
`net.minecraft.server.network.ServerGamePacketListenerImpl#handleSetStructureBlock`,
`net.minecraft.network.protocol.game.ServerboundSetStructureBlockPacket#write`,
`net.minecraft.client.gui.screens.inventory.StructureBlockEditScreen#init`,
`net.minecraft.client.gui.screens.inventory.StructureBlockEditScreen#sendToServer`,
`net.minecraft.client.gui.screens.inventory.StructureBlockEditScreen#onCancel`,
`net.minecraft.client.renderer.blockentity.BlockEntityWithBoundingBoxRenderer#extract`,
`net.minecraft.client.renderer.blockentity.BlockEntityWithBoundingBoxRenderer#submit`;
`reports/blocks.json#minecraft:structure_block`,
`reports/registries.json#minecraft:block_entity_type/minecraft:structure_block`,
`reports/minecraft/components/item/structure_block.json`,
`assets/minecraft/{blockstates,models/block,items}/structure_block*.json`.

**Test vectors:**

Use `EXP-BLK-027` and the completed operator-block protocol vectors. Cross permission/side/entity,
all modes/actions/names/bounds, single/multiple/no corners, memory/disk manager results, missing/
unequal/equal templates, integrity/seed/strict/entity branches, redstone edges, save/load defaults,
UI cancel/submit and every render gate. Assert mutation/message/dirty/update/cache/world order and
the absence of rollback after accepted failures.
