# Play Clientbound Recipe-Book Delta Protocol

`G01-P6-F004` implements both packets in
`PROTO-PLAY-CLIENTBOUND-RECIPE-BOOK-001` for Minecraft Java 26.2:

| ID | Identity | Adapter responsibility |
|---:|---|---|
| 63 | `minecraft:place_ghost_recipe` | replace ghost guidance for one exact current menu |
| 75 | `minecraft:recipe_book_remove` | remove session-local display IDs and refresh recipe UI |

Neither packet has a state ID, generation token, or acknowledgement role.

## Wire grammar

ID 63 carries a signed container VarInt and a complete registry-aware `RecipeDisplay`. It reuses
the five display codecs and all nested slot/item/component/trim registries already shared with the
initial recipe projection. Unknown display or nested registry IDs, malformed nesting, and residual
bytes fail closed.

ID 75 carries a generic signed-VarInt count followed by signed display-ID VarInts. Negative list
counts and truncated/overlong forms fault, while every individual display ID—including negative
values—reaches semantic lookup unchanged. Decoder allocation pre-sizing is capped at 65,536 while
the packet body remains the actual count bound.

## Session-local display identity

`RecipeDisplayIndex` rebuilds the feature-filtered display list in source traversal order. Disabled
displays are skipped and retained displays receive contiguous IDs from zero. One normalized parent
recipe may own multiple IDs. Negative and out-of-range lookup returns no entry.

These integers are valid only for the current recipe-manager generation. Ferrite retains parent
recipe identities and normalized display payloads; it never persists or compares display IDs
across reloads or sessions.

## Client projection

Ghost application requires exact current-container equality and a current screen implementing the
recipe-update listener. Failure is a silent no-op. Success replaces the prior decoded display
without changing menu slots, recipe knowledge, or sending a response.

Removal processes every supplied ID in wire order, deleting exact known and highlighted entries;
missing and repeated IDs are no-ops. After the complete list, including an empty list, the client
performs exactly one collection rebuild and one search-tree refresh, plus one screen callback when
the current screen is a recipe listener.

## Server publication

On an admitted placement that cannot craft, the bounded publisher returns nonempty crafting inputs,
clears the grid, and only then emits ID 63. Ordinary container deltas remain a later independent
convergence path.

Authoritative removal is keyed by normalized parent recipe. Removing a known parent also clears its
highlight and collects every current display ID owned by that parent. ID 75 is omitted when no
display IDs were collected, and the returned removal count is the number of displays rather than
the number of parent recipes.

Raw display/type IDs, container IDs, local highlights, ghost slots, search collections, and screen
state remain connection-local adapter data.

## Evidence

`crates/ferrite-protocol/tests/c3/play_clientbound_recipe_book.rs` owns both goldens, all five
display dispatches, malformed bounds, generation-local ID mapping, ghost gates, ordered
known/highlight removal, empty-list refresh, canonical publisher ordering, and end-to-end
codec/client convergence.
