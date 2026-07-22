# Blocks mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `BLK-JIGSAW-001` — Jigsaw edits synchronize a directed connector before optional immediate generation

**Parent:** `BLK-001`, `BLK-002`, `BLK-003`, `BLK-007`, `PLY-005`, `WGEN-003`, `CLI-005`,
`CLI-006`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked server and client classes, block and block-entity reports, protocol
owners and bundled client assets fix the jigsaw block's orientation, permission gate, editable
connector record, persistence, synchronization and edit-screen transaction. Immediate structure
expansion and placement remain owned by `WGEN-JIGSAW-CORE-001`; packet framing, decode faults and
serverbound dispatch admission remain owned by `PROTO-PLAY-SERVERBOUND-OPERATOR-BLOCKS-001`.

**Applies when:**

A jigsaw is placed, rotated, mirrored, used without an item, synchronized to a client, edited,
saved, loaded or asked to generate; the client validates or submits its edit screen; or the
worldgen connector matcher consumes its oriented record.

**Authoritative state:**

The block has one `orientation` property with twelve `FrontAndTop` values. Locked state IDs are:

| ID | Orientation | ID | Orientation |
|---:|---|---:|---|
| 21726 | `down_east` | 21732 | `up_south` |
| 21727 | `down_north` | 21733 | `up_west` |
| 21728 | `down_south` | 21734 | `west_up` |
| 21729 | `down_west` | 21735 | `east_up` |
| 21730 | `up_east` | 21736 | `north_up` (default) |
| 21731 | `up_north` | 21737 | `south_up` |

The first direction is the connector front and the second is its top. Placement sets front to the
clicked face. For an up/down face, top is the opposite of the placing context's horizontal
direction; for a horizontal face, top is up. Rotation and mirror apply their octahedral-group
transform to the complete front/top pair rather than rotating either direction independently.

The block is a full inherited cube with light-gray map color, correct-tool requirement, hardness
`-1`, blast resistance `3,600,000`, no loot table and no ticker. It belongs to both
`dragon_immune` and `wither_immune`. Generic game-master item placement and break rules remain in
`BLK-PLACE-001` and `BLK-BREAK-001`.

Its block entity has protocol ID `31` and seven editable fields:

- identifier `name`, default `minecraft:empty`;
- identifier `target`, default `minecraft:empty`;
- template-pool resource key `pool`, default `minecraft:empty`;
- arbitrary `final_state` string, default `minecraft:air`;
- joint `rollable` or `aligned`, constructor default `rollable`;
- signed integer `placement_priority`, default `0`;
- signed integer `selection_priority`, default `0`.

The setter methods only replace fields. They do not dirty the entity, validate registry membership,
send updates or normalize priorities.

**Transition and ordering:**

Placement chooses the complete front/top state before constructing the entity. An admitted local
use opens the screen from synchronized fields. Done sends one edit; Generate sends that edit,
closes, then sends generation. Server edit installs all seven fields, dirties, and queues the
same-state update in that order. A later generation request reads the then-current orientation,
pool and target before delegating all structure work. Save/load and update-tag paths serialize or
replace the seven fields independently of those live transactions.

**Placement and connector geometry:**

The block constructs exactly one matching jigsaw entity. `getFrontFacing` and `getTopFacing` read
the two directions from the live state. Connector attachment, owned by the jigsaw worldgen core,
requires opposite fronts, source target equal to candidate name, and either a rollable source joint
or equal rotated top directions; the candidate joint is not consulted.

The block's ordinary item is a `GameMasterBlockItem`. A nonnull player cannot obtain a placement
candidate unless both `instabuild` and `COMMANDS_GAMEMASTER` permission are present; a null player
bypasses this item-specific check. The separately owned break transaction rejects a
`GameMasterBlock` for the same missing permission before its ordinary unbreakable-state handling.

**Use and local screen opening:**

`useWithoutItem` first reads the block entity. A matching jigsaw entity plus
`canUseGameMasterBlocks` calls `player.openJigsawBlock` and returns `SUCCESS`; missing/wrong entity
or permission returns `PASS`. There is no reach, hand, hit-face, sneaking or side check inside this
hook; generic interaction owns whether it is called.

The base/server player implementation of `openJigsawBlock` does nothing. The local client-player
override immediately installs `JigsawBlockEditScreen` over the already synchronized client block
entity. No clientbound open-screen packet or menu is created. A server-side forged edit or generate
packet is independently rechecked for game-master permission.

**Client edit state and validation:**

Opening the screen copies all seven entity fields. It additionally starts generation levels at
`0` and `keep_jigsaws=true`. Name, target and pool boxes allow at most `128` characters and are the
only validity predicates: each must parse as an identifier. The final-state box allows `256`
characters and is not parsed client-side. Selection and placement boxes allow three characters
each, have no numeric filter, and parse as signed decimal integers at submission; any
`NumberFormatException` becomes zero.

The joint control contains rollable then aligned and starts from the synchronized entity value. It
is active and visible only when the live block state's front axis is vertical. A horizontal-front
entity can therefore retain and submit a preexisting rollable joint even though the control is
hidden.

The levels slider maps its normalized value through `floor(clampedLerp(0,20,value))`, so canonical
UI requests are `0..20`. The keep control starts on. These UI limits do not constrain forged packet
values: priorities and levels decode as unrestricted signed VarInts.

Done and Generate are active exactly while the three identifier fields are valid. Done, its button
and confirmation-key path send one set-jigsaw packet and close the screen. Generate performs Done
first, then sends a generate packet: the connection therefore observes the edit packet before the
generation packet even though the screen has already closed. Cancel, ordinary close and Escape
close without a packet. Resizing rebuilds widgets while preserving all typed fields, levels, joint
and the existing keep Boolean.

**Server edit transaction:**

After same-thread dispatch, an edit packet first requires `canUseGameMasterBlocks`. Denial silently
returns. The handler captures the live block state, reads the entity at the packet position and
continues only for a jigsaw entity; it does not additionally require the live block to be jigsaw.

An admitted entity receives fields in this order: name, target, pool key, final state, joint,
placement priority, selection priority. The handler then calls `setChanged` and
`sendBlockUpdated(position,capturedState,capturedState,3)`. Every field is installed before either
follow-up. Missing/wrong entities leave no residue. Identifier and packet-string decode failures
abort before dispatch; unknown joint text has already decoded as aligned. A syntactically valid but
absent pool remains a stored resource key.

This is a direct `ServerLevel.sendBlockUpdated` call rather than a `setBlock` transaction. The
method does not inspect its integer `3`: it queues the visible chunk's changed position, invalidates
the path-type cache, then sees equal old/new collision shapes and skips navigation recomputation.
The queued change projects the ordinary block state and entity update packet through the generic
block-convergence protocol. Setters invoked outside this handler have no such automatic dirtiness
or projection.

**Immediate generation boundary:**

After same-thread dispatch, generation repeats the same permission and matching-entity checks, then
passes the raw signed levels and keep Boolean to the entity. It does not dirty or update the entity.
The entity reads its current block-state front, uses the adjacent position in that direction,
requires the current template-pool key from the level registry, and calls the generic jigsaw core
with that pool holder, current target, raw levels, adjacent start and keep flag. Name, final state,
joint and both priorities are not direct root-call arguments. The Boolean generation result is
ignored; a missing configured pool fails at the required registry lookup before worldgen entry.

The canonical Generate button's preceding edit packet makes its newly typed pool, target and other
fields visible first. Forged or independently ordered packets instead consume whatever entity
fields are current when generation handles. The exact expansion, range, collision, RNG, piece
ordering, final-state replacement and partial world writes are specified by
`WGEN-JIGSAW-CORE-001`.

**Persistence and synchronization:**

Full save writes `name`, `target` and `pool` through their codecs; string `final_state`; enum
`joint`; and both integer priorities. Generic full metadata separately adds type/position and
components. Load replaces every field and uses independent defaults for missing or invalid values:

- identifiers become `minecraft:empty`;
- pool becomes the empty pool key;
- final state becomes `minecraft:air`;
- both priorities become zero;
- joint becomes `aligned` for a horizontal-front live state and `rollable` for a vertical-front
  state.

This joint load default differs from the unconditional constructor default. A newly placed
horizontal jigsaw begins rollable and normally saves that value, while a horizontal jigsaw loaded
from data with absent/invalid joint becomes aligned. Loading does not normalize an authored joint,
pool, final state or priority against the block orientation or configured registries.

`getUpdatePacket` creates the ordinary block-entity-data packet and `getUpdateTag` returns the
custom seven-field save without generic components or metadata. The client therefore receives the
editable record needed by its local screen. No block-entity renderer exists; visible orientation
uses the ordinary block-state model. The twelve locked asset variants rotate one cube-directional
jigsaw model to match the state table, and the item uses that same block model.

**Branches and aborts:**

All twelve states; every clicked face and horizontal placing direction; all rotations/mirrors;
matching/wrong/missing entity; client/server side; permission bits; valid/invalid/maximum-length
identifier, final-state and priority text; visible/hidden joint control; levels `0..20` and forged
signed values; Done/Generate/Cancel/close/confirmation/resize; packet ordering; every field value;
setChanged/update observation; configured/missing pool; success/empty/partial worldgen result; full
save, update tag and every missing/invalid field default.

**Constants and randomness:**

State IDs `21726..21737`; entity protocol ID `31`; identifiers/UI maximum `128`; final-state UI
maximum `256`; priority UI maximum three characters; canonical levels `0..20`; handler update flags
argument `3` (unused by that method); hardness `-1`; blast resistance `3,600,000`. Block, entity,
screen and handler transitions consume no RNG. Immediate generation delegates level RNG consumption
to the jigsaw core.

**Side effects:**

Local screen installation and closure; two serverbound operator-block packets; seven entity field
writes; chunk dirtiness; block-state/entity-data projection; required registry lookup; path-cache
invalidation; delegated structure piece and world writes. There are no inventory, redstone,
comparator, sound, particle, game-event, scheduled-tick or block-event effects in this subtype.

**Gates:**

Game-master item placement and use permission; generic interaction/break and packet-phase gates;
matching jigsaw entity; three client identifier parses; server packet decoding; configured pool for
generation. Difficulty and game rules do not gate jigsaw editing or immediate generation.

**Boundary cases and quirks:**

The edit screen is client-local rather than menu-opened. Horizontal joints can be invisibly
rollable. Constructor and missing-load joint defaults diverge. Client priority text is three
characters but server values are full signed integers. Generate submits edit before generation.
The handler accepts a jigsaw entity even under a non-jigsaw captured state. Field setters are inert
until an external caller dirties/projects. An absent stored pool survives edit/save but fails only
when generation performs its required lookup. Generation ignores its Boolean result.

**Evidence:**

`Confirmed`; `OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-REPORT-001`. Anchors:
`net.minecraft.world.level.block.JigsawBlock`,
`net.minecraft.world.level.block.entity.JigsawBlockEntity`,
`net.minecraft.world.level.block.entity.JigsawBlockEntity$JointType`,
`net.minecraft.client.gui.screens.inventory.JigsawBlockEditScreen`,
`net.minecraft.client.player.LocalPlayer#openJigsawBlock`,
`net.minecraft.server.network.ServerGamePacketListenerImpl#handleSetJigsawBlock`,
`net.minecraft.server.network.ServerGamePacketListenerImpl#handleJigsawGenerate`; locked block and
block-entity reports; bundled jigsaw blockstate/model/item assets; `EXP-BLK-021`.

**Test vectors:**

Exhaust the 12 state IDs and every placement/transform pair; permission, side and entity-type
branches; all UI lengths, identifier parses, numeric fallbacks, joint visibility, slider values,
resize and close actions; canonical and forged packet orders/values; exact field-write, dirty and
direct-update projection order; live-state mismatch; configured/missing pools and worldgen returns; full
save/update-tag data with every missing/wrong/invalid field. Run `EXP-BLK-021` as the executable
matrix and reuse `WGEN-JIGSAW-CORE-001` vectors for delegated world writes.
