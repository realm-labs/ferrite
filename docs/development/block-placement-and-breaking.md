# Block Placement and Breaking

`G01-P5-S002` owns the three `SourceSpecified` slices under `BLK-002`:
`BLK-PLACEMENT-001`, `BLK-BREAKING-001`, and `BLK-BREAK-CONTENT-001`.

## Placement boundary

`ferrite-gameplay::block::placement` represents placement as an ordered, deliberately non-atomic
transaction. The initial write is the only write whose result controls success. Double-high
placement first clears the upper position with flags `27`; generic, bed-foot, second-half, and
component writes use the locked flags `11`, `26`, `3`, and `2`. A successful initial write still
emits sound and the game event and consumes the item when callbacks replace the candidate, while
component, block-entity, callback, and criterion work is skipped.

The module also closes target selection, the seven-block horizontal scaffolding boundary, exact
door-hinge scoring/click quadrants, and the special block-item dispatch families. Candidate-state
formulas stay with their catalog family owners from `BLK-001`; packet admission, correction, and
projection remain isolated in the protocol/server runtime and are joined in `G01-P5-B1`.

## Breaking boundary

`ferrite-gameplay::block::breaking` owns the Java-float mining formula, the signed-tick active and
delayed progress state machine, and the generic destroy transaction order. The tracker retains the
stored destroy position after active cancellation, lets delayed work starve active work, publishes
air start at stage `10`, leaves existing progress active across an instant start, uses stop
threshold `0.7`, and never clamps crack stages.

The generic commit plan makes the callback and gate order observable: item denial, game-master and
adventure gates; pre-removal hook and base effects; fluid restoration; conditional destroy hook;
creative early success; then tool damage/stat and the removal-plus-correct-tool loot branch. Loot
evaluation and `spawnAfterBreak` remain present when `block_drops` is false even though item/XP
spawning is gated.

## Concrete hook ownership

`ferrite-gameplay::block::break_hook` resolves the locked 110 special block IDs into 23 hook
categories and records whether each category runs at attack, pre-removal, replacement destroy,
post-removal, or after-break time. Experience providers include the 17 `DropExperienceBlock` IDs,
redstone ores, sculk family, and spawner, and consume the project-owned deterministic gameplay RNG
with the locked draw cardinality.

Downstream loot, item, entity, statistic, effect, and client rendering implementations remain with
their explicit later slice owners. This module supplies their exact block dispatch point and
parameters; `G01-P5-B1` closes the Region mutation/projection composition without moving ownership
out of the gameplay crate.

## Validation

The committed test owner is `crates/ferrite-gameplay/tests/slices/blocks/blk_002.rs`. It locks:

- partial placement writes, reread replacement, flags, dispatch, scaffolding, and hinge boundaries;
- Java numeric order, unbreakable and zero hardness, and correct/wrong-tool divisors;
- active/delayed/stale/abort/instant progress quirks and unbounded stages;
- failed-removal generic commit behavior and the `block_drops` ordering boundary;
- exactly 110 locally imported special block IDs, 23 categories, hook points, XP ranges, and RNG
  cardinality.
