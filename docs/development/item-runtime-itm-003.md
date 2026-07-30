# ITM-003 Crafting and Processing Runtime

`G01-P6-S006` implements the audited recipe, crafting, cooking, brewing, Campfire, Crafter and
workstation rules primarily owned by `ITM-003`.

## Runtime boundary

The implementation is divided by gameplay responsibility:

- `recipe` owns the seven recipe domains, the exact 21-serializer catalog, key-sorted reload,
  preferred-recipe lookup, cropped crafting inputs and the Crafter's ten-entry identity-fenced LRU;
- `crafting` owns limited-crafting preview, stored-recipe credit, fresh remainder placement,
  four-tick Crafter triggering, balanced insertion, result-before-remainder delivery and six-tick
  animation;
- `furnace`, `brewing` and `campfire` own their distinct timer, fuel, cancellation, completion,
  cooldown and output-retry transitions;
- `workstation` owns Smithing, Stonecutter, Cartography and Loom transactions, including fresh map
  IDs, same-tick Stonecutter sound coalescing and the closed 43-pattern Loom catalog;
- `item_enchantment`, `grindstone` and `anvil` own shared enchanted-stack state, curse retention,
  repair-cost arithmetic, XP sampling, material and sacrifice repair, ordered enchantment
  application, rename-only cost handling and the strict 12% Anvil damage draw.

The modules expose deterministic semantic decisions. Registry loading, packet codecs, Region event
delivery, inventory-menu synchronization, recipe-display packets, statistics and client
presentation remain with their established owners.

## Deterministic and reload behavior

Recipe preparation sorts by resource key. A valid retained recipe wins before the first matching
key; Crafter cache entries normalize stack counts to one while retaining component equality and
are discarded when recipe-manager identity changes. Empty inputs never enter the cache.

Random behavior stays caller-driven. Furnace fractional XP accepts an explicit float draw,
Grindstone XP accepts one checked bounded draw, and Anvil degradation accepts the audited float
draw directly. This keeps the runtime independent of topology and allows the Region owner to bind
the appropriate named deterministic stream.

Crafting and workstation commits retain source order: manual crafting credits the stored recipe
before applying freshly resolved remainders; Crafter output and remainder deliveries precede input
shrink; Smithing consumes template, base and addition in slot order; Brewing applies the first
ordered container or potion mix to each bottle.

## Validation

`crates/ferrite-gameplay/tests/slices/items/itm_003.rs` locks all seven recipe domains, all 21
serializers, crop/cache behavior, manual and redstone crafting, Furnace/Smoker/Blast Furnace timer
semantics, four-slot Campfire behavior, Brewing, Smithing, Stonecutter, Cartography, all 43 Loom
patterns, Grindstone and Anvil boundaries.
