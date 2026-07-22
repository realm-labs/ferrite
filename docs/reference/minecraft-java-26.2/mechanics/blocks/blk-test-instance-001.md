# Blocks mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `BLK-TEST-INSTANCE-001` — Test-instance blocks edit, place and project GameTest runs

**Parent:** `SIM-002`, `SIM-005`, `BLK-001`, `BLK-002`, `BLK-003`, `BLK-007`,
`PLY-005`, `WGEN-003`, `CLI-001`, `CLI-005`, `CLI-006`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked server and client classes fix the complete six-field record, operator
actions, structure geometry/capture/placement, GameTest-runner handoff, persistence, local edit
screen, status response, beam/bounds/error rendering and every visible mutation order. The selected
test instance's executable body remains validation infrastructure outside gameplay scope, while its
block-owned setup and result projection are exact behavior here.

**Applies when:**

A `minecraft:test_instance_block` is placed, used, queried, edited, reset, saved, exported, run,
loaded or rendered; its protocol-ID-46 entity resolves a configured test instance/template,
manipulates the test volume or receives GameTest status and positional failures.

**Authoritative state:**

The block has one property-free state, ID 21742. It keeps the default full collision cube but is
non-occluding and explicitly never view-blocking. It has destroy time -1, explosion resistance
3,600,000 and no loot table, and belongs to `dragon_immune` and `wither_immune`. Its epic stack-64
`GameMasterBlockItem` has no special data components. The block and item use the same single
cube-all texture model.

The entity owns a `Data` record containing optional test-instance resource key, raw `Vec3i size`,
extra rotation, `ignoreEntities`, status and optional error component. Fresh values are no test,
zero size, rotation none, `ignoreEntities=false`, status cleared and no error. A separate ordered
error-marker list contains absolute block positions and components. All six data fields and the
marker list are durable and synchronized.

Status order/wire IDs are cleared 0, running 1 and finished 2; invalid wire IDs fall back to
cleared. `withStatus` preserves test/size/rotation/ignore, replaces status and clears the error;
`withError` preserves the geometry fields, forces finished and installs the error. Neither helper
clears positional markers.

The sole state is also the complete `minecraft:test_instance` POI state, registered with zero
tickets and valid range 1. Test commands use that POI to find nearby or looked-at instances.

**Transition and ordering:**

Empty-hand use first requires a matching entity and `canUseGameMasterBlocks`. Denial returns PASS;
admission returns SUCCESS on both sides, and only the client directly opens the local edit screen.
There is no menu or clientbound open-screen packet.

Serverbound action handling first moves to the level thread, requires the same permission and then
a matching entity at the packet position. It imposes no block identity or reach check. Action wire
order is INIT, QUERY, SET, RESET, SAVE, EXPORT, RUN at 0..6; every other signed VarInt becomes INIT.
Every action always carries the complete data record: optional configured test key, three signed
VarInts for size, strict rotation, ignore-entities Boolean, fallback status and optional trusted
component. Transport imposes no size bounds.

INIT and QUERY never install the record. They resolve its optional key in the current configured
test-instance registry and send only the requester a positionless status packet containing the
test description or a red no-test component. QUERY additionally resolves the test's structure
template and includes its size when found; INIT always omits size. Missing registry entry and
missing template are distinct: an existing test can return its description without a size.

SET and the four mutation actions first install the packet's entire record. `set` immediately
dirties and sends a flags-3 update with AIR as old state and the entity's current block state as new.
SET then performs no action. RESET/SAVE/EXPORT/RUN invoke the corresponding operation. Every
mutation path finally sends another AIR-to-current-state flags-3 update, even after action failure.
Canonical UI packets always carry cleared/no-error state, so every UI mutation initially clears a
previous beam error but leaves positional markers unless its selected operation explicitly clears
them. Forged status/error values are retained by SET, SAVE, EXPORT and failed RUN paths.

**Test registry, geometry and placement:**

A present test holder supplies structure identifier, intrinsic rotation, padding, sky-access and
required/optional status. Effective rotation is intrinsic rotation composed with the stored extra
rotation. Quarter turns swap X/Z for the transformed size. With padding `p`, structure position is
block position plus `(p,p+1,p+1)`; its inclusive box ends at transformed size minus one. The test
box inflates that structure box by `p`, and the permission-gated render box uses the same local
offset and transformed size.

The placement start corner uses the untransformed stored size: none adds zero, 90 adds
`(sizeZ-1,0,0)`, 180 adds `(sizeX-1,0,sizeZ-1)`, and 270 adds `(0,0,sizeX-1)`. Placement first
permanently force-loads every chunk intersecting the structure box. It then clears the entire test
box with AIR using flags 818 plus an explicit neighbor update per cell, clears scheduled block ticks
and block events in that box, and discards every non-player entity there. The entity performs a
second non-player discard query, then places the template with effective rotation, stored
ignore-entities, known-shape true, the start corner as origin and pivot, level RNG and flags 818.
This class never releases the forced chunks.

Canonical size editors clamp every axis to 1..48, but wire/persistence do not. Zero, negative and
large forged sizes therefore flow into normalized bounding boxes, clearing, force-loading, capture,
start-corner offsets and rendering without a handler clamp. Template placement itself still uses
the resolved template's content rather than clipping it to the stored size.

**Reset, save and export:**

RESET first removes only barrier blocks from the computed one-block boundary, clears positional
markers, and attempts placement. Successful placement sends a green reset-success message; a
missing template sends no reset failure. It then forces status cleared/no-error whether placement
succeeded or not.

SAVE chooses the registered test's structure identifier when its holder resolves; otherwise it
uses the optional test key's own identifier. No key sends a red unable-to-save message with block
coordinates. A key captures at the computed structure position with raw stored size, stored entity
inclusion, empty author, disk save enabled and AIR plus STRUCTURE_VOID ignored. The underlying
Boolean capture/save result is ignored, so a create or disk failure produces no SAVE response and
the method still returns the chosen identifier.

EXPORT always runs SAVE first. With an identifier it asks the structure manager for its optional
test-template output path. Disabled export, missing cached template and caught file-save failure
each send a red literal message and return true; successful text-structure export announces the
absolute path and returns false. This inverted result is ignored by the packet handler. Path
validation occurs before the caught file-save block. The release client hides the Export button,
but a forged packet can still invoke this server path.

**RUN and GameTest convergence:**

RUN with no registered holder sends a red no-test message. A holder whose template cannot be
resolved sends red no-test-structure. Both paths retain old positional markers. A successful path
first performs the placement transaction using the just-installed packet data, then clears markers,
clears the process-global `GameTestTicker.SINGLETON`, forgets all tracked failed tests and announces
the registered test name to the requester.

It creates one no-retry `GameTestInfo` using the packet's extra rotation and this block position.
Starting that runner uses the in-place spawner, which replaces the block at the same position with
a fresh test-instance block/entity. The replacement data uses the registered key, actual template
size or `(1,1,1)` fallback, the packet extra rotation, `ignoreEntities=false`, cleared and no error.
It places the structure a second time, encases its boundary in barriers except where any test-
instance block already exists, clears test-box block ticks/events again, and starts after the test's
setup delay. Thus successful RUN does not retain the packet entity object, raw size, ignore flag,
status, error or markers; the fresh runner-owned entity becomes authoritative.

The boundary is one block outside the structure on four walls and floor; the ceiling is included
unless the test declares sky access. On execution start the entity becomes running. A reported pass
becomes finished with no error; a reported failure becomes finished with its component, and an
absolute-position assertion additionally appends a positional marker. Pass cleanup discards every
non-player entity within the structure bounds inflated by one. Result reporting broadcasts its
ordinary messages to all players. The configured test body, environment and assertion schedule
remain delegated to the GameTest owner.

**Persistence and synchronization:**

Save always writes the required `data` codec and writes `errors` only when markers exist. The data
codec requires size, rotation, ignore-entities and status; test and error are optional. Load installs
the data only when the whole record decodes, otherwise retaining the constructed/current record. It
always clears markers and then installs the decoded marker list or empty default. Installing decoded
data uses the ordinary `set` path, including dirty/update behavior when a server level is already
attached.

Every `set`, status change and marker mutation calls the overridden `setChanged`; it performs
ordinary chunk dirtiness and, on a server level, an AIR-to-current-state flags-3 update. Clearing an
already empty marker list is the only marker operation that does not update. The update packet is
always present and its custom-only tag carries the complete data and marker records.

**Client screen and status response:**

The non-menu in-game screen has a 128-code-unit ID box, three 15-code-unit size boxes, all four
rotations, an include-entities toggle and Reset/Save/Run/Done/Cancel. Export appears only in IDE
builds. Opening immediately sends INIT. Every ID edit sends QUERY even when parsing fails; invalid
text also installs a local red invalid-ID description before the later server response can replace
it. Optional QUERY size overwrites all three current size strings.

The rotation control initializes from the entity's effective rotation, not its stored extra
rotation, so reopening and submitting a test with intrinsic rotation can compose it again. Save and
Export are active only for a valid identifier and selected rotation none; the server does not
enforce either UI restriction. Include-entities stores the inverse ignore flag. Size parsing clamps
to 1..48 and defaults invalid text to 1.

Reset/Save/Export/Run send once and close. Done sends SET and closes. Cancel/Escape/ordinary close
send nothing, because screen fields never mutate the local entity. Each outgoing UI record is
cleared/no-error. Clientbound status has no block position or request sequence; after main-thread
handoff it updates whichever test-instance screen is currently open. A delayed response from an
older screen or block can therefore overwrite the current description/size. The description
prefixes any error currently present on the open screen's synchronized entity, not packet-carried
error state.

**Client block-entity rendering:**

Cleared yields no beam. Running yields one opaque gray beam section. Finished without error yields
green; finished with error yields red when the resolved test is required (or missing) and orange
when it is optional. The single final section renders to height 2048 using ordinary beacon animation,
distance scaling and horizontal render-distance admission. Beam visibility is not permission-gated.

The always-BOX boundary renderer is visible only to a game-master-capable local player or spectator,
requires all transformed size axes at least one, and draws the ordinary opaque light-gray outline.
It renders no invisible-block cells. Error markers are copied and rendered without the bounds
permission gate: each absolute position receives a red 0.375-alpha filled cube inflated by 0.02 and
white centered always-on-top billboard text at scale 0.16. The combined renderer is offscreen-
eligible and admits either the beacon's horizontal effective-render-distance test or the box
renderer boundary.

**Branches and aborts:**

Permission/entity/side; all seven actions plus invalid action; optional/missing test and template;
raw versus UI size; all rotations and intrinsic composition; required/optional/absent holder;
marker-empty/nonempty; cache/disk/export/path outcomes; placement clearing/entity/chunk effects;
runner replacement and every result; malformed/absent data/errors; current/stale status response;
render permission/status/error/size/distance are distinct observable boundaries.

**Constants and randomness:**

State 21742; entity protocol ID 46; action IDs 0..6; status IDs 0..2; UI ID 128, size text 15 and
size clamp 1..48; structure offset `(0,1,1)`; block strength `(-1,3,600,000)`; placement flags 818;
updates 3 with AIR old state; marker padding 0.02, alpha 0.375, text height 1.2/scale 0.16; beam final
height 2048. Template placement consumes the live level RNG; other block-owned operations do not
create an RNG.

**Side effects:**

Local screen and two protocol directions; data/marker dirtiness and block-entity projection;
permanent forced chunks; exhaustive block replacement, neighbor updates, tick/event clearing,
non-player discard and template placement; cache/disk/text export; barrier shell; global GameTest
ticker/failed-tracker reset; runner/environment/test result; requester and broadcast messages;
beam, bounding-box and error-marker rendering.

**Gates:**

Game-master permission; matching entity; action; configured test holder/template; side and manager;
path/export availability; effective rotation/padding/sky-access/required fields; UI parsing; runner
and chunk readiness; client screen identity; render permission/status/geometry/distance.

**Boundary cases and quirks:**

Queries are positionless and unsequenced. Mutation fields commit before validation. `setChanged`
already updates, so the handler duplicates publication. SAVE ignores capture failure; EXPORT returns
true on failure and false on success. Forged sizes are unbounded. Forced chunks are never released.
Successful RUN places twice, clears global GameTest state, replaces its own entity and forces entity
inclusion on the runner copy. Save/export/set may leave old positional markers beside cleared data.
Effective rather than extra rotation seeds the UI. Beam and marker visibility are not operator-only.

**Evidence:**

`OFF-SERVER-001`, `OFF-CLIENT-001`, `OFF-REPORT-001`, `OFF-DATA-001`;
`net.minecraft.world.level.block.TestInstanceBlock#useWithoutItem`,
`net.minecraft.world.level.block.entity.TestInstanceBlockEntity#set`,
`net.minecraft.world.level.block.entity.TestInstanceBlockEntity#getStructureSize`,
`net.minecraft.world.level.block.entity.TestInstanceBlockEntity#getStructureBoundingBox`,
`net.minecraft.world.level.block.entity.TestInstanceBlockEntity#getRotation`,
`net.minecraft.world.level.block.entity.TestInstanceBlockEntity#setChanged`,
`net.minecraft.world.level.block.entity.TestInstanceBlockEntity#loadAdditional`,
`net.minecraft.world.level.block.entity.TestInstanceBlockEntity#saveAdditional`,
`net.minecraft.world.level.block.entity.TestInstanceBlockEntity#resetTest`,
`net.minecraft.world.level.block.entity.TestInstanceBlockEntity#saveTest`,
`net.minecraft.world.level.block.entity.TestInstanceBlockEntity#exportTest`,
`net.minecraft.world.level.block.entity.TestInstanceBlockEntity#runTest`,
`net.minecraft.world.level.block.entity.TestInstanceBlockEntity#placeStructure`,
`net.minecraft.world.level.block.entity.TestInstanceBlockEntity#processStructureBoundary`,
`net.minecraft.world.level.block.entity.TestInstanceBlockEntity#getBeamSections`,
`net.minecraft.server.network.ServerGamePacketListenerImpl#handleTestInstanceBlockAction`,
`net.minecraft.client.gui.screens.inventory.TestInstanceBlockEditScreen#init`,
`net.minecraft.client.gui.screens.inventory.TestInstanceBlockEditScreen#sendToServer`,
`net.minecraft.client.gui.screens.inventory.TestInstanceBlockEditScreen#setStatus`,
`net.minecraft.client.multiplayer.ClientPacketListener#handleTestInstanceBlockStatus`,
`net.minecraft.client.renderer.blockentity.TestInstanceRenderer#extractRenderState`,
`net.minecraft.client.renderer.blockentity.TestInstanceRenderer#submit`,
`net.minecraft.gametest.framework.GameTestInfo#prepareTestStructure`,
`net.minecraft.gametest.framework.ReportGameListener#reportPassed`,
`net.minecraft.gametest.framework.ReportGameListener#reportFailure`;
`reports/blocks.json#minecraft:test_instance_block`,
`reports/registries.json#minecraft:block_entity_type/minecraft:test_instance_block`,
`reports/minecraft/components/item/test_instance_block.json`,
`assets/minecraft/{blockstates,models/block,items}/test_instance_block.json`,
`data/minecraft/test_instance/always_pass.json`.

**Test vectors:**

Run `EXP-BLK-028` across permission/entity/action, registry/template and every raw/UI geometry value;
capture setter/update/message order, reset/save/export outcomes, double placement/replacement,
forced chunks, clear/discard/barrier effects and GameTest status/marker convergence. Reload every
data/error record and drive current/stale UI responses plus beam/bounds/marker rendering at all
gates. Reuse the completed operator-block protocol vectors for exact bytes.
