# Items mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ITM-BUNDLE-001` — Bundles preserve ordered weighted contents while selection stays transient

**Parent:** `PLY-005`, `PLY-006`, `ITM-001`, `ITM-002`, `ITM-003`, `ITM-004`, `ITM-006`,
`ITM-007`, `ENT-001`, `CLI-001`, `CLI-006`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked item registration, component codecs, item/container overrides, active-use
callbacks, transmute recipes, advancements, loot, tags, registries, packet handling and client
assets close the plain bundle and all sixteen dyed bundles.

**Applies when:**

Any stack in `minecraft:bundles` is inserted into, emptied, selected, used, crafted, recolored,
destroyed, persisted, reloaded or projected to an unmodified client.

**Authoritative state:**

| Ordered identity | Item protocol ID |
|---|---:|
| bundle | `1065` |
| white bundle | `1066` |
| orange bundle | `1067` |
| magenta bundle | `1068` |
| light-blue bundle | `1069` |
| yellow bundle | `1070` |
| lime bundle | `1071` |
| pink bundle | `1072` |
| gray bundle | `1073` |
| light-gray bundle | `1074` |
| cyan bundle | `1075` |
| purple bundle | `1076` |
| blue bundle | `1077` |
| brown bundle | `1078` |
| green bundle | `1079` |
| red bundle | `1080` |
| black bundle | `1081` |

All seventeen are common-rarity `BundleItem` instances with maximum stack size one and an empty
`bundle_contents` default component. The component stores an ordered list of nonempty item-stack
templates plus an in-memory selected index. Data and stream codecs encode only the ordered list;
construction from either codec selects index `-1`. Equality and hashing likewise compare only the
list. Selection is therefore prediction/interaction state, not durable or wire-comparable content.

**Transition and ordering:**

#### Capacity and content order

The capacity budget is the exact fraction `1`. One ordinary item costs
`1 / item.getMaxStackSize()` per unit. An item with a nonempty `bees` component costs the complete
budget per unit. An item carrying `bundle_contents` instead costs its nested content weight plus
`1/16`, so an empty bundle can be nested and a full inner bundle cannot. Empty stacks and items
whose `canFitInsideContainerItems` hook returns false are rejected.

Insertion takes
`min(source count, floor((1 - current weight) / unit weight))`. A matching stack means the same
item and components: its existing entry is removed, grown by the admitted count and moved to list
index zero. A new entry is split from the source and inserted at index zero. Thus successful
insertion always makes the inserted or merged identity newest. Weight arithmetic failure rejects a
new insert; constructing a mutable view from an arithmetically invalid component clears its list,
weight and selection. Ordinary operations never admit weight above one, although the component
codec itself does not impose a separate capacity validator.

Removal chooses the selected index when it is within the complete list and otherwise index zero.
It removes that whole stored stack entry, subtracts its complete weight and clears selection. It
never removes just one unit from an entry.

#### Cursor and slot overrides

When the bundle is the cursor stack over a menu slot:

- primary click on a nonempty slot asks `safeTake(source count, capacity count, player)` and inserts
  what was taken. A positive transfer plays Insert; zero plays Insert Fail;
- secondary click on an empty slot removes one whole entry and passes it to `safeInsert`. Any
  remainder is put back into the bundle; Remove One plays only when the complete removed stack was
  accepted;
- either handled branch writes a new immutable component, invokes
  `containerMenu.slotsChanged(player inventory)` and consumes normal click handling.

When the bundle occupies the menu slot:

- primary click with a nonempty cursor requires `slot.allowModification`; it inserts as much as
  capacity permits and plays Insert, or plays Insert Fail on denial/zero transfer;
- secondary click with an empty cursor requires `slot.allowModification`, removes one whole entry
  into the cursor and plays Remove One; the branch still rewrites/broadcasts and consumes handling
  when the slot is denied or the bundle is empty;
- primary click with an empty cursor clears selection and returns false so ordinary pickup can
  proceed. Other unhandled click/action combinations also clear selection before returning false.

An insert does not renumber or clear the stored selected integer even though it moves an entry to
index zero. A later selected removal therefore follows the resulting numeric index, not a retained
item identity. Removal always clears the index.

Insert and Remove One sounds are registry IDs `238` and `240`, volume `0.8`, pitch
`0.8 + nextFloat * 0.4`. Insert Fail ID `239` uses volume/pitch `1/1`. Menu convergence,
state-ID checking and ordinary click replay remain with `ITM-CONTAINER-001` and
`ITM-CONTAINER-CLICK-001`.

#### Held use and item-entity destruction

Using any bundle always starts a 200-tick `BUNDLE` animation and returns success. Only a player
executes the per-tick output callback. It attempts one removal immediately when remaining duration
is `200`, then at even remaining durations strictly below `190`: `188, 186, ... , 2`. Each
successful attempt removes one whole entry, plays Remove One, drops that stack through
`Player.drop(stack,true)`, plays Drop Contents ID `237` in the Players category at the player's
block position with volume `0.8` and pitch `0.8 + nextFloat * 0.4`, and awards one item-used stat
for that bundle identity. Empty attempts do none of those things; release simply stops future
attempts.

When a bundle item entity is destroyed after its ordinary damage gates, its callback first replaces
the entity's component with empty contents, then creates one item entity at the same coordinates
for every copied stored entry. Clearing first prevents the dying outer entity from retaining a
second copy.

#### Selection ingress and client prediction

In a hovered menu slot, the client bundle mouse action matches the `bundles` tag. With at least one
visible entry, accumulated vertical scroll (or negated horizontal scroll when vertical is zero)
moves one sign-normalized step, wraps through visible indices, mutates the local component first
and sends play serverbound `bundle_item_selected` with the menu-local slot and index. Hover exit,
quick move and swap request index `-1`.

The server resolves the handler-time current menu and slot. Invalid slot or absent component is a
no-op; index below `-1` is a decode fault; an out-of-list index clears selection; a valid index may
address a complete-list entry outside the displayed subset. The packet has no state ID, container
ID, acknowledgement or correction transaction. Exact codec and handler admission belong to
`PROTO-PLAY-SERVERBOUND-INVENTORY-AUXILIARY-001`; the client-first mutation and selection's
codec/equality exclusion explain why it remains usable until component reconstruction or later
mutation despite not synchronizing as durable state.

#### Crafting, recoloring and acquisition

The plain shaped equipment recipe is a one-column string above leather and yields one empty bundle.
Its advancement unlocks on possession of string or prior recipe unlock. Each dyed bundle has one
`crafting_transmute` equipment recipe in group `bundle_dye`: exactly one input from `bundles` plus
one matching dye produces that color. Its advancement unlocks on possession of that dye or prior
recipe unlock.

Transmute matching permits exactly one input bundle and the default exactly one dye, rejects every
other occupied ingredient, and rejects a result whose item and components equal the input. A bundle
therefore cannot be recolored to its current color. Assembly applies the original input component
patch to the new result template, preserving ordered contents and other patched components across
every genuine color change. Generic grid allocation, ingredient consumption, recipe-book state and
output placement retain their recipe/crafting owners.

The plain bundle additionally appears in exactly eight village chest tables:
`village_cartographer`, `village_desert_house`, `village_plains_house`,
`village_savanna_house`, `village_snowy_house`, `village_taiga_house`,
`village_tannery` and `village_weaponsmith`. Each has a final one-roll pool containing one
count-one bundle entry of default weight one and an empty entry of weight two, for probability
`1/3`. No dyed bundle has configured noncrafting acquisition. No scoped identity is furnace fuel,
compostable or emitted by a bundled trade.

**Client projection:**

The tooltip component appears only when tooltip display permits `bundle_contents`. It is 96 pixels
wide, shows empty/full translated text at exact weight zero/at least one, and fills a 94-pixel bar
by `truncate(weight * 94)` clamped to `0..94`. Nonempty contents use a four-column, at-most-three-row
grid. Up to 12 entries are all visible. Above 12, the visible count is `11` when size modulo four
is zero and otherwise `8`, `9` or `10` for remainders one, two or three; the top-left overflow cell
shows the summed item count of all hidden entries. The selected slot receives back/front highlight
sprites and its styled name is shown above the tooltip.

The ordinary inventory durability-style bar is hidden at weight zero, otherwise has width
`min(1 + truncate(weight * 12), 13)`. Its normal ARGB inputs are
`(1,0.44,0.53,1)` and weight at least one selects `(1,1,0.33,0.33)`. Arithmetic-error display uses
weight one.

Every identity has closed, open-back and open-front generated models with matching textures. The
item definition uses the closed model outside GUI context. In GUI, no selection is also closed;
selection composes open back, the selected stack's ordinary item layers, then open front.
Tools & Utilities orders Lead, plain bundle, all sixteen colors in the table order, then Compass.
Ordinary output visibility places the same entries in parent and search tabs.

**Branches and aborts:**

Empty/nonempty and insertable/prohibited input; ordinary, bee-bearing and nested weight; partial/full
capacity; matching/new entry; cursor-versus-slot side; primary/secondary and slot modification;
selected valid/invalid/hidden; held-use first/delayed/empty tick; item-entity destruction;
plain/same-color/different-color crafting; tooltip visibility, content size and GUI context.

**Constants and randomness:**

Capacity `1`; nested shell `1/16`; visible grid `4 * 3`; overflow maximum `11`; active duration
`200`, delay `10`, interval `2`; tooltip/bar widths `94/13`; item IDs `1065..1081`; sound IDs
`237..240`; eight independent village bundle pools at `1/3`. Only sound pitch consumes RNG here.

**Side effects:**

Bundle and source/cursor stacks, menu dirty/broadcast state, selected index, item-used stats,
dropped or released item entities, recipe/unlock/inventory state, sounds, tooltip/model selection
and client-first selection state.

**Gates:**

Bundle tag and component presence; `canFitInsideContainerItems`; exact fractional capacity;
item/component merge identity; slot take/place/modification hooks; active player use; handler-time
menu/slot; recipe/tag/dye snapshots; item-entity damage; tooltip display and render context.

**State read/written:**

Reads ordered item templates, cached/recomputed weight, selection, source/cursor/slot stacks, player
menu/inventory/use state, recipe and loot snapshots, client hover/scroll/render state. Writes bundle
contents/order/selection, source/cursor/slot and inventory/drop state, menu changes, stats, item
entities and client-local projection state.

**Failure behavior:**

Prohibited or overweight insert transfers zero and uses the applicable failure sound; rejected
slot modification never mutates contents; partial slot insertion reinserts its remainder; empty
removal/use has no removal/drop sounds or stat; invalid selection clears or no-ops at the stated
boundary; same-color transmute does not match; arithmetic-invalid mutable construction clears
contents; tooltip arithmetic error renders no content tooltip.

**Persistence boundary:**

The ordered item list and every nested stack component persist and stream through the bundle
component. Selected index, active-use progress, scroll accumulation, sound RNG and crafting/loot
draws do not persist. Save/load, component network decode and other list-only reconstruction reset
selection to `-1`. Data reload atomically replaces recipes, their unlock advancements, village loot
and bundle-tag membership without mutating existing stack contents or changing code-built capacity,
click/use behavior, default components or resource-pack-selected assets.

**Boundary cases and quirks:**

Bundle nesting is allowed and pays a `1/16` shell cost. A nonempty beehive costs a full bundle
regardless of its normal maximum. One stored entry may contain many units and is removed as a whole.
New inserts reorder index zero without preserving selected identity. Hidden but existing entries can
be selected by a direct valid packet. Selection changes equality neither locally nor over the wire.
Held use can emit many complete entries but awards the used-item stat once per successful emission.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-DATA-001`; `OFF-REPORT-001`;
`net.minecraft.world.item.Items`;
`net.minecraft.world.item.BundleItem`;
`net.minecraft.world.item.BundleItem#overrideStackedOnOther(net.minecraft.world.item.ItemStack,net.minecraft.world.inventory.Slot,net.minecraft.world.inventory.ClickAction,net.minecraft.world.entity.player.Player)`;
`net.minecraft.world.item.BundleItem#overrideOtherStackedOnMe(net.minecraft.world.item.ItemStack,net.minecraft.world.item.ItemStack,net.minecraft.world.inventory.Slot,net.minecraft.world.inventory.ClickAction,net.minecraft.world.entity.player.Player,net.minecraft.world.entity.SlotAccess)`;
`net.minecraft.world.item.BundleItem#onUseTick(net.minecraft.world.level.Level,net.minecraft.world.entity.LivingEntity,net.minecraft.world.item.ItemStack,int)`;
`net.minecraft.world.item.BundleItem#onDestroyed(net.minecraft.world.entity.item.ItemEntity)`;
`net.minecraft.world.item.component.BundleContents`;
`net.minecraft.world.item.component.BundleContents$Mutable`;
`net.minecraft.world.item.crafting.TransmuteRecipe`;
`net.minecraft.world.item.ItemUtils#onContainerDestroyed(net.minecraft.world.entity.item.ItemEntity,java.util.stream.Stream)`;
`net.minecraft.client.gui.BundleMouseActions`;
`net.minecraft.client.gui.screens.inventory.tooltip.ClientBundleTooltip`;
`net.minecraft.client.renderer.item.BundleSelectedItemSpecialRenderer`;
`net.minecraft.world.level.block.ColorCollection`;
`net.minecraft.world.item.CreativeModeTabs#bootstrap(net.minecraft.core.Registry)`;
`reports/registries.json#minecraft:{item,sound_event}`;
`reports/minecraft/components/item/{bundle,*_bundle}.json`;
`data/minecraft/tags/item/bundles.json`;
`data/minecraft/recipe/{bundle,*_bundle}.json`;
`data/minecraft/advancement/recipes/tools/{bundle,*_bundle}.json`;
`data/minecraft/loot_table/chests/village/{village_cartographer,village_desert_house,village_plains_house,village_savanna_house,village_snowy_house,village_taiga_house,village_tannery,village_weaponsmith}.json`;
`assets/minecraft/items/{bundle,*_bundle}.json`;
`assets/minecraft/models/item/{bundle,*_bundle,bundle_open_*,*_bundle_open_*,template_bundle_open_*}.json`;
`ITM-USE-001`; `ITM-CONTAINER-001`; `ITM-CONTAINER-CLICK-001`; `ITM-RECIPE-001`;
`ITM-CRAFT-001`; `ITM-LOOT-001`; `ITM-ADVANCEMENT-001`; `CLI-UI-001`;
`PROTO-PLAY-SERVERBOUND-INVENTORY-AUXILIARY-001`; `EXP-ITM-017`.

**Test vectors:**

Insert every stack-size class, bee-bearing item, empty/nested/full bundle and prohibited container
item at every residual fraction; verify merge/reorder/partial transfer and exact weight. Exercise
both click-override directions across action, slot and modification gates. Select visible/hidden/
invalid indices, insert around selection, scroll/leave/swap, persist and network-decode. Hold for
all 200 ticks, interrupt at every emission boundary and destroy a filled item entity. Craft plain,
same-color and every cross-color pair with patched components; exercise all eight loot pools,
reload boundaries, bars, tooltip sizes, models and tab order.

**Limits:**

Generic container admission/replay, active-use lifecycle, recipe allocation, loot evaluation,
advancement state, item-entity damage, component transport and rendering engines remain with their
cited owners. This leaf owns the seventeen identities, their component arithmetic and ordering,
concrete click/use/destruction callbacks, recipe/acquisition records and bundle-specific projection.
