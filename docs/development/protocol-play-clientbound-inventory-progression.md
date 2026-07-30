# Play Clientbound Inventory and Progression Protocol

Ferrite implements the three packets in
`PROTO-PLAY-CLIENTBOUND-INVENTORY-PROGRESSION-001` for Minecraft Java 26.2:

| ID | Identity | Adapter responsibility |
|---:|---|---|
| 51 | `minecraft:map_item_data` | project one map's decorations and dirty color patch |
| 123 | `minecraft:tag_query` | correlate nullable debug NBT with the latest callback |
| 130 | `minecraft:update_advancements` | update the client advancement tree and progress |

These IDs share no acknowledgement or generation counter. Map IDs, debug transaction IDs, and
advancement identifiers stay in distinct domains.

## Map codec and projection

Map IDs and scale retain signed VarInt and signed-byte values. Decoration lists are optional;
present empty clears the client set while absent retains it. Every decoration resolves a strict
`minecraft:map_decoration_type`, retains signed coordinates, masks rotation to four bits, and may
carry a trusted component name.

The color patch uses width zero as its absent sentinel. A present patch retains unsigned width,
height, and start coordinates plus a packet-bounded byte array without rectangle prevalidation.
The client projection creates missing map data once and keeps that first scale, lock flag, and
dimension thereafter. Decorations replace before patch application.

Patch application reproduces the locked nontransactional traversal: X is the outer loop, Y the
inner loop, the source index is `x + y * width`, and the destination is
`(start_x + x) + (start_y + y) * 128`. Short sources and out-of-range flat destinations fault after
the exact written prefix. X beyond 127 can alias a later row while the flat index remains valid.
Texture refresh occurs only after successful patch application.

The per-player publisher immediately consumes the complete dirty-pixel bounding box. Decoration
sampling evaluates the old wrapping counter modulo five and increments only while decorations are
dirty. A new holder therefore includes decorations on its first dirty opportunity; after being
dirtied again, four empty opportunities precede the fifth sample. Pixel-only packets never wait
for decoration cadence.

## Debug tag correlation

ID 123 carries a signed transaction followed by raw nullable network NBT. END is null; every
nonnull value must be a compound and is parsed with the default 2,097,152-byte quota and depth 512.

The client handler starts at counter `-1`, increments with signed wrapping, and owns only one
pending callback. Starting another query replaces that pending transaction. Only an exact latest
response invokes the callback. Successful return clears it; a throwing callback remains pending,
and stale, duplicate, unmatched, or callback-free responses are ignored.

Canonical block queries respond only with permission, but a permitted missing block entity
explicitly returns null. Entity queries require both permission and a currently resolved entity;
failure in either branch emits no response.

## Advancement wire model

Added holders retain list order and duplicates. Removed identifiers collapse into a set; progress
uses last-value-wins map semantics. Counts are packet-bounded and negative values fault.

Each holder carries an optional parent, optional display, nested UTF requirement groups, and the
telemetry flag. Displays contain trusted title/description components, a shared registry-aware item
stack template, strict task/challenge/goal frame, flags, optional background, and raw floats.
Higher flag bits are ignored. Progress maps criterion names to absent or signed epoch-millisecond
timestamps.

## Client advancement application

Application performs reset, recursive removal, dependency-retried addition, then progress in that
order. Unknown removals/progress and unresolved parent chains follow warning/ignore paths.
Duplicate added IDs replace lookup identity without deleting stale root/task/child nodes, matching
the malformed client topology behavior.

Known progress is normalized to the holder's requirement names: extra criteria are removed and
missing names become unobtained. Empty outer requirements are incomplete; every nonempty outer
group must contain an obtained member. Removal does not erase the separate progress cache, and
reset/removal do not clear the retained selected tab.

Every known progress entry notifies listeners. A non-reset complete entry emits telemetry only
when a client level exists. Toasts additionally require packet show-advancements and display
show-toast. There is no incomplete-to-complete edge check, so repeated complete deltas repeat both
effects.

## Canonical visibility publication

The bounded publisher evaluates the complete tree after dirty visibility. A node becomes visible
when itself or any descendant is complete; otherwise its own rule and at most two ancestors are
examined, with absent display or hidden state hiding and completed display showing. Newly visible
holders carry definitions and fresh progress; newly invisible holders carry removals; dirty
progress is sent only for holders visible after evaluation.

An empty flush emits nothing but still clears the first-packet flag. A nonempty first flush sets
reset, and later tokenless deltas do not. Parent cycles fail the bounded publisher rather than
looping.

## Evidence

`crates/ferrite-protocol/tests/c3/play_clientbound_inventory_progression.rs` owns all three empty
goldens, structured codecs, malformed bounds, collection normalization, map partial writes and
cadence, latest callback behavior, advancement tree/progress quirks, visibility publication, and a
publisher-to-codec-to-client convergence trace.
