# ITM-001 Item Runtime

`G01-P6-S004` implements the three audited slices primarily owned by `ITM-001`: shared active item
use, Chiseled Bookshelf transactions and Jukebox playback/storage.

## Runtime boundary

The item runtime now owns a component-bearing `ItemStack` value with explicit object identity.
Zero-count stacks normalize to empty, component fingerprints participate in equality and stacking,
and after-use processing preserves the locked `USE_REMAINDER` then `USE_COOLDOWN` order.

`use_lifecycle` separates base component dispatch from active use. It captures the hand stack and
duration, invokes ticks before decrementing the remaining duration, revalidates the item separately
from full stack equality, models server-only natural completion, and exposes the final update made
by release-driven items.

`bookshelf` owns the six-slot Chiseled Bookshelf boundary:

- exact front-face float section selection for all facings and row/column boundaries;
- captured occupancy interaction decisions and the five-member bookshelf-book tag;
- unclamped public storage, last-slot comparator memory and failed/same-state write behavior;
- raw load, no-update removal and clear paths which intentionally do not reconcile block state;
- automation gates and deterministic replacement-drop RNG accounting.

`jukebox` owns the independent block occupancy, item, active song and padded song-clock states. It
contains all 22 default disc profiles, accepts custom playable components, models start/stop and
20-tick event cadence, exposes comparator versus source-signal divergence, and preserves load,
failed-transfer restart and double-stop removal behavior.

Generic player inventory admission, hopper traversal, item-entity construction, Region event
delivery, persistence codecs and client sound/HUD projection remain with their dedicated owners.

## Validation

`crates/ferrite-gameplay/tests/slices/items/itm_001.rs` locks component dispatch and lifecycle
boundaries, remainder/cooldown ordering, all shelf section and state-divergence paths, replacement
RNG budgets, all 22 default songs, custom playable components, padded finish timing, load without
play, automation restart and removal stop pairs.
