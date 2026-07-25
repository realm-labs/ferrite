# Items mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ITM-BOOK-FAMILY-001` — Four Book identities split writing, enchanting and cloning while joining shelves, lecterns, loot and Librarian trades

**Parent:** `PLY-005`, `PLY-006`, `PLY-INPUT-001`, `PLY-INTERACT-001`,
`ITM-001`, `ITM-002`, `ITM-003`, `ITM-004`, `ITM-005`, `ITM-006`,
`ITM-007`, `ITM-USE-001`, `ITM-CONTAINER-001`, `ITM-RECIPE-001`,
`ITM-RECIPE-SERIALIZER-001`, `ITM-CRAFT-001`, `ITM-LOOT-001`,
`ITM-ADVANCEMENT-001`, `ITM-ENCHANT-001`, `ITM-GRINDSTONE-001`,
`ITM-ANVIL-001`, `ITM-BOOKSHELF-001`, `BLK-LECTERN-001`, `ENT-001`,
`ENT-PROJECTILE-001`, `MOB-RAID-001`,
`BLK-TRIAL-SPAWNER-001`, `WGEN-PIPELINE-001`,
`WGEN-STRUCTURE-DESERT-PYRAMID-001`, `WGEN-STRUCTURE-JUNGLE-TEMPLE-001`,
`WGEN-JIGSAW-OUTPOST-001`, `WGEN-STRUCTURE-SHIPWRECK-001`,
`WGEN-STRUCTURE-STRONGHOLD-001`, `WGEN-STRUCTURE-OCEAN-RUIN-001`,
`WGEN-STRUCTURE-WOODLAND-MANSION-001`, `WGEN-JIGSAW-ANCIENT-CITY-001`,
`WGEN-JIGSAW-BASTION-001`, `WGEN-STRUCTURE-MINESHAFT-001`,
`WGEN-JIGSAW-TRIAL-CHAMBERS-001`, `WGEN-JIGSAW-VILLAGES-001`,
`CLI-001`, `CLI-006`, `CLI-UI-001`, `CLI-EFFECT-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked registrations, components, item classes, recipes, tags, loot tables,
trade registries, protocol owners and client resources determine the complete Book-family
identity dispatch. Generic enchanting, anvil, grindstone, shelf, lectern, recipe, loot, fishing,
structure, merchant, Villager, protocol, stack and client algorithms remain with the cited
owners.

**Applies when:**

A `book`, `enchanted_book`, `writable_book` or `written_book` stack is crafted, enchanted,
cloned, written, signed, opened, shelved, lectern-mounted, traded, emitted by loot, renamed,
persisted, synchronized or rendered, including arbitrary ordinary component patches and
Trade-Rebalance replacements.

**Authoritative state:**

The locked item registrations are:

| Identity | Raw ID | Class | Rarity | Maximum stack | Identity defaults |
|---|---:|---|---|---:|---|
| `minecraft:book` | `1058` | plain `Item` | common | `64` | enchantable value `1` |
| `minecraft:enchanted_book` | `1274` | plain `Item` | rare | `1` | empty stored enchantments; forced glint |
| `minecraft:writable_book` | `1250` | `WritableBookItem` | common | `1` | empty writable-book content |
| `minecraft:written_book` | `1251` | `WrittenBookItem` | common | `16` | forced glint; no written-content default |

All four also receive the common name, direct item-model key, empty attribute modifiers,
enchantments and lore, repair cost, item-break sound, swing animation, tooltip display and use
effects. The defaults above are identity-significant: a default Written Book has no
`written_book_content`, while a default Writable Book has an empty `writable_book_content`.

`bookshelf_books` contains all four plus Knowledge Book. `lectern_books` contains only Writable
and Written Book. `book_cloning_target` contains Writable Book. Tag reload can change those three
consumer admissions without changing exact-identity branches.

**Transition and ordering:**

Recipes and recipe progression:

- shapeless `book` consumes three Paper and one Leather and creates one default Book;
- shaped `bookshelf` consumes planks/books/planks rows, including exactly three Book;
- shaped `enchanting_table` uses `" B "`, `"D#D"`, `"###"` for Book, Diamond and Obsidian;
- shapeless `writable_book` consumes one Book, Ink Sac and Feather and creates one default
  Writable Book;
- special `book_cloning` uses one Written Book source and occupied material slots matching
  `book_cloning_target`.

Paper possession or the Book recipe itself unlocks `book`. Exact Book possession independently
unlocks Bookshelf, Chiseled Bookshelf, Lectern and Writable Book recipes. Enchanting Table instead
unlocks from Obsidian, not Book. Chiseled Bookshelf and Lectern consume planks/slabs or slabs plus
Bookshelf, not a loose Book.

Book cloning requires at least two occupied crafting slots, exactly one Written Book carrying
`written_book_content`, generation admitted by the recipe codec, and at least one occupied
material slot. A second source, unrelated occupied slot, componentless Written Book or
disallowed generation rejects the whole grid. Vanilla's recipe admits source generation `0..1`;
the codec can encode a subset of `0..2`.

Each occupied material slot counts once regardless of stack count. Assembly copies the source
components through the transmute helper into Written Book, sets result count to the number of
material slots and replaces content with `craftCopy()`: title, author, pages and resolved flag are
preserved while generation increases by one. Vanilla therefore maps `0/1` to `1/2`. A configured
generation-`2` source can produce generation `3`; generation `3` cannot be admitted. The remaining
items transaction returns one source Written Book unchanged and consumes the blank materials.
Generic special-recipe placement, preview and take semantics remain with
`ITM-RECIPE-SERIALIZER-001` and `ITM-CRAFT-001`.

Writing, signing and opening:

Writable content is a list of at most `100` strings, each at most `1024` units, in both persistence
and stream codecs; its empty value has zero pages. Written content contains title length `0..32`,
an unrestricted persistence author string, generation `0..3`, pages defaulting to empty and
resolved defaulting false. Each persisted page uses the restricted flat component codec with
limit `32767`; the persisted and streamed page-list codecs impose no list-count cap. The stream
uses title UTF `32`, the default string codec for author, VarInt generation and the standard
component stream codec.

Using either subclass reads the held stack, calls `openItemGui(stack, hand)`, awards the
item-used statistic and returns success on both logical sides. Local Writable Book handling opens
the editor only when writable content exists. Canonical-server Written Book handling sends an
open-book packet only when written content exists: it first resolves unresolved pages and
broadcasts menu changes when resolution installed new content, then sends the hand. A default
componentless Written Book still succeeds and awards its statistic but sends no open packet.

Resolution attempts every page against the command-source context once. Complete success installs
resolved content and returns true. Any page failure preserves the original page values but marks
the content resolved and returns false, preventing automatic retries.

The client open-book handler rereads the live held stack for the packet's hand. It opens the
Written view when written content exists, otherwise the Writable view when writable content
exists, applying the local text-filter selection; both are view-only. The packet carries only the
hand, so a hand-stack change between server send and client handling changes the displayed
content. Exact edit-book limits, filtering, callback races, slot checks and sign conversion belong
to `PROTO-PLAY-SERVERBOUND-INVENTORY-AUXILIARY-001`; open-book wire and screen dispatch belong to
`PROTO-PLAY-CLIENTBOUND-SPECIAL-SCREENS-001`.

For the exact join, editing accepts a writable-content component rather than exact Writable Book
identity. An absent title replaces writable content. A present title transmutes the current stack
to Written Book, removes writable content and installs literal pages, filtered title, player-name
author, generation `0` and resolved true while preserving other components. The server accepts
only hotbar slots `0..8` or offhand slot `40`; after asynchronous filtering its callback rereads
that slot. It does not require the same hand, screen, stack revision or exact item identity.

Written content supplies tooltip lines: nonblank author emits gray `book.byAuthor`, and every
content-bearing stack emits gray `book.generation.<generation>`. Explicit `custom_name` wins the
hover name; otherwise a nonblank raw Written title becomes a literal hover name on any stack
carrying the component. A componentless Written Book uses its translated item name.

Enchanting, grindstone and anvil joins:

Book's enchantable value `1` admits it to the enchanting table. Offer generation rejects
enchantments outside the table set for Book and, when selection returns multiple entries, removes
one random entry. On commit the table owner recomputes, charges slot-plus-one levels and Lapis,
then `transmuteCopy`s Book to Enchanted Book, preserving count and other components and storing
the selected entries. Successful Book enchanting can trigger `story/enchant_item`; that
advancement's Enchanted Book is only its icon.

Default Enchanted Book lacks an enchantable component and is not table-enchantable. Its stored
entries do not act as ordinary active enchantments. `EnchantRandomlyFunction` and
`EnchantWithLevelsFunction` test exact Book and replace it with a new default Enchanted Book before
storing their choices, losing Book count and component patches; an empty random option leaves the
Book unchanged. `SetEnchantmentsFunction` instead exact-tests Book and uses `transmuteCopy`,
preserving count and unrelated components. Enchantment level output is clamped to `0..255` by
that function.

Grindstone input admits enchantment-bearing stacks. Removing every noncurse from Enchanted Book
produces ordinary Book only when no curse remains; the first input's nonenchantment components
survive the transmute/copy and repair cost resets. Any retained curse keeps Enchanted Book.
Ordinary Book without enchantments cannot enter this baseline branch.

In anvil addition, Enchanted Book supplies stored enchantments and halves each enchantment's unit
cost with floor and minimum `1`; support and compatibility rules still apply. Enchanted Book as
the base bypasses the supported-item gate for merged enchantments. Generic rename applies to all
four identities. Result, level charge, material use and rejection remain with `ITM-ANVIL-001`.

Shelf and lectern joins:

All four identities enter Chiseled Bookshelf through `bookshelf_books`. Enchanted Book alone
selects enchanted insert and pickup sounds; the other three select ordinary book sounds. The
shelf owner preserves the full one-count stack, including content and other component patches,
across its six slots and player transaction.

Only Writable and Written Book enter Lectern through `lectern_books`. Lectern copies one item,
leaves infinite-material players' held count intact, resolves Written content and leaves Writable
content unchanged. Menu, page, comparator, signal, take and persistence behavior remain with
`BLK-LECTERN-001`.

Loot acquisition:

Every row below starts with exact Book. A modifier may replace it with Enchanted Book as specified
above. Weights are per roll within the named pool:

| Base table and pool | Rolls | Total weight | Book-family rows |
|---|---:|---:|---|
| `blocks/bookshelf` | `1` | alternative | Silk Touch returns Bookshelf; otherwise `3` Book, then explosion decay |
| `chests/abandoned_mineshaft`, 0 | `1` | `71` | weight `10`, random enchantment from `on_random_loot` |
| `chests/ancient_city`, 0 | `5..10` | `84` | Swift Sneak EB `3`; random EB `5`; Book `5`, count `3..10` |
| `chests/bastion_other`, 0 | `1` | `89` | Soul Speed EB `10` |
| `chests/desert_pyramid`, 0 | `2..4` | `247` | random EB `20` |
| `chests/jungle_temple`, 0 | `2..6` | `89` | level-`30` EB `1` |
| `chests/pillager_outpost`, 3 | `2..3` | `22` | random EB `1` |
| `chests/shipwreck_map`, 1 | `3` | `38` | Book `5`, count `1..5` |
| `chests/simple_dungeon`, 0 | `1..3` | `144` | random EB `10` |
| `chests/stronghold_corridor`, 0 | `2..3` | `101` | level-`30` EB `1` |
| `chests/stronghold_crossing`, 0 | `1..4` | `62` | level-`30` EB `1` |
| `chests/stronghold_library`, 0 | `2..10` | `52` | Book `20`, count `1..3`; level-`30` EB `10` |
| `chests/underwater_ruin_big`, 1 | `1` | `23` | random EB `5` |
| `chests/village/village_desert_house`, 0 | `3..8` | `36` | Book `1` |
| `chests/village/village_plains_house`, 0 | `3..8` | `43` | Book `1` |
| `chests/woodland_mansion`, 0 | `1..3` | `107` | random EB `10` |

Trial `reward_rare` makes one roll of total `23`; two weight-`2` Enchanted-Book rows select one
uniform option from respectively
`{sharpness,bane_of_arthropods,efficiency,fortune,silk_touch,feather_falling}` and
`{riptide,loyalty,channeling,impaling,mending}`, for combined probability `4/23`.
`reward_ominous_rare` makes one roll of total `29`; three weight-`2` rows select respectively
`{knockback,punch,smite,looting,multishot}`, `{breach,density}`, or fixed Wind Burst I, for combined
probability `6/29`.

Fishing treasure has six equal rows, including level-`30` Enchanted Book. Its full root
probability is `T/(J+F+(open_water ? T : 0)) * 1/6`; at zero luck in open water this is `1/120`,
and outside open water it is zero. Librarian hero gift guarantees one plain Book. Piglin
bartering selects Soul Speed Enchanted Book at weight `5` of `469`. Their fishing, Villager AI and
Piglin transaction semantics remain with their owners.

Trade Rebalance replaces five chest tables. Abandoned Mineshaft preserves the base row and adds a
one-roll total-`5` pool with Efficiency EB weight `1`. Ancient City preserves base rows and adds
one roll of total `80` with Mending EB weight `4`. Desert's base denominator becomes `237` with
random EB weight `10`, then a one-roll total-`7` pool has Unbreaking EB weight `2`. Jungle
preserves its base row and adds one roll of total `2` with Unbreaking EB weight `1`. Pillager
preserves its base row and adds one roll of total `3` with Quick Charge EB weight `2`.

Librarian trades:

Baseline Librarian level tags are ordered sequences. The selected record counts are `2` at levels
`1..4` and `3` at level `5`, without duplicate candidates. Book-family inclusion is:

| Level | Eligible records | Selected | Marginal inclusion |
|---|---|---:|---|
| 1 | Paper buy, random Enchanted Book, Bookshelf sell | 2 | EB `2/3` |
| 2 | Book buy, random EB, Lantern sell | 2 | Book `2/3`; EB `2/3` |
| 3 | Ink Sac buy, random EB, Glass sell | 2 | EB `2/3` |
| 4 | Writable Book buy, random EB, Clock, Compass | 2 | each Book-family record `1/2` |
| 5 | two candle sells | 3 | none |

The Book buy wants `4` Book for one Emerald, maximum uses `12`, reputation discount `0.05` and
merchant XP `10`. The Writable Book buy wants `2` Writable Book for one Emerald, maximum uses
`12`, discount `0.05` and XP `30`.

Each baseline EB offer requires one Book as second cost and gives one Enchanted Book, with maximum
uses `12`, discount `0.2`, and XP `1/5/10/15` at levels `1/2/3/4`. It chooses uniformly among the
`40` expanded `tradeable` enchantments, then uniformly from that enchantment's minimum through
maximum level. For selected level `l`, base Emerald count is
`A = 2 + nextInt(5 + 10l) + 3l`; if the result contains `double_trade_price`, `A` doubles, then
the cost clamps to `1..64`. A modifier result with no stored enchantment makes the candidate null;
the distinct sampler removes it and continues without incrementing successful selections.

Under Trade Rebalance, the level tags are replaced. For the matching Villager biome variant,
levels `1..3` each have three eligible records and select two: EB inclusion is `2/3` at every
level, Book-buy inclusion is `2/3` at level `2`. Level `4` has Writable Book, Clock and Compass
and selects two, so Writable inclusion is `2/3` and EB is absent. Level `5` contains two candle
records plus the matching special EB and selects all three, making EB guaranteed. Six nonmatching
biome records fail their merchant predicate and are discarded while selection continues.

The three common enchantments per variant are:

| Variant | Common choices |
|---|---|
| desert | Fire Protection, Thorns, Infinity |
| jungle | Feather Falling, Projectile Protection, Power |
| plains | Punch, Smite, Bane of Arthropods |
| savanna | Knockback, Curse of Binding, Sweeping Edge |
| snow | Aqua Affinity, Looting, Frost Walker |
| swamp | Depth Strider, Respiration, Curse of Vanishing |
| taiga | Blast Protection, Fire Aspect, Flame |

They use the same random level and `A` formula. Level-five specials are Efficiency III in desert,
Unbreaking II in jungle, Protection III in plains, Sharpness III in savanna, Fortune II in taiga,
Silk Touch I in snow and Mending I in swamp. Their inclusive Emerald ranges are respectively
`11..46`, `8..33`, `11..46`, `11..46`, `8..33`, `10..38` and `10..38`; each also costs one Book,
gives one EB, has `12` uses, discount `0.2` and XP `30`.

**Persistence and reload boundary:**

All four stacks persist identity, count and component patches. Writable and Written content obey
their codecs; open/edit hand, async filter task and recipe/loot/trade cursors do not persist in the
stack. Shelf and Lectern own stored-stack persistence.

Tag reload changes future bookshelf, lectern and cloning admission. Recipe, loot, enchantment,
trade and optional-pack reload changes future evaluations without replaying completed crafts,
offers, loot or page resolution. Trade Rebalance replaces only the chest and Librarian records
listed above. Client resource reload independently controls names, item definitions, models,
textures and screen assets.

**Client and wire projection:**

Generic stack encoding projects raw ID `1058`, `1274`, `1250` or `1251` plus component patches.
Locked English names are `Book`, `Enchanted Book`, `Book and Quill` and `Written Book`.
Enchanted Book is rare and forced-glint; Written Book is common and forced-glint. Written title
can replace its visible name as specified above.

Each direct item definition selects the same-named generated model and texture. Book appears
exactly once in Ingredients between Paper and Firework Star. Writable Book appears exactly once
in Tools & Utilities between Map and Wind Charge. Written Book is absent from ordinary tabs.
Enchanted Book is generated dynamically at the end of Ingredients: one maximum-level book per
enabled enchantment in the parent tab and every level in search-only output. With all locked
enchantments enabled, that is `43` parent entries and `128` all-level search entries.

**Branches and aborts:**

Identity/default/count/component state; all recipe grids/unlocks; cloning source/material count,
generation and leftovers; writable/written codec, resolution and open/edit races; table, loot and
fixed enchantment selection; grindstone curse and anvil roles; shelf/lectern tags; every named
loot pack/table/pool/roll/weight/count/choice; Librarian pack/level/biome/candidate/filter/price;
persistence/reload; raw ID, name, tooltip, glint, model, texture, tab and wire.

**Constants and randomness:**

Raw IDs `1058/1274/1250/1251`; stacks `64/1/1/16`; Book enchantability `1`; writable
pages/count/length `100/1024`; written title/generation/page-component limits `32/0..3/32767`;
clone vanilla source generation `0..1`; `43` enchantments and `128` all-level entries; loot,
trade and price constants exactly as tabled.

**Side effects:**

Default or component-preserving item conversion; recipe knowledge; content edit/sign/resolve and
open-screen packets; statistics; stored enchantments, levels/Lapis and criteria; anvil/grindstone
result; shelf/lectern stack state; loot/table cursor; merchant offer/economy; persistence, wire
and client projection.

**Gates:**

Exact identity or named component/tag as stated; recipe grid and generation; logical side and
content presence; enchantment support/compatibility/curse; shelf/lectern admission; loot context,
optional pack and table conditions; Librarian profession/level/biome/candidate filtering;
registry/component decode and client resource bootstrap.

**State read/written:**

Reads identity, count, patches, writable/written/stored/ordinary enchantment components, recipe
grid/knowledge, player/menu/hand/slot/filter state, shelf/lectern state, loot context, merchant
variant/level/offers and resources. Writes only the stack, content, progression, block container,
loot, merchant, persistence, wire and client state listed above.

**Failure behavior:**

Malformed or out-of-range content fails generic component decode. Invalid clone grids have no
result. Componentless written use sends no open packet; failed page resolution seals original
pages as resolved. Unsupported enchanting/anvil or empty loot enchantment selection keeps or
rejects the source as specified by its owner. Nonmatching shelf/lectern tags reject insertion.
Unselected loot and trade candidates emit alternatives; null trade candidates are removed and
selection continues. Missing client resources cannot grant authority.

**Boundary cases and quirks:**

Written Book defaults without content despite its forced glint. Written title naming applies to
any stack carrying that component. Clone count is occupied blank slots, not blank stack counts,
and the original source remains as a remainder. Two loot enchanting functions discard Book
patches, while table commit and fixed Set Enchantments preserve them. A curse keeps grindstone
output Enchanted Book. The open-book packet carries only a hand. Rebalance special offers are
variant-predicate filtered before the no-duplicate successful-count target is reached.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-DATA-001`; `OFF-REPORT-001`;
`net.minecraft.world.item.Items`; `net.minecraft.world.item.WritableBookItem`;
`net.minecraft.world.item.WrittenBookItem`; `net.minecraft.world.item.ItemStack#getCustomName`;
`net.minecraft.world.item.component.WritableBookContent`;
`net.minecraft.world.item.component.WrittenBookContent`;
`net.minecraft.world.item.crafting.BookCloningRecipe`;
`net.minecraft.world.item.enchantment.EnchantmentHelper#createBook`;
`net.minecraft.world.level.storage.loot.functions.EnchantRandomlyFunction`;
`net.minecraft.world.level.storage.loot.functions.EnchantWithLevelsFunction`;
`net.minecraft.world.level.storage.loot.functions.SetEnchantmentsFunction`;
`net.minecraft.world.entity.npc.villager.AbstractVillager#addOffersFromItemListingsWithoutDuplicates`;
`net.minecraft.world.item.trading.VillagerTrade#getOffer`;
`net.minecraft.world.item.trading.TradeSet#calculateNumberOfTrades`;
`net.minecraft.world.item.CreativeModeTabs`;
`net.minecraft.data.recipes.packs.VanillaRecipeProvider`;
`net.minecraft.data.loot.packs.VanillaChestLoot`;
`net.minecraft.data.loot.packs.TradeRebalanceChestLoot`;
`reports/registries.json#minecraft:{item,recipe,loot_table,advancement,enchantment,villager_trade,trade_set}`;
`reports/minecraft/components/item/{book,enchanted_book,writable_book,written_book}.json`;
`data/minecraft/tags/item/{bookshelf_books,lectern_books,book_cloning_target}.json`;
`data/minecraft/recipe/{book,bookshelf,enchanting_table,writable_book,book_cloning}.json`;
`data/minecraft/loot_table/{blocks/bookshelf,chests/**,gameplay/fishing/treasure,gameplay/hero_of_the_village/librarian_gift,gameplay/piglin_bartering}.json`;
`data/minecraft/{villager_trade,trade_set,tags/villager_trade}/librarian/**`;
`assets/minecraft/{items,models/item,textures/item}/{book,enchanted_book,writable_book,written_book}.*`;
`ITM-RECIPE-SERIALIZER-001`; `ITM-ENCHANT-001`; `ITM-GRINDSTONE-001`;
`ITM-ANVIL-001`; `ITM-BOOKSHELF-001`; `BLK-LECTERN-001`;
`PROTO-PLAY-SERVERBOUND-INVENTORY-AUXILIARY-001`;
`PROTO-PLAY-CLIENTBOUND-SPECIAL-SCREENS-001`; `EXP-ITM-063`.

**Test vectors:**

Exercise default, removed and arbitrary component-patched forms of all four identities through
crafting, clone generations/material layouts/leftovers, edit/sign/filter races, resolution
success/failure, live-hand opening, table/loot/fixed enchanting, grindstone curse outcomes, anvil
merge/rename, every shelf and lectern admission, save/reload and synchronization.

Generate every base and Trade-Rebalance loot row through all rolls, weights, option sets and named
cursors. Generate baseline and rebalanced Librarian offers across every level, variant,
candidate ordering, null modifier, level/price draw, use/restock boundary and transaction. Reload
each data/resource domain and verify raw IDs, content codecs, names/tooltips/glint, models,
textures, exact tab placement and dynamic Enchanted-Book counts.
