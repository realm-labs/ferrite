# Blocks mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `BLK-SKULL-001` — Skulls retain power, profile data and animation while wither heads can consume a summon pattern

**Parent:** `SIM-003`, `SIM-005`, `BLK-001`, `BLK-002`, `BLK-003`, `BLK-007`,
`PLY-005`, `PLY-006`, `RED-001`, `ITM-001`, `ITM-003`, `ENT-001`, `ENT-005`,
`CLI-001`, `CLI-006`, `ENV-003`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked server source fixes the 14 floor/wall blocks, their 280 states,
placement and power transitions, three-field block entity, loot/component projection, custom
note-block sound and wither-summon transaction. Locked client source and assets fix profile
resolution, block/item models, world transforms and dragon/piglin animation. This rule owns those
skull-selected branches; generic placement, loot admission, note-block event transport, entity
admission and the spawned wither's later behavior retain their existing owners.

**Applies when:**

Any skeleton, wither-skeleton, zombie, player, creeper, dragon or piglin skull/head is placed as
its floor or wall form, powered, loaded, saved, componentized, broken or rendered; a player-head
name or `FillPlayerHead` loot function is evaluated; a custom head is above a note block; or a
wither-skeleton skull placement checks the wither pattern.

**Authoritative state:**

The seven floor blocks own `ROTATION=0..15` and `POWERED`, default 0/false, for 32 states each.
Their seven wall partners own horizontal `FACING` and `POWERED`, default north/false, for eight
states each. The locked contiguous ranges are:

| Type | Floor states | Wall states |
|---|---:|---:|
| skeleton | 10915..10946 | 10947..10954 |
| wither skeleton | 10955..10986 | 10987..10994 |
| zombie | 10995..11026 | 11027..11034 |
| player | 11035..11066 | 11067..11074 |
| creeper | 11075..11106 | 11107..11114 |
| dragon | 11115..11146 | 11147..11154 |
| piglin | 11155..11186 | 11187..11194 |

Every floor form has strength 1, push reaction `DESTROY`, no occlusion and its type-specific note
instrument: skeleton, wither skeleton, zombie, custom head, creeper, dragon or piglin. The
`wallVariant` constructor copies the floor loot table and description only; every wall form
therefore has strength 1 and `DESTROY` but retains default HARP and default occlusion. Both forms
retain ordinary collision and always reject pathfinding.

The exact `SKULL` block entity, protocol ID 16, is valid for all 14 blocks. It owns nullable
`ResolvableProfile owner`, nullable `Identifier noteBlockSound`, nullable component `customName`,
and transient client animation count/active fields initialized to 0/false.

**Transition and ordering:**

Generic standing/wall item selection uses attachment direction down. Floor placement converts
the placement rotation to the 16-segment `ROTATION`; rotations and mirrors transform that segment.
Wall placement scans `getNearestLookingDirections()` in order, skips vertical directions and sets
`FACING` opposite the first inspected direction whose adjacent block is not replaceable under the
placement context. No candidate returns null. Wall rotations and mirrors transform `FACING`.
Neither form adds survival or support-loss logic, so forced or already placed heads remain floating.

Initial `POWERED` is `hasNeighborSignal(pos)`. A neighbor callback returns on the client. On the
server it reads only that same position, compares the result to the stored property and, on a
difference, offers a flags-2 state write whose result is ignored. It neither checks above nor
schedules work.

Non-piglin floor shape is X/Z 4..12 and Y 0..8; piglin floor shape is X/Z 3..13 and Y 0..8.
Non-piglin wall base shape is X 4..12, Y 0..8, Z 8..16; piglin wall base is X 3..13, Y 0..8,
Z 8..16. Wall shapes rotate horizontally with facing.

**Persistence, components and loot:**

Save writes present values as `profile`, `note_block_sound` and `custom_name`. Load decodes each
independently; missing or invalid values become null. Loading into an existing entity does not
reset its transient animation fields, while reconstruction creates them at 0/false. The update
packet is always a block-entity-data packet and the update tag is the custom-only save, so all
three durable fields are projected while neither animation field is.

Implicit components replace/expose `PROFILE`, `NOTE_BLOCK_SOUND` and `CUSTOM_NAME`; removing
components from the stored tag discards exactly those three fields. Generic block placement
applies stack components and dirties the block entity before `setPlacedBy`.

Each floor loot table, shared by its wall partner, yields one corresponding item in one
unconditional roll, copies `CUSTOM_NAME` and has no explosion condition or decay. Player head
additionally copies `PROFILE` and `NOTE_BLOCK_SOUND`; the other six deliberately lose those two
fields when broken. Block-entity pre-removal adds no independent inventory drop.

All seven items stack to 64, equip to the head slot with `swappable=false`, and carry the hidden
head-slot attribute modifier `minecraft:waypoint_transmit_range_hide`: waypoint transmit range
-1 with `add_multiplied_total`. Wither-skeleton skull is rare, dragon head is epic and the other
five are uncommon. None has a default profile or note-block sound. `PlayerHeadItem#getName`
returns `<descriptionId>.named` with the profile name when `PROFILE` has one, otherwise the generic
item name; ordinary custom-name precedence remains generic. After its conditions, `FillPlayerHead`
sets a player-head stack's `PROFILE` to the selected Player's resolved game profile and otherwise
returns it unchanged.

**Client animation and rendering:**

Only exact dragon/piglin floor and wall blocks install the exact-type client ticker. While
`POWERED=true`, each client tick marks the entity animating and increments its Java int counter;
otherwise it clears the active flag but freezes the counter. The sampled animation value is
counter plus partial tick only while active, otherwise exactly the frozen counter. The server
never advances it and integer overflow retains Java semantics.

Dragon jaw X rotation is `(sin(animation * pi * 0.2) + 1) * 0.2`. Piglin left-ear Z rotation is
`-(cos(animation * pi * 0.2 * 1.2) + 2.5) * 0.2`; right-ear Z rotation is
`(cos(animation * pi * 0.2) + 2.5) * 0.2`. The other five models receive the value but have no
moving-part response.

All 14 blockstate JSONs ignore their properties and select `block/skull`, whose only baked content
is a soul-sand particle texture; visible geometry comes from the block-entity renderer. Fixed
entity textures are skeleton, wither skeleton, zombie, creeper, ender dragon and piglin. A player
head without profile uses the default player texture. A profiled player head resolves through the
five-minute access-expiring skin cache, using the deterministic default plus skin patch until a
completed successful profile/skin lookup supplies the resolved skin plus patch. Profiled player
heads render translucent; fixed paths use cutout with Z offset.

For a wall head the world transform translates to
`(0.5-0.25*facing.stepX, 0.25, 0.5-0.25*facing.stepZ)`, rotates Y by the negative opposite-facing
yaw and scales `(-1,-1,1)`. A floor head translates `(0.5,0,0.5)`, rotates Y by the negative
16-segment degrees and uses the same scale. Rendering submits animation, packed light, no overlay
and any break overlay. Each item definition uses a special renderer: six fixed `head` kinds and
the profile-aware `player_head`; dragon alone uses the dragon-head base model.

**Note-block join:**

A floor head immediately above a note block supplies its type instrument. A wall head retains HARP,
which does not work above note blocks, and therefore cannot select a mob/custom-head sound there.
The skull-item top-instrument tag makes note-block use on its upper face pass so placement may run.
For `CUSTOM_HEAD`, the note block reads `noteBlockSound` only from the exact skull block entity
above; null aborts the triggered event. A value becomes a direct variable-range sound in RECORDS,
volume 3 and pitch 1, with the server RNG's next long as seed. The existing note-block, sound and
`PROTO-PLAY-CLIENTBOUND-BLOCK-001` owners retain event transport; this rule owns the head/entity
selection and synchronized identifier.

**Wither-summon transaction:**

Placing either wither-skull form calls `checkSpawn` without requiring a player. It proceeds only
on the server, for an exact skull entity whose current block is either wither form, at Y at least
the level minimum, outside Peaceful difficulty, and with a full pattern match:

```text
^^^
###
~#~
```

`^` accepts either floor or wall wither skull, `#` is the reloadable
`minecraft:wither_summon_base_blocks` tag (locked to soul sand and soul soil), and `~` is air.
Pattern search covers anchors through trigger position plus (2,2,2) and all 24 ordered
perpendicular forward/up direction pairs. No match is a no-op. Null wither creation leaves the
pattern intact.

After successful creation, all nine matched cells—including the two already-air cells—are set to
air with flags 2 and each emits level event 2001 with its cached original state ID. The wither is
positioned from matched cell (1,2,0) at center X/Z and Y+0.55, with pitch 0 and yaw/body yaw 0 for
an X-axis forward direction or 90 otherwise, then made invulnerable. Before entity admission,
`SUMMONED_ENTITY` is triggered for every server player in the wither's bounding box inflated 50.
The result of `addFreshEntity` is ignored. Finally all nine positions request neighbor updates
with air, so admission failure does not restore the consumed structure.

The dispenser precheck is deliberately narrower: it requires the exact wither-skull item, server,
non-Peaceful difficulty, Y at least minimum+2 and the six-block base/air pattern before placing the
third skull. `ITM-DISPENSER-001` owns its item consumption and event wrapper; this rule owns the
subsequent structure check and consumption.

**Branches and aborts:**

Floor versus wall placement, replaceable wall support, unchanged/changed power, durable-field
decode, present/absent profile or sound, client ticker type, custom-note sound presence, wither
pattern/difficulty/height/side/entity-creation and entity-admission result are distinct branches.
Only null wither creation rolls back by never consuming the pattern; failed admission occurs after
irreversible block/event/criterion mutations.

**Constants and randomness:**

Fourteen blocks, 280 states, protocol ID 16, rotation modulus 16, power flags 2, animation step 1,
five-minute skin-cache expiry, note sound volume/pitch 3/1, 24 pattern orientations, nine cleared
and updated cells, spawn Y offset 0.55 and criterion inflation 50 are fixed. Power, placement,
animation and summon-pattern selection consume no RNG. Custom note playback consumes one server
long; profile/skin services and the spawned entity's later behavior belong to their own owners.

**Side effects:**

Block-state writes and neighbor callbacks; block-entity dirty/save/update/component state; item
loot and name/profile mutation; client cache lookup, animation and render submissions; note-block
sound/event projection; wither-pattern removal, break events, criterion triggers, entity admission
and neighbor updates.

**Gates:**

Placement context and replaceability; exact state/entity/item/type; server/client side; neighbor
signal; nullable/valid codecs and components; loot function conditions and selected Player; head
instrument/tag and custom identifier; difficulty, height, reloadable base tag, air/full-pattern and
entity creation/admission.

**Boundary cases and quirks:**

Wall variants copy neither the floor instrument nor `noOcclusion`. Heads have no post-placement
support rule. Power checks never include the block above and schedule nothing. Animation freezes
rather than resets when unpowered and is client-only/transient. Non-player loot preserves a custom
name but discards profile/sound. A custom head requires the floor form for above-note behavior.
Wither pattern clearing includes air cells, criteria precede ignored entity admission, and all
neighbor updates still occur after an admission failure.

**Evidence:**

`OFF-SERVER-001`, `OFF-CLIENT-001`, `OFF-DATA-001`, `OFF-REPORT-001`;
`net.minecraft.world.level.block.Blocks#wallVariant(net.minecraft.world.level.block.Block,boolean)`,
`net.minecraft.world.level.block.AbstractSkullBlock#newBlockEntity`,
`net.minecraft.world.level.block.AbstractSkullBlock#getTicker`,
`net.minecraft.world.level.block.AbstractSkullBlock#getStateForPlacement`,
`net.minecraft.world.level.block.AbstractSkullBlock#neighborChanged`,
`net.minecraft.world.level.block.SkullBlock#getShape`,
`net.minecraft.world.level.block.SkullBlock#getStateForPlacement`,
`net.minecraft.world.level.block.SkullBlock#rotate`,
`net.minecraft.world.level.block.SkullBlock#mirror`,
`net.minecraft.world.level.block.WallSkullBlock#getShape`,
`net.minecraft.world.level.block.WallSkullBlock#getStateForPlacement`,
`net.minecraft.world.level.block.WallSkullBlock#rotate`,
`net.minecraft.world.level.block.WallSkullBlock#mirror`,
`net.minecraft.world.level.block.WitherSkullBlock#setPlacedBy`,
`net.minecraft.world.level.block.WitherSkullBlock#checkSpawn(net.minecraft.world.level.Level,net.minecraft.core.BlockPos)`,
`net.minecraft.world.level.block.WitherSkullBlock#checkSpawn(net.minecraft.world.level.Level,net.minecraft.core.BlockPos,net.minecraft.world.level.block.entity.SkullBlockEntity)`,
`net.minecraft.world.level.block.WitherSkullBlock#canSpawnMob`,
`net.minecraft.world.level.block.WitherWallSkullBlock#setPlacedBy`,
`net.minecraft.world.level.block.entity.SkullBlockEntity#saveAdditional`,
`net.minecraft.world.level.block.entity.SkullBlockEntity#loadAdditional`,
`net.minecraft.world.level.block.entity.SkullBlockEntity#animation`,
`net.minecraft.world.level.block.entity.SkullBlockEntity#getAnimation`,
`net.minecraft.world.level.block.entity.SkullBlockEntity#getUpdatePacket()`,
`net.minecraft.world.level.block.entity.SkullBlockEntity#getUpdateTag`,
`net.minecraft.world.level.block.entity.SkullBlockEntity#applyImplicitComponents`,
`net.minecraft.world.level.block.entity.SkullBlockEntity#collectImplicitComponents`,
`net.minecraft.world.level.block.entity.SkullBlockEntity#removeComponentsFromTag`,
`net.minecraft.world.item.PlayerHeadItem#getName`,
`net.minecraft.world.level.storage.loot.functions.FillPlayerHead#getReferencedContextParams`,
`net.minecraft.world.level.storage.loot.functions.FillPlayerHead#run`,
`net.minecraft.world.level.block.NoteBlock#getCustomSoundId`,
`net.minecraft.world.level.block.NoteBlock#triggerEvent`,
`net.minecraft.client.renderer.blockentity.SkullBlockRenderer#extractRenderState(net.minecraft.world.level.block.entity.SkullBlockEntity,net.minecraft.client.renderer.blockentity.state.SkullBlockRenderState,float,net.minecraft.world.phys.Vec3,net.minecraft.client.renderer.feature.ModelFeatureRenderer$CrumblingOverlay)`,
`net.minecraft.client.renderer.blockentity.SkullBlockRenderer#submit(net.minecraft.client.renderer.blockentity.state.SkullBlockRenderState,com.mojang.blaze3d.vertex.PoseStack,net.minecraft.client.renderer.SubmitNodeCollector,net.minecraft.client.renderer.state.level.CameraRenderState)`,
`net.minecraft.client.renderer.blockentity.SkullBlockRenderer#resolveSkullRenderType`,
`net.minecraft.client.renderer.PlayerSkinRenderCache#getOrDefault`,
`net.minecraft.client.renderer.PlayerSkinRenderCache#lookup`,
`net.minecraft.client.renderer.special.SkullSpecialRenderer#submit`,
`net.minecraft.client.model.object.skull.DragonHeadModel#setupAnim(net.minecraft.client.model.object.skull.SkullModelBase$State)`,
`net.minecraft.client.model.object.skull.PiglinHeadModel#setupAnim(net.minecraft.client.model.object.skull.SkullModelBase$State)`;
`reports/blocks.json#minecraft:{skeleton_skull,skeleton_wall_skull,wither_skeleton_skull,wither_skeleton_wall_skull,zombie_head,zombie_wall_head,player_head,player_wall_head,creeper_head,creeper_wall_head,dragon_head,dragon_wall_head,piglin_head,piglin_wall_head}`,
`reports/registries.json#minecraft:block_entity_type/minecraft:skull`,
`reports/minecraft/components/item/{skeleton_skull,wither_skeleton_skull,zombie_head,player_head,creeper_head,dragon_head,piglin_head}.json`,
`data/minecraft/tags/block/wither_summon_base_blocks.json`,
`data/minecraft/tags/item/{skulls,noteblock_top_instruments}.json`,
`data/minecraft/loot_table/blocks/{skeleton_skull,wither_skeleton_skull,zombie_head,player_head,creeper_head,dragon_head,piglin_head}.json`,
`assets/minecraft/{blockstates,models,items}/**/*{skull,head}*.json`.

**Test vectors:**

Use `EXP-BLK-026`. Cross all 280 states and placement/support/power transitions; fresh/existing
load, malformed fields, components, update tags and seven loot tables; named/unnamed and filled
player profiles; floor/wall note-block behavior; dragon/piglin power cycles and skin completion;
and every wither height/difficulty/orientation/base/entity-creation/admission boundary. Assert
field/state/event/entity/render order, exact values and no side effect across each abort.
