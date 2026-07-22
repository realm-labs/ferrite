# Items mechanics

[Back to the leaf-rule manual](../README.md).

Each file contains one implementation-level leaf rule. Stable rule IDs remain the normative
references used by behavior pages, the completion ledger, the catalog, and tests.

## Leaf rules

### [`ITM-USE-001`](itm-use-001.md)

Item use separates start, per-tick use, release, and finish

### [`ITM-LOOT-001`](itm-loot-001.md)

Loot is generated from a context and consumed exactly once by its caller

### [`ITM-ENCHANT-001`](itm-enchant-001.md)

Enchantment behavior is component/effect driven and applies at defined hook sites

### [`ITM-ADVANCEMENT-001`](itm-advancement-001.md)

Advancement criteria are event listeners with requirement-matrix completion

### [`ITM-HUNGER-001`](itm-hunger-001.md)

Exhaustion is spent before regeneration or starvation selects its timer branch

### [`ITM-XP-001`](itm-xp-001.md)

Player experience normalizes progress across piecewise level costs

### [`ITM-CONTAINER-001`](itm-container-001.md)

The server replays predicted clicks before choosing delta or full correction

### [`ITM-CONTAINER-CLICK-001`](itm-container-click-001.md)

Seven input variants form one deterministic cursor/slot state machine

### [`ITM-CONTAINER-MOVE-001`](itm-container-move-001.md)

Quick-move uses a two-pass transfer primitive and a locked per-menu route

### [`ITM-CONTAINER-CONTROL-001`](itm-container-control-001.md)

Menu controls are separate server-validated transactions

### [`ITM-CONTAINER-CLOSE-001`](itm-container-close-001.md)

Closing resolves cursor and transient inputs before returning to inventory menu

### [`ITM-DROPPER-001`](itm-dropper-001.md)

A dropper selects one occupied slot, then inserts one item or ejects it

### [`ITM-ENDER-CHEST-001`](itm-ender-chest-001.md)

Ender-chest items belong to the player while the used block owns only open presentation

### [`ITM-BARREL-001`](itm-barrel-001.md)

A barrel owns 27 slots, materializes loot by caller, and exposes open state

### [`ITM-CHEST-001`](itm-chest-001.md)

Chests pair canonically while each half retains independent storage and opener state

### [`ITM-BOOKSHELF-001`](itm-bookshelf-001.md)

Chiseled-bookshelf interaction, automation and comparator state can diverge

### [`ITM-JUKEBOX-001`](itm-jukebox-001.md)

Jukebox item, song-clock, signal and client playback state are distinct

### [`ITM-HONEYCOMB-001`](itm-honeycomb-001.md)

Honeycomb replaces every unwaxed copper stage before emitting its wax transaction

### [`ITM-RECIPE-001`](itm-recipe-001.md)

Recipe reload produces ordered, type-partitioned lookup domains

### [`ITM-RECIPE-SERIALIZER-001`](itm-recipe-serializer-001.md)

All 21 serializers reduce to audited matching and assembly families

### [`ITM-CRAFT-001`](itm-craft-001.md)

Manual result take revalidates, awards, consumes, and places per-slot remainders

### [`ITM-FURNACE-001`](itm-furnace-001.md)

Furnace-family ticks spend burn time before advancing one validated cook

### [`ITM-CAMPFIRE-001`](itm-campfire-001.md)

Four campfire slots advance independently and eject at their stored deadlines

### [`ITM-BREW-001`](itm-brew-001.md)

Brewing fuel starts a 400-tick same-ingredient transaction over three bottles

### [`ITM-CRAFTER-001`](itm-crafter-001.md)

A crafter rising edge schedules one cached craft and pushes or dispenses every output

### [`ITM-STONECUTTER-001`](itm-stonecutter-001.md)

A selected key-ordered recipe consumes one input per result batch

### [`ITM-SMITHING-001`](itm-smithing-001.md)

Smithing previews the first matching recipe and consumes each occupied role after take

### [`ITM-CARTOGRAPHY-001`](itm-cartography-001.md)

Cartography consumes a map plus one material before post-processing the taken copy

### [`ITM-LOOM-001`](itm-loom-001.md)

Loom selection appends one ordered pattern layer while preserving a reusable pattern item

### [`ITM-GRINDSTONE-001`](itm-grindstone-001.md)

Grindstone commit removes non-curses, resets prior-work cost and grants randomized removal XP

### [`ITM-ANVIL-001`](itm-anvil-001.md)

Anvil preview prices repair, enchantment merge and rename before a level-paid damaging commit
