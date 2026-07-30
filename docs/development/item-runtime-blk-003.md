# BLK-003-Owned Item Runtime

`G01-P6-S003` implements nine audited item slices containing eighteen identities: Baked Potato,
five raw/cooked meat families, Cookie, Pumpkin Pie, the four-book family and Rabbit Hide.

## Runtime boundary

The item catalog adds exact persistent identities, Java raw IDs, family owners, stack limits,
rarity and glint defaults. Partition verification requires nine families and eighteen entries.

Responsibilities remain split:

- `consumption` owns the thirteen food profiles, including Raw Chicken's ordered 0.3-probability
  Hunger effect;
- `food_family` owns six 200/100/600-tick cooking profiles, Wolf healing, the exact Piglin food
  pair, Baked Potato/Cookie/Pumpkin Pie compost chances and Cookie's Parrot poison branch;
- `books` owns bookshelf/lectern/enchant/content roles, enchanted-book sounds and signed-book
  generation progression.

Generic cooking progress, RNG, loot evaluation, recipe allocation, trade selection, advancement,
entity mutation, persistence and projection retain their explicit shared owners.

## Validation

`crates/ferrite-gameplay/tests/slices/items/blk_003.rs` verifies all nine slices, eighteen raw IDs,
the imported family partition, thirteen food profiles, six cooking profiles, animal and composter
dispatch, Cookie's non-food Parrot path and all four Book defaults/roles. The prior two item
partitions run in the same focused suite.
