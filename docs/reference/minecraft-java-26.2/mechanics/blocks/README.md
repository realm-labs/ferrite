# Blocks mechanics

[Back to the leaf-rule manual](../README.md).

Each file contains one implementation-level leaf rule. Stable rule IDs remain the normative
references used by behavior pages, the completion ledger, the catalog, and tests.

## Leaf rules

### [`BLK-STATE-001`](blk-state-001.md)

Registered block states are closed, canonical property tuples

### [`BLK-PLACE-001`](blk-place-001.md)

Block-item placement is an ordered, non-atomic commit pipeline

### [`BLK-BREAK-001`](blk-break-001.md)

Player breaking has a separate authoritative progress and harvest transaction

### [`BLK-BREAK-HOOK-001`](blk-break-hook-001.md)

Concrete block break hooks and loot remain content-owned

### [`BLK-UPDATE-001`](blk-update-001.md)

State writes and neighbor/shape propagation are distinct operations

### [`BLK-SPAWNER-001`](blk-spawner-001.md)

An ordinary spawner freezes behind its live rule, then attempts an ordered entity batch

### [`BLK-COMMAND-001`](blk-command-001.md)

Command blocks retain trigger and chain state behind a live dispatch gate

### [`BLK-COMMAND-AREA-001`](blk-command-area-001.md)

Area commands precharge the whole inclusive box

### [`BLK-TRIAL-SPAWNER-001`](blk-trial-spawner-001.md)

Trial spawners detect a cohort, run a bounded encounter, and eject one reward per registered player

### [`BLK-VAULT-001`](blk-vault-001.md)

Vaults maintain a rewarded-player exclusion set and eject a resolved reward in reverse list order

### [`BLK-FALL-001`](blk-fall-001.md)

A falling block transfers block state into an entity and back

### [`BLK-COPPER-GOLEM-STATUE-001`](blk-copper-golem-statue-001.md)

Copper-golem statues preserve identity across pose, water, wax, weather, item, and entity
transitions

### [`BLK-BELL-001`](blk-bell-001.md)

Bells separate immediate ring ingress from queued hearing, shake, resonance, glow, and particles

### [`BLK-ENCHANTING-TABLE-001`](blk-enchanting-table-001.md)

Enchanting tables own menu ingress, custom name, bookshelf particles, and a client-only shared-RNG
book clock

### [`BLK-LECTERN-001`](blk-lectern-001.md)

Lecterns own book insertion, page menus, two-tick pulses, analog output, removal ejection, and
state/content divergence

### [`BLK-BANNER-001`](blk-banner-001.md)

Banners own support/pose, component-preserving identity, cauldron layer removal, map markers, and
layered rendering

### [`BLK-SHELF-001`](blk-shelf-001.md)

Shelves own powered side chains, direct stack swaps, directional occupancy output, and displayed
contents

### [`BLK-DECORATED-POT-001`](blk-decorated-pot-001.md)

Decorated pots own one-stack storage, four faces, shattering, and wobble

### [`BLK-BRUSHABLE-001`](blk-brushable-001.md)

Brushable blocks serialize ten accepted strokes into one exposed archaeology item

### [`BLK-SCULK-SENSOR-001`](blk-sculk-sensor-001.md)

Vibration selection becomes a distance-delayed, frequency-bearing redstone pulse

### [`BLK-JIGSAW-001`](blk-jigsaw-001.md)

Jigsaw edits synchronize a directed connector before optional immediate generation

### [`BLK-TEST-BLOCK-001`](blk-test-block-001.md)

Test blocks turn redstone edges into ordered block-based test outcomes

### [`BLK-CONDUIT-001`](blk-conduit-001.md)

Conduits scan water and frame state before powering players or attacking one wet enemy

### [`BLK-BEACON-001`](blk-beacon-001.md)

Beacons incrementally publish a colored sky beam before refreshing pyramid effects

### [`BLK-SIGN-001`](blk-sign-001.md)

Signs separate support, two-sided text, one editor, applicators, clicks and rendering

### [`BLK-SKULL-001`](blk-skull-001.md)

Skulls retain power, profile data and animation while wither heads can consume a summon pattern

### [`BLK-STRUCTURE-001`](blk-structure-001.md)

Structure blocks edit, cache, persist and project bounded template operations

### [`BLK-STRUCTURE-VOID-001`](blk-structure-void-001.md)

Structure void is an invisible replaceable block and a skip sentinel for captured/template cells

### [`BLK-AIR-001`](blk-air-001.md)

Air, cave air and void air share empty mechanics while retaining source and stack-sentinel roles

### [`BLK-BEDROCK-001`](blk-bedrock-001.md)

Bedrock combines zero break progress with protected entity, environment and generation roles

### [`BLK-REINFORCED-DEEPSLATE-001`](blk-reinforced-deepslate-001.md)

Reinforced deepslate is slowly breakable but identity-immovable, no-drop and tag-protected

### [`BLK-TINTED-GLASS-001`](blk-tinted-glass-001.md)

Tinted glass combines a transparent model and full collision with complete light dampening

### [`BLK-GLASS-001`](blk-glass-001.md)

Glass combines a translucent full collider with skylight propagation and Silk Touch-only loot

### [`BLK-STAINED-GLASS-001`](blk-stained-glass-001.md)

Sixteen stained-glass identities propagate skylight while recoloring beacon sections

### [`BLK-CONCRETE-001`](blk-concrete-001.md)

Sixteen concrete identities are solid dye-colored blocks and paired powder-solidification targets

### [`BLK-CLAY-001`](blk-clay-001.md)

Clay and Clay Ball join block loot, processing, villages, archaeology, tags and lush-cave generation

### [`BLK-COCOA-001`](blk-cocoa-001.md)

Cocoa Beans place a three-age jungle-log crop that joins growth, composting, recipes and natural jungle trees

### [`BLK-TERRACOTTA-001`](blk-terracotta-001.md)

Plain and dyed terracotta join ordinary solid cubes to substrate, trade and worldgen selectors

### [`BLK-GLAZED-TERRACOTTA-001`](blk-glazed-terracotta-001.md)

Glazed terracotta couples horizontal pattern orientation to push-only piston mobility

### [`BLK-QUARTZ-001`](blk-quartz-001.md)

Full quartz blocks join axis placement, processing, mason offers and bastion decoration

### [`BLK-SANDSTONE-001`](blk-sandstone-001.md)

Full sandstone blocks join processing, replacement, surface generation and desert structures

### [`BLK-STONE-VARIANT-001`](blk-stone-variant-001.md)

Granite, diorite and andesite join processing, trades, replacement and world generation

### [`BLK-STONE-BRICK-001`](blk-stone-brick-001.md)

Full stone bricks join processing, infestation hosts, masonry loot and structures

### [`BLK-BEACON-STORAGE-001`](blk-beacon-storage-001.md)

Beacon storage blocks join compacted materials to golems, piglins and generated treasure

### [`BLK-RAW-STORAGE-001`](blk-raw-storage-001.md)

Raw material storage blocks join compacting, piglins and ore-vein generation

### [`BLK-LAPIS-BLOCK-001`](blk-lapis-block-001.md)

Lapis block joins compacting, slow-bouncy selection and mansion decoration

### [`BLK-REDSTONE-BLOCK-001`](blk-redstone-block-001.md)

Redstone block joins constant signal, control-input semantics, compacting and ancient-city centers

### [`BLK-AMETHYST-BLOCK-001`](blk-amethyst-block-001.md)

Amethyst block joins projectile chimes, crystal footsteps, sculk resonance and geode generation

### [`BLK-BUDDING-AMETHYST-001`](blk-budding-amethyst-001.md)

Budding amethyst grows the directional, waterlogged bud and cluster chain

### [`BLK-CALCITE-SMOOTH-BASALT-001`](blk-calcite-smooth-basalt-001.md)

Calcite and smooth basalt join geode shells to replacement, cooking and ancient-city entrances

### [`BLK-DEEPSLATE-001`](blk-deepslate-001.md)

Base deepslate joins axis-aware placement to terrain replacement, ore hosts and ancient-city structure

### [`BLK-DEEPSLATE-MASONRY-001`](blk-deepslate-masonry-001.md)

Deepslate masonry joins crafting, sculk replacement and ancient-city degradation

### [`BLK-DRIPSTONE-BLOCK-001`](blk-dripstone-block-001.md)

Dripstone block joins pointed growth, cave features and acquisition

### [`BLK-SLIME-001`](blk-slime-001.md)

Slime block joins bounce, slow-step drag, piston adhesion, storage recipes and brewing

### [`BLK-HONEY-001`](blk-honey-001.md)

Honey block joins side sliding, piston adhesion, surface carrying, AI exclusions and recipes

### [`BLK-HONEYCOMB-BLOCK-001`](blk-honeycomb-block-001.md)

Honeycomb block joins compacting, sticky equipment and coral sounds

### [`BLK-BRICKS-001`](blk-bricks-001.md)

Bricks join masonry recipes, slow-bouncy equipment and structure palettes

### [`BLK-PACKED-MUD-001`](blk-packed-mud-001.md)

Packed mud joins mud recipes, regular equipment and trail-ruins aging

### [`BLK-MUD-BRICKS-001`](blk-mud-bricks-001.md)

Mud bricks join masonry recipes, slow-bouncy equipment and trail aging

### [`BLK-PURPUR-BLOCK-001`](blk-purpur-block-001.md)

Purpur blocks join End-city palettes, masonry and advancement display

### [`BLK-RED-NETHER-BRICKS-001`](blk-red-nether-bricks-001.md)

Red Nether bricks join masonry, slow-bouncy equipment and the Nether display

### [`BLK-NETHER-BRICKS-001`](blk-nether-bricks-001.md)

Nether Brick joins smelting and bartering to correct-tool masonry, fortress terrain and protected spawn floors

### [`BLK-NETHER-WART-BLOCK-001`](blk-nether-wart-block-001.md)

Nether wart blocks join composting, Nether growth, spawn exclusions and the client tutorial

### [`BLK-WARPED-WART-BLOCK-001`](blk-warped-wart-block-001.md)

Warped wart blocks join composting, warped growth and the client tutorial

### [`BLK-NETHER-SPROUTS-001`](blk-nether-sprouts-001.md)

Nether sprouts survive on tagged substrates and join warped vegetation

### [`BLK-NETHER-ROOTS-001`](blk-nether-roots-001.md)

Nether roots share support, potting, Enderman and forest-vegetation behavior

### [`BLK-NETHER-WART-001`](blk-nether-wart-001.md)

Nether wart grows by random tick and joins brewing, loot and Nether structures

### [`BLK-NETHER-STEM-001`](blk-nether-stem-001.md)

Nether stems and hyphae preserve axis through stripping, log consumers and fungus generation

### [`BLK-CORAL-BLOCK-001`](blk-coral-block-001.md)

Live coral blocks schedule delayed drying while dead coral blocks are terminal

### [`BLK-CORAL-PLANT-001`](blk-coral-plant-001.md)

Coral plants and fans join waterlogging, support and delayed drying

### [`BLK-FLOWER-POT-001`](blk-flower-pot-001.md)

Flower pots own code-built contents, interaction ordering and two filled-form exceptions

### [`BLK-COPPER-FULL-001`](blk-copper-full-001.md)

Full copper blocks own strict weathering, waxing and copper-golem construction

### [`BLK-SAPLING-001`](blk-sapling-001.md)

Ordinary tree saplings stage once, then run the exact small-or-mega tree transaction

### [`BLK-BAMBOO-001`](blk-bamboo-001.md)

Bamboo saplings and stalks share one item but use distinct survival, growth and generation transactions

### [`BLK-ANCIENT-DEBRIS-001`](blk-ancient-debris-001.md)

Ancient debris joins a resistant full cube and fire-resistant item to processing, bastion loot and scattered Nether ore

### [`BLK-STEM-CROP-001`](blk-stem-crop-001.md)

Melon and pumpkin stems mature, choose one fruit side and collapse when that fruit leaves

### [`BLK-MELON-001`](blk-melon-001.md)

Melon blocks turn stem, terrain and structure fruit into edible slices, seeds and Glistering Melon

### [`BLK-OVERWORLD-CROP-001`](blk-overworld-crop-001.md)

Ordinary overworld crops share farmland growth but diverge at beetroot RNG, harvest and item use

### [`BLK-TORCHFLOWER-CROP-001`](blk-torchflower-crop-001.md)

Torchflower crop replaces its second age with a mature flower after an outer growth gate

### [`BLK-PITCHER-CROP-001`](blk-pitcher-crop-001.md)

Pitcher crop becomes double-high at age three and keeps a separate placeable mature plant

### [`BLK-SWEET-BERRY-BUSH-001`](blk-sweet-berry-bush-001.md)

Sweet berry bushes couple four growth stages to harvest, movement damage and animal AI

### [`BLK-CAVE-VINES-001`](blk-cave-vines-001.md)

Cave vines preserve berry state while a downward-growing head becomes body

### [`BLK-CHORUS-001`](blk-chorus-001.md)

Chorus flowers turn connected stems into upward growth, branches, or dead tips

### [`BLK-SOUL-SAND-001`](blk-soul-sand-001.md)

Soul sand joins reduced collision to bubble columns, fire, plants, movement and Nether generation

### [`BLK-MAGMA-001`](blk-magma-001.md)

Magma joins hot-floor damage to downward bubbles, reloadable selectors and generation

### [`BLK-LAVA-CAULDRON-001`](blk-lava-cauldron-001.md)

A full lava cauldron joins bucket dispatch, ordered contact, comparator output and navigation

### [`BLK-TEST-INSTANCE-001`](blk-test-instance-001.md)

Test-instance blocks edit, place, persist and project operator-driven GameTest runs

### [`BLK-VINE-001`](blk-vine-001.md)

Vines add supported faces and spread through a density-bounded random walk
