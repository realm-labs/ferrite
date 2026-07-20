# Blocks mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `BLK-BANNER-001` — Banners own support/pose, component-preserving identity, cauldron layer removal, map markers, and layered rendering

**Parent:** `BLK-001`, `BLK-002`, `BLK-003`, `BLK-007`, `PLY-005`, `ITM-001`, `ITM-003`, `ITM-004`,
`WGEN-004`, `CLI-006`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked server/client source, all 32 block reports, 16 item reports/models, both
banner tags, 16 standing-color loot tables and locked banner-pattern data determine the complete
banner-owned transaction. Generic standing/wall placement, block destruction, recipe/loom commits,
saved-map sampling cadence, component/tooltip policy and rendering-resource admission retain their
named owners; this leaf fixes every banner-specific projection at those boundaries.

**Applies when:**

Any of the 16 colored standing or wall banners is placed, supported, cloned, saved, synchronized,
dropped or rendered; a patterned banner item is washed; a filled map is used on or later validates a
banner marker; or forced respawn tests a banner cell.

**Authoritative state:**

The exact color order is white, orange, magenta, light-blue, yellow, lime, pink, gray, light-gray,
cyan, purple, blue, brown, green, red, black. Each color owns one standing block with 16 `rotation`
states (default 8), one wall block with four horizontal `facing` states (default north), and one
standing item; totals are 32 blocks, 320 states and 16 items. Every block is wood-map-colored,
bass-instrumented, forced-solid, collisionless, lava-ignitable, strength 1 and wood-sounding.
Standing outline is an 8-wide column Y `0..16`; wall outline is a full-width slab Y `0..12.5`, Z
`14..16`, rotated by facing. The block entity owns immutable constructor base color, ordered pattern
layers, nullable custom name and generic residual positive components; it has no ticker.

**Transition and ordering:**

Generic standing/wall item placement selects a supported state, writes it, loads permitted typed
block-entity data, then applies item components. The banner entity subsequently saves/synchronizes
those fields and supplies clone/loot/render/map projections. Water-cauldron use removes one final
layer from a one-count copy before consumption/inventory disposition and fill lowering. Map use
derives color/name from the current entity before equality-toggle, capacity and decoration updates.

**State, support, placement, and forced respawn:**

A standing banner survives exactly when the block below reports `isSolid`; a wall banner tests the
block opposite its facing. Forced-solid banners can therefore support further banners despite having
no collision. Only an update from down for standing, or from the support direction for wall,
rechecks and returns air on failure. Standing placement sets
`rotation = round((playerYaw+180)*16/360) & 15`; rotate/mirror use the generic 16-segment transform.
Wall placement scans nearest-looking directions in order, considers horizontal directions, sets
facing opposite each and returns the first survivable state; rotate/mirror transform facing.
`BannerItem` uses the generic standing/wall selector with attachment direction down, maps both block
variants back to the one color item and exposes that standing block's color. Every banner explicitly
returns true to the forced-respawn free-cell test even though force-solid is true.

**Components, persistence, synchronization, and limits:**

Each default item has max stack 16 and an empty `BANNER_PATTERNS` component. Placement component
application always replaces entity patterns from the item's full component view (therefore default
empty overwrites pattern data loaded only from raw block-entity data), replaces custom name from the
item, and replaces residual stored components with positive nonimplicit patch entries; component
removals are not retained. The block's dye color, not any component, constructs base color. Generic
placement ordering and write/admission failures remain `BLK-PLACE-001`.

Save emits `patterns` only when nonempty, nullable `CustomName`, and generic residual `components`;
load safely parses name, decodes the complete layer list or defaults it to empty, then loads
residual components. Update packet creation and update tag serialize the full
save-without-position/type, including residual components. Component extraction exposes all residual
components plus patterns and custom name and removes the two legacy tag fields afterward. Neither
codec nor entity enforces the declared constant six-layer maximum. Loom and duplication recipes
admit/create at most six through `ITM-LOOM-001`/`ITM-CRAFT-001`, but commands or authored data can
persist more. Item tooltip lists only the first six layers, in order, as gray
`pattern.translation_key + "." + dye_name`; washing can still remove hidden later layers and
rendering can show more.

**Pick and loot:**

With a matching entity, clone/pick ignores the `includeData` Boolean, constructs the mapped color
item and applies every collected positive residual component plus patterns/custom name. A
missing/wrong entity falls back to the plain mapped color item. Wall blocks map to their standing
item. All 16 wall blocks override their standing counterpart's loot table. Each color table makes
one corresponding standing item only through `survives_explosion`, copying exactly custom name, item
name, tooltip display, banner patterns and rarity from collected entity components; other residual
components are clone-visible but not loot-visible.

**Water-cauldron cleaning:**

Only the 16 banner items have this dispatcher. Empty pattern layers return `TRY_WITH_EMPTY_HAND` on
both sides. A nonempty list returns success on both sides; the client changes nothing. The server
copies count one, removes exactly the last layer, then uses `createFilledResult` with creative-size
limiting disabled. Survival count one replaces the hand; a larger stack shrinks by one and adds or
nonrandomly drops the cleaned copy. Infinite-material players retain the original and likewise
receive/drop a new cleaned copy. It then awards only `minecraft:clean_banner`, requests the
water-cauldron level minus one (level one becomes empty cauldron) and emits `BLOCK_CHANGE` carrying
the requested state. Failed state write is ignored; there is no banner-cleaning sound,
`use_cauldron` stat or item-used stat.

**Filled-map banner markers:**

Filled-map use special-cases exact block tag `#minecraft:banners`, containing all 32 IDs. The client
always returns success. The server also returns success when saved map data is missing; with data, a
false toggle returns failure. Toggle uses banner center X/Z relative to map center and `1<<scale`,
requiring both normalized coordinates inclusively within `[-63,63]`; it does not compare clicked
dimension with the map's dimension. A matching entity becomes
`(position,baseColor,optional customName)`: pattern layers and generic/default name are ignored. Its
key is literal `banner-x,y,z`, and color selects the corresponding one of 16 `banner_<color>`
decoration types.

If the map already contains an equal record at that key, remove it and its decoration regardless of
capacity. Otherwise admission requires tracked decoration count not greater than 256, so count 256
can admit a 257th and count 257 rejects even replacement of a stale same-key record. Admission
stores/replaces the record, adds the colored decoration at doubled/clamped coordinates with name,
marks decorations/saved data dirty and returns true. Rotation is byte 8 for ordinary dimensions from
the submitted 180 degrees; when the map's dimension key is literal Nether and a level is supplied,
it instead uses wrapping signed-32-bit `(s*s*34187121+s*121)>>15 & 15` for `s=(int)(gameTime/10)`.
Saved-data reconstruction supplies no level and therefore initially uses 8. During later map pixel
sampling, `checkBanners` removes and dirties any marker at the sampled X/Z whose current
`(position,color,name)` is missing or unequal; pattern-only changes compare equal. The generic map
updater owns when those columns are sampled.

**Client block/item rendering:**

Base and layer tint uses the opaque texture-diffuse palette:
`F9FFFE,F9801D,C74EBD,3AB3DA,FED83D,80C71F,F38BAA,474F52,9D9D97,169C9C,8932B8,3C44AA,835432,5E7C16,B02E26,1D1D21`
in the color order above. Block render state selects standing versus wall model and transformation;
both translate `(0.5,0,0.5)`, scale `(0.6666667,-0.6666667,-0.6666667)` and rotate Y by the negative
segment-degrees or wall `toYRot`. Its phase is `(floorMod(7x+9y+13z+gameTime,100)+partialTick)/100`,
or game time zero without a level. Flag X rotation is `(-0.0125 + 0.01*cos(2PI*phase))*PI`.

Submission draws the base geometry/flag, then a base-color flag layer, then at most the first 16
pattern assets in list order with their dye colors and ordered layer indices. Break overlay applies
to base submissions, not pattern overlays. Layers beyond 16 persist but are invisible. All 16 locked
item models are ground-banner special models with the same translation/scale; they extract
`BANNER_PATTERNS` or use empty, render at phase zero and use the model-fixed item color. The loom
preview likewise renders a phase-zero flag and the same first-16 layer projection.

**Branches and aborts:**

All colors, 320 states and support chains; standing/wall selection, rotations/mirrors and forced
respawn; matching/missing/wrong entity; raw data versus full components and malformed load; residual
added/removed components; layer counts `0/1/6/7/16/17+`; clone versus explosion loot; cauldron
side/count/ability/inventory/write result; map missing data/entity, bounds equalities, equal/stale
marker, counts `255/256/257`, dimension and later validation; level-null/time/position/partial
render and missing assets/settings.

**Constants and randomness:**

State/shape, stack, layer, cauldron, map and renderer constants are above. Banner-owned logic
consumes no RNG. Generic placement/drop/loot/sound and resource/render scheduling own their streams;
Nether map-marker rotation is deterministic game-time arithmetic.

**Side effects:**

State/support writes; forced respawn admission; block-entity fields, dirty/save/update packets;
component-rich pick and restricted loot; item consumption/inventory/drop, clean stat, cauldron
write/event; saved map marker/decorations and dirty sync; client tooltip, item/block/loom rendering.

**Gates:**

Exact color IDs/tags, support `isSolid`, generic placement priority, current block-entity subtype,
component codecs, explosion survival, water-cauldron dispatcher and nonempty layers, logical
side/player ability/inventory, saved map existence/bounds/equality/count, map sampling, client
resources/settings and layer limits.

**Boundary cases and quirks:**

Collisionless banners support banners and are forced-respawn-passable. Item components overwrite
earlier raw pattern/name data. Six layers is an authoring/UI limit, while storage, washing and the
16-layer renderer expose different cutoffs. Pick copies more residual components than loot. Creative
washing duplicates a one-layer-cleaner copy while preserving the original. A map at count 256 admits
marker 257; at 257 it cannot replace a changed same-position marker, though an equal marker can
still be removed. Pattern changes never invalidate map markers.

**Evidence:**

`Confirmed`; `OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-DATA-001`; `OFF-REPORT-001`. Anchors:
`net.minecraft.world.level.block.AbstractBannerBlock`,
`net.minecraft.world.level.block.BannerBlock`, `net.minecraft.world.level.block.WallBannerBlock`,
`net.minecraft.world.item.BannerItem`, `net.minecraft.world.level.block.entity.BannerBlockEntity`,
`net.minecraft.world.level.block.entity.BannerPatternLayers`,
`net.minecraft.core.cauldron.CauldronInteractions#bannerInteraction`,
`net.minecraft.world.item.MapItem#useOn`, `net.minecraft.world.level.saveddata.maps.MapBanner`,
`net.minecraft.world.level.saveddata.maps.MapItemSavedData#toggleBanner`,
`net.minecraft.world.level.saveddata.maps.MapItemSavedData#checkBanners`,
`net.minecraft.client.renderer.blockentity.BannerRenderer`,
`net.minecraft.client.model.object.banner.BannerFlagModel`,
`net.minecraft.client.renderer.special.BannerSpecialRenderer`; locked reports, tags, pattern data,
item models and loot; `EXP-BLK-012`.

**Test vectors:**

Exhaust 32 IDs/320 states, support made from collisionless forced-solid banners, placement
direction/segment and respawn cells; item/raw/component/save/sync/clone/loot matrices with every
component and malformed/overlong layer list; cauldron empty/count/creative/inventory/write cases;
map data/subtype/dimension/name/color/pattern, inclusive coordinates, equal/stale markers and counts
around 256; marker validation cadence; palette, transformations, phase/flag formula, break overlay
and `0/6/16/17` tooltip/render limits. Run `EXP-BLK-012` as the executable matrix.
