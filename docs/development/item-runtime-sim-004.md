# SIM-004-Owned Material Item Runtime

`G01-P6-S012` implements the 15 audited material item slices primarily owned by `SIM-004`. The
partition closes 15 imported catalog families and completes the Goal 01 item-slice denominator.

## Runtime boundary

`item::runtime::sim_004` separates the partition by responsibility:

- `catalog` owns the exact item identities, protocol IDs, imported family closure, and common
  64-stack defaults;
- `joins` owns recipe, advancement, direct-unlock, non-source-block acquisition, brewing, trim,
  merchant, and decoded-template cardinalities;
- `loot` owns material-specific ore, Gravel, Glowstone, foliage, animal, Slime, Panda, and
  Looting arithmetic;
- `firework` owns Firework Star explosion components, base/fade/Rocket special-recipe
  classification, ordered component copying, Rocket lifetime/damage inputs, and inventory tint;
- `brewing` owns the ten Glowstone, fourteen Redstone, Potion-to-Splash Gunpowder, and
  Awkward-to-Turtle-Master Helmet edges;
- `dried_kelp` owns fast-food admission, loose/block Composter probabilities, three cooking
  records, the exact `4001`-tick block fuel, fire odds, and Butcher exchange;
- `materials` owns live role/tag admission, trim providers, repair targets, Lapis enchantment
  consumption, Stick fuel, Leather baby-Piglin rejection, and Slime-Ball food gates;
- `turtle` owns the one-shot negative-to-nonnegative adulthood boundary, Seagrass acceleration,
  Turtle Helmet repair arithmetic, and dry-eye Water Breathing refresh.

Generic loot-table selection and delivery, explosion decay, crafting-grid transactions, brewing
stand timing, merchant selection/economy, anvil commits, entity AI, world generation, Region
delivery, persistence, protocol codecs, and rendering remain with their dedicated owners. The
material modules provide exact deterministic inputs and transition descriptions to those systems.

## Exact arithmetic and ordering

Diamond, Emerald, Lapis, Quartz, and Redstone Ore profiles preserve their source block/item/state
IDs, tool tiers, base counts, XP ranges, cooking XP, and configured sizes. Ordinary ore Fortune
uses the source multiplier draw, while Redstone uses a uniform additive bonus. Silk returns the
Ore block and suppresses both material output and XP before any explosion decay. Gravel likewise
selects Silk before its whole-choice explosion gate; Glowstone instead adds Fortune, clamps to
four, and then applies per-item decay.

Living-attacker Looting is evaluated only on paths that own it. Juvenile Chickens emit no
Feather, Hoglin Leather uses a smaller base range, Frog-caused size-one Slime death guarantees
one Ball and bypasses Looting, and larger Slimes emit none. Leaf Stick chance is strictly less
than the level-specific threshold and is suppressed before its draw by Shears or Silk Touch.

Firework base crafting requires exactly one Gunpowder and at least one component-bearing dye,
permits one shape, trail, and twinkle modifier, and preserves dye encounter order. Fade crafting
preserves unrelated target patches, replaces fade colors, and synthesizes the default explosion
for a componentless Star. Rocket crafting consumes componentless Stars but copies only present
explosion records in row-major order. A Rocket with no copied explosion has no explosion-damage
transaction.

## Determinism and Region ownership

The authoritative Region supplies ordered grid cells, loot contexts, explicit bounded draws,
live tag membership, and entity predicates. No material transition draws from ambient process
state or depends on node placement. Ordered colors, fade lists, Rocket explosion lists, potion
edges, catalog families, and source/sink records remain stable across local and distributed
execution.

## Validation

`crates/ferrite-gameplay/tests/slices/items/sim_004.rs` verifies all 15 imported families and
identities, closed join counts, ore and source-block arithmetic, animal and foliage loot,
material roles, Dried Kelp, every Firework component transition, brewing edges, Turtle adulthood
and Helmet boundaries, and representative exact merchant sets.
