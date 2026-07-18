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
| damage type / enchantment / dimension type | 98 | bundled server data |
| environment attribute | 48 | `reports/registries.json` plus bundled biome data |

The current structural coverage checks 8,894 IDs. Its output separately reports IDs classified as `Unreviewed`; structural coverage is not behavioral readiness. All 25 menu IDs are explicitly classified to the source-specified slot layout, quick-move route and control transaction in `ITM-CONTAINER-*`; no menu catch-all remains. All 21 recipe-serializer IDs are explicitly assigned to the shaped/shapeless, component-special, cooking, stonecutting or smithing algorithms in `ITM-RECIPE-SERIALIZER-001`. All 49 block-entity types inherit the audited generic lifecycle but remain `Unreviewed` for subtype logic. All nine ticket types are explicitly divided by their simulation flag. `random_tick_speed`, the fire-spread radius and the fire burnout attribute are audited; the remaining rules/attributes remain explicit backlog. Registry entries outside these gameplay categories remain discoverable in `registries.json` and must receive a scoped completion entry before the manual can be declared complete.

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
