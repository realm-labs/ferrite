# SIM-004 Block Runtime

`G01-P5-S007` implements the 54 `SourceSpecified` block slices whose primary simulation owner is
`SIM-004`. The result is a protocol-neutral semantics layer in `ferrite-gameplay`. Region
integration owns world reads/writes, scheduled queues, ECS entities, content snapshots, persistence
and projection; it calls these kernels with ordered observations and explicit random results.

## Module ownership

| Module | Audited responsibility |
|---|---|
| `block::material` | Locked static block/item/state IDs, physical/harvest boundaries, material-family loot, and Crafting Table menu constants |
| `block::terrain` | Dirt Path/Farmland support and moisture, dirt substrates, snow-state maintenance, ground spread, tool transforms, Nylium, Moss, Mud, and Packed Ice |
| `block::crop` | Shared crop speed, Wheat/Carrot/Potato/Beetroot, Cocoa, Pitcher, Torchflower, fruit stems, and Sweet Berry Bush |
| `block::plant_growth` | Bamboo, Cactus, Sugar Cane, Cave Vines, Nether Vines, and all eight Saplings |
| `block::mushroom` | Small mushrooms, Nether fungi, huge-mushroom sticky faces, growth geometry, and loot |
| `block::chorus` | Chorus vertical/branch growth and Chorus Fruit teleport constants |
| `block::amethyst` | Budding-Amethyst draw order, direction selection, water retention, stages, and shard boundary |
| `block::aquatic` | Bubble Columns, Frogspawn scheduling/hatching, Tadpole placement, and Lily Pad support/contact |
| `block::snow` | Snow-layer geometry/support/stacking/melt plus Powder-Snow collision, freezing, damage, and fall sounds |
| `block::sponge` | Exact absorption candidate precedence/caps, Wet-Sponge drying, and furnace Bucket conversion |
| `block::decorative` | Cake, Flower Pot, Pumpkin carving, Carved-Pumpkin facing/golem order, and Melon counts |
| `block::copper` | Full-copper weathering, axe transforms, statue pose/comparator state, and Copper-Golem clocks |
| `block::incubation` | Sniffer-Egg interval, crack, hatch, pitch, and yaw decisions |
| `block::lodestone` | Lodestone/POI identities, Compass binding, and lazy tracker invalidation |
| `block::contact_blocks` | Cobweb contact multiplier and deferred movement residue |

The static material directory covers Ancient Debris, Blackstone, Clay, Dripstone Block, End Stone,
Melon, Mossy Cobblestone, Mud, Muddy Mangrove Roots, Nether Bricks, Nether Planks, Netherrack,
Packed Ice, Prismarine, Pumpkin, Resin, Smooth Stone, Sulfur/Cinnabar, Tuff and both workstation
tables. Their recipe, loot-container, trade, structure, world-generation and client-asset joins
remain data selected by the already imported catalog and their generic runtime owners.

## Determinism and transaction boundaries

- Random-tick, bone-meal, loot and entity-placement draws are explicit parameters. Kernels neither
  acquire ambient randomness nor reorder caller-provided observations.
- Strict source comparisons remain strict: Copper weathering rejects equality, Cactus flower
  placement admits its documented equality, mushroom/fungus bone meal rejects `0.4`, and
  Copper-Golem statue conversion admits `0.0058`.
- Results describe semantic effects even where the source ignores a low-level write Boolean.
  Region integration must preserve the documented later sound, event, schedule, neighbor update,
  item mutation or entity creation rather than inventing rollback.
- Reloadable tags and registries are immutable inputs from the active content snapshot. The
  kernels distinguish exact-identity checks from live membership checks.
- Static IDs are process-visible protocol identities. Persistent content continues to use stable
  resource identities rather than these dense numbers.

## Verification

The committed test owner is
`crates/ferrite-gameplay/tests/slices/blocks/sim_004.rs`. Its 33 tests cover all 54 slice owners,
including state and item IDs, direction and age boundaries, ordered alternatives, draw thresholds,
write-independent residue, growth caps, support gates, menu/POI constants and cross-block joins.
