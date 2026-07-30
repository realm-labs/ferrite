# PLY-005-Owned Item Runtime

`G01-P6-S010` implements the 43 audited item slices primarily owned by `PLY-005`. The partition
contains 138 item identities in 44 imported catalog families; Minecart spans the ordinary and
command-block families.

## Runtime boundary

`item::runtime::ply_005` separates the partition by responsibility:

- `catalog` closes imported family ownership and exact identity counts;
- `consumables`, `alchemy`, and `brewing_graph` own food, drink, stew, potion, Dragon's Breath,
  Water block transactions, and the twelve scoped brewing ingredients;
- `bundle` owns exact fractional capacity, ordered entries, transient selection, click overrides,
  held output, destruction, and recoloring admission;
- `projectiles`, `placements`, `vehicles`, and `buckets` own item-selected entity creation and
  subtype transactions for arrows, eggs, armor stands, End crystals, hanging decorations,
  Minecarts, and captured mobs;
- `equipment` owns Nautilus armor, Spear combat, and food-on-a-stick boost behavior;
- `knowledge` owns consume-before-validation and atomic ordered recipe resolution;
- `materials` owns fixed ingredient profiles, repair/fuel/drop arithmetic, exact Pottery Sherd and
  Smithing Template mappings, Trial Key comparison, Nether Star item-entity behavior, and fixed
  recipe joins.

Generic active-use lifecycle, inventory convergence, effect merging, entity motion and admission,
loot evaluation, recipe matching, advancement bookkeeping, merchant selection, Region command
delivery, protocol codecs, and client rendering remain with their dedicated owners.

## Exact transaction properties

Bundle contents retain an ordered stack list while selection is neither persisted nor compared.
Capacity uses reduced exact fractions, including the `1/16` nested-bundle overhead and full-cost
nonempty bee payload. Insertion moves the affected identity to index zero without renumbering the
numeric selection; removal takes a whole entry and clears selection. Invalid arithmetic clears a
mutable reconstruction, while an over-capacity codec value remains representable but admits no
further insertion.

Entity-producing item paths expose their audited mutation order and rejection asymmetries.
Rejected Minecart or mob admission does not restore already committed consumption/events; a
Dragon-owned cloud is selected by encounter order and loses radius before inventory delivery;
creative mob capture still consumes the empty bucket while creative release retains the filled
bucket. Armor Stand, End Crystal, painting/frame, arrow, egg, and bucket outputs retain their
identity-specific collision, timing, damage, and state-transfer boundaries.

Consumables retain exact durations, nutrition, saturation, effect order, probability draws, and
remainders. Potion effects preserve base-before-custom order and scale only finite positive
durations. Suspicious Stew remains always edible and applies its stored effects in order. Knowledge
Book validation consumes first, stops at the first missing key, and awards nothing until every key
resolves.

## Determinism and Region ownership

The authoritative Region owns entity, equipment, effect, vehicle, inventory, bundle, and use
state. These functions produce deterministic transaction descriptions and never use process
identity. Random decisions are passed in as already-consumed draw values or indices, preserving
the named-stream owners and encounter order recorded by the reference.

Catalog arrays, painting candidates, recipe keys, potion effects, bundle entries, and brewing slots
preserve source order. Entity admission remains generation-fenced by the Region runtime, and a
failed admission is interpreted according to the item transaction rather than topology.

## Validation

`crates/ferrite-gameplay/tests/slices/items/ply_005.rs` verifies all 44 imported families, 43
slices, 138 identities, exact catalog closure, every custom state-machine family, brewing graph,
material join, and the intentionally non-rollback ordering described above.
