# Blocks mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `BLK-HONEYCOMB-BLOCK-001` — Honeycomb block joins compacting, sticky equipment and coral sounds

**Parent:** `BLK-001`, `BLK-002`, `BLK-003`, `BLK-004`, `BLK-005`, `PLY-005`, `PLY-006`,
`ITM-004`, `ITM-006`, `ENT-001`, `ENT-005`, `ENV-003`, `CLI-006`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — the locked registration, reports, complete loot/recipe/advancement/tag data,
server class-reference search, all 1,212 decoded structure templates and exact client assets
exhaust this property-free identity. It has no block-specific implementation class or worldgen
consumer; its only content-selected runtime branch is the block item's sticky sulfur-cube
archetype membership.

**Applies when:**

`minecraft:honeycomb_block` is placed, written, mined, exploded, crafted, equipped on a sulfur
cube, persisted, mapped or rendered.

**Authoritative state:**

Honeycomb block is an ordinary property-free `Block` with no block entity and sole state `21817`.
Its locked block protocol ID is `914`, and its block-item raw ID is `1413`. Registration selects
map color `COLOR_ORANGE`, the default `HARP` note instrument, one-argument strength `0.6/0.6` and
the `CORAL_BLOCK` sound type. It does not require a correct tool.

The state is a full unit selection/collision/visual/occlusion cube with emission `0`, light
dampening `15`, shade brightness `0.2`, friction `0.6`, speed/jump factors `1`, restitution `0`,
solid redstone conduction, normal piston reaction, full sturdy faces and ordinary full-face spawn
support. It adds no random or scheduled tick, use, attack, entity-contact, neighbor, signal,
comparator or block-event override. No locked block tag directly contains the identity.

The coral sound type has volume/pitch multipliers `1/1` and selects sound registry IDs break `444`,
step `448`, place `447`, hit `446` and fall `445`. The ordinary block item is common, stacks to
`64` and has only standard block-item components. It is directly in the reloadable
`sulfur_cube_archetype/sticky` item tag.

**Transition and ordering:**

#### Placement, breaking and compacting

Ordinary placement and authoritative component/command writes always select state `21817`;
rotation and mirror are identity operations. Because registration does not set
`requiresCorrectToolForDrops`, hand, tool and tier do not gate its generic harvest path.

Its one-roll block loot table offers one matching item behind `survives_explosion` and uses random
sequence `minecraft:blocks/honeycomb_block`. Ordinary player removal therefore returns one item;
an admitted explosion can suppress that entry through the shared explosion-survival condition.
Silk Touch and Fortune do not alter the table.

The sole exact recipe is a shaped 2-by-2 square of four `honeycomb` items yielding one
`honeycomb_block`. Its decorations advancement has `has_honeycomb` inventory and
`has_the_recipe` criteria in one OR requirement and grants only this recipe. There is no reverse
block-to-honeycomb record. Grid orientation/reflection, consumption, output admission and
recipe-book publication remain with the generic crafting owners.

#### Sticky sulfur-cube equipment

The block item is the sole direct member of `sulfur_cube_archetype/sticky` and has no second
archetype tag. When a sulfur cube recomputes body equipment, registry iteration therefore selects
the sticky record exactly once for this item. That record fixes horizontal/vertical knockback
powers `0.4125/0.09`, `sticky.hit` and `sticky.push` sounds, push cooldown `0.5`, impulse threshold
`0.05`, and five attribute entries: additive knockback and explosion-knockback resistance `-2/-2`,
additive bounciness `0`, total-multiplied friction `+1`, and total-multiplied air drag
`-0.9900000002235174`.

Matching order, equipment replacement, transient modifier removal/addition, contact and knockback
math, sounds and entity projection remain with the sulfur-cube and entity owners. Reload changes
future tag/archetype matching without altering an already persisted world block.

#### Explicit generation absence

The locked server class-reference search finds `HONEYCOMB_BLOCK` only in registration, item,
creative-tab and data-provider classes. Exact bundled-data references are limited to its block
loot, recipe, recipe advancement and sticky item tag. No block tag, configured feature, placed
feature, processor or other worldgen record selects state `21817`.

The complete NBT scan finds zero honeycomb-block cells in all 1,212 bundled structure templates.
World generation and structure placement therefore have no identity-specific honeycomb-block
branch to implement; later player, command or component placement remains ordinary world
mutation.

**Client projection:**

The only blockstate variant unconditionally selects `minecraft:block/honeycomb_block`. That model
inherits `cube_all` and maps every face to `minecraft:block/honeycomb_block`; the item selector
points directly to the same model. Authoritative block updates publish state `21817`, inventory
projection uses item ID `1413`, material sounds use IDs `444..448` in the order listed above, and
map projection uses `COLOR_ORANGE`. This leaf adds no packet field, acknowledgement, ordering rule
or connection-local state.

**Branches and aborts:**

Ordinary/component placement; hand or any tool; player versus explosion removal and
survived/suppressed loot; shaped match/reflection/output capacity; either unlock criterion;
current/reloaded sticky tag and archetype snapshots; empty/other/honeycomb-block sulfur body
equipment; ordinary state versus block/item/sound/map projection; save/reload are distinct
branches.

**Constants and randomness:**

State/block/item IDs `21817/914/1413`; strength `0.6/0.6`; sound IDs break/step/place/hit/fall
`444/448/447/446/445`; emission `0`, dampening `15`, shade `0.2`, friction `0.6`, speed/jump `1`,
restitution `0`, stack `64`; recipe input/output `4/1`; sticky powers `0.4125/0.09`, cooldown
`0.5`, threshold `0.05` and five modifier amounts as listed above; structure cells `0`. The block
consumes no RNG; loot, crafting and sulfur-cube owners retain their own selection state.

**Side effects:**

Ordinary full-block placement/removal and conditional self loot; one crafting result/grant;
reload-selected sticky sulfur-cube equipment behavior; ordinary palette/inventory persistence;
coral material sounds, orange map shading and opaque cube-all projection.

**Gates:**

World-write and break authority; explosion context; active loot, recipe, advancement, item-tag and
archetype snapshots; crafting/output admission; sulfur body-equipment admission; valid registry,
map, sound and client-resource context.

**Boundary cases and quirks:**

The honeycomb block sounds like coral despite its name and texture. Its block form is not a piston-
sticky block and has none of `HoneyBlock`'s collision or movement hooks; only its item form selects
the sulfur-cube archetype named `sticky`. The compacting recipe has no reverse decompression path.
No locked structure or generation record creates the block.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-REPORT-001`; `OFF-DATA-001`;
`net.minecraft.world.level.block.Blocks`;
`net.minecraft.world.level.block.state.BlockBehaviour$Properties#strength`;
`net.minecraft.world.level.block.SoundType`;
`net.minecraft.world.entity.monster.cubemob.SulfurCube#matchingArchetypes`;
`reports/blocks.json#minecraft:honeycomb_block`;
`reports/registries.json#minecraft:{block,item}/minecraft:honeycomb_block`;
`reports/registries.json#minecraft:sound_event/minecraft:block.coral_block.*`;
`reports/minecraft/components/item/honeycomb_block.json`;
`data/minecraft/loot_table/blocks/honeycomb_block.json`;
`data/minecraft/recipe/honeycomb_block.json`;
`data/minecraft/advancement/recipes/decorations/honeycomb_block.json`;
`data/minecraft/tags/item/sulfur_cube_archetype/sticky.json`;
`data/minecraft/sulfur_cube_archetype/sticky.json`;
`data/minecraft/structure/**/*.nbt`;
`assets/minecraft/blockstates/honeycomb_block.json`;
`assets/minecraft/models/block/honeycomb_block.json`;
`assets/minecraft/items/honeycomb_block.json`.

**Test vectors:**

Run `EXP-BLK-058` across state/registry identity, ordinary/component writes, every tool and
ordinary/explosion loot context, the shaped recipe and both unlock paths, tag/archetype reload,
empty/other/honeycomb-block sulfur body equipment, all 1,212 structure inputs, save/reload, coral
sounds, map color and both block/item models. Assert exact constants, matching, modifier selection,
zero generation/template joins and vanilla-client convergence.

**Limits:**

Generic placement, breaking, loot, crafting, advancements, sulfur-cube equipment/contact/
knockback, packet encoding and client rendering remain with `BLK-PLACE-001`, `PLY-BREAK-001`,
`ITM-LOOT-001`, `ITM-RECIPE-001`, `ITM-ADVANCEMENT-001`, `ENT-KNOCKBACK-001`,
`PROTO-PLAY-CLIENTBOUND-BLOCK-001`, `PROTO-PLAY-CLIENTBOUND-SOUND-001` and `CLI-006`.
