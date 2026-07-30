# G01-P6-F004 Play Clientbound Recipe-Book Report

## Result

Ferrite implements and verifies IDs 63 `place_ghost_recipe` and 75 `recipe_book_remove`, completing
`PROTO-PLAY-CLIENTBOUND-RECIPE-BOOK-001`. Existing C1 display codecs are reused; new logic owns only
delta semantics, ephemeral display indexing, UI projection, and publication order.

## Verified boundaries

- Both packet goldens lock signed fields and exact body order.
- Structured ghost round trips dispatch all five recipe-display types through existing nested
  registry codecs; unknown types, negative counts, truncation, and residual data fail closed.
- Feature filtering produces contiguous generation-local display IDs and preserves multiple
  displays per normalized parent recipe.
- Ghost application requires both exact container equality and a recipe-listener screen, replaces
  only GUI guidance, and emits no response.
- Removal preserves wire order, removes exact known/highlight IDs, ignores duplicates and missing
  entries, and refreshes collections/search/screen exactly once even for an empty list.
- Server removal maps known parent recipes to every current display ID, clears parent highlights,
  returns display count, and omits empty packets.
- Failed placement returns inputs and clears the grid before ghost publication; the end-to-end
  codec/client trace converges ghost and removal without an acknowledgement.

## Evidence

- `crates/ferrite-protocol/tests/c3/play_clientbound_recipe_book.rs`
- `docs/development/protocol-play-clientbound-recipe-book.md`

The batch acceptance gate is `cargo ferrite task check` followed by `git diff --check`.
