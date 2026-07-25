# Items mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ITM-KNOWLEDGE-BOOK-001` — Knowledge books consume before atomically resolving their ordered recipe-key list

**Parent:** `PLY-005`, `PLY-006`, `PLY-INPUT-001`, `PLY-INTERACT-001`, `ITM-001`,
`ITM-002`, `ITM-003`, `ITM-004`, `ITM-005`, `ITM-006`, `ITM-USE-001`,
`ITM-CONTAINER-001`, `ITM-RECIPE-001`, `ITM-ADVANCEMENT-001`,
`ITM-BOOKSHELF-001`, `CLI-001`, `CLI-006`, `CLI-UI-001`, `CLI-EFFECT-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked registration/components, item-use bytecode, recipe-manager and server
recipe-book bytecode, direct item tag, registry report and client assets determine the complete
knowledge-book transaction. The generic recipe-book display protocol, advancement evaluator,
statistic projection, stack codec and chiseled-bookshelf mechanics remain with their cited owners.

**Applies when:**

A `knowledge_book` stack is created or patched, used from either hand on either logical side,
inserted into or removed from a chiseled bookshelf, persisted, synchronized, rendered, or used
before and after recipe/tag/resource reload.

**Authoritative state:**

`minecraft:knowledge_book` is raw item ID `1337`, epic, nondamageable, max stack `1`, and directly
belongs to `minecraft:bookshelf_books`. Its registered prototype has
`minecraft:recipes=[]`; that component is raw data-component-type ID `66`. Otherwise it has the
common empty modifiers/enchantments/lore, item-break sound, translated name, direct item-model key,
repair cost, swing animation, tooltip display and use effects. It has no consumable, food or
use-remainder component.

The recipes component is a persistent, network-synchronized ordered list of recipe resource keys.
Its codec validates key syntax, not membership in the live recipe manager: the list may be empty,
contain duplicates, name special or already-known recipes, or contain a syntactically valid
missing key. The registered empty list is used when the component is absent.

There is no standard recipe, advancement reward, loot table, trade or mob source that creates a
knowledge book, and it is emitted by no locked creative tab, including Operator Utilities. Normal
play therefore requires an externally supplied stack/component; commands, custom loot or other
administration use the ordinary stack-component boundary.

**Transition and ordering:**

Use reads the live hand stack, snapshots its recipes component or empty default, then immediately
calls `stack.consume(1, player)`. Survival loses one item before any validation. An
infinite-material player retains it. No active-use timer, sound, particle, game event, criterion or
remainder is started.

An empty list then returns `FAIL` on both projections. This includes the registered default and a
stack with the component removed. The failure does not restore a survival item.

A nonempty client list returns `SUCCESS` immediately after predicted consumption. The client does
not resolve keys, mutate its recipe book or award a statistic in this method. Server authority
instead performs an ordered validation pass:

1. Allocate a result list sized to the component list.
2. For each key in list order, query the current server `RecipeManager`.
3. Append every present `RecipeHolder`.
4. On the first absent key, log `Invalid recipe: <key>` at error level and return `FAIL`.

The server does not call `awardRecipes` until the entire list resolves. Thus one missing key
prevents every preceding/following recipe unlock, packet and recipe-unlocked criterion from this
use, but consumption has already happened. Keys after the first miss are not queried or logged.

When every key resolves, the server passes the complete ordered holder list to
`Player.awardRecipes`, ignores its returned count, awards the knowledge-book `ITEM_USED` statistic,
and returns `SUCCESS`. The stat is awarded even when every recipe was already known, duplicated or
special and therefore no new display is added.

For a server player, recipe-book insertion iterates the resolved holders in order. A holder is
skipped when its namespaced key is already known or its recipe reports special. Every remaining
holder is added to the durable known and highlight sets, each current display for that parent key
is appended in resolver order, and `recipe_unlocked` is triggered before the next holder. Duplicate
keys therefore unlock at most once. If at least one display was appended, one
`recipe_book_add` packet carries the accumulated display entries with `replace=false`; otherwise
no add packet is sent. Recipe/display packet fields and client replacement/extension behavior are
owned by the protocol recipe-book family.

**Bookshelf join:**

Direct membership in `bookshelf_books` makes this identity admissible to all six chiseled
bookshelf slots. Block dispatch takes precedence when the shelf handles the click: a free front
slot inserts one book through `ITM-BOOKSHELF-001` rather than invoking recipe unlock. Occupied,
off-front, empty-hand, infinite-material, comparator, save/reload and removal behavior are exactly
that owner's transaction. Reloaded item tags can remove this admission without changing knowledge
book use.

**Persistence and reload boundary:**

Stack persistence retains item identity, count and the exact ordered recipe-key list. Player
recipe-book known/highlight sets and item-used statistics persist through their generic owners;
client display entries are rebuilt/synchronized through the recipe-book protocol on connection.
The consumed item itself carries no continuation state.

Recipe data reload replaces the manager and parent-to-display resolver. A persisted key list is
not rewritten or prevalidated: the next use resolves every key against the new map, so a formerly
valid key can become the first missing-key failure and a formerly missing key can become valid.
Already-known keys remain governed by the player's recipe-book persistence. Item-tag reload
independently controls shelf admission, and resource reload independently controls rendering.

**Client and wire projection:**

The item stack/component patch projects raw item ID `1337` and, when patched from the prototype,
data-component ID `66` with its registry-aware recipe-key list through the generic item-stack
codec. A nonempty list is therefore available for the client's success prediction. Server recipe
unlock projects only newly admitted nonspecial displays through `recipe_book_add`; the component
keys themselves are not recipe display IDs and must never be substituted for them.

The direct item definition selects generated model `minecraft:item/knowledge_book` with texture
`minecraft:item/knowledge_book`. The default name is the epic-colored localized Knowledge Book;
the recipes component adds no built-in tooltip lines. No vanilla tab contains the item, although
search/inventory UI can render an externally supplied stack using the same model.

**Branches and aborts:**

Hand/side/ability/count; absent/empty/nonempty component; every ordered key present/missing,
duplicate, special and already known; recipe/display resolver output; criterion reentrancy;
bookshelf tag/face/slot; stack/player persistence; recipe/tag/resource reload; stack/component,
recipe-book, statistic and model projection.

**Constants and randomness:**

Item raw ID `1337`; component raw ID `66`; max stack `1`; registered list length `0`; exactly one
prevalidation consume attempt. There is no subtype RNG, timer or effect cadence. Recipe collection
and display ordering are the component and current resolver orders.

**Side effects:**

Hand count; one possible error log; server recipe known/highlight sets; recipe-unlocked criteria;
one optional recipe-book-add packet; item-used statistic; bookshelf contents/state/events under its
owner; durable stack/player data; direct item model/name.

**Gates:**

Generic interaction/cooldown admission; player ability for consumption; nonempty list; every key
present in the current recipe manager; server side for unlock/stat; recipe not already known and
not special for each actual unlock; nonempty display accumulation for a packet; direct item tag
plus shelf geometry/occupancy for storage; valid stack codec and client resource.

**State read/written:**

Reads hand stack/count/components, player ability, logical side, current recipe manager and display
resolver, player recipe known/highlight sets, shelf tag/state and client resources. Writes only the
hand count, log, recipe-book/advancement/statistic, optional shelf, persistence and client state
listed above.

**Failure behavior:**

Empty component and first missing recipe return `FAIL` after ability-sensitive consumption.
Missing-key validation produces no partial recipe unlock despite its already-built local prefix.
Client nonempty prediction cannot see server recipe-map failure and may report success until
authoritative inventory/recipe state converges. The ignored `awardRecipes` count cannot turn a
fully resolved use into failure. No failed branch restores the stack or awards the item-used stat.

**Boundary cases and quirks:**

The default epic item self-destructs unsuccessfully in survival because its registered recipe list
is empty. A fully valid list containing only special/already-known recipes still succeeds and
awards item use without an add packet. One missing key after valid entries consumes the book but
unlocks none; only that first miss is logged. Duplicate keys survive stack serialization but can
unlock only their first not-yet-known occurrence. The bookshelf tag admits this operational item
as an ordinary stored book without activating it.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-DATA-001`; `OFF-REPORT-001`;
`net.minecraft.world.item.Items`;
`net.minecraft.world.item.KnowledgeBookItem`;
`net.minecraft.world.item.ItemStack#consume`;
`net.minecraft.core.component.DataComponents`;
`net.minecraft.world.item.crafting.Recipe`;
`net.minecraft.world.item.crafting.RecipeManager#byKey`;
`net.minecraft.server.level.ServerPlayer#awardRecipes`;
`net.minecraft.stats.ServerRecipeBook#addRecipes`;
`net.minecraft.world.level.block.ChiseledBookShelfBlock`;
`reports/registries.json#minecraft:{item,data_component_type}`;
`reports/minecraft/components/item/knowledge_book.json`;
`data/minecraft/tags/item/bookshelf_books.json`;
`assets/minecraft/items/knowledge_book.json`;
`assets/minecraft/models/item/knowledge_book.json`;
`assets/minecraft/textures/item/knowledge_book.png`;
`ITM-BOOKSHELF-001`; `ITM-RECIPE-001`; `ITM-ADVANCEMENT-001`;
`protocol/play-clientbound.md`; `protocol/ordering-and-acknowledgements.md`;
`CLI-UI-001`; `CLI-EFFECT-001`; `EXP-ITM-035`.

**Test vectors:**

Use absent/removed/empty and ordered nonempty lists in both hands, projections and ability modes at
count boundaries. Cross first/middle/last missing keys, all-present lists, duplicates, special and
already-known recipes, zero/one/many display entries and criteria that unlock later keys. Capture
consumption, validation/log order, known/highlight state, criteria, stats and packet accumulation.
Persist/reload stacks and player books, then add/remove/change recipes, displays, bookshelf tag and
resources; exercise every shelf slot and capture exact component, model, name and no-tab
projection.

**Limits:**

This leaf does not duplicate generic stack/data-component encoding, recipe parsing/display
construction, advancement rewards, statistic batching, recipe-book packet layouts, chiseled
bookshelf storage or client inventory correction. Those remain with the cited owners; this rule
fixes the knowledge-book identity and its consume-before-validate recipe-list join.
