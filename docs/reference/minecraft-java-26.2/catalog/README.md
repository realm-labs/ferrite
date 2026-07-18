# Content behavior catalog

The catalog is the second layer of the manual. Algorithms live in [leaf rules](../mechanics/README.md); [`catalog.toml`](catalog.toml) maps every locked content ID to one of those algorithms.

The committed file deliberately does not copy Mojang's registries or data pack. Instead each category records the exact count and SHA-1 of its sorted, newline-terminated ID set. `mc-ref coverage` regenerates the set from the locked official reports/server jar, verifies the snapshot, then requires exactly one classification for every ID. Consequently a catch-all family cannot silently accept content added or removed by an upstream version change.

## Classification meanings

- `BehaviorFamily`: the ID inherits the referenced generic state machine. Its concrete dimensions, components, tags, or values are read with `mc-ref query`.
- `Special`: dispatch reaches explicit control flow that must receive a dedicated leaf rule as the manual deepens. The family references the current controlling rules.
- `DataOnly`: no independent ID-specific control flow was found. The ID parameterizes the referenced algorithm with locked data.
- `Unreviewed`: a temporary, explicit backlog classification. It prevents a broad selector from claiming that unaudited control flow is `DataOnly`; `mc-ref readiness` must reject it before the reference can be complete.

Classification is an implementation lookup, not a claim that two IDs have identical data. For example, all recipe JSON is `DataOnly`, but its serializer chooses the `ITM-CRAFT-001` matching algorithm and the JSON supplies different ingredients/results.

## Locked breadth

| Kind | IDs | Authoritative source |
|---|---:|---|
| block | 1,196 | `reports/blocks.json` |
| block entity type | 49 | `reports/registries.json` |
| fluid | 5 | `reports/registries.json` |
| ticket type / game rule | 68 | `reports/registries.json` |
| item | 1,537 | item component reports |
| entity type | 158 | `reports/registries.json` |
| mob effect / menu / recipe serializer / potion | 132 | `reports/registries.json` |
| recipe / loot table / advancement | 4,628 | bundled server data |
| worldgen entries | 963 | bundled server data |
| density function type | 34 | `reports/registries.json` |
| damage type / enchantment / dimension type | 98 | bundled server data |
| environment attribute | 48 | `reports/registries.json` plus bundled biome data |

The current structural coverage checks 8,943 IDs. Its output separately reports IDs classified as `Unreviewed`; structural coverage is not behavioral readiness. All 25 menu IDs are explicitly classified to the source-specified slot layout, quick-move route and control transaction in `ITM-CONTAINER-*`; no menu catch-all remains. All 21 recipe-serializer IDs are explicitly assigned to the shaped/shapeless, component-special, cooking, stonecutting or smithing algorithms in `ITM-RECIPE-SERIALIZER-001`. All 49 block-entity types inherit the audited generic lifecycle; End gateway now additionally owns its exact transition state while the remaining subtype logic stays `Unreviewed`. All nine ticket types are explicitly divided by their simulation flag. All four dimension types and all 48 environment-attribute IDs now have audited record, declaration, layer, synchronization and consumer-family ownership in `WGEN-DIMENSION-001`; the three portal gamerules, portal blocks and End-gateway state are owned by `WGEN-PORTAL-001`. All 34 density-function type IDs are audited behavior families: 18 pure composition, five normal-noise coordinate, old-blended, End-island, three old/new-generation blend, structure beardifier and five noise-chunk runtime markers. The shared normal-noise evaluator and all 63 parameter records are source-specified/data-only. All 35 locked density-function records are also audited: the three old-blended records parameterize their dedicated evaluator, while the other 32 are data-only generic composition trees with no ID-specific dispatch. All 11 material-condition and four material-rule IDs now own the generic SURFACE predicate, caching and ordered-state algorithms; all seven noise-setting rule trees are audited data-only compositions of that evaluator. Of the 66 biome records, `eroded_badlands`, `frozen_ocean`, and `deep_frozen_ocean` now own their source-coded surface extensions as `Special`; the other 63 remain explicitly data-only records. All four configured-carver records are now bound to the cave, Nether-cave or canyon behavior family and the shared CARVERS source/owner/seed/mask dispatcher; their family-specific geometry remains open in the completion ledger. Within the 963 worldgen entries, all seven world-preset compositions, 63 noise-parameter records and all 35 density-function records are explicitly data-only inputs, while the two multi-noise parameter-list IDs are special source dispatches owned by `WGEN-PIPELINE-001`; remaining worldgen families still use the temporary broad data family until their codec audits land. Registry entries outside these gameplay categories remain discoverable in `registries.json` and must receive a scoped completion entry before the manual can be declared complete.

## Lookup workflow

```sh
cargo run -p mc-reference --bin mc-ref -- query block minecraft:observer
cargo run -p mc-reference --bin mc-ref -- query block_entity_type minecraft:chest
cargo run -p mc-reference --bin mc-ref -- query item minecraft:bow
cargo run -p mc-reference --bin mc-ref -- query fluid minecraft:flowing_water
cargo run -p mc-reference --bin mc-ref -- query ticket_type minecraft:portal
cargo run -p mc-reference --bin mc-ref -- query game_rule minecraft:random_tick_speed
cargo run -p mc-reference --bin mc-ref -- coverage
```

Queries print normalized official properties plus classification and rule IDs. Raw reports and jars remain under `target/mc-reference/26.2/` and are never committed.

Block-item lookup is intentionally more specific than “this item maps to a block.” The catalog distinguishes ordinary, double-high, bed, sign, standing/wall, water-surface, scaffolding, game-master and solid-bucket dispatch. These selectors are locked to the official 26.2 item registrations and resolve before the generic `block_items` selector, so a new or moved special item cannot silently inherit ordinary placement.
