# Blocks mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `BLK-SIGN-001` — Signs separate support, two-sided text, one editor, applicators, clicks and rendering

**Parent:** `SIM-003`, `BLK-001`, `BLK-002`, `BLK-003`, `BLK-007`, `PLY-005`,
`PLY-INTERACT-001`, `ITM-003`, `ITM-USE-001`, `CLI-001`, `CLI-005`, `CLI-006`,
`CLI-UI-001`, `CLI-EFFECT-001`, `ENV-FLUID-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked server and client classes, block/item/registry reports, bundled sign
tags, loot tables and models, and the completed sign-update/open-editor protocol families determine
all placement, support, side selection, editing, filtering, applicator, click-action, persistence,
synchronization and renderer branches. Generic block placement/breaking, packet framing, text
filter execution and ordinary sound/particle transport remain with their existing owners.

**Applies when:**

Any of the 12 wood types is placed or updated as a standing sign, wall sign, ceiling hanging sign
or wall hanging sign; either sign block-entity type is edited, clicked, dyed, made glowing,
darkened or waxed; a hanging-sign item chains from another sign; or either side is saved,
synchronized or rendered.

**Authoritative state:**

The locked block set is 48 IDs: four forms for oak, spruce, birch, acacia, jungle, dark oak, pale
oak, crimson, warped, mangrove, bamboo and cherry. Standing forms have `rotation=0..15` plus
`waterlogged` for 32 states each; wall forms have four horizontal `facing` values plus
`waterlogged` for eight; ceiling hanging forms add boolean `attached` to rotation and waterlogging
for 64; wall hanging forms have the wall form's eight. This is 1,344 registered states. Ordinary
and hanging block-entity protocol IDs are 7 and 8. Only standing and ceiling-hanging forms have
items; all 24 items stack to 16 and place their paired wall alternative when selected by the
generic standing/wall placement pipeline.

Each entity owns immutable `frontText` and `backText`, each exactly four raw components, four
filtered components, one dye color and one glowing flag. New text is four empty components, black
and non-glowing. The entity also owns boolean `isWaxed=false` and a nullable authorized-editor
UUID. The two texts and wax flag persist and synchronize; editor authorization is transient.
Ordinary signs use line height 10 and maximum line width 90 pixels; hanging signs override these
with 9 and 60.

**Transition and ordering:**

Placement selects form, orientation, support relation and source fluid before the generic block
transaction installs state; placement-time block-entity tag handling then either suppresses or
opens editing. Neighbor updates schedule water work before the form-specific survival result.
Interactions select side, run the held-item or empty-hand branch, and commit applicator/click/edit
effects in the exact orders below. Entity ticks validate the transient editor lease, accepted
submissions rebuild text before clearing it, persistence reconstructs both sides before ordinary
update projection, and clients render only the synchronized snapshot plus their local editor
preview.

**Placement, shape and support:**

All forms are simple-waterlogged. A shape update on a waterlogged state schedules a water-fluid
tick before the block-specific support result continues. The shared standing/ceiling fallback
shape is a centered eight-pixel-wide column from Y 0 through 16. A wall sign instead uses its
facing-rotated 16-by-8-pixel board spanning Y 4.5..12.5 at Z 14..16; the board bounds' center is
also the side-selection center. A ceiling hanging sign uses a centered ten-pixel column when its
16-way rotation is not cardinal and a rotated 14-pixel-wide, two-pixel-thick shape from Y 0..10
when cardinal. A wall hanging sign joins a 16-by-4-pixel plank at Y 14..16 with a 14-pixel-wide
center member from Y 0..10; its collision shape is the plank alone. Wall hanging signs are never
pathfindable. Every sign state permits respawning inside it.

A standing sign defaults to rotation 8/non-waterlogged. Placement sets
`rotation=segment(playerRotation+180)` and reads source-water at the target. It survives exactly
while the block below reports `isSolid()`, and a failing DOWN-neighbor update replaces it with air.
A wall sign defaults north/non-waterlogged, scans the context's nearest-looking directions in
order, skips vertical directions, faces opposite the first horizontal candidate whose backing
neighbor reports `isSolid()`, then copies source-water; no candidate returns null. Only an update
from the backing direction can remove it.

A ceiling hanging sign defaults rotation 8, `attached=false`, non-waterlogged and survives when
the block above has a sturdy DOWN face at support type `CENTER`. Placement starts `attached=true`
when the above collision face is not full or secondary-use is active, otherwise false. When the
above block is in reloadable `all_hanging_signs` and secondary-use is not active, an axis-aligned
wall hanging sign above, or a cardinal ceiling sign above, forces false when its orientation axis
contains the player's horizontal direction. False attachment uses the segment of the opposite
player direction; true uses `segment(playerRotation+180)`. Only an UP-neighbor support failure
removes it. Attachment projects as `CEILING_MIDDLE` for true and `CEILING` for false.

A wall hanging sign defaults north/non-waterlogged and scans nearest-looking directions in order.
It accepts only horizontal directions whose axis differs from the clicked face's axis, faces
opposite that candidate, and requires both generic survival and `canPlace`. `canPlace` succeeds
when either clockwise or counterclockwise side can attach. A neighboring wall hanging sign can
support it only when the neighbor's facing axis contains this sign's facing direction; any other
neighbor must expose a sturdy `FULL` face toward this sign. No candidate returns null. Later only
a neighbor update on the axis perpendicular to facing can recheck and remove it. Its attachment
projection is `WALL`.

**Placement-time editing:**

After a sign item places through the generic block-item transaction, it first delegates custom
block-entity-tag application. On the server, if that delegation did not apply tag data, the player
exists, and the placed block/entity still match a sign, it authorizes that player and opens the
front editor. A successfully applied custom block-entity tag suppresses automatic editing. The
hanging-sign item additionally delegates placement admission to `WallHangingSignBlock.canPlace`
for its wall alternative.

**Side selection and one-editor lease:**

For any interaction, the sign computes a horizontal angle from its form-specific hitbox center to
the player. It selects front when the absolute wrapped difference from the block's Y rotation is
at most 90 degrees; the exact perpendicular boundary is front. A non-sign current block state
selects back. Text filtering selects the raw or filtered visible component set independently from
front/back selection.

Opening sets the player's UUID before sending the editor-open packet. A nonnull different UUID
blocks both item applicators and empty-hand editing; the same player is not considered another
editor. Each admitted entity tick clears authorization when that UUID no longer resolves to a
player in the level or the player fails `isWithinBlockInteractionRange(pos,4.0)`. Unloaded time has
no catch-up. The UUID is neither saved nor included in update data.

**Held-item interaction and applicators:**

Wrong or missing block entity returns `PASS`. A held item becomes an applicator candidate only
when it implements `SignApplicator` and the player may build. The client immediately returns
`SUCCESS` for a candidate or a waxed sign, otherwise `CONSUME`; authoritative server processing
follows. The server declines to the empty-hand path unless the candidate is unblocked by wax and
another editor. It then chooses the facing side, calls the applicator's admission and mutation,
and treats false/no-change as `TRY_WITH_EMPTY_HAND`.

The 16 dye items require at least one nonempty visible line and a `DYE` component whose color
differs from the side; success replaces the side color and plays the dye-use block sound at
volume/pitch 1. Glow ink requires nonempty visible text and changes false to true, playing its use
sound. Ink requires nonempty visible text and changes true to false, playing its use sound.
Honeycomb alone overrides the nonempty-text gate: it changes `isWaxed` false to true even on an
empty sign and emits level event 3003. Its non-sign copper mapping belongs to `ITM-HONEYCOMB-001`.

After any successful applicator mutation, the block executes click actions on the now-mutated
visible side, awards the held item's used stat, emits `BLOCK_CHANGE` with the player and current
sign state, consumes one with the living-entity-aware `consume` helper, and returns `SUCCESS`.
Thus infinite-material players keep the applicator item, and a preserved click style can execute
in the same interaction after recoloring, glow change or waxing.

**Empty-hand interaction and click actions:**

The empty-hand path is server-only. It selects a side and executes all supported click actions
before testing wax or editability. Lines are visited in order. `RUN_COMMAND` executes with the
player as entity/name/display name, the sign center as position, zero rotation, the current server
level, game-master permission, and no command-output source. `SHOW_DIALOG` opens its dialog and
`CUSTOM` dispatches its identifier/payload to the server. These three set the handled result and
do not stop later lines; all other click actions are ignored.

A waxed sign then plays its form-specific interaction-failure sound and returns
`SUCCESS_SERVER`, even when a click action already ran. An unwaxed sign returns success when any
supported action ran. Otherwise it may open editing only when there is no other editor, the player
may build, and all four currently visible components are either the canonical empty component or
have `PlainTextContents`; styled literals remain editable, while translatable and other component
content does not. Failure returns `PASS`.

**Hanging-sign chain precedence:**

Ceiling hanging use returns `PASS` to the placement pipeline when the held item is a hanging sign,
the hit face is exactly DOWN, and the selected side does not have a waxed visible `RUN_COMMAND`.
Wall hanging use does the same when the hit face's axis differs from its facing axis. Otherwise the
shared interaction runs. This guard intentionally recognizes only waxed run-command text:
`SHOW_DIALOG` and `CUSTOM` do not prevent chaining, and an unwaxed run-command sign also permits
placement precedence on those noneditable faces.

**Edit submission and filtering:**

Exact packet fields, four UTF bounds, formatting removal, asynchronous filtering, chunk/entity
revalidation and no-response rejection belong to `PROTO-PLAY-SERVERBOUND-SIGN-UPDATE-001`.
At entity commit time the sign independently requires not waxed, the submitting UUID equal to the
authorized UUID, and a nonnull level. Rejection logs and retains authorization. Success rebuilds
the selected immutable `SignText`, then clears authorization and unconditionally requests a
same-state block update with flags 3.

For each of the four official filtered results, commit preserves the style of the submitting
player's currently visible old line. With player filtering enabled, the filtered-or-empty literal
becomes both raw and filtered. Otherwise raw and filtered-or-empty literals are stored separately.
This preserves styling, including click actions, while replacing component contents with literal
text. Each line replacement creates a new text object; installing a different object dirties the
entity and requests its own flags-3 update. A normal accepted four-line submission therefore
requests one update during mutation and another unconditional update after clearing authorization.

**Persistence and synchronization:**

Full save and the custom update tag encode `front_text`, `back_text` through the `SignText` codec
and always encode `is_waxed`. Each raw array must have exactly four entries. Filtered messages are
optional and omitted when every entry equals raw; absence copies raw. Color defaults black and
glowing defaults false. Missing or malformed side data loads a fresh empty side; wax defaults
false.

During load, every raw and filtered component is resolved independently at the sign position
against a server command source with no player and game-master permission. A resolution syntax
failure retains the original component. Successful setters use object identity: the exact same
text object or same wax value is a no-op; a different object/value dirties and sends a same-state
flags-3 update. The ordinary block-entity-data packet carries protocol type 7 or 8 plus the custom
update tag.

**Client editor and rendering:**

The open-editor packet resolves the current sign entity, selects the ordinary or hanging screen,
and snapshots the chosen visible side into four strings. The local editor accepts a candidate line
only when current font width is at most 90 or 60 pixels; there is no second character-count limit.
Up wraps `(line-1)&3`; down and confirmation wrap `(line+1)&3`. Each local edit replaces the
client entity's selected text for preview. The non-pausing in-game screen closes when the local
player disappears, the entity is removed or the same range predicate fails. Removal sends exactly
one four-line update when a connection exists, including Done, escape and automatic closure.

World rendering draws both sides. For each side it chooses raw/filtered visibility, centers four
lines at ordinary Y `-20,-10,0,10` or hanging Y `-18,-9,0,9`, splits to the form's maximum pixel
width and takes only the first formatted sequence per line. Non-glowing text uses the dye RGB
scaled by 0.4 and ordinary packed light. Glowing text uses exact dye RGB and full-bright 15728880.
Its dark outline is always visible for black; other colors outline only while first-person scoping
or when camera-to-sign-center squared distance is strictly below 256. Black's special outline is
`-988212`; other outlines use the 0.4-scaled color. Models, textures and wood sound variants remain
locked data selected by wood type and attachment.

**Branches and aborts:**

All 48 blocks and 1,344 states; every placement direction, secondary-use, support/tag relation and
water state; front/back and exact 90-degree boundary; no/same/other/stale editor; raw/filtered
visibility; empty, literal, styled and nonliteral components; all click-event kinds and line
orders; waxed/unwaxed; every dye/same color, glow/ink no-op and honeycomb branch; accepted,
rejected, unchanged and delayed edits; save/update tags with all malformed/default fields; every
render color, glow, scope and distance boundary.

**Constants and randomness:**

48 blocks; 24 items; 12 wood types; 1,344 states; block-entity protocol IDs 7/8; four lines per
side; two sides; ordinary line height/width 10/90; hanging 9/60; 16 rotation segments; editor extra
range 4.0; front threshold 90 degrees inclusive; full-bright 15728880; outline squared-distance
threshold 256 strict; non-glow/dark-color multiplier 0.4; level event 3003; block-update flags 3.
The sign state machines consume no gameplay RNG. Ordinary sound packet seeding and event-particle
randomness remain in their generic client/transport owners.

**Side effects:**

Block placement/removal and water scheduling; editor authorization/open/close; text/color/glow/wax
mutation and chunk dirtiness; block-entity-data and same-state update projection; supported click
commands/dialogs/custom actions; item stat/consumption; `BLOCK_CHANGE`; applicator/failure sounds
and wax particles; front/back text rendering. Signs emit no redstone or comparator signal and have
no random-tick behavior.

**Gates:**

Placement support and context ordering; matching entity type; player build permission; editor
lease and range; selected side; wax state; visible nonempty text for dye/ink applicators; actual
value change; literal-content editability; handler and entity-time update authorization; client
connection for final submission. Difficulty and game rules do not gate sign behavior.

**Boundary cases and quirks:**

The exact perpendicular angle is front. Styled literal click actions survive edits and can execute
immediately after an applicator. Click actions execute before wax failure. Honeycomb can wax empty
text, while dye and ink applicators cannot mutate an empty visible side. Accepted edits normally
request two flags-3 updates. Rejected entity-time submissions retain the editor UUID. Wall/ceiling
hanging placement semantics depend on reloadable sign tags. Waxed dialog/custom text can lose
interaction precedence to hanging-sign chaining because only waxed `RUN_COMMAND` blocks it.

**Evidence:**

`Confirmed`; `OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-REPORT-001`; `OFF-DATA-001`. Anchors:
`net.minecraft.world.level.block.SignBlock#useItemOn`,
`net.minecraft.world.level.block.SignBlock#useWithoutItem`,
`net.minecraft.world.level.block.SignBlock#openTextEdit`,
`net.minecraft.world.level.block.StandingSignBlock#getStateForPlacement`,
`net.minecraft.world.level.block.WallSignBlock#getStateForPlacement`,
`net.minecraft.world.level.block.CeilingHangingSignBlock#getStateForPlacement`,
`net.minecraft.world.level.block.WallHangingSignBlock#canAttachTo`,
`net.minecraft.world.level.block.entity.SignBlockEntity#updateSignText`,
`net.minecraft.world.level.block.entity.SignBlockEntity#executeClickCommandsIfPresent`,
`net.minecraft.world.level.block.entity.SignBlockEntity#tick`,
`net.minecraft.world.level.block.entity.SignText#setMessage`,
`net.minecraft.world.item.SignItem#updateCustomBlockEntityTag`,
`net.minecraft.world.item.DyeItem#tryApplyToSign`,
`net.minecraft.world.item.GlowInkSacItem#tryApplyToSign`,
`net.minecraft.world.item.InkSacItem#tryApplyToSign`,
`net.minecraft.world.item.HoneycombItem#tryApplyToSign`,
`net.minecraft.client.gui.screens.inventory.AbstractSignEditScreen#removed`,
`net.minecraft.client.renderer.blockentity.AbstractSignRenderer#submit`; locked reports, tags, loot
and assets; `PROTO-PLAY-SERVERBOUND-SIGN-UPDATE-001`; the clientbound editor-open family;
`EXP-BLK-025`.

**Test vectors:**

Enumerate all placement/support/tag/water/chain combinations, then cross side angle, filtering,
editor UUID/range, wax, component contents, every click event and applicator change/no-op. Submit
accepted and rejected four-line updates around save/load, unload and update projection, then
compare ordinary/hanging UI width admission and both-side renderer geometry/colors/outlines. Run
`EXP-BLK-025` as the executable matrix and reuse the completed sign protocol vectors for exact
wire ordering.
