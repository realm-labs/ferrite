# BLK-001-Owned Item Runtime

`G01-P6-S001` installs the first audited item partition. It covers ten `SourceSpecified` slices
whose generated primary owner is `BLK-001`, containing eighteen concrete item identities.

## Runtime boundary

`ferrite-gameplay::item::runtime` separates four item-owned responsibilities:

- `catalog` maps all eighteen persistent identities to their ten behavior families, Java 26.2 raw
  IDs, slice owners, stack limits, rarity, forced-glint and fire-damage component boundaries;
- `consumption` provides the default Apple, Golden Apple and Enchanted Golden Apple food profiles,
  full-hunger admission and source-ordered status effects;
- `materials` provides exact live-tag dispatch for fuel, beacon payment, Allay duplication, horse
  feeding/temptation, Piglin interest, nuggets, tool/armor repair and trim materials;
- `interaction` provides item-specific Allay duplication, horse food, Zombie Villager cure,
  Iron Golem repair and Apple composting decisions.

The imported `minecraft:item` registry remains persistently ordered by resource identity. Java raw
IDs are therefore explicit adapter metadata and are never inferred from bundle array positions.
Catalog verification resolves persistent identities first, then checks exact family ownership. It
fails closed on missing identities, wrong families or unexpected members of an owned family.

## Cross-owner boundary

These ten leaf rules also enumerate recipes, loot tables, advancements, trades, world-generation
sources and client resources. Those records already have locked identities and provenance in the
content bundle, while their execution is deliberately delegated by the audited rules to the
generic recipe, crafting, loot, trade, advancement, world and client owners in later Phase 6–9
batches. This batch does not duplicate those engines or make their pending joins appear complete.

Likewise, the entity-specific functions here only decide identity-owned admission and constants.
The later entity and mob partitions own entity mutation, persistence, cooldown ticking, conversion,
effects, sounds and event publication. Phase closure tests will compose the decisions with those
transactions.

## Validation

The required test owner is
`crates/ferrite-gameplay/tests/slices/items/blk_001.rs`. It checks:

- exact ownership of ten slices, ten imported behavior families and eighteen item identities;
- all locked Java raw IDs independently from persistent bundle ordering;
- stack, rarity, glint and fire-resistance component boundaries in the local imported bundle;
- Apple admission and both Golden Apple effect sequences;
- exact positive and negative live-tag, material repair, fuel and minecart-fuel dispatch;
- Allay null-factory consumption, horse feeding, ordinary-only Zombie Villager cure,
  damage-gated Iron Golem repair and Apple-only composting.

`cargo ferrite content verify` remains the authoritative artifact-presence and digest gate. The
focused test skips only its local-bundle assertions when legal ignored artifacts are absent.
