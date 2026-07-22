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

### [`BLK-VINE-001`](blk-vine-001.md)

Vines add supported faces and spread through a density-bounded random walk
